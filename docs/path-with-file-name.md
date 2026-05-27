# Path With File Name

Status: `std::path::with_file_name(path, new_file_name)` is implemented as a
pure path transformation helper.

This slice lets small tools derive sibling output, report, cache, and sidecar
paths without reading the host filesystem or choosing a broader
path-normalization policy.

## Goals

Short-Term Goal: make file-transforming tools able to replace an input path's
final component when deriving output paths such as `summary.txt`.

Medium-Term Goal: keep path construction and inspection in `std::path` pure,
transparent, and artifact-safe.

Long-Term Goal: leave strict file-name validation, canonicalization, symlink
handling, and permission errors to later slices with concrete callers.

Final Goal: make ordinary local file workflows more practical while keeping
host effects out of pure path APIs.

## Implemented Contract

`std::path` exports:

```txt
pub fn with_file_name(path: Path, new_file_name: String): Path
```

The helper returns a new `Path` whose final component is replaced with
`new_file_name`. It is a pure lexical transformation: it does not touch the
filesystem, does not normalize path components, does not reject separators in
`new_file_name`, and does not resolve symlinks.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `path::with_file_name(path, new_file_name): Path` | Directly supports sibling output/report/cache names with no host effects. | Does not validate whether the replacement is a single component. | Select |
| `path::with_file_name_checked(path, new_file_name): Result[Path, Error]` | Could reject separators or empty names. | Requires a new path error model before callers need it. | Defer |
| `fs::canonicalize_path(path): Result[path::Path, io::IOError]` | Gives resolved absolute host paths. | Freezes symlink, missing-path, and permission behavior. | Covered separately in [fs-canonicalize-path.md](fs-canonicalize-path.md) |
| `env::current_dir(): Result[path::Path, io::IOError]` | Helps apps derive paths from process state. | Adds an ambient host effect, so it needed a separate contract. | Covered separately in [env-current-dir.md](env-current-dir.md) |

## Non-Goals

This slice does not add:

- filesystem canonicalization or absolute-path resolution;
- lexical normalization of `.` / `..`;
- validation that `new_file_name` is a single path component;
- path error records for pure path operations;
- symlink, permission, or existence checks.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_path_with_file_name_runs_as_virtual_package`
- `standard_path_with_file_name_replaces_single_component`
- `standard_path_with_file_name_type_mismatch_reports_expected_path`
- `standard_path_with_file_name_name_type_mismatch_reports_expected_string`
- `standard_path_with_file_name_artifact_run_uses_emitted_std_implementations`
- `package_std_path_with_file_name_sample_runs`

Release-readiness coverage keeps this document, the public `std::path` surface,
builtin typing, runtime behavior, sample docs, and artifact-backed execution
aligned.
