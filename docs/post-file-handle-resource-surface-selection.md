# Post-File-Handle Resource Surface Selection

Status: completed selection after the first read-only `std::fs::File`
runtime-handle implementation. The selected program stderr channel is now
implemented through scalar `eprint` / `eprintln`, text write handles and
statement-form `using` have since landed, and this note records the earlier
choice plus the deferred decisions needed before broader IO, process, network,
streaming, or binary APIs are added.

## Purpose

The read-only `std::fs::File` slice proved that Muga can expose a public opaque
runtime-backed handle through `.mgi`, load it from artifacts, consume it through
`close`, and reject stale or already-closed runtime handles. It did not make
the filesystem API broad enough for general streaming, binary IO, process
execution, or services.

The next step should therefore be selected by contract risk, not by adding the
largest missing standard-library package.

## Audit Of The Implemented File Handle

What is now proven:

- `.mgi` can persist `OpaqueHandleFacts`, a close function identity, and
  per-parameter `paramMode` for compiler-provided packages.
- Artifact-backed `run` can execute code that opens runtime handles without
  serializing live handles into `.mgi`, `.mgc`, or `.mgb`.
- The VM-local handle table prevents slot reuse from silently reviving stale
  handle values.
- Direct same-scope use after passing a binding to a consuming parameter is
  rejected as `T026`.
- Runtime-only invalid handle states, including aliases after close, are hard
  `R022` diagnostics instead of recoverable `Result::Err` values.

What remains deliberately unproven:

- Aliasing is not statically tracked. A copied handle value can still exist and
  will be caught only by the runtime slot/generation checks after close.
- At the time of this selection there was no lexical cleanup construct such as
  `using`; that gap is now closed by the first statement-form `using` slice.
- `read_text_from` is a full-file text operation that seeks to the start before
  reading. It does not define cursor, seek, chunking, streaming, or buffering
  semantics.
- `close` has not yet had to report write-buffer or flush failures, because
  the first handle is read-only.
- At the time of this selection there was no `Bytes` value type, no binary file
  surface, no stdin/stdout/stderr handle model, and no task/cancellation model.
  A later slice added minimal opaque resource `Bytes`, but the binary file,
  stream, task, and cancellation boundaries remain unproven here.

These limits are acceptable for the proof slice, but they are exactly why the
next surface should stay narrow.

## Selection Criteria

Choose the next surface by these rules:

1. It should improve practical CLI or small application usefulness immediately.
2. It must not require hidden exceptions, hidden async suspension, global
   dynamic reflection, or source-level references.
3. Public effects remain visible as functions, `Result`, or explicit runtime
   output channels.
4. If it adds public package items, they must be representable in `.mgi` and
   stable enough for future API diffing.
5. It should have focused source, package, artifact-backed, CLI JSON, and
   documentation coverage.
6. It must not force decisions about `Bytes`, streaming, cancellation,
   task-boundary movement, or native backend representation unless those are
   the selected surface itself.

## Candidates Compared

| Candidate | Practical value | Contract risk | Decision |
|---|---:|---:|---|
| Program stderr channel through scalar `eprint` / `eprintln` | High for CLI tools and agent/editor JSON workflows | Low: no handle lifetime, no `.mgi` package API, mirrors existing `print` / `println` | Implemented |
| Write-mode text file handles | Medium-high for reports and logs, but one-shot `fs::write_text_path` already exists | Medium-high: open modes, truncation, append, flush, close errors, cursor policy, wrong-mode behavior | Defer one slice; design after stderr |
| `Bytes` value type | High for HTTP, hashing, binary files, and encodings | High: literals, encoding, indexing, equality, display, JSON/HTTP mapping, allocation policy | Defer until binary/API contract design |
| Buffers / `StringBuilder` | Medium for efficient construction | Medium: mutation/builder semantics and relation to `Bytes` / formatting | Defer until repeated-construction pressure is concrete |
| Lexical cleanup `using` | High once more closeable resources exist | High: syntax, error-combination rules, cancellation interaction, double-close behavior | Deferred here; implemented after text file write evidence |
| stdout/stderr/stdin handles | Medium-high, but mostly for future process/service adapters | High: standard streams are ambient, often uncloseable, and task sharing is unclear | Defer; implement stderr as a channel, not a handle |
| Directory streams / generic streams | Medium for large data | High: iterator protocol, cleanup, backpressure, and possible async interaction | Defer |
| Process APIs | High for scripting and build tools | Very high: child handles, pipes, exit status, cancellation, security | Defer until stderr, `Bytes`, and resource cleanup are settled |
| HTTP / SSE / WebSocket / RPC | High for adoption and services | Very high: `Bytes`, sockets, TLS, cancellation, backpressure, schemas | Long-term platform work |
| `Duration` / `Instant` values | Medium for timeouts and benchmarks | Low-medium, but not a resource-handle continuation | Consider separately after the selected IO slice |

## Selected Surface

Implement a program stderr channel, not stdout/stderr handles. This surface is
implemented.

Initial public source shape:

```muga
eprint(value)
eprintln(value)
```

Rules:

- keep them as prelude builtins, mirroring existing `print` and `println`;
- accept only `Int`, `Bool`, and `String`, preserving the scalar formatting
  policy;
- return the original argument, matching `print` / `println` chain behavior;
- capture program stderr separately from program stdout in the runtime outcome;
- populate the existing `muga run --format json` `stderr` field instead of
  leaving it permanently empty;
- text-mode `muga run` writes program stdout to process stdout and program
  stderr to process stderr, while diagnostics remain compiler/runtime
  diagnostics;
- extend `muga test` reporting so per-test stderr is not lost when tests use
  `eprint` / `eprintln`;
- do not expose `std::io::stderr`, `std::io::stdout`, stdin handles, closeable
  standard stream handles, or stream redirection APIs in this slice.

Why this comes first:

- It makes Muga more useful for real CLI programs without introducing another
  resource lifetime model.
- It validates the command-output contract that already reserves program
  stderr in JSON.
- It leaves file write modes free to choose a better open-mode and close/flush
  contract after one more runtime-output slice.
- It keeps standard-stream handles deferred until task sharing, close behavior,
  and redirection semantics are documented.

## Follow-Up File Write Handle Direction

After stderr, the next resource-handle design should be text output file
handles. That design lives in
[text-output-file-handles.md](text-output-file-handles.md) and is now
implemented.

Recommended starting point:

```muga
pub fn create_text(file_path: path::Path): Result[File, io::IOError]
pub fn append_text(file_path: path::Path): Result[File, io::IOError]
pub fn write_text_to(file: File, text: String): Result[Unit, io::IOError]
pub fn flush(file: File): Result[Unit, io::IOError]
```

The design slice must decide before implementation:

- whether one `File` opaque type carries runtime read/write/append mode, or
  whether separate reader/writer opaque types are worth the extra close-function
  naming and ergonomics cost;
- whether wrong-mode operations return recoverable `io::IOError` values or
  hard runtime diagnostics;
- whether `close` attempts observable flush behavior for writable handles, and
  how close failures interact with consumed bindings;
- whether `read_text_from` keeps its seek-to-start behavior once write handles
  introduce cursor state;
- how artifact-backed tests cover fresh handle tables, stale aliases, write
  failures, and close/flush behavior.

The implemented follow-up design keeps one public `std::fs::File` type and stores
the access mode in the runtime slot. That keeps the public API small and
preserves the existing `fs::close(file)` function. Wrong-mode operations are
recoverable `io::IOError` values because the handle is valid but the requested
host operation is unsupported for its open mode.

## Short-Term Goal

Make Muga credible for small CLI and data-transformation programs while keeping
the v1 contract narrow:

- keep package artifacts, diagnostics, docs, and release gates green;
- keep the implemented program stderr channel covered by runtime, CLI JSON, and
  docs;
- document and then implement text output file handles;
- keep the focused `report_app` workflow example that combines args/env,
  stdout/stderr, text-file handle writes, JSON, `Result`, tests, local
  dependencies, and `run --built` covered;
- avoid `Bytes`, process, HTTP, async IO, and broad filesystem mutation until
  each has its own contract.

## Medium-Term Goal

Turn Muga from a compiler/runtime prototype into a useful package ecosystem
foundation:

- keep statement-form `using` narrow after read/write file handles proved
  close/error behavior;
- add `Bytes` with explicit encoding and equality/display policy before binary
  files, hashing, HTTP, or process pipes;
- add `Duration` / `Instant`, URL/URI, TOML/CSV, hashing, config, logging, and
  CLI argument parsing as small `std` package slices;
- improve project/workspace workflows only when entry-reachable JSON tooling
  is not enough;
- keep `.mgi` interfaces as the single source of truth for docs, editor tools,
  API diffing, and later schema generation.

## Long-Term Goal

Make Muga practical and adoptable for services and distributed applications:

- structured concurrency with task groups, `spawn`, `join`, cancellation, and
  typed channels;
- cancellation-aware async IO integrated with resource handles;
- HTTP client/server, SSE, WebSocket, RPC, TLS boundaries, and backpressure;
- schema and client generation from `.mgi` public contracts;
- registry, signing, provenance, vulnerability/audit, and package-health
  infrastructure;
- control-flow MIR, native backend, profiler/debugger support, and
  self-contained binary distribution;
- editor integration and learning material that continue to consume structured
  compiler outputs instead of private implementation bodies.

The long-term adoption bet is not "more syntax first". It is reliable package
contracts, explicit effects, strong tooling, and a standard library that grows
in small reviewable slices.
