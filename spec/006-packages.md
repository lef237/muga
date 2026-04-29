# Packages and Modules Draft

Status: draft with an implemented front-end subset. The current Rust compiler supports `package`, `import`, `pub`, and `alias::Name` lookup for directory-based packages. Manifest syntax, configurable source roots, selective imports, module-private visibility, `pkg`, and package-level caching are still deferred.

## 1. Design Goals

The package system should support the following goals:

- keep Muga visually simple and easy to read
- make dependencies explicit
- preserve the function-centered design
- keep `.` reserved for field access, chained calls, and record update
- make package boundaries cheap to resolve and cache
- avoid hidden initialization and file-order semantics
- provide a modern visibility model without introducing class-like ownership

## 2. Core Direction

The draft introduces a distinction between:

- script files, which are the current v1 file form
- package files, which are used for multi-file programs and libraries
- modules, which are the encapsulation boundary inside a package

This separation is intentional.

The current script form is good for:

- small examples
- experiments
- single-file execution

The package form is meant for:

- larger applications
- reusable libraries
- web backends and services
- fast incremental compilation

The module model is meant for:

- small, file-local abstractions
- hiding implementation details without creating many tiny packages
- keeping package boundaries focused on import, build, and cache behavior

In the current draft, a module is one `.muga` source file in package mode. Future manifest support may allow explicit multi-file modules, but v1 should start with file-as-module because it is simple and cheap to compile.

## 3. Two File Modes

### 3.1 Script file

A script file does not begin with `package`.

It keeps the current v1 behavior:

- top-level statements are allowed
- top-level bindings may execute
- the file may be run directly

### 3.2 Package file

A package file begins with a `package` declaration.

Once a file is in package mode:

- top-level executable statements are not allowed
- top-level items are restricted to declarations
- imports are explicit
- visibility may be marked with `pkg` or `pub`

This keeps package compilation deterministic and avoids runtime initialization order problems.

## 4. Package Model

The draft adopts the following model:

- one directory corresponds to one package
- every `.muga` file in that directory must declare the same package path
- the package path is written explicitly in each file
- file order is not semantically meaningful
- the compilation unit for caching is the package, not the file
- the smallest default encapsulation unit is the module/file, not the package

Example:

```txt
package app::web
```

This package path is expected to match the directory structure under a source root.

The exact source-root and dependency manifest format is deferred. The purpose of this draft is to define the language-facing part first.

## 5. Package, Module, and Visibility Model

Muga separates compilation units from encapsulation units:

- a package is the import, dependency, interface, and build-cache unit
- a module is the local encapsulation unit
- in v1 draft form, one package file is one module

This avoids the problem where every private implementation detail is visible everywhere in the package. Code can build small abstractions inside one file without splitting the project into many tiny directories.

The intended visibility levels are:

| Syntax | Meaning |
|---|---|
| no modifier | visible only inside the declaring module/file |
| `pkg` | visible inside the same package |
| `pub` | visible from importing packages |

This applies to:

- top-level `record` declarations
- top-level `fn` declarations
- record fields

Current implementation note:

- the compiler currently implements only a subset: top-level `pub` and package-level flattening
- module-private default and `pkg` are target design and should be implemented before real package interfaces harden

Example:

```txt
package app::counter

record Counter {
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

Here `Counter.value` is an implementation detail of this module. Other modules should use `new_counter`, `inc`, and `value` rather than accessing the field directly.

## 6. Package Syntax

### 6.1 Package path

Package paths use `::`-separated identifiers:

```txt
app::web
std::http
company::auth::session
```

`::` is chosen intentionally so that:

- `.` remains visually stable for fields and chains
- package qualification does not look like field access
- type names and value names can use the same qualified form

### 6.2 Concrete grammar

At the parser level, the file grammar is intentionally split in two:

```ebnf
file          := script_file | package_file
script_file   := stmt*
package_file  := package_decl import_decl* package_item*
package_decl  := "package" package_path
package_path  := IDENT ("::" IDENT)*
import_decl   := "import" package_path import_alias?
import_alias  := "as" IDENT
package_item  := visibility? record_decl
               | visibility? func_decl
visibility    := "pub"
               | "pkg"
qualified_ref := IDENT "::" IDENT
```

Additional parser rules for package mode:

- `package` must be the first significant token in the file
- `import` declarations must come after `package` and before the first item
- `pub` and `pkg` are valid on top-level `record` and `fn`
- top-level items are separated by newlines
- type and value qualification uses exactly `alias::Name`

In package mode:

- `record_decl` and `func_decl` keep their existing meanings
- `assign_like_stmt`, `if_stmt`, `while_stmt`, and `expr_stmt` are not allowed at the top level

## 7. Imports

An import introduces a package alias into the current file.

Without `as`, the local alias is the last segment of the package path.

Example:

```txt
import std::http
import company::auth::session as auth_session
```

This makes the following local aliases available:

- `http`
- `auth_session`

Imported names are then referenced through qualified package access:

```txt
http::Request
http::Response
http::serve
auth_session::Token
```

v1-like package rules:

- wildcard imports are not part of the draft
- selective imports are not part of the draft
- re-export syntax is not part of the draft
- if two imports would introduce the same alias, that is an error unless one uses `as`

## 8. Top-Level Items in Package Mode

Package files may contain only:

- `record` declarations
- `fn` declarations

This means package mode explicitly excludes:

- top-level `x = e`
- top-level `mut x = e`
- top-level `if`
- top-level `while`
- top-level expression statements

This is a deliberate performance and clarity choice.

It gives the compiler:

- no hidden initialization semantics
- no cross-file execution ordering
- no package import side effects during interface loading

## 9. Visibility

The target draft uses module-private-by-default visibility.

- a top-level item without a modifier is visible only within the declaring module/file
- a top-level item with `pkg` is visible from other modules in the same package
- a top-level item with `pub` is visible from other packages

Example:

```txt
package app::users

pub record User {
  name: String
}

pub fn display_name(user: User): String {
  user.name
}
```

Here:

- `User` is public
- `display_name` is public

Imported packages expose only `pub` items.

`pkg` items are not exposed through package interfaces.

Module-private items are not visible from sibling files in the same package.

This is deliberately more restrictive than package-wide private visibility. The goal is to allow small abstractions inside one file without forcing every implementation detail to be visible throughout the package.

## 10. Qualified Name Use

The same `package_alias::Name` form is used for both types and values.

This is intentionally limited to one alias segment followed by one item name:

```txt
users::User
users::display_name
```

The alias may itself refer to a longer package path through `import ... as ...` or through the default "last path segment" rule.

Example:

```txt
package app::web

import std::http
import app::users

pub fn handle(req: http::Request): http::Response {
  user = users::find_current(req)
  users::respond_with_name(user)
}
```

This keeps value and type lookup visually consistent.

Within the current package:

- top-level names from the current module may be referenced unqualified
- top-level names from sibling modules may be referenced only if they are `pkg` or `pub`
- module-private top-level names are not visible from sibling modules
- package-visible and public top-level names are collected across files before body checking

Across packages:

- references must be qualified through an imported package alias

## 11. Public API Signature Policy

To support both minimal annotations and fast package compilation, package interfaces store **resolved public signatures**.

Users do not have to write every public signature by hand when the compiler can infer it uniquely.

The important boundary is:

- package authors may omit annotations when local inference is sufficient
- importers read cached package interfaces, not the full bodies of unchanged dependencies
- package interfaces contain concrete resolved signatures whether they were written or inferred

### 11.1 Public functions

Every `pub fn` must have an inferable public signature.

That signature may come from:

- explicit annotations
- local inference inside the defining package
- a mix of both

```txt
pub fn display_name(user: User) {
  user.name
}

pub fn age_next(user: User) {
  user.age + 1
}
```

These are valid because the compiler can infer the exported signatures:

```txt
display_name: User -> String
age_next: User -> Int
```

The generated package interface stores those resolved signatures.

Annotations remain required when a public signature cannot be inferred uniquely from local information.

Examples:

```txt
pub fn id(x) {
  x
}
```

```txt
pub fn apply(x, f) {
  f(x)
}
```

These are invalid without more annotations because the exported signature is ambiguous.

### 11.2 Public records

`record` fields already require explicit types, so `pub record` introduces no additional annotation burden there.

However, a `pub record` may still contain non-public fields. Such fields are part of the record's representation but are not directly nameable outside their visibility boundary.

### 11.3 Why this rule exists

This rule is recommended for three reasons:

- source code keeps the same inference-first style in private and public functions
- exported signatures can still be loaded and hashed without typechecking unchanged dependency bodies
- package interfaces remain stable and cheap to cache once generated

Private functions remain free to use local inference.

The cost trade-off is explicit:

- the defining package must typecheck public bodies when generating or refreshing its interface
- downstream packages can use the cached inferred interface without reading those bodies again
- first builds may do slightly more work, but incremental and dependency builds stay fast

### 11.4 Public signatures may not leak non-public names

A public item may not mention a non-public top-level name in its visible type.

This includes both:

- module-private names
- `pkg` names

Examples of invalid public API:

```txt
package app::users

record InternalUser {
  name: String
}

pub fn display_name(user: InternalUser): String {
  user.name
}
```

```txt
package app::web

import app::users

pub record Session {
  user: users::InternalUser
}
```

These are invalid because importers of the package could see the public API but would have no legal way to name the leaked private type.

## 12. Build and Compilation Model

The package system is designed around package-level compilation units.

The intended pipeline is:

1. read package headers
2. collect imports and public declarations
3. build an interface summary for each package
4. reject import cycles
5. typecheck and lower package bodies only after imported interfaces are known

This enables:

- per-package caching
- parallel compilation of independent packages
- cheap recompilation when only private bodies change

In particular, the draft intentionally does not rely on:

- cross-package type inference
- package load order effects
- top-level execution during import

## 13. Cycles

Import cycles are prohibited.

Example:

- `app::web` imports `app::users`
- `app::users` imports `app::web`

This is an error.

The draft keeps the dependency graph acyclic so that:

- interface loading is simple
- package compilation order is deterministic
- build caching stays cheap

## 14. Executable Packages

The draft reserves `package main` for executable packages.

Example:

```txt
package main

import app::web

fn main(): Int {
  web::serve()
}
```

In this model:

- `main` does not need `pub`
- other packages should not import `package main`
- the build tool chooses an entry package rather than a single file

The exact CLI shape is deferred.

Current implementation note:

- `cargo run -- check path/to/entry.muga` already supports package files
- `cargo run -- path/to/entry.muga` already runs a package graph by flattening imported packages into one internal program
- the current file-based CLI accepts any package path, as long as the chosen entry package contains `fn main()`
- the source root is currently inferred from the entry file path and the declared package path

## 15. Why This Is Meant To Feel Modern

The draft aims to borrow the good parts of modern languages without carrying in their full complexity.

It keeps:

- explicit imports
- explicit visibility
- package-level compilation units
- aliasing when import names would collide
- strongly typed public boundaries

It avoids, for now:

- wildcard imports
- implicit re-exports
- top-level import side effects
- nested module trees inside a file
- protocol-like solving at package boundaries
- package-scoped execution order rules

## 16. Example

```txt
package app::users

pub record User {
  name: String
}

pub fn display_name(user: User): String {
  user.name
}
```

```txt
package app::web

import app::users

pub fn show(user: users::User): String {
  users::display_name(user)
}
```

## 17. Deferred Topics

This draft intentionally leaves the following topics for later:

- dependency manifest syntax such as `muga.toml`
- source-root discovery rules
- standard library package layout
- selective imports
- wildcard imports
- re-exports
- package-scoped constants or immutable top-level values
- generic packages
- protocol/trait-like abstractions
- testing and benchmark file conventions

## 18. Recommendation

If Muga continues to optimize for:

- simple reading
- low annotation burden inside implementations
- explicit boundaries
- fast compilation

then this package design is a good fit:

- script mode stays lightweight
- package mode stays explicit
- public APIs stay easy to cache
- `.` remains visually stable
- the compiler never needs whole-program global inference
