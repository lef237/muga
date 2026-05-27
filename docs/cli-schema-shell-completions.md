Status: CLI schema-backed shell completion implementation landed for generated
cli-tool workflows; shell-agnostic JSON completion specs, richer nested command
traversal, and static file/directory value sources are implemented.

# CLI Schema Shell Completions

Wrapper records, command enums, field metadata, compact short options, and
positional metadata now give Muga enough CLI schema information to generate
completion scripts for Muga-authored tools. This design keeps those completions
separate from `muga shell-completions`, which remains the static completion
script for the `muga` developer tool itself.

## Goals

Short-Term Goal: define one command-line surface that can render deterministic
completion scripts from a concrete strict CLI schema without changing language
syntax or the `std::cli` runtime API.

Medium-Term Goal: make generated `cli-tool` projects installable as conventional
developer CLIs with command, alias, root option, leaf option, enum value, and
positional completion support.

Long-Term Goal: let one `CliSchema` drive parsing, help, diagnostics, artifacts,
generated project templates, and shell completions across source and built
artifact workflows.

Final Goal: make Muga tools feel publishable by default, so users can generate a
typed CLI app, build it, and ship completion scripts without hand-maintaining
parallel shell metadata.

## Selected User Surface

The implemented app/tool completion command is:

```bash
muga cli-completions <bash|zsh|fish> --program <name> --type <Type> [--package <package>] [--artifact-root <dir>|--built] <source-file>
```

For the generated strict CLI starter, the expected invocation is:

```bash
muga cli-completions fish --program cli-tool --type Root samples/projects/cli_tool/src/main/main.muga
```

The command writes a shell script to stdout and leaves stderr empty on success.
It loads and type-checks the entry the same way existing source-backed tooling
commands do, or consumes existing artifacts through `--artifact-root` / `--built`
when requested. `--type` is required so the command does not guess between
multiple CLI records in one package. `--package` selects a non-main package
schema when the requested CLI type lives outside the entry package.

## Implemented Slice

The first implementation adds `muga cli-completions` as a CLI mode separate
from `muga shell-completions`, validates `bash`, `zsh`, and `fish` targets, and
renders deterministic scripts to stdout. Diagnostics and usage errors stay on
stderr.

Schema loading works for:

- source-backed checking of the entry file;
- explicit artifact roots through `--artifact-root <dir>`;
- default built artifacts through `--built`;
- explicit package/type lookup with `--package` and `--type`.

The generator reconstructs the selected package signature's CLI schema into the
same `CliSchema` data model used by parsing and help. The first shell renderers
cover the generated strict `cli-tool` workflow:

- wrapper root/global options before command selection;
- command names plus command aliases such as `run` / `r` and `inspect` / `i`;
- leaf command options, long aliases, short options, and help flags;
- visible zero-payload enum values for option values;
- Bool option value candidates as `true` / `false`;
- static file/directory sources from `@cli(value_source: "...")` for option
  values;
- omission of hidden options and commands from generated candidates.

Nested command enum payload data is preserved in the completion model and the
shell renderers now traverse command scopes recursively. A nested shape such as
`tool admin user --role ...` selects the root scope, then the `admin` scope,
then the `user` leaf scope so bash, zsh, and fish can offer the leaf options
and value candidates instead of stopping at the first command token.

## Schema Rules

The implementation accepts the same strict target shapes as
`cli::help_for_required[T]`:

- plain strict records with option and positional fields;
- command enum schemas;
- wrapper records with one `@cli(subcommand)` field;
- nested command enum payloads.

It rejects unsupported/generic targets with the existing
`UnsupportedTarget`-style diagnostic language rather than inventing a new schema
model. Hidden fields and hidden commands must be omitted from generated
completion candidates.

For the checked-in strict starter, `Root` is the completion type anchor and
contains a global profile option plus a `Command` enum selected through
`@cli(subcommand)`. The same schema also renders root help with `Global Options:`,
so shell completion should use that existing option scope instead of
reconstructing CLI metadata from source text.

## Completion Semantics

Wrapper schema completion follows parse scope:

1. Before the command token, offer root/global options, `-h` / `--help`, command
   names, and command aliases.
2. Once a command token or alias is present, stop offering root/global options
   and offer only the selected leaf schema's options, enum values, and
   positional fallback behavior.
3. For nested command enums, repeat the command-selection step recursively.
4. For plain record schemas, complete the record's visible options and
   positionals directly.

Field completion policy:

- long options use `--name`; aliases use `--alias`;
- short options use `-x`;
- Bool options complete the flag form and may complete attached `true` / `false`
  values for shells that support value completion;
- enum-valued options complete visible zero-payload enum variant names;
- repeated options remain offerable because shell state tracking is intentionally
  best-effort in the first implementation;
- string/int/list/option positionals fall back to file completion unless
  `@cli(value_source: "directory")` selects directory fallback;
- String, Option[String], and List[String] option values can use
  `@cli(value_source: "file")` or `@cli(value_source: "directory")` so bash,
  zsh, fish, and JSON consumers have an explicit path source;
- fields whose types are unsupported by the completion schema are omitted when
  they have no explicit `@cli(...)` metadata, matching `cli::parse_or[T]`
  default-preservation behavior for config records;
- compact short clusters such as `-dc3` are parsed by Muga but are not a first
  completion goal beyond exposing individual short options.

## Artifact And Output Boundaries

The generator should reuse `CliSchema` payloads already carried through typed
HIR, MIR, bytecode, and `.mgb` implementation artifacts. It must not execute the
target program and must not call user `main`.

Shell output is deterministic text. The implementation renders bash, zsh, and
fish because `muga shell-completions` already supports that shell set.
Shell-agnostic completion-spec output is now available through
`muga cli-completions --format json`; its contract is documented in
[cli-completion-json-spec.md](cli-completion-json-spec.md). Shell scripts remain
renderers over the same `CliSchema` facts rather than the only machine-readable
contract.

The command must not install or source the generated script. The non-mutating
installer integration path is `muga emit-cli-completions --format json --output-dir ...`,
documented in
[cli-completion-installer-integration.md](cli-completion-installer-integration.md);
users, package managers, and future installers still decide where to place the
generated files.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| New `muga cli-completions <shell> --program <name> --type <Type> <source>` command | Keeps app completion generation separate from static `muga shell-completions`; gives immediate installable shell output; reuses existing source/artifact entry loading and explicit type anchors. | Adds a new CLI mode and shell rendering code. Needs careful escaping and tests per shell. | Select: separate command selected |
| Extend `muga shell-completions` to inspect source when extra flags are present | Reuses a known command name. | Breaks the current tool-only/no-source contract and makes a static command mode-dependent. | Reject |
| Extend editor `muga completions --format json` | Avoids another command. | Confuses editor symbol completion with shell argument completion and changes a stable JSON contract. | Reject |
| Add `std::cli::completion_spec[T]` first | Keeps completion data inside Muga programs. | Apps would still need host-specific script generation, and runtime APIs should not own installable shell scripts. | Defer |
| Generate only JSON completion specs first | Easier to test and shell-agnostic. | Lower immediate onboarding value because users cannot install JSON directly. | Completed after shell packaging |
| Generate completions in `muga new --template cli-tool` immediately | Strong first-run experience. | Premature before the generator command and artifact path are stable. | Defer |
| Emit an explicit completion package directory | Gives installers one command for bash, zsh, fish, and JSON without shell-profile mutation. | Adds a small file-writing CLI surface. | Implemented in [cli-completion-installer-integration.md](cli-completion-installer-integration.md) |

## Non-Goals

This design does not add:

- shell-profile edits or automatic installation;
- app execution during completion generation;
- runtime-owned process exits or help printing;
- config/TOML discovery for command defaults;
- dynamic values from env vars, processes, networks, or config discovery;
- completion support for compact short clusters beyond individual short flags;
- registry/publish packaging of generated completion files.

## Implementation Plan

1. Done: implement command enum schemas in
   [cli-subcommand-metadata.md](cli-subcommand-metadata.md).
2. Done: implement wrapper records and root/global option adoption in
   [cli-wrapper-root-options.md](cli-wrapper-root-options.md).
3. Done: record the schema-backed shell completion design here.
4. Done: implement `muga cli-completions <bash|zsh|fish> --program <name>
   --type <Type> ...`, including source, artifact-root, and `--built`
   coverage for the generated `cli-tool` starter.
5. Done: audit generated-project completion adoption in
   [post-cli-schema-shell-completion-adoption-gap-selection.md](post-cli-schema-shell-completion-adoption-gap-selection.md)
   and add install documentation plus a generated `cli-tool` README.
6. Done: add a generated project packaging hook that writes bash, zsh, and fish
   scripts into `completions/`.
7. Done: add shell-agnostic JSON completion specs in
   [cli-completion-json-spec.md](cli-completion-json-spec.md).
8. Done: implement richer nested command traversal for bash, zsh, fish, and
   JSON completion coverage.
9. Done: add static file/directory value-source metadata in
   [cli-completion-value-sources.md](cli-completion-value-sources.md).
10. Done: add non-mutating completion package emission in
    [cli-completion-installer-integration.md](cli-completion-installer-integration.md).
11. Next: evaluate TOML/config discovery before dynamic completion producers or
    host-mutating installer behavior.
