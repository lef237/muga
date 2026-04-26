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
- expose identifier references as analysis records that carry source spans and `BindingId`
- expose expression type results from typechecking
- keep the current runtime and bytecode behavior unchanged

Source spans are acceptable as a temporary bridge for this step because the current AST does not yet carry stable node IDs. Before typed HIR becomes the main compiler boundary, Muga should add explicit node identity, such as `ExprId` and `StmtId`, where span-based lookup is not precise enough.

## Package Identity

Package work should introduce a package symbol graph before package flattening is removed.

Recommended model:

- `PackageId` identifies one loaded package
- `PackageItemId` identifies one top-level item in that package
- imports map local alias symbols to `PackageId`
- qualified references resolve to `(PackageId, PackageItemId)`

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

## Current Migration Status

Done:

- resolver scopes use `Symbol -> BindingId`
- typechecker scopes use `Symbol -> BindingId`
- resolver and typechecker both keep internal binding tables
- shared ID wrapper types exist in `src/identity.rs`

Remaining:

1. expose resolved identity data instead of keeping it only inside resolver/typechecker
2. replace temporary span-keyed analysis records with stable node identity where needed
3. add package-aware identities before replacing package flattening
4. lower into typed HIR using resolved identities
