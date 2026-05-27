Status: typed JSON encoding implemented

# Typed JSON Encoding

Muga's JSON/config contract is now useful in four directions: typed decoding,
runtime validation, JSON Schema export, and typed JSON output. The first typed
encoding implementation converts concrete records/enums and supported structural
values to canonical JSON through the same field, enum, validation, and
package-interface metadata.

This document records the compiler-owned typed JSON encoding boundary, selected
and implemented. It is pure data conversion only. It does not add file writes,
HTTP, client generation, TOML, generic type instantiation, broader validators,
or host-effect APIs.

## Goals

Short-Term Goal: encode concrete typed Muga values into `std::json::Value`
using the same public contract as `json::decode[T]` and
`muga schema --format json`.

Medium-Term Goal: let applications produce config files, API payloads, examples,
golden fixtures, and schema round-trip tests without manually constructing
`json::Value` trees.

Long-Term Goal: make Muga public data models bidirectional contracts: packages
can accept, validate, publish, and emit JSON through stable package interfaces
without runtime reflection or framework-specific generation.

## Selected API

Add a compiler-recognized conversion helper as the first implementation target:

```muga
pub fn to_value[T](value: T): Result[json::Value, json::Error]
```

Add a direct string convenience in the same design, but allow the implementation
to stage it after `to_value` if needed:

```muga
pub fn encode_typed[T](value: T): Result[String, json::Error]
```

`json::encode(value: json::Value)` remains unchanged. Muga has no overload-based
dispatch, so `encode_typed` avoids changing the existing `encode` name while
still giving users a one-call path to a JSON string. `encode_typed(value)` is
semantically equivalent to `json::encode(try json::to_value(value))`, including
existing raw-number validation inside `json::encode`.

Both helpers are compiler-recognized at direct call sites. The type checker
derives the target schema from the argument type, validates that it is concrete
and supported, and lowering carries a serializable schema payload into bytecode
so artifact-backed execution does not need source files or runtime reflection.

## Supported First Targets

The first encoding implementation supports the same concrete family as
the current decoders and schema exporter:

- `String`;
- `Int`;
- `Bool`;
- `Option[T]`, except nested `Option[Option[T]]`;
- recursive `List[T]`;
- `Map[String, T]`;
- `std::json::Value`;
- concrete non-generic records over supported fields;
- concrete non-generic enums over supported payloads.

Reject unresolved type parameters, generic records/enums, functions, opaque
handles, non-string map keys, `Unit`, `Result[T, E]`, and any unsupported nested
shape at compile time. The implementation reuses the existing unsupported JSON
target diagnostic style and names the active helper.

## Mapping Semantics

| Muga value | JSON value mapping |
|---|---|
| `String` | `json::Value::String(value)` |
| `Int` | `json::int(value)` so integer rendering remains validated by existing JSON number policy |
| `Bool` | `json::Value::Bool(value)` |
| top-level `Option::None` | `json::Value::Null` |
| record field `Option::None` | omit the property from the output object |
| `Option::Some(value)` | encode `value` recursively |
| `List[T]` | JSON array with recursively encoded items |
| `Map[String, T]` | JSON object with existing string keys and recursively encoded values |
| `std::json::Value` | pass through unchanged |
| concrete record | JSON object using primary field wire names |
| zero-payload enum variant | JSON string tag |
| one-payload enum variant | single-key JSON object with recursively encoded payload |

Record field ordering should follow public record field order. Existing
`json::encode` already sorts `json::Value::Object` map keys for deterministic
string output; `to_value` should not depend on runtime map ordering for
correctness.

## Attribute Semantics

- `@json(rename: "...")` determines the canonical output property or enum tag.
- `@json(alias: "...")` is input compatibility metadata only and is never emitted.
- `@json(deny_unknown_fields)` affects decoding and schema validation, not output
  construction.
- `@validate(...)` is checked during typed encoding. Encoding a type-correct but
  invalid value should return `json::ErrorKind::Validation` with `offset = -1`
  and the same path rendering policy as decoding.

Validation-on-encode is selected because schema export would otherwise publish a
contract that Muga itself can emit invalidly. The cost is that encoding can fail,
which is why both helpers return `Result`.

## Option Policy

Record fields with `Option::None` are omitted rather than emitted as `null`.
This is the canonical output policy for optional fields because schema export
already marks optional fields as not required, and strict decoding treats missing
optional fields as `Option::None`.

Top-level options, list items, map values, and enum payloads cannot be omitted, so
`Option::None` encodes as JSON `null` in those positions. This loses the
input-only distinction between present `null` and missing fields, but Muga's
typed value also does not distinguish those states after decoding.

## Enum Policy

Use the same external representation as decoding and schema export:

- zero-payload variants encode as string tags;
- one-payload variants encode as single-key objects;
- primary tags use `@json(rename: "...")` when present, otherwise the Muga variant
  name;
- aliases are not emitted;
- payload validation errors append the variant wire key to the rendered path.

Generic enum encoding remains deferred because instantiated schema identity must
be stable across typing, `.mgi`, `.mgb`, schema export, and future client
generation.

## Schema And Artifacts

The implementation reuses the existing compiler-owned schema pipeline as
much as possible:

- type checking can derive an encoding schema from the same concrete type facts
  used by `json::decode[T]`;
- local and imported public record/enum shapes come from package-aware type
  information and loaded `.mgi` interfaces;
- `.mgb` implementation artifacts should carry the full encoding schema payload
  needed by runtime conversion;
- malformed or stale schema payloads are hard artifact errors before execution;
- a future mechanical rename from `JsonDecodeSchema` to a neutral
  `JsonContractSchema` is acceptable only if it lowers maintenance cost after the
  first encoder works.

The first implementation may carry `JsonDecodeSchema` through a new encode
instruction to avoid creating a parallel metadata format. The important contract
is semantic parity, not the Rust type name.

## Diagnostics

Unsupported target diagnostics name the active helper:

- `` `json::to_value` supports only ... ``;
- `` `json::encode_typed` supports only ... ``.

The supported-target summary names `String`, `Int`, `Bool`, `Option[T]`,
`List[T]`, `Map[String, T]`, `std::json::Value`, concrete non-generic records,
and concrete non-generic enums.

Validation failures are recoverable runtime `json::Error` values:

- string validators use the field path, such as `.host`;
- integer validators use the field path, such as `.retries`;
- list and map item validators use `.items[0]` and `.limits.max`;
- enum payload validators include the variant key, such as `.Manual.value`;
- offsets are `-1` because the error comes from a constructed Muga value, not
  parsed JSON text.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| `json::to_value[T](value)` compiler intrinsic | Reusable boundary for config generation, tests, docs, future schema examples, and direct `json::encode` composition. It avoids overloading and keeps conversion separate from string rendering. | Requires type-directed lowering, schema payloads, runtime conversion, and validation-on-encode behavior. | Select first |
| `json::encode_typed[T](value)` direct string helper | Best ergonomics for applications that just need a JSON string. | Duplicates `to_value` plus existing `encode`; should be implemented as a convenience over the same schema semantics, not as a separate policy. | Select as same-design convenience |
| Overload existing `json::encode` for typed values | Most discoverable name. | Muga has no overload dispatch; changing `encode` would create ambiguity with `json::Value` and weaken current explicitness. | Reject |
| Runtime reflective builtin | Simple-looking implementation surface. | Muga has no runtime type reflection; loaded-interface and artifact-backed execution would become source-dependent or unsound. | Reject |
| Generated source helper functions | Avoids new bytecode instructions. | Requires generated names, source mapping, package hashing, diagnostics, and artifact policy before the schema contract is proven. | Defer |
| Encode by first exporting JSON Schema and using an external generator | Avoids compiler/runtime work. | Pushes a core data conversion feature outside Muga and does not help ordinary Muga programs. | Reject |

## Non-Goals

This design does not add:

- TOML/YAML/JSON5 output;
- file writing or config discovery;
- OpenAPI, RPC, or client generation;
- generic record/enum instantiation;
- custom serializer functions;
- regex/list/map length/cross-field validators beyond existing `@validate(...)`;
- `Float`, `Decimal`, `Bytes`, binary encodings, process APIs, network APIs, or
  streams;
- source-level call type arguments;
- reflection, macros, or runtime `Any`.

## Implemented Coverage

- source execution for scalar, option, list, map, record, nested record, and enum
  encoding through `standard_json_encode_typed_record_runs`;
- record `@json(rename: "...")` primary output names and no alias output;
- omitted `Option::None` record fields and `null` top-level/list/map optional
  values;
- zero-payload enum string tags and one-payload enum object output;
- validation-on-encode failures with path-aware `json::ErrorKind::Validation`
  through `standard_json_encode_typed_reports_validation_errors`;
- unsupported generic record/enum, opaque, function, `Unit`, nested option, and
  non-string map key targets, with `standard_json_encode_typed_rejects_unsupported_targets`
  covering the generic-record diagnostic;
- artifact-backed execution and `run --built` coverage for schema payloads;
- loaded-interface package coverage without provider source through
  `standard_json_encode_typed_interface_artifact_run_uses_schema_payload`;
- direct `json::encode(try json::to_value(value))` output and implemented
  `json::encode_typed(value)`;
- release-readiness evidence for std package signatures, typing, lowering,
  runtime, artifacts, docs, and implementation queue alignment.

## Implementation Plan

1. Done: audit post-schema-export adoption and select typed JSON encoding design
   in this document.
2. Done: select compiler-owned `json::to_value[T](value)` as the reusable first
   API and `json::encode_typed[T](value)` as the same-design string convenience.
3. Done: define supported targets, option omission/null policy, enum mapping,
   validation-on-encode, schema/artifact behavior, diagnostics, coverage, and
   non-goals.
4. Done: implement the smallest typed JSON encoding slice across std package
   signatures, type checking, MIR/bytecode/artifacts, runtime conversion, docs,
   and release readiness.
5. Done: audit the typed JSON encoding result in
   [cli-parser-schema.md](cli-parser-schema.md) and select full CLI parser
   schema design next.
6. Done: design full CLI parser schemas in
   [cli-parser-schema.md](cli-parser-schema.md), selecting
   `cli::parse_or[T](args, defaults)` and
   `cli::usage_for[T](program, defaults)`.
7. Next: implement the smallest CLI parser schema overlay before TOML, full
   client generation, generic encoding/decoding, broader validators, config
   discovery automation, or host-effect APIs.
