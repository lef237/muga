Status: CLI short option metadata implemented

# CLI Short Option Metadata

Muga now has typed CLI schemas, field-level long names and aliases, strict
required-option parsing, generated usage, and record-level command summaries.
The next ergonomic gap is short option spelling. Real command-line tools
commonly accept `-h`, `-t`, or `-n` beside long options, but Muga records can
currently express only long names and long aliases.

This slice adds the smallest short-option metadata surface:

```muga
@cli(about: "Run typed strict CLI commands")
pub record Command {
  @cli(short: "t", help: "Target resource name")
  target: String

  @cli(name: "dry-run", short: "d", help: "Preview changes")
  dry_run: Bool
}
```

## Goals

Short-Term Goal: let generated strict tools and config apps accept familiar
single-letter options while preserving deterministic usage output.

Medium-Term Goal: keep long names, aliases, short names, hidden fields, help
text, validation, command summaries, interfaces, artifacts, and future
completion metadata in one `CliSchema`.

Long-Term Goal: make typed command records the practical source of truth for
parser behavior, generated help, validation, templates, future completions, and
future subcommand metadata.

## Public Syntax

Allow `short: "x"` inside a field-level `@cli(...)` attribute:

```muga
pub record Settings {
  @cli(name: "host", short: "H", help: "Server host")
  host: String

  @cli(short: "v", help: "Enable verbose logging")
  verbose: Bool
}
```

Rules:

- `short` is field-level only; record-level `@cli(...)` continues to support
  only `about`;
- a field may specify at most one `short`;
- the value must be a one-character ASCII alphabetic string without a leading
  dash;
- digits, punctuation, empty strings, tabs, carriage returns, and newlines are
  rejected;
- short names are case-sensitive;
- hidden fields may have short names because they remain parseable;
- duplicate short names in one CLI schema are rejected, including duplicates on
  hidden fields;
- `h` is not globally reserved. Apps and templates may still preempt `-h` for
  help before calling `cli::parse[T]` or `cli::parse_or[T]`.

Long option duplicate checks remain unchanged: primary long names and long
aliases share one accepted-name set, while short names use a separate
single-character namespace.

## Parser Behavior

`cli::parse[T](args)` and `cli::parse_or[T](args, defaults)` should accept:

- long forms exactly as today: `--target value` and `--target=value`;
- short separated values: `-t value`;
- short inline values with equals: `-t=value`;
- bare short `Bool` flags: `-d` means `true`;
- explicit short `Bool` values: `-d=false` and `-d false`;
- short names for scalar, enum, `Option` scalar, and scalar-list fields when
  the corresponding long field type is supported.

The short-metadata slice itself did not support:

- combined short flags such as `-abc`;
- attached short values such as `-ofile`;
- short aliases separate from the primary short name;
- short names for positional fields or subcommands.

Combined bool flags and attached short values are now implemented as a runtime
parser follow-up in
[compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md).

Separated values should avoid swallowing likely option markers. If a short
option needs a value and the next token is another recognized option marker,
the parser should report `MissingValue` for the first option. Values that begin
with a dash can still be passed with the explicit equals form, for example
`-n=-1` or `-s=-literal`.

Runtime error `argument` strings should use the spelling supplied by the user:
`-t` for a short option and `--target` for a long option. Missing required
fields in strict parsing may continue to report the primary long option name,
because that is the stable canonical field spelling.

## Usage Rendering

Generated usage should render short names in the primary option cell:

```text
Usage: cli-tool [options]

Run typed strict CLI commands
  -t, --target <String>  required; Target resource name
  -d, --dry-run[=<Bool>]  Preview changes
```

Formatting rules:

- fields remain ordered by record declaration order;
- fields without short names keep the existing `--long` rendering;
- aliases remain shown as long-option annotations such as
  `aliases: --old-name`;
- `repeatable`, aliases, defaults, required markers, validation markers, and
  help text keep their existing ordering after the option cell;
- hidden fields are still omitted from usage output.

## App-Owned Help

This design does not add built-in help branching, exits, or command framework
behavior. Generated starters should continue to branch explicitly before
calling the typed parser.

To keep `-h` practical without a command framework, the implementation may add
a pure helper:

```muga
cli::has_short_flag(args, "h")
```

The helper should match exact `-h` tokens before `--`, reject no values at
runtime because it is pure source code over `List[String]`, and stay separate
from schema parsing. Generated templates can then use:

```muga
cli::has_flag(args, "help") || cli::has_short_flag(args, "h")
```

## Schema And Artifacts

`CliFieldSchema` should carry `short: Option<Symbol>`.

Implementation behavior:

- parser and formatter preserve field-level `@cli(short: "...")`;
- typing stores short metadata on record fields and validates duplicate short
  names independently from long names and aliases;
- package signatures and `.mgi` interfaces persist short metadata for public
  records so downstream packages can lower CLI schemas without provider source;
- typed HIR, MIR, bytecode, `.mgb` implementation artifacts, and runtime parser
  and usage rendering preserve short names;
- old interfaces and old schema payloads without short metadata remain readable
  as `short = None`;
- malformed new schema/interface payloads reject before execution.

Prefer an additive schema trailer for short metadata rather than inserting a
new fixed token into the middle of the existing recursive field payload. That
keeps older payloads readable and avoids ambiguity with nested CLI value schema
tokens.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| Field-level `@cli(short: "x")` | Familiar CLI ergonomics; small extension to proven field metadata; works for config and strict records. | Requires parser, usage, duplicate checks, and artifact/interface persistence. | Select |
| Long aliases only, for example `@cli(alias: "t")` | No new metadata key. | `--t` is not a real short option and would confuse usage/help expectations. | Reject |
| Add only `cli::has_short_flag` | Helps `-h` branches. | Does not let typed schemas parse short options. | Reject as standalone |
| Support combined flags immediately | More Unix-like for bool groups. | Requires disambiguation with value-taking options and better error policy. | Defer |
| Support attached values immediately | Familiar for `-ofile`. | Ambiguous with combined flags and less necessary than `-o value` / `-o=value`. | Defer |
| Reserve `-h` globally | Avoids conflicts with generated help. | Prevents legitimate schema fields and introduces app-framework policy into metadata. | Reject |
| Positionals or subcommands first | Important for larger CLIs. | Larger parsing and dispatch surface; short options are a smaller polish slice. | Defer |

## Non-Goals

This design does not add:

- built-in `--help` / `-h` branching, process exits, or exit codes;
- combined short flags such as `-abc`;
- attached values such as `-ofile`;
- positionals, variadic arguments, or subcommands;
- environment variable bindings, TOML, config discovery automation, shell
  completion generation, full client generation, or host-effect APIs.

## Implementation Plan

1. Done: field-level `@cli(name, alias, help, hidden)` metadata and dedicated
   `CliSchema`.
2. Done: strict `cli::parse[T](args)`.
3. Done: generated strict usage with
   `cli::usage_for_required[T](program)`.
4. Done: record-level `@cli(about: "...")` command summaries.
5. Done: audit CLI command metadata adoption and select field-level short
   option metadata as the next CLI ergonomics slice.
6. Done: implement `@cli(short: "...")` across parser validation,
   formatting, typing, package signatures/interfaces, `CliSchema`, artifacts,
   runtime parsing and usage rendering, `cli::has_short_flag`, starter/sample
   adoption, tests, and docs.
7. Done: audit CLI short option metadata adoption in
   [post-cli-short-option-metadata-adoption-gap-selection.md](post-cli-short-option-metadata-adoption-gap-selection.md).
8. Done: design CLI positional field metadata in
   [cli-positional-field-metadata.md](cli-positional-field-metadata.md).
9. Done: implement CLI positional field metadata.
10. Done: audit CLI positional field metadata adoption.
11. Done: design built-in CLI help policy in
   [cli-built-in-help-policy.md](cli-built-in-help-policy.md).
12. Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata.
13. Later: revisit combined short flags, attached values, built-in help
   branching, subcommands, TOML, config discovery automation,
   shell completion generation, full client generation, generic
   encoding/decoding, broader validators, or host-effect APIs.
