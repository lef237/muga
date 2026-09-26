# Concurrency

Status: Phase 1 (structured task groups: `group`, `spawn`, `join`) is
implemented in the Rust compiler and reference VM. Section 5 is the
specification for the implemented behavior. The remaining sections record
design constraints for future runtime and IO work; they are not implementation
queues. Further concurrency surface is parked in [ROADMAP.md](../ROADMAP.md).

## 1. Design Goals

Muga's concurrency model should aim for all of the following:

- lightweight task creation
- simple and readable syntax
- strong defaults for safety
- explicit structure for task lifetime
- high runtime performance
- a compiler-friendly design that does not force expensive global analysis

## 2. Core Direction

The recommended direction is:

- lightweight tasks
- structured concurrency
- immutable-by-default sharing
- explicit joins and cancellation
- no async function coloring as the primary user model

The base model is:

- `group { ... }` creates a task scope
- `spawn expr` starts lightweight concurrent work inside that scope
- `task.join()` waits for a task and returns its result

## 3. Scope

Phase 1 defines and implements `group`, `spawn`, `join()`, structured failure
propagation, structured cancellation, and task-boundary capture rules; section
5 specifies the implemented behavior. Channels, `select`, timeouts, detached
tasks, supervision, and actor-style features are not planned until the
stability gate (5.9) is resolved and real programs show that a standard-package
abstraction such as `task::spawn_map` is not enough.

## 4. Why This Fits Muga

This phased direction fits Muga's existing language shape:

- bindings are immutable by default
- explicit structure is preferred over hidden behavior
- local reasoning is preferred over global magic
- the language already favors simple surface forms over heavy abstraction systems

Structured task scopes also fit the package and compiler roadmap well:

- they are easier to typecheck than detached background execution
- they are easier to lower into typed HIR and MIR
- they make runtime leaks and forgotten tasks easier to prevent

## 5. Phase 1: Structured Task Core (Implemented)

Phase 1 is implemented. This section is the specification for the implemented
behavior. It chose syntax over a standard-package abstraction because the core
lifetime rule — child tasks may not outlive their `group` — is enforced by
lexical structure; a package-level scope value could escape its scope and
would need escape analysis to stay honest.

### 5.1 Task scopes

The primary concurrency construct is a lexical task scope expression:

```muga
import std::task

result = group {
  user_task = spawn fetch_user(id)
  orders_task = spawn fetch_orders(id)

  Page {
    user: user_task.task::join()
    orders: orders_task.task::join()
  }
}
```

- `group { ... }` is an expression. Its body is a value block: statements
  followed by a final expression, and the `group` evaluates to that final
  expression.
- The scope defines the lifetime boundary for child tasks created inside it:
  leaving the `group` means every child task spawned in it has completed.
- `group` expressions nest; each `group` is its own task scope.
- If one child task fails, remaining child tasks are cancelled and the
  failure propagates out of the group (see 5.5 for the exact Phase 1 form).

### 5.2 Spawning tasks

```muga
task = spawn expr
```

- `group` and `spawn` are keywords and cannot be used as binding names.
- `spawn expr` is a prefix expression form parsed at the same level as prefix
  `try`. `spawn f(x)` and `spawn user.users::birthday().age` work directly,
  and `spawn group { ... }` spawns a nested task scope directly; wrap other
  single-expression forms in parentheses, as in
  `spawn (if flag { a() } else { b() })`.
- `spawn` is allowed only inside the body of an enclosing `group` expression
  in the same function. `spawn` outside a `group` is rejected with `T030`.
- Function boundaries reset the group context: a helper function or `fn`
  expression body cannot use `spawn` unless it opens its own `group`, even
  when it is called from inside one.
- `spawn` does not reset the group context for its own operand: a nested
  `spawn` inside another `spawn` operand belongs to the same enclosing
  `group`.
- The result of `spawn` is a task handle carrying the operand's type.

### 5.3 Joining tasks and task handle typing

```muga
import std::task

value = task::join(handle)
value = handle.task::join()
```

- `join` is an ordinary public generic function in the `std::task` standard
  package, `pub fn join[T](task: Task[T]): T`. The qualified chained form
  `handle.task::join()` is the usual dot-call surface for
  `task::join(handle)`; there is no separate method semantics and no prelude
  name. Keeping `join` in a package preserves the flat prelude: Muga rejects
  shadowing, so a new prelude name would break every program that already
  uses that name, including `path::join`.
- `spawn expr` has type `Task[T]` when `expr` has type `T`, and
  `task::join` returns `T`.
- `Task[T]` is an internal compiler type. User source cannot write it in
  type annotations, record fields, or function signatures (`T013` unknown
  generic type); only the compiler-provided `std::task` package spells it,
  in the signature of `join`. Because public package functions require
  explicit signatures, task handles cannot cross user package boundaries.
- Task handles are ordinary immutable values otherwise: they can be bound,
  joined more than once (each `join` returns the completed value), or left
  unjoined — the `group` still waits for the task.

### 5.4 Sharing and capture rules

The task boundary is the `spawn` operand.

- Reading enclosing immutable bindings (parameters, `for` items, `using`
  bindings, `match` payload bindings, and ordinary immutable bindings) inside
  a `spawn` operand is allowed.
- Referencing an enclosing `mut` binding inside a `spawn` operand is rejected
  with `E013`, for reads as well as writes. Bind an immutable copy first or
  pass the value in through a function argument. (Assignment from nested
  functions is already `E004`; `E013` additionally closes plain reads across
  the task boundary so a future parallel runtime cannot observe intermediate
  states.)
- Function values may be captured and called. A closure created outside the
  `spawn` operand may internally read outer `mut` bindings; this indirect
  read is allowed in Phase 1 because the reference execution is
  deterministic. Closure capture must be revisited before any parallel
  runtime lands.
- Runtime-backed handles such as `fs::File` may be captured and used inside
  `spawn` operands in Phase 1; deterministic reference execution makes this
  safe. Handle send/share rules must be revisited before parallel execution,
  as section 10 already requires.

### 5.5 Execution model, failure, and cancellation

Phase 1 fixes the observable structure, not a scheduler:

- Task execution order is implementation-defined within the structure that
  `group`, `spawn`, and `join` allow. Programs must not rely on sibling
  tasks interleaving.
- The reference VM executes deterministically: `spawn` runs the child task to
  completion at the spawn site, and `join` returns the completed value.
  Leaving a `group` therefore trivially satisfies "wait for all children".
- If a child task fails with a runtime error, the failure propagates out of
  the enclosing `group` as a runtime failure at the spawn site. Sibling
  tasks that were not spawned yet never start. This is the Phase 1 form of
  "one failure cancels the remaining siblings"; a parallel runtime must
  preserve the same observable guarantee with real cancellation.
- Recoverable errors stay explicit values: a task whose operand evaluates to
  `Result[T, E]` produces a `Task[Result[T, E]]`, and the caller handles the
  `Result` after `join` as usual; `try handle.task::join()` composes normally
  inside `Result`-returning functions.

### 5.6 What Phase 1 does not include

- no channels, `select`, timeouts, deadlines, or detached tasks (see
  section 3)
- no source-level `Task[T]` type syntax and no user-nameable task type
- no timeout API: Phase 1 does not promise async IO behavior, so time-based
  cancellation waits for the IO/runtime integration path in section 10
- no async function coloring, in line with section 6

### 5.7 Phase 1 usage notes

Realistic programs (see `samples/packages/app/std_task_result/main.muga`,
`samples/packages/app/std_task_list/main.muga`,
`samples/packages/app/std_task_for/main.muga`, and
`samples/projects/task_app/src/main/main.muga`) showed:

- A literal list of spawned tasks joined through `list::map` reads naturally,
  because the mapped closure only calls `join`, never `spawn`.
- `try handle.task::join()` composes inside `Result`-returning functions, so
  fallible fan-out reads like ordinary sequential `Result` code.
- Fire-and-forget `spawn` inside a `for` loop works, because a loop is not a
  function boundary.
- Result-collecting fan-out over a runtime-sized collection cannot use `spawn`
  directly: `spawn` inside a mapped closure is rejected with `T030` (5.2), and
  `T013` forbids naming `List[Task[T]]` (5.3). `task::spawn_map` (5.8) covers
  this case as a library function instead of new syntax.

### 5.8 `spawn_map`: fan-out over a runtime-sized collection

```muga
import std::task

pub fn spawn_map[T, U](items: List[T], f: T -> U): List[U]
```

`task::spawn_map(items, f)` spawns `f` on every item of `items` and returns
their results as a `List[U]`, in input order. See
`samples/packages/app/std_task_spawn_map/main.muga`.

- `spawn_map` is an ordinary `std::task` package function, defined in terms
  of `group`, `spawn`, `join`, and `push` (5.1-5.3); it does not add new
  syntax or a new diagnostic.
- Its public signature never names `Task[T]`: callers pass and receive plain
  `List` values, so the `T013` restriction on writing `Task[T]` (5.3) never
  applies to a caller of `spawn_map`.
- `spawn_map` opens its own `group` internally and joins every spawned task
  before returning, so it may be called from any function, not only from
  inside an enclosing `group`. It behaves like a self-contained task scope
  whose result is the collected list.
- If `f` returns `Result[T, E]`, `spawn_map` returns `List[Result[T, E]]`;
  callers reduce it with ordinary `list` functions (`list::all`, `list::map`,
  a `for` loop with `try`, and so on). If an item's call fails with a runtime
  error, the failure propagates out of `spawn_map` the same way it propagates
  out of a `group` (5.5): items after the failing one never run.
- `spawn_map` runs eagerly and sequentially under the Phase 1 reference VM,
  the same as `list::map`; the difference is contract, not observable
  behavior yet. `spawn_map` documents fan-out work whose children are joined
  before it returns, which is the hook a future parallel runtime needs; a
  plain `list::map` makes no such promise and must stay sequential.

### 5.9 Stability Gate

Phase 1 syntax is implemented, but implementation alone does not guarantee it
belongs in the stable language surface. Before stabilization, Muga must validate the
same contract with at least one runtime that provides overlapping progress for
suspended or blocking tasks, actual parallel execution, or both. That runtime
must implement real sibling cancellation and failure propagation, preserve the
observable rules in section 5.5, enforce capture safety, and clean up resources
when a group exits. CPU parallel speedup is not an admission requirement;
structured concurrency may earn its value through IO overlap, lifetime control,
and cancellation.

If that contract is not ready to stabilize, the implemented syntax may remain
available as an explicitly experimental feature, but `group`, `spawn`,
`join`, `spawn_map`, and the internal `Task` contract should be deferred from
the stable language contract. Deferral does not require deleting the implementation.
Further concurrency surface and a stable service-IO surface must wait for this gate;
focused IO prototypes may be used to test suspension and cancellation.

## 6. No Async Function Coloring As The Primary Model

The recommended direction is to avoid making the entire language revolve around `async fn` and `await` coloring.

That means the primary user experience should stay close to:

- ordinary functions
- explicit task scopes
- explicit `spawn`
- explicit `join`

This keeps Muga readable and makes concurrency feel like a clear extension of the core language rather than a second language living beside it.

This does not forbid future async-specific APIs.

It only means they should not become the main model unless there is strong evidence they are necessary.

## 7. Example Usage

### 7.1 Phase 1 fan-out in a request handler

```muga
fn handle(req: http::Request): http::Response {
  group {
    user_task = spawn users::fetch(req.user_id)
    orders_task = spawn orders::recent(req.user_id)
    profile_task = spawn profiles::load(req.user_id)

    http::json(Page {
      user: user_task.task::join()
      orders: orders_task.task::join()
      profile: profile_task.task::join()
    })
  }
}
```

This is the clearest first target for Muga concurrency:

- a small lexical scope
- a few lightweight spawned tasks
- explicit joins at the point where results are needed

### 7.2 Phase 1 package-qualified chained call inside a task

```muga
group {
  next_age = spawn user.users::birthday().age
  next_age.task::join()
}
```

This sample shows that Muga's normal expression style should remain usable inside concurrent code.

## 8. Open Design Constraints

The following constraints should stay visible while this draft evolves:

- concurrency syntax alone does not determine performance
- scheduler quality, allocation behavior, synchronization costs, and backend quality will dominate real results; performance is validated through benchmarks rather than assumptions
- the task core should be implementable without requiring expensive global effect analysis
- diagnostics for task failure, cancellation, and cross-task source spans will matter early
- task designs should fit typed HIR and MIR lowering and the native backend cleanly

## 9. Deferred Topics

This document does not yet fix detached tasks, async IO integration, scheduler
details, or task type syntax in source. Decide them after the native runtime
exists and benchmark data is available.

## 10. Runtime And IO Integration Path

The task model and the IO runtime are separate decisions.

`group`, `spawn`, and `join` should establish task lifetime, result collection, failure propagation, cancellation, and capture rules. They do not by themselves define scalable socket IO, timers, backpressure, or service shutdown. Muga should avoid treating concurrency syntax as proof of runtime performance.

After the structured task core exists, the IO path should be:

1. define opaque resource handles for sockets, listeners, timers, files, and process-like OS resources
2. specify handle ownership, close/drop behavior, and task send/share rules
3. integrate handles with cancellation so a cancelled task can stop pending IO promptly
4. distinguish scheduler-aware nonblocking APIs from host APIs that may block an OS thread
5. add deadlines and timeouts as ordinary typed APIs that compose with task cancellation
6. use bounded queues, stream APIs, or explicit readiness to represent backpressure
7. benchmark large numbers of mostly-idle connections before designing higher-level service APIs

HTTP, SSE, WebSocket, and any future RPC streaming support should be layered above these lower decisions. They should not smuggle scheduler, cancellation, or backpressure semantics into framework conventions.

Recommended resource-style shape, not a committed syntax:

```muga
group {
  conn = try tcp::connect(addr)
  response_task = spawn handle_connection(conn)
  response_task.join()
}
```

The important constraints are:

- IO failures remain visible as `Result[T, E]`
- resource handles are opaque values, not transparent records
- cancellation behavior is specified at the API boundary
- hidden async suspension does not become ordinary function-call behavior
- task and resource facts can be represented in typed HIR, MIR, and package interfaces
