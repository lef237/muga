# Error Catalog v1

This document defines the expected diagnostic categories for the v1 split specification. The wording may vary by implementation, but the category and trigger condition should remain stable.

## E001: Immutable Update

Trigger:

- `x = e` where `x` is an immutable binding in the current scope

Recommended message:

```txt
cannot update immutable binding `x`
```

Referenced examples:

- [examples/invalid/001-immutable-update.md](./examples/invalid/001-immutable-update.md)

## E002: Duplicate Binding In Current Scope

Trigger:

- `mut x = e` where `x` already exists in the current scope
- `fn f(...) { ... }` where `f` already exists in the current scope
- duplicate parameter names within one parameter list

Recommended message:

```txt
duplicate binding `x` in the current scope
```

Referenced examples:

- [examples/invalid/002-duplicate-mutable-binding.md](./examples/invalid/002-duplicate-mutable-binding.md)

## E003: Shadowing Prohibited

Trigger:

- introducing a new binding whose name already exists in an enclosing scope

This includes:

- `mut x = e` in an inner scope when `x` exists outside
- `x = e` in an inner scope when it would otherwise introduce a new immutable binding that collides with an enclosing immutable name
- function declarations and parameters that reuse an enclosing name

Recommended message:

```txt
shadowing is prohibited for `x`
```

Referenced examples:

- [examples/invalid/003-shadowing-in-block.md](./examples/invalid/003-shadowing-in-block.md)

## E004: Outer-Scope Mutation Prohibited

Trigger:

- `x = e` in an inner scope where `x` resolves to a mutable binding in an enclosing scope

Recommended message:

```txt
cannot update outer-scope mutable binding `x` in v1
```

Referenced examples:

- [examples/invalid/004-outer-scope-mutation.md](./examples/invalid/004-outer-scope-mutation.md)

## E005: Annotation Required

Trigger:

- a function parameter type is not uniquely inferable
- a function return type is not uniquely inferable

Recommended message:

```txt
type annotation required because inference is not unique
```

Referenced examples:

- [examples/invalid/005-ambiguous-identity.md](./examples/invalid/005-ambiguous-identity.md)

## E006: Recursive Function Requires Annotation

Trigger:

- a directly recursive function has neither an annotated parameter nor an explicit return type

Recommended message:

```txt
recursive function requires at least one parameter or return type annotation
```

Referenced examples:

- [examples/invalid/006-unannotated-recursion.md](./examples/invalid/006-unannotated-recursion.md)

## E007: Mutual Recursion Requires Explicit Signatures

Trigger:

- a mutually recursive function group lacks explicit signatures

Recommended message:

```txt
mutually recursive functions require explicit signatures in v1
```

Referenced examples:

- [examples/invalid/007-unannotated-mutual-recursion.md](./examples/invalid/007-unannotated-mutual-recursion.md)
