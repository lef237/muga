# Collections Draft

Status: design draft with implemented first slices. The Rust compiler implements local binding annotations, `List[T]` type annotations, list literals, empty-list expected-type checking, typed HIR/package-interface representation, bytecode lowering, VM list values, `len` / `is_empty` / `push` / `get` / `set`, direct list indexing, `for item in list` iteration over `List[T]`, narrow `std::list` helpers, `Option[T]`, `Option::Some`, `Option::None`, exhaustive Option `match`, the first `Map[K, V]` slice, and narrow `std::map` key/value helpers.

This draft defines the recommended direction for Muga collections on top of the current generics MVP.

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
7. `for item in list` iteration for `List[T]` only (implemented)
8. `Map[K, V]` with scalar keys and value-returning operations (implemented)
9. narrow helper packages that do not require iterator abstractions or structural equality (implemented for `std::list` and `std::map`)
10. later collection extensions such as `Set[T]`, fixed arrays, and map
    literals; the narrow Bytes builder and codec foundation is tracked
    separately as active standard-library work

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

Generics are supported by the current language. The first collection implementation may still be phased so that generic type expressions and builtin generic collection types land before user-defined generic records and functions.

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

When the indexed value type is ambiguous, diagnostics should suggest annotating it as `List[T]`.

Safe lookup uses `get` and returns `Option[T]`. A negative index or an out-of-bounds index returns `Option::None`.

When a `get` receiver type is ambiguous, diagnostics should suggest annotating the receiver as `List[T]` or `Map[K, V]`.

Value-returning update uses `set` and follows direct indexing bounds behavior:

```muga
updated = numbers.set(0, 10)
```

`set` returns a new `List[T]` value at the source level.

The implemented `std::list` package adds ordinary helper functions:

- `list::map(items, f): List[U]`
- `list::filter(items, predicate): List[T]`
- `list::fold(items, initial, f): U`
- `list::any(items, predicate): Bool`
- `list::all(items, predicate): Bool`

Using package-qualified chained calls, assuming `list` is the visible helper package alias:

```muga
mapped = values.list::map(transform)
kept = mapped.list::filter(predicate)
total = kept.list::fold(0, add)
```

`list::map` and `list::filter` return new lists and preserve item order. `list::fold` processes items left-to-right. `list::any` and `list::all` return `Bool` and may stop once the result is known. These helpers are ordinary package functions; they do not introduce iterator abstractions or lazy views.

`List.contains` remains deferred because the current equality policy is scalar-only and does not define generic structural equality for list elements.

### 5.1 Collection And Range Maturity Target

The next eager helpers should be chosen from real programs, with `find`,
`position`, `reverse`, `concat`, `flat_map`, `take`, `drop`, and
comparator-based `sort_by` as the first candidates. Comparator parameters keep
ordering explicit without adding traits or overloaded comparison.

Muga also needs allocation-free integer range iteration if ordinary
numeric loops are in scope. Prefer a small builtin `Range` value constructed by
`range(start, end)` and accepted directly by `for` before adding range
punctuation or a general iterator abstraction. A range must not allocate a
`List[Int]` proportional to its length.

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

This is implemented as a compiler-known standard enum-like type. General user-defined enum declarations are also implemented for the current MVP shape, and this source spelling remains compatible with that ordinary enum model.

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
- Result propagation uses visible `try` syntax instead of postfix `expr?`; prefix `try expr` is implemented, while future dot-chain propagation should use `expr.try`
- keeping `T?` as future sugar avoids taking that syntax too early

If `T?` is added later, it should mean exactly `Option[T]`, not a separate nullable type.

### 6.2 Option helpers

Optional chaining is useful for field access and chained calls, but it is not enough for arbitrary `Option` value transformation.
The implemented `std::option` helper package provides the minimal value-helper set:

- `option::is_some(option)`
- `option::is_none(option)`
- `option::map(option, f)`
- `option::and_then(option, f)`
- `option::value_or(option, fallback)`

Using package-qualified chained calls, assuming `option` is the visible helper package alias:

```muga
(
  maybe_age
    .option::map(fn(age) { age + 1 })
)

(
  maybe_user
    .option::and_then(fn(user) { user.address })
)

(
  maybe_name
    .option::value_or("Guest")
)
```

The helper semantics are:

- `option::is_some(option)` returns `true` for `Some(_)` and `false` for `None`.
- `option::is_none(option)` returns `true` for `None` and `false` for `Some(_)`.
- `option::map(option, f)` applies `f` to the `Some` payload and preserves `None`.
- `option::and_then(option, f)` applies `f` to the `Some` payload when `f` itself returns `Option[U]`, and preserves `None`.
- `option::value_or(option, fallback)` unwraps `Some(value)` or returns `fallback` for `None`.
- these helpers transform an `Option` value; they do not return early from the enclosing function.
- `option::value_or` is an ordinary strict function call, so `fallback` is evaluated before the call. A lazy fallback should use a separately named helper such as future `option::value_or_else`.

This gives `Option` and `Result` similar value-chaining shapes while preserving their different meanings:

- `option::map` / `option::and_then` model absence-preserving transformation
- `result::map` / `result::and_then` model success-path transformation while preserving errors
- neither helper family is propagation syntax
- `try` syntax remains the only function-level propagation family, and only for `Result`; prefix `try expr` is implemented, while future dot-chain propagation should use `expr.try`

Do not automatically combine `Option` and `Result` in these helpers:

```muga
maybe_text.option::map(fn(text) { text.parse_int() })
```

has type `Option[Result[Int, String]]`, not `Result[Option[Int], String]`.
If code needs to invert or traverse that shape, use explicit `match` or a named helper such as future `option::traverse_result`.

### 6.3 Optional chaining direction

If Muga later adds optional chaining, it should belong to `Option`, not to `Result`.
The `?` syntax family should have one primary meaning: optional absence.

Candidate surface syntax:

```muga
user?.name
user?.address?.city
text?.trim()
```

The intended meaning is local value transformation, not early return from the enclosing function:

```muga
user?.name
```

desugars conceptually to:

```muga
match user {
  Option::Some(value) => Option::Some(value.name)
  Option::None => Option::None
}
```

For chained calls, Muga's existing method-like call syntax remains surface syntax over functions.
Therefore:

```muga
text?.trim()
```

means "if `text` is `Some(value)`, call the ordinary `trim(value)`-style chained function and wrap the result in `Option::Some`; otherwise return `Option::None`".

Flattening rule:

- if the selected field or chained call returns `U`, the optional chain segment returns `Option[U]`
- if the selected field or chained call already returns `Option[U]`, the optional chain segment returns `Option[U]`, not `Option[Option[U]]`
- preserving a nested `Option[Option[U]]` should require explicit `match` or an explicit helper

This keeps common optional access ergonomic:

```muga
user?.address?.city  // Option[City]
```

Non-goals for optional chaining:

- `?.` must not propagate `Result` errors
- `?.` must not return early from the enclosing function
- `try` must not work on `Option[T]`
- `expr?` must not become a second spelling for `try expr` or future `expr.try`
- `maybe_text?.parse_int()` should not implicitly become `Result[Option[Int], String]`; if `parse_int` returns `Result[Int, String]`, the direct optional-chain result is `Option[Result[Int, String]]`

If code needs to combine optional and fallible computations, prefer explicit `match` or future named helpers such as `option::traverse_result`.

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

Arbitrary record keys should be deferred. The current equality policy is scalar-only and does not define structural equality or hashing for records, enums, lists, maps, `Option`, or `Result`.

Implemented initial operations:

- `Map.empty(): Map[K, V]`
- `len(self: Map[K, V]): Int`
- `is_empty(self: Map[K, V]): Bool`
- `contains(self: Map[K, V], key: K): Bool`
- `insert(self: Map[K, V], key: K, value: V): Map[K, V]`
- `remove(self: Map[K, V], key: K): Map[K, V]`
- `get(self: Map[K, V], key: K): Option[V]`
- `map::keys(self: Map[K, V]): List[K]`
- `map::values(self: Map[K, V]): List[V]`

`Map.empty()` requires an expected `Map[K, V]` type, usually from a local binding annotation, function return type, parameter type, or surrounding call expectation:

```muga
ages: Map[String, Int] = Map.empty()
ages = ages.insert("Ada", 20)
```

Like `List[T]`, the default API is non-destructive at the source level. `insert` and `remove` return a new `Map[K, V]` value. Efficient internal representations can be optimized later.

When `insert` or `remove` receiver types are ambiguous, diagnostics should suggest annotating the receiver as `Map[K, V]`. When `contains` is ambiguous between `String.contains` and `Map.contains`, diagnostics should suggest annotating the receiver as `String` or `Map[K, V]`.

The implemented `std::map` package provides `map::keys` and `map::values` as ordinary package functions:

```muga
keys = ages.map::keys()
values = ages.map::values()
```

Both helpers allocate new lists at the source level and return entries in the map's deterministic entry order. Inserting a new key appends that key to the order; replacing an existing key updates the value without moving the key.

`map::entries` should be added once the public shape below is validated in real
code; it does not require structural equality or hashing:

```muga
pub record Entry[K, V] {
  key: K
  value: V
}
```

The VM currently preserves insertion order with a linear entry vector. Before
the representation becomes stable, it should add a key-to-entry index or another measured representation so
normal lookup and update do not remain linear while deterministic iteration is
preserved.

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

The following remain deferred unless promoted by the roadmap decisions:

- `Set[T]`
- fixed-size `Array[T, N]`
- tuple types
- map literals
- arbitrary record keys for `Map`
- collection comprehensions
- equality and hashing constraint systems
- advanced generic features such as higher-kinded types and specialization

The promoted Bytes builder and opt-in derived equality/hash investigations are
separate narrow work. They must not be used to introduce a general mutable
collection API or behavior-constraint system by accident.
