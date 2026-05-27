# Filesystem Path Metadata
Status: `std::fs::PathMetadata` and
`std::fs::path_metadata_path(target_path)` are implemented as a narrow
host-error-backed existing-path metadata record.

## Goal

Short-term: let tools read the typed path kind and last-modified timestamp for
files or directories through one public record.

Medium-term: give generated report/cache/archive workflows a recoverable
preflight helper without adding size, permission, owner, or symlink policy.

Long-term: keep broad all-path metadata, recursive directory sizing, and
platform-specific file-type details out of this slice.

Final goal: make practical filesystem tooling easier while keeping Muga's
standard-library contracts explicit and reviewable.

## Public Shape

```muga
pub record PathMetadata {
  status: PathStatus
  kind: PathKind
  modified: time::UnixMillis
}

pub fn path_metadata_path(target_path: path::Path): Result[PathMetadata, io::IOError]
```

`path_metadata_path` first calls `modified_unix_millis_path(target_path)`.
Missing paths or host metadata failures therefore return the same
`io::IOError` shape and `operation = "modified_unix_millis"` as the scalar
helper. On success it attaches the current `PathInfo` classification.

## Candidates Compared

| Candidate | Benefit | Cost | Decision |
|---|---|---|---|
| `PathMetadata` with `status`, `kind`, and `modified` | Directly supports cache/report preflight for files and directories; reuses existing host-error behavior. | Does not expose size, permissions, owner, or symlink-specific facts. | Select |
| Add `size` to `PathMetadata` | More convenient for regular files. | Changes the existing record and reopens directory byte-size semantics. | Reject; use [fs-path-size-metadata.md](fs-path-size-metadata.md) |
| Add a one-shot runtime metadata builtin | Could keep all fields from one host metadata read. | Freezes broader metadata policy before concrete callers need permissions, symlinks, or created/accessed timestamps. | Defer |
| Recursive directory metadata | Useful for package audits. | Needs traversal, symlink, partial-failure, and cancellation policy. | Defer |

## Validation

- `standard_fs_path_metadata_path_returns_public_record`
- `standard_fs_path_metadata_path_missing_path_returns_io_error`
- `standard_fs_path_metadata_path_type_mismatch_reports_expected_path`
- `standard_fs_path_metadata_artifact_run_uses_emitted_std_implementations`
- `package_std_fs_path_metadata_sample_runs`
- `fs_path_metadata_record_is_documented_and_covered`
