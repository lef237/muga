# Muga By Example

Status: v1 learning path. This guide orders existing runnable examples into a
first tour from local bindings to packages and artifact-backed builds. It is
not a language expansion, release trigger, or substitute for the specification.

## How To Use This Guide

Install or run the compiler as described in
[installation-and-onboarding.md](installation-and-onboarding.md), then run each
example from the repository root. Use `muga` if it is installed, or replace
`muga` with `cargo run --locked --` when validating the checkout.

Keep scratch projects and explicit artifact roots under `~/tmp/`.

Start with the generated CLI app when you want the shortest local project
loop:

```bash
muga new --list-templates
muga new --template app ~/tmp/muga-hello
muga run ~/tmp/muga-hello/src/main/main.muga -- --name=Grace
muga build ~/tmp/muga-hello/src/main/main.muga
muga run --built ~/tmp/muga-hello/src/main/main.muga -- Grace
MUGA_PROGRAM=hello sh ~/tmp/muga-hello/scripts/package-app.sh
```

That template imports `std::env` and `std::cli`, accepts a positional name or
`--name`, prints `hello Muga` by default, and returns the same greeting as
`main`. It also includes a local README and `scripts/package-app.sh` helper for
creating a source-free bundle, running the bundle, emitting a `.mga` archive,
and verifying that archive without shell-profile mutation.

Generate the report starter when the first project should read and write files:

```bash
muga new --template report-app ~/tmp/muga-example-report
cd ~/tmp/muga-example-report
muga run src/main/main.muga
muga build src/main/main.muga
muga run --built src/main/main.muga -- data/daily.txt data/built-summary.txt
```

## Learning Map

| Step | Topic | Example | Commands |
|---|---|---|---|
| 1 | bindings, functions, loops, and `main` | `samples/println_sum.muga` | `muga run`, `muga check` |
| 2 | records, field access, and chained calls | `samples/record_user.muga`, `samples/record_with_update.muga` | `muga run` |
| 3 | explicit `Result` and `try` | `samples/result_try.muga`, `samples/string_parse_int.muga` | `muga run`, `muga check` |
| 4 | packages and imports | `samples/packages/app/main/main.muga` | `muga run`, `muga metadata --format json` |
| 5 | tests and generated docs | `muga new --template test ~/tmp/muga-example-test` | `muga test`, `muga doc` |
| 6 | local dependencies | `samples/projects/local_path_app/src/main/main.muga`, `muga new --template package-app ~/tmp/muga-example-package` | `muga run`, `muga build` |
| 7 | reusable `Result` package workflow | `samples/projects/report_app/src/main/main.muga`, `muga new --template report-app ~/tmp/muga-example-report` | `muga run --format json`, `muga run --built` |
| 8 | JSON config and typed CLI schema overlays | `samples/projects/config_app/src/main/main.muga`, `muga new --template config-app ~/tmp/muga-example-config` | `muga run`, `muga run --format json`, `muga run --built` |
| 9 | strict CLI-only tools | `samples/projects/cli_tool/src/main/main.muga` | `muga run`, `muga run --format json`, `muga run --built` |
| 10 | resource bytes export workflow | `samples/projects/resource_export/src/main/main.muga` | `muga run`, `muga build`, `muga run --built` |
| 11 | standard-library package samples | `samples/packages/app/std_io/main.muga`, `samples/packages/app/std_path_normalize/main.muga`, `samples/packages/app/std_path_strip_prefix/main.muga`, `samples/packages/app/std_fs_path/main.muga`, `samples/packages/app/std_fs_metadata/main.muga`, `samples/packages/app/std_fs_path_metadata/main.muga`, `samples/packages/app/std_fs_path_size_metadata/main.muga`, `samples/packages/app/std_fs_read_dir_recursive/main.muga`, `samples/packages/app/std_fs_directory_size_metadata/main.muga`, `samples/packages/app/std_fs_remove_dir_all/main.muga`, `samples/packages/app/std_fs_copy_dir_all/main.muga`, `samples/packages/app/std_fs_move_dir_all/main.muga`, `samples/packages/app/std_fs_rename/main.muga`, `samples/packages/app/std_fs_file_size/main.muga`, `samples/packages/app/std_fs_modified_time/main.muga`, `samples/packages/app/std_fs_write_bytes/main.muga`, `samples/packages/app/std_fs_canonicalize/main.muga`, `samples/packages/app/std_env/main.muga`, `samples/packages/app/std_env_current_dir/main.muga`, `samples/packages/app/std_env_temp_dir/main.muga`, `samples/packages/app/std_cli/main.muga`, `samples/packages/app/std_time/main.muga`, `samples/packages/app/std_string/main.muga`, `samples/packages/app/std_fmt/main.muga`, `samples/packages/app/std_json/main.muga`, `samples/packages/app/std_config/main.muga`, `samples/packages/app/std_hash/main.muga` | `muga run` |
| 12 | artifact-backed builds | `samples/packages/app/artifact_facade/main.muga` | `muga build`, `muga check --built`, `muga run --built` |

## 1. Bindings And Functions

Start with `samples/println_sum.muga`:

```bash
muga check samples/println_sum.muga
muga run samples/println_sum.muga
```

This sample introduces immutable-by-default bindings, `mut` for local mutation,
`while`, function calls, `println`, and a zero-argument `main`. The expected
output is:

```text
10
10
```

The first line is printed by the program. The second line is the returned value
of `main`.

## 2. Records And Chained Calls

Read and run the record examples:

```bash
muga run samples/record_user.muga
muga run samples/record_with_update.muga
```

Use these examples to learn `record` declarations, record literals, field
access, chained-call syntax such as `user.display_name()`, and `with(...)`
record updates. Records stay value-oriented; classes, inheritance, and hidden
property effects are not part of the v1 surface.

## 3. `Result` And `try`

Recoverable failures are explicit values:

```bash
muga check samples/result_try.muga
muga run samples/result_try.muga
muga run samples/string_parse_int.muga
```

`samples/result_try.muga` shows `Result::Ok`, `Result::Err`, prefix `try`, and
exhaustive `match`. The string parsing sample shows the same pattern with a
prelude helper that can fail.

## 4. Packages And Imports

Move from a single source file to a package entrypoint:

```bash
muga run samples/packages/app/main/main.muga
muga metadata --format json samples/packages/app/main/main.muga
muga workspace --format json samples/packages/app/main/main.muga
```

This path introduces `package`, `import`, public items, package aliases, and
the JSON metadata commands used by editor and agent tools. Source imports name
logical package paths, not filesystem paths.

## 5. Tests And Generated Docs

Create a throwaway test project under `~/tmp/`:

```bash
muga new --template test ~/tmp/muga-example-test
muga test ~/tmp/muga-example-test/src/main/main.muga
muga test --format json ~/tmp/muga-example-test/src/main/main.muga
muga doc ~/tmp/muga-example-test/src/main/main.muga
```

The generated template demonstrates static `@test` functions returning
`Result[Unit, String]`, `std::test` scalar assertions, structured JSON test
output, and Markdown docs generated from public package interfaces. It also
includes a local README with `muga test`, `muga test --format json`, and
`muga doc` commands.

## 6. Local Dependencies

Run a manifest project with a local path dependency:

```bash
muga run samples/projects/local_path_app/src/main/main.muga
muga build samples/projects/local_path_app/src/main/main.muga
muga why-rebuild --built samples/projects/local_path_app/src/main/main.muga
muga new --template package-app ~/tmp/muga-example-package
cd ~/tmp/muga-example-package
muga run app/src/main/main.muga -- Ada
muga build app/src/main/main.muga
muga run --built app/src/main/main.muga -- --name=Ada
sh scripts/package-package-app.sh
```

The app imports `shared::logging` from a sibling manifest project. `muga build`
also writes or validates local dependency metadata in `muga.lock`; malformed
lockfiles are rejected instead of silently overwritten. The generated
`package-app` starter turns the same local-dependency shape into a fresh
project tree with `app/`, `shared/`, workspace JSON, and source-free app bundle
packaging.

## 7. Reusable `Result` Package Workflow

The report sample combines local dependencies, args/env, stdout/stderr,
text-file handle writes, reusable package APIs, JSON run output, explicit
`Result` error handling, and built-artifact execution:

```bash
muga run --format json samples/projects/report_app/src/main/main.muga -- samples/projects/report_app/data/daily.txt ~/tmp/muga-report-app-summary.txt
muga build samples/projects/report_app/src/main/main.muga
muga run --built --format json samples/projects/report_app/src/main/main.muga -- samples/projects/report_app/data/daily.txt ~/tmp/muga-report-app-summary.txt
```

Use it after the smaller local dependency example when you want to see package
boundaries with real data under `samples/projects/report_app/data/daily.txt`
and a generated report under `~/tmp/`.

For a first-project version of the same file-processing loop, generate the
single-project starter:

```bash
muga new --template report-app ~/tmp/muga-example-report
cd ~/tmp/muga-example-report
muga run src/main/main.muga
muga run src/main/main.muga -- data/daily.txt data/custom-summary.txt
muga run --built src/main/main.muga -- data/daily.txt data/built-summary.txt
sh scripts/run-report.sh data/daily.txt data/script-summary.txt
sh scripts/package-report-app.sh
```

The generated template keeps local dependency teaching in the checked-in
sample, but gives new projects a compact `std::fs` + `std::path` report writer
with root-changing and source-free package helpers.

## 8. JSON Config And Typed CLI Schema Overlays

The config app sample composes existing stdlib packages into a small typed JSON
config workflow. The generated `config-app` template provides the same
practical starter without copying the repository sample:

```bash
muga new --template config-app ~/tmp/muga-example-config
muga run ~/tmp/muga-example-config/src/main/main.muga -- --help
muga run ~/tmp/muga-example-config/src/main/main.muga -- --config ~/tmp/muga-example-config/config/settings.json --port=5050
sh ~/tmp/muga-example-config/scripts/run-with-config.sh --tag ops
muga run samples/projects/config_app/src/main/main.muga
muga run samples/projects/config_app/src/main/main.muga -- --name=Grace --port 9090 --verbose=true --tag=ops --tag=admin
muga build samples/projects/config_app/src/main/main.muga
muga run --built --format json samples/projects/config_app/src/main/main.muga -- --config samples/projects/config_app/config/settings.json --port=5050
```

Use it to see `std::config`, `std::path`, `std::env`, `std::cli`,
`std::result::map_err`, and `std::string` work together with explicit
CLI > config > defaults precedence. This typed JSON config workflow now reads
structural settings directly: `Option[String]` owner fields, nested records, a
record-list shape (`List[Record]` via `List[Server]`), and a typed
`Map[String, Int]` limits map are decoded through
`config::load_json_or(config_path, default_settings())` without manual
`json::Value` metadata plumbing. The sample keeps configuration as ordinary
Muga code: path selection, typed JSON config loading, explicit error mapping,
typed `cli::parse_or[T]` settings overlays including repeated `--tag` list
values with `--tags` as a compatibility alias, `@cli(...)` field metadata,
generated `cli::help_for[T]` help text, explicit text assembly, and
`Result[String, String]` at the app boundary.
The JSON schema decoding design in
[json-schema-decoding.md](json-schema-decoding.md) selects and implements
`json::decode_or[T](value, fallback)` as the first decoder before required
`json::decode` or broader `std::config` work.
The `std::config` JSON default loading design in
[std-config-json-loading.md](std-config-json-loading.md) keeps the first config
loading boundary minimal before TOML, generated config templates, or full CLI
parser schemas.
[std-config-json-loading.md](std-config-json-loading.md) now records the
implemented `std::config::load_json_or[T](path, fallback)` and
`std::config::load_json[T](path)` slices. The generated config app keeps the
default-overlay form, while stricter apps can now require complete JSON config
files without manual read/parse/decode plumbing.
The implemented generated `muga new --template config-app` starter now carries
that onboarding slice.
The generated-template follow-up is now carried by
[json-required-decoding.md](json-required-decoding.md), which selects and
implements required `json::decode[T](value)` before TOML or broader decoder
target types.
[json-required-decoding.md](json-required-decoding.md) defines the strict
decoder contract and implementation for JSON inputs that should report missing
fields instead of falling back to defaults.
[json-decoder-target-expansion.md](json-decoder-target-expansion.md) implements
the structural target expansion for `Option[T]`, recursive `List[T]`, and typed
`Map[String, T]` across the existing JSON/config decoders.
The implemented `config_app` sample and generated `config-app` starter carry the
structural config workflow with `Option[String]`, nested records,
`List[Record]`, and typed `Map[String, Int]` settings so this learning path
shows typed nested settings without manual `json::Value` metadata plumbing.
The decoder expansion also implements enum JSON/config decoder support, using
zero-payload string tags and one-payload single-key objects before generic enum
decoding, field/variant schema polish, TOML, full CLI parser schemas, config
discovery, formatting helpers, or host effects.
The schema polish implementation in [json-config-schema-polish.md](json-config-schema-polish.md)
supports `@json(rename: "...")` on record fields and enum variants before
aliases, validation attributes, TOML, full CLI schemas, schema generation,
generic decoding, or host effects.
The strict unknown-field policy implementation in
[json-config-strict-unknown-fields.md](json-config-strict-unknown-fields.md)
supports record-level `@json(deny_unknown_fields)`, accepted wire-key
semantics, path-aware unknown-key errors, `.mgi` record flags, and `RF` decoder
artifact tokens before aliases, validation attributes, TOML, full CLI schemas,
schema generation, generic decoding, or host effects.
The alias metadata design in
[json-config-alias-metadata.md](json-config-alias-metadata.md) chooses repeated
`@json(alias: "...")` arguments inside a single field/variant `@json(...)` attribute,
accepted-name conflict checks, strict unknown-field integration, and `RG`/`EG`
artifact tokens before implementation.

## 9. Strict CLI-Only Tools

Use the strict CLI tool sample when the command should require options instead
of loading config defaults:

```bash
muga run samples/projects/cli_tool/src/main/main.muga -- --help
muga run samples/projects/cli_tool/src/main/main.muga -- run --help
muga run samples/projects/cli_tool/src/main/main.muga -- --profile=dev run service -dc3 -aApply -Tops --tags=prod --owner Kai
muga run samples/projects/cli_tool/src/main/main.muga -- inspect service -v
muga run samples/projects/cli_tool/src/main/main.muga -- run --count=3 --action Audit
muga new --template cli-tool my-cli-tool
muga build samples/projects/cli_tool/src/main/main.muga
muga run --built --format json samples/projects/cli_tool/src/main/main.muga -- --profile=prod run batch --count=5 --action Apply
```

This sample and the generated `cli-tool` template use `std::env::args()`,
strict `std::cli::parse_request[T]`, a `Root` wrapper record with a
`--profile` / `-p` global option, a command enum, generated root/leaf help,
`@cli(...)` command and field metadata, compact short options,
`@validate(...)`, zero-payload enum parsing, `Bool`, `Option[String]`,
`List[String]`, and `std::result::map_err` to keep recoverable `cli::Error`
values at a `Result[String, String]` app boundary. It answers `--help` / `-h`
through a typed `cli::Request[Root]` value while the lower-level
`cli::help_requested(args)` and `cli::help_for_required[Root]` helpers
remain available.
The subcommand adoption rationale and root/global option adoption boundary
are recorded in
[post-cli-subcommand-schema-adoption-gap-selection.md](post-cli-subcommand-schema-adoption-gap-selection.md).
Wrapper-record root/global option parsing for `tool --global run ...` shapes,
including schema/artifact lowering, generated wrapper help, and `cli-tool`
sample/template adoption, is recorded in
[cli-wrapper-root-options.md](cli-wrapper-root-options.md).
Schema-backed shell completions for generated apps are implemented in
[cli-schema-shell-completions.md](cli-schema-shell-completions.md), which
adds `muga cli-completions <bash|zsh|fish> --program <name> --type <Type>
...` as a `CliSchema`-driven command separate from static
`muga shell-completions` for the Muga developer tool.
Generated `cli-tool` projects include a generated `cli-tool` README with source
and `--built` completion generation commands plus a
`scripts/generate-completions.sh` packaging hook, following the adoption audit in
[post-cli-schema-shell-completion-adoption-gap-selection.md](post-cli-schema-shell-completion-adoption-gap-selection.md).
That hook now calls
`muga emit-cli-completions --format json --output-dir completions --program cli-tool --type Root ...`
to write bash, zsh, fish, and `.completions.json` artifacts through the
non-mutating package emission contract in
[cli-completion-installer-integration.md](cli-completion-installer-integration.md).

Generated `config-app` projects also support config path discovery: `--config`
wins, `MUGA_CONFIG_PATH` supplies a deployment default when the flag is absent,
and `config/settings.json` remains the generated fallback. The boundary is
documented in [config-path-discovery.md](config-path-discovery.md). Generated
projects include a local README plus `scripts/run-with-config.sh` and
`scripts/package-config-app.sh`; those helpers are documented in
[config-app-run-helper.md](config-app-run-helper.md).

For project-aware tools, `muga workspace --format json` reports manifest roots,
source roots, resource roots, and dependency source/resource roots. Wrappers can
use that metadata to set paths such as `MUGA_CONFIG_PATH` without changing the
process working directory. The contract is documented in
[workspace-manifest-metadata.md](workspace-manifest-metadata.md), and declared
resource archive inclusion is documented in
[package-resource-archives.md](package-resource-archives.md).
Runtime code can read declared UTF-8 resources with
`std::fs::read_resource_text(package, path)`, and declared binary resources with
`std::fs::read_resource_bytes(package, path)`. Local binary files can be read as
opaque `std::bytes::Bytes`, inspected with `bytes::size` / `bytes::at`, hashed
with `std::hash::sha256_hex`, and written back with `std::fs::write_bytes` /
`write_bytes_path`. These boundaries are documented in
[runtime-package-resource-lookup.md](runtime-package-resource-lookup.md),
[binary-file-read.md](binary-file-read.md),
[binary-file-write.md](binary-file-write.md), and
[bytes-sha256-hash.md](bytes-sha256-hash.md).

## 10. Resource Bytes Export Workflow

Run the manifest resource export sample when you want to see package-owned
bytes become a verified local file:

```bash
muga run samples/projects/resource_export/src/main/main.muga
muga run samples/projects/resource_export/src/main/main.muga -- ~/tmp/muga-resource-export-payload.bin
muga build samples/projects/resource_export/src/main/main.muga
muga run --built samples/projects/resource_export/src/main/main.muga -- ~/tmp/muga-resource-export-payload-built.bin
muga emit-app-bundle --format json --source-free --output-dir ~/tmp/muga-resource-export-bundle --program resource-export samples/projects/resource_export/src/main/main.muga
muga run-app-bundle ~/tmp/muga-resource-export-bundle -- ~/tmp/muga-resource-export-bundle-payload.bin
muga new --template resource-export ~/tmp/muga-example-resource
cd ~/tmp/muga-example-resource
muga run src/main/main.muga -- dist/generated-payload.bin
sh scripts/package-resource-export.sh
```

The sample declares `[package] resources = "resources"`, reads
`resources/static/payload.bin` with
`std::fs::read_resource_bytes("resource_export", "static/payload.bin")`,
computes `std::hash::sha256_hex(data)`, writes the payload with
`std::fs::write_bytes_path`, verifies the materialized path with
`std::fs::path_metadata_path`, verifies the round trip with
`std::fs::read_bytes_path`, and removes the temporary output. Its expected
result is
`Result::Ok(14|file|true|e54f8e906eaac9d311ba74b926b071faee0dc5a0036dd5a5e3c2b23b55f39728)`.
The source-free bundle commands show the same binary resource workflow running
from bundle-local artifacts and resources without copied source files. The
generated `resource-export` template turns that workflow into a first-project
starter with a package helper.
The focused design note is
[resource-bytes-export-sample.md](resource-bytes-export-sample.md).

## 11. Standard Library Package Samples

Run the current compiler-provided package samples:

```bash
muga run samples/packages/app/std_io/main.muga
muga run samples/packages/app/std_path_join/main.muga
muga run samples/packages/app/std_path_normalize/main.muga
muga run samples/packages/app/std_path_with_file_name/main.muga
muga run samples/packages/app/std_path_strip_prefix/main.muga
muga run samples/packages/app/std_path_with_extension/main.muga
muga run samples/packages/app/std_fs_path/main.muga
muga run samples/packages/app/std_fs_metadata/main.muga
muga run samples/packages/app/std_fs_path_metadata/main.muga
muga run samples/packages/app/std_fs_path_size_metadata/main.muga
muga run samples/packages/app/std_fs_read_dir_recursive/main.muga
muga run samples/packages/app/std_fs_directory_size_metadata/main.muga
muga run samples/packages/app/std_fs_remove_dir_all/main.muga
muga run samples/packages/app/std_fs_copy_dir_all/main.muga
muga run samples/packages/app/std_fs_move_dir_all/main.muga
muga run samples/packages/app/std_fs_rename/main.muga
muga run samples/packages/app/std_fs_file_size/main.muga
muga run samples/packages/app/std_fs_modified_time/main.muga
muga run samples/packages/app/std_fs_write_bytes/main.muga
muga run samples/packages/app/std_fs_canonicalize/main.muga
muga run samples/packages/app/std_env/main.muga
muga run samples/packages/app/std_env_current_dir/main.muga
muga run samples/packages/app/std_env_temp_dir/main.muga
muga run samples/packages/app/std_cli/main.muga
muga run samples/packages/app/std_time/main.muga
muga run samples/packages/app/std_string/main.muga
muga run samples/packages/app/std_fmt/main.muga
muga run samples/packages/app/std_json/main.muga
muga run samples/packages/app/std_config/main.muga
muga run samples/packages/app/std_hash/main.muga
```

These samples cover the user-facing `std::io`, `std::fs`, `std::path`,
`std::env`, `std::cli`, `std::time`, `std::string`, `std::fmt`, `std::json`,
`std::config`, `std::bytes`, and `std::hash` slices
without widening the v1 surface. The `std::fs` samples include one-shot IO,
metadata, typed `PathInfo` path classification, optional-size
`PathSizeMetadata`, directory copy/move, rename, and existing-path canonicalization. The `std::env`
samples cover environment
lookup, program arguments, and explicit `Result`-returning current/temp-directory
reads. The `std::cli` sample demonstrates
`positional`, `option`, repeated option value, and typed `Int` / `Bool` parsing
helpers over an explicit `List[String]`. The `std::string` sample demonstrates explicit
`to_string()` conversion plus `string::concat_all` / `string::join` text assembly.
The `std::fmt` sample demonstrates repeat, left/right padding, scalar-value
truncation, and explicit `{}` placeholder substitution without language interpolation. The `std::json` sample demonstrates parse/encode,
scalar/composite object-field accessor/default/required helpers, scalar array
projection helpers, direct scalar-array object-field helpers, JSON path helpers,
typed JSON path scalar projection helpers, and typed JSON path collection
projection helpers returning `json::Error`. The scalar array projection surface
remains explicit over `List[json::Value]`, and JSON paths use typed field/index
segments rather than a string parser. The `std::hash` sample demonstrates
reading local bytes, inspecting the first byte, and computing a lowercase
SHA-256 hex digest without adding codecs, mutable buffers, or broad crypto APIs.
The `std_fs_write_bytes` sample writes opaque `Bytes` to a temporary file,
reads them back, and removes the file without adding byte builders, binary
handles, or streams.
`std::fs::rename_path` covers one-step path renames through
`io::PathPairError` without adding recursive copy/delete fallback policy.
`std::fs::file_size_path` reports scalar byte length through `io::IOError`
without adding public metadata records or accessed/created timestamp policy.
`std::fs::modified_unix_millis_path` reports last-modified time through
`time::UnixMillis` without adding public metadata records, broader timestamp
APIs, permissions, or symlink policy.
`std::fs::path_info` returns `PathInfo` with a typed `PathKind` branch target
and the underlying `PathStatus` without adding host-error-backed metadata
policy.
`std::fs::path_metadata_path` returns host-error-backed `PathMetadata` for
existing files or directories without adding size, permissions, or symlink
classification policy.
`std::fs::path_size_metadata_path` returns `PathSizeMetadata` with
`Option::Some(bytes)` for regular files and `Option::None` for directories or
other existing paths without adding recursive directory sizing.
`std::fs::read_dir_recursive_path` lists descendants in deterministic
pre-order without mixing aggregation into the listing API, globbing, recursive
removal, or directory-copy policy.
`std::fs::directory_size_metadata_path` returns deterministic recursive
regular-file byte totals plus file/directory/other counts without adding
destructive behavior to the aggregate API, globbing, public symlink
classification, or sandbox policy.
`std::fs::remove_dir_all_path` removes a generated directory tree through
`io::IOError` without adding trash/recycle-bin integration,
globbing, or sandbox policy.
`std::fs::copy_dir_all_path` copies a directory tree into a new destination
through `io::PathPairError` without adding merge/overwrite, metadata
preservation, rollback, host-rename acceleration, globbing, or sandbox policy.
`std::fs::move_dir_all_path` moves a directory tree by copying it to a new
destination and then removing the source, without adding atomic rename
acceleration, rollback, merge/overwrite, globbing, or sandbox policy.
`std::path::with_file_name` derives sibling output names without filesystem
reads, path validation, or normalization.
`std::path::normalize` cleans `.` and internal `..` components without reading
the filesystem, resolving symlinks, or enforcing sandbox containment.
`std::path::strip_prefix` derives relative display or archive paths without
filesystem reads, normalization, or sandbox containment policy.
`std::path::with_extension` derives output or sidecar names without reading the
filesystem, normalizing paths, or resolving symlinks.
The full
stdlib package docs and samples review, including artifact-backed execution
samples where useful, is recorded in
[stdlib-package-samples-review.md](stdlib-package-samples-review.md).

## 12. Artifact-Backed Builds

Build, explain, and consume package artifacts explicitly:

```bash
muga build samples/packages/app/artifact_facade/main.muga
muga check --built samples/packages/app/artifact_facade/main.muga
muga run --built samples/packages/app/artifact_facade/main.muga
muga why-rebuild --built samples/packages/app/artifact_facade/main.muga
```

For a custom artifact root, keep the directory under `~/tmp/`:

```bash
mkdir -p ~/tmp/muga-example-artifacts
muga emit-artifacts --artifact-root ~/tmp/muga-example-artifacts samples/packages/app/artifact_facade/main.muga
muga check --artifact-root ~/tmp/muga-example-artifacts samples/packages/app/artifact_facade/main.muga
muga run --artifact-root ~/tmp/muga-example-artifacts samples/packages/app/artifact_facade/main.muga
```

This final step ties together `.mgi` public interfaces, `.mgc` check caches,
and `.mgb` implementation artifacts. Ordinary `check` and `run` remain
source-compatible; `--built` and `--artifact-root` are explicit artifact-backed
workflows and must not fall back to dependency source bodies.

## Local Archive Preview

The deterministic `.mgp` archive path is available for local workflows:

```bash
mkdir -p ~/tmp/muga-example-archives
muga emit-package-archive --archive-root ~/tmp/muga-example-archives --dependency-snippet samples/projects/local_path_app/src/main/main.muga
```

Treat this as local package-identity practice. Remote registries, network
fetching, package signing, and publishing workflows remain deferred. The
archive command prints the generated path plus its `sha256:<hex>` content hash;
verify the generated file name with `muga verify-package-archive <archive>`, or
pass that printed hash with `--expected-hash sha256:<hex>` when a local handoff
renames the `.mgp` file. Use
`muga unpack-package-archive [--format text|json] [--expected-hash sha256:<hex>] --output-dir <dir>
<archive>` to materialize the verified package for local review; the JSON form
reports the restored root, hash, and files for scripts.

## Maintenance Rules

- Every command in this guide should either use an existing runnable sample or
  a generated throwaway project under `~/tmp/`.
- Keep planned or post-v1 snippets out of `samples/`; use `docs/` for design
  sketches.
- Keep this guide release-neutral. It should make the current surface easier to
  learn, not pressure maintainers to cut a release.
- When a sample changes, update this guide and the release-readiness coverage
  together.
