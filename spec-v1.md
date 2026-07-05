# Muga Spec v1

This is the compact v1 overview. The split specifications in [spec/](./spec) are the detailed references; this file exists to show the whole language shape without duplicating every rule.

## Goals

Muga v1 prioritizes:

- simple local reading
- low syntactic overhead
- static typing with minimal annotations
- fast compiler architecture
- predictable package boundaries

The language is compiler-first. The current VM is a reference execution backend, not a separate semantics engine.

## V1 Completion Boundary

The v1 surface syntax is considered closed around the grammar defined in this overview and the detailed specs. Finishing v1 means implementing and documenting that closed surface, keeping runnable samples and rejection tests aligned with it, preserving the explicit package artifact workflow, and passing `scripts/v1-release-gate.sh`.

The v1 grammar includes:

- script files with top-level statements
- package files with `package`, `import`, `pub`, `pkg`, and `as`
- immutable and mutable bindings with optional local type annotations
- `record`, `enum`, package-mode `pub opaque type`, `fn`, and anonymous `fn`
- compiler-recognized `@test` metadata on function declarations for `muga test`
- `if` / `else if`, `while`, `for item in list`, `break`, `continue`, `return expr`, statement-form `using` cleanup, final-expression value blocks, and exhaustive `match`
- ordinary calls, chained calls, package-qualified chained calls, field access, record update, list literals, and list indexing
- prefix `try expr` for `Result[T, E]`
- explicit declaration type parameters on records, enums, and functions
- generic type expressions in annotations and signatures

The following are not planned for ordinary Muga code:

- `class`, inheritance, member-owned methods, member ownership semantics, or class-style encapsulation
- method dispatch as a separate semantic category from ordinary function calls
- overloaded function dispatch, overloaded operator dispatch, or user-defined overload sets
- general `type` declarations or type aliases as alternate spellings for `record`, `enum`, or enum-plus-record combinations
- type aliases added only to shorten public API shapes or avoid explicit `record` / `enum` declarations; package-mode `pub opaque type` is a separate narrow form, not a type alias
- source-level references such as `ref T`, `mut ref T`, `&value`, `*value`, pointer syntax, ownership syntax, borrowing syntax, raw pointer arithmetic, or general writable aliases
- implicit exceptions or `throws`
- postfix Result propagation `expr?`
- `protocol`, `trait`, `interface`, or `typeclass` declarations for shared behavior
- behavior-conformance systems, protocol bounds, trait bounds, typeclass solving, default implementations, blanket implementations, protocol objects, or conformance-based dot lookup

The following are explicitly not v1 completion blockers and are not active
implementation work. Reconsider them only after real Muga programs show that
the current explicit forms are hard to read, easy to misuse, or blocking an
important workflow:

- future Result chain propagation `expr.try`, optional shorthand `T?`, and
  Option-only optional chaining `?.`
- explicit call-site type arguments such as `id[Int](1)`
- wildcard imports, selective imports, re-export syntax, or package top-level execution
- broad catch-all wildcard match arms, nested patterns, match guards, multi-payload enum variants, or named-field enum variants
- map literals, `Set[T]`, arbitrary `Map` key types, broad collection APIs, or iterator abstractions
- `pub opaque record` for user-defined hidden record representations
- concurrency features beyond the implemented structured task groups
  (`group` / `spawn` / `std::task`): channels, `select`, `async`, or `await`
- `String.len()`, substring/slice indexing, and richer parse error types until their semantics are explicitly chosen

## Core Rules

Bindings are immutable by default:

```muga
x = 1
mut total = 0
total = total + 1
```

`x = e` is one syntactic form. Name resolution decides whether it introduces a new immutable binding or updates an existing mutable binding in the current function scope.

Rules:

- no `let`
- `mut x = e` introduces a new mutable binding
- `x = e` introduces an immutable binding when `x` is not already defined in the current scope
- `x = e` updates `x` when `x` resolves to a mutable binding in the same function
- updating an immutable binding is an error
- shadowing is prohibited
- nested blocks in the same function may update enclosing mutable bindings
- inner functions may read outer bindings but may not update them

## Syntax

Comments use `//`. Statements are separated by newlines.

Core expression and statement forms include:

- integer, boolean, and string literals
- short-circuit Boolean `and` / `or`
- `if` / `else if` statements and value-producing `if` expressions
- `while` statements
- `for item in list` statements for `List[T]` values
- `break` and `continue` for the nearest enclosing loop
- explicit `return expr` from the nearest function
- `using name = try acquire() { ... }` statements for runtime-backed opaque
  handles with compiler-known close metadata
- function declarations and anonymous functions
- `@test` on named function declarations, recognized only as static tool metadata
- record declarations, record literals, field access, non-destructive record
  update, and package-mode `pub opaque type` names
- ordinary calls and chained calls
- list literals and list indexing
- exhaustive `match` for compiler-known `Option[T]`, `Result[T, E]`, and user-defined enums, with `_` allowed only as a one-payload variant discard
- `try expr` propagation for `Result[T, E]`
- package declarations and imports in package mode

Type annotations use `:`:

```muga
value: Int = 1
mut items: List[Int] = []
fn add(a: Int, b: Int): Int {
  a + b
}
```

Function types use `->`:

```muga
fn apply(value: Int, f: Int -> Int): Int {
  f(value)
}
```

Generic type expressions use square brackets:

```muga
List[Int]
Option[String]
Result[Int, String]
Map[String, Int]
```

The implementation supports generic type expressions for `List`, `Option`, `Result`, and `Map`, plus explicit user-defined generic records and functions.

Package-mode `pub opaque type Name` declares a public nominal type name with a
hidden representation. It can be used in annotations and public signatures, and
is persisted in `.mgi` interfaces, but ordinary source code cannot construct it,
access fields, match on it, compare it structurally, or format it through
`to_string`. Opaque type declarations are public-only and non-generic in the
current slice.

`pub opaque type` is not Muga's general `type` declaration and is not a type
alias. In v1 it is a narrow interface form for compiler/runtime/native/external
values such as `std::fs::File` and `std::bytes::Bytes`, or for package
interfaces that deliberately do not commit to a source-level field layout.
Ordinary user-defined data should use `record` or `enum`. If real packages need
ordinary Muga records whose fields are hidden outside the defining package, the
separate future candidate is `pub opaque record`, not a general type alias.

The compiler-provided `std::fs::File` is a runtime-backed opaque handle.
`using` introduces one immutable handle binding scoped to its block and runs
the handle's compiler-known close function exactly once after successful
acquisition. Cleanup runs on normal fallthrough and on `try`, `return`,
`break`, and `continue` exits. If cleanup returns `Result::Err`, the enclosing
function returns that error; nested cleanup attempts every active cleanup in
last-acquired, first-closed order and returns the first cleanup error under the
current one-error `Result[T, E]` model.

## Functions And Inference

Type annotations should be omitted when local inference can determine a unique type:

```muga
fn double(x) {
  x * 2
}
```

Annotations remain required when inference is ambiguous, recursive constraints need a stable starting point, or the current implementation has an explicit boundary such as public package signatures.

Function bodies produce their final expression. Use `return expr` only for an explicit early exit from the nearest named or anonymous function; top-level `return` is rejected. `break` and `continue` target the nearest enclosing loop in the same function and are rejected outside loops.

Higher-order functions are supported:

```muga
fn apply(x: Int, f): Int {
  f(x)
}

fn main(): Int {
  apply(10, fn(n) {
    n + 1
  })
}
```

Local bidirectional inference covers selected higher-order cases. When a callback type cannot be inferred uniquely, write the function type.

## Records And Calls

Records are nominal data declarations:

```muga
record User {
  name: String
  age: Int
}
```

Record literals use the record name:

```muga
user = User {
  name: "Ada"
  age: 20
}
```

Field access and update:

```muga
name = user.name
older = user.with(age: user.age + 1)
```

Behavior is modeled with functions. Chained calls are surface syntax over function calls:

```muga
fn birthday(user: User): User {
  user.with(age: user.age + 1)
}

older = user.birthday()
```

`self` is only a conventional parameter name. v1 has no classes, inheritance, receiver overloading, or function-valued record fields.

## Collections And Enum-Like Standard Types

Implemented collection/error core:

- `List[T]`
- `Option[T]`
- `Result[T, E]`
- `Map[K, V]` for `Int`, `Bool`, and `String` keys
- `std::list` helpers `map`, `filter`, `fold`, `any`, and `all`
- `std::map` helpers `keys` and `values`

Examples:

```muga
numbers = [1, 2, 3]
first = numbers.get(0)

match first {
  Option::Some(value) => value
  Option::None => 0
}
```

```muga
result: Result[Int, String] = Result::Ok(1)

match result {
  Result::Ok(value) => value
  Result::Err(message) => 0
}
```

`Option[T]` is the canonical optional spelling. `T?` is reserved as possible future shorthand, and any future `?.` syntax should be Option-only optional chaining. `Result[T, E]` is explicit; prefix `try expr` propagates `Result` errors, while postfix Result propagation `expr?` is not part of v1. If a future Result chain propagation form is added, it should use postfix keyword syntax `expr.try`; that is also outside v1.

User-defined enum declarations are implemented with optional unconstrained type parameters, zero-payload and one-payload variants, qualified constructors/patterns, payload discard `_` inside one-payload variant patterns, and exhaustive `match`.

## Packages

Script mode is for standalone files. Package mode starts with `package` or is inferred from a nearby `muga.toml`.

```muga
package app::main

import util::numbers

fn main(): Int {
  1.numbers::inc_twice()
}
```

Package-mode visibility:

- unmodified top-level items are module/file-private
- `pkg` items are visible to sibling files in the same package
- `pub` items are importable from other packages

Package interfaces and implementation artifacts are explicit v1 workflow artifacts:

- `.mgi` stores public package interfaces
- `.mgc` stores package check-cache proofs
- `.mgb` stores MIR-lowered bytecode implementation artifacts

`muga check --artifact-root` and `muga run --artifact-root` consume those artifacts without reading dependency implementation bodies. Normal package execution without `--artifact-root` remains source-compatible and may still read dependency source bodies.

`muga build <entry>` writes the same artifact set to the default `.muga/build` directory, preserving unchanged artifacts and reporting `written` / `reused` status for each path. `muga build --format json <entry>` reports the same artifact root, artifact paths, artifact kinds, and written/reused status in one JSON object. `muga emit-artifacts --format json --artifact-root <dir> <entry>`, `muga emit-interface --format json --artifact-root <dir> <entry>`, and `muga emit-check-cache --format json --artifact-root <dir> <entry>` report the explicit artifact root plus emitted artifact paths, artifact kinds, and URIs in one JSON object. `muga check --built <entry>` and `muga run --built <entry>` consume that default directory explicitly. `run` also accepts program arguments after `--`, including `run --built <entry> -- arg`.

`muga syntax --format json <entry>` lexes and parses one source file for faster editor feedback. It emits the same diagnostic JSON envelope as `check`, including entry source context in `diagnostics[].context`, but does not run resolver, typechecker, package import loading, or artifact checks. Package `check --format json` diagnostics add entry package context when available, and artifact-backed checks also add artifact-root context. Artifact diagnostics that know a concrete `.mgi`, `.mgc`, or `.mgb` path add artifact-file context with the artifact kind and `file://` URI. Stale or hash-mismatched artifact diagnostics also add artifact hash, source hash, dependency interface hash, and regeneration-command context when the compiler has that data. `muga run --format json <entry>` reports captured program stdout, the currently empty program stderr channel, the returned `main` value when present, and compiler/runtime diagnostics as one schema-versioned JSON object. `muga explain <diagnostic-code>` prints `errors.md` diagnostic guidance for exact catalog entries or stable diagnostic-code families. `muga test --format json <entry>` reports discovered tests, pass/fail status, failure messages, per-test stdout, summary counts, and pre-run compiler diagnostics as one schema-versioned JSON object.

`muga metadata --format json <entry>` checks a package entrypoint and emits
package/module/item/export metadata plus public interface docs and rendered
types for editor, LSP, CI, and agent consumers. Public interface metadata
includes opaque `handleFacts` and function-parameter `paramMode`; source-defined
functions currently report `borrow` parameters only. It is a tooling contract,
not a source-language feature.

`muga workspace --format json <entry>` checks a package entrypoint and emits
workspace metadata for editor and LSP tooling. The initial tooling contract
covers loaded packages, module source files, the default artifact root, and
dependency edges reachable from the entrypoint; it is not a full project
workspace mode.

`muga completions --format json <entry>` checks a package entrypoint and emits
visible package/interface completions for editor and LSP tooling. Completion
items include import aliases plus public records, enums, opaque types, and
functions from the entry package and directly imported packages, including
public docs, signatures, and opaque/function metadata.

`muga definition --format json --line <line> --column <column> <entry>` checks
a package entrypoint and emits go-to-definition data for the selected source
position. The initial tooling contract covers import aliases, local bindings,
and package/interface item references with compiler-owned package/module/item
ids and source spans.

`muga references --format json --line <line> --column <column> <entry>` checks
a package entrypoint and emits find references data for the selected source
position. The initial tooling contract covers declaration plus entry module
references for import aliases, local bindings, and package/interface item
references.

`muga hover --format json --line <line> --column <column> <entry>` checks a
package entrypoint and emits declaration hover data for the selected source
position. Public declarations include public docs and signatures from the same
package interface model.

## Value Semantics

Ordinary source code uses value semantics. APIs that update ordinary data should return new values, as `List.push`, `List.set`, `Map.insert`, `Map.remove`, and `record.with(...)` do.

The implementation may optimize with internal sharing, copy elision, builders, buffers, resource handles, MIR lowering, or native backend work. These optimizations must not change source-level meaning.

Explicit source-level references such as `ref T`, `mut ref T`, `&value`, `*value`, and raw pointer syntax are not planned for ordinary Muga code.

## Current Implementation Boundary

Implemented:

- parser/resolver/typechecker/HIR/bytecode/VM pipeline
- typed HIR and MIR-lowered bytecode for the reference VM
- records, enums, functions, local inference, closures, higher-order functions, and explicit generic records/functions
- package-mode `pub opaque type` names in signatures and `.mgi` interfaces,
  including the compiler-provided runtime-backed `std::fs::File` handle
- exhaustive `match` for `Option[T]`, `Result[T, E]`, and user-defined enums, including payload discard `_` inside one-payload variant patterns
- prefix `try expr` propagation for `Result[T, E]`
- short-circuit Boolean `and` / `or`
- `else if`, explicit `return expr`, `break` / `continue`, and `for item in list`
- statement-form `using` cleanup for runtime-backed opaque handles
- scalar-only `==` / `!=` equality for `Int`, `Bool`, and `String`
- packages, module privacy, `pkg`, `pub`, import aliases, package-aware checking, and minimal manifest project mode
- local path dependencies, local `.mgp` archive dependencies, and minimal local path/archive `muga.lock` metadata
- `List`, `Option`, `Result`, `Map`, and the current `String` helper slice
- `std::option` and `std::result` helper packages for value transformation without propagation syntax
- `std::fs::FileMetadata` plus `std::fs::file_metadata_path(file_path)` for regular-file size and modified-time metadata
- `std::fs::PathStatus`, `PathKind`, `PathInfo`, `PathMetadata`, and `PathSizeMetadata` plus `std::fs::path_status(target_path)`, `path_kind(target_path)`, `path_info(target_path)`, `path_metadata_path(target_path)`, and `path_size_metadata_path(target_path)` for grouping existing path status predicates, modified-time metadata, and optional regular-file size while broader metadata fields remain deferred
- `std::fs::read_dir_recursive_path(root_path)` and `std::fs::DirectorySizeMetadata` / `directory_size_metadata_path(root_path)` for deterministic read-only descendant traversal and recursive byte/count aggregation while globbing, public symlink classification, and sandbox policy remain deferred
- `std::fs::remove_dir_all_path(dir_path)`, `std::fs::copy_dir_all_path(from_path, to_path)`, and `std::fs::move_dir_all_path(from_path, to_path)` for recursive directory removal, no-overwrite recursive directory copy, and no-overwrite copy-then-remove directory move while trash/recycle-bin integration, globbing, public symlink classification, merge/overwrite copy policy, atomic rename fallback, rollback, and sandbox policy remain deferred
- `std::fs::write_bytes` and `std::fs::write_bytes_path` for full-file binary writes over opaque `std::bytes::Bytes` while binary handles, streams, codecs, and mutable buffers remain deferred
- `std::string` helper package with `string::concat_all` and `string::join` for pure `List[String]` concatenation and separator joins while keeping non-string conversion explicit
- `std::fmt` helper package with `fmt::repeat`, `fmt::pad_left`, `fmt::pad_right`, `fmt::truncate_chars`, and `fmt::format_values` for pure formatting over explicit `String` values while language interpolation, localization, and builders remain deferred
- `std::list` and `std::map` helper packages for narrow collection transformations and key/value extraction
- `std::cli` helper package for pure positional, long-flag, single/repeated long-option lookup, typed `Int` / `Bool` parsing, compiler-owned `cli::parse_or[T](args, defaults)`, strict `cli::parse[T](args)`, and `cli::usage_for[T](program, defaults)` over explicit `List[String]` argument values, returning recoverable `cli::Error` values including `MissingArgument`
- `std::process` helper package for direct child process execution through
  `process::run(command, args)` and `process::run_with(command, args,
  options)`, with `Options { cwd: Option[path::Path], env: List[EnvVar] }`,
  `Output { status, success, stdout, stderr }`, nonzero exits captured as
  `Result::Ok(Output)`, and spawn/wait/capture/UTF-8 failures returned as
  recoverable `process::Error` values
- structured task groups per
  [spec/007-concurrency-draft.md](./spec/007-concurrency-draft.md) section 5:
  `group { ... }` expression scopes, prefix `spawn` allowed only inside an
  enclosing `group` body (`T030`) with `spawn group { ... }` for nested
  scopes, immutable-only capture across the `spawn` boundary (`E013`), an
  internal `Task[T]` handle type that user source cannot spell (`T013`), and
  the `std::task` helper package whose `task::join(handle)` /
  `handle.task::join()` returns the completed child value; the reference VM
  executes deterministically, running each child task to completion at its
  spawn site within implementation-defined ordering
- `std::json` helper package for explicit `json::Value` parse/encode, integer-number conversion, value/object-field scalar/composite accessor/default/required helpers, scalar array projection helpers, direct scalar-array object-field helpers, typed-segment JSON path helpers, typed JSON path scalar projection helpers, typed JSON path collection projection helpers, and compiler-owned `json::decode_or[T](value, fallback)` / strict `json::decode[T](value)` schema decoding for `String`, `Int`, `Bool`, `Option[T]`, recursive `List[T]`, typed `Map[String, T]`, `Map[String, json::Value]`, concrete non-generic records over supported fields, and concrete non-generic enums over supported payloads, all returning `json::Error`
- compiler-provided `std::test` scalar assertion helpers for `muga test`
- `muga --help`, `muga -h`, `muga help`, and `muga help <command>` for command usage
- `muga fmt [--check]` for deterministic formatting of v1 source files while preserving line comments
- `muga doc <entry>` for Markdown docs generated from public package interface records, enums, opaque types, functions, and item-level public source comments
- `muga syntax --format json <entry>` for faster editor feedback from lexing and parsing one source file
- entry source context in CLI JSON `diagnostics[].context`, plus entry package context for package check diagnostics, artifact-root context for artifact-backed check diagnostics, and concrete artifact-file context for `.mgi`, `.mgc`, and `.mgb` diagnostics when available
- `muga explain <diagnostic-code>` for command-line diagnostic guidance backed by `errors.md`
- `muga run --format json <entry>` for machine-readable run stdout, stderr, returned `main` value, and diagnostics
- `muga test --format json <entry>` for machine-readable `@test` results, captured per-test stdout, and test summary counts
- `muga metadata --format json <entry>` for package/module/item/export metadata plus public interface docs and rendered types
- `muga workspace --format json <entry>` for workspace metadata with loaded packages, module source files, the default artifact root, and dependency edges
- `muga completions --format json <entry>` for visible package/interface completions with import aliases plus public docs and signatures
- `muga definition --format json --line <line> --column <column> <entry>` for go-to-definition data over import aliases, local bindings, and package/interface item references
- `muga references --format json --line <line> --column <column> <entry>` for find references data over import aliases, local bindings, and package/interface item references in the entry module
- `muga hover --format json --line <line> --column <column> <entry>` for declaration hover data with public docs and signatures
- `muga new --list-templates [--format json]` for listing available starters, and `muga new [--template app|lib|test|config-app|cli-tool|report-app|resource-export|package-app] <project-dir>` for creating an app, library package, package with tests, config app, strict CLI tool, report app, resource export app, or app plus local library starter; the app template is a small `std::env` / `std::cli` starter that prints `hello Muga` by default and accepts a positional name or `--name`, the config app template generates `config/settings.json` and uses `std::config::load_json_or[T]`, the cli-tool template uses strict `cli::parse[T]`, the report-app template uses `std::fs` plus `std::path::with_extension` to read a text input and write a sidecar summary, the resource-export template uses manifest resources plus `std::bytes` / `std::hash` / `std::fs::PathMetadata` to write and verify a bundled binary payload, and the package-app template generates sibling `app/` and `shared/` packages using a local path dependency
- persisted `.mgi` package interfaces, `.mgc` check-cache artifacts, and `.mgb` implementation artifacts
- `muga build --format json <entry>` for structured `.mgi`, `.mgc`, and `.mgb` artifact status
- `muga emit-artifacts --format json --artifact-root <dir> <entry>`, `muga emit-interface --format json --artifact-root <dir> <entry>`, and `muga emit-check-cache --format json --artifact-root <dir> <entry>` for structured explicit artifact emission output
- artifact-backed `check` and `run`, including `--built`, without dependency implementation source fallback

Concrete enum JSON/config decoding uses zero-payload string tags and one-payload
single-key objects. For schema polish, record fields and enum variants can use
`@json(rename: "...")` to decode external wire names while constructing Muga
values with source-level field and variant names, and record declarations can
use `@json(deny_unknown_fields)` to reject unexpected JSON/config object keys.
Record fields and enum variants can also use input-only
`@json(alias: "...")` metadata to accept legacy JSON/config names; aliases share
the strict accepted-key set and primary/alias conflicts are rejected or reported
as decode ambiguity. Record fields can use narrow `@validate(...)` metadata:
`non_empty`, `min_len`, and `max_len` for `String` / `Option[String]`, plus
`min` and `max` for `Int` / `Option[Int]`. Validation failures return
path-aware `json::ErrorKind::Validation` values through `std::json`; through
`std::config`, the same decode failure is reported as `config::ErrorKind::Decode`
with the validation message and offset.
Generic enum decoding, record-level or cross-field validation, user-defined
validator functions, and TOML remain outside the v1 JSON/config decoder surface.

Not implemented:

`std::process` and structured task groups (`group` / `spawn` / `std::task`)
are implemented. The remaining items in this list are parked work unless the
roadmap promotes a specific correctness or release-readiness issue to P0.

- public-signature inference for `pub fn`
- URL/Git/registry dependency forms, remote package fetching, publishing/install workflows, and full published-package lockfile enforcement
- project-mode artifact-root configuration and full incremental package artifact reuse
- structural equality, map literals, `Set[T]`, arbitrary `Map` key types, and broad collection APIs
- broader JSON schema decoding targets such as generic records, generic enums,
  nested `Option[Option[T]]`, non-string map keys, record-level or cross-field
  validation, user-defined validators, or stricter schema policies beyond the implemented
  `json::decode_or[T]`, `json::decode[T]`, `config::load_json_or[T]`, and
  `config::load_json[T]`
  target set plus opt-in `@json(deny_unknown_fields)` and input-only
  `@json(alias: "...")` / field-level `@validate(...)`
- source-level consuming parameter declarations, broader runtime-backed
  resource-handle families, `using` expressions/multiple bindings, and
  aggregate cleanup errors
- control-flow-oriented MIR and native backend
- concurrency beyond structured task groups: channels, `select`, timeouts,
  deadlines, detached tasks, and any parallel scheduler behind the
  deterministic reference execution

## Detailed References

- [spec/001-core-language.md](./spec/001-core-language.md)
- [spec/002-name-resolution.md](./spec/002-name-resolution.md)
- [spec/003-typing.md](./spec/003-typing.md)
- [spec/004-functions.md](./spec/004-functions.md)
- [spec/005-records.md](./spec/005-records.md)
- [spec/006-packages.md](./spec/006-packages.md)
- [spec/007-concurrency-draft.md](./spec/007-concurrency-draft.md) (section 5
  is the implemented structured task groups specification; other sections are
  design drafts)
- [spec/008-collections.md](./spec/008-collections.md)
- [spec/009-generics.md](./spec/009-generics.md)
- [spec/010-references-decision.md](./spec/010-references-decision.md)
- [spec/011-value-semantics.md](./spec/011-value-semantics.md)
- [spec/013-enums-results.md](./spec/013-enums-results.md)
