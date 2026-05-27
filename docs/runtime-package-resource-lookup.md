# Runtime Package Resource Lookup

Status: read-only runtime package resource lookup is implemented for manifest
projects through `std::fs::read_resource_text(package_path, resource_path)` and
`std::fs::read_resource_bytes(package_path, resource_path)`.

This slice turns manifest-declared resource roots into a small runtime API
without exposing host paths or adding installed-app layout policy.

## Goals

Short-Term Goal: let applications read packaged UTF-8 text resources such as
default config, help text, templates, and small static fixtures at runtime, and
read small binary assets without exposing host paths.

Medium-Term Goal: make the same call work for local source trees, materialized
local `.mgp` archive dependency caches, `muga run`, `muga test`, and explicit
artifact-backed `run --built` workflows.

Long-Term Goal: keep future installed-app and registry layouts free to choose
their on-disk structure while preserving the same read-only source API.

Final Goal: make Muga packages practical to distribute by letting code consume
the resources already covered by deterministic package identity.

## Implemented Contract

`std::fs` now exports:

```txt
pub fn read_resource_text(package_path: String, resource_path: String): Result[String, io::IOError]
pub fn read_resource_bytes(package_path: String, resource_path: String): Result[bytes::Bytes, io::IOError]
```

`std::bytes` exports the initial opaque byte value API:

```txt
pub opaque type Bytes
pub fn size(bytes: Bytes): Int
pub fn empty(bytes: Bytes): Bool
```

The `package_path` argument must name the entry manifest package or one of its
resolved manifest dependencies with `[package] resources = "..."`.

The `resource_path` argument is relative to that package's declared resource
root. It must be non-empty, slash-separated, and must not be absolute or contain
backslashes, drive syntax, empty segments, `.`, `..`, `.git`, or `.muga`.

Runtime lookup is read-only. The API returns text contents directly rather than
returning a host `Path`, and returns binary contents as an opaque `Bytes` value
rather than a mutable buffer. Callers therefore cannot accidentally feed package
resources to write/delete functions. Lookup canonicalizes the declared resource
root and candidate path before reading, and returns `io::IOError` if the
resource is missing, invalid, undeclared, a directory, or escapes through a
symlink. The text API additionally returns `io::IOError` for non-UTF-8 bytes.

Package archives and app bundles may preserve binary resource files. The binary
runtime API deliberately exposes only `bytes::size` and `bytes::empty` so binary
file handles, mutation, codecs, broader cryptographic APIs, and buffer builders remain separate
design decisions.

`muga run`, `muga test`, `muga run --built`, and library artifact-backed run
paths pass the entry manifest's resource-root map into the runtime. Local
archive dependencies therefore read from their materialized `.muga/packages`
cache roots.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `fs::read_resource_text(package, path)` | Small read-only UTF-8 API, supports dependencies, avoids host path leakage, works for source trees and archive caches. | Requires explicit package strings until package-relative caller context exists. | Select |
| `fs::read_resource_bytes(package, path)` plus opaque `std::bytes::Bytes` | Makes archived binary resources usable for assets and generated files without committing to binary file handles or encodings. | Only supports `bytes::size` and `bytes::empty` until concrete callers justify more operations. | Select |
| `fs::resource_path(package, path): path::Path` | Composes with existing `std::fs` functions. | Leaks layout and allows writes/removal through ordinary file APIs. | Reject |
| `fs::read_resource_text(path)` using the caller package | Ergonomic for libraries. | Runtime does not yet carry reliable current-package context through closures and artifacts. | Defer |
| Config-specific resource lookup | Helps generated config apps. | Too narrow and duplicates the package resource mechanism. | Reject |
| Installed app layout now | Necessary for distribution polish. | Needs launcher/install policy beyond this read-only API. | Defer |

## Non-Goals

This slice does not add:

- binary file handles, mutable bytes, codecs, broader cryptographic APIs, or buffer builders;
- host path-returning resource APIs;
- package-relative caller inference;
- resource listing or glob APIs;
- installed-app launchers or shell-profile mutation;
- URL, Git, registry, or remote package fetching.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_fs_read_resource_text_reads_manifest_entry_resources_for_source_and_built_runs`
- `standard_fs_read_resource_text_reads_archive_dependency_resources_from_cache`
- `standard_fs_read_resource_text_reports_invalid_paths_and_missing_roots`
- `standard_fs_read_resource_text_is_available_to_package_tests`
- `standard_fs_read_resource_bytes_reads_manifest_entry_resources_for_source_and_built_runs`
- `standard_fs_read_resource_bytes_reads_archive_dependency_resources_from_cache`

Release-readiness coverage keeps this document, `std::fs`, builtin typing,
runtime context wiring, package archive resources, and documentation handoffs
aligned.

## Next

The first installed-app layout is now documented in
[installed-app-bundles.md](installed-app-bundles.md), including bundle-local
dependencies. The next resource-related distribution slice should keep registry
publishing, binary streams/codecs/handles, broad cryptographic APIs, broad TOML
parsing, and shell-profile installer mutation separate.
