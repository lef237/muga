# Fuzzing And Malformed-Input Plan

Status: v1 validation plan. This document describes the malformed-input
surface that should stay covered by deterministic tests now and by future fuzz
harnesses later. It does not add a new language feature, public package
workflow, release requirement, or default CI fuzz job.

## Goals

- Treat parser and artifact readers as trust boundaries.
- Convert every discovered crash, panic, infinite loop, or unbounded allocation
  into a small deterministic regression test.
- Keep fuzzing separate from public performance claims and release timing.
- Keep all temporary files under `~/tmp/` when a harness needs a filesystem
  workspace.

Panic is a bug for malformed external input. The expected result is a
diagnostic, not crash. A reader may reject an input with a stable diagnostic
family, but it must not accept malformed data silently, rewrite malformed input
before reporting it, or fall back to dependency source bodies in artifact-backed
execution.

## Input Surfaces

### Parser And Single-File Syntax

Primary APIs:

- `lexer::lex`
- `parser::parse`
- `muga syntax --format json <entry>`

Deterministic coverage should include valid samples, rejecting conformance
fixtures, mixed newline forms, comment placement, Unicode strings, deeply nested
blocks, incomplete expressions, invalid tokens, invalid attributes, and package
syntax used outside file-based package mode.

Fuzz harness shape: generate UTF-8 source text, run lex and parse, and assert
that the result is either a parsed program or diagnostics with bounded spans.
Do not run resolver, typechecker, package loading, or artifact reads in the
syntax-only harness.

Existing regression anchors:

- `invalid_examples_fail_frontend`
- `rejecting_conformance_programs_report_expected_codes`
- `hash_comments_are_rejected`
- `crlf_newlines_are_counted_once`
- `muga_syntax_scope_is_documented`

### Package Archive `.mgp`

Primary APIs:

- `validate_package_archive_bytes`
- `read_package_archive`
- `materialize_package_archive_bytes`
- `materialize_package_archive`

Malformed cases should include truncated length-prefixed entries, hash
mismatches, duplicate manifest entries, duplicate source/resource entries,
unsorted source/resource entries, resource entries before sources, undeclared
resource entries, non-UTF-8 paths, source/resource-root escapes, unsafe
manifest source/resource roots, tool metadata, non-source entries, empty
archives, and archives that materialize into non-empty destinations.

Fuzz harness shape: generate bytes for the archive reader, optionally pair them
with an expected `sha256:<hex>` value, and assert that accepted archives
round-trip through validation without path escapes. Materialization harnesses
must use fresh `~/tmp/` destinations and must verify that rejected inputs do not
write partial package trees.

Existing regression anchors:

- `package_archive_readback_rejects_hash_mismatch`
- `package_archive_readback_rejects_malformed_entries`
- `package_archive_materialization_rejects_non_empty_destination_without_writes`
- `package_archive_materialization_rejects_unsafe_manifest_source_roots`
- `package_archive_materialization_rejects_unsafe_manifest_resource_roots`
- `manifest_local_archive_dependency_rejects_hash_mismatch`
- `manifest_local_archive_dependency_rejects_stale_cache_hash`
- `manifest_local_archive_dependency_rejects_cache_file_collision`
- `standard_fs_read_resource_text_reports_invalid_paths_and_missing_roots`

### App Bundle Archive `.mga`

Primary APIs:

- `verify_app_bundle_archive`
- `verify_app_bundle_archive_with_expected_hash`
- `unpack_app_bundle_archive`
- `read_verified_app_bundle_archive`
- `read_verified_app_bundle_archive_with_expected_hash`
- `parse_app_bundle_archive_bytes`
- `expected_app_bundle_archive_hash_from_path`
- `validate_app_bundle_expected_archive_hash`

Malformed cases should include renamed archives without the generated
`*-sha256-<hash>.mga` name, hash mismatches between the file name and bytes,
truncated length-prefixed entries, duplicate files, unsorted files, non-UTF-8
headers or paths, absolute paths, `.` / `..` path components, and archives that
target non-empty destinations.

Fuzz harness shape: generate bytes for the app bundle archive parser, pair
them with canonical and non-canonical file names or explicit expected hashes,
and assert that rejected inputs do not create output directories or partial
bundle trees. Materialization harnesses must use fresh `~/tmp/` destinations.

Existing regression anchors:

- `cli_unpack_app_archive_validates_hash_from_filename`
- `app_archive_readback_rejects_malformed_entries_without_writes`
- `app_archive_unpack_rejects_non_empty_output_without_writes`

### Local `muga.lock`

Primary APIs:

- `validate_lockfile_text`
- `muga build <entry>` lockfile validation before update
- `muga why-rebuild --format json <entry>` lockfile explanation

Malformed cases should include duplicate top-level fields, unsupported lockfile
versions, malformed string escapes, missing local archive hashes, path/archive
field conflicts, duplicate dependency entries, graph-inconsistent dependencies,
stale metadata, and malformed files that must remain unchanged after rejection.

Fuzz harness shape: generate lockfile text and pass it to `validate_lockfile_text`
with a synthetic path. Filesystem integration tests should write malformed
lockfiles under `~/tmp/`, run build or why-rebuild, and assert that malformed
lockfiles are rejected rather than overwritten.

Existing regression anchors:

- `build_rejects_malformed_local_path_dependency_lockfile`
- `build_rejects_malformed_local_archive_dependency_lockfile`
- `cli_why_rebuild_json_reports_stale_local_path_lockfile_metadata`
- `cli_why_rebuild_json_reports_fresh_local_archive_lockfile_metadata`

### Package Interface `.mgi`

Primary APIs:

- `PackageInterfaceGraph::read_persisted_file`
- `PackageInterfaceGraph::read_persisted_artifacts`
- `validate_package_references_against_interfaces`

Malformed cases should include invalid headers, malformed field counts,
duplicate package or item identities, invalid dependency rows, invalid escaped
fields, hash mismatches, missing transitive public type interfaces, stale enum
or record shapes, stale function signatures, and wrong-package artifacts.

Fuzz harness shape: generate persisted `.mgi` text and assert that the reader
either rejects it with diagnostics or produces a graph that passes interface
validation invariants. When a generated interface is accepted, downstream
checking must still reject stale or inconsistent public references.

Existing regression anchors:

- `package_check_reports_hash_mismatched_interface_artifact`
- `package_check_reports_stale_generic_interface_artifact_with_regeneration_command`
- `typed_hir_rejects_stale_package_interface_signatures`
- `typed_hir_rejects_stale_package_interface_record_shapes`
- `typed_hir_rejects_stale_package_interface_enum_shapes`
- `typed_hir_rejects_stale_package_interface_item_identity`

### Check-Cache `.mgc`

Primary APIs:

- `read_package_check_artifact`
- `validate_package_check_artifact`
- `package_check_cache_key`

Malformed cases should include invalid headers, malformed dependency counts,
malformed hash rows, hash mismatches, stale source hashes, stale dependency
interface hashes, missing files, and explicit `--built` default-artifact
failures.

Fuzz harness shape: generate persisted `.mgc` text, read it through the cache
artifact reader, and compare accepted values against recomputed cache keys for
known fixtures. Any stale or mismatched cache must fail closed with
regeneration-command context.

Existing regression anchors:

- `package_cache_rejects_stale_checked_artifact`
- `package_cache_rejects_stale_dependency_interface_artifact`
- `cli_check_json_reports_hash_and_regeneration_context_for_stale_check_cache`
- `cli_run_reports_missing_package_check_artifact`
- `cli_run_built_reports_missing_default_check_cache_artifact`
- `cli_run_built_reports_stale_default_check_cache_artifact`

### Implementation Artifact `.mgb`

Primary APIs:

- `implementation_artifact::read_persisted_file`
- `implementation_artifact::read_persisted_artifacts`
- `PackageImplementationArtifact::validate_against_interfaces`
- artifact-backed `muga run`

Malformed cases should include invalid headers, hash mismatches, stale source
hashes, stale interface hashes, stale dependency interface hashes, wrong
package names, duplicate item or local identities, invalid bytecode symbol
references, invalid local counts, invalid jump targets, invalid escaped fields,
and structurally valid artifacts that are inconsistent with loaded `.mgi`
interfaces.

Fuzz harness shape: generate persisted `.mgb` text and assert that accepted
artifacts validate against their interfaces before execution. Artifact-backed
run must keep failing closed; no source-body fallback is allowed when `--built`
or `--artifact-root` selected explicit artifacts.

Existing regression anchors:

- `cli_run_reports_stale_dependency_implementation_artifact`
- `cli_run_reports_dependency_interface_mismatched_implementation_artifact`
- `cli_run_reports_hash_mismatched_dependency_implementation_artifact`
- `cli_run_reports_wrong_package_implementation_artifact`
- `implementation_artifact_rejects_invalid_bytecode_symbol_ref`
- `implementation_artifact_rejects_invalid_bytecode_local_count`
- `implementation_artifact_rejects_invalid_bytecode_jump_target`
- `cli_why_rebuild_json_reports_invalid_and_hash_mismatched_artifacts`

## Promotion Rules

When fuzzing or manual malformed-input testing finds a failure:

1. Minimize the input and add a deterministic regression test first.
2. Place persistent fixtures under `conformance/` only when they are source
   language contract cases; otherwise prefer focused Rust tests near the reader.
3. Assert the stable diagnostic family and the no-panic/no-silent-accept
   behavior.
4. Preserve malformed files after rejection when the user supplied the file.
5. Keep generated temp workspaces under `~/tmp/`.
6. Record any new class of malformed input in this document before broadening
   fuzz harness scope.

## Future Harnesses

Future `cargo fuzz` or equivalent harnesses should be opt-in local tooling, not
part of `scripts/v1-release-gate.sh` by default. The release gate should keep
running deterministic regressions, while fuzz jobs can run on a separate manual
or scheduled workflow once dependency, runtime, and corpus costs are understood.
