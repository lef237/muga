# Package Resource Archives

Status: manifest-declared package resources, including binary files, are now
included in local package content hashes, deterministic `.mgp` archives,
archive readback, non-mutating package archive verification, materialization,
and local archive dependency cache validation.

This slice made packaged applications more practical by preserving resources in
package identity and archives. The follow-up read-only runtime lookup is now
recorded in [runtime-package-resource-lookup.md](runtime-package-resource-lookup.md).
Resource inclusion is explicit: projects opt in with `[package] resources = "resources"` in `muga.toml`.

## Goals

Short-Term Goal: let local package archives carry project-owned resources such
as default JSON config, help text, templates, images, or small static files.

Medium-Term Goal: make archive hashes reflect the source tree and the declared
resource tree, so local archive dependencies and caches fail closed when either
code or resource content changes.

Long-Term Goal: prepare for runtime package resource lookup and installed app
layouts on top of the same manifest-declared resource root, without making
`run` change current directories or guess paths.

Final Goal: make Muga packages practical to distribute and reuse by preserving
the bytes an application needs while keeping package identity deterministic.

## Implemented Contract

Manifest projects may add:

```toml
[package]
name = "app"
source = "src"
resources = "resources"
```

The `resources` value must be a non-empty relative slash-separated directory
path. It must not use absolute paths, `..`, `.` segments, Windows drive syntax,
backslashes, `.git`, or `.muga`.

When `resources` is present:

- `package_content_hash` hashes `muga.toml`, sorted `.muga` files under the
  manifest source root, and sorted regular files under the resource root.
- `write_package_archive` and `muga emit-package-archive` write those resources
  as deterministic `resource` entries after source `file` entries.
- `validate_package_archive_bytes` rejects duplicate, unsorted, unsafe, or
  undeclared resource entries.
- `muga verify-package-archive` validates archive bytes, manifest entry, source
  entries, and resource entries without materializing files or updating caches.
  By default it uses the generated `*-sha256-<hash>.mgp` file name as the
  expected hash; `--expected-hash sha256:<hex>` supports renamed archives and
  manifest/lockfile-driven checks.
- `materialize_package_archive_bytes` and `materialize_package_archive` write
  resource entry bytes under the manifest-declared resource root.
- `muga unpack-package-archive [--format text|json] [--expected-hash
  sha256:<hex>] --output-dir <dir> <archive-file>` exposes that same
  materialization path from the CLI. Without `--expected-hash`, it validates the
  generated hash-bearing file name before writing files; with
  `--expected-hash`, it supports renamed local handoffs. The JSON form reports
  restored root/hash/file metadata and structured `archive` plus `outputDir`
  errors for CI and future package managers.
- local `.mgp` archive dependency caches recompute their content hash over
  materialized sources and resources.
- `muga workspace --format json` reports `resourceRoot` for the root project
  and each entry-reachable manifest dependency when present.

Tool metadata directories under the source or resource root are skipped during
archive emission. Source and manifest archive entries must remain UTF-8 text;
resource entries may contain arbitrary bytes. Runtime
`std::fs::read_resource_text` decodes UTF-8 resources, while
`std::fs::read_resource_bytes` returns arbitrary resource bytes as opaque
`std::bytes::Bytes`.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| Manifest-declared `resources = "resources"` | Explicit, deterministic, easy to hash and materialize, no hidden filesystem policy. | Requires projects to opt in. Runtime text lookup still requires UTF-8. | Select |
| Implicit conventional `resources/` directory | Zero config for common projects. | Silent package hash changes when a directory appears; harder to explain and audit. | Reject |
| Glob-based resource lists | Fine-grained inclusion. | Adds pattern grammar, path precedence, and cross-platform edge cases too early. | Defer |
| Binary resource archive bytes | Needed for images and compiled assets in app/package archives. | Runtime exposure is intentionally limited to opaque `Bytes` resource reads; binary file handles, mutation, codecs, and hashing remain separate decisions. | Select |
| Non-mutating package archive verification | Lets recipients, CI, and future package managers validate `.mgp` bytes before materialization or dependency-cache use. | Adds one CLI command; explicit hash mode is required for renamed archives. | Select |
| Runtime package resource lookup | Lets source code open package resources directly. | Needed the archive/cache path policy implemented here first. | Done in the follow-up read-only API |
| Installed app layout now | Helps distribution. | Premature before resource lookup, release channel policy, and binary packaging. | Defer |

## Non-Goals

This slice does not add:

- package manager or registry publishing;
- URL, Git, or remote fetching;
- binary file handles, writes, mutation, codecs, broader cryptographic APIs, or buffer builders;
- glob or include/exclude syntax;
- `muga.toml` TOML decoding as a user-facing stdlib feature;
- shell-profile installer mutation or installed-app launchers.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `package_content_hash_covers_manifest_declared_resources`
- `package_archive_emission_includes_manifest_declared_resources`
- `package_archive_preserves_binary_manifest_resources`
- `package_archive_readback_validates_hash_and_entries`
- `package_archive_readback_rejects_malformed_entries`
- `cli_verify_package_archive_validates_hash_from_filename`
- `package_archive_materialization_writes_validated_source_tree`
- `package_archive_materialization_rejects_unsafe_manifest_resource_roots`
- `manifest_local_archive_dependency_materializes_declared_resources_and_validates_cache_hash`
- `standard_fs_read_resource_bytes_reads_manifest_entry_resources_for_source_and_built_runs`
- `standard_fs_read_resource_bytes_reads_archive_dependency_resources_from_cache`
- `project_manifest_metadata_reports_roots_and_dependency_sources`

Release-readiness coverage keeps this document, the package/archive code, the
workspace JSON surface, and malformed-input plan aligned.

## Next

The next resource-related slice should not add broad TOML parsing. The practical
follow-up after read-only runtime lookup has started in
[installed-app-bundles.md](installed-app-bundles.md). Further distribution work
should preserve the bundle-local dependency policy without exposing host paths.
