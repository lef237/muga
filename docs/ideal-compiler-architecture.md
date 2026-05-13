# Ideal Compiler Architecture

Status: design note. This describes the structure Muga should have if the project were designed from scratch with the current language direction already known.

## Goal

The cleanest Muga architecture is a package-aware compiler pipeline with one semantic path:

```text
source files
  -> syntax
  -> package graph
  -> name resolution
  -> type checking
  -> typed HIR
  -> package interfaces
  -> MIR
  -> VM bytecode or native backend
```

The VM remains a reference execution backend, but it should not define a separate semantic path. `check`, `run`, and future `build` should share parsing, name resolution, type checking, typed HIR, package interfaces, and MIR.

## Top-Level Workspace

From scratch, use a Rust workspace with narrow crates:

```text
crates/
  muga_cli/
  muga_driver/
  muga_span/
  muga_diagnostic/
  muga_symbol/
  muga_ids/
  muga_syntax/
  muga_package/
  muga_hir/
  muga_sema/
  muga_interface/
  muga_mir/
  muga_vm/
  muga_codegen/
  muga_test_support/
```

For the current repository size, these can begin as modules inside one crate. The important rule is ownership, not crate count: each stage owns one data model and exposes explicit outputs to the next stage.

Two splits are worth preserving even before a workspace split:

- `span` / source-map data should be independent from diagnostic rendering.
- `symbol` / interning should be independent from semantic identity. A `Symbol` is interned spelling, not a resolved language entity.

## Crate Responsibilities

### `muga_cli`

Owns only command-line behavior:

- argument parsing
- selecting `check`, `run`, or `build`
- printing diagnostics and runtime output
- process exit codes

It should not know package loading internals, type inference, HIR, MIR, or backend details.

### `muga_driver`

Owns orchestration:

- builds a compilation session
- invokes package loading
- runs each compiler stage in order
- decides which artifacts are needed for `check`, `run`, or `build`
- exposes stable public API functions for tests and external callers

The driver is the only layer that should know the full pipeline.

### `muga_diagnostic`

Owns diagnostic data and rendering over source locations:

- `Diagnostic`
- primary spans
- related notes
- suggestions and replacements
- stable file/source references by `SourceId`
- rendering policy

Diagnostics should carry source IDs, not just line/column spans, once multi-file packages and cached interfaces are first-class. Source maps and line tables belong to `muga_span`; diagnostics consume them through an interface.

### `muga_span`

Owns source identity and source locations:

- `SourceId`
- `Span`
- `Position`
- line tables
- source file names and virtual source names

This crate should not know diagnostic severity, suggestions, package identity, or compiler stages.

### `muga_symbol`

Owns string interning:

- `Symbol`
- `SymbolInterner`
- lookup from symbol to source text

It should not attach language meaning to names. `Symbol` is only interned spelling.

### `muga_ids`

Owns typed identity wrappers and arenas:

- `PackageId`
- `ModuleId`
- `ItemId`
- `VariantId`
- `BodyId`
- `StmtId`
- `ExprId`
- `BindingId`
- `LocalId`
- `TypeVarId`
- `TypeId`
- `DefId`

IDs must be explicit about their scope. A local binding ID must not be confused with a package item ID, and an item ID must not be recovered from a mangled name.

Use two identity layers:

- session IDs: compact IDs valid only inside one compiler session
- artifact IDs: stable serialized identities for interfaces, caches, lockfiles, and package artifacts

Session IDs:

```text
PackageId
ModuleId
ItemId
VariantId
BindingId
ExprId
StmtId
TypeId
```

Artifact IDs:

```text
PackageVersionId     = sha256(canonical package archive)
LocalPackageSourceId = sha256(canonical local source set + relevant manifest fields)
PackageInstanceId    = PackageVersionId | LocalPackageSourceId
PublicItemKey        = { package_instance, namespace, kind, export_path }
PrivateItemKey       = { package_instance, module_path, namespace, kind, source_name }
VariantKey           = { enum_item_key, variant_name }
```

Artifact IDs are mapped into session IDs when an interface is loaded. Persisted data must not contain session-local `PackageItemId`, `BindingId`, or `Symbol`.

`package_path` alone is not artifact identity. Caches and lockfiles must distinguish two versions or two local source instances of the same logical package path.

Current-to-target identity mapping:

```text
Current PackageItemId  -> target session ItemId, never serialized
Current ModuleId       -> target session ModuleId
Current PackageId      -> target session PackageId
Current BindingId      -> target session BindingId
Target ItemRef         -> serialized public/private item artifact identity
Target VariantRef      -> serialized enum variant artifact identity
```

Glossary:

- `ItemId`: session-local identity for a package item during one compile
- `ItemRef`: serialized artifact identity for a public or private item
- `PackageItemId`: current implementation name for what should become session-local `ItemId`
- `PublicItemKey`: stable key for an exported item in an interface
- `PrivateItemKey`: stable key for a private item in implementation/cache metadata
- `VariantId`: session-local identity for an enum variant
- `VariantRef`: serialized artifact identity for an enum variant

### `muga_syntax`

Owns source syntax only:

- lexer
- tokens
- parser
- parsed AST
- node ID assignment
- source text spans
- parse diagnostics

The syntax AST should preserve source shape but should not know package item identity, binding identity, or inferred types.

Recommended split:

```text
syntax/
  token.rs
  lexer.rs
  ast.rs
  parser.rs
  lower.rs
```

If syntax grows, introduce a lightweight CST only for formatting or precise parser recovery. The compiler pipeline can continue from AST.

### `muga_package`

Owns package discovery and source graph construction:

- manifest discovery
- source roots
- package paths
- file-to-module mapping
- import graph
- package cycles
- visibility collection
- dependency source/interface lookup

It must not flatten packages into one program. Its output should be a `PackageGraph` whose nodes point to parsed modules and imported interface artifacts.

Core outputs:

```text
PackageGraph
  packages: PackageId -> Package
  modules: ModuleId -> ParsedModule
  items: ItemId -> DeclHeader
  imports: ModuleId -> ImportEdge
```

### `muga_hir`

Owns compiler-owned program shapes after parsing:

- resolved HIR
- typed HIR
- item bodies
- pattern representation
- stable body/item ownership

Use two explicit layers:

```text
Parsed AST      source-shaped, strings, no resolved names
Resolved HIR    names resolved to BindingId / ItemId
Typed HIR       resolved HIR plus TypeId / TypeInfo on expressions
```

Do not keep a separate untyped HIR for the VM. If an execution backend needs a simpler representation, it should consume MIR.

### `muga_sema`

Owns semantic analysis:

- prelude installation
- namespace model
- name resolution
- type representation
- local inference
- generic declarations
- enum/variant checking
- exhaustiveness checking
- public signature inference

Resolution and typing should be separated internally, but they should share one semantic database and one item/body model.

The resolver must allocate binding and item targets once. The typechecker consumes resolved HIR and inference variables; it must not rebuild lexical scopes or reinstall a separate prelude from strings.

Public signature inference belongs here, but with a strict boundary: infer only from the defining package's annotations and local body constraints. Never infer a public signature from downstream call sites. Until persisted interfaces exist, requiring explicit `pub fn` signatures remains the safer implementation rule.

If public signatures are inferred after persisted interfaces exist, the interface builder must record the dependency public APIs used during that inference. A dependency function call that helps determine an exported return type is an interface input even if the final exported type does not directly mention that dependency item.

Recommended submodules:

```text
sema/
  prelude.rs
  namespaces.rs
  resolve.rs
  types.rs
  infer.rs
  records.rs
  enums.rs
  generics.rs
  match_check.rs
  signatures.rs
```

### `muga_interface`

Owns public package artifacts:

- interface data model
- interface serialization
- interface hashing
- dependency interface loading
- stale interface diagnostics
- export lookup

Interfaces should describe public declarations, not compiler implementation details:

```text
PackageInterface
  package path
  package instance/hash
  public records
  public enums and variants
  public functions
  type parameters
  resolved public signatures
  dependency fingerprints for rechecking
```

Downstream packages typecheck against interfaces. They should not inspect dependency bodies unless rebuilding those dependencies.

Do not persist the in-memory typed HIR or current session type structs directly. Define a versioned interface tree:

```text
InterfaceTypeRef
  Int
  Bool
  String
  Nominal(ItemRef, [InterfaceTypeRef])
  Function([InterfaceTypeRef], InterfaceTypeRef)
  TypeParam(name)
```

`ItemRef` is stable artifact identity, not a session-local item ID. A nominal item can be a public record, public enum, opaque runtime type, or standard prelude type such as `List` and `Map`.

Enum variants also need stable artifact identity:

```text
VariantRef = { enum: ItemRef, variant: VariantKey }
```

Downstream exhaustiveness checking, MIR lowering, and ABI/cache invalidation should use `VariantRef`, not source spelling or variant list position alone.

The first implementation can use deterministic hand-written text or JSON to avoid adding dependencies. The semantic model should not depend on that encoding.

### `muga_mir`

Owns execution-shaped IR:

- explicit locals
- explicit control flow
- evaluation order
- temporaries
- desugared pattern matching
- calls to known/runtime intrinsics
- ownership/copy-elision metadata when available

MIR is the shared backend input. The VM and native backend should both consume MIR or a simple lowering from MIR.

### `muga_vm`

Owns reference execution:

- MIR-to-bytecode lowering, or direct MIR interpretation if that proves simpler
- bytecode representation
- runtime values
- builtin runtime functions
- runtime diagnostics

The VM should not resolve names, infer types, or decide language semantics.

### `muga_codegen`

Owns native backend work:

- backend-independent codegen traits
- Cranelift lowering first
- target configuration
- object/executable emission

This crate should start after typed HIR, package interfaces, and MIR are stable.

## Semantic Data Model

The ideal compiler has a central semantic database per compilation session:

```text
Session
  sources
  symbols
  package_graph
  item_table
  body_table
  type_table
  diagnostics
```

Each stage appends structured facts:

- parser creates `SourceId`, AST nodes, `StmtId`, and `ExprId`
- package loader creates `PackageId`, `ModuleId`, and item headers
- resolver creates `BindingId`, resolved identifier facts, and resolved item paths
- typechecker creates expression types, inferred signatures, and checked item bodies
- interface builder creates public package artifacts
- MIR lowering creates local slots, blocks, and backend-ready control flow

Facts should be keyed by IDs. Later stages should not rediscover facts by string lookup.

The semantic database is also the prelude owner. Builtins should be declared once as prelude records/functions/enums with signatures and lowering hooks. Resolver, typechecker, MIR lowering, bytecode, and runtime should consume that shared metadata rather than duplicating builtin name lists.

Language-visible builtins resolve as normal prelude items. A builtin lowering/runtime hook is metadata attached to that item, not a separate name-resolution target.

## Namespaces

Use explicit namespaces from the beginning:

- value namespace: locals, parameters, functions, builtins, enum constructors
- type namespace: records, enums, type parameters, compiler-known types
- package namespace: import aliases
- variant namespace: variants under an enum item

Qualified names should resolve to structured targets:

```text
ResolvedPath
  Local(BindingId)
  Item(ItemId)
  Variant(VariantId)
  PackageItem { package: PackageId, item: ItemId }
```

If an item is backed by runtime/compiler behavior, the item table may carry `BuiltinId` or an equivalent lowering hook. Name resolution should still point at the item.

Do not encode package qualification as a mangled string. Mangled names are backend labels only.

## Types

Use one canonical, interned type representation inside the compiler:

```text
Type
  Int
  Bool
  String
  Nominal(ItemId, Vec<Type>)
  Function(Vec<Type>, Type)
  TypeParam(TypeParamId)
  Error
```

`Nominal` covers records, enums, and standard library / prelude collection types such as `List[T]`, `Map[K, V]`, `Option[T]`, and `Result[T, E]`. The item table records whether the nominal item is a record, enum, opaque runtime type, or compiler intrinsic type.

`Option[T]` and `Result[T, E]` should be standard enum items in the semantic model, even if they are compiler-known during bootstrapping.

Avoid permanent special cases such as separate `TypeInfo::Option` and `TypeInfo::Result` once user-defined enums exist. Compiler-known metadata can seed the prelude; semantic analysis should then treat them like ordinary enum declarations.

Typed HIR and MIR can refer to canonical `TypeId`. Package interfaces should use `InterfaceTypeRef`, then map those references back to `TypeId` when loaded.

## Package Interfaces

Build interfaces immediately after type checking a package:

```text
Package source
  -> parsed modules
  -> headers
  -> resolved bodies
  -> typed HIR
  -> interface
```

An interface contains enough to typecheck downstream packages:

- public item identities
- public names and visibility
- public type parameters
- record fields for transparent public records
- enum variants and payload types
- function parameter and return types
- stable identities of public dependency items referenced by exported signatures

It must not contain private bodies, inferred local variables, MIR, bytecode, or backend labels.

Suggested first interface fields:

```text
schema_version
compiler_semantics_version
package_path
package_instance
package_source_fingerprint
direct_dependencies:
  - package_path
  - package_hash
  - public_api_hash
  - recheck_fingerprint
exports:
  records:
    - item_key
    - name
    - transparency
    - fields
  functions:
    - item_key
    - name
    - params
    - return_type
  enums:
    - item_key
    - name
    - type_params
    - variants:
      - variant_key
      - name
      - payload
public_api_hash
recheck_fingerprint
```

Use two hashes:

- `public_api_hash`: exported names, item kind, resolved public type shapes, enum variants, representation transparency, and stable identities of dependency-owned public items that appear in exported signatures
- `recheck_fingerprint`: public API hash plus direct dependency public API hashes visible to package bodies, language edition, enabled semantic features, prelude/std version, and compiler semantic version

The public API hash answers "does a downstream public type view change?" The recheck fingerprint answers "can this package's previously checked bodies still be trusted against the dependency interfaces it used?"

Both hashes should exclude spans, docs, local import aliases, file order, and backend mangled names. Source references for diagnostics can live in a sidecar artifact.

Inferred public signatures must be closed before they can enter an interface:

- no unresolved inference variables
- no session-local `TypeId`, `TypeVarId`, `BindingId`, or `PackageItemId`
- no references to private items that would leak across the package boundary
- no source-local import aliases as identity
- all type parameters must be declared on the public item or enclosing public type

If a public signature cannot be closed into `InterfaceTypeRef`, the compiler should reject the public item or require explicit annotations that expose a legal public contract.

## Cache Model

Cache by package, with separate keys for semantic products and backend products:

```text
interface_key =
  package_source_fingerprint
  + direct dependency public_api_hashes used by exported signatures
  + public_signature_inference_dependency_fingerprint
  + language edition / semantic feature set
  + prelude/std interface hash
  + compiler_semantics_version

typed_hir_key =
  package_source_fingerprint
  + direct dependency public_api_hashes visible to package bodies
  + language edition / semantic feature set
  + prelude/std interface hash
  + compiler_semantics_version

mir_key =
  typed_hir_key
  + mir_version

backend_key =
  mir_key
  + target
  + backend version
  + optimization flags
```

Invalidation rule:

- if a package source fingerprint changes but its public API hash is unchanged, downstream packages do not need typechecking
- if a direct dependency public API hash changes, importers that reference it must be re-resolved and retyped
- if only a dependency implementation changes and its public API hash is unchanged, importers do not need retyping
- if a transitive dependency changes but the direct dependency public API hashes visible to an importer are unchanged, the importer needs at most relink/rebuild, not retypecheck
- if only backend config changes, reuse typed/interface artifacts and rebuild backend artifacts
- if compiler semantic or interface schema version changes, invalidate affected artifacts
- if a lockfile package hash does not match fetched content, stop with a hard error

The package source fingerprint must include all inputs that can change semantic output: source bytes, relevant manifest fields, dependency resolution, language edition, enabled semantic features, prelude/std version, and compiler semantic flags. Backend-only configuration belongs only in backend keys.

`public_signature_inference_dependency_fingerprint` is empty when public signatures are fully explicit. If a public signature is inferred from calls or type facts involving dependencies, it records the public API hashes of the dependency items used by that inference so stale inferred interfaces are not reused.

## Pipeline Modes

`check`:

```text
discover entry package and dependency graph
load cached dependency interfaces unless a dependency is selected for rebuild
scan package headers and reject cycles
topologically, for each rebuilt package:
  parse modules
  resolve against local headers plus dependency interfaces
  typecheck
  close public signatures
  build/validate interface
```

`run`:

```text
check pipeline
lower typed HIR to MIR
lower MIR to VM bytecode
execute VM
```

`build`:

```text
check pipeline
lower typed HIR to MIR
lower MIR to native backend
emit executable/library
```

## Refactoring Target For The Current Codebase

The current codebase should move toward the ideal in this order:

1. Add user-defined enum identity to the semantic model.
2. Complete stable `ItemRef` / `InterfaceTypeRef` for interface design. The current `.mgi` v2 text no longer writes session-local package/item IDs, but it still maps through the existing `TypeInfo` model instead of a first-class interface type model.
3. Introduce stable `VariantRef` and include variants in the interface model.
4. Define public API hash vs recheck fingerprint before writing persisted interfaces.
5. Replace compiler-known `Option` / `Result` special cases with prelude enum metadata where possible.
6. Continue replacing the remaining compatibility uses of flattened AST/HIR now that default package `check_path`, `compile_typed_path`, and loaded-interface typed compilation use the package-aware graph.
7. Keep interface generation on package-aware typed HIR and move the serialized interface model toward first-class `InterfaceTypeRef` / `ItemRef` data.
8. Make package import lookup consume interfaces as its primary input.
9. Mature MIR from the current expression-shaped backend IR into a control-flow-oriented IR, then lower VM bytecode from that form.
10. Remove the old untyped HIR compatibility API once external callers no longer need it.
11. Split large files by responsibility before splitting into workspace crates.

Completed structural steps:

- package interface data now lives in `interface`, not `typed_hir`
- shared public type summaries now live in `types`, not only inside `typing`
- resolver, typechecker output, runtime, and package builtin lookup share `prelude::BuiltinId`
- typed HIR lowering reads package item identity from AST declarations instead of recovering it from mangled names
- loaded-interface typed compilation no longer synthesizes dependency interface AST stubs or routes them through the legacy flattened typed path
- VM bytecode now consumes `mir::Program`
- default compile APIs lower typed HIR into MIR; the legacy untyped AST-to-HIR compatibility module has been removed

Critical current risks to eliminate:

- resolver and typechecker still build scopes independently
- MIR is still expression-shaped, though it now has explicit execution bodies, body terminators, hoisted body-local function definitions, typed binding/package-item identity, typed assignment update mode, and binding-keyed runtime names with package function references canonicalized to their defining binding, and is not yet a control-flow-oriented backend IR
- package interfaces still use session-local IDs and compiler-owned type structs in memory, even though `.mgi` v2 maps stable artifact identities back into fresh session IDs when loaded
- builtin type rules and runtime behavior are still implemented separately
- bytecode match lowering still assumes compiler-known enum shapes

## Design Rules

- One semantic source of truth: typed HIR and interface facts, not duplicated resolver/typechecker/backend logic.
- IDs cross stage boundaries; strings remain source labels and diagnostic text.
- Package flattening is never a semantic boundary.
- Interfaces are the only dependency boundary for downstream checking.
- MIR is the only backend boundary.
- VM runtime behavior must not bypass checked semantics.
- Builtins should be prelude items with explicit metadata, not scattered string checks.
- Diagnostics should be accumulated with source IDs and declaration-site notes.
- Optimization is allowed only below the source-level value semantics boundary.

## Non-Goals For The Ideal Core

- whole-program inference as the default package model
- classes or inheritance
- trait/protocol/typeclass dispatch in v1
- field-level visibility as the first representation-hiding mechanism
- source-level references or borrowing syntax
- backend-specific semantics
- permanent distinction between compiler-known enum-like types and user-defined enums

## Summary

The most beautiful structure is not a larger parser or a smarter VM. It is a clean semantic spine:

```text
package graph -> resolved HIR -> typed HIR -> interface -> MIR -> backend
```

Everything else should attach to that spine without owning language semantics.
