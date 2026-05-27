Status: implemented first slice

# Lexical Resource Cleanup

Muga now has runtime-backed `std::fs::File` handles with explicit
`fs::close(file)`. The integrated `report_app` sample proves the feature is
useful, but it also shows why manual close sequencing is not enough: a `try`
after a successful open can leave later cleanup code unreachable.

This document records the deliberately narrow lexical cleanup construct added before
adding
`Bytes`, stdout/stderr handles, process APIs, network APIs, streams, async IO,
or broader runtime-backed resources.

## Selected Syntax

First slice:

```muga
using file = try fs::create_text(output) {
  wrote = try fs::write_text_to(file, rendered)
  flushed = try fs::flush(file)
}
```

`using` is a statement, not an expression. It introduces one immutable binding
whose scope is the block. The binding must be a runtime-backed opaque handle
whose package interface carries a close function in `OpaqueHandleFacts`.

`with` remains reserved for record update and is not a resource-lifetime
keyword.

## Why This Shape

| Candidate | Pros | Cons | Decision |
|---|---|---|---|
| `using name = expr { ... }` statement | Clear lifetime boundary, easy to teach, avoids expression-type questions in first slice | New syntax and lowering | Select |
| `using name = expr { ... }` expression | Useful for returning block values | Forces immediate block-result and cleanup-error typing decisions | Defer |
| `defer fs::close(file)` | Familiar in some languages and flexible | Easy to hide lifetimes, hard to constrain to resource handles, can pile up arbitrary effects | Defer |
| `fs::with_create_text(path, fn(file) { ... })` | No new syntax | `try` inside closures returns from the closure, not the caller; less readable for ordinary users | Defer |
| General destructors/finalizers | Requires little syntax at call sites | Hidden effects, timing ambiguity, poor fit for explicit `Result` effects | Reject |

## Type Rules

- The initializer must type-check before the block. `try` in the initializer
  keeps its current behavior and returns early from the enclosing function if
  acquisition fails.
- After `try`, the bound value must be a runtime-backed opaque handle.
- The handle's interface metadata must name a close function whose first
  parameter mode is `consume`.
- The close function must return `Result[Unit, E]`.
- The enclosing function must return `Result[T, E]` with the same cleanup error
  type `E`. This keeps the first slice compatible with existing `try`
  propagation and avoids adding aggregate error types.
- The first slice supports one binding per `using`. Multiple resources should
  use nested `using` blocks so cleanup order is explicit.
- Nested cleanup unwinds in last-acquired, first-closed order. If more than one
  cleanup returns `Result::Err`, Muga returns the first cleanup error observed
  in that order after still attempting the remaining active cleanups. Later
  cleanup errors are ignored until an aggregate cleanup error type is designed.

## Cleanup Semantics

The cleanup function runs exactly once if the handle binding was successfully
created.

When the block completes normally:

1. run the close function;
2. if close returns `Result::Ok(())`, execution continues after the block;
3. if close returns `Result::Err(error)`, the enclosing function returns
   `Result::Err(error)`.

When the block exits by `try`, `return`, `break`, or `continue`:

1. run the close function before completing the control transfer;
2. if close returns `Result::Ok(())`, preserve the original control transfer;
3. if close returns `Result::Err(error)`, return `Result::Err(error)` from the
   enclosing function.

This means cleanup failure wins when both the body and cleanup fail. That keeps
cleanup errors visible with the existing one-error `Result[T, E]` model. A
future aggregate cleanup error can revisit this after a real use case justifies
the extra type surface.

When a cleanup itself fails inside nested `using`, active outer cleanups are
still attempted before the selected cleanup error is returned. This preserves
the "created handles are cleaned exactly once" rule even though the first
cleanup error remains the only error value that can be represented.

Hard runtime diagnostics, process termination, and future cancellation are not
part of the first slice. Cancellation-aware cleanup must be designed with async
IO later.

## Dataflow Rules

- The `using` binding is available only inside the block.
- Passing the binding to its close function inside the block is rejected; the
  binding is already managed by the cleanup statement.
- Passing the binding to any loaded-interface parameter with `paramMode=consume`
  inside the block is rejected unless it is the compiler-inserted cleanup call.
- Borrowing operations such as `fs::write_text_to(file, text)` and
  `fs::flush(file)` remain allowed.
- Aliases created inside the block must not be usable after cleanup. The first
  implementation should reject obvious same-scope alias escape patterns and
  keep broader escape analysis out of scope.

## Lowering And Runtime

The compiler should lower `using` as structured control flow, not as a library
call. The lowered form must make cleanup run for normal fallthrough and for
source-level `try`, `return`, `break`, and `continue` exits from the block.

The runtime handle table and stale-handle diagnostics stay unchanged:

- the compiler-inserted close consumes the handle;
- stale aliases after cleanup still fail with the existing runtime handle
  diagnostics if the checker did not reject them statically;
- `fs::close` keeps its current public behavior outside `using`.

## Formatter And Interfaces

- `muga fmt` should format `using` like `if` and `for`: keyword, binding,
  initializer, space, block.
- `.mgi` does not need a new public item kind for `using`; it consumes existing
  opaque handle metadata.
- Public interface hashes must stay unchanged for implementation-only edits
  that add or remove private `using` statements.

## First Test Slice

Add focused coverage for:

- parser and formatter support for `using`;
- type rejection for non-handle bindings;
- type rejection when the enclosing function cannot return the close error;
- source execution for file write/flush cleanup without explicit close;
- close failure after normal body returning `Result::Err`;
- body `try` failure still triggering close;
- nested `using` acquisition failure still triggering outer close;
- nested cleanup failure branches still attempting outer cleanup before return;
- explicit `fs::close(file)` inside `using` rejected;
- use-after-using rejected for obvious same-scope bindings;
- artifact-backed execution and `run --built`;
- `report_app` using the construct after the focused tests pass.

## Deferred

- `using` expressions with block values;
- multiple bindings in one `using`;
- general destructors, finalizers, or arbitrary `defer`;
- aggregate primary/cleanup error records;
- cleanup across hard runtime diagnostics or future task cancellation;
- standard-stream handles, process handles, sockets, HTTP clients, streams, and
  async resources;
- `Bytes`, buffers, builders, and binary IO.
