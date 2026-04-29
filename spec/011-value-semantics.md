# Value Semantics And Internal Sharing Draft

Status: design draft.

This document records Muga's current direction for ordinary value passing, internal sharing, and future read-only borrowing.

## 1. Core Decision

Muga source code should use value semantics by default.

That means:

- expressions evaluate to values
- function calls bind evaluated argument values to parameter bindings
- parameters are immutable bindings
- assigning to a parameter is an error
- `record.with(...)` is non-destructive and returns a new record value
- ordinary source code should not expose pointer identity or implicit mutable aliasing

This does not require the compiler to physically copy every value.

The implementation may pass, borrow, share, move, or elide copies internally whenever the difference is not observable from source code.

Recommended wording:

```txt
Muga values have value semantics at the source level.
Function calls bind evaluated argument values to immutable parameter bindings.
The compiler may pass, borrow, share, move, or elide copies internally when doing so is not observable.
```

## 2. Current VM Behavior

The current VM is value-oriented.

Implementation observations:

- runtime values are represented as `Value`
- `Int` and `Bool` are immediate values
- `String` and `Record` are owned values inside `Value`
- `Function` values are already shared through `Rc`
- `LoadName` reads a binding value and pushes it onto the stack
- function calls pop evaluated argument values and bind them to immutable parameter bindings in a new function environment
- field access returns a cloned field value
- `record.with(...)` consumes the base record value, replaces selected fields in a local copy, and returns a record value

This is sufficient for the current interpreter/VM, but it is not the desired long-term performance model for large records, strings, lists, maps, or buffers.

## 3. Why Not Default Reference Semantics

Muga should not make "reference passing" the default source-level rule.

Reasons:

- reference semantics make aliasing part of the language model
- future mutable references would need exclusivity rules
- future concurrency would need capture and sharing rules
- source-level reference identity would complicate equality and diagnostics
- a default reference model can force the compiler to preserve identity that users should not need to observe

Immutability by default makes internal sharing safe in many cases, but that is an implementation strategy, not a reason to expose references as the default language semantics.

The preferred split is:

- source semantics: value semantics
- implementation strategy: share immutable storage internally when safe
- explicit future feature: non-escaping read-only `ref T`

## 4. Relationship To `ref T`

`ref T` is the preferred future syntax for explicit read-only borrowing.

It should not become the default calling convention.

It is also not required for the v1 implementation.

Muga can proceed with ordinary value semantics first, then add `ref T` later only if real examples show that explicit read-only borrowing is needed for performance, APIs, or diagnostics.

Use ordinary `T` when the function conceptually consumes or works with a value:

```txt
fn birthday(user: User): User {
  user.with(age: user.age + 1)
}
```

Use future `ref T` when the function only reads a large value and should make that contract visible:

```txt
fn display_name(user: ref User): String {
  user.name
}
```

The first `ref T` version, if implemented, should be:

- read-only
- non-escaping
- primarily allowed in parameter positions
- allowed in receiver-shaped first parameter positions
- usable without call-site address syntax

This keeps calls lightweight:

```txt
display_name(user)
```

not:

```txt
display_name(&user)
```

## 5. Performance Model

Value semantics do not imply slow code.

The compiler and runtime should optimize ordinary values with internal representation choices such as:

- immediate scalar values for `Int` and `Bool`
- shared immutable storage for `String`
- shared immutable storage for future `List[T]`, `Map[K, V]`, and buffers
- compact record layout in MIR/native backends
- stack allocation for non-escaping values
- copy elision when a value is moved directly into a call or binding
- scalar replacement for small records
- structural sharing or copy-on-write-like lowering for `record.with(...)` when safe
- inlining and specialization for hot generic or higher-order functions

The performance goal should be:

- keep source semantics simple
- avoid unnecessary copies in MIR and native code
- measure allocation and copy behavior continuously

The main risk is a naive implementation that copies large aggregates repeatedly. That is an IR and backend problem, not a reason to expose pointer syntax in ordinary Muga code.

## 6. Compile-Time Model

Fast compilation is still a core goal.

The value-semantics direction helps compile speed because it avoids making full alias analysis, lifetime inference, or whole-program ownership inference mandatory for ordinary code.

Recommended compile-time approach:

- keep ordinary calls simple and value-semantics based
- use typed HIR to record resolved calls, binding identity, and expression types
- lower to MIR where moves, copies, borrows, local slots, and evaluation order are explicit
- perform local escape analysis before advanced whole-program optimization
- use package interfaces so dependencies do not need to be rechecked repeatedly
- reserve `ref T` for explicit local borrow contracts rather than inferring pervasive reference behavior

This keeps Muga open to high-performance implementation without making every program pay the complexity cost of a heavy borrow system.

## 7. Design Consequences

For v1 and near-term implementation:

- ordinary function parameters should continue to be immutable bindings
- `record.with(...)` should remain non-destructive
- record field assignment such as `user.name = "Ada"` should remain out of v1
- raw pointer syntax should remain out of ordinary source code
- future `ref T` should be added deliberately, not implicitly
- VM behavior may stay simple while MIR/native backend work improves performance

For collections:

- `List[T]`, `Map[K, V]`, and `String` should not be deeply copied on every ordinary pass
- their implementation should use shared storage or another efficient representation
- source-level mutation semantics must be decided before mutable collection operations are added

For concurrency:

- immutable values may be shareable across tasks when their representation is safe
- mutable references should not cross task boundaries by default
- task capture rules should be specified before structured concurrency is implemented

## 8. Open Questions

Before this design is fully normative, decide:

- whether this document should be merged into the core language spec or remain a separate design note
- the exact first implementation boundary for `ref T`
- whether `ref T` appears in package interfaces in the first borrow release
- how large records are represented in MIR
- how `record.with(...)` is lowered for large records
- how future collection storage handles sharing, copying, and mutation
- whether aggregate equality remains explicit and limited, or expands beyond primitive equality
