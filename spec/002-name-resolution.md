# Name Resolution Specification v1

Derived from [mini-language-spec-v1.md](../mini-language-spec-v1.md). This document is normative for scope construction, binding introduction, update resolution, shadowing, and non-local update rejection.

## 1. Scope Model

The language uses lexical scopes arranged as a tree.

- the program root is a scope
- each block introduces a child scope
- each function body introduces a child scope
- function parameters belong to the function-body scope

Lookup for expression names searches:

1. the current scope
2. the nearest enclosing scope
3. repeated outward until the root scope

## 2. Binding Kinds

The resolver distinguishes the following binding kinds:

- immutable local binding
- mutable local binding
- function binding
- parameter binding

Function bindings and parameter bindings are immutable.

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
3. If `x` does not exist in the current scope and an enclosing scope contains a mutable binding named `x`, reject the program as an outer-scope mutation.
4. If `x` does not exist in the current scope and an enclosing scope contains an immutable binding, function binding, or parameter binding named `x`, reject the program as prohibited shadowing.
5. Otherwise, introduce a new immutable binding `x` in the current scope.

Rule 3 is the v1 interpretation of non-local stateful assignment. Outer-scope reads are allowed, but outer-scope writes are not.

In other words, `x = e` may update only a mutable binding in the current scope. Every current-scope immutable name, including parameters and function names, rejects `x = e`.

## 6. Function Name Predeclaration

Within a single scope, all function declarations are entered into that scope before their bodies are resolved.

This enables:

- direct recursion
- references to later functions in the same scope
- mutually recursive function groups

Function bodies are then resolved against that completed function-binding set.

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

## 8. Name Resolution Examples

### 8.1 Valid local update

```txt
mut total = 0
total = total + 1
```

### 8.2 Invalid immutable update

```txt
x = 1
x = 2
```

### 8.3 Invalid shadowing

```txt
flag = true
value = 1

if flag {
  value = 2
}
```

### 8.4 Invalid outer-scope mutation

```txt
mut total = 0

fn add(x: Int) {
  total = total + x
}
```

## 9. Practical Note

Because `x = e` may introduce a fresh immutable binding, a misspelled name can accidentally create a new binding:

```txt
mut count = 0
coutn = count + 1
```

If `coutn` is not otherwise defined, the resolver accepts it as a new immutable binding. A compiler or linter should warn on suspiciously similar names.
