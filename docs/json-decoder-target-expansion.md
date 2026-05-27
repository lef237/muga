Status: structural and concrete enum JSON decoder targets implemented

# JSON Decoder Target Expansion

This design and implementation expands the compiler-owned JSON schema decoder
target set after strict `json::decode[T](value)` proved the no-fallback path.
The implemented first expansion was structural: add `Option[T]`, recursive
`List[T]`, and typed `Map[String, T]` support across
`json::decode_or[T]`, `json::decode[T]`, and `config::load_json_or[T]`.
The follow-up enum implementation now adds concrete non-generic user enums over
supported payloads using the representation selected here.

## Current Boundary

Before this slice, the implemented decoders supported:

- `String`, `Int`, and `Bool`;
- scalar `List[String]`, `List[Int]`, and `List[Bool]`;
- `Map[String, json::Value]`;
- concrete non-generic records over supported fields;
- `json::decode_or[T](value, fallback)` default-overlay semantics;
- strict `json::decode[T](value)` required-field semantics;
- `config::load_json_or[T](path, fallback)` file read/parse/default-overlay
  semantics over the same schema payload.

That target set was sufficient for generated config apps and basic strict API
records, but it forced manual `json::Value` handling for nullable fields,
arrays of records, typed object maps, and enum mode/status values. The
implementation now supports the structural and concrete enum expansions below.

## Selected First Expansion

Implement these structural targets first:

```muga
Option[T]
List[T]
Map[String, T]
```

where `T` is any supported decoder target except nested `Option[Option[T]]`.
This means the first expansion supports examples such as:

```muga
record Owner {
  name: String
}

record Settings {
  owner: Option[Owner]
  tags: List[String]
  servers: List[Server]
  limits: Map[String, Int]
}
```

Concrete non-generic records remain the only supported record target shape.
Generic records, generic enums, opaque types, function types, non-string map
keys, `Option[Option[T]]`, and `Unit` stay rejected by the same `T006`
unsupported-target diagnostic family until explicitly added.

## Option Semantics

`Option[T]` maps JSON absence and JSON `null` differently depending on which
decoder is active:

| Context | Missing record field | Present `null` | Present non-null |
|---|---|---|---|
| `json::decode[T]` strict record field | `Option::None` | `Option::None` | `Option::Some(decoded T)` |
| `json::decode_or[T]` record field | fallback field value | `Option::None` | `Option::Some(decoded T)` |
| `config::load_json_or[T]` record field | fallback field value | `Option::None` | `Option::Some(decoded T)` |
| top-level `Option[T]` | not applicable | `Option::None` | `Option::Some(decoded T)` |

Rationale:

- strict decoding should still reject missing required fields, but an
  `Option[T]` field explicitly says absence is valid;
- default-overlay decoding should preserve its existing missing-field rule, so
  a missing optional field keeps the fallback value instead of becoming
  `Option::None`;
- explicit JSON `null` should override fallback values because it is present
  input data;
- `Option[Option[T]]` is rejected because `null` cannot distinguish
  `Option::None` from `Option::Some(Option::None)` in this decoder shape.

Wrong-shaped non-null values still return path-aware `json::Error` or
`config::ErrorKind::Decode`.

For default-overlay decoding of a present non-null `Option[T]`, a fallback
`Option::Some(inner)` decodes `T` with the inner fallback, while
`Option::None` decodes `T` with strict required-field semantics. Recursive
`List[T]` items and typed `Map[String, T]` values do not receive per-index or
per-key fallback values; present collection items decode strictly.

## List Semantics

Replace the scalar-only list target policy with recursive `List[T]` support:

- the JSON value must be an array;
- each item decodes using the item schema;
- item errors use the existing index path form such as `.servers[1].port`;
- scalar list artifact tokens (`LS`, `LI`, `LB`) remain valid for existing
  artifacts;
- the compiler may keep emitting scalar-list tokens for scalar lists and use a
  new recursive list schema token only when the item target is not one of the
  existing scalar cases.

This unlocks `List[Record]`, `List[Option[T]]`, `List[Map[String, T]]`, and
nested lists without adding iterator or collection protocol changes.

## Typed Map Semantics

Add typed `Map[String, T]` values while preserving the existing
`Map[String, json::Value]` raw-object target:

- the JSON value must be an object;
- each string key is preserved as the map key;
- each value decodes using the value schema;
- value errors use the same field path form as object fields, such as
  `.limits.max` or `.servers.primary.port`;
- non-string map keys remain unsupported because JSON object keys are strings;
- `Map[String, json::Value]` keeps the current raw object behavior and does not
  recursively decode values.

For default-overlay decoding, a missing record field of typed map type uses the
fallback field value. A present map replaces that field with the decoded map; it
does not merge individual keys with the fallback map because the schema has no
declared key set.

## Enum Semantics

The follow-up enum JSON/config decoder implementation uses the representation
selected during the structural design:

- zero-payload variants decode from a string tag equal to the variant name;
- one-payload variants decode from a single-key object whose key is the variant
  name and whose value decodes using the payload schema;
- unknown variant tags return path-aware `json::Error`;
- payload shape errors append the variant key to the path;
- default-overlay decoding reuses the same-variant fallback payload when the
  enum payload schema supports nested fallback behavior;
- enum decoding must remain expected-type driven, so a tag is interpreted only
  inside the target enum type.

This direction keeps simple enum fields compact while leaving payload variants
unambiguous. Generic enum decoding and aliases remain deferred; the follow-up
schema-polish slice implements explicit field/variant wire names through
`@json(rename: "...")` with package-interface and artifact metadata.

## Lowering And Artifacts

Keep existing artifact meaning stable:

- existing `JsonDecodeSchema` tokens (`S`, `I`, `B`, `LS`, `LI`, `LB`, `M`,
  `R`, `O`, `L`, and `MT`) must continue to parse and validate;
- add the enum schema variant alongside recursive `Option`, `List`, and typed
  string maps;
- the enum token is
  `E <type_symbol> <variant_count> <variant_symbol> <payload_flag> [payload_schema]...`;
- `JsonDecodeSchema::map_symbols` and `validate_symbols` must recurse through
  enum variant payloads as well as structural nested schemas;
- typed HIR, MIR, bytecode, `DecodeJson`, `DecodeJsonRequired`, and
  `LoadJsonConfig` keep their instruction split and only receive richer schema
  payloads;
- `.mgb` structural validation must reject malformed nested schema payloads
  before runtime execution.

## Implementation Result

The structural expansion and concrete enum follow-up are implemented through the
existing compiler-owned schema pipeline:

- `JsonDecodeSchema` now carries recursive `Option`, `List`, and
  typed string-map variants plus concrete enum schemas while preserving old
  scalar list and raw object-map artifact tokens;
- artifact text accepts and emits `O <schema>`, `L <schema>`, and
  `MT <schema>` nested schema payloads plus `E ...` enum payloads;
- typing accepts `Option[T]`, recursive `List[T]`, and typed `Map[String, T]`
  for supported `T` and concrete non-generic user enums over supported
  payloads, while still rejecting `Option[Option[T]]`, generic records,
  generic enums, non-string map keys, opaque types, functions, and `Unit`;
- runtime default-overlay and strict decoders share the richer schema payloads
  across `DecodeJson`, `DecodeJsonRequired`, and `LoadJsonConfig`;
- source, artifact-backed execution, `run --built`, path-aware list/map errors,
  enum tag/payload errors, missing optional fields, explicit `null`,
  default-overlay optional and enum-payload fallback, and unsupported target
  diagnostics are covered in `tests/examples.rs`;
- release readiness tracks the implementation queue and docs/spec alignment.

## Diagnostics

Unsupported target diagnostics should keep naming the active helper:

- `` `json::decode` supports only ... `` for strict JSON decoding;
- `` `json::decode_or` supports only ... `` for default-overlay JSON decoding;
- `` `config::load_json_or` supports only ... `` for config loading.

The target summary should be updated from scalar lists and raw object maps to:

`String`, `Int`, `Bool`, `Option[T]`, `List[T]`, `Map[String, T]`,
`Map[String, json::Value]`, concrete non-generic records over supported fields,
and concrete non-generic enums over supported payloads.

When rejecting `Option[Option[T]]`, non-string map keys, generic records,
generic enums, function types, opaque handles, or `Unit`, diagnostics should
include the rendered rejected type.

## Required Coverage

- source execution for optional record fields, top-level options, recursive
  lists, lists of records, typed maps, and nested combinations;
- strict missing optional fields returning `Option::None`;
- default-overlay missing optional fields preserving fallback values;
- explicit JSON `null` overriding fallback optional fields to `Option::None`;
- typed map, list item, and enum payload shape errors with path-aware messages;
- unknown enum tags with path-aware messages;
- unsupported nested option, generic enum, generic record, and non-string map
  key targets;
- artifact-backed execution and `run --built` coverage for nested option/list
  /typed-map and enum schema payloads;
- release-readiness evidence for schema persistence, typing, runtime, examples,
  docs, and implementation queue alignment.

## Non-Goals

This target family still must not add generic record decoding, generic enum
decoding, opaque type decoding, non-string map keys, `Unit` decoding, strict
unknown-field rejection, field rename attributes, validation attributes,
source-level call type arguments, generated schemas, TOML/YAML/JSON5, config
discovery, full CLI parser schemas, formatting templates, `Bytes`, process
APIs, network APIs, or streams.

## Implementation Plan

1. Done: audit strict decoder adoption and select broader decoder target
   design as the next slice.
2. Done: define null, missing-field, fallback, typed map, recursive list, enum
   direction, artifact, diagnostic, coverage, and non-goal policies.
3. Done: add recursive `Option`, `List`, and typed-map variants to
   `JsonDecodeSchema` while preserving existing artifact tokens.
4. Done: extend typing schema generation for `Option[T]`, recursive `List[T]`, and
   typed `Map[String, T]`, including unsupported nested options and non-string
   map keys.
5. Done: extend runtime default-overlay and strict decoders for option, recursive
   list, and typed map semantics.
6. Done: add source, artifact-backed, missing/null/default-overlay, unsupported
   target, and `run --built` coverage.
7. Done: audit structural config workflow adoption and select concrete enum
   JSON/config decoding as the next data-boundary slice.
8. Done: add concrete non-generic enum schemas, runtime decoding, diagnostics,
   source/config/artifact/`run --built` coverage, and docs.
9. Done: design and implement JSON/config schema polish for field/variant wire names before
    choosing TOML, full CLI parser schemas, formatting templates, config
    discovery, `Bytes`, process APIs, network APIs, or streams.
