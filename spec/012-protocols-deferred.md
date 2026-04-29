# Protocol-Like Abstractions Decision Note

Status: deferred decision note. This is not implemented behavior.

This document records Muga's current position on traits, interfaces, protocols, typeclasses, and similar shared-behavior abstractions.

## 1. Decision

Muga v1 should not introduce trait, interface, protocol, or typeclass declarations.

The v1 direction is:

- keep one ordinary function namespace
- keep receiver-style calls as surface syntax over ordinary functions
- keep generic functions small and explicit
- keep package interfaces simple and cache-friendly
- use higher-order functions when behavior should be passed as a value
- use enum or sum-type design for closed sets of variants
- revisit protocol-like abstractions only after enum, match, collections, and package interfaces are stable

If Muga later adds this family of features, the preferred name is `protocol`.

`interface` should stay available for compiler artifacts such as package interfaces.

`trait` should not be the first spelling because it tends to imply a larger feature set than Muga currently wants, such as bounds, default implementations, blanket implementations, specialization, or coherence rules.

## 2. Why v1 Does Not Need It

Protocol-like abstractions solve real problems, but they are not the next required mechanism for Muga.

The immediate Muga v1 needs are better served by:

- nominal records for data
- ordinary functions for behavior
- generic functions for reusable type-preserving code
- higher-order functions for passing behavior explicitly
- package-qualified functions for avoiding name collisions
- future enum or sum types for closed variant sets
- future `match` for readable case analysis

This is enough for many common APIs without introducing a separate behavior-conformance system.

Example using a higher-order function:

```muga
fn render_each[T](items: List[T], render: T -> String): List[String] {
  // body omitted
}

render_each(users, user_name)
render_each(posts, post_title)
```

The behavior is explicit: the caller passes the rendering function.

Example using a future closed sum type:

```muga
type Animal {
  Dog(Dog)
  Cat(Cat)
}

fn name(animal: Animal): String {
  match animal {
    Dog(dog) => dog.name
    Cat(cat) => cat.name
  }
}
```

This is the right shape when the set of cases is known and should be checked exhaustively.

## 3. Rationale

### 3.1 Preserve Simple Name Resolution

Muga v1 deliberately has no overloading.

This keeps calls cheap to resolve and easy to explain:

```muga
fn len(items: List[Int]): Int { ... }
fn len(text: String): Int { ... } // duplicate binding in v1
```

Adding protocol-like dispatch too early would reopen this decision.

The compiler would need to decide whether `value.len()` means:

- a visible ordinary function named `len`
- a package-qualified function
- a protocol requirement
- a default protocol implementation
- a dynamically dispatched operation

Muga should not add that ambiguity before typed HIR, package interfaces, and MIR are stable.

### 3.2 Keep Dot Calls Stable

Muga currently gives dot syntax only a few stable meanings:

- `expr.name` is field access
- `expr.name(...)` is chained call surface syntax
- `expr.alias::name(...)` is package-qualified chained call
- `expr.with(...)` is record update

Protocol-based method lookup would add another meaning to `expr.name(...)`.

That may be useful later, but it should not be mixed into v1 while Muga is still keeping dot expressions intentionally small.

### 3.3 Keep Generic v1 Small

The v1 generics MVP excludes:

- trait bounds
- protocol bounds
- typeclasses
- specialization
- overloaded generic dispatch
- implicit polymorphic generalization

This keeps generic signatures and package interfaces easier to store, hash, and reuse.

Adding protocol-like bounds would make generics more powerful but also larger:

```muga
fn show[T: ToText](value: T): String {
  to_text(value)
}
```

This shape may be useful later. It should not be part of the first generics implementation.

### 3.4 Preserve Fast Package Interfaces

Muga wants fast separate compilation.

Package interfaces should initially store:

- public type names
- public function signatures
- public record shapes and field visibility
- generic signatures

Protocol-like abstractions would add more interface data:

- protocol declarations
- conformance records
- conformance visibility
- conflict rules
- dispatch strategy
- possible default implementations

Those are manageable, but they are not free. They should wait until the simpler package interface format exists.

### 3.5 Avoid Premature Dynamic Dispatch

A protocol-like type can mean different implementation strategies:

- static generic dispatch
- dictionary-passing
- runtime vtable dispatch
- boxed existential values

Each strategy affects performance, allocation, package interfaces, and diagnostics.

Muga should not choose one before there are concrete standard-library needs and benchmarks.

## 4. What To Use Instead

Use these first.

### 4.1 Named Functions

Prefer explicit names when behavior is specific:

```muga
fn open_text_file(path: String): TextFile { ... }
fn open_image_file(path: String): ImageFile { ... }
```

If a common operation is useful, write an explicit wrapper:

```muga
fn open_user_file(file: UserFile): OpenedFile {
  match file {
    Text(path) => open_text_file(path)
    Image(path) => open_image_file(path)
  }
}
```

This avoids overload resolution and keeps compile-time behavior obvious.

### 4.2 Higher-Order Functions

Use function parameters when the operation should be caller-defined:

```muga
fn apply[T, U](value: T, f: T -> U): U {
  f(value)
}
```

### 4.3 Enum Or Sum Types

Use a future enum or sum type when different concrete values should be treated as one closed family:

```muga
type Document {
  Text(TextDocument)
  Image(ImageDocument)
}
```

This is preferable when the set of alternatives is known and exhaustive checking matters.

### 4.4 Package Qualification

Use package aliases to keep names clear:

```muga
text::open(path)
image::open(path)
```

This allows packages to use natural local names without forcing global overloading.

## 5. Future Reconsideration

Muga should reconsider protocol-like abstractions when at least one of these becomes painful:

- standard library APIs need `Eq`, `Hash`, `Compare`, `ToText`, `Read`, `Write`, or iterator-like constraints
- generic collections need user-defined equality or hashing
- package-qualified functions become too verbose for common cross-type capabilities
- higher-order function parameters become repetitive boilerplate
- enum or sum types do not fit because the set of implementors must remain open

Even then, the first design should remain small.

Recommended future shape:

```muga
protocol ToText[T] {
  fn to_text(value: T): String
}
```

Potential future use:

```muga
fn show[T: ToText](value: T): String {
  to_text(value)
}
```

Do not include in the first version:

- automatic structural conformance
- default implementations
- blanket implementations
- protocol inheritance
- protocol objects
- dynamic dispatch by default
- protocol-based dot lookup
- user-defined overload sets

These features may be useful, but each one should justify itself separately.

## 6. Naming

If this feature family is added later, prefer `protocol`.

Reasons:

- `interface` already has a clear compiler meaning in Muga through package interfaces
- `trait` suggests a broad family of features that Muga may not want
- `protocol` reads as a behavior contract without implying class ownership
- the word fits Muga's function-centered model better than member-owned method terminology

This is only a naming preference. It is not permission to add the feature before the underlying need is proven.

## 7. Summary

Muga v1 should stay with data and functions:

- records define data
- functions define behavior
- higher-order functions pass behavior explicitly
- generic functions reuse logic across types
- future enum and match handle closed variant families
- package qualification handles naming boundaries

Protocol-like abstractions should remain deferred until real examples prove that these tools are not enough.
