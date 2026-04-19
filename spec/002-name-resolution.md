# Name Resolution Specification v1

Derived from [mini-language-spec-v1.md](../mini-language-spec-v1.md). This document is normative for scope construction, binding introduction, update resolution, shadowing, non-local update rejection, and the name-oriented part of dot-expression resolution.

## 1. Scope Model

The language uses lexical scopes arranged as a tree.

- the program root is a scope
- each block introduces a child scope
- each function body introduces a child scope
- function parameters belong to the function-body scope

For update resolution, function scopes are also boundaries:

- a mutable binding may be updated from nested blocks in the same function
- a mutable binding may not be updated across a function boundary

Lookup for expression names searches:

1. the current scope
2. the nearest enclosing scope
3. repeated outward until the root scope

Type names are resolved in a separate type namespace.

## 1.1 Type Namespace

v1 distinguishes:

- the value namespace, which contains locals, functions, and parameters
- the type namespace, which contains nominal record names

`record User { ... }` introduces `User` in the type namespace only.

## 2. Binding Kinds

The resolver distinguishes the following binding kinds:

- immutable local binding
- mutable local binding
- function binding
- parameter binding

Function bindings and parameter bindings are immutable.

Record names are not value bindings.

## 3. Shadowing Policy

Shadowing is prohibited in v1.

A new binding is rejected if the same name already exists in any enclosing scope.

This prohibition applies equally to:

- `mut x = e`
- `x = e` when interpreted as a new binding
- function declarations
- function parameters

## 4. Resolution Rules for `mut x = e`

For:

```txt
mut x = e
```

the resolver applies the following rules:

1. If `x` already exists in the current scope, reject the program as a duplicate binding.
2. Otherwise, if `x` exists in any enclosing scope, reject the program as prohibited shadowing.
3. Otherwise, introduce a new mutable binding `x` in the current scope.

## 5. Resolution Rules for `x = e`

For:

```txt
x = e
```

the resolver applies the following rules in order:

1. If `x` exists in the current scope as a mutable binding, this is an update of that binding.
2. If `x` exists in the current scope as an immutable local binding, function binding, or parameter binding, reject the program as an immutable update.
3. If `x` does not exist in the current scope but an enclosing scope in the same function contains a mutable binding named `x`, this is an update of the nearest such binding.
4. If `x` does not exist in the current scope but an enclosing scope in the same function contains an immutable binding, function binding, or parameter binding named `x`, reject the program as an immutable update.
5. If `x` is not found in the current function and an outer function scope contains a mutable binding named `x`, reject the program as an outer-scope mutation.
6. If `x` is not found in the current function and an outer function scope contains an immutable binding, function binding, or parameter binding named `x`, reject the program as prohibited shadowing.
7. Otherwise, introduce a new immutable binding `x` in the current scope.

Rule 5 is the v1 interpretation of non-local stateful assignment across function boundaries. Reads may cross a function boundary, but writes may not.

In other words, `x = e` may update a mutable binding in the current scope or in an enclosing block of the same function. Every immutable name in that same function region rejects `x = e`, and writes across a function boundary are disallowed.

## 6. Function Name Predeclaration

Within a single scope, all function declarations are entered into that scope before their bodies are resolved.

This enables:

- direct recursion
- references to later functions in the same scope
- mutually recursive function groups

Function bodies are then resolved against that completed function-binding set.

## 6.1 Record Name Predeclaration

Top-level record declarations are entered into the type namespace before type expressions and record literals that refer to them are validated.

## 7. Parameter Rules

When resolving a function declaration:

- each parameter introduces a new immutable binding in the function-body scope
- parameter names must be unique within the parameter list
- parameter names must not conflict with bindings in enclosing scopes

Example:

```txt
x = 1

fn bad(x: Int) {
  x
}
```

This is invalid because the parameter `x` would shadow the outer binding.

## 8. Dot-Expression Resolution

The parser distinguishes:

- `expr.name`
- `expr.name(args...)`

For `expr.name`:

- `name` is interpreted as a field name candidate
- validation of that field name depends on the static type of `expr`
- lexical value bindings named `name` are irrelevant

For `expr.name(args...)`:

- the lexical function name `name` is looked up in the ordinary function namespace
- if that function is receiver-style and applicable to the type of `expr`, the chained call resolves to that function
- otherwise, if ordinary call resolution for `name(expr, args...)` succeeds, the chained call resolves by UFCS-style desugaring
- otherwise, the expression is rejected

Because v1 has no overloading, there is at most one visible ordinary function binding named `name`.

Anonymous functions do not participate in `expr.name(args...)`.

Record fields are never considered callable by chained call syntax in v1.

## 9. Name Resolution Examples

### 9.1 Valid local update

```txt
mut total = 0
total = total + 1
```

### 9.2 Valid enclosing-block update in the same function

```txt
fn sum_to(n: Int) {
  mut i = 0

  while i < n {
    i = i + 1
  }

  i
}
```

### 9.3 Invalid immutable update

```txt
x = 1
x = 2
```

### 9.4 Invalid shadowing

```txt
flag = true
value = 1

if flag {
  value = 2
}
```

### 9.5 Invalid outer-scope mutation

```txt
mut total = 0

fn add(x: Int) {
  total = total + x
}
```

## 10. Practical Note

Because `x = e` may introduce a fresh immutable binding, a misspelled name can accidentally create a new binding:

```txt
mut count = 0
coutn = count + 1
```

If `coutn` is not otherwise defined, the resolver accepts it as a new immutable binding. A compiler or linter should warn on suspiciously similar names.
