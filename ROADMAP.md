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
- [ ] **NOW:** establish representative benchmark and real-program workloads,
  then use them to audit standard-library duplication, aggregate copying,
  `Map` scaling, and the actual value of Phase 1 task syntax before expanding
  the language or starting Phase 2 concurrency.

Baseline checks recorded during the 2026-06-05 implementation audit:

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

## Versioning And V1 Direction

Muga is not on a short release-candidate path. The `v1` specifications and
conformance directory describe the evolving contract that the project may
eventually stabilize as `1.0.0`; they do not mean that the current feature set
is frozen or mature enough for that promise.

- [ ] Keep releases in the current `0.x` line and increment only PATCH for
  each small release (`0.6.0` to `0.6.1`, then `0.6.2`, and so on), including
  pre-v1 feature and breaking releases. See [RELEASING.md](./RELEASING.md).
- [ ] Continue testing the language on sustained, non-trivial programs before
  treating the v1 candidate surface as closed.
- [ ] Allow the candidate v1 language, standard packages, tools, and formats to
  change when real usage exposes a foundational gap. Keep the specs,
  diagnostics, samples, and conformance fixtures aligned with each change.
- [ ] Treat `scripts/v1-release-gate.sh` as the minimum quality gate for every
  release, not as proof that the project is ready to call itself v1.
- [ ] Start `1.0.0-rc.N` only after all v1 maturity criteria below are met and
  the expected post-v1 work is compatible maintenance rather than foundational
  language or ecosystem work.

### V1 Maturity Criteria

All of these are required before the first v1 release candidate:

- [ ] Real-world validation: multiple sustained programs exercise the language,
  standard packages, dependency model, artifacts, and deployment workflows;
  remaining gaps are understood rather than hidden by small samples alone.
- [ ] Language stability: core semantics, type behavior, error handling,
  concurrency, resource lifetime, package boundaries, and compatibility rules
  are coherent and no known foundational redesign is queued.
- [ ] Implementation reliability: the compiler and reference runtime are robust
  against invalid input and ordinary host failures, with regression,
  conformance, stress, and negative-path coverage appropriate for a mature
  language implementation.
- [ ] Tooling and ecosystem completeness: formatting, testing, documentation,
  editor-facing queries, build artifacts, dependency locking, distribution,
  installation, and upgrade workflows are dependable for real projects.
- [ ] Operational quality: supported platforms, performance expectations,
  compatibility policy, release process, and recovery procedures are explicit
  and have been exercised.
- [ ] Documentation quality: the language and standard packages are taught and
  referenced without relying on implementation archaeology, and all v1
  documents describe one consistent contract.
- [ ] Low post-v1 redesign risk: known remaining work can be delivered mostly
  as compatible fixes, performance improvements, or optional additions without
  reopening the language's foundations.

## Pre-v1 P0: Compatibility And Durability

These are foundational requirements discovered during the 2026-07-12 maturity
audit. They take priority over expanding the language surface because they
prevent silent misconfiguration, accidental reinterpretation, and corrupted
build state.

- [ ] Make `muga.toml` validation strict: reject unknown sections and fields,
  duplicate fields, and malformed non-comment lines with source locations and
  actionable diagnostics instead of silently ignoring them.
- [ ] Design a source-compatibility declaration for manifest projects before
  pre-v1 changes accumulate. Decide whether this is a language revision,
  edition, compiler compatibility range, or a combination, and define how an
  older project is diagnosed or migrated by a newer compiler.
- [ ] Enforce the recorded `muga_version` compatibility policy when reading an
  existing `muga.lock`. The current parser validates only that the field exists
  and is a string; it does not compare it with the running compiler.
- [ ] Make compiler-owned writes crash-safe. Write lockfiles, `.mgi`, `.mgb`,
  `.mgc`, archives, bundle metadata, and installation ownership metadata to a
  sibling temporary file, flush as required by the durability policy, and
  atomically replace the destination only after successful serialization.
- [ ] Add interruption and failure-path tests proving that a failed write does
  not destroy the last valid lockfile, artifact, archive, or installation
  record.

## Pre-v1 P1: Diagnostics, Robustness, And Portability

- [ ] Add a first-class diagnostic severity model and lint pipeline. Start with
  unused imports, bindings, and parameters, unreachable code, and discarded
  `Result` values; define command-line and machine-readable allow/warn/deny
  behavior before stabilizing it.
- [ ] Add fuzz targets for the lexer, parser, manifest and lockfile readers,
  persisted package artifacts, and `.mgp` / `.mga` archive readers. Every
  arbitrary input must either produce a bounded result or a diagnostic, never
  an uncontrolled panic or unbounded allocation.
- [ ] Define and test nesting, recursion, graph, file-count, and byte-size
  limits at untrusted input boundaries. Limit failures must use stable,
  actionable diagnostics.
- [ ] Define the supported host matrix and run CI on at least Linux, macOS, and
  Windows for path, process, filesystem, archive, bundle, install/uninstall,
  line-ending, and artifact reproducibility behavior.
- [ ] Stabilize the CLI process contract: exit status classes, stdout/stderr
  ownership in text and JSON modes, broken-pipe handling, and Ctrl-C behavior
  including cleanup of child processes, tasks, and partial output.

## Pre-v1 P1: Language And Standard-Library Maturity

These items were promoted by the 2026-07-12 language-surface audit. Promotion
means Muga must implement them or record an evidence-backed decision not to
include them before v1; it does not pre-approve unreviewed syntax.

- [ ] Decide the numeric scope for v1. If Muga remains a general-purpose
  language, specify and implement an explicit `Float64` type, literals,
  arithmetic, conversions, formatting, JSON numbers, `NaN`/infinity behavior,
  equality, and hashing. Keep decimal money arithmetic as a separate later
  type or package rather than an implicit numeric mode.
- [ ] Add allocation-free integer ranges that `for` can consume without first
  constructing a `List[Int]`. Prefer a small `range(start, end)` value before
  committing range punctuation or a general iterator/protocol system.
- [ ] Fill the small eager collection core with evidence-backed helpers such
  as `find`, `position`, `reverse`, `concat`, `flat_map`, `take`, `drop`, and
  comparator-based `sort_by`; add a concrete public `map::Entry[K, V]` shape
  and `map::entries` without waiting for structural equality.
- [ ] Expand `Bytes` into a usable binary foundation: UTF-8 conversion,
  list conversion, slicing, concatenation, hex and Base64 codecs, an efficient
  builder, and binary file handles. Keep broad cryptography separate.
- [ ] Add a minimal operational time and randomness layer: `Duration`, a
  monotonic clock, sleep, checked duration arithmetic, and OS-backed secure
  random bytes. Defer calendar/time-zone policy and seeded PRNG design until
  their contracts are explicit.
- [ ] Decide whether opt-in compiler-derived equality and hashing belong in v1.
  Any design must avoid a behavior-conformance system, persist capabilities in
  package interfaces, reject unsupported payloads such as functions and
  handles, define `Float64`/`NaN` behavior, and unlock structural assertions,
  `List.contains`, `Set[T]`, and non-scalar map keys only when sound.
- [ ] After ordinary unused warnings exist, test typo-created bindings in real
  programs. Add a narrowly scoped similar-name warning only if recurring
  mistakes escape those warnings; it is not a baseline v1 requirement. Reopen
  explicit update syntax only if diagnostics still leave material correctness
  problems. Do not reserve `set` as the leading candidate: it is visually close
  to a future `Set[T]` type and overlaps with collection `.set(...)` vocabulary.
- [ ] Re-evaluate the blanket prohibition on function-valued record fields.
  If real callback, strategy, parser, validator, or event-handler APIs need
  them, allow storage while keeping invocation explicit through `call(field,
  args...)` or another non-dot-call form.
- [ ] Resolve the Phase 1 concurrency admission gate before v1 stabilization.
  Validate the contract with at least one runtime that provides overlapping
  progress for suspended/blocking tasks or actual parallel execution, plus real
  cancellation, failure propagation, capture safety, and resource cleanup.
  Parallel speedup is not required. If the contract is not ready, keep the
  implementation experimental but defer `group` / `spawn` / `Task` from the
  stable v1 contract instead of deleting the code merely to satisfy the gate.

## Pre-v1 P1: Runtime Performance Foundations

- [ ] Replace one-shot millisecond health checks with repeatable benchmark
  scenarios covering warm-up, multiple iterations, median/tail latency,
  allocations, peak memory, compiler stages, cold/warm package builds, VM
  instruction throughput, and large `String` / `List` / `Map` / record values.
  Emit machine-readable results suitable for comparing releases without using
  noisy wall-clock thresholds as correctness tests.
- [ ] Replace the VM's insertion-ordered linear `Map` lookup with an indexed
  representation that preserves deterministic iteration while making normal
  `get`, `contains`, `insert`, and `remove` scale near constant time.
- [ ] Introduce shared immutable or copy-on-write storage for `String`, `Bytes`,
  `List`, `Map`, records, and enum payloads. Preserve source-level value
  semantics while measuring and eliminating field-access, lookup, argument,
  and update clones.
- [ ] After aggregate costs are measured, reduce front-end allocation where it
  matters, including reconsidering the lexer's whole-source `Vec[char]` copy
  and repeated runtime type/field strings. Do not prioritize these changes
  ahead of measured aggregate-copy and map costs.

## Pre-v1 P1: API Surface Reduction

- [ ] Choose one canonical filesystem API over `path::Path`; remove the
  duplicate String-path operations and `_path` suffixes before v1 unless real
  programs prove both forms have distinct value.
- [ ] Make typed schema-driven `std::cli` parsing the primary API. Move manual
  `positional_*`, `option_*`, and flag scanning behind a clearly low-level
  namespace or remove them when usage evidence shows they are redundant.
- [ ] Reduce the combinatorial `std::json` accessor matrix. Keep parse/encode,
  typed `decode[T]` / conversion, and a small composable dynamic `Value`
  traversal core; remove convenience combinations that duplicate those paths.
- [ ] Consolidate public artifact commands around `muga build`. Group expert
  interface/cache/artifact emission under one clearly advanced namespace or
  mark it unstable instead of stabilizing several overlapping top-level
  commands.

## Completed Milestones

Finished work is summarized here. Full checklists, audit notes, and decision
logs live in git history; the resulting rules live in the specs, `errors.md`,
and `RELEASING.md`.

- [x] `std::process` (the last capability under the previous near-term v1
  plan): narrow
  recoverable process execution through explicit `Options` / `Output` records,
  nonzero child exits captured as `Result::Ok(Output)`, no shell
  interpolation, `path::Path` cwd, explicit env overrides, and source-free
  bundle coverage.
- [x] Previous v1 release-hardening pass: aligned the then-current
  `spec-v1.md` boundary with the implementation, audited unfinished work, added
  template/sample/bundle test coverage, and established the release-quality
  gate. This pass is historical evidence, not a declaration of v1 readiness or
  a permanent feature freeze.
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

## Maturity Track P2: Service IO

Do not stabilize service IO before the Phase 1 concurrency admission gate is
resolved and task lifetime, shutdown, and backpressure semantics are explicit.
Focused IO prototypes may still be used to validate scheduler suspension,
cancellation, and cleanup behavior before either surface is committed.

- [ ] Choose the first service IO target: sockets or minimal HTTP/JSON.
- [ ] Keep resource handles opaque and closeable.
- [ ] Define shutdown behavior before exposing listeners or streams.
- [ ] Define backpressure behavior before exposing streaming request/response
  APIs.
- [ ] Keep JSON integration explicit through `std::json` schemas.
- [ ] Prove source, built-artifact, and source-free bundle execution.

## Maturity Track P2: Backend Performance Path

Backend work follows the pre-v1 measurement, representation, and API work
above. Performance claims still require evidence.

- [ ] Introduce control-flow-oriented MIR only after the VM and artifact tests
  show the current bytecode path is the bottleneck.
- [ ] Keep native backend work deferred until MIR, package artifacts, and
  runtime representation have measurable pressure.

## Maturity Track P2: Distribution Path

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
does not mean it is planned or permanently deferred until after v1. It means
the current `0.x` implementation may keep shipping without it, and the feature
should not be promoted only because it would make a few examples shorter. Real
usage may still show that a parked item, or a different solution to the same
problem, is necessary for a mature v1.

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
- [ ] `Set[T]`, arbitrary `Map` key types, map literals, and structural
  collection operations remain parked until the promoted equality/hash and
  range/collection decisions establish a sound smaller foundation.
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
- [ ] broad cryptography, service runtime APIs, and async IO remain parked;
  the minimal Bytes codecs, builder, and binary file-handle foundation are
  promoted above and should not imply a broad crypto or streaming framework.
- [ ] URL/Git/registry dependencies, remote fetching, publishing workflows,
  package signing, SBOMs, and full published-package lockfile enforcement;
  revisit after local `.mgp` / `.mga` workflows are stable in real use.
- [ ] control-flow-oriented MIR and a native backend remain parked until the
  promoted benchmark and runtime-representation work shows the reference VM or
  current bytecode is the limiting factor.
- [ ] concurrency features beyond implemented Phase 1 structured task groups:
  channels, `select`, timeouts, and the later phases in
  `spec/007-concurrency-draft.md`; the draft is not an implementation queue.
  Before any further concurrency syntax is added, re-confirm whether Muga
  needs syntax at all or whether a standard package abstraction (like
  `task::spawn_map`) is simpler.
- [ ] `pub opaque record` for user-defined hidden record representations; this
  is not in the current v1 candidate. Revisit only after real package APIs need
  smart constructors while hiding ordinary Muga record fields. Keep this
  separate from runtime/compiler-backed `pub opaque type`.

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
`task::spawn_map` to close the dynamic fan-out gap. The next maturity work is
to establish representative benchmarks and real-program workloads, reduce
duplicated standard-library and CLI surfaces, improve `Map` and aggregate
representations, and make evidence-backed decisions on numeric scope, ranges,
equality/hash, and whether task syntax is ready to stabilize in v1 or should
remain experimental outside the v1 contract.
`scripts/v1-release-gate.sh` remains the baseline release-quality command, but
passing it does not by itself establish v1 readiness.
Channels, `select`, service IO, remote registries, broad collection systems,
and native backend work stay deferred until those foundations justify them.
