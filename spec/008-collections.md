# Collections Draft

Status: design draft only. This document is not implemented in the Rust compiler yet.

This draft defines the recommended direction for Muga collections before implementing generics, collection literals, dictionary-like maps, or collection APIs.

## 1. Design Goals

Muga collections should support:

- simple and readable source code
- static typing with minimal annotations
- fast parsing and type checking
- immutable-by-default programming
- practical web-development use cases
- a future path to efficient native code generation

The smallest useful collection surface should be stabilized before adding a broad standard library.

## 2. Recommended Phase Order

The recommended order is:

1. local binding type annotations
2. simple generic type syntax for collection types
3. `List[T]`
4. `Option[T]`
5. `Map[K, V]`
6. later collection extensions such as `Set[T]`, fixed arrays, bytes, builders, and map literals

This order keeps the first implementation small.

`List[T]` should come before `Map[K, V]` because list literals and homogeneous sequences are simpler to type and easier to lower.

`Option[T]` should come before or alongside `Map[K, V]` because safe map lookup naturally returns either a value or no value.

## 3. Local Binding Type Annotations

Muga prefers inference, but collection literals need an expected type in some cases.

Target syntax:

```muga
numbers: List[Int] = []
mut names: List[String] = []
```

The annotation belongs to the binding, not to a `let` keyword. Muga still does not introduce `let`.

This syntax is especially useful for empty collections, because `[]` alone does not provide an element type.

## 4. Generic Type Syntax

Collection types should use square-bracket type arguments:

```muga
List[Int]
Map[String, Int]
Option[User]
```

The same syntax can later extend to user-defined generic records and functions:

```muga
record Box[T] {
  value: T
}

fn id[T](value: T): T {
  value
}
```

Generic declarations are not part of the first collection implementation. The immediate need is source type expressions such as `List[Int]` and `Map[String, Int]`.

## 5. List

`List[T]` is the recommended first collection type.

It represents an ordered, homogeneous, dynamically sized collection.

Examples:

```muga
numbers = [1, 2, 3]
more = numbers.push(4)
first = more[0]
count = more.len()
```

Typing rules:

- all elements in a list literal must have the same type
- `[1, 2, 3]` has type `List[Int]`
- `["a", "b"]` has type `List[String]`
- `[]` requires an expected type
- indexing uses an `Int` index
- indexing returns the element type

Examples:

```muga
numbers = [1, 2, 3]       // List[Int]
names = ["Ada", "Muga"]   // List[String]
empty: List[Int] = []
```

The default API should be value-oriented and non-destructive:

```muga
more = numbers.push(4)
changed = more.set(0, 10)
```

This fits Muga's immutable-by-default design. The implementation may later optimize this with copy-on-write, builder APIs, or uniqueness analysis, but those optimizations should not change the source-level meaning.

Recommended initial operations:

- `len(self: List[T]): Int`
- `is_empty(self: List[T]): Bool`
- `push(self: List[T], value: T): List[T]`
- `set(self: List[T], index: Int, value: T): List[T]`
- `get(self: List[T], index: Int): Option[T]`

Index syntax:

```muga
value = numbers[0]
```

Direct indexing should be bounds-checked. A failed bounds check is a runtime error unless Muga later introduces a different checked-indexing policy.

Safe lookup should use `get` and return `Option[T]`.

## 6. Option

`Option[T]` represents either a value or no value.

It is needed for collection APIs such as:

```muga
numbers.get(0)        // Option[Int]
users.get("ada")      // Option[User]
```

The exact syntax for constructing and handling `Option[T]` should be decided with enum or sum-type design. Until then, this draft only reserves `Option[T]` as the recommended return shape for safe lookup.

## 7. Map

`Map[K, V]` is the recommended dictionary/hash type.

It is needed for practical code, especially:

- JSON-like data
- HTTP headers
- query parameters
- caches
- lookup tables
- grouping data by key

Examples:

```muga
ages: Map[String, Int] = Map.empty()
ages = ages.insert("Ada", 20)
age = ages.get("Ada")       // Option[Int]
```

Initial key types should be limited to simple built-in comparable/hashable types:

- `String`
- `Int`
- `Bool`

Arbitrary record keys should be deferred until Muga has a clear equality and hashing model.

Recommended initial operations:

- `empty(): Map[K, V]`
- `len(self: Map[K, V]): Int`
- `is_empty(self: Map[K, V]): Bool`
- `contains(self: Map[K, V], key: K): Bool`
- `insert(self: Map[K, V], key: K, value: V): Map[K, V]`
- `remove(self: Map[K, V], key: K): Map[K, V]`
- `get(self: Map[K, V], key: K): Option[V]`

Like `List[T]`, the default API should be non-destructive. Efficient internal representations can be optimized later.

## 8. Map Literals

Map literals should be deferred.

Reason:

- `{ ... }` is already used for blocks and record literals
- adding another brace-based expression too early increases parser and reader ambiguity
- `Map.empty()` plus `insert` is enough for the first implementation

If Muga later adds a map literal, it should use an explicit form rather than overloading plain braces.

Possible future syntax:

```muga
ages = map {
  "Ada": 20
  "Muga": 1
}
```

This syntax is not decided.

## 9. No Source-Level Symbols Or Atoms In v1

Muga should not add a Ruby-style symbol type in v1.

Recommended alternatives:

- use `record` when the shape is known
- use future enums or sum types for finite tags
- use `String` keys for external data, JSON-like data, headers, and user input
- use compiler-internal symbols only inside the implementation, not as a source-language feature

If a source-level interned key type is needed later, it should probably be named `Atom`, not `Symbol`, to avoid confusion with compiler symbol interning.

Possible future syntax:

```muga
status = #ok
```

This is intentionally deferred.

## 10. Deferred Collection Topics

The following should not block the first collection implementation:

- `Set[T]`
- fixed-size `Array[T, N]`
- `Bytes`
- tuple types
- map literals
- arbitrary record keys for `Map`
- collection comprehensions
- builder or mutable collection APIs
- equality and hashing protocols
- advanced generic functions

The immediate goal is a small, typed, useful collection core.
