# Filesystem Rename

Status: `std::fs::rename_path(from_path, to_path)` is implemented as the first
single-step filesystem rename/move helper.

This slice fills a common CLI/app gap without adding recursive copy/delete
fallback, broad mutation helpers, shell execution, or host-specific overwrite
policy. It reuses the existing two-path filesystem error record,
`std::io::PathPairError`.

## Goals

Short-Term Goal: let file-oriented tools move or rename a known path after
creating, copying, hashing, or validating local files.

Medium-Term Goal: keep one-shot filesystem mutation helpers aligned around
explicit `path::Path` values and recoverable `Result` errors.

Long-Term Goal: leave cross-device fallback, watch APIs, and richer filesystem
transaction policy to later slices with concrete callers.

Final Goal: make Muga practical for ordinary local file workflows while keeping
host effects small and explicit.

## Implemented Contract

`std::fs` exports:

```txt
pub fn rename_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]
```

The helper delegates one rename operation to the host filesystem. On success it
returns `Result::Ok(())`; on failure it returns `Result::Err(io::PathPairError)`
with `operation = "rename"`, the source path in `from_path`, the destination
path in `to_path`, and the host error kind/message. Existing-target behavior is
host-filesystem behavior for this first slice.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `fs::rename_path(from, to)` | Covers common file moves with the same two-path error surface as `copy_file_path`. | Existing-target semantics vary by host filesystem. | Select |
| File-only `rename_file_path` | More conservative name. | Adds a preflight file check and still cannot avoid race conditions. | Defer |
| Recursive move/cross-device fallback | Friendlier for directory moves and package managers. | Requires traversal, overwrite, rollback, and partial-failure policy. | Defer |
| Directory copy helpers | Useful for project scaffolding. | Needs traversal, symlink, permission, and overwrite decisions. | Covered separately as no-overwrite `copy_dir_all_path` |

## Non-Goals

This slice does not add:

- recursive copy/delete fallback or directory-copy behavior;
- cross-device copy-and-delete fallback;
- overwrite/backup/transaction policy;
- symlink-specific APIs;
- process, shell, or package-manager integration.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_fs_rename_path_moves_file_as_virtual_package`
- `standard_fs_rename_path_missing_source_returns_path_pair_error`
- `standard_fs_rename_path_type_mismatch_reports_expected_path`
- `standard_fs_rename_artifact_run_uses_emitted_std_implementations`
- `package_std_fs_rename_sample_runs`

Release-readiness coverage keeps this document, `std::fs`, builtin typing,
runtime behavior, sample docs, and artifact-backed execution aligned.
