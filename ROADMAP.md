# Muga Roadmap

This file is the single working checklist for implementation order. Language
rules belong in [spec-v1.md](./spec-v1.md) and [spec/](./spec/). Executable
behavior should be proved with Rust tests, conformance fixtures, and runnable
Muga samples.

## Resume Cursor

- [ ] **NOW P0:** implement the first `std::process` vertical slice.
- [ ] **NEXT P0:** prove `std::process` through source runs, built-artifact
  runs, at least one runnable sample, and release-gate coverage.
- [ ] **THEN P1:** revisit structured task groups only after `std::process`
  establishes the next external-effect boundary.

Last verified locally:

- [x] `cargo test --locked`
- [x] `scripts/v1-release-gate.sh`

## Current State

Muga currently has:

- [x] lexer, parser, resolver, typechecker, typed HIR, MIR lowering, bytecode,
  and a reference VM runtime
- [x] `check`, `run`, `test`, `fmt`, `doc`, `build`, `syntax`, `doctor`,
  `explain`, `metadata`, `schema`, `workspace`, `why-rebuild`, `api-diff`,
  `completions`, `definition`, `references`, `hover`, `new`, artifact,
  archive, bundle, app install, and completion-package commands
- [x] package interfaces and artifacts through `.mgi`, `.mgc`, and `.mgb`
- [x] local path and local `.mgp` archive dependencies with lockfile metadata
- [x] app and package archive workflows through `.mga` and `.mgp`
- [x] source-free app bundles, app archive round-trips, non-mutating install
  inventory, and generated app completion packages
- [x] core v1 language surface: immutable-by-default bindings, `mut`, records,
  enums, functions, closures, local inference, explicit generic
  records/functions, `Option`, `Result`, prefix `try`, exhaustive `match`,
  `for`, `break`, `continue`, `return`, `Unit`, package imports, `pub opaque
  type`, runtime-backed `std::fs::File`, and statement-form `using`
- [x] standard package slices for `std::io`, `std::fs`, `std::path`,
  `std::env`, `std::cli`, `std::time`, `std::bytes`, `std::hash`,
  `std::string`, `std::fmt`, `std::list`, `std::map`, `std::option`,
  `std::result`, `std::json`, `std::config`, and `std::test`
- [x] diagnostic JSON context for source, package, artifact-root, concrete
  artifacts, hashes, and regeneration commands where available

## P0: `std::process`

Goal: add a narrow, recoverable process execution API without adding shell
syntax, async runtime assumptions, or concurrency syntax.

### API Shape

- [ ] Decide the public package surface in `src/std_package.rs`.
- [ ] Add `std::process` as a virtual package.
- [ ] Use explicit records and enums rather than ad hoc strings:
  - [ ] `ErrorKind`
  - [ ] `Error`
  - [ ] `EnvVar` or equivalent explicit env override shape
  - [ ] `Options` with optional cwd and explicit env overrides
  - [ ] `Output` with status, success, stdout, and stderr
- [ ] Treat nonzero child exit as captured `Output`, not as `Result::Err`.
- [ ] Reserve `Result::Err` for spawn, wait, cwd/env setup, and capture/UTF-8
  failures.
- [ ] Keep command execution direct. Do not add shell interpolation or
  `sh -c` helpers in the first slice.
- [ ] Use `path::Path` for cwd rather than raw host path strings in public APIs.
- [ ] Keep environment inheritance rules explicit in docs and tests.

### Compiler And Runtime

- [ ] Add `PROCESS_PACKAGE` and process builtin constants in `src/std_package.rs`.
- [ ] Add process builtin ids and debug labels in `src/prelude.rs`.
- [ ] Permit the new internal builtins only for `std::process`.
- [ ] Add typechecker rules in `src/typing.rs` for process builtins and public
  result/error shapes.
- [ ] Add runtime execution in `src/runtime.rs` using `std::process::Command`.
- [ ] Capture stdout and stderr deterministically as `String` values.
- [ ] Convert recoverable host errors into public `process::Error` records.
- [ ] Reject malformed internal runtime values with hard runtime diagnostics,
  following the existing `std::fs` handle pattern.
- [ ] Ensure built-artifact execution works without dependency source fallback.

### Samples And Tests

- [ ] Add one runnable sample under `samples/packages/app/std_process/`.
- [ ] Add focused source-run tests in `tests/examples.rs`.
- [ ] Add artifact-backed `std::process` tests that hide dependency sources.
- [ ] Test successful exit with captured stdout and stderr.
- [ ] Test nonzero exit as successful capture with `success == false`.
- [ ] Test cwd handling with `path::Path`.
- [ ] Test env override handling.
- [ ] Test spawn failure as `Result::Err(process::Error)`.
- [ ] Test type mismatch diagnostics for command, args, cwd, and env records.
- [ ] Add release-gate smoke coverage for the sample.

### Documentation

- [ ] Update `spec-v1.md` current implementation boundary after the slice lands.
- [ ] Update `spec/003-typing.md` standard package surface after the slice lands.
- [ ] Add process-specific diagnostic guidance in `errors.md` only if a new
  public diagnostic family is introduced.
- [ ] Keep `README.md` quickstart unchanged unless a process example becomes
  the best first standard-library example.

### Done When

- [ ] `cargo fmt --check` passes.
- [ ] `scripts/clippy-check.sh` passes.
- [ ] `cargo test --locked` passes.
- [ ] `scripts/v1-release-gate.sh` passes.
- [ ] `std::process` works through `muga run --built` and source-free bundle
  execution without reading dependency source bodies.

## P1: Structured Task Groups

Do not start this until `std::process` is complete and release-gated.

- [ ] Reconcile `spec/007-concurrency-draft.md` with the implemented value,
  package, handle, and artifact model.
- [ ] Decide whether the first task API is syntax (`group` / `spawn` / `join`)
  or a standard package abstraction.
- [ ] Define task lifetime rules: child tasks may not outlive their group.
- [ ] Define failure propagation and cancellation behavior.
- [ ] Define capture rules for immutable values, mutable bindings, and
  runtime-backed handles.
- [ ] Define timeout boundaries without promising async socket IO.
- [ ] Add AST/parser support only after the type and runtime behavior are
  settled.
- [ ] Add typed HIR, MIR/bytecode, and VM support.
- [ ] Add conformance fixtures for accepted and rejected task usage.
- [ ] Add benchmark-health checks only as local health measurements, not public
  performance claims.

## P2: Service IO

Do not start this before task lifetime, shutdown, and backpressure semantics
are explicit.

- [ ] Choose the first service IO target: sockets or minimal HTTP/JSON.
- [ ] Keep resource handles opaque and closeable.
- [ ] Define shutdown behavior before exposing listeners or streams.
- [ ] Define backpressure behavior before exposing streaming request/response
  APIs.
- [ ] Keep JSON integration explicit through `std::json` schemas.
- [ ] Prove source, built-artifact, and source-free bundle execution.

## P2: Performance Path

Performance work needs evidence before native backend claims.

- [ ] Extend benchmark health checks with representative compiler, package,
  artifact, and runtime workloads.
- [ ] Identify the hottest runtime representations with measurements.
- [ ] Improve `List`, `Map`, `String`, and `Bytes` representation only when
  source semantics remain unchanged.
- [ ] Introduce control-flow-oriented MIR only after the VM and artifact tests
  show the current bytecode path is the bottleneck.
- [ ] Keep native backend work deferred until MIR, package artifacts, and
  runtime representation have measurable pressure.

## P2: Distribution Path

Distribution should build on the existing `.mgp` / `.mga` work.

- [ ] Harden install inventory UX and diagnostics around app bundle ownership.
- [ ] Add more source-free bundle smoke cases for std packages that use host
  effects.
- [ ] Decide whether project-mode artifact-root configuration is needed after
  more build/reuse evidence.
- [ ] Keep package identity tied to `.mgp` content hashes.
- [ ] Defer URL/Git/registry fetching until local archive identity, lockfile
  behavior, and install inventory remain stable across releases.

## Documentation Hygiene

- [ ] Keep this file as the roadmap and avoid adding another planning document.
- [ ] Keep detailed language prose in `spec-v1.md` or split `spec/` files.
- [ ] Keep examples runnable; invalid examples belong under `examples/invalid/`,
  `conformance/v1/rejecting/`, or `spec/snippets/`.
- [ ] When implementation changes a public rule, update the closest spec and
  add a focused Rust test in the same change.
- [ ] When adding a public diagnostic code or changing its trigger, update
  `errors.md` and add or adjust a focused test.

## Stability Rules

- [ ] Keep the v1 source model small.
- [ ] Prefer code, samples, conformance fixtures, and Rust tests over long
  design prose.
- [ ] Do not make normal `check` or `run` silently depend on built artifacts.
- [ ] Keep artifact-backed package execution hard-failing without dependency
  source fallback.
- [ ] Keep public `pub fn` signatures explicit until public-signature inference
  is deliberately implemented.
- [ ] Keep diagnostics stable, actionable, and source/artifact-aware.

## Explicitly Deferred

These are not active implementation work unless a concrete, tested slice above
forces the decision:

- [ ] classes, inheritance, traits, protocols, typeclasses, overloaded dispatch
- [ ] source-level references, mutable references, pointer syntax, borrowing
- [ ] postfix `expr?`, future `expr.try`, `T?`, optional chaining
- [ ] broad wildcard matching, nested patterns, guards, multi-payload variants
- [ ] map literals, `Set[T]`, arbitrary `Map` keys, broad collection APIs,
  iterator protocols
- [ ] URL/Git/registry dependencies, remote fetching, publishing workflows,
  package signing, SBOMs, full published-package lockfile enforcement
- [ ] binary streams, codecs, broad cryptography, service runtime APIs, async IO,
  native backend work

## Short Version

Muga's next concrete step is `std::process`. Implement it as a small standard
package slice, prove it through source and artifact-backed execution, then only
move to structured task groups once the external-effect boundary is stable.
