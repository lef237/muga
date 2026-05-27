# Diagnostics And Command Output

Status: v1 maintenance contract for human output plus the first
machine-readable diagnostic shape. This document defines the stable surface that
tools, CI, LSP prototypes, and agents should depend on instead of scraping
informal prose.

## Human Output

Human text is the default for every command.

- `muga --help`, `muga -h`, and `muga help` write the top-level usage text to
  stdout and exit successfully. `muga help <command>` writes the matching usage
  lines for a known command.
- `muga check <entry>` writes `ok` to stdout on success. On compiler
  diagnostics it writes one or more display diagnostics to stderr and exits 1.
- `muga run <entry>` writes program output to stdout. If `main()` returns a
  value, that value is printed after program output. If there is no `main()`,
  `ok` is printed after successful top-level execution.
- `muga test <entry>` writes one `test <name> ... ok` or
  `test <name> ... FAILED` line per discovered `@test` function, followed by a
  `test result: ...` summary. It exits 0 only when every test passes.
- `muga fmt <entry>` rewrites the file in place when formatting changes are
  needed and writes `formatted<TAB><path>`. It writes `ok` when the file is
  already formatted. `muga fmt --check <entry>` never writes; it writes
  `would format<TAB><path>` and exits 1 when formatting changes are needed.
  The formatter preserves line comments, including same-line trailing comments.
- `muga doc <entry>` writes Markdown documentation for public package records,
  enums, opaque types, and functions to stdout. It is generated from the same public interface
  graph used for `.mgi` artifacts, including item-level public source comments
  written as `///` before public records, enums, opaque types, and functions.
- `muga explain <diagnostic-code>` writes the matching `errors.md` catalog
  entry to stdout when one exists. For stable diagnostic-code prefixes without
  an exact entry, it writes the documented diagnostic family and points users to
  the diagnostic message, related notes, suggestions, and `errors.md` guidance.
- `muga doctor [--format text|json]` writes a read-only, tool-only environment
  check to stdout. Text mode is tab-separated; JSON mode writes one
  schema-versioned object. It does not parse source, load packages, inspect
  artifacts, mutate caches, or use the network.
- `muga shell-completions <bash|zsh|fish>` writes a static shell completion
  script to stdout. It is tool-only and does not install itself or inspect a
  project.
- `muga cli-completions <bash|zsh|fish> --program <name> --type <Type>
  [--package <package>] [--artifact-root <dir>|--built] <entry>` writes a
  deterministic generated-app shell completion script to stdout. It loads
  source or existing artifacts to read the selected `CliSchema`; diagnostics
  and usage errors are written to stderr.
- `muga cli-completions --format json --program <name> --type <Type>
  [--package <package>] [--artifact-root <dir>|--built] <entry>` writes a
  shell-agnostic generated-app completion spec to stdout. It uses the same
  `CliSchema` source as the shell renderers and does not accept a shell
  argument.
- `muga emit-cli-completions [--format text|json] --output-dir <dir>
  --program <name> --type <Type> [--package <package>]
  [--artifact-root <dir>|--built] <entry>` writes bash, zsh, fish, and
  shell-agnostic JSON completion artifacts into the selected directory.
- `muga emit-app-completions [--format text|json] --output-dir <dir>
  [--program <name>] --type <Type> [--package <package>] <bundle-dir>` writes
  the same completion artifacts from app bundle interface artifacts, including
  source-free bundles. Text output is one deterministic `written<TAB><path>`
  line per file, JSON output reports generated files, and neither command
  edits shell startup files.
- `muga list-installed-apps [--format text|json] --output-dir <bin-dir>` reads
  install ownership metadata without mutating files. Human output is either
  `empty<TAB><bin-dir><TAB>no installed apps` or one
  `<state><TAB><program><TAB><launcher><TAB><bundle><TAB><reason>` line per
  metadata entry.
- `muga install-app [--format text|json] ...` and
  `muga uninstall-app [--format text|json] ...` write deterministic
  written/removed file rows in text mode. Their JSON forms leave stderr empty on
  success and on structured errors.
- `muga syntax --format json <entry>` writes lex/parse diagnostics for one
  source file to stdout as one JSON object. It currently has no human text mode.
- `muga metadata --format json <entry>` writes package/module/item/export
  metadata plus public interface docs and rendered types to stdout as one JSON
  object. It currently has no human text mode.
- `muga workspace --format json <entry>` writes workspace metadata for loaded
  packages, module source files, the default artifact root, and dependency
  edges to stdout as one JSON object. It currently has no human text mode.
- `muga completions --format json <entry>` writes visible package/interface
  completions to stdout as one JSON object. It currently has no human text
  mode.
- `muga definition --format json --line <line> --column <column> <entry>`
  writes go-to-definition data for the selected source position to stdout as
  one JSON object. It currently has no human text mode.
- `muga references --format json --line <line> --column <column> <entry>`
  writes find references data for the selected source position to stdout as one
  JSON object. It currently has no human text mode.
- `muga hover --format json --line <line> --column <column> <entry>` writes
  declaration hover data for the selected source position to stdout as one JSON
  object. It currently has no human text mode.
- `muga new [--template app|lib|test|config-app|cli-tool|report-app|resource-export|package-app] <project-dir>` creates a new
  starter project tree and writes `created<TAB><project-dir>` followed by
  `entry<TAB><entry-path>` on success. It refuses targets that already exist
  and are not empty. The `config-app` template also writes
  `config/settings.json` and source that loads it with
  `std::config::load_json_or[T]`. The generated config path precedence is
  `--config` first, then `MUGA_CONFIG_PATH`, then `config/settings.json`. The
  `report-app` template writes `data/daily.txt` and a root-changing
  `scripts/run-report.sh` helper. The `resource-export` template declares
  `[package] resources = "resources"` and writes `resources/static/payload.bin`.
  The `package-app` template creates sibling `app/` and `shared/` packages,
  with `app/muga.toml` depending on `shared/` through a local path dependency.
- `muga new --list-templates [--format json]` lists current starter templates.
  Text output writes one `template<TAB><name><TAB><description>` line per
  template. JSON output writes one schema-versioned object with `command` set
  to `"new"`, `status` set to `"ok"`, and `templates[]` entries containing
  `name`, `aliases`, and `description`.
- `muga build <entry>` writes one tab-separated line per artifact:
  `written<TAB><path>` or `reused<TAB><path>`.
- `muga why-rebuild [--artifact-root <dir>|--built] <entry>` writes one
  tab-separated line per lockfile, archive-cache entry, and package artifact:
  `state<TAB>kind<TAB>package<TAB>path<TAB>reason`, followed by
  `run: <command>` guidance when regeneration is available. It is read-only and
  does not build, refresh, delete, or materialize artifacts.
- `emit-app-bundle [--format text|json]`, `emit-interface`, `emit-check-cache`,
  `emit-artifacts`,
  `emit-package-archive [--format text|json]`, and
  `emit-app-archive [--format text|json]` write generated bundle/artifact
  paths or archive metadata to stdout. The app-bundle and archive emission JSON
  forms leave stderr empty on success and on structured errors. Example
  commands are `muga emit-app-bundle --format json ...`,
  `muga emit-package-archive --format json ...`, and
  `muga emit-app-archive --format json ...`.
- `verify-app-archive [--expected-hash sha256:<hex>] <archive-file>`
  validates app archive bytes and entry headers without writing files. Without
  `--expected-hash`, it validates the generated `*-sha256-<hash>.mga` file
  name and uses that hash. Text output is tab-separated `status`, `archive`,
  `hash`, and `files` lines.
- `verify-package-archive [--expected-hash sha256:<hex>] <archive-file>`
  validates package archive bytes, manifest, source entries, and resource
  entries without materializing files or updating caches. Without
  `--expected-hash`, it validates the generated `*-sha256-<hash>.mgp` file
  name and uses that hash. Text output is tab-separated `status`, `archive`,
  `hash`, `manifest`, `sources`, and `resources` lines.
- `unpack-app-archive [--format text|json] [--expected-hash sha256:<hex>]
  --output-dir <dir> <archive-file>` validates the same app archive hash
  boundary, then materializes a bundle into an absent or empty destination.
  Text output is one `written<TAB><path>` line per file.
- `unpack-package-archive [--format text|json] [--expected-hash sha256:<hex>]
  --output-dir <dir> <archive-file>` validates the same package archive hash
  boundary, then materializes `muga.toml`, source files, and declared resources
  into an absent or empty destination. Text output is one `written<TAB><path>`
  line per file.
- CLI usage errors write a message plus usage text to stderr and exit 2.

The exact prose of human diagnostics may improve. Tools should use JSON output
when available.

## JSON Output

The first JSON contracts are `muga syntax --format json <entry>`,
`muga check --format json <entry>`,
`muga run --format json <entry>`,
`muga test --format json <entry>`,
`muga build --format json <entry>`,
`muga doctor --format json`,
`muga why-rebuild --format json [--artifact-root <dir>|--built] <entry>`,
`muga api-diff --format json --old-artifact-root <dir> --new-artifact-root <dir> --package <package>`,
`muga emit-cli-completions --format json --output-dir <dir> --program <name> --type <Type> <entry>`,
`muga emit-app-completions --format json --output-dir <dir> --type <Type> <bundle-dir>`,
`muga emit-app-bundle --format json [--source-free] --output-dir <dir> <entry>`,
`muga install-app --format json [--replace-owned] --output-dir <bin-dir> <bundle-dir>`,
`muga uninstall-app --format json --output-dir <bin-dir> --program <name>`,
`muga verify-app-archive --format json [--expected-hash sha256:<hex>] <archive-file>`,
`muga verify-package-archive --format json [--expected-hash sha256:<hex>] <archive-file>`,
`muga unpack-app-archive --format json [--expected-hash sha256:<hex>] --output-dir <dir> <archive-file>`,
`muga unpack-package-archive --format json [--expected-hash sha256:<hex>] --output-dir <dir> <archive-file>`,
`muga list-installed-apps --format json --output-dir <bin-dir>`,
`muga emit-artifacts --format json --artifact-root <dir> <entry>`,
`muga emit-interface --format json --artifact-root <dir> <entry>`,
`muga emit-check-cache --format json --artifact-root <dir> <entry>`,
`muga cli-completions --format json --program <name> --type <Type> <entry>`,
`muga metadata --format json <entry>`,
`muga workspace --format json <entry>`,
`muga completions --format json <entry>`,
`muga definition --format json --line <line> --column <column> <entry>`,
`muga references --format json --line <line> --column <column> <entry>`, and
`muga hover --format json --line <line> --column <column> <entry>`.

The concrete editor/LSP adapter workflow that composes these commands is
documented in [editor-json-workflow.md](editor-json-workflow.md). It validates
syntax, check, workspace, metadata, hover, completions, definition, references,
run, and test JSON as one workflow without adding a new protocol layer.
The artifact/cache explanation contract for non-mutating `muga why-rebuild
--format json` output is documented in
[artifact-cache-explanations.md](artifact-cache-explanations.md).
The package API diff JSON object uses `"command":"api-diff"` and reports the
package, status, summary counts, and changes from
[mgi-api-diff.md](mgi-api-diff.md).
The completion package emission JSON objects use
`"command":"emit-cli-completions"` or `"command":"emit-app-completions"`,
report the source `"entry"` or app `"bundle"`, `"outputDir"`, `"program"`,
target package/type, and deterministic `"files"` path/URI lists, and leave
stderr empty.
The app bundle emission JSON object uses `"command":"emit-app-bundle"`, a
`"root"` path/URI object, `"entry"`, `"launcher"`, `"program"`,
`"sourceMode"`, deterministic `"artifacts"` and `"files"` path/URI lists, and
leaves stderr empty. Error JSON uses top-level `"entry"` and `"outputDir"`
path/URI objects and leaves stderr empty.
The app install JSON object uses `"command":"install-app"`, `"bundle"`,
`"outputDir"`, `"launcher"`, `"metadata"`, `"program"`, `"replaceOwned"`, and
a deterministic `"files"` list. The app uninstall JSON object uses
`"command":"uninstall-app"`, `"outputDir"`, `"launcher"`, `"metadata"`,
`"program"`, and `"files"`. Structured install/uninstall errors leave stderr
empty.
The app archive verification JSON object uses
`"command":"verify-app-archive"`, an `"archive"` path/URI object, `"hash"`,
and a deterministic `"files"` list. Error JSON uses the same top-level
`"archive"` key and leaves stderr empty.
The app archive emission JSON object uses `"command":"emit-app-archive"`, an
`"archive"` path/URI object, `"program"`, `"hash"`, and a deterministic
`"files"` list of bundle-local file paths as path/URI objects. Error JSON uses
top-level `"bundle"` and `"archiveRoot"` path/URI objects and leaves stderr
empty.
The app archive unpack JSON object uses `"command":"unpack-app-archive"`, a
`"root"` path/URI object, and a deterministic `"files"` list of materialized
paths as path/URI objects. Error JSON uses top-level `"archive"` and
`"outputDir"` path/URI objects and leaves stderr empty.
The package archive verification JSON object uses
`"command":"verify-package-archive"`, an `"archive"` path/URI object, `"hash"`,
`"manifest"`, deterministic `"sources"` and `"resources"` lists, and the same
stderr-empty error contract.
The package archive emission JSON object uses
`"command":"emit-package-archive"`, `"entryPackage"`, an `"archive"` path/URI
object, `"hash"`, and `"dependencySnippet"` for the pasteable local archive
dependency entry. Error JSON uses top-level `"entry"` and `"archiveRoot"`
path/URI objects and leaves stderr empty.
The package archive unpack JSON object uses
`"command":"unpack-package-archive"`, a `"root"` path/URI object, `"hash"`, and
a deterministic `"files"` list of materialized paths as path/URI objects. Error
JSON uses top-level `"archive"` and `"outputDir"` path/URI objects and leaves
stderr empty.
The installed-app inventory JSON object uses
`"command":"list-installed-apps"`, an `"outputDir"` path/URI object,
`"metadataDir"`, and an `"apps"` list with program, state, reason, launcher,
metadata, bundle, and bundleLauncher fields. Drift states are reported with
`"status":"ok"` so tooling can inspect before mutating files.
The generated-app shell-agnostic completion contract for
`muga cli-completions --format json` is documented in
[cli-completion-json-spec.md](cli-completion-json-spec.md).

For `check`, success writes one JSON object to stdout and leaves stderr empty:

```json
{"schemaVersion":1,"command":"check","entry":{"path":"samples/println_sum.muga","uri":"file:///workspace/muga/samples/println_sum.muga"},"status":"ok","diagnostics":[]}
```

On compiler diagnostics, stdout contains one JSON object, stderr is empty, and
the process exits 1:

```json
{"schemaVersion":1,"command":"check","entry":{"path":"conformance/v1/rejecting/name-resolution/immutable_update.muga","uri":"file:///workspace/muga/conformance/v1/rejecting/name-resolution/immutable_update.muga"},"status":"error","diagnostics":[]}
```

Expanded for readability:

```json
{
  "schemaVersion": 1,
  "command": "check",
  "entry": {
    "path": "samples/println_sum.muga",
    "uri": "file:///workspace/muga/samples/println_sum.muga"
  },
  "status": "error",
  "diagnostics": []
}
```

`schemaVersion` is an integer. Version `1` means:

- `command` is the CLI command name.
- `entry.path` is the entry path exactly as the user passed it to the CLI.
- `entry.uri` is a best-effort absolute `file://` URI for editor, LSP, CI, and
  agent consumers. When the path exists it is built from the canonical path; if
  canonicalization fails it is built from the current working directory plus the
  provided path.
- `status` is `ok` or `error`.
- `diagnostics` is an array of diagnostic objects.

`entry` identifies the command target, not every possible source file involved
in package-aware checking. Per-diagnostic file/package/artifact identity should
be added through `context` when multi-file editor workflows need that precision.
Current CLI JSON diagnostic errors attach an entry source context to each
diagnostic's `diagnostics[].context` entry so editor, LSP, CI, and agent
consumers can map diagnostics without copying the top-level entry object.
For `check --format json` errors, diagnostics also include an entry package
context when the entry package path can be parsed. Artifact-backed checks add
an `artifactRoot` context for either the explicit `--artifact-root` directory
or the default `.muga/build` directory selected by `--built`. Artifact-backed
diagnostics that know a concrete `.mgi`, `.mgc`, or `.mgb` path also add an
`artifactFile` context entry with the artifact kind and a `file://` URI.

Usage and argument parsing errors are not yet JSON-stabilized; they keep the
human stderr plus exit-2 contract.

For `doctor`, success writes one JSON object to stdout and leaves stderr empty:

```json
{"schemaVersion":1,"command":"doctor","status":"ok","diagnostics":[],"checks":[{"name":"version","status":"ok","message":"muga 0.2.0"}]}
```

`doctor` field rules:

- `status` is `"ok"` when every environment check is ok and `"warn"` when at
  least one check reports a warning.
- `diagnostics` is present for command-output consistency and is currently
  empty because `doctor` is not a compiler diagnostic command.
- `checks[].name` is a stable check identifier such as `version`, `executable`,
  `cwd`, `home`, `temp`, or `path`.
- `checks[].status` is `"ok"` or `"warn"`.
- `checks[].message` is human-readable and may contain host-specific paths.
- The command is tool-only: it does not parse source, load package graphs,
  inspect artifacts, mutate caches, or use the network.

For generated app completions, JSON mode writes one object to stdout and leaves
stderr empty:

```json
{"schemaVersion":1,"command":"cli-completions","entry":{"path":"samples/projects/cli_tool/src/main/main.muga","uri":"file:///workspace/muga/samples/projects/cli_tool/src/main/main.muga"},"status":"ok","diagnostics":[],"program":"cli-tool","target":{"package":"cli_tool::main","type":"Root"},"completion":{"kind":"wrapper","type":"Root","about":"Run a typed strict CLI tool","options":[],"positionals":[],"commands":[],"subcommand":null}}
```

`cli-completions` JSON field rules:

- `program` is the external command name supplied through `--program`.
- `target.package` and `target.type` identify the selected concrete CLI schema.
- `completion.kind` is `record`, `command`, or `wrapper`.
- `completion.options` contains visible long/short options, repeatability,
  value expectations, static candidates, and help text.
- `completion.positionals` contains visible positional fields in index order
  with a conservative `fallback: "file"` marker.
- `completion.commands` contains visible command names, aliases, summaries, and
  nested schemas.
- `completion.subcommand` carries a wrapper record's `@cli(subcommand)` field
  and nested command schema, or `null`.
- Usage and argument errors remain human stderr plus exit 2. In JSON mode, the
  command does not accept a shell argument because shell-specific rendering is
  not part of the JSON contract.

For `run`, success writes one JSON object to stdout and leaves stderr empty:

```json
{"schemaVersion":1,"command":"run","entry":{"path":"samples/println_sum.muga","uri":"file:///workspace/muga/samples/println_sum.muga"},"status":"ok","diagnostics":[],"stdout":"10\n","stderr":"","mainResult":"10"}
```

`run` field rules:

- `status` is `"ok"` when the program executes successfully and `"error"` when
  compiler or runtime diagnostics prevent a successful run.
- `stdout` captures text emitted by the program through `print` and `println`.
- `stderr` captures text emitted by the program through `eprint` and
  `eprintln`. Text-mode `muga run` writes that captured program stderr to the
  process stderr stream on successful runs.
- `mainResult` is the string form of the returned `main()` value, matching the
  extra value line printed by text-mode `muga run`. It is `null` when there is
  no `main()` and only top-level statements execute.
- Program arguments after `--` keep the same behavior as text-mode `muga run`
  and are visible through `std::env::args()`.
- On compiler or runtime diagnostics, `muga run --format json` uses the same
  `status: "error"` and `diagnostics` envelope as `check`; result fields such
  as `stdout`, `stderr`, and `mainResult` are omitted.
- Runtime diagnostics may include `related` call-context notes for nested
  function call sites and the entrypoint being executed. Text output renders
  those as `note:` lines; JSON output preserves them in `diagnostics[].related`.
  Schema version 1 does not expose a separate `stackTrace` field; callers
  should use `diagnostics[].related` call-context notes for runtime stack
  context until a concrete consumer needs a more structured representation.

For `test`, success writes one JSON object to stdout and leaves stderr empty:

```json
{"schemaVersion":1,"command":"test","entry":{"path":"samples/tests.muga","uri":"file:///workspace/muga/samples/tests.muga"},"status":"ok","diagnostics":[],"tests":[{"name":"passes","status":"passed","message":null,"diagnostics":[],"stdout":"hello\n","stderr":""}],"summary":{"passed":1,"failed":0}}
```

When tests run but at least one test fails, stdout still contains one JSON
object, stderr is empty, and the process exits 1:

```json
{"schemaVersion":1,"command":"test","entry":{"path":"samples/tests.muga","uri":"file:///workspace/muga/samples/tests.muga"},"status":"error","diagnostics":[],"tests":[{"name":"fails","status":"failed","message":"boom","diagnostics":[],"stdout":"","stderr":""}],"summary":{"passed":0,"failed":1}}
```

`test` field rules:

- `status` is `"ok"` when all discovered tests pass and `"error"` when any
  discovered test fails or compiler diagnostics prevent running tests.
- `tests` is present only after test discovery, validation, and bytecode
  compilation succeed.
- `tests[].name` is the script function name or package-qualified test name
  used by text-mode `muga test`.
- `tests[].status` is `"passed"` or `"failed"`.
- `tests[].message` is `null` for passing tests and a string for assertion or
  `Result::Err(...)` failures.
- `tests[].diagnostics` preserves runtime diagnostics for a failed test using
  the same diagnostic object shape and entry source context as command
  diagnostics, including `related` call-context notes when the failure crosses
  function calls.
- Failed `std::test` scalar assertions keep `tests[].message` as the assertion
  failure string and also add an `R021` diagnostic in `tests[].diagnostics`.
  Its primary span points at the user assertion call, not the internal
  `std::test` wrapper body, and `related` can include the enclosing test or
  helper call context.
  Schema version 1 keeps that runtime stack context in `tests[].diagnostics[]`
  diagnostic `related` notes rather than a separate test-specific stack field.
- `tests[].stdout` captures stdout emitted while that individual test ran.
- `tests[].stderr` captures stderr emitted while that individual test ran.
- `summary.passed` and `summary.failed` are integer counts matching text-mode
  output.
- On compiler diagnostics before test execution, `muga test --format json`
  uses the same `status: "error"` and `diagnostics` envelope as `check`.

For `build`, success writes one JSON object to stdout and leaves stderr empty:

```json
{"schemaVersion":1,"command":"build","entry":{"path":"samples/packages/app/artifact_facade/main.muga","uri":"file:///workspace/muga/samples/packages/app/artifact_facade/main.muga"},"status":"ok","diagnostics":[],"artifactRoot":{"path":"samples/packages/app/artifact_facade/.muga/build","uri":"file:///workspace/muga/samples/packages/app/artifact_facade/.muga/build"},"artifacts":[{"status":"written","artifactKind":"interface","path":"samples/packages/app/artifact_facade/.muga/build/app__artifact_facade.mgi","uri":"file:///workspace/muga/samples/packages/app/artifact_facade/.muga/build/app__artifact_facade.mgi"}]}
```

- `artifactRoot` is the default `.muga/build` directory selected by the same
  rules as text-mode `muga build`.
- `artifacts` is ordered the same way as text build output.
- `artifacts[].status` is `"written"` when the artifact was created or
  replaced and `"reused"` when the existing artifact content was preserved.
- `artifacts[].artifactKind` is `"interface"` for `.mgi`, `"implementation"`
  for `.mgb`, and `"checkCache"` for `.mgc`.
- `artifacts[].path` is the artifact path and `artifacts[].uri` is a
  best-effort absolute `file://` URI.

On compiler diagnostics, `muga build --format json <entry>` uses the same
`status: "error"` and `diagnostics` envelope as `check`, with entry source
context plus entry package and default-build artifact-root context when those
can be inferred.

For artifact/cache explanations, `muga why-rebuild --format json
[--artifact-root <dir>|--built] <entry>` writes one JSON object to stdout and
leaves stderr empty:

```json
{"schemaVersion":1,"command":"why-rebuild","entry":{"path":"samples/packages/app/artifact_facade/main.muga","uri":"file:///workspace/muga/samples/packages/app/artifact_facade/main.muga"},"status":"ok","diagnostics":[],"artifactRoot":{"path":"samples/packages/app/artifact_facade/.muga/build","uri":"file:///workspace/muga/samples/packages/app/artifact_facade/.muga/build","selection":"built"},"lockfile":{"kind":"lockfile","path":"samples/projects/local_archive_app/muga.lock","uri":"file:///workspace/muga/samples/projects/local_archive_app/muga.lock","state":"fresh","reason":"package lockfile metadata matches current dependencies","dependencies":[{"packagePath":"shared","sourceKind":"archive","source":"../archives/shared.mgp","hashKind":"archive","hash":"sha256:<hex>","dependencies":[]}],"metadataHash":[{"kind":"artifactHash","role":"actual","hashKind":"lockfile","value":"sha256:<hex>"}],"regenerationCommand":[]},"archiveCache":[{"kind":"archiveCache","packagePath":"shared","path":"samples/projects/local_archive_app/.muga/packages/shared-sha256-<hex>","uri":"file:///workspace/muga/samples/projects/local_archive_app/.muga/packages/shared-sha256-<hex>","source":"../archives/shared.mgp","sourceUri":"file:///workspace/muga/samples/projects/archives/shared.mgp","state":"fresh","reason":"package archive dependency cache matches declared archive hash","metadataHash":[{"kind":"artifactHash","role":"actual","hashKind":"archiveCache","packagePath":"shared","value":"sha256:<hex>"}],"regenerationCommand":[]}],"packages":[{"path":"app::artifact_facade","role":"entry"}],"artifacts":[{"artifactKind":"interface","packagePath":"app::artifact_facade","path":"samples/packages/app/artifact_facade/.muga/build/app__artifact_facade.mgi","uri":"file:///workspace/muga/samples/packages/app/artifact_facade/.muga/build/app__artifact_facade.mgi","state":"fresh","reason":"artifact metadata matches current package interface","artifactFile":{"kind":"artifactFile","role":"interface","artifactKind":"interface","path":"samples/packages/app/artifact_facade/.muga/build/app__artifact_facade.mgi","uri":"file:///workspace/muga/samples/packages/app/artifact_facade/.muga/build/app__artifact_facade.mgi"},"artifactHash":[{"kind":"artifactHash","role":"actual","hashKind":"interface","packagePath":"app::artifact_facade","value":"sha256:<hex>"}],"regenerationCommand":[]}]}
```

- The command is read-only and non-mutating. It does not build, refresh,
  delete, or materialize artifacts.
- `artifactRoot.selection` is `"built"` for the default `.muga/build`
  directory and `"artifactRoot"` for an explicit `--artifact-root`.
- `lockfile` is `null` outside manifest projects. For manifest projects it
  reports `muga.lock` as `"kind":"lockfile"` with `state`, `reason`,
  `dependencies`, `metadataHash`, and `regenerationCommand` fields without
  rewriting the file. Local path dependencies use `"sourceKind":"path"` and
  local `.mgp` archive dependencies use `"sourceKind":"archive"`;
  lockfile-level hashes use `"hashKind":"lockfile"`.
- `archiveCache` lists local `.mgp` dependency cache entries under
  `.muga/packages` as `"kind":"archiveCache"` objects with cache path/URI,
  source archive path/URI, `metadataHash`, `state`, `reason`, and
  `regenerationCommand`; archive-cache hashes use
  `"hashKind":"archiveCache"`.
- `packages[].role` is `"entry"` or `"dependency"`.
- `artifacts[].artifactKind` is `"interface"` for `.mgi`, `"implementation"`
  for `.mgb`, and `"checkCache"` for `.mgc`.
- `artifacts[].state` is `"missing"`, `"fresh"`, `"stale"`,
  `"hashMismatch"`, `"invalid"`, or `"unknown"`.
- `artifactFile`, `artifactHash`, and `regenerationCommand` reuse the context
  names from JSON diagnostics so tooling can share parsing logic.
- Artifact states are reported inside a successful explanation. Compiler,
  manifest, or package-loading diagnostics that prevent explanation use the
  same `status: "error"` and `diagnostics` envelope as `check`.
- Text output is the default for terminal users. Machine consumers should keep
  using `--format json`.

For explicit artifact emission, `emit-artifacts --format json`,
`emit-interface --format json`, and `emit-check-cache --format json` write one
JSON object to stdout and leave stderr empty:

```json
{"schemaVersion":1,"command":"emit-artifacts","entry":{"path":"samples/packages/app/enum_demo/main.muga","uri":"file:///workspace/muga/samples/packages/app/enum_demo/main.muga"},"status":"ok","diagnostics":[],"artifactRoot":{"path":"artifacts","uri":"file:///workspace/muga/artifacts"},"artifacts":[{"artifactKind":"interface","path":"artifacts/app__enum_demo.mgi","uri":"file:///workspace/muga/artifacts/app__enum_demo.mgi"},{"artifactKind":"implementation","path":"artifacts/app__enum_demo.mgb","uri":"file:///workspace/muga/artifacts/app__enum_demo.mgb"},{"artifactKind":"checkCache","path":"artifacts/app__enum_demo.mgc","uri":"file:///workspace/muga/artifacts/app__enum_demo.mgc"}]}
```

- `command` is `"emit-artifacts"`, `"emit-interface"`, or
  `"emit-check-cache"`. The compact command fields are
  `"command":"emit-interface"` and `"command":"emit-check-cache"` for the
  single-purpose variants.
- `artifactRoot` is the explicit `--artifact-root` directory.
- `artifacts` is ordered the same way as text output.
- `artifacts[].artifactKind` is `"interface"` for `.mgi`, `"implementation"`
  for `.mgb`, and `"checkCache"` for `.mgc`.
- `artifacts[].path` is the artifact path and `artifacts[].uri` is a
  best-effort absolute `file://` URI.

On compiler diagnostics, these commands use the same `status: "error"` and
`diagnostics` envelope as `check`, with entry source context plus entry package
and output artifact-root context when those can be inferred.

For `syntax`, success writes one JSON object to stdout and leaves stderr empty:

```json
{"schemaVersion":1,"command":"syntax","entry":{"path":"samples/println_sum.muga","uri":"file:///workspace/muga/samples/println_sum.muga"},"status":"ok","diagnostics":[]}
```

On lexing, parsing, file-read, manifest layout, or inferred package diagnostics,
stdout contains one JSON object, stderr is empty, and the process exits 1:

```json
{"schemaVersion":1,"command":"syntax","entry":{"path":"samples/bad.muga","uri":"file:///workspace/muga/samples/bad.muga"},"status":"error","diagnostics":[]}
```

`syntax` field rules:

- `syntax` reuses the same envelope and diagnostic object shape as `check`,
  with `"command": "syntax"`.
- It lexes and parses one source file for faster editor feedback.
- It does not run resolver, typechecker, package import loading, package
  interface validation, or artifact checks.
- In manifest projects, it still validates that the source file can be parsed
  with the package path inferred from the manifest source layout.

For `metadata`, success writes one JSON object to stdout and leaves stderr
empty. Compiler diagnostics use the same `status: "error"` and `diagnostics`
shape as `check`, with `"command": "metadata"`.

Expanded success shape:

```json
{
  "schemaVersion": 1,
  "command": "metadata",
  "entry": {
    "path": "samples/packages/app/main/main.muga",
    "uri": "file:///workspace/muga/samples/packages/app/main/main.muga"
  },
  "status": "ok",
  "diagnostics": [],
  "entryPackage": { "id": 0, "path": "app::main" },
  "entryModule": { "id": 0, "package": 0, "path": "main" },
  "packages": [
    {
      "id": 0,
      "path": "app::main",
      "imports": [],
      "modules": [],
      "items": [],
      "exports": {
        "records": [],
        "enums": [],
        "opaqueTypes": [],
        "functions": []
      },
      "publicInterface": {
        "dependencies": [],
        "records": [],
        "enums": [],
        "opaqueTypes": [],
        "functions": []
      }
    }
  ]
}
```

`metadata` field rules:

- `entryPackage` and `entryModule` identify the checked package entrypoint.
- `packages[].imports` describes import aliases, resolved package ids, import
  paths, and source spans.
- `packages[].modules` and `packages[].items` expose compiler-owned package,
  module, and package-item ids for editor navigation and completions.
- `packages[].exports` lists public records, enums, opaque types, and functions
  with item ids, names, mangled names, and spans.
- `packages[].publicInterface` is generated from the same interface model as
  `.mgi` and `muga doc`, including public doc comments, type parameters, fields,
  enum variants, opaque type names and `handleFacts`, function parameters with
  `paramMode`, return types, and spans. Source-defined functions currently
  report `paramMode: "borrow"`; compiler-provided interfaces may later report
  `consume` before source syntax exists.

For `workspace`, success writes one JSON object to stdout and leaves stderr
empty. Compiler diagnostics use the same `status: "error"` and `diagnostics`
shape as `check`, with `"command": "workspace"`.

Expanded success shape:

```json
{
  "schemaVersion": 1,
  "command": "workspace",
  "entry": {
    "path": "samples/packages/app/main/main.muga",
    "uri": "file:///workspace/muga/samples/packages/app/main/main.muga"
  },
  "status": "ok",
  "diagnostics": [],
  "artifactRoot": {
    "path": "samples/packages/app/main/.muga/build",
    "uri": "file:///workspace/muga/samples/packages/app/main/.muga/build"
  },
  "project": {
    "manifest": {
      "path": "samples/projects/app/muga.toml",
      "uri": "file:///workspace/muga/samples/projects/app/muga.toml"
    },
    "root": {
      "path": "samples/projects/app",
      "uri": "file:///workspace/muga/samples/projects/app"
    },
    "sourceRoot": {
      "path": "samples/projects/app/src",
      "uri": "file:///workspace/muga/samples/projects/app/src"
    },
    "resourceRoot": {
      "path": "samples/projects/app/resources",
      "uri": "file:///workspace/muga/samples/projects/app/resources"
    },
    "packagePath": "app",
    "directDependencies": ["shared"],
    "dependencies": [
      {
        "packagePath": "shared",
        "root": {
          "path": "samples/projects/shared",
          "uri": "file:///workspace/muga/samples/projects/shared"
        },
        "sourceRoot": {
          "path": "samples/projects/shared/src",
          "uri": "file:///workspace/muga/samples/projects/shared/src"
        },
        "resourceRoot": null,
        "sourceKind": "path",
        "source": "../shared",
        "hash": null,
        "dependencies": []
      }
    ]
  },
  "entryPackage": { "id": 0, "path": "app::main" },
  "entryModule": { "id": 0, "package": 0, "path": "main.muga" },
  "packages": [
    {
      "id": 0,
      "path": "app::main",
      "role": "entry",
      "imports": [
        {
          "alias": "users",
          "package": 2,
          "path": "util::users",
          "span": {
            "start": { "line": 4, "column": 1 },
            "end": { "line": 4, "column": 19 }
          }
        }
      ],
      "modules": [
        {
          "module": { "id": 0, "package": 0, "path": "main.muga" },
          "sourceFile": {
            "path": "samples/packages/app/main/main.muga",
            "uri": "file:///workspace/muga/samples/packages/app/main/main.muga"
          },
          "lineCount": 12,
          "byteLength": 184,
          "checked": true
        }
      ]
    }
  ],
  "dependencyEdges": [
    {
      "from": { "id": 0, "path": "app::main" },
      "to": { "id": 2, "path": "util::users" },
      "alias": "users",
      "path": "util::users",
      "span": {
        "start": { "line": 4, "column": 1 },
        "end": { "line": 4, "column": 19 }
      }
    }
  ]
}
```

`workspace` field rules:

- `artifactRoot` is the default `.muga/build` directory that `muga build`,
  `check --built`, and `run --built` use for the entrypoint.
- `project` is present for manifest projects and `null` for package-mode source
  trees without `muga.toml`. It reports the manifest path, project root, source
  root, optional resource root, root package path, sorted direct dependencies,
  and resolved dependency source/resource roots. Dependency `sourceKind` is
  `"path"` or `"archive"`; archive dependencies also report their declared
  content hash.
- `entryPackage` and `entryModule` identify the checked package entrypoint.
- `packages[].role` is `entry` for the entry package and `dependency` for all
  other loaded packages reachable from imports.
- `packages[].modules[].sourceFile` contains a path and best-effort `file://`
  URI for real source files. It is `null` for compiler-provided virtual package
  modules.
- `packages[].modules[].checked` is `true` when the module was resolved and
  typechecked in the successful command run.
- `dependencyEdges` flattens package imports into explicit from/to edges for
  editor graph views. This is entry-reachable workspace metadata, not full
  project workspace discovery.

For `completions`, success writes one JSON object to stdout and leaves stderr
empty. Compiler diagnostics use the same `status: "error"` and `diagnostics`
shape as `check`, with `"command": "completions"`.

Expanded success shape:

```json
{
  "schemaVersion": 1,
  "command": "completions",
  "entry": {
    "path": "samples/packages/app/main/main.muga",
    "uri": "file:///workspace/muga/samples/packages/app/main/main.muga"
  },
  "status": "ok",
  "diagnostics": [],
  "completions": [
    {
      "label": "users",
      "kind": "import",
      "detail": "import util::users",
      "package": { "id": 2, "path": "util::users" },
      "module": null,
      "item": null,
      "signature": null,
      "docComments": [],
      "span": {
        "start": { "line": 3, "column": 1 },
        "end": { "line": 3, "column": 19 }
      }
    },
    {
      "label": "User",
      "kind": "record",
      "detail": "pub record User { name: String, age: Int }",
      "package": { "id": 2, "path": "util::users" },
      "module": { "id": 2, "package": 2, "path": "model.muga" },
      "item": 0,
      "signature": "pub record User { name: String, age: Int }",
      "docComments": [],
      "span": {
        "start": { "line": 3, "column": 5 },
        "end": { "line": 6, "column": 2 }
      }
    }
  ]
}
```

`completions` field rules:

- `completions` is a deterministic snapshot of package/interface completion
  candidates visible from the checked entry package. It currently includes
  import aliases plus public records, enums, opaque types, and functions from the entry
  package and directly imported packages.
- `label` is the completion text. `kind` is `import`, `record`, `enum`,
  `opaqueType`, or `function`. `detail` is a human-readable import description or public
  signature.
- `package`, `module`, `item`, and `span` reuse the same compiler-owned package
  metadata ids and spans as `metadata` when they exist. Import aliases have
  `module: null`, `item: null`, and the import statement span.
- Public declarations include `signature` and `docComments` from the same
  interface model as `.mgi`, `muga metadata`, `muga hover`, and `muga doc`.
  Opaque type signatures are rendered as `pub opaque type Name`. Public
  completion items also include a `metadata` object: opaque types expose
  `handleFacts`, functions expose `paramModes`, and records/enums currently use
  an empty object.

For `definition`, success writes one JSON object to stdout and leaves stderr
empty. Compiler diagnostics use the same `status: "error"` and `diagnostics`
shape as `check`, with `"command": "definition"`.

Expanded success shape:

```json
{
  "schemaVersion": 1,
  "command": "definition",
  "entry": {
    "path": "samples/packages/app/main/main.muga",
    "uri": "file:///workspace/muga/samples/packages/app/main/main.muga"
  },
  "status": "ok",
  "diagnostics": [],
  "position": { "line": 11, "column": 17 },
  "definition": {
    "kind": "function",
    "name": "birthday",
    "binding": null,
    "bindingKind": null,
    "package": { "id": 2, "path": "util::users" },
    "module": { "id": 8, "package": 2, "path": "ops.muga" },
    "item": 17,
    "span": {
      "start": { "line": 3, "column": 5 },
      "end": { "line": 5, "column": 2 }
    },
    "selectionSpan": {
      "start": { "line": 3, "column": 5 },
      "end": { "line": 5, "column": 2 }
    }
  }
}
```

`definition` field rules:

- `position` is the requested 1-based source line and column.
- `definition` is `null` when no supported definition target covers the
  requested position.
- The initial supported target scope is import aliases, local bindings, and
  package/interface item references in the checked entry module.
- `kind` is `import`, `binding`, `record`, `enum`, or `function`.
  `bindingKind` is `immutable`, `mutable`, `function`, or `parameter` for local
  bindings and `null` for package/interface items and imports.
- `package`, `module`, `item`, and `span` reuse the same compiler-owned package
  metadata ids and spans as `metadata` when they exist. Import aliases have
  `module: null`, `item: null`, and the import statement span.
- `selectionSpan` currently matches `span`; later versions may narrow it to the
  exact identifier once source-name spans are tracked separately from
  declaration spans.

For `references`, success writes one JSON object to stdout and leaves stderr
empty. Compiler diagnostics use the same `status: "error"` and `diagnostics`
shape as `check`, with `"command": "references"`.

Expanded success shape:

```json
{
  "schemaVersion": 1,
  "command": "references",
  "entry": {
    "path": "samples/packages/app/main/main.muga",
    "uri": "file:///workspace/muga/samples/packages/app/main/main.muga"
  },
  "status": "ok",
  "diagnostics": [],
  "position": { "line": 11, "column": 17 },
  "target": {
    "kind": "function",
    "name": "birthday",
    "binding": null,
    "bindingKind": null,
    "package": { "id": 2, "path": "util::users" },
    "module": { "id": 8, "package": 2, "path": "ops.muga" },
    "item": 17,
    "span": {
      "start": { "line": 3, "column": 5 },
      "end": { "line": 5, "column": 2 }
    },
    "selectionSpan": {
      "start": { "line": 3, "column": 5 },
      "end": { "line": 5, "column": 2 }
    }
  },
  "references": [
    {
      "kind": "declaration",
      "name": "birthday",
      "binding": null,
      "package": { "id": 2, "path": "util::users" },
      "module": { "id": 8, "package": 2, "path": "ops.muga" },
      "item": 17,
      "span": {
        "start": { "line": 3, "column": 5 },
        "end": { "line": 5, "column": 2 }
      }
    },
    {
      "kind": "reference",
      "name": "users::birthday",
      "binding": null,
      "package": { "id": 2, "path": "util::users" },
      "module": { "id": 8, "package": 2, "path": "ops.muga" },
      "item": 17,
      "span": {
        "start": { "line": 11, "column": 8 },
        "end": { "line": 11, "column": 23 }
      }
    }
  ]
}
```

`references` field rules:

- `position` is the requested 1-based source line and column.
- `target` is the same shape as the `definition` command's `definition` field,
  or `null` when no supported target covers the requested position.
- `references` is empty when `target` is `null`.
- The initial supported scope is declaration plus entry module references for
  import aliases, local bindings, and package/interface item references. It is
  not yet a full-workspace reference search.
- `references[].kind` is `declaration`, `reference`, or `write`.
- `package`, `module`, `item`, `binding`, and `span` reuse the same
  compiler-owned metadata ids and spans as `metadata` and `definition` when
  they exist.

For `hover`, success writes one JSON object to stdout and leaves stderr empty.
Compiler diagnostics use the same `status: "error"` and `diagnostics` shape as
`check`, with `"command": "hover"`.

Expanded success shape:

```json
{
  "schemaVersion": 1,
  "command": "hover",
  "entry": {
    "path": "samples/packages/util/users/model.muga",
    "uri": "file:///workspace/muga/samples/packages/util/users/model.muga"
  },
  "status": "ok",
  "diagnostics": [],
  "position": { "line": 3, "column": 12 },
  "hover": {
    "item": 0,
    "name": "User",
    "kind": "record",
    "visibility": "public",
    "package": { "id": 0, "path": "util::users" },
    "module": { "id": 0, "package": 0, "path": "model.muga" },
    "span": {
      "start": { "line": 3, "column": 5 },
      "end": { "line": 6, "column": 2 }
    },
    "signature": "pub record User { name: String, age: Int }",
    "docComments": []
  }
}
```

`hover` field rules:

- `position` is the requested 1-based source line and column.
- `hover` is `null` when no declaration header covers the requested position.
- `hover.item`, `hover.package`, `hover.module`, `hover.kind`,
  `hover.visibility`, and `hover.span` reuse the same compiler-owned package
  metadata ids and spans as `metadata`.
- Public declarations include `signature` and `docComments` from the same
  interface model as `.mgi`, `muga metadata`, and `muga doc`. Non-public
  declarations currently return `signature: null`, empty `docComments`, and
  `metadata: null`. Public opaque hovers expose `metadata.handleFacts`; public
  function hovers expose `metadata.paramModes`.

## Diagnostic Object

Every diagnostic object has this shape:

```json
{
  "code": "E001",
  "severity": "error",
  "message": "human diagnostic message",
  "span": {
    "start": { "line": 1, "column": 1 },
    "end": { "line": 1, "column": 5 }
  },
  "related": [
    {
      "message": "related note",
      "span": {
        "start": { "line": 1, "column": 1 },
        "end": { "line": 1, "column": 5 }
      }
    }
  ],
  "suggestions": [
    {
      "message": "suggestion text",
      "span": null,
      "replacement": null
    }
  ],
  "context": [
    {
      "kind": "source",
      "role": "entry",
      "path": "samples/println_sum.muga",
      "uri": "file:///workspace/muga/samples/println_sum.muga"
    },
    {
      "kind": "package",
      "role": "entry",
      "path": "app::main"
    },
    {
      "kind": "artifactRoot",
      "role": "check-input",
      "path": "artifacts",
      "uri": "file:///workspace/muga/artifacts"
    },
    {
      "kind": "artifactFile",
      "role": "dependency-interface",
      "artifactKind": "interface",
      "path": "artifacts/util__numbers.mgi",
      "uri": "file:///workspace/muga/artifacts/util__numbers.mgi"
    }
  ]
}
```

Field rules:

- `code` is the stable diagnostic code documented in `errors.md`.
- `severity` is currently always `error`.
- `message` is human-readable and may improve; tools should key behavior on
  `code` plus spans/context.
- `span`, related spans, and suggestion spans use 1-based line and column
  numbers when source spans are available. Default synthetic spans use `0`.
- `related` preserves related notes.
- `suggestions` preserves fix guidance. `span` and `replacement` are nullable.
- `context` is an array of structured context entries. CLI JSON compiler
  diagnostics currently include one source entry with `kind: "source"`,
  `role: "entry"`, the command target path, and a best-effort absolute
  `file://` URI. Library-level `Diagnostic::to_json_object()` may still render
  an empty context when no command target is known.
- `check --format json` diagnostics may also include a package entry with
  `kind: "package"`, `role: "entry"`, and the logical package path when the
  entry package can be parsed. Artifact-backed checks may also include an
  artifact root entry with `kind: "artifactRoot"`, `role: "check-input"` for
  explicit `--artifact-root` or `role: "default-build"` for `--built`, the
  artifact root path, and a best-effort absolute `file://` URI.
- Diagnostics that know a concrete artifact file may include an artifact file
  entry with `kind: "artifactFile"`, `artifactKind: "interface"`,
  `"checkCache"`, or `"implementation"`, roles such as
  `"dependency-interface"`, `"check-cache"`, `"implementation"`, or
  `"dependency-implementation"`, the artifact path, and a best-effort absolute
  `file://` URI. Current covered paths include `.mgi`, `.mgc`, and `.mgb`
  diagnostics.
- Stale or hash-mismatched artifact diagnostics may include artifact hash
  entries with `"kind": "artifactHash"`, roles such as `"expected"` or
  `"actual"`, `"hashKind"` values such as `"artifact"`, `"source"`,
  `"interface"`, or `"dependencyInterface"`, an optional `"packagePath"`, and
  the hash `value`. Compact examples include
  `"kind":"artifactHash","role":"expected","hashKind":"source"` and
  `"hashKind":"dependencyInterface","packagePath":"util::numbers"`.
  When an implementation artifact reports `dependency interface set changed`,
  added dependencies include expected `dependencyInterface` hash context and
  removed dependencies include actual `dependencyInterface` hash context.
- Artifact diagnostics with known regeneration actions may include
  regeneration command entries with `"kind": "regenerationCommand"`, a role
  such as `"default-build"`, `"artifact-root"`, `"interface"`, or
  `"check-cache"`, and a command string such as
  `"command":"muga emit-check-cache --artifact-root <dir> <entry>"`.

## Expansion Order

Keep the next JSON additions narrow:

1. do not add new JSON command surfaces unless a concrete consumer needs a new
   contract
2. broaden editor/agent use of artifact/cache explanations only through the
   JSON `muga why-rebuild` contract, which now has focused archive-cache
   coverage and separate human text output

Do not make LSP/editor tooling or agent workflows scrape human output when a
JSON contract exists.
