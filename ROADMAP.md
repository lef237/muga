# Muga Roadmap

This file is the single working checklist for implementation order. Language
rules belong in [spec-v1.md](./spec-v1.md) and [spec/](./spec/). Executable
behavior should be proved with Rust tests, conformance fixtures, and runnable
Muga samples.

## Resume Cursor

- [x] **DONE:** `0.5.0` shipped on 2026-07-02: version bump, release gate
  with publish dry run, `v0.5.0` tag, crates.io publish through the release
  workflow, and GitHub Release all verified. Muga stays in the `0.x` series;
  the `1.0.0` compatibility promise has not started.
- [x] **DONE:** structured task groups Phase 1 implemented on 2026-07-03:
  `group` / `spawn` syntax, the `std::task` package with `join`, capture and
  scope diagnostics (`T030`, `E013`), artifact support, conformance
  fixtures, and samples.
- [x] **DONE:** `0.6.0` shipped on 2026-07-03 with the structured task
  groups slice: version bump, release gate with publish dry run, `v0.6.0`
  tag, crates.io publish through the release workflow, and GitHub Release
  all verified.
- [x] **DONE:** gathered real task-group usage on 2026-07-05: added
  `std_task_result`, `std_task_list`, and `std_task_for` package samples plus
  a `task_app` project sample with bundle coverage, fixed a `try`/generic
  return-type typing bug (`try` rejected any call whose *unspecialized*
  generic signature had a bare type-parameter return, including
  `task::join`), and recorded findings in
  spec/007-concurrency-draft.md#57-phase-1-usage-notes. Conclusion: fixed-
  arity fan-out (literal task lists, or fire-and-forget `spawn` inside a
  `for` loop) works well; dynamic, result-collecting fan-out over a
  runtime-sized collection has no expressible form (`T030` inside mapped
  closures, `T013` blocks a hand-written `List[Task[T]]`). This gap is
  narrower than channels; a small `std::task` fan-out combinator could close
  it without Phase 2.
- [x] **DONE:** closed the dynamic-fan-out gap on 2026-07-05 with
  `task::spawn_map[T, U](items: List[T], f: T -> U): List[U]`, a `std::task`
  package function (no new syntax or diagnostics) that spawns `f` over every
  item and joins all results before returning; see
  spec/007-concurrency-draft.md#58-spawn_map-fan-out-over-a-runtime-sized-collection
  and `samples/packages/app/std_task_spawn_map/main.muga`.
- [ ] **NOW:** gather real usage of `task::spawn_map` and the rest of the
  Phase 1 core before deciding whether to promote Phase 2 (channels) or
  service IO work.

Last verified locally on 2026-06-05 during the pre-v1 implementation audit:

- [x] `cargo fmt --check`
- [x] `git diff --check`
- [x] `scripts/clippy-check.sh`
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
- [x] structured task groups Phase 1: `group` expression scopes, prefix
  `spawn` with `T030` / `E013` diagnostics, the internal `Task[T]` handle
  type, and deterministic reference execution per spec/007 section 5
- [x] standard package slices for `std::io`, `std::fs`, `std::path`,
  `std::env`, `std::process`, `std::cli`, `std::time`, `std::bytes`, `std::hash`,
  `std::string`, `std::fmt`, `std::list`, `std::map`, `std::option`,
  `std::result`, `std::json`, `std::config`, `std::task`, and `std::test`
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

## Completed Milestones

Finished work is summarized here. Full checklists, audit notes, and decision
logs live in git history; the resulting rules live in the specs, `errors.md`,
and `RELEASING.md`.

- [x] `std::process` (the last planned v1 standard-library capability): narrow
  recoverable process execution through explicit `Options` / `Output` records,
  nonzero child exits captured as `Result::Ok(Output)`, no shell
  interpolation, `path::Path` cwd, explicit env overrides, and source-free
  bundle coverage.
- [x] V1 release hardening: v1 feature freeze, `spec-v1.md` boundary aligned
  with the implementation, unfinished-work audit, template/sample/bundle test
  coverage, and the release gate as the authoritative readiness command.
- [x] Pre-v1 implementation audit (2026-06-05): hotspot review across parser,
  resolver, typing, MIR/VM, artifacts, and CLI contracts; production
  panic-site classification; regression tests for formatter idempotence,
  `using` cleanup paths, package visibility, reserved `std` package paths,
  and the `T027` `using` diagnostic split.
- [x] Release candidate preparation: chose to stay in the `0.x` series rather
  than start the `1.0.0` compatibility promise; `0.5.0` shipped 2026-07-02 and
  `0.6.0` shipped 2026-07-03 through the release workflow (see `RELEASING.md`
  for the process).
- [x] Structured task groups Phase 1: `group` / `spawn` syntax with `T030` /
  `E013` diagnostics, the internal `Task[T]` handle, `std::task` with `join`
  and `spawn_map`, artifact and conformance coverage, and benchmark-health
  checks. Design decisions and semantics are recorded in
  [spec/007-concurrency-draft.md](./spec/007-concurrency-draft.md) section 5.

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

These are known implementation gaps or design extensions. Parking an item here
does not mean it is planned. It means the current implementation is allowed to
ship without it, and the feature should not be promoted only because it would
make a few examples shorter.

Move a parked item into active work only when all of these are true:

- real Muga programs show repeated readability, correctness, or workflow pain
  that the current explicit form does not handle well
- the proposed solution preserves Muga's function-centered, value-oriented,
  non-overloaded source model
- the feature has a small grammar, clear diagnostics, package-interface rules,
  and focused Rust tests before implementation begins
- the feature does not introduce protocol/trait/typeclass-style behavior,
  class-style dispatch, implicit effects, or multiple competing spellings for
  the same operation

- [ ] public-signature inference for `pub fn`; there is no active plan to add
  this. Keep explicit public signatures as the default because they stabilize
  package interfaces, docs, API diffs, and artifact-backed checking.
- [ ] project-mode artifact-root configuration and full incremental package
  artifact reuse; revisit after real projects show repeated build pain.
- [ ] structural equality, `List.contains`, structural `assert_eq`,
  `Map.entries`, `Set[T]`, arbitrary `Map` key types, map literals, and broad
  collection APIs; not queued. Revisit only with an explicit equality/hash
  design that does not introduce behavior-conformance systems.
- [ ] broader JSON/config schema targets such as generic records, generic
  enums, nested `Option[Option[T]]`, non-string map keys, and record-level,
  cross-field, or user-defined validation beyond the implemented narrow
  field-level `@validate(...)` slice; revisit after the current concrete
  schema slice is exercised.
- [ ] future `expr.try`, `T?`, and `Option`-only optional chaining; revisit
  only if explicit `try`, `Option`, and helper packages become too noisy in
  real code. Do not add them merely as shorter spellings.
- [ ] broad wildcard matching, nested patterns, guards, multi-payload variants,
  and named-field enum variants; revisit only with concrete examples that make
  the current exhaustive `match` form hard to read or easy to get wrong.
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
- [ ] concurrency features beyond implemented Phase 1 structured task groups:
  channels, `select`, timeouts, and the later phases in
  `spec/007-concurrency-draft.md`; the draft is not an implementation queue.
  Before any further concurrency syntax is added, re-confirm whether Muga
  needs syntax at all or whether a standard package abstraction (like
  `task::spawn_map`) is simpler.
- [ ] `pub opaque record` for user-defined hidden record representations; this
  is not a v1 feature. Revisit only after real package APIs need smart
  constructors while hiding ordinary Muga record fields. Keep this separate from
  runtime/compiler-backed `pub opaque type`.

## Documentation Hygiene

- [ ] Keep this file as the roadmap and avoid adding another planning document.
- [ ] Keep detailed language prose in `spec-v1.md` or split `spec/` files.
- [ ] Keep example programs runnable under `samples/`; invalid or
  not-yet-implemented source belongs under `conformance/v1/rejecting/` or
  `spec/snippets/`.
- [ ] When implementation changes a public rule, update the closest spec and
  add a focused Rust test in the same change.
- [ ] When adding a public diagnostic code or changing its trigger, update
  `errors.md` and add or adjust a focused test.

## Stability Rules

- [ ] Keep the v1 source model small.
- [ ] Prefer code, samples, conformance fixtures, and Rust tests over long
  design prose.
- [ ] Do not add syntax only to reduce character count. Prefer explicit spelling
  when it improves local readability and keeps the grammar smaller.
- [ ] Keep one canonical spelling for each semantic operation unless real code
  proves that a second spelling makes programs easier to read, not merely
  shorter.
- [ ] Treat draft-only documents as design notes, not implementation queues.
  A draft feature still needs a fresh roadmap promotion decision before work
  starts.
- [ ] Keep `pub opaque type` narrow. It is for public opaque names whose
  representation is compiler/runtime/native/external-backed or intentionally not
  committed to a source-level field layout; ordinary user data should use
  `record` or `enum`, and hidden ordinary records should wait for a deliberate
  `pub opaque record` design.
- [ ] Do not make normal `check` or `run` silently depend on built artifacts.
- [ ] Keep artifact-backed package execution hard-failing without dependency
  source fallback.
- [ ] Keep public `pub fn` signatures explicit by default; do not add inference
  for public package APIs unless the roadmap records stronger evidence than
  annotation convenience.
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
- [x] do not add general `type` declarations or type aliases as alternate
  spellings for `record`, `enum`, or enum-plus-record combinations; keep data
  declarations explicit
- [x] do not add type aliases merely to shorten public API shapes or avoid
  writing explicit `record` / `enum` declarations. This does not remove the
  narrow package-mode `pub opaque type` form, which is not a type alias.
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

Muga shipped `0.5.0`, shipped structured task groups
(`group` / `spawn` / `std::task::join`) as `0.6.0`, and then added
`task::spawn_map` to close the dynamic fan-out gap.
`scripts/v1-release-gate.sh` remains the authoritative readiness command.
The next concrete step is gathering real usage of `task::spawn_map` and the
Phase 1 core before deciding whether to promote Phase 2 (channels) or service
IO work. Channels, `select`, service IO, remote registries, broad collections,
and native backend work stay deferred until real task-group usage justifies
them.
