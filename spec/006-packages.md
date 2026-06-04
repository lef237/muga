# Packages and Modules Draft

Status: draft with an implemented front-end subset. The current Rust compiler supports explicit `package`, `import`, `pkg`, `pub`, `alias::Name` lookup, directory-based packages, module/file identity for top-level items, top-level module-private visibility, public package-mode `pub opaque type` names including compiler-provided runtime-backed `std::fs::File`, a minimal `muga.toml` project mode that infers package paths from `name` and `source` and may declare `resources = "resources"` for package-owned resources, local path dependencies through `[dependencies] name = { path = "..." }`, local archive dependencies through `[dependencies] name = { archive = "...", hash = "sha256:<hex>" }`, explicit `.mgi` / `.mgc` / `.mgb` artifact workflows, `muga build` emission to a default `.muga/build` artifact directory with unchanged-artifact reuse and written/reused status output, package-local source hashes in `.mgb` artifacts, public `.mgi` interface hashes that ignore diagnostic-only spans, dependency-level parallel package artifact builds, minimal `muga.lock` generation and validation with local path dependency `source_hash` metadata plus local archive dependency `hash` metadata, a library helper that computes the first canonical package content hash over `muga.toml` plus sorted `.muga` source files and declared resources, deterministic `.mgp` source/resource archive emission through `emit-package-archive` plus pasteable dependency snippet output through `--dependency-snippet`, library `.mgp` archive readback/hash validation, library materialization of validated `.mgp` bytes into absent or empty local source/resource trees, local `.mgp` archive dependency cache consumption and local cache/lockfile edge-case hardening through `.muga/packages`, read-only runtime package resource lookup through `std::fs::read_resource_text` and `std::fs::read_resource_bytes`, non-mutating app bundle emission through `emit-app-bundle --source-free` with bundle-local dependency trees, source-free app bundle execution through `run-app-bundle`, non-mutating app launcher and ownership metadata placement plus guarded owned updates/uninstalls through `install-app --replace-owned` and `uninstall-app`, source-free app completion package emission through `emit-app-completions`, deterministic `.mga` app archive emission/unpacking, explicit `check --built` / `run --built` consumption of that default artifact directory, package-aware checking over the unflattened package graph, and artifact-backed package execution through MIR-lowered bytecode artifacts. Registries, URL/Git dependency forms, remote package fetching, publishing/install workflows, full published-package lockfile enforcement, selective imports, full incremental project-level artifact reuse, control-flow-oriented MIR, broader runtime-backed resource handles, and any future per-field record visibility are still deferred.

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

A package file is a source file that belongs to a package.

There are two ways for this to happen:

- explicit file-based package mode: the file begins with a `package` declaration
- manifest project mode: the file is under the manifest source root, and the package path is inferred from the directory layout

Once a file is in package mode:

- top-level executable statements are not allowed
- top-level items are restricted to declarations
- imports are explicit
- visibility may be marked with `pkg` or `pub`

This keeps package compilation deterministic and avoids runtime initialization order problems.

## 4. Package Model

The draft adopts the following model:

- one directory corresponds to one package
- every `.muga` file in that directory belongs to the same package
- explicit file-based package mode requires each file to declare the same package path
- manifest project mode infers the package path from the manifest and directory layout, so files may omit `package`
- file order is not semantically meaningful
- the compilation unit for caching is the package, not the file
- the smallest default encapsulation unit is the module/file, not the package

Example:

```txt
package app::web
```

This package path is expected to match the directory structure under a source root.

Full dependency manifest syntax is deferred. The current implementation supports
`[package]` with `name`, `source`, and optional `resources`, plus local path and
local `.mgp` archive forms in `[dependencies]`:

```toml
[package]
name = "app"
source = "src"
resources = "resources"
```

The `resources` value opts into deterministic package resource inclusion. It
must be a relative slash-separated directory path, and currently includes
regular files in package content hashes, `.mgp` archives, materialization, local
archive dependency cache validation, and source-backed app bundle emission.
Runtime `std::fs::read_resource_text` decodes UTF-8 text resources, while
`std::fs::read_resource_bytes` returns opaque `std::bytes::Bytes`.

```toml
[dependencies]
shared = { path = "../shared" }
archived_shared = { archive = "../archives/archived_shared-sha256-....mgp", hash = "sha256:..." }
```

The dependency key must match the target manifest's `[package] name`. Local archive dependencies are validated against the declared hash, materialized under `.muga/packages/<package>-sha256-<hash>`, and reused only when the cached source/resource tree re-hashes to the same content hash. Missing archive hashes, empty archive paths, `path`/`hash` combinations, cache path collisions, stale cache content, package-name mismatches, and malformed archive lockfile entries are rejected. `muga build` currently records local path dependency source descriptors plus SHA-256 `source_hash` metadata, and local archive source descriptors plus `hash` metadata, in `muga.lock`; it refreshes well-formed stale local metadata and rejects malformed or unsupported existing lockfiles with `PK026`. URL, Git, registry, version solving, remote fetching, publishing/install workflows, and full lockfile enforcement remain deferred.

`pub opaque type` declarations for future runtime-backed handles are
represented in `.mgi` as public nominal type names without exposing a field
layout or runtime token. The interface-only slice is implemented for package
mode, type checking, artifacts, docs, metadata, hover, completions, definition,
references, and downstream loaded-interface checking. `.mgi` v5 also stores
opaque `handleFacts` plus function-parameter `paramMode` metadata, and those
facts are exposed through package metadata, hover/completion metadata, and
generated docs. The typechecker rejects direct same-scope use-after-consume for
loaded-interface `consume` parameters with `T026`. Runtime-backed handle values,
source-level consuming parameter syntax, and broad effectful standard-library
APIs remain deferred until a concrete runtime-backed handle slice requires them.

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

Current implementation note:

- the compiler currently implements top-level module-private, `pkg`, and `pub` visibility before flattening
- imports expose only `pub` items
- per-field record visibility is not part of the committed v1 package model
- package-level flattening still exists for the compatibility AST returned by `check_path`; package-aware typed HIR now carries package item identity, is generated from unflattened module checks, and lowers through the existing HIR/bytecode VM path for default package execution

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
script_file   := top_item*
package_file  := package_decl? import_decl* package_item*
package_decl  := "package" package_path
package_path  := IDENT ("::" IDENT)*
import_decl   := "import" package_path import_alias?
import_alias  := "as" IDENT
package_item  := visibility? record_decl
               | visibility? enum_decl
               | "pub" "opaque" "type" IDENT
               | visibility? func_decl
visibility    := "pub"
               | "pkg"
qualified_ref := IDENT "::" IDENT
```

Additional parser rules for package mode:

- if present, `package` must be the first significant token in the file
- without manifest inference, `package` is required
- `import` declarations must come after `package` when it is present and before the first item
- `pub` and `pkg` are valid on top-level `record`, `enum`, and `fn`
- the current opaque type slice accepts only top-level `pub opaque type Name`;
  `pkg opaque type` and module-private `opaque type` are deferred
- top-level items are separated by newlines
- type and value item qualification uses `alias::Name`
- enum variant construction and patterns may use `alias::Enum::Variant`

In package mode:

- `record_decl`, `enum_decl`, `pub opaque type`, and `func_decl` keep their existing meanings
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
- `enum` declarations
- `pub opaque type` declarations
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

The same `package_alias::Name` form is used for package-qualified types and values.

Package item qualification is intentionally limited to one alias segment followed by one item name:

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

Enum variants are namespaced below the enum item. Across package boundaries, their constructors and patterns use the qualified enum item plus the variant:

```txt
states::Status::Ready(1)
states::Status::Pending
```

Within the current package:

- top-level names from the current module may be referenced unqualified
- top-level names from sibling modules may be referenced only if they are `pkg` or `pub`
- module-private top-level names are not visible from sibling modules
- package-visible and public top-level names are collected across files before body checking

Across packages:

- references must be qualified through an imported package alias

## 11. Public API Signature Policy

To support both minimal annotations and fast package compilation, package interfaces store **resolved public signatures**.

For the v1 completion target, public functions still require explicit public signatures in source. Public-signature inference is the intended later policy once the current artifact workflow and diagnostics are stable.

The longer-term policy is that users should not have to write every public signature by hand when the compiler can infer it uniquely.

The important boundary is:

- v1 package authors write explicit public function signatures
- private package bodies may still use local inference when the type is unique
- importers read cached package interfaces, not the full bodies of unchanged dependencies
- package interfaces contain concrete resolved signatures

If public-signature inference is added later, the defining package may infer a public signature locally before storing that same concrete resolved signature in the package interface. Downstream packages should still never infer a dependency public API from their own call sites.

### 11.1 Public functions

Every `pub fn` must have a public signature.

In the v1 completion target, that signature must be explicit enough for the compiler to know the full callable type before downstream package checking:

- public parameter types must be explicit
- the public return type must be explicit
- generic public functions must declare their type parameters explicitly

Public-signature inference remains a post-v1 extension. If added later, the signature may come from explicit annotations, local inference inside the defining package, or a mix of both, and the generated package interface will still store the resolved signature.

```txt
pub fn display_name(user: User) {
  user.name
}

pub fn age_next(user: User) {
  user.age + 1
}
```

These are examples of the post-v1 inference direction, where the compiler could infer the exported signatures:

```txt
display_name: User -> String
age_next: User -> Int
```

The generated package interface would store those resolved signatures.

In v1, write the signatures explicitly:

```txt
pub fn display_name(user: User): String {
  user.name
}

pub fn age_next(user: User): Int {
  user.age + 1
}
```

Annotations also remain required in any future inferred-signature mode when a public signature cannot be inferred uniquely from local information.

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

### 11.2 Public records, enums, and opaque types

`record` fields already require explicit types, so `pub record` introduces no additional annotation burden there.

In the committed v1 model, a `pub record` is transparent: its field names and field types are part of the public record shape. Importing packages can use record literals, field access, and `record.with(...)` for those public fields.

`pub enum` exposes its enum name, type parameters, variant names, and payload types through the package interface. Variant constructors and patterns remain qualified as `alias::Enum::Variant`; unqualified variant imports are not part of v1.

If a representation should be hidden inside a module or package, keep the record itself non-public and expose functions that do not leak that non-public type across a wider visibility boundary.

If a package needs to expose a public type name while hiding its representation, the current interface-only form is:

```muga
pub opaque type File
```

Importing packages can name this type in annotations and public signatures, but
ordinary source code cannot construct it with a record literal, access fields,
match on it, compare it structurally, or format it with `to_string`. The current
slice has no type parameters for opaque type declarations and does not add
runtime-backed values. Source-defined opaque types currently receive
conservative `handleFacts` defaults: not runtime-backed, not copyable, not
cloneable, not sendable, not shareable, not structurally comparable, not
serializable, not closeable, and no named close function.

The remaining opaque representation directions are:

- `pub opaque record` for ordinary Muga data whose fields are visible only to the defining module
- runtime-backed values plus source syntax for consuming parameters and
  capability metadata on `pub opaque type` handles

Per-field visibility is a weaker candidate and should be reconsidered only if concrete code needs partially transparent public records.

### 11.3 Why this rule exists

This rule is recommended for three reasons:

- source code keeps the same inference-first style in private and public functions
- exported signatures can still be loaded and hashed without typechecking unchanged dependency bodies
- package interfaces remain stable and cheap to cache once generated

Private functions remain free to use local inference.

The cost trade-off is explicit:

- the defining package must typecheck public bodies when generating or refreshing its interface
- downstream packages can use the cached interface without reading those bodies again
- first builds may do slightly more work, but incremental and dependency builds stay fast

### 11.4 Package interfaces as application contracts

The same resolved public signatures should also support future application tooling.

For compiler purposes, `.mgi` is the public package interface. For application tooling, the same artifact can be treated as a typed contract made of:

- public transparent records and their field types
- public enums, variants, payload types, and type parameters
- public opaque type names with hidden representation
- public function signatures with explicit parameter and return types
- public `Result[T, E]` shapes where `E` is part of the failure contract
- stable package/item identities and interface hashes

Future generators may consume `.mgi` to produce API documentation, schema files, TypeScript clients, or service stubs. Those generators should not inspect private package bodies, depend on source file order, or infer protocol semantics from naming conventions.

This is not a v1 requirement. Before implementing generators, the design must define:

- stable external naming rules for packages, items, enum variants, and fields
- supported type mappings for each target schema or client language
- how `Option`, `Result`, `List`, `Map`, and generic user types are represented externally
- how opaque future runtime handles are excluded or represented
- how generated artifacts are invalidated from `.mgi` interface hashes

Packages should opt into HTTP, RPC, or other external protocols through explicit adapter APIs. A plain `pub fn` should remain a package-level function contract, not automatically a network endpoint.

### 11.5 Public signatures may not leak non-public names

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
- `cargo run -- path/to/entry.muga` already runs a package graph through the package-aware typed HIR and MIR/bytecode VM path
- the entry file identifies the entry package, and the compiler reads all `.muga` files in that package directory
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

## 16. Large Project Layout

For larger codebases, Muga should keep the mental model simple:

- directory = package
- file = module and default encapsulation boundary
- manifest = project, dependency, and source-root configuration
- package interface = cached public API summary

Example future project layout:

```txt
my_service/
  muga.toml
  src/
    main/
      main.muga

    config/
      config.muga

    http/
      server.muga
      router.muga

    users/
      model.muga
      repository.muga
      service.muga

    orders/
      model.muga
      service.muga

    db/
      connection.muga
      query.muga
```

In this layout, the manifest package name supplies the package-path prefix.

If `muga.toml` declares `name = "my_service"` and `source = "src"`, then:

- `src/main/` maps to `my_service::main`
- `src/users/` maps to `my_service::users`
- `src/http/` maps to `my_service::http`

Each directory under the source root is one package. All `.muga` files in the same directory belong to the same package path.
In manifest project mode, those files may omit the package declaration because the package path is inferred. If a file still includes an explicit package declaration, it must match the inferred package path.

Example:

```txt
// src/users/model.muga
pub record User {
  name: String
  age: Int
}
```

```txt
// src/users/service.muga
pub fn display_name(user: User): String {
  user.name
}
```

The files in `src/users/` form one package. They may refer to package-visible declarations from the same package without importing that package.

Other packages import the package by logical package path:

```txt
// src/main/main.muga
import my_service::users

fn main(): Int {
  user = users::User {
    name: "Ada"
    age: 20
  }

  println(users::display_name(user))
}
```

The source code does not import by filesystem path such as `./users` or `../users`. Filesystem layout is handled by the source root and manifest.

This keeps code portable across operating systems and keeps import names stable when files move inside a package.

## 17. Distribution and Dependency Model

The distribution model is built on two non-negotiable properties:

- every dependency is identified by a cryptographic content hash
- no Muga-operated infrastructure is required for a project to build

Every other layer (registries, short names, publishing workflow) sits on top of these two and can be replaced or removed without breaking existing projects. This keeps Muga's "no hidden behavior" rule consistent between the language and its tooling, and keeps projects buildable for the long term independent of any single host or organization.

### 17.1 Layered Architecture

The dependency system is organized in four layers, from most authoritative to most convenient:

1. Content layer — a package version is identified by `sha256:<hex>` over its canonical archive. The hash is the primary identity. Two artifacts with the same hash are the same package version regardless of where they came from.
2. Transport layer — packages are fetched over plain HTTPS or Git from any host. URLs are locations where the bytes can be retrieved; they are not part of package identity.
3. Human layer — manifests use SemVer requirements and short aliases for readability. Resolution turns these into concrete `(source, hash)` pairs and records them in a lockfile.
4. Naming layer (optional) — a registry, if one exists, maps short names to `(version, source, hash)` records. It is a convenience, not a trust root.

Builds use layer 1 as the only source of identity. Layers 2 to 4 are advisory and replaceable.

### 17.2 Package Identity

A published package version has exactly one identity: the SHA-256 hash of its canonical archive.

The canonical archive contains:

- `muga.toml`
- every `.muga` file under the declared `source` root, in sorted path order
- every regular file under the optional declared `resources` root, in sorted path order
- the precomputed package interface file, if present
- nothing else (no VCS metadata, no editor files, no build outputs)

Hash representation: `sha256:` followed by lower-case hexadecimal. Other algorithms are reserved for future use behind the same `<algo>:<hex>` form.

Current implementation note: `emit-package-archive --archive-root <dir> <entry>` writes the first deterministic Muga-native `.mgp` source/resource archive. Its bytes are the canonical local input containing `muga.toml`, sorted `.muga` files under the declared source root, and sorted regular files under the optional declared resource root; `.muga` and `.git` tool directories are ignored. The `sha256:<hex>` content hash is computed directly over those archive bytes and appears in the archive filename. The optional `--dependency-snippet` output keeps the archive bytes unchanged but prints a pasteable `[dependencies]` entry using the manifest package name, archive path, and content hash. Library readback validates `.mgp` bytes against optional expected `sha256:<hex>` values, parses manifest, source, and resource entries without trusting filenames, preserves arbitrary resource bytes, and rejects malformed length-prefixed entries, duplicate or unsorted source/resource paths, non-UTF-8 manifest/source entries, source/resource-root escapes, undeclared resource entries, tool metadata directories, and non-source file entries. Library materialization then writes validated archive bytes into an absent or empty local source/resource tree, preserves the validated content hash, rejects unsafe manifest `source` or `resources` roots, and rejects non-empty destinations. Local archive dependencies use the same verifier, materialize to `.muga/packages`, reject cached source/resource trees whose recomputed hash no longer matches the declared hash, and reject malformed archive dependency or lockfile metadata instead of silently repairing it. `emit-app-bundle --source-free --output-dir <dir> [--program <name>] <entry>` now can omit copied source files while keeping declared resources, bundle-local dependency manifests/resources under `.muga/bundle-deps`, `.muga/app-bundle` entry metadata, and `.muga/build` artifacts in a bundle with a `bin/<program>` launcher; without `--source-free`, the same command also copies root and dependency source trees plus source-hash `muga.lock` metadata for inspection. `run-app-bundle <bundle-dir>` executes the bundle from `muga.toml`, `.muga/app-bundle`, manifest resources, `.mgi` interfaces, and `.mgb` implementation artifacts without reading copied source files; `install-app [--replace-owned] --output-dir <bin-dir> [--program <name>] <bundle-dir>` writes a wrapper plus `<bin-dir>/.muga/installed-apps/<program>.toml` ownership metadata into an explicit bin directory without editing shell profiles, and only replaces an existing launcher/metadata when `--replace-owned` verifies prior Muga ownership; `uninstall-app --output-dir <bin-dir> --program <name>` removes only that ownership-verified launcher and metadata, leaving bundles and shell profiles untouched; `emit-app-completions --output-dir <dir> [--program <name>] --type <type> [--package <package>] <bundle-dir>` writes generated app completion packages from bundle `.mgi` interfaces without requiring source files; `emit-app-archive` and `unpack-app-archive` round-trip that bundle directory as a deterministic `.mga` file. The future registry security design must keep this `.mgp` hash as the package identity and treat registries as naming/discovery services rather than trust roots. This slice deliberately does not yet include precomputed interface bytes, URL/Git fetching, registries, publish workflows, shell-profile install workflows, or full published-package lockfile enforcement.

Current edition/fingerprint note: Muga has no edition selector yet. Future package artifacts and lockfiles should record language edition and semantic feature-set fingerprints as semantic interpretation metadata while keeping `.mgp` `sha256:<hex>` values as package byte identity. This is design-only and does not add manifest syntax, cross-edition imports, artifact bytes, or edition migration tooling.

Two consequences:

- two URLs serving the same bytes resolve to the same package and share cache entries
- any byte change produces a different identity, so there is no ambiguity about which `json@1.2.0` was actually installed

### 17.3 Manifest

`muga.toml` declares dependencies in `[dependencies]`. In typical use, only the short form appears; the other forms exist for explicit cases.

```toml
[package]
name = "my_service"
version = "0.1.0"
source = "src"

[dependencies]
# Registry short form — the everyday case
json = "^1.2"
http = "^0.4"

# URL form — for packages not in the registry or self-hosted archives
metrics = { url = "https://example.org/muga/metrics-0.4.0.tar.gz", version = "0.4.0" }

# Git form with version — resolves to a matching Git tag (Go-style ergonomics)
analytics = { git = "https://github.com/example/analytics.git", version = "0.4.0" }

# Git form with rev — pins to an exact commit; for unpublished branches or forks
custom = { git = "https://github.com/example/custom.git", rev = "f1e2d3c4b5a6..." }

# Path form — for monorepo and workspace use; not allowed in published packages
shared = { path = "../shared" }
```

Required fields per form:

| Form | Required keys | Optional keys |
|---|---|---|
| Registry short form | a SemVer requirement string | — |
| URL form | `url` | `version`, `hash` |
| Git form | `git`, and one of `rev` or `version` | the other of `rev` or `version`, `hash` |
| Path form | `path` | — |

URL form does not prescribe a file extension or archive format. The URL must return archive bytes when fetched over HTTPS; supported formats include `.tar.gz`, `.tar.zst`, `.zip`, the Muga-native `.mgp`, and any future addition. The format is identified by the HTTP `Content-Type` header, falling back to magic bytes in the response body. This keeps URL form host-agnostic and format-agnostic — GitHub Releases, S3, self-hosted Artifactory, or a plain static file server are all valid.

Git form accepts either `rev` (a full commit SHA, for exact-commit pinning) or `version` (a SemVer-shaped string the resolver matches against the repository's Git tags, accepting both `0.4.0` and `v0.4.0` forms). Tag-based resolution gives Go-like ergonomics: pointing at a repository and a version is enough, and the resolver records the resolved commit SHA in the lockfile so the build remains reproducible even if the tag is later moved or deleted.

Hashes are never typed by hand. The resolver fetches each dependency, computes the SHA-256 of the canonical archive, and records it in `muga.lock`. From the second build onward, every fetch is verified against the lockfile entry. This is the same model used by Cargo, npm, Deno, and Go.

The optional `hash` field on URL form and Git form is for users who want to defend the very first install as well: when the publisher announces an expected hash through an out-of-band channel (release notes, signed announcement), the user pastes it into `muga.toml`, and the resolver refuses to accept any other bytes. Omitting the field leaves the first install in a Trust-On-First-Use posture.

Source `.muga` files never reference URLs, hashes, registry names, or filesystem paths. Source code imports always use logical package paths only.

### 17.3.1 Typical User Workflow

Adding a dependency goes through tooling, not by hand-editing the manifest:

```text
# registry short form — looks the package up in the registry
muga add json

# URL form — downloads one archive file from any HTTPS host
# (GitHub Releases, S3, self-hosted, anywhere; any archive format works)
muga add --url https://github.com/example/metrics/releases/download/v0.4.0/metrics.tar.gz

# Git form (tag) — clones a repository and resolves to the matching Git tag
muga add --git https://github.com/example/analytics.git --version 0.4.0

# Git form (commit) — pins to an exact commit; for unpublished branches or forks
muga add --git https://github.com/example/custom.git --rev <full-commit-sha>

# path form — local development dependency
muga add --path ../shared
```

The distinction between `--url` and `--git` is what is being fetched, not where it lives:

- `--url` retrieves a single archive file at the given URL (any HTTPS host, any archive format)
- `--git` clones a repository and selects the tree at the given commit SHA or tag

Both can target GitHub or any other host.

Each command:

1. resolves the source to concrete bytes
2. computes the SHA-256 of the canonical archive
3. writes the manifest entry in the simplest form sufficient
4. updates `muga.lock` with the resolved version, source, hash, and transitive dependencies

The user reviews the diff to `muga.toml` and `muga.lock` and commits both to version control. Day-to-day work after that does not involve hashes at all.

### 17.4 Lockfile

`muga.lock` is the source of truth for what actually gets built. It is generated by the resolver and should be committed to version control.

Current implementation note: today, `muga build` writes the first minimal `muga.lock` metadata for manifest projects, focused on local path dependencies and local `.mgp` archive dependencies. Path entries record source descriptors plus `source_hash = "sha256:<hex>"` over the dependency `muga.toml`, `.muga` files under its source root, and declared resource bytes when present, using the same selection as `.mgp` archive emission. Archive entries record `source = { archive = "..." }` plus `hash = "sha256:<hex>"`, validate the archive bytes on first materialization, and reuse only cache entries whose source/resource tree re-hashes to the same content hash. Existing well-formed local lockfiles are refreshed when stale. Malformed, duplicate, graph-inconsistent, unsupported, or archive-hash-incomplete existing lockfiles are rejected with `PK026` and are not silently overwritten. Local path `source_hash` remains rebuild/review metadata for local development; local archive `hash` is the package content identity for the archive bytes. Current builds do not yet enforce the lockfile as the source of truth for URL/Git/registry package bytes.

The lockfile records, for every direct and transitive dependency:

- the local alias used in source code
- the resolved package path
- the resolved version (informational)
- the source descriptor (`url`, `git`, or `path`)
- the content hash
- the dependency aliases this package itself uses
- the compiler version that produced the resolution

Example:

```toml
# muga.lock — generated; do not edit by hand
lockfile_version = 1
muga_version = "0.4.0"

[[package]]
alias = "json"
path = "json"
version = "1.2.3"
source = { url = "https://example.org/muga/json-1.2.3.pkg" }
hash = "sha256:9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
dependencies = []

[[package]]
alias = "http"
path = "http"
version = "0.4.1"
source = { git = "https://github.com/example/http.git", rev = "f1e2d3c4..." }
hash = "sha256:..."
dependencies = ["json"]
```

Build behavior:

- if `muga.lock` exists, the build uses exactly the listed hashes and does not consult the resolver
- if any fetched artifact's hash does not match the lockfile, the build fails with a verification error
- if `muga.toml` adds a new dependency or a SemVer requirement no longer satisfies the lockfile, the resolver updates the lockfile and the change must be reviewed in version control
- if `muga.lock` is absent, the resolver creates it from `muga.toml`

A registry going offline does not break a project whose lockfile is already populated. The hashes are sufficient to verify any cached or mirrored copy of the bytes, regardless of where they are obtained.

### 17.5 Source Code Imports

Source files continue to use only logical package paths. The dependency machinery is not visible at the call site.

```txt
package app::web

import json
import http::server as server

pub fn handle(request: server::Request): String {
  json::encode(request)
}
```

The local alias (`json`, `server`) is determined by the manifest, not by the URL or hash. Renaming a dependency in the manifest changes the import name without modifying the dependency itself.

### 17.6 Optional Registry as a Naming Layer

A registry, if Muga ever operates one, is restricted to the following role:

- maintain a directory of `name -> (version, source, hash)` records
- provide search and discovery
- enforce naming policy (uniqueness, squatting prevention, organization scoping)

It does not:

- serve package bytes directly (those live wherever the publisher chose to host them)
- act as a trust root (the hash in the lockfile is the trust root)
- become required for builds (URL form and Git form work without it)

If the registry disappears, projects that have already resolved their dependencies still build, because every lockfile entry contains both a source descriptor and a hash. New projects that depend only on registry short names would need to migrate to URL or Git form, but `.muga` source code does not change.

Naming convention for registry entries should be scoped, not flat. `@owner/name` style is recommended over bare `name` to remove first-come-first-served conflicts and typosquatting incentives. The exact registry policy is deferred until a registry is actually established.

### 17.7 Local Development

Local path dependencies are supported in `muga.toml` for monorepo and workspace use:

```toml
[dependencies]
shared = { path = "../shared" }
```

Source code imports the logical package as usual:

```txt
import shared::logging
```

Path dependencies:

- are resolved by reading the target directory directly
- do not contribute a published-package `hash` identity to the lockfile; the path entry is recorded instead
- may record a local `source_hash` so rebuild planning and code review can see dependency source changes without pretending local paths are immutable published artifacts
- must not appear in a published package; publishing fails if any direct or transitive dependency uses `path`

### 17.8 Publishing

A published package version is a single archive containing the canonical contents listed in 17.2. Publishing produces:

- the archive (`<name>-<version>.pkg`)
- the SHA-256 hash of the archive
- optionally, a detached signature using a separately specified signing scheme

Publishing does not require uploading to any specific host. A publisher may:

- upload the archive to their own HTTPS server and announce the URL and hash
- create a Git tag whose commit SHA pins the tree, alongside the archive hash
- submit the `(name, version, source, hash)` tuple to the optional naming layer

Consumers verify the hash on download. A mismatch is a hard error and aborts the build.

### 17.9 Cache Integration

The dependency model composes with the existing cache key design. A built artifact is keyed by:

- the package's content hash (from 17.2)
- the resolved interface hashes of every direct dependency
- the compiler version
- the target backend

Implications:

- two projects depending on the same `(content hash, dependency interface hashes, compiler version, target)` share a cache entry
- a dependency whose private implementation changes but whose public interface hash does not change does not invalidate downstream artifacts
- the cache is content-addressable end to end and can be safely shared across machines, CI runners, and mirrors

### 17.10 Trust and Verification

Default trust model:

- the lockfile hash is the trust root for every dependency
- the resolver refuses to install or run any artifact whose hash does not match
- the compiler version recorded in the lockfile is checked against the running compiler; mismatch produces a warning, and a downgrade across a major version produces an error

Optional layers, deferred to a separate specification:

- detached signatures and a signature-verification policy
- a Muga-operated append-only transparency log of `(package path, version, hash)` records, modeled on Go's `sum.golang.org`, that any client can consult to detect divergent or rewritten history
- a default caching proxy modeled on Go's `proxy.golang.org`, providing availability when an upstream URL or repository disappears
- reproducibility checks that re-derive the interface hash from sources

These layers are additive. None of them changes the meaning of the content hash, and none of them is required for a build to succeed. Once a transparency log is operational, the optional `hash` field in 17.3 gains an additional verification path — the log can be cross-checked against the lockfile entry without any change to the manifest format.

### 17.11 Current Implementation Boundary

The current implementation has a local package loader, minimal manifest mode with local path dependencies and local archive dependencies, optional manifest-declared package resources, package identity data, in-memory package interface summaries, persisted package interface artifacts with span-independent public hashes, explicit package check cache artifacts, explicit MIR-lowered bytecode package implementation artifacts with package-local source hashes, build-time reuse for unchanged generated artifacts with visible written/reused status, dependency-level parallel package artifact builds, generated/validated `muga.lock` metadata for local path dependency source descriptors plus `source_hash` values and local archive source descriptors plus `hash` values, the first library-level package content hash over `muga.toml` plus sorted source files and declared resources, deterministic `.mgp` source/resource archive emission with optional pasteable dependency snippet output, library `.mgp` readback/hash validation, library materialization of validated `.mgp` bytes into absent or empty local source/resource trees, local `.mgp` archive dependency cache consumption through `.muga/packages`, read-only runtime package resource lookup through `std::fs::read_resource_text` and `std::fs::read_resource_bytes`, runtime `std::bytes::Bytes` values with SHA-256 hashing support, non-mutating app bundle emission with optional source-free output through `emit-app-bundle --source-free`, source-free app bundle execution through `run-app-bundle`, non-mutating app launcher and ownership metadata placement plus guarded owned updates/uninstalls through `install-app --replace-owned` and `uninstall-app`, source-free app completion package emission through `emit-app-completions`, deterministic `.mga` app archive emission/unpacking, and focused local archive cache/lockfile failure coverage. It does not yet have URL/Git/registry dependency forms, remote package fetching, publishing/install workflows, full lockfile enforcement, registries, or full incremental project-level artifact reuse.

It currently:

- accepts a file entrypoint
- discovers `muga.toml` by walking up from the entry file
- supports `[package] name = "..."`, `source = "..."`, and optional `resources = "..."`
- supports `[dependencies] name = { path = "..." }` for local path dependencies whose target manifest name matches the dependency key
- supports `[dependencies] name = { archive = "...", hash = "sha256:<hex>" }` for local `.mgp` archive dependencies whose materialized target manifest name matches the dependency key
- infers package paths from source-root-relative directories in manifest project mode
- infers the source root from the entry file path and declared package path in explicit file-based package mode
- resolves imported package paths through the entry manifest's source root or declared local dependency roots, while keeping filesystem paths out of `.muga` source files
- reads all `.muga` files in each loaded package directory
- follows `import` declarations recursively within the local source tree
- rejects import alias collisions
- enforces module-private, `pkg`, and `pub` top-level visibility before flattening
- records package, module, and item identity in `PackageSymbolGraph`
- routes public import lookup through `PackageExportGraph`
- can return an unflattened package graph containing package files plus package/module/item/export metadata
- can build source and module package signatures from the unflattened graph while preserving package item identity and module/same-package/import visibility for records, enums, opaque types, and functions
- can run package-aware module body resolver/typecheck passes against those source/module signatures, including per-module typed HIR outputs lowered with package binding identity
- exposes package-wide typed HIR aggregated from unflattened module check outputs with remapped local IDs and symbols
- can collect dependency signatures directly from loaded in-memory or persisted package interfaces for package-aware module checks without reading dependency implementation source
- can build loaded-interface dependency package graph metadata directly from package interfaces without dependency AST stubs
- routes `muga check --artifact-root`, `muga run --artifact-root`, `muga check --built`, and `muga run --built` through explicit package artifact paths
- emits `.mgi` interface artifacts from the package-aware typed HIR aggregate
- persists public opaque type names, `PackageOpaque` signature references, and
  runtime-backed handle facts for `std::fs::File` in `.mgi` interfaces without
  exposing concrete handle values
- keeps `.mgi` public interface hashes stable across implementation-only body and diagnostic-span changes while preserving spans in the artifact text for diagnostics, and reports stale generic interface artifacts with artifact-root context plus regeneration-command suggestions
- emits `.mgb` package implementation artifacts containing MIR-lowered bytecode bodies and package-local source hashes needed for artifact-backed execution and future rebuild planning
- returns package-aware typed HIR from loaded/interface-artifact typed compilation paths
- can lower package-aware typed HIR through the existing HIR/bytecode VM path
- generates in-memory package interface summaries for public records, enums, opaque types, functions, and direct interface dependencies
- validates typed package references against generated summaries
- persists `.mgi` direct dependency metadata and follows those dependencies when artifact-backed checking needs transitive public-signature type interfaces
- includes loaded direct/transitive dependency interface hashes in `.mgc` check cache keys
- writes `.mgc` check cache artifacts only after package-aware artifact checking succeeds
- writes `.mgi`, `.mgb`, and `.mgc` artifacts through `muga build` to `.muga/build` under the nearest manifest root, or under the entry file's directory when no manifest is present, reporting each artifact as `written` or `reused`
- writes or updates `muga.lock` next to `muga.toml` during manifest `muga build`, preserving the file when generated content is unchanged, refreshing well-formed stale local metadata, and rejecting malformed or unsupported existing lockfiles with `PK026`
- computes a `sha256:<hex>` package content hash over `muga.toml`, sorted `.muga` source files under the manifest source root, and sorted files under the optional manifest resource root
- emits deterministic `.mgp` source/resource archives through `emit-package-archive --archive-root <dir> <entry>`, naming the archive with the package name and content hash, and can print a pasteable local archive dependency entry through `--dependency-snippet`
- reads and validates `.mgp` source/resource archive bytes through library APIs, computing the hash from the bytes, optionally checking an expected `sha256:<hex>`, and rejecting malformed or non-canonical manifest/source/resource entry layout
- materializes validated `.mgp` source/resource archive bytes into absent or empty local source/resource trees
- consumes local `.mgp` archive dependencies by hash through `.muga/packages`, rejecting malformed declarations, mismatched archives, stale or colliding source/resource cache entries, package-name mismatches, and malformed archive lockfile metadata
- reads manifest-declared package resources at runtime through `std::fs::read_resource_text(package_path, resource_path)` and `std::fs::read_resource_bytes(package_path, resource_path)`, including source trees, package tests, local archive dependency caches, and explicit built-artifact runs
- emits a non-mutating app bundle through `emit-app-bundle [--source-free] --output-dir <dir> [--program <name>] <entry>`, copying source files and source-hash `muga.lock` metadata by default and omitting root/dependency source trees plus lockfile metadata with `--source-free` while keeping declared resources, bundle-local dependency manifests/resources, `muga.toml`, `.muga/app-bundle` entry metadata, `.muga/build` artifacts, and a `bin/<program>` launcher
- executes app bundles through `run-app-bundle [--format text|json] <bundle-dir> [-- <program-arg>...]` from bundle-local manifests, `.muga/app-bundle`, resources, `.mgi` interfaces, and `.mgb` implementation artifacts without reading copied source files
- installs or updates a non-mutating app bundle wrapper and `<bin-dir>/.muga/installed-apps/<program>.toml` ownership metadata through `install-app [--replace-owned] --output-dir <bin-dir> [--program <name>] <bundle-dir>`, replacing existing launchers/metadata only when ownership metadata matches and never editing shell startup files
- uninstalls only ownership-verified launcher/metadata files through `uninstall-app --output-dir <bin-dir> --program <name>`, leaving bundle directories and shell profiles untouched
- emits generated app completion packages from source-free app bundle interfaces through `emit-app-completions --output-dir <dir> [--program <name>] --type <type> [--package <package>] <bundle-dir>`
- emits and unpacks deterministic `.mga` app bundle archives through `emit-app-archive --archive-root <dir> [--program <name>] <bundle-dir>` and `unpack-app-archive --output-dir <dir> <archive-file>`
- builds package `.mgi` / `.mgb` artifacts by deterministic dependency levels and runs independent same-level package artifact work concurrently
- records each `.mgb` package's own source hash separately from its public interface hash and dependency interface hashes
- preserves unchanged generated `.mgi`, `.mgb`, and `.mgc` artifacts during `muga build`; interface reuse is keyed by the persisted interface hash, while implementation and check-cache reuse currently requires identical generated artifact text
- consumes the same default `.muga/build` directory through `check --built` and `run --built`, reports missing/stale default artifact failures with `muga build <entry>` guidance, and leaves plain `check` and `run` source-compatible
- validates `.mgb` artifacts against loaded `.mgi` interface hashes before artifact-backed execution and rejects missing, stale, hash-mismatched, structurally invalid, wrong-package, or dependency-interface-mismatched implementation artifacts without source-tree fallback
- executes direct and transitive dependency bytecode from `.mgb` artifacts without reading dependency source files from the consumer source tree
- remaps independently generated `.mgb` package item references onto the loaded `.mgi` interface identities and reserves private implementation item ids past the entry program's package item ids before bytecode merge

Artifact-root configuration is intentionally not part of `muga.toml` yet. The current manifest owns package naming, source-root inference, and local path dependency roots. Artifact-backed checking, running, and custom-root artifact emission are explicit CLI workflows through `--artifact-root`, `--built`, `emit-artifacts`, `emit-interface`, and `emit-check-cache`. `muga build` is a fixed default-output convenience over the same artifact writer, and `--built` is a fixed default-input convenience over the same artifact-backed check/run paths; neither is manifest configuration. Project-level artifact-root config should be reconsidered after lockfiles and a package-aware project driver exist, most likely as a non-semantic `[build]` or `[cache]` setting rather than as part of package identity.

This is enough to validate the package surface and the next interface boundary. It is not the final compilation model. The dependency layers in 17.1 to 17.10 are target design, to be implemented incrementally on top of the existing manifest, package graph, typed HIR, and interface-summary work.

## 18. Example

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

## 19. Deferred Topics

This draft intentionally leaves the following topics for later:

- build-target declarations in `muga.toml` (libraries, applications, tests, benchmarks)
- signing, transparency log, and reproducibility verification (extensions to the trust model in 17.10)
- registry policy: scoping rules, ownership transfer, deprecation, and yank semantics
- full source-root discovery rules
- standard library package layout
- selective imports
- wildcard imports
- re-exports
- package-scoped constants or immutable top-level values
- generic packages
- protocol/trait-like abstractions
- testing and benchmark file conventions

## 20. Recommendation

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
