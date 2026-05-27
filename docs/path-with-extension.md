# Path With Extension

Status: `std::path::with_extension(path, new_extension)` is implemented as a pure
path transformation helper.

This slice lets small tools derive output, cache, report, and sidecar paths
from an input path without reading the host filesystem or choosing a broader
path-normalization policy.

## Goals

Short-Term Goal: make file-transforming tools able to derive `*.json`,
`*.txt`, `*.html`, or similar output paths from an input file path.

Medium-Term Goal: keep path construction and inspection in `std::path` pure,
transparent, and artifact-safe.

Long-Term Goal: leave filesystem-backed path resolution, symlink handling,
canonicalization, and permission errors to later `std::fs` slices.

Final Goal: make ordinary local file workflows more practical while keeping
host effects out of pure path APIs.

## Implemented Contract

`std::path` exports:

```txt
pub fn with_extension(path: Path, new_extension: String): Path
```

The helper returns a new `Path` whose final extension is replaced with
`new_extension`. An empty `new_extension` removes the final extension. The
helper does not touch the filesystem, does not normalize path components, and
does not resolve symlinks.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `path::with_extension(path, new_extension): Path` | Directly supports report/cache/sidecar output names with no host effects. | Only changes the final extension. | Select |
| `path::with_file_name(path, new_file_name): Path` | Useful for sibling outputs. | Needs an explicit replacement contract. | Covered separately in [path-with-file-name.md](path-with-file-name.md) |
| `fs::canonicalize_path(path): Result[path::Path, io::IOError]` | Gives resolved absolute host paths. | Freezes symlink, missing-path, and permission behavior. | Covered separately in [fs-canonicalize-path.md](fs-canonicalize-path.md) |
| `env::current_dir(): Result[path::Path, io::IOError]` | Helps apps derive paths from process state. | Adds an ambient host effect, so it needed a separate contract. | Covered separately in [env-current-dir.md](env-current-dir.md) |

## Non-Goals

This slice does not add:

- filesystem canonicalization or absolute-path resolution;
- lexical normalization of `.` / `..`;
- file-name replacement or sibling-path helpers;
- validation of extension contents;
- symlink, permission, or existence checks.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_path_with_extension_runs_as_virtual_package`
- `standard_path_with_extension_empty_extension_strips_extension`
- `standard_path_with_extension_type_mismatch_reports_expected_path`
- `standard_path_with_extension_extension_type_mismatch_reports_expected_string`
- `standard_path_with_extension_artifact_run_uses_emitted_std_implementations`
- `package_std_path_with_extension_sample_runs`

Release-readiness coverage keeps this document, the public `std::path` surface,
builtin typing, runtime behavior, sample docs, and artifact-backed execution
aligned.
