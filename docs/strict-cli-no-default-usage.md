Status: strict CLI no-default usage helper implemented

# Strict CLI No-Default Usage Helper

Strict CLI tools now parse required command-line records with
`cli::parse[T](args)`. This design and implementation define the generated
usage helper that closes the manual-help gap without fake defaults:

```muga
pub fn usage_for_required[T](program: String): String
```

Callers provide the record type explicitly:

```muga
usage = cli::usage_for_required[Command]("cli-tool")
```

The helper is the strict/no-default companion to
`cli::usage_for[T](program, defaults)`. It renders options from the same
compiler-owned `CliSchema` used by `cli::parse[T](args)`.

## Goals

Short-Term Goal: replace duplicated manual strict CLI usage text with generated
required-field usage driven by record metadata, validation, and `CliSchema`.

Medium-Term Goal: keep generated `cli-tool` starters practical while making the
type-anchor rule explicit enough for later schema-producing helpers.

Long-Term Goal: let Muga expose typed schemas, CLI parsers, usage text,
validation, artifacts, and generated tools from one contract instead of
requiring users to hand-sync strings.

## Public API

Add one compiler-recognized helper to `std::cli`:

```muga
pub fn usage_for_required[T](program: String): String
```

Example:

```muga
fn usage_text(): String {
  string::concat_all([
    cli::usage_for_required[Command]("cli-tool"),
    "\n  --help  Show this help"
  ])
}
```

The helper stays pure. It does not read `env::args()`, print output, add a
`--help` branch, exit the process, inspect terminal width, or discover config
files. App-owned options such as `--help` and `--config` remain explicit
call-site text.

## Type Anchor Policy

`usage_for_required` returns `String`, so the expected return type cannot carry
`T`. The implementation adds explicit source-level call type arguments for this
helper:

```muga
cli::usage_for_required[Command]("cli-tool")
```

Rules:

- exactly one explicit type argument is required;
- the type argument must resolve to a concrete non-generic record;
- the record target must be supported by strict `cli::parse[T](args)`;
- generic records, unresolved type parameters, non-records, nested records,
  maps, opaque handles, one-payload enums, and other unsupported strict targets
  are rejected at type checking;
- `cli::parse[T](args)` keeps its existing expected-result inference policy and
  does not require explicit type arguments.

Diagnostics should name the helper and show a concrete repair:

```text
`cli::usage_for_required` requires one explicit record type argument, for example `cli::usage_for_required[Command]("cli-tool")`
```

## Source Call Type Arguments

This design introduces the minimal source syntax needed by the helper:

```muga
callee[TypeArg1, TypeArg2](arg1, arg2)
```

The parser should attach explicit type arguments to call expressions only when
the bracketed type argument list is immediately followed by `(`. Existing list
indexing, type expressions, and ordinary calls stay unchanged.

The checker accepts explicit call type arguments only on
`cli::usage_for_required`. User-defined generic functions continue using
inference until a broader explicit-generic-call design is justified.

## Usage Rendering Contract

`usage_for_required[T](program)` returns deterministic plain text:

```text
Usage: cli-tool [options]
  --target <String>  required; non-empty; Target resource name
  --count <Int>  required; range: 1..10; Number of items to process
  --action <Action>  required; values: Audit, Apply; Command action
  --dry-run[=<Bool>]  Preview changes without applying them
  --tag <String>  repeatable; aliases: --tags; Tag filter
  --owner <String>  Optional owner
```

Formatting rules:

- keep declaration order;
- render primary option names from `@cli(name)`, then `@json(rename)`, then the
  field name fallback;
- render aliases as `aliases: --name, --other`;
- omit `@cli(hidden)` fields from usage while still rejecting hidden required
  fields that cannot synthesize an absent value;
- mark required `String`, `Int`, and zero-payload enum fields with `required`;
- render `Bool` fields as `--name[=<Bool>]`;
- render `Option[T]` fields with their inner value type and no `required`
  marker;
- render `List[T]` fields with `repeatable`;
- render zero-payload enum accepted tags as `values: A, B`;
- render supported validation metadata compactly, such as `range: 1..10` and
  `non-empty`;
- append field help after generated metadata, separated by `; `;
- do not include app-owned `--help` text.

## Schema And Artifacts

Use the existing dedicated `CliSchema` representation:

- typing should derive the same schema as strict `cli::parse[T](args)`;
- typed HIR should record a distinct `CliUsageRequired` or equivalent operation
  with its `CliSchema`;
- MIR, bytecode, and `.mgb` persistence should carry enough schema data to
  render usage without provider source;
- `.mgi` interfaces should continue carrying public record CLI metadata and
  validation rules;
- explicit artifact roots and `run --built` should render exactly the same
  usage as source execution;
- malformed, stale, or unsupported CLI schema payloads remain hard artifact
  errors before execution.

Do not change the meaning of `cli::usage_for[T](program, defaults)` or
`cli::parse_or[T](args, defaults)`.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| `cli::usage_for_required[T](program)` with explicit call type argument | Smallest public helper for strict no-default usage; directly replaces manual help and reuses `CliSchema`. | Requires minimal source call type-argument syntax and helper-specific diagnostics. | Select |
| Overload `cli::usage_for[T](program)` by arity | Shorter name and mirrors the existing helper. | Muga does not otherwise rely on user-visible overloads; arity overloading would blur overlay/default and strict/no-default semantics. | Reject |
| Infer `T` from expected `String` result | Avoids new syntax. | Impossible because every usage helper returns `String`; the expected type carries no record target. | Reject |
| Require a fake default record | No new syntax. | Reintroduces placeholder defaults and cannot honestly mark required fields. | Reject |
| Add a schema witness or type-token value | Could generalize to other schema APIs. | Adds a new value abstraction before one concrete helper proves it is worth carrying. | Defer |
| Add record-level command metadata first | Useful later for descriptions and examples. | Field metadata is enough for the first strict usage helper; command metadata can layer on after generated usage exists. | Defer |
| Add combined short flags, attached values, subcommands, TOML, config discovery, or host APIs | Valuable long-term ergonomics. | Larger surfaces that should not block generated strict usage. | Defer |

## Non-Goals

This design does not add:

- generated `--help` branching or process exits;
- record-level command metadata;
- short flags or combined short flags;
- positional fields;
- subcommands;
- environment variables or config discovery;
- TOML/YAML/JSON5 loading;
- generic record schema instantiation;
- schema witness/type-token values;
- explicit call type arguments for ordinary user-defined generic functions;
- full client generation, process APIs, network APIs, streams, or broader host
  effects.

## Implementation Plan

1. Done: implement `cli::parse[T](args)` strict parsing and `CliSchema` reuse.
2. Done: add checked-in and generated strict CLI tool starters.
3. Done: add deterministic manual `--help` to the strict sample and generated
   starter.
4. Done: audit manual help adoption and fold the selected type-anchor policy
   into this no-default usage helper design.
5. Done: write this no-default usage helper design and release-readiness
   coverage.
6. Done: implement `cli::usage_for_required[T](program)` with minimal explicit
   call type arguments, source/artifact/`run --built` coverage, and replacement
   of manual strict CLI usage text in the sample and generated template.
7. Done: audit strict CLI no-default usage helper adoption and fold the
   selected command-metadata direction into `cli-command-metadata.md`.
8. Done: implement record-level CLI command metadata from
(cli-command-metadata.md).
   Done: audit CLI command metadata adoption and keep the selected short-option
   direction in [cli-short-option-metadata.md](cli-short-option-metadata.md). Done: design CLI short option metadata in [cli-short-option-metadata.md](cli-short-option-metadata.md). Done: implement CLI short option metadata. Done: audit CLI short option metadata adoption. Done: design CLI positional field metadata in [cli-positional-field-metadata.md](cli-positional-field-metadata.md). Done: implement CLI positional field metadata. Done: audit CLI positional field metadata adoption. Done: design built-in CLI help policy in [cli-built-in-help-policy.md](cli-built-in-help-policy.md). Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata.
9. Later: revisit combined short flags, attached values, subcommands, config discovery, TOML, schema
   witnesses, and broader explicit generic calls after command summaries are
   real.
