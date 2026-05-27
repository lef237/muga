Status: parse-integrated CLI help workflow implemented

# Parse-Integrated CLI Help Workflow

Muga now has schema-backed strict parsing, overlay parsing, generated usage,
generated help, positional operands, short options, aliases, command summaries,
validation markers, hidden fields, source/artifact parity, and generated
starter adoption. The remaining CLI starter boilerplate is the app-owned branch
that asks for help before parsing.

This slice implements a typed request workflow that combines help detection
with parsing while keeping printing, return values, and process status explicit
in ordinary Muga code.

## Goals

Short-Term Goal: remove the repeated `if cli::help_requested(args)` branch from
generated strict and config CLIs without changing `cli::parse[T]`,
`cli::parse_or[T]`, `cli::help_for[T]`, or `cli::help_for_required[T]`.

Medium-Term Goal: keep help text, parsed command values, recoverable CLI
errors, schema lowering, source/build/`run --built` execution, and templates
under one typed request contract.

Long-Term Goal: prepare Muga CLIs for subcommands and shell completion
generation by making "help vs parsed command" an explicit data shape rather
than starter-specific control-flow glue.

Final Goal: make practical Muga command-line tools concise enough to recommend
while preserving the language's explicit-effect boundary.

## Public API

Add a generic request enum and two compiler-owned helpers to `std::cli`:

```muga
pub enum Request[T] {
  Help(String)
  Parsed(T)
}

pub fn parse_request[T](args: List[String], program: String): Result[Request[T], Error]
pub fn parse_request_or[T](args: List[String], program: String, defaults: T): Result[Request[T], Error]
```

`parse_request[T](args, program)` is the strict no-default workflow. It requires
one explicit concrete record type argument, like `help_for_required[T]` and
`usage_for_required[T]`, because the helper must render help even when parsing
does not run and has no defaults value to infer from.

`parse_request_or[T](args, program, defaults)` is the overlay/config workflow.
It infers `T` from `defaults`, like `parse_or[T](args, defaults)` and
`help_for[T](program, defaults)`.

The argument order follows parsing, not help rendering: `args` comes first
because this is a parse workflow, and `program` comes second because it is used
only when the request is help text.

Example strict starter shape:

```muga
fn main(): Result[String, String] {
  args = env::args()
  request = try result::map_err(
    cli::parse_request[Root](args, "cli-tool"),
    cli_error_message
  )

  match request {
    cli::Request::Help(help) => {
      printed = println(help)
      Result::Ok(help)
    }
    cli::Request::Parsed(root) => {
      rendered = render_root(root)
      printed = println(string::concat_all(["cli-tool ", rendered]))
      Result::Ok(rendered)
    }
  }
}
```

Example config overlay shape:

```muga
request = try result::map_err(
  cli::parse_request_or(settings_args(args), "config-app", default_settings()),
  cli_error_message
)
```

Generated config apps can still resolve a config file path before this call;
TOML/config discovery remains a separate policy.

## Behavior

For both helpers:

1. Call `cli::help_requested(args)` first.
2. If help was requested, return `Result::Ok(cli::Request::Help(generated_help))`.
3. If help was not requested, parse normally.
4. On parse success, return `Result::Ok(cli::Request::Parsed(value))`.
5. On parse failure, return `Result::Err(error)`.

Help therefore wins over unrelated parse errors before `--`, matching common
CLI behavior. After `--`, help tokens are ordinary positional input because
`help_requested(args)` already stops scanning at `--`.

This design keeps existing lower-level helpers:

- `cli::parse[T](args)` remains strict parsing only;
- `cli::parse_or[T](args, defaults)` remains overlay parsing only;
- `cli::help_requested(args)` remains a pure low-level predicate;
- `cli::help_for[T](program, defaults)` and
  `cli::help_for_required[T](program)` remain explicit generated help helpers
  for custom workflows.

## Schema And Artifacts

`parse_request` and `parse_request_or` are compiler-owned schema helpers.

Implementation should:

- expose `Request[T]`, `parse_request`, and `parse_request_or` in the virtual
  `std::cli` package;
- recognize both helpers during typing and store distinct schema call entries;
- allow explicit call type arguments for `parse_request[T]` in addition to
  `usage_for_required[T]` and `help_for_required[T]`;
- reuse strict schema lowering and help-name conflict diagnostics from
  `help_for_required` for `parse_request`;
- reuse overlay/default schema lowering and help-name conflict diagnostics from
  `help_for` for `parse_request_or`;
- lower to typed HIR, MIR, bytecode, and `.mgb` implementation artifacts with
  enough schema payload to run without provider source;
- keep malformed or stale request schema payloads as artifact load errors;
- keep source execution, emitted artifacts, and `run --built` behavior
  identical.

## Diagnostics

The implementation should add or reuse diagnostics for:

- `parse_request` without exactly one explicit concrete record type argument;
- `parse_request` with a generic, non-record, unsupported, or unresolved target;
- `parse_request_or` with unsupported defaults target;
- either helper targeting a schema that conflicts with built-in `--help` or
  `-h`;
- malformed request schema payloads in artifacts.

Diagnostic text should name the selected helper, for example:

```text
`cli::parse_request` requires exactly 1 explicit record type argument but found 0
```

and:

```text
`cli::parse_request_or` reserves `--help` and `-h`; field `host` uses `@cli(short: "h")`
```

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| `Request[T]` plus `parse_request[T]` / `parse_request_or[T]` | Makes help vs parsed command an explicit value, preserves app-owned printing/status, supports strict and overlay workflows, and keeps low-level helpers available. | Adds a generic enum plus two compiler-owned helpers and artifact schema payloads. | Select |
| `parse_with_help[T]` / `parse_or_with_help[T]` names | Very explicit. | Longer and less aligned with the request value apps will match. `parse_request` reads as a workflow, not just a parser flag. | Reject |
| Return `Result[T, cli::Error]` where help is `ErrorKind::Help` | Minimal new public types. | Help is not an error and would make ordinary `try` flows treat success help as failure. | Reject |
| Return `Result[T, cli::RequestError]` with help in the error branch | Avoids a separate success enum. | Still puts help on the error path and complicates app error mapping. | Reject |
| Runtime auto-print and exit | Smallest generated app code. | Hides effects and status policy before Muga has a stable process exit/status API. | Reject |
| Only improve templates using low-level helpers | No new compiler/runtime surface. | Leaves a repeated branch in every generated app and does not give tooling a typed request contract. | Reject |
| Design subcommands first | Important for larger tools. | Subcommands need per-command help and dispatch; they should build on a settled request shape. | Defer |
| Combined short flags, attached short values, shell completion generation, TOML/config discovery, full client generation, generic encoding/decoding, broader validators, or host-effect APIs | Valuable follow-up surfaces. | Orthogonal or larger than the immediate request workflow gap. | Defer |

## Non-Goals

This design does not add:

- runtime-owned printing or process exits;
- process status APIs;
- custom help flag names;
- subcommands or nested command dispatch;
- shell completion generation;
- combined short flags such as `-abc`;
- attached short values such as `-ofile`;
- TOML/config discovery automation;
- full client generation, generic encoding/decoding, broader validators, or
  host-effect APIs.

## Implementation Plan

1. Done: implement schema-backed parsing, usage, help, metadata, artifacts, and
   starter adoption.
2. Done: audit built-in CLI help helper adoption in
   [post-built-in-cli-help-helper-adoption-gap-selection.md](post-built-in-cli-help-helper-adoption-gap-selection.md).
3. Done: design parse-integrated CLI help workflow here.
4. Done: implement `cli::Request[T]`, `cli::parse_request[T]`, and
   `cli::parse_request_or[T]` with source/artifact/`run --built` coverage and
   generated strict/config template adoption.
5. Done: audit parse-integrated CLI help workflow adoption in
   [post-parse-integrated-cli-help-workflow-adoption-gap-selection.md](post-parse-integrated-cli-help-workflow-adoption-gap-selection.md).
6. Done: design compact CLI short option syntax in
   [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md).
7. Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata.
8. Later: revisit subcommands, shell completion generation, TOML/config
   discovery automation, full client generation, generic encoding/decoding,
   broader validators, and host-effect APIs after compact short option behavior
   is specified.
