# Shell Completions And Doctor

Status: release-neutral tool-only adoption surface.

This document defines the small CLI usability slice for shell completions and
`muga doctor`. Both commands are tool-only: they do not parse or check Muga
source, do not load package graphs, do not read or write `.mgi`, `.mgc`, `.mgb`,
or `.mgp` artifacts, do not mutate caches, and do not use the network.

## Commands

```bash
muga shell-completions <bash|zsh|fish>
muga doctor [--format text|json]
```

`muga shell-completions <bash|zsh|fish>` writes a static completion script to
stdout for the requested shell. The completion scope is intentionally minimal:
top-level command names, common CLI options, output formats, project templates,
and the supported shell names for `shell-completions`. It does not inspect the
current project, package manifest, source tree, artifact root, or installed
shell configuration.

Generated Muga app completions are intentionally separate. Use
`muga cli-completions <bash|zsh|fish> --program <name> --type <Type> ...` when
completion candidates should come from a `CliSchema` for a source or artifact
workflow; that surface is documented in
[cli-schema-shell-completions.md](cli-schema-shell-completions.md). Package
managers, editor adapters, and future renderers can use
`muga cli-completions --format json --program <name> --type <Type> ...` for the
same generated-app completion facts without choosing a shell; that contract is
documented in [cli-completion-json-spec.md](cli-completion-json-spec.md).

`muga doctor [--format text|json]` writes a read-only environment report. The
initial checks cover the Muga version, current executable path, current working
directory, home directory, temporary directory, and `PATH` availability. The
command exits successfully after completing the report; warnings are represented
in output so humans and tools can decide whether they are actionable.

## Human Output

Text mode is tab-separated for terminal scanning:

```text
doctor	status	ok
ok	version	muga 0.2.0
ok	executable	current executable: /path/to/muga
ok	cwd	current directory: /workspace/muga
ok	home	home directory: /home/user
ok	temp	temporary directory: /tmp
ok	path	PATH has 8 entries
```

The exact paths and counts are environment-specific. The stable fields are
`doctor<TAB>status<TAB><ok|warn>` followed by
`<ok|warn><TAB><check-name><TAB><message>` lines.

## JSON Output

JSON mode emits one schema-versioned object:

```json
{
  "schemaVersion": 1,
  "command": "doctor",
  "status": "ok",
  "diagnostics": [],
  "checks": [
    {
      "name": "version",
      "status": "ok",
      "message": "muga 0.2.0"
    }
  ]
}
```

Field rules:

- `command` is `"doctor"`.
- `status` is `"ok"` when every check is ok and `"warn"` when at least one
  check is a warning.
- `diagnostics` is present for command-output consistency and is currently
  empty because this command reports environment checks instead of compiler
  diagnostics.
- `checks[].name` is a stable check identifier.
- `checks[].status` is `"ok"` or `"warn"`.
- `checks[].message` is human-readable and may include host-specific paths.

## Boundaries

This slice intentionally avoids broader installation management:

- no binary installer or shell-profile mutation;
- no automatic shell configuration edits;
- no source parsing, package graph loading, or artifact/cache inspection;
- no release, publish, registry, signing, provenance, or network checks;
- no replacement for the release gate in `scripts/v1-release-gate.sh`.

The practical standard-library direction remains separate: the first
`std::json` implementation follows the package contract in
[std-json-first-slice.md](std-json-first-slice.md), which documents `Result`
ergonomics, scalar/collection mapping, schema evolution, and diagnostics.
