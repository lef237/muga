# Documentation Guide

Status: documentation map. This file is the human-oriented entry point for the
documentation tree. It keeps README short while preserving the detailed design,
decision, and release-readiness records that are useful for maintainers and
coding agents.

## Reading Order

Use this order when you need to understand the project direction before making
changes:

1. [../README.md](../README.md) for the user-facing summary.
2. [strategy-and-implementation-plan.md](strategy-and-implementation-plan.md)
   for the north star, phase order, and non-goals.
3. [../ROADMAP.md](../ROADMAP.md) for the current implementation priority.
4. [implementation-resume-plan.md](implementation-resume-plan.md) for the
   implementation ledger and concrete next-slice handoff.
5. [practical-language-readiness.md](practical-language-readiness.md) before
   starting broad stdlib, resource, service, concurrency, or performance work.
6. The relevant spec under [../spec](../spec) before changing language
   behavior.

## Current Direction

Muga's current priority is Core Capability Acceleration. Keep the language
model small and explicit, but prefer a thin end-to-end implementation of a
practical core capability over another one-off polish slice when the capability
fits Muga's design. The active order is process execution, structured task
groups, service IO, performance foundations, then distribution/ecosystem work.
Each slice still needs a public contract, runtime behavior, artifact behavior,
sample or template coverage, focused tests, and release-readiness evidence.

The next implementation-facing handoff is in
[implementation-resume-plan.md](implementation-resume-plan.md): start the core
acceleration queue unless a maintainer explicitly redirects the work. The
first preferred slice is the `std::process` spine, followed by structured
task groups and then service IO. Existing package/resource adoption work is
still the baseline that accelerated slices must preserve.
The shell-agnostic JSON completion spec now reuses `CliSchema`,
shell-specific scripts traverse nested command scopes over the same model,
`@cli(value_source: "file"|"directory")` carries static path sources through
source and artifact workflows, and
[cli-completion-installer-integration.md](cli-completion-installer-integration.md)
records the non-mutating `emit-cli-completions` and `emit-app-completions`
package emission paths.
[config-path-discovery.md](config-path-discovery.md) records the generated
config-app `MUGA_CONFIG_PATH` path discovery slice before TOML parsing or
runtime-owned config/resource discovery.
[config-app-run-helper.md](config-app-run-helper.md) records the generated
config-app README plus `scripts/run-with-config.sh` and
`scripts/package-config-app.sh` helpers that apply the path-discovery policy and
package source-free bundles without runtime magic.
[workspace-manifest-metadata.md](workspace-manifest-metadata.md) records the
`workspace --format json` manifest root/source root/resource root metadata
slice that tooling can use before runtime resource lookup and installed layouts.
[package-resource-archives.md](package-resource-archives.md) records explicit
`[package] resources = "resources"` inclusion in package hashes, `.mgp`
archives, non-mutating verification, materialization, and local archive
dependency caches.
[runtime-package-resource-lookup.md](runtime-package-resource-lookup.md)
records the read-only `std::fs::read_resource_text(package, path)` and
`std::fs::read_resource_bytes(package, path)` runtime APIs over those
manifest-declared resources.
[binary-file-read.md](binary-file-read.md) records read-only
`std::fs::read_bytes` / `std::fs::read_bytes_path` plus `bytes::at` inspection.
[binary-file-write.md](binary-file-write.md) records full-file
`std::fs::write_bytes` / `std::fs::write_bytes_path` over opaque `Bytes`.
[bytes-sha256-hash.md](bytes-sha256-hash.md) records the narrow
`std::hash::sha256_hex(bytes)` digest helper.
[resource-bytes-export-sample.md](resource-bytes-export-sample.md) records the
manifest resource byte export sample and generated starter that compose
resource lookup, hashing, and full-file binary writes without adding new API
surface.
[path-normalize.md](path-normalize.md) records the narrow
`std::path::normalize(path)` pure lexical cleanup helper.
[path-with-file-name.md](path-with-file-name.md) records the narrow
`std::path::with_file_name(path, new_file_name)` pure path transformation.
[path-with-extension.md](path-with-extension.md) records the narrow
`std::path::with_extension(path, new_extension)` pure path transformation.
[path-strip-prefix.md](path-strip-prefix.md) records the narrow
`std::path::strip_prefix(path, base)` pure path relationship helper.
[env-current-dir.md](env-current-dir.md) records the narrow
`std::env::current_dir()` ambient current-directory read as an explicit
`Result[path::Path, io::IOError]`.
[env-temp-dir.md](env-temp-dir.md) records the narrow `std::env::temp_dir()`
ambient temporary-directory read as an explicit `Result[path::Path, io::IOError]`.
[fs-canonicalize-path.md](fs-canonicalize-path.md) records the narrow
`std::fs::canonicalize_path(target_path)` existing-path host resolution helper.
[fs-file-size.md](fs-file-size.md) records the narrow
`std::fs::file_size_path(path)` scalar metadata helper.
[fs-modified-unix-millis.md](fs-modified-unix-millis.md) records the narrow
`std::fs::modified_unix_millis_path(path)` timestamp helper.
[fs-file-metadata-record.md](fs-file-metadata-record.md) records the narrow
`std::fs::FileMetadata` regular-file metadata record.
[fs-path-status.md](fs-path-status.md) records the narrow
`std::fs::PathStatus` grouping layer over existing path metadata predicates.
[fs-path-info.md](fs-path-info.md) records the narrow `std::fs::PathKind` and
`std::fs::PathInfo` grouping layer over `PathStatus`.
[fs-path-metadata.md](fs-path-metadata.md) records the narrow
`std::fs::PathMetadata` existing-path metadata record.
[fs-path-size-metadata.md](fs-path-size-metadata.md) records the narrow
`std::fs::PathSizeMetadata` all-path metadata record with optional file size.
[fs-read-dir-recursive.md](fs-read-dir-recursive.md) records the narrow
`std::fs::read_dir_recursive_path(root_path)` read-only traversal helper.
[fs-directory-size-metadata.md](fs-directory-size-metadata.md) records the narrow
`std::fs::DirectorySizeMetadata` recursive directory aggregate.
[fs-rename-path.md](fs-rename-path.md) records the narrow
`std::fs::rename_path(from, to)` two-path filesystem helper.
[fs-remove-dir-all.md](fs-remove-dir-all.md) records the narrow
`std::fs::remove_dir_all_path(dir_path)` recursive directory removal helper.
[fs-copy-dir-all.md](fs-copy-dir-all.md) records the narrow
`std::fs::copy_dir_all_path(from, to)` recursive directory copy helper.
[fs-move-dir-all.md](fs-move-dir-all.md) records the copy-then-remove
`std::fs::move_dir_all_path(from, to)` recursive directory move helper.
[std-fmt-text-layout.md](std-fmt-text-layout.md) records the narrow pure
`std::fmt` repeat, padding, scalar truncation, and explicit placeholder helpers.
[installed-app-bundles.md](installed-app-bundles.md) records the first
non-mutating `emit-app-bundle` layout, optional source-free output,
bundle-local dependency policy, source-free `run-app-bundle` execution, and
launcher plus guarded install/uninstall/list ownership metadata and completion
package emission boundaries for manifest projects.

## Audit Result

The documentation tree is large because it has been doing three jobs:

- User-facing onboarding and learning.
- Stable contracts for language, tooling, package artifacts, and standard
  library behavior.
- Historical release-readiness evidence for why the project chose small,
  ordered implementation slices.

Do not treat all three classes as equal reading material. The first two classes
are active product documentation. The third class is an audit trail, and it is
the primary cleanup target now that the code, samples, and tests carry the
implemented behavior.

References are not retention proof. A document should remain only when it is an
active user guide, a public contract, or a design constraint that cannot be
understood from code, tests, samples, or specs. Historical comparison notes,
case studies, and completed selection logs should be deleted or folded into a
short active document when they no longer drive implementation.

Current cleanup priority:

1. Keep README and this guide short.
2. Keep active handoffs in `ROADMAP.md` and `implementation-resume-plan.md`.
3. Keep feature contracts in focused design docs or specs.
4. Prefer Rust tests, Muga samples, and executable CLI contracts over
   release-readiness assertions that merely prove a documentation file exists.
5. Delete historical files once the current behavior is covered by code,
   samples, tests, or a smaller active contract.

## Document Classes

### User And Learning Docs

- [installation-and-onboarding.md](installation-and-onboarding.md): install,
  first project, shell completions, generated app completions, and artifact
  quickstarts.
- [muga-by-example.md](muga-by-example.md): example-driven learning path.
- [../README.md](../README.md): concise project entry point.

### Specifications

- [../mini-language-spec-v1.md](../mini-language-spec-v1.md): compact v1
  language reference.
- [../spec/001-core-language.md](../spec/001-core-language.md)
- [../spec/002-name-resolution.md](../spec/002-name-resolution.md)
- [../spec/003-typing.md](../spec/003-typing.md)
- [../spec/004-functions.md](../spec/004-functions.md)
- [../spec/005-records.md](../spec/005-records.md)
- [../spec/006-packages.md](../spec/006-packages.md)
- [../spec/007-concurrency-draft.md](../spec/007-concurrency-draft.md)
- [../spec/008-collections.md](../spec/008-collections.md)
- [../spec/009-generics.md](../spec/009-generics.md)
- [../spec/010-references-draft.md](../spec/010-references-draft.md)
- [../spec/011-value-semantics.md](../spec/011-value-semantics.md)
- [../spec/012-protocols-deferred.md](../spec/012-protocols-deferred.md)
- [../spec/013-enums-results.md](../spec/013-enums-results.md)

### Tooling And Contracts

- [diagnostics-and-output.md](diagnostics-and-output.md): stable JSON and text
  command-output contracts.
- [editor-json-workflow.md](editor-json-workflow.md): concrete editor/LSP
  adapter path over existing commands.
- [artifact-cache-explanations.md](artifact-cache-explanations.md):
  `muga why-rebuild` contract.
- [shell-completions-and-doctor.md](shell-completions-and-doctor.md):
  tool-only completion and environment-check commands.
- [cli-schema-shell-completions.md](cli-schema-shell-completions.md):
  generated app shell completions from `CliSchema`.
- [cli-completion-json-spec.md](cli-completion-json-spec.md):
  shell-agnostic generated app completion contracts from `CliSchema`.
- [cli-completion-installer-integration.md](cli-completion-installer-integration.md):
  non-mutating generated app completion package emission for installers.
- [config-path-discovery.md](config-path-discovery.md):
  generated config-app `MUGA_CONFIG_PATH` config path discovery.
- [config-app-run-helper.md](config-app-run-helper.md):
  generated config-app README plus run/package helpers with optional explicit
  install/list handoff.
- [generated-report-app-template.md](generated-report-app-template.md):
  generated single-project file-processing starter and package helper for
  `muga new --template report-app`.
- [runtime-package-resource-lookup.md](runtime-package-resource-lookup.md):
  read-only `std::fs::read_resource_text` lookup for manifest-declared package
  resources in source, test, archive dependency, and built-artifact runs.
- [installed-app-bundles.md](installed-app-bundles.md):
  non-mutating app bundles with a `bin/<program>` launcher, optional
  source-free output, and source-free artifact runner.
- [workspace-manifest-metadata.md](workspace-manifest-metadata.md):
  manifest roots, source/resource roots, and dependency source/resource metadata
  in workspace JSON.
- [package-resource-archives.md](package-resource-archives.md):
  manifest-declared resource inclusion for package hashes and `.mgp` archives.
- [mgi-api-diff.md](mgi-api-diff.md): `.mgi` API diff contract, library comparator, and `muga api-diff` CLI.

### Standard Library And API Boundaries

- [standard-library-review-rules.md](standard-library-review-rules.md)
- [stdlib-package-samples-review.md](stdlib-package-samples-review.md)
- [path-with-file-name.md](path-with-file-name.md)
- [path-with-extension.md](path-with-extension.md)
- [fs-file-size.md](fs-file-size.md)
- [fs-file-metadata-record.md](fs-file-metadata-record.md)
- [fs-path-status.md](fs-path-status.md)
- [fs-rename-path.md](fs-rename-path.md)
- [fs-move-dir-all.md](fs-move-dir-all.md)
- [std-fmt-text-layout.md](std-fmt-text-layout.md)
- [std-json-first-slice.md](std-json-first-slice.md)
- [std-json-implementation-audit.md](std-json-implementation-audit.md)
- [std-config-json-loading.md](std-config-json-loading.md)
- [json-schema-decoding.md](json-schema-decoding.md)
- [json-required-decoding.md](json-required-decoding.md)
- [json-decoder-target-expansion.md](json-decoder-target-expansion.md)
- [json-config-schema-polish.md](json-config-schema-polish.md)
- [json-config-strict-unknown-fields.md](json-config-strict-unknown-fields.md)
- [json-config-alias-metadata.md](json-config-alias-metadata.md)
- [json-config-validation-attributes.md](json-config-validation-attributes.md)
- [json-config-schema-export.md](json-config-schema-export.md)
- [json-typed-encoding.md](json-typed-encoding.md)

### Resource And Runtime Boundaries

- [opaque-resource-handles.md](opaque-resource-handles.md)
- [text-output-file-handles.md](text-output-file-handles.md)
- [lexical-resource-cleanup.md](lexical-resource-cleanup.md)
- [benchmark-health-checks.md](benchmark-health-checks.md)
- [fuzzing-malformed-input-plan.md](fuzzing-malformed-input-plan.md)

### CLI Schema Design

- [cli-parser-schema.md](cli-parser-schema.md)
- [strict-cli-parser-schema.md](strict-cli-parser-schema.md)
- [strict-cli-no-default-usage.md](strict-cli-no-default-usage.md)
- [cli-field-metadata.md](cli-field-metadata.md)
- [cli-command-metadata.md](cli-command-metadata.md)
- [cli-short-option-metadata.md](cli-short-option-metadata.md)
- [cli-positional-field-metadata.md](cli-positional-field-metadata.md)
- [cli-built-in-help-policy.md](cli-built-in-help-policy.md)
- [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md)
- [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md)
- [cli-subcommand-metadata.md](cli-subcommand-metadata.md)
- [cli-wrapper-root-options.md](cli-wrapper-root-options.md)

### Strategy, Release, And Maintenance

- [v1-release-checklist.md](v1-release-checklist.md)
- [release-gate-alignment.md](release-gate-alignment.md)
- [registry-security-design.md](registry-security-design.md)
- [edition-feature-fingerprint-policy.md](edition-feature-fingerprint-policy.md)
- [modern-language-gap-inventory-2026-05-22.md](modern-language-gap-inventory-2026-05-22.md)
- [modern-language-gap-decisions-2026-05-22.md](modern-language-gap-decisions-2026-05-22.md)
- [internal/identity-model.md](internal/identity-model.md)

## Decision Logs

Files named `post-*-adoption-gap-selection.md` are historical decision logs.
They are intentionally kept because release-readiness tests use them as evidence
that the project chose small, ordered slices rather than expanding the surface
opportunistically. They are not the primary reading path.

The current active decision chain is:

- [cli-schema-shell-completions.md](cli-schema-shell-completions.md): generated
  app shell completions from `CliSchema`.
- [post-cli-schema-shell-completion-adoption-gap-selection.md](post-cli-schema-shell-completion-adoption-gap-selection.md):
  completion onboarding and packaging hook adoption.
- [cli-completion-json-spec.md](cli-completion-json-spec.md):
  shell-agnostic JSON completion specs.
- [cli-completion-value-sources.md](cli-completion-value-sources.md):
  static file/directory value-source metadata for generated completions.
- [cli-completion-installer-integration.md](cli-completion-installer-integration.md):
  explicit completion package emission before shell-profile mutation.
- [config-path-discovery.md](config-path-discovery.md): generated config-app
  path discovery through `MUGA_CONFIG_PATH`.
- [config-app-run-helper.md](config-app-run-helper.md): generated config-app
  local README and `scripts/run-with-config.sh`.
- [workspace-manifest-metadata.md](workspace-manifest-metadata.md): manifest
  root/source/resource root metadata for project-aware tooling.
- [package-resource-archives.md](package-resource-archives.md): explicit
  resource archive inclusion.
- [runtime-package-resource-lookup.md](runtime-package-resource-lookup.md):
  read-only runtime resource lookup after source/archive/cache resource roots.
- [installed-app-bundles.md](installed-app-bundles.md): first installed-app
  layout, dependency bundling, optional source-free output, source-free bundle
  runner, non-mutating install wrapper with ownership metadata, guarded owned
  updates, metadata-backed uninstall, installed-app inventory, app completion
  package emission, and non-mutating hash verification for `.mga` archive transport.
- [generated-package-app-template.md](generated-package-app-template.md):
  generated app plus local library package starter.
- Next: keep runtime `Bytes`, shell-profile mutation, registry publishing, and
  dynamic completion producers as separate scoped decisions.

When a decision log becomes obsolete, do not delete it casually. First update
the roadmap/resume plan, remove or move any release-readiness assertions that
depend on it, and leave a short replacement note that says where the current
contract now lives.

## Cleanup Policy

Prefer these cleanup actions:

- Keep README short and user-facing.
- Move implementation history to this guide, the roadmap, or the implementation
  resume plan.
- Keep design contracts near the feature they specify.
- Keep decision logs only when they still explain ordering or prevent repeated
  debate.
- Remove duplicated prose only after tests and cross-links point at the new
  source of truth.

Avoid these cleanup actions:

- Deleting decision logs that release-readiness still uses as evidence.
- Moving spec content into README.
- Mixing future design candidates into runnable `samples/`.
- Adding release, publish, registry, or installer recommendations before the v1
  checklist and trust boundaries say they are ready.
