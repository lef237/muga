# Practical Language Readiness

Status: design and prioritization note. This is not a language specification.

Purpose: record what Muga still needs before it feels practical for real programs, and record what should stay out of the language so implementation pressure does not pull Muga away from its core direction. For the cross-cutting strategic phase sequence, measurement stance, and context-free resume guidance, see [docs/strategy-and-implementation-plan.md](strategy-and-implementation-plan.md).

Read this after [ROADMAP.md](../ROADMAP.md). The roadmap remains the source of
truth for the active implementation slice. The current implementation policy is
Core Capability Acceleration: keep Muga's source model small, but prioritize
thin end-to-end practical spines such as `std::process`, structured task
groups, service IO, performance foundations, and distribution trust before more
minor polish.

## Baseline

Current Muga is already computationally expressive enough for ordinary algorithms:

- functions, anonymous functions, closures, direct recursion, and mutual recursion
- `if`, `while`, final-expression function bodies, and local `mut`
- records, value-returning updates, List, Map, Option, Result, enum, exhaustive match with payload discard `_` inside variant patterns, and `try expr` `Result` propagation
- package/module boundaries, visibility, package interfaces, and explicit artifact workflows

That is enough to treat the language core as computationally complete in the ordinary abstract sense. It is not enough to call the language broadly practical yet. The missing work is mostly around reusable abstraction, standard-library surface, IO/resource effects, project/dependency workflow, performance, API/schema tooling, and service runtime behavior.

## Practical North Star

Muga should grow as a small, readable, compiler-first application language whose ordinary code stays statically understandable.

Preserve this spine:

- static types with local inference instead of broad dynamic escape hatches
- `Option[T]` for absence and `Result[T, E]` for recoverable effects
- immutable-by-default bindings and source-level value semantics
- records, enums, functions, modules, and package interfaces as the main abstraction tools
- explicit package artifacts so unchanged dependencies can be checked and run without reading implementation bodies
- no runtime metaprogramming, implicit exceptions, implicit nullable values, hidden async suspension, or ordinary source-level references as default mechanisms

The post-v1 practical platform should build on that spine:

- use `.mgi` package interfaces as typed public contracts for packages and services
- add small standard-library slices that return typed values and explicit `Result` errors
- introduce opaque resource handles for files, sockets, timers, processes, and other OS-backed effects
- implement structured concurrency before channels, and channels before `select`/timeouts
- integrate cancellation-aware asynchronous IO only after the task and resource-handle models are stable
- generate external schemas, clients, or service stubs from public Muga signatures instead of from hidden conventions

The acceleration rule is conservative: a core slice may move earlier than older
roadmap notes implied, but it still needs a public contract, explicit `Result`
errors, package/interface or artifact behavior where relevant, runnable samples,
and focused release-readiness evidence. Do not use acceleration as permission
for hidden async suspension, ambient framework conventions, or untyped escape
hatches.

## Cross-Cutting Design Boundaries

These boundaries are not separate implementation phases. They are guardrails for
future memory, resource, concurrency, schema, testing, and service work. Use
them to keep new features aligned with Muga's value-oriented source model,
explicit effects, stable package interfaces, and local readability.

### Runtime memory and resource lifetime

Ordinary Muga values should not expose allocation or deallocation timing.
Source code should not observe whether the implementation uses reference
counting, tracing collection, arenas, stack allocation, internal borrowing, or
copy elision. The implementation is responsible for reclaiming unreachable
ordinary values, and long-running service runtimes must not rely on user-visible
manual memory management to avoid ordinary heap growth.

This leaves room for multiple backend strategies, but the reference and native
runtimes must handle closure environments and internal sharing carefully. If an
implementation strategy can create reference cycles, it must also provide a way
to collect or avoid those cycles.

Resource lifetime is different from ordinary memory lifetime. Files, sockets,
timers, processes, task handles, and similar OS- or runtime-backed values should
have explicit resource APIs and visible lifetime rules. Memory cleanup may be
non-deterministic; resource cleanup should be deterministic where practical.

### Opaque resource capabilities

Opaque resource types should carry small, explicit capability facts that can be
represented in package interfaces. The first model should be narrower than a
general ownership or borrowing system.

Recommended defaults for runtime-backed opaque resources:

- not copyable
- not shareable across tasks
- not sendable across tasks
- not constructible outside the defining package or compiler runtime
- closed only through explicit APIs or lexical cleanup constructs

Capabilities such as copyable, cloneable, sendable, shareable, and closeable
should be opt-in facts on the opaque resource design, not inferred from ordinary
record structure. Consuming operations, such as close or task transfer, should
make later use of the consumed binding an error.

This keeps ordinary source code value-oriented while giving resource APIs enough
structure to prevent double close, use after close, accidental task sharing, and
hidden mutable aliasing.

### Structured cleanup

Muga should prefer a lexical cleanup construct over finalizers or hidden
exception behavior for important resources.

Candidate shape:

```muga
using file = try fs::open(path) {
  try file.write(text)
}
```

Rules to decide before implementation:

- the cleanup operation always runs when the binding was successfully created
- close failures are visible as values, not silently discarded
- behavior is specified when both the block body and cleanup operation fail
- cancellation still runs required cleanup, subject to a clearly documented
  cancellation boundary
- explicit close and lexical cleanup cannot double-close the same resource

The first slice may be compiler-known and limited to a small set of resource
families. It should not introduce general destructors or hidden finalization as
ordinary program logic.

### Task-boundary memory model

Structured concurrency should treat task boundaries as typed boundaries.

Recommended rules:

- immutable values may be captured or shared when their representation is safe
- mutable binding capture across a `spawn` boundary is rejected by default
- opaque resource handles may cross a task boundary only when their capabilities
  allow it
- channel `send` / `recv` establishes the synchronization needed to make
  transferred values visible
- `join()` failure is visible through `Result` or an explicit join-result enum,
  not through hidden exceptions

The goal is that data races and forgotten lifetime edges are hard to express in
ordinary Muga code. Locks and shared mutable synchronization may be added later,
but they should remain explicit library/resource APIs rather than the default
coordination style.

### Practical standard types

The small scalar core should grow only when concrete APIs need the type. Avoid
adding vague convenience aliases before their semantics are clear.

Recommended order:

1. `Bytes` for binary IO, HTTP payloads, hashing, and encoding boundaries.
2. `Buffer` and `StringBuilder` for efficient repeated construction.
3. `Duration` and `Instant` for timeouts, deadlines, elapsed time, and
   benchmarks.
4. `DateTime` with an explicit UTC-first policy; richer timezone behavior
   should be a deliberate standard-library dependency choice.
5. `F64` or `Float` for JSON and numeric APIs, with NaN, equality, ordering,
   and serialization behavior explicitly documented.
6. `Decimal` for money, billing, and exact decimal APIs; it should not be a
   synonym for floating point.

These types should follow the same source-level value semantics as other
ordinary values. Mutable internal storage should be hidden behind builders,
buffers, or resource APIs where needed.

### Static metadata and attributes

Muga will likely need limited metadata for tests, generated schemas, external
names, deprecation, and tooling. This should be static metadata, not a macro or
runtime reflection system.

Candidate examples:

```muga
@deprecated("use new_name")
@external_name("user_id")
@test
```

Attributes should be accepted only when the compiler or an explicit tool knows
their meaning. Public API metadata that affects downstream tools should be
stored in `.mgi`. Attributes should not rewrite code, inject hidden control
flow, change ordinary name resolution, or become the primary abstraction
mechanism.

### API compatibility and editions

Muga should turn package interfaces into a compatibility advantage. The
initial `.mgi` API diff contract lives in
[mgi-api-diff.md](mgi-api-diff.md) and defines the comparison input, public
identity model, classifications, deprecation handling, implemented library
comparator, `muga api-diff` CLI wrapper, and JSON output.

Initial compatibility policy:

- public function signature changes are breaking
- public enum variant additions are breaking for downstream exhaustive matches
  unless a future compatibility mechanism explicitly handles them
- required public record field additions are breaking for transparent records
- implementation-only body changes should not change public interface hashes
- deprecation should be expressed with static metadata
- language edition and enabled semantic feature set belong in package/build
  fingerprints; see
  [edition-feature-fingerprint-policy.md](edition-feature-fingerprint-policy.md)

Edition support should preserve old source meaning when syntax or semantic
rules need to evolve. It should not be a workaround for unstable public
interfaces.

### Serialization and schema evolution

Generated schemas, clients, and service stubs should consume `.mgi` interfaces
plus explicit adapter metadata. Public Muga types and wire formats should be
close, but not treated as automatically identical.

Before implementing generators, decide:

- how records, enums, `Option`, `Result`, `List`, `Map`, scalars, and opaque
  types map to the target format
- whether `Option::None` is encoded as an absent field, `null`, or a
  format-specific representation
- enum tagging and unknown variant behavior
- unknown field, default value, rename, and field removal policy
- how `Result` errors map to HTTP, RPC, or other transport failures
- how generated names remain stable across package versions

Unsupported public types should make generation fail clearly. Generators should
not inspect private bodies, infer endpoints from naming conventions, or rely on
runtime reflection.

### Testing and benchmarks

Testing support should arrive as ordinary Muga code plus compiler-recognized
discovery metadata, not as a separate language.

Implemented first shape:

```muga
import std::test

@test
fn parses_age(): Result[Unit, String] {
  age = try "42".parse_int()
  if age == 42 {
    Result::Ok(())
  } else {
    Result::Err("age did not parse")
  }
}
```

Initial rules:

- `muga test` discovers `@test` functions through source or package metadata
- tests return `Unit` or `Result[Unit, E]`
- `Result::Err(error)` fails the test and reports the error payload
- `std::test` provides the first assertion helpers:
  `test::assert_true`, `test::assert_eq_int`, `test::assert_eq_bool`, and
  `test::assert_eq_string`
- structural equality assertions stay out of v1 under the scalar-only equality
  policy

Property testing, fuzzing, golden tests, and benchmarks are useful later, but
the first goal is a small, reliable test runner that works with package
interfaces and fast edit-check-run workflows.

### Observability and operations

Service-oriented Muga code will need structured logging, metrics, tracing,
configuration, and graceful shutdown. These should be standard-library and
runtime facilities built on explicit values, resource handles, and task
contexts, not hidden global mutation.

Recommended direction:

- logs are structured key/value events, not only formatted strings
- metrics use an explicit registry or task context
- tracing context propagates through structured tasks and explicit adapters
- request context is an ordinary typed value or compiler-known task context,
  not ambient dynamic state
- graceful shutdown is expressed through task groups, cancellation, and resource
  cleanup rules

This keeps operational behavior inspectable by tools and compatible with
Muga's preference for explicit effects and typed contracts.

## Priority Order

### 0. Finish the v1 package/artifact foundation

Keep the current roadmap priority first:

- harden explicit `.mgi` / `.mgc` / `.mgb` artifact workflows
- keep `--artifact-root` explicit and fail loudly on missing or stale artifacts
- improve diagnostics and samples around dependency-body-free `check` and `run`
- avoid starting broad language-surface work while package execution semantics are still moving

Reason: practical standard libraries and reusable packages depend on stable package interfaces and implementation artifacts. Building many surface features before that boundary is stable will create churn.

### 0.5. Recommended functional additions before v1

If development continues before the v1 compatibility promise, prioritize small
features that make Muga programs easier to write and validate without changing
the core language model.

Implemented pre-v1 usability additions in this lane:

1. Narrow `List` and `Map` helper functions that do not require iterator
   protocols.
2. A scalar-only equality policy that keeps structural equality out of v1.

Do not treat these as a reason to cut a release. They are implementation
candidates for making the v1 surface more usable while release timing remains a
maintainer decision.

The first `muga test` shape has landed and should stay small:

```muga
import std::test

@test
fn parses_age(): Result[Unit, String] {
  age = try "42".parse_int()
  test::assert_eq_int(42, age)
}
```

Initial rules:

- attributes are compiler/tool-recognized static metadata, not macros
- tests return `Unit` or `Result[Unit, E]`
- `Result::Err(error)` fails the test and reports the error payload
- scalar assertion helpers return `Result[Unit, String]` and can be composed
  with `try`
- scalar assertions remain the v1 assertion boundary because equality is
  scalar-only
- package and artifact-backed workflows must be able to discover or preserve
  test metadata without reading unrelated private dependency bodies

Implemented `Option` helpers in `std::option`:

- `is_some`
- `is_none`
- `map`
- `and_then`
- `value_or`

Implemented `Result` helpers in `std::result`:

- `is_ok`
- `is_err`
- `map`
- `map_err`
- `and_then`
- `value_or`

These helpers should transform values. They should not become propagation
syntax; early return stays visible as `try expr`.

Implemented collection helpers:

- `std::list`: `map`, `filter`, `fold`, `any`, `all`
- `std::map`: `keys`, `values`

These helpers allocate new lists for transformations or key/value extraction
and preserve list or map entry order. Keep `List.contains`, `Map.entries`,
iterator protocols, map literals, `Set[T]`, arbitrary `Map` key types, and
broad collection APIs out until the equality, hashing, allocation, entry-record,
and package interface consequences are explicit.

### 0.6. Developer-experience additions that help adoption

Practical adoption is not only a language-surface problem. Muga also needs a
small set of dependable tools that make the current model easy to learn, edit,
test, and publish as reusable packages. These should not force release timing;
they are candidates for making the v1 path more usable while version bumps
remain maintainer decisions.

Recommended order:

1. Continue LSP/editor support on top of entry-aware `check --format json`
   output, `muga metadata --format json` package facts, and initial
   `muga hover --format json` declaration hovers. Initial
   `muga completions --format json` visible package/interface completions and
   `muga definition --format json` go-to-definition data and
   `muga references --format json` entry-module reference data and
   `muga workspace --format json` entry-reachable workspace metadata have
   landed. `muga syntax --format json` now provides single-file parse feedback,
   and CLI JSON diagnostics now include entry source context in
   `diagnostics[].context`. Artifact-backed check diagnostics now add entry
   package, artifact-root, and concrete artifact-file context where available.
   `muga build --format json` now reports artifact root, artifact kind, path,
   URI, and written/reused status. Explicit artifact emission commands now
   support `--format json` with artifact root, kind, path, and URI output.
   Artifact diagnostics now expose structured hash and regeneration-command
   context where available, including `.mgb` dependency-interface set changes
   in `run` diagnostics and `why-rebuild --format json`. Artifact-backed
   execution now has representative dependency API coverage combining stdlib
   packages, `try`, generic records/functions, enums, and transitive
   dependencies without source-body fallback. `.mgi` public interface hash
   stability now has representative coverage after implementation-only edits
   and source-span movement. `.mgb` structural validation and bytecode merge
   behavior now has representative coverage for control-flow-heavy dependency
   bodies, private package items, and independently generated artifacts. The
   `muga build` reuse output and lockfile update behavior now have focused
   coverage for local path and local archive dependencies after dependency
   implementation-only edits, public signature edits, archive content updates,
   and malformed lockfiles. Recursive annotation diagnostics now point direct
   recursion at parameter/return annotations and mutual recursion at explicit
   signatures for every function in the group. Package-mode public signatures
   now have representative coverage for every v1-supported public type shape
   through in-memory and persisted interfaces. The stdlib package docs and samples review
   now covers `std::io`, `std::fs`, `std::path`, `std::env`, `std::cli`,
   `std::time`, `std::string`, `std::fmt`, and `std::json`, including artifact-backed execution samples where useful.
   The release gate and GitHub Actions are now aligned by invoking
   `scripts/v1-release-gate.sh` directly from CI and the release workflow.
   Minimal shell completions and `muga doctor` have landed as a tool-only
   adoption surface.
   `muga test --format json`
   now reports structured
   test results, captured per-test stdout/stderr, summary counts, runtime
   diagnostic call context, and source-spanned assertion failure diagnostics.
   `muga run --format json` now reports captured program stdout/stderr,
   returned `main` values, and compiler/runtime diagnostics with runtime
   call-context notes.
   `samples/projects/report_app` now shows local dependencies, args/env,
   stdout/stderr, text-file handle writes, JSON run output, `Result` errors,
   reusable APIs, artifact-backed execution, and `run --built` in one runnable
   manifest project. `muga explain <diagnostic-code>` now
   prints the documented diagnostic catalog entry or stable diagnostic-code
   family from `errors.md`. [editor-json-workflow.md](editor-json-workflow.md)
   now documents and tests a concrete adapter flow across syntax, check,
   workspace, metadata, hover, completions, definition, references, run, and
   test JSON. `muga why-rebuild` now implements compact human text output, and
   `muga why-rebuild --format json` implements the first non-mutating
   artifact/cache explanation contract from
   [artifact-cache-explanations.md](artifact-cache-explanations.md), including
   local archive-cache metadata, before editor or agent tools depend on rebuild
   reasoning. Runtime/debug v1 reporting now keeps stack context in
   `related` call-context notes, failed scalar assertions in source-spanned
   `R021` diagnostics, and artifact next-actions in `regenerationCommand`
   context. [benchmark-health-checks.md](benchmark-health-checks.md) now
   defines release-neutral local checks for compiler stages, package artifact
   reuse, and representative String/List/Map runtime paths.
2. Keep adding examples only when they demonstrate a new existing workflow,
   not to widen the v1 language surface.
3. The implemented first `std::json` package contract from
   [std-json-first-slice.md](std-json-first-slice.md) is audited in
   [std-json-implementation-audit.md](std-json-implementation-audit.md)
   against docs, samples, artifact-backed behavior, and release-readiness
   evidence. Keep the `Result` ergonomics, scalar/collection mapping, schema
   evolution, and diagnostics boundary intact before broadening schema
   generation, HTTP/RPC, `Float`, `Decimal`, `Bytes`, streaming APIs, or
   resource handles. Choose the next narrow stdlib/API boundary only after
   documenting that contract.

Tooling should prefer structured compiler output over scraping human text.
`muga syntax --format json` now returns single-file lex/parse diagnostics for
faster editor feedback without resolver/typechecker or package import work.
`muga check --format json` now includes the entry path and a best-effort
`file://` URI for editor, CI, LSP, and agent consumers, and compiler JSON
diagnostics now copy that entry source context into `diagnostics[].context`.
Artifact-backed `check --format json` diagnostics also include entry package,
artifact-root, and concrete artifact-file context when available. `muga doc`
now emits Markdown from
public package interface records, enums, functions, and item-level public source
comments stored in `.mgi`; these are public source comments, not interface hash
inputs. The implemented
`muga fmt --check` path preserves source meaning and line comments. `muga new`
now lists available starter templates and creates small app, lib, test, config
app, strict CLI tool, and report app manifest projects.
`muga metadata --format json` now exposes package/module/item/export metadata
plus public interface docs and rendered types for editor, LSP, CI, and agent
consumers. `muga hover --format json` now returns declaration hover data with
public docs and signatures. `muga completions --format json` now returns
visible package/interface completions with import aliases plus public docs and
signatures. `muga definition --format json` now returns go-to-definition data
for import aliases, local bindings, and package/interface item references.
`muga references --format json` now returns declaration plus entry-module
references for the same initial target set. `muga workspace --format json` now
returns loaded packages, module source files, the default artifact root, and
dependency edges reachable from an entrypoint. `muga build --format json` now
returns structured artifact status for `.mgi`, `.mgc`, and `.mgb` outputs, and
explicit artifact emission commands now return structured artifact root, kind,
path, and URI data. Artifact diagnostics now carry structured hash and
regeneration-command context where available. `muga test --format json` now
returns structured test outcomes for editor, LSP, CI, and agent tooling.
`muga run --format json` now returns captured run stdout, stderr, returned
`main` value, and diagnostics for the same consumers.
[editor-json-workflow.md](editor-json-workflow.md) documents the current
single-entry editor adapter sequence and is backed by the
`json_backed_editor_workflow_uses_existing_command_contracts` regression test.
`muga why-rebuild` now implements compact human text output for terminal users,
and `muga why-rebuild --format json` implements the initial non-mutating
artifact/cache explanation contract from
[artifact-cache-explanations.md](artifact-cache-explanations.md) over
`artifactRoot`, `artifactFile`, `artifactHash`, and `regenerationCommand`
facts, with stale dependency-interface and lockfile metadata coverage now in
place alongside local archive-cache metadata and implementation
dependency-interface set-change hash context.
LSP and documentation generation should consume the same package/interface facts
as the compiler, so tools do not need to read unrelated private dependency
bodies.

### 0.7. Syntax candidates to track, not rush

Muga should add syntax only when repeated real programs show that library APIs
or tooling are not enough. The default for v1 is still to keep the grammar
small. Syntax candidates should be tracked separately from release readiness and
must update the mini spec, split specs, parser diagnostics, formatter rules,
package interfaces, samples, and release-readiness checks before they become
part of the supported surface.

Recommended syntax priority:

1. Static attributes beyond the implemented `@test` metadata. Attributes
   should remain compiler/tool-recognized metadata, not macros or runtime
   reflection.
2. Named arguments for clarity at long or same-typed call sites, for example
   `copy(from: source, to: destination)`. Before implementation, decide label
   storage in `.mgi`, compatibility rules for label renames, whether positional
   and named arguments may mix, and how diagnostics report unknown or duplicate
   labels.
3. First-slice statement-form `using` cleanup is implemented after opaque
   resource handles. Further `using` expressions or multiple bindings should
   continue to express deterministic resource cleanup, not general destructors
   or hidden exception behavior.
4. Range or slicing syntax only after string and collection slicing semantics
   are settled. If added, it must stay aligned with `String.slice_chars(start,
   count)` and the byte/scalar/grapheme policy.
5. Small pattern-matching refinements only when examples justify them. Keep
   wildcard-heavy matching, broad catch-all enum patterns, and protocol-like
   destructuring out until exhaustiveness and diagnostics remain simple.
6. String interpolation or formatting templates only after placeholder grammar,
   builders, escaping, conversion, and localization expectations are explicit.
7. Optional shorthand such as `T?` and `?.` remains reserved for Option-only
   features later. It should not become Result propagation or nullable-by-
   default behavior.

Keep postfix `expr?` out for `Result` propagation. If Result chain propagation
is ever added, prefer visible postfix keyword syntax such as `expr.try` and
document it as distinct from value-transforming `Result` helpers.

### 0.8. Ecosystem and maintenance foundations

The next missing pieces in Muga's whole-language picture are mostly trust,
maintenance, and learning systems. They do not add expressive power, but they
make the language usable by people, editors, package tools, and coding agents
without depending on informal conventions.

Recommended order:

1. A conformance test suite tied to the mini spec and split specs. It should
   contain valid programs, rejecting programs with expected diagnostic codes,
   package/artifact workflow cases, and compatibility fixtures that can be run
   against future compiler changes.
2. Stable machine-readable diagnostics. The CLI and library should be able to
   emit a schema with `code`, `severity`, entry path/URI, primary span, related
   spans, suggestions, package/artifact context, and a human message. LSP, CI,
   and agent workflows should use this instead of scraping display text.
3. `.mgi` API compatibility tooling. The first library comparator and CLI now compare
   public interfaces and classifies function signature changes, record field
   changes, enum variant changes, opaque handle facts, and implementation-only
   edits.
4. Standard-library design rules. Before broadening `std`, use the review
   checklist in [standard-library-review-rules.md](standard-library-review-rules.md):
   no hidden IO in property access, recoverable effects return `Result`,
   absence is `Option`, opaque resources are not transparent records, public
   error types are explicit, and ambiguous convenience names wait until their
   semantics are stable.
5. Runtime debugging and failure reports. Runtime errors now keep structured
   related notes for nested function call sites and entry/test execution, and
   failed `std::test` scalar assertions now point at the user assertion call
   with `R021`. Package/artifact next actions use `regenerationCommand`
   context, and v1 keeps runtime stack context in `related` notes rather than
   a separate stack-trace schema.
6. Benchmark health checks. Release-neutral local checks now cover compiler
   stages, package artifact reuse, and representative String/List/Map/runtime
   paths through `scripts/benchmark-health-check.sh`. Avoid strict public
   performance claims until MIR, backend, and service runtime layers exist.
7. Fuzzing and malformed-input planning. The v1 trust-boundary plan in
   [fuzzing-malformed-input-plan.md](fuzzing-malformed-input-plan.md) covers
   parser/syntax, package archive, lockfile, interface, check-cache, and
   implementation artifact readers.
8. Installation and onboarding. The release-neutral guide in
   [installation-and-onboarding.md](installation-and-onboarding.md) covers
   `cargo install`, local checkout installs, `muga --version`, top-level and
   command-specific `muga help`, generated project quickstarts, and later binary release
   expectations without treating them as release triggers.
9. Education docs such as "Muga by Example". The release-neutral learning path
   in [muga-by-example.md](muga-by-example.md) progresses from bindings and
   records to `Result`, packages, tests, local dependencies, and
   artifact-backed builds.
10. Supply-chain and registry security. The design in
   [registry-security-design.md](registry-security-design.md) preserves the
   current `.mgp` archive hashing foundation and scopes future signing,
   provenance, lockfile enforcement, cache validation, and malicious or
   abandoned package handling before remote fetching.
11. Edition and feature-set policy. The design in
    [edition-feature-fingerprint-policy.md](edition-feature-fingerprint-policy.md)
    records how package/build fingerprints should include the language edition
    or enabled semantic feature set before syntax or semantic changes need
    backward-compatible migration.

The best v1-aligned subset is a conformance-suite skeleton, a documented
machine-readable diagnostic shape, `.mgi` API-diff design notes in
[mgi-api-diff.md](mgi-api-diff.md), and standard library review rules in
[standard-library-review-rules.md](standard-library-review-rules.md). Binary
distribution, remote registry security, benchmarks with public thresholds, and
edition migrations can remain post-v1 unless a concrete v1 workflow needs them
sooner.

### 0.9. Modern language gap decision pass

The broad modern-language inventory for 2026-05-22 lives in
[modern-language-gap-inventory-2026-05-22.md](modern-language-gap-inventory-2026-05-22.md),
and the classification pass lives in
[modern-language-gap-decisions-2026-05-22.md](modern-language-gap-decisions-2026-05-22.md).

The decision from that pass is to prioritize validation and tool contracts
before adding broad syntax or platform APIs.

V1 validation/support work:

- conformance-suite layout
- JSON diagnostics and stable command-output contracts
- `.mgi` API compatibility diffing
- standard-library review rules in
  [standard-library-review-rules.md](standard-library-review-rules.md)
- doc-comment and generated-doc rules
- `muga metadata` package metadata and artifact/cache explanation commands
- runtime/debug failure reporting through call-context notes, `R021`, and
  artifact regeneration-command next actions
- lightweight benchmark health-check design
- fuzzing and malformed-input plans for parser, archive, lockfile, interface,
  check-cache, and implementation artifact readers in
  [fuzzing-malformed-input-plan.md](fuzzing-malformed-input-plan.md)
- installation/onboarding docs in
  [installation-and-onboarding.md](installation-and-onboarding.md) and
  Muga-by-example learning path in [muga-by-example.md](muga-by-example.md)
- edition or semantic feature-set fingerprint policy in
  [edition-feature-fingerprint-policy.md](edition-feature-fingerprint-policy.md)

Optional pre-v1 usability work:

- LSP/editor prototype building on entry-aware JSON diagnostics,
  `muga metadata` package facts, `muga hover` declaration hovers, and JSON
  `muga test` results
- Artifact/cache explanation command design before editor or agent tools depend
  on rebuild reasoning

Post-v1 platform work:

- opaque resources, `using`, `Bytes`, buffers/builders, time types, `Float`,
  JSON, HTTP, service runtime, structured concurrency, channels, async IO,
  workspaces, dev/test/bench dependencies, version solving, remote registries,
  signing/provenance, `muga audit`, SBOMs, full incremental rebuild planning,
  MIR/native/WASM backends, profilers, and binary distribution channels

Deliberate non-goals for the current direction:

- universal null, implicit exceptions, postfix `expr?` for `Result`, runtime
  reflection as a core mechanism, macros/code rewriting as ordinary
  abstraction, user-defined operators, overloaded dispatch, dynamic `Any` as
  normal interop, source-level references or borrowing, hidden async
  suspension, class inheritance, hidden-IO property access, arbitrary
  unsandboxed build scripts, and near-term scientific/ML/mobile/embedded focus

If a future task comes from the inventory but is not in the first two lists, it
should be treated as post-v1 or require a focused design note before it changes
the v1 surface.

### 1. Harden user-defined generic records and functions

The first generic records/functions slice is implemented. The next work is hardening: examples, docs, and diagnostics for the explicit type-parameter model.

Recommended shape:

```muga
record Box[T] {
  value: T
}

fn id[T](value: T): T {
  value
}
```

Rules to preserve:

- type parameters are explicit on declarations
- ordinary unannotated functions are not implicitly generalized
- call sites use local type-argument inference when possible
- package interfaces store resolved generic public signatures
- no bounds, protocols, typeclasses, higher-kinded types, specialization, or polymorphic recursion in the first implementation

Reason: users can now write small reusable libraries on top of builtin `List[T]`, `Option[T]`, `Result[T, E]`, and `Map[K, V]`; the remaining risk is making the behavior obvious and stable at package boundaries.

### 2. Harden `try expr` for Result propagation

`Result[T, E]`, exhaustive `match`, and prefix `try expr` are the current semantic base. The next work is not more error syntax; it is making practical fallible APIs return `Result` consistently.

Recommended shape:

```muga
fn load_age(path: String): Result[Int, String] {
  text = try read_file(path)
  text.parse_int()
}
```

Current rules are conservative:

- `try expr` works only when `expr` has type `Result[T, E]`
- the enclosing function must return `Result[U, E]`
- propagated error types must match exactly at first
- do not make `try` work for `Option[T]` in the first version
- do not add postfix Result propagation `expr?`; if chain propagation is added later, use postfix keyword syntax `expr.try`
- keep the `?` syntax family reserved for optional-value features such as future `T?` and Option-only `?.`
- do not add implicit exceptions or `throws`

Reason: `try` makes early return visible at the expression site and fits Muga's preference for explicit control flow.

Result-heavy code still needs a compact success path. For a future chain-propagation syntax, prefer postfix keyword `expr.try` so the early-return marker remains a word in the dot chain:

```muga
fn load_age(path: String): Result[Int, String] {
  age = read_file(path).try.trim().parse_int().try
  Result::Ok(age)
}
```

This is a future syntax direction, not the current implementation. It should be equivalent to applying prefix `try` at that chain point: `read_file(path).try.trim()` means `(try read_file(path)).trim()`.

For chains that transform a `Result` value without returning early, prefer ordinary value-chain helpers such as `result::map` and `result::and_then`, assuming `result` is the visible helper package alias:

```muga
fn load_age(path: String): Result[Int, String] {
  (
    read_file(path)
      .result::map(fn(text) { text.trim() })
      .result::and_then(fn(text) { text.parse_int() })
  )
}
```

These helpers transform `Result` values and preserve errors; they do not return early from the enclosing function. Use `try expr`, or future `expr.try`, when the function intentionally unwraps an intermediate success value and propagates `Err`.

### 3. Treat public package interfaces as application contracts

The package interface is not only a compiler cache boundary. It should become the typed public contract for reusable packages and service APIs.

The contract source should be:

- `pub record` for transparent data shapes
- `pub enum` for closed state, variants, and typed error families
- `pub fn` signatures for callable package or service operations
- `Result[T, E]` return types for recoverable failures where `E` is also part of the public contract
- deterministic `.mgi` interface artifacts as the stable serialized form

Once this boundary is stable, tools can generate from `.mgi` without re-reading implementation bodies:

- API documentation
- JSON Schema for public records/enums where the mapping is well-defined
- OpenAPI or service descriptions for explicit HTTP/RPC adapter packages
- TypeScript or other client bindings
- server/client stubs for a future Muga-owned RPC shape

Do not make this a hidden framework convention. A package should opt into an external protocol through explicit adapter APIs, and the generator should fail when a public type has no agreed external representation.

Keep these decisions visible before implementing generators:

- how Muga scalar, record, enum, `Option`, `List`, `Map`, and `Result` types map to the target schema
- whether public transparent records are enough, or whether a future `pub opaque type` is needed for runtime handles
- how package and item identity in `.mgi` maps to stable external names
- how error enums/records become transport-level failures without implicit exceptions
- how generated artifacts are versioned and invalidated when interface hashes change

### 4. Grow a small practical standard library

The next practical bottleneck is not syntax. It is the absence of ordinary APIs.

Prioritize packages in this order:

1. `std::string`: the first prelude-helper slices implement `trim`, `contains`, `starts_with`, `ends_with`, `is_empty`, `char_count`, `byte_len`, `replace`, `split`, `concat`, `slice_chars`, `parse_int`, and `parse_bool`. `char_count` and `slice_chars` use explicit Unicode scalar-value semantics; `byte_len` reports the UTF-8 byte size. Keep ambiguous `String.len()`, range syntax or substring aliases, grapheme-cluster APIs, and richer string error types deferred until the relevant standard-library slice needs those decisions.
2. `std::fmt` or equivalent formatting helpers: the first explicit prelude slice is `to_string` for `Int`, `Bool`, and `String` plus `String.concat`, and `std::fmt` now adds pure `repeat`, `pad_left`, `pad_right`, `truncate_chars`, and explicit `format_values` helpers over `String` values. Language interpolation, builders, localization, terminal display width, and implicit conversion remain deferred.
3. `std::path`, `std::fs`, `std::bytes`, `std::hash`, and `std::io`: the first path/text-file, path joining, pure lexical path cleanup, parent lookup, prefix stripping, path name/stem extraction, file-name replacement, extension extraction/replacement, absolute-path classification, directory listing, recursive directory listing, directory size metadata, directory creation, recursive directory creation, single-file removal, empty-directory removal, recursive directory removal, single-file copy, no-overwrite recursive directory copy, no-overwrite copy-then-remove recursive directory move, one-step rename, scalar file-size metadata, modified-time metadata, a regular-file metadata record, path status/kind/info grouping, existing-path metadata, optional regular-file size metadata for existing paths, existing-path canonicalization, package resource text/bytes, local binary file read/write, opaque `Bytes` inspection, and SHA-256 hex slices are implemented (`path::Path`, `path::from_string`, `path::as_string`, `path::join`, `path::normalize`, `path::file_name`, `path::with_file_name`, `path::parent`, `path::strip_prefix`, `path::extension`, `path::file_stem`, `path::with_extension`, `path::is_absolute`, `fs::read_text`, `fs::read_bytes`, `fs::write_text`, `fs::write_bytes`, `fs::read_text_path`, `fs::read_bytes_path`, `fs::write_text_path`, `fs::write_bytes_path`, `fs::read_resource_text`, `fs::read_resource_bytes`, `bytes::size`, `bytes::empty`, `bytes::at`, `hash::sha256_hex`, `fs::read_dir_path`, `fs::read_dir_recursive_path`, `fs::directory_size_metadata_path`, `fs::create_dir_path`, `fs::create_dir_all_path`, `fs::remove_file_path`, `fs::remove_dir_path`, `fs::remove_dir_all_path`, `fs::copy_file_path`, `fs::copy_dir_all_path`, `fs::move_dir_all_path`, `fs::rename_path`, `fs::file_size_path`, `fs::modified_unix_millis_path`, `fs::FileMetadata`, `fs::file_metadata_path`, `fs::PathStatus`, `fs::PathKind`, `fs::PathInfo`, `fs::PathMetadata`, `fs::PathSizeMetadata`, `fs::DirectorySizeMetadata`, `fs::path_status`, `fs::path_kind`, `fs::path_info`, `fs::path_metadata_path`, `fs::path_size_metadata_path`, `fs::canonicalize_path`, `fs::exists_path`, `fs::is_file_path`, `fs::is_dir_path`, `io::IOError`, and `io::PathPairError`). Keep stdout/stderr handles, resource handles, binary streams, mutable buffers/builders, encoding/decoding, broader cryptographic APIs, accessed/created timestamps, permissions, strict path component validation, public symlink classification, sandbox containment, host-rename acceleration, rollback, merge/overwrite directory copy/move operations, and broader mutation operations as follow-up slices.
4. `std::env`, `std::cli`, `std::time`, and `std::process`: the first environment slices are implemented as `env::get_var(name): Option[String]`, `env::args(): List[String]`, `env::current_dir(): Result[path::Path, io::IOError]`, and `env::temp_dir(): Result[path::Path, io::IOError]`; the first CLI helper slice is implemented as pure `cli::positional`, `cli::positional_or`, `cli::has_flag`, `cli::has_short_flag`, `cli::option`, and `cli::option_or` over explicit `List[String]` values; typed scalar `std::cli` helpers now parse positional and option values as `Int` or `Bool` through `Result`; compiler-owned `cli::parse_or[T]` / `cli::usage_for[T]` plus `@cli(...)` metadata cover config/default overlays; [strict-cli-parser-schema.md](strict-cli-parser-schema.md) implements strict `cli::parse[T](args)` for required command-line-only records with `MissingArgument` errors; `samples/projects/cli_tool` implements the checked-in strict CLI tool sample; generated `muga new --template cli-tool` adoption is implemented from that sample shape; [strict-cli-no-default-usage.md](strict-cli-no-default-usage.md) implements `cli::usage_for_required[T](program)` with explicit call type arguments and replaces historical strict CLI manual help duplication; [cli-command-metadata.md](cli-command-metadata.md) implements `@cli(about: "...")` usage summaries; [cli-short-option-metadata.md](cli-short-option-metadata.md) implements `@cli(short: "x")`; [post-cli-short-option-metadata-adoption-gap-selection.md](post-cli-short-option-metadata-adoption-gap-selection.md) selects typed CLI positional field metadata design next; [cli-positional-field-metadata.md](cli-positional-field-metadata.md) implements `@cli(positional: N)` typed operands with generated `Arguments:` usage, interface/artifact persistence, and strict/template coverage; [post-cli-positional-field-metadata-adoption-gap-selection.md](post-cli-positional-field-metadata-adoption-gap-selection.md) selects the built-in CLI help policy in [cli-built-in-help-policy.md](cli-built-in-help-policy.md), leading to `cli::help_requested` and generated help helpers; [cli-built-in-help-policy.md](cli-built-in-help-policy.md) implements `cli::help_requested`, `cli::help_for`, and `cli::help_for_required` with schema-backed help rendering and generated template adoption; [post-built-in-cli-help-helper-adoption-gap-selection.md](post-built-in-cli-help-helper-adoption-gap-selection.md) selected parse-integrated CLI help workflow design; [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md) implements `cli::Request[T]`, `cli::parse_request[T]`, and `cli::parse_request_or[T]` across strict/config starters; [post-parse-integrated-cli-help-workflow-adoption-gap-selection.md](post-parse-integrated-cli-help-workflow-adoption-gap-selection.md) selects compact CLI short option syntax design next; [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md) implements `-abc`, `-ofile`, and `-abo=value` over existing short metadata; [post-compact-cli-short-option-syntax-adoption-gap-selection.md](post-compact-cli-short-option-syntax-adoption-gap-selection.md) selected CLI subcommand metadata design; [cli-subcommand-metadata.md](cli-subcommand-metadata.md) now implements enum/variant metadata plus strict command enum schemas through source validation, `.mgi` package interfaces, `.mgb` schema payloads, recursive runtime dispatch/help, artifact-backed execution, and `run --built`; [post-cli-subcommand-schema-adoption-gap-selection.md](post-cli-subcommand-schema-adoption-gap-selection.md) adopts that command-tree shape in `samples/projects/cli_tool` and generated `cli-tool` starters with `run` / `inspect` subcommands; [cli-wrapper-root-options.md](cli-wrapper-root-options.md) implements strict wrapper records with one `@cli(subcommand)` field for root/global options, including schema/artifact lowering, runtime parse/help, source/artifact/`run --built` coverage, and strict sample/generated `cli-tool` adoption with `--profile` / `-p`; [cli-schema-shell-completions.md](cli-schema-shell-completions.md) implements `muga cli-completions <bash|zsh|fish> --program <name> --type <Type> ...` as the `CliSchema`-backed generated app completion surface across source, `--artifact-root`, and `--built` workflows; [post-cli-schema-shell-completion-adoption-gap-selection.md](post-cli-schema-shell-completion-adoption-gap-selection.md) implements generated `cli-tool` completion onboarding through install docs, a generated `cli-tool` README, and `scripts/generate-completions.sh`; and the first time slice is implemented as `time::UnixMillis` plus `time::now_unix_millis(): UnixMillis`. Keep richer CLI parsing/help, richer time APIs, unique temp-file allocation/cleanup policy, canonicalization, and project-root lookup behind concrete need; move process execution up as the first core acceleration slice.
The generated-app completion surface now also has shell-agnostic JSON specs and
static file/directory value-source metadata in
[cli-completion-json-spec.md](cli-completion-json-spec.md) and
[cli-completion-value-sources.md](cli-completion-value-sources.md), plus
non-mutating completion package emission in
[cli-completion-installer-integration.md](cli-completion-installer-integration.md).
Dynamic completion producers and shell-profile installation remain deferred
with process execution and broader host effects.
The generated `muga new --template report-app` starter is implemented as a
single-project file-processing template in
[generated-report-app-template.md](generated-report-app-template.md), covering
`std::fs` text reads/writes, `std::path::with_extension`, explicit `Result`
mapping, source runs, built-artifact runs, a root-changing helper, and a
source-free package helper. The generated `muga new --template package-app`
starter is implemented in [generated-package-app-template.md](generated-package-app-template.md)
as the first multi-package starter, reusing local path dependencies, workspace
JSON, package artifacts, and source-free app bundles without adding a workspace
manifest or registry policy.

5. `std::json`: the first parse/encode slice, value/object-field scalar and
   composite accessors, defaults, required-field helpers, scalar array
   projection helpers, JSON path projection helpers, and compiler-owned
   `json::decode_or[T](value, fallback)` / strict `json::decode[T](value)`
   decoders are implemented through explicit data types and `Result`. The
   slice is scoped in
   [std-json-first-slice.md](std-json-first-slice.md) and audited in
   [std-json-implementation-audit.md](std-json-implementation-audit.md).
6. `std::http`: only after resource handles, Result ergonomics, package workflow, and cancellation-aware IO policy are stable.

Recommended API style:

```muga
import std::fs
import std::io
import std::path
import std::time

text_path = "notes/todo.txt"
file_path = path::from_string(text_path)
child_path = path::join(file_path, "archive.txt")
clean_path: path::Path = path::normalize(path::from_string("notes/./archive/../todo.txt"))
name: Option[String] = path::file_name(file_path)
sibling_path: path::Path = path::with_file_name(file_path, "summary.txt")
parent: Option[path::Path] = path::parent(file_path)
relative: Option[path::Path] = path::strip_prefix(file_path, path::from_string("notes"))
extension: Option[String] = path::extension(file_path)
stem: Option[String] = path::file_stem(file_path)
json_path: path::Path = path::with_extension(file_path, "json")
absolute: Bool = path::is_absolute(file_path)
result: Result[String, io::IOError] = fs::read_text(text_path)
written: Result[Unit, io::IOError] = fs::write_text(text_path, text)
path_result: Result[String, io::IOError] = fs::read_text_path(file_path)
entries: Result[List[path::Path], io::IOError] = fs::read_dir_path(file_path)
descendants: Result[List[path::Path], io::IOError] = fs::read_dir_recursive_path(file_path)
directory_size: Result[fs::DirectorySizeMetadata, io::IOError] = fs::directory_size_metadata_path(file_path)
created: Result[Unit, io::IOError] = fs::create_dir_path(file_path)
created_all: Result[Unit, io::IOError] = fs::create_dir_all_path(file_path)
removed_file: Result[Unit, io::IOError] = fs::remove_file_path(file_path)
removed_dir: Result[Unit, io::IOError] = fs::remove_dir_path(file_path)
removed_tree: Result[Unit, io::IOError] = fs::remove_dir_all_path(file_path)
copied_file: Result[Unit, io::PathPairError] = fs::copy_file_path(file_path, child_path)
copied_tree: Result[Unit, io::PathPairError] = fs::copy_dir_all_path(file_path, child_path)
moved_tree: Result[Unit, io::PathPairError] = fs::move_dir_all_path(file_path, child_path)
renamed_file: Result[Unit, io::PathPairError] = fs::rename_path(file_path, child_path)
size: Result[Int, io::IOError] = fs::file_size_path(file_path)
modified: Result[time::UnixMillis, io::IOError] = fs::modified_unix_millis_path(file_path)
metadata: Result[fs::FileMetadata, io::IOError] = fs::file_metadata_path(file_path)
resolved: Result[path::Path, io::IOError] = fs::canonicalize_path(file_path)
status: fs::PathStatus = fs::path_status(file_path)
info: fs::PathInfo = fs::path_info(file_path)
kind: fs::PathKind = fs::path_kind(file_path)
path_metadata: Result[fs::PathMetadata, io::IOError] = fs::path_metadata_path(file_path)
path_size_metadata: Result[fs::PathSizeMetadata, io::IOError] = fs::path_size_metadata_path(file_path)
exists: Bool = fs::exists_path(file_path)
is_file: Bool = fs::is_file_path(file_path)
is_dir: Bool = fs::is_dir_path(file_path)
```

`io::IOError` is currently a transparent record with `operation`, `path`, `kind`, `message`, and `raw_code: Option[Int]`. `io::PathPairError` uses the same error details plus `from_path` and `to_path` for two-path filesystem operations. This is intentionally enough for one-shot text IO and `try expr` propagation, but not a final resource-handle abstraction or full filesystem model.

```muga
import std::env
import std::io
import std::path

maybe_path: Option[String] = env::get_var("PATH")
program_args: List[String] = env::args()
cwd: Result[path::Path, io::IOError] = env::current_dir()
tmp: Result[path::Path, io::IOError] = env::temp_dir()
```

```muga
import std::time

now: time::UnixMillis = time::now_unix_millis()
```

Use:

- `Result[T, E]` for recoverable effects
- `Unit` as the success value for effect-only operations
- `Option[T]` for absence
- value-returning updates for ordinary data
- builder/buffer types for repeated construction
- resource/handle types for files, sockets, timers, and OS-backed state

Avoid:

- implicit throwing exceptions
- property access with hidden IO
- dynamic `Any` as the normal interop path
- global mutable runtime state as the default API style

Resource handles are the next major design boundary for effects. They should be opaque public types, not transparent records, when source code should not observe or construct their representation. Handle APIs should make ownership, close/drop behavior, send/share rules, and cancellation behavior explicit enough for typed HIR, MIR, and package interfaces.

The post-JSON stdlib/API boundary selection in
[post-json-stdlib-boundary-selection.md](post-json-stdlib-boundary-selection.md)
keeps that as the next design prerequisite. Do not add stdout/stderr handles,
file handles, process APIs, HTTP/SSE/WebSocket/RPC, streaming APIs, `Bytes`,
buffers, schema/client generation, or broader `std::json` behavior before the
opaque resource-handle boundary is documented. The boundary is now documented
in [opaque-resource-handles.md](opaque-resource-handles.md): `pub opaque type`
interface support and `.mgi` identity are implemented, and future
runtime-backed handles still need explicit capability defaults, consuming
operations, explicit `close`, task-boundary and cancellation rules, and runtime
diagnostics before any broad IO or network API is added.

The capability and close metadata plan is now part of that boundary. The
metadata-only interface slice has landed: `.mgi` v5 persists
`OpaqueHandleFacts`, close-function identity, and `paramMode`, includes them in
public hashes, and exposes them through metadata, hover/completion metadata, and
docs without source syntax. The consuming-parameter checker now rejects direct
same-scope use-after-consume for loaded-interface `consume` parameters. The
first runtime-backed file-handle boundary and implementation are done as
read-only `std::fs::File` with `open_text`, `read_text_from`, consuming `close`,
VM-local runtime slots, source and artifact-backed execution coverage, and hard
`R022` stale/closed-handle diagnostics. The post-file-handle selection in
[post-file-handle-resource-surface-selection.md](post-file-handle-resource-surface-selection.md)
chose a program stderr channel through scalar `eprint` / `eprintln`, and that
channel is implemented. This is deliberately not a stdout/stderr handle model.
Text output file handles are implemented from
[text-output-file-handles.md](text-output-file-handles.md): one public
`std::fs::File` stores read/write/append mode in the runtime slot,
`create_text`, `append_text`, `write_text_to`, and `flush` are available, only
`close` consumes, and wrong-mode operations return recoverable `io::IOError`
values. The integrated `report_app` workflow sample now demonstrates args/env,
stdout/stderr, text-file handle writes, JSON run output, `Result`, local
dependencies, artifact-backed execution, and `run --built` together. `Bytes`,
buffering modes, stdout/stderr handles, async IO, and streaming APIs still need
separate contracts before implementation. The concrete statement-form `using`
contract lives in [lexical-resource-cleanup.md](lexical-resource-cleanup.md),
and the first implementation is now landed with nested cleanup unwind hardening
before broader IO/resource APIs. Minimal pure `std::cli` helpers are
implemented over explicit `List[String]` values, and the report workflow no
longer hand-rolls argument defaults over `env::args()`. The CLI-first generated
app template uses `std::env` and `std::cli` and is covered by project-template source, generated-project tests, and onboarding examples before richer CLI parsers, formatting templates,
`Bytes`, process APIs, network APIs, streams, or broader host effects. The
implemented typed scalar `std::cli` parsing helpers for `Int` and `Bool` are
covered by the stdlib package source, samples, and examples tests before full
CLI parser schemas, config-file loading, formatting templates, `Bytes`,
process APIs, network APIs, streams, or broader host effects. The
post-typed-cli implementation path is now carried by the implemented JSON
value accessor helpers in `std::json`, returning `json::Error` for wrong
shapes, before config-file loading, schema decoding, full CLI parser schemas,
`Bytes`, process APIs, network APIs, streams, or broader host effects.
The post-json-accessor implementation path is now carried by the implemented
`samples/projects/config_app` JSON config workflow sample that composes
existing `std::config`, `std::path`, `std::json`, `std::env`, `std::cli`, and
`std::result::map_err` with explicit CLI > config > defaults precedence before
TOML, broader schema tooling, full CLI parser schemas, `Bytes`, process APIs,
network APIs, streams, or broader host effects.
The post-config-workflow selection in
[post-config-workflow-adoption-gap-selection.md](post-config-workflow-adoption-gap-selection.md)
chooses the implemented `config_app` refresh that uses existing
`std::result::map_err` for app-boundary error normalization before new error
unions, `std::config`, TOML, schema decoding, full CLI parser schemas,
formatting templates, `Bytes`, process APIs, network APIs, streams, or broader
host effects.
The post-result-mapping path is now carried by the implemented narrow pure
`std::string` text assembly helpers (`string::concat_all` / `string::join`)
before formatting templates, interpolation, builders, broader config/schema
work, full CLI parser schemas, `Bytes`, process APIs, network APIs, streams,
or broader host effects.
The post-string-assembly path is now carried by the implemented narrow
`std::json` required object-field helpers before broader `std::config`, TOML,
schema tooling, full CLI parser schemas, formatting templates, interpolation,
`std::fmt`, builders, `Bytes`, process APIs, network APIs, streams, or broader
host effects.
The post-required-json-field path is now carried by the implemented narrow
`std::json` array/object field helpers before JSON paths, broader
config/schema work, TOML, full CLI parser schemas, formatting templates,
`Bytes`, process APIs, network APIs, streams, or broader host effects.
The nested JSON config workflow refresh carries the post-composite-json-field
path for `samples/projects/config_app`, using composite/typed `std::json`
helpers for `tags`, owner metadata, servers, and limits before JSON paths,
broader schema decoding, `std::config` expansion, TOML, full CLI parser
schemas, formatting templates, `Bytes`, process APIs, network APIs, streams,
or broader host effects.
The post-nested-json-config path is now carried by the implemented pure
`std::json` scalar array projection helpers before JSON paths, schema
decoding, broader object-field matrices, `std::config` expansion, TOML, full
CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs,
streams, or broader host effects.
The post-json-array-projection path is now carried by the implemented direct
`std::json` scalar-array object-field helpers before JSON paths, schema
decoding, `std::config` expansion, TOML, full CLI parser schemas, formatting
templates, `Bytes`, process APIs, network APIs, streams, or broader host
effects.
The post-direct-json-array-field path is now carried by the implemented
repeated `std::cli` option value helpers, so JSON/default list settings can be
overridden from explicit CLI arguments before JSON paths, schema decoding,
`std::config`, TOML, full CLI parser schemas, formatting templates, `Bytes`,
process APIs, network APIs, streams, or broader host effects.
The post-repeated-cli-option path is now carried by the implemented
`std::json` path helpers, so nested JSON access improves before schema
decoding, `std::config`, TOML, full CLI parser schemas, formatting templates,
`Bytes`, process APIs, network APIs, streams, or broader host effects.
The post-json-path path is now carried by the implemented typed JSON path scalar
projection helpers before typed array/object path helpers, schema decoding,
`std::config`, TOML, full CLI parser schemas, formatting templates, `Bytes`,
process APIs, network APIs, streams, or broader host effects.
The post-typed-json-path-scalar path is now carried by the implemented typed JSON path collection
projection helpers before schema decoding, `std::config`, TOML, full CLI
parser schemas, generated config app templates, formatting templates, `Bytes`,
process APIs, network APIs, streams, or broader host effects.
The JSON schema decoding design path is now carried by
[json-schema-decoding.md](json-schema-decoding.md) before implementing
required `json::decode`, broader `std::config`, TOML, full CLI parser schemas,
generated config app templates, formatting templates, `Bytes`, process APIs,
network APIs, streams, or broader host effects.
The JSON schema decoding design in
[json-schema-decoding.md](json-schema-decoding.md) selects a compiler-owned
`json::decode_or[T](value, fallback)` default-overlay decoder before required
`json::decode[T]`, `std::config`, TOML, full CLI parser schemas, generated
config app templates, formatting templates, `Bytes`, process APIs, network
APIs, streams, or broader host effects.
The post-JSON-schema-decoder path is now carried by the minimal `std::config`
JSON default loading design before TOML, required `json::decode[T]`, generated
config app templates, full CLI parser schemas, or broader host effects.
The selected design and implementation in
[std-config-json-loading.md](std-config-json-loading.md) fixes the first
`std::config` public API, error shape, schema-lowering model, artifact
behavior, `LoadJsonConfig` / `LoadJsonConfigRequired` artifact payloads, and
`config_app` coverage. The implemented helpers are
`config::load_json_or[T](path, fallback)` and `config::load_json[T](path)`. The
generated config app template now turns that workflow into
`muga new --template config-app`.
[config-path-discovery.md](config-path-discovery.md) adds the first explicit
config path discovery policy for generated config apps: `--config`,
`MUGA_CONFIG_PATH`, then the generated JSON default.
[workspace-manifest-metadata.md](workspace-manifest-metadata.md) adds a
machine-readable project metadata layer to `muga workspace --format json` so
tools can find manifest roots, source roots, resource roots, and dependency source/resource roots without
asking the runtime to guess resource locations.
[config-app-run-helper.md](config-app-run-helper.md) adds the generated
`scripts/run-with-config.sh` first-run path and
`scripts/package-config-app.sh` source-free package handoff for config apps,
using `MUGA_BIN` and `MUGA_CONFIG_PATH` without shell profile mutation.
That path is now carried by the implemented generated
`muga new --template config-app` starter before TOML, required JSON decoding,
full CLI parser schemas, formatting templates, broader decoder targets, or
broader host effects.
The generated-template adoption path is now carried by
[json-required-decoding.md](json-required-decoding.md), which selected required
`json::decode[T](value)` before TOML, broader decoder target types, full CLI
parser schemas, formatting templates, config discovery, or broader platform
APIs.
[json-required-decoding.md](json-required-decoding.md) defines and implements
that strict decoder with expected `Result[T, json::Error]` target typing,
path-aware missing-field errors, ignored unknown fields, no-fallback schema
lowering, and artifact-safe `DecodeJsonRequired` payloads.
[json-decoder-target-expansion.md](json-decoder-target-expansion.md) implements
the decoder expansion for `Option[T]`, recursive `List[T]`, typed
`Map[String, T]`, and concrete non-generic enums across `json::decode_or[T]`,
`json::decode[T]`, and `config::load_json_or[T]`, with generic decoding,
field/variant schema polish, and TOML still deferred.
The implemented `config_app` sample and generated `config-app` starter carry
the structural config workflow with `Option[String]`, nested records,
`List[Record]`, and typed `Map[String, Int]` settings before TOML, full CLI
parser schemas, formatting templates, config discovery, or broader platform
APIs.
The decoder expansion implements enum JSON/config decoder support, using
zero-payload string tags and one-payload single-key objects before generic enum
decoding, field/variant schema polish, TOML, full CLI parser schemas,
formatting templates, config discovery, or broader platform APIs.
The schema polish implementation in
[json-config-schema-polish.md](json-config-schema-polish.md) supports
`@json(rename: "...")` on record fields and enum variants before aliases,
validation attributes, TOML, full CLI schemas, schema generation, generic
decoding, or broader platform APIs.
The strict unknown-field policy implementation in
[json-config-strict-unknown-fields.md](json-config-strict-unknown-fields.md)
supports record-level `@json(deny_unknown_fields)`, accepted wire-key
semantics, path-aware unknown-key errors, `.mgi` record flags, and `RF` decoder
artifact tokens before aliases, validation attributes, TOML, full CLI schemas,
schema generation, generic decoding, or broader platform APIs.
The alias metadata design in
[json-config-alias-metadata.md](json-config-alias-metadata.md) chooses repeated
`@json(alias: "...")` arguments inside a single field/variant `@json(...)` attribute,
accepted-name conflict checks, strict unknown-field integration, and `RG`/`EG`
artifact tokens before implementation.

### 5. Improve loops, iteration, and collection ergonomics

`while`, `break` / `continue`, and `for item in list` cover the current core loop surface. Future collection ergonomics should now focus on practical data types rather than broadening loop protocols.

Recommended order:

1. Keep `List.contains` and structural collection assertions deferred unless the scalar-only equality policy is deliberately expanded.
2. Broader `Bytes`, `StringBuilder`, and `Buffer` APIs for practical IO and text assembly.
3. `Set[T]` and broader List/Map operations.
4. Map literals only after the parser shape is settled.

Possible map literal shape if added later:

```muga
ages = map {
  "Ada": 20
  "Grace": 30
}
```

Do not overload plain `{ ... }` for maps. Braces already carry block and record-literal meaning.

Delay iterator protocols until ordinary generics, collections, package interfaces, and standard-library examples show a concrete need.

### 6. Add project dependency workflow

After artifact workflows are reliable, make project mode practical:

- default project build artifacts through `muga build` are implemented as `.muga/build`, and `check --built` / `run --built` consume them explicitly
- local path dependency declarations in `muga.toml` are implemented for package roots outside the entry source tree
- package-level unchanged artifact reuse over the local dependency graph is implemented for `muga build`, with CLI output reporting each artifact as `written` or `reused`
- package-local source-hash metadata in `.mgb` implementation artifacts is implemented
- public interface hash stability for implementation-only changes and representative source-span movement is implemented
- parallel package builds over dependency levels are implemented
- local path `muga.lock` metadata plus malformed-lockfile validation/update policy is implemented; full non-local lockfile enforcement remains
- canonical package content hashing over `muga.toml`, sorted `.muga` source files, and optional manifest-declared text/binary resource files is implemented
- deterministic `.mgp` package archive emission, JSON archive/unpack metadata, pasteable archive dependency snippet output, library readback/hash validation, local materialization into absent or empty source/resource trees, local archive dependency cache consumption including declared resources, local archive dependency cache/lockfile edge-case hardening, read-only runtime resource lookup through `std::fs::read_resource_text` and `std::fs::read_resource_bytes`, non-mutating dependency-aware app bundle emission through `muga emit-app-bundle --format json --source-free`, source-free bundle execution through `muga run-app-bundle`, explicit bin-dir launcher plus ownership metadata placement and guarded owned updates/uninstalls through `muga install-app --format json --replace-owned` and `muga uninstall-app --format json`, non-mutating installed-app inventory through `muga list-installed-apps`, generated package-helper install/list hooks through `MUGA_INSTALL_DIR`, source-free app completion package emission through `muga emit-app-completions --format json`, and deterministic `.mga` app archive transport with JSON archive/unpack metadata are implemented; remote package fetching, publishing/install workflows, registries, and signing remain deferred
- full incremental artifact reuse
- better cross-package diagnostics for stale interfaces and caches
- registry/archive/signing design is documented in
  [registry-security-design.md](registry-security-design.md) after the local
  project dependency foundation

Reason: practical reuse requires dependable package loading and cache invalidation before it requires a public registry.

### 7. Build the performance path

The current VM is a reference backend. Practical performance needs compiler work more than surface syntax.

The final performance ambition is explicit: optimized/native Muga should be
able to compete with Rust or C++ on representative programs where value
semantics, static package contracts, and compiler-managed memory are a good
fit. Until the MIR, optimizer, native backend, and benchmark suite exist, this
remains an engineering target rather than a public claim.

The compiler-speed ambition is equally explicit: Muga should target faster
day-to-day compile feedback than Go on representative Muga projects. The first
comparison is not whole-world cold builds; it is the loop users feel most often:
syntax feedback, package checking with loaded interfaces, warm incremental
builds, and later watch/daemon responses. Cold builds still matter, but they
should be measured separately from warm and incremental compiler feedback.

Recommended order:

1. keep syntax/check/build latency visible and preserve fast edit-check-run
   feedback
2. define Go and fast-compiler comparison workloads for cold, warm, and
   incremental checks before making public compile-speed claims
3. add full incremental package/project artifact reuse with precise invalidation
4. use package interfaces and check caches so dependencies are not rechecked by
   body unless their public contract changes
5. consider watch/compiler-daemon workflows only after the artifact model is
   measured
6. control-flow-oriented MIR
7. efficient String/List/Map representations
8. copy elision and destructive-update lowering when a value is uniquely owned
9. escape analysis and stack allocation for non-escaping values
10. inlining and specialization for hot generic or higher-order functions
11. native backend after package and MIR boundaries are stable
12. representative benchmark suites before strict Rust/C++-class public claims

Keep source-level value semantics. Do not introduce `ref T`, `mut ref T`, `&value`, pointer syntax, or borrowing syntax as the ordinary performance answer.

### 8. Add structured concurrency as a core acceleration target

Concurrency is important for practical services. It should now move ahead of
minor polish once the `std::process` spine is complete, but it must stay
structured and explicit rather than becoming hidden async suspension.

Recommended first shape:

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

Rules to preserve:

- child tasks cannot outlive their `group`
- `join()` is explicit
- failure and cancellation are structured
- immutable captures are easy
- mutable captures across task boundaries are rejected or made explicit
- typed channels come after task groups
- do not make `async fn` / `await` the primary model unless later evidence forces that direction

### 9. Add asynchronous IO and service runtime after the task core

Concurrency syntax alone does not make Muga practical for services. After resource handles and task groups exist, the runtime needs a cancellation-aware IO story.

Recommended order:

1. opaque socket, listener, timer, and stream handle types
2. scheduler integration for readiness-based socket IO
3. deadline and timeout APIs that compose with task cancellation
4. nonblocking read/write APIs that return `Result`
5. clear separation between APIs that may block an OS thread and APIs that yield to the Muga scheduler
6. backpressure support through bounded channels, stream APIs, or explicit write readiness
7. graceful shutdown hooks for servers and long-lived task groups
8. benchmarks for large numbers of mostly-idle connections and mixed CPU/IO workloads

The standard-library layering should be:

- resource handles first
- structured task groups second
- typed channels third
- selection/timeouts fourth
- socket and stream APIs once cancellation and scheduling rules are stable
- HTTP/SSE/WebSocket APIs only after the lower layers can express backpressure and shutdown correctly

Avoid building a web framework before these pieces exist. It would force protocol and runtime semantics into ad hoc library behavior instead of stable language and standard-library contracts.

## Features To Keep Out

These should not be implemented for v1, and should not be added later without concrete examples, benchmarks, and package-interface impact analysis.

### Classes and inheritance

Do not add `class`, class-owned methods, instance variables, constructors tied to classes, or inheritance.

Use records for data, functions for behavior, modules for encapsulation, and chained-call syntax for call-site ergonomics.

### Ordinary source-level references

Do not add:

- `ref T`
- `mut ref T`
- `&value`
- `*value`
- pointer arithmetic
- general writable aliases

Use value semantics, internal sharing, builders/buffers, resource handles, and backend optimizations instead.

### Universal null or implicit nullable types

Do not make every `T` implicitly nullable.

Use `Option[T]` for absence. `T?` may remain future shorthand for `Option[T]`, but it should not become a separate nullable type. If optional chaining is added, keep `?.` Option-only and local; it should not propagate `Result` or return from the enclosing function. Use named helpers such as `option::map`, `option::and_then`, and `option::value_or` for general optional value chaining.

### Implicit exceptions

Do not add exception-style control flow as the default recoverable-error model.

Use `Result[T, E]`, exhaustive `match`, and `try expr`. If Result propagation needs to stay inside a dot chain later, use postfix keyword `expr.try`, not postfix `expr?`. Use named `Result` helpers for fluent value chaining rather than propagation.

### Broad protocol/trait/typeclass system in v1

Do not add protocol-like abstractions before generics, enums, collections, package interfaces, and standard-library examples make a real need clear.

If this family is added later:

- prefer the name `protocol`
- keep protocol declarations small
- do not add protocol inheritance, blanket implementations, specialization, protocol objects, or protocol-based dot lookup in the first version

### Overloaded dispatch and operator overloading

Do not add overloaded functions or user-defined operators in v1.

Reason: Muga currently has simple name resolution and stable dot-call meaning. Overloading would make diagnostics, package interfaces, and compile-time behavior more complex.

### Whole-program inference

Do not infer public API meaning from arbitrary downstream call sites.

Allowed:

- infer locally inside a function, module, or package when unique
- store resolved public signatures in package interfaces

Disallowed:

- making a dependency's public signature depend on downstream usage
- implicitly generalizing ordinary declarations into generic functions

### Hidden async suspension

Do not make ordinary calls hide suspension points.

Concurrency should start with structured task scopes and explicit `join()`, not with a second async-colored function world.

### Runtime metaprogramming as a core mechanism

Do not rely on reflection, monkey patching, or dynamic runtime code generation as the normal abstraction path.

Compile-time generation can be considered later, but it should not define the v1 core.

## Reconsideration Rule

Before adding a feature outside the priority order, require all of the following:

1. A concrete user-facing program becomes materially simpler or safer.
2. The feature preserves local readability.
3. The feature has a small parser and typechecker story.
4. The feature can be represented in package interfaces.
5. The feature does not require whole-program inference.
6. The feature does not overload an existing syntax marker with an unrelated meaning.
7. The feature has diagnostics that can be explained clearly.
8. The feature does not undermine value semantics or structured concurrency safety.

When in doubt, prefer a library API, package convention, or explicit function over a new language feature.
