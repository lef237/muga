# Function Specification v1

Derived from [mini-language-spec-v1.md](../mini-language-spec-v1.md). This document defines function declarations, anonymous functions, parameter semantics, return semantics, and recursion-related requirements.

## 1. Function Declarations

A function declaration has the form:

```txt
fn add(a, b) {
  a + b
}
```

Semantically, a function declaration introduces a new immutable binding in the current scope.

It is close to the desugared model:

```txt
add = fn(a, b) {
  a + b
}
```

The binding is immutable, so the function name cannot later be updated with `=`.

Example:

```txt
fn add(a: Int, b: Int) {
  a + b
}

add = fn(x: Int, y: Int) {
  x - y
}   # error: function bindings are immutable
```

## 2. Anonymous Functions

Anonymous functions are expressions.

```txt
double = fn(x) {
  x * 2
}
```

They follow the same parameter and return rules as named functions.

## 3. Parameter Semantics

Parameters are introduced as immutable bindings in the function-body scope.

Therefore:

- parameters cannot be reassigned
- parameters participate in the no-shadowing rule
- parameter names must be unique within the same function

Invalid:

```txt
fn bump(x: Int) {
  x = x + 1
}
```

## 4. Return Semantics

The value of a function is the value of the final expression in its body.

`return` is not required in v1.

Function bodies are value blocks, so every function body ends with a final expression.

Example:

```txt
fn abs(x: Int) {
  if x < 0 {
    -x
  } else {
    x
  }
}
```

## 5. Name Availability Inside Functions

When a function body is resolved and typed, the following names may be available:

- its parameters
- bindings declared earlier in the same scope
- function names predeclared in the same scope
- readable bindings from enclosing scopes

The following are not allowed:

- updating an enclosing mutable binding
- introducing a local binding that shadows an enclosing binding

## 6. Closure Capture

Functions use lexical scope and may capture readable bindings from enclosing scopes.

Example:

```txt
base = 10

fn add_base(x: Int) {
  x + base
}
```

Captured outer bindings remain subject to the ordinary v1 rules:

- outer bindings may be read
- outer mutable bindings may not be updated from the inner function

## 7. Direct Recursion

Direct recursion is allowed.

At least one of the following must be annotated:

- one or more parameters
- the return type

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

## 8. Mutual Recursion

Mutual recursion is allowed only when explicit signatures are present for the entire recursive group.

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

Invalid:

```txt
fn is_even(n) {
  if n == 0 {
    true
  } else {
    is_odd(n - 1)
  }
}

fn is_odd(n) {
  if n == 0 {
    false
  } else {
    is_even(n - 1)
  }
}
```

## 9. Summary

Functions in v1 are ordinary immutable bindings of function values, with:

- immutable parameters
- final-expression returns
- lexical closure capture for readable outer bindings
- access to the prelude builtin `print`
- inference-first signatures
- limited, explicit requirements for direct recursion and mutual recursion
