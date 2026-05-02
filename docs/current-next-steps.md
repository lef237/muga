# Current Next Steps

Status: working note. This is a resume guide for continuing Muga design and implementation.

## 1. Current Direction

Muga's current direction is:

- compiler-first
- VM retained as a reference execution backend
- function-centered, with no classes
- `record` for data and functions for behavior
- local inference first, without whole-program inference as the default model
- package interfaces for fast separate compilation
- explicit package qualification with `::`
- module/file privacy before package-only privacy
- v1 generics as a small MVP
- no trait, interface, protocol, typeclass, or overloaded dispatch declarations in v1
- `List[T]` and `Option[T]` first for collections, then safe lookup and `Map[K, V]`
- no explicit source-level references in ordinary Muga code
- value semantics with internal sharing and copy elision
- structured task groups before channels or async-function coloring

The most important constraint is that ergonomics should not come at the cost of unstable semantics or slow whole-program compilation.

## 2. What Was Recently Decided

These points are now documented and should be treated as the current baseline:

- Muga will not introduce classes.
- Class inheritance is out of scope.
- Method-like calls are surface syntax over functions.
- Ruby is an important readability reference, but language features should be chosen by Muga's own constraints.
- Whole-program inference should not be the default compilation model.
- Public signatures may be inferred in the defining package, then stored in package interfaces.
- v1 generics include generic type expressions, generic records, and generic functions.
- v1 generics do not include bounds, typeclasses, higher-kinded types, const generics, or specialization.
- v1 does not introduce trait, interface, protocol, typeclass, or overloaded dispatch declarations.
- if a protocol-like abstraction is added later, `protocol` is the preferred spelling.
- generic declarations must declare their type parameters explicitly, such as `fn id[T](value: T): T`.
- `Option[T]` is the canonical spelling for optional values.
- `T?` is only reserved as possible future shorthand for `Option[T]`.
- `List[T]` means zero or more values.
- `Option[T]` means zero or one value.
- Empty list literals require an expected type such as `items: List[Int] = []`.
- Ordinary source code should use value semantics.
- The implementation may share immutable storage internally when that is not observable.
- Explicit source-level references such as `ref T`, `mut ref T`, `*T`, and `&value` are not planned for ordinary Muga code.
- Write-oriented APIs should prefer value-returning updates, builder/buffer types, or resource handles.
- performance competitive with fast mainstream compiled languages should be pursued through package interfaces, typed HIR, MIR, internal sharing, copy elision, resource handles, and native backend work.

## 3. Current Implementation Status

Recently completed:

- `TypeCheckOutput` exposes resolved call information.
- typed HIR `CallExpr` preserves the resolved callee.
- parser/AST call origin is threaded into typed HIR for ordinary calls, chained calls, and package-qualified chained calls.
- tests cover ordinary function calls, local function-value calls, builtin calls, chained calls, and package-qualified calls.
- package loading now records `ModuleId` data in `PackageSymbolGraph`.
- `pkg` is accepted for top-level package items.
- unmodified top-level package items are module-private, `pkg` / `pub` items are visible to sibling files, and imports expose only `pub` items.
- typed HIR identifiers can distinguish local bindings from package item targets.
- typed HIR package call targets use `PackageItemId`-backed callee data.
- typed HIR package record types use `PackageItemId`-backed type data.
- diagnostics can carry related notes and suggestions while preserving the existing single-line display for simple diagnostics.
- package module-private visibility diagnostics now point at the private declaration and suggest `pkg` when sharing within a package is intended.
- duplicate binding/record/field diagnostics point at the previous declaration.
- selected record literal/update and field-access diagnostics point at the relevant record or field declaration.
- package import alias conflicts and public-signature diagnostics include actionable suggestions.
- typed HIR can generate in-memory package interface summaries for public records and functions.
- interface summaries store `PackageItemId` and resolved `TypeInfo` for public signatures.
- `pkg` items are excluded from package interface summaries.
- typed package compilation validates public package references against generated interface summaries.
- interface validation uses package/name lookup and rejects stale public item identity, function signatures, and record field shapes.
- package import lookup now reads `PackageExportGraph`, a `PackageSymbolGraph`-derived public export surface, instead of dependency item maps.
- `PackageExportGraph` can also be derived from typed package interface summaries, giving the next pipeline step a drop-in export lookup shape.
- local binding annotations parse and typecheck as `name: Type = expr` and `mut name: Type = expr`.
- generic type expressions parse as `Type[Arg1, Arg2]`.
- `List[T]` type annotations and list literals are implemented through AST, resolver, typechecker, HIR, bytecode, VM runtime, typed HIR, and in-memory package interface summaries.
- `len`, `is_empty`, and value-returning `push` are implemented as typed prelude builtins and work through chained-call syntax.
- Empty list literals are accepted only when an expected `List[T]` type is available, such as `items: List[Int] = []`.
- `Option[T]`, `Option::Some`, `Option::None`, and exhaustive Option `match` are implemented through AST, resolver, typechecker, HIR, bytecode, VM runtime, typed HIR, and package interface summaries.
- `Map[K, V]`, list indexing, `set`, and safe lookup are not implemented yet.
- the existing VM bytecode path remains behavior-compatible.

## 4. Recommended Next Implementation Task

The best next implementation task is:

1. add safe list lookup, most likely `get(self: List[T], index: Int): Option[T]`
2. keep direct indexing and `set` out of the next slice unless the runtime bounds-error policy is finalized
3. keep general enum declarations deferred unless another feature requires them
4. use the same `Option[T]` representation for future `Map[K, V]` lookup

Why this comes next:

- `List[T]` now has syntax, static typing, typed HIR representation, package interface representation, runtime values, and a small non-indexing operation surface
- `get` naturally returns `Option[T]`, and optional values now have a source-level construction and consumption story
- `set` and direct indexing require a clear bounds-error policy
- `Option[T]` also informs future `Map[K, V]` lookup semantics

Expected result:

- safe lookup exposes absence without introducing runtime errors for ordinary misses
- later indexing, `set`, and `Map[K, V]` work can build on the same collection typing/runtime path

## 5. Decisions To Make Soon

These decisions affect near-term implementation and should be made before implementing the related feature.

### 5.1 Before the next collection slice

Decide:

- direct indexing policy: runtime bounds error for `xs[i]`, safe lookup through `xs.get(i)`
- whether general enum/sum-type declarations should land before `Result[T, E]`
- how future `Result[T, E]` and error propagation should relate to `match`

Current recommendation:

- local binding annotations, generic type expression parsing, `List[T]`, list literals, `len`, `is_empty`, `push`, `Option[T]`, `Option::Some`, `Option::None`, and exhaustive Option `match` are now implemented
- parse generic declarations as `record Box[T]` and `fn id[T](value: T): T`
- implement generic records and generic functions as part of v1
- rely on local type-argument inference rather than explicit call-site type arguments in the v1 MVP
- defer bounds, typeclasses, higher-kinded types, const generics, and specialization
- defer trait, interface, protocol, and overloaded dispatch declarations
- implement safe list lookup before safe map lookup
- keep `T?` reserved, not implemented
- do not implement map literals in the first collection slice

### 5.2 Before package interface implementation

Decide:

- package interface file/data format
- source-root and manifest conventions
- how module/file identity is represented in typed HIR
- how `pkg` visibility is enforced
- how inferred public signatures are serialized
- how package-interface hashes are computed

Current recommendation:

- keep the current package syntax
- stop flattening packages only after package item identity and typed HIR references are stable
- store resolved public signatures in package interfaces

### 5.3 Before concurrency implementation

Decide:

- whether task handles are source-nameable as `Task[T]`
- how `group` returns values
- how failure propagation is represented
- how cancellation is observed
- exact capture rules for immutable and mutable values across task boundaries
- whether channels are Phase 2 after `group` / `spawn` / `join`

Current recommendation:

- implement structured task groups before channels
- do not make `async fn` / `await` the primary model
- reject mutable outer capture across task boundaries by default

### 5.4 Before enum / error handling design

Decide:

- enum or sum-type syntax
- pattern matching syntax
- whether future `Result[T, E]` is an ordinary enum or a compiler-known standard type
- whether the current compiler-known `Option[T]` should later be replaced by ordinary enum declarations without changing source syntax
- whether `?` is reserved for optional shorthand, error propagation, optional chaining, or some combination

Current recommendation:

- keep `Option[T]` canonical
- keep `Option::Some` / `Option::None` qualified
- reserve `T?` only as possible future shorthand
- do not spend `?` on multiple meanings until error handling is designed

### 5.5 Before write-oriented API implementation

Decide:

- which standard types should represent repeated construction, such as `StringBuilder` or `Buffer`
- which standard types should represent external effects, such as file, socket, process, and timer handles
- whether ordinary collection update APIs return new values, builder-like values, or both
- how resource handles are owned, closed, and shared
- how resource handles interact with structured concurrency
- how MIR represents destructive update lowering for uniquely owned values

Current recommendation:

- do not add explicit source-level references
- do not add raw pointer, address-of, or dereference syntax
- prefer value-returning updates for ordinary data
- prefer builder/buffer types for repeated construction
- prefer resource/handle types for file, socket, process, timer, and OS-backed effects
- use MIR/native lowering for copy elision and internal destructive update when safe

The value semantics and performance direction lives in [spec/011-value-semantics.md](../spec/011-value-semantics.md).
The explicit references decision note lives in [spec/010-references-draft.md](../spec/010-references-draft.md).

## 6. What Not To Reopen Now

These decisions are settled enough to avoid re-litigating during the next implementation slice:

- no `let`
- immutable by default
- `mut` for mutable bindings
- no shadowing
- no classes
- `record` instead of `struct`
- no function-valued record fields in v1
- `expr.name` is field access
- `expr.name(...)` is chained call
- no trait, interface, protocol, typeclass, or overloaded dispatch declarations in v1
- package qualification uses `::`
- comments use `//`
- no source-level raw pointers in v1
- VM and compiler can coexist through a shared checked pipeline

## 7. Resume Checklist

When resuming implementation:

1. Run `cargo test`.
2. Read [ROADMAP.md](../ROADMAP.md) "Recommended Immediate Next Steps".
3. Read [docs/internal/identity-model.md](./internal/identity-model.md).
4. Start with safe list lookup (`get`) unless a compiler-core package task is explicitly being resumed first.
5. After each compiler-core change, keep `check`, `run`, and existing samples behavior-compatible.

Useful validation commands:

```bash
cargo test
cargo run -- check samples/println_sum.muga
cargo run -- samples/println_sum.muga
cargo run -- samples/packages/app/main/main.muga
```
