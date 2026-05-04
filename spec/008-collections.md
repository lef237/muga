# Collections Draft

Status: design draft with implemented first slices. The Rust compiler implements local binding annotations, `List[T]` type annotations, list literals, empty-list expected-type checking, typed HIR/package-interface representation, bytecode lowering, VM list values, `len` / `is_empty` / `push` / `get` / `set`, direct list indexing, `Option[T]`, `Option::Some`, `Option::None`, exhaustive Option `match`, and the first `Map[K, V]` slice.

This draft defines the recommended direction for Muga collections on top of the v1 generics MVP.

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

1. local binding type annotations (implemented)
2. simple generic type syntax for collection types (implemented for type expressions)
3. `List[T]` type annotations, list literals, `len`, `is_empty`, and `push` (implemented)
4. `Option[T]` construction and exhaustive `match` (implemented)
5. safe list lookup with `get(self: List[T], index: Int): Option[T]` (implemented)
6. direct list indexing and value-returning `set` (implemented)
7. `Map[K, V]` with scalar keys and value-returning operations (implemented)
8. later collection extensions such as `Set[T]`, fixed arrays, bytes, builders, and map literals

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

Collection types use the square-bracket type arguments defined in [009-generics.md](./009-generics.md):

```muga
List[Int]
Map[String, Int]
Option[User]
```

The same syntax is also used by user-defined generic records and functions:

```muga
record Box[T] {
  value: T
}

fn id[T](value: T): T {
  value
}
```

Generics are part of the v1 target. The first collection implementation may still be phased so that generic type expressions and builtin generic collection types land before user-defined generic records and functions.

## 5. List

`List[T]` is the recommended first collection type.

It represents an ordered, homogeneous, dynamically sized collection.

Examples:

```muga
numbers = [1, 2, 3]
```

Typing rules:

- all elements in a list literal must have the same type
- `[1, 2, 3]` has type `List[Int]`
- `["a", "b"]` has type `List[String]`
- `[]` requires an expected type
- indexing uses an `Int` index
- direct indexing returns the element type

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

`len`, `is_empty`, value-returning `push`, safe lookup `get`, value-returning `set`, and index syntax are implemented.

Index syntax:

```muga
value = numbers[0]
```

Direct indexing is bounds-checked. A negative index or out-of-bounds index is a runtime error.

Safe lookup uses `get` and returns `Option[T]`. A negative index or an out-of-bounds index returns `Option::None`.

Value-returning update uses `set` and follows direct indexing bounds behavior:

```muga
updated = numbers.set(0, 10)
```

`set` returns a new `List[T]` value at the source level.

## 6. Option

`Option[T]` represents a value that may or may not exist.

It is not a collection. It is a small result type used when an operation can fail without being an exceptional runtime error.

Current construction syntax:

```muga
present: Option[Int] = Option::Some(1)
missing: Option[Int] = Option::None
```

Current consumption syntax is exhaustive `match`:

```muga
match present {
  Option::Some(value) => value
  Option::None => 0
}
```

`Option::None` needs an expected `Option[T]` type, usually from a binding annotation, function return type, parameter type, or branch expectation.

This is implemented as a compiler-known standard enum-like type. General user-defined enum declarations are still deferred, but this source spelling should remain compatible with that future direction.

The important point is that absence becomes part of the static type.

For example, direct indexing can fail if the index is out of bounds:

```muga
value = numbers[10]  // runtime bounds error if the index does not exist
```

Safe lookup should instead return `Option[T]`:

```muga
numbers.get(0)        // Option[Int]
users.get("ada")      // Option[User]
```

This forces the caller to handle both cases:

- there is a value
- there is no value

Why this matters:

- it avoids using `null` as a universal missing-value marker
- it lets the typechecker see that a lookup may fail
- it makes collection APIs safer without turning ordinary misses into runtime errors

`List.get` and `Map.get` are implemented and use the same `Option[T]` source-level representation.

### 6.1 Optional shorthand

`Option[T]` should remain the canonical source spelling for now.

Muga may later add `T?` as shorthand:

```muga
User?      // future shorthand for Option[User]
String?    // future shorthand for Option[String]
```

This is intentionally not part of the first collection implementation.

Reason:

- `Option[T]` is explicit and works before deciding the rest of the `?` syntax family
- `?` may also be useful for future error propagation or optional chaining
- keeping `T?` as future sugar avoids taking that syntax too early

If `T?` is added later, it should mean exactly `Option[T]`, not a separate nullable type.

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

Implemented initial operations:

- `Map.empty(): Map[K, V]`
- `len(self: Map[K, V]): Int`
- `is_empty(self: Map[K, V]): Bool`
- `contains(self: Map[K, V], key: K): Bool`
- `insert(self: Map[K, V], key: K, value: V): Map[K, V]`
- `remove(self: Map[K, V], key: K): Map[K, V]`
- `get(self: Map[K, V], key: K): Option[V]`

`Map.empty()` requires an expected `Map[K, V]` type, usually from a local binding annotation, function return type, parameter type, or surrounding call expectation:

```muga
ages: Map[String, Int] = Map.empty()
ages = ages.insert("Ada", 20)
```

Like `List[T]`, the default API is non-destructive at the source level. `insert` and `remove` return a new `Map[K, V]` value. Efficient internal representations can be optimized later.

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

## 9. Deferred Collection Topics

The following should not block the first collection implementation:

- `Set[T]`
- fixed-size `Array[T, N]`
- `Bytes`
- tuple types
- map literals
- arbitrary record keys for `Map`
- collection comprehensions
- builder or mutable collection APIs
- equality and hashing protocol-like abstractions
- advanced generic features such as bounds, typeclasses, higher-kinded types, and specialization

The immediate goal is a small, typed, useful collection core.
