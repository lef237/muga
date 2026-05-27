Status: std::config JSON default loader implemented

# std::config JSON Default Loading Design

The post-JSON-schema-decoder adoption audit selects `std::config` JSON default
loading as the next config/API boundary. The purpose of this slice is to remove
the repeated file-read, JSON-parse, schema-decode, and error-normalization
pipeline that remains after `json::decode_or[T](value, fallback)`, while keeping
path choice and CLI > config > defaults precedence explicit in ordinary Muga
code.

The implemented boundary is two compiler-owned helpers:

```muga
pub fn load_json_or[T](file_path: path::Path, fallback: T): Result[T, Error]
pub fn load_json[T](file_path: path::Path): Result[T, Error]
```

This is deliberately not a general config framework. It loads exactly one JSON
file chosen by the caller. `load_json_or` decodes it with the same
default-overlay semantics as `json::decode_or[T]`; `load_json` decodes it with
the same required-field semantics as `json::decode[T]`.

## Selected Public API

The first `std::config` package exposes:

```muga
package std::config

import std::path

pub enum ErrorKind {
  Read
  Parse
  Decode
}

pub record Error {
  kind: ErrorKind
  path: path::Path
  message: String
  offset: Int
  raw_code: Option[Int]
}

pub fn load_json_or[T](file_path: path::Path, fallback: T): Result[T, Error]
pub fn load_json[T](file_path: path::Path): Result[T, Error]
```

`ErrorKind::Read` represents host text-read failures. `ErrorKind::Parse`
represents invalid JSON text. `ErrorKind::Decode` represents a JSON value that
parsed successfully but does not match the supported target schema.

`offset` is the JSON byte offset for parse errors, and `-1` for read and
decode errors. `raw_code` carries the host IO raw code for read failures when
available and `Option::None` for parse and decode failures. `message` carries a
user-facing message from the underlying IO or JSON failure without requiring
apps to import `std::io` or manually map `json::Error`.

## Semantics

`config::load_json_or(file_path, fallback)` behaves as:

1. Read UTF-8 text from `file_path`.
2. Parse that text as `json::Value`.
3. Decode the value as the concrete type of `fallback` using
   `json::decode_or` default-overlay semantics.
4. Return `Result::Ok(decoded)` on success.
5. Return `Result::Err(config::Error)` on read, parse, or decode failure.

The fallback value supplies both the target type and missing-field defaults.
Unknown JSON object fields are ignored in the first slice, exactly as in
`json::decode_or[T]`. Present wrong-shaped fields return `ErrorKind::Decode`
with the JSON path in `message`, for example
`expected JSON String at path .tags[1]`.

`config::load_json(file_path)` behaves the same through file read and JSON
parse, then decodes with strict required-field semantics. Because it has no
fallback value, the target type must come from an expected `Result[T,
config::Error]` context, such as an annotated local binding, an annotated
function return, or `try config::load_json(...)` in a `Result[..., config::Error]`
function where the expected success type is known.

The caller still decides path selection and precedence. The generated config
apps now use the path discovery policy documented in
[config-path-discovery.md](config-path-discovery.md): `--config` first,
`MUGA_CONFIG_PATH` next, and the generated JSON file as the fallback. For
example, the current config app keeps CLI path lookup and CLI setting overrides
in app code:

```muga
args = env::args()
config_path = path::from_string(cli::option_or(args, "config", "samples/projects/config_app/config/settings.json"))
configured = try result::map_err(config::load_json_or(config_path, default_settings()), config_error_message)
settings = try result::map_err(cli::parse_or(settings_args(args), configured), cli_error_message)
```

This preserves explicit CLI > config > defaults behavior while removing the
local `read_config`, hand-written settings override, and direct `json::parse` /
`json::decode_or` plumbing from app code. The small `settings_args` helper keeps
the config path outside the settings record while `cli::parse_or[T]` owns the
typed settings overlay.

## Compiler And Artifact Model

Both helpers are compiler-recognized at direct call sites. They cannot be
plain generic Muga functions because ordinary runtime values do not carry type
reflection, and an ordinary `std::config` body cannot inspect `T`.

The implementation reuses the existing JSON schema machinery:
`load_json_or[T]` reuses the `json::decode_or[T]` schema machinery, and
`load_json[T]` reuses the strict `json::decode[T]` schema machinery.

- type checking validates the target with the same supported schema set as
  `json::decode_or[T]` and `json::decode[T]`;
- unsupported targets are compile-time diagnostics;
- lowering carries a serializable decoder schema into MIR and bytecode;
- emitted `.mgb` artifacts store the schema payload needed for artifact-backed
  execution;
- artifact-backed `run` must not load dependency source bodies to reconstruct
  schemas.

The implementation uses distinct MIR/bytecode instructions, `LoadJsonConfig`
for default-overlay loading and `LoadJsonConfigRequired` for required loading,
instead of trying to synthesize user-visible call chains. The dedicated
instructions keep the IO/parse/decode error mapping in one place and keep
artifact serialization explicit.

## Supported Target Types

The supported target set is exactly the current `json::decode_or[T]` /
`json::decode[T]` target set:

- `String`
- `Int`
- `Bool`
- `Option[T]` for supported non-nested optional targets
- recursive `List[T]`
- typed `Map[String, T]`
- `Map[String, json::Value]`
- concrete non-generic records whose fields recursively use only supported
  target types
- concrete non-generic enums whose payloads recursively use only supported
  target types

For enum targets, zero-payload variants decode from string tags and one-payload
variants decode from single-key objects using the variant name or the explicit
`@json(rename: "...")` wire name. For schema polish, record fields and enum variants
can use `@json(rename: "...")` wire names and `@json(alias: "...")` input aliases
across JSON/config decoding, and records can opt into
`@json(deny_unknown_fields)` to reject unexpected JSON/config object keys.
Generic enum decoding remains deferred.

`std::config` still does not add generic record decoding, generic enum
decoding, `Option[Option[T]]`, non-string map keys, `Float`, `Decimal`, custom
validators, required-field annotations, TOML, or config discovery.

## Diagnostics And Errors

Compile-time unsupported target diagnostics should name `config::load_json_or`
or `config::load_json` and point users at the same concrete target rules as
`json::decode_or` / `json::decode`.

Runtime recoverable errors should be ordinary `config::Error` values:

| Failure | Error kind | Offset | raw_code | Message source |
|---|---|---:|---|---|
| file cannot be read | `Read` | `-1` | copied from `io::IOError.raw_code` | underlying IO message |
| JSON text is invalid | `Parse` | copied from `json::Error.offset` | `Option::None` | underlying JSON parse message |
| JSON shape is wrong | `Decode` | `-1` | `Option::None` | underlying JSON decode message with path |

Hard runtime diagnostics remain appropriate only for compiler/runtime bugs,
for example a malformed artifact instruction or a corrupted internal value.
User data and host IO failures should stay in `Result`.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| `config::load_json_or[T](path, fallback): Result[T, config::Error]` | Removes the remaining boilerplate in `config_app`; keeps defaults explicit; reuses proven decoder schema payloads; gives apps one public config error type. | Requires another compiler-owned generic direct call and a new public std package. | Select first implementation |
| `config::load_json[T](path): Result[T, config::Error]` | Supports required config files and stricter app startup without manual read/parse/decode plumbing. | Requires expected-type diagnostics and a second artifact instruction. | Select after required JSON decoding is stable |
| Return `Result[T, String]` | Shorter for samples. | Loses machine-readable error kind, offset, path, and raw IO code; violates stdlib review rules for effect APIs. | Reject |
| Return nested source errors such as `Result[T, Result[io::IOError, json::Error]]` | Preserves underlying details. | Awkward to match and does not distinguish parse from decode without an additional wrapper. | Reject |
| Public enum payload error variants | Preserves full typed source errors. | Muga enum variants carry one payload, so parse/decode variants need extra records; matching is noisier than the first config app needs. | Defer |
| String path overload `load_json_or_path` | Convenient for small scripts. | Creates parallel path APIs before the `std::path` contract needs it; current config app already uses `path::from_string`. | Defer |
| Ambient discovery such as `config::load_default` | Conventional for frameworks. | Hides precedence, working-directory policy, environment behavior, and package layout assumptions. | Reject for first slice |
| TOML support | Common for app settings. | Requires a new parser, numeric/date policy, table/array semantics, and separate error contracts. | Defer |
| Required `json::decode[T]` | Useful for strict payloads. | Does not solve file read, parse, and error-normalization boilerplate by itself. | Implemented foundation |
| Generated config app template | Helps onboarding. | Should teach the selected `std::config` API after it exists. | Defer |

## Non-Goals

The first `std::config` implementation did not add config discovery, implicit
environment-variable loading, automatic CLI merging, TOML/YAML/JSON5, a JSONPath
string parser, validation attributes,
generated source, macros, runtime reflection, schema generation, service
manifests, package manifest config, formatting templates, `Bytes`, process
APIs, network APIs, streams, or standard-stream handles.
Generated app path discovery is a later source-level template policy in
[config-path-discovery.md](config-path-discovery.md), not hidden behavior inside
`std::config`.

## Implementation Plan

1. Done: record the post-JSON-schema-decoder selection and release-readiness
   evidence.
2. Done: write this design with the selected public API, error model, schema
   lowering, artifact behavior, diagnostics, and non-goals.
3. Done: add `std::config` to the virtual std package set with the public
   `ErrorKind`, `Error`, and placeholder `load_json_or[T]` signature.
4. Done: reuse `JsonDecodeSchema` validation for direct
   `config::load_json_or[T]`
   calls and lower the file path, fallback, and schema into MIR/bytecode.
5. Done: implement runtime text read, JSON parse, schema decode, and
   `config::Error` construction.
6. Done: persist and validate the new bytecode instruction in implementation
   artifacts.
7. Done: refresh `samples/projects/config_app` and add source, artifact-backed,
   shape-error, and `run --built --format=json` coverage.
8. Done: audit adoption, add the generated config app template, implement
   required `json::decode[T]`, and expand structural decoder targets.
9. Done: add `config::load_json[T]` with required decode semantics and a
   persisted `LoadJsonConfigRequired` instruction.
10. Done: add `samples/packages/app/std_config` to demonstrate strict and
   default-overlay JSON config loading from source and artifact-backed runs.
11. Next: keep TOML, config discovery, process APIs, network APIs, streams, and
   broader config frameworks separate until a narrower adoption gap requires
   them.
