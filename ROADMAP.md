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
- but the resolver and typechecker are still mostly string-based
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
   - `pub fn` must be fully annotated
3. package qualification uses `::`
4. package mode and script mode are distinct

Those topics may still need refinement, but they are no longer the main blockers.

## Priority Order

## 1. Resolver And Typechecker Symbolization

Goal:

- move `resolver` and `typechecker` from string-heavy lookup to `SymbolId`-based lookup
- introduce stable IDs for bindings and locals

Why this is first:

- it directly affects front-end hot paths
- it reduces repeated string hashing and cloning
- typed HIR becomes much easier once bindings have stable IDs
- compile-speed work should start here, not at the backend

Expected outcomes:

- `SymbolId` for names in resolver/typechecker
- `BindingId` / `LocalId` style internal identities
- less repeated `HashMap<String, ...>` traffic in inner loops

## 2. Typed HIR

Goal:

- lower checked programs into a typed HIR where names, bindings, and expression types are already fixed

Why this is second:

- later stages should not redo name resolution or type inference
- this is the real boundary between front-end analysis and code generation
- receiver-style resolution should be finalized here, not as a separate late patch

Expected outcomes:

- each HIR expression has a resolved type
- each identifier use points to a resolved binding ID
- each function call has an already chosen callee shape
- receiver-style and chained-call resolution become explicit compiler data, not repeated logic

Note:

- the earlier roadmap item "receiver-style resolution" is now folded into this step
- this is the right place to make that rule final

## 3. Package Interfaces And Real Package Compilation Units

Goal:

- replace the current package-flattening strategy with real package interfaces

Why this is third:

- flattening is fine for early execution
- it is not the right long-term shape for fast compilation
- package-level interfaces are required before cache and incremental compilation

Expected outcomes:

- package header / interface summaries
- imported package metadata available without loading full bodies
- package graph built explicitly
- package-level checking separated from whole-program flattening

Likely related work:

- source-root and manifest design
- entry-package conventions
- cleaner import graph diagnostics

## 4. Build Cache And Incremental Compilation

Goal:

- reuse unchanged packages instead of rebuilding everything

Why this is fourth:

- package interfaces must exist first
- compile speed at Go-like scale depends heavily on package caching

Expected outcomes:

- source hash and interface hash
- dependency graph tracking
- reuse of unchanged package artifacts
- invalidation only when needed

This is the step where Muga starts to become a genuinely fast multi-package compiler instead of a fast small-language prototype.

## 5. Split VM Bytecode From Compiler MIR

Goal:

- keep VM bytecode for interpreter execution
- add a separate compiler-oriented MIR for native code generation

Why this is fifth:

- the current bytecode is good for execution and testing
- it is not necessarily the right IR for fast native codegen
- typed HIR should feed a MIR designed for code generation, not just VM execution

Expected outcomes:

- one path for interpreter / VM
- one path for compiler backend
- cleaner separation of concerns

## 6. Native Backend

Goal:

- compile Muga programs to native code

Recommended direction:

- prefer a fast backend first
- Cranelift is the most likely good first target
- avoid a heavyweight LLVM-first strategy unless Muga later needs that tradeoff

Why this is not earlier:

- backend speed matters
- but bad front-end and package architecture will dominate build time first

## 7. Benchmarking And Profiling Infrastructure

Goal:

- measure compile time by stage and track regressions

This should start early and continue through every phase, but it becomes especially valuable once the first fast compiler path exists.

Expected measurements:

- lex time
- parse time
- resolve time
- typecheck time
- lower time
- package-interface loading time
- codegen time

This should be treated as ongoing infrastructure, not as a one-time final polish step.

## 8. Standard Library And Web-Oriented Capabilities

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

1. symbol-based resolver
2. symbol-based typechecker
3. typed HIR
4. receiver-style resolution finalization inside typed HIR lowering/checking
5. package interfaces instead of flattening
6. cache and incremental compilation

This order best matches the current state of the codebase.

## Short Version

The roadmap is now:

1. make the front end fast internally
2. make checked programs explicit through typed HIR
3. make packages real compilation units
4. add cache and incremental compilation
5. split VM IR from compiler IR
6. add a fast native backend

That is the most coherent path toward a simple, modern, and very fast Muga compiler.
