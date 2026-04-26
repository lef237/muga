# Muga Roadmap

This document tracks the current implementation roadmap for Muga.

It replaces older ad hoc roadmap notes by separating:

- what is already decided
- what is already implemented
- what should come next

The roadmap is optimized for these goals:

- simple and readable language design
- low syntactic overhead
- strong static typing with minimal annotations
- very fast compilation
- a clear path from today's interpreter-oriented implementation to a fast native compiler

## Current Snapshot

As of now, Muga already has:

- lexer, parser, resolver, typechecker
- HIR lowering
- bytecode compiler and VM runtime
- symbol interning in HIR / bytecode / runtime
- records, field access, record update, and UFCS-style chains
- higher-order functions with local bidirectional inference
- file-based package mode with `package`, `import`, `pub`, `alias::Name`, and `as`

The biggest remaining architectural gap is this:

- the parser / package layer has moved forward
- resolver and typechecker now use symbol-based scope lookup internally
- resolved local binding and expression type data are now exposed as reusable compiler data
- typed HIR does not exist yet
- package compilation is still implemented by flattening packages into one internal program

That means the language surface is ahead of the compiler core.

## What Is Already Settled

These are no longer top-level roadmap questions:

1. package syntax exists
   - `package`
   - `import`
   - `pub`
   - `alias::Name`
2. public API annotation policy exists
   - public functions must have inferable resolved signatures
   - current implementation still enforces fully annotated `pub fn` until package interfaces exist
3. package qualification uses `::`
4. package mode and script mode are distinct

Those topics may still need refinement, but they are no longer the main blockers.

## Execution Strategy

Muga should continue as a compiler-first project.

That means:

- the long-term primary product is a native compiler
- the current VM / bytecode runtime should remain as a secondary execution backend
- the VM should be treated as a reference execution path for semantics, testing, and fast iteration

What should be avoided:

- keeping a separate AST interpreter forever
- duplicating language semantics across multiple unrelated execution engines
- letting the VM define semantics independently from the checked IR pipeline

The intended long-term shape is:

1. source
2. parser / resolver / typechecker
3. typed HIR
4. MIR
5. backend A: VM / bytecode
6. backend B: native compiler

This gives Muga three benefits at once:

- the compiler remains the main direction
- the VM remains useful for debugging, testing, and semantic validation
- the language does not pay for duplicated front-end logic

Type checking and inference must stay shared across both backends.

That means:

- `muga check`, `muga run`, and future `muga build` should accept the same programs
- VM execution is for fast development feedback, not a looser type system
- native compilation is for final executables and performance-sensitive delivery
- inferred package interfaces should be generated once and reused by both execution paths

## Cross-Cutting Infrastructure

These are not late polish tasks.

They should start early and continue through the rest of the roadmap.

### Benchmarking And Profiling

Muga wants very fast compilation, so compile-time measurement has to be treated as core infrastructure.

That means:

- measure early
- keep measuring after each architectural step
- use stage-by-stage numbers to validate whether a change actually helps

Expected measurements:

- lex time
- parse time
- resolve time
- typecheck time
- HIR lowering time
- package-interface loading time
- MIR lowering time
- codegen time

This was previously described as a later standalone phase.

The better interpretation is:

- benchmarking begins before or during symbolization work
- benchmarking continues through typed HIR, package interfaces, caching, and backend work

### Diagnostics Architecture

Diagnostics should also be treated as core architecture, not as UI polish.

This matters more once Muga has:

- package-qualified names
- import graphs
- interface checking
- cached package artifacts
- multiple lowering stages

The diagnostic data model should eventually support:

- stable source spans
- cross-package error reporting
- references to resolved bindings and package symbols
- room for candidate suggestions and related notes

This does not have to be a giant subsystem immediately, but the data model should be made deliberate before package interfaces and caching harden.

## Priority Order

## 1. Resolver And Typechecker Identity Outputs

Goal:

- make resolver and typechecker produce reusable identity data for later compiler stages
- keep local binding identity available beyond diagnostics-only checking

Why this is first:

- it directly affects front-end hot paths
- the first internal symbolization pass is already in place
- typed HIR becomes much easier once bindings have stable IDs
- compile-speed work should start here, not at the backend

Expected outcomes:

- resolved identifier uses can point to `BindingId` or package item identity
- typechecker can consume or produce the same identity vocabulary as resolver
- less repeated name lookup when lowering into typed HIR

Immediate implementation slice:

- expose accepted bindings as compiler data, not only as scope internals
- expose identifier-use resolution as `ExprId -> BindingId` data
- expose expression type results as typed analysis data keyed by `ExprId`
- keep the existing `check`, `run`, and bytecode paths behavior-compatible while this data becomes available

The current identity design note lives in [docs/internal/identity-model.md](./docs/internal/identity-model.md).

## 2. Package Symbol Graph And Identity Model

Goal:

- define how symbols are identified across package boundaries before typed HIR and package interfaces are locked in

Why this is second:

- the current package implementation still flattens packages into one internal program
- that is exactly the phase where symbol identity tends to drift if it is not fixed early
- package interfaces, import resolution, and typed HIR all need a stable answer to "what symbol is this, across packages?"

Expected outcomes:

- a package-aware symbol identity model
- stable IDs or handles for package-level items
- a clear distinction between local binding identity and package-exported symbol identity
- a package symbol graph that survives the end of flattening

This is the architectural bridge between:

- symbol-based local analysis
- real package compilation units

## 3. Typed HIR

Goal:

- lower checked programs into a typed HIR where names, bindings, and expression types are already fixed

Why this is third:

- later stages should not redo name resolution or type inference
- this is the real boundary between front-end analysis and code generation
- receiver-style resolution should be finalized here, not as a separate late patch

Expected outcomes:

- each HIR expression has a resolved type
- each identifier use points to a resolved binding ID
- each package-level reference points to a resolved package symbol identity
- each function call has an already chosen callee shape
- visibility, import resolution, and qualified-path resolution are already settled
- receiver-style and chained-call resolution become explicit compiler data, not repeated logic

Note:

- the earlier roadmap item "receiver-style resolution" is now folded into this step
- this is the right place to make that rule final
- by the time a program reaches typed HIR, at least these things should be fixed:
  - binding identity
  - resolved callee shape
  - resolved type
  - package-qualified symbol identity

HIR boundary:

- typed HIR should still be relatively high-level
- it should preserve language-level structure where that helps semantics and diagnostics
- it should not yet be forced into backend-oriented control-flow form

## 4. Package Interfaces And Real Package Compilation Units

Goal:

- replace the current package-flattening strategy with real package interfaces
- store resolved public signatures in package interfaces, whether the signatures were written or inferred

Why this is fourth:

- flattening is fine for early execution
- it is not the right long-term shape for fast compilation
- package-level interfaces are required before cache and incremental compilation

Expected outcomes:

- package header / interface summaries
- imported package metadata available without loading full bodies
- package graph built explicitly
- package-level checking separated from whole-program flattening
- inferred public function signatures cached as part of the package interface

Policy:

- source code should remain inference-first even for `pub fn` when inference is unique
- package interfaces should contain explicit resolved signatures
- downstream packages should not infer through dependency bodies
- recursive, mutually recursive, or ambiguous public APIs may still require annotations

Likely related work:

- source-root and manifest design
- entry-package conventions
- cleaner import graph diagnostics

## 5. Build Cache And Incremental Compilation

Goal:

- reuse unchanged packages instead of rebuilding everything

Why this is fifth:

- package interfaces must exist first
- compile speed at Go-like scale depends heavily on package caching

Expected outcomes:

- source hash and interface hash
- dependency graph tracking
- reuse of unchanged package artifacts
- invalidation only when needed

This is the step where Muga starts to become a genuinely fast multi-package compiler instead of a fast small-language prototype.

## 6. Split VM Bytecode From Compiler MIR

Goal:

- keep VM bytecode for interpreter execution
- add a separate compiler-oriented MIR for native code generation

Why this is sixth:

- the current bytecode is good for execution and testing
- it is not necessarily the right IR for fast native codegen
- typed HIR should feed a MIR designed for code generation, not just VM execution

Expected outcomes:

- typed HIR remains semantics-fixed but still relatively high-level
- MIR makes control flow, evaluation order, temporaries, and locals explicit
- one path for interpreter / VM
- one path for compiler backend
- cleaner separation of concerns

Design policy:

- keep the VM as a supported reference backend
- do not let it become a second independent compiler architecture
- new semantics should enter through typed HIR / MIR first, then flow to both backends

This boundary matters:

- if typed HIR becomes too low-level, it collapses into MIR and the separation loses value
- if typed HIR stays too high-level, both the VM and native backend pay for repeated lowering work

The intended split is:

- typed HIR: semantics fixed, still language-shaped
- MIR: execution-shaped IR for backend work

## 7. Native Backend

Goal:

- compile Muga programs to native code

Recommended direction:

- prefer a fast backend first
- Cranelift is the most likely good first target
- avoid a heavyweight LLVM-first strategy unless Muga later needs that tradeoff

Why this is not earlier:

- backend speed matters
- but bad front-end and package architecture will dominate build time first

## 8. Concurrency Model And Runtime

Goal:

- give Muga a lightweight and high-performance concurrency model that is simple to read, easy to use, and safe by default

Design target:

- lightweight tasks in the spirit of goroutine-style ease of use
- structured lifetimes rather than fire-and-forget by default
- typed message-passing primitives
- explicit cancellation and timeout support
- strong defaults against accidental shared-state bugs

Recommended direction:

- keep concurrency compiler-friendly and runtime-friendly
- prefer structured concurrency over unbounded detached task creation
- use task handles and task groups instead of function coloring as the primary model
- use typed channels or equivalent message-passing primitives for coordination
- treat shared mutable state as opt-in, not as the default style

Why this is here and not earlier:

- concurrency design affects runtime, MIR shape, diagnostics, and standard library boundaries
- it should not be fully implemented before the compiler core and package model are stable
- but it should be designed before web-oriented libraries harden around a weaker model

What Muga should try to combine:

- Go-like ease of spawning lightweight concurrent work
- structured lifetime management similar to modern structured-concurrency systems
- static restrictions that fit Muga's immutable-by-default model
- runtime designs that can be benchmarked directly against real concurrent workloads

Important constraint:

- the roadmap should treat "Go-level or better" as a benchmark target, not as something syntax alone can guarantee
- the actual result will depend on scheduler design, allocation behavior, synchronization costs, and backend quality

Initial language-level direction:

- Phase 1 should stabilize `group`, `spawn`, and `join`
- Phase 2 can add channels with readable send/receive operations
- Phase 3 can add `select`-style waiting, timeouts, and related coordination features
- the primary model should stay task-group based rather than centered on colored async functions

Safety direction:

- immutable-by-default values should remain easy to share
- mutable capture across task boundaries should be restricted or made explicit
- synchronization primitives should exist, but they should not define the primary style

## 9. Standard Library And Web-Oriented Capabilities

Goal:

- add the packages and runtime surface needed for practical web development

Why this is later:

- language ergonomics and compiler architecture should stabilize first
- standard library work is much cheaper once package compilation and caching are real

Likely topics:

- `std` package layout
- IO
- HTTP
- strings and collections
- concurrency model

## Recommended Immediate Next Steps

If work resumes right now, the best order is:

1. package-aware symbol identity design
2. typed HIR
3. receiver-style resolution finalization inside typed HIR lowering/checking
4. diagnostic data model tightening
5. package interfaces instead of flattening
6. cache and incremental compilation

This order best matches the current state of the codebase.

## Short Version

The roadmap is now:

1. measure performance from the beginning
2. make the front end fast internally
3. fix symbol identity across locals and packages
4. make checked programs explicit through typed HIR
5. make packages real compilation units
6. add cache and incremental compilation
7. split VM IR from compiler IR
8. add a fast native backend
9. design and implement structured high-performance concurrency

That is the most coherent path toward a simple, modern, and very fast Muga compiler.
