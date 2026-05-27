Status: CLI schema-backed shell completion adoption audit completed; install docs, generated cli-tool README, packaging hook, shell-agnostic JSON completion spec, nested traversal, value sources, and non-mutating completion package emission implemented

# Post CLI Schema Shell Completion Adoption Gap Selection

`muga cli-completions` can now render bash, zsh, and fish scripts from
`CliSchema` data for source, explicit artifact-root, and `--built` workflows,
and [cli-completion-json-spec.md](cli-completion-json-spec.md) now exposes the
same completion model through `muga cli-completions --format json`.
[cli-completion-installer-integration.md](cli-completion-installer-integration.md)
adds `muga emit-cli-completions --format json --output-dir ...` so generated
projects and package managers can write bash, zsh, fish, and JSON completion
artifacts in one non-mutating command with machine-readable file metadata.
The remaining adoption gap is discoverability: generated `cli-tool` users need
a copyable path from `muga new` to an installable completion script without
making Muga mutate shell configuration or package manager state.

## Goals

Short-Term Goal: make generated `cli-tool` projects show the exact
`muga cli-completions` commands needed for source and built workflows.

Medium-Term Goal: let first-project onboarding teach the difference between
static `muga shell-completions` for the developer tool and schema-backed app
completions for generated Muga CLIs.

Long-Term Goal: prepare generated Muga CLI projects for conventional
distribution without committing v1 to one shell-profile, installer, package
manager, or completion file layout.

Final Goal: make Muga-authored tools feel publishable by default, with typed
parsing, generated help, generated completions, and artifact-backed execution
discoverable from the generated project itself.

## Audit Findings

- The generator command is documented in
  [cli-schema-shell-completions.md](cli-schema-shell-completions.md) and covered
  by source, artifact-root, and `--built` tests.
- [installation-and-onboarding.md](installation-and-onboarding.md) documents
  static `muga shell-completions`, but it does not yet show generated app
  completions in a first-project workflow.
- `muga new --template cli-tool` writes a strong source example but no local
  README, so the generated project does not carry the completion generation
  command with it.
- Automatic shell-profile installation would cross a host mutation boundary
  that current onboarding intentionally avoids.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| Add install documentation plus a generated `cli-tool` README | Immediate discoverability; no new runtime semantics; keeps shell install paths user/package-manager controlled; exercises the already implemented source and `--built` command shape. | Adds one generated text file and documentation maintenance. | Select |
| Generate completion files during `muga new --template cli-tool` | Makes scripts visible in the project tree immediately. | Requires running schema generation during project creation or committing stale generated script snapshots that can drift from source. | Defer |
| Add shell-agnostic JSON completion specs next | Useful for package managers and external generators. | Lower immediate user value than copyable shell scripts, and it introduces a second completion contract. | Done after onboarding |
| Add non-mutating completion package emission | Gives installers one command for bash, zsh, fish, and JSON artifacts without shell-profile edits. | Needs the shell and JSON contracts to be stable first. | Done after value sources |
| Add automatic shell-profile installation | Smoothest local UX. | Mutates host configuration, varies by shell/platform, and conflicts with the release-neutral onboarding boundary. | Reject for v1 |
| Prioritize richer nested traversal in shell scripts | Improves uncommon deep command trees. | The checked-in/generated `cli-tool` workflow is one command level; install discoverability is the current adoption blocker. | Defer |
| Add TOML/config discovery before completion onboarding | Helps future default/config-aware completions. | Broader config semantics are still deferred and not needed to install current static schema completions. | Defer |

## Selected Slice

Implement generated completion onboarding now:

1. Add a generated `README.md` to `muga new --template cli-tool` with source and
   `--built` completion commands for bash, zsh, and fish.
2. Add first-project onboarding documentation showing how to generate
   `cli-tool` app completions separately from static `muga` tool completions.
3. Add a generated `scripts/generate-completions.sh` hook that writes bash,
   zsh, and fish scripts into `completions/` without running the app.
4. Keep scripts printed to stdout only. Users can redirect them into the shell
   or package-manager path appropriate for their environment.

## Implementation Plan

1. Done: implement `muga cli-completions` in
   [cli-schema-shell-completions.md](cli-schema-shell-completions.md).
2. Done: audit generated-project completion adoption here.
3. Done: add generated `cli-tool` README and onboarding docs with copyable
   source and `--built` completion commands.
4. Done: add a generated project packaging hook as
   `scripts/generate-completions.sh`.
5. Done: add shell-agnostic JSON completion specs in
   [cli-completion-json-spec.md](cli-completion-json-spec.md).
6. Done: implement richer nested command traversal across the shell renderers.
7. Done: add static file/directory value-source metadata in
   [cli-completion-value-sources.md](cli-completion-value-sources.md).
8. Done: add non-mutating completion package emission in
   [cli-completion-installer-integration.md](cli-completion-installer-integration.md).
9. Next: evaluate TOML/config discovery separately; keep shell-profile
   installation and dynamic completion producers deferred.
