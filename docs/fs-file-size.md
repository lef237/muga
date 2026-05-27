# Filesystem File Size

Status: `std::fs::file_size_path(file_path)` is implemented as the first
numeric filesystem metadata helper.

This slice makes file-oriented tools able to inspect local byte length without
adding a broad metadata record, broader timestamp policy, permission model,
recursive directory sizing, or symlink-specific behavior.

## Goals

Short-Term Goal: let CLI tools and small apps report or validate a known file's
byte length before reading, hashing, copying, archiving, or writing reports.

Medium-Term Goal: keep scalar metadata queries aligned with `path::Path` values
and recoverable `Result` errors.

Long-Term Goal: leave richer metadata records, accessed/created timestamps,
permissions, symlink policy, and recursive directory size calculations to later
slices with concrete callers.

Final Goal: make ordinary local file workflows practical while keeping host
effects explicit and small.

## Implemented Contract

`std::fs` exports:

```txt
pub fn file_size_path(file_path: path::Path): Result[Int, io::IOError]
```

The helper reads host filesystem metadata for the provided path and returns the
regular file's byte length as `Int`. Directories and other non-file paths return
an `InvalidInput` IO error rather than platform-specific metadata lengths. On
failure it returns `Result::Err(io::IOError)` with `operation = "file_size"`.
If the host length cannot fit in Muga `Int`, it is reported as an `InvalidData`
IO error.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `fs::file_size_path(path): Result[Int, io::IOError]` | Directly supports reports, validation, and preflight checks without new public data types. | Provides only one scalar metadata fact. | Select |
| `fs::file_metadata_path(path): Result[FileMetadata, io::IOError]` | Groups file size with modified time for regular-file reports and preflight checks. | Requires timestamp policy, so it follows after the scalar timestamp helper. | Covered separately in [fs-file-metadata-record.md](fs-file-metadata-record.md) |
| `fs::metadata_path(path): Result[Metadata, io::IOError]` | Groups multiple metadata facts under one call. | Freezes an all-path public record before symlink, permission, directory, and platform policy are settled. | Defer |
| `fs::modified_unix_millis_path(path)` | Useful for build tools. | Follows host metadata behavior and exposes only Unix-millisecond precision. | Covered separately in [fs-modified-unix-millis.md](fs-modified-unix-millis.md) |
| Recursive directory size | Useful for package audits. | Requires traversal, symlink, permission, and partial-failure policy. | Defer |

## Non-Goals

This slice does not add:

- all-path filesystem metadata records;
- accessed or created timestamps;
- permission, owner, or symlink APIs;
- recursive directory size calculations;
- streaming file reads, buffers, or binary writes.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_fs_file_size_path_runs_as_virtual_package`
- `standard_fs_file_size_path_missing_file_returns_io_error`
- `standard_fs_file_size_path_directory_returns_io_error`
- `standard_fs_file_size_path_type_mismatch_reports_expected_path`
- `standard_fs_file_size_artifact_run_uses_emitted_std_implementations`
- `package_std_fs_file_size_sample_runs`

Release-readiness coverage keeps this document, `std::fs`, builtin typing,
runtime behavior, sample docs, and artifact-backed execution aligned.
