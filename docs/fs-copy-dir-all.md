# Recursive Directory Copy

Status: `std::fs::copy_dir_all_path(from_path, to_path)` is implemented as a
narrow recursive directory copy helper over `path::Path` values.

This slice complements read-only recursive traversal and recursive removal
without adding merge, overwrite, metadata preservation, trash/recycle-bin
integration, globbing, sandbox containment, or public symlink classification.

## Goals

Short-term: let Muga tools copy a generated or validated directory tree without
shelling out to platform-specific commands.

Medium-term: support project scaffolding, bundle staging, fixture setup, and
local package workflows through the same explicit `Result[Unit,
io::PathPairError]` surface used by two-path filesystem helpers.

Long-term: keep tree copy policy predictable so richer copy options,
transactional rollback, metadata preservation, host-rename moves, and sandbox
checks can be designed later without changing this first helper.

Final goal: move Muga closer to practical adoption by covering common local
file workflow setup and staging operations while keeping host effects visible
and recoverable.

## Public Shape

```muga
pub fn copy_dir_all_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]
```

The function copies the directory tree at `from_path` into a new directory at
`to_path`. The destination root must not already exist, and the destination
must not be the source directory or inside the source directory. Regular files
and directories are copied in deterministic path order. Symlinks and other
special entries return `io::PathPairError`.

On success the function returns `Result::Ok(())`. On failure it returns
`io::PathPairError` with `operation = "copy_dir_all"`, the source path in
`from_path`, the destination path in `to_path`, and the host error kind/message.
If a nested entry fails, the error paths identify that nested source and target.

The public `std::fs` wrapper calls a compiler-provided runtime helper:

```muga
pub fn copy_dir_all_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError] {
  __muga_std_fs_copy_dir_all(path::as_string(from_path), path::as_string(to_path))
}
```

This helper does not roll back partially copied directories after an error. It
does not merge into existing directories or overwrite existing entries.

## Candidates Compared

| Candidate | Benefit | Cost | Decision |
|---|---|---|---|
| Add no-overwrite `copy_dir_all_path(from, to)` | Covers the common safe staging/scaffolding case with existing path-pair errors. | Partial failure can leave a partially created target tree. | Select |
| Add merge/overwrite copy | More convenient for sync workflows. | Requires overwrite, conflict, metadata, and rollback policy. | Defer |
| Add host-rename/cross-device move fallback | Useful for installers and package managers. | Requires host-specific rename, rollback, and partial-failure policy. | Defer |
| Add glob/pattern copy | Ergonomic for selective staging. | Requires pattern syntax, traversal, ordering, and safety policy. | Defer |

## Deferred Policy

- metadata, permissions, timestamps, ownership, and extended attributes are not
  preserved by contract.
- symlinks and special entries are rejected instead of copied or followed.
- copy-then-remove directory move is covered by `move_dir_all_path`; rollback,
  dry-run plans, ignore patterns, trash/recycle-bin integration, host-rename
  acceleration, and sandbox containment remain separate slices.

## Validation

- `package_std_fs_copy_dir_all_sample_runs`
- `standard_fs_copy_dir_all_path_copies_directory_tree`
- `standard_fs_copy_dir_all_path_missing_source_returns_path_pair_error`
- `standard_fs_copy_dir_all_path_existing_target_returns_path_pair_error`
- `standard_fs_copy_dir_all_path_rejects_destination_inside_source`
- `standard_fs_copy_dir_all_path_type_mismatch_reports_expected_path`
- `standard_fs_copy_dir_all_artifact_run_uses_emitted_std_implementations`
- `fs_copy_dir_all_is_documented_and_covered`
