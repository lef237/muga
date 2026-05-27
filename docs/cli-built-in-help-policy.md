Status: built-in CLI help helpers implemented

# Built-In CLI Help Policy

Muga can now describe practical command-line records with long names, aliases,
short options, hidden fields, command summaries, required strict parsing,
generated usage, and positional operands. The remaining starter boilerplate is
the explicit help branch:

```muga
if cli::has_flag(args, "help") or cli::has_short_flag(args, "h") {
  usage = usage_text()
  printed = println(usage)
  return Result::Ok(usage)
}
```

This design makes help detection and generated help text a schema-owned
standard-library policy while preserving the current boundary: applications
still decide whether to print, return, or map the help text to a process status.

## Goals

Short-Term Goal: remove duplicated `--help` / `-h` checks and manually appended
help rows from generated strict CLI tools without changing `cli::parse[T]` or
adding runtime-owned printing/exiting.

Medium-Term Goal: keep usage text, help text, command summaries, option names,
short options, aliases, hidden fields, validation markers, positional operands,
interfaces, and artifacts under one documented `CliSchema` contract.

Long-Term Goal: make Muga CLIs publishable from a small typed command record
with predictable generated help before adding subcommands, shell completion
generation, config discovery, richer process status APIs, or client generation.

Final Goal: make practical Muga command-line tools feel small to write, easy to
audit, and stable enough for real users to install and recommend.

## Public API

Add three `std::cli` helpers:

```muga
pub fn help_requested(args: List[String]): Bool
pub fn help_for[T](program: String, defaults: T): String
pub fn help_for_required[T](program: String): String
```

`help_requested(args)` is a pure token scan. It returns `true` for exact
`--help` or `-h` tokens that appear before `--`. `--` stops help detection, so
`["--", "--help"]` is treated as positional input rather than a help request.
Attached or valued forms such as `--help=true`, `--help=false`, `-h=true`, or
`-hvalue` are not help requests in the first slice.

`help_for[T](program, defaults)` returns generated help for overlay/config
records using the same schema target and defaults as `usage_for[T](program,
defaults)`. `help_for_required[T](program)` returns generated help for strict
no-default records using the same explicit record type anchor policy as
`usage_for_required[T](program)`.

Example strict CLI shape:

```muga
fn main(): Result[String, String] {
  args = env::args()
  if cli::help_requested(args) {
    help = cli::help_for_required[Command]("cli-tool")
    printed = println(help)
    return Result::Ok(help)
  }

  parsed: Result[Command, cli::Error] = cli::parse(args)
  command = try result::map_err(parsed, cli_error_message)
  rendered = render(command)
  printed = println(rendered)
  Result::Ok(rendered)
}
```

The helper names intentionally distinguish generated help from generated usage:

- `usage_for` / `usage_for_required` describe the parser contract and do not
  claim that `--help` is accepted by the parser;
- `help_for` / `help_for_required` describe the user-facing help contract and
  include the built-in help row.

## Help Rendering

Generated help should preserve the existing usage rendering for command
summaries, arguments, options, aliases, validation annotations, defaults, and
value metavars, then append the built-in help row as the final visible option:

```text
Usage: cli-tool [options] <target>

Manage target resources

Arguments:
  <target>  required; Target resource name

Options:
  -n, --count <Int>  required; range: 1..10; Number of items to process
  -h, --help  Show this help
```

Rendering rules:

- the help row is always visible for `help_for` and `help_for_required`;
- the help row appears after schema options, even when the record has hidden
  options;
- if a command has no schema options, the help output still has an `Options:`
  section containing `-h, --help  Show this help`;
- hidden fields remain hidden and do not affect the visible help row;
- positional operands continue to render in the usage line and `Arguments:`
  section;
- generated usage helpers remain unchanged unless explicitly called through a
  help helper.

## Reserved Help Names

Built-in help is opt-in. A schema rendered by `help_for` or
`help_for_required` reserves `--help` and `-h`.

The first implementation should reject a help helper target when any field uses
one of these spellings as a primary name, alias, or short option, including
hidden fields:

- default field name `help`;
- `@cli(name: "help")`;
- `@cli(alias: "help")`;
- `@cli(short: "h")`.

Plain `cli::parse[T]`, `cli::parse_or[T]`, `usage_for`, `usage_for_required`,
`has_flag`, and `has_short_flag` keep their current behavior. Projects that need
a custom help policy can keep using the lower-level helpers.

## Parser Behavior

`cli::parse[T](args)` and `cli::parse_or[T](args, defaults)` do not treat help
specially in this slice. Applications should check `cli::help_requested(args)`
before strict or overlay parsing when they opt into built-in help.

This keeps all process-facing behavior app-owned:

- the runtime does not print help automatically;
- the runtime does not exit the program automatically;
- help is not represented as a `cli::Error`;
- help is not silently accepted after `--`;
- parse errors keep their existing `UnknownArgument`, `MissingArgument`,
  `MissingValue`, `InvalidValue`, `Validation`, and `UnsupportedTarget`
  semantics.

## Schema And Artifacts

`help_for` should reuse the same schema lowering as `usage_for`.
`help_for_required` should reuse the same explicit type-argument schema lowering
as `usage_for_required`.

Implementation behavior:

- the virtual `std::cli` package exposes the three helpers;
- typing recognizes `help_for` and `help_for_required` as compiler-owned schema
  helpers and stores distinct typed schema call entries;
- `help_for_required` requires exactly one explicit concrete non-generic record
  type argument, like `usage_for_required`;
- `help_for` infers the target record from `defaults`, like `usage_for`;
- typed HIR, MIR, bytecode, and `.mgb` implementation artifacts carry enough
  schema data to render help without provider source;
- package signatures and `.mgi` interfaces keep carrying the underlying public
  record CLI metadata;
- source execution and `run --built` render identical help;
- malformed or stale schema payloads remain artifact errors before execution.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| `cli::help_requested` plus generated `help_for` / `help_for_required` | Removes duplicated help flag scans and manual help rows while preserving app-owned printing/status decisions. Reuses existing usage/schema lowering with a small public surface. | Requires an opt-in reservation policy for `--help` and `-h`, plus two generated text helpers. | Select |
| Change `usage_for` / `usage_for_required` to include help automatically | Fewer helper names. | Breaks the current distinction between parser contract usage and app-owned help, and may make existing usage tests/users display a flag the parser does not accept by itself. | Reject |
| Parse-integrated `cli::parse_with_help[T]` returning a `Help` / `Parsed` enum | Stronger workflow API and one call site for help plus parsing. | Hides more control flow, requires a new generic result enum, and should wait until the smaller help policy is proven. | Defer |
| Runtime auto-print and exit on `--help` | Familiar in many host CLIs. | Introduces hidden effects and process-status policy before Muga has a general process exit API. | Reject |
| Treat help as `cli::ErrorKind::Help` | Avoids a new output helper. | Help is not an error; it would complicate ordinary error handling and `try` flows. | Reject |
| Reserve `-h` globally in all CLI schemas | Prevents future ambiguity. | Breaking and unnecessary for projects that intentionally use `-h` as a normal short option without opting into built-in help. | Reject |
| Keep only manual `has_flag` / `has_short_flag` help branches | No implementation work. | Leaves the next visible starter-template boilerplate unsolved. | Reject |
| Subcommands, shell completion generation, TOML/config discovery, full client generation, broader validators, generic encoding/decoding, or host-effect APIs | Important later surfaces. | Larger or orthogonal to the immediate CLI help ergonomics gap. | Defer |

## Diagnostics

The implementation should add targeted diagnostics for:

- `help_for_required` without exactly one explicit concrete record type
  argument;
- `help_for_required` with a generic, non-record, unsupported, or unresolved
  target;
- `help_for` with an unsupported defaults target;
- a help helper target whose field name, CLI name, alias, or short option
  conflicts with `--help` or `-h`;
- malformed help schema payloads in artifacts.

Diagnostics should name the chosen helper and explain the opt-in reservation:

```text
`cli::help_for_required` reserves `--help` and `-h`; field `host` uses `@cli(short: "h")`
```

## Non-Goals

This design does not add:

- parse-integrated help result enums;
- runtime-owned printing;
- process exit or process status APIs;
- custom help flag names;
- long help topics, examples, manpage output, or terminal-width wrapping;
- subcommands or nested command help;
- combined short flags such as `-abc`;
- attached short values such as `-ofile`;
- shell completion generation;
- TOML/config discovery automation;
- full client generation, generic encoding/decoding, broader validators, or
  host-effect APIs.

## Implementation Plan

1. Done: implement field-level CLI names, aliases, help, hidden fields, command
   summaries, short options, strict parsing, generated usage, and positional
   operands.
2. Done: audit CLI positional metadata adoption in
   [post-cli-positional-field-metadata-adoption-gap-selection.md](post-cli-positional-field-metadata-adoption-gap-selection.md).
3. Done: design built-in CLI help policy here.
4. Done: implement `cli::help_requested`, `cli::help_for`,
   `cli::help_for_required`, help-name conflict diagnostics,
   source/artifact/`run --built` coverage, and strict/config template adoption.
5. Done: audit built-in CLI help helper adoption in
   [post-built-in-cli-help-helper-adoption-gap-selection.md](post-built-in-cli-help-helper-adoption-gap-selection.md).
6. Done: design parse-integrated CLI help workflow in
   [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md).
7. Done: implement parse-integrated CLI help workflow.
8. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata.
9. Later: revisit custom help labels,
   subcommands, completion generation, TOML/config discovery, full client
   generation, generic encoding/decoding, broader validators, and host-effect
   APIs after the simple opt-in help policy is implemented and audited.
