# Generated Package App Template
Status: implemented. This records why `muga new --template package-app` is the
next generated-project slice after single-package app, CLI, config, report, and
resource-export starters.

## Goals

Short-term: let new users generate a runnable app plus reusable local library
package without learning the manifest dependency shape from repository samples
first.

Medium-term: make local path dependencies, workspace JSON, build artifacts, and
source-free app bundles part of the first-project path.

Long-term: preserve the package/artifact foundation for later registry and
installer work without adding workspace manifests, registry resolution, or
shell-profile mutation now.

Final goal: move Muga closer to practical adoption by making package reuse and
distribution visible from `muga new`.

## Candidates Compared

| Candidate | Benefit | Cost | Decision |
|---|---|---|---|
| `package-app` generated starter with sibling `app/` and `shared/` packages | Reuses local path dependencies, package-aware build, workspace JSON, and app bundle packaging without new runtime/API surface. | Adds one multi-directory template and package helper to maintain. | Select |
| Rich all-path metadata record | Useful for file tools. | Extends stdlib semantics and does not improve first-project package reuse. | Defer |
| Recursive directory operations | Useful for installers and generators. | Larger host-effect policy around overwrite, symlink, and partial-failure behavior. | Defer |
| Formatting/interpolation or `std::fmt` | Improves source ergonomics. | Adds language/library surface before package distribution gaps are closed. | Defer |
| TOML/config discovery | Useful for operational apps. | Requires broader config policy and overlaps with existing explicit JSON config starter. | Defer |
| Shell-profile installation or registry publishing | Direct adoption/distribution benefit. | Mutates user environment or requires trust/publishing policy. | Defer |

## Selected Shape

`muga new --template package-app <dir>` creates:

- `app/muga.toml` with `[dependencies] <root>_shared = { path = "../shared" }`
- `app/src/main/main.muga`, importing the generated shared package
- `shared/muga.toml`
- `shared/src/greetings/main.muga`
- root `README.md`
- `scripts/package-package-app.sh`

The template does not create a root workspace manifest. The entry path remains
`app/src/main/main.muga`, so existing package-root discovery handles the app
manifest and sibling dependency exactly like checked-in local dependency
samples.

## Validation

- `cli_new_creates_package_app_template`
- `cli_new_lists_project_templates`
- `muga_new_scope_is_documented`
- `generated_package_app_template_is_implemented_and_covered`

The generated helper emits a source-free app bundle, runs it through
`run-app-bundle`, archives it as `.mga`, verifies the archive, and can
explicitly install/list the launcher when `MUGA_INSTALL_DIR` is set. The bundle
must include dependency artifacts while omitting copied source trees.
