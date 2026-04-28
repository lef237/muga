# References And Borrowing Draft

Status: design draft. This document is not implemented behavior yet.

This draft defines Muga's current direction for pointer-like and reference-like concepts.

The main decision is:

- Muga should not expose raw pointers in v1.
- Muga should reserve room for safe read-only borrowing through `ref T`.
- Muga should avoid `*T`, `*expr`, and `&expr` as ordinary source-level syntax.

The reason is not that pointers are useless internally. The reason is that Muga wants high performance without putting low-level pointer notation into everyday code.

## 1. Goals

Muga wants:

- simple code for beginners and experienced programmers
- strong static analysis
- fast compilation
- high runtime performance
- safe and readable concurrency
- low symbolic load

This means Muga should separate:

- user-facing ownership and borrowing syntax
- compiler/runtime internal pointer representation
- low-level escape hatches for possible future FFI or systems work

Raw pointer notation is not required for the first two.

## 2. Non-Goals For v1

The following are not part of v1:

- source-level raw pointer types such as `*T`
- source-level address creation such as `&x`
- source-level dereference such as `*x`
- pointer arithmetic
- nullable raw references
- storing borrowed references in records
- returning borrowed references from ordinary functions
- mutable references
- unsafe memory operations

These features either add substantial semantic weight, require lifetime rules, or conflict with Muga's syntax marker discipline.

## 3. Recommended Future Surface Syntax

If Muga adds safe borrowing, the preferred syntax is:

```muga
ref T
```

Examples:

```muga
fn display_name(user: ref User): String {
  user.name
}

fn total(data: ref BigData): Int {
  data.a + data.b + data.c + data.d
}

fn inspect(user: ref User, f: (ref User) -> String): String {
  f(user)
}
```

`ref T` means a read-only borrowed view of a `T`.

It does not mean:

- ownership transfer
- mutation permission
- nullable pointer
- heap allocation
- raw address access

## 4. `ref T` Versus `Ref[T]`

Muga should not use `ref T` and `Ref[T]` as two spellings for the same concept.

They mean different things:

- `ref T` is a language-level borrowed parameter type.
- `Ref[T]` would be an ordinary nominal generic type, if Muga ever defines one.

Recommended decision:

- use `ref T` for the first borrow feature
- do not introduce `Ref[T]` for ordinary borrowing
- reserve any future `Ref[T]`-like type for a separate, explicit managed-reference abstraction if it is ever needed

The reason is that `ref T` can stay lightweight and non-escaping:

```muga
fn display_name(user: ref User): String {
  user.name
}
```

By contrast, a first-class `Ref[T]` type suggests that references can be stored, returned, placed in records, captured by closures, or passed across tasks:

```muga
record Holder {
  user: Ref[User] // not the first borrow direction
}
```

That is a much larger feature. It requires clear answers for lifetime, ownership, aliasing, mutation, concurrency, and runtime representation.

If Muga later needs a first-class reference-like value, it should probably use a more specific name such as `Box[T]`, `Cell[T]`, `Shared[T]`, `Handle[T]`, or `Buffer`, depending on the semantics. A generic `Ref[T]` name is too broad for v1 and too easy to confuse with `ref T`.

## 5. Interaction With Generics

`ref T` should compose with generics, but it should not become an ordinary generic type application.

Good:

```muga
fn len[T](items: ref List[T]): Int {
  items.len()
}

fn render_each[T](items: ref List[T], render: (ref T) -> String): List[String] {
  // body omitted
}

fn inspect[T](value: ref T, f: (ref T) -> String): String {
  f(value)
}
```

In these examples:

- `T` is still an ordinary type parameter
- `List[T]` is still an ordinary generic type
- `ref List[T]` means "borrow a list"
- `ref T` means "borrow a value whose type is the type parameter `T`"
- type-argument inference can still infer `T` from the actual argument type

The first borrow version should not allow borrowed references to be stored inside generic containers:

```muga
List[ref User]        // deferred
Option[ref User]      // deferred
Map[String, ref User] // deferred
```

Reason:

- a borrowed value inside a generic container can escape the call
- this requires lifetime or ownership rules
- it complicates package interfaces and diagnostics

The preferred shape is to borrow the container or borrow each callback argument:

```muga
fn names(users: ref List[User], render: (ref User) -> String): List[String] {
  // body omitted
}
```

This keeps generics useful while preserving the non-escaping borrow model.

Recommended type grammar extension:

```ebnf
type_expr          := function_type
function_type      := function_domain "->" type_expr
                    | borrow_type
function_domain    := borrow_type
                    | "(" type_expr_list? ")"
borrow_type        := "ref" borrow_type
                    | non_function_type
non_function_type  := type_primary type_args?
type_primary       := "Int"
                    | "Bool"
                    | "String"
                    | IDENT
type_args          := "[" type_expr_list "]"
type_expr_list     := type_expr ("," type_expr)*
```

With this grammar:

- `ref List[T]` is valid
- `(ref T) -> String` is valid
- `ref T -> String` parses as `(ref T) -> String`
- `List[ref T]` can be parsed, but should be rejected by the first borrow checker as an escaping borrowed type

This is a type-system restriction, not a parser problem.

## 6. Call-Site Rule

Calls should stay light.

If a function expects `ref T`, a value of type `T` may be passed directly:

```muga
fn total(data: ref BigData): Int {
  data.a + data.b + data.c + data.d
}

stats = BigData {
  a: 1
  b: 2
  c: 3
  d: 4
}

total(stats)
```

The compiler may borrow `stats` for the duration of the call.

There is no ordinary source-level address operator:

```muga
total(&stats) // not Muga direction
```

The borrow should be explicit in the callee signature, not noisy at every call site.

## 7. Auto-Deref Rule

Muga should not require explicit dereference syntax for common reads.

Field access and chained calls on `ref T` should behave as if they read through the reference:

```muga
fn display_name(user: ref User): String {
  user.name
}

fn birthday(user: ref User): User {
  user.with(age: user.age + 1)
}
```

The following style should not be required:

```muga
fn display_name(user: ref User): String {
  (*user).name // not Muga direction
}
```

The auto-deref rule should be deliberately small:

- field access may read through `ref T`
- chained calls may use the referenced `T` as the receiver
- ordinary calls may pass `ref T` where `ref T` is expected

Muga should not add broad implicit dereference coercions until the exact safety and diagnostics model is clear.

## 8. Initial Restrictions

The first implementation of `ref T`, if added, should be non-escaping.

Allowed in the first version:

- function parameter types
- receiver-shaped first parameter types
- function type parameter positions

Examples:

```muga
fn render(user: ref User): String {
  user.name
}

fn apply_to_user(user: ref User, f: (ref User) -> String): String {
  f(user)
}
```

Deferred until Muga has a lifetime or ownership model:

```muga
record Holder {
  user: ref User // deferred
}

fn get_user_ref(user: ref User): ref User {
  user // deferred
}

saved = user.borrow() // deferred
```

This restriction keeps the first borrow model useful for performance without forcing Muga to design full lifetime syntax immediately.

## 9. Mutable References

`mut ref T` is not part of the initial design.

If Muga adds it later, it must come with strict rules:

- a mutable reference must be exclusive while active
- immutable reads and mutable writes cannot alias unsafely
- mutable references cannot cross task boundaries by default
- mutation through a reference must be clear in the function signature
- diagnostics must explain why a borrow is rejected

Possible future syntax:

```muga
fn fill(buffer: mut ref Buffer, value: Int): Unit {
  buffer.write(value)
}
```

This is intentionally deferred because it affects:

- alias analysis
- concurrency safety
- MIR lowering
- diagnostics
- standard library design

## 10. Raw Pointers And Unsafe Code

Raw pointers should not be part of ordinary Muga.

If Muga eventually needs FFI or systems-level escape hatches, raw pointer operations should be isolated behind an explicit unsafe boundary, not mixed into normal application code.

Possible future direction:

- `unsafe` package or block
- FFI-only pointer types
- no implicit conversion from safe `ref T` to raw pointer
- no pointer arithmetic in safe code

This preserves Muga's everyday readability while leaving a path for low-level interop if it becomes necessary.

## 11. Performance Model

Not exposing raw pointer syntax does not imply slow code.

The compiler and runtime may still use pointers internally for:

- passing large records without copying
- representing strings, lists, maps, and buffers
- sharing immutable collection storage
- stack allocation
- escape analysis
- scalar replacement
- copy elision
- in-place lowering when a value is uniquely owned

For Muga, the high-performance path should be:

1. keep value semantics at the source level where possible
2. use `ref T` for read-only access to large values
3. lower records and collections into efficient internal representations
4. optimize `record.with(...)` to avoid unnecessary full copies
5. inline or specialize higher-order functions where possible
6. use typed HIR and MIR so later stages do not redo semantic work
7. measure allocation, copy, and compile-time costs continuously

The key risk is not "no pointer syntax".

The key risk is a naive implementation that:

- copies large records repeatedly
- boxes every function value
- allocates every intermediate value on the heap
- implements immutable updates as full deep copies
- makes collections copy eagerly

Those are implementation and IR design problems, not reasons to expose raw pointers in the surface language.

## 12. Go-Level Or Better Performance Target

Muga can target Go-level or better performance with this direction, but syntax alone cannot guarantee it.

The decisive factors are:

- allocation behavior
- escape analysis
- stack vs heap placement
- data layout
- collection representation
- scheduler design for concurrency
- package interfaces and build cache
- native backend quality
- benchmark discipline

Compared with a GC-centered implementation, Muga may have opportunities to win in workloads where:

- allocation pressure is lower
- temporary values are optimized away
- immutable data can be shared cheaply
- hot functions are inlined or specialized
- package interfaces avoid rechecking dependency bodies

But Muga should treat "Go-level or better" as a benchmark target, not as a claim guaranteed by syntax.

## 13. Relation To Syntax Marker Discipline

This draft follows Muga's syntax marker discipline:

- `ref` is a word, not an overloaded punctuation marker
- `*` remains available for multiplication
- `&` does not need to mean both address creation and bit operations
- field access remains `expr.name`
- chained calls remain `expr.name(...)`
- type annotations still use `:`

This avoids making one symbol carry unrelated meanings such as:

- pointer type constructor
- dereference operation
- address creation
- multiplication
- bitwise operation
- mutation marker

## 14. Examples

### Reading A Large Record

```muga
record Stats {
  a: Int
  b: Int
  c: Int
  d: Int
}

fn total(stats: ref Stats): Int {
  stats.a + stats.b + stats.c + stats.d
}
```

### Non-Destructive Update

```muga
record User {
  name: String
  age: Int
}

fn birthday(user: ref User): User {
  next_age = user.age + 1
  user.with(age: next_age)
}
```

### Higher-Order Function With Borrowed Input

```muga
fn inspect_user(user: ref User, f: (ref User) -> String): String {
  f(user)
}

inspect_user(user, fn(u: ref User): String {
  u.name
})
```

### Chained Style

```muga
user.birthday().display_name()
```

If `birthday` accepts `ref User` and returns `User`, the chain remains readable while the compiler can avoid unnecessary copies.

## 15. Open Decisions

Before implementing `ref T`, decide:

- exact type grammar for `ref T` and `(ref T) -> U`
- whether `ref T` is allowed in local variable annotations in the first version
- exact non-escaping rule
- exact auto-deref boundaries
- whether record update may borrow fields internally
- how `ref T` appears in package interfaces
- how diagnostics explain borrow failures
- how `ref T` interacts with future task spawning

Do not implement `mut ref T` until those questions are stable.

## 16. External References

These references are not prescriptions for Muga, but they inform the trade-offs:

- [Rust Reference: pointer types](https://doc.rust-lang.org/reference/types/pointer.html)
- [Rust Book: references and borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html)
- [Go compiler README](https://go.dev/src/cmd/compile/README)
- [Go garbage collector guide](https://go.dev/doc/gc-guide)
- [Swift language guide: structures and classes](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/classesandstructures/)
- [Cranelift](https://cranelift.dev/)
