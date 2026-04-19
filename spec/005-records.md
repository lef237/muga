# Records and Dot Expressions Specification v1

Derived from [mini-language-spec-v1.md](../mini-language-spec-v1.md). This document defines nominal records, record literals, field access, record update, chained dot calls, and their interaction with receiver-style functions.

## 1. Core Direction

Muga remains function-centered.

v1 explicitly does not introduce:

- classes
- member ownership semantics
- method dispatch as a separate semantic category

Instead, v1 uses:

- records as concrete named data containers
- ordinary functions as the place where operations are defined
- dot syntax for field access, chained call surface syntax, and record update

## 2. Record Declarations

A record declaration has the form:

```txt
record User {
  name: String
  age: Int
}
```

v1 rules:

- record declarations are top-level only
- a record introduces a nominal type name
- field names must be unique within the record
- field order is declaration order, but field access is by name
- record fields must not have function type in v1

## 3. Record Literals

A record literal has the form:

```txt
User {
  name: "Ada"
  age: 20
}
```

v1 rules:

- the type name must resolve to a declared record
- every declared field must be provided exactly once
- extra fields are errors
- field initializers are checked against the declared field types

## 4. Field Access

A field access has the form:

```txt
expr.name
```

This always means "read field `name` from the value of `expr`".

Examples:

```txt
user.name
point.x
config.port
```

In v1, field access is read-only syntax. Assignment through field access such as `user.name = "Ada"` is not part of v1.

## 5. Record Update

A record update has the form:

```txt
expr.with(field1: value1, field2: value2)
```

v1 rules:

- the base expression must have a record type
- the result has the same record type as the base expression
- each mentioned field must exist on that record type
- each mentioned field may appear at most once
- at least one field must be updated
- each replacement expression must match the declared field type
- unspecified fields are preserved from the original value
- the update is non-destructive

Example:

```txt
older = user.with(age: user.age + 1)
```

This creates a new `User` value with only `age` replaced.

## 6. Chained Dot Calls

A chained call has the form:

```txt
expr.name(arg1, arg2, ...)
```

This always means method-style or UFCS-style chained call syntax.

Resolution order:

1. try the visible ordinary function binding named `name`
2. if that binding is receiver-style and applicable to the type of `expr`, resolve as that receiver function
3. otherwise, if `name(expr, arg1, arg2, ...)` is a valid ordinary function call, resolve by UFCS-style desugaring
4. otherwise, reject the expression

Because v1 has no overloading, there is at most one visible ordinary function named `name`.

## 7. No Function-Valued Fields in v1

Record fields may not have function type in v1.

Therefore the following is invalid:

```txt
record User {
  formatter: String -> String
}
```

This keeps the meaning of dot expressions stable:

- `expr.name` always means field access
- `expr.name(...)` always means chained call
- function-valued field call is not part of the v1 language model

This restriction is separate from higher-order functions.

Muga v1 still allows function values in ordinary bindings and parameter positions. The prohibition applies only to record fields.

## 8. Receiver Parameters

Receiver-style functions use `self: Type` as the first parameter.

Example:

```txt
fn display_name(self: User): String {
  self.name
}
```

v1 rules:

- the receiver parameter must be first
- the receiver parameter must be written literally as `self: Type`
- the receiver type annotation is mandatory
- `self` is still just an immutable parameter binding in the function body
- receiver-style functions are still ordinary named functions

The ordinary call form remains valid:

```txt
display_name(user)
```

and chained-call syntax may desugar to the same call:

```txt
user.display_name()
```

## 9. v1 Limitation: No Receiver Overloading

The current v1 model keeps one ordinary function namespace and does not add overloading by receiver type.

Therefore, the following is invalid in the same scope:

```txt
fn len(self: List): Int { ... }
fn len(self: String): Int { ... }   # duplicate binding in v1
```

This is the main short-term limitation of the receiver-style design under the current no-overloading policy.

## 10. Short Example

```txt
record User {
  name: String
  age: Int
}

fn display_name(self: User): String {
  self.name
}

user = User {
  name: "Ada"
  age: 20
}

user.name
user.with(age: user.age + 1)
user.display_name()
```

## 11. Higher-Order Functions Remain Allowed

The following is valid in principle even though function-valued record fields are not:

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

## 12. Notes for Future Extensions

The current design leaves room for future work on:

- protocol/trait-like dispatch
- limited overloading keyed by receiver type
- mutable or persistent-update record operations
- generic record declarations
