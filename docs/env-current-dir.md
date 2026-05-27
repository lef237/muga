# Environment Current Directory

Status: `std::env::current_dir()` is implemented.

## Goal Alignment

### Short-Term Goal

Give package-mode programs a narrow way to read the process current working
directory without inventing application config discovery, project root lookup,
or path canonicalization rules.

### Medium-Term Goal

Make generated CLI and config applications easier to run from different
directories while keeping app-owned path policy visible in source code.

### Long-Term Goal

Preserve a predictable host-effect boundary for practical tooling, local
distribution, and package artifact execution before adding broader process,
installer, service, or registry behavior.

### Final Goal

Move Muga toward practical implementation and adoption by adding small,
documented runtime capabilities that real tools need, in an order that does
not hide policy inside the runtime.

## Implemented Contract

```muga
pub fn current_dir(): Result[path::Path, io::IOError]
```

`env::current_dir()` reads the host process current working directory and wraps
it as `path::Path`. It returns `Result::Err(io::IOError)` when the host cannot
return a current directory or when that path is not valid Unicode.

The runtime error context is stable:

- `operation = "current_dir"`
- `path = "."`

The helper does not canonicalize, resolve symlinks, search for `muga.toml`, or
guess a project/config/resource root. The returned `path::Path` stores the host
path text returned by the runtime.

## Candidates Compared

| Candidate | Benefit | Risk | Decision |
|---|---|---|---|
| `env::current_dir(): Result[path::Path, io::IOError]` | Useful for relative-path tools and generated apps; effect and failure are explicit. | Reads ambient process state. | Implemented |
| `env::temp_dir(): Result[path::Path, io::IOError]` | Useful for temporary workspaces and tests. | Reads a separate ambient host convention. | Covered separately in [env-temp-dir.md](env-temp-dir.md) |
| `fs::canonicalize_path(path): Result[path::Path, io::IOError]` | Useful for stable absolute paths. | Freezes host symlink, missing path, and permission behavior. | Covered separately in [fs-canonicalize-path.md](fs-canonicalize-path.md) |
| Runtime project/config root lookup | Convenient for generated apps. | Hides precedence and conflicts with explicit tooling metadata. | Defer |

## Non-Goals

- No environment mutation.
- No process execution or shell integration.
- No temp-file allocation or cleanup policy.
- No canonicalization, normalization, symlink resolution, or project-root
  discovery.
- No implicit config/resource lookup.

## Validation

- `standard_env_current_dir_returns_process_current_dir`
- `standard_env_current_dir_rejects_arguments`
- `standard_env_current_dir_artifact_run_uses_emitted_std_implementations`
- `package_std_env_current_dir_sample_runs`
