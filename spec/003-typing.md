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

In addition, v1 introduces:

- user-defined nominal record types introduced by `record`
- source-level function types written with `->`

Therefore, source `type_expr` is:

```ebnf
type_expr          := function_type
function_type      := function_domain "->" type_expr
                    | non_function_type
function_domain    := non_function_type
                    | "(" type_expr_list? ")"
non_function_type  := "Int"
                    | "Bool"
                    | "String"
                    | IDENT
type_expr_list     := type_expr ("," type_expr)*
```

Examples:

- `Int -> Int`
- `(Int, String) -> Bool`
- `() -> Int`

There are no generics, no user-written type variables, and no polymorphic type syntax in v1.

## 3. Prelude Built-ins

The v1 prelude currently provides:

- `print`

`print` accepts exactly one argument of type `Int`, `Bool`, or `String`, writes its textual representation to standard output, and returns that same value.

Because `print` accepts several concrete types, it does not by itself make an unconstrained parameter uniquely inferable.

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

## 4. Higher-Order Functions

v1 supports higher-order functions.

Allowed in principle:

- passing a named function as an argument
- passing an anonymous function as an argument
- storing a function in a local binding

Example:

```txt
fn inc(x: Int): Int {
  x + 1
}

fn apply(x: Int, f: Int -> Int): Int {
  f(x)
}

apply(10, inc)
apply(10, fn(n: Int): Int {
  n + 1
})
```

## 5. Record Typing

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

## 6. Field Access and Chained Call Typing

For field access:

```txt
expr.name
```

`expr` must have a record type that declares a field `name`. The expression type is the declared type of that field.

For chained call:

```txt
expr.name(arg1, arg2)
```

the receiver expression `expr` is typed first.

Then:

1. if `name` resolves to a receiver-style function, the call is typed as a call of that function with `expr` as the first argument
2. otherwise, if `name(expr, arg1, arg2)` is a valid ordinary function call, the chained call is typed as that UFCS-style desugaring
3. otherwise, the expression is a type error

Because record fields may not have function type in v1, `expr.name(...)` never means a call through a function-valued field.

## 7. Operator Typing Rules

The built-in operator typing rules are:

- unary `-` : `Int -> Int`
- unary `!` : `Bool -> Bool`
- `+`, `-`, `*`, `/` : `Int -> Int -> Int`
- `<`, `<=`, `>`, `>=` : `Int -> Int -> Bool`
- `==`, `!=` : allowed only for identical primitive types among `Int`, `Bool`, and `String`

String concatenation is not part of v1. Therefore, `+` is `Int`-only.

## 8. Inference Sources

v1 inference may use:

- literal types
- operator constraints
- branch result agreement
- explicit annotations already present in the same declaration
- explicit function-type annotations on parameters

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

## 9. Local Bindings

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

## 10. Conditions and Branches

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

## 11. Function Parameter Inference

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

## 12. Function Return Inference

The return type of a function is inferred from the final expression in the body.

When control flow branches, the return type is inferred from the unified branch result type.

If the body does not provide enough information to infer a unique return type, a return annotation is required.

## 13. Inference Boundary

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
- polymorphic generalization
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

## 14. Mandatory Annotations

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

## 15. Direct Recursion Rule

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

## 13. Mutual Recursion Rule

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
