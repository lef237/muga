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
- record fields participate in the package/module visibility model
- a field without a visibility modifier is module-private in package mode
- `pkg` fields are visible inside the same package
- `pub` fields are visible from importing packages

The current compiler implementation does not yet enforce field visibility. This section defines the target design before package interfaces harden.

Example:

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
```

Here `Counter` is public, but `value` is not. Other packages can name `Counter` and call public functions that return it, but they cannot directly read, construct, or update `value`.

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
- the current module must be allowed to name every initialized field

If a public record has private fields, code outside the declaring module cannot construct it directly with a record literal. The record should provide constructor-style functions instead.

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

Field access is allowed only when the current module is allowed to see that field:

- module-private fields are visible only in the declaring module/file
- `pkg` fields are visible in the same package
- `pub` fields are visible from importing packages

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
- the current module must be allowed to name each updated field
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

An instantiated generic record type may be used in record literals and type annotations:

```txt
box = Box[Int] {
  value: 1
}
```

Generic record fields still follow the same record rules:

- fields may not have function type in v1
- field visibility follows the package/module model
- field access remains `expr.name`
- record update remains `expr.with(...)`

The full generics policy is defined in [009-generics.md](./009-generics.md).

## 14. Notes for Future Extensions

The current design leaves room for future work on:

- protocol-like dispatch, if later justified by concrete examples
- limited overloading keyed by receiver type
- mutable or persistent-update record operations

The policy for protocol-like abstractions is defined in [012-protocols-deferred.md](./012-protocols-deferred.md).
