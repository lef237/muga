# Editor JSON Workflow

Status: concrete smoke workflow for editor, LSP, CI, and agent adapters that
compose existing CLI JSON contracts without scraping human output.

This is not a new protocol layer. It is the recommended command sequence for a
single-entry editor adapter until a persistent server exists. Each command writes
one JSON object to stdout and leaves stderr empty for compiler diagnostics.

## Workflow

Use the open file for fast parse feedback:

```bash
muga syntax --format json src/main/main.muga
```

Use the package entrypoint for semantic diagnostics and source context:

```bash
muga check --format json src/main/main.muga
```

Load workspace and package facts after the entrypoint checks cleanly:

```bash
muga workspace --format json src/main/main.muga
muga metadata --format json src/main/main.muga
```

Use 1-based source positions for navigation and hover requests:

```bash
muga hover --format json --line 4 --column 12 src/main/main.muga
muga completions --format json src/main/main.muga
muga definition --format json --line 14 --column 12 src/main/main.muga
muga references --format json --line 14 --column 12 src/main/main.muga
```

Use the same JSON envelope for runnable feedback:

```bash
muga run --format json src/main/main.muga
muga test --format json src/tests/main.muga
```

## Adapter Rules

- Treat `schemaVersion`, `command`, `entry`, `status`, and `diagnostics` as the
  common envelope across these commands.
- Use `diagnostics[].context` for source, package, artifact, hash, and
  regeneration-command identity instead of parsing diagnostic display text.
- Use `workspace` for loaded packages, module files, default artifact root,
  manifest root/source root/resource root metadata, dependency source/resource
  roots, and dependency edges.
- Use `metadata` for public package records, enums, functions, source docs, and
  rendered type strings.
- Use `hover`, `completions`, `definition`, and `references` for navigation
  data. Positions are 1-based line and column values.
- Use `run` and `test` JSON output for editor task panels instead of mixing
  human stdout with structured diagnostics.

The regression test
`json_backed_editor_workflow_uses_existing_command_contracts` builds a package
with an app module, an imported module, and a test module, then exercises every
command above as one editor workflow.
