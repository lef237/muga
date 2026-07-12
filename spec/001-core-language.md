# Core Language Specification

Status: current language specification; see
[LANGUAGE.md](../LANGUAGE.md#specification-status).

This document defines the surface language and the core execution-facing rules. Name resolution, typing, and function-specific rules are split into companion documents:

- [002-name-resolution.md](./002-name-resolution.md)
- [003-typing.md](./003-typing.md)
- [004-functions.md](./004-functions.md)
- [005-records.md](./005-records.md)

## 1. Design Constraints

The language is intentionally small and follows these constraints:

- no `let`
- immutable bindings by default
- `mut` introduces mutable bindings
- `x = e` is resolved statically as either a new immutable binding or an update
- shadowing is prohibited
- outer-scope updates are prohibited
- type annotations are omitted unless inference cannot determine a unique type
- higher-order functions are supported

## 2. Syntax Marker Discipline

Muga prefers one primary conceptual role per symbol.

This does not require every punctuation character to appear in only one grammar production. It means the same visual marker should not carry several unrelated meanings that must be remembered from context.

Guidelines:

- a symbol should have one primary conceptual role
- related surface forms are allowed only when the relationship is obvious
- unrelated roles should use different syntax, even if the result is longer
- keywords are acceptable when they make code easier to read
- compact notation must not override local readability

Current examples:

- `:` marks type-related annotation positions
- `->` appears inside function type expressions
- `::` marks package-qualified access
- `.` is reserved for field access and chained-call surface syntax
- `=` is statement-level binding/update syntax, not an expression operator

Muga currently does not plan pointer-like, reference-like, ownership, or borrowing syntax in ordinary source code.

If such concepts are ever reconsidered, they must follow this rule.

In particular, Muga should avoid using one marker for several context-dependent roles such as:

- type constructor
- value dereference
- address creation
- mutation marker
- pattern binding modifier

## 3. Core Binding Forms

### 3.1 Mutable binding

```txt
mut x = e
```

This form always attempts to introduce a new mutable binding in the current scope.

### 3.2 Plain assignment-like form

```txt
x = e
```

This form is parsed uniformly and resolved later:

- if `x` is not defined in the current scope, it may introduce a new immutable binding
- if `x` is already a mutable binding in the current scope, it updates that binding
- if `x` is already any immutable name in the current scope, it is an error

For `x = e`, current-scope immutable names include ordinary immutable bindings, function names, and parameters.

The exact static resolution rules are normative in [002-name-resolution.md](./002-name-resolution.md).

### 3.3 Update-Ambiguity Evaluation

Using `x = e` for both immutable introduction and mutable update keeps source
small, but a misspelled update can introduce a new immutable binding. Ordinary
unused-binding warnings are the first mitigation and should be tested in
representative programs before a specialized lint or syntax change is designed.

A similar-name warning is optional, not a baseline requirement. Consider it
only if real mistakes routinely escape unused warnings, and then limit it to a
plain binding introduction that closely resembles an earlier mutable binding
in the same function. Broad comparisons across every visible identifier would
produce noise for intentional names such as `item` / `items`.

An explicit update form should be reconsidered only if those diagnostics leave
material correctness problems. No spelling is currently preferred. In
particular, `set` should not be reserved as the leading candidate because it is
visually close to a future `Set[T]` type and overlaps with collection
`.set(...)` vocabulary. If an explicit form is eventually adopted, it must
reject an unresolved target and replace ordinary `x = e` updates rather than
create two equivalent mutable-update spellings.

## 4. Blocks and Scope

The language uses lexical scoping.

- every block `{ ... }` creates a new scope
- every function body creates a new scope
- name lookup prefers the nearest enclosing scope
- bindings are visible from their declaration point to the end of the enclosing block

Within a single function body, an inner block may update a mutable binding introduced by an enclosing block in the same function.

Across a function boundary, outer bindings may be read from inner scopes, but Muga does not allow updating outer-scope bindings.

## 5. Statements and Expressions

The language has the following core constructs:

- binding/update statements
- record declarations
- function declarations
- `if` statements and `if` expressions
- `while` statements
- `for item in list` statements over `List[T]`
- `break` and `continue` statements inside loops
- `return expr` statements inside functions
- expression statements
- record literals
- field access, chained dot calls, and record updates
- package/module visibility modifiers in package mode

To keep the grammar unambiguous, Muga distinguishes:

- statement blocks, which contain ordinary statements
- value blocks, which end in a required final expression or terminal `return expr`

Function bodies and `if` expressions use value blocks. `return expr` exits the nearest named or anonymous function and is not allowed at top level. `break` and `continue` target the nearest enclosing loop and do not cross a nested named or anonymous function boundary.

`if` without `else` is statement-only. `if` with `else` may appear in expression position and yields the branch result value. `else if` is accepted as readable nested-`if` sugar in both forms; value-producing chains must still end in a final `else`.

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

## 6. Lexical Conventions

### 6.1 Whitespace and comments

Muga uses line comments only:

```txt
// comment until end of line
```

Semicolons are not used.

Newlines are statement separators, with the following exceptions:

- inside `(` ... `)`, newlines are non-significant
- a newline immediately following `=`, `,`, or a binary operator does not terminate the statement

Within a block, statements are separated by newlines. Multiple statements on one line are not allowed.

### 6.2 Identifiers and keywords

Identifiers are ASCII-only and match:

```txt
[A-Za-z_][A-Za-z0-9_]*
```

Reserved keywords are:

- `fn`
- `record`
- `enum`
- `match`
- `mut`
- `package`
- `import`
- `pub`
- `pkg`
- `as`
- `if`
- `else`
- `while`
- `for`
- `in`
- `break`
- `continue`
- `return`
- `try`
- `and`
- `or`
- `true`
- `false`

### 6.3 Literals

The minimal literal set is:

- decimal integer literals
- boolean literals `true` and `false`
- string literals `"..."` with escapes `\\`, `\"`, `\n`, and `\t`
- the unit literal `()`

Raw strings and multiline strings are not currently supported.

Integer literals are 64-bit signed. The accepted value range is `-2^63 ..= 2^63 - 1`. To accommodate `-2^63`, an integer literal that immediately follows a unary `-` and is not followed by `.` or `(` is parsed as a single signed literal. In every other position a positive integer literal must fit in `i64`.

### 6.4 Numeric Maturity Target

The current language is integer-only. Muga must either document a
deliberately integer-only application scope or specify an explicit `Float64`
type for general-purpose numeric and JSON work. A `Float64` design must cover
literals, arithmetic, explicit `Int` conversion, formatting, JSON
serialization, `NaN`, infinities, signed zero, equality, hashing, diagnostics,
and persisted interfaces. It must not introduce implicit cross-type numeric
conversion. Decimal money arithmetic should remain a distinct later type or
package rather than an implicit mode of binary floating point.

## 7. Operators and Precedence

The current operator set is:

- unary: `-`, `!`, `try`
- multiplicative: `*`, `/`
- additive: `+`, `-`
- comparison: `<`, `<=`, `>`, `>=`
- equality: `==`, `!=`
- boolean: `and`, `or`

All binary operators are left-associative. `and` and `or` are short-circuiting:
the right operand of `left and right` is evaluated only when `left` is `true`,
and the right operand of `left or right` is evaluated only when `left` is
`false`.

Precedence, from strongest to weakest:

1. postfix field access / chained call / ordinary call / indexing
2. unary and `try`
3. multiplicative
4. additive
5. comparison
6. equality
7. `and`
8. `or`

`=` is not an expression operator. It appears only in assign-like statements.

The dot operator has three surface forms:

- `expr.name` for field access
- `expr.name(args...)` or `expr.alias::name(args...)` for method-style or UFCS-style chained call
- `expr.with(field: value, ...)` for non-destructive record update

Because record fields cannot have function type, the dot operator keeps those three stable meanings without field-function-call ambiguity.

`try expr` unwraps `Result::Ok(value)` or returns `Result::Err(error)` early from the nearest function. Its precise type rules are specified in [013-enums-results.md](./013-enums-results.md).

## 8. Grammar Sketch

This is an EBNF sketch of the current grammar. `type_expr` is defined abstractly here and constrained further by [003-typing.md](./003-typing.md). Package-specific file grammar is defined in [006-packages.md](./006-packages.md). Records, enums, collections, and dot expressions are introduced here, with detailed semantics in companion specs.

```ebnf
program           := top_item*
top_item          := record_decl
                   | enum_decl
                   | stmt

stmt              := assign_like_stmt
                   | func_decl
                   | if_stmt
                   | while_stmt
                   | for_stmt
                   | break_stmt
                   | continue_stmt
                   | return_stmt
                   | expr_stmt

assign_like_stmt  := "mut" IDENT type_annot? "=" expr
                   | IDENT type_annot? "=" expr
type_annot        := ":" type_expr

record_decl       := "record" IDENT type_params? "{" record_field_decl* "}"
record_field_decl := IDENT ":" type_expr

enum_decl         := "enum" IDENT type_params? "{" enum_variant_decl* "}"
enum_variant_decl := IDENT
                   | IDENT "(" type_expr ")"

type_params       := "[" IDENT ("," IDENT)* "]"

func_decl         := "fn" IDENT type_params? "(" params? ")" return_annot? value_block
return_annot      := ":" type_expr
type_expr_list    := type_expr ("," type_expr)*

params            := param ("," param)*
param             := IDENT
                   | IDENT ":" type_expr

while_stmt        := "while" expr stmt_block
for_stmt          := "for" IDENT "in" expr stmt_block
if_stmt           := "if" expr stmt_block ("else" (stmt_block | if_stmt))?
break_stmt        := "break"
continue_stmt     := "continue"
return_stmt       := "return" expr
expr_stmt         := expr

expr              := if_expr
                   | match_expr
                   | or_expr

if_expr           := "if" expr value_block "else" (value_block | if_expr)
match_expr        := "match" expr "{" match_arm* "}"
match_arm         := variant_pattern "=>" expr
variant_pattern   := qualified_name
                   | qualified_name "(" pattern_payload ")"
pattern_payload   := IDENT
                   | "_"

or_expr           := and_expr ("or" and_expr)*
and_expr          := equality_expr ("and" equality_expr)*
equality_expr     := comparison_expr (("==" | "!=") comparison_expr)*
comparison_expr   := additive_expr (("<" | "<=" | ">" | ">=") additive_expr)*
additive_expr     := multiplicative_expr (("+" | "-") multiplicative_expr)*
multiplicative_expr := unary_expr (("*" | "/") unary_expr)*
unary_expr        := ("-" | "!" | "try") unary_expr
                   | postfix_expr
postfix_expr      := primary_expr postfix_tail*
postfix_tail      := "(" args? ")"
                   | "[" expr "]"
                   | ".with" "(" record_update_item ("," record_update_item)* ")"
                   | "." IDENT ("(" args? ")")?
                   | "." IDENT "::" IDENT "(" args? ")"
record_update_item := IDENT ":" expr
args              := expr ("," expr)*

primary_expr      := literal
                   | qualified_name
                   | list_lit
                   | record_lit
                   | anon_fn
                   | "(" expr ")"
qualified_name    := IDENT ("::" IDENT)*
list_lit          := "[" args? "]"
record_lit        := qualified_name "{" record_field_init* "}"
record_field_init := IDENT ":" expr
literal           := INT_LIT
                   | STRING_LIT
                   | "true"
                   | "false"
                   | "()"

anon_fn           := "fn" "(" params? ")" return_annot? value_block
stmt_block        := "{" stmt* "}"
value_block       := "{" value_block_stmt* (expr | return_stmt) "}"
value_block_stmt  := non_expr_stmt
                   | return_stmt
non_expr_stmt     := assign_like_stmt
                   | func_decl
                   | if_stmt
                   | while_stmt
                   | for_stmt
                   | break_stmt
                   | continue_stmt
```

In a value block, only non-expression statements may appear before the final expression or terminal `return expr`. This reserves a single trailing value slot and keeps final-expression return syntax deterministic. `break` and `continue` may appear before that final slot when the value block is inside a loop, but they are not valid terminal values.

### 8.1 Record literal disambiguation in conditions

The grammar above lists `record_lit` as one form of `primary_expr`, but the surface forms `if expr stmt_block`, `while expr stmt_block`, and `for item in expr stmt_block` are syntactically ambiguous when `expr` ends with an identifier and `stmt_block` begins with `{`. Muga resolves this ambiguity by disallowing a top-level record literal in the condition position of `if` / `while` and the iterable position of `for`. To use a record literal there, wrap it in parentheses:

```txt
if (P { x: 1 }) { ... }
while (P { x: 1 }) { ... }
for item in (P { x: 1 }) { ... }
```

Because every `if`/`while` condition must have type `Bool` and every `for` iterable must have type `List[T]`, a parenthesized record literal still fails type checking in these positions. The restriction therefore costs nothing in practice and preserves stable, locally readable surface syntax.

## 9. Execution-Oriented Summary

The core language model is:

- `mut x = e` introduces a new mutable binding
- `x = e` either introduces a new immutable binding or updates an existing mutable binding in the current scope
- immutable bindings cannot be updated
- function names are ordinary immutable bindings
- function parameters are immutable bindings
- functions are ordinary values and may be passed as arguments
- record declarations introduce nominal type names
- enum declarations introduce nominal sum types with qualified variants
- `expr.name` is field access only
- `expr.name(...)` and `expr.alias::name(...)` are chained call syntax
- `expr.with(field: value, ...)` is a record-only non-destructive update expression
- `expr[index]` indexes a list value
- `match` is exhaustive over enum variants in the current MVP
- `Variant(_)` discards a one-payload enum variant payload without introducing a binding; broad catch-all `_ =>` arms are not currently supported
- `try expr` propagates `Result[T, E]` errors from the nearest function
- the value of a function body is the final expression in that body
- `if` without `else` is statement-only
- `while` is statement-only
- `for item in list` is statement-only and iterates `List[T]` values in list order
- `break` and `continue` are statement-only and target the nearest enclosing loop
- the top-level program does not produce a value

## 10. Examples

Valid:

```txt
x = 1
mut total = 0
total = total + x
```

Invalid:

```txt
x = 1
x = 2   // error: immutable binding cannot be updated
```
