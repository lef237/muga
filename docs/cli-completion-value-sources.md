# CLI Completion Value Sources

Status: static filesystem value-source metadata implemented.

Generated app completions already expose command trees, options, enum values,
Bool candidates, and positional fallback. The next practical gap was telling
tools when a `String` CLI value is expected to be a file or directory path
without executing the target program or introducing config discovery.

## Goals

Short-Term Goal: let Muga authors annotate file and directory CLI values in the
same `@cli(...)` metadata that already drives parsing, help, artifacts, shell
completion, and JSON completion specs.

Medium-Term Goal: make generated completion specs useful to shells, package
manager hooks, editor adapters, and agents that need path-aware value
completion without scraping field names such as `path` or `config`.

Long-Term Goal: keep `CliSchema` as the single static contract for parsing,
help, diagnostics, artifacts, generated completions, and future installer
integration.

Final Goal: make Muga-authored tools feel publishable: common CLI value hints
should be declared once, checked by the compiler, persisted in artifacts, and
available to shell-specific and shell-agnostic consumers.

## Selected User Surface

Record fields may use:

```muga
pub record Options {
  @cli(name: "config", short: "c", value_source: "file")
  config: Option[String]

  @cli(name: "out-dir", short: "o", value_source: "directory")
  out_dir: String

  @cli(positional: 1, value_source: "directory")
  workspace: String
}
```

`value_source` accepts only `"file"` and `"directory"`. It is metadata only:
CLI parsing, help text, defaults, and validation behavior do not change.

The attribute is valid only on fields whose CLI value schema is `String`,
`Option[String]`, or `List[String]`. `Int`, `Bool`, and enum-backed values are
rejected because they already have scalar parsing rules or static candidate
sets.
In short: `String`, `Option[String]`, or `List[String]` can carry a filesystem
value source.

## Completion Contract

`CliSchema` carries `value_source` as a field-level fact and persists it through
package signatures, `.mgi` interfaces, `.mgb` schema payloads, MIR lowering, and
artifact-backed checks.

`muga cli-completions --format json` now emits `valueSource` on option and
positional entries:

- `"file"` for file path values;
- `"directory"` for directory path values;
- `null` when no explicit source is declared.

Positional entries still include `fallback`; when `value_source: "directory"`
is present the fallback becomes `"directory"`, otherwise it remains `"file"` for
backward-compatible consumers.

Shell renderers use the static source for option values:

- bash uses `compgen -f` or `compgen -d`;
- zsh uses `_files` or `_files -/`;
- fish keeps file values on normal file completion and uses
  `__fish_complete_directories` with file fallback disabled for directory
  values.

Static enum and Bool candidates continue to take precedence over filesystem
completion.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| Field-level `@cli(value_source: "file"|"directory")` | Reuses the existing CLI metadata surface, is compiler-checkable, persists in existing schema artifacts, and improves both shell and JSON completions without runtime effects. | Adds one field fact through parser, signatures, interfaces, artifacts, and renderers. | Select |
| Infer path completion from names such as `path`, `file`, or `dir` | No source syntax. | Heuristic and unstable; breaks localized or domain-specific names and cannot be artifact-contract evidence. | Reject |
| Add dynamic command callbacks for completion values | Very flexible. | Requires executing user code or host processes during completion, plus security, timeout, cancellation, and packaging policy. | Defer |
| Add TOML/config discovery first | Useful for config apps. | Broader precedence and filesystem discovery semantics; does not solve static completion contracts for strict CLI tools. | Defer |
| Add non-mutating installer integration | Better end-user setup and package-manager handoff. | Needs stable completion data first and must avoid mutating user shell configuration. | Implemented in [cli-completion-installer-integration.md](cli-completion-installer-integration.md) |

## Non-Goals

This slice does not add:

- runtime execution during completion;
- environment, process, network, or config-driven dynamic candidates;
- TOML/config discovery;
- shell-profile edits or automatic installation;
- glob, MIME, extension, or schema-specific file filtering;
- completion behavior for compact short clusters beyond existing option names.

## Implementation Plan

1. Done: validate `@cli(value_source: "file"|"directory")` in the parser.
2. Done: reject non-String CLI value schemas in type checking.
3. Done: carry value-source metadata through typed HIR, package signatures,
   `.mgi` interfaces, `CliSchema`, MIR, and `.mgb` artifacts.
4. Done: emit `valueSource` in completion JSON and use directory fallback for
   annotated positional fields.
5. Done: use file/directory sources for bash, zsh, and fish option value
   completion.
6. Done: add non-mutating completion package emission in
   [cli-completion-installer-integration.md](cli-completion-installer-integration.md).
7. Next: evaluate TOML/config discovery; keep dynamic completion callbacks and
   host-mutating installer behavior deferred until Muga has a host-effect policy
   for executing completion producers.
