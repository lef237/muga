# Filesystem Path Status Record

Status: `std::fs::PathStatus` and `std::fs::path_status(target_path)` are
implemented as a small grouping layer over the existing path metadata
predicates.

This slice does not add a new runtime metadata operation. It packages
`exists_path`, `is_file_path`, and `is_dir_path` into a plain public record so
CLI tools, generated starters, and preflight checks can pass a single status
value around without committing Muga to symlink, permission, or broad all-path
metadata policy.

## Goals

Short-Term Goal: reduce repetitive path preflight glue in practical file tools
while preserving the existing non-throwing predicate behavior.

Medium-Term Goal: keep path status data representable in `.mgi` and artifact
execution without adding hidden IO through properties.

Long-Term Goal: leave richer all-path metadata, symlink classification,
permission bits, accessed/created timestamps, and directory sizing to later
slices with concrete callers.

Final Goal: make ordinary local file workflows easier to write while keeping
host filesystem boundaries explicit and reviewable.

## Implemented Contract

`std::fs` exports:

```txt
pub record PathStatus {
  exists: Bool
  is_file: Bool
  is_dir: Bool
}

pub fn path_status(target_path: path::Path): PathStatus
```

`path_status` returns a record whose fields are exactly the values returned by
`exists_path(target_path)`, `is_file_path(target_path)`, and
`is_dir_path(target_path)`. Missing paths therefore produce
`PathStatus { exists: false, is_file: false, is_dir: false }` rather than a
recoverable `Result` error, matching the predicate slice that already exists.

The record is ordinary public data. Accessing `status.exists`,
`status.is_file`, or `status.is_dir` performs no IO.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `fs::path_status(path): PathStatus` | Groups the existing common preflight facts without new runtime semantics; easy to teach in `std_fs_metadata`; artifact-safe because it is compiler-provided package code. | Still cannot distinguish symlinks, permissions, or unusual filesystem kinds. | Select |
| Full `fs::metadata_path(path): Result[Metadata, io::IOError]` | More expressive and could include kind, size, timestamps, permissions, and symlink facts. | Freezes broad all-path metadata policy too early and needs platform-specific error semantics. | Defer |
| Add more scalar predicates now | Keeps each fact minimal. | Grows API one name at a time and keeps users manually bundling status values. | Reject |
| Change existing predicates to return `Result` | Could expose permission and platform errors. | Breaks the established simple preflight contract and overfits this small grouping slice. | Reject |

## Non-Goals

This slice does not add:

- a broad all-path `Metadata` record;
- symlink, file-type, permission, owner, accessed-time, or created-time policy;
- recursive directory metadata, directory sizing, or directory copy behavior;
- a new runtime builtin or host metadata operation;
- string-path overloads for `path_status`.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_fs_path_status_returns_public_record`
- `standard_fs_path_status_type_mismatch_reports_expected_path`
- `standard_fs_metadata_artifact_run_uses_emitted_std_implementations`
- `package_std_fs_metadata_sample_runs`

Release-readiness coverage keeps this document, `std::fs`, the runnable sample,
and artifact-backed execution aligned.
