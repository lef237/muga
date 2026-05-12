# Compiler Identity Model

Status: implementation planning note.

This note defines the identity model Muga should use before typed HIR and real package interfaces are introduced.

## Goals

- avoid repeated string-based lookup in compiler hot paths
- give resolver, typechecker, typed HIR, and package interfaces a shared vocabulary
- keep local identity and package identity separate
- make package flattening removable without redesigning all IDs later

## Name Text vs Identity

Muga should keep these concepts separate:

- source text name: the spelling from the source file
- `Symbol`: interned spelling inside one compiler session
- `BindingId`: resolved binding inside a checked program body or scope tree
- `LocalId`: lowered local storage slot after name resolution and typing
- `PackageId`: package node in the package graph
- `ModuleId`: module/file node inside a package
- `PackageItemId`: top-level exported or private item inside a package

`Symbol` is not enough by itself because two different scopes can define the same spelling. A resolved identifier should eventually point to a `BindingId` or `PackageItemId`, not just to the interned text.

## Local Binding Identity

Resolver should assign a `BindingId` whenever it accepts a new binding:

- immutable local binding
- mutable local binding
- function binding
- function parameter
- prelude binding

Each scope maps `Symbol -> BindingId`.

Each `BindingId` records at least:

- symbol
- binding kind
- declaration span

Typed HIR should later store resolved identifier uses as `BindingId` instead of looking names up again.

The first migration step is intentionally smaller than full typed HIR:

- expose the accepted binding table from resolver/typechecker
- expose identifier references as analysis records that carry `ExprId`, source spans, and `BindingId`
- expose expression type results from typechecking
- keep the current runtime and bytecode behavior unchanged

Source spans are still kept for diagnostics, but analysis consumers should prefer explicit node identity. The current AST carries `ExprId` and `StmtId`, and resolver/typechecker outputs use `ExprId` for identifier references and expression types. Package flattening renumbers node IDs after combining files so IDs remain unique inside the final checked program.

## Package Identity

Package loading now introduces a package symbol graph before package flattening is removed.

Recommended model:

- `PackageId` identifies one loaded package
- `ModuleId` identifies one source module/file inside that package
- `PackageItemId` identifies one top-level item in that package
- imports map local alias symbols to `PackageId`
- qualified references resolve to `(PackageId, PackageItemId)`

Current implementation:

- `load_from_entry` returns both the flattened program and `PackageSymbolGraph`
- `PackageSymbolGraph` stores package nodes, top-level item nodes, and import edges
- module records keep package membership and source file/module path
- item records keep source name, kind, visibility, declaring module, source span, and current mangled name
- the existing VM path still consumes the flattened program

Module-private visibility is now enforced for top-level package items during package rewriting. Unmodified top-level items are visible only inside their declaring source file, `pkg` items are visible to sibling files in the same package, and imports expose only `pub` items. Per-field record visibility is not a required v1 slice; if public hidden representations become necessary, opaque records or opaque types should be evaluated first.

This lets the compiler distinguish:

- local binding identity
- current-package top-level item identity
- imported package item identity

## Typed HIR Boundary

By the time code reaches typed HIR, these should be fixed:

- each identifier use has a resolved local binding or package item identity
- each expression has a resolved type
- each call has a resolved callee shape
- each qualified path has a resolved package identity
- visibility and import checks are complete

Typed HIR should not perform string-based name lookup.

Typed HIR should consume analysis outputs rather than rerunning resolver or typechecker logic. In particular, identifier expressions should already know their binding identity, and expressions should already have a resolved type.

Current typed HIR status:

- checked AST can lower into `typed_hir::Program`
- typed HIR keeps language-shaped statements and expressions
- expressions carry `ExprId` and resolved `TypeInfo`
- identifier expressions carry `BindingId` plus `IdentTarget`, which can point at a package item
- assignment statements carry target `BindingId`
- package symbol graph is preserved on typed HIR programs
- call expressions carry explicit resolved callee shape and call origin
- package call targets and package record types carry `PackageItemId`-backed identity
- package item identity is still partly recovered from flattened mangled names during typed HIR lowering; remove that transitional dependency before replacing package flattening
- `Option` and `Result` match patterns are represented as enum variant patterns with enum name, variant name, and optional payload binding
- compiler-known enum metadata currently describes `Option` and `Result` and feeds current match validation plus runtime construction/branching

## Current Migration Status

Done:

- resolver scopes use `Symbol -> BindingId`
- typechecker scopes use `Symbol -> BindingId`
- resolver and typechecker both keep internal binding tables
- shared ID wrapper types exist in `src/identity.rs`
- AST expressions and statements carry `ExprId` / `StmtId`
- resolver exposes accepted bindings and identifier references
- typechecker exposes accepted bindings, identifier references, and expression types
- package loading exposes `PackageSymbolGraph`
- initial typed HIR exists
- typed HIR calls carry resolved callee shape and call origin
- package loading exposes `ModuleId` data and enforces top-level module-private / `pkg` / `pub` visibility
- typed HIR package identifiers, call targets, and record types point to package item identities
- diagnostics support related notes and suggestions for selected resolver, typechecker, and package errors
- typed HIR can generate in-memory package interface summaries for public records and functions
- typed package compilation validates public package references against generated interface summaries
- typed package interface validation uses package/name lookup and checks stale public item identity, function signatures, and record field shapes
- import/package-qualified lookup is routed through `PackageExportGraph` before whole-program flattening
- `PackageExportGraph` can be derived from either package identity data or typed package interface summaries
- local binding annotations, generic type expression syntax, `List[T]` / `Option[T]` / `Result[T, E]` / `Map[K, V]` `TypeInfo` cases, typed prelude list and map operations, direct list indexing, typed Option/Result construction, typed Option/Result `match`, enum-variant-shaped match patterns, and compiler-known enum metadata for Option/Result variants are in place

Remaining:

1. extend enum/variant identity from compiler-known `Option` / `Result` metadata to user-defined enums
2. remove remaining typed HIR package item recovery from flattened mangled names before package flattening is replaced
3. make downstream package checking consume typed package interface summaries instead of the source-level export surface
4. continue expanding structured diagnostics as new enum and interface errors are introduced

## Foundation Note

The typed HIR and package symbol graph land as a foundation ahead of the remaining items above.

The reason is that they add reusable compiler data without replacing the existing VM execution path. The remaining items are handled as follow-up compiler-core work on top of this foundation.
