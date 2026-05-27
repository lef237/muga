# Recursive Directory Listing

Status: `std::fs::read_dir_recursive_path(root_path)` is implemented as a
read-only recursive directory traversal helper over `path::Path` values.

This slice intentionally follows the existing direct directory listing API
instead of adding destructive recursive operations or aggregate directory-size
metadata first.

## Goals

Short-term: make ordinary file-tree inspection possible from Muga source using
typed paths, explicit `Result` errors, deterministic ordering, and emitted
standard-library artifacts.

Medium-term: use the same traversal surface for package/resource audits,
recursive size reporting, report generation, and source-free bundle validation
without teaching every sample to hand-roll recursion.

Long-term: keep this read-only traversal as the foundation for later directory
copy, synchronization, and cleanup APIs once overwrite, partial-failure,
symlink, sandbox, and safety policies are explicit.

Final goal: move Muga closer to practical adoption by making real command-line
and file-processing tools possible while keeping the standard library small,
typed, deterministic, and reviewable.

## Public Shape

```muga
pub fn read_dir_recursive_path(root_path: path::Path): Result[List[path::Path], io::IOError]
```

The returned list contains descendants of `root_path`, not `root_path` itself.
Each direct child is returned before its own descendants. Direct children use
the same deterministic sorted path order as `read_dir_path`.

The public `std::fs` wrapper calls a compiler-provided runtime helper:

```muga
pub fn read_dir_recursive_path(root_path: path::Path): Result[List[path::Path], io::IOError] {
  __muga_std_fs_read_dir_recursive(path::as_string(root_path))
}
```

The runtime traversal sorts each directory's direct children, appends each
child, and then descends into entries whose directory-entry metadata reports a
directory. That keeps the traversal deterministic and avoids adding public
symlink classification in this slice.

Errors return the first `io::IOError` from reading the root, reading a child
directory, converting an entry path to UTF-8 text, or inspecting an entry type.

## Candidates Compared

| Candidate | Benefit | Cost | Decision |
|---|---|---|---|
| Add `read_dir_recursive_path(root_path)` | Unlocks read-only tree inspection, recursive reports, and later directory-size work using existing path and IO error types. | Does not aggregate metadata by itself and needs a small runtime helper for entry-type traversal. | Select |
| Add recursive directory size metadata first | Directly answers byte-size reporting needs. | Needs traversal policy, partial failure shape, symlink/cycle handling, and overflow/performance decisions before callers can inspect the tree. | Defer |
| Add recursive remove or directory copy first | Useful for installers and cleanup tools. | Destructive or overwrite-capable APIs need stricter safety, ownership, sandbox, and partial-failure policy. | Covered separately |
| Add glob/walk pattern matching | Ergonomic for many file tools. | Requires pattern syntax, escaping, platform matching rules, and ordering policy beyond the current stdlib scope. | Defer |

## Deferred Policy

- public symlink classification remains deferred; this helper does not expose
  symlink metadata and recurses only into entries reported as directories by
  host directory-entry metadata.
- partial-result reporting remains deferred; traversal stops at the first
  recoverable `io::IOError`.
- directory-size aggregation is covered by the separate
  `DirectorySizeMetadata` slice and recursive removal is covered by
  `remove_dir_all_path`; no-overwrite directory copy is covered by
  `copy_dir_all_path`; copy-then-remove directory move is covered by
  `move_dir_all_path`; globbing, permissions, and sandbox containment remain
  separate slices.

## Validation

- `package_std_fs_read_dir_recursive_sample_runs`
- `standard_fs_read_dir_recursive_path_returns_descendants`
- `standard_fs_read_dir_recursive_path_missing_dir_returns_io_error`
- `standard_fs_read_dir_recursive_path_does_not_recurse_into_symlink_dirs`
- `standard_fs_read_dir_recursive_path_type_mismatch_reports_expected_path`
- `standard_fs_read_dir_recursive_artifact_run_uses_emitted_std_implementations`
- `fs_read_dir_recursive_is_documented_and_covered`
