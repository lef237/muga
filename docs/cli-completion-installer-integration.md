# CLI Completion Installer Integration

Status: non-mutating generated-app completion package emission implemented,
including source-free app bundle completion package emission and the generated
`cli-tool` package helper.

Muga already exposes generated app completions as shell scripts and as a
shell-agnostic JSON spec. This slice turns those separate outputs into one
installable package directory without editing shell startup files, installing
into host-specific locations, or running the target app.

## Goals

Short-Term Goal: give generated CLI projects one explicit command that writes
all supported completion artifacts into a user-selected directory.

Medium-Term Goal: let generated `cli-tool` projects, package managers, and
installer scripts consume stable completion files without scraping stdout from
multiple commands.

Long-Term Goal: keep completion distribution driven by `CliSchema` artifacts so
source, explicit artifact roots, and built workflows produce the same package.

Final Goal: make Muga-authored tools practical to publish by default: a user can
generate a typed CLI, build it, and ship shell plus JSON completion artifacts
without hand-maintaining host-specific metadata.

## Selected User Surface

The package emission command is:

```bash
muga emit-cli-completions [--format text|json] --output-dir <dir> --program <name> --type <Type> [--package <package>] [--artifact-root <dir>|--built] <source-file>
muga emit-app-completions [--format text|json] --output-dir <dir> [--program <name>] --type <Type> [--package <package>] <bundle-dir>
```

For a generated `cli-tool` project:

```bash
muga emit-cli-completions --format json --output-dir completions --program cli-tool --type Root src/main/main.muga
muga emit-app-completions --format json --output-dir completions --type Root dist/cli-tool
sh scripts/package-cli-tool.sh
```

The command writes four deterministic files:

- `<program>.bash`
- `_<program>`
- `<program>.fish`
- `<program>.completions.json`

The file stem uses the program name with conservative filename normalization.
Unsafe path separators, whitespace, control characters, and non-ASCII
characters are collapsed to `_`.

On success, stdout reports one tab-separated line per artifact in deterministic
order:

```text
written<TAB><output-dir>/<program>.bash
written<TAB><output-dir>/_<program>
written<TAB><output-dir>/<program>.fish
written<TAB><output-dir>/<program>.completions.json
```

With `--format json`, stdout reports the input entry or bundle, output
directory, program, target package/type, and generated file paths; structured
diagnostics also stay on stdout with stderr empty. The command never executes
the target app and never edits shell profiles. `emit-app-completions` reads only
`.muga/app-bundle` metadata and `.muga/build` interface artifacts from an app
bundle, and derives `--program` from a single bundle launcher when omitted, so
source-free bundles can emit the same package after distribution. Generated
`cli-tool` starters also include `scripts/package-cli-tool.sh`, which composes
source-free bundle emission, bundle execution, app completion package emission,
`.mga` archive creation, and archive verification without shell-profile
mutation.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| Add `muga emit-cli-completions --output-dir ...` | Reuses the existing `CliSchema` completion model, emits shell and JSON artifacts in one command, and gives package managers a stable directory contract. | Adds one CLI mode and file-write surface. | Select |
| Add `muga emit-app-completions --output-dir ...` | Reuses persisted app-bundle interfaces so source-free bundles can ship completions without copied source trees. | Adds one app-bundle reader surface for completion metadata. | Select |
| Add machine-readable completion package emission | Lets CI and packagers consume generated file metadata without scraping `written` rows. | Does not choose shell profile locations or installer ownership. | Select |
| Extend `scripts/generate-completions.sh` only | Keeps implementation in generated projects. | Leaves every template to duplicate shell and JSON output logic and does not help package managers outside generated starters. | Reject |
| Add shell-profile installation | Best first-run convenience. | Mutates user-owned startup files and introduces platform-specific rollback, idempotence, and security policy before Muga has installer channels. | Defer |
| Add package-manager-specific installers | Directly helps Homebrew, npm, cargo-binstall, or distro packaging. | Premature until binary release channels and signing/provenance policy are defined. | Defer |
| Add TOML/config discovery first | Useful for default-aware tools. | Does not improve the current static completion distribution path and opens broader config precedence semantics. | Defer |
| Add dynamic completion producers | Very flexible. | Requires executing user code or host processes during completion plus timeout, cancellation, and trust policy. | Defer |

## Non-Goals

This slice does not add:

- shell-profile edits or automatic installation;
- package-manager manifests or binary release installers;
- TOML/config discovery;
- dynamic completion values from env vars, processes, networks, or config
  discovery;
- app execution during completion generation.

## Implementation Plan

1. Done: add `muga emit-cli-completions [--format text|json] --output-dir
   <dir> --program <name> --type <Type> ...`.
2. Done: reuse the existing source, `--artifact-root`, and `--built`
   completion model loading.
3. Done: write bash, zsh, fish, and `.completions.json` artifacts with
   `written<TAB><path>` output.
4. Done: refresh the generated `cli-tool` packaging hook to call the new
   command.
5. Done: add `muga emit-app-completions [--format text|json] --output-dir <dir> [--program <name>] --type <Type> ...` for source-free app bundles.
6. Done: refresh the generated `cli-tool` package helper to emit a source-free
   bundle, app completions, `.mga` archive, and archive verification.
7. Done: add `--format json` metadata for completion package emission.
8. Next: evaluate TOML/config discovery as a separate config/defaults slice;
   keep shell-profile installation and dynamic completion producers deferred
   until host-effect and release-channel policy exists.
