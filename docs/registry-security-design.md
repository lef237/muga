# Registry Security Design

Status: future trust design. This document preserves the current `.mgp` hash
foundation and defines the security boundary that URL, Git, registry, signing,
provenance, lockfile enforcement, cache validation, and malicious-package
handling must satisfy before remote fetching is added.

This is not a new dependency form, registry implementation, publishing
workflow, signing system, `muga audit`, SBOM generator, release trigger, or
default CI/network requirement.

## Current Foundation

The implemented local package path already provides the base trust primitive:

- `package_content_hash` computes `sha256:<hex>` over `muga.toml`, sorted
  `.muga` source files under the manifest source root, and sorted UTF-8 files
  under an optional manifest-declared resource root.
- `write_package_archive` and `muga emit-package-archive` write deterministic
  `.mgp` source/resource archives whose bytes are the canonical package content
  input.
- `validate_package_archive_bytes` and `read_package_archive` validate archive
  bytes, source entries, resource entries, and optional expected hashes without
  trusting filenames.
- `materialize_package_archive_bytes` and `materialize_package_archive`
  materialize validated source/resource archives only into absent or empty
  destinations.
- local `.mgp` dependencies require `[dependencies] name = { archive = "...",
  hash = "sha256:<hex>" }`, validate the declared hash, and materialize sources
  plus declared resources under `.muga/packages/<package>-sha256-<hash>`.
- runtime `std::fs::read_resource_text(package, path)` reads only from those
  manifest-declared resource roots and does not expose host paths.
- `muga build` writes and validates local path/archive `muga.lock` metadata.
- `muga why-rebuild --format json` exposes lockfile and archive-cache state
  without mutating the project.

Local path dependency `source_hash` values remain development metadata. Local
archive dependency `hash` values are package content identity. Future remote
dependencies must use package content identity, not source location, as the
authoritative package version.

## Trust Model

The dependency system should remain layered:

1. Content identity is authoritative. A package version is identified by
   `sha256:<hex>` over canonical archive bytes.
2. Transport is replaceable. URL, Git, mirrors, caches, or a registry may
   provide bytes, but none of those locations define package identity.
3. Lockfiles are the build truth after resolution. A locked build verifies
   bytes against `muga.lock` and fails closed on mismatch.
4. Registries are naming and discovery services. A registry may map
   `(name, version)` to `(source, hash, metadata)`, but it is not a trust root.
5. Signing and provenance attest metadata and publication history; they do not
   replace content-hash verification.

Every remote fetch path must end at the same verifier used for local `.mgp`
archives. If fetched bytes do not match the expected hash, the build must fail
before materialization, checking, or execution.

## Threat Model

The future remote package workflow must explicitly handle:

- registry metadata compromise or rollback;
- mirror or CDN serving different bytes for an existing version;
- Git tags moved after resolution;
- first-install trust-on-first-use risk when no out-of-band hash is supplied;
- cache poisoning, stale cache entries, or package/cache path collisions;
- lockfile edits, malformed lockfiles, unsupported versions, or missing hashes;
- dependency confusion, typosquatting, name squatting, and package takeover;
- malicious or abandoned packages after they verify correctly;
- yanked packages and advisories without breaking already locked builds by
  surprise;
- malformed archives, path escapes, unsafe manifest source/resource roots, and
  partial materialization.

Hash collisions are not treated as a practical operational threat for SHA-256
today, but the stored spelling must stay algorithm-qualified (`sha256:<hex>`)
so future algorithms can be introduced deliberately.

## Required Remote Workflows

### First Resolution

When URL, Git, or registry dependency forms are added, resolution should:

1. resolve a manifest requirement to concrete bytes and metadata;
2. canonicalize or read the package as `.mgp` bytes;
3. compute the package content hash;
4. compare that hash with any manifest-provided expected hash;
5. write `muga.lock` with source metadata, resolved version or commit, package
   path, dependency edges, and the content hash;
6. require the user to review `muga.toml` and `muga.lock` changes.

If the user supplies an expected hash, the first resolution is pinned. If no
hash is supplied, the first resolution is trust-on-first-use and must be made
visible in documentation and diagnostics.

### Locked Build

When `muga.lock` exists for remote dependencies, builds should:

1. avoid resolver lookups unless the manifest changed or the user explicitly
   requests an update;
2. fetch from any allowed source or mirror only to obtain bytes;
3. verify bytes against the lockfile hash before cache reuse or
   materialization;
4. reuse only cache entries whose recomputed content hash matches the lockfile;
5. fail closed on missing hashes, mismatches, malformed metadata, unsupported
   lockfile versions, or graph-inconsistent dependencies.

A registry outage must not break a project whose lockfile and cache already
contain verified bytes.

### Publishing

Publishing should be immutable at the `(name, version, hash)` level:

1. produce a canonical `.mgp` archive;
2. compute and display the content hash;
3. optionally attach publisher signatures and provenance attestations;
4. publish metadata that records the source location and content hash;
5. reject attempts to replace an existing `(name, version)` with different
   bytes unless the registry policy explicitly creates a new version or marks
   the old version yanked.

Registry metadata may be signed, mirrored, or stored in a transparency log, but
build correctness still depends on verifying archive bytes against the content
hash.

## Signing And Provenance

Future signing should be additive:

- publisher signatures bind package name, version, content hash, and source
  archive metadata;
- registry signatures bind index snapshots, yanks, advisories, and ownership
  metadata;
- provenance attestations describe build source, CI identity, and release
  workflow inputs when available;
- key rotation, revocation, and ownership transfer must be represented as
  metadata changes rather than silent package replacement.

Signature failure, missing required provenance, or revoked ownership should be a
policy error before dependency materialization. Optional signatures should be
reported clearly without changing the hash verification result.

## Cache Validation

Remote caches should follow the same fail-closed rules as local archive
dependencies:

- cache paths are content-addressed by package path plus algorithm/hash;
- cache reuse requires recomputing the content hash over cached sources or
  preserved archive bytes;
- rejected cache entries must not be silently repaired by reading dependency
  source bodies from elsewhere;
- failed materialization must not leave partial package trees that later count
  as valid;
- `muga why-rebuild --format json` should remain the inspection path for cache
  state before broader audit tooling exists.

## Malicious-Package Handling

Hash verification proves which bytes were selected; it does not prove that the
package is safe. Before remote fetching, Muga needs a policy for:

- yanked versions and whether locked builds keep working;
- advisory metadata and a future `muga audit` command;
- package ownership transfer and abandoned-package warnings;
- dependency confusion and reserved names or organization scopes;
- install-time behavior that never runs package code, build scripts, or
  artifact bytecode merely because a dependency was fetched;
- clear diagnostics when a package is blocked by policy despite matching its
  hash.

The v1 local package workflow has no arbitrary build scripts. That property
should be preserved when remote registries are introduced.

## Non-Goals For The Current Repository

This design does not add:

- URL, Git, or registry dependency declarations;
- network fetching or remote mirrors;
- package publishing or installer workflows;
- registry signing, provenance verification, advisory databases, or
  `muga audit`;
- full published-package lockfile enforcement;
- binary distribution, SBOM generation, or package health scoring.

The local `.mgp` verifier, local archive dependency cache, and lockfile
metadata are the implemented foundation. Remote functionality should be built
only after these trust rules are testable.

## Promotion Rules

Before adding any remote dependency implementation:

1. update `spec/006-packages.md` with the exact manifest and lockfile syntax;
2. add deterministic tests for hash mismatch, stale cache, malformed lockfile,
   yanked/advisory policy, and registry metadata tampering;
3. keep all network tests out of the default release gate unless they are fully
   hermetic;
4. preserve offline builds from a populated lockfile and validated cache;
5. keep source imports as logical package paths only;
6. document first-install trust-on-first-use behavior clearly.
