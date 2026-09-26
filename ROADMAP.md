# Muga Roadmap

This file is the single working checklist for implementation order. Language
rules belong in [LANGUAGE.md](./LANGUAGE.md) and [spec/](./spec/). Executable
behavior should be proved with Rust tests, conformance fixtures, and runnable
Muga samples.

## Resume Cursor

- [ ] **NOW:** work "Phase 0: Measurement Baseline". Order: the runtime
  benchmark suite with Go and Rust baselines (Phase 1's gate depends on it),
  then the mini-git AI-authoring benchmark port, then edit-check loop latency,
  then the diagnostic severity model. Diagnostics remain error-only
  (`severity: "error"` is fixed in `src/diagnostic.rs`).
- [ ] **NEXT:** "Phase 1: Native Backend Feasibility".
- [ ] **NEXT:** decide a `0.6.1` release independently of the phase work,
  following `RELEASING.md`. `v0.6.0` is the last published release; the
  compatibility work, the lint slice, and the formatter change are unreleased.

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
- [x] current core language surface: immutable-by-default bindings, `mut`, records,
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
- [x] `muga lint` with the `S001` chained-call style lint, `--fix`, and
  `// muga-lint: allow-next-line <codes>` suppressions
- [x] strict `muga.toml` validation, a required `[package] language_revision`,
  `muga.lock` `muga_version` enforcement, and crash-safe compiler-owned writes
  (see spec/006-packages.md)

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
- [x] Muga has one semantics and two backends: the reference VM serves the
  development loop (`check`, `run`, `test`) and Rust generation serves
  release builds. Neither backend is a separate semantics engine; running
  conformance on both proves they agree.

## Direction

Recorded on 2026-09-26. The one-sentence goal is:

> Muga is a quiet, statically typed language that aims to run faster than Go
> while staying easy for both people and AI coding agents to read and write.

"Aims" is deliberate: no performance claim is published until the Phase 0
benchmarks show it.

### Direction Metrics

Every phase below is judged against these three metrics. A change that makes
one of them worse needs an explicit reason recorded with it.

- **Performance:** on the Phase 0 runtime benchmark suite, native Muga release
  builds reach at least Go's median throughput, and a release build is one
  self-contained executable.
- **AI authoring:** on the Phase 0 AI-authoring benchmark, an AI coding agent
  writing Muga needs no more time and cost than it needs for Go, with a pass
  rate at least as high.
- **Human readability:** one canonical spelling per operation, and code that
  can be understood by reading it locally. This is not reduced to a number;
  every syntax or API decision records how it affects local reading.

### Evidence Behind The Direction

- The mini-git benchmark
  ([mame/ai-coding-lang-bench](https://github.com/mame/ai-coding-lang-bench),
  March 2026, 13 languages, 20 runs each) measured Claude Code implementing a
  small Git clone. Its likely speed drivers were familiarity from training
  data, type-system complexity, and perceived difficulty such as Rust
  ownership; the only failed runs were in Rust and Haskell.
- A follow-up with benchmark-owned scaffolds
  ([kmizu/another-ai-coding-lang-bench](https://github.com/kmizu/another-ai-coding-lang-bench))
  found Go faster than every dynamic language it tested and concluded that
  compiler throughput and build-cycle latency matter more than the
  static/dynamic split.
- Spinel ([matz/spinel](https://github.com/matz/spinel)) compiles a Ruby subset
  ahead of time to C through whole-program type inference and reports large
  speedups over YJIT. Readable source with native performance is achievable
  today; Muga starts from static types and has no dynamic features to
  restrict, which makes native code generation simpler.

These are small-scale, time-bound measurements. Muga treats them as reasons to
measure its own behavior in Phase 0, not as settled conclusions.

### Bets

- **Native performance through Rust generation.** Release builds emit Rust and
  compile it with rustc, so LLVM optimizes the result and the existing Rust
  runtime code can be reused. Muga's source model (value semantics, no
  source-level references, no traits, 64-bit `Int`) lets generated Rust avoid
  lifetimes entirely, so generated code must always compile: a rustc error is
  a Muga compiler bug, never a user-facing diagnostic.
- **A fast feedback loop through the reference VM.** `check`, `run`, and `test`
  stay on the fast-starting VM. The slow native build never enters the
  edit-check loop that people and AI agents repeat.
- **AI-authorable by evidence.** Muga has no training-data familiarity, so it
  compensates with a compact reference, diagnostics that translate habits from
  other languages into Muga, and a small standard-library surface. Design
  choices are checked against the AI-authoring benchmark instead of intuition.
- **A readable surface without ownership syntax.** Familiar imperative control
  flow with immutability by default, `Option` / `Result`, prefix `try`, and
  exhaustive `match`; no classes, traits, overloading, implicit conversions,
  shadowing, or source-level references.
- **Machine-consumable tooling as a product surface.** Stable machine-readable
  diagnostics with replacements, editor/agent queries, API diffing, and
  explainable rebuilds remain primary interfaces of the language.
- **A completed structured-concurrency contract.** `group` / `spawn` / `Task`
  with real parallel execution in the native runtime, cancellation, failure
  propagation, capture safety, and cleanup, rather than more concurrency
  surface.

### Non-Goals

- do not add further compilation targets (JavaScript, WebAssembly, C, or
  others) beside the VM and Rust generation; revisit only if Phase 1 shows
  Rust generation cannot meet the performance metric
- do not require the native build for `check`, `run`, or `test`
- do not publish performance claims without repeatable benchmark results
- do not expose ownership, borrowing, or lifetimes in Muga source to make
  generated Rust faster; performance work belongs in value representation and
  the compiler
- do not build or operate a remote package registry before local archive
  identity, lockfile behavior, and install inventory are stable
- do not compete on type-system expressiveness; the "Not Planned" list stays
  authoritative

## Versioning

Muga improves through small `0.x` releases; `RELEASING.md` owns the version
numbering policy. `scripts/release-gate.sh` is the minimum quality gate for
every release, not evidence that foundational design is complete.

### 1.0 Readiness Criteria

Version `1.0.0` names a maturity judgment, not completion of the ordinary
roadmap. All of these are required before the first `1.0.0` release candidate:

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
  referenced without relying on implementation archaeology, and all current
  documents describe one consistent contract.
- [ ] Low post-1.0 redesign risk: known remaining work can be delivered mostly
  as compatible fixes, performance improvements, or optional additions without
  reopening the language's foundations.

Meeting these criteria permits a `1.0.0-rc.N`; it does not require every parked
idea to be implemented. Until then Muga simply continues its normal `0.x`
release sequence.

## Phase 0: Measurement Baseline

Phase 0 creates the evidence the direction metrics depend on. Record results as
machine-readable baselines so later phases compare against them.

- [ ] Build a repeatable runtime benchmark suite with Go and Rust reference
  implementations of the same programs. Cover CPU-bound work (integer loops,
  recursion, records, enums with `match`), `String` / `List` / `Map` heavy work,
  and CLI-realistic work (JSON processing, directory traversal, text
  processing). Record warm-up, iterations, median and tail times, allocations,
  and peak memory; emit machine-readable results; never use noisy wall-clock
  thresholds as correctness tests. This replaces the one-shot millisecond health
  checks.
- [ ] Port the mini-git AI-authoring benchmark to Muga with the same
  specification and tests as mame/ai-coding-lang-bench, starting from a `muga
  new` scaffold so setup cost is measured separately from agent time. Measure
  pass rate, agent time, and cost with no Muga reference, and again with the
  compact reference once Phase 2 provides it. Run Go under the same harness as
  the comparison baseline, and keep transcripts: they are the input for the
  Phase 2 diagnostics.
- [ ] Measure the edit-check loop: `muga check`, `muga run`, and `muga test`
  latency on small and medium projects, cold and warm.
- [ ] Add a first-class diagnostic severity model and lint pipeline. Start with
  unused imports, bindings, and parameters, unreachable code, and discarded
  `Result` values; define command-line and machine-readable allow/warn/deny
  behavior before stabilizing it. Warnings are feedback that AI agents act on,
  and finished items wait on this model: the `errors.md` lint contract,
  `muga.lock` `muga_version` warnings for accepted-but-different versions, and
  the typo-created-binding decision in Phase 2.

## Phase 1: Native Backend Feasibility

Phase 1 answers the largest technical risk first: can Muga's source model reach
Go-level performance through Rust generation?

- [ ] Choose the emission input (typed HIR or MIR) and record why. Introduce a
  control-flow-oriented MIR only if that choice needs it.
- [ ] Emit Rust for a subset: `Int`, `Bool`, functions, `if`, `while`, records,
  enums, and `match`. Compile the generated crate with rustc and run it against
  the same conformance fixtures as the VM for that subset.
- [ ] Decide the runtime value representation shared by both backends: unboxed
  scalars, reference counting with copy-on-write for aggregates (in-place update
  when uniquely owned), and monomorphized generics. Record the decision in
  spec/011-value-semantics.md.
- [ ] Compare the subset against Go on the Phase 0 CPU-bound benchmarks.
- [ ] Gate: continue to Phase 3 when the subset reaches Go's median on those
  benchmarks, or when the measured gap has a concrete, recorded explanation and
  plan. If Rust generation cannot close the gap, record the evidence and
  re-evaluate C generation or Cranelift before Phase 3.

## Phase 2: AI Authoring

Phase 2 may proceed in parallel with Phase 1 where the work is independent.
Re-run the AI-authoring benchmark after each item and record the result.

- [ ] Add a compact, versioned language reference for AI agents and people
  (working name `muga guide`), printed by the `muga` binary so it always matches
  the installed compiler. It covers the whole syntax, the core standard-library
  APIs, and canonical patterns, in a size that fits in an agent's context.
- [ ] Add diagnostics that recognize habits from other languages and suggest the
  Muga spelling with machine-applicable replacements, such as postfix `?`
  (prefix `try`), `let`, `null` / `nil`, `class`, `impl`, and `if let`. Build
  the list from Phase 0 benchmark transcripts rather than guessing.
- [ ] Reduce the standard-library and CLI surface to one canonical spelling per
  operation:
  - [ ] Choose one canonical filesystem API over `path::Path`; remove the
    duplicate String-path operations and `_path` suffixes before stabilizing the
    API unless real programs prove both forms have distinct value.
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
- [ ] Decide the `x = e` binding/update rule with evidence. After ordinary
  unused warnings exist, test typo-created bindings in real programs and in
  AI-authoring benchmark transcripts. Add a narrowly scoped similar-name warning
  only if recurring mistakes escape those warnings; it is not a baseline
  requirement. Reopen explicit update syntax only if diagnostics still leave
  material correctness problems. Do not reserve `set` as the leading candidate:
  it is visually close to a future `Set[T]` type and overlaps with collection
  `.set(...)` vocabulary.

## Phase 3: Native Backend Completion

- [ ] Split the standard-package runtime out of the VM's `Value` into a typed
  runtime crate (working name `muga-rt`) used by both backends.
- [ ] Extend Rust generation to the whole current language: closures, generics,
  `Option` / `Result` / `try`, `String`, `List`, `Map`, `Bytes`, packages and
  artifacts, `using`, and the standard packages.
- [ ] Run the full conformance suite on both backends in CI; any divergence is a
  bug in one of them.
- [ ] Add `muga build --release` producing one self-contained executable, with
  the source-free app bundle model as the content source. Report a missing Rust
  toolchain as a stable, actionable diagnostic.
- [ ] Implement the Phase 1 representation: shared immutable or copy-on-write
  storage for `String`, `Bytes`, `List`, `Map`, records, and enum payloads.
  Preserve source-level value semantics while measuring and eliminating
  field-access, lookup, argument, and update clones.
- [ ] Replace the VM's insertion-ordered linear `Map` lookup with an indexed
  representation that preserves deterministic iteration while making normal
  `get`, `contains`, `insert`, and `remove` scale near constant time.
- [ ] Give `group` / `spawn` real parallel execution in the native runtime and
  resolve the concurrency stability gate under "Ongoing: Language And
  Standard-Library Maturity".
- [ ] Measure the full benchmark suite against Go and record whether the
  performance metric is met.
- [ ] After aggregate costs are measured, reduce front-end allocation where it
  matters, including reconsidering the lexer's whole-source `Vec[char]` copy and
  repeated runtime type/field strings. Do not prioritize these changes ahead of
  measured aggregate-copy and map costs.

## Phase 4: Trustworthy Delegation

Code written by an AI agent should be reviewable from its signatures and
runnable with bounded authority.

- [ ] Design how effects such as filesystem, process, environment, and network
  access become visible in function signatures or package interfaces, without
  traits, implicit effects, or expensive global analysis. Prototype against the
  standard packages before committing syntax.
- [ ] Add run-time permissions for `muga run` and built executables (for example
  `--allow fs`), keeping today's unrestricted behavior as the default until the
  design is settled.

## Ongoing: Diagnostics, Robustness, And Portability

- [ ] Add fuzz targets for the lexer, parser, manifest and lockfile readers,
  persisted package artifacts, and `.mgp` / `.mga` archive readers. Every
  arbitrary input must either produce a bounded result or a diagnostic, never an
  uncontrolled panic or unbounded allocation.
- [ ] Define and test nesting, recursion, graph, file-count, and byte-size
  limits at untrusted input boundaries. Limit failures must use stable,
  actionable diagnostics.
- [ ] Define the supported host matrix and run CI on at least Linux, macOS, and
  Windows for path, process, filesystem, archive, bundle, install/uninstall,
  line-ending, and artifact reproducibility behavior.
- [ ] Stabilize the CLI process contract: exit status classes, stdout/stderr
  ownership in text and JSON modes, broken-pipe handling, and Ctrl-C behavior
  including cleanup of child processes, tasks, and partial output.
- [ ] Decide whether ordinary source rewrites (`muga fmt`, `muga lint --fix`)
  should join the crash-safe write protocol. They are user-owned rather than
  compiler-owned state, and atomic replacement would replace a symlinked
  source file with a regular file.
- [ ] Version the `muga.toml` schema, narrow `PK014` spans to the offending key
  or value, and record `language_revision` in artifacts and `muga.lock` as
  semantic interpretation metadata.
- [ ] Sweep temporary files abandoned by a killed process once they can be
  told apart from a concurrent build's.
- [ ] Harden install inventory UX and diagnostics around app bundle ownership,
  and add source-free bundle smoke cases for std packages that use host
  effects.

## Ongoing: Language And Standard-Library Maturity

These items were promoted by the 2026-07-12 language-surface audit and proceed
alongside the phases. Promotion means Muga should evaluate or implement them as
current work; it does not pre-approve unreviewed syntax or tie the decision to a
future version label.

- [ ] Implement `Float64` (decided 2026-09-26: Muga is a general-purpose
  language and competes with Go). Specify and implement the type, literals,
  arithmetic, conversions, formatting, JSON numbers, `NaN`/infinity behavior,
  equality, and hashing. Keep decimal money arithmetic as a separate later type
  or package rather than an implicit numeric mode. Land it before the Phase 3
  benchmark measurement so numeric benchmarks can be written.
- [ ] Add allocation-free integer ranges that `for` can consume without first
  constructing a `List[Int]`. Prefer a small `range(start, end)` value before
  committing range punctuation or a general iterator/protocol system.
- [ ] Fill the small eager collection core with evidence-backed helpers such as
  `find`, `position`, `reverse`, `concat`, `flat_map`, `take`, `drop`, and
  comparator-based `sort_by`; add a concrete public `map::Entry[K, V]` shape and
  `map::entries` without waiting for structural equality.
- [ ] Expand `Bytes` into a usable binary foundation: UTF-8 conversion, list
  conversion, slicing, concatenation, hex and Base64 codecs, an efficient
  builder, and binary file handles. Keep broad cryptography separate.
- [ ] Add a minimal operational time and randomness layer: `Duration`, a
  monotonic clock, sleep, checked duration arithmetic, and OS-backed secure
  random bytes. Defer calendar/time-zone policy and seeded PRNG design until
  their contracts are explicit.
- [ ] Decide whether opt-in compiler-derived equality and hashing belong in
  Muga. Any design must avoid a behavior-conformance system, persist
  capabilities in package interfaces, reject unsupported payloads such as
  functions and handles, define `Float64`/`NaN` behavior, and unlock structural
  assertions, `List.contains`, `Set[T]`, and non-scalar map keys only when
  sound.
- [ ] Re-evaluate the blanket prohibition on function-valued record fields. If
  real callback, strategy, parser, validator, or event-handler APIs need them,
  allow storage while keeping invocation explicit through `call(field, args...)`
  or another non-dot-call form.
- [ ] Resolve the stability gate for structured task groups (concurrency
  Phase 1 in spec/007, unrelated to the roadmap phases above). Validate the
  contract with at least one runtime that provides overlapping progress for
  suspended/blocking tasks or actual parallel execution, plus real
  cancellation, failure propagation, capture safety, and resource cleanup.
  Parallel speedup is not required. If the contract is not ready, keep the implementation experimental
  instead of treating `group` / `spawn` / `Task` as stable or deleting the code
  merely to satisfy the gate. The Phase 3 native runtime is the intended
  parallel runtime for this validation.

## Parked Non-Blockers

These are known implementation gaps or design extensions. Parking an item here
does not mean it is planned or permanently deferred. It means
the current `0.x` implementation may keep shipping without it, and the feature
should not be promoted only because it would make a few examples shorter. Real
usage may still show that a parked item, or a different solution to the same
problem, is necessary for the mature language.

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
  revisit after local `.mgp` / `.mga` workflows are stable in real use. Keep
  package identity tied to `.mgp` content hashes.
- [ ] concurrency features beyond the implemented structured task groups:
  channels, `select`, timeouts, detached tasks, and supervision. Before any
  further concurrency syntax is added, re-confirm whether Muga needs syntax at
  all or whether a standard package abstraction (like `task::spawn_map`) is
  simpler.
- [ ] service IO (sockets or minimal HTTP/JSON). Do not start before the
  structured-task-group stability gate is resolved and task lifetime,
  shutdown, and backpressure semantics are explicit; keep resource handles
  opaque and closeable and JSON integration explicit through `std::json`.
- [ ] `pub opaque record` for user-defined hidden record representations; this
  is not in the current language. Revisit only after real package APIs need
  smart constructors while hiding ordinary Muga record fields. Keep this
  separate from runtime/compiler-backed `pub opaque type`.

## Documentation Hygiene

- [ ] Keep this file as the roadmap and avoid adding another planning document.
- [ ] Keep the compact current language overview in `LANGUAGE.md` and detailed
  prose in the split `spec/` files.
- [ ] Keep example programs runnable under `samples/`; invalid or
  not-yet-implemented source belongs under `conformance/current/rejecting/` or
  `spec/snippets/`.
- [ ] When implementation changes a public rule, update the closest spec and
  add a focused Rust test in the same change.
- [ ] When adding a public diagnostic code or changing its trigger, update
  `errors.md` and add or adjust a focused test.

## Stability Rules

- [ ] Keep the current source model small.
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
