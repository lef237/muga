# Enums, Result, And Error Propagation Draft

Status: design draft with an implemented MVP. The current Rust compiler implements compiler-known `Option[T]` and `Result[T, E]`, plus user-defined `enum` declarations with optional unconstrained type parameters, zero-payload and one-payload variants, qualified constructors/patterns, exhaustive `match`, VM execution, typed HIR, and in-memory package interface summaries. AST/HIR/typed HIR match patterns use enum-variant-shaped internal data, runtime enum values use a generic enum-value representation, and variant facts are consumed by typechecking, bytecode lowering, and runtime branching. Error propagation syntax is not implemented yet.

## 1. Goals

Enums and result handling should support:

- explicit sum types for ordinary data modeling
- `Option[T]` as the standard optional-value type
- `Result[T, E]` as the standard recoverable-error type
- exhaustive `match`
- readable package interfaces
- fast local type checking
- future stdlib APIs for file, IO, process, time, HTTP, and concurrency

## 2. Non-Goals For The First Slice

The first slice should not include:

- wildcard-heavy pattern matching
- nested/destructuring patterns beyond one variant payload
- guards on match arms
- implicit conversions between enum variants and payload types
- exception-style control flow
- broad stdlib effect APIs
- overloading or protocol-based variant dispatch

## 3. MVP Source Syntax

The first user-defined enum slice uses this declaration shape:

```muga
enum Option[T] {
  Some(T)
  None
}

enum Result[T, E] {
  Ok(T)
  Err(E)
}
```

Grammar sketch:

```text
enum-decl    = visibility? "enum" Ident type-params? "{" enum-variant* "}"
type-params  = "[" Ident ("," Ident)* "]"
enum-variant = Ident | Ident "(" type-expr ")"
visibility   = "pub" | "pkg"
```

Variant declarations use the same boundary rule as record fields: newline or comma separates entries, and a trailing separator is accepted.

Variant construction uses qualified names:

```muga
present: Option[Int] = Option::Some(1)
missing: Option[Int] = Option::None

ok: Result[Int, String] = Result::Ok(1)
err: Result[Int, String] = Result::Err("missing")
```

Pattern matching uses the same qualified form:

```muga
match result {
  Result::Ok(value) => value
  Result::Err(message) => 0
}
```

This matches the source shape already implemented for compiler-known `Option[T]` and `Result[T, E]`.

## 4. MVP Variant Shape

The MVP supports:

- zero-payload variants, such as `None`
- one-payload variants, such as `Some(T)` or `Err(E)`
- generic enum declarations with unconstrained type parameters
- variants are namespaced under their enum type
- variant constructors are not imported into the local namespace unqualified
- enum values are nominal, not structural

Deferred:

- multi-payload tuple-like variants
- named-field variants
- per-variant visibility
- unqualified variant imports
- wildcard patterns
- nested patterns
- pattern guards
- recursive enum layout optimization
- derived equality/hash behavior

This MVP is enough to model `Option[T]` and `Result[T, E]`.

## 5. Typing Rules

For an enum declaration:

```muga
enum Result[T, E] {
  Ok(T)
  Err(E)
}
```

`Result[Int, String]` is a nominal instantiated enum type.

Variant constructor typing:

- `Result::Ok(value)` has type `Result[T, E]` when `value` has type `T` and `E` is known from expected type or local inference.
- `Result::Err(value)` has type `Result[T, E]` when `value` has type `E` and `T` is known from expected type or local inference.
- a zero-payload constructor such as `Option::None` requires an expected enum type when type arguments cannot be inferred.

Match typing:

- the matched value must have an enum type
- each arm pattern must belong to that enum type
- each variant must be covered exactly once in the MVP
- duplicate variant arms are errors
- missing variant arms are errors
- payload bindings are immutable local bindings inside the arm expression
- all arm result expressions must have the same type, or must match the surrounding expected type

Identity rules for the MVP:

- enum declarations are top-level items
- package-mode enum declarations receive `PackageItemId`, the same as public records and functions
- imported enum types and variants resolve through package interfaces, not through string reconstruction from flattened names
- a public enum exposes its name, type parameters, variants, and payload types in the package interface
- module-private and `pkg` enum visibility follows the same package visibility rules as records/functions

## 6. Relationship To Current Option

`Option[T]` is currently compiler-known. That can remain true internally during migration, but its source semantics should match ordinary enum behavior.

The implementation has moved the MVP toward one internal ADT model: match patterns are represented as enum variant patterns, runtime values use a generic enum value shape, and `Option` / `Result` match validation plus bytecode/runtime branching consult compiler-known enum metadata. Source enum declarations now consume the same runtime and match model, while compiler-known `Option[T]` and `Result[T, E]` remain special for prelude compatibility.

Compatibility requirements:

- `Option[T]` remains the canonical spelling.
- `Option::Some(value)` remains valid.
- `Option::None` remains valid.
- existing exhaustive Option `match` remains valid.
- `Result::Ok(value)`, `Result::Err(error)`, and exhaustive Result `match` remain valid.
- `T?` remains reserved and unimplemented.

## 7. Result Before Error Propagation Sugar

`Result[T, E]` should be implemented before any propagation operator.

Initial source usage should be explicit:

```muga
fn parse_int(text: String): Result[Int, String] {
  Result::Err("not implemented")
}

fn main(): Int {
  result = parse_int("10")
  match result {
    Result::Ok(value) => value
    Result::Err(message) => 0
  }
}
```

This explicit form is now implemented for compiler-known `Result[T, E]`. Only after it stays stable should Muga add propagation sugar.

Current recommendation:

- prefer prefix `try expr` for `Result` propagation if sugar is added
- do not use postfix `?` for `Result` propagation in the current design direction
- keep `T?` reserved only as possible future shorthand for `Option[T]`
- do not introduce optional chaining in the first error-propagation design

Rationale:

- `try` is a word, so the possible early return is visible at the expression site
- postfix `?` is compact, but it is less readable for beginners and conflicts with future `T?` shorthand
- Muga's simplicity goal is local readability, not only the fewest characters
- explicit `match` remains the recovery mechanism when the caller wants to handle the error locally

Candidate `try` shape:

```muga
fn load_age(path: String): Result[Int, String] {
  text = try read_file(path)
  parse_int(text)
}
```

This should desugar approximately to:

```muga
fn load_age(path: String): Result[Int, String] {
  read = read_file(path)
  match read {
    Result::Ok(text) => parse_int(text)
    Result::Err(error) => Result::Err(error)
  }
}
```

For multiple fallible steps:

```muga
fn load_user(path: String): Result[User, String] {
  text = try read_file(path)
  data = try parse_json(text)
  user_from_json(data)
}
```

Without `try`, the same flow remains expressible with explicit `match`, but gets nested quickly. That is the reason to consider a propagation form at all.

When the caller wants to recover instead of propagate, use `match`:

```muga
fn load_or_guest(path: String): User {
  result = load_user(path)
  match result {
    Result::Ok(user) => user
    Result::Err(message) => User {
      name: "Guest"
      age: 0
    }
  }
}
```

Open `try` decisions:

- whether `try expr` is allowed only inside functions returning `Result[_, E]`
- whether propagated error types must match exactly or whether conversion is ever allowed
- whether `try` should also work with `Option[T]`; current recommendation is no for v1
- whether `try` is an expression in all expression positions or only in assignment/final-expression positions

Deferred alternatives:

- postfix `expr?`: rejected for now as less readable and too close to future `T?`
- checked `throws`: deferred because it adds a separate effect system to function types and package interfaces
- implicit exceptions: rejected as a default because failure would be hidden from ordinary signatures

## 8. Package Interfaces

Package interfaces must eventually store public enum declarations:

- enum item identity
- enum name
- type parameter names
- variant names
- payload type for each variant, if present
- visibility
- declaration span or source reference for diagnostics

Public function signatures may then mention enum types such as:

```muga
pub fn read_file(path: String): Result[String, IOError]
```

In-memory summaries now represent public user-defined enum declarations and public signatures that mention user enum types. Persisted interfaces should carry the same resolved enum identity, type parameters, variants, payload types, and public signatures from the start.

## 9. Runtime Representation

The runtime now uses a simple representation for current enum-like values:

```text
Enum {
  type_name,
  variant_name,
  payload: none | value
}
```

This preserves existing `Option::Some(...)`, `Option::None`, `Result::Ok(...)`, and `Result::Err(...)` behavior. User-defined enums reuse the same value shape.

Performance-specific representations can be handled later in MIR/native lowering.

## 10. Implementation Checklist

- [x] Represent current `Option` match patterns in AST/HIR/typed HIR as enum variant patterns.
- [x] Represent current runtime `Option` values with a generic enum value shape.
- [x] Add a compiler-known enum metadata table for current `Option` variants.
- [x] Make current Option match validation, bytecode lowering, and runtime branching consume that metadata.
- [x] Add compiler-known `Result[T, E]`, `Result::Ok`, and `Result::Err`.
- [x] Make current Result match validation, bytecode lowering, and runtime branching consume the same metadata path.
- [x] Represent public compiler-known `Result[T, E]` signatures in in-memory package interface summaries.
- [x] Add parser support for `enum` declarations, type parameters, and variant declarations.
- [x] Add AST nodes for enum declarations and enum variants.
- [x] Add resolver/typechecker identity for enum declarations and variants.
- [x] Extend `TypeInfo` to represent enum types and enum item identity.
- [x] Generalize compiler-known `match` beyond `Option[T]` to `Result[T, E]`.
- [x] Add exhaustive match checking over compiler-known enum variants.
- [x] Add HIR, bytecode, and runtime support for compiler-known enum construction and matching.
- [x] Add typed HIR support for enum declarations, variant constructors, match patterns, and enum types.
- [x] Add in-memory package interface summaries for public enum declarations.
- [x] Preserve all existing `Option[T]` behavior and tests.
- [x] Add `Result[T, E]` tests before adding any propagation operator.

## 11. Recommended Phasing

1. Harden enum diagnostics, package visibility, imported qualified variants, and in-memory interface validation.
2. Extend persisted package interfaces and cache formats now that enum/result signatures have source-level identity.
3. Revisit `try expr` propagation syntax.
4. Continue toward MIR/native lowering once package boundaries are real inputs.
