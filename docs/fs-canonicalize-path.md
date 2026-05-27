# Filesystem Canonicalize Path

Status: `std::fs::canonicalize_path(target_path)` is implemented.

This slice gives tools a narrow, recoverable way to ask the host filesystem for
the canonical path of an existing `std::path::Path` value. It intentionally
does not add project-root lookup, config discovery, path search, or pure
lexical normalization.

## Goals

Short-Term Goal: let CLI tools and generated apps turn known existing local
paths into stable absolute host paths after explicitly choosing the input path.

Medium-Term Goal: compose cleanly with `env::current_dir()` and `std::path`
helpers while preserving filesystem failure as `Result`.

Long-Term Goal: keep symlink, permission, missing-path, and host-specific path
resolution behavior visible as a filesystem effect instead of a property or
pure path helper.

Final Goal: make ordinary local path workflows practical enough for adoption
without hiding policy inside Muga's runtime.

## Implemented Contract

`std::fs` exports:

```txt
pub fn canonicalize_path(target_path: path::Path): Result[path::Path, io::IOError]
```

The helper delegates to the host filesystem's canonicalization behavior for
the supplied path. Successful results are wrapped as `path::Path`. Missing
paths, permission failures, and other host errors return `Result::Err` with
`operation = "canonicalize"` and `path` set to the input path text.
If the canonicalized host path is not valid Unicode, the helper also returns an
`InvalidData` IO error.

The helper resolves according to the host platform. It is not a pure lexical normalizer,
and it may resolve symlinks when the host does.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `fs::canonicalize_path(path): Result[path::Path, io::IOError]` | Directly supports reports, config preflight, archives, and generated tools that need stable existing host paths. | Requires an existing path and follows host symlink/permission behavior. | Select |
| `path::normalize(path): path::Path` | Pure and artifact-friendly. | Does not prove existence or resolve host links. | Covered separately in [path-normalize.md](path-normalize.md) |
| `env::temp_dir(): Result[path::Path, io::IOError]` | Useful for temporary outputs. | Platform convention and existence policy are separate from canonicalization. | Covered separately in [env-temp-dir.md](env-temp-dir.md) |
| Runtime project-root lookup | Convenient for generated apps. | Hides precedence and duplicates explicit workspace/config metadata. | Defer |

## Non-Goals

This slice does not add:

- pure lexical normalization;
- strict path component validation;
- project-root, config-root, or resource-root discovery;
- temp-directory selection;
- symlink-specific controls;
- recursive traversal or globbing.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `standard_fs_canonicalize_path_resolves_existing_file`
- `standard_fs_canonicalize_path_missing_file_returns_io_error`
- `standard_fs_canonicalize_path_type_mismatch_reports_expected_path`
- `standard_fs_canonicalize_artifact_run_uses_emitted_std_implementations`
- `package_std_fs_canonicalize_sample_runs`

Release-readiness coverage keeps this document, `std::fs`, builtin typing,
runtime behavior, sample docs, and artifact-backed execution aligned.
