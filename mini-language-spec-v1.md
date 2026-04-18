# Mini Language Spec v1

## Overview

This document captures the current design of a small programming language with the following goals:

- no `let`
- immutability by default
- `mut` for mutable bindings
- `=` serves as either binding or mutable update depending on name resolution
- shadowing is prohibited
- type annotations are omitted by default and only required when inference fails
- function declarations are treated as ordinary immutable bindings of function values

This is an intentionally small, implementation-oriented first draft.

---

## 1. Binding and Assignment Model

### 1.1 Core syntax

```txt
x = 1        # new immutable binding if x is undefined in current scope
mut y = 1    # new mutable binding

y = 2        # update if y is mutable in current scope
x = 2        # error if x is immutable in current scope
```

### 1.2 Static meaning rules

#### Rule A: mutable binding

```txt
mut x = e
```

- If `x` is undefined in the current scope, introduce a new mutable binding.
- If `x` is already defined in the current scope, this is an error.

#### Rule B: plain assignment-like form

```txt
x = e
```

- If `x` is undefined in the current scope, introduce a new immutable binding.
- If `x` is already defined as mutable in the current scope, update it.
- If `x` is already any immutable name in the current scope, this is an error.

Here, current-scope immutable names include ordinary immutable bindings, function names, and parameters.

### 1.3 Design note

`x = e` is parsed as one syntactic form. Whether it means "new binding" or "update" is determined later during static analysis / name resolution.

---

## 2. Scope Rules

### 2.1 Lexical scope

The language uses lexical scoping.

- A block `{ ... }` creates a new scope.
- A function body creates a new scope.
- Variable lookup searches the nearest enclosing scope first.

Example:

```txt
x = 1

if cond {
  y = x + 1
}

x   # OK
y   # error: y is out of scope
```

### 2.2 Updates within a function

A mutable variable may be updated from the current scope or from a nested block in the same function body.

Across a function boundary, outer-scope variables may be read, but not updated from an inner scope in v1.

Example:

```txt
mut total = 0

fn add_total(x) {
  total = total + x   # error in v1
}
```

---

## 3. Lexical Conventions

### 3.1 Whitespace and comments

v1 uses only line comments:

```txt
# comment until end of line
```

Semicolons are not used.

Newlines act as statement separators, with these exceptions:

- inside `(` ... `)`, newlines are non-significant
- a newline immediately after `=`, `,`, or a binary operator does not end the statement

Within a block, statements are separated by newlines. Multiple statements on one line are not part of v1.

### 3.2 Identifiers and keywords

Identifiers are ASCII-only and match:

```txt
[A-Za-z_][A-Za-z0-9_]*
```

Reserved keywords are:

- `fn`
- `mut`
- `if`
- `else`
- `while`
- `true`
- `false`

### 3.3 Literals

The minimal v1 literal set is:

- decimal integer literals
- boolean literals `true` and `false`
- string literals `"..."` with escapes `\\`, `\"`, `\n`, and `\t`

Raw strings and multiline strings are not part of v1.

---

## 4. Shadowing Policy

Shadowing is prohibited in v1.

That means a new binding may not reuse a name that already exists in any enclosing scope.

Example:

```txt
x = 1

if cond {
  x = 2   # error: shadowing prohibited
}
```

This keeps `=` easier to read because it reduces ambiguity between:

- new binding
- update
- shadowing

---

## 5. Operators

The v1 operator set is:

- unary: `-`, `!`
- multiplicative: `*`, `/`
- additive: `+`, `-`
- comparison: `<`, `<=`, `>`, `>=`
- equality: `==`, `!=`

All binary operators are left-associative.

Precedence, from strongest to weakest:

1. postfix call
2. unary
3. multiplicative
4. additive
5. comparison
6. equality

`=` is not an expression operator. It appears only in assign-like statements.

---

## 6. Functions

## 6.1 Function declarations

```txt
fn add(a, b) {
  a + b
}
```

A function declaration introduces a new immutable binding in the current scope.

Semantically, this is close to:

```txt
add = fn(a, b) {
  a + b
}
```

### 6.2 Anonymous functions

Anonymous functions are expressions.

```txt
double = fn(x) {
  x * 2
}
```

### 6.3 Function parameter rules

Function parameters are introduced as immutable bindings in the function scope.

Example:

```txt
fn add_one(x) {
  x + 1
}

fn bad(x) {
  x = x + 1   # error: parameters are immutable
}
```

### 6.4 Return value

The return value of a function is the value of the final expression in its body.

Function bodies are value blocks, so every function body ends with a final expression.

Example:

```txt
fn abs(x) {
  if x < 0 {
    -x
  } else {
    x
  }
}
```

`return` is not required in v1.

### 6.5 Closure capture

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

---

## 7. Type Inference and Type Annotations

## 7.1 General policy

Type annotations should be omitted whenever possible.

The language should infer types automatically unless inference is ambiguous or impractical.

### 7.2 Built-in types and source type expressions

The minimal v1 built-in types are:

- `Int`
- `Bool`
- `String`

Function types exist in the implementation model, but they are not part of source-level type syntax in v1.

Therefore, source `type_expr` is restricted to:

```ebnf
type_expr := "Int" | "Bool" | "String"
```

There are no generics, no user-written type variables, and no polymorphic type syntax in v1.

### 7.3 Operator typing rules

The built-in operator typing rules are:

- unary `-` : `Int -> Int`
- unary `!` : `Bool -> Bool`
- `+`, `-`, `*`, `/` : `Int -> Int -> Int`
- `<`, `<=`, `>`, `>=` : `Int -> Int -> Bool`
- `==`, `!=` : allowed only for identical primitive types among `Int`, `Bool`, and `String`

String concatenation is not part of v1. Therefore, `+` is `Int`-only.

### 7.4 Local bindings

Local variable types are inferred from the right-hand side.

Example:

```txt
x = 1        # x : Int
name = "a"  # name : String
```

Mutable updates must preserve the original type exactly. v1 does not define implicit conversions or subtyping.

### 7.5 Function return types

A function's return type is inferred from the final expression, or from all branches if control flow branches.

All branches must agree on the same type.

### 7.6 Parameter type inference

Parameter types may be omitted if they can be inferred uniquely from usage.

Example:

```txt
fn inc(x) {
  x + 1
}
```

If the language has only `Int` arithmetic here, `x` may be inferred as `Int`.

### 7.7 Inference boundary

v1 intentionally uses local-only inference.

Allowed:

- infer local binding types from the right-hand side
- infer function parameter types from operators and other constraints inside the same function body
- infer function return types from the function body
- infer `if` expression result types from branch agreement

Disallowed:

- inferring a callee parameter type from call sites alone
- propagating constraints across unrelated top-level declarations
- polymorphic generalization

### 7.8 When type annotations are required

Type annotations are required only when inference cannot determine a unique type.

Example:

```txt
fn id(x) {
  x
}
```

This is ambiguous and requires annotation, for example:

```txt
fn id_int(x: Int) -> Int {
  x
}
```

### 7.9 Recursion rule

To keep the implementation simpler in v1:

- non-recursive functions: infer as much as possible
- recursive functions: require at least one annotation on either
  - a parameter type, or
  - the return type
- mutually recursive functions: require explicit signatures

Example:

```txt
fn fact(n: Int) {
  if n == 0 {
    1
  } else {
    n * fact(n - 1)
  }
}
```

---

## 8. Grammar (EBNF)

```ebnf
program      := stmt*

stmt         := assign_like_stmt
              | func_decl
              | if_stmt
              | while_stmt
              | expr_stmt

assign_like_stmt := "mut" IDENT "=" expr
                  | IDENT "=" expr

func_decl    := "fn" IDENT "(" params? ")" return_annot? value_block
return_annot := "->" type_expr

params       := param ("," param)*
param        := IDENT
              | IDENT ":" type_expr

if_stmt      := "if" expr stmt_block ("else" stmt_block)?
while_stmt   := "while" expr stmt_block
stmt_block   := "{" stmt* "}"

expr_stmt    := expr

expr         := if_expr
              | equality_expr

equality_expr := comparison_expr (("==" | "!=") comparison_expr)*
comparison_expr := additive_expr (("<" | "<=" | ">" | ">=") additive_expr)*
additive_expr := multiplicative_expr (("+" | "-") multiplicative_expr)*
multiplicative_expr := unary_expr (("*" | "/") unary_expr)*
unary_expr   := ("-" | "!") unary_expr
              | call_expr

call_expr    := primary_expr ("(" args? ")")*
primary_expr := literal
              | IDENT
              | anon_fn
              | "(" expr ")"
args         := expr ("," expr)*

anon_fn      := "fn" "(" params? ")" return_annot? value_block
if_expr      := "if" expr value_block "else" value_block
value_block  := "{" non_expr_stmt* expr "}"
non_expr_stmt := assign_like_stmt
               | func_decl
               | if_stmt
               | while_stmt
literal      := INT_LIT
              | STRING_LIT
              | "true"
              | "false"
```

### Important note

The grammar intentionally does **not** distinguish syntactically between:

- introducing a new immutable binding
- updating an existing mutable binding

That distinction is made by static semantic rules during name resolution.

To keep final-expression return syntax unambiguous in v1, value-producing blocks reserve a single trailing expression slot. Earlier statements inside a value block must be non-expression statements.

---

## 9. Static Semantic Rules Summary

```txt
Rule A: mut x = e
- if x does not exist in current scope: introduce mutable binding
- otherwise: error

Rule B: x = e
- if x does not exist in current scope: introduce immutable binding
- if x exists as mutable in current scope: update it
- if x exists as any immutable name in current scope: error

Rule C: shadowing
- introducing a new binding with the same name as any enclosing-scope binding is an error

Rule D: outer-scope mutation
- reading from outer scopes is allowed
- mutating outer-scope bindings across a function boundary is disallowed in v1
```

---

## 10. Execution-Oriented Summary

- `if` without `else` is statement-only
- `while` is statement-only
- the top-level program does not produce a value

---

## 11. Examples

### 8.1 Valid program

```txt
base = 10
mut total = 0

fn plus_base(x) {
  x + base
}

fn sum_to(n: Int) {
  mut i = 0
  mut acc = 0

  while i < n {
    acc = acc + i
    i = i + 1
  }

  acc
}
```

### 8.2 Errors

```txt
x = 1
x = 2          # error: immutable update
```

```txt
mut y = 1
mut y = 2      # error: redefinition in current scope
```

```txt
z = 1
if cond {
  z = 2        # error: shadowing prohibited
}
```

```txt
mut total = 0
fn add_total(x) {
  total = total + x   # error: outer-scope mutation prohibited in v1
}
```

---

## 12. Known Trade-off

Because `set` is intentionally not used, this design accepts one notable trade-off:

```txt
mut count = 0
coutn = count + 1
```

If `coutn` is undefined in the current scope, this becomes a new immutable binding rather than an update error.

Therefore, a practical compiler or linter should ideally warn on:

- suspiciously similar identifiers
- likely typos in newly introduced names

---

## 13. Current Design Summary

This v1 language currently has the following shape:

- no `let`
- immutable by default
- `mut` for mutable bindings
- `=` means either new immutable binding or mutable update depending on current-scope resolution
- shadowing prohibited
- lexical scoping
- function names are immutable bindings
- function parameters are immutable
- function return value is the final expression
- comments use `#`
- statements are separated by newlines
- source type annotations are limited to `Int`, `Bool`, and `String`
- type inference is local-only
- type annotations are omitted unless inference fails
- recursive functions require limited annotation

---

## 14. Next Possible Steps

Natural next topics for the spec:

1. minimal type system definition
2. type inference algorithm sketch
3. control flow semantics
4. error message rules
5. module system
6. parser and name resolver architecture
