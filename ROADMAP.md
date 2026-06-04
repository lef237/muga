# Muga Roadmap

This file is the single working checklist for implementation order. Language
rules belong in [spec-v1.md](./spec-v1.md) and [spec/](./spec/). Executable
behavior should be proved with Rust tests, conformance fixtures, and runnable
Muga samples.

## Resume Cursor

- [ ] **NOW P0:** prepare a v1 release candidate when the version and release
  timing are decided.
- [ ] **NEXT P0:** follow [RELEASING.md](./RELEASING.md), including the
  publish dry run, version bump, tag, and release workflow.
- [ ] **POST-v1:** revisit structured task groups only after v1 ships.

Last verified locally on 2026-06-04 after `std::process` and v1 release
hardening:

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
  `std::env`, `std::process`, `std::cli`, `std::time`, `std::bytes`, `std::hash`,
  `std::string`, `std::fmt`, `std::list`, `std::map`, `std::option`,
  `std::result`, `std::json`, `std::config`, and `std::test`
- [x] diagnostic JSON context for source, package, artifact-root, concrete
  artifacts, hashes, and regeneration commands where available

## Design Commitments

These are direction-setting commitments, not just missing implementation work.

- [x] Muga is function-centered: records define data, ordinary functions define
  behavior, and dot calls remain surface syntax over functions.
- [x] Muga keeps one ordinary function namespace and avoids overloaded dispatch.
- [x] Muga does not add protocol/trait/interface/typeclass-style behavior
  conformance; use ordinary functions, higher-order functions, explicit
  wrappers, package qualification, and enums with `match`.
- [x] Muga uses value semantics in ordinary source. The implementation may use
  sharing, handles, copy elision, or native representations internally, but
  ordinary code should not expose pointer, reference, ownership, or borrowing
  syntax.
- [x] Muga keeps package boundaries explicit and artifact-backed execution
  honest; source-free execution must not silently fall back to dependency
  source bodies.
- [x] Muga prefers explicit recoverable error values over implicit exceptions.

## V1 Release Strategy

The recommended path is now to stop adding v1 user-visible features and
stabilize for release.

- [x] Treat `std::process` as the last planned user-visible feature before v1.
- [x] Do not start structured concurrency, service IO, remote registries,
  native backend work, broad collections, or new source syntax before v1.
- [x] Spend the remaining v1 work on release hardening:
  documentation alignment, sample/template coverage, diagnostics, and release
  gate reliability.
- [x] Only promote another task to pre-v1 P0 if it is a correctness bug,
  release-gate failure, source-free/artifact fallback violation, broken sample
  or template, broken public diagnostic contract, or direct contradiction in
  the v1 specs.

## Completed P0: `std::process`

Goal: add the final planned v1 standard-library capability: a narrow,
recoverable process execution API without adding shell syntax, async runtime
assumptions, or concurrency syntax.

### API Shape

- [x] Decide the public package surface in `src/std_package.rs`.
- [x] Add `std::process` as a virtual package.
- [x] Use explicit records and enums rather than ad hoc strings:
  - [x] `ErrorKind`
  - [x] `Error`
  - [x] `EnvVar` as the explicit env override shape
  - [x] `Options` with optional cwd and explicit env overrides
  - [x] `Output` with status, success, stdout, and stderr
- [x] Treat nonzero child exit as captured `Output`, not as `Result::Err`.
- [x] Reserve `Result::Err` for spawn, wait, cwd/env setup, and capture/UTF-8
  failures.
- [x] Keep command execution direct. Do not add shell interpolation or
  `sh -c` helpers in the first slice.
- [x] Use `path::Path` for cwd rather than raw host path strings in public APIs.
- [x] Keep environment inheritance rules explicit in docs and tests.

### Compiler And Runtime

- [x] Add `PROCESS_PACKAGE` and process builtin constants in `src/std_package.rs`.
- [x] Add process builtin ids and debug labels in `src/prelude.rs`.
- [x] Permit the new internal builtins only for `std::process`.
- [x] Add typechecker rules in `src/typing.rs` for process builtins and public
  result/error shapes.
- [x] Add runtime execution in `src/runtime.rs` using `std::process::Command`.
- [x] Capture stdout and stderr deterministically as `String` values.
- [x] Convert recoverable host errors into public `process::Error` records.
- [x] Reject malformed internal runtime values with hard runtime diagnostics,
  following the existing `std::fs` handle pattern.
- [x] Ensure built-artifact execution works without dependency source fallback.

### Samples And Tests

- [x] Add one runnable sample under `samples/packages/app/std_process/`.
- [x] Add a manifest project sample under `samples/projects/process_app/` for
  source-free bundle coverage.
- [x] Add focused source-run tests in `tests/examples.rs`.
- [x] Add artifact-backed `std::process` tests that hide dependency sources.
- [x] Test successful exit with captured stdout and stderr.
- [x] Test nonzero exit as successful capture with `success == false`.
- [x] Test cwd handling with `path::Path`.
- [x] Test env override handling.
- [x] Test spawn failure as `Result::Err(process::Error)`.
- [x] Test type mismatch diagnostics for command, args, cwd, and env records.
- [x] Add release-gate smoke coverage for the package sample and source-free
  bundle coverage for the project sample.

### Documentation

- [x] Update `spec-v1.md` current implementation boundary after the slice lands.
- [x] Update `spec/003-typing.md` standard package surface after the slice lands.
- [x] Add process-specific diagnostic guidance in `errors.md` only if a new
  public diagnostic family is introduced.
- [x] Keep `README.md` quickstart unchanged unless a process example becomes
  the best first standard-library example.

### Done When

- [x] `cargo fmt --check` passes.
- [x] `scripts/clippy-check.sh` passes.
- [x] `cargo test --locked` passes.
- [x] `scripts/v1-release-gate.sh` passes.
- [x] `std::process` works through `muga run --built` and source-free bundle
  execution without reading dependency source bodies.

## P0: V1 Release Hardening

Completed after `std::process` landed and was release-gated.

- [x] Freeze new v1 language and standard-library features.
- [x] Update `spec-v1.md` so implemented and not-implemented boundaries match
  the Rust implementation after `std::process`.
- [x] Confirm every item in `spec-v1.md` "Not implemented" is either
  explicitly post-v1 or not planned.
- [x] Re-run a focused unfinished-work audit with `rg` over `src/`, `tests/`,
  `samples/`, and `spec/` for `TODO`, `FIXME`, `todo!`, `unimplemented!`,
  stale "future" examples, and deleted-doc links.
- [x] Keep any remaining `unreachable!` or `panic!` sites limited to internal
  invariant checks and tests, not user-facing incomplete features.
- [x] Verify all `muga new` templates, runnable samples, package samples, and
  source-free app bundle workflows are covered by Rust tests or the release
  gate.
- [x] Make the release gate the authoritative v1 readiness command.
- [x] Update `README.md`, `RELEASING.md`, `errors.md`, and this roadmap only
  where they affect a v1 user or releaser.
- [x] Run `cargo fmt --check`, `scripts/clippy-check.sh`, `cargo test --locked`,
  and `scripts/v1-release-gate.sh` from the final tree.

## Post-v1 P1: Structured Task Groups

Do not start this before v1 release unless the release definition changes.

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

## Post-v1 P2: Service IO

Do not start this before task lifetime, shutdown, and backpressure semantics
are explicit.

- [ ] Choose the first service IO target: sockets or minimal HTTP/JSON.
- [ ] Keep resource handles opaque and closeable.
- [ ] Define shutdown behavior before exposing listeners or streams.
- [ ] Define backpressure behavior before exposing streaming request/response
  APIs.
- [ ] Keep JSON integration explicit through `std::json` schemas.
- [ ] Prove source, built-artifact, and source-free bundle execution.

## Post-v1 P2: Performance Path

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

## Post-v1 P2: Distribution Path

Distribution should build on the existing `.mgp` / `.mga` work.

- [ ] Harden install inventory UX and diagnostics around app bundle ownership.
- [ ] Add more source-free bundle smoke cases for std packages that use host
  effects.
- [ ] Decide whether project-mode artifact-root configuration is needed after
  more build/reuse evidence.
- [ ] Keep package identity tied to `.mgp` content hashes.
- [ ] Defer URL/Git/registry fetching until local archive identity, lockfile
  behavior, and install inventory remain stable across releases.

## Parked Non-Blockers

These are known implementation gaps or design extensions. They should not block
v1 unless a concrete release-gate failure or user-facing correctness issue moves
one into P0.

- [ ] public-signature inference for `pub fn`; keep explicit public signatures
  for v1 because they stabilize package interfaces.
- [ ] project-mode artifact-root configuration and full incremental package
  artifact reuse; revisit after real projects show repeated build pain.
- [ ] structural equality, `List.contains`, structural `assert_eq`,
  `Map.entries`, `Set[T]`, arbitrary `Map` key types, map literals, and broad
  collection APIs; revisit only with an explicit equality/hash design that does
  not introduce behavior-conformance systems.
- [ ] broader JSON/config schema targets such as generic records, generic
  enums, nested `Option[Option[T]]`, non-string map keys, and validation
  attributes; revisit after the current concrete schema slice is exercised.
- [ ] future `expr.try`, `T?`, and `Option`-only optional chaining; revisit
  only if explicit `try`, `Option`, and helper packages become too noisy in
  real code.
- [ ] broad wildcard matching, nested patterns, guards, multi-payload variants,
  and named-field enum variants; revisit only with concrete examples that make
  the current exhaustive `match` form too verbose.
- [ ] source-level consuming parameter declarations, broader runtime-backed
  handle families, `using` expressions, multiple `using` bindings, and
  aggregate cleanup errors; revisit after `std::process` and more handle APIs
  prove the need.
- [ ] binary streams, codecs, broad cryptography, service runtime APIs, and
  async IO; revisit after resource handles, `std::process`, and post-v1 task
  lifetime rules are stable.
- [ ] URL/Git/registry dependencies, remote fetching, publishing workflows,
  package signing, SBOMs, and full published-package lockfile enforcement;
  revisit after local `.mgp` / `.mga` workflows are stable in real use.
- [ ] control-flow-oriented MIR, native backend, and representation performance
  work; revisit after benchmark data shows the reference VM or current bytecode
  is the limiting factor.

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

## Not Planned

These features conflict with Muga's current direction. Do not treat them as
future backlog items.

- [x] do not add classes, inheritance, member-owned methods, member ownership
  semantics, or class-style encapsulation
- [x] do not add method dispatch as a separate semantic category from ordinary
  function calls
- [x] do not add overloaded function dispatch, overloaded operator dispatch, and
  user-defined overload sets
- [x] do not add source-level references, mutable references, pointer syntax,
  ownership syntax, borrowing syntax, raw pointer arithmetic, or general
  writable aliases in ordinary Muga code
- [x] do not add implicit exceptions or `throws`
- [x] do not add postfix `expr?` for `Result` propagation
- [x] do not add `protocol`, `trait`, `interface`, or `typeclass` declarations
  for shared behavior
- [x] do not add behavior-conformance systems, protocol bounds, trait bounds,
  typeclass solving, default implementations, blanket implementations,
  protocol objects, or conformance-based dot lookup

## Short Version

Muga's next concrete step is release-candidate preparation once the version and
timing are decided. `std::process` has landed as the final planned
user-visible standard-library slice before v1, v1 release hardening is complete,
and `scripts/v1-release-gate.sh` is the authoritative readiness command.
Structured task groups, service IO, remote registries, broad collections, and
native backend work are post-v1.
