# Environment Temporary Directory

Status: `std::env::temp_dir()` is implemented.

## Goal Alignment

### Short-Term Goal

Give package-mode programs a narrow way to discover the host temporary
directory for scratch files, generated reports, and tests without adding
runtime-owned temp-file creation policy.

### Medium-Term Goal

Make generated tools easier to adapt for local workflows while keeping unique
name selection, cleanup, and filesystem mutation explicit in application code.

### Long-Term Goal

Preserve a predictable host-effect boundary for practical tooling before
adding broader process, sandbox, installer, or service behavior.

### Final Goal

Move Muga toward practical implementation and adoption by adding small,
documented runtime capabilities in an order that keeps policy decisions visible.

## Implemented Contract

```muga
pub fn temp_dir(): Result[path::Path, io::IOError]
```

`env::temp_dir()` reads the host platform's temporary-directory convention and
wraps it as `path::Path`. It returns `Result::Err(io::IOError)` when that path
is not valid Unicode and cannot be represented as a Muga `String`.

The runtime error context is stable:

- `operation = "temp_dir"`
- `path = "."`

The helper does not create a directory, allocate a unique name, clean up files,
canonicalize, resolve symlinks, or enforce sandbox containment. Callers must
still derive child paths explicitly and use `std::fs` operations for mutation.

## Candidates Compared

| Candidate | Benefit | Risk | Decision |
|---|---|---|---|
| `env::temp_dir(): Result[path::Path, io::IOError]` | Useful for generated tools, tests, and temporary outputs; non-Unicode host paths stay recoverable. | Reads ambient host convention and can be misused as unique temp-file policy. | Implemented |
| `env::temp_dir(): path::Path` | Simpler call sites. | Would hide non-Unicode failure and diverge from `env::current_dir`'s explicit host-path boundary. | Reject |
| `fs::create_temp_dir(prefix): Result[path::Path, io::IOError]` | Could provide unique allocation. | Requires naming, collision, cleanup, and security policy that this slice should not freeze. | Defer |
| Runtime project/config root lookup | Convenient for generated apps. | Hides precedence and conflicts with explicit tooling metadata. | Defer |

## Non-Goals

- No unique temp-file or temp-directory allocator.
- No cleanup or lifecycle management.
- No environment mutation.
- No process execution or shell integration.
- No canonicalization, normalization, symlink resolution, or sandbox policy.

## Validation

- `standard_env_temp_dir_returns_process_temp_dir`
- `standard_env_temp_dir_rejects_arguments`
- `standard_env_temp_dir_artifact_run_uses_emitted_std_implementations`
- `package_std_env_temp_dir_sample_runs`
