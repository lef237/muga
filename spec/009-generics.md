# Generics Specification v1

Status: v1 design draft. This document defines the intended Muga v1 generics MVP. The current Rust compiler does not implement this yet.

Generics are in scope for Muga v1, but only in a deliberately small form. The goal is to support practical typed code such as `List[T]`, `Option[T]`, `Map[K, V]`, reusable records, and simple reusable functions without introducing a large trait/typeclass system in the first version.

## 1. Design Goals

Generics should support:

- collection types such as `List[Int]` and `Map[String, User]`
- optional values such as `Option[User]`
- reusable records such as `Box[T]`
- reusable functions such as `fn id[T](value: T): T`
- annotation-light call sites
- fast local type checking
- stable package interfaces

Generics should not turn v1 into a whole-program inference language.

## 2. v1 Scope

Muga v1 includes:

- generic type expressions: `Name[T]`, `Name[K, V]`
- builtin generic types: `List[T]`, `Option[T]`, `Map[K, V]`
- generic record declarations
- generic function declarations
- local type-argument inference at function call sites
- generic signatures stored in package interfaces

Muga v1 does not include:

- explicit call-site type arguments
- trait bounds or protocol bounds
- typeclasses
- higher-kinded types
- const generics
- variance annotations
- subtyping
- user-defined specialization
- overloaded generic dispatch
- generic packages
- implicit polymorphic generalization of non-generic declarations
- polymorphic recursion

These exclusions are intentional. They keep the typechecker small, diagnostics simpler, and package compilation cache-friendly.

## 3. Syntax

Muga uses square brackets for type arguments and type parameters.

Type application:

```muga
List[Int]
Map[String, Int]
Option[User]
Box[String]
```

Generic record:

```muga
record Box[T] {
  value: T
}
```

Generic function:

```muga
fn id[T](value: T): T {
  value
}
```

Multiple type parameters:

```muga
record Pair[A, B] {
  first: A
  second: B
}

fn choose[T](left: T, right: T): T {
  left
}
```

Muga uses `[]` instead of `<>` because `<` and `>` are ordinary comparison operators.

## 4. Source Type Expressions

The v1 type expression grammar becomes:

```ebnf
type_expr          := function_type
function_type      := function_domain "->" type_expr
                    | non_function_type
function_domain    := non_function_type
                    | "(" type_expr_list? ")"
non_function_type  := type_primary type_args?
type_primary       := "Int"
                    | "Bool"
                    | "String"
                    | IDENT
type_args          := "[" type_expr_list "]"
type_expr_list     := type_expr ("," type_expr)*
```

Function types and generic types compose normally:

```muga
List[Int -> String]
Option[(Int, String) -> Bool]
Map[String, List[Int]]
```

## 5. Type Parameters

Type parameters are introduced by a generic declaration.

```muga
fn wrap[T](value: T): Box[T] {
  Box[T] {
    value: value
  }
}
```

Rules:

- type parameter names must be unique within the same type-parameter list
- type parameters are visible in the declaration signature
- type parameters are visible in the declaration body
- type parameters are not values
- type parameters do not introduce runtime bindings
- type parameters must not shadow built-in type names

Recommended style:

- use `T` for one generic type
- use `K` and `V` for map-like key/value pairs
- use short uppercase names such as `A`, `B`, `E` for small generic abstractions

## 6. Generic Records

A generic record declaration introduces a family of nominal record types.

```muga
record Box[T] {
  value: T
}
```

Examples:

```muga
int_box = Box[Int] {
  value: 1
}

name_box = Box[String] {
  value: "Ada"
}
```

`Box[Int]` and `Box[String]` are different instantiated record types.

Record literals for generic records should use an explicit instantiated type:

```muga
Box[Int] {
  value: 1
}
```

Inferring record literal type arguments from fields may be considered later, but it is not required for the v1 MVP.

## 7. Generic Functions

A generic function must declare its type parameters explicitly.

Valid:

```muga
fn id[T](value: T): T {
  value
}
```

Not valid as an implicitly generic function:

```muga
fn id(value) {
  value
}
```

The second form is not automatically generalized to `fn id[T](value: T): T`.

Reason:

- implicit generalization makes package interfaces harder to reason about
- ambiguous functions should ask for a visible type parameter list
- explicit generic declarations keep public APIs stable

## 8. Type-Argument Inference

Call sites should omit type arguments when they can be inferred locally.

```muga
fn id[T](value: T): T {
  value
}

a = id(1)       // T = Int
b = id("Ada")   // T = String
```

Inference sources:

- argument types
- expected return type from the local expression context
- explicit type annotations in the same declaration

Muga should not infer type arguments from arbitrary downstream package call sites when defining a public API.

## 9. Explicit Call-Site Type Arguments

Explicit call-site type arguments are not part of the v1 MVP.

Potential future syntax:

```muga
id[Int](1)
```

This is useful, but it competes syntactically with future indexing syntax such as:

```muga
items[0]
```

The v1 MVP should rely on local type-argument inference and expected types instead. If explicit call-site type arguments are added later, they should be limited to positions that remain unambiguous for the parser.

## 10. Generic Built-ins And Collections

The collection draft depends on this generics MVP.

Initial generic builtin types:

- `List[T]`
- `Option[T]`
- `Map[K, V]`

Examples:

```muga
numbers: List[Int] = []
users: Map[String, User] = Map.empty()
maybe_user: Option[User] = users.get("ada")
```

The exact construction and pattern matching syntax for `Option[T]` is deferred to enum or sum-type design.

`T?` is reserved as possible future shorthand for `Option[T]`, but `Option[T]` is the canonical v1 spelling.

## 11. Recursion

Generic recursive functions follow the existing recursion annotation rules.

Valid:

```muga
fn first_or[T](xs: List[T], fallback: T): T {
  if xs.is_empty() {
    fallback
  } else {
    xs[0]
  }
}
```

Polymorphic recursion is not part of v1.

That means a generic function's recursive calls should use the same type parameters rather than recursively calling itself at unrelated instantiations.

## 12. Package Interfaces

Public generic declarations should be stored in package interfaces as generic signatures.

Example source:

```muga
pub fn id[T](value: T): T {
  value
}
```

Interface shape:

```txt
pub fn id[T](T): T
```

Downstream packages typecheck against the interface. They should not need to re-open the generic function body unless implementation artifacts are being rebuilt.

This keeps Muga's source code inference-first while preserving fast package compilation.

## 13. Implementation Notes

The implementation should proceed in small slices:

1. parse generic type expressions
2. parse type parameter lists on records and functions
3. represent type parameters and type applications in AST
4. add generic type information to resolver/typechecker
5. support generic records
6. support generic functions with local type-argument inference
7. update typed HIR to preserve generic signatures and instantiated types
8. update package symbol graph and future interfaces to store generic signatures

The compiler may choose monomorphization, boxed runtime representation, or another backend strategy later. The source-level v1 semantics should not depend on that backend choice.
