# Path Strip Prefix

Status: `std::path::strip_prefix(path, base)` is implemented as a pure path
relationship helper.

This slice lets generated apps, archive tooling, and report builders derive
relative display or package paths from already-selected roots without reading
the host filesystem or choosing a broader path-normalization policy.

## Goals

Short-Term Goal: make path-heavy tools able to convert a path under a known
base into a relative path for manifests, archives, and human-readable output.

Medium-Term Goal: compose cleanly with `std::fs::canonicalize_path` and
`std::env::current_dir` while keeping prefix comparison itself pure and
artifact-safe.

Long-Term Goal: leave lexical normalization, symlink policy, and checked path
component validation to later slices with concrete callers.

Final Goal: make ordinary project, report, and package workflows practical
without hiding host effects inside pure path APIs.

## Implemented Contract

`std::path` exports:

```txt
pub fn strip_prefix(path: Path, base: Path): Option[Path]
```

The helper returns `Option::Some(relative)` when `base` is a component-aware
prefix of `path`, otherwise it returns `Option::None`.
If `path` and `base` are equal, the result is a `Path` whose text is empty. The
helper is pure: it does not touch the filesystem, does not canonicalize, does
not normalize `.` / `..`, does not resolve symlinks, and does not check whether
either path exists.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `path::strip_prefix(path, base): Option[Path]` | Directly supports relative archive/resource paths, reports, and post-canonicalization display paths with no host effects. | Caller must choose whether to canonicalize before comparing. | Select |
| `path::normalize(path): Path` | Useful for text-only cleanup. | Does not provide containment or host resolution. | Covered separately in [path-normalize.md](path-normalize.md) |
| `env::temp_dir(): Result[path::Path, io::IOError]` | Useful for tests and temporary workspaces. | Reads an ambient host convention and does not allocate unique names. | Covered separately in [env-temp-dir.md](env-temp-dir.md) |
| `fs::canonicalize_path(path): Result[path::Path, io::IOError]` | Proves existence and resolves host paths. | It is a host effect and already covered separately in [fs-canonicalize-path.md](fs-canonicalize-path.md). | Covered |

## Non-Goals

This slice does not add:

- filesystem canonicalization or absolute-path resolution;
- lexical normalization of `.` / `..`;
- symlink, permission, or existence checks;
- a new path error type for pure prefix mismatches;
- path containment or sandbox security policy.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_path_strip_prefix_returns_relative_path`
- `standard_path_strip_prefix_non_prefix_returns_none`
- `standard_path_strip_prefix_equal_paths_returns_empty_path`
- `standard_path_strip_prefix_path_type_mismatch_reports_expected_path`
- `standard_path_strip_prefix_base_type_mismatch_reports_expected_path`
- `standard_path_strip_prefix_artifact_run_uses_emitted_std_implementations`
- `package_std_path_strip_prefix_sample_runs`

Release-readiness coverage keeps this document, the public `std::path` surface,
builtin typing, runtime behavior, sample docs, and artifact-backed execution
aligned.
