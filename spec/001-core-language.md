# Core Language Specification v1

Derived from [mini-language-spec-v1.md](../mini-language-spec-v1.md). This document defines the surface language and the core execution-facing rules. Name resolution, typing, and function-specific rules are split into companion documents:

- [002-name-resolution.md](./002-name-resolution.md)
- [003-typing.md](./003-typing.md)
- [004-functions.md](./004-functions.md)
- [005-records.md](./005-records.md)

## 1. Design Constraints

The v1 language is intentionally small and follows these constraints:

- no `let`
- immutable bindings by default
- `mut` introduces mutable bindings
- `x = e` is resolved statically as either a new immutable binding or an update
- shadowing is prohibited
- outer-scope updates are prohibited
- type annotations are omitted unless inference cannot determine a unique type
- higher-order functions are supported

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
- if `x` is already any immutable name in the current scope, it is an error

For `x = e`, current-scope immutable names include ordinary immutable bindings, function names, and parameters.

The exact static resolution rules are normative in [002-name-resolution.md](./002-name-resolution.md).

## 3. Blocks and Scope

The language uses lexical scoping.

- every block `{ ... }` creates a new scope
- every function body creates a new scope
- name lookup prefers the nearest enclosing scope
- bindings are visible from their declaration point to the end of the enclosing block

Within a single function body, an inner block may update a mutable binding introduced by an enclosing block in the same function.

Across a function boundary, outer bindings may be read from inner scopes, but v1 does not allow updating outer-scope bindings.

## 4. Statements and Expressions

The language has the following core constructs:

- binding/update statements
- record declarations
- function declarations
- `if` statements and `if` expressions
- `while` statements
- expression statements
- record literals
- field access and chained dot calls

To keep the grammar unambiguous, v1 distinguishes:

- statement blocks, which contain ordinary statements
- value blocks, which end in a required final expression and therefore produce a value

Function bodies and `if` expressions use value blocks.

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

## 5. Lexical Conventions

### 5.1 Whitespace and comments

v1 uses line comments only:

```txt
# comment until end of line
```

Semicolons are not used.

Newlines are statement separators, with the following exceptions:

- inside `(` ... `)`, newlines are non-significant
- a newline immediately following `=`, `,`, or a binary operator does not terminate the statement

Within a block, statements are separated by newlines. Multiple statements on one line are not allowed in v1.

### 5.2 Identifiers and keywords

Identifiers are ASCII-only and match:

```txt
[A-Za-z_][A-Za-z0-9_]*
```

Reserved keywords are:

- `fn`
- `record`
- `mut`
- `if`
- `else`
- `while`
- `true`
- `false`

### 5.3 Literals

The minimal v1 literal set is:

- decimal integer literals
- boolean literals `true` and `false`
- string literals `"..."` with escapes `\\`, `\"`, `\n`, and `\t`

Raw strings and multiline strings are not part of v1.

## 6. Operators and Precedence

The v1 operator set is:

- unary: `-`, `!`
- multiplicative: `*`, `/`
- additive: `+`, `-`
- comparison: `<`, `<=`, `>`, `>=`
- equality: `==`, `!=`

All binary operators are left-associative.

Precedence, from strongest to weakest:

1. postfix field access / chained call / ordinary call
2. unary
3. multiplicative
4. additive
5. comparison
6. equality

`=` is not an expression operator. It appears only in assign-like statements.

The dot operator has two surface forms:

- `expr.name` for field access
- `expr.name(args...)` for method-style or UFCS-style chained call

Because record fields cannot have function type in v1, dot syntax has only those two intended meanings.

## 7. Grammar Sketch

This is a v1-oriented EBNF sketch. `type_expr` is defined abstractly here and constrained further by [003-typing.md](./003-typing.md). Records and dot expressions are introduced here, with detailed semantics in [005-records.md](./005-records.md).

```ebnf
program           := top_item*
top_item          := record_decl
                   | stmt

stmt              := assign_like_stmt
                   | func_decl
                   | if_stmt
                   | while_stmt
                   | expr_stmt

assign_like_stmt  := "mut" IDENT "=" expr
                   | IDENT "=" expr

record_decl       := "record" IDENT "{" record_field_decl* "}"
record_field_decl := IDENT ":" type_expr

func_decl         := "fn" IDENT "(" params? ")" return_annot? value_block
return_annot      := ":" type_expr
type_expr_list    := type_expr ("," type_expr)*

params            := param ("," param)*
param             := IDENT
                   | IDENT ":" type_expr

while_stmt        := "while" expr stmt_block
if_stmt           := "if" expr stmt_block ("else" stmt_block)?
expr_stmt         := expr

expr              := if_expr
                   | equality_expr

if_expr           := "if" expr value_block "else" value_block
equality_expr     := comparison_expr (("==" | "!=") comparison_expr)*
comparison_expr   := additive_expr (("<" | "<=" | ">" | ">=") additive_expr)*
additive_expr     := multiplicative_expr (("+" | "-") multiplicative_expr)*
multiplicative_expr := unary_expr (("*" | "/") unary_expr)*
unary_expr        := ("-" | "!") unary_expr
                   | postfix_expr
postfix_expr      := primary_expr postfix_tail*
postfix_tail      := "(" args? ")"
                   | "." IDENT ("(" args? ")")?
args              := expr ("," expr)*

primary_expr      := literal
                   | IDENT
                   | record_lit
                   | anon_fn
                   | "(" expr ")"
record_lit        := IDENT "{" record_field_init* "}"
record_field_init := IDENT ":" expr
literal           := INT_LIT
                   | STRING_LIT
                   | "true"
                   | "false"

anon_fn           := "fn" "(" params? ")" return_annot? value_block
stmt_block        := "{" stmt* "}"
value_block       := "{" non_expr_stmt* expr "}"
non_expr_stmt     := assign_like_stmt
                   | func_decl
                   | if_stmt
                   | while_stmt
```

In a value block, only non-expression statements may appear before the final expression. This reserves a single trailing expression slot and keeps final-expression return syntax deterministic.

## 8. Execution-Oriented Summary

The core language model is:

- `mut x = e` introduces a new mutable binding
- `x = e` either introduces a new immutable binding or updates an existing mutable binding in the current scope
- immutable bindings cannot be updated
- function names are ordinary immutable bindings
- function parameters are immutable bindings
- functions are ordinary values and may be passed as arguments
- record declarations introduce nominal type names
- `expr.name` is field access only
- `expr.name(...)` is chained call syntax
- the value of a function body is the final expression in that body
- `if` without `else` is statement-only
- `while` is statement-only
- the top-level program does not produce a value

## 9. Examples

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
