# Practical Language Readiness

Status: design and prioritization note. This is not a language specification.

Purpose: record what Muga still needs before it feels practical for real programs, and record what should stay out of the language so implementation pressure does not pull Muga away from its core direction.

Read this after [ROADMAP.md](../ROADMAP.md). The roadmap remains the source of truth for the active implementation slice. This document explains the language-feature backlog that should follow once the v1 package/artifact workflow is stable.

## Baseline

Current Muga is already computationally expressive enough for ordinary algorithms:

- functions, anonymous functions, closures, direct recursion, and mutual recursion
- `if`, `while`, final-expression function bodies, and local `mut`
- records, value-returning updates, List, Map, Option, Result, enum, exhaustive match, and `try expr` `Result` propagation
- package/module boundaries, visibility, package interfaces, and explicit artifact workflows

That is enough to treat the language core as computationally complete in the ordinary abstract sense. It is not enough to call the language broadly practical yet. The missing work is mostly around reusable abstraction, standard-library surface, IO/resource effects, project/dependency workflow, performance, and tooling.

## Priority Order

### 0. Finish the v1 package/artifact foundation

Keep the current roadmap priority first:

- harden explicit `.mgi` / `.mgc` / `.mgb` artifact workflows
- keep `--artifact-root` explicit and fail loudly on missing or stale artifacts
- improve diagnostics and samples around dependency-body-free `check` and `run`
- avoid starting broad language-surface work while package execution semantics are still moving

Reason: practical standard libraries and reusable packages depend on stable package interfaces and implementation artifacts. Building many surface features before that boundary is stable will create churn.

### 1. Harden user-defined generic records and functions

The first generic records/functions slice is implemented. The next work is hardening: examples, docs, and diagnostics for the explicit type-parameter model.

Recommended shape:

```muga
record Box[T] {
  value: T
}

fn id[T](value: T): T {
  value
}
```

Rules to preserve:

- type parameters are explicit on declarations
- ordinary unannotated functions are not implicitly generalized
- call sites use local type-argument inference when possible
- package interfaces store resolved generic public signatures
- no bounds, protocols, typeclasses, higher-kinded types, specialization, or polymorphic recursion in the first implementation

Reason: users can now write small reusable libraries on top of builtin `List[T]`, `Option[T]`, `Result[T, E]`, and `Map[K, V]`; the remaining risk is making the behavior obvious and stable at package boundaries.

### 2. Harden `try expr` for Result propagation

`Result[T, E]`, exhaustive `match`, and prefix `try expr` are the current semantic base. The next work is not more error syntax; it is making practical fallible APIs return `Result` consistently.

Recommended shape:

```muga
fn load_age(path: String): Result[Int, String] {
  text = try read_file(path)
  parse_int(text)
}
```

Current rules are conservative:

- `try expr` works only when `expr` has type `Result[T, E]`
- the enclosing function must return `Result[U, E]`
- propagated error types must match exactly at first
- do not make `try` work for `Option[T]` in the first version
- do not add postfix `?`
- do not add implicit exceptions or `throws`

Reason: `try` makes early return visible at the expression site and fits Muga's preference for explicit control flow.

### 3. Grow a small practical standard library

The next practical bottleneck is not syntax. It is the absence of ordinary APIs.

Prioritize packages in this order:

1. `std::string`: slicing or substring, trim, split, contains, starts/ends checks, replace, parse helpers where appropriate.
2. `std::fmt` or equivalent formatting helpers, without adding broad implicit conversion.
3. `std::fs` and `std::io`: read/write files, stdout/stderr handles, basic directory operations.
4. `std::time`, `std::env`, and `std::process`: enough for CLI tools.
5. `std::json`: parse and encode through explicit data types and `Result`.
6. `std::http`: only after resource handles, Result ergonomics, and package workflow are stable.

Recommended API style:

```muga
result: Result[String, IOError] = fs::read_text(path)
```

Use:

- `Result[T, E]` for recoverable effects
- `Option[T]` for absence
- value-returning updates for ordinary data
- builder/buffer types for repeated construction
- resource/handle types for files, sockets, timers, and OS-backed state

Avoid:

- implicit throwing exceptions
- property access with hidden IO
- dynamic `Any` as the normal interop path
- global mutable runtime state as the default API style

### 4. Improve loops, iteration, and collection ergonomics

`while` is enough for core expressiveness, but practical code needs more readable loops.

Recommended order:

1. `break` and `continue` for `while`.
2. A simple `for item in list` form for `List[T]`.
3. `Bytes`, `StringBuilder`, and `Buffer` for practical IO and text assembly.
4. `Set[T]` and broader List/Map operations.
5. Map literals only after the parser shape is settled.

Possible map literal shape if added later:

```muga
ages = map {
  "Ada": 20
  "Grace": 30
}
```

Do not overload plain `{ ... }` for maps. Braces already carry block and record-literal meaning.

Delay iterator protocols until ordinary generics, collections, package interfaces, and standard-library examples show a concrete need.

### 5. Add project dependency workflow

After artifact workflows are reliable, make project mode practical:

- dependency declarations in `muga.toml`
- lockfiles
- package source roots and local path dependencies
- full incremental artifact reuse
- better cross-package diagnostics for stale interfaces and caches
- registry/archive/signing design only after local/project dependencies work

Reason: practical reuse requires dependable package loading and cache invalidation before it requires a public registry.

### 6. Build the performance path

The current VM is a reference backend. Practical performance needs compiler work more than surface syntax.

Recommended order:

1. control-flow-oriented MIR
2. efficient String/List/Map representations
3. copy elision and destructive-update lowering when a value is uniquely owned
4. escape analysis and stack allocation for non-escaping values
5. inlining and specialization for hot generic or higher-order functions
6. native backend after package and MIR boundaries are stable

Keep source-level value semantics. Do not introduce `ref T`, `mut ref T`, `&value`, pointer syntax, or borrowing syntax as the ordinary performance answer.

### 7. Add structured concurrency later

Concurrency is important for practical services, but it should not be first in the language-feature queue.

Recommended first shape:

```muga
group {
  user_task = spawn fetch_user(id)
  orders_task = spawn fetch_orders(id)

  Page {
    user: user_task.join()
    orders: orders_task.join()
  }
}
```

Rules to preserve:

- child tasks cannot outlive their `group`
- `join()` is explicit
- failure and cancellation are structured
- immutable captures are easy
- mutable captures across task boundaries are rejected or made explicit
- typed channels come after task groups
- do not make `async fn` / `await` the primary model unless later evidence forces that direction

## Features To Keep Out

These should not be implemented for v1, and should not be added later without concrete examples, benchmarks, and package-interface impact analysis.

### Classes and inheritance

Do not add `class`, class-owned methods, instance variables, constructors tied to classes, or inheritance.

Use records for data, functions for behavior, modules for encapsulation, and chained-call syntax for call-site ergonomics.

### Ordinary source-level references

Do not add:

- `ref T`
- `mut ref T`
- `&value`
- `*value`
- pointer arithmetic
- general writable aliases

Use value semantics, internal sharing, builders/buffers, resource handles, and backend optimizations instead.

### Universal null or implicit nullable types

Do not make every `T` implicitly nullable.

Use `Option[T]` for absence. `T?` may remain future shorthand for `Option[T]`, but it should not become a separate nullable type.

### Implicit exceptions

Do not add exception-style control flow as the default recoverable-error model.

Use `Result[T, E]`, exhaustive `match`, and `try expr`.

### Broad protocol/trait/typeclass system in v1

Do not add protocol-like abstractions before generics, enums, collections, package interfaces, and standard-library examples make a real need clear.

If this family is added later:

- prefer the name `protocol`
- keep protocol declarations small
- do not add protocol inheritance, blanket implementations, specialization, protocol objects, or protocol-based dot lookup in the first version

### Overloaded dispatch and operator overloading

Do not add overloaded functions or user-defined operators in v1.

Reason: Muga currently has simple name resolution and stable dot-call meaning. Overloading would make diagnostics, package interfaces, and compile-time behavior more complex.

### Whole-program inference

Do not infer public API meaning from arbitrary downstream call sites.

Allowed:

- infer locally inside a function, module, or package when unique
- store resolved public signatures in package interfaces

Disallowed:

- making a dependency's public signature depend on downstream usage
- implicitly generalizing ordinary declarations into generic functions

### Hidden async suspension

Do not make ordinary calls hide suspension points.

Concurrency should start with structured task scopes and explicit `join()`, not with a second async-colored function world.

### Runtime metaprogramming as a core mechanism

Do not rely on reflection, monkey patching, or dynamic runtime code generation as the normal abstraction path.

Compile-time generation can be considered later, but it should not define the v1 core.

## Reconsideration Rule

Before adding a feature outside the priority order, require all of the following:

1. A concrete user-facing program becomes materially simpler or safer.
2. The feature preserves local readability.
3. The feature has a small parser and typechecker story.
4. The feature can be represented in package interfaces.
5. The feature does not require whole-program inference.
6. The feature does not overload an existing syntax marker with an unrelated meaning.
7. The feature has diagnostics that can be explained clearly.
8. The feature does not undermine value semantics or structured concurrency safety.

When in doubt, prefer a library API, package convention, or explicit function over a new language feature.
