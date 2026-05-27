Status: strict CLI parser schema implemented

# Strict CLI Parser Schema

The first CLI schema implementation made config-style apps practical through
`cli::parse_or[T](args, defaults)`: command-line values overlay a typed default
or config-loaded record. CLI-only tools still have a visible gap. A command
such as a generator, linter, or deployment helper often has required options and
should not invent placeholder defaults just to parse arguments.

This design adds the strict no-default companion:

```muga
pub fn parse[T](args: List[String]): Result[T, Error]
```

The implementation should reuse the existing `CliSchema`, `@cli(...)`
metadata, validation rules, enum parsing, interface persistence, and artifact
payloads. It should not add TOML, config discovery, source-level call type
arguments, positional fields, combined short flags, attached values, subcommands, record-level command
metadata, client generation, or host-effect APIs.

## Goals

Short-Term Goal: let command-line-only records parse required options without
placeholder defaults while preserving recoverable `cli::Error` values.

Medium-Term Goal: support both practical settings workflows: config/default
overlays through `parse_or`, and strict required-option tools through `parse`.

Long-Term Goal: keep Muga's typed data contract usable across config loading,
CLI parsing, validation, schema/artifact reuse, generated docs, and future
tool/client generation without runtime reflection or duplicate glue.

## Selected Public API

Add a compiler-recognized strict parser to `std::cli`:

```muga
pub fn parse[T](args: List[String]): Result[T, Error]
```

Add one `ErrorKind` variant:

```muga
pub enum ErrorKind {
  UnknownArgument
  MissingArgument
  MissingValue
  InvalidValue
  Validation
  UnsupportedTarget
}
```

`MissingArgument` is used when a required field is absent. `MissingValue`
continues to mean an option spelling was present but did not receive the value
it requires.

The function stays pure over explicit `List[String]`. It does not read
`env::args()`, print usage, exit on `--help`, read config files, discover
environment variables, or alter process status.

## Type Target Policy

`parse` has no default value from which the checker can infer `T`. Match the
already implemented strict `json::decode[T](value)` policy: derive `T` only from an expected `Result[T, cli::Error]` target.

Supported contexts:

- `parsed: Result[Settings, cli::Error] = cli::parse(args)` fixes `T` as
  `Settings`;
- `settings: Settings = try cli::parse(args)` fixes `T` as `Settings` when the
  surrounding function returns `Result[_, cli::Error]`;
- passing `cli::parse(args)` to a parameter typed `Result[Settings, cli::Error]`
  fixes `T` as `Settings`.

Rejected contexts:

- `parsed = cli::parse(args)` because no expected result type exists;
- expected types other than `Result[T, cli::Error]`;
- source-level calls such as `cli::parse[Settings](args)` until Muga has a
  general source-level type-argument design.

The diagnostic should ask the user to annotate the binding, add a typed `try`
context, or pass the call where `Result[Settings, cli::Error]` is expected.
Do not add source-level call type arguments in this slice.

## Supported Targets

The first strict parser should accept concrete non-generic records whose fields
are all representable from CLI input or have a clear absent-value synthesis
policy.

Required field types:

- `String`;
- `Int`;
- concrete zero-payload enums.

Synthesized-when-absent field types:

- `Bool` defaults to `false`;
- `Option[String]`, `Option[Int]`, `Option[Bool]`, and supported
  zero-payload-enum options default to `Option::None`;
- `List[String]`, `List[Int]`, `List[Bool]`, and supported zero-payload-enum
  lists default to `[]`.

Rejected strict targets:

- non-record top-level targets;
- unresolved type parameters;
- generic records/enums;
- functions;
- `Unit`;
- `Result[T, E]`;
- opaque handles;
- nested records;
- maps;
- `std::json::Value`;
- one-payload enums;
- lists of records or other unsupported element types;
- nested `Option[Option[T]]`;
- any other field that `parse_or` can only preserve from defaults.

The important difference from `parse_or` is that strict parsing has no defaults
to preserve unsupported fields. Reject such targets at type checking with a
diagnostic that names `cli::parse`.

## Field Metadata Semantics

Strict parsing uses the same field-level metadata as `parse_or`:

- primary names come from `@cli(name: "...")`, then `@json(rename: "...")`,
  then the Muga field name;
- accepted aliases come only from repeated `@cli(alias: "...")`;
- `@json(alias: "...")` remains JSON/config compatibility metadata and is not a
  CLI alias;
- `@cli(help: "...")` stays usage metadata;
- `@cli(hidden)` fields remain parseable but are omitted from existing
  `usage_for`.

Hidden strict fields must be able to synthesize an absent value. A hidden
`String`, `Int`, or required enum field would be required but undiscoverable in
usage, so `cli::parse` should reject that schema at type checking unless a
future command-metadata design provides another help surface.

Duplicate primary/alias checks are identical to the existing `CliSchema`
pipeline, including hidden fields because they remain parseable.

## Argument Semantics

Reuse the implemented long-option grammar:

- `--name value` and `--name=value` are accepted for all supported field
  shapes;
- a bare `--flag` sets a `Bool` field to `true`;
- `--flag=false` and `--flag false` set a `Bool` field to `false`;
- repeated scalar options keep last-value-wins behavior;
- repeated list options append in argument order from an initially empty list;
- `--` stops option parsing;
- unknown `--name` before `--` returns `UnknownArgument`;
- present non-bool options without a following value return `MissingValue`;
- absent required `String`, `Int`, or zero-payload enum fields return
  `MissingArgument`;
- invalid scalar or enum values return `InvalidValue`;
- `@validate(...)` runs after parsing and absent-value synthesis, returning
  `Validation` on failure.

Runtime error `argument` strings should use the user's spelling for present
arguments and the primary option name for missing required fields, for example
`--server-host`.

## Usage Decision

Do not add a strict no-default usage helper in this slice.

`cli::usage_for[T](program, defaults)` remains the overlay/config helper. It has
a value argument, so `T` is inferable and displayable defaults are available.
Strict parsing has neither defaults nor source-level type arguments. Adding a
schema witness value, explicit user-built schema, or special-purpose type-token
API would be awkward and would duplicate the typed contract this feature is
trying to preserve.

The strict parser implementation should instead make missing-required errors
clear and should leave required-marker usage generation for a later slice that
first designs one of these broader anchors:

- source-level call type arguments such as `cli::usage[Settings]("tool")`;
- record-level command metadata and a tool command that emits usage from a
  named public record;
- a general type-token/schema API that is useful beyond CLI parsing.

## Schema And Artifacts

Use the existing dedicated `CliSchema` payload and add only the minimum mode
metadata needed to distinguish overlay and strict parsing during lowering and
runtime execution.

Implementation expectations:

- the virtual `std::cli` package exposes `parse[T]` with the same fallback-error
  body pattern as other compiler intrinsics;
- typing records strict parse schemas separately from `parse_or` schemas;
- typed HIR, MIR, bytecode, and `.mgb` persistence carry a distinct strict CLI
  parse operation or an explicit strict mode bit;
- `.mgi` interfaces continue to carry public record CLI metadata and validation
  rules without requiring provider source;
- malformed `.mgb` strict CLI schema payloads are hard artifact errors before
  execution;
- source execution, explicit artifact roots, and `run --built` use identical
  parse behavior.

Do not change the existing `parse_or` or `usage_for` artifact meaning.

## Implementation Status

The implementation follows this contract:

- `std::cli` exposes `parse[T](args)` with the same fallback-error body pattern
  as other compiler-recognized helpers;
- typing recognizes only the public `std::cli::parse` binding, requires an
  expected `Result[T, cli::Error]` target, rejects unsupported strict fields,
  and rejects hidden required fields that cannot synthesize absent values;
- `cli::ErrorKind` includes `MissingArgument` for absent required fields;
- `Bool`, `Option[T]`, and `List[T]` synthesize `false`, `Option::None`, and
  `[]` when absent;
- supported zero-payload enum, `Option[Enum]`, and `List[Enum]` targets share
  the same tag parsing as `parse_or`;
- typed HIR, MIR, bytecode, and `.mgb` persistence carry a distinct
  `CliParse` operation with the existing `CliSchema` payload;
- source execution, explicit artifact roots, and `run --built` preserve strict
  parsing behavior;
- no strict no-default usage helper was added.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| `cli::parse[T](args)` with expected-result type inference | Directly solves CLI-only required options, matches `json::decode[T]`, reuses `CliSchema`, and avoids new syntax. | Requires a missing-required policy and a distinct lowering/runtime path. | Select |
| Reuse `cli::parse_or[T](args, defaults)` with placeholder defaults | No compiler work. | Forces fake values into every CLI-only tool and hides required-option errors. | Reject |
| Add source-level call type arguments now | Would make `cli::parse[Settings](args)` and future no-default usage helpers obvious. | This is a general language feature, not a CLI parser implementation detail. | Defer |
| Add a schema witness or type-token usage API now | Could enable no-default usage text without type arguments. | Awkward user ergonomics and a new abstraction that has not been justified beyond usage text. | Reject for this slice |
| Treat absent `Bool` as required | Makes every field uniformly required. | Boolean flags conventionally mean false when absent; required booleans are usually better modeled as enums. | Reject |
| Synthesize `Bool=false`, `Option::None`, and `[]` | Keeps common CLI defaults implicit without placeholder records. | Users needing required list/non-empty behavior must use validation. | Select |
| Preserve unsupported fields like `parse_or` | Would keep broader record targets accepted. | Strict parsing has no default record from which to preserve field values. | Reject |
| Add TOML/config discovery/subcommands/short flags together | Higher-level CLI polish. | Larger than the required-option gap and depends on settled strict parse behavior. | Defer |

## Implementation Plan

1. Done: implement `parse_or[T]`, `usage_for[T]`, `CliSchema`, `@cli(...)`
   names, aliases, help, hidden fields, validation, and artifact persistence.
2. Done: audit generated `config-app` metadata adoption and select strict
   parser design.
3. Done: write this strict parser schema design and release-readiness coverage.
4. Done: implement `cli::parse[T](args)` with expected-target inference,
   `MissingArgument`, strict target validation, absent-value synthesis, runtime
   parsing, source/artifact/`run --built` coverage, and docs.
5. Done: audit strict CLI parser adoption and keep the selected adoption
   evidence in this design, the checked-in sample, and release-readiness tests.
6. Done: implement a checked-in strict CLI tool sample at
   `samples/projects/cli_tool` before TOML, config discovery, no-default usage
   helpers, combined short flags, attached values, subcommands, full client generation, generic
   encoding/decoding, broader validators, or host-effect APIs.
7. Done: audit strict CLI tool sample adoption and keep the selected generated
   template evidence in code, examples, and release-readiness tests.
8. Done: implement generated `muga new --template cli-tool` adoption with
   template parsing, usage/completions, tests, docs, and release-readiness
   coverage.
9. Done: audit generated cli-tool template adoption and keep the historical
   manual-help evidence in sample/template tests.
10. Done: implement strict CLI manual help adoption.
11. Done: audit strict CLI manual help adoption and keep the selected
   no-default usage helper evidence in `strict-cli-no-default-usage.md`.
12. Done: design the strict CLI no-default usage helper in
   [strict-cli-no-default-usage.md](strict-cli-no-default-usage.md).
13. Done: implement `cli::usage_for_required[T](program)` with explicit call
   type arguments, schema lowering, source/artifact coverage, and
   sample/template adoption.
14. Done: audit strict CLI no-default usage helper adoption and keep the
   selected command-metadata evidence in `cli-command-metadata.md`.
15. Done: implement record-level CLI command metadata from
(cli-command-metadata.md).
   Done: audit CLI command metadata adoption and keep the selected short-option
   evidence in [cli-short-option-metadata.md](cli-short-option-metadata.md). Done: design CLI short option metadata in [cli-short-option-metadata.md](cli-short-option-metadata.md). Done: implement CLI short option metadata. Done: audit CLI short option metadata adoption. Done: design CLI positional field metadata in [cli-positional-field-metadata.md](cli-positional-field-metadata.md). Done: implement CLI positional field metadata. Done: audit CLI positional field metadata adoption. Done: design built-in CLI help policy in [cli-built-in-help-policy.md](cli-built-in-help-policy.md). Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata.
16. Later: extend generated usage with combined short flags, attached values, and subcommands only after
   command summaries are implemented and audited.
