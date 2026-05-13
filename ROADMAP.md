# Muga Roadmap

This roadmap is the source of truth for the next implementation priority and the longer design path. For the detailed implementation ledger, resume checklist, and next-slice test plan, see [docs/implementation-resume-plan.md](./docs/implementation-resume-plan.md).

## Current Snapshot

Implemented compiler/runtime pieces:

- lexer, parser, resolver, typechecker, HIR lowering, bytecode compiler, and VM runtime
- `check` and `run` entrypoints
- symbol-based local binding identity and package item identity
- typed HIR carrying expression types, local binding targets, call targets, call origin, and package item references
- diagnostics with related notes and suggestions in selected resolver, typechecker, record, and package paths

Implemented language surface:

- immutable-by-default bindings, `mut`, no shadowing, local-only inference, higher-order functions, and closure capture
- records, field access, `record.with(...)`, chained calls, and package-qualified chained calls
- local binding annotations and function type annotations with `->`
- `List[T]`, `Option[T]`, `Result[T, E]`, and `Map[K, V]` type expressions
- list literals, indexing, `len`, `is_empty`, `push`, `get`, and `set`
- `Option::Some`, `Option::None`, `Result::Ok`, `Result::Err`, and exhaustive `match` for compiler-known `Option` and `Result`
- user-defined `enum` declarations with optional unconstrained type parameters, zero-payload and one-payload variants, qualified construction/patterns, exhaustive `match`, typed HIR, VM execution, and in-memory package interface summaries
- enum diagnostics, package enum visibility coverage, imported `alias::Enum::Variant` constructors/patterns, package enum call-target identity, and stale enum interface validation
- deterministic v1 package interface text persistence, content hashes, artifact path naming, file round-trip, and loaded-interface validation for public records/functions/enums
- loaded package interfaces can act as the dependency boundary for downstream typed checking without reading dependency implementation bodies
- `Map.empty`, `contains`, `get`, `insert`, and `remove` for `Int`, `Bool`, and `String` keys
- file-based package mode with `package`, `import`, `pkg`, `pub`, `as`, module-private top-level items, and `alias::Name`
- minimal `muga.toml` project mode with `[package] name/source`
- in-memory package interface summaries for public records/enums/functions and validation of public package references against those summaries

Current architectural gaps:

- user-defined generic records/functions are not implemented
- `pub fn` still requires explicit public signatures
- normal package checking/execution still flattens packages and reads dependency source
- package interface artifact discovery, cache integration, and invalidation are not implemented
- VM bytecode still lowers from the older HIR path, not from typed HIR/MIR

## Settled Direction

These are baseline decisions, not active roadmap questions:

- no classes or inheritance
- data is modeled with `record`; behavior is modeled with functions
- method-like calls are surface syntax over functions
- `::` is package qualification
- module/file-private top-level items are the default in package mode; `pkg` shares inside a package; `pub` exports across packages
- v1 has no trait, interface, protocol, typeclass, or overloaded dispatch declarations
- source values use value semantics; internal sharing/copy elision is an implementation detail
- ordinary Muga code does not use `ref T`, `mut ref T`, address-of, dereference, or raw pointer syntax
- `Option[T]` is canonical optional spelling; `T?` remains reserved
- `Result[T, E]` is the recoverable-error type; possible propagation sugar should be visible as `try expr`
- package interfaces store resolved public signatures so downstream packages do not infer through dependency bodies

Related design notes:

- collections: [spec/008-collections.md](./spec/008-collections.md)
- generics: [spec/009-generics.md](./spec/009-generics.md)
- explicit references: [spec/010-references-draft.md](./spec/010-references-draft.md)
- value semantics: [spec/011-value-semantics.md](./spec/011-value-semantics.md)
- protocol-like abstractions: [spec/012-protocols-deferred.md](./spec/012-protocols-deferred.md)
- enums and result handling: [spec/013-enums-results.md](./spec/013-enums-results.md)

## Immediate Priority

The next code slice is interface artifact discovery and invalidation:

1. Find persisted dependency interface artifacts from the package/interface root instead of requiring callers to pass an already-loaded graph.
2. Reject missing, hash-mismatched, or stale artifacts with regeneration guidance.
3. Keep interface-backed typed checking aligned with current body-based checking on existing package samples.
4. Avoid silent fallback to dependency implementation bodies in artifact-backed checking.
5. Keep full package caching, MIR, native backend work, wildcard enum patterns, and `try expr` deferred until artifact-backed checking is stable.

## Compiler Architecture Path

1. **Package interfaces as real inputs**

   Persist public records/functions/enums, resolved signatures, item identity, and hashes. Downstream packages should check against interface artifacts instead of dependency bodies.

2. **Remove package flattening**

   Use package graph and interface data as the normal checking/compilation boundary. Keep package-aware diagnostics useful across that boundary.

3. **Build cache and incremental compilation**

   Reuse unchanged package interface and implementation artifacts. Invalidate by source hash, interface hash, and dependency graph.

4. **MIR**

   Lower typed HIR into a compiler-oriented MIR that makes control flow, evaluation order, temporaries, and locals explicit.

5. **Native backend**

   Add a fast native backend after the semantic boundary and package model are stable. Cranelift remains the likely first backend candidate; LLVM can be reconsidered later if its tradeoffs become useful.

6. **Structured concurrency**

   Design `group` / `spawn` / `join` first, then typed channels, then `select`-style coordination and timeouts. Do not make `async fn` / `await` the primary concurrency model unless later evidence justifies it.

7. **Standard library**

   Add IO, HTTP, strings, richer collections, process/time APIs, and web-oriented packages after package compilation and error handling are stable enough to support them cleanly.

## Cross-Cutting Work

Benchmarking and profiling should continue through every compiler step:

- lex, parse, resolve, typecheck
- HIR and typed HIR lowering
- package interface loading/validation
- MIR lowering
- bytecode/native codegen

Diagnostics remain part of the architecture, not a late polish layer. New enum, package-interface, cache, MIR, and backend work should keep stable spans, declaration-site notes, and actionable suggestions where they materially improve debugging.

## Queued Decisions

Package-interface queue:

- interface artifact discovery
- interface cache keys and invalidation
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

## Deferred Surface Work

These should stay deferred unless the active implementation slice requires them:

- map literals, `Set[T]`, arbitrary `Map` key types, and broad collection APIs
- generic records/functions beyond the planned MVP slice
- bounds, typeclasses, higher-kinded types, const generics, specialization, and polymorphic recursion
- wildcard-heavy pattern matching, guards, nested destructuring, and named-field enum variants
- persisted dependency declarations, registries, package archives, lockfiles, and package signing
- source-level references, mutable references, pointer syntax, or borrowing syntax

## Short Version

The coherent path is:

1. persisted package interfaces
2. package checking without flattening
3. cache and incremental compilation
4. MIR
5. native backend
6. structured concurrency
7. practical standard library
