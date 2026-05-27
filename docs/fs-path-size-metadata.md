# Filesystem Path Size Metadata

Status: `std::fs::PathSizeMetadata` and
`std::fs::path_size_metadata_path(target_path)` are implemented as a narrow
size-bearing all-path metadata record.

## Goal

Short-term: let tools carry path kind, last-modified time, and regular-file
size from one public value while still accepting directories as existing paths.

Medium-term: make package/report/cache preflight code easier to write without
forcing every caller to separately branch on `PathKind` before reading size.

Long-term: keep recursive directory sizing, symlink classification,
permissions, owner data, and accessed/created timestamps out of this slice.

Final goal: make practical filesystem tooling more ergonomic while preserving
explicit and reviewable host filesystem contracts.

## Public Shape

```muga
pub record PathSizeMetadata {
  status: PathStatus
  kind: PathKind
  modified: time::UnixMillis
  size: Option[Int]
}

pub fn path_size_metadata_path(target_path: path::Path): Result[PathSizeMetadata, io::IOError]
```

`path_size_metadata_path` first calls `path_metadata_path(target_path)`.
Missing paths or host timestamp failures therefore preserve the same
`io::IOError` shape as `PathMetadata`.

If the path is a regular file, it then calls `file_size_path(target_path)` and
returns `size = Option::Some(bytes)`. Directories and other existing path kinds
return `size = Option::None`, avoiding platform-specific directory byte-size
semantics and avoiding any recursive traversal promise.

## Candidates Compared

| Candidate | Benefit | Cost | Decision |
|---|---|---|---|
| Add `PathSizeMetadata` with `size: Option[Int]` | Adds the next practical metadata grouping without changing existing `PathMetadata`; callers get file sizes and safe directory handling. | Performs a second file-size read for regular files. | Select |
| Add `size` to `PathMetadata` | Smaller public surface. | Changes an existing public record and reopens directory-size semantics. | Reject |
| Use host `metadata.len()` for all paths | One host metadata read could fill an `Int` for files and directories. | Directory byte-size values are platform-specific and non-recursive, which is easy to misuse. | Defer |
| Recursive directory size metadata | Useful for package audits and installers. | Needs traversal, symlink, partial-failure, cancellation, and performance policy. | Defer |

## Validation

- `standard_fs_path_size_metadata_path_returns_public_record`
- `standard_fs_path_size_metadata_path_missing_path_returns_io_error`
- `standard_fs_path_size_metadata_path_type_mismatch_reports_expected_path`
- `standard_fs_path_size_metadata_artifact_run_uses_emitted_std_implementations`
- `package_std_fs_path_size_metadata_sample_runs`
- `fs_path_size_metadata_record_is_documented_and_covered`
