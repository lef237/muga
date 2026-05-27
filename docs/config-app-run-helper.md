# Config App Run Helper

Status: generated `config-app` projects now include non-mutating
`scripts/run-with-config.sh` and `scripts/package-config-app.sh` helpers plus a
local README.

The helpers are intentionally small. They discover the generated project
directory from the script location, set or pass `MUGA_CONFIG_PATH` to
`config/settings.json` when the environment does not already provide a value,
and keep all path policy outside the runtime. They do not edit shell profiles,
create global config, or change Muga runtime discovery semantics.

## Goals

Short-Term Goal: let a newly generated config app run and package from any
current directory with its generated JSON settings file.

Medium-Term Goal: give users and wrappers a concrete pattern for deployment
time `MUGA_CONFIG_PATH`, source-free bundle packaging, app completions, and
archive verification without requiring TOML parsing, jq, or shell profile
mutation.

Long-Term Goal: keep generated app launch helpers separate from future package
resource lookup and installed application layouts.

Final Goal: make Muga practical for small operational tools by making the first
generated config workflow runnable without hidden runtime behavior.

## Contract

Generated config apps include:

```sh
sh scripts/run-with-config.sh --tag ops
sh scripts/package-config-app.sh
```

The run helper:

- computes `project_dir` from `scripts/run-with-config.sh`;
- uses `MUGA_BIN` when set, or `muga` from `PATH`;
- uses existing `MUGA_CONFIG_PATH` when set;
- otherwise sets `MUGA_CONFIG_PATH` to `$project_dir/config/settings.json`;
- forwards all arguments after `--` to the generated app.

The package helper:

- computes `project_dir` from `scripts/package-config-app.sh`;
- emits a source-free app bundle under `dist/config-app`;
- runs that bundle with `MUGA_CONFIG_PATH` pointing at the generated JSON file
  unless the environment already provides one;
- emits app completions for `Settings` from bundle interfaces;
- archives the bundle as `.mga` and verifies the archive;
- when `MUGA_INSTALL_DIR` is set, installs the bundle into that explicit bin
  directory and lists the owned launcher state.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| Add generated `scripts/run-with-config.sh` | Immediate first-run value; works from any current directory; keeps path policy outside runtime. | Adds one script to the generated template. | Select |
| Add generated `scripts/package-config-app.sh` | Connects the existing typed JSON config path to source-free bundle execution, app completion package emission, `.mga` archive creation, and archive verification. | Adds another generated script and inherits the existing app-bundle rule that the output directory must be absent or empty. | Select |
| Require users to pass `--config` every time | No template surface. | Reintroduces the friction that `MUGA_CONFIG_PATH` reduced. | Reject |
| Teach the runtime to read declared package resources | Stronger app-level behavior. | Needs explicit archive/cache resource roots before installed layout policy. | Done in [runtime-package-resource-lookup.md](runtime-package-resource-lookup.md) |
| Generate a jq-based workspace JSON wrapper | Exercises manifest metadata. | Adds external tool dependency and shell portability risk. | Defer |
| Add TOML config parsing now | Familiar format. | Does not solve first-run config path ergonomics and widens format policy. | Defer |

## Non-Goals

This slice does not add:

- shell profile installation;
- installed-app launchers;
- TOML/YAML/JSON5 parsing;
- automatic runtime config discovery.

## Implementation Plan

1. Done: add `README.md` to generated `config-app` projects with source,
   helper, workspace JSON, build, and built-run commands.
2. Done: add `scripts/run-with-config.sh` using `MUGA_BIN` and
   `MUGA_CONFIG_PATH`.
3. Done: cover the generated helper script in template tests and
   release-readiness checks.
4. Done: add `scripts/package-config-app.sh` using source-free app bundle
   emission, bundle execution with `MUGA_CONFIG_PATH`, app completion package
   emission, `.mga` archive creation, archive verification, and optional
   explicit-bin install/list through `MUGA_INSTALL_DIR`.
5. Done: package resource inclusion is implemented in
   [package-resource-archives.md](package-resource-archives.md).
6. Done: read-only runtime resource lookup is implemented in
   [runtime-package-resource-lookup.md](runtime-package-resource-lookup.md).
7. Done: first app bundle layout and launcher boundary are
   implemented in [installed-app-bundles.md](installed-app-bundles.md).
8. Done: dependency-aware app bundles use bundle-local dependency trees.
9. Done: `install-app` writes non-mutating app launcher wrappers.
10. Done: `.mga` app archives provide single-file bundle transport.
11. Done: `run-app-bundle` executes bundles from manifest resources and
   `.muga/build` artifacts without reading copied sources.
12. Done: `emit-app-bundle --source-free` omits copied source trees while
   preserving manifest/resource/artifact execution.
13. Done: `install-app` writes ownership metadata next to installed launchers.
14. Done: `install-app --replace-owned` updates only metadata-owned launchers.
15. Done: `list-installed-apps` reports metadata-owned launchers and drift.
16. Done: `uninstall-app` removes only metadata-owned launchers and metadata.
17. Done: `emit-app-completions` writes completion packages from app bundles.
18. Done: app bundles now carry archive/install/completion handoff commands.
18. Done: package archives and local archive caches preserve binary resources.
19. Done: minimal runtime `Bytes` resource/local file reads are implemented;
    next, defer binary streams/codecs/handles, broader cryptographic APIs, and
    shell-profile installer mutation.
