Status: full CLI parser schema design selected

Implementation status: first CLI parser schema slice implemented; first
field-level CLI metadata implemented in
[cli-field-metadata.md](cli-field-metadata.md); strict no-default parser
implemented in [strict-cli-parser-schema.md](strict-cli-parser-schema.md)

# CLI Parser Schema

Muga's first practical command-line surface is intentionally small:
`std::env::args()` returns explicit arguments, and `std::cli` exposes pure helper
functions over `List[String]`. That works for small programs, but generated
config apps and real tools still repeat the same mapping code from CLI arguments
into typed settings records.

This document designs the next CLI boundary. The selected first implementation
is a compiler-owned typed overlay parser:

```muga
pub fn parse_or[T](args: List[String], defaults: T): Result[T, cli::Error]
```

The parser should read explicit argument lists, overlay supported fields on a
default typed value, and preserve unsupported or file-only fields from the
default. This lets a program keep the already-proven precedence:

1. hard-coded defaults;
2. JSON config through `std::config::load_json_or[T]`;
3. CLI overrides through `std::cli::parse_or[T]`.

Generated usage/help is designed as the paired read-only surface:

```muga
pub fn usage_for[T](program: String, defaults: T): String
```

`usage_for` takes a default value because Muga does not yet have source-level
call type arguments, and the return type alone cannot infer `T`.

## Goals

Short-Term Goal: replace manual `apply_args(settings, args)` functions in
config-style apps with one compiler-owned `std::cli` overlay parser for concrete
records.

Medium-Term Goal: let one typed settings record drive JSON config loading,
schema export, typed JSON encoding, CLI parsing, and generated usage text
without runtime reflection.

Long-Term Goal: make Muga a practical language for local tools and service
launchers where configuration, command-line interfaces, docs, and tests all use
one explicit public data model.

## Selected Public API

Add a compiler-recognized overlay parser to `std::cli`:

```muga
pub fn parse_or[T](args: List[String], defaults: T): Result[T, Error]
```

Add a generated usage helper over the same schema:

```muga
pub fn usage_for[T](program: String, defaults: T): String
```

Add public error records:

```muga
pub enum ErrorKind {
  UnknownArgument
  MissingValue
  InvalidValue
  Validation
  UnsupportedTarget
}

pub record Error {
  kind: ErrorKind
  argument: String
  message: String
}
```

The first parser remains pure over explicit `List[String]`. It must not read
process arguments implicitly, exit on help, print usage text, read config files,
or apply hidden environment-variable precedence.

## Supported First Targets

The first implementation should accept a concrete non-generic record as `T`.
It should expose only supported CLI fields and preserve the rest from
`defaults`.

Supported exposed field types:

- `String`;
- `Int`;
- `Bool`;
- `Option[String]`, `Option[Int]`, and `Option[Bool]`;
- `List[String]`, `List[Int]`, and `List[Bool]`;
- concrete zero-payload enums, parsed from their canonical string tags.

Preserved overlay-only field types:

- nested records;
- `Map[String, T]`;
- `std::json::Value`;
- lists of records;
- concrete one-payload enums;
- any other JSON/config-supported type that does not yet have a clear CLI
  syntax.

Preserved fields are copied from `defaults` and omitted from generated usage.
If a user later annotates such a field as CLI-exposed, the type checker should
reject it with a targeted unsupported CLI field diagnostic.

Rejected targets:

- unresolved type parameters;
- generic records/enums;
- functions;
- opaque handles;
- `Unit`;
- `Result[T, E]`;
- non-record top-level `T`;
- nested `Option[Option[T]]`.

Strict no-default parsing, `cli::parse[T](args)`, is now implemented from
[strict-cli-parser-schema.md](strict-cli-parser-schema.md). It keeps overlay
parsing unchanged, derives `T` from an expected `Result[T, cli::Error]` target
like strict JSON decoding, adds `MissingArgument` for absent required options,
and synthesizes absent `Bool`, `Option`, and `List` fields.

## Naming And Metadata

The first implementation should derive long option names from the primary JSON
wire name when present, otherwise from the Muga field name. That keeps generated
config and CLI names aligned for current apps.

CLI-specific metadata is still needed before richer CLI surfaces:

```muga
@cli(name: "server-host")
@cli(alias: "host")
@cli(positional: 1)
@cli(help: "Server host name")
@cli(hidden)
```

This design selects the `@cli(...)` namespace, but does not require the first
implementation to support every argument. The recommended staging is:

1. derive names from JSON primary names or field identifiers;
2. add `@cli(name: "...")` and `@cli(positional: N)` before strict parsing;
3. add `@cli(alias: "...")`, `@cli(help: "...")`, and `@cli(hidden)` before
   polished generated help.

Do not reuse `@json(alias: "...")` as CLI aliases. JSON aliases are input
compatibility metadata for payload migration. CLI aliases have help text,
conflict, and discoverability rules of their own.

## Argument Semantics

Long options:

- `--name value` and `--name=value` are accepted for string, int, bool, option,
  list item, and enum fields;
- a bare `--name` for `Bool` sets the field to `true`;
- `--name=false` and `--name false` set a bool field to `false`;
- a missing value for a non-bool option returns `ErrorKind::MissingValue`;
- repeated scalar options use the last value;
- repeated list options replace the default list when at least one value is
  present, matching existing `cli::option_values_or`;
- `--` stops option parsing and later values are treated as positionals;
- unknown `--name` before `--` returns `ErrorKind::UnknownArgument`.

Positionals:

- no fields are positional by default;
- `@cli(positional: N)` opts a field into positional parsing, using the
  1-based policy later fixed in
  [cli-positional-field-metadata.md](cli-positional-field-metadata.md);
- positionals are read after option parsing rules remove option values;
- the first positional implementation keeps positional fields separate from
  named option fields;
- duplicate positional indexes are compile-time schema errors.

Enums:

- zero-payload enums parse from primary JSON tags or variant names;
- aliases are not accepted unless future CLI metadata explicitly adds them;
- one-payload enum parsing is deferred because the CLI syntax is not obvious.

Lists:

- repeated `--tag value` / `--tag=value` creates `List[String]`;
- `List[Int]` and `List[Bool]` parse each item and return the first invalid
  item as `ErrorKind::InvalidValue`;
- comma splitting is deferred because escaping and empty-item policy need a
  separate decision.

## Validation

`@validate(...)` should run after parsing each exposed field. Validation errors
return `ErrorKind::Validation` with the option or positional label in
`argument`, and the same user-facing validation message style as JSON typed
encoding.

The first implementation returns the first error as `Result::Err(Error)`.
Accumulating all parse and validation errors is deferred until Muga has enough
CLI error cases to justify a public `List[cli::Error]` result shape.

## Generated Usage

`usage_for(program, defaults)` returns deterministic plain text:

```text
Usage: app [options]

Options:
  --name <String>        default: Muga
  --port <Int>          default: 8080
  --verbose[=<Bool>]    default: false
  --tag <String>        repeatable
```

The exact spacing can evolve during implementation, but the contract must be
stable enough for tests. Usage generation should:

- order fields by record declaration order;
- show primary long names only in the first implementation;
- mark list fields as repeatable;
- omit preserved unsupported fields;
- show default scalar values when they are displayable;
- avoid reading terminal width or locale.

Help flag behavior stays explicit. Programs can use existing helpers:

```muga
if cli::has_flag(args, "help") {
  println(cli::usage_for("app", defaults))
} else {
  settings = try cli::parse_or(args, defaults)
  ...
}
```

The parser itself should not special-case `--help`, print text, or terminate the
program.

## Schema And Artifacts

The parser should be compiler-owned like typed JSON decoding/encoding:

- type checking derives a CLI schema from concrete record metadata and the
  `defaults` argument type;
- package interfaces must carry enough record, field, JSON name, validation, and
  future CLI metadata to parse loaded-interface types without provider source;
- `.mgb` implementation artifacts must carry the complete CLI schema payload
  needed by runtime parsing and usage generation;
- malformed or stale CLI schema payloads are hard artifact errors before
  execution;
- `parse_or` and `usage_for` should work under source execution, explicit
  artifact roots, and `run --built`.

The first field-level metadata design in
[cli-field-metadata.md](cli-field-metadata.md) now implements a dedicated
`CliSchema` Rust type as the implementation boundary instead of continuing to
overload `JsonDecodeSchema`.
JSON and CLI contracts overlap but are not the same: option aliases, help text,
hidden fields, positional fields, bool flags, generated usage, and CLI-only
metadata stay separate.

The first `@cli(...)` metadata slice carries CLI schema payloads through typed
HIR, MIR, bytecode, `.mgb` artifacts, explicit artifact roots, and `run
--built`, while still reusing `@validate(...)` for parsed field validation.
JSON wire names remain the fallback for primary CLI names only when
`@cli(name: "...")` is absent.

## Implemented First Slice

The first implementation covers:

- public `std::cli::ErrorKind` and `std::cli::Error`;
- compiler-owned `cli::parse_or[T](args, defaults)` and
  `cli::usage_for[T](program, defaults)`;
- concrete non-generic record targets;
- exposed fields for `String`, `Int`, `Bool`, `Option[String]`,
  `Option[Int]`, `Option[Bool]`, `List[String]`, `List[Int]`, `List[Bool]`,
  and zero-payload enums;
- preservation of omitted unsupported fields from the default record value;
- long options with `--name value`, `--name=value`, bare bool flags, repeated
  scalar-list options, and runtime `cli::Error` values for unknown, missing,
  invalid, and validation failures;
- deterministic plain-text usage generation for exposed fields;
- schema payload propagation through typed HIR, MIR, bytecode, `.mgb`
  artifacts, explicit artifact roots, and `run --built`, now using dedicated
  `CliSchema` payloads;
- field-level `@cli(name: "...", alias: "...", help: "...", hidden)` metadata
  for primary option names, aliases, help text, and hidden parseable fields.

The implementation does not yet add positional fields, short flags, strict
no-default parsing, subcommands, config discovery, TOML, or client/schema
generation.

## Diagnostics

Compile-time unsupported target diagnostics should name the active helper:

- `` `cli::parse_or` supports only concrete non-generic record targets ``;
- `` `cli::usage_for` supports only concrete non-generic record targets ``.

Recoverable runtime errors should be ordinary `cli::Error` values:

- unknown option: `UnknownArgument`, argument `--unknown`;
- missing value: `MissingValue`, argument `--port`;
- invalid scalar: `InvalidValue`, argument `--port`, message including the raw
  value;
- invalid enum tag: `InvalidValue`, argument `--mode`, message listing accepted
  tags;
- validation failure: `Validation`, argument `--name`, message from
  `@validate(...)`.

The first implementation should not use compiler diagnostics for malformed user
arguments. CLI arguments are runtime input, so they belong in `Result`.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| `cli::parse_or[T](args, defaults)` overlay parser | Replaces manual config-app argument overlay, preserves CLI > config > defaults precedence, infers `T` from defaults, and can preserve unsupported file-only fields. | Requires compiler-owned schema lowering, runtime parsing, a new error type, and careful supported-field policy. | Select first |
| `cli::usage_for[T](program, defaults)` | Gives generated apps stable help text without source-level type arguments; reuses the same schema as parsing. | Needs deterministic formatting and default rendering policy. | Select first |
| Strict `cli::parse[T](args)` | Useful for CLI-only tools with required options. | Needs expected-target inference, missing-required errors, absent-value synthesis, and a clear deferral for no-default usage text until a type anchor exists. | Implemented |
| Explicit schema records built by users | Avoids compiler intrinsic work and can be flexible. | Too verbose for generated apps; duplicates type information already present in records/interfaces; hard to keep in sync with validation and JSON contracts. | Reject for first slice |
| Runtime reflection over arbitrary records | Looks simple at API level. | Muga has no runtime reflection, and artifact-backed execution must not require provider source. | Reject |
| Reuse `@json(alias)` and all JSON names as CLI metadata | Reduces new syntax. | JSON compatibility aliases have different semantics from CLI aliases and help text. It would make payload migration affect command-line behavior accidentally. | Reject |
| Full subcommands and nested command schemas | Important for polished tools. | Requires command dispatch, per-subcommand records, help sections, shared options, and exit/status conventions. | Defer |
| Environment variable and config discovery integration | Useful for twelve-factor apps and services. | Hides precedence before the explicit CLI overlay parser is proven. | Defer |
| Multi-error accumulation | Better UX for large CLIs. | Requires a public result shape such as `Result[T, List[cli::Error]]` or a new report record, and broad test/documentation policy. | Defer |
| Short flags and combined flags | Familiar for Unix-style CLIs. | Needs alias syntax, grouping rules, value attachment policy, and help rendering. Long options already match existing helpers. | Defer |

## Non-Goals

This design does not add:

- implicit `env::args()` reading;
- automatic process exit or stdout/stderr output;
- TOML/YAML/JSON5 config parsing;
- config discovery or environment variable precedence;
- subcommands;
- short flags such as `-v` or `-abc`;
- comma-separated list parsing;
- custom parser callbacks;
- generated shell completions from CLI schemas;
- full client generation, OpenAPI, RPC, HTTP, process APIs, `Bytes`, streams, or
  broader host effects;
- generic record/enum schema instantiation;
- source-level call type arguments.

## Implementation Plan

1. Done: implement pure `std::cli` helpers for positional values, flags, long
   options, repeated string options, and typed scalar parsing.
2. Done: implement JSON/config typed records, validation, schema export, and
   typed JSON encoding so settings records are stable data contracts.
3. Done: audit typed JSON encoding adoption in
   [json-typed-encoding.md](json-typed-encoding.md) and select full CLI parser
   schema design.
4. Done: select `cli::parse_or[T](args, defaults)` plus
   `cli::usage_for[T](program, defaults)` as the first compiler-owned CLI schema
   boundary.
5. Done: implement the smallest CLI parser schema slice for concrete
   non-generic record overlay, source/artifact/`run --built` execution, generated
   usage, docs, and release-readiness coverage.
6. Done: audit the first implementation and select generated `config-app` CLI
   schema adoption.
7. Done: refresh the config-app sample and template to use `cli::parse_or[T]`.
8. Done: expose generated config-app usage with `cli::usage_for[T]`.
9. Done: design and implement first field-level `@cli(...)` metadata.
10. Done: refresh generated config-app settings metadata and audit adoption.
11. Done: design strict `cli::parse[T](args)` in
    [strict-cli-parser-schema.md](strict-cli-parser-schema.md).
12. Done: implement `cli::parse[T](args)` before TOML, full client generation,
    generic encoding/decoding, broader validators, config discovery automation,
    combined short flags, attached values, subcommands, or host-effect APIs.
13. Done: audit strict CLI parser adoption and keep the selected adoption
    evidence in this design, the checked-in sample, and release-readiness tests.
14. Done: implement a checked-in strict CLI tool sample at
    `samples/projects/cli_tool` before TOML, config discovery, no-default usage
    helpers, combined short flags, attached values, subcommands, full client generation, generic
    encoding/decoding, broader validators, or host-effect APIs.
15. Done: audit strict CLI tool sample adoption and keep the selected generated
    template evidence in code, examples, and release-readiness tests.
16. Done: implement generated `muga new --template cli-tool` adoption with
    template parsing, usage/completions, tests, docs, and release-readiness
    coverage.
17. Done: audit generated cli-tool template adoption and keep the historical
    manual-help evidence in sample/template tests.
18. Done: implement strict CLI manual help adoption.
19. Done: audit strict CLI manual help adoption and keep the selected
    no-default usage helper evidence in `strict-cli-no-default-usage.md`.
20. Done: design the strict CLI no-default usage helper in
    [strict-cli-no-default-usage.md](strict-cli-no-default-usage.md).
21. Done: implement `cli::usage_for_required[T](program)` with explicit call
    type arguments, schema lowering, source/artifact coverage, and
    sample/template adoption.
22. Done: audit strict CLI no-default usage helper adoption and keep the
    selected command-metadata evidence in `cli-command-metadata.md`.
23. Done: implement record-level CLI command metadata from
(cli-command-metadata.md).
    Done: audit CLI command metadata adoption and keep the selected short-option
    evidence in [cli-short-option-metadata.md](cli-short-option-metadata.md). Done: design CLI short option metadata in [cli-short-option-metadata.md](cli-short-option-metadata.md). Done: implement CLI short option metadata. Done: audit CLI short option metadata adoption. Done: design CLI positional field metadata in [cli-positional-field-metadata.md](cli-positional-field-metadata.md). Done: implement CLI positional field metadata. Done: audit CLI positional field metadata adoption. Done: design built-in CLI help policy in [cli-built-in-help-policy.md](cli-built-in-help-policy.md). Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata.
