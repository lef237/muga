# Recursive Directory Move

Status: `std::fs::move_dir_all_path(from_path, to_path)` is implemented as a
narrow no-overwrite recursive directory move helper over `path::Path` values.

This slice composes the already selected recursive copy and removal policies
without adding atomic filesystem transactions, merge/overwrite behavior,
rollback, metadata preservation, trash/recycle-bin integration, globbing,
sandbox containment, or public symlink classification.

## Goals

Short-term: let Muga tools stage, relocate, or install generated directory
trees without shelling out to platform-specific commands.

Medium-term: support generated project/package workflows through the same
explicit `Result[Unit, io::PathPairError]` surface used by two-path filesystem
helpers.

Long-term: keep recursive move policy explicit so future atomic rename,
transactional rollback, metadata preservation, sandbox checks, and richer
copy/move options can be designed later without changing this first helper.

Final goal: move Muga closer to practical adoption by covering common local
directory relocation workflows while keeping host effects visible and
recoverable.

## Public Shape

```muga
pub fn move_dir_all_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]
```

The function copies the directory tree at `from_path` into a new directory at
`to_path`, then recursively removes `from_path` after the copy succeeds. The
destination root must not already exist, and the destination must not be the
source directory or inside the source directory. Regular files and directories
are copied in deterministic path order. Symlinks and other special entries
return `io::PathPairError`.

On success the function returns `Result::Ok(())`. On failure it returns
`io::PathPairError` with `operation = "move_dir_all"`, the source path in
`from_path`, the destination path in `to_path`, and the host error kind/message.
If a nested copy entry fails, the error paths identify that nested source and
target. If source removal fails after a successful copy, the top-level source
and destination paths are reported.

The public `std::fs` wrapper calls a compiler-provided runtime helper:

```muga
pub fn move_dir_all_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError] {
  __muga_std_fs_move_dir_all(path::as_string(from_path), path::as_string(to_path))
}
```

This helper is not atomic. It does not roll back a partially copied target after
a copy failure, and it does not remove the target if deleting the source fails
after the copy succeeds. It does not merge into existing directories or
overwrite existing entries.

## Candidates Compared

| Candidate | Benefit | Cost | Decision |
|---|---|---|---|
| Add copy-then-remove `move_dir_all_path(from, to)` | Deterministic, cross-device, no-overwrite semantics reuse existing recursive copy/removal policy. | Not atomic; partial failures can leave source and target state for callers to inspect. | Select |
| Use host `rename` first with fallback | Faster on same filesystem and may preserve metadata. | Existing-target, symlink, cross-device, and special-entry behavior become host-dependent. | Defer |
| Add transactional rollback | Friendlier failure cleanup. | Requires rollback error aggregation, conflict handling, and recovery policy. | Defer |
| Add merge/overwrite move | More convenient for sync workflows. | Requires overwrite, conflict, metadata, and rollback policy. | Defer |

## Deferred Policy

- metadata, permissions, timestamps, ownership, and extended attributes are not
  preserved by contract.
- symlinks and special entries are rejected instead of moved, copied, or
  followed.
- rollback, dry-run plans, ignore patterns, trash/recycle-bin integration,
  host-rename acceleration, globbing, and sandbox containment remain separate
  slices.

## Validation

- `package_std_fs_move_dir_all_sample_runs`
- `standard_fs_move_dir_all_path_moves_directory_tree`
- `standard_fs_move_dir_all_path_missing_source_returns_path_pair_error`
- `standard_fs_move_dir_all_path_existing_target_returns_path_pair_error`
- `standard_fs_move_dir_all_path_rejects_destination_inside_source`
- `standard_fs_move_dir_all_path_type_mismatch_reports_expected_path`
- `standard_fs_move_dir_all_artifact_run_uses_emitted_std_implementations`
- `fs_move_dir_all_is_documented_and_covered`
