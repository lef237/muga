# Installed App Bundles

Status: the first non-mutating installed-app layout is implemented for
manifest projects, including local path and local archive dependencies, through
`muga emit-app-bundle`. A non-mutating `muga install-app` wrapper command can
place an app launcher in a user-chosen bin directory without editing shell
profiles, and `muga list-installed-apps` reports owned launchers and drift
without changing files. `muga emit-app-archive` and `muga unpack-app-archive`
provide a hash-bearing single-file transport form for those bundles, and
`muga verify-app-archive` validates that transport without writing files.
`muga emit-app-completions` emits generated CLI completion packages from
bundle interfaces. `muga run-app-bundle` executes a bundle from its manifest,
resources, and `.muga/build` artifacts without reading copied source files.

This slice packages the already-supported source, resource, and built-artifact
runtime boundary into a directory that can be moved or archived without changing
the user's shell profile or installing files into global locations.

## Goals

Short-Term Goal: make generated and small manifest apps runnable from any
current directory through a bundle-local launcher.

Medium-Term Goal: keep `muga build`, manifest-declared resources, `muga.lock`,
and explicit `run --built` behavior aligned in a copied layout.

Long-Term Goal: preserve room for source-free artifact runners, registry
packages, and package-manager installation without committing to host mutation
now.

Final Goal: make Muga apps practical to share by giving users a concrete bundle
boundary and explicit launcher placement before registry publishing exists.

## Implemented Contract

```sh
muga emit-app-bundle [--format text|json] [--source-free] --output-dir <dir> [--program <name>] <source-file>
muga run-app-bundle [--format text|json] <bundle-dir> [-- <program-arg>...]
muga install-app [--format text|json] [--replace-owned] --output-dir <bin-dir> [--program <name>] <bundle-dir>
muga list-installed-apps [--format text|json] --output-dir <bin-dir>
muga uninstall-app [--format text|json] --output-dir <bin-dir> --program <name>
muga emit-app-completions [--format text|json] --output-dir <dir> [--program <name>] --type <type> [--package <package>] <bundle-dir>
muga emit-app-archive [--format text|json] --archive-root <dir> [--program <name>] <bundle-dir>
muga verify-app-archive [--format text|json] [--expected-hash sha256:<hex>] <archive-file>
muga unpack-app-archive [--format text|json] [--expected-hash sha256:<hex>] --output-dir <dir> <archive-file>
```

`emit-app-bundle` requires a `muga.toml` manifest project and an absent or
empty `--output-dir`. The optional `--program` value names the launcher under
`bin/`; when omitted, the launcher name is derived from the manifest package
path. `--source-free` omits copied root and dependency source trees while
keeping manifests, resources, `.muga/build`, `.muga/app-bundle`, and launchers.
`--format json` reports the bundle root, entry, launcher, source mode, artifact
paths, and written file paths for CI and packager tools.

The emitted layout is source-backed:

```txt
<dir>/
  muga.toml
  muga.lock
  src/...                 # .muga source files
  resources/...           # manifest-declared resources, when present
  .muga/build/...         # default built artifacts from muga build
  .muga/bundle-deps/...   # dependency source/resource trees, when present
  .muga/app-bundle        # entry package metadata for run-app-bundle
  bin/<program>           # launcher script
  README.md
```

The generated `README.md` is part of the handoff surface. It includes copyable
commands for `muga run-app-bundle .`, `muga install-app`, `muga uninstall-app`,
`muga list-installed-apps`, `muga emit-app-completions`, `muga emit-app-archive`, and
`muga verify-app-archive` handoff, while still documenting that Muga does not
edit shell startup files.

When dependencies are present, the bundle re-renders the root and dependency
`muga.toml` files with bundle-local path dependencies under
`.muga/bundle-deps/<package path>`. Local archive dependencies are copied from
their already-validated materialized cache and become bundle-local path
dependencies inside the app bundle. Source-backed bundles write `muga.lock`
for the copied dependency trees, not the original developer-machine paths.
`--source-free` omits that source-hash lockfile because the runner consumes
bundle-local artifacts and resources rather than copied source trees.

The launcher uses `muga` from `PATH`, or `MUGA_BIN` when set, and runs:

```sh
muga run-app-bundle <bundle-root> -- "$@"
```

This runner reads `.muga/app-bundle` for the entry package plus the
bundle-local `muga.toml`, resource directories, `.mgi` interfaces, and `.mgb`
implementation artifacts. It does not read the copied source tree during
execution, so source-backed and `--source-free` layouts share the same runtime
boundary.
Resource lookup still flows through
`std::fs::read_resource_text(package, path)` and the bundle-local manifest
resource root.

`install-app` writes a wrapper launcher into the requested `--output-dir`.
When `--program` is omitted, the bundle must contain exactly one launcher under
`bin/`. The command creates the output directory if needed, but it never
overwrites an existing launcher or install metadata unless `--replace-owned`
can verify prior Muga ownership, and it never edits shell startup files. Before
writing the wrapper or metadata, it validates that the bundle metadata and
`.muga/build` artifacts can be loaded without source files. It also records
ownership metadata at
`<bin-dir>/.muga/installed-apps/<program>.toml` with the installed launcher,
bundle root, and bundle launcher paths for future guarded update/uninstall
workflows. `--format json` reports the bundle, install directory, launcher,
metadata, program, replace-owned mode, and written file paths for CI and
packager tools.

`list-installed-apps [--format text|json] --output-dir <bin-dir>` reads that
ownership metadata without mutating files. Text output is one tab-separated row
per app, or an `empty` row when no metadata exists. JSON output includes the
output directory, metadata directory, and per-app launcher, metadata, bundle,
bundle launcher, state, and reason fields. The state is `ready`,
`invalidMetadata`, `metadataMismatch`, `missingLauncher`, `launcherMismatch`,
or `missingBundleLauncher`. Drift is reported as data, not a command failure,
so installers and users can inspect a bin directory before deciding whether to
reinstall, uninstall, or repair files.

`uninstall-app` requires an explicit `--program` and the same `--output-dir`.
It reads ownership metadata, verifies the installed launcher still matches the
Muga wrapper recorded there, and removes only the launcher plus metadata file.
It does not remove bundle directories, shell-profile entries, or package-manager
state. `--format json` reports the install directory, launcher, metadata,
program, and removed file paths; structured errors keep stderr empty.

Generated package helper scripts stay non-mutating by default. When
`MUGA_INSTALL_DIR` is set, they run `install-app --replace-owned` and
`list-installed-apps` against that explicit bin directory after archive
verification, giving users a copyable local install handoff without shell
profile edits.

`emit-app-completions` reads the bundle's `.muga/app-bundle` entry package,
`.muga/build` interface artifacts, and single `bin/<program>` launcher when
`--program` is omitted, so source-free bundles can still emit the same bash,
zsh, fish, and `.completions.json` package as `emit-cli-completions`. It writes
only to the requested `--output-dir`, supports JSON file metadata output, and
never edits shell profiles.

`emit-app-archive` writes a deterministic `.mga` archive for a bundle directory
under `--archive-root`, naming the file as
`<program>-sha256-<64-hex>.mga`; it validates the bundle metadata and
`.muga/build` artifacts before writing the archive. `--format json` reports the
archive path, program, hash, and archived file paths for CI and packager tools.
`verify-app-archive` validates the archive bytes and entry headers without
writing files. By default it requires the generated hash-bearing file name and
validates bytes against the encoded `sha256:<hex>`; `--expected-hash
sha256:<hex>` supports renamed archives and out-of-band hashes.
`unpack-app-archive` uses the same generated name or explicit expected-hash
validation before creating output directories, restores the archive into an
absent or empty directory, and marks restored `bin/<program>` launchers
executable. `--format json` reports the restored root and file paths, while
structured errors include the archive and requested output directory for CI and
packager tools.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| Source-backed bundle plus launcher | Smallest practical layout, copies resources and artifacts, avoids host mutation. | Requires `muga` at launch time and keeps source files in the bundle. | Select |
| Source-free bundle runner | Runs from `.mgi` / `.mgb` artifacts and manifest resources without reading source files; supports artifact-only emitted layouts. | Adds one explicit app-bundle execution command. | Select |
| Source-free emitted bundle | Smaller and closer to native app distribution while keeping manifest/resource/artifact execution intact. | Source inspection requires a separate source-backed bundle or archive. | Select |
| Machine-readable bundle emission | Lets CI and packagers consume bundle root/launcher/artifact/file metadata without scraping text output. | Does not install bundles, choose PATH locations, or publish registries. | Select |
| Dependency-aware source tree bundle | Needed for larger apps with local path or archive dependencies; keeps runtime offline and self-contained. | Re-renders manifests and lockfile for the bundle layout, so comments and original archive/path spelling are not preserved. | Select |
| Preserve archive dependencies verbatim | Retains archive identity in the emitted manifest. | Requires archive file placement and first-run cache policy; a missing cache would make the launcher depend on archive paths. | Defer |
| Non-mutating install wrapper | Gives users a PATH-friendly launcher while keeping shell-profile edits and package-manager ownership outside Muga. | The wrapper points at the current bundle path, so moving the bundle requires reinstalling. | Select |
| Installer ownership metadata | Gives future update/uninstall commands a machine-readable ownership record without taking over shell profiles. | Adds one hidden metadata file next to the chosen bin directory. | Select |
| Guarded owned update | Lets users repoint an installed launcher to a new bundle without deleting files, but only when metadata proves Muga owns the launcher path. | Does not remove old bundles or implement uninstall. | Select |
| Metadata-backed uninstall | Gives users a reversible-feeling install lifecycle while limiting deletion to files proven to be Muga-owned. | Leaves old bundle directories and shell profile cleanup to the user. | Select |
| Machine-readable install/uninstall | Lets CI and packagers consume launcher, metadata, and removed/written file paths without scraping text output. | Does not choose PATH locations, mutate shell profiles, or publish registries. | Select |
| Non-mutating installed-app inventory | Lets users, CI, and package-manager wrappers inspect owned launchers and drift before changing files. | Adds a small status taxonomy over install metadata and launcher contents. | Select |
| Generated helper install/list hook | Lets generated project package scripts complete the local explicit-bin install handoff when users opt in through `MUGA_INSTALL_DIR`. | Still leaves shell profile edits and package-manager ownership outside the helper. | Select |
| Source-free completion package emission | Lets packagers generate completion files from distributed bundles without needing source trees, with JSON file metadata for automation. | Requires CLI schemas to be present in package interfaces. | Select |
| Mutating `muga install` | Familiar end-user command. | Must decide shell profile edits, uninstall behavior, package-manager ownership, and permissions. | Defer |
| Archive-only app bundle | Easy to transport and keeps the directory layout as the execution boundary after extraction. | Still requires an explicit unpack step before launch or install. | Select |
| Hash-bearing app archive filename | Reuses the deterministic `.mga` filename as the local integrity check and fails before writing files when bytes are renamed incorrectly or modified. | Renaming an archive to a non-generated name prevents unpacking until it is re-emitted or restored to the generated name. | Select |
| Non-mutating app archive verification | Gives CI, package managers, and recipients a way to check `.mga` bytes before choosing an output directory. | Adds one CLI command; explicit hash mode is required for renamed archives. | Select |
| Machine-readable archive emission | Lets CI and packagers consume `.mga` path/hash/file metadata without scraping text output. | Does not install bundles, publish registries, or choose shell completion locations. | Select |
| Machine-readable archive unpack | Lets CI and packagers consume restored bundle root/file metadata and unpack failures without scraping text output. | Does not install bundles, choose PATH locations, or publish registries. | Select |
| Pre-install/pre-archive bundle validation | Prevents broken bundles from becoming PATH launchers or transport artifacts. | Adds artifact-loading cost before install/archive writes. | Select |

## Non-Goals

This slice does not add:

- shell-profile mutation or automatic global install locations;
- registry publishing or package-manager metadata;
- binary streams, mutable bytes, codecs, broad cryptographic APIs, or resource listing APIs;
- automatic config discovery beyond existing generated helpers.

## Validation

Focused coverage lives in `tests/examples.rs`:

- `cli_emit_app_bundle_writes_source_backed_layout_and_launcher`
- `emit_app_bundle_reports_bundle_local_artifact_paths`
- `cli_emit_app_bundle_writes_dependency_aware_layout_and_launcher`
- `cli_emit_app_bundle_rejects_output_inside_dependency_source_root`
- `cli_install_app_writes_non_mutating_launcher_for_bundle`
- `cli_list_installed_apps_reports_owned_launchers`
- `cli_install_and_archive_reject_broken_app_bundle_without_writes`
- `cli_emit_app_bundle_source_free_uses_artifacts_without_bundle_sources`
- `cli_emit_app_completions_writes_package_from_source_free_bundle`
- `cli_emit_and_unpack_app_archive_round_trips_bundle_launcher`
- `cli_unpack_app_archive_validates_hash_from_filename`
- `cli_emit_app_archive_rejects_archive_root_inside_bundle`

Release-readiness coverage keeps this document, CLI help, library emission,
launcher behavior, resource copying, dependency layout, and the handoff aligned.

## Next

The next distribution slice should stay narrow: binary streams/codecs/handles,
broad cryptographic APIs, shell-profile mutation, registry publishing,
package-manager metadata, and dynamic completion producers remain separate
decisions.
