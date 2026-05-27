# Resource Bytes Export Sample

Status: `samples/projects/resource_export` is implemented as a manifest project
sample, and `muga new --template resource-export` is implemented as a generated
starter. Both read declared binary package resources, compute a SHA-256 digest,
write the bytes to an explicit output, verify path metadata, and read them
back.

## Goal

Short-Term Goal: prove that the existing `Bytes`, package resource lookup,
hashing, path, temp-dir, and binary write APIs compose into a practical asset
export workflow.

Medium-Term Goal: give onboarding material a runnable project that resembles
small tool behavior without adding streams, codecs, resource handles, or broader
crypto.

Long-Term Goal: keep asset distribution and local materialization workflows
portable across source runs, explicit artifact runs, and app bundles.

Final Goal: make Muga useful for small application and tooling workflows that
ship deterministic resources and verify their bytes.

## Selected Shape

The sample is a manifest project with `[package] resources = "resources"`.
Runtime code calls `fs::read_resource_bytes("resource_export",
"static/payload.bin")`, hashes the returned `Bytes` with `hash::sha256_hex`,
writes the payload with `fs::write_bytes_path`, confirms the materialized path
with `fs::path_metadata_path`, verifies bytes with `fs::read_bytes_path`, and
removes the temporary output. The same sample is covered as a source-free app
bundle so declared binary resources stay usable without copied source files.

The generated `resource-export` template applies the same API boundary to an
inferred package name, writes `resources/static/payload.bin`, leaves the
selected output file in `dist/` by default so new users can inspect it, reports
`size|kind|is_file|sha256|output`, and includes
`scripts/package-resource-export.sh` for source-free bundle, `.mga`, and
archive-verification handoff, with optional explicit install/list through
`MUGA_INSTALL_DIR`.

## Candidates Compared

| Candidate | Benefit | Cost | Decision |
|---|---|---|---|
| Project sample over existing APIs | Demonstrates a real workflow with no new semantics and works for source plus artifact-backed runs. | Slightly more sample surface to maintain. | Select |
| Generated resource-export starter | Makes the resource workflow discoverable from `muga new` and exercises source-free app packaging with no new API surface. | Adds one template and helper script to maintain. | Select |
| New resource export helper | Shorter app code. | Premature API; hides path, cleanup, and verification policy that applications should choose. | Defer |
| Binary stream/resource handles | Scales to large assets. | Requires lifetime, buffering, errors, and `using` policy for binary handles. | Defer |
| Docs-only mention | No code maintenance. | Does not prove the workflow stays runnable. | Reject |

## Validation

- `manifest_resource_export_project_sample_runs`
- `manifest_resource_export_project_sample_runs_against_emitted_artifacts`
- `manifest_resource_export_source_free_bundle_runs_without_sources`
- `cli_new_creates_resource_export_template`
- `resource_bytes_export_sample_is_documented_and_covered`
- `scripts/v1-release-gate.sh` emits and runs the source-free resource export
  bundle
