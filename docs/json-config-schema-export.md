Status: JSON/config schema export implemented

# JSON/Config Schema Export

Muga can now decode, validate, and export practical concrete JSON/config
models. The implemented schema export boundary lets editors, CI jobs, docs, and
non-Muga consumers inspect the same contract that source execution, `.mgi`
interfaces, and `.mgb` decoder payloads use.

This implementation intentionally covers schema export only. It does not add
TOML, client generation, JSON encoding from typed records, generic user-type
schema instantiation, broader validators, or host-effect APIs.

## Goals

Short-Term Goal: export a concrete public record or enum as a machine-readable
JSON Schema document that reflects Muga's primary wire names, strictness, enum
shape, and first validation attributes.

Medium-Term Goal: make generated docs, config editors, CI validation, future
TOML loading, future CLI schemas, and future client generators consume one
contract instead of reinterpreting Muga source.

Long-Term Goal: make Muga public data models useful outside Muga programs, so
packages can publish trustworthy configuration and API contracts.

## Format Decision

Use JSON Schema Draft 2020-12 as the standard layer, with Muga-specific
metadata under `x-muga` extension objects.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "muga:app::settings",
  "$ref": "#/$defs/app::settings::Settings",
  "$defs": {
    "app::settings::Settings": {
      "type": "object",
      "properties": {
        "server_host": { "type": "string", "minLength": 1 }
      },
      "required": ["server_host"],
      "additionalProperties": false,
      "x-muga": {
        "qualifiedName": "app::settings::Settings",
        "kind": "record"
      }
    }
  }
}
```

JSON Schema is the practical interoperability target. A Muga-native schema JSON
would be easier to make exact, but external validators, editors, and API tools
would still need a translation step. The `x-muga` layer preserves facts that
plain JSON Schema cannot safely express, such as aliases as input-compatibility
metadata and the exact Muga source type.

## CLI/API Decision

The implementation adds a focused schema command rather than overloading editor
metadata:

`muga schema --format json [--package <package>] [--type <type>] [--decode-mode required|overlay] <source-file>`

Rules:

- `--format json` is required; the command emits JSON Schema, not a human
  summary;
- `--type` selects one public record or enum and makes it the root `$ref`;
- without `--type`, the command emits a package schema document whose `$defs`
  contains all exportable public concrete records/enums in the selected package;
- `--package` selects a package from the checked package graph; when omitted,
  use the entry package;
- `--decode-mode required` is the default and matches strict
  `json::decode[T]`;
- `--decode-mode overlay` matches `json::decode_or[T]` and
  `config::load_json_or[T]`, where missing fields may come from defaults;
- schema export is read-only and should not write `.mgi`, `.mgc`, `.mgb`, or
  build-cache files;
- a library API should use the same internal renderer so CLI and tests do not
  diverge.

Do not add client generation in this slice. A future client generator should
consume these schema documents after the schema contract is stable.

## Source And Interface Scope

The first implementation exports only public concrete record and enum
contracts:

- concrete non-generic public records;
- concrete non-generic public enums;
- fields or payloads composed from `String`, `Int`, `Bool`, `Option[T]`,
  `List[T]`, `Map[String, T]`, supported concrete public records/enums, and
  `std::json::Value`;
- dependency package contracts loaded from in-memory or persisted interfaces
  when the existing package check can resolve them.

Reject with explicit diagnostics:

- generic user records/enums, even when a use site appears instantiated;
- functions, opaque types, runtime-backed handles, and non-data public items;
- maps with non-string keys;
- payloads containing unsupported types;
- private records/enums as root export targets.

The implementation adds `T029` for unsupported schema export targets or field
types, with a source span when available and a suggestion to export a concrete
public record/enum composed of supported data types.

## Type Mapping

| Muga type | JSON Schema mapping | Muga extension |
|---|---|---|
| `String` | `{ "type": "string" }` | `{ "x-muga": { "type": "String" } }` when needed for exactness |
| `Int` | `{ "type": "integer" }`; explicit `@validate(min/max)` becomes `minimum` / `maximum` | `{ "x-muga": { "type": "Int", "intBits": 64 } }`; do not emit implicit i64 bounds as standard keywords in the first slice because common JavaScript validators may handle those large boundary values imprecisely |
| `Bool` | `{ "type": "boolean" }` | optional `x-muga.type` |
| `Option[T]` | `anyOf: [<T schema>, { "type": "null" }]` | `{ "x-muga": { "optional": true } }` on the field schema |
| `List[T]` | `{ "type": "array", "items": <T schema> }` | optional item source metadata |
| `Map[String, T]` | `{ "type": "object", "additionalProperties": <T schema> }` | `{ "x-muga": { "mapKey": "String" } }` |
| `std::json::Value` | `true` | `{ "x-muga": { "type": "std::json::Value" } }` |
| concrete record | `$ref` to the record definition | definition-level `x-muga.kind = "record"` |
| concrete enum | string enum or `oneOf` over tag/object alternatives | definition-level `x-muga.kind = "enum"` |

## Record Mapping

Record definitions emit:

- `type: "object"`;
- `properties` keyed by the primary JSON/config wire name:
  `@json(rename: "...")` when present, otherwise the Muga field name;
- `required` fields according to the selected decode mode;
- `additionalProperties: false` only for `@json(deny_unknown_fields)`;
- field-level `x-muga` metadata for the Muga field name, primary wire name, and
  aliases.

Required fields:

- in `required` mode, every non-`Option[T]` field is required;
- in `required` mode, `Option[T]` fields may be omitted because Muga strict
  decoding treats a missing optional field as `Option::None`;
- in `overlay` mode, fields are not required because missing values may come
  from the fallback/default record.

Aliases:

- aliases are input compatibility metadata, not canonical output names;
- emit aliases under a Muga extension such as
  `"x-muga": { "field": "host", "aliases": ["legacy_host"] }`;
- do not generate `anyOf` property alternatives for aliases in the first slice,
  because JSON Schema cannot represent Muga's "primary or alias, but not both"
  ambiguity policy cleanly without a large and fragile schema.

Validation:

- `@validate(non_empty)` maps to `minLength: 1`;
- `@validate(min_len: n)` maps to `minLength: n`;
- `@validate(max_len: n)` maps to `maxLength: n`;
- `@validate(min: n)` maps to `minimum: n`;
- `@validate(max: n)` maps to `maximum: n`;
- if `non_empty` and `min_len` both exist, emit the stricter `minLength` while
  preserving the exact original rule list under `x-muga.validation`.

## Enum Mapping

Use the same external shapes as JSON/config decoding:

- zero-payload variants decode from string tags;
- one-payload variants decode from single-key objects;
- primary variant tags use `@json(rename: "...")` when present, otherwise the
  Muga variant name;
- aliases are emitted only under `x-muga` metadata in the first slice.

If all variants are zero-payload, emit a compact string schema:

```json
{
  "type": "string",
  "enum": ["auto", "manual"],
  "x-muga": {
    "kind": "enum",
    "variants": [
      { "name": "Auto", "wireName": "auto", "aliases": ["automatic"] },
      { "name": "Manual", "wireName": "manual", "aliases": [] }
    ]
  }
}
```

If any variant has a payload, emit `oneOf` alternatives:

- zero-payload alternative: `{ "const": "<wire-name>" }`;
- payload alternative:
  `{ "type": "object", "properties": { "<wire-name>": <payload schema> },
     "required": ["<wire-name>"], "additionalProperties": false }`.

## Definition Identity

Use qualified Muga names as `$defs` keys:

- `app::settings::Settings`;
- `util::api::Mode`.

References use JSON Pointer fragments such as
`#/$defs/app::settings::Settings`. This keeps schema documents readable and
stable across source file layout changes. If a future package registry needs
globally unique schema URIs, it can layer registry package identity into `$id`
without changing the `$defs` contract.

Recursive record shapes should be emitted with `$ref`; JSON Schema validators
that support Draft 2020-12 can handle recursive definitions.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| JSON Schema Draft 2020-12 plus `x-muga` extensions | Directly useful with common editors/validators while preserving Muga-only facts. Draft 2020-12 is the current JSON Schema dialect and aligns with OpenAPI 3.1-era tooling. | Some Muga facts need extensions, and not every external validator will understand them. | Select |
| Muga-native schema JSON only | Exact representation of Muga semantics and easier implementation. | Low external adoption value; every consumer would need a Muga-specific adapter. | Reject |
| OpenAPI schema component output first | Valuable for service APIs. | Requires endpoint/operation modeling, request/response layout, errors, and transport choices that Muga has not designed yet. | Defer |
| Embed schemas into `metadata --format json` | Reuses an existing command. | Metadata is editor/workspace-oriented; schema export needs a clean validator-friendly document and package/type selection. | Reject |
| Full client generation | High adoption value after schemas are stable. | Adds target-language packaging, naming, versioning, and runtime choices. | Defer |

## Tests

The implementation is covered by these anchors:

- `muga schema --format json --type Settings` emits a Draft 2020-12 document
  with `$schema`, `$ref`, and `$defs`;
- record properties use `@json(rename: "...")` primary names and preserve alias
  metadata under `x-muga`;
- `@json(deny_unknown_fields)` emits `additionalProperties: false`;
- `@validate(non_empty, min_len, max_len, min, max)` emits standard validation
  keywords and preserves exact rules under `x-muga.validation`;
- `Option[T]` maps to nullable `anyOf` and is omitted from `required` in
  `required` mode;
- `--decode-mode overlay` emits no required fields for overlay records;
- zero-payload enums emit compact string `enum` schemas;
- mixed/payload enums emit `oneOf` over string/object alternatives;
- schema export works for a dependency package through loaded interfaces when
  dependency source is not present and interfaces are available;
- unsupported generic or opaque targets fail with a focused diagnostic.

## Deferred Work

- client generation and OpenAPI endpoint generation;
- TOML/YAML/JSON5 parsing;
- full CLI parser schemas/help generation;
- JSON encoding from typed records/enums;
- generic user record/enum schema instantiation;
- regex/pattern validators and regex dialect policy;
- list/map length validators and enum payload validators;
- cross-field validation and whole-record predicates;
- schema version negotiation beyond a stable v1 output shape;
- remote package registry schema publishing;
- host-effect APIs such as `Bytes`, process, network, or streams.

## Implementation Plan

1. Done: implement typed JSON/config decoding, wire-name metadata, strict
   unknown-field policy, alias metadata, and validation attributes.
2. Done: audit post-validation adoption and select schema export as the next
   JSON/config adoption slice.
3. Done: select JSON Schema Draft 2020-12 plus `x-muga` extensions, a focused
   `muga schema --format json` command, required/overlay decode modes, concrete
   public record/enum scope, and explicit type/attribute mappings.
4. Done: implement the smallest schema export slice across schema rendering,
   CLI parsing, diagnostics, source/interface package coverage, docs, and
   release readiness.
5. Done: audit the post-schema-export JSON/config adoption gap in
   [json-typed-encoding.md](json-typed-encoding.md).
6. Done: design typed JSON encoding in
   [json-typed-encoding.md](json-typed-encoding.md), selecting
   `json::to_value[T](value)` as the reusable first API.
7. Done: implement the smallest typed JSON encoding slice in
   [json-typed-encoding.md](json-typed-encoding.md).
8. Done: audit typed JSON encoding adoption in
   [cli-parser-schema.md](cli-parser-schema.md)
   and select full CLI parser schema design next.
9. Done: design full CLI parser schemas in
   [cli-parser-schema.md](cli-parser-schema.md).
10. Next: implement the smallest CLI parser schema overlay before TOML, full
    client generation, generic encoding/decoding, broader validators, config
    discovery automation, or host effects.
