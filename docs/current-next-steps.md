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
- `List[T]` first for collections, then `Option[T]`, then `Map[K, V]`
- no raw pointers in v1; future safe borrowing is drafted around read-only `ref T`
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
- generic declarations must declare their type parameters explicitly, such as `fn id[T](value: T): T`.
- `Option[T]` is the canonical spelling for optional values.
- `T?` is only reserved as possible future shorthand for `Option[T]`.
- `List[T]` means zero or more values.
- `Option[T]` means zero or one value.
- Empty list literals require an expected type such as `items: List[Int] = []`.
- Pointer-like syntax such as `*T`, `*expr`, and `&expr` should not be ordinary Muga source syntax.
- If safe borrowing is added later, the current draft prefers non-escaping read-only `ref T` before any mutable reference model.
- Ordinary source code should use value semantics.
- The implementation may share immutable storage internally when that is not observable.
- `ref T` is not required for v1 and should remain deferred until concrete performance or API pressure justifies it.

## 3. Recommended Next Implementation Task

The best next implementation task is still:

1. finalize call resolution data in typed HIR
2. make ordinary calls, chained calls, builtins, and package-qualified calls carry explicit resolved callee shape
3. keep existing VM behavior compatible while typed HIR becomes the compiler-facing semantic boundary

Why this comes next:

- typed HIR already exists
- calls are the largest remaining semantic gap in typed HIR
- later package interfaces and MIR should not redo callee resolution
- this supports future collections, generics, package interfaces, and native backend work

Expected result:

- a typed HIR call expression says which binding, builtin, or function value it calls
- parser-level call origin distinguishes ordinary calls from chained calls
- package-qualified calls currently point to flattened bindings and can later point to `PackageItemId`
- later lowering stages do not need to repeat resolver/typechecker logic

Completed work order for this task:

1. Add resolved call information to the typechecker output.
2. Preserve that call information in typed HIR call expressions.
3. Add tests for ordinary function calls, local function-value calls, builtin calls, and package-qualified calls.
4. Keep the existing VM bytecode path behavior-compatible.
5. Add AST-level call origin data so typed HIR can distinguish ordinary calls from chained calls instead of relying on parser desugaring.

Current implementation status:

- `TypeCheckOutput` exposes resolved call information.
- typed HIR `CallExpr` preserves the resolved callee.
- parser/AST call origin is threaded into typed HIR for ordinary calls, chained calls, and package-qualified chained calls.
- tests cover ordinary function calls, local function-value calls, builtin calls, chained calls, and package-qualified calls.
- the existing VM bytecode path remains behavior-compatible.

Remaining immediate follow-up:

1. When package interfaces are introduced, upgrade package-qualified call targets from flattened `BindingId` data to package-aware item identity.
2. Use the resolved callee and call-origin data as MIR/package-interface inputs rather than re-running call resolution later.

## 4. Decisions To Make Soon

These decisions affect near-term implementation and should be made before implementing the related feature.

### 4.1 Before collection implementation

Decide:

- exact grammar for local binding type annotations: recommended `name: Type = expr` and `mut name: Type = expr`
- how `Option[T]` values are constructed and consumed
- whether `match` or another pattern form is needed before exposing `Option[T]` broadly
- direct indexing policy: runtime bounds error for `xs[i]`, safe lookup through `xs.get(i)`

Current recommendation:

- implement local binding annotations and generic type expressions first
- parse generic type expressions as `Type[Arg1, Arg2]`
- parse generic declarations as `record Box[T]` and `fn id[T](value: T): T`
- implement generic records and generic functions as part of v1
- rely on local type-argument inference rather than explicit call-site type arguments in the v1 MVP
- defer bounds, typeclasses, higher-kinded types, const generics, and specialization
- implement `List[T]` and list literals before `Map[K, V]`
- keep `T?` reserved, not implemented
- do not implement map literals in the first collection slice

### 4.2 Before package interface implementation

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

### 4.3 Before concurrency implementation

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

### 4.4 Before enum / error handling design

Decide:

- enum or sum-type syntax
- pattern matching syntax
- whether `Option[T]` and future `Result[T, E]` are ordinary enums or compiler-known standard types
- whether `?` is reserved for optional shorthand, error propagation, optional chaining, or some combination

Current recommendation:

- keep `Option[T]` canonical
- reserve `T?` only as possible future shorthand
- do not spend `?` on multiple meanings until error handling is designed

### 4.5 Before reference / borrow implementation

Decide:

- exact type grammar for `ref T` and `(ref T) -> U`
- whether `ref T` is allowed only in parameter positions at first
- exact non-escaping rule
- exact auto-deref boundaries for field access and chained calls
- how borrowed values appear in package interfaces
- how borrow diagnostics explain lifetime or escape failures
- how `ref T` interacts with structured task spawning

Current recommendation:

- do not add source-level raw pointers
- do not add `*T`, `*expr`, or `&expr` as ordinary syntax
- add no explicit dereference operator in the first borrow design
- treat read-only `ref T` as the only near-term candidate
- defer `mut ref T` until aliasing and concurrency rules are specified

The draft lives in [spec/010-references-draft.md](../spec/010-references-draft.md).

## 5. What Not To Reopen Now

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
- package qualification uses `::`
- comments use `//`
- no source-level raw pointers in v1
- VM and compiler can coexist through a shared checked pipeline

## 6. Resume Checklist

When resuming implementation:

1. Run `cargo test`.
2. Read [ROADMAP.md](../ROADMAP.md) "Recommended Immediate Next Steps".
3. Read [docs/internal/identity-model.md](./internal/identity-model.md).
4. Start with typed HIR callee-shape finalization unless a language-design decision is explicitly being made first.
5. After each compiler-core change, keep `check`, `run`, and existing samples behavior-compatible.

Useful validation commands:

```bash
cargo test
cargo run -- check samples/println_sum.muga
cargo run -- samples/println_sum.muga
cargo run -- samples/packages/app/main/main.muga
```
