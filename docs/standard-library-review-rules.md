# Standard Library Review Rules

Status: v1 maintenance design. Use this checklist before adding or broadening
compiler-provided `std::*` packages, prelude helpers, or runtime-backed
standard-library functions.

## Purpose

Muga's standard library should make effects visible, keep package interfaces
stable, and avoid turning convenience APIs into hidden language semantics. A
new standard-library slice should be small enough to review as a public
contract and should preserve the current v1 direction:

- absence is represented by `Option[T]`
- recoverable effects return `Result[T, E]`
- public error types are explicit records or enums
- property access must not perform hidden IO
- runtime-backed resources use opaque resource handles before broad IO APIs
- public signatures must be representable in `.mgi`

## Review Checklist

### 1. Scope

- The slice solves one narrow workflow and names the deferred follow-up APIs.
- It does not introduce a broad platform surface such as HTTP, process
  management, streaming IO, async runtime hooks, or registry behavior without a
  separate design note.
- It can be taught through one focused sample and a small set of diagnostics.
- It does not require new syntax unless the v1 checklist and specs explicitly
  move that syntax into scope first.

### 2. Public Contract

- Every public function has an explicit signature.
- Public records, enums, type aliases, and opaque types have stable
  names and package paths.
- Public types are representable in `.mgi` and can be compared by the API diff
  rules in [mgi-api-diff.md](mgi-api-diff.md).
- Adding, removing, or changing public enum variants, record fields, function
  parameters, or return types is treated as a compatibility decision, not an
  implementation detail.
- If the API is compiler-provided or virtual, artifact-backed `check` and `run`
  must work without reading private source bodies from the dependency tree.

### 3. Effects And Absence

- A pure helper returns the value directly and must not read files, environment
  variables, clocks, process state, network sockets, or global mutable state.
- A recoverable operation returns `Result[T, E]`; it should not encode ordinary
  failures as sentinel values, empty strings, panics, or runtime traps.
- A missing value that is not an error returns `Option[T]`.
- A function that can both be absent and fail should model both cases
  explicitly, for example `Result[Option[T], E]` when absence is a successful
  outcome.
- Property access and record fields are data access only. Do not add
  properties that perform hidden IO, block, allocate external resources, mutate
  external state, or depend on the current time.
- Time, environment, filesystem, process, and network APIs must make
  nondeterminism visible through function names and signatures.

### 4. Error Types

- Public effect APIs use public error records or enums, not unstructured
  `String` errors, unless the slice is explicitly temporary and documented as
  such.
- Error types include stable machine-readable fields before relying on display
  text. Current filesystem errors use `io::IOError` and `io::PathPairError`
  with operation, path details, kind, message, and `raw_code: Option[Int]`.
- Single-path operations should use a single-path error type. Two-path
  operations should preserve both path roles in the error type.
- Errors should carry enough context for diagnostics and user recovery without
  requiring private implementation bodies.
- Nonrecoverable compiler or runtime bugs may still be hard errors; ordinary
  OS, parse, validation, and user-data failures should be modeled through
  `Result`.

### 5. Runtime-Backed Values

- Transparent records are appropriate for plain data such as `path::Path` and
  `time::UnixMillis`.
- Runtime-backed values such as files, sockets, timers, child processes,
  streams, and HTTP connections must not be represented as transparent records
  that users can forge.
- Add broad handle-based APIs only after `pub opaque type` and ownership /
  cleanup rules are designed. The current boundary is
  [opaque-resource-handles.md](opaque-resource-handles.md).
- A future resource API must document close behavior, error behavior during
  cleanup, task-boundary movement, cancellation, and whether handles are
  cloneable or shareable.

### 6. Naming

- Effectful functions use verbs such as `read`, `write`, `create`, `remove`,
  `copy`, `open`, `close`, `spawn`, or `now`.
- Pure predicates use names such as `is_*` and return `Bool`.
- Optional lookups use names that make absence ordinary, such as `get_var`,
  `file_name`, `parent`, `extension`, or future `find_*` helpers.
- APIs that accept `std::path::Path` should say so in their signature. Keep
  string-path convenience APIs narrow and avoid adding parallel names unless
  both forms have clear use cases.
- Ambiguous convenience names should wait until semantics are stable. For
  example, broad `len`, formatting, slicing, globbing, recursive removal,
  rename, and directory-copy APIs need separate semantics before becoming
  standard conveniences.

### 7. Tests And Docs

- Add focused source tests for success, recoverable failure, type mismatch, and
  import/package visibility behavior.
- If the API is compiler-provided, cover emitted `.mgi` / `.mgb` artifacts and
  artifact-backed execution.
- Add diagnostics tests for unsupported argument types and stale or missing
  artifact cases when relevant.
- Add or update a runnable sample when the API is user-facing.
- Update README, the practical readiness notes, and any affected specs before
  treating the API as stable.

## Current Standard-Library Reading

The current implemented slices follow these rules:

- `std::string` and `std::fmt` keep text assembly and layout explicit:
  `std::fmt` only covers repeat, left/right padding, and scalar truncation as
  documented in [std-fmt-text-layout.md](std-fmt-text-layout.md), leaving
  templates, interpolation, format specifiers, localization, and builders
  separate.
- `std::path` exposes transparent `Path` data plus pure path operations,
  including lexical cleanup as documented in
  [path-normalize.md](path-normalize.md), file-name replacement as documented in
  [path-with-file-name.md](path-with-file-name.md) and extension replacement
  as documented in [path-with-extension.md](path-with-extension.md), plus
  component-aware prefix stripping as documented in
  [path-strip-prefix.md](path-strip-prefix.md).
- `std::fs` exposes text-file and path filesystem operations through explicit
  functions returning `Result`, including scalar `file_size_path` metadata as
  documented in [fs-file-size.md](fs-file-size.md), modified-time metadata as
  documented in [fs-modified-unix-millis.md](fs-modified-unix-millis.md), the
  regular-file `FileMetadata` record as documented in
  [fs-file-metadata-record.md](fs-file-metadata-record.md),
  path status/kind/info/metadata grouping as documented in
  [fs-path-status.md](fs-path-status.md) and
  [fs-path-info.md](fs-path-info.md), plus existing-path metadata as documented
  in [fs-path-metadata.md](fs-path-metadata.md), plus optional regular-file
  size metadata as documented in
  [fs-path-size-metadata.md](fs-path-size-metadata.md),
  deterministic read-only recursive directory listing as documented in
  [fs-read-dir-recursive.md](fs-read-dir-recursive.md), and recursive
  directory size metadata as documented in
  [fs-directory-size-metadata.md](fs-directory-size-metadata.md),
  recursive directory removal as documented in
  [fs-remove-dir-all.md](fs-remove-dir-all.md),
  recursive directory copy as documented in
  [fs-copy-dir-all.md](fs-copy-dir-all.md),
  recursive directory move as documented in
  [fs-move-dir-all.md](fs-move-dir-all.md),
  existing-path `canonicalize_path` resolution as documented in
  [fs-canonicalize-path.md](fs-canonicalize-path.md), and one-step
  `rename_path` as documented in [fs-rename-path.md](fs-rename-path.md).
- `std::io` exposes public error records that downstream packages can name or
  match without reading private implementation bodies.
- `std::env::get_var` returns `Option[String]` because a missing environment
  variable is ordinary absence.
- `std::env::current_dir` returns `Result[path::Path, io::IOError]` because
  the ambient current directory can fail and must not imply config discovery,
  path canonicalization, or process execution.
- `std::env::temp_dir` returns `Result[path::Path, io::IOError]` because the
  ambient temporary-directory convention can produce a path that is not valid
  Unicode and must not imply unique temp-file allocation or cleanup.
- `std::cli` is pure over explicit `List[String]` values; lookup and typed
  `Int` / `Bool` parsing helpers do not read ambient process state or own
  global parser state.
- `std::time::now_unix_millis` is explicitly named as a nondeterministic clock
  read and returns transparent timestamp data.
- `std::test` exposes scalar assertion helpers as ordinary functions returning
  `Result[Unit, String]`, so `muga test` failures remain explicit and can use
  `try` without special assertion syntax.
- `std::list` and `std::map` expose helper functions that avoid structural
  equality; v1 equality remains scalar-only for `Int`, `Bool`, and `String`.

The completed stdlib package docs and samples review for `std::io`, `std::fs`,
`std::path`, `std::env`, `std::cli`, `std::time`, `std::string`, `std::fmt`,
and the first `std::json` slice plus accessor follow-up is recorded in
[stdlib-package-samples-review.md](stdlib-package-samples-review.md). It maps
the current runnable samples to artifact-backed execution tests and should be
updated whenever one of those public package contracts changes.

## Deferred Until Separate Design

Do not add these as incidental standard-library growth:

- stdout/stderr handles, stdin handles, or file handles before opaque resource
  handles exist and are implemented from the selected boundary in
  [post-json-stdlib-boundary-selection.md](post-json-stdlib-boundary-selection.md)
  and [opaque-resource-handles.md](opaque-resource-handles.md); the
  post-file-handle selection in
  [post-file-handle-resource-surface-selection.md](post-file-handle-resource-surface-selection.md)
  has implemented scalar program stderr and text output file handles, not
  standard-stream handles; text output file handles are implemented from
  [text-output-file-handles.md](text-output-file-handles.md) and future growth
  should stay inside a fresh contract
- binary `Bytes`, buffers, encoders, or streaming APIs before byte/string
  ownership and allocation behavior is documented
- richer all-path filesystem metadata records, accessed/created timestamps,
  merge/overwrite directory copy or move, atomic directory move fallback,
  rollback, globbing, strict path component validation, symlink
  policy, or permission mutation without filesystem semantics and failure cases
- process execution, signals, environment mutation, HTTP, SSE, WebSocket, RPC,
  or service runtime APIs before cancellation and resource cleanup rules
- `std::json` growth outside the first package contract in
  [std-json-first-slice.md](std-json-first-slice.md) and the implemented
  accessor follow-up, which fix the `Result` ergonomics, scalar/collection
  mapping, schema evolution, diagnostics, and `json::Error`-returning
  object-field accessor boundary and are audited in
  [std-json-implementation-audit.md](std-json-implementation-audit.md), plus
  schema/client generation, HTTP/RPC, `Float`, `Decimal`, `Bytes`, streaming
  APIs, or resource handles before those separate policies are settled

When in doubt, keep the first slice smaller and make the deferred decision
explicit.
