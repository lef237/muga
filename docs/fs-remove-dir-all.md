# Recursive Directory Removal

Status: `std::fs::remove_dir_all_path(dir_path)` is implemented as the narrow
recursive directory removal helper over `path::Path` values.

This slice extends the existing one-path filesystem mutation family without
adding trash/recycle-bin integration, globbing, sandbox containment, or public
symlink classification.

## Goals

Short-term: let Muga tools clean generated directory trees through the same
typed `Result[Unit, io::IOError]` shape used by `remove_file_path` and
`remove_dir_path`.

Medium-term: support generated project cleanup, installer/uninstaller
preflights, test fixture cleanup, and package export workflows without asking
every app to shell out to platform-specific commands.

Long-term: keep recursive deletion explicit and separate from richer
filesystem policy so future sandbox, trash, dry-run, and partial-failure APIs
can be designed deliberately.

Final goal: move Muga closer to practical adoption by supporting common CLI
tool cleanup workflows while keeping destructive operations visible,
recoverable, and reviewable.

## Public Shape

```muga
pub fn remove_dir_all_path(dir_path: path::Path): Result[Unit, io::IOError]
```

The function attempts to remove `dir_path` and all descendants using the host
runtime's recursive directory removal behavior. On success it returns
`Result::Ok(())`. On failure it returns `io::IOError` with
`operation = "remove_dir_all"` and the path originally passed by the caller.

The public `std::fs` wrapper calls a compiler-provided runtime helper:

```muga
pub fn remove_dir_all_path(dir_path: path::Path): Result[Unit, io::IOError] {
  __muga_std_fs_remove_dir_all(path::as_string(dir_path))
}
```

This helper is intentionally destructive. It does not implement a trash bin,
dry-run plan, sandbox check, ignore patterns, or a partial-success report.

## Candidates Compared

| Candidate | Benefit | Cost | Decision |
|---|---|---|---|
| Add `remove_dir_all_path(dir_path)` | Unlocks generated-tree cleanup and uninstall-style workflows with the existing one-path error shape. | Destructive and host-policy-backed; callers must choose paths carefully. | Select |
| Add recursive directory copy first | Useful for installers and workspace sync. | Needs overwrite, merge, metadata, symlink, and partial-copy policy before it is predictable. | Covered later as no-overwrite `copy_dir_all_path` |
| Add trash/recycle-bin deletion | Safer for humans. | Requires platform-specific behavior and persistence policy outside the current stdlib scope. | Defer |
| Add glob/pattern deletion | Ergonomic for cleanup tools. | Requires pattern syntax, escaping, traversal, ordering, and safety policy. | Defer |

## Deferred Policy

- public symlink classification remains deferred; this helper exposes only the
  host recursive-removal result through `io::IOError`.
- partial-success reporting remains deferred; the operation either returns
  `Ok(())` or the first host error reported by the runtime.
- no-overwrite directory copy is covered by `copy_dir_all_path`; copy-then-remove
  directory move is covered by `move_dir_all_path`; trash/recycle-bin
  integration, globbing, dry-run plans, permissions, ownership, and sandbox
  containment remain separate slices.

## Validation

- `package_std_fs_remove_dir_all_sample_runs`
- `standard_fs_remove_dir_all_path_removes_non_empty_tree`
- `standard_fs_remove_dir_all_path_missing_dir_returns_io_error`
- `standard_fs_remove_dir_all_path_file_returns_io_error`
- `standard_fs_remove_dir_all_path_type_mismatch_reports_expected_path`
- `standard_fs_remove_dir_all_artifact_run_uses_emitted_std_implementations`
- `fs_remove_dir_all_is_documented_and_covered`
