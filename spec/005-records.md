# Records and Dot Expressions Specification v1

This document defines nominal records, record literals, field access, record update, chained dot calls, and their interaction with receiver-style functions.

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
- v1 does not include per-field visibility modifiers
- record visibility is controlled at the top-level record declaration
- a visible record is transparent: its declared fields are nameable wherever the record shape is visible

Per-field visibility is not a committed v1 feature. It is also not the preferred first answer for public type names with hidden representations; future `opaque record` or `opaque type` designs should be considered first.

Example:

```txt
package app::users

pub record User {
  name: String
  age: Int
}

pub fn display_name(user: User): String {
  user.name
}
```

Here `User` is public and its fields are part of the public record shape. Other packages can name `User`, construct it with a record literal, read its fields, and use `record.with(...)` on those fields.

If a representation should be hidden inside a module or package, the v1 recommendation is to keep the record non-public and expose functions that do not leak that non-public type across a wider visibility boundary. Passing a hidden representation across package boundaries should be handled by a future opaque representation feature, not by making transparent records more complicated in v1.

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
- the record type and record shape must be visible at the literal site

A public record is transparent in v1, so importing packages may construct it with a record literal when the public record type is visible.

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

Field access is allowed when the static type is a visible record type whose shape is available in the current context. v1 has no per-field visibility restriction beyond record-level visibility.

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
- the record type and record shape must be visible at the update site
- unspecified fields are preserved from the original value
- the update is non-destructive

Example:

```txt
older = user.with(age: user.age + 1)
```

This creates a new `User` value with only `age` replaced.

## 6. Chained Dot Calls

A chained call has one of these forms:

```txt
expr.name(arg1, arg2, ...)
expr.alias::name(arg1, arg2, ...)
```

This always means method-style or UFCS-style chained call syntax.

Resolution order:

1. for `expr.name(...)`, try the visible ordinary function binding named `name`
2. for `expr.alias::name(...)`, resolve `alias::name` as a qualified ordinary function reference
3. if that function is receiver-style and applicable to the type of `expr`, resolve as that receiver function
4. otherwise, if the corresponding ordinary call is valid, resolve by UFCS-style desugaring
5. otherwise, reject the expression

Example:

```txt
10.start().inc().inc().value.double()
```

may be understood as repeated UFCS-style desugaring, equivalent to:

```txt
double(inc(inc(start(10))).value)
```

Likewise, package-qualified chained calls follow the same rule:

```txt
user.users::birthday().age
```

is equivalent to:

```txt
users::birthday(user).age
```

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
- `expr.name(...)` and `expr.alias::name(...)` always mean chained call
- function-valued field call is not part of the v1 language model

This restriction is separate from higher-order functions.

Muga v1 still allows function values in ordinary bindings and parameter positions. The prohibition applies only to record fields.

## 8. Receiver Parameters

Receiver-style functions use an explicitly annotated first parameter of record type.

Example:

```txt
fn display_name(self: User): String {
  self.name
}
```

v1 rules:

- the receiver parameter must be first
- the receiver parameter must have an explicit record-type annotation
- any identifier may be used for that parameter; `self` is conventional but not required
- that parameter is still just an immutable parameter binding in the function body
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
fn len(self: String): Int { ... }   // duplicate binding in v1
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

## 11. Encapsulation Example

This pattern is the preferred way to build small abstractions without introducing classes:

```txt
package app::counter

pub record Counter {
  value: Int
}

pub fn new_counter(): Counter {
  Counter {
    value: 0
  }
}

pub fn inc(counter: Counter): Counter {
  counter.with(value: counter.value + 1)
}

pub fn value(counter: Counter): Int {
  counter.value
}
```

Users of `app::counter` can hold a `Counter` value and call public functions, but cannot directly access `counter.value` outside the defining module. This keeps Muga function-centered while still allowing file-sized encapsulation.

## 12. Higher-Order Functions Remain Allowed

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

## 13. Generic Records

Generic record declarations are part of the v1 target.

```txt
record Box[T] {
  value: T
}
```

An instantiated generic record type may be used in type annotations:

```txt
box: Box[Int] = Box {
  value: 1
}
```

Record literals use the record name itself. For generic records, type arguments are inferred from field values or from an expected type such as a binding annotation, parameter type, return type, or surrounding expression. When an expected generic record type is known, the instantiated field types are also available to contextual field values such as `[]`, `Map.empty()`, and `Option::None`. Explicit record-literal type arguments such as `Box[Int] { ... }` are not part of the v1 surface syntax.

Generic record fields still follow the same record rules:

- fields may not have function type in v1
- v1 has no per-field visibility modifiers
- field access remains `expr.name`
- record update remains `expr.with(...)`

The full generics policy is defined in [009-generics.md](./009-generics.md).

## 14. Notes for Future Extensions

The current design leaves room for future work on:

- mutable or persistent-update record operations
- opaque records or opaque types for public type names with hidden representations
- per-field visibility only if concrete code shows that opaque representations and transparent records are not enough

It deliberately does not leave room for class-style method ownership,
behavior-conformance dispatch, or limited overloading keyed by receiver type in
ordinary Muga code. Keep behavior in ordinary functions and use package
qualification, explicit wrapper functions, higher-order functions, or enums
with `match` when different types need related operations.

### 14.1 Opaque Representation Candidate

Some values should expose a public type name without exposing their fields or construction form. Examples include `Session`, `DbConnection`, `HttpClient`, `Parser`, and `TokenStream`.

These are not best modeled as "a record where a few fields are private". They are better modeled as a type with a hidden representation and public functions.

Future-only `opaque record` sketch:

```txt
package app::sessions

pub opaque record Session {
  id: String
  token: String
  expires_at: Int
}

pub fn new_session(id: String, token: String): Session {
  Session {
    id: id
    token: token
    expires_at: 0
  }
}

pub fn session_id(session: Session): String {
  session.id
}
```

Inside the declaring module, `Session` behaves like a record. From another package, only the type name and public functions are visible:

```txt
package app::ui

import app::sessions

pub fn label(session: sessions::Session): String {
  sessions::session_id(session)
}

pub fn invalid_access(session: sessions::Session): String {
  session.id
}

pub fn invalid_literal(): sessions::Session {
  sessions::Session {
    id: "ada"
    token: "secret"
    expires_at: 0
  }
}
```

`invalid_access` would be rejected because the representation is hidden. `invalid_literal` would be rejected because importing packages cannot construct opaque records directly.

For external resources or runtime-backed values, the representation may not be a Muga record at all. A future `opaque type` is a better fit:

```txt
package std::db

pub opaque type DbConnection

pub fn connect(url: String): Result[DbConnection, DbError]
pub fn execute(connection: DbConnection, sql: String): Result[Int, DbError]
```

`opaque record` is useful when the implementation is ordinary Muga data but should remain hidden. `opaque type` is useful when the implementation is a VM/runtime/native handle, or when Muga should not commit to a source-level field layout.

### 14.2 Per-Field Visibility Candidate

Per-field visibility should not be the next representation-hiding feature. It may be reconsidered only if concrete code needs a partially transparent public record: a type whose name and some fields are public, while other fields remain hidden.

If per-field visibility is ever added, the recommended constraints are:

- field visibility must not exceed the record's own visibility
- writing `pub` fields inside a `pkg` or module-private record should be a compile-time error
- writing `pkg` fields inside a module-private record should be a compile-time error
- a public record with non-public fields could not be constructed, read, or updated through those fields outside their visibility boundary
- constructor-style functions should be used when non-public fields are present

Records remain data declarations. They do not participate in behavior
conformance, conformance-based dispatch, or overloaded method lookup.
