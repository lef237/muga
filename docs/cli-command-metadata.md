Status: CLI command metadata implemented

# CLI Command Metadata

The first CLI schema slices made typed option parsing and generated usage real
for both config/default overlays and strict required-option tools. Field-level
metadata can now name options, aliases, hidden fields, and per-field help, but a
command record cannot yet describe the command itself.

This design and implementation add the smallest record-level CLI metadata
slice:

```muga
@cli(about: "Inspect and apply changes to a target resource")
pub record Command {
  @cli(help: "Target resource name")
  target: String
}
```

## Goals

Short-Term Goal: let generated CLI usage include a command summary without
hand-written app prose.

Medium-Term Goal: keep `cli::usage_for[T]` and
`cli::usage_for_required[T]` aligned on one `CliSchema` metadata payload across
source execution, explicit artifact roots, and `run --built`.

Long-Term Goal: make typed command records the source of truth for command
description, options, validation, artifacts, future shell completions, and
future subcommand metadata.

## Public Syntax

Allow `@cli(about: "...")` directly before a record declaration:

```muga
@cli(about: "Run typed strict CLI commands")
pub record Command {
  target: String
}
```

Rules:

- record declarations may have at most one `@cli` attribute;
- the only record-level argument in this slice is `about`;
- `about` must be a non-empty string literal;
- tabs, carriage returns, and newlines are rejected;
- `@cli(name)`, `@cli(alias)`, `@cli(help)`, and `@cli(hidden)` remain
  field-level only;
- field-level `@cli(...)` semantics are unchanged.

## Usage Rendering

When command `about` metadata is present, usage helpers render it immediately
after the `Usage:` line and before options:

```text
Usage: cli-tool [options]

Run typed strict CLI commands
  --target <String>  required; Target resource name
```

For overlay/default usage:

```text
Usage: config-app [options]

Configure a generated service

Options:
  --name <String>  default: "Muga"  Application display name
```

Formatting remains deterministic:

- no command metadata means existing output stays unchanged;
- `about` is rendered exactly as a single line;
- field order, aliases, defaults, validation markers, and help text keep their
  existing ordering;
- app-owned lines such as `--help` or `--config` remain explicit call-site
  additions.

## Schema And Artifacts

`CliSchema` now carries `about: Option<Symbol>`.

Implementation behavior:

- parser and formatter preserve record-level `@cli(about: "...")`;
- typing stores command metadata on record definitions and lowers it into every
  CLI schema built for that record;
- package signatures and `.mgi` interfaces persist metadata for public records
  so downstream packages can render usage without provider source;
- typed HIR, MIR, bytecode, `.mgb` implementation artifacts, and runtime usage
  rendering preserve the metadata;
- old interfaces and old schema payloads without command metadata remain
  readable as `about = None`;
- malformed new schema payloads reject before execution.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| Record-level `@cli(about: "...")` | Small, visible improvement to generated usage; aligns with the record-driven CLI contract and prepares future subcommands. | Requires metadata persistence through interfaces and artifacts. | Select |
| Reuse source doc comments as command descriptions | Avoids another attribute. | Public docs and CLI usage have different compatibility expectations; doc comments may be long or formatted for API docs. | Reject |
| Add `about` to `usage_for(program, defaults, about)` arguments | Avoids schema metadata. | Reintroduces call-site prose duplication and cannot survive artifact-only schema use. | Reject |
| Add examples, footer, categories, and headings now | Richer help output. | Larger compatibility surface before a single command summary is proven. | Defer |
| Add short flags or subcommands first | Familiar CLI ergonomics. | Needs command metadata and usage layout policy; larger than a pure metadata slice. | Defer |

## Non-Goals

This design does not add:

- built-in `--help` branching, process exits, or exit codes;
- command names separate from the `program` argument;
- examples, footers, categories, groups, or custom option headings;
- short flags, combined flags, positionals, or subcommands;
- environment variables, config discovery, TOML/YAML/JSON5 loading;
- shell-completion generation from command metadata;
- full client generation, process APIs, network APIs, streams, or broader host
  effects.

## Implementation Plan

1. Done: field-level `@cli(name, alias, help, hidden)` metadata and dedicated
   `CliSchema`.
2. Done: strict `cli::parse[T](args)` and generated strict usage with
   `cli::usage_for_required[T](program)`.
3. Done: audit generated strict usage adoption and select record-level command
   metadata as the next smallest CLI help gap.
4. Done: implement `@cli(about: "...")` across parser validation, typing,
   interfaces, `CliSchema`, artifacts, runtime usage rendering, samples,
   templates, tests, and docs.
5. Done: audit CLI command metadata adoption and select field-level short
   option metadata as the next CLI ergonomics slice.
6. Done: design CLI short option metadata in [cli-short-option-metadata.md](cli-short-option-metadata.md). Done: implement CLI short option metadata. Done: audit CLI short option metadata adoption. Done: design CLI positional field metadata in [cli-positional-field-metadata.md](cli-positional-field-metadata.md). Done: implement CLI positional field metadata. Done: audit CLI positional field metadata adoption. Done: design built-in CLI help policy in [cli-built-in-help-policy.md](cli-built-in-help-policy.md). Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata.
7. Later: revisit combined short flags, attached values, subcommands, TOML, config discovery automation,
   full client generation, generic encoding/decoding, broader validators, or
   host-effect APIs after command summaries are audited.
