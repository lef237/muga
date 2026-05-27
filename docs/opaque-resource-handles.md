# Opaque Resource Handle Boundary

Status: design boundary with the interface-only `pub opaque type` slice,
metadata-only handle facts, consuming-parameter diagnostics, runtime-backed
`std::fs::File` text handles, and statement-form `using` cleanup implemented.
This document does not implement `Bytes`, process APIs, HTTP, streaming APIs,
or broader filesystem APIs. The post-file-handle selection is recorded in
[post-file-handle-resource-surface-selection.md](post-file-handle-resource-surface-selection.md).

## Purpose

Muga's current standard library uses transparent data records for plain values
such as `path::Path`, `time::UnixMillis`, `io::IOError`, and
`json::Error`. Runtime-owned values such as open files, sockets, listeners,
timers, child processes, streams, HTTP clients, and task handles are different:
source code should be able to name them, pass them, and close them without being
able to construct or inspect their representation.

The first resource boundary is therefore:

```muga
pub opaque type Name
```

This is a public type name with a hidden representation. It is not a record, it
does not expose fields, and it is not constructible outside the defining
package or compiler runtime.

## Source Contract

The future source form is package-mode only:

```muga
package std::fs

pub opaque type File

pub fn open(path: path::Path): Result[File, io::IOError]
pub fn close(file: File): Result[Unit, io::IOError]
```

Initial rules:

- opaque type declarations are top-level package declarations;
- `pub opaque type` names can appear in public function parameters, return
  types, records, enums, and generic type arguments;
- module-private opaque types may be allowed later, but the first implementation
  should focus on public names because `.mgi` representation is the release
  boundary;
- callers can name an imported opaque type but cannot construct it, match on
  it, access fields, compare it structurally, format it through `to_string`, or
  serialize it by default;
- constructors for runtime-backed handles are ordinary functions such as
  `open`, `listen`, `spawn`, or `now_timer`, and recoverable failures return
  `Result`.

Opaque types are nominal. `std::fs::File` and `std::net::Socket` remain
different even if their runtime representation is currently the same.

## Package Interface Contract

`.mgi` must represent opaque public names directly instead of lowering them to
records or aliases. A future persisted interface entry should include:

- package path;
- item kind `opaqueType`;
- public name;
- type parameter list, currently empty for implemented opaque type names;
- future capability facts for resource-like behavior;
- optional doc comments and source spans for tooling;
- a stable contribution to the public interface hash.

No private runtime token, OS descriptor, allocator pointer, file path, or hidden
field layout belongs in `.mgi`.

Downstream checking must be able to use opaque type names from loaded
interfaces without reading dependency source bodies. Artifact-backed `check`,
`run`, `doc`, `metadata`, hover, completion, definition, references, and future
API diff tooling should treat opaque type identity like other package item
identity: package path plus public name plus item kind.

## API Diff Contract

The `.mgi` API diff rules should classify opaque type changes as follows:

- adding a new public opaque type is source-compatible metadata;
- removing or renaming a public opaque type is breaking;
- changing an item from opaque type to record, enum, function, or alias is
  breaking;
- changing type parameter arity is breaking;
- changing capability facts is breaking when it removes an allowed use, and
  source-compatible but reviewable when it only adds a capability;
- changing documentation-only text remains source-compatible metadata.

Unknown future opaque-type metadata should fail closed until this document or
the API diff design defines a compatibility rule.

## Resource Capability Defaults

Opaque resource handles use explicit capability facts. The default for a
runtime-backed resource handle is:

- not copyable;
- not cloneable;
- not shareable across tasks;
- not sendable across tasks;
- not structurally comparable;
- not serializable;
- closeable only through documented APIs or future lexical cleanup.

Capabilities are opt-in. A type such as a future `time::Timer` or `task::Task[T]`
must say whether it can be sent or shared across task boundaries before APIs
can move it there.

These facts are narrower than a general ownership or borrowing system. Ordinary
Muga values remain value-oriented; resource handles get special lifecycle facts
because they represent external state.

## Consuming Operations

Some functions consume a handle. A consuming operation transfers ownership of
the resource into that call, and using the consumed binding afterward should be
a type error.

Examples of consuming operations:

- `fs::close(file): Result[Unit, io::IOError]`
- future task transfer when a handle is `sendable`;
- future stream shutdown APIs.

The first implementation should avoid a broad annotation system if possible.
Compiler-provided packages may encode consuming-parameter metadata in their
virtual package definitions and `.mgi` artifacts before user-defined consuming
parameters are exposed. Once user-defined consuming APIs are needed, the syntax
and `.mgi` representation must be designed explicitly.

Non-consuming operations may use the handle without invalidating it:

```muga
try fs::write(file, "hello")
try fs::flush(file)
```

The exact call shape can be free functions or package-qualified chained calls,
but property access must remain data access and must not perform hidden IO.

## Close And Cleanup

Explicit close is the first cleanup contract:

```muga
result: Result[Unit, io::IOError] = fs::close(file)
```

Rules:

- close returns `Result[Unit, E]` for recoverable cleanup failures;
- close consumes the handle on success or recoverable failure, because the
  runtime state after a failed close may be unusable or host-specific;
- a second close of the same source binding should be rejected before runtime
  when the typechecker can see it;
- if a stale, already-closed, or invalid runtime handle still reaches the VM,
  the runtime reports a diagnostic rather than silently ignoring it;
- best-effort runtime cleanup may exist as a leak-prevention fallback, but
  observable cleanup errors must not be silently discarded when source code
  asked for explicit cleanup.

Statement-form `using` now provides lexical cleanup for runtime-backed opaque
handles with close metadata. It keeps cleanup errors explicit through
`Result[T, E]`, attempts nested active cleanups in last-acquired, first-closed
order, and leaves aggregate cleanup errors to a later contract.

## Task Boundaries And Cancellation

The default handle is not sendable and not shareable across task boundaries.
Future concurrency APIs must reject moving or sharing such handles unless the
opaque type has the relevant capability.

When a handle operation participates in cancellation:

- the function signature should still report recoverable operation failures
  through `Result`;
- cancellation must not make cleanup disappear;
- cleanup should run in a documented cancellation boundary;
- APIs must say whether they block the OS thread, cooperate with the Muga
  scheduler, or require a future async runtime.

`Task[T]`, channels, timers, sockets, listeners, streams, and child processes
should be modeled as opaque resource handles only after those send/share and
cancellation facts are explicit.

## Runtime Diagnostics

Runtime diagnostics for invalid handle use should be hard errors when source
typechecking cannot prove safety. Initial diagnostic situations:

- use after close or consumption reaches runtime;
- handle identity belongs to another package/runtime family;
- operation receives a stale runtime slot;
- operation receives a handle from an incompatible task or scheduler context;
- host cleanup reports an unrecoverable runtime bug rather than a normal
  recoverable error.

Recoverable host failures belong in the public error type, not in hard runtime
diagnostics.

## Capability And Close Metadata Plan

The next implementation boundary should add metadata, not handle values. It
should make future compiler-provided handles explicit in `.mgi` and typed HIR
without adding user-facing resource syntax or broad standard-library APIs.

### Opaque Handle Facts

Each runtime-backed opaque type should carry an `OpaqueHandleFacts` record in
the package interface. Source-defined opaque types get the default facts unless
a future source syntax explicitly says otherwise. Compiler-provided packages can
set non-default facts in their virtual package definitions.

Default facts for a runtime-backed handle:

- `runtimeBacked = true` only for compiler/runtime-owned handles;
- `copyable = false`;
- `cloneable = false`;
- `sendable = false`;
- `shareable = false`;
- `structurallyComparable = false`;
- `serializable = false`;
- `closeable = false` until a close function is named;
- no implicit destructor or observable finalizer.

The interface format should encode these facts on `opaque-type` entries. Omitted
facts in legacy or ordinary source interfaces mean the conservative defaults.
Unknown fact names should fail closed when loading a newer `.mgi` unless the
loader has an explicit compatibility rule.

### Consuming Parameters

Function signatures need parameter ownership metadata before source code can
use handles safely. The first metadata shape should be per-parameter:

- `borrow`, the default, does not invalidate the argument binding;
- `consume` invalidates the argument binding after the call is accepted;
- future modes such as `share` or `send` must not be added until task-boundary
  rules require them.

The first implementation should store this as `.mgi` function parameter
metadata, for example a `paramMode` field in the persisted `param` line or an
equivalent versioned field. User-defined source functions have `borrow`
parameters only. Compiler-provided packages may mark parameters as `consume`
before user-facing consuming-parameter syntax exists.

The typechecker should reject obvious use-after-consume in a single function
body. This is a resource-specific dataflow check, not a general borrowing
system. It should track local bindings that have been passed to a consuming
parameter and reject later reads, second closes, field access, calls, equality,
formatting, or returns that use the consumed binding.

### Close Function Metadata

Explicit close is the first cleanup contract. A close function is an ordinary
public function plus metadata:

- it takes exactly one consuming handle parameter;
- it returns `Result[Unit, E]`;
- success and recoverable failure both consume the source binding;
- runtime failures such as stale slots or wrong handle families remain hard
  diagnostics rather than `Result::Err` values.

The opaque type metadata may name one close function once stable package item
identity is available. If close metadata is absent, tooling must not infer a
close function by name.

Statement-form `using` consumes this close metadata for the implemented
lexical cleanup slice. Expression-form `using`, multiple bindings, and
aggregate cleanup errors remain deferred.

### Interface And API Diff Rules

Capability metadata is part of the public contract for runtime-backed opaque
handles:

- adding a new public opaque handle type is source-compatible metadata;
- removing a capability is breaking;
- adding `copyable`, `cloneable`, `sendable`, or `shareable` is
  source-compatible but reviewable;
- changing `closeable` or the named close function is breaking unless both old
  and new close functions have identical public identity and signature;
- changing a parameter from `borrow` to `consume` is breaking;
- changing a parameter from `consume` to `borrow` is source-compatible but
  reviewable because it changes lifecycle behavior.

Opaque capability facts should participate in the public interface hash. Pure
documentation and diagnostic spans should continue to be ignored by the hash.

### First Candidate API

The first runtime-backed handle candidate should be a deliberately small
compiler-provided text-file handle in `std::fs`, because the existing
`std::path`, `std::fs`, and `std::io` slices already define path values,
recoverable IO errors, and artifact-backed std package execution.

Candidate shape for the later implementation slice:

```muga
package std::fs

pub opaque type File

pub fn open_text(path: path::Path): Result[File, io::IOError]
pub fn read_text_from(file: File): Result[String, io::IOError]
pub fn close(file: File): Result[Unit, io::IOError]
```

Only `close` should consume `file` in the first handle slice. `open_text`
constructs a runtime-backed `File`; ordinary source code still cannot construct
or inspect one. The first runtime implementation should stay read-only:
`write_text_to` remains deferred because it requires explicit open modes,
cursor/seek semantics, overwrite/truncate/append decisions, and more mutation
tests. This keeps binary `Bytes`, buffering, write modes,
append/create/truncate modes, stdin/stdout/stderr handles, async IO, file
locking, permissions, and streaming APIs out of the first runtime-handle slice.

### First Runtime File Handle Implementation Design

The first runtime-backed handle slice should prove the end-to-end handle model
without broadening `std::fs` into a streaming or mutation API.

Runtime representation:

- add a VM-local handle table owned by one run, not by `.mgi`, `.mgc`, or `.mgb`
  artifacts;
- represent a handle value as `{ family, slot, generation }`, where `family =
  "std::fs::File"` for this slice;
- store each file slot as `Open { path, file }` or `Closed { generation }`;
- increment or otherwise invalidate the generation when a slot is closed, so a
  stale copied runtime value cannot accidentally refer to a new file;
- never serialize, compare, format, or clone handle values.

Compiler-provided interface:

- `std::fs::File` is a compiler-provided `pub opaque type` with
  `runtimeBacked=true`, `copyable=false`, `cloneable=false`, `sendable=false`,
  `shareable=false`, `structurallyComparable=false`, `serializable=false`,
  `closeable=true`, and `closeFunction=std::fs::close`;
- `open_text(path: path::Path): Result[File, io::IOError]` constructs a read-only
  text handle or returns a recoverable `io::IOError`;
- `read_text_from(file: File): Result[String, io::IOError]` borrows the handle
  and reads UTF-8 text from the current file contents;
- `close(file: File): Result[Unit, io::IOError]` consumes the binding and closes
  the slot.

Diagnostics and cleanup:

- use-after-consume remains a typechecker diagnostic (`T026`);
- stale slot, wrong family, double close that reaches runtime, or use after a
  successful close is a hard runtime diagnostic, not `Result::Err`;
- host IO failures from `open_text`, `read_text_from`, and `close` remain
  recoverable `io::IOError` values;
- VM shutdown may best-effort close leaked handles, but there is no observable
  destructor/finalizer and no user-visible success path should depend on that
  cleanup.

Artifact behavior:

- package interfaces persist only opaque handle facts and parameter modes;
- implementation artifacts may name the compiler-provided builtins but must not
  persist live handle slots or host file descriptors;
- `run --built` and artifact-backed dependency execution allocate a fresh handle
  table per VM run.

Initial tests should cover source and artifact-backed `open_text` +
`read_text_from` + `close`, recoverable missing-file errors, `T026` after
`close`, hard runtime diagnostics for stale/double-close only if such a value can
be produced internally, and metadata/hover/doc exposure for `std::fs::File`.

Implementation status: the first read-only `std::fs::File` slice is implemented.
The VM stores `File` as an opaque runtime handle backed by a per-run handle
table, `open_text` / `read_text_from` / `close` run from both source and `.mgb`
artifacts, host failures remain recoverable `io::IOError` values, and stale or
closed aliases that reach runtime report hard `R022` diagnostics. The scalar
program stderr channel selected in
[post-file-handle-resource-surface-selection.md](post-file-handle-resource-surface-selection.md)
is implemented through `eprint` / `eprintln` and is explicitly not a
standard-stream handle model. Text output file handles are implemented from
[text-output-file-handles.md](text-output-file-handles.md): the same public
`std::fs::File` type now supports runtime read/write/append modes, explicit
`flush`, and recoverable wrong-mode `io::IOError` values. Binary `Bytes`,
buffering, stdin/stdout/stderr handles, async IO, and streaming APIs remain
deferred.

### Implementation Order

1. Add interface data structures for opaque handle facts and function parameter
   modes, preserving conservative defaults for source-defined opaque types.
2. Persist and reload those facts in `.mgi`, with stable public hashes and
   legacy defaults.
3. Expose the metadata through `muga metadata`, `muga hover`, `muga doc`, and
   future API diff inputs without changing source syntax.
4. Add typechecker dataflow for consuming parameters, initially exercised only
   by compiler-provided test fixtures or synthetic interface summaries.
5. Add the first compiler-provided `std::fs::File` runtime handle only after
   steps 1-4 are covered.

Steps 1-5 are implemented through the metadata, diagnostic, and read-only
runtime slices: `.mgi` v5
persists `OpaqueHandleFacts` on `opaque-type` entries and `paramMode` on
`param` entries, legacy interfaces default to conservative facts and `borrow`,
public interface hashes include the metadata, and `muga metadata`, hover,
completions, and docs expose it without source syntax. The typechecker also
rejects direct same-scope use-after-consume for bindings passed to loaded
interface `consume` parameters with `T026`. The first runtime-backed
`std::fs::File` value now runs through VM-local slots for `open_text`,
`read_text_from`, and consuming `close`.

## Non-Goals

This design does not add:

- runtime-backed values beyond the current text `std::fs::File` slice;
- `pub opaque record`;
- general references, borrowing, lifetimes, destructors, finalizers, or RAII;
- stdout/stderr/stdin handles or file-handle families beyond `std::fs::File`;
- sockets, listeners, streams, timers, task handles, or child process handles;
- `Bytes`, buffers, encoders, or streaming APIs;
- HTTP/SSE/WebSocket/RPC;
- schema/client generation;
- new `std::json` behavior.

## First Implementation Slice

The first implementation slice is implemented and does not add a broad runtime
API. It only adds enough compiler and interface support to make opaque type
names sound:

1. parse and resolve `pub opaque type Name` in package mode;
2. represent public opaque type names in typed HIR and `.mgi`;
3. allow opaque type names in public signatures and downstream loaded
   interfaces;
4. reject construction, field access, pattern matching, structural equality,
   and formatting for opaque values unless a specific API permits it;
5. update API diff, metadata, doc, hover, completions, definition, references,
   and release-readiness evidence;
6. keep runtime-backed handle values and effectful stdlib packages deferred
   until consuming-operation metadata and close behavior are covered.

### Interface Slice Plan

The implemented first slice follows this order:

1. add parser and AST support for package-mode `pub opaque type Name`;
2. add package item identity and visibility support for public opaque type
   declarations, without admitting `pkg` or private opaque declarations yet;
3. add a nominal `TypeInfo` form for package opaque types so public function,
   record, enum, collection, `Option`, `Result`, and function-type signatures
   can refer to them;
4. persist opaque type entries in `.mgi` with a stable public interface hash and
   validate loaded opaque type identities without reading dependency source
   bodies;
5. expose opaque type items through `muga doc`, `metadata`, `hover`,
   `completions`, `definition`, and `references`;
6. add focused rejecting coverage for unsupported uses: construction, field
   access, match patterns, structural equality, and formatting;
7. update release-readiness evidence and run the release gate.

The slice is complete: a downstream package can check against an
artifact-backed public signature containing an imported opaque type name, and
ordinary source code still has no way to create a runtime-backed handle value.

Only after that interface slice is stable should Muga add a tiny compiler-known
resource family such as a file handle or task handle.
