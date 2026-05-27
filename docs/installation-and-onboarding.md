# Installation And Onboarding

Status: v1 onboarding guide. This document describes how a user or maintainer
can install Muga, verify the installed command, and run a first project without
turning installation work into a release trigger.

## Requirements

- Rust 1.95 or later.
- Cargo from the same Rust toolchain.
- A fresh workspace under `~/tmp/` for local first-run experiments.

Check the host toolchain before installing:

```bash
rustc --version
cargo --version
```

## Install Paths

Install the published crate when you want the latest package available from
Cargo:

```bash
cargo install muga
```

Install the current checkout when validating this repository before a release:

```bash
cargo install --path . --locked
```

For development without installing, run the command through Cargo from the
repository root:

```bash
cargo run --locked -- --version
cargo run --locked -- samples/println_sum.muga
```

## Version Check

After installation, verify the command that is first on `PATH`:

```bash
muga --version
muga version
muga --help
muga doctor
```

Both forms print the crate package version as `muga <version>`. If that version
is not the one you expected, inspect `PATH`, rerun `cargo install muga`, or use
`cargo install --path . --locked` from the intended checkout. Version checks
are for operator confidence only; they do not imply that a release should be
cut.

`muga --help`, `muga -h`, and `muga help` print the top-level command usage to
stdout and exit successfully. `muga help <command>` prints the matching usage
lines for a known command.

`muga doctor [--format text|json]` is the tool-only environment check for the
same first-run path. It reports the installed command version, executable path,
current directory, home directory, temporary directory, and `PATH` availability
without parsing source, loading packages, inspecting artifacts, mutating caches,
or using the network.

## Shell Completions

Print a static completion script for your shell and install it using that
shell's normal configuration path:

```bash
muga shell-completions bash
muga shell-completions zsh
muga shell-completions fish
```

The completion script covers top-level commands and common CLI options only.
It does not edit shell startup files, inspect projects, or make `doctor` a
release gate. The command contract is documented in
[shell-completions-and-doctor.md](shell-completions-and-doctor.md).

Generated Muga apps use a separate schema-backed completion command. For a
generated `cli-tool` project, create the project, generate app completions from
the `Root` CLI schema, and redirect stdout to the path your shell or package
manager expects:

```bash
muga new --template cli-tool ~/tmp/muga-cli
muga cli-completions fish --program cli-tool --type Root ~/tmp/muga-cli/src/main/main.muga
muga cli-completions --format json --program cli-tool --type Root ~/tmp/muga-cli/src/main/main.muga
muga emit-cli-completions --format json --output-dir ~/tmp/muga-cli/completions --program cli-tool --type Root ~/tmp/muga-cli/src/main/main.muga
muga emit-app-bundle --format json --source-free --output-dir ~/tmp/muga-cli/bundle --program cli-tool ~/tmp/muga-cli/src/main/main.muga
muga emit-app-completions --format json --output-dir ~/tmp/muga-cli/app-completions --type Root ~/tmp/muga-cli/bundle
muga build ~/tmp/muga-cli/src/main/main.muga
muga cli-completions zsh --program cli-tool --type Root --built ~/tmp/muga-cli/src/main/main.muga
cd ~/tmp/muga-cli
sh scripts/generate-completions.sh
sh scripts/package-cli-tool.sh
```

This command reads source or existing artifacts to inspect `CliSchema` data; it
does not run the generated app or install shell files. `emit-cli-completions`
writes bash, zsh, fish, and `.completions.json` files into an explicit output
directory so package managers or user-owned shell setup can place them later;
`--format json` reports those files for CI and packager tools.
`emit-app-completions` writes the same completion package from a source-free app
bundle's interface artifacts, so distributed bundles do not need source files to
ship completions.
The generated `scripts/package-cli-tool.sh` helper composes source-free bundle
emission, bundle execution, app completion package emission, `.mga` archive
creation, machine-readable bundle/archive metadata, and archive verification
without editing shell startup files.
Generated app completion design, shell output, JSON contracts, and package
emission are documented in
[cli-schema-shell-completions.md](cli-schema-shell-completions.md),
[cli-completion-json-spec.md](cli-completion-json-spec.md), and
[cli-completion-installer-integration.md](cli-completion-installer-integration.md).

## First Project

Use a fresh path under `~/tmp/` because `muga new` refuses non-empty targets:

```bash
muga new --list-templates
muga new --template app ~/tmp/muga-hello
muga run ~/tmp/muga-hello/src/main/main.muga
muga run ~/tmp/muga-hello/src/main/main.muga -- Ada
muga check ~/tmp/muga-hello/src/main/main.muga
muga fmt --check ~/tmp/muga-hello/src/main/main.muga
muga build ~/tmp/muga-hello/src/main/main.muga
muga check --built ~/tmp/muga-hello/src/main/main.muga
muga run --built ~/tmp/muga-hello/src/main/main.muga -- --name=Ada
muga why-rebuild --built ~/tmp/muga-hello/src/main/main.muga
muga emit-app-bundle --format json --source-free --output-dir ~/tmp/muga-hello-bundle --program hello ~/tmp/muga-hello/src/main/main.muga
sh ~/tmp/muga-hello-bundle/bin/hello --name=Ada
muga run-app-bundle ~/tmp/muga-hello-bundle -- --name=Ada
muga install-app --format json --replace-owned --output-dir ~/tmp/muga-bin --program hello ~/tmp/muga-hello-bundle
muga list-installed-apps --format json --output-dir ~/tmp/muga-bin
sh ~/tmp/muga-bin/hello --name=Ada
muga uninstall-app --format json --output-dir ~/tmp/muga-bin --program hello
muga emit-app-archive --format json --archive-root ~/tmp/muga-archives --program hello ~/tmp/muga-hello-bundle
muga verify-app-archive ~/tmp/muga-archives/hello-sha256-....mga
MUGA_PROGRAM=hello sh ~/tmp/muga-hello/scripts/package-app.sh
```

The generated app imports `std::env` and `std::cli`, prints and returns
`hello Muga` by default, accepts a positional name such as `Ada`, and accepts a
long option through `--name Ada` or `--name=Ada`. The build commands write
`.mgi`, `.mgc`, and `.mgb` artifacts under the project's default `.muga/build`
directory, and the explicit `--built` commands consume those artifacts without
changing ordinary source-compatible `check` and `run` behavior.
`emit-app-bundle` writes an app directory with bundle-local dependencies and a
launcher; `--source-free` omits copied sources. The launcher and
`run-app-bundle` execute from the bundle-local manifest, resources, and
`.muga/build` artifacts without reading copied source files. `install-app`
writes a wrapper and ownership metadata into the explicit bin directory you
provide; it does not edit shell startup files and only overwrites an existing
launcher when `--replace-owned` verifies prior Muga ownership metadata.
`list-installed-apps` reads the same metadata and reports launcher drift
without mutating files. `install-app`, `list-installed-apps`, and
`uninstall-app` all support JSON output for CI and packager tools.
`uninstall-app` uses the metadata to remove only the owned launcher and
metadata file, leaving bundle directories and shell profiles untouched.
`emit-app-archive` writes a deterministic `.mga` file for moving the bundle as
one artifact; keep the generated `*-sha256-<hash>.mga` file name so
`verify-app-archive [--format text|json] [--expected-hash sha256:<hex>] <archive>` or
`unpack-app-archive [--format text|json] [--expected-hash sha256:<hex>] --output-dir <dir> <archive>`
can validate the bytes before running or installing. The JSON unpack form
reports restored root/file metadata for CI and package-manager wrappers. Use
`--expected-hash sha256:<hex>` when CI, a package manager, or a handoff document
stores the expected hash outside a renamed archive. Generated app projects also
include `README.md` plus `scripts/package-app.sh`; the helper uses `MUGA_BIN`,
`MUGA_PROGRAM`, `MUGA_BUNDLE_DIR`, `MUGA_ARCHIVE_DIR`, and optional
`MUGA_INSTALL_DIR` to create a source-free bundle, run it, archive it, verify
the archive, and explicitly install/list the launcher without editing shell
startup files.

Use the package app template when the first project should show a runnable app
and a reusable local library package together:

```bash
muga new --template package-app ~/tmp/muga-package
cd ~/tmp/muga-package
muga run app/src/main/main.muga -- Ada
muga workspace --format json app/src/main/main.muga
muga build app/src/main/main.muga
muga run --built app/src/main/main.muga -- --name=Ada
sh scripts/package-package-app.sh
```

The generated `app/` package imports `shared/` through a local path dependency,
so source runs, built-artifact runs, workspace JSON, and source-free app bundle
packaging all exercise the same dependency graph without adding a workspace
manifest or registry policy.

Use the report app template when the first project should read a text file and
write a sidecar summary:

```bash
muga new --template report-app ~/tmp/muga-report
cd ~/tmp/muga-report
muga run src/main/main.muga
muga run src/main/main.muga -- data/daily.txt data/custom-summary.txt
muga build src/main/main.muga
muga run --built src/main/main.muga -- data/daily.txt data/built-summary.txt
sh scripts/run-report.sh data/daily.txt data/script-summary.txt
sh scripts/package-report-app.sh
```

The generated report app imports `std::fs`, `std::path`, `std::cli`,
`std::env`, `std::result`, and `std::string`, derives the default output with
`path::with_extension`, maps `io::IOError` into an app boundary string, and
keeps relative data paths stable through `scripts/run-report.sh`. The design
boundary and source-free package helper are documented in
[generated-report-app-template.md](generated-report-app-template.md).

Use the resource export template when the first project should ship a binary
asset as a manifest-declared package resource and materialize it locally:

```bash
muga new --template resource-export ~/tmp/muga-resource
cd ~/tmp/muga-resource
muga run src/main/main.muga -- dist/generated-payload.bin
muga build src/main/main.muga
muga run --built src/main/main.muga -- dist/built-payload.bin
sh scripts/package-resource-export.sh
```

The generated resource app declares `[package] resources = "resources"`, reads
`resources/static/payload.bin` with `std::fs::read_resource_bytes`, writes the
selected output with `std::fs::write_bytes_path`, verifies file path metadata
with `std::fs::path_metadata_path`, verifies the bytes with
`std::fs::read_bytes_path`, and packages the same workflow as a source-free app
bundle.

Use the config app template when the first project should include a typed JSON
settings file and CLI overrides:

```bash
muga new --template config-app ~/tmp/muga-config
muga run ~/tmp/muga-config/src/main/main.muga -- --help
muga run ~/tmp/muga-config/src/main/main.muga -- --config ~/tmp/muga-config/config/settings.json --port=5050
MUGA_CONFIG_PATH=~/tmp/muga-config/config/settings.json muga run ~/tmp/muga-config/src/main/main.muga -- --tag=ops
sh ~/tmp/muga-config/scripts/run-with-config.sh --tag=ops
muga build ~/tmp/muga-config/src/main/main.muga
muga run --built ~/tmp/muga-config/src/main/main.muga -- --config ~/tmp/muga-config/config/settings.json --tags=ops
sh ~/tmp/muga-config/scripts/package-config-app.sh
```

The generated config app writes `config/settings.json`, imports `std::config`,
loads settings through `std::config::load_json_or[T]`, discovers a config path
from `MUGA_CONFIG_PATH` when `--config` is absent, and keeps CLI > config >
defaults precedence visible in the source. The path-discovery boundary is
documented in [config-path-discovery.md](config-path-discovery.md).
Project-aware wrappers can also inspect `muga workspace --format json` for the
manifest root, source root, and resource root before choosing deployment paths;
that contract is documented in
[workspace-manifest-metadata.md](workspace-manifest-metadata.md).
The generated `scripts/run-with-config.sh` and `scripts/package-config-app.sh`
helpers are documented in [config-app-run-helper.md](config-app-run-helper.md).

## First Test Project

The test template shows the smallest `@test` workflow and public API doc path:

```bash
muga new --template test ~/tmp/muga-test
muga test ~/tmp/muga-test/src/main/main.muga
muga test --format json ~/tmp/muga-test/src/main/main.muga
muga doc ~/tmp/muga-test/src/main/main.muga
```

`muga test --format json` is the onboarding path for tools that need structured
test results. `muga doc` renders public package docs from the same interface
model used by `.mgi` artifacts. Generated test and library projects include a
local README with their first `check`, `test`, `doc`, and build commands.

## Package And Artifact Next Steps

After the first generated project works, use the repository samples to learn
the package workflow:

```bash
muga run samples/projects/local_path_app/src/main/main.muga
muga run --format json samples/projects/report_app/src/main/main.muga -- samples/projects/report_app/data/daily.txt ~/tmp/muga-report-app-summary.txt
muga build samples/projects/report_app/src/main/main.muga
muga run --built --format json samples/projects/report_app/src/main/main.muga -- samples/projects/report_app/data/daily.txt ~/tmp/muga-report-app-summary.txt
muga build samples/packages/app/artifact_facade/main.muga
muga check --built samples/packages/app/artifact_facade/main.muga
muga run --built samples/packages/app/artifact_facade/main.muga
```

The `report_app` commands demonstrate args/env, stdout/stderr, text-file handle writes,
JSON run output, `Result`, local dependencies, and `run --built`.

For explicit artifact roots, keep temporary directories under `~/tmp/`:

```bash
mkdir -p ~/tmp/muga-artifacts
muga emit-artifacts --artifact-root ~/tmp/muga-artifacts samples/packages/app/artifact_facade/main.muga
muga check --artifact-root ~/tmp/muga-artifacts samples/packages/app/artifact_facade/main.muga
muga run --artifact-root ~/tmp/muga-artifacts samples/packages/app/artifact_facade/main.muga
```

## Release-Neutral Boundaries

Installation and onboarding docs are release-neutral. They should make the
current command easier to try, but they must not:

- require binary release artifacts or installers before v1;
- require remote package fetching, registries, signing, or publishing
  workflows;
- change package, artifact, lockfile, or source semantics;
- turn `cargo install`, version checks, or quickstarts into release triggers.

Future binary release channels can build on this guide once maintainers decide
that packaging, signing, provenance, platform support, and compatibility policy
are ready.
