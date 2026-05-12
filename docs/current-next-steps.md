# Current Next Steps

Status: working resume note. The implementation ledger and last verification snapshot live in [implementation-resume-plan.md](./implementation-resume-plan.md).

## Baseline

Current direction:

- compiler-first, with the VM retained as a reference execution backend
- function-centered language with records for data and no classes
- local inference first; no whole-program inference as the default model
- package interfaces for fast separate compilation
- explicit package qualification with `::`
- module/file privacy by default; `pkg` for package-internal sharing; `pub` for imports
- v1 generics as a small MVP, without bounds, typeclasses, higher-kinded types, const generics, specialization, traits, interfaces, protocols, or overloaded dispatch
- `List[T]`, `Option[T]`, `Result[T, E]`, and the first `Map[K, V]` slice as the current collection/error core
- value semantics in source code, with internal sharing and copy elision as implementation details
- no explicit source-level references in ordinary Muga code
- structured task groups before channels or async-function coloring

## Current Implementation

Implemented:

- resolver/typechecker identity data and typed HIR call targets
- package graph identity with `PackageId`, `ModuleId`, and `PackageItemId`
- module-private, `pkg`, and `pub` package visibility before flattening
- `interface::PackageExportGraph` for public import lookup
- `interface` owns in-memory package interface summaries for public records/functions
- `types` owns shared public `TypeInfo` data used by typechecker output, typed HIR, and interfaces
- package rewriting attaches item identity to flattened AST declarations; typed HIR and `interface` use those IDs instead of recovering them from mangled names
- shared prelude/builtin identity catalog; resolver, typechecker, runtime, and package builtin lookup all use `BuiltinId`
- local binding annotations and generic type expressions
- `List[T]` literals, indexing, `len`, `is_empty`, `push`, `get`, and `set`
- compiler-known `Option[T]` and `Result[T, E]` with qualified constructors and exhaustive `match`
- generic runtime enum value shape and compiler-known enum metadata for current `Option` / `Result`
- `Map[K, V]` for `Int`, `Bool`, and `String` keys with `Map.empty`, `contains`, `get`, `insert`, and `remove`
- related-note and suggestion diagnostics in selected resolver, typechecker, record, and package errors

Not implemented:

- user-defined enum declarations
- user-defined generic records/functions
- map literals, `Set[T]`, arbitrary `Map` key types, and broad collection APIs
- public-signature inference for `pub fn`
- persisted package interfaces, dependency manifests, registries, lockfiles, package caches, MIR, and native codegen
- `try expr` or any other error propagation sugar

## Next Code Slice

Implement user-defined `enum` declarations before broad stdlib or persisted package-interface work.

Scope:

1. Parse zero-payload and one-payload enum variants.
2. Add enum/variant identity to resolver, typechecker, HIR, typed HIR, and package summaries.
3. Reuse the current `Option` / `Result` enum metadata and runtime value path.
4. Keep exhaustive `match`; defer wildcard patterns and nested destructuring.
5. Keep `try expr` deferred until enum identity and public enum signatures are stable.

Why this is next:

- `Option[T]` and `Result[T, E]` already prove the enum-like runtime and checking path.
- Public APIs for IO, process, HTTP, time, and concurrency should use explicit `Result[T, E]` before adding propagation sugar.
- Persisted package interfaces should not freeze before public enum declarations and enum identities are representable.

## Decisions Before Coding

Resolve these before or during the enum slice:

- exact `enum` declaration grammar
- whether MVP variants are limited to zero or one unnamed payload
- whether all variant construction and patterns remain qualified as `Enum::Variant`
- exhaustiveness diagnostics for missing and duplicate variants
- enum identity across local declarations, packages, and package interfaces
- whether compiler-known `Option` / `Result` later become ordinary stdlib enum declarations without changing source syntax

Keep these decisions closed for the next slice:

- no `let`
- immutable by default
- no shadowing
- no classes or inheritance
- `record` instead of `struct`
- no function-valued record fields in v1
- no trait, interface, protocol, typeclass, or overloaded dispatch declarations in v1
- `Option[T]` remains canonical; `T?` remains reserved
- no postfix `?` for the current error-propagation direction
- no source-level pointer/reference syntax

## Later Queues

Package-interface queue:

- persisted package interface format
- interface hashes and cache keys
- downstream checking from interface artifacts
- source-root and manifest conventions
- serialization of inferred public signatures once supported

Concurrency queue:

- whether task handles are source-nameable as `Task[T]`
- `group` return behavior
- failure and cancellation representation
- capture rules across task boundaries
- channels as a later phase after `group` / `spawn` / `join`

Write-oriented API queue:

- builder/buffer types for repeated construction
- resource/handle types for files, sockets, processes, timers, and OS-backed effects
- MIR/native lowering for copy elision and internal destructive update

## Resume Checklist

1. Run `cargo test`.
2. Read [implementation-resume-plan.md](./implementation-resume-plan.md).
3. Read [ROADMAP.md](../ROADMAP.md).
4. Read [spec/013-enums-results.md](../spec/013-enums-results.md).
5. Start with user-defined enum declarations unless a package-interface task is explicitly being resumed first.
6. After compiler-core changes, keep `check`, `run`, and existing samples behavior-compatible.

Useful validation commands:

```bash
cargo test
cargo run -- check samples/println_sum.muga
cargo run -- samples/println_sum.muga
cargo run -- samples/packages/app/main/main.muga
```
