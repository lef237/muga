# Mini Language Spec v1

This is the compact v1 overview. The split specifications in [spec/](./spec) are the detailed references; this file exists to show the whole language shape without duplicating every rule.

## Goals

Muga v1 prioritizes:

- simple local reading
- low syntactic overhead
- static typing with minimal annotations
- fast compiler architecture
- predictable package boundaries

The language is compiler-first. The current VM is a reference execution backend, not a separate semantics engine.

## Core Rules

Bindings are immutable by default:

```muga
x = 1
mut total = 0
total = total + 1
```

`x = e` is one syntactic form. Name resolution decides whether it introduces a new immutable binding or updates an existing mutable binding in the current function scope.

Rules:

- no `let`
- `mut x = e` introduces a new mutable binding
- `x = e` introduces an immutable binding when `x` is not already defined in the current scope
- `x = e` updates `x` when `x` resolves to a mutable binding in the same function
- updating an immutable binding is an error
- shadowing is prohibited
- nested blocks in the same function may update enclosing mutable bindings
- inner functions may read outer bindings but may not update them

## Syntax

Comments use `//`. Statements are separated by newlines.

Core expression and statement forms include:

- integer, boolean, and string literals
- `if` statements and value-producing `if` expressions
- `while` statements
- function declarations and anonymous functions
- record declarations, record literals, field access, and non-destructive record update
- ordinary calls and chained calls
- list literals and list indexing
- exhaustive `match` for compiler-known `Option[T]` and `Result[T, E]`
- package declarations and imports in package mode

Type annotations use `:`:

```muga
value: Int = 1
mut items: List[Int] = []
fn add(a: Int, b: Int): Int {
  a + b
}
```

Function types use `->`:

```muga
fn apply(value: Int, f: Int -> Int): Int {
  f(value)
}
```

Generic type expressions use square brackets:

```muga
List[Int]
Option[String]
Result[Int, String]
Map[String, Int]
```

The implementation currently supports generic type expressions for `List`, `Option`, `Result`, and `Map`. User-defined generic records/functions remain part of the v1 design target but are not implemented yet.

## Functions And Inference

Type annotations should be omitted when local inference can determine a unique type:

```muga
fn double(x) {
  x * 2
}
```

Annotations remain required when inference is ambiguous, recursive constraints need a stable starting point, or the current implementation has an explicit boundary such as public package signatures.

Function bodies produce their final expression. `return` is not required in v1.

Higher-order functions are supported:

```muga
fn apply(x: Int, f): Int {
  f(x)
}

fn main(): Int {
  apply(10, fn(n) {
    n + 1
  })
}
```

Local bidirectional inference covers selected higher-order cases. When a callback type cannot be inferred uniquely, write the function type.

## Records And Calls

Records are nominal data declarations:

```muga
record User {
  name: String
  age: Int
}
```

Record literals use the record name:

```muga
user = User {
  name: "Ada"
  age: 20
}
```

Field access and update:

```muga
name = user.name
older = user.with(age: user.age + 1)
```

Behavior is modeled with functions. Chained calls are surface syntax over function calls:

```muga
fn birthday(user: User): User {
  user.with(age: user.age + 1)
}

older = user.birthday()
```

`self` is only a conventional parameter name. v1 has no classes, inheritance, receiver overloading, or function-valued record fields.

## Collections And Enum-Like Standard Types

Implemented collection/error core:

- `List[T]`
- `Option[T]`
- `Result[T, E]`
- `Map[K, V]` for `Int`, `Bool`, and `String` keys

Examples:

```muga
numbers = [1, 2, 3]
first = numbers.get(0)

match first {
  Option::Some(value) => value
  Option::None => 0
}
```

```muga
result: Result[Int, String] = Result::Ok(1)

match result {
  Result::Ok(value) => value
  Result::Err(message) => 0
}
```

`Option[T]` is the canonical optional spelling. `T?` is reserved as possible future shorthand. `Result[T, E]` is explicit; possible future propagation sugar is documented as `try expr`, not postfix `?`.

User-defined enum declarations are the next planned implementation slice.

## Packages

Script mode is for standalone files. Package mode starts with `package` or is inferred from a nearby `muga.toml`.

```muga
package app::main

import util::numbers

fn main(): Int {
  1.numbers::inc_twice()
}
```

Package-mode visibility:

- unmodified top-level items are module/file-private
- `pkg` items are visible to sibling files in the same package
- `pub` items are importable from other packages

The current implementation still flattens packages internally before checking/execution. Typed HIR and in-memory package summaries already carry package item identity; persisted package interfaces and cache artifacts are future work.

## Value Semantics

Ordinary source code uses value semantics. APIs that update ordinary data should return new values, as `List.push`, `List.set`, `Map.insert`, `Map.remove`, and `record.with(...)` do.

The implementation may optimize with internal sharing, copy elision, builders, buffers, resource handles, MIR lowering, or native backend work. These optimizations must not change source-level meaning.

Explicit source-level references such as `ref T`, `mut ref T`, `&value`, `*value`, and raw pointer syntax are not planned for ordinary Muga code.

## Current Implementation Boundary

Implemented:

- parser/resolver/typechecker/HIR/bytecode/VM pipeline
- typed HIR foundation
- records, functions, local inference, closures, higher-order functions
- packages, module privacy, `pkg`, `pub`, import aliases, and minimal manifest project mode
- `List`, `Option`, `Result`, and the first `Map` slice
- in-memory package interface summaries for public records/functions

Not implemented:

- user-defined enum declarations
- user-defined generic records/functions
- public-signature inference for `pub fn`
- persisted package interface files and package caches
- dependency manifests, registries, and lockfiles
- MIR and native backend
- structured concurrency

## Detailed References

- [spec/001-core-language.md](./spec/001-core-language.md)
- [spec/002-name-resolution.md](./spec/002-name-resolution.md)
- [spec/003-typing.md](./spec/003-typing.md)
- [spec/004-functions.md](./spec/004-functions.md)
- [spec/005-records.md](./spec/005-records.md)
- [spec/006-packages.md](./spec/006-packages.md)
- [spec/008-collections.md](./spec/008-collections.md)
- [spec/009-generics.md](./spec/009-generics.md)
- [spec/010-references-draft.md](./spec/010-references-draft.md)
- [spec/011-value-semantics.md](./spec/011-value-semantics.md)
- [spec/012-protocols-deferred.md](./spec/012-protocols-deferred.md)
- [spec/013-enums-results.md](./spec/013-enums-results.md)
