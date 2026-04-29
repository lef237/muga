# Value Semantics And Internal Sharing Draft

Status: design draft.

This document records Muga's current direction for ordinary value passing, internal sharing, write-oriented APIs, and performance.

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
- write-oriented APIs: use explicit resource or builder types instead of general source-level references

## 4. No Planned `ref T`

Muga should not plan `ref T` as a normal language feature.

This is a stronger position than "not required for v1".

The current direction is:

- do not add `ref T` for ordinary read-only borrowing
- do not add `mut ref T` for ordinary write access
- do not add call-site address syntax such as `&value`
- do not expose pointer identity in ordinary Muga code

Reasons:

- it adds a second way to think about every function parameter
- it makes aliasing part of the source language
- writable references require exclusivity rules
- escaping references require lifetime or ownership rules
- references interact heavily with generics, closures, package interfaces, and structured concurrency
- the same performance goals can usually be met by internal sharing, copy elision, and better resource types

Use ordinary `T` when a function works with a value:

```txt
fn birthday(user: User): User {
  user.with(age: user.age + 1)
}
```

If a value is large, the compiler may still pass it internally by pointer or shared storage when that is not observable.

This means the source stays simple:

```txt
fn display_name(user: User): String {
  user.name
}
```

while the implementation may avoid copying `User`.

## 5. Write-Oriented APIs

Write-oriented APIs should not require general mutable references.

Muga should prefer one of three patterns.

### 5.1 Return A New Value

For ordinary data, return an updated value:

```txt
next = user.with(age: user.age + 1)
items = items.push(item)
```

If the old value is no longer used, MIR/native lowering may perform the update in place internally.

### 5.2 Use Builder Or Buffer Types

For repeated construction, use explicit builder-like values:

```txt
mut builder = StringBuilder.new()
builder = builder.push("hello")
builder = builder.push(" world")
builder.to_string()
```

For byte or text buffers, the same style should stay readable:

```txt
mut buf = Buffer.empty()
buf = buf.append("hello")
buf = buf.append(" world")
```

The source still uses value update, but the implementation can keep the builder storage mutable and uniquely owned internally.

### 5.3 Use Resource Or Handle Types

For external side effects, use resource handles:

```txt
writer = file_writer("out.txt")
writer.write("hello")
writer.flush()
```

Here `writer` represents an external resource. The side effect is part of the resource API, not a general `mut ref T` mechanism.

This keeps write effects readable without exposing arbitrary writable aliases.

## 6. Performance Model

Value semantics do not imply slow code.

The compiler and runtime should optimize ordinary values with internal representation choices such as:

- immediate scalar values for `Int` and `Bool`
- shared immutable storage for `String`
- shared immutable storage for future `List[T]`, `Map[K, V]`, and buffers
- resource handles for files, sockets, timers, and OS-backed state
- compact record layout in MIR/native backends
- stack allocation for non-escaping values
- copy elision when a value is moved directly into a call or binding
- scalar replacement for small records
- destructive update lowering when a value is uniquely owned
- structural sharing or copy-on-write-like lowering for `record.with(...)` when safe
- inlining and specialization for hot generic or higher-order functions

The performance goal should be:

- keep source semantics simple
- avoid unnecessary copies in MIR and native code
- measure allocation and copy behavior continuously

The main risk is a naive implementation that copies large aggregates repeatedly. That is an IR and backend problem, not a reason to expose pointer syntax in ordinary Muga code.

## 7. Top-Tier Compiled-Language Performance

This design can support performance competitive with fast mainstream compiled languages, but it does not guarantee it by syntax alone.

The direction is compatible with that goal because:

- ordinary source semantics do not force observable reference identity
- immutable data can be shared safely inside the runtime
- package interfaces can avoid rechecking unchanged dependencies
- typed HIR and MIR can avoid repeating semantic analysis
- value update syntax can lower to in-place mutation when uniqueness is known
- resource handles can model IO without general mutable references
- native code generation can choose efficient ABI-level passing conventions

To reach or exceed that performance class, Muga needs implementation work in these areas:

- fast package interfaces and build cache
- local incremental compilation
- low-allocation front end
- MIR with explicit locals, moves, calls, and control flow
- escape analysis and stack allocation
- efficient string/list/map representations
- copy elision and destructive update lowering
- fast native backend
- benchmark-driven decisions from the beginning

The syntax choice alone is not enough. The important point is that removing `ref T` from the surface language does not block these optimizations.

## 8. Compile-Time Model

Fast compilation is still a core goal.

The value-semantics direction helps compile speed because it avoids making full alias analysis, lifetime inference, or whole-program ownership inference mandatory for ordinary code.

Recommended compile-time approach:

- keep ordinary calls simple and value-semantics based
- use typed HIR to record resolved calls, binding identity, and expression types
- lower to MIR where moves, copies, local slots, resource operations, and evaluation order are explicit
- perform local escape analysis before advanced whole-program optimization
- use package interfaces so dependencies do not need to be rechecked repeatedly
- avoid source-level reference features that require global lifetime or alias reasoning

This keeps Muga open to high-performance implementation without making every program pay the complexity cost of a heavy borrow system.

## 9. Design Consequences

For v1 and near-term implementation:

- ordinary function parameters should continue to be immutable bindings
- `record.with(...)` should remain non-destructive
- record field assignment such as `user.name = "Ada"` should remain out of v1
- explicit reference syntax should remain out of ordinary source code
- `ref T`, `mut ref T`, and `&value` should not be planned features
- VM behavior may stay simple while MIR/native backend work improves performance

For collections:

- `List[T]`, `Map[K, V]`, and `String` should not be deeply copied on every ordinary pass
- their implementation should use shared storage or another efficient representation
- source-level mutation semantics must be decided before mutable collection operations are added

For write-oriented standard library APIs:

- prefer value-returning updates for ordinary data
- prefer builder/buffer types for repeated construction
- prefer resource/handle types for file, socket, process, timer, and OS-backed effects
- avoid general writable aliases as a language feature

For concurrency:

- immutable values may be shareable across tasks when their representation is safe
- resource handles must define their own send/share rules
- mutable aliases should not cross task boundaries because ordinary Muga should not expose them
- task capture rules should be specified before structured concurrency is implemented

## 10. Open Questions

Before this design is fully normative, decide:

- whether this document should be merged into the core language spec or remain a separate design note
- how large records are represented in MIR
- how `record.with(...)` is lowered for large records
- how future collection storage handles sharing, copying, and mutation
- how builder and buffer types express efficient repeated writes
- how resource handles express side effects, ownership, and task-safety
- whether aggregate equality remains explicit and limited, or expands beyond primitive equality
