Status: default-overlay JSON schema decoder implemented

# JSON Schema Decoding Design

Muga now has enough explicit `std::json` helpers for manual data access:
object-field helpers, typed scalar-array helpers, typed path traversal, and
typed path collection projection. The remaining practical config friction is
constructing application records by hand after those lookups. A schema decoder
can remove that boilerplate, but only if it respects the existing package and
artifact model.

The first implementation is a compiler-owned, type-directed
`json::decode_or[T](value, fallback)` helper. The fallback value supplies both
the target type and field defaults, which matches `samples/projects/config_app`
and avoids relying on expected-type inference for the first schema slice.

## Selected API

The first implementation adds this public `std::json` helper:

```muga
pub fn decode_or[T](value: json::Value, fallback: T): Result[T, json::Error]
```

Candidate follow-up, not part of the first implementation:

```muga
pub fn decode[T](value: json::Value): Result[T, json::Error]
```

`decode_or` is compiler-recognized rather than an ordinary Muga function at
direct call sites. The type checker validates that `T` is concrete and
supported, then lowering carries a compact schema into bytecode so runtime
decoding does not require source files or runtime reflection.

## Supported First Target Types

The first implementation supports concrete, non-generic targets composed
from:

- `String`
- `Int`
- `Bool`
- `List[String]`
- `List[Int]`
- `List[Bool]`
- `Map[String, json::Value]`
- records whose fields recursively use only the supported first target types

Records may be local or imported from loaded `.mgi` interfaces if their public
shape is available to the package-aware checker. In other words, the
implementation boundary is: public shape is available to the package-aware
checker before lowering creates the decoder schema. Generic records, enums,
`Option[T]`, `Result[T, E]`, functions, opaque types, `Map[String, T]` for
non-`json::Value` values, `Float`, `Decimal`, and arbitrary user-defined scalar
conversions remain unsupported in the first slice.
The required implementation boundary is: public shape is available to the package-aware checker.

A later structural expansion in
[json-decoder-target-expansion.md](json-decoder-target-expansion.md) now adds
`Option[T]`, recursive `List[T]`, and typed `Map[String, T]` support while
leaving the first-slice rationale here intact.

This target set is deliberately enough for the current `Settings` shape:

```muga
record Settings {
  name: String
  port: Int
  verbose: Bool
  tags: List[String]
  metadata: Map[String, json::Value]
}
```

## Decoding Semantics

For a record target, the JSON value must be an object. Each record field is
decoded from the object field with the same name:

- if the JSON object has the field, decode that field recursively;
- if the JSON object is missing the field, use the corresponding field from
  `fallback`;
- unknown JSON object fields are ignored in the first slice;
- present fields with the wrong shape return `json::Error`;
- `null` is a present value and is a wrong shape for all first-slice target
  types;
- list fields replace the fallback list when present; individual items are
  decoded with item-index diagnostics;
- `Map[String, json::Value]` fields accept a JSON object and keep raw
  `json::Value` entries without recursively decoding the values.

For top-level scalar, list, or raw JSON-object-map targets, `fallback` fixes the
type but is not used unless the selected implementation later adds a missing
top-level concept. The first practical use is record overlay.

## Diagnostics

Decoder diagnostics should reuse the existing rendered JSON path policy:

- wrong record base: `expected JSON Object at path <root>`;
- wrong scalar field: `expected JSON String at path .name`;
- wrong integer field, including non-integral numbers:
  `expected JSON Int at path .port`;
- wrong list item: `expected JSON String at path .tags[1]`;
- wrong nested record field: `expected JSON Bool at path .server.tls`;
- unsupported target types are compile-time diagnostics, not runtime
  `json::Error` values.

Decoder-created `json::Error` values use `ErrorKind::UnexpectedToken` and
`offset = -1`, matching the existing constructed-value and helper errors that
do not point into a source JSON byte offset.

## Schema Source

The schema source is Muga type information, not runtime reflection:

- local records come from the current package-aware type environment;
- imported records come from loaded `.mgi` public interfaces;
- record field names and field types are the schema;
- public interface hashes already change when public record fields change, so
  downstream artifact reuse has a compatibility signal;
- bytecode should carry the schema needed for runtime decoding so artifact-
  backed execution does not load dependency source.

The first implementation rejects generic target types, unresolved type
parameters, private inaccessible package record shapes, recursive record shapes
that would decode infinitely, and unsupported field types with clear compiler
diagnostics.

## Implementation Shape

The implementation avoids pretending that an ordinary generic stdlib function
can inspect `T` at runtime. The implemented shape is:

1. expose `json::decode_or[T]` from the virtual `std::json` package;
2. make type checking recognize direct calls as a schema-decoding intrinsic;
3. validate and lower the concrete target `T` into a serializable decoder
   schema; this serializable decoder schema is the artifact-safe runtime input;
4. add a MIR/bytecode `DecodeJson` form that carries that schema;
5. decode runtime `json::Value` records/lists/scalars into ordinary runtime
   `Value` instances with existing record type names and field ordering;
6. preserve artifact-backed execution by storing every needed schema fact in
   emitted artifacts.

If the lowering proves too invasive, the fallback implementation plan is a
compiler-generated helper body at build time. A pure runtime builtin without a
schema payload is rejected because it cannot decode empty lists or loaded
record fields safely.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| `json::decode_or[T](value, fallback)` compiler intrinsic | Directly shrinks config overlays; fallback fixes the target type and supplies field defaults; first target set matches `config_app`; unknown fields can be ignored without losing user data in raw `metadata` maps | Requires type-directed lowering and schema payloads, but avoids expected-type-only inference for the first slice | Select first implementation |
| `json::decode[T](value)` required decoder | Useful for messages and strict config files; no fallback value required | Needs expected-type inference or explicit annotation at every use, missing-field policy, and less direct `config_app` value | Defer |
| Runtime reflective builtin | Simple API surface | Muga has no runtime type reflection; empty lists, imported record fields, and artifact-backed execution would be unsound or source-dependent | Reject |
| Generated source helper functions | Could avoid new bytecode if generated Muga code calls existing helpers | Requires code generation naming, source mapping, artifact hashing, and generated diagnostics before the core schema contract is proven | Defer |
| Decoder builder API | Avoids compiler intrinsics and lets users write explicit schemas | Muga lacks ergonomic heterogeneous builders; would likely be noisier than current record construction | Reject for first slice |
| `std::config` with JSON/TOML discovery | Useful for conventional apps | Depends on schema/default policy and adds file discovery, precedence, and format choices beyond `std::json` | Defer |

## Non-Goals

The first schema decoding implementation must not add field attributes, renamed
fields, custom validators, strict unknown-field rejection, enum decoding,
`Option[T]` null/missing policy, generic record schemas, arbitrary map decoding,
TOML, YAML, JSON5, JSONPath strings, macros, reflection, schema generation,
OpenAPI/client generation, `std::config`, generated app templates, formatting
templates, interpolation, `Bytes`, process APIs, network APIs, streams, or
service runtime behavior.

The structural target expansion is now implemented separately; enum decoding,
generic records, field attributes, TOML, schema generation, config discovery,
and host effects remain non-goals here.

## Implementation Plan

1. Done: add release-readiness coverage for this design and mark the
   implementation queue so the next row is `json::decode_or` implementation.
2. Done: add `json::decode_or[T]` to the virtual `std::json` package signature
   as a compiler-recognized helper.
3. Done: add typechecker validation for concrete supported target schemas and
   rejecting tests for unsupported target types.
4. Done: lower `decode_or` calls with schema payloads that survive emitted
   artifacts.
5. Done: implement runtime decoding for the supported first target set.
6. Done: add source, artifact-backed, and config workflow coverage, including a
   `config_app` refresh that replaces `settings_from_config` with
   `json::decode_or(config, default_settings())`.
7. Next: audit adoption before adding required `decode[T]`, `std::config`,
   TOML, generated config app templates, full CLI parser schemas, or broader
   platform APIs.
