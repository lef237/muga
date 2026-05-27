Status: required JSON decoder implemented

# Required JSON Decoding

This implemented slice adds the strict companion to
`json::decode_or[T](value, fallback)`: `json::decode[T](value)`. The goal is to
decode JSON payloads where missing fields are data errors, while preserving the
artifact-safe schema lowering model already proven by `decode_or` and
`config::load_json_or`.

## Selected API

Add this public `std::json` function:

```muga
pub fn decode[T](value: json::Value): Result[T, json::Error]
```

The function is compiler-recognized only when called through the public
`std::json` package binding. The virtual package body should remain a fallback
error body, matching `decode_or`, so successful direct calls always depend on
compiler schema lowering.

## Type Target Policy

Unlike `decode_or`, the required decoder has no fallback value to supply `T`.
The first slice therefore derives `T` only from the expected return type of the
call:

- `decoded: Result[Settings, json::Error] = json::decode(value)` fixes `T` as
  `Settings`;
- `settings: Settings = try json::decode(value)` fixes `T` as `Settings` when
  the surrounding function returns `Result[_, json::Error]`;
- passing `json::decode(value)` to a parameter typed
  `Result[Settings, json::Error]` fixes `T` as `Settings`;
- `decoded = json::decode(value)` is rejected because the target type is not
  recoverable from the call itself.

If the expected type is absent or is not `Result[T, json::Error]`, report an
actionable type diagnostic that asks the user to annotate the binding or create
a `try` context with an explicit target type. Do not add source-level call type
arguments for this slice.

## Supported Targets

Use the same current structural target set as `json::decode_or[T]`:

- `String`;
- `Int`;
- `Bool`;
- `Option[T]` for supported non-nested optional targets;
- recursive `List[T]`;
- typed `Map[String, T]`;
- `Map[String, json::Value]`;
- concrete non-generic records whose fields recursively use the supported
  target set.

Continue rejecting unsupported targets with the existing schema target
diagnostics: nested `Option[Option[T]]`, enums, generic records, non-string map
keys, function types, `Unit`, runtime-backed opaque handles, and user opaque
types stay out of scope.

## Decode Semantics

Scalar, `Option[T]`, recursive list, typed map, and
`Map[String, json::Value]` decoding match the non-fallback side of `decode_or`.

Record decoding is strict:

- the JSON value must be an object;
- every schema field must be present in the object;
- missing fields return `json::Error` with `kind:
  json::ErrorKind::UnexpectedToken`, `offset: -1`, and a message that includes
  the missing field path;
- wrong-shape fields return the same path-aware shape errors used by
  `decode_or`;
- unknown object fields remain ignored so input records are forward-compatible;
- nested records apply the same required-field rule recursively.

The missing-field message should use rendered schema paths such as `.name`,
`.server.port`, or `.tags[1]` where applicable. This keeps the error shape
compatible with existing path-aware JSON helper diagnostics without adding a
new public `ErrorKind` variant.

## Lowering And Artifacts

Do not reuse the `DecodeJson` bytecode instruction in a way that changes
existing artifact meaning. Add a distinct required-decoder lowering path:

- typing records required decoder schemas separately from `decode_or` schemas;
- typed HIR and MIR preserve whether a call is default-overlay or required;
- bytecode emits a new required JSON decode instruction, or an equivalent
  persisted instruction variant, that carries only the JSON value and the
  schema;
- `.mgb` persistence serializes and validates the required decode schema using
  the existing `JsonDecodeSchema` artifact text format;
- runtime required decoding does not read a fallback value and calls a strict
  decoder path for records.

This keeps existing `DecodeJson` / `LoadJsonConfig` artifact behavior stable.

## Implementation Status

The implementation follows this contract:

- `std::json` exposes `decode[T](value)` with the same fallback-error body
  pattern as `decode_or`, while direct successful calls are compiler-lowered;
- typing recognizes only the public `std::json::decode` binding, requires an
  expected `Result[T, json::Error]` target, rejects unsupported targets with
  the same schema diagnostics as `decode_or`, and emits an actionable
  annotation diagnostic when the target is absent;
- typed HIR, MIR, bytecode, and `.mgb` persistence carry a distinct required
  decoder schema path through `DecodeJsonRequired`;
- runtime decoding reuses scalar, scalar-list, and `Map[String, json::Value]`
  conversion while applying strict required-field record decoding;
- source execution, missing-field paths, unsupported targets, unannotated call
  diagnostics, artifact-backed execution, and `run --built` are covered in
  `tests/examples.rs`.

## Examples

```muga
import std::json

record Settings {
  name: String
  port: Int
}

fn decode_settings(value: json::Value): Result[Settings, json::Error] {
  json::decode(value)
}

fn render_settings(value: json::Value): Result[String, json::Error] {
  settings: Settings = try json::decode(value)
  Result::Ok(settings.name.concat(":").concat(settings.port.to_string()))
}
```

A missing `port` field should produce a `Result::Err` with a message containing
`missing required JSON field at path .port`.

## Required Coverage

- source execution for scalar, list, map, record, nested record, and `try`
  propagation cases;
- missing-field and wrong-shape error tests with path-aware messages;
- unsupported-target rejecting tests;
- inference diagnostics for unannotated calls;
- artifact-backed execution and `run --built` coverage to prove schema payloads
  survive `.mgb` persistence;
- release-readiness evidence that docs, std package source, typing, HIR/MIR,
  bytecode, runtime, artifact persistence, and examples stay aligned.

## Non-Goals

This slice must not add `Option[T]` null/missing semantics, enum decoding,
generic record decoding, arbitrary typed map decoding, strict unknown-field
rejection, field rename attributes, validation attributes, source-level call
type arguments, generated schemas, TOML/YAML/JSON5, config discovery, full CLI
parser schemas, formatting templates, `Bytes`, process APIs, network APIs, or
streams.

## Implementation Plan

1. Done: document the required decoder contract and release-readiness evidence.
2. Done: add `json::decode[T]` to the virtual `std::json` package signature and
   recognize direct calls during typing.
3. Done: add typechecking for expected `Result[T, json::Error]`, actionable
   diagnostics for missing expected targets, and supported-target validation.
4. Done: preserve required decoder schemas through typed HIR, MIR, bytecode, and
   `.mgb` artifacts without changing `decode_or` artifact semantics.
5. Done: implement strict runtime decoding for records and reuse existing scalar,
   scalar-list, and object-map decoding.
6. Done: add source, artifact-backed, missing-field, unsupported-target, inference
   diagnostic, `try`, and `run --built` coverage.
7. Next: audit adoption before choosing TOML, broader decoder target types, full CLI
   parser schemas, formatting templates, config discovery, `Bytes`, process
   APIs, network APIs, or streams.
