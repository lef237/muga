# Typing Specification v1

Derived from [mini-language-spec-v1.md](../mini-language-spec-v1.md). This document defines the v1 typing policy, with emphasis on inference-first ergonomics and the limited cases where annotations are mandatory.

## 1. Typing Policy

The language prefers omission of type annotations.

- local bindings should infer their type from the right-hand side
- function parameter and return types should be inferred when the result is unique
- annotations are required only when inference cannot determine a unique type

## 2. Built-in Types and Source Type Expressions

The minimal v1 built-in types are:

- `Int`
- `Bool`
- `String`
- `Unit`

In addition, v1 introduces:

- user-defined nominal record types introduced by `record`
- source-level function types written with `->`
- generic type expressions written with `[]`

Therefore, source `type_expr` is:

```ebnf
type_expr          := function_type
function_type      := function_domain "->" type_expr
                    | non_function_type
function_domain    := non_function_type
                    | "(" type_expr_list? ")"
non_function_type  := type_primary type_args?
type_primary       := "Int"
                    | "Bool"
                    | "String"
                    | "Unit"
                    | IDENT
type_args          := "[" type_expr_list "]"
type_expr_list     := type_expr ("," type_expr)*
```

Examples:

- `Int -> Int`
- `(Int, String) -> Bool`
- `() -> Int`
- `List[Int]`
- `Map[String, Int]`
- `Option[User]`
- `Result[Unit, io::IOError]`

`Unit` has exactly one source value, written `()`. It is the preferred success value for effect-only fallible APIs such as future file writes, closes, and directory operations: `Result[Unit, E]`.

The v1 target includes a restricted generics MVP.

Examples:

```txt
record Box[T] {
  value: T
}

fn id[T](value: T): T {
  value
}
```

The current Rust implementation supports generic type expressions for compiler-known `List[T]`, `Option[T]`, `Result[T, E]`, and `Map[K, V]`, plus explicit user-defined generic records and functions. The generics MVP is specified in [009-generics.md](./009-generics.md).

## 3. Prelude Built-ins

The v1 prelude currently provides:

- `print`
- `println`
- `len`, `is_empty`, `push`, `get`, and `set` for `List[T]`
- `Map.empty`, `len`, `is_empty`, `contains`, `get`, `insert`, and `remove` for `Map[K, V]`
- `to_string` for `Int`, `Bool`, and `String`
- `is_empty`, `contains`, `trim`, `char_count`, `starts_with`, `ends_with`, `replace`, `split`, `concat`, `slice_chars`, `parse_int`, and `parse_bool` for `String`

`print` accepts exactly one argument of type `Int`, `Bool`, or `String`, writes its textual representation to standard output without a trailing newline, and returns that same value.

`println` accepts exactly one argument of type `Int`, `Bool`, or `String`, writes its textual representation to standard output as one line, and returns that same value.

`Int.to_string()`, `Bool.to_string()`, and `String.to_string()` return `String`. `to_string` is explicit and intentionally does not introduce implicit string conversion.

`String.is_empty()` returns `Bool`, `String.contains(needle)` returns `Bool`, `String.trim()` returns `String`, `String.char_count()` returns `Int`, `String.starts_with(prefix)` / `String.ends_with(suffix)` return `Bool`, `String.replace(old, new)` returns `String`, `String.split(separator)` returns `List[String]`, `String.concat(other)` returns `String`, `String.slice_chars(start, count)` returns `Result[String, String]`, `String.parse_int()` returns `Result[Int, String]`, and `String.parse_bool()` returns `Result[Bool, String]`. `String.char_count()` and `String.slice_chars(start, count)` count and index Unicode scalar values, not UTF-8 bytes or user-perceived grapheme clusters. `slice_chars` accepts zero-based `start` plus `count`; negative values or ranges beyond the string return `Result::Err("invalid slice range")`. `replace("", new)` returns the original string unchanged, and `split("")` returns a one-item list containing the original string.

`String.len()` is intentionally not part of this string-helper slice. Future length/indexing APIs should stay explicit: add `String.byte_len()` when bytes or I/O APIs need byte size, keep any range syntax or substring aliases aligned with `slice_chars` before adding them, and reserve grapheme-cluster APIs until the standard library has a Unicode segmentation dependency/versioning policy. Fallible string helpers currently return `Result[_, String]`; richer string-specific error records or enums should be introduced only after several string APIs need a shared error shape.

## 4. Standard Library Package Slice

The first compiler-provided standard packages are:

```muga
import std::fs
import std::io
```

`std::io` exports:

```muga
pub record IOError {
  operation: String
  path: String
  kind: String
  message: String
  raw_code: Option[Int]
}
```

`std::fs` exports:

```muga
pub fn read_text(path: String): Result[String, io::IOError]
pub fn write_text(path: String, text: String): Result[Unit, io::IOError]
```

`read_text` reads a UTF-8 text file into a `String`. `write_text` writes a `String` to a file and uses `Unit` as the success payload. Recoverable filesystem failures return `Result::Err(io::IOError)`. The current slice intentionally does not add resource handles, `Path`, binary `Bytes`, directory APIs, stdout/stderr handles, permissions APIs, or asynchronous IO.

Because `print` and `println` accept several concrete types, neither one by itself makes an unconstrained parameter uniquely inferable.

Example:

```txt
fn show_int(x) {
  print(x + 1)
}
```

This is valid because `x + 1` constrains the argument to `Int`.

By contrast:

```txt
fn show(x) {
  print(x)
}
```

still requires annotation in v1.

## 5. Higher-Order Functions

v1 supports higher-order functions.

Allowed in principle:

- passing a named function as an argument
- passing an anonymous function as an argument
- storing a function in a local binding

Example:

```txt
fn inc(x) {
  x + 1
}

fn apply(x: Int, f): Int {
  f(x)
}

apply(10, inc)
apply(10, fn(n) {
  n + 1
})
```

If a higher-order parameter is used in a way that determines a unique function type inside the same function body, its function-type annotation may be omitted.

Examples:

```txt
fn apply(x: Int, f): Int {
  f(x)
}

fn offset(x: Int, f) {
  f(x) + 1
}
```

By contrast, this remains ambiguous in v1:

```txt
fn apply(x, f) {
  f(x)
}
```

This also remains ambiguous in v1:

```txt
fn show(x: Int, f) {
  println(f(x))
}
```

because `println` accepts `Int`, `Bool`, or `String`, so the callback result type is not uniquely determined.

An explicit arrow annotation remains valid and useful:

```txt
fn show(x: Int, f: Int -> String): String {
  println(f(x))
}
```

## 6. Record Typing

For:

```txt
record User {
  name: String
}
```

`User` is a nominal type.

A record literal:

```txt
User {
  name: "Ada"
}
```

has type `User` if and only if:

- every declared field is provided exactly once
- no extra fields are present
- each field initializer has the declared field type
- every record field type must be a non-function type in v1

## 7. Field Access and Chained Call Typing

For field access:

```txt
expr.name
```

`expr` must have a record type that declares a field `name`. The expression type is the declared type of that field.

For chained call:

```txt
expr.name(arg1, arg2)
expr.alias::name(arg1, arg2)
```

the receiver expression `expr` is typed first.

Then:

1. if the callee is plain `name` and `name` resolves to a receiver-style function, the call is typed as a call of that function with `expr` as the first argument
2. otherwise, if the corresponding ordinary call is valid, the chained call is typed as that UFCS-style desugaring
3. otherwise, the expression is a type error

Because record fields may not have function type in v1, `expr.name(...)` and `expr.alias::name(...)` never mean a call through a function-valued field.

## 8. Record Update Typing

For:

```txt
expr.with(field1: value1, field2: value2)
```

the base expression `expr` must have a record type.

The expression type is the same record type as the base expression if and only if:

- every updated field name exists on that record type
- each replacement expression has exactly the declared type of that field
- no field name appears more than once in the same update

Unspecified fields are preserved from the base value.

The update is non-destructive. The result is a new record value rather than a mutation of the original record.

`expr.with(...)` is not typed as an ordinary chained call in v1.

## 9. Operator Typing Rules

The built-in operator typing rules are:

- unary `-` : `Int -> Int`
- unary `!` : `Bool -> Bool`
- `+`, `-`, `*`, `/` : `Int -> Int -> Int`
- `<`, `<=`, `>`, `>=` : `Int -> Int -> Bool`
- `==`, `!=` : allowed only for identical primitive types among `Int`, `Bool`, and `String`

String concatenation uses explicit `String.concat(other)`. The `+` operator remains `Int`-only.

## 10. Inference Sources

v1 inference may use:

- literal types
- operator constraints
- branch result agreement
- expected types from the surrounding expression inside the same function body
- explicit annotations already present in the same declaration
- explicit function-type annotations on parameters

v1 does not use call sites in other functions or modules as an inference source.

In future module or package boundaries, explicit function-type annotations are expected to remain the preferred interface style even when a local implementation might be inferable.

Examples:

```txt
x = 1          // Int
name = "m"     // String
```

```txt
fn inc(x) {
  x + 1
}
```

If `+` here is the integer addition operator in v1, `x` is inferred as `Int`.

## 11. Local Bindings

For a binding:

```txt
x = e
mut y = e
```

the binding type is inferred from the type of `e`.

For mutable bindings, every later update in the same scope must be type-compatible with the original inferred type.

Example:

```txt
mut total = 0
total = total + 1
```

`total` has type `Int`.

Mutable updates must preserve the original type exactly. v1 does not define implicit conversions or subtyping.

Local bindings may also hold function values.

Example:

```txt
inc = fn(x: Int): Int {
  x + 1
}
```

Collection literals and enum-like constructors sometimes need an expected type. Local binding annotations provide that type without introducing `let`.

Syntax:

```txt
items: List[Int] = []
mut names: List[String] = []
```

This is needed because an empty collection literal does not determine its element type by itself.

This syntax is implemented for local bindings and is used to give empty collection literals and `Option::None` an expected type.

## 12. Conditions and Branches

The condition expression of:

- `if`
- `while`

must have type `Bool`.

For an `if` expression, both branches must produce the same result type.

Example:

```txt
fn abs(n: Int) {
  if n < 0 {
    -n
  } else {
    n
  }
}
```

Both branches produce `Int`, so the `if` expression has type `Int`.

For an `if` expression, the branch result types must match exactly.

## 13. Function Parameter Inference

A parameter annotation may be omitted when the parameter type is uniquely determined from the function body and surrounding constraints.

Example:

```txt
fn double(x) {
  x * 2
}
```

If `*` is defined only for `Int` in v1, `x` is inferred as `Int`.

Inference fails when a parameter remains unconstrained.

Example:

```txt
fn id(x) {
  x
}
```

This requires annotation because the type of `x` is not uniquely determined.

For higher-order functions, parameter annotation is often the intended source of the function shape.

Example:

```txt
fn apply(x: Int, f: Int -> Int): Int {
  f(x)
}
```

## 14. Function Return Inference

The return type of a function is inferred from the final expression in the body.

When control flow branches, the return type is inferred from the unified branch result type.

If the body does not provide enough information to infer a unique return type, a return annotation is required.

## 15. Inference Boundary

v1 intentionally uses local-only inference.

Allowed:

- infer local binding types from the right-hand side
- infer function parameter types from operators and other constraints inside the same function body
- infer function return types from the function body
- infer `if` expression result types from branch agreement
- typecheck higher-order calls once explicit function-type annotations are present

Disallowed:

- inferring a callee parameter type from call sites alone
- propagating constraints across unrelated top-level declarations
- implicit polymorphic generalization of non-generic declarations
- inferring a complete higher-order parameter shape from distant call sites alone

This means:

```txt
fn inc(x) {
  x + 1
}
```

is valid, but:

```txt
fn id(x) {
  x
}
```

is not.

## 16. Mandatory Annotations

Annotations are required in the following cases:

1. a function parameter type is not uniquely inferable
2. a function return type is not uniquely inferable
3. a recursive function has neither an annotated parameter nor an annotated return type
4. a mutually recursive function participates in a recursive group without an explicit signature
5. a receiver parameter must have an explicit type annotation
6. a higher-order parameter shape is not uniquely inferable

For v1, an explicit function signature means:

- at least one parameter or the return type is annotated for direct recursion
- every function in a mutually recursive group has enough annotations to determine its full callable type before body checking

## 17. Direct Recursion Rule

For a directly recursive function, at least one of the following must be present:

- an annotation on one or more parameters
- an explicit return type annotation

Valid:

```txt
fn fact(n: Int) {
  if n == 0 {
    1
  } else {
    n * fact(n - 1)
  }
}
```

Also valid:

```txt
fn fact(n): Int {
  if n == 0 {
    1
  } else {
    n * fact(n - 1)
  }
}
```

Invalid:

```txt
fn fact(n) {
  if n == 0 {
    1
  } else {
    n * fact(n - 1)
  }
}
```

## 18. Mutual Recursion Rule

Mutually recursive functions require explicit signatures.

In v1, this means each function in the recursive group must carry enough annotations for its callable type to be known before any body in the group is checked.

Valid:

```txt
fn is_even(n: Int): Bool {
  if n == 0 {
    true
  } else {
    is_odd(n - 1)
  }
}

fn is_odd(n: Int): Bool {
  if n == 0 {
    false
  } else {
    is_even(n - 1)
  }
}
```

For implementation purposes, "explicit signature" for a mutually recursive group means that each function's full callable type is known before any body in the group is checked.
