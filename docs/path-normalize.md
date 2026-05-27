# Path Normalize

Status: `std::path::normalize(path)` is implemented as a pure lexical path
cleanup helper.

This slice lets generated apps, report builders, and archive tooling remove
obvious `.` components and internal `..` pairs from display/output paths without
reading the host filesystem or claiming sandbox containment.

## Goals

Short-Term Goal: make path-heavy tools able to produce cleaner relative output
paths after joining, replacing names, or deriving report/cache paths.

Medium-Term Goal: keep text-only path cleanup in `std::path` pure and
artifact-safe while letting callers combine it with `std::fs::canonicalize_path`
when they need existing host-path resolution.

Long-Term Goal: leave symlink policy, strict path component validation, and
sandbox containment to later slices with concrete security requirements.

Final Goal: make ordinary project, report, and package workflows practical
without hiding host effects inside pure path APIs.

## Implemented Contract

`std::path` exports:

```txt
pub fn normalize(path: Path): Path
```

The helper performs lexical cleanup only:

- removes `.` components;
- removes a normal component followed by `..`;
- preserves leading `..` components on relative paths;
- returns `.` when a non-empty relative path collapses to no components;
- leaves empty paths empty.

The helper does not touch the filesystem, does not canonicalize, does not
resolve symlinks, does not prove existence, and must not be used as a sandbox containment check.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `path::normalize(path): Path` | Directly cleans generated display/output paths with no host effects. | Can be misused as security containment unless docs and tests keep it lexical only. | Select |
| `fs::canonicalize_path(path): Result[path::Path, io::IOError]` | Proves existence and resolves host paths. | Host effect; missing paths and symlinks are part of the contract. | Covered separately in [fs-canonicalize-path.md](fs-canonicalize-path.md) |
| `path::normalize_checked(path): Result[Path, Error]` | Could reject suspicious components. | Needs a path error model and security policy before callers require it. | Defer |
| Sandbox containment helper | Useful for archive extraction and server tools. | Requires root policy, symlink policy, case-folding, and platform-specific security review. | Defer |

## Non-Goals

This slice does not add:

- filesystem canonicalization or absolute-path resolution;
- symlink, permission, or existence checks;
- strict path component validation;
- sandbox containment or archive extraction safety policy;
- a new path error type.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_path_normalize_removes_dot_and_internal_parent_components`
- `standard_path_normalize_preserves_leading_parent_components`
- `standard_path_normalize_collapsed_relative_path_returns_dot`
- `standard_path_normalize_type_mismatch_reports_expected_path`
- `standard_path_normalize_artifact_run_uses_emitted_std_implementations`
- `package_std_path_normalize_sample_runs`

Release-readiness coverage keeps this document, the public `std::path` surface,
builtin typing, runtime behavior, sample docs, and artifact-backed execution
aligned.
