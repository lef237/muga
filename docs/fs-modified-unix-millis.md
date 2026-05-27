# Filesystem Modified Unix Milliseconds

Status: `std::fs::modified_unix_millis_path(target_path)` is implemented as a
narrow filesystem timestamp helper.

This slice lets generated apps, build helpers, report tooling, and archive
preflight checks inspect a host path's last-modified timestamp without freezing a
broad public metadata record, permission model, or symlink-specific policy.

## Goals

Short-Term Goal: let file-oriented tools compare or report a known path's
last-modified time using the existing `std::time::UnixMillis` record.

Medium-Term Goal: keep timestamp reads explicit, recoverable, and aligned with
`std::path::Path`, `std::io::IOError`, and artifact-backed stdlib execution.

Long-Term Goal: leave accessed/created timestamps, all-path metadata records,
permission details, and symlink policy to later slices with concrete callers.

Final Goal: make ordinary local project, report, cache, and packaging workflows
practical while keeping host filesystem effects visible at the API boundary.

## Implemented Contract

`std::fs` exports:

```txt
pub fn modified_unix_millis_path(target_path: path::Path): Result[time::UnixMillis, io::IOError]
```

The helper reads host filesystem metadata for the provided path and returns the
last-modified timestamp as milliseconds after the Unix epoch. Missing paths,
permission failures, unsupported host metadata, timestamps before the Unix
epoch, and values that cannot fit in Muga `Int` return
`Result::Err(io::IOError)`.

The runtime error context is stable:

- `operation = "modified_unix_millis"`
- `path = path::as_string(target_path)`

The helper follows the host platform's ordinary metadata behavior. It does not
define an all-path public metadata record, expose timestamp precision beyond
milliseconds, read accessed or created times, inspect permissions, define a
symlink policy, or perform sandbox containment checks.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `fs::modified_unix_millis_path(path): Result[time::UnixMillis, io::IOError]` | Directly supports cache/report/build preflight checks with an existing time record. | Exposes one timestamp fact and follows host metadata behavior. | Select |
| `fs::file_metadata_path(path): Result[FileMetadata, io::IOError]` | Groups regular-file size and modified time for reports and preflight checks. | Remains file-only and composes scalar helpers. | Covered separately in [fs-file-metadata-record.md](fs-file-metadata-record.md) |
| `fs::metadata_path(path): Result[Metadata, io::IOError]` | Groups file type, size, timestamps, and permissions under one call. | Freezes an all-path public record before platform, symlink, permission, and precision policy are settled. | Defer |
| `fs::modified_path(path): Result[time::SystemTime, io::IOError]` | Could preserve richer host timestamp shape. | Requires a broader time model before Muga has callers that need it. | Defer |
| Watch/incremental filesystem API | Useful for build tools and services. | Requires async/event semantics, debounce policy, and platform-specific failure behavior. | Defer |

## Non-Goals

This slice does not add:

- all-path filesystem metadata records;
- accessed or created timestamps;
- permission, owner, or symlink APIs;
- filesystem watches or incremental invalidation;
- sandbox containment or archive extraction safety policy;
- timestamp formatting, timezone, or calendar APIs.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_fs_modified_unix_millis_path_returns_timestamp_record`
- `standard_fs_modified_unix_millis_path_missing_file_returns_io_error`
- `standard_fs_modified_unix_millis_path_type_mismatch_reports_expected_path`
- `standard_fs_modified_unix_millis_artifact_run_uses_emitted_std_implementations`
- `package_std_fs_modified_time_sample_runs`

Release-readiness coverage keeps this document, `std::fs`, builtin typing,
runtime behavior, sample docs, and artifact-backed execution aligned.
