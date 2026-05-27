# MGI API Diff Design

Status: v1 maintenance implementation note. The library API
`muga::api_diff::diff_package_interfaces` now compares two loaded
`PackageInterfaceGraph` values and classifies compatible, source-compatible,
breaking, and unknown changes. The `muga api-diff` CLI wraps the same contract
for persisted artifact roots.

## Purpose

`.mgi` files are the serialized public contract for Muga packages. An API diff
compares two package interface sets and classifies public API changes without
reading private dependency source bodies, `.mgb` implementation bytecode, or
`.mgc` check-cache proofs.

The implemented library comparator and CLI answer two questions:

- did the public source contract change?
- if it changed, is the change compatible, source-compatible but reviewable,
  breaking, or unknown?

## Inputs

The diff input is two loaded `PackageInterfaceGraph` values, usually read from
old and new artifact roots. The current library API is:

```rust
muga::api_diff::diff_package_interfaces(old, new, "app::api", symbols)
```

The CLI compares one package from two persisted artifact roots:

```bash
muga api-diff --old-artifact-root old-build --new-artifact-root new-build --package app::api --format json
```

The diff uses only data already present in `.mgi`:

- package path and dependency package paths
- public records, enums, `pub opaque type` names, and functions
- public type parameter names and arity
- public record field names and field types
- public enum variant names and optional payload types
- public opaque `handleFacts`
- public function parameter names, parameter types, `paramMode`, and return type
- resolved package item references embedded in public type expressions
- the stable public interface hash as a fast equality check

The diff must ignore diagnostic-only source spans. It must not use private
function bodies, package-local implementation source hashes, source file paths,
human diagnostic wording, `.mgb`, or `.mgc`.

## Identity

External item identity is the tuple of package path, item kind, and public item
name. Persisted package and item IDs remain useful for artifact validation and
loading, but API diff output should report names because `.mgi` loading remaps
artifact IDs into fresh session-local IDs.

Type comparison is structural:

- builtin types compare by builtin name
- type parameters compare by position within the enclosing item, with matching
  names reported as a metadata check
- package item types compare by referenced package path, item kind, and public
  item name
- generic type applications compare by base type plus ordered type arguments

The implementation sorts changes by severity, package path, item kind, item
name, and member name so JSON and human output remain deterministic.

## Classifications

### Compatible

A change is compatible when the old and new `.mgi` public interface hashes are
equal for the package being compared. Examples include implementation-only body
changes, private helper changes, source-span movement, `.mgb` bytecode changes,
and `.mgc` cache regeneration.

### Source-Compatible

A change is source-compatible when existing downstream source should keep
checking but the public `.mgi` shape changed and maintainers should see it in a
diff report. Initial examples:

- adding a new public function, record, enum, or opaque type name
- reordering public record fields without changing their names or types
- reordering public enum variants without changing their names or payloads
- renaming a public function parameter while preserving positional parameter
  count, parameter types, type parameters, and return type
- changing type parameter names while preserving arity and use positions
- adding `copyable`, `cloneable`, `sendable`, or `shareable` capability facts
  to a runtime-backed opaque handle
- changing a function parameter from `consume` to `borrow`

Muga currently has positional calls and named record fields, so parameter-name
changes are not source-breaking. They still change public metadata and may
matter for docs or generated tools.

### Breaking

A change is breaking when existing downstream source may fail to check, change
meaning, or lose an item it names. Initial examples:

- removing or renaming a public package path
- removing or renaming a public record, enum, opaque type, or function
- changing the kind of a public item while keeping the same name
- changing public type parameter arity
- changing a public opaque type into a record, enum, function, or alias
- removing an opaque handle capability, changing `closeable`, or changing the
  named close function except for an identical public identity/signature
- changing a public function parameter count, parameter type, return type, or
  type parameter use
- changing a function parameter from `borrow` to `consume`
- changing a public record field type
- adding, removing, or renaming a required public record field
- adding, removing, or renaming a public enum variant
- changing an enum variant payload from absent to present, present to absent,
  or one public type to another
- changing the public type identity of a referenced package item

Adding an enum variant is breaking because downstream exhaustive matches can
become incomplete. Adding a required record field is breaking because
downstream record literals may become incomplete.

### Unknown

A change is unknown when the diff cannot make a sound classification. Initial
examples:

- either `.mgi` file uses an unsupported persisted schema version
- either interface graph fails validation
- the package path being compared exists only as a dependency context and not
  as the target package
- duplicate public names or identity collisions are detected
- a future `.mgi` feature appears without a documented compatibility rule
- deprecation metadata appears before the deprecation policy is finalized
- opaque type capability or consuming-parameter metadata appears outside the
  compatibility rules in [opaque-resource-handles.md](opaque-resource-handles.md)
- an edition or semantic feature-set fingerprint differs before edition
  compatibility rules in
  [edition-feature-fingerprint-policy.md](edition-feature-fingerprint-policy.md)
  exist

Unknown changes should make an automated release check fail closed while still
printing a precise reason.

## Deprecations

Deprecation is not represented in the current `.mgi` format. When static
metadata is added later, deprecating a public item should be classified as
source-compatible metadata, removing deprecated metadata should be
source-compatible metadata, and removing the deprecated item itself should
remain breaking.

## Output Contract

The CLI supports a tab-separated human summary and schema-versioned JSON. The
JSON is stable enough for CI, editors, release bots, and coding agents. The
library returns `PackageApiDiff`, `PackageApiDiffSummary`, and
`PackageApiDiffChange`; CLI rendering preserves those fields.

JSON shape:

```json
{
  "schemaVersion": 1,
  "command": "api-diff",
  "status": "breaking",
  "package": "app::api",
  "summary": {
    "compatible": 0,
    "sourceCompatible": 2,
    "breaking": 1,
    "unknown": 0
  },
  "changes": [
    {
      "classification": "breaking",
      "kind": "function-signature-changed",
      "path": "app::api::parse",
      "message": "public function return type changed from Result[Token, ParseError] to Token"
    }
  ]
}
```

The overall status is the highest-severity classification present, ordered as
`unknown`, `breaking`, `sourceCompatible`, then `compatible`. The default
command exits successfully for classified diffs, including `breaking` and
`unknown`, so review tools can decide policy from the reported status. Loading
or validation diagnostics exit non-zero. `--fail-on breaking` makes release
gates fail closed for non-empty `unknown` or `breaking` results. The
`--fail-on source-compatible` threshold also fails on reviewable
source-compatible changes.

## Implementation Order

1. Done: add a library diff over two already loaded `PackageInterfaceGraph`
   values.
2. Done: add deterministic fixture tests for compatible, source-compatible,
   breaking, and unknown changes in `tests/api_diff.rs`.
3. Done: add a CLI wrapper with `--format text|json` after the library output is
   stable.
4. Done: add release-gate integration; persisted fixtures now cover current
   `.mgi` records, enums, functions, generic signatures, public type
   references, and implementation-only edits.

The implementation should keep using `.mgi` as the typed public contract. It
should not read source bodies to decide compatibility.
