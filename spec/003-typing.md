# Typing Specification v1

Derived from [mini-language-spec-v1.md](../mini-language-spec-v1.md). This document defines the v1 typing policy, with emphasis on inference-first ergonomics and the limited cases where annotations are mandatory.

## 1. Typing Policy

The language prefers omission of type annotations.

- local bindings should infer their type from the right-hand side
- function parameter and return types should be inferred when the result is unique
- annotations are required only when inference cannot determine a unique type

## 2. Inference Sources

v1 inference may use:

- literal types
- operator constraints
- call-site constraints
- branch result agreement
- explicit annotations already present in the same declaration

Examples:

```txt
x = 1          # Int
name = "m"     # String
```

```txt
fn inc(x) {
  x + 1
}
```

If `+` here is the integer addition operator in v1, `x` is inferred as `Int`.

## 3. Local Bindings

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

## 4. Conditions and Branches

The condition expression of:

- `if`
- `while`

must have type `Bool`.

For an `if` expression, both branches must produce a compatible result type.

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

## 5. Function Parameter Inference

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

## 6. Function Return Inference

The return type of a function is inferred from the final expression in the body.

When control flow branches, the return type is inferred from the unified branch result type.

If the body does not provide enough information to infer a unique return type, a return annotation is required.

## 7. Mandatory Annotations

Annotations are required in the following cases:

1. a function parameter type is not uniquely inferable
2. a function return type is not uniquely inferable
3. a recursive function has neither an annotated parameter nor an annotated return type
4. a mutually recursive function participates in a recursive group without an explicit signature

For v1, an explicit function signature means:

- at least one parameter or the return type is annotated for direct recursion
- every function in a mutually recursive group has enough annotations to determine its full callable type before body checking

## 8. Direct Recursion Rule

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
fn fact(n) -> Int {
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

## 9. Mutual Recursion Rule

Mutually recursive functions require explicit signatures.

In v1, this means each function in the recursive group must carry enough annotations for its callable type to be known before any body in the group is checked.

Valid:

```txt
fn is_even(n: Int) -> Bool {
  if n == 0 {
    true
  } else {
    is_odd(n - 1)
  }
}

fn is_odd(n: Int) -> Bool {
  if n == 0 {
    false
  } else {
    is_even(n - 1)
  }
}
```

## 10. Minimal Type Vocabulary

The split v1 documents assume at least the following example-level types:

- `Int`
- `Bool`
- `String`
- function types

This document does not yet standardize a full type-expression grammar beyond what is needed for annotations in examples.
