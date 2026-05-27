# Filesystem File Metadata Record

Status: `std::fs::FileMetadata` and
`std::fs::file_metadata_path(file_path)` are implemented as the first public
filesystem metadata record.

This slice groups the already accepted file-size and modified-time facts for
regular files without defining directory metadata, symlink behavior,
permissions, accessed/created timestamps, or a broad all-path metadata record.

## Goals

Short-Term Goal: let report tools, archive helpers, and generated apps pass a
single regular-file metadata value around instead of issuing separate size and
timestamp calls at every call site.

Medium-Term Goal: keep public filesystem data records narrow, package-interface
safe, and aligned with `path::Path`, `time::UnixMillis`, and `io::IOError`.

Long-Term Goal: leave all-path metadata, directory metadata, symlink policy,
permissions, accessed/created timestamps, and recursive traversal APIs to later
slices with concrete callers.

Final Goal: make ordinary local file workflows practical while preserving
explicit, recoverable host filesystem boundaries.

## Implemented Contract

`std::fs` exports:

```txt
pub record FileMetadata {
  size: Int
  modified: time::UnixMillis
}

pub fn file_metadata_path(file_path: path::Path): Result[FileMetadata, io::IOError]
```

`file_metadata_path` is a regular-file helper. It first reads
`file_size_path(file_path)`, then reads
`modified_unix_millis_path(file_path)`, and returns a `FileMetadata` record.
Failures preserve the underlying operation:

- missing or non-file paths from the size step return `operation = "file_size"`;
- timestamp failures return `operation = "modified_unix_millis"`.

That keeps the new record compositional and avoids freezing a new runtime
metadata operation before the broader all-path policy is settled.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `fs::file_metadata_path(path): Result[FileMetadata, io::IOError]` | Solves the common report/cache/archive need by bundling file size and modified time; uses existing host behavior and error contracts. | Performs two metadata reads and remains regular-file-only. | Select |
| `fs::metadata_path(path): Result[Metadata, io::IOError]` | One call could group type, size, and timestamps for files and directories. | Freezes directory, symlink, timestamp, permission, and platform policy too early. | Defer |
| Add `is_file` / `is_dir` fields to `FileMetadata` | Would let callers branch on type from the same value. | Contradicts the regular-file-only contract and reopens all-path metadata semantics. | Reject |
| Keep only scalar helpers | Avoids any new public data type. | Forces every practical report or preflight API to carry separate values and duplicate glue. | Reject |

## Non-Goals

This slice does not add:

- all-path `Metadata` records;
- directory metadata records;
- symlink-specific behavior;
- permissions, owner, accessed time, or created time;
- recursive metadata collection or directory sizing;
- a new runtime metadata builtin.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_fs_file_metadata_path_returns_public_record`
- `standard_fs_file_metadata_path_missing_file_returns_io_error`
- `standard_fs_file_metadata_artifact_run_uses_emitted_std_implementations`
- `package_std_fs_file_metadata_sample_runs`

Release-readiness coverage keeps this document, `std::fs`, samples, docs, and
artifact-backed execution aligned.
