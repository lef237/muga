# Core Language Specification v1

Derived from [mini-language-spec-v1.md](../mini-language-spec-v1.md). This document defines the surface language and the core execution-facing rules. Name resolution, typing, and function-specific rules are split into companion documents:

- [002-name-resolution.md](./002-name-resolution.md)
- [003-typing.md](./003-typing.md)
- [004-functions.md](./004-functions.md)

## 1. Design Constraints

The v1 language is intentionally small and follows these constraints:

- no `let`
- immutable bindings by default
- `mut` introduces mutable bindings
- `x = e` is resolved statically as either a new immutable binding or an update
- shadowing is prohibited
- outer-scope updates are prohibited
- type annotations are omitted unless inference cannot determine a unique type

## 2. Core Binding Forms

### 2.1 Mutable binding

```txt
mut x = e
```

This form always attempts to introduce a new mutable binding in the current scope.

### 2.2 Plain assignment-like form

```txt
x = e
```

This form is parsed uniformly and resolved later:

- if `x` is not defined in the current scope, it may introduce a new immutable binding
- if `x` is already a mutable binding in the current scope, it updates that binding
- if `x` is already an immutable binding in the current scope, it is an error

The exact static resolution rules are normative in [002-name-resolution.md](./002-name-resolution.md).

## 3. Blocks and Scope

The language uses lexical scoping.

- every block `{ ... }` creates a new scope
- every function body creates a new scope
- name lookup prefers the nearest enclosing scope
- bindings are visible from their declaration point to the end of the enclosing block

Outer-scope bindings may be read from inner scopes, but v1 does not allow updating outer-scope bindings.

## 4. Statements and Expressions

The language has the following core constructs:

- binding/update statements
- function declarations
- `if` statements and `if` expressions
- `while` statements
- expression statements

Blocks may appear in expression position. When a block is evaluated as an expression, its value is the value of its final expression.

`if` without `else` is statement-only. `if` with `else` may appear in expression position and yields the branch result value.

Example:

```txt
abs = fn(n: Int) {
  if n < 0 {
    -n
  } else {
    n
  }
}
```

## 5. Grammar Sketch

This is a v1-oriented EBNF sketch. `type_expr` is defined abstractly here and constrained further by [003-typing.md](./003-typing.md).

```ebnf
program           := stmt*

stmt              := assign_like_stmt
                   | func_decl
                   | if_stmt
                   | while_stmt
                   | expr_stmt

assign_like_stmt  := "mut" IDENT "=" expr
                   | IDENT "=" expr

func_decl         := "fn" IDENT "(" params? ")" return_annot? block_expr
return_annot      := "->" type_expr

params            := param ("," param)*
param             := IDENT
                   | IDENT ":" type_expr

while_stmt        := "while" expr block_expr
if_stmt           := "if" expr block_expr ("else" block_expr)?
expr_stmt         := expr

expr              := literal
                   | IDENT
                   | call_expr
                   | anon_fn
                   | binary_expr
                   | if_expr
                   | block_expr
                   | "(" expr ")"

if_expr           := "if" expr block_expr "else" block_expr
call_expr         := expr "(" args? ")"
args              := expr ("," expr)*

anon_fn           := "fn" "(" params? ")" return_annot? block_expr
block_expr        := "{" stmt* expr? "}"
```

## 6. Execution-Oriented Summary

The core language model is:

- `mut x = e` introduces a new mutable binding
- `x = e` either introduces a new immutable binding or updates an existing mutable binding in the current scope
- immutable bindings cannot be updated
- function names are ordinary immutable bindings
- function parameters are immutable bindings
- the value of a function body is the final expression in that body

## 7. Examples

Valid:

```txt
x = 1
mut total = 0
total = total + x
```

Invalid:

```txt
x = 1
x = 2   # error: immutable binding cannot be updated
```
