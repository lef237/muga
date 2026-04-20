# Concurrency Draft

Status: design draft only. This document is not implemented in the Rust compiler yet.

This draft narrows the recommended direction for Muga concurrency into phases, so the first implemented core stays small, readable, and compiler-friendly.

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

Phase 1 should define and implement:

- `group`
- `spawn`
- `join()`
- structured failure propagation
- structured cancellation
- task-boundary capture rules

Phase 1 is the recommended first stable concurrency surface for Muga.

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

## 5. Phase 1: Structured Task Core

### 5.1 Task scopes

The recommended primary concurrency construct is a lexical task scope:

```muga
group {
  ...
}
```

This scope defines the lifetime boundary for child tasks created inside it.

Recommended Phase 1 rules:

- child tasks may not outlive their enclosing `group`
- leaving the group waits for child task completion
- if one child task fails, remaining child tasks are cancelled and the failure propagates out of the group
- cancellation propagates through nested task scopes

This gives Muga structured concurrency by default.

### 5.2 Spawning tasks

The recommended spawn form is:

```muga
task = spawn expr
```

Examples:

```muga
user_task = spawn fetch_user(id)
orders_task = spawn fetch_orders(id)
```

Recommended properties:

- `spawn` is lightweight
- the spawned expression runs concurrently
- the result is a task handle
- the handle may be joined later

This keeps task creation close to an ordinary expression-based style.

### 5.3 Joining tasks

The recommended result-collection form is:

```muga
value = task.join()
```

Example:

```muga
group {
  user_task = spawn fetch_user(id)
  orders_task = spawn fetch_orders(id)

  Page {
    user: user_task.join()
    orders: orders_task.join()
  }
}
```

Recommended Phase 1 behavior:

- wait until the task finishes
- if it succeeds, return its value
- if it fails, propagate failure through the enclosing group

This style keeps result flow explicit without forcing a separate async-only function hierarchy.

### 5.4 Sharing and safety

The recommended safety direction is:

- immutable values are easy to share across tasks
- mutable capture across task boundaries is restricted or made explicit
- channels and ownership transfer are the preferred coordination style once they exist
- locks and shared mutable synchronization may exist later, but they should not define the primary style

Initial Phase 1 recommendation:

- reading outer immutable bindings from a spawned task is allowed
- capturing or mutating outer mutable bindings across a task boundary is rejected in the default model

That keeps task interactions easier to reason about and avoids many accidental races in the common case.

### 5.5 Failure and cancellation

Cancellation should be part of the model from the start.

Recommended Phase 1 direction:

- group failure cancels sibling tasks
- cancellation is structured and propagates downward
- `join()` observes normal completion or propagated failure

This is preferable to a model where cancellation is bolted on later.

### 5.6 Task handle typing

Muga will eventually need an internal notion equivalent to `Task[T]`.

However, this draft does **not** yet fix:

- how task-handle types are written in source
- whether task handles are user-nameable types in v1
- whether task handles are modeled as nominal or builtin runtime types

Phase 1 only requires that the compiler and runtime carry task-result typing internally.

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
      user: user_task.join()
      orders: orders_task.join()
      profile: profile_task.join()
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
  next_age.join()
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

## 11. Performance Target

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

## 12. Recommendation

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
