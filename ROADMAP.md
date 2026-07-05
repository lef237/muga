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

## P0: Pre-v1 Implementation Audit

Completed on 2026-06-05. Do not bump versions, create tags, push, or publish
until an explicit release target is chosen.

- [x] Audit Rust implementation hotspots:
  - [x] parser and formatter round-trip behavior
  - [x] resolver and package visibility rules
  - [x] typechecker rules for records, enums, generics, control flow, and
    standard packages
  - [x] MIR, bytecode, and VM behavior for user-reachable runtime paths
  - [x] artifact loading, package archives, app bundles, and source-free
    execution
  - [x] CLI JSON/text diagnostic contracts
- [x] Classify every production `panic!`, `unreachable!`, `unwrap`, and
  `expect` as either an internal invariant or a user-reachable bug.
- [x] Add focused Rust tests for any discovered behavior gap, even if the code
  already appears correct.
- [x] Add or update runnable Muga samples only where a public v1 workflow lacks
  sample coverage.
- [x] Re-run `cargo test --locked` and `scripts/v1-release-gate.sh`.
- [x] Record the audit result here before returning to release-candidate
  preparation.

Audit notes:

- [x] 2026-06-05: searched `src/`, `tests/`, `samples/`, `spec/`, and top-level
  docs for `TODO`, `FIXME`, `todo!`, `unimplemented!`, and debug output
  leftovers. No production incomplete implementation marker was found.
- [x] 2026-06-05: added a formatter regression test that copies repository
  `samples/` and `conformance/v1`, runs manifest-aware `format_path` over every
  `.muga` file, and verifies the result is idempotent.
- [x] 2026-06-05: added a bytecode regression test proving statement-form
  `using` emits cleanup call sites for `try`, explicit `return`, `break`,
  `continue`, and normal fallthrough paths.
- [x] 2026-06-05: audited package visibility and interface signature
  construction against `src/package.rs`, `src/package_signature.rs`, and
  `src/interface.rs`; added regression coverage proving public APIs cannot
  expose `pkg` types and `pkg` APIs cannot expose module-private types, including
  nested function/generic signature shapes.
- [x] 2026-06-05: audited production `unwrap`/`expect`/`panic`-style sites and
  fixed a user-reachable standard namespace edge by reserving `std` package
  paths for compiler-provided standard packages. Added explicit source,
  manifest, and import tests.
- [x] 2026-06-05: separated `using`'s non-`Result` enclosing-function diagnostic
  from the `try` diagnostic so the user-facing message reports `T027` and names
  `using`.
- [x] 2026-06-05: audited existing artifact/source-free tests for missing,
  stale, wrong-package, dependency-interface-mismatched, and source-free bundle
  execution paths. Existing coverage already proves artifact-backed workflows do
  not silently fall back to dependency source bodies.
- [x] 2026-06-05: ran `cargo fmt --check`, `git diff --check`,
  `cargo test --locked`, and `scripts/v1-release-gate.sh` after the first audit
  slice.
- [x] 2026-06-05: final audit verification passed with `cargo fmt --check`,
  `git diff --check`, `scripts/clippy-check.sh`, `cargo test --locked`, and
  `scripts/v1-release-gate.sh`.

## P0: Release Candidate Preparation

Do not bump versions, create tags, push, or publish without an explicit release
target decision. This starts only after the pre-v1 implementation audit is
complete.

- [x] Decide the next release target:
  - [ ] `1.0.0-rc.1` if this is the first v1 release candidate.
  - [ ] `1.0.0` only if the v1 compatibility promise should begin now.
  - [x] another `0.x` only if this should remain a pre-v1 release: `0.5.0`,
    chosen on 2026-07-02. The v1-scope implementation is complete, but the
    project intentionally stays in `0.x` instead of binding itself to the v1
    milestone now.
- [ ] Confirm the working tree contains only intended release changes.
- [ ] Run `scripts/v1-release-gate.sh` from the chosen-version
  release-candidate tree.
- [ ] Bump `Cargo.toml` and `Cargo.lock` to the chosen version.
- [ ] Commit the version bump.
- [ ] Run `scripts/v1-release-gate.sh --with-publish-dry-run`.
- [ ] Create and verify the annotated tag for the chosen version.
- [ ] Push `main` and the tag.
- [ ] Verify the GitHub Actions release workflow, crates.io version, and GitHub
  Release.

## P1: Structured Task Groups

Promoted on 2026-07-02: the release definition changed. The completed v1-scope
implementation ships as `0.5.0` instead of a `1.0.0` release candidate, and
structured task groups are the next implementation work after `0.5.0` ships.

- [x] Reconcile `spec/007-concurrency-draft.md` with the implemented value,
  package, handle, and artifact model. Section 5 is now the implemented
  Phase 1 specification; channels and later phases stay drafts.
- [x] Decide whether the first task API is syntax (`group` / `spawn` / `join`)
  or a standard package abstraction. Decision: `group` and `spawn` are
  syntax because the lifetime rule "child tasks may not outlive their group"
  is enforced by lexical structure; a package-level scope value could escape
  and would need escape analysis. `join` is an ordinary `std::task` package
  function (`task::join(handle)` / `handle.task::join()`), not a prelude
  name, because Muga rejects shadowing and a new prelude name would collide
  with existing user functions and `path::join`.
- [x] Define task lifetime rules: child tasks may not outlive their group.
  `group { ... }` is an expression scope; leaving it means all children
  completed.
- [x] Define failure propagation and cancellation behavior. A child runtime
  failure propagates out of the enclosing `group`; siblings not yet spawned
  never start. Execution order is implementation-defined; the reference VM
  is deterministic and runs each child to completion at its spawn site.
- [x] Define capture rules for immutable values, mutable bindings, and
  runtime-backed handles. Immutable reads are allowed; `mut` references
  across the `spawn` boundary are rejected (`E013`), including reads;
  runtime-backed handles are allowed under deterministic execution and must
  be revisited before parallel execution.
- [x] Define timeout boundaries without promising async socket IO. Phase 1
  ships no timeout API; time-based cancellation waits for the IO/runtime
  integration path.
- [x] Add AST/parser support only after the type and runtime behavior are
  settled. `group` is an expression with a value-block body; `spawn` parses
  at the prefix `try` level and accepts `spawn group { ... }` directly.
- [x] Add typed HIR, MIR/bytecode, and VM support. Bytecode gains a
  `WrapTask` instruction with `.mgb` encode/decode/validation support, and
  `.mgi` signatures serialize `Task[T]` for `std::task::join`.
- [x] Add conformance fixtures for accepted and rejected task usage:
  `conformance/v1/valid/control/task_group_spawn.muga` plus rejecting
  fixtures for `T030`, `T013`, and `E013`.
- [x] Add benchmark-health checks only as local health measurements, not
  public performance claims: `runtime.std-task` in the representative
  runtime health check.

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
  enums, nested `Option[Option[T]]`, non-string map keys, and validation
  attributes; revisit after the current concrete schema slice is exercised.
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
- [ ] concurrency syntax from `spec/007-concurrency-draft.md`; the draft is not
  an implementation queue. Before any task syntax is added, re-confirm whether
  Muga needs syntax at all or whether a standard package abstraction is simpler.
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

Muga shipped `0.5.0` and then implemented structured task groups
(`group` / `spawn` / `std::task::join`) as the first post-`0.5.0` slice.
`scripts/v1-release-gate.sh` remains the authoritative readiness command.
The next concrete step is choosing the release target for the task-groups
slice. Channels, `select`, service IO, remote registries, broad collections,
and native backend work stay deferred until real task-group usage justifies
them.
