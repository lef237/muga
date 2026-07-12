# Enums, Result, And Error Propagation Draft

Status: design draft with an implemented and hardened MVP. The current Rust compiler implements compiler-known `Option[T]` and `Result[T, E]`, plus user-defined `enum` declarations with optional unconstrained type parameters, zero-payload and one-payload variants, qualified constructors/patterns, payload discard `_` inside variant patterns, exhaustive `match`, VM execution, typed HIR, package interface summaries, package enum visibility checks, imported `alias::Enum::Variant` coverage, stale enum interface validation, downstream typed checking from loaded interface summaries, explicit-root `.mgi` artifact discovery for typed checking, source/dependency-hash `.mgc` package check cache validation, `muga check --artifact-root` consumption of those artifacts, CLI emission of `.mgi` / `.mgc` artifacts, and prefix `try expr` propagation for `Result[T, E]`. AST/HIR/typed HIR match patterns use enum-variant-shaped internal data, runtime enum values use a generic enum-value representation, and variant facts are consumed by typechecking, bytecode lowering, and runtime branching.

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
- overloaded or behavior-conformance-based variant dispatch

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
- broad catch-all wildcard patterns
- nested patterns
- pattern guards
- recursive enum layout optimization
- derived equality/hash behavior beyond the current scalar-only equality policy

This MVP is enough to model `Option[T]` and `Result[T, E]`.

`Option[T]`, `Result[T, E]`, and user-defined enums do not support `==` or `!=`. Use exhaustive `match` and compare scalar payload fields explicitly. Derived enum equality/hash behavior is a future feature and must define package-interface persistence, diagnostics, and behavior for unsupported payload types before it is added.

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
- `_` may appear only inside a one-payload variant pattern, such as `Result::Ok(_)`, and discards that payload without introducing a binding
- `_ =>` broad catch-all arms are not currently supported
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

## 7. Result Propagation

`Result[T, E]` is the recoverable-error type. Explicit `match` is still the recovery mechanism, and prefix `try expr` is the implemented propagation form. If Muga adds a dot-chain propagation form later, it should use postfix keyword syntax `expr.try`, not postfix `expr?`.

Local recovery remains explicit:

```muga
fn main(): Int {
  result = "10".parse_int()
  match result {
    Result::Ok(value) => value
    Result::Err(message) => 0
  }
}
```

This explicit form is implemented for compiler-known `Result[T, E]`, and prefix `try expr` is implemented for propagation.

Current propagation shape:

- use prefix `try expr` for the implemented `Result` propagation form
- reserve future dot-chain `Result` propagation for postfix keyword syntax `expr.try`
- do not use postfix `expr?` for `Result` propagation in the current design direction
- keep the `?` syntax family reserved for optional-value features such as future `T?` and `?.`
- do not make optional chaining part of `Result` propagation

Rationale:

- `try` is a word, so the possible early return is visible at the expression site
- `expr.try` keeps that word visible inside Muga's normal dot-chain style
- postfix `expr?` is compact, but it hides a function-level early return behind a small marker
- Muga's simplicity goal is local readability, not only the fewest characters
- explicit `match` remains the recovery mechanism when the caller wants to handle the error locally
- reserving `?` for optional values gives the syntax family one primary meaning: absence, not recoverable errors

Implemented `try` shape:

```muga
fn load_age(path: String): Result[Int, String] {
  text = try read_file(path)
  text.parse_int()
}
```

This should desugar approximately to:

```muga
fn load_age(path: String): Result[Int, String] {
  read = read_file(path)
  match read {
    Result::Ok(text) => text.parse_int()
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

Without `try`, the same flow remains expressible with explicit `match`, but gets nested quickly. That is the reason Muga includes a propagation form.

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

Implemented `try` decisions:

- `try expr` is allowed only inside functions whose return type is, or can be inferred as, `Result[_, E]`
- propagated error types must match exactly; no conversion hook exists
- `try` works only with `Result[T, E]`, not `Option[T]`
- `try` is a prefix expression and can appear wherever its unwrapped `T` type is valid
- invalid `try Result::Ok(...)` / `try Result::Err(...)` placements should report the `try` placement error without a secondary missing-expected-Result diagnostic for the constructor
- the current compiler implements only prefix `try expr`; future `expr.try` is a design direction, not an implemented syntax
- `expr?` is not an alias for `try expr` or future `expr.try`

### 7.1 Result Chain Propagation And Value Chaining

Rust-style `expr?` is useful because it keeps success-path code compact:

```rust
let age = read_to_string(path)?.trim().parse::<i64>()?;
```

Muga should support the same practical need without assigning function-level early return to postfix Result propagation `expr?`. The preferred future propagation syntax is postfix keyword `expr.try`:

```muga
fn load_age(path: String): Result[Int, String] {
  age = read_file(path).try.trim().parse_int().try
  Result::Ok(age)
}
```

`expr.try` should be syntax, not a helper method. It is equivalent to applying `try` at that point in the chain:

```muga
read_file(path).try.trim()
```

is equivalent to:

```muga
(try read_file(path)).trim()
```

Future `expr.try` rules should match prefix `try expr` rules:

- `expr` must have type `Result[T, E]`
- the enclosing function must return, or be inferred to return, `Result[U, E]`
- `Result::Ok(value)` continues the chain as `value: T`
- `Result::Err(error)` returns `Result::Err(error)` from the enclosing function
- propagated error types must match exactly at first
- `expr.try` works only with `Result[T, E]`, not `Option[T]`
- `try` remains a reserved keyword in this position; user-defined methods should not be able to overload this control-flow behavior

This gives Muga two spellings for the same Result propagation operation:

- `try expr` for the already implemented prefix form
- `expr.try` for a future dot-chain form that keeps fluent success paths compact

Ordinary `Result` value chaining remains a separate concern. The implemented `std::result` helper package uses package-qualified chained calls; the following assumes `result` is the visible helper package alias:

```muga
fn load_age(path: String): Result[Int, String] {
  (
    read_file(path)
      .result::map(fn(text) { text.trim() })
      .result::and_then(fn(text) { text.parse_int() })
  )
}
```

The helper semantics are:

- `result::is_ok(result)` returns `true` for `Ok(_)` and `false` for `Err(_)`.
- `result::is_err(result)` returns `true` for `Err(_)` and `false` for `Ok(_)`.
- `result::map(result, f)` applies `f` to the `Ok` payload and preserves `Err` unchanged.
- `result::map_err(result, f)` applies `f` to the `Err` payload and preserves `Ok` unchanged.
- `result::and_then(result, f)` applies `f` to the `Ok` payload when `f` itself returns `Result[U, E]`, and preserves `Err` unchanged.
- `result::value_or(result, fallback)` unwraps `Ok(value)` or returns `fallback` for `Err`.
- these helpers transform a `Result` value; they do not return early from the enclosing function.
- no helper performs implicit error conversion unless such a conversion API is explicitly added.

When a function wants to unwrap intermediate values and propagate errors, use `try`:

```muga
fn load_age(path: String): Result[Int, String] {
  text = try read_file(path)
  age = try text.trim().parse_int()
  Result::Ok(age)
}
```

This keeps the control-flow distinction explicit:

- use `try expr`, and future `expr.try`, for function-level `Err` propagation
- use `result::map` / `result::and_then` for fluent value transformation
- use explicit `match` for local recovery or fallback behavior
- do not make `Result` participate in `?.` optional chaining
- do not automatically transform `Option[Result[T, E]]` into `Result[Option[T], E]` or the reverse

Deferred alternatives:

- postfix `expr?` for `Result`: rejected as too implicit for Muga's error-propagation direction
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

In-memory summaries now represent public user-defined enum declarations and public signatures that mention user enum types. The current persisted interface text format round-trips the same resolved enum identity, type parameters, variants, payload types, public signatures, and source spans. Persisted interfaces also carry deterministic content hashes. Downstream typed checking can use loaded interface summaries or discovered `.mgi` artifacts without reading dependency implementation bodies. Package check cache keys now combine entry package source hashes and dependency interface hashes, with missing or stale `.mgc` artifacts rejected before artifact-backed checking proceeds. The CLI can emit artifacts with `muga emit-interface` and `muga emit-check-cache`, then consume them with `muga check --artifact-root`.

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
- [x] Add payload discard `_` inside enum variant patterns without adding broad catch-all match arms.
- [x] Add in-memory package interface summaries for public enum declarations.
- [x] Preserve all existing `Option[T]` behavior and tests.
- [x] Add `Result[T, E]` tests before adding any propagation operator.
- [x] Add prefix `try expr` propagation for `Result[T, E]`.

## 11. Recommended Phasing

1. Add project artifact-root config or dependency artifact discovery around the existing interface and check-cache metadata.
2. Add full package artifact storage/reuse.
3. Harden `try expr` across package artifacts and any future stdlib APIs that return `Result`.
4. Continue toward MIR/native lowering once package boundaries are real inputs.
