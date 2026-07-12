# Concurrency

Status: Phase 1 (structured task groups: `group`, `spawn`, `join`) is
implemented in the Rust compiler and reference VM. Section 5 is the
specification for the implemented behavior. Sections describing channels,
selection, time, and service IO remain design drafts and are not
implementation queues.

This document narrows the recommended direction for Muga concurrency into phases, so the first implemented core stays small, readable, and compiler-friendly.

## 1. Design Goals

Muga's concurrency model should aim for all of the following:

- lightweight task creation
- simple and readable syntax
- strong defaults for safety
- explicit structure for task lifetime
- high runtime performance
- a compiler-friendly design that does not force expensive global analysis

This draft is intentionally positive and forward-looking.

It is not meant to criticize any existing language. The goal is to combine the clearest parts of modern concurrency design into something that fits Muga.

## 2. Core Direction

The recommended direction is:

- lightweight tasks
- structured concurrency
- immutable-by-default sharing
- explicit joins and cancellation
- no async function coloring as the primary user model

The most important recommendation is that **Muga should stabilize task groups first**. Channels, `select`, timeouts, and service-style runtime features should come later.

In practical terms, the preferred base model is:

- `group { ... }` creates a task scope
- `spawn expr` starts lightweight concurrent work inside that scope
- `task.join()` waits for a task and returns its result

This draft still recommends typed channels, but **not as part of the smallest first implementation**.

## 3. Phased Rollout

### 3.1 Phase 1: Structured task core

Phase 1 defines and implements:

- `group`
- `spawn`
- `join()`
- structured failure propagation
- structured cancellation
- task-boundary capture rules

Phase 1 is implemented; section 5 specifies the implemented behavior.

### 3.2 Phase 2: Typed channels

Only after the task core is clear should Muga add:

- typed channels
- buffered and unbuffered channel behavior
- `send` / `recv`
- channel close semantics
- worker-pool style coordination

This phase depends on a clearer story for task and channel types.

### 3.3 Phase 3: Selection and time

After channels are stable, Muga can add:

- `select` or equivalent multi-wait syntax
- timeout support
- deadline support
- cancellation-token style APIs if they are still needed

This should come after real usage and benchmarks exist for the smaller core.

### 3.4 Later phases

The following are intentionally later topics:

- detached background tasks
- supervision trees
- long-lived service runtimes
- async IO integration details
- distributed runtime or actor-style features

These may matter for web systems, but they should not shape the smallest useful core.

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
  as section 11 already requires.
- Channels and ownership transfer remain the preferred coordination style
  once they exist (Phase 2).

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

- no channels, `select`, timeouts, deadlines, or detached tasks; those stay
  in later phases and remain drafts
- no source-level `Task[T]` type syntax and no user-nameable task type
- no timeout API: Phase 1 does not promise async IO behavior, so time-based
  cancellation waits for the IO/runtime integration path in section 11
- no async function coloring, in line with section 7

### 5.7 Phase 1 usage notes

These notes come from writing realistic `group` / `spawn` / `join` programs
against the implemented Phase 1 core, gathered to judge whether Phase 2
(channels) is the right next step. See
`samples/packages/app/std_task_result/main.muga`,
`samples/packages/app/std_task_list/main.muga`,
`samples/packages/app/std_task_for/main.muga`, and
`samples/projects/task_app/src/main/main.muga`.

What works well for fixed-shape fan-out, the case section 8.1 targets:

- A literal list of spawned tasks joined through `list::map` and `list::fold`
  reads naturally: `tasks = [spawn work(1), spawn work(2), spawn work(3)]`
  then `list::map(tasks, fn(t) { t.task::join() })`. The closure passed to
  `list::map` only calls `join`, never `spawn`, so it never crosses a task
  boundary.
- `try handle.task::join()` and `try task::join(handle)` both compose inside
  `Result`-returning functions, so recoverable-error fan-out (spawn several
  fallible calls, `try`-join each, combine) reads like ordinary sequential
  `Result` code.
- Fire-and-forget `spawn` inside a `for` loop over a runtime-sized collection
  works: the loop is not a function boundary, so each iteration can `spawn`
  directly, and the enclosing `group` still waits for every iteration's task
  before it returns. This covers batch side-effecting work where no result
  needs to flow back.

What did not work before `task::spawn_map` (5.8) closed the gap:

- Dynamic, result-collecting fan-out over a runtime-sized collection had no
  expressible form using `spawn` directly. `list::map`'s callback is an
  ordinary `fn` value, and function boundaries reset the group context
  (5.2), so `spawn` inside a mapped closure is rejected with `T030` even
  though the mapped closure runs inside the same `group`. `List` does have
  user-facing mutation (`push`, used by `list::map` itself), so building a
  list by hand is not the blocker; the blocker is that a user-written `mut`
  binding or return type cannot name `List[Task[T]]`, because `T013` forbids
  `Task[T]` syntax outside `std::task` (5.3).
- In practice this meant fan-out whose arity is known at the call site
  (a fixed literal list, or a fixed number of named `spawn` bindings) worked,
  but "spawn one task per row of this query result" or "one task per file in
  this directory listing" could not be written without giving up on
  collecting per-task results.

This gap was narrower than what channels solve. Channels (section 6) target
streaming and worker-pool coordination between independently-scheduled
tasks; the missing piece here was simpler: a way to spawn and join over an
existing collection without a user ever naming `Task[T]`. `task::spawn_map`
(5.8) closes it as a `std::task` library function, not new syntax.

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
Channels, `select`, and a stable service-IO surface must wait for this gate;
focused IO prototypes may be used to test suspension and cancellation.

## 6. Phase 2: Typed Channels

After the task core is stable, the recommended first coordination primitive is a typed channel.

Suggested construction form:

```muga
jobs = channel(Job, capacity: 64)
results = channel(Result, capacity: 64)
```

Suggested operations:

```muga
jobs.send(job)
job = jobs.recv()
```

Recommended properties:

- channels are typed
- channels may be buffered
- send and receive block according to channel state
- channel use should be easy to read in source

This keeps the syntax consistent with the rest of Muga:

- method-like surface forms
- explicit values
- no special symbolic arrows required

### 6.1 Why channels are not Phase 1

Channels depend on unresolved questions that do not need to block the task core:

- generic type story for `Channel[T]`
- close semantics
- `recv()` behavior at end-of-stream
- buffering guarantees
- fairness and wake-up policy

Those are real design questions, but they are easier to answer after the simpler task core exists.

## 7. No Async Function Coloring As The Primary Model

The recommended direction is to avoid making the entire language revolve around `async fn` and `await` coloring.

That means the primary user experience should stay close to:

- ordinary functions
- explicit task scopes
- explicit `spawn`
- explicit `join`

This keeps Muga readable and makes concurrency feel like a clear extension of the core language rather than a second language living beside it.

This does not forbid future async-specific APIs.

It only means they should not become the main model unless there is strong evidence they are necessary.

## 8. Example Usage

### 8.1 Phase 1 fan-out in a request handler

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

### 8.2 Phase 1 package-qualified chained call inside a task

```muga
group {
  next_age = spawn user.users::birthday().age
  next_age.task::join()
}
```

This sample shows that Muga's normal expression style should remain usable inside concurrent code.

### 8.3 Phase 2 worker pipeline

```muga
group {
  jobs = channel(Int, capacity: 64)
  results = channel(Int, capacity: 64)

  producer = spawn produce_jobs(jobs)
  worker1 = spawn worker(jobs, results)
  worker2 = spawn worker(jobs, results)

  first = results.recv()
  second = results.recv()

  producer.join()
  worker1.join()
  worker2.join()

  first + second
}
```

This style is still recommended, but it should come after the smaller task core is working well.

## 9. Open Design Constraints

The following constraints should stay visible while this draft evolves:

- concurrency syntax alone does not determine performance
- scheduler quality, allocation behavior, synchronization costs, and backend quality will dominate real results
- the task core should be implementable without requiring expensive global effect analysis
- diagnostics for task failure, cancellation, and cross-task source spans will matter early
- task and channel designs should fit future typed HIR and MIR lowering cleanly

## 10. Deferred Topics

This draft does not yet fix the full design of:

- `select` or multi-channel wait syntax
- channel closing semantics
- detached tasks
- supervisor-style task trees
- async IO integration
- scheduler details
- task type syntax in source
- channel type syntax in source
- interaction with generic types

Those topics should be decided after the compiler core is stronger and after benchmarking data exists.

## 11. Runtime And IO Integration Path

The task model and the IO runtime are separate decisions.

`group`, `spawn`, and `join` should establish task lifetime, result collection, failure propagation, cancellation, and capture rules. They do not by themselves define scalable socket IO, timers, backpressure, or service shutdown. Muga should avoid treating concurrency syntax as proof of runtime performance.

After the structured task core exists, the IO path should be:

1. define opaque resource handles for sockets, listeners, timers, files, and process-like OS resources
2. specify handle ownership, close/drop behavior, and task send/share rules
3. integrate handles with cancellation so a cancelled task can stop pending IO promptly
4. distinguish scheduler-aware nonblocking APIs from host APIs that may block an OS thread
5. add deadlines and timeouts as ordinary typed APIs that compose with task cancellation
6. use bounded channels, stream APIs, or explicit readiness to represent backpressure
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

## 12. Performance Target

The performance goal is ambitious:

- very lightweight task creation
- low scheduling overhead
- strong throughput under large numbers of concurrent tasks
- practical competitiveness with established lightweight-concurrency runtimes

However, syntax alone does not guarantee this.

Real results will depend on:

- scheduler design
- memory allocation behavior
- synchronization costs
- channel implementation
- native backend quality

So the right policy is:

- keep the syntax small and clear
- make the semantics structured and safe
- validate performance through benchmarks rather than assumptions

## 13. Recommendation

The recommended Muga concurrency direction is:

1. stabilize `group`
2. stabilize `spawn`
3. stabilize `join`
4. define structured failure and cancellation
5. add typed channels only after the task core is solid
6. add `select` and time-based waiting only after channels are proven out

This is the clearest path toward concurrency that is:

- easy to write
- easy to read
- safe by default
- compatible with Muga's compiler and runtime goals
