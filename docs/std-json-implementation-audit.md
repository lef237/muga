# First std::json Implementation Audit

Status: completed implementation audit for the first `std::json` package slice,
the pure accessor follow-up, required object-field helpers, composite
object-field helpers, scalar array projection helpers, and direct scalar-array
object-field helpers. This audit checks the implemented code against
[std-json-first-slice.md](std-json-first-slice.md), the implemented accessor
follow-up, the runnable sample, artifact-backed behavior, and
release-readiness evidence.
It is not approval to broaden the standard library beyond the documented JSON
boundaries.

## Result

The implemented `std::json` surface matches the first-slice contract:

- `json::Value`, `json::Number`, `json::ErrorKind`, and `json::Error` are
  exposed from the virtual `std::json` package and are representable in `.mgi`.
- `parse`, `encode`, `number_as_int`, and `int` return explicit data or
  `Result[_, json::Error]`; user-data failures do not become panics or sentinel
  strings.
- `as_bool`, `as_string`, `as_number`, `as_int`, `as_array`, `as_object`,
  `array_strings`, `array_ints`, `array_bools`, `object_get`, scalar and
  composite `object_*` field helpers, direct scalar-array object-field helpers
  (`object_string_array*`, `object_int_array*`, and `object_bool_array*`), `_or`
  defaults, and `_required` helpers return data, `Option`, or
  `Result[_, json::Error]` without adding config-file loading, JSON paths, or
  schema decoding.
- Scalars, arrays, objects, duplicate keys, deterministic object key ordering,
  raw number validation, and recoverable diagnostic data are covered by focused
  source tests.
- `.mgi` / `.mgb` emission and artifact-backed execution are covered for the
  virtual package, so downstream code does not require private stdlib source
  bodies.
- Growth into schema generation, HTTP/RPC, `Float`, `Decimal`, `Bytes`,
  streaming APIs, or resource handles remains deferred.

## Evidence Map

| Contract area | Evidence |
|---|---|
| Result ergonomics | `standard_json_parse_objects_arrays_and_scalars`, `standard_json_parse_reports_data_error_kinds`, `standard_json_encode_rejects_invalid_raw_number`, and the runnable sample all exercise `Result[_, json::Error]` rather than runtime traps for user data. |
| Scalar/collection mapping | `standard_json_parse_objects_arrays_and_scalars`, `standard_json_encode_sorts_object_keys`, `standard_json_encode_escapes_strings`, and `package_std_json_sample_runs` cover null, bool, number, string, array, object, and deterministic object encoding. |
| Accessor helpers | `standard_json_value_accessors_run_as_virtual_package`, `standard_json_required_object_fields_report_missing_errors`, `standard_json_composite_object_fields_run_as_virtual_package`, `standard_json_path_helpers_run_as_virtual_package`, `standard_json_path_scalar_helpers_run_as_virtual_package`, `standard_json_path_collection_helpers_run_as_virtual_package`, `standard_json_scalar_array_projections_run_as_virtual_package`, `standard_json_scalar_array_field_helpers_run_as_virtual_package`, `standard_json_value_accessors_report_shape_errors`, `standard_json_accessor_artifact_run_uses_emitted_std_implementations`, and `package_std_json_sample_runs` cover typed value access, object-field defaults, required fields, composite fields, JSON path helpers, typed scalar path projection helpers, typed collection path projection helpers, scalar array projection, direct scalar-array object-field helpers, missing fields, wrong shapes, path-aware errors, and artifact-backed execution. |
| Schema evolution | `std_package.rs` exposes the stable first public data shapes and functions only; `standard_json_annotation_without_import_suggests_import` keeps source naming and import behavior explicit. |
| Diagnostics | `standard_json_parse_reports_data_error_kinds`, `standard_json_parse_exposes_error_offset`, `standard_json_number_as_int_validates_raw_numbers`, `standard_json_encode_rejects_invalid_raw_number`, `standard_json_parse_reports_nesting_limit`, and `standard_json_encode_reports_nesting_limit` cover machine-readable error kinds and offset behavior. |
| Artifact-backed execution | `standard_json_artifact_run_uses_emitted_std_implementations` and `standard_json_accessor_artifact_run_uses_emitted_std_implementations` emit `std__json.mgi` / `std__json.mgb` and run against the artifact root. |
| Deferred surface | `std-json-first-slice.md`, `standard-library-review-rules.md`, and release-readiness checks keep schema generation, HTTP/RPC, `Float`, `Decimal`, `Bytes`, streaming APIs, and resource handles outside this slice. |

## Findings

No implementation gap was found that requires broadening the public API. The
audit did add missing evidence for four edge behaviors that were implied by the
contract but not individually fixed by tests:

- `encode` escapes quote and backslash characters in `Value::String`.
- `parse` exposes a stable byte offset for duplicate-key diagnostics.
- `encode` rejects user-constructed invalid `Number::Raw` values with
  `InvalidNumber` and `offset = -1`.
- `encode` applies the same nesting-limit policy as `parse`.

The implementation limit is 128 nested arrays or objects for both parse and
encode. Errors created while validating constructed values use `offset = -1`
because they do not point into a source JSON string.

## Next Boundary

The nested JSON config workflow is implemented with `tags`, owner metadata,
servers, limits, and the composite/typed object-field helpers.
The scalar array projection and direct scalar-array object-field helper
boundaries are implemented. The JSON path helper boundary is also implemented
with typed field/index segments, optional and required traversal, and
path-aware missing/wrong-shape diagnostics. The typed JSON path scalar
projection helpers are implemented before a broader path helper matrix, schema
decoding, `std::config`, TOML, or full CLI parser schemas. The typed JSON path collection
projection helpers are also implemented as the next narrow JSON slice. The
JSON schema decoding design is carried by
[json-schema-decoding.md](json-schema-decoding.md) before implementing required
`json::decode`, broader `std::config`, TOML, generated config app templates, or
full CLI parser schemas.
The JSON schema decoding design in
[json-schema-decoding.md](json-schema-decoding.md) selects compiler-owned
`json::decode_or[T](value, fallback)` as the first decoder. The minimal
`std::config` JSON default loading design is carried by
[std-config-json-loading.md](std-config-json-loading.md) before broadening
`std::json`. The selected design and implementation in
[std-config-json-loading.md](std-config-json-loading.md) keeps
`std::config::load_json_or[T]` and `std::config::load_json[T]`
compiler-owned and reuses the decoder schema payload. Keep the same review
lens before changing `std::json` beyond that:
`Result` ergonomics,
scalar/collection mapping, schema evolution, diagnostics, `.mgi`
compatibility, required schema decoding, TOML, full CLI parser schemas, and the
deferred surfaces listed above.
The generated `muga new --template config-app` template is implemented, so
broader `std::json` decoding changes stay deferred until that onboarding path
is audited against the stable `std::config` API.
That generated-template follow-up is now carried by
[json-required-decoding.md](json-required-decoding.md), which selects required
`json::decode[T](value)` before broadening decoder target types or adding TOML.
[json-required-decoding.md](json-required-decoding.md) fixes and implements the
strict decoder contract around expected `Result[T, json::Error]` targets,
missing record fields, ignored unknown fields, no-fallback schema lowering, and
artifact-safe `DecodeJsonRequired` payloads.
[json-decoder-target-expansion.md](json-decoder-target-expansion.md) implements
the decoder target expansion for `Option[T]`, recursive `List[T]`, typed
`Map[String, T]`, and concrete non-generic enums across `json::decode_or[T]`,
`json::decode[T]`, `config::load_json_or[T]`, and `config::load_json[T]`, with
schema artifact payloads, null/missing/default semantics, path-aware collection
and enum errors, and generic decoding plus field/variant schema polish
deferred.
The implemented `config_app` sample and generated `config-app` starter carry
the structural config workflow with `Option[String]`, nested records,
`List[Record]`, and typed `Map[String, Int]` settings before TOML, full CLI
parser schemas, formatting templates, config discovery, or broader host
effects.
The decoder expansion implements enum JSON/config decoder support, using
zero-payload string tags and one-payload single-key objects before generic enum
decoding, field/variant schema polish, TOML, full CLI parser schemas,
formatting templates, config discovery, or broader host effects.
[json-config-schema-polish.md](json-config-schema-polish.md) implements
`@json(rename: "...")` on record fields and enum variants before aliases,
validation attributes, TOML, full CLI schemas, schema generation, generic
decoding, or broader host effects.
[json-config-strict-unknown-fields.md](json-config-strict-unknown-fields.md)
implements record-level `@json(deny_unknown_fields)`, accepted wire-key
semantics, path-aware unknown-key errors, `.mgi` record flags, and `RF` decoder
artifact tokens before aliases, validation attributes, TOML, full CLI schemas,
schema generation, generic decoding, or broader host effects.
[json-config-alias-metadata.md](json-config-alias-metadata.md) implements
repeated `@json(alias: "...")` arguments inside a single field/variant `@json(...)`
attribute, accepted-name conflict checks, strict unknown-field integration, and
`RG`/`EG` artifact tokens.
[json-config-validation-attributes.md](json-config-validation-attributes.md)
implements the post-alias trust slice: field-level `@validate(...)` metadata
with scalar string/int validators, path-aware validation errors, `.mgi` v8
metadata, and `RV` decoder artifact tokens.
[json-config-schema-export.md](json-config-schema-export.md) implements the
post-validation adoption slice: `muga schema --format json` for JSON Schema Draft 2020-12
output with Muga
`x-muga` extensions, required/overlay decode modes, concrete public record/enum
scope, validation keywords, alias metadata, loaded-interface package coverage,
and explicit deferrals.
[json-typed-encoding.md](json-typed-encoding.md) implements typed JSON encoding
with compiler-owned `json::to_value[T](value)` plus
`json::encode_typed[T](value)`, canonical primary wire-name output, omitted
optional record fields, enum output matching decode/schema export,
validation-on-encode, artifact schema behavior, and the post-schema-export
bidirectional contract slice.
[cli-parser-schema.md](cli-parser-schema.md) selects compiler-owned
`cli::parse_or[T](args, defaults)` and
`cli::usage_for[T](program, defaults)` as that first typed CLI schema boundary,
preserving CLI > config > defaults precedence while keeping TOML, strict
no-default parsing, subcommands, short flags, config discovery automation, full
client generation, and host effects deferred.
