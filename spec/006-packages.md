# Packages and Modules Draft

Status: draft with an implemented front-end subset. The current Rust compiler supports explicit `package`, `import`, `pkg`, `pub`, `alias::Name` lookup, directory-based packages, module/file identity for top-level items, top-level module-private visibility, a minimal `muga.toml` project mode that infers package paths from `name` and `source`, explicit `.mgi` / `.mgc` artifact workflows, and a library-only package-aware check scaffold that validates package boundary rules plus source/module signatures over the unflattened package graph while retaining module body-check and package-wide typed HIR outputs. Dependency manifests, registries, selective imports, package-aware execution without flattening, package-level caching, and any future per-field record visibility are still deferred.

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

Full dependency manifest syntax is deferred. The current implementation supports only a minimal `[package]` manifest with `name` and `source`.

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
script_file   := stmt*
package_file  := package_decl? import_decl* package_item*
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

- if present, `package` must be the first significant token in the file
- without manifest inference, `package` is required
- `import` declarations must come after `package` when it is present and before the first item
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

Current implementation note: `pub fn` declarations still require explicit parameter and return annotations. Inferred public signatures are the target package-interface policy, not current behavior.

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

In the committed v1 model, a `pub record` is transparent: its field names and field types are part of the public record shape. Importing packages can use record literals, field access, and `record.with(...)` for those public fields.

If a representation should be hidden inside a module or package, keep the record itself non-public and expose functions that do not leak that non-public type across a wider visibility boundary.

If a package later needs to expose a public type name while hiding its representation, the preferred future direction is an opaque representation feature:

- `pub opaque record` for ordinary Muga data whose fields are visible only to the defining module
- `pub opaque type` for runtime/native handles or values whose representation should not be source-level fields

Per-field visibility is a weaker candidate and should be reconsidered only if concrete code needs partially transparent public records.

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
- the precomputed package interface file, if present
- nothing else (no VCS metadata, no editor files, no build outputs)

Hash representation: `sha256:` followed by lower-case hexadecimal. Other algorithms are reserved for future use behind the same `<algo>:<hex>` form.

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

URL form does not prescribe a file extension or archive format. The URL must return archive bytes when fetched over HTTPS; supported formats include `.tar.gz`, `.tar.zst`, `.zip`, the Muga-native `.pkg`, and any future addition. The format is identified by the HTTP `Content-Type` header, falling back to magic bytes in the response body. This keeps URL form host-agnostic and format-agnostic — GitHub Releases, S3, self-hosted Artifactory, or a plain static file server are all valid.

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
- do not contribute a content hash to the lockfile; the path entry is recorded instead
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

The current implementation has a local package loader, minimal manifest mode, package identity data, in-memory package interface summaries, persisted package interface artifacts, and explicit package check cache artifacts. It does not yet have dependency declarations, published-package content hashing, lockfiles, registries, package archives, or automatic project-level artifact reuse.

It currently:

- accepts a file entrypoint
- discovers `muga.toml` by walking up from the entry file
- supports `[package] name = "..."` and `source = "..."`
- infers package paths from source-root-relative directories in manifest project mode
- infers the source root from the entry file path and declared package path in explicit file-based package mode
- reads all `.muga` files in each loaded package directory
- follows `import` declarations recursively within the local source tree
- rejects import alias collisions
- enforces module-private, `pkg`, and `pub` top-level visibility before flattening
- records package, module, and item identity in `PackageSymbolGraph`
- routes public import lookup through `PackageExportGraph`
- can return an unflattened package graph containing package files plus package/module/item/export metadata
- can build source and module package signatures from the unflattened graph while preserving package item identity and module/same-package/import visibility for records, enums, and functions
- can run and retain an initial package-aware module body typecheck pass against those source/module signatures, including per-module typed HIR outputs lowered with package binding identity
- exposes package-wide typed HIR aggregated from unflattened module check outputs with remapped local IDs and symbols
- can collect dependency signatures directly from loaded in-memory or persisted package interfaces for package-aware module checks without reading dependency implementation source
- can build loaded-interface dependency package graph metadata directly from package interfaces without dependency AST stubs
- routes `muga check --artifact-root` through the package-aware artifact path
- emits `.mgi` interface artifacts from the package-aware typed HIR aggregate
- returns package-aware typed HIR from loaded/interface-artifact typed compilation paths
- can lower package-aware typed HIR through the existing HIR/bytecode VM path
- generates in-memory package interface summaries for public records, enums, functions, and direct interface dependencies
- validates typed package references against generated summaries
- persists `.mgi` direct dependency metadata and follows those dependencies when artifact-backed checking needs transitive public-signature type interfaces
- includes loaded direct/transitive dependency interface hashes in `.mgc` check cache keys
- still flattens loaded packages into one internal program before the main resolver/typechecker/runtime path

Artifact-root configuration is intentionally not part of `muga.toml` yet. The current manifest owns only package naming and source-root inference. Artifact-backed checking and artifact emission are explicit CLI workflows through `--artifact-root`, `emit-artifacts`, `emit-interface`, and `emit-check-cache`. Project-level artifact-root config should be reconsidered after dependency declarations, lockfiles, and a package-aware project driver exist, most likely as a non-semantic `[build]` or `[cache]` setting rather than as part of package identity.

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
