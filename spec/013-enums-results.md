# Enums, Result, And Error Propagation Draft

Status: design draft. The current Rust compiler implements `Option[T]`, `Option::Some`, `Option::None`, and exhaustive Option `match` as compiler-known enum-like behavior. AST/HIR/typed HIR match patterns now use enum-variant-shaped internal data, runtime Option values use a generic enum-value representation, and Option variant facts are stored in a compiler-known enum metadata table consumed by typechecking, bytecode lowering, and runtime branching. General user-defined enum declarations, `Result[T, E]`, and error propagation are not implemented yet.

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

This matches the source shape already implemented for `Option[T]`.

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

The implementation has started moving toward one internal ADT model: match patterns are represented as enum variant patterns, runtime values use a generic enum value shape, and current `Option` match validation plus bytecode/runtime branching consult compiler-known enum metadata. The next step is to add `Result[T, E]` to that metadata and then make later source enum declarations consume the same model so that `Option[T]`, `Result[T, E]`, and user-defined enums do not permanently require separate code paths.

Compatibility requirements:

- `Option[T]` remains the canonical spelling.
- `Option::Some(value)` remains valid.
- `Option::None` remains valid.
- existing exhaustive Option `match` remains valid.
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

Only after this is stable should Muga decide whether to add `?`.

Open `?` decisions:

- whether `?` is only for `Result` propagation
- whether `T?` is future shorthand for `Option[T]`
- whether optional chaining ever uses `?`
- how to avoid grammar and readability conflicts between those uses

Current recommendation:

- keep `T?` reserved only as possible future `Option[T]` shorthand
- do not implement `?` propagation in the first `Result` slice
- revisit propagation after explicit `Result` match works across packages

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

Before persisted interfaces are introduced, in-memory summaries should be extended far enough to represent enum/result types in public signatures.

## 9. Runtime Representation

The runtime now uses a simple representation for current enum-like values:

```text
Enum {
  type_name,
  variant_name,
  payload: none | value
}
```

This currently preserves existing `Option::Some(...)` and `Option::None` behavior. `Result` and user-defined enums should reuse the same value shape.

Performance-specific representations can be handled later in MIR/native lowering.

## 10. Implementation Checklist

- [x] Represent current `Option` match patterns in AST/HIR/typed HIR as enum variant patterns.
- [x] Represent current runtime `Option` values with a generic enum value shape.
- [x] Add a compiler-known enum metadata table for current `Option` variants.
- [x] Make current Option match validation, bytecode lowering, and runtime branching consume that metadata.
- [ ] Add parser support for `enum` declarations, type parameters, and variant declarations.
- [ ] Add AST nodes for enum declarations and enum variants.
- [ ] Add resolver/typechecker identity for enum declarations and variants.
- [ ] Extend `TypeInfo` to represent enum types and enum item identity.
- [ ] Generalize `match` beyond `Option[T]`.
- [ ] Add exhaustive match checking over enum variants.
- [ ] Add HIR, bytecode, and runtime support for general enum construction and matching.
- [ ] Add typed HIR support for enum declarations, variant constructors, match patterns, and enum types.
- [ ] Add in-memory package interface summaries for public enum declarations.
- [ ] Preserve all existing `Option[T]` behavior and tests.
- [ ] Add `Result[T, E]` tests before adding any propagation operator.

## 11. Recommended Phasing

1. Finish small naming cleanup in the current Option bytecode/runtime helpers only if it reduces duplication for Result.
2. Add compiler-known `Result[T, E]`, `Result::Ok`, `Result::Err`, and exhaustive Result `match`.
3. Add user-defined enum declarations with zero/one-payload variants.
4. Add generic enum declarations if they are not covered by the previous step.
5. Revisit `?` propagation syntax.
6. Extend persisted package interfaces and cache formats once enum/result signatures are stable.
