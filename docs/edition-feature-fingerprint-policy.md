# Edition And Feature Fingerprint Policy

Status: future compatibility policy. This document defines how future language
editions and semantic feature sets should participate in package artifacts,
cache keys, lockfiles, diagnostics, and API compatibility checks before Muga
adds syntax or semantic changes that need backward-compatible migration.

This is not an implemented edition selector, manifest field, artifact format
change, `muga fix` command, migration guide, release trigger, or reason to add
named arguments, broader `using` forms, range syntax, interpolation, `T?`, or
`?.`.

## Current Foundation

Muga currently has one implicit v1 language semantics. The implemented package
and artifact surfaces already have stable places where edition or feature-set
data must eventually participate:

- `.mgi` package interface artifacts use the `muga-package-interface-v11`
  schema header and stable public interface hashes.
- `.mgc` package check-cache artifacts use the `muga-package-check-v1` schema
  header, a `PackageCheckCacheKey`, the entry `source_hash`, and dependency
  interface hashes.
- `.mgb` implementation artifacts use the
  `muga-package-implementation-bytecode-v1` schema header and a
  `PackageImplementationArtifact` with `interface_hash`, `source_hash`, and
  dependency interface hashes.
- local `muga.lock` metadata records `lockfile_version = 1`, `muga_version`,
  local path `source_hash` values, and local archive `hash` values.
- `source_fingerprint_input_from_entry` is the current source input for
  package check-cache and implementation hashes.
- `.mgp` archive hashes identify canonical package bytes and should remain
  content identity, not compiler semantic identity.
- `muga why-rebuild --format json` and artifact diagnostics already expose
  artifact hashes, lockfile/cache states, and `regenerationCommand` context.

These are format and cache foundations, not an edition implementation. Future
work should add edition and feature-set data deliberately instead of relying on
compiler version strings alone.

## Terms

- **Language edition**: a coarse package-level selector for source meaning.
  Editions preserve old source behavior while allowing future syntax or
  semantic rules to evolve.
- **Semantic feature set**: a sorted, explicit set of narrower semantic toggles
  when an edition alone is too coarse. Feature flags are not experimental
  macros; they are compatibility inputs.
- **Compiler semantic version**: the compiler's semantic interpretation level,
  distinct from backend-only changes, release packaging, or diagnostics-only
  wording changes.
- **Prelude/std semantic version**: the public interface and compiler-provided
  semantics of built-in packages such as `std::io`, `std::fs`, `std::path`,
  `std::env`, `std::cli`, `std::time`, `std::option`, `std::result`,
  `std::list`, `std::map`, and `std::json`.
- **Artifact schema version**: the persisted format header such as
  `muga-package-interface-v11`, `muga-package-check-v1`, or
  `muga-package-implementation-bytecode-v1`. Schema versions describe bytes on
  disk; editions describe source meaning.
- **Public API hash**: the stable hash of exported package names, item kinds,
  public type shapes, enum variants, representation transparency, and stable
  dependency-owned public identities that appear in exported signatures.
- **Recheck fingerprint**: the semantic cache key that decides whether checked
  package bodies or implementation artifacts can be reused.

## Policy

### Make Source Meaning Explicit Before Changing It

Before adding an incompatible syntax or semantic rule, Muga should have a
package-scoped edition or semantic feature-set selector. The first selector
should preserve current v1 source meaning for existing packages. Missing
edition metadata should continue to mean the current legacy default until a
separate migration plan changes that rule.

Generated projects may start writing an explicit edition only after the
compiler, artifacts, docs, and tests agree on the spelling. Existing projects
must not silently change behavior because the compiler was upgraded.

### Keep Edition Metadata Package-Scoped

Edition and feature-set metadata belong to the package manifest and package
artifacts, not to individual import statements or command-line defaults.
Command-line flags may be useful for debugging, but reproducible package
behavior must be recoverable from source-controlled package metadata and the
lockfile.

When manifest syntax is implemented, the spelling should be explicit and
reviewable. A future shape could be package-level metadata such as:

```toml
[package]
edition = "v1"
semantic_features = []
```

This document does not add that syntax.

### Separate Content Identity From Semantic Identity

`.mgp` archive `sha256:<hex>` values remain canonical package byte identity.
They should not change just because a compiler adds a new edition. The package
archive proves which bytes were selected; the edition and feature-set
fingerprint proves how those bytes are interpreted.

`muga.lock` should eventually record the edition and semantic feature set used
for each resolved package when remote or multi-edition workflows exist. A
locked build should fail closed if a dependency's lockfile metadata, manifest,
or artifact fingerprint disagrees with the selected package bytes.

### Split Public API Hashes From Recheck Fingerprints

Future artifact keys should use two semantic identities:

- `public_api_hash` for exported names, public type shapes, enum variants,
  representation transparency, and dependency-owned public identities visible
  in exported signatures.
- `recheck_fingerprint` for the package body checking context:
  `public_api_hash`, direct dependency public API hashes visible to bodies,
  source fingerprint, language edition, semantic feature set, prelude/std
  semantic version, compiler semantic version, and relevant artifact schema
  versions.

Implementation-only edits should not change `public_api_hash`. Source edits,
edition changes, feature-set changes, prelude/std semantic changes, and
compiler semantic changes that can affect typechecking or lowering must change
the relevant recheck fingerprint.

Backend-only target or optimization settings belong in backend artifact keys,
not in `.mgi` public API hashes or package check-cache keys.

### Update Artifact Boundaries Deliberately

When the edition selector is implemented, every persisted semantic artifact
must either store or derive the same edition and semantic feature-set data:

- `.mgi` should include edition and feature-set data when they affect public
  type meaning, exported identities, enum exhaustiveness, representation
  transparency, or source compatibility classification.
- `.mgc` should include edition and feature-set data in the package check-cache
  key so stale checked bodies cannot be reused across semantic modes.
- `.mgb` should include edition and feature-set data in the implementation
  artifact body or dependency metadata so bytecode is not reused across
  incompatible lowering rules.
- `muga why-rebuild --format json` should expose fingerprint mismatch reasons
  with the same artifact hash and `regenerationCommand` context used by
  existing package/artifact diagnostics.

Artifact schema headers should still change when the on-disk format changes.
Edition data is not a substitute for schema versioning.

### API Diff And Migration Rules

`.mgi` API diffing should classify a changed edition or semantic feature-set
fingerprint as `unknown` until compatibility rules for that edition pair are
documented. Unknown changes should fail automated release guidance closed while
printing a precise reason.

Edition migration tooling such as `muga fix` is post-v1. A future migration
command should make source edits explicit, should not rewrite package
artifacts as proof of compatibility, and should not bypass normal `muga check`,
`muga test`, `muga build`, or API-diff validation.

Cross-edition imports may be allowed only when public interface compatibility
rules are explicit. If an importer cannot prove that a dependency's public API
hash is meaningful under the importer's edition and feature set, the importer
should fail closed with a diagnostic rather than reinterpreting dependency
source bodies.

### Diagnostics

Diagnostics should be actionable and should name the mismatched dimension:

- unsupported edition;
- unsupported semantic feature;
- artifact compiled for a different edition;
- artifact compiled for a different semantic feature set;
- package check cache fingerprint mismatch;
- implementation artifact fingerprint mismatch;
- public interface edition compatibility is unknown.

Each artifact diagnostic should keep concrete artifact-file context, expected
and actual hash or fingerprint context, and a `regenerationCommand` such as
`muga build <entry>`, `muga emit-artifacts --artifact-root <dir> <entry>`,
`muga emit-interface --artifact-root <dir> <entry>`, or
`muga emit-check-cache --artifact-root <dir> <entry>`.

## Non-Goals For The Current Repository

This design does not add:

- an `edition` or `semantic_features` manifest field;
- new `.mgi`, `.mgc`, `.mgb`, `.mgp`, or `muga.lock` bytes;
- cross-edition typechecking;
- edition migration tooling or `muga fix`;
- new syntax or semantic behavior;
- a release timing decision.

## Promotion Rules

Before implementing any incompatible syntax or semantic change:

1. add an explicit package-scoped edition or semantic feature-set selector;
2. update `.mgi`, `.mgc`, `.mgb`, and `muga.lock` tests for fingerprint
   persistence and mismatch diagnostics;
3. keep `.mgp` content hashes independent from semantic interpretation;
4. teach `muga why-rebuild --format json` to explain edition or feature-set
   fingerprint mismatches;
5. update `.mgi` API-diff compatibility rules for the edition pair or fail
   closed as `unknown`;
6. document user migration steps before enabling the syntax or semantic change
   by default.
