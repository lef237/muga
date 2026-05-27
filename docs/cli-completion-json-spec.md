# CLI Completion JSON Spec

Status: shell-agnostic generated-app completion spec implemented, including
static file/directory value sources.

`muga cli-completions` originally produced bash, zsh, and fish scripts directly
from a concrete `CliSchema`. That is useful for generated projects, but shell
scripts are a poor shared contract for package managers, installers, editor
extensions, and future completion renderers. This document defines the
schema-versioned JSON form that exposes the same completion facts without tying
consumers to one shell.

## Goals

Short-Term Goal: expose a deterministic JSON completion contract for generated
Muga CLIs without changing the Muga source language or `std::cli` runtime API.

Medium-Term Goal: let shell renderers, package-manager hooks, editor adapters,
and coding agents consume one recursive completion model instead of scraping
bash, zsh, or fish output.

Long-Term Goal: make `CliSchema` the shared source for parsing, help,
diagnostics, shell completions, JSON completion specs, generated templates, and
future installer integration.

Final Goal: make Muga-authored tools feel publishable by default: typed parse
contracts, generated help, generated completions, and machine-readable metadata
should be available from source and artifact-backed workflows.

## Selected User Surface

Shell-specific scripts keep the existing shape:

```bash
muga cli-completions fish --program cli-tool --type Root src/main/main.muga
```

The shell-agnostic contract uses `--format json` and does not accept a shell
argument:

```bash
muga cli-completions --format json --program cli-tool --type Root src/main/main.muga
muga cli-completions --format json --program cli-tool --type Root --built src/main/main.muga
muga cli-completions --format json --program cli-tool --type Root --artifact-root ~/tmp/muga-artifacts src/main/main.muga
```

The command loads source, explicit artifact roots, or default built artifacts
the same way the shell renderer does. It does not execute the target program.
`--program` remains required because the completion spec describes an external
command name. `--type` remains required so the command does not guess between
multiple CLI records or command enums.
`muga emit-app-completions` reuses the same `completion`, `program`, and
`target` objects for source-free app bundles, but its top-level input key is
`bundle` and its `command` is `emit-app-completions`.

## JSON Shape

The top-level output follows the existing command-output convention:

```json
{
  "schemaVersion": 1,
  "command": "cli-completions",
  "entry": {
    "path": "src/main/main.muga",
    "uri": "file:///workspace/src/main/main.muga"
  },
  "status": "ok",
  "diagnostics": [],
  "program": "cli-tool",
  "target": {
    "package": "cli_tool::main",
    "type": "Root"
  },
  "completion": {}
}
```

`completion` is recursive. Its `kind` is one of:

- `record`: a leaf option/positional schema.
- `command`: an enum-backed command tree.
- `wrapper`: a root record with options plus one `@cli(subcommand)` field.

Every completion schema includes:

- `type`: the Muga record or enum name.
- `about`: command summary text or `null`.
- `options`: visible non-positional fields.
- `positionals`: visible positional fields in index order.
- `commands`: visible command names and aliases for command schemas.
- `subcommand`: the wrapper subcommand field plus nested schema, or `null`.

Option entries include:

- `field`: Muga field name.
- `names`: long option names with `--`, including aliases.
- `short`: short option with `-`, or `null`.
- `takesValue`: whether completion should expect a value after the option.
- `repeatable`: whether the option can appear repeatedly.
- `help`: field help text or `null`.
- `valueSource`: `"file"`, `"directory"`, or `null`.
- `value`: value schema.
- `candidates`: static value candidates such as enum tags or Bool values.

Positional entries include `field`, `index`, `fallback`, `help`, `valueSource`,
`value`, and `candidates`. The fallback remains `"file"` unless the field uses
`@cli(value_source: "directory")`, in which case it is `"directory"` for
path-aware consumers.
Older positional consumers can continue to treat `fallback: "file"` as the
default when `valueSource` is `null`.
Unsupported record fields without explicit `@cli(...)` metadata are omitted
from completion output, matching `cli::parse_or[T]` default-preservation
behavior for config records. Unsupported fields with explicit CLI metadata stay
diagnostic errors.

Value schema `kind` values are `string`, `int`, `bool`, `option`, `list`, and
`enum`. Enum values expose both the Muga variant `name` and the completion
token in `completion`, which follows the same visible tag used by CLI parsing.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| Add `muga cli-completions --format json` | Reuses the existing command, source/artifact loading, `--program`, `--package`, and `--type` anchors; keeps shell renderers and JSON specs beside each other. | Requires making the shell positional optional only for JSON mode. | Select |
| Add a new `muga cli-completion-spec` command | Clear separation from shell scripts. | Adds another command for the same `CliSchema` concept and duplicates argument validation. | Reject |
| Extend editor `muga completions --format json` | Avoids another JSON contract. | Mixes source-symbol completion with shell argument completion and destabilizes an editor contract. | Reject |
| Put completion specs in `std::cli` runtime APIs | Lets Muga programs emit their own specs. | Requires app execution or runtime API expansion, which is wrong for package managers and installers. | Reject |
| Implement richer nested traversal first | Improves uncommon deep command trees. | Shell output remains the only contract, making future renderers harder to validate. | Defer |
| Add static file/directory value sources | Lets JSON consumers and shell renderers complete common path values from one checked `CliSchema` fact. | Requires one field-level `@cli(...)` metadata extension and artifact/interface persistence. | Implemented in [cli-completion-value-sources.md](cli-completion-value-sources.md) |
| Implement TOML/config discovery first | Useful for default-aware tools. | Opens broader config precedence and filesystem-discovery semantics before the static schema contract is complete. | Defer |
| Implement non-mutating installer integration | Better end-user setup and package-manager handoff without shell-profile mutation. | Needs the stable shell and JSON contracts first. | Implemented in [cli-completion-installer-integration.md](cli-completion-installer-integration.md) |

## Implementation Notes

The JSON spec is generated from the same `CliSchema` reconstruction used by the
shell renderers. Hidden fields and hidden commands are omitted because this is
a completion contract, not a private introspection API. The output preserves
wrapper, command, and record structure recursively so future renderers can
support deeper command trees without changing the top-level command surface.

Static file and directory value sources are represented explicitly through
`@cli(value_source: "...")`. Config-driven, environment-driven, or
command-executed completions remain future work.

## Non-Goals

This slice does not add:

- app execution during completion generation;
- shell-profile edits or automatic installation;
- TOML/config discovery;
- dynamic completion values from env vars, processes, networks, or config
  discovery;
- runtime-owned printing/exits;
- new source syntax;
- package publishing or binary installer behavior.

## Implementation Plan

1. Done: keep existing bash, zsh, and fish renderers working.
2. Done: add `muga cli-completions --format json --program <name> --type <Type> ...`.
3. Done: emit recursive wrapper/command/record completion JSON from source,
   explicit artifact roots, and `--built` workflows.
4. Done: document the JSON contract here and in the command-output guide.
5. Done: use this JSON contract to improve nested command traversal across
   bash, zsh, and fish renderers while preserving recursive JSON output.
6. Done: add static file/directory value-source metadata in
   [cli-completion-value-sources.md](cli-completion-value-sources.md).
7. Done: add non-mutating completion package emission in
   [cli-completion-installer-integration.md](cli-completion-installer-integration.md).
8. Next: evaluate TOML/config discovery before dynamic completion producers or
   host-mutating installer behavior.
