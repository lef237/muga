# Muga Roadmap

This roadmap is the source of truth for the next implementation priority and the longer design path. For the strategic phase sequence, non-goals, and measurement stance, see [docs/strategy-and-implementation-plan.md](./docs/strategy-and-implementation-plan.md). For the detailed implementation ledger, resume checklist, and next-slice test plan, see [docs/implementation-resume-plan.md](./docs/implementation-resume-plan.md).

## Current Snapshot

Implemented compiler/runtime pieces:

- lexer, parser, resolver, typechecker, typed HIR, initial MIR lowering, bytecode compiler, and VM runtime
- `check` and `run` entrypoints
- symbol-based local binding identity and package item identity
- typed HIR carrying expression types, local binding targets, call targets, call origin, and package item references
- diagnostics with related notes and suggestions in selected resolver, typechecker, record, and package paths

Implemented language surface:

- immutable-by-default bindings, `mut`, no shadowing, local-only inference, higher-order functions, and closure capture
- Bool-only short-circuit `and` / `or` keyword operators
- records, field access, `record.with(...)`, chained calls, and package-qualified chained calls
- local binding annotations and function type annotations with `->`
- `Unit` with the `()` literal for effect-only success values
- compiler-provided `std::io::IOError`, `std::io::PathPairError`, `std::fs::FileMetadata`, `std::fs::PathStatus`, `std::fs::PathKind`, `std::fs::PathInfo`, `std::fs::PathMetadata`, `std::fs::PathSizeMetadata`, `std::fs::DirectorySizeMetadata`, `std::fs::read_text`, `std::fs::read_bytes`, `std::fs::write_text`, `std::fs::read_text_path`, `std::fs::read_bytes_path`, `std::fs::write_text_path`, `std::fs::read_dir_path`, `std::fs::read_dir_recursive_path`, `std::fs::directory_size_metadata_path`, `std::fs::create_dir_path`, `std::fs::create_dir_all_path`, `std::fs::remove_file_path`, `std::fs::remove_dir_path`, `std::fs::remove_dir_all_path`, `std::fs::copy_file_path`, `std::fs::copy_dir_all_path`, `std::fs::move_dir_all_path`, `std::fs::canonicalize_path`, `std::fs::file_size_path`, `std::fs::modified_unix_millis_path`, `std::fs::file_metadata_path`, `std::fs::path_status`, `std::fs::path_kind`, `std::fs::path_info`, `std::fs::path_metadata_path`, `std::fs::path_size_metadata_path`, `std::fs::exists_path`, `std::fs::is_file_path`, `std::fs::is_dir_path`, minimal `std::path::Path`, `std::path::join`, `std::path::normalize`, `std::path::file_name`, `std::path::with_file_name`, `std::path::parent`, `std::path::strip_prefix`, `std::path::extension`, `std::path::file_stem`, `std::path::with_extension`, `std::path::is_absolute`, `std::bytes::Bytes`, `std::bytes::size`, `std::bytes::empty`, `std::bytes::at`, `std::hash::sha256_hex`, `std::env::get_var`, `std::env::args`, `std::env::current_dir`, `std::env::temp_dir`, and `std::time::now_unix_millis` package slices
- `List[T]`, `Option[T]`, `Result[T, E]`, and `Map[K, V]` type expressions
- list literals, indexing, `len`, `is_empty`, `push`, `get`, and `set`
- string helpers `is_empty`, `contains`, `trim`, `char_count`, `byte_len`, `starts_with`, `ends_with`, `replace`, `split`, `concat`, `slice_chars`, `parse_int`, and `parse_bool`
- explicit formatting helpers `to_string` for `Int`, `Bool`, and `String`
- `std::json` parse/encode, integer-number conversion, value/object-field scalar/composite accessor/default/required helpers, scalar array projection helpers, direct scalar-array object-field helpers, typed-segment JSON path helpers, typed JSON path scalar projection helpers, and typed JSON path collection projection helpers that return `json::Error`, plus pure `std::string` text assembly helpers (`string::concat_all` / `string::join`) and `std::fmt` formatting helpers (`repeat` / `pad_left` / `pad_right` / `truncate_chars` / `format_values`) over explicit `String` values
- `Option::Some`, `Option::None`, `Result::Ok`, `Result::Err`, and exhaustive `match` for compiler-known `Option` and `Result`
- user-defined `enum` declarations with optional unconstrained type parameters, zero-payload and one-payload variants, qualified construction/patterns, payload discard `_` inside one-payload variant patterns, exhaustive `match`, typed HIR, VM execution, and in-memory package interface summaries
- prefix `try expr` propagation for `Result[T, E]` with exact error-type matching
- `else if` chains, explicit `return expr` from the nearest named or anonymous function, `break` / `continue` for loops, and `for item in list` over `List[T]`
- enum diagnostics, package enum visibility coverage, imported `alias::Enum::Variant` constructors/patterns, package enum call-target identity, and stale enum interface validation
- deterministic v4 package interface text persistence with stable artifact package/item IDs, content hashes that ignore diagnostic-only spans, direct dependency metadata, artifact path naming, file round-trip, and loaded-interface validation for public records/functions/enums/opaque types
- loaded package interfaces and discovered `.mgi` artifacts can act as the dependency boundary for downstream typed checking, including transitive public-signature type dependencies, without reading dependency implementation bodies
- independently generated `.mgi` artifacts are remapped from stable artifact identities into a fresh session-local package/item identity namespace when loaded together, so one artifact root can safely contain artifacts from separate provider builds
- package check cache keys combine entry package source hashes with loaded direct/transitive dependency interface hashes, and `.mgc` check artifacts are rejected when missing or stale
- `muga check --artifact-root <dir>` consumes `.mgi` and `.mgc` artifacts for dependency-body-free package checking
- `muga emit-interface` and `muga emit-artifacts` write reachable `.mgi` interfaces from package-aware typed HIR, and `emit-artifacts` also writes reachable `.mgb` implementation artifacts containing MIR-lowered bytecode programs plus the entry `.mgc` check cache; lower-level `emit-check-cache` validates against `.mgi` artifacts before writing `.mgc`; `emit-interface --format json`, `emit-check-cache --format json`, and `emit-artifacts --format json` expose artifact root, artifact kind, path, and URI as schema-versioned JSON for editor, LSP, CI, and agent consumers
- `muga build <entry>` writes the same `.mgi` / `.mgb` / `.mgc` artifact set to the default `.muga/build` directory, preserves unchanged generated artifacts instead of rewriting them, reports each artifact as `written` or `reused`, and builds independent package artifacts in the same dependency level concurrently with deterministic output ordering
- `muga build --format json <entry>` exposes the same artifact root, artifact kind, path, URI, and written/reused status as one schema-versioned JSON object for editor, LSP, CI, and agent consumers
- `muga why-rebuild [--format text|json] [--artifact-root <dir>|--built] <entry>` exposes non-mutating `.mgi` / `.mgc` / `.mgb` artifact states plus manifest lockfile and local archive-cache metadata, with compact tab-separated text output for humans and artifact-file, artifact-hash, and regeneration-command data as schema-versioned JSON for editor, LSP, CI, and agent consumers
- `.mgb` implementation artifacts record each package's source hash separately from public interface and dependency interface hashes
- public `.mgi` interface hashes remain stable across implementation-only body and diagnostic-span changes
- manifest `muga build` writes and validates a minimal `muga.lock` with local path dependency source descriptors plus SHA-256 `source_hash` metadata and local archive dependency descriptors plus `hash` metadata; well-formed stale metadata is refreshed, while malformed or unsupported lockfiles are rejected with `PK026`
- library package content hashing computes `sha256:<hex>` over `muga.toml`, sorted `.muga` files under the manifest source root, and sorted files under an optional manifest-declared `[package] resources = "resources"` root, using the future published-package hash shape without adding network dependency forms
- `muga emit-package-archive [--format text|json] --archive-root <dir> <entry>` writes a deterministic `.mgp` source/resource archive from that canonical content input, skipping tool metadata directories and non-source files, and `--dependency-snippet` prints a pasteable local archive dependency entry
- library archive readback validates `.mgp` bytes against optional expected `sha256:<hex>` values, parses manifest/source/resource entries without trusting filenames, preserves arbitrary resource bytes, and rejects malformed layout, duplicate or unsorted paths, non-UTF-8 manifest/source entries, source/resource-root escapes, undeclared resources, tool metadata, and non-source entries
- `muga verify-package-archive [--format text|json] [--expected-hash sha256:<hex>] <archive-file>` validates generated hash-bearing `.mgp` filenames or explicit expected hashes, archive bytes, and manifest/source/resource entries without materializing files or mutating caches
- `muga unpack-package-archive [--format text|json] [--expected-hash sha256:<hex>] --output-dir <dir> <archive-file>` and library archive materialization write validated `.mgp` bytes to an absent or empty local source/resource tree, preserve the reported content hash, write declared resources under the manifest resource root, reject unsafe manifest source/resource roots, and reject non-empty destinations
- local `.mgp` archive dependencies use `[dependencies] name = { archive = "...", hash = "sha256:<hex>" }`, validate the archive bytes, materialize or reuse `.muga/packages` cache entries including declared resources, reject malformed forms, stale or colliding caches, and package-name mismatches, and write/validate minimal lockfile `hash` metadata
- `muga check --built <entry>` and `muga run --built <entry>` explicitly consume the default `.muga/build` directory without changing plain source-compatible `check` / `run`
- `muga syntax --format json <entry>` exposes single-file lex/parse diagnostics for faster editor, LSP, CI, and agent feedback without running resolver, typechecker, import loading, or artifact checks
- `muga explain <diagnostic-code>` exposes the documented diagnostic catalog entry or stable diagnostic-code family from `errors.md` for terminal users and agent workflows
- CLI JSON compiler diagnostics include entry source context in `diagnostics[].context`, giving editor, LSP, CI, and agent consumers a per-diagnostic source path and `file://` URI; artifact-backed `check --format json` diagnostics also include entry package, artifact-root, and concrete artifact-file context when available
- `muga run --format json <entry>` exposes captured program stdout, captured program stderr, the returned `main` value when present, and compiler/runtime diagnostics as schema-versioned JSON for editor, LSP, CI, and agent consumers
- `muga test --format json <entry>` exposes discovered test names, pass/fail status, failure messages, per-test stdout/stderr, summary counts, and compiler diagnostics as schema-versioned JSON for editor, LSP, CI, and agent consumers
- `muga metadata --format json <entry>` exposes package/module/item/export metadata plus public interface docs and rendered types for editor, LSP, CI, and agent consumers
- `muga workspace --format json <entry>` exposes loaded packages, module source files, the default artifact root, manifest/source/resource roots, and dependency edges reachable from an entrypoint for editor, LSP, CI, and agent consumers
- `muga hover --format json --line <line> --column <column> <entry>` exposes declaration hover data with public docs and signatures for editor, LSP, CI, and agent consumers
- `muga completions --format json <entry>` exposes visible package/interface completions with import aliases plus public docs and signatures for editor, LSP, CI, and agent consumers
- `muga definition --format json --line <line> --column <column> <entry>` exposes go-to-definition data for import aliases, local bindings, and package/interface item references for editor, LSP, CI, and agent consumers
- `muga references --format json --line <line> --column <column> <entry>` exposes find references data for import aliases, local bindings, and package/interface item references in the entry module for editor, LSP, CI, and agent consumers
- `docs/editor-json-workflow.md` and `json_backed_editor_workflow_uses_existing_command_contracts` validate a concrete editor adapter flow across syntax, check, workspace, metadata, hover, completions, definition, references, run, and test JSON without scraping human output
- `muga why-rebuild [--format text|json] [--artifact-root <dir>|--built] <entry>` implements the first non-mutating artifact/cache explanation contract for `.mgi`, `.mgc`, `.mgb`, manifest lockfile metadata, local archive-cache metadata states, and implementation dependency-interface set changes before editor or agent tools depend on rebuild reasoning
- `muga run --built <entry> -- args...` passes program arguments through `std::env::args()`, CLI usage separates `check` from `run` so it does not imply `check` accepts program arguments, and missing default build artifacts under `--built` point users at `muga build <entry>`
- `muga run --artifact-root <dir>` validates `.mgi` / `.mgc` / structurally checked `.mgb` artifacts, executes direct and transitive dependencies without reading dependency source files from the source tree, remaps independently generated implementation item references onto loaded interface identities, and rejects wrong-package, dependency-interface-mismatched, or dependency-interface-set-changed `.mgb` files with artifact hash and regeneration-command context
- `muga build` reuse output and lockfile update behavior are covered for local path and local archive dependencies after dependency implementation-only edits, public signature edits, archive content updates, and malformed lockfiles
- recursive annotation diagnostics now point direct recursion at parameter/return annotations and mutual recursion at explicit signatures for every function in the group
- package-mode public signatures now have representative coverage for every v1-supported public type shape through in-memory and persisted interfaces
- The stdlib package docs and samples review now covers `std::io`, `std::fs`, `std::path`, `std::env`, `std::cli`, `std::time`, `std::string`, `std::fmt`, and the first `std::json` slice, including artifact-backed execution samples where useful; the release gate and GitHub Actions are now aligned through `scripts/v1-release-gate.sh`; `muga shell-completions <bash|zsh|fish>` and `muga doctor [--format text|json]` have landed as a tool-only adoption surface; the first `std::json` slice is implemented from `docs/std-json-first-slice.md` and audited in `docs/std-json-implementation-audit.md`; the post-JSON stdlib/API boundary selection in `docs/post-json-stdlib-boundary-selection.md` chooses opaque resource-handle design before broader runtime-backed stdlib APIs; `docs/opaque-resource-handles.md` defines that boundary; the first `pub opaque type` interface slice now provides `.mgi` identity, editor/doc tooling, and downstream loaded-interface checking for public opaque names; the metadata-only `OpaqueHandleFacts` / `paramMode` interface slice, consuming checker, first runtime file-handle design, and read-only `std::fs::File` runtime implementation feed the post-file-handle selection in `docs/post-file-handle-resource-surface-selection.md`; and the selected scalar program stderr channel is now implemented through `eprint` / `eprintln`, with text output file handles implemented from `docs/text-output-file-handles.md` and demonstrated by the integrated `report_app` workflow before cancellation, stdout/stderr handles, or broader handle values
- `samples/projects/report_app` now demonstrates args/env, stdout/stderr, text-file handle writes, JSON run output, `Result`, local dependencies, artifact-backed execution, and `run --built` in one focused workflow.
- The implemented `report_app` workflow exposed the resource-cleanup gap, and [docs/lexical-resource-cleanup.md](./docs/lexical-resource-cleanup.md) records the statement-form `using` contract before `Bytes`, formatting templates, stdout/stderr handles, process APIs, network APIs, streaming APIs, or broader host effects.
- [docs/lexical-resource-cleanup.md](./docs/lexical-resource-cleanup.md) records the implemented statement-form `using` cleanup path for runtime-backed opaque handles, including nested cleanup unwind hardening; broader IO/resource APIs remain separate decisions.
- Minimal pure `std::cli` helpers over explicit `List[String]` values are implemented and covered by std package source, `std_cli` samples, `report_app`, and examples tests before `Bytes`, formatting templates, process APIs, network APIs, streams, or broader host effects.
- `std::cli` now provides `cli::positional`, `cli::positional_or`, `cli::has_flag`, `cli::option`, `cli::option_or`, repeated option value helpers, and typed scalar `Int` / `Bool` parsing helpers, with source/artifact coverage and the `report_app` / `config_app` samples refreshed to use it.
- The CLI-first generated app template uses `std::env` and `std::cli` before richer CLI parsers, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects; its behavior is covered by the project-template source, generated-project tests, and onboarding examples.
- Typed scalar `std::cli` parsing helpers for `Int` and `Bool` are implemented and covered by the stdlib package source, `std_cli` sample, and examples tests before full CLI parser schemas, config-file loading, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- `std::json` value and object-field accessor helpers are implemented and covered by the std package source, `samples/packages/app/std_json`, and examples tests, returning `json::Error` for wrong shapes before config-file loading, schema decoding, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- `samples/projects/config_app` implements the JSON config workflow sample with `std::config`, `std::json`, `std::env`, `std::cli`, and `std::result::map_err`, preserving explicit CLI > config > defaults precedence before TOML, broader schema tooling, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- [docs/post-config-workflow-adoption-gap-selection.md](./docs/post-config-workflow-adoption-gap-selection.md) selects the implemented `config_app` refresh that uses existing `std::result::map_err` for app-boundary error normalization before new error unions, `std::config`, TOML, schema decoding, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- Narrow pure `std::string` text assembly helpers (`string::concat_all` / `string::join`) are implemented and covered by `src/std_package.rs`, `samples/packages/app/std_string`, `samples/projects/config_app`, and examples tests before formatting templates, interpolation, builders, broader config/schema work, full CLI parser schemas, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- [docs/std-fmt-text-layout.md](./docs/std-fmt-text-layout.md) adds narrow pure `std::fmt` helpers for repeat, left/right padding, scalar-value truncation, and explicit `{}` placeholder substitution before language interpolation syntax, format specifiers, localization, or builders.
- Narrow `std::json` required object-field helpers are implemented and covered by the std package source, `samples/packages/app/std_json`, and examples tests before broader `std::config`, TOML, schema tooling, full CLI parser schemas, formatting templates, interpolation, `std::fmt`, builders, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- Narrow `std::json` array/object field helpers are implemented and covered by the std package source, `samples/packages/app/std_json`, and examples tests before JSON paths, broader config/schema work, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- `samples/projects/config_app` now covers the nested JSON config workflow with composite/typed `std::json` helpers for `tags`, owner metadata, servers, and limits before JSON paths, broader schema decoding, `std::config` expansion, TOML, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- Pure `std::json` scalar array projection helpers are implemented and covered by code, `samples/packages/app/std_json`, `samples/projects/config_app`, and examples tests before JSON paths, schema decoding, broader object-field matrices, `std::config` expansion, TOML, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- Direct `std::json` scalar-array object-field helpers are implemented and covered by code, samples, and tests before JSON paths, schema decoding, `std::config` expansion, TOML, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- Repeated `std::cli` option value helpers are implemented and covered by code, samples, and tests before JSON paths, schema decoding, `std::config`, TOML, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- `std::json` path helpers are implemented and covered by code, samples, and tests before schema decoding, `std::config`, TOML, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- Typed `std::json` path scalar projection helpers are implemented and covered by code, samples, and tests before typed array/object path helpers, schema decoding, `std::config`, TOML, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- Typed `std::json` path collection projection helpers are implemented and covered by code, samples, and tests before schema decoding, `std::config`, TOML, full CLI parser schemas, generated config app templates, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- [docs/json-schema-decoding.md](./docs/json-schema-decoding.md) selects and implements a compiler-owned `json::decode_or[T](value, fallback)` default-overlay decoder before required `json::decode[T]`, `std::config`, TOML, full CLI parser schemas, generated config app templates, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- [docs/std-config-json-loading.md](./docs/std-config-json-loading.md) records the implemented compiler-owned `std::config::load_json_or[T](path, fallback)` and strict `std::config::load_json[T](path)` helpers with public config errors, compiler-lowered schemas, artifact-backed execution, and `config_app` coverage before choosing TOML, config discovery, generated config app template expansion, or broader host effects.
- The generated `muga new --template config-app` starter is implemented and covered by templates, samples, and tests before TOML, full CLI parser schemas, formatting templates, broader decoder targets, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- [docs/config-path-discovery.md](./docs/config-path-discovery.md) implements generated config-app path discovery as `--config` > `MUGA_CONFIG_PATH` > generated JSON default, keeping CLI > config > defaults explicit before TOML parsing, package resource lookup, service manifests, or runtime-owned config precedence.
- [docs/workspace-manifest-metadata.md](./docs/workspace-manifest-metadata.md) extends `muga workspace --format json` with manifest root, source root, resource root, package path, direct dependency, and dependency source/resource metadata so project-aware tooling can derive config/resource paths before runtime lookup and installed layouts.
- [docs/config-app-run-helper.md](./docs/config-app-run-helper.md) adds a generated config-app README plus `scripts/run-with-config.sh` and `scripts/package-config-app.sh` helpers using `MUGA_BIN` and `MUGA_CONFIG_PATH`, so first-run config workflows and source-free package handoff work from any current directory before runtime-owned config discovery, TOML parsing, or shell-profile mutation.
- [docs/package-resource-archives.md](./docs/package-resource-archives.md) adds manifest-declared text/binary package resource inclusion for hashes, `.mgp` archives, `unpack-package-archive` materialization, and local archive dependency caches before binary writes, streams, codecs, mutable buffers, or broader cryptographic APIs.
- [docs/runtime-package-resource-lookup.md](./docs/runtime-package-resource-lookup.md) adds read-only `std::fs::read_resource_text(package, path)` and `std::fs::read_resource_bytes(package, path)` lookup for manifest-declared resources in source, test, local archive dependency, and explicit built-artifact runs.
- [docs/binary-file-read.md](./docs/binary-file-read.md) adds read-only `std::fs::read_bytes`, `std::fs::read_bytes_path`, and `bytes::at` for local binary file inspection before binary writes, streams, codecs, mutable buffers, or broader cryptographic APIs.
- [docs/binary-file-write.md](./docs/binary-file-write.md) adds full-file `std::fs::write_bytes` and `std::fs::write_bytes_path` over opaque `Bytes` before binary file handles, streams, codecs, mutable buffers, or recursive export policy.
- [docs/bytes-sha256-hash.md](./docs/bytes-sha256-hash.md) adds `std::hash::sha256_hex(bytes)` for file/resource verification before streaming hash handles, HMAC, signatures, KDFs, checksum families, or broader cryptographic APIs.
- [docs/path-normalize.md](./docs/path-normalize.md) adds `std::path::normalize(path)` as a pure lexical cleanup helper before symlink policy, strict path validation, sandbox containment, or host path resolution.
- [docs/path-with-file-name.md](./docs/path-with-file-name.md) adds `std::path::with_file_name(path, new_file_name)` as a pure sibling output path helper before strict path component validation, canonicalization, symlink policy, or host path resolution.
- [docs/path-with-extension.md](./docs/path-with-extension.md) adds `std::path::with_extension(path, new_extension)` as a pure output/sidecar path helper before canonicalization, symlink policy, or host path resolution.
- [docs/path-strip-prefix.md](./docs/path-strip-prefix.md) adds `std::path::strip_prefix(path, base)` as a pure component-aware relative path helper before lexical normalization, symlink policy, sandbox containment, or host path resolution.
- [docs/fs-canonicalize-path.md](./docs/fs-canonicalize-path.md) adds `std::fs::canonicalize_path(target_path)` as a recoverable existing-path host resolution helper before pure lexical normalization, project-root lookup, config discovery, unique temp-file policy, or symlink-specific controls.
- [docs/env-current-dir.md](./docs/env-current-dir.md) adds `std::env::current_dir()` as an explicit `Result[path::Path, io::IOError]` current-directory read before temp-file allocation, canonicalization, project-root lookup, runtime-owned config discovery, or process execution.
- [docs/env-temp-dir.md](./docs/env-temp-dir.md) adds `std::env::temp_dir()` as an explicit `Result[path::Path, io::IOError]` host temporary-directory read before unique temp-file allocation, cleanup policy, sandbox containment, or process execution.
- [docs/fs-file-size.md](./docs/fs-file-size.md) adds `std::fs::file_size_path(path)` as a narrow scalar metadata helper before all-path metadata records, accessed/created timestamps, permissions, symlink policy, or recursive directory sizing.
- [docs/fs-modified-unix-millis.md](./docs/fs-modified-unix-millis.md) adds `std::fs::modified_unix_millis_path(target_path)` as a narrow last-modified timestamp helper before all-path metadata records, broader timestamp APIs, or symlink-specific controls.
- [docs/fs-file-metadata-record.md](./docs/fs-file-metadata-record.md) adds `std::fs::FileMetadata` plus `std::fs::file_metadata_path(file_path)` as a regular-file metadata record before all-path metadata, directory metadata, accessed/created timestamps, permissions, symlink policy, or recursive traversal APIs.
- [docs/fs-path-status.md](./docs/fs-path-status.md), [docs/fs-path-info.md](./docs/fs-path-info.md), [docs/fs-path-metadata.md](./docs/fs-path-metadata.md), and [docs/fs-path-size-metadata.md](./docs/fs-path-size-metadata.md) add `std::fs::PathStatus`, `PathKind`, `PathInfo`, `PathMetadata`, and `PathSizeMetadata` plus `path_status`, `path_kind`, `path_info`, `path_metadata_path`, and `path_size_metadata_path` as grouping layers over existing metadata predicates, modified-time metadata, and optional regular-file size before recursive directory sizing, symlink classification, permissions, or broader metadata fields.
- [docs/fs-read-dir-recursive.md](./docs/fs-read-dir-recursive.md) adds `std::fs::read_dir_recursive_path(root_path)` as a deterministic read-only traversal helper without mixing aggregate metadata, destructive operations, directory copy, globbing, symlink classification, or sandbox policy into the listing API.
- [docs/fs-directory-size-metadata.md](./docs/fs-directory-size-metadata.md) adds `std::fs::DirectorySizeMetadata` and `directory_size_metadata_path(root_path)` as a deterministic read-only recursive aggregate without mixing destructive behavior, globbing, public symlink classification, or sandbox policy into the aggregate API.
- [docs/fs-remove-dir-all.md](./docs/fs-remove-dir-all.md) adds `std::fs::remove_dir_all_path(dir_path)` as the first destructive recursive directory helper before trash/recycle-bin policy, globbing, or sandbox containment.
- [docs/fs-copy-dir-all.md](./docs/fs-copy-dir-all.md) adds `std::fs::copy_dir_all_path(from, to)` as a no-overwrite recursive directory copy helper before merge/overwrite policy, rollback, globbing, or sandbox containment.
- [docs/fs-move-dir-all.md](./docs/fs-move-dir-all.md) adds `std::fs::move_dir_all_path(from, to)` as a no-overwrite copy-then-remove recursive directory move helper before host-rename acceleration, rollback, merge/overwrite policy, globbing, or sandbox containment.
- [docs/fs-rename-path.md](./docs/fs-rename-path.md) adds `std::fs::rename_path(from, to)` as a narrow two-path filesystem helper without directory-copy semantics, copy/delete fallback, cross-device fallback, or broader mutation policy.
- [docs/installed-app-bundles.md](./docs/installed-app-bundles.md) adds `muga emit-app-bundle --format json --source-free`, `muga run-app-bundle`, `muga install-app --format json --replace-owned`, `muga list-installed-apps`, `muga uninstall-app --format json`, `muga emit-app-completions --format json`, `muga emit-app-archive --format json`, and `muga unpack-app-archive [--format text|json] [--expected-hash sha256:<hex>]` for non-mutating app bundles with copied resources, bundle-local dependencies, `.muga/build` artifacts, `muga.lock`, source-free emitted layouts, source-free artifact execution, user-chosen launcher placement, install ownership metadata, guarded owned updates/uninstalls, installed-app inventory, generated helper install/list hooks, source-free completion package emission, machine-readable bundle/install/completion/archive emission/unpack, and deterministic `.mga` transport for manifest projects.
- [docs/json-required-decoding.md](./docs/json-required-decoding.md) records the implemented required `json::decode[T](value)` strict JSON boundary before TOML, broader decoder target types, full CLI parser schemas, formatting templates, config discovery, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- [docs/json-required-decoding.md](./docs/json-required-decoding.md) designs and implements strict `json::decode[T](value)` with expected `Result[T, json::Error]` target typing, required record fields, no-fallback schema lowering through `DecodeJsonRequired`, artifact-safe payloads, source/artifact coverage, and explicit deferrals for TOML, broader decoder target types, full CLI parser schemas, formatting templates, and host effects.
- [docs/json-decoder-target-expansion.md](./docs/json-decoder-target-expansion.md) designs and implements the decoder target expansion for `Option[T]`, recursive `List[T]`, typed `Map[String, T]`, and concrete non-generic enums, including zero-payload string tags and one-payload single-key objects, with null/missing/default-overlay semantics, artifact schema payloads, diagnostics, source/artifact/`run --built` coverage, and explicit deferral of generic enum decoding, TOML, full CLI parser schemas, formatting templates, config discovery, and host effects.
- The implemented `config_app` sample and generated `muga new --template config-app` starter carry the structural config workflow, using expanded decoder targets for `Option[String]`, nested records, `List[Record]`, and typed `Map[String, Int]` settings before TOML, full CLI parser schemas, formatting templates, config discovery, and host effects.
- [docs/json-config-schema-polish.md](./docs/json-config-schema-polish.md) implements `@json(rename: "...")` on record fields and enum variants as the first JSON/config schema-polish slice before validation attributes, TOML, full CLI parser schemas, schema generation, generic decoding, or host effects.
- [docs/json-config-strict-unknown-fields.md](./docs/json-config-strict-unknown-fields.md) implements record-level `@json(deny_unknown_fields)`, accepted wire-key semantics, `.mgi` record flags, and `RF` decoder artifact tokens as the next JSON/config trust slice before validation attributes, TOML, full CLI parser schemas, schema generation, generic decoding, or host effects.
- [docs/json-config-alias-metadata.md](./docs/json-config-alias-metadata.md) implements field and enum-variant `@json(alias: "...")` metadata, accepted-name conflict rules, strict unknown-field interaction, `.mgi` v7 metadata, and `RG`/`EG` decoder artifact tokens before validation attributes, TOML, full CLI parser schemas, schema generation, generic decoding, or host effects.
- [docs/json-config-validation-attributes.md](./docs/json-config-validation-attributes.md) implements the post-alias trust slice: field-level `@validate(...)` metadata with scalar string/int validators, path-aware validation errors, `.mgi` v8 metadata, and `RV` decoder artifact tokens before TOML, full CLI parser schemas, schema/client generation, generic decoding, or host effects.
- [docs/json-config-schema-export.md](./docs/json-config-schema-export.md) implements the post-validation adoption slice: `muga schema --format json` for JSON Schema Draft 2020-12 output with Muga `x-muga` extensions, required/overlay decode modes, concrete public record/enum scope, validation keywords, alias metadata, and loaded-interface package coverage.
- [docs/json-typed-encoding.md](./docs/json-typed-encoding.md) implements the post-schema-export typed JSON encoding and bidirectional contract slice: compiler-owned `json::to_value[T](value)` plus `json::encode_typed[T](value)`, canonical primary wire-name output, omitted optional record fields, enum output matching decode/schema export, validation-on-encode, artifact schema behavior, and explicit source/interface coverage.
- [docs/cli-parser-schema.md](./docs/cli-parser-schema.md) implements the post-typed-JSON practical app slice: the first compiler-owned `cli::parse_or[T](args, defaults)` and `cli::usage_for[T](program, defaults)` typed CLI schema boundary for concrete non-generic record overlays. The `config-app` sample and project template now use `cli::parse_or[T]` for CLI > config > defaults settings overlays and expose `cli::usage_for[T]` through a `--help` path before TOML, strict no-default parsing, subcommands, short flags, config discovery automation, full client generation, or host effects.
- [docs/cli-field-metadata.md](./docs/cli-field-metadata.md) implements field-level `@cli(name: "...", alias: "...", help: "...", hidden)` plus a dedicated `CliSchema` artifact boundary. The generated `config-app` settings use field help plus `--tag` / `--tags` metadata before TOML, config discovery automation, strict no-default parsing, full client generation, or host effects.
- [docs/strict-cli-parser-schema.md](./docs/strict-cli-parser-schema.md) implements compiler-owned `cli::parse[T](args)` for command-line-only records with expected-result type inference, `MissingArgument` errors, absent `Bool`/`Option`/`List` synthesis, strict unsupported-field rejection, and source/artifact/`run --built` coverage before TOML, config discovery automation, combined short flags, attached values, subcommands, full client generation, generic encoding/decoding, broader validators, or host effects.
- The checked-in strict CLI sample at [samples/projects/cli_tool/src/main/main.muga](./samples/projects/cli_tool/src/main/main.muga) adopts `cli::parse_request[T]` with a strict root command, typed subcommands, generated help, compact short options, and shell-completion coverage before TOML, config discovery automation, full client generation, generic encoding/decoding, broader validators, or host effects.
- Generated `muga new --template cli-tool` adoption is implemented from the same strict CLI shape, with source/build/`run --built`, README, completion helper, and packaging helper coverage.
- [docs/strict-cli-no-default-usage.md](./docs/strict-cli-no-default-usage.md) implements `cli::usage_for_required[T](program)` with explicit call type arguments as the generated strict usage helper, replacing the historical strict CLI manual help duplication before command metadata, combined short flags, attached values, subcommands, TOML, config discovery automation, full client generation, or host effects.
- [docs/cli-command-metadata.md](./docs/cli-command-metadata.md) implements record-level `@cli(about: "...")` command summaries in generated usage before short options, subcommands, TOML, config discovery automation, full client generation, or host effects.
- [docs/cli-short-option-metadata.md](./docs/cli-short-option-metadata.md) implements field-level `@cli(short: "x")` short options as the next CLI ergonomics slice before built-in help branching, positionals, subcommands, TOML, config discovery automation, shell completion generation, full client generation, or host effects.
- [docs/cli-short-option-metadata.md](./docs/cli-short-option-metadata.md) implements field-level `@cli(short: "x")` for typed CLI schemas, including exact short parser forms, `-x, --long` usage rendering, app-owned `cli::has_short_flag(args, "h")`, and interface/artifact-compatible schema payloads.
- [docs/post-cli-short-option-metadata-adoption-gap-selection.md](./docs/post-cli-short-option-metadata-adoption-gap-selection.md) selects typed CLI positional field metadata design next, so typed CLI schemas can model primary operands before combined short flags, attached values, built-in help branching, subcommands, TOML, config discovery automation, shell completion generation, full client generation, or host effects.
- [docs/cli-positional-field-metadata.md](./docs/cli-positional-field-metadata.md) implements field-level `@cli(positional: N)` with explicit 1-based indexes, generated positional usage, source/interface/artifact persistence, and strict `cli-tool` template adoption.
- [docs/post-cli-positional-field-metadata-adoption-gap-selection.md](./docs/post-cli-positional-field-metadata-adoption-gap-selection.md) selects the built-in CLI help policy in [docs/cli-built-in-help-policy.md](./docs/cli-built-in-help-policy.md), which led to `cli::help_requested` and generated help helpers after positional operands landed.
- [docs/cli-built-in-help-policy.md](./docs/cli-built-in-help-policy.md) implements `cli::help_requested`, `cli::help_for`, and `cli::help_for_required`, including `--`-aware detection, schema-backed `-h, --help` rendering, conflict diagnostics, artifact-backed execution, and generated config/strict CLI template adoption before parse-integrated help result enums, combined short flags, attached values, subcommands, shell completions, TOML/config discovery automation, full client generation, or host effects.
- [docs/post-built-in-cli-help-helper-adoption-gap-selection.md](./docs/post-built-in-cli-help-helper-adoption-gap-selection.md) selected parse-integrated CLI help workflow design, keeping runtime-owned printing/exits deferred while preparing generated starters to match a typed help-or-parsed request.
- [docs/parse-integrated-cli-help-workflow.md](./docs/parse-integrated-cli-help-workflow.md) implements `cli::Request[T]`, `cli::parse_request[T]`, and `cli::parse_request_or[T]` across strict/config starters before runtime-owned printing/exits, subcommands, shell completions, TOML/config discovery automation, full client generation, or host effects.
- [docs/post-parse-integrated-cli-help-workflow-adoption-gap-selection.md](./docs/post-parse-integrated-cli-help-workflow-adoption-gap-selection.md) audits request workflow adoption and selects compact CLI short option syntax design next, so combined bool flags and attached short values can be specified before implementation.
- [docs/compact-cli-short-option-syntax.md](./docs/compact-cli-short-option-syntax.md) implements compact short tokens such as `-abc`, `-ofile`, and `-abo=value` as runtime parser behavior over existing short metadata before subcommands, generated app shell completions, TOML/config discovery automation, or runtime-owned printing/exits.
- [docs/post-compact-cli-short-option-syntax-adoption-gap-selection.md](./docs/post-compact-cli-short-option-syntax-adoption-gap-selection.md) audits compact short syntax adoption and selected CLI subcommand metadata design; [docs/cli-subcommand-metadata.md](./docs/cli-subcommand-metadata.md) now implements enum/variant metadata plus strict command enum schemas through source validation, `.mgi` package interfaces, `.mgb` schema payloads, recursive runtime dispatch/help, artifact-backed execution, and `run --built` before wrapper-record root/global options and generated app shell completions.
- [docs/post-cli-subcommand-schema-adoption-gap-selection.md](./docs/post-cli-subcommand-schema-adoption-gap-selection.md) adopts command enum schemas in [samples/projects/cli_tool/src/main/main.muga](./samples/projects/cli_tool/src/main/main.muga) and generated `muga new --template cli-tool` starters with `run` / `inspect` subcommands while preserving compact short options, validation, generated root/leaf help, artifact-backed execution, and recoverable `cli::Error` mapping.
- [docs/cli-wrapper-root-options.md](./docs/cli-wrapper-root-options.md) implements strict wrapper records with one `@cli(subcommand)` field for root/global options, including schema/artifact support, runtime parse/help, source/artifact/`run --built` coverage, and strict sample/generated `cli-tool` adoption with `--profile` / `-p`.
- [docs/cli-schema-shell-completions.md](./docs/cli-schema-shell-completions.md) implements a schema-backed generated app completion surface as `muga cli-completions <bash|zsh|fish> --program <name> --type <Type> ...`, driven by wrapper, command, option, alias, short-option, enum-value, and positional `CliSchema` data across source, `--artifact-root`, and `--built` workflows while keeping `muga shell-completions` static for the `muga` developer tool.
- [docs/post-cli-schema-shell-completion-adoption-gap-selection.md](./docs/post-cli-schema-shell-completion-adoption-gap-selection.md) audits generated app completion adoption and implements install documentation, a generated `cli-tool` README, and a generated `scripts/generate-completions.sh` packaging hook before richer completion contracts and installer integration.
- [docs/cli-completion-json-spec.md](./docs/cli-completion-json-spec.md) implements `muga cli-completions --format json --program <name> --type <Type> ...` as the shell-agnostic generated-app completion contract, exposing recursive wrapper, command, record, option, positional, alias, short-option, enum, Bool candidate, and static file/directory value-source data from the same `CliSchema` source/artifact/`--built` workflows. The shell renderers now traverse nested command scopes recursively and use `@cli(value_source: "file"|"directory")` for path-valued options before future TOML/config discovery, dynamic completion producers, or installer integration.
- [docs/cli-completion-installer-integration.md](./docs/cli-completion-installer-integration.md) implements non-mutating generated app completion package emission as `muga emit-cli-completions --format json --output-dir <dir> --program <name> --type <Type> ...`, writing bash, zsh, fish, and `.completions.json` artifacts with text or JSON metadata output before shell-profile installation, package-manager-specific installers, TOML/config discovery, or dynamic completion producers.
- `Map.empty`, `contains`, `get`, `insert`, and `remove` for `Int`, `Bool`, and `String` keys
- file-based package mode with `package`, `import`, `pkg`, `pub`, `as`, module-private top-level items, and `alias::Name`
- minimal `muga.toml` project mode with `[package] name/source`, local path dependencies through `[dependencies] name = { path = "..." }`, and local archive dependencies through `[dependencies] name = { archive = "...", hash = "sha256:<hex>" }`
- unflattened package graph loading preserves package files plus package/module/item/export metadata before the legacy flattening path
- a library-only package-aware check path validates package boundary, import, visibility, and public-signature rules from the unflattened package graph before package-aware module checking
- package-aware source and per-module signature environments resolve record/enum/function signatures from the unflattened graph while preserving package item identity and module/same-package/import visibility
- package-aware module body resolution/typechecking consumes those module signature environments, and the package-aware API retains per-module resolver/typecheck outputs plus typed HIR programs
- package-aware check results aggregate per-module typed HIR from unflattened module check outputs with remapped local IDs and symbols instead of using the legacy flattened typed path
- default package `check` runs the package-aware validation path and no longer reloads a flattened package AST after validation
- default package `compile_typed_path` returns the package-aware typed HIR aggregate instead of the legacy flattened typed HIR
- flattened package loader APIs are explicitly named `load_flattened_*` so compatibility AST use is visible at call sites
- package-aware checking and loaded/interface-artifact typed compilation collect dependency signatures and build dependency graph metadata directly from loaded interfaces without reading dependency source bodies, and `muga check --artifact-root` plus interface artifact emission use package-aware paths
- the legacy interface-stub flattened typed compilation path has been removed; loaded/interface-artifact typed compilation now uses the package-aware semantic path only
- package-aware typed HIR lowers through the initial MIR module before VM bytecode generation for package records, enums, functions, and calls
- in-memory package interface summaries for public records/enums/opaque types/functions and validation of public package references against those summaries

Current architectural gaps:

- `pub fn` still requires explicit public signatures
- runtime-backed standard-library resource handles beyond text `std::fs::File` and opaque `Bytes`, binary write handles, host-rename-accelerated directory moves, broader mutation APIs, stdout/stderr handles, richer time/process APIs, and richer stdlib package families are not implemented
- normal package execution still reads dependency source bodies when no artifact root is supplied; explicit artifact-backed execution is available for dependency-source-tree-free runs
- remaining package work is explicit artifact workflow documentation/sample hardening and later normal project/artifact integration; package-aware checking is now the default package validation path
- project-mode artifact-root config, URL/Git/registry dependency forms, remote package fetching, publishing/install workflows, full published-package lockfile enforcement, registries, binary streams/codecs, broader cryptographic APIs, and full incremental package artifact reuse are not implemented; `muga build` emits default `.muga/build` artifacts with unchanged-artifact reuse, dependency-level parallel package artifact builds, and minimal validated local path/archive dependency lockfiles, library content hashing can compute the first future package identity hash over sources plus declared resources, `emit-package-archive` can write deterministic `.mgp` source/resource archives, pasteable local archive dependency snippets, and JSON archive metadata, library readback can validate those archive bytes, `unpack-package-archive` and library materialization can unpack validated archives into absent or empty local source/resource trees with JSON unpack metadata, local archive dependencies can consume `.mgp` files through `.muga/packages` with focused failure-case hardening, `std::fs::read_resource_text` can read declared UTF-8 resources, `std::fs::read_resource_bytes` can read opaque resource `Bytes`, `std::fs::read_bytes` can read local binary files, `std::fs::write_bytes` can write opaque `Bytes` to local files, and `std::hash::sha256_hex` can hash `Bytes` at runtime, `emit-app-bundle --source-free` can write an artifact-run app layout with bundle-local dependencies and JSON bundle metadata, `install-app --format json --replace-owned` can place/update an ownership-verified wrapper and metadata, `list-installed-apps` can report owned launcher state, `uninstall-app --format json` can remove only ownership-verified launcher metadata, `emit-app-completions --format json` can write completion packages from bundle interfaces, `.mga` app archives can transport bundle directories with JSON archive/unpack metadata, and `check --built` / `run --built` consume artifacts explicitly, but ordinary `check` / `run` do not automatically consume built artifacts
- VM bytecode execution now consumes an initial expression-shaped MIR with explicit execution bodies, body terminators, hoisted body-local function definitions, typed binding/package-item identity, typed assignment update mode, runtime names carrying binding/local identity, and slot-backed runtime environments with package function references canonicalized to their defining binding; control-flow-oriented MIR and native lowering are post-v1 unless needed to close a concrete artifact/execution gap
- default compile APIs lower typed HIR into MIR; the old untyped AST-to-HIR compatibility module has been removed

## Settled Direction

These are baseline decisions, not active roadmap questions:

- no classes or inheritance
- data is modeled with `record`; behavior is modeled with functions
- method-like calls are surface syntax over functions
- `::` is package qualification
- module/file-private top-level items are the default in package mode; `pkg` shares inside a package; `pub` exports across packages
- v1 has no trait, interface, protocol, typeclass, or overloaded dispatch declarations
- source values use value semantics; internal sharing/copy elision is an implementation detail
- ordinary Muga code does not use `ref T`, `mut ref T`, address-of, dereference, or raw pointer syntax
- `Option[T]` is canonical optional spelling; `T?` remains reserved, and any future `?.` syntax should be Option-only optional chaining
- fluent `Option` pipelines use ordinary helpers such as `option::map` / `option::and_then` / `option::value_or`
- `Result[T, E]` is the recoverable-error type; implemented propagation uses visible prefix `try expr`, not postfix `expr?`
- future dot-chain `Result` propagation should use postfix keyword syntax `expr.try`, preserving the visible `try` marker inside Muga's chain style
- fluent `Result` value pipelines use ordinary helpers such as `result::map` / `result::and_then`; helpers are not propagation syntax
- package interfaces store resolved public signatures so downstream packages do not infer through dependency bodies

Related design notes:

- v1 release checklist: [docs/v1-release-checklist.md](./docs/v1-release-checklist.md)
- strategic implementation plan: [docs/strategy-and-implementation-plan.md](./docs/strategy-and-implementation-plan.md)
- practical language readiness: [docs/practical-language-readiness.md](./docs/practical-language-readiness.md)
- artifact/cache explanation command: [docs/artifact-cache-explanations.md](./docs/artifact-cache-explanations.md)
- collections: [spec/008-collections.md](./spec/008-collections.md)
- generics: [spec/009-generics.md](./spec/009-generics.md)
- explicit references: [spec/010-references-draft.md](./spec/010-references-draft.md)
- value semantics: [spec/011-value-semantics.md](./spec/011-value-semantics.md)
- protocol-like abstractions: [spec/012-protocols-deferred.md](./spec/012-protocols-deferred.md)
- enums and result handling: [spec/013-enums-results.md](./spec/013-enums-results.md)

## Immediate Priority

The roadmap defines implementation priorities and the intended v1 boundary; it
does not prescribe release timing or a required next version number. While the
language and package specifications are still expected to change, use the v1
checklist as a stability target and let release timing remain a maintainer
decision.

The v1 surface is feature-frozen. That still protects the small source model,
but it no longer means every next slice should be polish. The active priority
is **Core Capability Acceleration**: choose the practical core capability that
most increases Muga's usefulness, then implement it vertically with the same
quality bar as the smaller slices.

Core acceleration priority:

1. `std::process` as the first external-effect spine: child command execution,
   status/stdout/stderr capture, explicit cwd/env options, public error types,
   artifact-backed execution, and runnable samples.
2. Structured task groups: explicit `spawn` / `join`, scoped lifetimes,
   failure propagation, cancellation, and timeout boundaries before channels or
   hidden async.
3. Service IO: socket and minimal HTTP/JSON workflows only after resource and
   task semantics can express shutdown and backpressure.
4. Performance path: control-flow MIR, runtime representation work, and
   benchmark evidence before native backend claims.
5. Distribution path: build on `.mgp` / `.mga`, source-free bundles,
   verification, and install inventory before registry publishing.

The earlier v1 hardening work remains the baseline that accelerated slices must
not break:

1. Keep docs, runnable samples, and specs aligned for short-circuit `and` / `or`, `else if`, explicit `return expr`, `break` / `continue`, `for item in list`, payload discard `_` in enum variant patterns, explicit generic `record` / `fn`, prefix `try expr`, the first `String` helper builtins, explicit `to_string` formatting helpers, `std::fmt` text-layout helpers, `Unit`, the `std::io` / `std::fs` text-file slice, minimal `std::path::Path`, path joining, parent lookup, file-name/stem extraction, extension extraction/replacement, absolute-path classification, Path-aware text-file helpers, directory listing, recursive directory listing, directory size metadata, directory creation, recursive directory creation, single-file removal, empty-directory removal, recursive directory removal, single-file copy, recursive directory copy/move, metadata predicates and `PathStatus`/`PathKind`/`PathInfo`/`PathMetadata`/`PathSizeMetadata`/`DirectorySizeMetadata`, `std::env::get_var`, `std::env::args`, `std::cli` positional/flag/option helpers, typed scalar `std::cli` parsing helpers, `std::time::now_unix_millis`, `std::test` scalar assertion helpers, `std::option` / `std::result` value helpers, `std::string` text assembly helpers, `std::list` / `std::map` collection helpers, line-comment-preserving deterministic `muga fmt`, `muga build`, `check --built` / `run --built`, local path dependencies, unchanged build artifact reuse with visible written/reused status, `.mgb` source-hash metadata, public interface hash stability for implementation-only changes, dependency-level parallel package builds, minimal local path/archive dependency lockfiles, canonical package content hashing including manifest-declared resources, deterministic `.mgp` package source/resource archive emission, `.mgp` archive readback/hash validation, local `.mgp` materialization, local archive dependency cache consumption, local archive dependency cache/lockfile hardening, `emit-package-archive --dependency-snippet`, CLI usage/spec alignment for artifact-backed program arguments, `--built` default artifact diagnostics, and the initial `muga test` / `@test` workflow. Keep invalid future snippets out of `samples/`; they belong under `spec/snippets/`.
2. Expand diagnostics around generic arity, duplicate type parameters, generic record literal expected-type context, stale generic package interfaces, invalid string helper receivers, and artifact-backed execution failures. Stale generic `.mgi` artifact diagnostics now include artifact-root context and concrete regeneration-command suggestions, full `std::io::...` type spellings now point users to `import std::io` plus the local `io::...` alias form, invalid `try Result::Ok(...)` placements report the `try` placement problem without also asking for a redundant Result constructor annotation, all current `E005` ambiguity diagnostics now include targeted annotation guidance, ambiguous collection/string helpers suggest the supported receiver annotations, `.mgb` implementation artifact diagnostics include the concrete artifact path plus package context, and JSON diagnostics include concrete artifact-file, artifact-hash, and regeneration-command context for known `.mgi`, `.mgc`, and `.mgb` paths.
3. Keep package interfaces storing resolved generic public signatures for records/functions and keep artifact execution proving fallible, helper-heavy, stdlib-backed, and representative composite dependency APIs without source-body fallback.
4. Keep bounds, protocols/typeclasses, higher-kinded types, specialization, and polymorphic recursion out of the MVP.
5. Keep the explicit `.mgi` / `.mgc` / `.mgb` artifact workflow as the package boundary while hardening public signatures.
6. If adding more functional or tooling surface before v1, prefer small usability slices that do not change the core model and only additional collection helpers whose equality/allocation behavior is already specified. `muga --help`, `muga help <command>`, `muga syntax --format json`, entry-aware `check --format json`, `muga explain <diagnostic-code>`, `muga run --format json`, `muga test --format json`, `muga build --format json`, artifact-emission JSON output, `muga metadata --format json`, `muga workspace --format json`, `muga hover --format json`, `muga completions --format json`, `muga definition --format json`, `muga references --format json`, entry source context in `diagnostics[].context`, package/artifact-root plus concrete artifact-file/hash/regeneration context for artifact-backed diagnostics, a concrete JSON-backed editor workflow smoke test, and a runnable local-dependency report sample have landed. `muga new --list-templates` and templates for app, lib, test, config app, strict CLI tool, report app, resource export, and package app projects plus public source comments in generated docs have landed. The v1 equality policy is scalar-only for `Int`, `Bool`, and `String`.
7. Add trust and maintenance surfaces before broad language growth when they help preserve the current contract: conformance tests, machine-readable diagnostics, stable command-output contracts, `.mgi` API compatibility diffing, standard-library review rules, doc-comment/API-doc rules, package metadata, artifact/cache explanations, fuzzing plans, debug/failure reports, installation/onboarding docs, and example-driven education. The read-only `muga why-rebuild` artifact/cache explanation output lives in [docs/artifact-cache-explanations.md](./docs/artifact-cache-explanations.md), now covers local archive-cache metadata, and has compact human text output plus JSON for tools. Runtime/debug v1 reporting uses call-context related notes for nested function calls and entry/test execution, source-spanned `R021` diagnostics for failed scalar assertions, and `regenerationCommand` context for package/artifact next-actions. Release-neutral benchmark health checks now cover compiler stages, package artifact reuse, and representative String/List/Map runtime paths in [docs/benchmark-health-checks.md](./docs/benchmark-health-checks.md). Parser, package archive, lockfile, interface, check-cache, and implementation artifact malformed-input hardening is planned in [docs/fuzzing-malformed-input-plan.md](./docs/fuzzing-malformed-input-plan.md). Release-neutral install, version-check, generated-project onboarding, shell completions, and `muga doctor` are documented in [docs/installation-and-onboarding.md](./docs/installation-and-onboarding.md) and [docs/shell-completions-and-doctor.md](./docs/shell-completions-and-doctor.md). Example-driven learning from bindings to artifact-backed builds is documented in [docs/muga-by-example.md](./docs/muga-by-example.md). Future registry security, signing, provenance, lockfile enforcement, cache validation, and malicious-package handling before remote fetching are scoped in [docs/registry-security-design.md](./docs/registry-security-design.md). Future edition and semantic feature-set fingerprints before syntax migration are scoped in [docs/edition-feature-fingerprint-policy.md](./docs/edition-feature-fingerprint-policy.md); `.mgi` public interface hash stability has been audited across implementation-only edits, source-span movement, generic public shapes, stdlib-backed signatures, and transitive public types, the first library and CLI `.mgi` API diff comparator classifies compatible/source-compatible/breaking/unknown public changes, `.mgb` structural validation plus bytecode merge behavior now has representative coverage for control-flow-heavy dependency bodies, private package items, and independently generated artifacts, `muga build` reuse output plus lockfile update behavior are covered for local path/local archive dependency implementation-only edits, public signature edits, and malformed lockfiles, recursive annotation diagnostics now point users at parameter/return signature fixes, and package-mode public signatures now have representative coverage for every v1-supported public type shape through in-memory and persisted interfaces. The stdlib package docs and samples review now covers `std::io`, `std::fs`, `std::path`, `std::env`, `std::cli`, `std::time`, `std::string`, `std::fmt`, and the first `std::json` slice, including artifact-backed execution samples where useful. The release gate and GitHub Actions are now aligned through [docs/release-gate-alignment.md](./docs/release-gate-alignment.md) and `scripts/v1-release-gate.sh`. The first `std::json` slice is now implemented from [docs/std-json-first-slice.md](./docs/std-json-first-slice.md) and audited in [docs/std-json-implementation-audit.md](./docs/std-json-implementation-audit.md), after documenting and preserving `Result` ergonomics, scalar/collection mapping, schema evolution, and diagnostics. The post-JSON stdlib/API boundary selection in [docs/post-json-stdlib-boundary-selection.md](./docs/post-json-stdlib-boundary-selection.md) chooses opaque resource-handle design as the next prerequisite before broader runtime-backed APIs, [docs/opaque-resource-handles.md](./docs/opaque-resource-handles.md) defines that boundary, the first `pub opaque type` interface slice is implemented, the metadata-only `OpaqueHandleFacts` / `paramMode` interface slice, consuming checker, first runtime file-handle design, and read-only `std::fs::File` implementation now feed the post-file-handle selection in [docs/post-file-handle-resource-surface-selection.md](./docs/post-file-handle-resource-surface-selection.md), and scalar `eprint` / `eprintln` implement the selected program stderr channel, while text output file handles are implemented from `docs/text-output-file-handles.md` before broader runtime handles. Keep these surfaces release-neutral. The modern-language gap inventory and classification live in [docs/modern-language-gap-inventory-2026-05-22.md](./docs/modern-language-gap-inventory-2026-05-22.md) and [docs/modern-language-gap-decisions-2026-05-22.md](./docs/modern-language-gap-decisions-2026-05-22.md).
8. Track syntax candidates separately from release readiness. `@test` is admitted only as static metadata for `muga test`, and the first statement-form `using` cleanup slice is implemented for runtime-backed opaque handles. Named arguments, `using` expressions/multiple bindings, range/slicing syntax, pattern-matching refinements, string interpolation, `T?`, and `?.` remain design candidates unless the v1 checklist, specs, parser diagnostics, formatter rules, samples, and focused tests are deliberately updated first.
9. Keep wildcard-heavy or catch-all pattern matching, native backend work, broad stdlib effects, broader runtime-backed resource handles, binary streams/codecs/handles, broader cryptographic APIs, broader mutation APIs, ambiguous `String.len()` semantics, range syntax or substring aliases, grapheme-cluster APIs, richer string error types, richer formatting templates/interpolation/builders, service/runtime APIs, workspaces, dev/test/bench dependencies, version solving, `muga audit`, SBOM generation, binary distribution/installers, remote registry security/signing/provenance, edition migrations, strict public performance claims, and full incremental project artifact reuse deferred until a concrete API slice requires those decisions. The first JSON package slice is implemented from [docs/std-json-first-slice.md](./docs/std-json-first-slice.md), audited in [docs/std-json-implementation-audit.md](./docs/std-json-implementation-audit.md), and must not expand into schema generation, HTTP/RPC, `Float`, `Decimal`, `Bytes`, streaming APIs, or resource handles. The first `pub opaque type` interface slice, metadata-only `OpaqueHandleFacts` / `paramMode` interface slice, and read-only `std::fs::File` runtime handle are documented in [docs/opaque-resource-handles.md](./docs/opaque-resource-handles.md); the post-file-handle selection in [docs/post-file-handle-resource-surface-selection.md](./docs/post-file-handle-resource-surface-selection.md) chooses scalar `eprint` / `eprintln`, [docs/text-output-file-handles.md](./docs/text-output-file-handles.md) defines the implemented write-mode file handles, [docs/lexical-resource-cleanup.md](./docs/lexical-resource-cleanup.md) defines the implemented statement-form `using` cleanup slice, and immediate stdout/stderr handles, process APIs, HTTP/SSE/WebSocket/RPC, streaming APIs, broader runtime-backed handle values, or buffers remain deferred. `String.byte_len()` is the byte-size spelling; range syntax should stay aligned with `String.slice_chars(start, count)` if syntax-level slicing is added, and grapheme APIs should wait for a Unicode segmentation policy.

## V1 Definition Of Done

Muga v1 is complete when:

- the closed grammar in [mini-language-spec-v1.md](./mini-language-spec-v1.md) and [spec/001-core-language.md](./spec/001-core-language.md) is implemented, documented, and covered by runnable or rejecting examples
- the release gate and feature-freeze checklist in [docs/v1-release-checklist.md](./docs/v1-release-checklist.md) passes
- `samples/` contains only runnable sample entrypoints, support source files for runnable entrypoints, or intentionally invalid sample trees, while post-v1 design snippets live under `spec/snippets/`
- package mode accepts only top-level `record`, `enum`, `pub opaque type`, and `fn` declarations, with module-private default visibility, `pkg`, `pub`, and explicit imports
- public package interfaces persist public records, enums, opaque types, and explicitly signed public functions in `.mgi` artifacts
- explicit artifact workflows and `muga build` can emit `.mgi`, `.mgc`, and `.mgb` artifacts that artifact-backed `check` and `run` consume, including the default `.muga/build` path through `--built`; `muga build` preserves unchanged generated artifacts and reports written/reused artifact status
- diagnostics keep stable code families, actionable annotation/import guidance, package/artifact path context where available, and hard artifact-backed failures without source-body fallback
- normal source-compatible `check` and `run` behavior remains unchanged when no `--artifact-root` is provided
- public-signature inference for `pub fn` is documented as post-v1; v1 public functions require explicit parameter and return types
- deferred surface areas remain out of v1: broad catch-all wildcard patterns, map literals, `Set[T]`, arbitrary `Map` keys, traits/protocols/typeclasses, references/borrowing syntax, postfix Result propagation `expr?`, future Result chain propagation `expr.try`, optional shorthand `T?`, optional chaining `?.`, call-site type arguments, iterator protocols, concurrency syntax, control-flow MIR, native backend, and full incremental project artifact reuse
- AI agents should not proactively suggest publishing, tagging, or cutting a release until the v1 completion criteria above are satisfied, unless a maintainer explicitly asks for release preparation

## Compiler Architecture Path

1. **Package interfaces as real inputs**

   Persist public records/functions/enums/opaque types, resolved signatures, item identity, and hashes. Downstream packages should check against interface artifacts instead of dependency bodies.

2. **Remove package flattening**

   Use package graph and interface data as the normal checking/compilation boundary. Keep package-aware diagnostics useful across that boundary.

3. **Build cache and incremental compilation**

   Reuse unchanged package interface and implementation artifacts. Invalidate by source hash, interface hash, and dependency graph. The short-term performance target is a fast edit-check-run loop that can beat Go-style compiler feedback on representative Muga projects, so artifact reuse, rebuild explanations, and future watch/daemon work should outrank broad syntax growth.

4. **MIR**

   Lower typed HIR into a compiler-oriented MIR that makes control flow, evaluation order, temporaries, and locals explicit.

5. **Native backend**

   Add a fast native backend after the semantic boundary and package model are stable. Cranelift remains the likely first backend candidate; LLVM can be reconsidered later if its tradeoffs become useful. The long-term runtime target is Rust/C++-class performance on representative Muga workloads, but public claims require the benchmark suite and optimizer layers to exist first.

6. **Structured concurrency**

   Design `group` / `spawn` / `join` first, then typed channels, then `select`-style coordination and timeouts. Do not make `async fn` / `await` the primary concurrency model unless later evidence justifies it.

7. **Standard library**

   Add IO, HTTP, strings, richer collections, process/time APIs, and web-oriented packages after package compilation and error handling are stable enough to support them cleanly.

8. **Interface-backed application tooling**

   Treat `.mgi` public package interfaces as the stable typed contract for future schema generation, service adapters, and client/server stubs. Generators should consume resolved public records, enums, and function signatures rather than dependency implementation bodies or hidden framework conventions.

The post-v1 language-feature order and the list of features to keep out of Muga are maintained in [docs/practical-language-readiness.md](./docs/practical-language-readiness.md).

## Post-V1 Platform Direction

After the v1 package/artifact workflow is stable, practical application work should grow in this order:

1. keep public package interfaces as the single source of truth for reusable package contracts
2. add narrow standard-library slices that use `Option`, `Result`, and explicit public error types
3. introduce opaque resource handles for files, sockets, timers, processes, and OS-backed effects
4. implement control-flow MIR and runtime representations that can optimize value semantics without adding source-level references
5. implement structured concurrency with `group`, `spawn`, and `join`, including failure and cancellation rules
6. add typed channels, then `select`-style coordination, timeouts, and deadlines
7. integrate cancellation-aware asynchronous IO with the scheduler and resource handles
8. broaden beyond the first `std::json` slice, or build `std::http`, SSE, WebSocket, and future RPC support, only after resource ownership, cancellation, and backpressure are expressible
9. generate external schemas, docs, and clients from `.mgi` interfaces once type-to-schema mappings are explicit

The guiding constraint is that ordinary Muga code should stay short and readable while the compiler can still resolve names, types, effects, public contracts, and package boundaries without dynamic runtime discovery.

The performance and concurrency spine is therefore:

1. short term: preserve fast syntax/check/build feedback through package artifacts, precise invalidation, and release-neutral benchmark health checks
2. medium term: add full incremental project artifact reuse, warm-cache measurements against Go and other fast compilers, and only then consider watch or compiler-daemon workflows
3. long term: implement control-flow MIR, efficient runtime representations, optimizer passes, and a native backend before making Rust/C++-class performance claims
4. platform term: implement structured concurrency, typed channels, cancellation-aware IO, and service-runtime benchmarks after resource handles and cancellation rules are stable

## Cross-Cutting Work

Performance work should follow the measurement stance in [docs/strategy-and-implementation-plan.md](./docs/strategy-and-implementation-plan.md): keep lightweight health checks now, and introduce strict external benchmarking only when the relevant build, MIR, native, or service-runtime layer exists.

Maintenance work should preserve the language contract before it expands the
language. Prefer conformance fixtures, structured diagnostic output, `.mgi`
compatibility diffing, standard-library review rules, runtime failure context,
and example-driven education before broad new APIs or syntax.

The decision pass in
[docs/modern-language-gap-decisions-2026-05-22.md](./docs/modern-language-gap-decisions-2026-05-22.md)
classifies modern-language gaps into v1 validation/support, optional pre-v1
usability, post-v1 platform work, and deliberate non-goals. Use that
classification before pulling new items into the active roadmap.

When measurement is appropriate, keep coverage across compiler steps:

- lex, parse, resolve, typecheck
- typed HIR lowering
- package interface loading/validation
- MIR lowering
- bytecode/native codegen

Diagnostics remain part of the architecture, not a late polish layer. New enum, package-interface, cache, MIR, and backend work should keep stable spans, declaration-site notes, and actionable suggestions where they materially improve debugging.

## Queued Decisions

Package-interface queue:

- diagnostics for stdlib error records and any remaining package-interface artifact edge cases
- eventual project-mode artifact-root config after lockfiles and package-aware project build state
- source-root and manifest conventions
- serialization of inferred public signatures once supported
- stable external names for schema/client generation from `.mgi`
- explicit type-to-schema mappings for records, enums, `Option`, `Result`, `List`, and `Map`
- versioning and invalidation rules for generated API artifacts keyed by interface hashes

Concurrency queue:

- whether task handles are source-nameable as `Task[T]`
- `group` return behavior
- failure and cancellation representation
- capture rules across task boundaries
- channels as a later phase after `group` / `spawn` / `join`
- timeout/deadline API shape after channels
- scheduler integration with cancellation-aware resource handles and nonblocking IO

Write-oriented API queue:

- builder/buffer types for repeated construction
- resource/handle types for files, sockets, processes, timers, and OS-backed effects
- handle ownership, close/drop behavior, and send/share rules
- clear separation between blocking host APIs and scheduler-aware nonblocking APIs
- MIR/native lowering for copy elision and internal destructive update

## Deferred Surface Work

These should stay deferred unless the active implementation slice requires them:

- map literals, `Set[T]`, arbitrary `Map` key types, and broad collection APIs
- bounds, typeclasses, higher-kinded types, const generics, specialization, and polymorphic recursion
- wildcard-heavy pattern matching, guards, nested destructuring, and named-field enum variants
- URL/Git/registry dependency declarations, remote package fetching, publishing/install workflows, full published-package lockfile enforcement, and package signing
- source-level references, mutable references, pointer syntax, or borrowing syntax

## Short Version

The coherent path to v1 is:

1. close the MIR/runtime identity foundation for the reference VM
2. make package execution work without dependency implementation bodies
3. keep `emit-artifacts` / `check --artifact-root` / artifact-backed execution explicit and non-silent
4. document and test the v1 package workflow end to end
5. only then resume larger post-v1 work: control-flow MIR, native backend, richer generics, structured concurrency, and practical standard library expansion
