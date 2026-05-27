# Muga

Muga is a compiler-first programming language for small, readable application
programs. The current implementation emphasizes immutable-by-default bindings,
local type inference, value semantics, records plus functions, explicit
`Option[T]` / `Result[T, E]`, and package interfaces that can be checked without
reading dependency implementation bodies.

This repository contains the language design, examples, and Rust
compiler/runtime implementation as Muga moves toward v1.

## Current Status

Muga is in a v1 hardening phase. The v1 release priority is to keep the promise narrow: source-compatible `check` / `run`, explicit package artifacts, actionable
diagnostics, generated project starters, and stable machine-readable command
contracts. The detailed scope lives in [docs/v1-release-checklist.md](./docs/v1-release-checklist.md),
and release gate alignment lives in [docs/release-gate-alignment.md](./docs/release-gate-alignment.md).

The explicit package artifact workflow remains the v1 package boundary. Plain `check` and `run` remain source-compatible. `check --built` and `run --built` consume artifacts from `.muga/build` only when requested.
Manifest projects may declare `[package] resources = "resources"` to include
text or binary resource files in package content hashes, deterministic `.mgp`
archives, `muga unpack-package-archive [--format text|json] [--expected-hash sha256:<hex>]`, and
local archive dependency caches.
`std::fs::read_resource_text` reads manifest-declared UTF-8 resources at
runtime for source, test, local archive dependency, and explicit built-artifact
runs.
`std::fs::read_resource_bytes` reads the same manifest-declared resources as
opaque `std::bytes::Bytes`; `bytes::size` and `bytes::empty` are the initial
inspection helpers.
`std::fs::read_bytes` / `read_bytes_path` read local binary files into `Bytes`;
`std::fs::write_bytes` / `write_bytes_path` write the same opaque payload as
full-file binary output, with `bytes::at` for inspection.
`std::hash::sha256_hex` computes lowercase SHA-256 hex digests for `Bytes`.
`muga emit-app-bundle [--format text|json] [--source-free] --output-dir <dir> [--program <name>] <entry>`
writes a non-mutating app bundle for manifest projects, including local path
and local archive dependencies, manifest-declared resources, `.muga/build`
artifacts, and a `bin/<program>` launcher. Source-backed bundles also include
`muga.lock`; `--source-free` omits copied source files and source-hash
lockfile metadata.
`muga install-app [--format text|json] [--replace-owned] --output-dir <bin-dir> [--program <name>] <bundle-dir>`
writes a wrapper plus ownership metadata into a chosen bin directory without shell startup edits;
`--replace-owned` verifies prior Muga ownership. `muga list-installed-apps [--format text|json] --output-dir <bin-dir>`
reports owned launcher state, and `muga uninstall-app [--format text|json] --output-dir <bin-dir> --program <name>` removes matching launcher/metadata files.
`muga emit-app-completions [--format text|json] --output-dir <dir> [--program <name>] --type <type> <bundle-dir>`
emits shell and JSON completion files from source-free bundle interfaces.
`muga run-app-bundle [--format text|json] <bundle-dir> [-- <program-arg>...]`
executes the bundle from its manifest, resources, and `.muga/build` artifacts
without reading copied source files.
`muga emit-app-archive [--format text|json] --archive-root <dir> [--program <name>] <bundle-dir>`
and `muga unpack-app-archive [--format text|json] [--expected-hash sha256:<hex>] --output-dir <dir> <archive-file>`
provide a deterministic `.mga` transport form.
`muga verify-app-archive [--format text|json] [--expected-hash sha256:<hex>] <archive-file>`
validates archive bytes without writing files; unpack validates either the
generated `*-sha256-<hash>.mga` file name or an explicit expected hash before
writing files.

Artifact roles:

- `.mgi`: public package interface for downstream checking.
- `.mgc`: entry package check-cache proof.
- `.mgb`: MIR-lowered bytecode implementation artifact for artifact-backed run.

Deferred work includes broad platform APIs, registry publishing, TOML parsing,
shell-profile installer mutation, full incremental project reuse, control-flow
MIR, native backend work, wildcard-heavy pattern matching, broad collection
APIs, concurrency syntax, schema-backed shell completion packaging beyond the
explicit completion package emitter, dynamic completion value producers, formatting
templates, interpolation, builders, binary streams/codecs,
broader cryptographic APIs, `Float`, `Decimal`, process APIs,
HTTP/SSE/WebSocket/RPC, streaming APIs, broader resource handles, and
release/publish automation.

## Install

Install the published command:

```bash
cargo install muga
```

Install this checkout:

```bash
cargo install --path . --locked
```

Development without installing:

```bash
cargo run --locked -- --version
cargo run --locked -- samples/println_sum.muga
```

More installation, version-check, shell completions, `muga doctor`, generated
app completions, and first-project guidance is in
[docs/installation-and-onboarding.md](./docs/installation-and-onboarding.md).
`muga shell-completions <bash|zsh|fish>` and `muga doctor [--format text|json]`
are tool-only commands.

## Quickstart

Create a simple app under `~/tmp/`:

```bash
muga new --template app ~/tmp/muga-hello
muga run ~/tmp/muga-hello/src/main/main.muga
muga run ~/tmp/muga-hello/src/main/main.muga -- Ada
muga check ~/tmp/muga-hello/src/main/main.muga
muga build ~/tmp/muga-hello/src/main/main.muga
muga run --built ~/tmp/muga-hello/src/main/main.muga -- --name=Ada
muga emit-app-bundle --format json --source-free --output-dir ~/tmp/muga-hello-bundle --program hello ~/tmp/muga-hello/src/main/main.muga
sh ~/tmp/muga-hello-bundle/bin/hello --name=Ada
muga run-app-bundle ~/tmp/muga-hello-bundle -- --name=Ada
muga install-app --format json --replace-owned --output-dir ~/tmp/muga-bin --program hello ~/tmp/muga-hello-bundle
muga list-installed-apps --format json --output-dir ~/tmp/muga-bin
sh ~/tmp/muga-bin/hello --name=Ada
muga uninstall-app --format json --output-dir ~/tmp/muga-bin --program hello
muga emit-app-archive --format json --archive-root ~/tmp/muga-archives --program hello ~/tmp/muga-hello-bundle
muga verify-app-archive ~/tmp/muga-archives/hello-sha256-....mga
MUGA_PROGRAM=hello MUGA_INSTALL_DIR=~/tmp/muga-bin sh ~/tmp/muga-hello/scripts/package-app.sh
```

Create a typed JSON config app:

```bash
muga new --template config-app ~/tmp/muga-config
muga run ~/tmp/muga-config/src/main/main.muga -- --help
muga run ~/tmp/muga-config/src/main/main.muga -- --config ~/tmp/muga-config/config/settings.json --port=5050
MUGA_CONFIG_PATH=~/tmp/muga-config/config/settings.json muga run ~/tmp/muga-config/src/main/main.muga -- --tag=ops
sh ~/tmp/muga-config/scripts/run-with-config.sh --tag=ops
sh ~/tmp/muga-config/scripts/package-config-app.sh
```

Try the strict CLI starter and generated app completions:

```bash
muga new --template cli-tool ~/tmp/muga-cli
muga run ~/tmp/muga-cli/src/main/main.muga -- --help
muga cli-completions fish --program cli-tool --type Root ~/tmp/muga-cli/src/main/main.muga
muga emit-cli-completions --format json --output-dir ~/tmp/muga-cli/completions --program cli-tool --type Root ~/tmp/muga-cli/src/main/main.muga
cd ~/tmp/muga-cli
sh scripts/package-cli-tool.sh
```

## Common Commands

```bash
muga --version
muga --help
muga doctor
muga doctor --format json
muga shell-completions bash
muga explain E001
muga syntax --format json path/to/file.muga
muga check path/to/file.muga
muga check --format json path/to/file.muga
muga run --format json path/to/file.muga -- arg1 arg2
muga test path/to/file.muga
muga test --format json path/to/file.muga
muga fmt --check path/to/file.muga
muga doc path/to/package/main.muga
muga metadata --format json path/to/package/main.muga
muga workspace --format json path/to/package/main.muga
muga completions --format json path/to/package/main.muga
muga definition --format json --line 4 --column 8 path/to/package/main.muga
muga references --format json --line 4 --column 8 path/to/package/main.muga
muga hover --format json --line 2 --column 12 path/to/package/main.muga
muga schema --format json path/to/package/main.muga
muga cli-completions fish --program cli-tool --type Root path/to/package/main.muga
muga cli-completions --format json --program cli-tool --type Root path/to/package/main.muga
muga emit-cli-completions --format json --output-dir completions --program cli-tool --type Root path/to/package/main.muga
muga new --list-templates
muga new --template app path/to/project
muga new --template report-app path/to/report-project
muga build path/to/package/main.muga
muga build --format json path/to/package/main.muga
muga why-rebuild --built path/to/package/main.muga
muga why-rebuild --format json --built path/to/package/main.muga
muga emit-package-archive --format json --archive-root path/to/archives --dependency-snippet path/to/package/main.muga
muga verify-package-archive --format json --expected-hash sha256:... path/to/archives/package.mgp
muga unpack-package-archive --format json --expected-hash sha256:... --output-dir path/to/unpacked-package path/to/archives/package.mgp
muga check --built path/to/package/main.muga
muga run --built path/to/package/main.muga
muga emit-app-bundle --format json --source-free --output-dir path/to/bundle --program my-app path/to/package/main.muga
muga run-app-bundle path/to/bundle -- arg
muga install-app --format json --replace-owned --output-dir path/to/bin --program my-app path/to/bundle
muga list-installed-apps --format json --output-dir path/to/bin
muga uninstall-app --format json --output-dir path/to/bin --program my-app
muga emit-app-completions --format json --output-dir path/to/completions --type Root path/to/bundle
muga emit-app-archive --format json --archive-root path/to/app-archives --program my-app path/to/bundle
muga verify-app-archive --format json path/to/app-archives/my-app-sha256-....mga
muga verify-app-archive --format json --expected-hash sha256:... path/to/app-archives/my-app.mga
muga unpack-app-archive --format json --expected-hash sha256:... --output-dir path/to/unpacked-renamed path/to/app-archives/my-app.mga
muga unpack-app-archive --output-dir path/to/unpacked path/to/app-archives/my-app-sha256-....mga
muga emit-artifacts --artifact-root path/to/artifacts path/to/package/main.muga
muga emit-interface --artifact-root path/to/artifacts --package util::numbers path/to/package/main.muga
muga emit-check-cache --artifact-root path/to/artifacts path/to/package/main.muga
muga api-diff --old-artifact-root old-artifacts --new-artifact-root new-artifacts --package util::numbers --format json --fail-on breaking
muga check --artifact-root path/to/artifacts path/to/package/main.muga
muga run --artifact-root path/to/artifacts path/to/package/main.muga
```

Key command scopes:

- `muga explain <diagnostic-code>` prints the matching `errors.md` catalog entry
  or diagnostic family.
- `muga syntax --format json <entry>` lexes and parses one source file for
  faster editor feedback.
- `muga test` for compiler-recognized `@test` functions runs script or package
  tests with `std::test` helpers such as `test::assert_eq_int`.
- `muga doc` emits Markdown documentation for public package records, enums,
  opaque types, and functions from the same public interface graph. Public source comments written as `///` are included.
- `muga new --list-templates [--format json]` lists starter templates; `muga new
  [--template app|lib|test|config-app|cli-tool|report-app|resource-export|package-app] <project-dir>` creates an app, library, package-with-test, config app, strict CLI tool, report app, resource export, or local package app skeleton.

Machine-readable command output is documented in
[docs/diagnostics-and-output.md](./docs/diagnostics-and-output.md). The
workspace JSON includes manifest roots, source roots, resource roots, and
dependency source/resource roots for project-aware editor, CI, and wrapper
tooling. Artifact state explanation is
documented in
[docs/artifact-cache-explanations.md](./docs/artifact-cache-explanations.md).
The JSON-backed editor workflow is documented in
[docs/editor-json-workflow.md](./docs/editor-json-workflow.md).

## Language Shape

- No `let`; bindings are immutable by default, and `mut` opts into mutation.
- Shadowing and mutation across function boundaries are rejected.
- Type inference is local-first.
- Data uses nominal `record`; behavior uses functions.
- `expr.name` is field access, `expr.name(...)` is chained-call syntax, and
  `expr.with(...)` is value-returning record update.
- `List[T]`, `Map[K, V]`, `Option[T]`, `Result[T, E]`, user-defined `enum`,
  exhaustive `match`, and prefix `try expr` are implemented.
- `and` / `or` are Bool-only, left-to-right, short-circuiting keyword
  operators.
- Equality is scalar-only for `Int`, `Bool`, and `String`; structural equality remains deferred.
- Classes, inheritance, traits, protocols, typeclasses, overloaded dispatch,
  ordinary source-level references, postfix Result propagation `expr?`, broad
  wildcard matching, map literals, `Set[T]`, arbitrary `Map` keys, iterator
  protocols, `T?`, and `?.` are outside v1.

The compact v1 language reference is [mini-language-spec-v1.md](./mini-language-spec-v1.md).
The split specs live under [spec/](./spec/).

## Implemented Standard Library Surface

The current compiler-provided packages include:

- `std::io`, `std::fs`, and `std::path` for typed text/binary file/path workflows,
  including `std::fs::File` handles, recursive directory listing/size metadata/copy/removal/move, `FileMetadata`, `PathInfo`, `PathMetadata`/`PathSizeMetadata`, package resources, and `using` cleanup.
- `std::env`, `std::cli`, and `std::time` for argument/env access, typed CLI parsing/help/completions, and `time::UnixMillis`.
- `std::test` with scalar assertions.
- `std::option`, `std::result`, `std::string`, `std::fmt`, `std::list`, and `std::map`.
- `std::json` and `std::config` for typed JSON/config workflows, including `config::load_json_or` and `config::load_json`.
- `std::bytes` with opaque `Bytes`, `bytes::size`, `bytes::empty`, and `bytes::at`.
- `std::hash` with `hash::sha256_hex` for `Bytes`.

Useful helper names include `option::map`, `option::and_then`,
`option::value_or`, `result::map`, `result::map_err`, `result::and_then`,
`result::value_or`, `string::concat_all`, `string::join`, `list::map`,
`list::filter`, `list::fold`, `list::any`, `list::all`, `map::keys`, and
`map::values`.

The first `std::json` boundary is intentionally Result-oriented and keeps
scalar/collection mapping, schema evolution, diagnostics, JSON Schema export,
typed decoding, and typed encoding explicit. It does not open schema generation
for services, HTTP APIs, `Float`, `Decimal`, `Bytes`, streaming APIs, or broad
resource handles. See [docs/std-json-first-slice.md](./docs/std-json-first-slice.md),
[docs/std-json-implementation-audit.md](./docs/std-json-implementation-audit.md),
[docs/json-schema-decoding.md](./docs/json-schema-decoding.md),
[docs/json-required-decoding.md](./docs/json-required-decoding.md),
[docs/json-decoder-target-expansion.md](./docs/json-decoder-target-expansion.md),
[docs/json-config-schema-polish.md](./docs/json-config-schema-polish.md),
[docs/json-config-strict-unknown-fields.md](./docs/json-config-strict-unknown-fields.md),
[docs/json-config-alias-metadata.md](./docs/json-config-alias-metadata.md),
[docs/json-config-validation-attributes.md](./docs/json-config-validation-attributes.md),
[docs/json-config-schema-export.md](./docs/json-config-schema-export.md), and
[docs/json-typed-encoding.md](./docs/json-typed-encoding.md).

The stdlib package docs and samples review is
[docs/stdlib-package-samples-review.md](./docs/stdlib-package-samples-review.md).
Standard-library review rules are in
[docs/standard-library-review-rules.md](./docs/standard-library-review-rules.md).

## Project And Package Workflow

Manifest projects use `muga.toml`:

```toml
[package]
name = "my_app"
source = "src"

[dependencies]
shared = { path = "../shared" }
archived_shared = { archive = "../archives/shared-sha256-....mgp", hash = "sha256:..." }
```

Local path dependencies, local `.mgp` archive dependencies, deterministic
package content hashing, deterministic `.mgp` source/resource archive
emission, non-mutating package archive verification, CLI package archive
unpacking/materialization, manifest-declared text/binary resource inclusion,
UTF-8 runtime resource lookup, source-backed app bundle emission with bundle-local
dependencies, install ownership metadata, installed-app inventory, generated
package-helper install hooks, archive emission JSON, guarded uninstall, source-free app completion package emission, minimal local `muga.lock` metadata, and malformed
lockfile rejection are implemented.
URL/Git/registry dependency forms, remote fetching, publishing/install
workflows, package signing, and full published-package lockfile enforcement
remain deferred.

## Documentation Map

Start here:

- [docs/README.md](./docs/README.md): documentation map and reading order.
- [docs/strategy-and-implementation-plan.md](./docs/strategy-and-implementation-plan.md):
  north star, phase sequence, and non-goals.
- [ROADMAP.md](./ROADMAP.md): current implementation priority.
- [docs/implementation-resume-plan.md](./docs/implementation-resume-plan.md):
  implementation ledger, resume checklist, and next-slice test plan.
- [docs/practical-language-readiness.md](./docs/practical-language-readiness.md):
  practical post-v1 backlog and boundaries.
- [docs/muga-by-example.md](./docs/muga-by-example.md): learning path through
  bindings, records, `Result`, packages, tests, local dependencies, and
  artifact-backed builds.

Maintenance and trust:

- [errors.md](./errors.md)
- [docs/mgi-api-diff.md](./docs/mgi-api-diff.md)
- [docs/fuzzing-malformed-input-plan.md](./docs/fuzzing-malformed-input-plan.md)
- [docs/benchmark-health-checks.md](./docs/benchmark-health-checks.md)
- [docs/registry-security-design.md](./docs/registry-security-design.md)
- [docs/edition-feature-fingerprint-policy.md](./docs/edition-feature-fingerprint-policy.md)
- [docs/release-gate-alignment.md](./docs/release-gate-alignment.md)
- [conformance/README.md](./conformance/README.md)

Design boundaries:

- [docs/opaque-resource-handles.md](./docs/opaque-resource-handles.md)
- [docs/text-output-file-handles.md](./docs/text-output-file-handles.md)
- [docs/lexical-resource-cleanup.md](./docs/lexical-resource-cleanup.md)
- [docs/cli-parser-schema.md](./docs/cli-parser-schema.md)
- [docs/strict-cli-parser-schema.md](./docs/strict-cli-parser-schema.md)
- [docs/strict-cli-no-default-usage.md](./docs/strict-cli-no-default-usage.md)
- [docs/cli-field-metadata.md](./docs/cli-field-metadata.md)
- [docs/cli-command-metadata.md](./docs/cli-command-metadata.md)
- [docs/cli-short-option-metadata.md](./docs/cli-short-option-metadata.md)
- [docs/cli-positional-field-metadata.md](./docs/cli-positional-field-metadata.md)
- [docs/cli-built-in-help-policy.md](./docs/cli-built-in-help-policy.md)
- [docs/parse-integrated-cli-help-workflow.md](./docs/parse-integrated-cli-help-workflow.md)
- [docs/compact-cli-short-option-syntax.md](./docs/compact-cli-short-option-syntax.md)
- [docs/cli-subcommand-metadata.md](./docs/cli-subcommand-metadata.md)
- [docs/cli-wrapper-root-options.md](./docs/cli-wrapper-root-options.md)
- [docs/cli-schema-shell-completions.md](./docs/cli-schema-shell-completions.md)
- [docs/cli-completion-json-spec.md](./docs/cli-completion-json-spec.md)
- [docs/cli-completion-value-sources.md](./docs/cli-completion-value-sources.md)
- [docs/cli-completion-installer-integration.md](./docs/cli-completion-installer-integration.md)
- [docs/config-path-discovery.md](./docs/config-path-discovery.md)
- [docs/config-app-run-helper.md](./docs/config-app-run-helper.md)
- [docs/workspace-manifest-metadata.md](./docs/workspace-manifest-metadata.md)
- [docs/std-config-json-loading.md](./docs/std-config-json-loading.md)

Historical decision logs are kept as evidence, not as the main reading path.
Use [docs/README.md](./docs/README.md) to find the active reading path and to
understand when a historical log can be removed safely.

## Samples

Runnable sample entrypoints and support files live under `samples/`.
Future-looking snippets that are not valid v1 source live under
`docs/design-snippets/`.

Important sample paths:

- `samples/println_sum.muga`
- `samples/result_try.muga`
- `samples/packages/app/main/main.muga`
- `samples/packages/app/artifact_facade/main.muga`
- `samples/projects/local_path_app/src/main/main.muga`
- `samples/projects/report_app/src/main/main.muga`
- `samples/projects/config_app/src/main/main.muga`
- `samples/projects/cli_tool/src/main/main.muga`
- `samples/projects/resource_export/src/main/main.muga`
- `samples/packages/app/std_io/main.muga`
- `samples/packages/app/std_path/main.muga`
- `samples/packages/app/std_path_join/main.muga`
- `samples/packages/app/std_path_normalize/main.muga`
- `samples/packages/app/std_path_file_name/main.muga`
- `samples/packages/app/std_path_with_file_name/main.muga`
- `samples/packages/app/std_path_parent/main.muga`
- `samples/packages/app/std_path_strip_prefix/main.muga`
- `samples/packages/app/std_path_extension/main.muga`
- `samples/packages/app/std_path_file_stem/main.muga`
- `samples/packages/app/std_path_with_extension/main.muga`
- `samples/packages/app/std_path_is_absolute/main.muga`
- `samples/packages/app/std_fs_path/main.muga`
- `samples/packages/app/std_fs_read_dir/main.muga`
- `samples/packages/app/std_fs_metadata/main.muga`
- `samples/packages/app/std_fs_path_metadata/main.muga`
- `samples/packages/app/std_fs_path_size_metadata/main.muga`
- `samples/packages/app/std_fs_read_dir_recursive/main.muga` and `samples/packages/app/std_fs_directory_size_metadata/main.muga`
- `samples/packages/app/std_fs_create_dir/main.muga`
- `samples/packages/app/std_fs_create_dir_all/main.muga`
- `samples/packages/app/std_fs_remove_file/main.muga`
- `samples/packages/app/std_fs_remove_dir/main.muga` and `samples/packages/app/std_fs_remove_dir_all/main.muga`
- `samples/packages/app/std_fs_copy_file/main.muga`, `samples/packages/app/std_fs_copy_dir_all/main.muga`, and `samples/packages/app/std_fs_move_dir_all/main.muga`
- `samples/packages/app/std_fs_rename/main.muga`
- `samples/packages/app/std_fs_file_size/main.muga`
- `samples/packages/app/std_fs_modified_time/main.muga`
- `samples/packages/app/std_fs_file_metadata/main.muga`
- `samples/packages/app/std_fs_canonicalize/main.muga`
- `samples/packages/app/std_env/main.muga`
- `samples/packages/app/std_env_args/main.muga`
- `samples/packages/app/std_env_current_dir/main.muga`
- `samples/packages/app/std_env_temp_dir/main.muga`
- `samples/packages/app/std_cli/main.muga`
- `samples/packages/app/std_cli_schema/main.muga`
- `samples/packages/app/std_time/main.muga`
- `samples/packages/app/std_string/main.muga` and `samples/packages/app/std_fmt/main.muga`
- `samples/packages/app/std_json/main.muga`
- `samples/packages/app/std_hash/main.muga`
- `samples/packages/app/std_list/main.muga`
- `samples/packages/app/std_map/main.muga`
- `samples/packages/app/std_option/main.muga`
- `samples/packages/app/std_result/main.muga`

Example-driven learning is in [docs/muga-by-example.md](./docs/muga-by-example.md).

## Validation

Local offline release-quality gate:

```bash
scripts/v1-release-gate.sh
```

Release-time dry run, for maintainers only:

```bash
scripts/v1-release-gate.sh --with-publish-dry-run
```

Release-neutral benchmark health checks:

```bash
scripts/benchmark-health-check.sh
```
