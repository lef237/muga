# Concurrency Draft

Status: design draft only. This document is not implemented in the Rust compiler yet. It defines a recommended direction for lightweight, structured, high-performance concurrency in Muga.

## 1. Design Goals

Muga's concurrency model should aim for all of the following:

- lightweight task creation
- simple and readable syntax
- strong defaults for safety
- explicit structure for task lifetime
- high runtime performance
- a compiler-friendly design that does not force expensive global analysis

This draft is intentionally positive and forward-looking.

It is not meant to criticize any existing language. The goal is to combine the easiest and clearest parts of modern concurrency design into something that fits Muga.

## 2. Core Direction

The recommended direction is:

- lightweight tasks
- structured concurrency
- typed message passing
- immutable-by-default sharing
- explicit joins and cancellation
- no "function coloring" as the primary user model

In practical terms, the preferred base model is:

- `group { ... }` creates a task scope
- `spawn expr` starts lightweight concurrent work inside that scope
- `task.join()` waits for a task and returns its result
- `channel(Type, capacity: N)` creates a typed channel
- `send` / `recv` style operations coordinate tasks

## 3. Why This Fits Muga

This direction fits Muga's existing language shape:

- bindings are immutable by default
- explicit structure is preferred over hidden behavior
- local reasoning is preferred over global magic
- the language already favors simple surface forms over heavy abstraction systems

Structured task scopes also fit the package / compiler roadmap well:

- they are easier to typecheck than detached background execution
- they are easier to lower into typed HIR and MIR
- they make runtime leaks and forgotten tasks easier to prevent

## 4. Task Scopes

The recommended primary concurrency construct is a lexical task scope:

```muga
group {
  ...
}
```

This scope defines the lifetime boundary for child tasks created inside it.

Initial draft rules:

- child tasks may not outlive their enclosing `group`
- leaving the group waits for child task completion
- if one child task fails, remaining child tasks are cancelled and the failure propagates out of the group
- cancellation propagates through nested task scopes

This gives Muga structured concurrency by default.

Detached background tasks may still be added later, but they should not be the default style.

## 5. Spawning Tasks

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

## 6. Joining Tasks

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

This should behave as:

- wait until the task finishes
- if it succeeds, return its value
- if it fails, propagate failure to the group

This style keeps result flow explicit without forcing a separate async-only function hierarchy.

## 7. No Async Function Coloring As The Primary Model

The recommended direction is to avoid making the entire language revolve around `async fn` and `await` coloring.

That means the primary user experience should stay close to:

- ordinary functions
- explicit task scopes
- explicit spawn
- explicit join

This is recommended because it keeps Muga readable and makes concurrency feel like a clear extension of the core language rather than a second language living beside it.

This does not forbid future async-specific APIs.

It only means they should not become the main model unless there is strong evidence they are necessary.

## 8. Channels

The recommended first coordination primitive is a typed channel.

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

## 9. Sharing And Safety

The recommended safety direction is:

- immutable values are easy to share across tasks
- mutable capture across task boundaries is restricted or made explicit
- channels and ownership transfer are the preferred coordination style
- locks and shared mutable synchronization may exist, but they should not define the primary style

This matches Muga's immutable-by-default design.

Initial draft recommendation:

- reading outer immutable bindings from a spawned task is allowed
- capturing or mutating outer mutable bindings across a task boundary is rejected in the default model

That keeps task interactions easier to reason about and avoids many accidental races in the common case.

## 10. Cancellation

Cancellation should be part of the model from the start.

Recommended direction:

- group failure cancels sibling tasks
- cancellation is structured and propagates downward
- task APIs may later grow explicit timeout and cancellation-token forms

This is preferable to a model where cancellation is bolted on after the fact.

## 11. Sample Style

### 11.1 Parallel request fan-out

```muga
group {
  user_task = spawn fetch_user(id)
  orders_task = spawn fetch_orders(id)
  profile_task = spawn fetch_profile(id)

  Page {
    user: user_task.join()
    orders: orders_task.join()
    profile: profile_task.join()
  }
}
```

### 11.2 Worker pipeline

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

### 11.3 Package-qualified chained call inside a task

```muga
group {
  next_age = spawn user.users::birthday().age
  next_age.join()
}
```

This sample shows that Muga's normal expression style should remain usable inside concurrent code.

## 12. Deferred Topics

This draft does not yet fix the full design of:

- `select` or multi-channel wait syntax
- channel closing semantics
- detached tasks
- supervisor-style task trees
- async IO integration
- scheduler details
- task type syntax in source
- interaction with generic types

Those topics should be decided after the compiler core is stronger and after benchmarking data exists.

## 13. Performance Target

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

## 14. Recommendation

The recommended Muga concurrency direction is:

1. `group`
2. `spawn`
3. `join`
4. typed channels
5. structured cancellation
6. shared mutability kept explicit and secondary

This is the clearest path toward concurrency that is:

- easy to write
- easy to read
- safe by default
- compatible with Muga's compiler and runtime goals
