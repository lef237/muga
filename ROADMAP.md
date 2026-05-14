# Muga Roadmap

This roadmap is the source of truth for the next implementation priority and the longer design path. For the detailed implementation ledger, resume checklist, and next-slice test plan, see [docs/implementation-resume-plan.md](./docs/implementation-resume-plan.md).

## Current Snapshot

Implemented compiler/runtime pieces:

- lexer, parser, resolver, typechecker, typed HIR, initial MIR lowering, bytecode compiler, and VM runtime
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
- deterministic v2 package interface text persistence with stable artifact package/item IDs, content hashes, direct dependency metadata, artifact path naming, file round-trip, and loaded-interface validation for public records/functions/enums
- loaded package interfaces and discovered `.mgi` artifacts can act as the dependency boundary for downstream typed checking, including transitive public-signature type dependencies, without reading dependency implementation bodies
- independently generated `.mgi` artifacts are remapped from stable artifact identities into a fresh session-local package/item identity namespace when loaded together, so one artifact root can safely contain artifacts from separate provider builds
- package check cache keys combine entry package source hashes with loaded direct/transitive dependency interface hashes, and `.mgc` check artifacts are rejected when missing or stale
- `muga check --artifact-root <dir>` consumes `.mgi` and `.mgc` artifacts for dependency-body-free package checking
- `muga emit-interface` and `muga emit-artifacts` write reachable `.mgi` interfaces from package-aware typed HIR, and `emit-artifacts` also writes reachable `.mgb` implementation artifacts containing MIR-lowered bytecode programs plus the entry `.mgc` check cache; lower-level `emit-check-cache` validates against `.mgi` artifacts before writing `.mgc`
- `muga run --artifact-root <dir>` validates `.mgi` / `.mgc` / structurally checked `.mgb` artifacts, executes direct and transitive dependencies without reading dependency source files from the source tree, remaps independently generated implementation item references onto loaded interface identities, and rejects wrong-package or dependency-interface-mismatched `.mgb` files
- `Map.empty`, `contains`, `get`, `insert`, and `remove` for `Int`, `Bool`, and `String` keys
- file-based package mode with `package`, `import`, `pkg`, `pub`, `as`, module-private top-level items, and `alias::Name`
- minimal `muga.toml` project mode with `[package] name/source`
- unflattened package graph loading preserves package files plus package/module/item/export metadata before the legacy flattening path
- a library-only package-aware check path validates package boundary, import, visibility, and public-signature rules from the unflattened package graph before package-aware module checking
- package-aware source and per-module signature environments resolve record/enum/function signatures from the unflattened graph while preserving package item identity and module/same-package/import visibility
- package-aware module body resolution/typechecking consumes those module signature environments, and the package-aware API retains per-module resolver/typecheck outputs plus typed HIR programs
- package-aware check results aggregate per-module typed HIR from unflattened module check outputs with remapped local IDs and symbols instead of using the legacy flattened typed path
- default package `check` runs the package-aware validation path and no longer reloads a flattened package AST after validation
- default package `compile_typed_path` returns the package-aware typed HIR aggregate instead of the legacy flattened typed HIR
- flattened package loader APIs are explicitly named `load_flattened_*` so compatibility AST use is visible at call sites
- package-aware checking and loaded/interface-artifact typed compilation collect dependency signatures and build dependency graph metadata directly from loaded interfaces without reading dependency source bodies, and `muga check --artifact-root` plus interface artifact emission use package-aware paths
- the legacy interface-stub flattened typed compilation path has been removed; loaded/interface-artifact typed compilation now uses the package-aware semantic path only
- package-aware typed HIR lowers through the initial MIR module before VM bytecode generation for package records, enums, functions, and calls
- in-memory package interface summaries for public records/enums/functions and validation of public package references against those summaries

Current architectural gaps:

- user-defined generic records/functions are not implemented
- `pub fn` still requires explicit public signatures
- normal package execution still reads dependency source bodies when no artifact root is supplied; explicit artifact-backed execution is available for dependency-source-tree-free runs
- remaining package work is explicit artifact workflow documentation/sample hardening and later normal project/artifact integration; package-aware checking is now the default package validation path
- project-mode artifact-root config and full incremental package artifact reuse are not implemented
- VM bytecode execution now consumes an initial expression-shaped MIR with explicit execution bodies, body terminators, hoisted body-local function definitions, typed binding/package-item identity, typed assignment update mode, runtime names carrying binding/local identity, and slot-backed runtime environments with package function references canonicalized to their defining binding; control-flow-oriented MIR and native lowering are post-v1 unless needed to close a concrete artifact/execution gap
- default compile APIs lower typed HIR into MIR; the old untyped AST-to-HIR compatibility module has been removed

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

- practical language readiness: [docs/practical-language-readiness.md](./docs/practical-language-readiness.md)
- collections: [spec/008-collections.md](./spec/008-collections.md)
- generics: [spec/009-generics.md](./spec/009-generics.md)
- explicit references: [spec/010-references-draft.md](./spec/010-references-draft.md)
- value semantics: [spec/011-value-semantics.md](./spec/011-value-semantics.md)
- protocol-like abstractions: [spec/012-protocols-deferred.md](./spec/012-protocols-deferred.md)
- enums and result handling: [spec/013-enums-results.md](./spec/013-enums-results.md)

## Immediate Priority

The next code slices should convert the completed package-aware checking foundation into a v1-ready package/artifact experience:

1. Treat the current MIR/runtime identity cleanup as a foundation-closing slice, not an open-ended backend rewrite. The VM should execute checked package programs by lowered local identity and semantic package identity, but control-flow MIR and native lowering should wait unless they are required to close a concrete v1 execution gap.
2. Continue hardening dependency-body-free package execution. `check --artifact-root` avoids dependency implementation bodies, and `run --artifact-root` consumes MIR-lowered bytecode `.mgb` implementation artifacts for dependencies without source-tree fallback.
3. Keep artifact roots explicit on the CLI for v1. `muga emit-artifacts`, `muga check --artifact-root`, and any run-time artifact-root flag should fail loudly on missing or stale artifacts instead of silently falling back to dependency source bodies.
4. Harden the explicit artifact workflow with samples, diagnostics, and documentation before adding manifest-owned artifact configuration.
5. Do not add a `muga.toml` artifact-root field until dependency declarations, lockfiles, and a package-aware project driver exist. Revisit it later as a non-semantic `[build]` or `[cache]` setting once the dependency graph is manifest-owned.
6. Keep generic records/functions, wildcard enum patterns, `try expr`, native backend work, broad stdlib effects, and full incremental reuse deferred unless one becomes necessary to finish the v1 package/artifact path. Once the v1 package/artifact workflow is stable, resume language-surface work in the order recorded in [docs/practical-language-readiness.md](./docs/practical-language-readiness.md).

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

The post-v1 language-feature order and the list of features to keep out of Muga are maintained in [docs/practical-language-readiness.md](./docs/practical-language-readiness.md).

## Cross-Cutting Work

Benchmarking and profiling should continue through every compiler step:

- lex, parse, resolve, typecheck
- typed HIR lowering
- package interface loading/validation
- MIR lowering
- bytecode/native codegen

Diagnostics remain part of the architecture, not a late polish layer. New enum, package-interface, cache, MIR, and backend work should keep stable spans, declaration-site notes, and actionable suggestions where they materially improve debugging.

## Queued Decisions

Package-interface queue:

- documenting and sampling the explicit `.mgi` / `.mgc` / `.mgb` workflow as the v1 execution artifact contract
- broader `run --artifact-root` diagnostics for missing, stale, hash-mismatched, structurally invalid, wrong-package, or dependency-interface-mismatched artifacts
- eventual project-mode artifact-root config after dependency declarations and lockfiles
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

The coherent path to v1 is:

1. close the MIR/runtime identity foundation for the reference VM
2. make package execution work without dependency implementation bodies
3. keep `emit-artifacts` / `check --artifact-root` / artifact-backed execution explicit and non-silent
4. document and test the v1 package workflow end to end
5. only then resume larger post-v1 work: control-flow MIR, native backend, richer generics, structured concurrency, and practical standard library expansion
