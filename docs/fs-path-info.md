# Filesystem Path Info
Status: `std::fs::PathKind`, `std::fs::PathInfo`,
`std::fs::path_kind(target_path)`, and `std::fs::path_info(target_path)` are
implemented as the first all-path metadata grouping over existing path
predicates.

## Goal

Short-term: let user code branch on missing/file/directory/other path shape
without manually repeating `exists_path`, `is_file_path`, and `is_dir_path`.

Medium-term: provide a stable record shape that generated tools and samples can
use before richer metadata policy is designed.

Long-term: keep symlink, permission, timestamp, and directory sizing semantics
out of this slice so future host-effect APIs can be designed deliberately.

Final goal: make filesystem preflight code easier to write while preserving
Muga's explicit, narrow standard-library contracts.

## Public Shape

```muga
pub enum PathKind {
  Missing
  File
  Directory
  Other
}

pub record PathInfo {
  status: PathStatus
  kind: PathKind
}

pub fn path_kind(target_path: path::Path): PathKind
pub fn path_info(target_path: path::Path): PathInfo
```

`path_info` calls `path_status` once, derives the kind from that status, and
returns both values. Missing paths return `PathKind::Missing`; existing paths
that are neither files nor directories return `PathKind::Other`.

## Candidates Compared

| Candidate | Benefit | Cost | Decision |
|---|---|---|---|
| `PathKind` plus `PathInfo` over existing predicates | Adds a typed branch target and grouped record without new host APIs. | Still inherits existing predicate behavior for symlinks and permission-denied paths. | Select |
| Add size/modified fields to all-path metadata | More data in one call. | Reopens directory-size, timestamp, symlink, and error policy. | Partly covered by [fs-path-metadata.md](fs-path-metadata.md) and [fs-path-size-metadata.md](fs-path-size-metadata.md) |
| Add `Result[PathInfo, io::IOError]` backed by host metadata | Preserves host errors. | Requires a new runtime operation and missing-path policy before the pure grouping is proven useful. | Defer |
| Add recursive directory metadata | Useful for larger tools. | Needs traversal, symlink, partial-failure, and cancellation policy. | Defer |

## Validation

- `standard_fs_path_info_returns_kind_and_status`
- `standard_fs_path_info_type_mismatch_reports_expected_path`
- `standard_fs_metadata_artifact_run_uses_emitted_std_implementations`
- `fs_path_info_is_documented_and_covered`
