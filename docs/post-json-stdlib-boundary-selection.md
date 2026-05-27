# Post-JSON Standard-Library Boundary Selection

Status: completed boundary-selection note after the first `std::json`
implementation audit. This note chooses the next narrow standard-library/API
boundary to design before any broader runtime-backed package surface is added.

## Selection

The next boundary is an opaque resource-handle design, not a new effectful
runtime API. Before adding stdout/stderr handles, file handles, process APIs,
HTTP/SSE/WebSocket/RPC, streaming APIs, `Bytes`/buffers, or schema/client
generation, Muga needs a public contract for runtime-backed values that source
code can name but cannot forge.

This is not a new effectful runtime API.

The selected next slice should produce a design note for resource handles. It
should not implement `std::http`, process execution, streaming IO, `Bytes`,
schema generation, or new `std::json` behavior.

That design is now recorded in
[opaque-resource-handles.md](opaque-resource-handles.md). It keeps the first
implementation-facing step limited to a `pub opaque type` interface slice
before any runtime-backed handle value or broader effectful API is added.

## Candidates Reviewed

| Candidate | Decision | Reason |
|---|---|---|
| Expand `std::json` into schema/client generation or HTTP/RPC integration | Defer | The first JSON slice is intentionally limited to parse/encode, `Number`, and explicit `json::Error`; broader schema and service integration need separate contracts. |
| Add `std::http`, SSE, WebSocket, RPC, or process APIs | Defer | These need ownership, cancellation, close/error behavior, and package-interface representation for runtime-backed handles. |
| Add stdout/stderr, stdin, file, socket, timer, stream, or child-process handles | Defer until the selected design note exists | Transparent records would let users forge values that should be runtime-owned. |
| Add `Bytes`, buffers, encoders, or streaming APIs | Defer | Byte/string ownership, allocation behavior, and handle interaction are not documented yet. |
| Add `List.contains`, structural collection assertions, `Map.entries`, map literals, `Set[T]`, or iterator protocols | Defer | These either require expanding the scalar-only equality policy, choosing an entry-record shape, or adding broader collection/protocol semantics. |
| Add formatting templates, interpolation, range syntax, `T?`, `?.`, `expr?`, named arguments, or broader `using` forms | Defer | These are syntax/language decisions, not the next standard-library boundary. |

## Resource-Handle Design Scope

The next design slice should answer:

- how a public opaque type is represented in `.mgi` without exposing a
  constructible record shape;
- whether handles are move-only, cloneable, shareable, or ordinary values in
  the current value-semantics model;
- how explicit `close`/cleanup functions report recoverable failures through
  `Result`;
- whether automatic cleanup exists, and if so which errors are observable;
- how cancellation, task boundaries, and future concurrency interact with
  handles;
- how stale or invalid handle values are reported in runtime diagnostics;
- which first API would be allowed only after the design is accepted.

## Non-Goals

This selection does not add:

- `pub opaque type` syntax;
- runtime handle values;
- filesystem or network handle APIs;
- process execution;
- HTTP/SSE/WebSocket/RPC packages;
- streaming JSON, schema generation, or client generation;
- `Bytes`, buffers, or encoders;
- iterator protocols, map literals, `Set[T]`, or structural equality.

## Completion Criteria For The Next Slice

The next slice is complete when an opaque resource-handle boundary design:

- names the public contract and deferred APIs;
- explains `.mgi` representation and API-diff implications;
- states ownership, close, clone/share, cancellation, and cleanup behavior;
- keeps recoverable effects as `Result`;
- updates the standard-library review rules, practical readiness notes, ROADMAP,
  and implementation resume plan;
- adds release-readiness checks for the selected boundary.
