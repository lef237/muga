# Enums, Result, And Error Propagation Draft

Status: design draft. The current Rust compiler implements compiler-known `Option[T]`, `Result[T, E]`, their qualified constructors, and exhaustive `match` for both. AST/HIR/typed HIR match patterns use enum-variant-shaped internal data, runtime Option/Result values use a generic enum-value representation, and variant facts are stored in a compiler-known enum metadata table consumed by typechecking, bytecode lowering, and runtime branching. General user-defined enum declarations and error propagation are not implemented yet.

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

## 3. Source Syntax Direction

Recommended enum syntax:

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

Variant construction should use qualified names:

```muga
present: Option[Int] = Option::Some(1)
missing: Option[Int] = Option::None

ok: Result[Int, String] = Result::Ok(1)
err: Result[Int, String] = Result::Err("missing")
```

Pattern matching should use the same qualified form:

```muga
match result {
  Result::Ok(value) => value
  Result::Err(message) => 0
}
```

This matches the source shape already implemented for compiler-known `Option[T]` and `Result[T, E]`.

## 4. MVP Variant Shape

Recommended MVP:

- zero-payload variants, such as `None`
- one-payload variants, such as `Some(T)` or `Err(E)`
- variants are namespaced under their enum type
- variant constructors are not imported into the local namespace unqualified
- enum values are nominal, not structural

Deferred:

- multi-payload tuple-like variants
- named-field variants
- wildcard patterns
- nested patterns
- pattern guards
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

## 6. Relationship To Current Option

`Option[T]` is currently compiler-known. That can remain true internally during migration, but its source semantics should match ordinary enum behavior.

The implementation has started moving toward one internal ADT model: match patterns are represented as enum variant patterns, runtime values use a generic enum value shape, and current `Option` / `Result` match validation plus bytecode/runtime branching consult compiler-known enum metadata. The next step is to make source enum declarations consume the same model so that `Option[T]`, `Result[T, E]`, and user-defined enums do not permanently require separate code paths.

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

Before persisted interfaces are introduced, in-memory summaries should be extended far enough to represent user-defined enum declarations in public signatures. Compiler-known `Result[T, E]` signatures are already represented.

## 9. Runtime Representation

The runtime now uses a simple representation for current enum-like values:

```text
Enum {
  type_name,
  variant_name,
  payload: none | value
}
```

This currently preserves existing `Option::Some(...)`, `Option::None`, `Result::Ok(...)`, and `Result::Err(...)` behavior. User-defined enums should reuse the same value shape.

Performance-specific representations can be handled later in MIR/native lowering.

## 10. Implementation Checklist

- [x] Represent current `Option` match patterns in AST/HIR/typed HIR as enum variant patterns.
- [x] Represent current runtime `Option` values with a generic enum value shape.
- [x] Add a compiler-known enum metadata table for current `Option` variants.
- [x] Make current Option match validation, bytecode lowering, and runtime branching consume that metadata.
- [x] Add compiler-known `Result[T, E]`, `Result::Ok`, and `Result::Err`.
- [x] Make current Result match validation, bytecode lowering, and runtime branching consume the same metadata path.
- [x] Represent public compiler-known `Result[T, E]` signatures in in-memory package interface summaries.
- [ ] Add parser support for `enum` declarations, type parameters, and variant declarations.
- [ ] Add AST nodes for enum declarations and enum variants.
- [ ] Add resolver/typechecker identity for enum declarations and variants.
- [ ] Extend `TypeInfo` to represent enum types and enum item identity.
- [x] Generalize compiler-known `match` beyond `Option[T]` to `Result[T, E]`.
- [x] Add exhaustive match checking over compiler-known enum variants.
- [x] Add HIR, bytecode, and runtime support for compiler-known enum construction and matching.
- [ ] Add typed HIR support for enum declarations, variant constructors, match patterns, and enum types.
- [ ] Add in-memory package interface summaries for public enum declarations.
- [ ] Preserve all existing `Option[T]` behavior and tests.
- [x] Add `Result[T, E]` tests before adding any propagation operator.

## 11. Recommended Phasing

1. Add user-defined enum declarations with zero/one-payload variants.
2. Add generic enum declarations if they are not covered by the previous step.
3. Revisit `try expr` propagation syntax.
4. Extend persisted package interfaces and cache formats once enum/result signatures are stable.
