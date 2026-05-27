# Implementation Resume Plan
Status: current implementation ledger and resume handoff. The detailed history now lives in the active progress snapshot, v1 work queue, implementation table, and individual decision documents; this opening section is intentionally short so a maintainer can find the next slice quickly.
Purpose: if prior conversation context is lost, read [docs/strategy-and-implementation-plan.md](strategy-and-implementation-plan.md) for strategic phase order, then read [ROADMAP.md](../ROADMAP.md), then this file. This file records what the repository currently implements, what was verified, and the concrete test plan for the next code slice.
Operational note for agents: when `muga build samples/projects/cli_tool/src/main/main.muga`, a resource-export bundle smoke, or a release gate leaves known sample `muga.lock` files behind, use the dedicated checked script `scripts/trash-generated-muga-locks.sh` instead of ad hoc deletion. Run `scripts/trash-generated-muga-locks.sh --dry-run` first when you only need to confirm cleanup; Git ignores `samples/projects/*/muga.lock`, and known generated locks currently include `samples/projects/cli_tool/muga.lock` and `samples/projects/resource_export/muga.lock`.
## Core Acceleration Route
The current default is Core Capability Acceleration, not another pass over small polish items. Preserve the small v1 language model, package-aware checking, persisted interfaces, explicit check artifacts, package-wide typed HIR aggregation, MIR-backed bytecode generation, and slot-backed runtime locals, then use that foundation to implement practical core capabilities vertically.
Recommended order:
1. Add the `std::process` spine first: child execution, captured status/stdout/stderr, explicit cwd/env, public errors, artifact-backed execution, and samples.
2. Add the structured task spine next, then service IO only after process/resource/task rules can express shutdown and backpressure.
3. Build the performance spine with control-flow MIR evidence before native backend claims.
4. Build the distribution spine from existing `.mgp` / `.mga`, source-free bundle, verification, install inventory, and API-diff foundations.
5. Every accelerated slice must still update docs, tests, artifacts, samples, and release-readiness evidence before commit.
## Active Progress Snapshot
Current strategic phase: Phase 2 package build productization from
[docs/strategy-and-implementation-plan.md](strategy-and-implementation-plan.md).
- [x] `and` / `or` are implemented as Bool-only, left-to-right, short-circuiting keyword operators.
- [x] `else if` is implemented as nested-`if` sugar for statement and value-producing chains.
- [x] `return expr` exits the nearest named or anonymous function and is rejected at top level.
- [x] `break` / `continue` target the nearest loop in the same function and are rejected outside loops.
- [x] `for item in list` iterates `List[T]` with an immutable loop item scoped to the body.
- [x] payload discard `_` works inside qualified one-payload enum variant patterns without broad catch-all matching.
- [x] `muga build <entry>` writes `.mgi` / `.mgc` / `.mgb` artifacts to the default `.muga/build` directory.
- [x] `muga check --built <entry>` and `muga run --built <entry>` consume the default `.muga/build` directory explicitly.
- [x] `[dependencies] name = { path = "..." }` resolves local path dependencies through the dependency manifest's own `name` / `source`.
- [x] `[dependencies] name = { archive = "...", hash = "sha256:<hex>" }` resolves local `.mgp` archive dependencies through `.muga/packages` cache materialization and the dependency manifest's own `name` / `source`.
- [x] `muga build <entry>` preserves unchanged generated `.mgi`, `.mgb`, and `.mgc` artifacts and reports written/reused artifacts through the library API and CLI output.
- [x] `.mgb` implementation artifacts record package-local source hashes separately from interface and dependency interface hashes.
- [x] `.mgi` public interface hashes ignore diagnostic-only spans, so implementation-only body/span movement does not force public interface rewrites.
- [x] `muga build <entry>` builds package artifacts by deterministic dependency levels and runs independent same-level package work concurrently.
- [x] `muga build <entry>` writes/updates minimal local path `muga.lock` metadata with SHA-256 dependency `source_hash` values.
- [x] existing local path `muga.lock` files are parsed and validated before update; well-formed stale metadata is refreshed, while malformed or unsupported lockfiles fail with `PK026`.
- [x] library package content hashing computes `sha256:<hex>` over `muga.toml`, sorted `.muga` files under the manifest source root, and optional manifest-declared resource bytes.
- [x] `muga emit-package-archive [--format text|json] --archive-root <dir> <entry>` writes deterministic `.mgp` source/resource archives whose bytes match the canonical package content input, including binary resources.
- [x] library archive readback validates `.mgp` bytes against optional expected `sha256:<hex>` values, parses manifest/source/resource entries without trusting filenames, preserves resource bytes, and rejects malformed layout, duplicate or unsorted paths, non-UTF-8 manifest/source entries, source/resource-root escapes, undeclared resources, tool metadata, and non-source file entries.
- [x] CLI/library archive materialization writes validated `.mgp` bytes into an absent or empty local source/resource tree, preserves the content hash, rejects unsafe manifest source/resource roots, and rejects non-empty destinations.
- [x] local `.mgp` archive dependencies validate `[dependencies] name = { archive = "...", hash = "sha256:<hex>" }`, materialize or reuse `.muga/packages` cache entries including declared resources, reject malformed archive forms, stale or colliding cache content, and package-name mismatches, and write/validate lockfile `hash` metadata.
- [x] `muga emit-package-archive --dependency-snippet` can print a pasteable `[dependencies]` entry, and the CLI workflow test covers emitting an `.mgp`, using the snippet, running, building, caching, and lockfile emission.
- [x] local project build reuse/stale diagnostics are visible enough for the current v1 workflow: CLI `muga build` reports `written` / `reused` artifact status, and stale/missing check/implementation artifact diagnostics point at package artifact regeneration.
- [x] `muga build` reuse output and lockfile update behavior now have focused CLI coverage for local path and local archive dependencies after dependency implementation-only edits, public signature edits, archive content updates, and malformed lockfiles.
- [x] generic record literals propagate known field expected types into contextual values such as `[]`, `Map.empty()`, and `Option::None`, avoiding misleading ambiguity diagnostics when a binding/parameter/return annotation already fixes the record type arguments.
- [x] stale generic package interface artifact diagnostics include artifact-root context and point at concrete regeneration commands: `muga build`, `muga emit-artifacts`, or `muga emit-interface`.
- [x] full `std::io::IOError` and `std::io::PathPairError` type spellings in source diagnostics now point users to `import std::io` and the local `io::...` alias form.
- [x] invalid `try Result::Ok(...)` placements now keep the primary `T023` placement diagnostic without also emitting a redundant `T021` constructor expected-type diagnostic.
- [x] ambiguous `to_string` receiver diagnostics now suggest annotating the receiver as `Int`, `Bool`, or `String`.
- [x] ambiguous `print` / `println` argument diagnostics now suggest annotating the argument as `Int`, `Bool`, or `String`.
- [x] ambiguous `len` / `is_empty` argument diagnostics now suggest the supported `List[T]`, `Map[K, V]`, or `String` annotations.
- [x] ambiguous list indexing and `for` iterable diagnostics now suggest annotating the value as `List[T]`.
- [x] all current `E005` ambiguity diagnostics now include targeted annotation guidance, including unresolved function signatures and `get` / `contains` / `insert` / `remove` receivers.
- [x] recursive annotation diagnostics now include action suggestions: direct recursion points at a parameter type annotation or explicit return type, and mutual recursion points at explicit signatures for every function in the group.
- [x] CLI usage now separates `check` from `run` program-argument syntax, `run --built` program arguments are covered, and the mini v1 spec reflects implemented `muga build`, local dependency, and local lockfile behavior.
- [x] default `.muga/build` artifact diagnostics reached through `--built` add direct `muga build <entry>` guidance for missing interface artifacts, missing/stale check caches, and missing/stale implementation artifacts.
- [x] artifact-backed `run` has explicit missing/stale `.mgc` check-cache diagnostic coverage for both `--artifact-root` and `--built`, and `.mgc` diagnostics now include `muga emit-check-cache` as a focused regeneration path.
- [x] `.mgb` implementation artifact diagnostics include the concrete artifact file path plus package context for stale interface hashes, stale dependency interface hashes, hash mismatches, and invalid bytecode structure.
- [x] v1 release boundary hardening now has an explicit feature-freeze checklist, sample policy, diagnostic policy, artifact workflow policy, and release gate in `docs/v1-release-checklist.md`.
- [x] post-v1 concurrency design snippets have moved out of runnable `samples/` into `docs/design-snippets/`, so `samples/` remains reserved for runnable entrypoints, their support files, or intentionally invalid sample trees.
- [x] CI and release workflows now run CLI smoke checks for source `check` / `run`, `muga build`, `check --built`, `run --built`, offline package/API-diff verification, and package/app archive verification in addition to fmt, clippy, and tests.
- [x] `tests/release_readiness.rs` and `scripts/v1-release-gate.sh` make release-checklist scope, sample, diagnostic, doc-link, CI, and release-gate readiness verifiable.
- [x] Release version and timing are intentionally separate maintainer decisions; the docs should describe v1 requirements and post-v1 direction without forcing a specific next release.
- [x] Initial `conformance/` suite skeleton is wired into `cargo test` and release readiness, with valid, rejecting, and package artifact workflow fixtures tied to the mini spec and split specs.
- [x] Initial stable diagnostic JSON schema and command-output contract is documented and implemented for `muga check --format json`, while existing human text output remains unchanged.
- [x] `muga check --format json` now includes the entry path and a best-effort absolute `file://` URI so editor, CI, LSP, and agent prototypes can map command diagnostics without scraping CLI arguments.
- [x] Minimal `muga doc` now emits Markdown docs for public package records, enums, functions, and item-level public source comments from the same interface graph used for `.mgi` artifacts.
- [x] `.mgi` API diff now has a library comparator, `muga api-diff` CLI wrapper, persisted public-shape fixtures, and design coverage for input scope, public identity, Compatible / Source-Compatible / Breaking / Unknown classifications, deprecation handling, and JSON output in [mgi-api-diff.md](mgi-api-diff.md).
- [x] Standard-library review rules now define scope, public contract, explicit `Result` effects, `Option` absence, public error types, hidden-IO limits, opaque-resource boundaries, naming, tests, and deferred platform APIs in [standard-library-review-rules.md](standard-library-review-rules.md).
- [x] Minimal `muga test` now discovers compiler-recognized `@test` functions in scripts and packages, validates zero-argument `Unit` / `Result[Unit, E]` test signatures, runs tests through the bytecode runtime, and reports pass/fail summaries.
- [x] `std::test` now provides scalar assertion helpers for `muga test`: `test::assert_true`, `test::assert_eq_int`, `test::assert_eq_bool`, and `test::assert_eq_string`, all returning `Result[Unit, String]`.
- [x] Deterministic `muga fmt` now formats v1 source files, supports CI-friendly `--check`, and preserves line comments instead of dropping them.
- [x] `std::option` and `std::result` now provide narrow value helpers for `is_some` / `is_none`, `is_ok` / `is_err`, `map`, `map_err`, `and_then`, and `value_or`, without adding propagation syntax.
- [x] `std::list` / `std::map` now provide narrow helpers for list transforms, list predicates, folds, and map key/value extraction, without adding iterator protocols or structural equality.
- [x] The v1 equality policy is documented as scalar-only for `Int`, `Bool`, and `String`; structural equality, structural assertions, `List.contains`, and equality-sensitive collection APIs remain deferred.
- [x] Minimal `muga new` now creates app, lib, test, and config app manifest project templates while refusing non-empty targets.
- [x] Initial `muga syntax --format json` now returns single-file lex/parse diagnostics for faster editor feedback without running resolver, typechecker, import loading, or artifact checks.
- [x] Initial `muga metadata --format json` now exposes package/module/item/export metadata plus public interface docs and rendered types for editor, LSP, CI, and agent consumers.
- [x] Initial `muga hover --format json` now returns declaration hover data with public docs and signatures from the package interface model.
- [x] Initial `muga completions --format json` now returns visible package/interface completions with import aliases plus public docs and signatures for editor and LSP consumers.
- [x] Initial `muga definition --format json` now returns go-to-definition data for import aliases, local bindings, and package/interface item references with package/module/item ids and spans.
- [x] Initial `muga references --format json` now returns find references data for import aliases, local bindings, and package/interface item references in the checked entry module.
- [x] Initial `muga workspace --format json` now returns entry-reachable workspace metadata for loaded packages, module source files, default artifact root, and dependency edges.
- [x] CLI JSON compiler diagnostics now attach entry source context in `diagnostics[].context` so editor, LSP, CI, and agent consumers can map each diagnostic directly to the checked entry file.
- [x] Artifact-backed `check --format json` diagnostics now attach entry package, artifact-root, and concrete artifact-file context when available.
- [x] Initial `muga build --format json` artifact status output reports artifact root, artifact kind, path, URI, and written/reused status for `.mgi`, `.mgc`, and `.mgb` build products.
- [x] Initial `muga emit-artifacts --format json`, `muga emit-interface --format json`, and `muga emit-check-cache --format json` output reports explicit artifact root, artifact kind, path, and URI data.
- [x] Artifact diagnostics now expose structured dependency hash, source hash, artifact hash, and regeneration-command context where that data is already computed; the follow-up audit now covers `.mgb` dependency-interface set changes in both `run` diagnostics and `why-rebuild --format json`.
- [x] Initial `muga test --format json` output reports structured test results, failure messages, captured per-test stdout/stderr, summary counts, and pre-run compiler diagnostics.
- [x] Initial `muga run --format json` output reports captured program stdout/stderr, returned `main` values, and compiler/runtime diagnostics.
- [x] `samples/projects/report_app` and `samples/projects/report_shared` now provide a runnable manifest project sample that combines a local path dependency, text-file IO, `Result` error handling, reusable public APIs, and artifact-backed execution coverage.
- [x] Initial `muga explain <diagnostic-code>` output prints exact `errors.md` catalog entries or stable diagnostic-code family guidance.
- [x] `docs/editor-json-workflow.md` and `json_backed_editor_workflow_uses_existing_command_contracts` now validate a concrete editor adapter flow across syntax, check, workspace, metadata, hover, completions, definition, references, run, and test JSON without scraping human output.
- [x] Artifact/cache explanation output is designed in
  [artifact-cache-explanations.md](artifact-cache-explanations.md) for a future
  non-mutating `muga why-rebuild --format json` command.
- [x] Initial read-only `muga why-rebuild --format json` output explains
  `.mgi` / `.mgc` / `.mgb` artifact states without mutating artifacts, with
  focused fresh, missing, stale source/dependency-interface, hashMismatch,
  invalid, `--built`, and explicit `--artifact-root` CLI coverage.
- [x] `muga why-rebuild --format json` now explains manifest `muga.lock`
  metadata for local path and local `.mgp` archive dependencies without
  rewriting the lockfile.
- [x] `muga why-rebuild --format json` now explains verified local `.mgp`
  archive cache metadata under `.muga/packages`.
- [x] `muga why-rebuild` now has compact human text output for artifact,
  lockfile, and archive-cache states while keeping machine consumers on
  `--format json`.
- [x] Runtime diagnostics now add `related` call-context notes for nested
  function call sites and the entry or test function being executed, and both
  `muga run --format json` and `muga test --format json` preserve those notes.
- [x] Failed `std::test` scalar assertions now add `R021` diagnostics with a
  primary span at the user assertion call while preserving the existing
  `tests[].message` failure string.
- [x] Runtime/debug reporting v1 follow-up is closed: runtime stack context is
  represented by `diagnostics[].related` call-context notes, failed scalar
  assertions use source-spanned `R021` diagnostics, and package/artifact
  next-actions use existing `regenerationCommand` context instead of adding a
  separate stack-trace schema.
- [x] Release-neutral benchmark health checks now run through
  `scripts/benchmark-health-check.sh` and ignored `tests/benchmark_health.rs`
  coverage for compiler stages, package artifact reuse, and representative
  String/List/Map runtime paths without public performance claims.
- [x] Fuzzing and malformed-input planning now covers parser/syntax, package
  archive `.mgp`, local `muga.lock`, package interface `.mgi`, check-cache
  `.mgc`, and implementation artifact `.mgb` trust boundaries in
  [fuzzing-malformed-input-plan.md](fuzzing-malformed-input-plan.md).
- [x] Installation and onboarding docs now cover `cargo install`, local checkout
  installs, `muga --version`, generated-project quickstarts, and later
  binary-release expectations in
  [installation-and-onboarding.md](installation-and-onboarding.md).
- [x] "Muga by Example" now orders existing runnable examples from bindings and
  records through `Result`, packages, tests, local dependencies, and
  artifact-backed builds in [muga-by-example.md](muga-by-example.md).
- [x] Registry security design now preserves the `.mgp` hash foundation and
  scopes future signing, provenance, lockfile enforcement, cache validation,
  and malicious-package handling in
  [registry-security-design.md](registry-security-design.md).
- [x] Edition and semantic feature-set fingerprint policy now defines how future
  package artifacts, cache keys, lockfiles, diagnostics, and API diffing should
  account for source-meaning changes in
  [edition-feature-fingerprint-policy.md](edition-feature-fingerprint-policy.md).
- [x] Package-mode public signatures now have representative round-trip coverage
  for every v1-supported public type shape through in-memory and persisted
  interfaces, including same-package and imported public type identities.
- [x] The stdlib package docs and samples review now covers `std::io`,
  `std::fs`, `std::path`, `std::env`, `std::cli`, `std::time`, `std::string`,
  `std::fmt`, and the first `std::json` slice, including runnable samples and artifact-backed execution samples where useful in
  [stdlib-package-samples-review.md](stdlib-package-samples-review.md).
- [x] The release gate and GitHub Actions are aligned in
  [release-gate-alignment.md](release-gate-alignment.md): CI invokes
  `scripts/v1-release-gate.sh`, and the release workflow invokes
  `scripts/v1-release-gate.sh --with-publish-dry-run` before publishing.
- [x] Minimal shell completions and `muga doctor` are implemented as a
  tool-only adoption surface.
- [x] The first `std::json` package contract from
  [std-json-first-slice.md](std-json-first-slice.md) is implemented with
  `parse`, `encode`, `number_as_int`, and `int`, while keeping schema
  generation, HTTP/RPC, `Float`, `Decimal`, `Bytes`, streaming APIs, and
  resource handles deferred.
- [x] The implemented first `std::json` slice is audited in
  [std-json-implementation-audit.md](std-json-implementation-audit.md), with
  added evidence for string escaping, error offsets, invalid raw-number
  encoding, parse/encode nesting limits, and artifact-backed execution.
- [x] The post-JSON standard-library boundary selection is recorded in
  [post-json-stdlib-boundary-selection.md](post-json-stdlib-boundary-selection.md):
  design opaque resource handles before adding stdout/stderr handles, file
  handles, process APIs, HTTP/SSE/WebSocket/RPC, streaming APIs, `Bytes`,
  buffers, or schema/client generation.
- [x] The opaque resource-handle boundary is designed in
  [opaque-resource-handles.md](opaque-resource-handles.md), covering future
  `pub opaque type`, `.mgi` identity, API diff classification, capability
  defaults, consuming operations, explicit close, task-boundary/cancellation
  rules, runtime diagnostics, and non-goals.
- [x] The first `pub opaque type` interface slice is implemented: package-mode
  public opaque type declarations have parser/AST support, package item
  identity, nominal `TypeInfo`, `.mgi` persistence, editor/doc tooling exposure,
  downstream loaded-interface checking, and rejecting coverage for construction,
  field access, match, equality, formatting, non-public declarations, and type
  arguments.
- [x] The runtime-backed opaque handle metadata boundary is designed in
  [opaque-resource-handles.md](opaque-resource-handles.md): add
  `OpaqueHandleFacts`, consuming parameter modes, explicit close metadata,
  `.mgi` persistence/hash/API-diff rules, use-after-consume diagnostics, and a
  small future `std::fs::File` text-handle candidate before implementing handle
  values.
- [x] The metadata-only interface slice is implemented: `.mgi` v5 persists
  `OpaqueHandleFacts`, close-function identity, and function-parameter
  `paramMode`; legacy `.mgi` files default to conservative facts and `borrow`;
  public interface hashes include the metadata; and `muga metadata`, hover,
  completions, and docs expose it without source syntax or runtime handle
  values.
- [x] The consuming-parameter dataflow checker is implemented for loaded
  interface metadata: direct same-scope uses after passing a binding to a
  `consume` parameter now report `T026` with a related note at the consuming
  call, covered by a synthetic loaded-interface fixture.
- [x] The first runtime-backed `std::fs::File` handle implementation boundary is
  designed: the initial slice is read-only `open_text` / `read_text_from` /
  `close`, uses a VM-local `{family, slot, generation}` handle table, keeps live
  handles out of artifacts, maps stale/wrong-family/double-close to hard runtime
  diagnostics, and leaves write modes/cursors/truncation deferred.
- [x] The first read-only runtime-backed `std::fs::File` implementation is
  landed: `open_text`, `read_text_from`, and consuming `close` use VM-local
  runtime slots, expose `runtimeBacked` / `closeFunction` / `consume` metadata
  through `.mgi`, run from source and `.mgb` artifacts, return recoverable
  `io::IOError` values for host IO failures, and report stale/closed handle
  aliases as hard `R022` runtime diagnostics.
- [x] The post-file-handle resource-surface selection is recorded in
  [post-file-handle-resource-surface-selection.md](post-file-handle-resource-surface-selection.md):
  it chose a program stderr channel through scalar `eprint` / `eprintln`,
  explicitly not stdout/stderr handles; that channel is now implemented, while
  text file write handles, binary `Bytes`, buffering, async IO, streams,
  process APIs, and network APIs remain deferred behind fresh contracts.
- [x] The program stderr output channel is implemented through scalar prelude
  `eprint` / `eprintln`, a separate runtime stderr buffer, text-mode `run`
  stderr emission, and `run` / `test` JSON `stderr` fields.
- [x] The text output file handle design is recorded in
  [text-output-file-handles.md](text-output-file-handles.md): keep one public
  `std::fs::File` type with runtime `Read` / `Write` / `Append` modes, add
  `create_text`, `append_text`, `write_text_to`, and `flush`, keep only
  `close` consuming, report wrong-mode operations as recoverable `io::IOError`
  values, and keep `Bytes`, streams, standard-stream handles, process APIs, and
  async IO deferred.
- [x] The text output file handle implementation is landed: `std::fs::File`
  runtime slots now track `Read` / `Write` / `Append`, `create_text`,
  `append_text`, `write_text_to`, and `flush` are implemented, wrong-mode
  reads/writes return recoverable `io::IOError` values, `close` flushes writable
  handles before closing, only `close` consumes, and source plus artifact-backed
  execution are covered.
- [x] The practical `report_app` workflow sample now demonstrates args/env,
  stdout/stderr, text-file handle writes, JSON run output, `Result`, tests,
  local dependencies, artifact-backed execution, and `run --built` together.
- [x] The practical `report_app` workflow exposed the resource-cleanup gap; the
  active contract moved into [lexical-resource-cleanup.md](lexical-resource-cleanup.md)
  before `Bytes`, formatting templates, stdout/stderr handles, process APIs,
  network APIs, streaming APIs, or broader host effects.
- [x] The lexical resource cleanup design is recorded in
  [lexical-resource-cleanup.md](lexical-resource-cleanup.md): `using` is a
  statement over runtime-backed opaque handles, cleanup failure wins when both
  body and cleanup fail, explicit close of the managed binding is rejected, and
  source/artifact/`run --built` coverage is required before refreshing
  `report_app`.
- [x] First-slice `using` lexical cleanup is implemented and hardened for
  nested resources: acquisition/body control transfers and cleanup-error
  branches attempt every active cleanup in LIFO order, while the first cleanup
  error remains the returned error until aggregate cleanup errors are designed.
- [x] Minimal pure `std::cli` helpers over explicit `List[String]` values are
  covered by the std package source, `std_cli` sample, `report_app`, and
  examples tests before `Bytes`, formatting templates, process APIs, network
  APIs, streaming APIs, or broader host effects.
- [x] The first `std::cli` helper slice is implemented as a pure compiler-
  provided package: `positional`, `positional_or`, `has_flag`, `option`, and
  `option_or` cover long flags/options and `--` termination over explicit
  argument lists, with source/artifact coverage and `report_app` refreshed.
- [x] The CLI-first app template and typed scalar `std::cli` parsing helpers
  are implemented after the post-`std::cli` and post-template adoption
  selections; generated apps now demonstrate `std::env` / `std::cli`, and
  `std::cli` can parse `Int` / `Bool` positional and option values through
  `Result`.
- [x] The post-typed-cli implementation path is now represented by code and
  coverage: JSON value accessor helpers are implemented in `std::json` as the
  next practical API boundary before full CLI parser schemas, config-file
  loading, `Bytes`, formatting templates, process APIs, network APIs, streams,
  or broader host effects.
- [x] The selected `std::json` value and object-field accessor helpers are
  implemented with source and artifact-backed coverage, keeping missing fields
  non-errors and wrong shapes as `json::Error` values.
- [x] The post-json-accessor implementation path is now represented by
  `samples/projects/config_app` and coverage: the JSON config workflow sample
  composes existing `std::config`, `std::path`, `std::json`, `std::env`, and
  `std::cli` before adding TOML, broader schema tooling, full CLI parser
  schemas, `Bytes`, process APIs, network APIs, streams, or broader host
  effects.
- [x] The selected JSON config workflow sample is implemented in
  `samples/projects/config_app`, with source, emitted-artifact, config
  shape-error, and `run --built --format=json` coverage for explicit CLI >
  config > defaults precedence.
- [x] The post-config-workflow adoption gap selection is recorded in
  [post-config-workflow-adoption-gap-selection.md](post-config-workflow-adoption-gap-selection.md):
  refresh `config_app` to use existing `std::result::map_err` for app-boundary
  error normalization before adding a `std::config` package, TOML, schema
  decoding, full CLI parser schemas, formatting templates, `Bytes`, process
  APIs, network APIs, streams, or broader host effects.
- [x] The selected `std::result::map_err` config workflow refresh is
  implemented: `config_app` now maps IO and JSON errors with the existing
  `std::result` helper and no longer carries local one-off JSON result
  wrappers, while preserving source, artifact-backed, config shape-error, and
  `run --built --format=json` coverage.
- [x] The post-result-mapping implementation path is now represented by code
  and coverage: narrow pure `std::string` text assembly helpers landed before
  formatting templates, interpolation, builders, broader config/schema work,
  full CLI parser schemas, `Bytes`, process APIs, network APIs, streams, or
  broader host effects.
- [x] The selected first `std::string` text assembly helper slice is
  implemented: `std::string` now exposes pure `concat_all` and `join` helpers
  over explicit `List[String]` values, `samples/packages/app/std_string`
  covers the public surface, `config_app` uses the helpers for app text
  assembly, and source plus artifact-backed tests cover the package.
- [x] The post-string-assembly implementation path is now represented by code
  and coverage: narrow pure `std::json` required scalar object-field helpers
  landed before broader `std::config`, TOML, schema tooling, full CLI parser
  schemas, formatting templates, interpolation, `std::fmt`, builders, `Bytes`,
  process APIs, network APIs, streams, or broader host effects.
- [x] The selected `std::json` required object-field helper slice is
  implemented: `object_string_required`, `object_int_required`, and
  `object_bool_required` now return `json::Error` for missing required fields,
  the `std_json` sample demonstrates required and defaulted fields together,
  and source plus artifact-backed tests cover the helper surface.
- [x] The post-required-json-field implementation path is now represented by
  code and coverage: narrow pure `std::json` array/object field helpers landed
  before JSON paths, broader schema/config work, TOML, full CLI parser schemas,
  formatting templates, `Bytes`, process APIs, network APIs, streams, or
  broader host effects.
- [x] The selected `std::json` composite object-field helper slice is
  implemented: `object_array`, `object_array_or`, `object_array_required`,
  `object_object`, `object_object_or`, and `object_object_required` now mirror
  the scalar object-field helper family for nested JSON data, with source,
  sample, docs/spec, release-readiness, and artifact-backed coverage.
- [x] The post-composite-json-field implementation path is now represented by
  `samples/projects/config_app` and coverage: the config sample uses
  composite/typed JSON helpers for nested settings before adding JSON paths,
  broader schema decoding, `std::config` expansion, TOML, full CLI parser
  schemas, formatting templates, `Bytes`, process APIs, network APIs, streams,
  or broader host effects.
- [x] The selected nested JSON config workflow refresh is implemented:
  `samples/projects/config_app` now includes nested `tags` / `metadata`
  settings, extracts them through `std::json` composite object-field helpers,
  keeps scalar CLI overrides explicit, and preserves source, artifact-backed,
  composite shape-error, and `run --built --format=json` coverage.
- [x] The post-nested-json-config implementation path is now represented by
  code and coverage: pure `std::json` scalar array projection helpers landed
  before adding JSON paths, schema decoding, broader object-field matrices,
  `std::config` expansion, TOML, full CLI parser schemas, formatting templates,
  `Bytes`, process APIs, network APIs, streams, or broader host effects.
- [x] The selected `std::json` scalar array projection helper slice is
  implemented: `array_strings`, `array_ints`, and `array_bools` convert
  `List[json::Value]` into typed scalar lists with index-specific
  `json::Error` values, and `config_app` now projects `tags` into
  `List[String]`.
- [x] The post-json-array-projection implementation path is now represented by
  code and coverage: direct `std::json` scalar-array object-field helpers
  landed before JSON paths, schema decoding, `std::config` expansion, TOML,
  full CLI parser schemas, formatting templates, `Bytes`, process APIs,
  network APIs, streams, or broader host effects.
- [x] The selected direct `std::json` scalar-array object-field helper slice is
  implemented: `object_string_array*`, `object_int_array*`, and
  `object_bool_array*` compose object lookup with typed scalar-list projection,
  return field-aware index diagnostics, and let `config_app` read `tags` through
  `json::object_string_array_or`.
- [x] The post-direct-json-array-field implementation path is now represented by
  code and coverage: repeated `std::cli` option value helpers let
  JSON/default list settings be overridden from explicit CLI arguments before
  JSON paths, schema decoding, `std::config`, TOML, full CLI parser schemas,
  formatting templates, `Bytes`, process APIs, network APIs, streams, or
  broader host effects.
- [x] The selected repeated `std::cli` option value helper slice is implemented:
  `option_values` and `option_values_or` collect repeated long-option string
  values in encounter order, preserve `--` termination and missing-value skip
  behavior, and let `config_app` override JSON/default `tags` from repeated
  `--tag` values.
- [x] The post-repeated-cli-option implementation path is now represented by
  code and coverage: pure `std::json` path helpers landed before schema
  decoding, `std::config`, TOML, full CLI parser schemas, formatting templates,
  `Bytes`, process APIs, network APIs, streams, or broader host effects.
- [x] The selected `std::json` path helper slice is implemented:
  `PathSegment`, `at`, and `at_required` traverse nested object fields and
  array indexes, report path-aware missing/wrong-shape errors, and refresh the
  `std_json` and `config_app` samples without adding a string path parser,
  JSONPath, schema decoding, `std::config`, TOML, or host effects.
- [x] The post-json-path implementation path is now represented by code and
  coverage: typed JSON path scalar projection helpers landed before typed
  array/object path helpers, schema decoding, `std::config`, TOML, full CLI
  parser schemas, formatting templates, `Bytes`, process APIs, network APIs,
  streams, or broader host effects.
- [x] The selected typed JSON path scalar projection helper slice is
  implemented: `at_string*`, `at_int*`, and `at_bool*` preserve
  optional/default/required missing-path behavior, report path-aware terminal
  scalar errors, and let `config_app` read nested metadata owner with
  `json::at_string_or`.
- [x] The post-typed-json-path-scalar implementation path is now represented by
  code and coverage: typed JSON path collection projection helpers landed before
  schema decoding, `std::config`, TOML, full CLI parser schemas, generated
  config app templates, formatting templates, `Bytes`, process APIs, network
  APIs, streams, or broader host effects.
- [x] The selected typed JSON path collection projection helper slice is
  implemented: `at_array*`, `at_object*`, `at_string_array*`, `at_int_array*`,
  and `at_bool_array*` preserve optional/default/required missing-path behavior,
  report path-aware terminal collection and scalar-array item errors, and let
  the `std_json` sample read nested labels with `json::at_string_array_required`.
- [x] The post-typed-json-path-collection implementation path is now represented
  by the JSON schema decoding design and coverage before adding `std::config`,
  TOML, full CLI parser schemas, generated config app templates, formatting
  templates, `Bytes`, process APIs, network APIs, streams, or broader host
  effects.
- [x] The selected JSON schema decoding design slice is complete before
  implementing `json::decode`, `std::config`, TOML, full CLI parser schemas,
  generated config app templates, formatting templates, `Bytes`, process APIs,
  network APIs, streams, or broader host effects.
- [x] JSON schema decoding design is documented in
  [json-schema-decoding.md](json-schema-decoding.md): select a compiler-owned
  `json::decode_or[T](value, fallback)` default-overlay decoder as the first
  implementation, with `.mgi` record schema input, nested path diagnostics,
  supported first target types, and explicit non-goals.
- [x] The selected `json::decode_or[T]` implementation slice is complete:
  direct calls lower to an artifact-serialized decoder schema, runtime decodes
  concrete supported records/scalars/lists/maps with path diagnostics, and
  `samples/projects/config_app` now uses `json::decode_or(config,
  default_settings())`.
- [x] The post-JSON-schema-decoder implementation path is now represented by the
  `std::config` JSON default loading design and coverage before implementing
  required `json::decode[T]`, TOML, full CLI parser schemas, generated config
  app templates, formatting templates, `Bytes`, process APIs, network APIs,
  streams, or broader host effects.
- [x] The selected `std::config` JSON default loading design is documented in
  [std-config-json-loading.md](std-config-json-loading.md): implement
  `config::load_json_or[T](path, fallback)` with a public `config::Error`
  shape, compiler-lowered decoder schemas, artifact-backed execution, and a
  focused `config_app` refresh before TOML, required `json::decode[T]`,
  generated config app templates, full CLI parser schemas, or broader host
  effects.
- [x] The selected `std::config` JSON default loader is implemented: direct
  `config::load_json_or[T](path, fallback)` calls lower with schema payloads,
  runtime read/parse/decode failures return public `config::Error` values,
  implementation artifacts persist `LoadJsonConfig`, and `config_app` now uses
  the API with source, artifact-backed, config shape-error, and `run --built
  --format=json` coverage.
- [x] Audit `std::config::load_json_or[T]` adoption and
  choose the next config/API boundary before TOML, required `json::decode[T]`,
  generated config app templates, full CLI parser schemas, formatting
  templates, `Bytes`, process APIs, network APIs, streams, or broader host
  effects.
- [x] The post-`std::config` JSON loader implementation path is now represented
  by the generated config app template and coverage before TOML, required
  `json::decode[T]`, full CLI parser schemas, formatting templates, broader
  decoder targets, or broader host effects.
- [x] The generated config app template is implemented: `muga new --template config-app`
  writes `src/main/main.muga` plus `config/settings.json`, uses
  `std::config::load_json_or[T]`, typed/repeated CLI overrides, public config
  error mapping, and source/artifact/`run --built` coverage.
- [x] Generated config app template adoption is audited before TOML, required
  `json::decode[T]`, full CLI parser schemas, formatting templates, broader
  decoder target types, `Bytes`, process APIs, network APIs, streams, or
  broader host effects.
- [x] The post-generated-config-app-template implementation path is now
  represented by the required `json::decode[T](value)` design and coverage
  before TOML, broader decoder target types, full CLI parser schemas, formatting
  templates, config discovery, `Bytes`, process APIs, network APIs, streams, or
  broader host effects.
- [x] Required `json::decode[T](value)` design is complete with expected target
  type policy, missing-field diagnostics, schema payloads, artifact behavior,
  non-goals, and focused test expectations.
- [x] Required JSON decoding is implemented from
  [json-required-decoding.md](json-required-decoding.md): strict
  `json::decode[T](value)` now uses expected `Result[T, json::Error]` target
  policy, missing-field record diagnostics, no-fallback schema lowering,
  artifact payload preservation through `DecodeJsonRequired`, and focused
  source/artifact/`run --built` coverage before TOML, broader decoder targets,
  full CLI parser schemas, or broader host effects.
- [x] Strict `json::decode[T](value)` adoption is now represented by
  [json-decoder-target-expansion.md](json-decoder-target-expansion.md):
  broader JSON decoder target support is designed and implemented before TOML,
  full CLI parser schemas, formatting templates, config discovery, `Bytes`,
  process APIs, network APIs, streams, or broader host effects.
- [x] Broader JSON decoder target support is designed and implemented in
  [json-decoder-target-expansion.md](json-decoder-target-expansion.md):
  structural `Option[T]`, recursive `List[T]`, typed `Map[String, T]`, and
  concrete non-generic enum decoding now work across `json::decode_or[T]`,
  strict `json::decode[T]`, and `config::load_json_or[T]` before generic
  decoding, TOML, full CLI parser schemas, formatting templates, config
  discovery, `Bytes`, process APIs, network APIs, streams, or broader host
  effects.
- [x] Structural JSON decoder adoption is now represented by the implemented
  `config_app` sample and generated `muga new --template config-app` starter:
  the practical config workflow uses typed optional, list, nested record, and
  map settings before TOML, full CLI parser schemas, formatting templates,
  config discovery, `Bytes`, process APIs, network APIs, streams, or broader
  host effects.
- [x] Structural config workflow refresh is implemented: `samples/projects/config_app`,
  `muga new --template config-app`, generated `config/settings.json`, docs, and
  tests now use structural typed settings (`Option[String]`, nested records,
  `List[Record]`, typed `Map[String, Int]`) instead of manual `json::Value`
  metadata access while preserving CLI > config > defaults behavior.
- [x] The refreshed structural config workflow is now represented by
  [json-decoder-target-expansion.md](json-decoder-target-expansion.md):
  concrete enum JSON/config decoding is implemented before TOML, full CLI
  parser schemas, formatting templates, config discovery, or host APIs.
- [x] Concrete enum JSON/config decoding is implemented
  across `json::decode_or[T]`, strict `json::decode[T]`, and
  `config::load_json_or[T]`, including zero-payload string tags, one-payload
  single-key objects, path-aware diagnostics, schema/artifact payloads, and
  source/config/artifact/`run --built` coverage.
- [x] Enum decoder adoption is now represented by
  [json-config-schema-polish.md](json-config-schema-polish.md): field and
  variant wire names, diagnostics, artifact payloads, and package-interface
  compatibility are designed and implemented before TOML, full CLI parser
  schemas, formatting helpers, config discovery, or host APIs.
- [x] JSON/config schema polish is designed and implemented through the first
  rename slice in [json-config-schema-polish.md](json-config-schema-polish.md):
  `@json(rename: "...")` on record fields and enum variants now works before
  aliases, validation attributes, TOML, full CLI schemas, schema generation,
  generic decoding, or host APIs.
- [x] Post-rename JSON/config adoption is now represented by
  [json-config-strict-unknown-fields.md](json-config-strict-unknown-fields.md):
  strict unknown-field policy is designed and implemented before aliases,
  validation attributes, TOML, full CLI schemas, schema generation, generic
  decoding, or host APIs.
- [x] JSON/config strict unknown-field policy is designed and implemented in
  [json-config-strict-unknown-fields.md](json-config-strict-unknown-fields.md):
  record-level `@json(deny_unknown_fields)` now rejects unexpected JSON/config
  object keys with path-aware errors while preserving permissive behavior for
  unannotated records.
- [x] Post-strict JSON/config adoption is now represented by
  [json-config-alias-metadata.md](json-config-alias-metadata.md): alias
  metadata is designed and implemented before validation attributes, TOML, full
  CLI schemas, schema generation, generic decoding, or host APIs.
- [x] JSON/config alias metadata is designed and implemented in
  [json-config-alias-metadata.md](json-config-alias-metadata.md): field and
  enum-variant aliases extend accepted JSON/config names for migration while
  preserving canonical Muga field and variant names across source, package
  interfaces, `.mgb` artifacts, strict unknown-field checks, and runtime
  ambiguity diagnostics.
- [x] Post-alias JSON/config adoption is covered by
  [json-config-validation-attributes.md](json-config-validation-attributes.md):
  validation attribute design and implementation were selected before TOML,
  full CLI parser schemas, schema/client generation, generic decoding, or host
  APIs.
- [x] JSON/config validation attributes are designed in
  [json-config-validation-attributes.md](json-config-validation-attributes.md):
  field-level `@validate(...)` metadata was selected with scalar string/int
  validators, path-aware validation errors, `.mgi` v8 planning, and `RV`
  artifact planning.
- [x] JSON/config validation attributes are implemented in
  [json-config-validation-attributes.md](json-config-validation-attributes.md):
  parser/formatter, typing, typed HIR, package signatures, `.mgi` v8
  interfaces, `RV` decoder artifacts, runtime `json::ErrorKind::Validation`,
  config decode mapping, source/artifact/`run --built` coverage, docs, and
  release readiness are wired.
- [x] Post-validation JSON/config adoption is covered by
  [json-config-schema-export.md](json-config-schema-export.md): JSON/config
  schema export design and implementation were selected before TOML, full CLI
  parser schemas, full client generation, generic decoding, JSON encoding,
  broader validators, or host-effect APIs.
- [x] JSON/config schema export is designed in
  [json-config-schema-export.md](json-config-schema-export.md): JSON Schema
  Draft 2020-12 plus Muga `x-muga` extensions, a focused
  `muga schema --format json` command, required/overlay decode modes, concrete
  public record/enum scope, and type/attribute mappings were selected.
- [x] JSON/config schema export is implemented in
  [json-config-schema-export.md](json-config-schema-export.md):
  `muga schema --format json` renders JSON Schema Draft 2020-12 documents with
  Muga `x-muga` extensions, required/overlay decode modes, validation keywords,
  alias metadata, concrete record/enum package/type selection, and
  loaded-interface package coverage.
- [x] Post-schema-export JSON/config adoption is covered by
  [json-typed-encoding.md](json-typed-encoding.md): typed JSON encoding was
  selected and implemented before TOML, full CLI parser schemas, full client
  generation, generic encoding/decoding, broader validators, or host-effect
  APIs.
- [x] Typed JSON encoding is implemented in
  [json-typed-encoding.md](json-typed-encoding.md): compiler-owned
  `json::to_value[T](value)` plus `json::encode_typed[T](value)`, canonical
  primary wire-name output, omitted optional record fields, enum output matching
  decode/schema export, validation-on-encode, artifact schema behavior, and
  explicit source/interface coverage are implemented.
- [x] Post-typed-JSON-encoding adoption is covered by
  [cli-parser-schema.md](cli-parser-schema.md): full CLI parser schema design
  was selected before TOML, full client generation, generic encoding/decoding,
  broader validators, config discovery automation, or host-effect APIs.
- [x] Full CLI parser schemas are designed in
  [cli-parser-schema.md](cli-parser-schema.md): a compiler-owned
  `cli::parse_or[T](args, defaults)` overlay parser, paired
  `cli::usage_for[T](program, defaults)`, public `cli::Error`, concrete record
  target scope, supported/preserved field policy, argument semantics,
  validation, usage text, artifacts, and explicit deferrals were selected.
- [x] The first CLI parser schema implementation is landed: `std::cli` exposes
  `ErrorKind`, `Error`, compiler-owned `cli::parse_or[T](args, defaults)`, and
  `cli::usage_for[T](program, defaults)` for concrete non-generic record
  overlays. The first runtime slice parses long options for scalar,
  `Option`-scalar, scalar-list, and zero-payload enum fields, preserves
  unsupported fields from defaults, reuses validation metadata, carries schema
  payloads through typed HIR, MIR, bytecode, `.mgb` artifacts, explicit
  artifact roots, and `run --built`, and keeps TOML, combined short flags, attached values, subcommands,
  strict required parsing, config discovery, and full client generation
  deferred.
- [x] The post-CLI-parser adoption gap is covered by generated `config-app`
  adoption: refresh `samples/projects/config_app` and
  `muga new --template config-app` to use `cli::parse_or[T]` for settings
  overlays while keeping config-path lookup explicit.
- [x] Generated `config-app` CLI schema adoption is implemented: the runnable
  sample and project template now keep `--config` lookup explicit, filter that
  app-level option out of settings args, and call `cli::parse_or[T]` for typed
  settings overlays with simple `cli::Error` string mapping.
- [x] The post-config-app CLI schema adoption gap is covered by generated
  `config-app` usage adoption: expose usage with `cli::usage_for[T]` before
  TOML, `@cli(...)`, dedicated `CliSchema`, config discovery automation, strict
  no-default parsing, full client generation, generic encoding/decoding,
  broader validators, or host-effect APIs.
- [x] Generated config-app usage adoption is implemented: the runnable sample
  and project template expose `--help`, render settings options with
  `cli::usage_for[T]`, append the explicit app-level `--config` option, and
  skip config loading/settings parsing on the help path.
- [x] The post-config-app usage adoption gap is covered by
  [cli-field-metadata.md](cli-field-metadata.md): design first `@cli(...)`
  field metadata before TOML, config discovery automation, strict no-default
  parsing, dedicated `CliSchema` implementation, full client generation,
  generic encoding/decoding, broader validators, or host-effect APIs.
- [x] The first `@cli(...)` field metadata design is recorded in
  [cli-field-metadata.md](cli-field-metadata.md): field-level
  `@cli(name: "...", alias: "...", help: "...", hidden)` plus a dedicated
  `CliSchema` implementation boundary before TOML, config discovery automation,
  strict no-default parsing, full client generation, generic encoding/decoding,
  broader validators, or host-effect APIs.
- [x] The first `@cli(...)` field metadata implementation is landed:
  `@cli(name: "...", alias: "...", help: "...", hidden)` now flows through
  parser/formatter, typing, package interfaces, typed HIR, MIR, bytecode,
  `.mgb` artifacts, runtime parse/usage behavior, source/artifact tests, and
  docs via a dedicated `CliSchema`.
- [x] Generated config-app CLI metadata adoption is implemented: the runnable
  sample and project template now use `@cli(...)` field help, singular
  `--tag`, and `--tags` as a compatibility alias while preserving explicit
  `--config`, CLI > config > defaults precedence, source/artifact behavior,
  and app-boundary error strings.
- [x] Rust lint hardening is implemented: `Cargo.toml` now denies Rust warnings
  and the enforced Clippy groups, `clippy.toml` pins the MSRV, the release gate
  routes through `scripts/clippy-check.sh`, current Clippy violations are fixed,
  and `tests/release_readiness.rs` keeps the script, docs, and lint policy
  aligned.
- [x] CLI/schema regression coverage is expanded: `samples/projects/config_app`
  and the generated `config-app` template export public settings records for
  JSON Schema, schema export is covered for both the sample and generated
  template, and `samples/packages/app/std_cli_schema/main.muga` adds runnable
  and artifact-backed coverage for `@cli(...)`, JSON rename fallback,
  validation, enum parsing, hidden fields, and generated usage.
- [x] The post-config-app CLI metadata adoption gap is covered by
  [strict-cli-parser-schema.md](strict-cli-parser-schema.md): design strict
  `cli::parse[T](args)` before TOML, config discovery automation, combined
  short flags, attached values, subcommands, full client generation, generic
  encoding/decoding, broader validators, or host-effect APIs.
- [x] Strict CLI parser schema design is recorded in
  [strict-cli-parser-schema.md](strict-cli-parser-schema.md): implement
  compiler-owned `cli::parse[T](args)` with expected-result type inference,
  `MissingArgument`, absent `Bool`/`Option`/`List` synthesis, strict target
  validation, and existing `CliSchema` artifact reuse before TOML, config
  discovery automation, combined short flags, attached values, subcommands, or no-default usage helpers.
- [x] Strict CLI parser schema implementation is complete: `std::cli` exposes
  compiler-owned `cli::parse[T](args)`, typing derives `T` from an expected
  `Result[T, cli::Error]`, runtime returns `MissingArgument` for absent required
  fields, `Bool`/`Option`/`List` fields synthesize absent values, and
  source/artifact/`run --built` coverage preserves the existing `CliSchema`
  payload path without adding no-default usage helpers.
- [x] Post-strict CLI parser adoption gap selection is retired into code
  evidence: `samples/projects/cli_tool` implements the strict CLI sample before
  TOML, config discovery automation, full client generation, generic
  encoding/decoding, broader validators, or host-effect APIs.
- [x] Strict CLI tool sample adoption is complete:
  `samples/projects/cli_tool` uses strict `cli::parse[T](args)` for required
  command-line-only options, enum/list/option fields, validation, recoverable
  `cli::Error` mapping, source execution, artifact-root execution, and
  `run --built` JSON output.
- [x] Post-strict CLI tool sample adoption gap selection is retired into code
  evidence: generated `muga new --template cli-tool` adoption mirrors the
  sample through source/build/`run --built`, README, completion helper, and
  packaging helper coverage.
- [x] Generated cli-tool template adoption is implemented: `muga new
  --template cli-tool` creates a strict CLI-only manifest project that mirrors
  `samples/projects/cli_tool`, CLI usage and shell completions list the
  template, and generated source/build/`run --built` workflows are covered.
- [x] Post-generated cli-tool template adoption gap selection is retired into
  code evidence: the historical strict CLI manual help path is covered by
  sample/template tests and later replaced by generated strict usage.
- [x] Strict CLI manual help adoption is implemented:
  `samples/projects/cli_tool` and generated `cli-tool` starters answer
  `--help` before calling `cli::parse[T](args)`, and
  source/generated/build/`run --built` coverage preserves that app boundary.
- [x] Post-strict CLI manual help adoption gap selection is retired into code
  evidence: `strict-cli-no-default-usage.md` captures the type-anchor policy
  and generated strict usage helper design before broader CLI ergonomics.
- [x] Strict CLI no-default usage helper design is recorded in
  [strict-cli-no-default-usage.md](strict-cli-no-default-usage.md).
- [x] Strict CLI no-default usage helper implementation is complete:
  `cli::usage_for_required[T](program)` supports explicit call type arguments,
  generated strict usage rendering, source/artifact/`run --built` coverage, and
  replacement of duplicated sample/template manual usage text before command
  metadata, combined short flags, attached values, subcommands, TOML, config discovery automation, full
  client generation, or host-effect APIs.
- [x] Post-strict CLI no-default usage helper adoption gap selection is retired
  into code evidence: [cli-command-metadata.md](cli-command-metadata.md)
  captures the selected record-level command summary design and implementation.
- [x] CLI command metadata design is recorded in
  [cli-command-metadata.md](cli-command-metadata.md): add record-level
  `@cli(about: "...")` command summaries to generated usage without changing
  parsing.
- [x] CLI command metadata implementation is complete: record-level
  `@cli(about: "...")` flows through parser validation, typing, package
  signatures, `.mgi` interfaces, `CliSchema`, artifacts, runtime usage
  rendering, strict sample/template adoption, and `std_cli_schema` package
  sample coverage.
- [x] Post-CLI command metadata adoption gap selection is retired into code
  evidence: [cli-short-option-metadata.md](cli-short-option-metadata.md)
  captures the selected field-level short option design and implementation.
- [x] CLI short option metadata design is recorded in
  [cli-short-option-metadata.md](cli-short-option-metadata.md): add
  field-level `@cli(short: "x")` short names to typed CLI schemas before
  implementation.
- [x] CLI short option metadata is implemented: `@cli(short: "x")` flows
  through parser validation, typing, interfaces, `CliSchema`, artifacts,
  runtime parsing and usage rendering, `cli::has_short_flag`, and the
  cli-tool/std_cli_schema starter samples.
- [x] Post-CLI short option metadata adoption gap selection is recorded in
  [post-cli-short-option-metadata-adoption-gap-selection.md](post-cli-short-option-metadata-adoption-gap-selection.md):
  design typed CLI positional field metadata before implementation.
- [x] CLI positional field metadata design is recorded in
  [cli-positional-field-metadata.md](cli-positional-field-metadata.md):
  field-level `@cli(positional: N)` with explicit 1-based indexes, parser and
  usage behavior, interface/artifact persistence, and deferrals before
  implementation.
- [x] CLI positional field metadata is implemented: `@cli(positional: N)` flows
  through parser validation, typing, package signatures, `.mgi` interfaces,
  `CliSchema`, artifacts, runtime parsing and usage rendering, source/build
  tests, and the strict `cli-tool` template/sample.
- [x] Post-CLI positional field metadata adoption gap selection is recorded in
  [post-cli-positional-field-metadata-adoption-gap-selection.md](post-cli-positional-field-metadata-adoption-gap-selection.md):
  design built-in CLI help policy before combined short flags, attached values,
  subcommands, shell completion generation, TOML/config discovery automation,
  full client generation, generic encoding/decoding, broader validators, or
  host-effect APIs.
- [x] Built-in CLI help policy design is recorded in
  [cli-built-in-help-policy.md](cli-built-in-help-policy.md): add
  `cli::help_requested(args)`, `cli::help_for[T](program, defaults)`, and
  `cli::help_for_required[T](program)` before parse-integrated help result
  enums, runtime-owned printing/exits, custom help flags, subcommands, shell
  completion generation, TOML/config discovery automation, full client
  generation, generic encoding/decoding, broader validators, or host-effect
  APIs.
- [x] Built-in CLI help helpers are implemented: `cli::help_requested(args)`
  respects `--`, `cli::help_for[T](program, defaults)` and
  `cli::help_for_required[T](program)` render schema-backed help with
  `-h, --help`, help-name conflict diagnostics reject opt-in schema
  collisions, source/build/`run --built` execution is covered, and generated
  config/strict CLI templates use the helpers.
- [x] Post-built-in CLI help helper adoption gap selection is recorded in
  [post-built-in-cli-help-helper-adoption-gap-selection.md](post-built-in-cli-help-helper-adoption-gap-selection.md):
  design parse-integrated CLI help workflow before combined short flags,
  attached values, subcommands, shell completion generation, TOML/config
  discovery automation, full client generation, generic encoding/decoding,
  broader validators, or host-effect APIs.
- [x] Parse-integrated CLI help workflow design is recorded in
  [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md):
  add `cli::Request[T]`, `cli::parse_request[T](args, program)`, and
  `cli::parse_request_or[T](args, program, defaults)` before runtime-owned
  printing/exits, custom help flags, subcommands, shell completion generation,
  TOML/config discovery automation, full client generation, generic
  encoding/decoding, broader validators, or host-effect APIs.
- [x] Parse-integrated CLI help workflow is implemented: `std::cli::Request[T]`,
  `cli::parse_request[T](args, program)`, and
  `cli::parse_request_or[T](args, program, defaults)` lower through typed HIR,
  MIR, bytecode, implementation artifacts, and runtime execution; help wins
  before parse errors and stops at `--`; source/artifact/`run --built` tests
  cover strict and overlay request workflows; generated strict/config templates
  adopt the request API while preserving app-owned printing/status decisions.
- [x] Post-parse-integrated CLI help workflow adoption gap selection is recorded
  in
  [post-parse-integrated-cli-help-workflow-adoption-gap-selection.md](post-parse-integrated-cli-help-workflow-adoption-gap-selection.md):
  design compact CLI short option syntax before runtime-owned printing/exits,
  subcommands, shell completion generation, TOML/config discovery automation,
  full client generation, generic encoding/decoding, broader validators, or
  host-effect APIs.
- [x] Compact CLI short option syntax design is recorded in
  [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md):
  specify combined bool flags and attached short values before implementation,
  without changing `CliSchema` metadata or built-in help exact-match behavior.
- [x] Compact CLI short option syntax is implemented: `cli::parse[T]`,
  `cli::parse_or[T]`, `cli::parse_request[T]`, and `cli::parse_request_or[T]`
  now accept combined bool flags such as `-abc`, attached values such as
  `-ofile`, and explicit compact final values such as `-abo=value`, while
  preserving exact `-x value` / `-x=value` behavior, `--` boundaries, and
  existing `CliSchema` artifacts.
- [x] Post-compact CLI short option syntax adoption gap selection is recorded
  in
  [post-compact-cli-short-option-syntax-adoption-gap-selection.md](post-compact-cli-short-option-syntax-adoption-gap-selection.md):
  design CLI subcommand metadata before implementation, generated app shell
  completions, TOML/config discovery automation, runtime-owned printing/exits,
  full client generation, generic encoding/decoding, broader validators, or
  host-effect APIs.
- [x] CLI subcommand metadata design is recorded in
  [cli-subcommand-metadata.md](cli-subcommand-metadata.md): use strict
  enum-backed command trees whose variants carry command-record or nested
  command-enum payloads, with explicit command names, aliases, summaries,
  hidden commands, root/subcommand help routing, record-leaf compact option
  parsing, `CliSchema` command payloads, and artifact compatibility rules
  before implementation.
- [x] First CLI subcommand enum metadata plumbing is implemented: enum
  declarations and variants accept validated `@cli(...)` metadata, duplicate
  sibling command names are diagnosed, typed HIR/package signatures preserve the
  metadata, and `.mgi` package interfaces introduced `muga-package-interface-v10`
  for enum command metadata before the wrapper field marker moved the current
  writer to `muga-package-interface-v11`.
- [x] Historical handoff `Next recommended slice: implement CLI subcommand metadata`
  is now split into completed enum metadata plumbing and the planned strict
  command schema/runtime slice below.
- [x] Strict CLI command enum schemas and runtime dispatch/help are implemented:
  strict helpers accept concrete non-generic command enums, nested command enum
  payloads lower through `CliSchema.commands`, `CC` schema artifact payloads,
  MIR/bytecode/implementation artifacts, source execution, artifact-backed
  execution, and `run --built`; runtime dispatches command names and aliases
  recursively, renders root/branch/leaf help through `cli::parse_request[T]`,
  omits hidden commands from help, and rejects invalid command schema shapes at
  type checking.
- [x] CLI subcommand schema adoption audit is recorded in
  [post-cli-subcommand-schema-adoption-gap-selection.md](post-cli-subcommand-schema-adoption-gap-selection.md):
  refresh `samples/projects/cli_tool` and generated `muga new --template
  cli-tool` starters to use `Command::Run(RunCommand)` /
  `Command::Inspect(InspectCommand)` while preserving compact short options,
  validation, generated root/leaf help, artifact-backed execution, and
  recoverable `cli::Error` mapping.
- [x] CLI wrapper-record root/global options design is recorded in
  [cli-wrapper-root-options.md](cli-wrapper-root-options.md): use a strict
  wrapper record with exactly one `@cli(subcommand)` field to return global
  options and a selected command enum together, while deferring root
  positionals, after-command global options, overlay/default wrappers, shell
  completions, TOML/config discovery automation, and runtime-owned exits.
- [x] CLI wrapper-record subcommand metadata plumbing is implemented:
  parser/formatter/type-checker, typed HIR, package signatures, and `.mgi`
  package interfaces preserve `cli_subcommand: bool`; invalid marker
  combinations, duplicate wrapper markers, non-enum marker fields, and invalid
  command enum targets are diagnosed before schema lowering.
- [x] CLI wrapper-record schema and runtime support is implemented: wrapper
  records lower to `CliSchema.subcommand`, persist `CW` schema/artifact
  payloads, parse root/global options before command dispatch, render wrapper
  root help, and run through source, artifact-backed, and `run --built`
  coverage.
- [x] CLI wrapper-record sample/template adoption is implemented: the checked-in
  `samples/projects/cli_tool` project and generated `muga new --template
  cli-tool` starter now parse a `Root` wrapper with a `--profile` / `-p`
  global option plus the existing `run` / `inspect` command tree.
- [x] CLI schema-backed shell completion design is recorded in
  [cli-schema-shell-completions.md](cli-schema-shell-completions.md): use a
  separate `muga cli-completions <bash|zsh|fish> --program <name> --type
  <Type> ...` command for generated Muga apps, driven by wrapper, command,
  leaf option, alias, short option, enum-value, and positional `CliSchema`
  data while keeping `muga shell-completions` static for the `muga` developer
  tool.
- [x] CLI schema-backed shell completion implementation is in place:
  `muga cli-completions <bash|zsh|fish> --program <name> --type <Type> ...`
  loads source, `--artifact-root`, and `--built` `CliSchema` data and emits
  bash/zsh/fish scripts for generated `cli-tool` root options, command aliases,
  leaf options, enum values, Bool values, and help flags.
- [x] CLI schema-backed shell completion adoption audit is recorded in
  [post-cli-schema-shell-completion-adoption-gap-selection.md](post-cli-schema-shell-completion-adoption-gap-selection.md):
  install documentation, a generated `cli-tool` README, and
  `scripts/generate-completions.sh` are implemented before richer nested
  traversal, TOML/config discovery, richer value sources, or installer
  integration.
- [x] Shell-agnostic JSON completion specs are implemented in
  [cli-completion-json-spec.md](cli-completion-json-spec.md):
  `muga cli-completions --format json --program <name> --type <Type> ...`
  loads the same source, `--artifact-root`, and `--built` `CliSchema` data and
  emits a recursive wrapper/command/record completion contract for generated
  app installers, package managers, editor adapters, and future renderers.
- [x] Richer nested command traversal is implemented for generated app
  completions: bash, zsh, and fish renderers now track recursive command-scope
  transitions so nested command enum payloads can offer leaf command options,
  help flags, and value candidates instead of stopping at the first command
  token.
- [x] Static CLI completion value-source metadata is implemented in
  [cli-completion-value-sources.md](cli-completion-value-sources.md):
  `@cli(value_source: "file"|"directory")` is compiler-checked for String-like
  CLI values, persisted through interfaces and artifacts, exposed as
  `valueSource` in JSON, and used by bash/zsh/fish option-value completion.
- [x] Non-mutating CLI completion installer integration is implemented in
  [cli-completion-installer-integration.md](cli-completion-installer-integration.md):
  `muga emit-cli-completions --format json --output-dir <dir> --program <name>
  --type <Type> ...` writes bash, zsh, fish, and `.completions.json` files with
  text or JSON metadata output while avoiding shell-profile edits.
- [x] Generated config-app path discovery is implemented in
  [config-path-discovery.md](config-path-discovery.md): generated config apps
  use `--config` first, `MUGA_CONFIG_PATH` second, and the generated JSON file
  as the final path fallback while keeping CLI > config > defaults explicit.
- [x] Workspace manifest metadata is implemented in [workspace-manifest-metadata.md](workspace-manifest-metadata.md): `muga workspace --format json` reports manifest path, project root, source/resource roots, root package path, direct dependencies, and dependency source/resource roots for project-aware tooling.
- [x] Generated config-app helpers are implemented in [config-app-run-helper.md](config-app-run-helper.md): generated config apps include a local README, `scripts/run-with-config.sh`, and `scripts/package-config-app.sh` using `MUGA_BIN` plus `MUGA_CONFIG_PATH`.
- [x] Package resource archives are implemented in [package-resource-archives.md](package-resource-archives.md): `[package] resources = "resources"` includes text/binary resource files in package hashes, `.mgp` archives, materialization, local archive dependency caches, and workspace `resourceRoot` metadata.
- [x] Runtime package resource lookup is implemented in
  [runtime-package-resource-lookup.md](runtime-package-resource-lookup.md):
  `std::fs::read_resource_text(package, path)` reads manifest-declared UTF-8
  resources in source, test, local archive dependency, and explicit built runs.
- [x] Next recommended slice: design installed-app resource layout and launcher boundary; implemented as `emit-app-bundle --source-free`, bundle-local dependencies, `run-app-bundle`, `install-app`, `emit-app-completions`, and `.mga` archive/unpack in [installed-app-bundles.md](installed-app-bundles.md).
- [x] Binary package resources and archive/install completion handoff polish are done; keep runtime `Bytes`, shell-profile mutation, registry publishing, dynamic completion producers, and broad TOML parsing deferred.
When finishing a slice, update this snapshot, the implementation table below,
the strategy progress table, and the relevant specs before ending the turn.
## Core Acceleration Queue
Use this queue when resuming without context. Pick the first unchecked core capability that still fits the active goal, finish it vertically with focused tests and docs, update this queue, then commit. If a slice widens the source language instead of only adding stdlib/runtime/tooling capability, update the v1 checklist, ROADMAP, mini spec, and focused tests first.
- [ ] `std::process` spine: child execution with captured status/stdout/stderr, explicit cwd/env, public errors, artifact-backed tests, and a runnable sample.
- [ ] Structured task spine: scoped task groups, explicit spawn/join, failure propagation, cancellation, timeout boundaries, and no hidden async suspension.
- [ ] Service IO spine: minimal socket/HTTP JSON workflow after resource and task rules can express shutdown and backpressure.
- [ ] Performance spine: control-flow MIR and runtime representation work backed by benchmark-health evidence before native backend claims.
- [ ] Distribution spine: publish/install work on archive verification, source-free bundles, install inventory, and API-diff trust boundaries.
## V1 Guardrail Queue

### Required V1 Boundary Work

The rule for this queue is to keep the v1 feature freeze intact unless a scope
change is first documented in the v1 checklist, ROADMAP, mini spec, split
specs, and focused tests.

- [ ] Run the offline release gate after each boundary slice and record any new failure pattern as a focused checklist item rather than as a release-timing decision.
- [ ] Keep `README.md`, `ROADMAP.md`, `mini-language-spec-v1.md`, split specs, and samples aligned whenever implemented syntax, stdlib behavior, package workflow, or diagnostics change.
- [ ] Keep `tests/release_readiness.rs` enforcing sample policy, diagnostic-code documentation, release-gate documentation, and CI/release workflow coverage.
- [ ] Keep `errors.md` aligned with every public diagnostic code family and require new or changed diagnostics to include actionable guidance where a user can fix the program.
- [ ] Keep `samples/` limited to runnable entrypoints, support files, or intentionally invalid fixtures; move future-looking snippets to `docs/design-snippets/`.
- [ ] Keep ordinary `check` / `run` source-compatible and keep artifact-backed behavior explicit through `--artifact-root` or `--built`.
- [ ] Do not have AI agents proactively suggest a publish, tag, or release cut until the v1 completion criteria are satisfied, unless the maintainer explicitly asks for release preparation.

### V1 Hardening Candidates

- [x] Audit package/artifact diagnostics for missing dependency hash, source hash, and regeneration-command JSON context; focused tests now cover `.mgb` dependency-interface set changes with expected dependency hashes and regeneration commands. Entry source, entry package, artifact-root, and concrete artifact-file context are already present in CLI JSON diagnostics where available.
- [x] Add more artifact-backed execution coverage for representative dependency APIs that combine stdlib packages, `try`, generic records/functions, enums, and transitive dependencies; `artifact_run_covers_representative_dependency_api_without_source` now exercises that combination without dependency source-body fallback.
- [x] Audit `.mgi` public interface hash stability after implementation-only edits and source-span movement across records, enums, generic functions, stdlib-backed signatures, and transitive public types; `package_interface_hash_stays_stable_for_representative_public_shapes` now covers the representative public shapes.
- [x] Audit `.mgb` structural validation and bytecode merge behavior for control-flow-heavy dependency bodies, private package items, and independently generated artifacts; `artifact_run_merges_independent_control_flow_dependency_implementations` now covers independently generated interfaces/implementations, private implementation helpers, and branch-heavy dependency bytecode before merge/runtime execution.
- [x] Audit `muga build` reuse output and lockfile update behavior for local path and local archive dependencies after dependency implementation-only edits, public signature edits, and malformed lockfiles; focused CLI tests now cover local path source-hash refreshes, local archive hash refreshes, public interface rewrites, preserved reusable interfaces/check caches, and fail-closed malformed lockfile validation.
- [x] Add focused diagnostics/tests for any remaining ambiguity or expected-type failure that still leaves users without a clear annotation/import/visibility/artifact-regeneration action; direct and mutual recursion annotation diagnostics now include concrete signature suggestions, and `errors.md` records the required guidance.
- [x] Review package-mode public signatures and ensure every v1-supported public type shape round-trips through in-memory and persisted interfaces; `package_public_signatures_round_trip_representative_type_shapes` now covers scalars, `Unit`, generic params, records, enums, collections, `Result`, function types, same-package/imported package identities, stdlib-backed public types, and downstream loaded-interface checking without dependency source.
- [x] Review stdlib package docs and samples for `std::io`, `std::fs`, `std::path`, `std::env`, and `std::time`, including artifact-backed execution samples where useful; [stdlib-package-samples-review.md](stdlib-package-samples-review.md) now maps the runnable samples to artifact-backed tests.
- [x] Keep the release gate and GitHub Actions aligned whenever local gate changes, including `scripts/v1-release-gate.sh`; [release-gate-alignment.md](release-gate-alignment.md) now records the script as canonical, and CI/release workflows invoke it directly.

### Recommended Functional V1 Candidates

These are not automatic release triggers. They are the functional additions
most likely to make Muga feel usable before the v1 compatibility promise, if
development continues before v1.

- [x] Add a minimal `muga test` workflow: compiler-recognized `@test` functions, discovery through source/package metadata, `Unit` or `Result[Unit, E]` test returns, and clear failure diagnostics.
- [x] Add the smallest static attribute surface needed for tests, starting with `@test`; keep attributes compiler/tool-recognized only, without macro expansion, code rewriting, or runtime reflection.
- [x] Add concrete test assertions for supported scalar types first, such as `test::assert_true`, `test::assert_eq_int`, `test::assert_eq_bool`, and `test::assert_eq_string`.
- [x] Decide the v1 equality policy for records, enums, lists, maps, `Option`, and `Result`: equality stays scalar-only for `Int`, `Bool`, and `String`; structural equality remains out of v1.
- [x] Add `Option` helpers such as `is_some`, `is_none`, `map`, `and_then`, and `value_or` as ordinary package functions that do not add new propagation syntax.
- [x] Add `Result` helpers such as `is_ok`, `is_err`, `map`, `map_err`, `and_then`, and `value_or` as value-transforming helpers; keep early return expressed through `try expr`.
- [x] Add narrow `List` and `Map` helpers that are useful without iterator protocols and structural equality: `std::list` provides `map`, `filter`, `fold`, `any`, and `all`; `std::map` provides `keys` and `values`.
- [x] Add runnable and artifact-backed samples that use Option/Result helpers and collection helpers across package boundaries.

### Recommended Tooling And Adoption Candidates

These can improve practical adoption without widening the core language model.
They are not release triggers, and each should use structured compiler/package
facts instead of scraping human-oriented output.

- [x] Add deterministic `muga fmt` for v1 source files, including a CI-friendly `--check` mode and line-comment preservation.
- [x] Add entry path and file URI metadata to `muga check --format json` so editor and LSP prototypes can attach diagnostics to the checked file without relying on human output or command-line scraping.
- [x] Add `muga doc` generation from `.mgi` public records, enums, opaque types, and functions without reading unrelated private dependency bodies.
- [x] Add minimal `muga new` templates for an app, a library package, and a package with tests.
- [x] Add public source comments to generated docs with item-level `///` storage in `.mgi`.
- [x] Add `muga syntax --format json` for single-file lex/parse diagnostics and faster editor feedback.
- [x] Add `muga metadata --format json` for package/module/item/export metadata plus public interface docs and rendered types.
- [x] Add `muga hover --format json` for declaration hover data with public docs and signatures.
- [x] Add `muga completions --format json` for visible package/interface completions from import aliases and public `.mgi`-backed records/enums/opaque types/functions.
- [x] Add `muga definition --format json` for go-to-definition over import aliases, local bindings, and package/interface item references.
- [x] Add `muga references --format json` for find references over import aliases, local bindings, and package/interface item references in the entry module.
- [x] Add `muga workspace --format json` for entry-reachable workspace metadata over loaded packages, module source files, default artifact root, and dependency edges.
- [x] Add entry source context to CLI JSON compiler diagnostics so `diagnostics[].context` can identify the checked source file directly.
- [x] Add entry package and artifact-root context for artifact-backed `check --format json` diagnostics.
- [x] Add concrete artifact-file context entries for JSON diagnostics that already know a specific `.mgi`, `.mgc`, or `.mgb` path.
- [x] Add JSON contracts for `muga build` artifact status lines.
- [x] Add JSON contracts for artifact emission commands.
- [x] Add dependency hash, source hash, and regeneration-command context for artifact diagnostics.
- [x] Add JSON `muga test` output after assertion diagnostics and test stdout capture are stable.
- [x] Add JSON `run` output with explicit program stdout, stderr, and main-result separation.
- [x] Add more runnable package examples that show local dependencies, artifact-backed execution, `Result` errors, text-file IO, and small reusable APIs.
- [x] Add `muga explain <diagnostic-code>` using the documented diagnostic index.
- [x] Add a concrete JSON-backed editor workflow smoke test that composes existing syntax, check, workspace, metadata, hover, completions, definition, references, run, and test output.
- [x] Add minimal command-line shell completions and a `muga doctor` environment check if they remain tool-only.
- [x] Decide the first `std::json` slice only after Result/helper ergonomics, scalar/collection mapping, schema evolution, and diagnostics are documented.
- [x] Implement the first `std::json` package contract from [std-json-first-slice.md](std-json-first-slice.md), without schema generation, HTTP/RPC, `Float`, `Decimal`, `Bytes`, streaming APIs, or resource handles.
- [x] Audit the implemented first `std::json` slice against docs, samples, artifact-backed behavior, and release-readiness evidence before broadening any standard-library surface.
- [x] Choose the next narrow stdlib/API boundary only after documenting a contract and checking deferred surfaces.
- [x] Choose the post-`using` adoption/API boundary: minimal pure `std::cli` helpers over explicit argument lists were selected and implemented; `Bytes`, formatting templates, process APIs, network APIs, streams, and broader host effects remain deferred.
- [x] Add the first `std::cli` helper slice for positional arguments, long flags, and long options over `List[String]`, then refresh `report_app`.
- [x] Select the post-`std::cli` practical adoption gap: refresh the generated app template into a small CLI-first starter using `std::env` and `std::cli`; richer CLI parsers, formatting templates, `Bytes`, process APIs, network APIs, streams, and broader host effects remain deferred.
- [x] Refresh `muga new --template app` so generated app projects run as useful CLI starters from source and built artifacts.
- [x] Select the post-template adoption/API gap: add typed scalar `std::cli` parsing helpers for `Int` and `Bool` before richer CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- [x] Implement typed scalar `std::cli` parsing helpers over existing positional/option lookup behavior.
- [x] Select the post-typed-cli adoption/API gap: add JSON value and object-field accessor helpers before full CLI parser schemas, config-file loading, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- [x] Implement JSON value and object-field accessor helpers in `std::json`, keeping them pure over existing `json::Value`, `json::Number`, `json::Error`, `Option`, `Result`, `Map`, and `List` contracts.
- [x] Select the post-json-accessor adoption/API gap: add a JSON config workflow sample that composes existing stdlib packages and explicit CLI > config > defaults precedence before `std::config`, TOML, schema decoding, full CLI parser schemas, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- [x] Implement the selected JSON config workflow sample with source, emitted-artifact, config shape-error, and `run --built --format=json` coverage.
- [x] Select the post-config-workflow adoption/API gap: refresh `config_app` to use existing `std::result::map_err` for explicit app-boundary error normalization before adding `std::config`, TOML, schema decoding, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- [x] Implement the selected `std::result::map_err` config workflow refresh.
- [ ] Select the post-result-mapping adoption/API gap before adding `std::config`, TOML, schema decoding, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects.
- [x] Design the opaque resource-handle boundary before adding broader runtime-backed stdlib APIs.
- [ ] Plan the first opaque type interface slice before adding runtime-backed handle values or broader stdlib APIs.

### Recommended Trust And Maintenance Candidates

These make the language easier to validate, teach, package, and operate. They
should validate the existing v1 surface first, not become a reason to widen the
language or force a release. The classification pass from the broader modern
language inventory lives in
[modern-language-gap-decisions-2026-05-22.md](modern-language-gap-decisions-2026-05-22.md);
use that file when deciding whether a candidate belongs to v1 validation,
optional pre-v1 usability, post-v1 platform work, or deliberate non-goals.

- [x] Add a conformance-suite layout tied to the mini spec and split specs, with valid programs, rejecting programs, expected diagnostic codes, and package/artifact workflow fixtures.
- [x] Define a stable machine-readable diagnostic schema for CLI/library/LSP/agent use, including code, severity, primary span, related spans, suggestions, and package/artifact context.
- [x] Define stable command-output contracts for human text and JSON modes so tools and AI agents do not scrape unstable prose.
- [x] Design `.mgi` API compatibility diffing for public function signatures, records, enums, deprecations, and implementation-only edits before automating release guidance.
- [x] Write standard-library review rules covering explicit `Result` effects, `Option` for absence, public error types, no hidden IO in property access, and opaque resource handles for runtime-backed values.
- [x] Define doc-comment and public API documentation rules for item-level `///` comments on public records, enums, opaque types, and functions.
- [x] Add initial `muga metadata --format json` package metadata for single-entry editor and agent tools.
- [x] Add initial `muga workspace --format json` entry-reachable workspace metadata for editor and agent tools.
- [x] Add initial `muga syntax --format json` single-file parse feedback for editor and agent tools.
- [x] Add initial entry source context in CLI JSON diagnostic objects for editor and agent tools.
- [x] Add initial entry package and artifact-root context in artifact-backed check JSON diagnostics.
- [x] Add initial concrete artifact-file context in artifact-backed check JSON diagnostics.
- [x] Design broader artifact/cache explanation commands, such as future `muga why-rebuild`, before broad LSP or agent tooling depends on them. See [artifact-cache-explanations.md](artifact-cache-explanations.md).
- [x] Implement initial read-only `muga why-rebuild --format json` output over local package graphs and `.mgi` / `.mgc` / `.mgb` artifact state.
- [x] Broaden `muga why-rebuild` coverage for stale dependency-interface hashes in `.mgb` implementation artifacts and `.mgc` check-cache explanations.
- [x] Broaden `muga why-rebuild` coverage for local path/archive lock metadata.
- [x] Broaden `muga why-rebuild` coverage for local archive cache metadata.
- [x] Add human text output for `muga why-rebuild`.
- [x] Add runtime call-context related notes for nested function calls and entry/test execution.
- [x] Add source-spanned `R021` diagnostics for failed `std::test` scalar assertions.
- [x] Close remaining runtime/debug reporting follow-up by documenting the v1 boundary: runtime stack context uses `related` call-context notes, failed scalar assertions use `R021`, and artifact next actions use `regenerationCommand` context instead of a new stack-trace schema.
- [x] Add lightweight benchmark health checks for compiler stages, package artifact reuse, and representative String/List/Map/runtime paths without making public performance claims.
- [x] Add fuzzing and malformed-input test plans for parser, package archive, lockfile, interface, check-cache, and implementation artifacts.
- [x] Document installation and onboarding paths such as `cargo install`, version checks, quickstarts, and later binary-release expectations without having AI agents push for a release.
- [x] Preserve the `.mgp` hash foundation and design future registry security around signing, provenance, lockfile enforcement, cache validation, and malicious-package handling before remote fetching.
- [x] Draft "Muga by Example" education material that progresses from bindings and records to `Result`, packages, tests, local dependencies, and artifact-backed builds.
- [x] Document an edition or semantic feature-set fingerprint policy before syntax or semantic changes need backward-compatible migration.

Recommended order from the modern-language gap decision pass:

- [x] Build the conformance-suite skeleton and wire it into release readiness.
- [x] Define JSON diagnostics and stable command-output contracts.
- [x] Add entry-aware `check --format json` output for editor, CI, LSP, and agent consumers.
- [x] Add minimal `muga doc` generated from public interface records, enums, opaque types, and functions.
- [x] Implement first `.mgi` API diff library comparison, CLI wrapper, and compatibility classifications.
- [x] Write stdlib review rules and link them from the v1 checklist.
- [x] Add the initial `muga test` / `@test` workflow.
- [x] Add scalar assertion helpers for `Int`, `Bool`, and `String`.
- [x] Add line-comment-preserving deterministic `muga fmt` for v1 source files.
- [x] Add minimal `muga new` templates for app, lib, test, and config app manifest projects.
- [x] Add item-level public source comments to `.mgi` and generated docs without changing interface hashes.
- [x] Add initial `muga metadata --format json` package facts for editor, LSP, CI, and agent tooling.
- [x] Add initial `muga workspace --format json` entry-reachable workspace metadata for editor, LSP, CI, and agent tooling.
- [x] Add initial `muga syntax --format json` single-file parse feedback for editor, LSP, CI, and agent tooling.
- [x] Add initial `muga hover --format json` declaration hovers for editor, LSP, CI, and agent tooling.
- [x] Add initial `muga completions --format json` visible package/interface completions for editor, LSP, CI, and agent tooling.
- [x] Add initial `muga definition --format json` go-to-definition data for editor, LSP, CI, and agent tooling.
- [x] Add entry source context to CLI JSON diagnostics for editor, LSP, CI, and agent tooling.
- [x] Add entry package and artifact-root context to artifact-backed check JSON diagnostics for editor, LSP, CI, and agent tooling.
- [x] Add concrete artifact-file context to artifact-backed check JSON diagnostics for editor, LSP, CI, and agent tooling.
- [x] Add JSON contracts for `muga build` artifact status lines.
- [x] Add JSON contracts for artifact emission commands.
- [x] Add dependency hash, source hash, and regeneration-command context for artifact diagnostics.
- [x] Add JSON `muga test` output after assertion diagnostics and test stdout capture are stable.
- [x] Add JSON `run` output with explicit program stdout, stderr, and main-result separation.

### Syntax Candidates To Track

The default is to keep these outside v1 unless the scope change is explicitly
documented across the checklist, specs, parser diagnostics, formatter rules,
samples, and focused tests.

- [x] Keep `@test` as the first static attribute candidate for `muga test`; do not add macro expansion, code rewriting, or runtime reflection.
- [ ] Decide whether named arguments are worth adding for long or same-typed call sites; define `.mgi` label storage, label-rename compatibility, positional/named mixing rules, and diagnostics before implementation.
- [x] First-slice `using` lexical cleanup is implemented for runtime-backed opaque handles with deterministic close/error rules; keep broader destructors/finalizers deferred.
- [x] Nested `using` cleanup unwinding is hardened so cleanup-error branches still attempt active outer cleanups before returning the first cleanup error.
- [ ] Keep range/slicing syntax deferred until string/list slicing semantics, allocation behavior, and byte/scalar/grapheme policy are settled.
- [ ] Keep pattern-matching refinements small and evidence-driven; broad wildcard or catch-all matching remains deferred.
- [ ] Keep string interpolation/templates deferred until `std::fmt`, builders, escaping, and localization expectations are explicit.
- [ ] Keep `T?` and `?.` reserved for future Option-only ergonomics; do not use `expr?` for Result propagation.

### Scope Decisions To Track Before V1

- [ ] Decide whether any additional narrow stdlib slice is truly required before v1; if yes, document the API, diagnostics, samples, and artifact-backed execution expectations before implementing it.
- [x] Initial scalar assertion helpers are implemented as `std::test` functions returning `Result[Unit, String]`; broad structural equality stays out under the scalar-only v1 equality policy.
- [x] Option/Result helper packages are in v1 scope as ordinary `std::option` / `std::result` package functions; postfix `expr?`, future `expr.try`, and Option-only `?.` remain out of v1.
- [x] Narrow `std::list` / `std::map` helpers are in v1 scope; keep `List.contains`, `Map.entries`, iterator protocols, broad collection APIs, map literals, `Set[T]`, and arbitrary `Map` key types deferred unless the scalar-only equality policy, v1 checklist, and specs are deliberately expanded.
- [x] Deterministic `muga fmt`, minimal `.mgi`-backed `muga doc` with public source comments, and `muga new` app/lib/test/config-app templates, including the CLI-first app template, are v1-scope adoption work with focused readiness checks; they remain release-neutral tooling surfaces.
- [x] Broader JSON-backed editor workflow validation is v1-scope adoption work; the concrete smoke test composes existing command contracts without making it a release-timing trigger.
- [x] Conformance tests, machine-readable diagnostics, `.mgi` API-diff design, stdlib review rules, and Muga-by-example docs are v1-scope maintenance work that validate the current surface rather than widening it.
- [ ] Decide whether any syntax candidate beyond static `@test` and first-slice `using` belongs before v1; default named arguments, range/slicing, pattern refinements, interpolation, `T?`, and `?.` stay post-v1 unless specs and tests are deliberately updated.
- [ ] Decide whether any project workflow improvement is required before v1 beyond explicit `.muga/build`, `--built`, local path dependencies, local archive dependencies, and minimal local lockfiles.
- [ ] Decide whether any compiler-internal hardening is required before v1 for MIR/runtime identity, bytecode validation, or package interface remapping; keep control-flow MIR and native backend out unless the current VM path cannot correctly represent a v1 behavior.
- [ ] Decide whether public-signature inference for `pub fn` remains post-v1; if it moves into v1, update the v1 checklist, specs, interface artifacts, diagnostics, and tests first.

### Post-V1 Backlog To Preserve

- [ ] Project-mode artifact-root configuration after lockfiles and a package-aware project build state are mature.
- [ ] Full incremental package artifact reuse and package-local rebuild planning beyond current unchanged-artifact preservation.
- [ ] URL/Git/registry dependency forms, remote fetching, publishing/install workflows, package signing, and full published-package lockfile enforcement.
- [ ] Broader opaque resource-handle families and deterministic lifetime extensions before broad filesystem, socket, process, timer, or service APIs.
- [ ] `Bytes`, buffers/builders, richer time/process APIs, JSON/HTTP/SSE/WebSocket/RPC, schema generation, and client generation.
- [ ] Named arguments, default arguments, `using` expressions/multiple bindings, range/slicing syntax, pattern refinements, string interpolation, and Option-only `T?` / `?.` ergonomics after concrete examples justify the parser/typechecker/interface cost.
- [ ] Workspaces, dev/test/bench dependencies, version solving, source replacement, vendoring, package yanking, `muga audit`, SBOM generation, binary distribution/installers, strict public performance benchmarks, remote registry trust/signing/provenance, and edition migration tooling after the local v1 package/artifact contract is stable.
- [ ] Control-flow MIR, native backend, optimizer work, and performance benchmarking claims.
- [ ] Structured concurrency with `group`, `spawn`, `join`, cancellation rules, typed channels, timeouts, and `select`-style coordination.

### Deliberate Non-Goals From Modern Gap Pass

- [ ] Keep universal null, implicit exceptions, postfix `expr?` Result propagation, runtime reflection as a core abstraction mechanism, macro/code-rewriting systems, user-defined operators, overloaded dispatch, dynamic `Any`, source-level references/borrowing/raw pointers, hidden async suspension, class inheritance, property access with hidden IO, arbitrary unsandboxed build scripts, and near-term scientific/ML/mobile/embedded focus out unless a later design note overturns the decision.

## Post-V1 Direction Snapshot

When v1 foundation work is complete, resume larger feature work in the order captured by [docs/strategy-and-implementation-plan.md](strategy-and-implementation-plan.md), [ROADMAP.md](../ROADMAP.md), and [docs/practical-language-readiness.md](practical-language-readiness.md):

1. keep `.mgi` public interfaces as the typed contract for packages and future API/schema tooling
2. grow narrow standard-library APIs with `Option`, `Result`, explicit public error types, and no hidden IO in property access
3. introduce opaque resource handles before adding broad filesystem, socket, process, timer, or server APIs
4. build control-flow MIR and runtime representations that optimize value semantics without source-level references
5. implement structured concurrency with `group`, `spawn`, `join`, failure propagation, cancellation, and task-boundary capture rules
6. add typed channels, then timeouts/deadlines and `select`-style coordination
7. integrate cancellation-aware asynchronous IO only after the task and resource-handle models are stable
8. add HTTP/SSE/WebSocket/RPC layers and external client/schema generation only above those explicit lower-level contracts

Do not treat a web framework, `async fn`/`await`, runtime metaprogramming, implicit exceptions, or dynamic `Any` as shortcuts around the package/interface/runtime foundation.

## Verification Snapshot

- [x] `cargo fmt`, `git diff --check`, `cargo test --locked --test release_readiness clippy_policy_is_configured_and_release_gated`, `cargo test --locked --test release_readiness stdlib_package_docs_and_samples_review_is_documented_and_covered`, `cargo test --locked --test examples package_std_cli_schema_sample_runs`, `cargo test --locked --test examples cli_new_creates_app_lib_and_test_templates`, `cargo test --locked --test examples manifest_config_project_sample_exports_schema_contract`, `scripts/clippy-check.sh`, `scripts/v1-release-gate.sh`, and `scripts/benchmark-health-check.sh` passed after hardening the Rust lint policy and adding CLI/schema sample coverage: 937 default tests passed, 3 benchmark health tests ignored by default, 3 manual benchmark health tests passed, 0 failures.
- [x] `cargo fmt`, `cargo fmt --check`, `git diff --check`, `cargo test --locked --test release_readiness release_docs_and_workflows_cover_v1_gate`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after aligning GitHub Actions with the canonical `scripts/v1-release-gate.sh` release gate: 695 tests passed, 3 benchmark health tests ignored by default, 0 failures.
- [x] `cargo fmt`, `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples package_std_io_sample_runs`, `cargo test --locked --test examples package_std`, `cargo test --locked --test examples standard_fs_artifact_run_uses_emitted_std_implementations`, `cargo test --locked --test examples standard_env_artifact_run_uses_emitted_std_implementations`, `cargo test --locked --test examples standard_time_artifact_run_uses_emitted_std_implementations`, `cargo test --locked --test release_readiness stdlib_package_docs_and_samples_review_is_documented_and_covered`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after reviewing stdlib package docs and samples for `std::io`, `std::fs`, `std::path`, `std::env`, and `std::time`, including artifact-backed execution samples where useful: 695 tests passed, 3 benchmark health tests ignored by default, 0 failures.
- [x] `cargo fmt`, `cargo fmt --check`, `git diff --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test --locked --test examples package_public_signatures_round_trip_representative_type_shapes`, `cargo test --locked --test examples package_interfaces_round_trip_public_records_functions_and_enums`, `cargo test --locked --test examples package_interface_hash_stays_stable_for_representative_public_shapes`, `cargo test --locked --test release_readiness public_signature_round_trip_audit_is_documented_and_covered`, `cargo test --locked --test release_readiness`, and `scripts/v1-release-gate.sh` passed after auditing package-mode public signatures across every representative v1-supported public type shape through in-memory and persisted interfaces: 693 tests passed, 3 benchmark health tests ignored by default, 0 failures.
- [x] `cargo fmt`, `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples artifact_run_merges_independent_control_flow_dependency_implementations`, `cargo test --locked --test release_readiness implementation_artifact_structural_audit_is_documented_and_covered`, `cargo test --locked --test release_readiness`, and `scripts/v1-release-gate.sh` passed after auditing `.mgb` structural validation and bytecode merge behavior for control-flow-heavy dependency bodies, private package items, and independently generated artifacts: 685 tests passed, 3 benchmark health tests ignored by default, 0 failures.
- [x] `cargo fmt`, `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples package_interface_hash_stays_stable_for_representative_public_shapes`, `cargo test --locked --test release_readiness public_interface_hash_stability_audit_is_documented_and_covered`, `cargo test --locked --test release_readiness`, and `scripts/v1-release-gate.sh` passed after auditing `.mgi` public interface hash stability across implementation-only edits, source-span movement, generic public shapes, stdlib-backed signatures, and transitive public types: 683 tests passed, 3 benchmark health tests ignored by default, 0 failures.
- [x] `cargo fmt`, `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples artifact_run_covers_representative_dependency_api_without_source`, `cargo test --locked --test release_readiness representative_artifact_dependency_api_coverage_is_documented_and_covered`, `cargo test --locked --test release_readiness`, and `scripts/v1-release-gate.sh` passed after adding representative artifact-backed dependency API coverage that combines stdlib packages, `try`, generic records/functions, enums, and transitive dependencies without source-body fallback: 681 tests passed, 3 benchmark health tests ignored by default, 0 failures.
- [x] `cargo fmt`, `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples dependency_interface_set_changed`, `cargo test --locked --test release_readiness`, and `scripts/v1-release-gate.sh` passed after auditing package/artifact diagnostics and adding `.mgb` dependency-interface set-change hash plus regeneration-command context for `run` diagnostics and `why-rebuild --format json`: 679 tests passed, 3 benchmark health tests ignored by default, 0 failures.
- [x] `cargo fmt`, `cargo fmt --check`, `git diff --check`, `cargo test --locked --test release_readiness edition_feature_fingerprint_policy_is_documented_and_covered`, `cargo test --locked --test release_readiness registry_security_design_is_documented_and_covered`, `cargo test --locked --test release_readiness muga_by_example_learning_path_is_documented_and_covered`, `cargo test --locked --test release_readiness`, and `scripts/v1-release-gate.sh` passed after adding the edition and semantic feature-set fingerprint policy for future package artifacts, cache keys, lockfiles, diagnostics, and API-diff compatibility before syntax or semantic migration: 677 tests passed, 3 benchmark health tests ignored by default, 0 failures.
- [x] `cargo fmt`, `cargo fmt --check`, `git diff --check`, `cargo test --locked --test release_readiness registry_security_design_is_documented_and_covered`, `cargo test --locked --test release_readiness muga_by_example_learning_path_is_documented_and_covered`, `cargo test --locked --test release_readiness`, and `scripts/v1-release-gate.sh` passed after adding the future registry security design that preserves the `.mgp` hash foundation and scopes signing, provenance, lockfile enforcement, cache validation, and malicious-package handling before remote fetching: 676 tests passed, 3 benchmark health tests ignored by default, 0 failures.
- [x] `cargo fmt`, `git diff --check`, `cargo test --locked --test release_readiness muga_by_example_learning_path_is_documented_and_covered`, `cargo test --locked --test release_readiness installation_onboarding_paths_are_documented_and_covered`, `cargo test --locked --test release_readiness`, and `scripts/v1-release-gate.sh` passed after adding the release-neutral "Muga by Example" learning path over existing runnable samples, generated test projects, local dependencies, and artifact-backed build commands: 675 tests passed, 3 benchmark health tests ignored by default, 0 failures.
- [x] `cargo fmt`, `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_version_reports_package_version`, `cargo test --locked --test release_readiness installation_onboarding_paths_are_documented_and_covered`, `cargo test --locked --test release_readiness fuzzing_malformed_input_plan_is_documented_and_covered`, `cargo test --locked --test release_readiness`, and `scripts/v1-release-gate.sh` passed after adding `muga --version` / `muga version` plus release-neutral installation and onboarding docs for `cargo install`, local checkout installs, generated-project quickstarts, and later binary-release expectations: 674 tests passed, 3 benchmark health tests ignored by default, 0 failures.
- [x] `cargo fmt`, `cargo fmt --check`, `git diff --check`, `cargo test --locked --test release_readiness fuzzing_malformed_input_plan_is_documented_and_covered`, `cargo test --locked --test release_readiness benchmark_health_checks_are_documented_and_covered`, `cargo test --locked --test release_readiness`, `cargo test --locked --test benchmark_health`, and `scripts/v1-release-gate.sh` passed after adding release-neutral fuzzing and malformed-input planning for parser, package archive, lockfile, interface, check-cache, and implementation artifacts: 672 tests passed, 3 benchmark health tests ignored by default, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test benchmark_health`, `cargo test --locked --test release_readiness benchmark_health_checks_are_documented_and_covered`, `cargo test --locked --test release_readiness`, `scripts/benchmark-health-check.sh`, and `scripts/v1-release-gate.sh` passed after adding release-neutral benchmark health checks for compiler stages, package artifact reuse, and representative String/List/Map runtime paths: 671 tests passed, 3 benchmark health tests ignored by default, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked call_context`, `cargo test --locked assertion_failure`, `cargo test --locked --test release_readiness runtime_debug_reporting_boundary_is_documented_and_covered`, `cargo test --locked --test release_readiness diagnostic_json_and_command_output_contract_are_documented`, `cargo test --locked --test release_readiness`, and `scripts/v1-release-gate.sh` passed after closing the runtime/debug reporting v1 follow-up in docs and release-readiness coverage: 670 tests, 0 failures.
- [x] `cargo test --locked assertion_failure`, `cargo test --locked muga_test_assertion_helpers_report_scalar_failures`, `cargo test --locked call_context`, `cargo test --locked --test release_readiness diagnostic_json_and_command_output_contract_are_documented`, `cargo test --locked --test release_readiness muga_test_scope_is_documented`, and `scripts/v1-release-gate.sh` passed after adding source-spanned `R021` diagnostics for failed `std::test` scalar assertions: 669 tests, 0 failures.
- [x] `cargo test --locked call_context`, `cargo test --locked diagnostic_json_and_command_output_contract_are_documented`, and `scripts/v1-release-gate.sh` passed after adding runtime call-context related notes for `run` and `test` diagnostics: 668 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_why_rebuild`, `cargo test --locked --test release_readiness artifact_cache_explanation_design_is_documented`, `cargo test --locked --test release_readiness diagnostic_json_and_command_output_contract_are_documented`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding compact human text output to `muga why-rebuild`: 665 tests, 0 failures. CLI spot checks also confirmed `target/debug/muga why-rebuild --built samples/packages/app/artifact_facade/main.muga` prints tab-separated text output and `target/debug/muga why-rebuild --format json --built samples/packages/app/artifact_facade/main.muga` preserves the JSON contract.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_why_rebuild_json`, `cargo test --locked --test release_readiness artifact_cache_explanation_design_is_documented`, `cargo test --locked --test release_readiness diagnostic_json_and_command_output_contract_are_documented`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding local archive-cache metadata explanations to `muga why-rebuild --format json`: 662 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_why_rebuild_json`, `cargo test --locked --test release_readiness artifact_cache_explanation_design_is_documented`, `cargo test --locked --test release_readiness diagnostic_json_and_command_output_contract_are_documented`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding local path/archive lockfile metadata explanations to `muga why-rebuild --format json`: 662 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_why_rebuild_json`, `cargo test --locked --test release_readiness artifact_cache_explanation_design_is_documented`, `cargo test --locked --test release_readiness diagnostic_json_and_command_output_contract_are_documented`, and `scripts/v1-release-gate.sh` passed after adding stale dependency-interface coverage to `muga why-rebuild --format json`: 660 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_why_rebuild_json`, `cargo test --locked --test release_readiness artifact_cache_explanation_design_is_documented`, `cargo test --locked --test release_readiness diagnostic_json_and_command_output_contract_are_documented`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding initial read-only `muga why-rebuild --format json` artifact/cache explanations: 659 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test release_readiness artifact_cache_explanation_design_is_documented`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding the artifact/cache explanation design and release-readiness coverage: 655 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples json_backed_editor_workflow_uses_existing_command_contracts`, `cargo test --locked --test release_readiness json_backed_editor_workflow_is_documented_and_covered`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding the concrete JSON-backed editor workflow smoke test and docs: 654 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_explain`, `cargo test --locked --test release_readiness muga_explain_scope_is_documented`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding `muga explain <diagnostic-code>` output: 652 tests, 0 failures. CLI spot checks also confirmed `cargo run --locked -- explain E001`, `cargo run --locked -- explain T024`, and the expected unknown-code rejection for `cargo run --locked -- explain ZZ999`.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples manifest_report_project_sample`, `cargo test --locked --test examples manifest`, `cargo test --locked --test release_readiness equality_policy_is_documented_and_covered`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, `cargo run --locked -- samples/projects/report_app/src/main/main.muga`, and `scripts/v1-release-gate.sh` passed after adding the runnable local-dependency report sample with text-file IO and artifact-backed execution coverage: 647 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_run_json`, `cargo test --locked --test examples cli_check_rejects_program_args_separator`, `cargo test --locked --test examples cli_run`, `cargo test --locked --test release_readiness diagnostic_json_and_command_output_contract_are_documented`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding `muga run --format json` output: 645 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_test_json_reports_success_contract_on_stdout`, `cargo test --locked --test examples cli_test_json_reports_failure_contract_on_stdout`, `cargo test --locked --test examples cli_test_json_reports_diagnostic_contract_on_stdout`, `cargo test --locked --test examples cli_test`, `cargo test --locked --test examples muga_test`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding `muga test --format json` output: 641 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_check_json_reports_hash_and_regeneration_context_for_stale_check_cache`, `cargo test --locked --test examples package_cache_rejects_stale_dependency_interface_artifact`, `cargo test --locked --test examples cli_run_reports_dependency_interface_mismatched_implementation_artifact`, `cargo test --locked --test examples cli_check_json`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding artifact hash and regeneration-command JSON diagnostic context: 638 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_emit_artifacts_json_reports_artifact_contract`, `cargo test --locked --test examples cli_emit_interface_json_reports_filtered_artifact_contract`, `cargo test --locked --test examples cli_emit_check_cache_json_reports_artifact_contract`, `cargo test --locked --test examples cli_emit_check_cache_json_reports_diagnostic_contract_on_stdout`, `cargo test --locked --test examples cli_emit`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding JSON output for explicit artifact emission commands: 637 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_build_json_reports_artifact_status_contract`, `cargo test --locked --test examples cli_build_json_reports_diagnostic_contract_on_stdout`, `cargo test --locked --test release_readiness diagnostic_json_and_command_output_contract_are_documented`, `cargo test --locked --test examples cli_build`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding `muga build --format json` artifact status and diagnostic output: 633 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_check_json`, `cargo test --locked --test examples diagnostic_json_includes_artifact_file_context_when_available`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding concrete artifact-file context for `.mgi`, `.mgc`, and `.mgb` diagnostics: 631 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_check_json`, `cargo test --locked --test examples cli_check_json_reports_package_and_artifact_context_for_artifact_backed_diagnostics`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, `cargo run --locked -- check --format json --artifact-root ~/tmp/muga-json-context-missing samples/packages/app/main/main.muga`, and `scripts/v1-release-gate.sh` passed after adding entry package and artifact-root context to JSON check diagnostics: 630 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_check_json_reports_diagnostic_contract_on_stdout`, `cargo test --locked --test examples cli_syntax_json_reports_fast_parse_feedback_for_editor_tools`, `cargo test --locked --test examples diagnostic_json_includes_stable_structured_fields`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after adding entry source context to CLI JSON diagnostics; `cargo run --locked -- check --format json conformance/v1/rejecting/name-resolution/immutable_update.muga` produced the expected error JSON with source context: 629 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_syntax`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, `cargo run --locked -- syntax --format json samples/println_sum.muga`, and `scripts/v1-release-gate.sh` passed after adding initial `muga syntax --format json`: 629 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_workspace`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, `cargo run --locked -- workspace --format json samples/packages/app/main/main.muga`, and `scripts/v1-release-gate.sh` passed after adding initial `muga workspace --format json`: 627 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_references`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, `cargo run --locked -- references --format json --line 11 --column 17 samples/packages/app/main/main.muga`, and `scripts/v1-release-gate.sh` passed after adding initial `muga references --format json`: 625 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_definition`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, `cargo run --locked -- definition --format json --line 11 --column 17 samples/packages/app/main/main.muga`, and `scripts/v1-release-gate.sh` passed after adding initial `muga definition --format json`: 623 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_completions`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, `cargo run --locked -- completions --format json samples/packages/app/main/main.muga`, and `scripts/v1-release-gate.sh` passed after adding initial `muga completions --format json`: 621 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_hover`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, `cargo run --locked -- hover --format json --line 3 --column 12 samples/packages/util/users/model.muga`, and `scripts/v1-release-gate.sh` passed after adding initial `muga hover --format json`: 619 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_metadata`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, `cargo run --locked -- metadata --format json samples/packages/app/main/main.muga`, and `scripts/v1-release-gate.sh` passed after adding initial `muga metadata --format json`: 617 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples package_interface`, `cargo test --locked --test release_readiness`, and `scripts/v1-release-gate.sh` passed after adding item-level public source comments to `.mgi` and `muga doc`: 615 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test examples cli_new`, `cargo test --locked --test release_readiness`, and `scripts/v1-release-gate.sh` passed after adding minimal `muga new` app/lib/test project templates: 614 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked doc`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, `cargo run --locked -- doc samples/projects/my_service/src/main/main.muga`, and `scripts/v1-release-gate.sh` passed after adding minimal `.mgi`-backed `muga doc` Markdown generation: 611 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked json`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, `cargo run --locked -- check --format json samples/println_sum.muga`, `cargo run --locked -- check --format json conformance/v1/rejecting/name-resolution/immutable_update.muga`, and `scripts/v1-release-gate.sh` passed after adding entry path and `file://` URI metadata to `check --format json`: 609 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked json`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, `cargo run --locked -- check --format json samples/println_sum.muga`, `cargo run --locked -- check --format json conformance/v1/rejecting/name-resolution/immutable_update.muga`, and `scripts/v1-release-gate.sh` passed after adding the stable diagnostic JSON schema and initial `check --format json` command-output contract: 574 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked --test conformance`, `cargo test --locked --test release_readiness`, `cargo clippy --all-targets -- -D warnings`, and `scripts/v1-release-gate.sh` passed after wiring the initial conformance-suite skeleton into release readiness: 570 tests, 0 failures.
- [x] `cargo test --locked --test release_readiness`, `scripts/v1-release-gate.sh`, and `git diff --check` passed after v1 RC readiness verification; `scripts/v1-release-gate.sh --with-publish-dry-run` remains available when preparing a publish.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test --locked`, and `cargo clippy --all-targets -- -D warnings` passed after `.mgb` implementation artifact diagnostic context hardening: 560 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after artifact-backed run check-cache diagnostic hardening: 561 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after completing `--built` default artifact diagnostic coverage: 557 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after `--built` default artifact diagnostic guidance: 554 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after CLI/spec package workflow alignment: 552 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after completing current `E005` ambiguity diagnostic guidance: 551 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after the list-only ambiguity diagnostic guidance slice: 546 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after the `len` / `is_empty` ambiguity diagnostic guidance slice: 544 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after the `print` / `println` ambiguity diagnostic guidance slice: 542 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after the `to_string` ambiguity diagnostic guidance slice: 541 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after the invalid `try Result::Ok(...)` placement diagnostic cleanup: 541 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after the fully qualified `std::io` error-type diagnostics slice: 541 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after the stale generic interface artifact diagnostics slice: 539 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after the contextual generic record literal hardening slice: 538 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, `cargo package --locked --allow-dirty --offline --list`, and `cargo package --locked --allow-dirty --offline` passed after the manifest string parsing and local build reuse diagnostics slices: 537 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, representative sample runs, `cargo package --locked --allow-dirty --offline --list`, `cargo package --locked --allow-dirty --offline`, and post-commit `cargo package --locked --offline` passed after the local archive dependency snippet workflow slice: 537 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, representative sample runs, and `cargo publish --dry-run --locked` passed after the local archive dependency hardening slice: 536 tests, 0 failures; the dry-run reached crates.io, saw the expected existing `muga@0.2.0`, and aborted upload because it was a dry run.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, and representative sample runs passed after the local archive dependency/cache consumption slice: 530 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests --locked`, `cargo test --locked`, `cargo clippy --all-targets -- -D warnings`, `cargo build --locked`, and representative sample runs passed after the local package archive materialization slice: 527 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the package archive readback validation slice: 524 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the deterministic package archive emission slice: 521 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the canonical package content hash slice: 519 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the local path lockfile validation/update slice: 517 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the local path lockfile metadata slice: 516 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the parallel package build slice: 514 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the public interface hash stability slice: 513 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the `.mgb` package-local source-hash metadata slice: 512 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the unchanged build artifact reuse slice: 512 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the local path dependency metadata slice: 511 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the `check --built` / `run --built` slice: 505 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the minimal `muga build` slice: 502 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the payload discard `_` enum-pattern slice: 499 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the `for item in list` slice: 491 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the `break` / `continue` slice: 482 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the explicit `return expr` slice: 472 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo build`, and representative sample runs passed after the `else if` syntax slice: 463 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::fs::copy_file_path` / `std::io::PathPairError` slice: 454 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::fs::remove_dir_path` slice: 447 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::fs::remove_file_path` slice: 442 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::path::is_absolute` slice: 437 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::fs::create_dir_all_path` slice: 432 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::path::file_stem` slice: 427 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::path::extension` slice: 422 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::path::parent` slice: 417 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::path::file_name` slice: 412 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::fs::read_dir_path` slice: 407 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::path::join` slice: 402 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::fs::create_dir_path` slice: 398 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::fs` path metadata predicate slice: 393 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the `std::env::args` program-argument slice: 389 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the minimal `std::time` package slice: 381 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the minimal `std::env` package slice: 375 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the Path-aware `std::fs` text-file helper slice: 369 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the minimal `std::path` package slice: 365 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the first compiler-provided `std::io` / `std::fs` text-file package slice: 339 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after prefix `try expr` plus artifact/control-flow hardening: 296 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the generic records/functions slice: 285 tests, 0 failures.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the latest MIR/runtime identity slice.
- [x] `cargo fmt --check`, `git diff --check`, `cargo check --tests`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the latest `.mgb` artifact hardening and sample slice: 274 tests, 0 failures.
- [x] `cargo test --locked` passed after slot-backed runtime locals and bytecode local metadata: 258 tests, 0 failures.
- [x] `target/debug/muga samples/println_sum.muga` printed:

```text
10
10
```

- [x] `target/debug/muga check samples/packages/app/main/main.muga` printed `ok`.
- [x] `target/debug/muga samples/packages/app/main/main.muga` printed `23`.
- [x] `target/debug/muga samples/projects/my_service/src/main/main.muga` printed:

```text
Ada
21
```

- [x] `target/debug/muga samples/packages/app/enum_demo/main.muga` printed `7`.
- [x] `target/debug/muga check samples/packages/app/enum_demo/main.muga` printed `ok`.

## Current Implementation Ledger

### Core Language

- [x] `//` comments, newline-separated statements, and CRLF line counting.
- [x] immutable-by-default bindings.
- [x] `mut` bindings and same-function mutable updates.
- [x] `x = e` as either new immutable binding or mutable update depending on resolved scope.
- [x] no shadowing.
- [x] blocks, final-expression function bodies, and explicit `return expr`.
- [x] `if` statements, `if` expressions, `else if` chains, `while` loops, `for item in list`, and `break` / `continue`.
- [x] short-circuit `and` / `or` Boolean operators.
- [x] integer overflow and division-by-zero runtime diagnostics.

### Functions And Inference

- [x] named `fn` declarations.
- [x] recursive and mutually recursive functions with the current annotation rules.
- [x] anonymous function expressions.
- [x] closure capture.
- [x] higher-order functions.
- [x] local bidirectional inference for the implemented cases.
- [x] function type annotations with `->`.
- [x] user-defined generic functions with explicit declaration type parameters and call-site inference.

### Records And Calls

- [x] `record` declarations.
- [x] generic `record` declarations with explicit type parameters.
- [x] nominal record literals.
- [x] field access with `expr.name`.
- [x] non-destructive record update with `expr.with(...)`.
- [x] chained UFCS-style calls with `expr.name(...)`.
- [x] package-qualified chained calls with `expr.alias::name(...)`.
- [x] typed HIR preserves direct, chained, qualified chained, builtin, value, and package-item call targets.
- [x] AST, HIR, and typed HIR preserve compiler-known enum match arms as enum-variant-shaped patterns: enum name, variant name, and explicit payload mode (`none`, binding, or discard).

### Packages, Modules, And Interfaces

- [x] file-based package mode with `package`, `import`, `pub`, `pkg`, `as`, and `alias::Name`.
- [x] manifest project mode with minimal `[package] name/source`.
- [x] package symbol graph with `PackageId`, `ModuleId`, and `PackageItemId`.
- [x] module/file-private top-level items by default.
- [x] `pkg` visibility for sibling files in the same package.
- [x] `pub` visibility for importable items.
- [x] public export lookup through `interface::PackageExportGraph`.
- [x] typed HIR public record/function statements carry package item identity.
- [x] shared public `TypeInfo` data lives in `types`, with `typing` retaining a compatibility re-export.
- [x] `interface` owns in-memory package interface summaries for public records and functions.
- [x] interface summaries preserve public `TypeInfo`, package record identity, collection types, and compiler-known `Result` signatures.
- [x] `interface` validates typed package compilation references against generated in-memory interfaces.
- [x] resolver, typechecker output, runtime, and package builtin filtering share `prelude::BuiltinId`.
- [x] compiler-provided virtual `std::io`, `std::fs`, `std::path`, `std::env`, and `std::time` packages participate in normal package loading, package interfaces, and `.mgb` artifact emission.
- [x] package rewriting attaches `PackageItemId` to flattened AST record/function declarations so typed HIR no longer recovers item identity from mangled names.
- [x] the package loader can return an unflattened package graph with original package files plus package/module/item/export metadata.
- [x] package enum constructor call targets carry enum `PackageItemId` when the enum comes from package mode.
- [x] package interfaces have a deterministic v2 text format with stable artifact package/item IDs and file write/read helpers.
- [x] persisted package interface round-trip preserves direct dependency metadata, public records, functions, enums, `TypeInfo`, loaded item identity, enum variants, and payload types.
- [x] persisted package interfaces include deterministic content hashes and reject hash mismatches.
- [x] package interface artifact path naming is deterministic for package paths.
- [x] typed package compilation can validate against loaded package interface summaries.
- [x] loaded package interfaces can be used as the dependency boundary for downstream typed checking without reading dependency implementation bodies.
- [x] package interface artifacts can be discovered from an explicit interface root for downstream typed checking.
- [x] interface artifact discovery follows transitive `.mgi` dependencies needed by public signatures.
- [x] missing and hash-mismatched interface artifacts are rejected with regeneration guidance.
- [x] package check cache keys include entry package source hashes and loaded direct/transitive dependency interface hashes.
- [x] missing or stale `.mgc` package check artifacts are rejected with regeneration guidance.
- [x] `muga check --artifact-root <dir>` consumes `.mgi` and `.mgc` artifacts through the package-aware check path without reading dependency implementation bodies.
- [x] persisted `.mgi` artifacts write stable artifact package/item IDs and are remapped to fresh session-local package and item IDs when loaded, avoiding artifact-root collisions between separate provider builds.
- [x] `muga emit-interface` writes `.mgi` artifacts and `muga emit-check-cache` writes `.mgc` only after the package checks successfully against `.mgi` artifacts.
- [x] `muga emit-interface` emits all reachable package interfaces when `--package` is omitted, or one selected package when `--package` is supplied.
- [x] `muga emit-artifacts` writes reachable MIR-lowered bytecode `.mgb` package implementation artifacts alongside reachable `.mgi` interfaces and the entry `.mgc` check cache.
- [x] `muga build <entry>` writes the same artifact set to `.muga/build` under the nearest manifest root, or under the entry file's directory when no manifest is present.
- [x] library-only package-aware checking validates package boundary, import, visibility, and public-signature rules over the unflattened package graph before package-aware module checking.
- [x] package-aware checking builds source and per-module signature environments from the unflattened package graph, preserving package item identity for records/enums/functions, validating generic enum arity, and recording module/same-package/import visibility.
- [x] package-aware checking runs module body resolver/typecheck passes against the module signature environments and retains the per-module resolver/typecheck outputs.
- [x] retained package-aware module typecheck outputs preserve package binding identity needed by typed HIR lowering.
- [x] package-aware checking exposes per-module typed HIR outputs lowered from retained module typecheck outputs.
- [x] package-aware checking collects dependency signatures directly from in-memory or persisted package interfaces without reading dependency source bodies.
- [x] loaded/interface-artifact package-aware checking consumes interface signatures directly; dependency interface AST stubs and stub body checks are no longer part of the typed path.
- [x] loaded-interface package graph construction uses package interfaces directly instead of loading or synthesizing dependency AST modules.
- [x] package-aware check results expose package-wide typed HIR aggregated from per-module outputs without using the legacy flattened typed path.
- [x] default package `check` runs package-aware validation and no longer reloads a flattened package AST after validation.
- [x] default package `compile_typed_path` returns the package-aware typed HIR aggregate instead of the legacy flattened typed HIR.
- [x] flattened package loader APIs are explicitly named `load_flattened_*` so compatibility AST use is visible at call sites.
- [x] interface artifact emission uses the package-aware typed HIR aggregate instead of the legacy flattened typed path.
- [x] loaded/interface-artifact typed compilation returns package-aware typed HIR without loading dependency implementation bodies.
- [x] the legacy `compile_typed_path_against_interfaces` / interface-stub flattened compilation path has been removed.
- [x] bytecode generation consumes `mir::Program`; the legacy untyped AST-to-HIR compatibility module has been removed.
- [x] default `compile_source` / `compile_path` now lower typed HIR into MIR.
- [x] MIR now has explicit entry/function `Body` nodes with body terminators and body-local function definitions, so bytecode compiles execution bodies instead of reading top-level statements and function value blocks directly.
- [x] MIR preserves typed HIR binding and package-item identity on function definitions, parameters, assignments, and identifier uses, and bytecode now carries those identities into runtime name references.
- [x] MIR and bytecode preserve typed assignment mode (`new binding` vs `update`) so runtime no longer infers assignment semantics from name lookup alone.
- [x] bytecode and runtime name references now carry optional semantic `BindingId` plus display symbol, and package function item references are canonicalized to the defining function binding while preserving import bindings in metadata.
- [x] runtime new-binding assignment trusts checked `BindingId` semantics and no longer re-runs shadowing checks through display-name parent-scope lookup.
- [x] bytecode records the CLI entrypoint as a `NameRef`, so runtime invokes `main` by binding identity instead of scanning the root environment by display name.
- [x] bytecode `NameRef` and binding metadata now carry `LocalId`; runtime environments are keyed by lowered local identity while retaining optional `BindingId` for cross-stage identity and diagnostics.
- [x] bytecode records total local capacity and runtime environment storage is now slot-backed by `LocalId` instead of a hash map.
- [x] bytecode exposes a local metadata table for binding-backed and synthetic locals, preparing the next frame-layout step.
- [x] default package `run` lowers package-aware typed HIR through MIR before bytecode generation.
- [x] package-aware typed HIR can lower through the MIR/bytecode VM path for package records/enums/functions.
- [x] explicit artifact-backed package execution reads dependency bytecode bodies from `.mgb` artifacts and does not fall back to dependency source files.
- [x] `.mgb` bytecode bodies are structurally validated on read before bytecode merge/runtime execution, including symbol, local, binding, function, package item, and jump-target references.
- [x] artifact-backed `run` executes transitive dependency implementation artifacts without dependency source files in the consumer tree.
- [x] independently generated `.mgi` and `.mgb` artifacts can be combined when the public interface hash matches; `.mgb` package item references are remapped onto the loaded interface items.
- [x] `.mgb` private package item ids are reserved after the entry program's package item ids before bytecode merge, avoiding collisions between entry-private functions and dependency-private implementation functions.
- [x] artifact-backed `run` has CLI coverage for wrong-package `.mgb` files and stale dependency interface hashes in `.mgb` dependency metadata.
- [x] `samples/packages/app/artifact_facade/main.muga` provides a small `app -> api -> model` package sample that is covered by package execution and interface-backed checking tests.

### Diagnostics

- [x] `Diagnostic` supports primary span, related notes, suggestions, and replacements.
- [x] simple diagnostics still display as one line.
- [x] duplicate declarations can point at previous declarations.
- [x] package visibility diagnostics can point at private declarations and suggest `pkg` or `pub`.
- [x] record literal/update and field diagnostics include declaration-site context in selected cases.
- [x] every current `E005` ambiguity diagnostic points at a targeted annotation strategy.
- [x] user enum diagnostics cover unknown enum/variant constructor references, generic expected-type failures, constructor arity, missing arms, duplicate arms, and foreign arms.
- [x] cross-package diagnostics for persisted interfaces and caches include package/artifact context for missing, stale, hash-mismatched, and invalid artifact cases.

### Collections And Enum-Like Standard Types

- [x] local binding annotations such as `items: List[Int] = []`.
- [x] generic type expressions for compiler-known `List[T]`, `Option[T]`, `Result[T, E]`, and `Map[K, V]`.
- [x] `List[T]` type checking, list literals, empty-list expected-type checking, typed HIR, bytecode, and VM runtime values.
- [x] list `len`, `is_empty`, value-returning `push`, safe `get`, value-returning `set`, and direct indexing.
- [x] direct list indexing returns `T`; negative or out-of-bounds indexes are runtime errors.
- [x] safe list `get` returns `Option[T]`; negative or out-of-bounds indexes return `Option::None`.
- [x] compiler-known `Option[T]`, `Option::Some`, `Option::None`, and exhaustive Option `match`.
- [x] runtime `Option` values now use a generic `EnumValue` shape while preserving the existing `Option::Some(...)` / `Option::None` display and behavior.
- [x] compiler-known enum metadata now describes `Option` and its `Some` / `None` variants.
- [x] parser, resolver, package builtin filtering, typechecker match validation, bytecode lowering, and VM runtime Option branching consume that enum metadata instead of scattering variant strings.
- [x] compiler-known `Result[T, E]`, `Result::Ok`, `Result::Err`, and exhaustive Result `match`.
- [x] runtime `Result` values use the same generic `EnumValue` shape as `Option`.
- [x] in-memory package interface summaries can contain public `Result[T, E]` signatures.
- [x] `Map[K, V]` with `Int`, `Bool`, and `String` keys.
- [x] `Map.empty`, `len`, `is_empty`, `contains`, safe `get`, value-returning `insert`, and value-returning `remove`.
- [x] user-defined `enum` declarations with optional unconstrained type parameters.
- [x] user-defined enum zero-payload and one-payload variants.
- [x] qualified user enum construction and patterns with exhaustive `match`.
- [x] payload discard `_` is implemented for compiler-known and user-defined one-payload enum variant patterns without introducing a binding.
- [x] user-defined enum runtime values use the same generic `EnumValue` display shape.
- [x] typed HIR and in-memory package interface summaries preserve public user enum declarations and public signatures containing user enum types.
- [x] imported package enum constructors and patterns such as `alias::Enum::Variant` are covered.
- [x] public, `pkg`, and module-private enum visibility cases are covered.
- [x] in-memory package interface validation catches stale enum identity, type parameter, variant, and payload mismatches.
- [ ] map literals are deferred.
- [ ] arbitrary map key types are deferred.
- [ ] `Set[T]` is deferred.
- [x] prefix `try expr` `Result` propagation is implemented and hardened for source, nested control flow, closures, and artifact-backed dependency execution.
- [x] first `String` helper builtins are implemented: `is_empty`, `contains`, `trim`, `char_count`, `byte_len`, `starts_with`, `ends_with`, `replace`, `split`, `concat`, `slice_chars`, `parse_int`, and `parse_bool`.
- [x] first explicit formatting helpers are implemented: `to_string` for `Int`, `Bool`, and `String`.
- [x] short-circuit `and` / `or` keyword operators are implemented through lexer, parser, typechecker, typed HIR, MIR, bytecode jump lowering, and runtime tests.
- [x] `break` / `continue` are implemented through lexer, parser, typechecker, typed HIR, MIR, bytecode scope-unwind jump lowering, and runtime tests.
- [x] `for item in list` is implemented for `List[T]` through lexer, parser, resolver, typechecker, typed HIR, MIR, bytecode list-index lowering, runtime, and artifact round-trip tests.
- [x] `Unit` and the `()` literal are implemented through parser, typechecker, typed HIR, MIR, bytecode/runtime, package interfaces, and `.mgb` implementation artifacts.
- [x] first practical stdlib package slice is implemented: `std::io::IOError`, `std::io::PathPairError`, `std::fs::read_text(path): Result[String, io::IOError]`, and `std::fs::write_text(path, text): Result[Unit, io::IOError]`.
- [x] minimal `std::path` package slice is implemented: transparent `std::path::Path`, `path::from_string(text): Path`, `path::as_string(path): String`, `path::join(base, child): Path`, `path::file_name(path): Option[String]`, `path::parent(path): Option[Path]`, `path::extension(path): Option[String]`, `path::file_stem(path): Option[String]`, and `path::is_absolute(path): Bool`.
- [x] Path-aware `std::fs` text-file helpers are implemented: `fs::read_text_path(path::Path): Result[String, io::IOError]` and `fs::write_text_path(path::Path, text): Result[Unit, io::IOError]`.
- [x] Path-aware `std::fs` directory listing is implemented: `fs::read_dir_path(path::Path): Result[List[path::Path], io::IOError]` returns sorted direct entries as `path::Path` values.
- [x] Path-aware `std::fs` directory creation is implemented: `fs::create_dir_path(path::Path): Result[Unit, io::IOError]` creates exactly one directory and reports recoverable failures through `io::IOError`.
- [x] Path-aware `std::fs` recursive directory creation is implemented: `fs::create_dir_all_path(path::Path): Result[Unit, io::IOError]` creates missing parent directories and succeeds when the target directory already exists.
- [x] Path-aware `std::fs` single-file removal is implemented: `fs::remove_file_path(path::Path): Result[Unit, io::IOError]` removes one filesystem file and reports recoverable failures through `io::IOError`.
- [x] Path-aware `std::fs` empty-directory removal is implemented: `fs::remove_dir_path(path::Path): Result[Unit, io::IOError]` removes one empty filesystem directory and reports recoverable failures through `io::IOError`.
- [x] Path-aware `std::fs` single-file copy is implemented: `fs::copy_file_path(from: path::Path, to: path::Path): Result[Unit, io::PathPairError]` copies one filesystem file, overwrites an existing target file when the host filesystem permits it, and reports recoverable failures through `io::PathPairError`.
- [x] Path-aware `std::fs` metadata predicates are implemented: `fs::exists_path(path::Path): Bool`, `fs::is_file_path(path::Path): Bool`, and `fs::is_dir_path(path::Path): Bool`; `fs::PathStatus` plus `fs::path_status(path::Path): PathStatus` now groups those predicate results without adding rich all-path metadata.
- [x] minimal `std::env` package slice is implemented: `env::get_var(name): Option[String]` returns `Option::None` for missing or non-Unicode process environment variables, and `env::args(): List[String]` reads CLI/library program arguments supplied after `--`.
- [x] minimal `std::time` package slice is implemented: transparent `std::time::UnixMillis` and `time::now_unix_millis(): UnixMillis`.
- [x] package-aware typechecking installs transitive public package record signatures needed by imported function return/parameter types, so `std::fs` callers can inspect `IOError` and `PathPairError` fields without directly importing `std::io`, while explicit source annotations can still use `import std::io` or an alias.

## Architecture Facts To Keep In Mind

- The VM/bytecode path is the current execution backend and should remain a reference backend.
- typed HIR is the semantic boundary for package interfaces and MIR lowering.
- The default compile APIs and bytecode backend now consume an initial expression-shaped MIR with explicit entry/function bodies, body terminators, hoisted body-local function definitions, typed HIR binding/package-item identity, and typed assignment update mode. Bytecode/runtime name references carry optional binding identity, lowered local identity, and display symbols; runtime environments are slot-backed by `LocalId`; and package function item references resolve to the defining function binding at bytecode lowering. MIR is now the only backend-facing IR while it is matured toward a control-flow-oriented backend IR.
- `Option[T]` and `Result[T, E]` remain compiler-known enum-like types for now; user-defined enums use a parallel source-level enum model.
- `match` supports compiler-known `Option[T]` / `Result[T, E]` and user-defined enums; match patterns are represented internally as enum variant patterns with explicit payload mode.
- Runtime enum-like values use a generic enum-value representation.
- Runtime `Unit` is a first-class value and should be used as the success payload for effect-only `Result` APIs.
- `std::io::IOError` is currently a transparent record for one-shot text IO: `operation`, `path`, `kind`, `message`, and `raw_code: Option[Int]`. `std::io::PathPairError` mirrors those details with `from_path` and `to_path` for two-path filesystem operations.
- `Map` runtime storage is a simple vector of key/value entries, which is correct for semantics but not a final performance representation.
- Package interfaces now have a deterministic v2 text format with stable artifact package/item IDs and file round-trip helpers.
- Loaded package interface summaries can now act as the downstream dependency boundary for typed checking.
- A library API can discover dependency `.mgi` artifacts from an explicit interface root for typed checking.
- Interface artifacts now record direct dependencies, and artifact discovery follows those dependencies so public signatures can mention types from transitive packages without reading dependency bodies.
- A library API can compute package check cache keys and validate `.mgc` artifacts against source plus loaded dependency interface hashes.
- CLI `check --artifact-root` can consume `.mgi` and `.mgc` artifacts without reading dependency implementation bodies.
- CLI `emit-interface` and `emit-check-cache` can produce the artifacts consumed by `check --artifact-root`, with `.mgc` emission gated by a successful package-aware artifact check.
- CLI `emit-interface` can emit all reachable interfaces without manually naming each dependency package.
- CLI `emit-artifacts` emits reachable `.mgi` interfaces, reachable MIR-lowered bytecode `.mgb` implementation artifacts, and the entry `.mgc` check cache in one explicit artifact-root workflow; CLI `build` emits the same artifact set to `.muga/build`, and CLI `check --built` / `run --built` consume it explicitly.
- CLI usage distinguishes `check` from `run` program-argument syntax, and `run --built <entry> -- args...` is covered through `std::env::args`.
- Default build artifact diagnostics reached through `--built` append direct `muga build <entry>` guidance on package artifact failures, while custom `--artifact-root` diagnostics keep the broader explicit-root suggestions. Missing/stale `.mgc` check-cache diagnostics also mention `muga emit-check-cache` for focused cache regeneration.
- Manifest projects support local path dependencies through `[dependencies] name = { path = "..." }` and local archive dependencies through `[dependencies] name = { archive = "...", hash = "sha256:<hex>" }`; dependency keys must match target manifest package names, and imported logical package paths resolve through dependency source roots without filesystem paths in `.muga` files.
- `muga build` preserves unchanged generated artifacts under `.muga/build` and reports each artifact as `written` or `reused` on stdout; `.mgi` reuse is keyed by persisted interface hash, while `.mgb` and `.mgc` reuse currently requires identical generated artifact text.
- `.mgb` implementation artifacts record package-local source hashes separately from public interface hashes and dependency interface hashes, giving future rebuild planning an implementation-input key without stretching `.mgc`.
- `.mgi` public interface hashes ignore diagnostic-only spans while preserving those spans in artifact text, so implementation-only body/span movement can reuse public interface artifacts and keep downstream dependency hashes stable.
- `muga build` derives deterministic dependency levels from the loaded package graph and builds independent package `.mgi` / `.mgb` artifacts in the same level concurrently while preserving deterministic artifact result ordering.
- `muga build` writes or updates `muga.lock` beside the manifest with local path dependency source descriptors plus SHA-256 `source_hash` metadata and local archive dependency descriptors plus `hash` metadata; existing well-formed local lockfiles are refreshed when stale, while malformed, duplicate, graph-inconsistent, or unsupported local lockfiles fail with `PK026`.
- Library package content hashing returns `sha256:<hex>` over `muga.toml`, sorted `.muga` files under the manifest source root, and optional manifest-declared resource bytes. It shares the deterministic input shape with local path `source_hash` metadata but stays a separate future published-package identity helper.
- `muga emit-package-archive [--format text|json] --archive-root <dir> <entry>` writes deterministic `.mgp` source/resource archives from the canonical content input, skipping `.muga` / `.git` tool directories.
- Library `.mgp` readback validates archive bytes against optional expected `sha256:<hex>` values and rejects malformed manifest/source/resource entry layout.
- CLI/library `.mgp` materialization writes validated archive bytes into an absent or empty local source/resource tree, preserves the content hash, rejects unsafe manifest source/resource roots, and rejects non-empty destinations.
- Local archive dependencies reuse the `.mgp` verifier, materialize or reuse `.muga/packages/<package>-sha256-<hash>` cache roots including declared resources, reject malformed forms, stale or colliding cache roots, and package-name mismatches, and load the cached package through the normal package graph.
- The package loader can now return unflattened package files with the same package graph/export metadata used by the legacy flattening path.
- A library-only package-aware check entrypoint validates package boundary, import, visibility, and public-signature rules directly over the unflattened package graph before package-aware module checking.
- The package-aware source and module signature environments resolve same-package and imported public record/enum/function signatures from the unflattened graph while preserving `PackageItemId` identities and source-visible module names.
- The package-aware check entrypoint now runs module body resolution/typechecking with those module signatures and retains per-module resolver/typecheck outputs.
- Retained package-aware module typecheck outputs now carry package binding identity through typed HIR lowering, so module-local lowering can preserve package item call targets without relying on flattened AST metadata.
- The package-aware API now exposes those lowered per-module typed HIR programs alongside each module typecheck output.
- The package-aware API can now collect dependency signatures directly from in-memory or persisted package interfaces, letting package-aware module checks run without dependency implementation source or synthesized interface AST modules.
- Loaded-interface package-aware checks now build dependency package graph metadata directly from package interfaces instead of loading or synthesizing dependency AST modules.
- The legacy interface-stub flattened typed compilation path has been removed; loaded/interface-artifact typed compilation now has one package-aware semantic path.
- Package-aware check results now expose package-wide typed HIR aggregated from per-module outputs, with local binding/statement/expression IDs and symbols remapped into one typed HIR program.
- CLI default package `check`, default package `compile_typed_path`, `check --artifact-root`, interface artifact emission, and loaded/interface-artifact typed compilation now use package-aware paths; default package `check` no longer reloads a flattened AST after validation.
- Remaining flattened package loader APIs now use explicit `load_flattened_*` names.
- Package-aware typed HIR can now lower through the MIR/bytecode VM path for package records, enums, functions, and calls.
- Default package execution now lowers package-aware typed HIR through MIR before bytecode generation, while still reading dependency bodies.
- Project-mode artifact-root config is intentionally deferred until lockfiles and a package-aware project driver exist; `--built` is fixed default-directory CLI sugar, not manifest configuration.
- Full incremental artifact reuse, package-local rebuild planning, full published-package lockfile enforcement, URL/Git/registry dependency resolution, remote package fetching, and publish/install workflows are still not implemented.

## Recommended Next Implementation

The generic records/functions foundation, prefix `try expr` propagation, short-circuit `and` / `or`, `else if`, explicit `return expr`, `break` / `continue`, `for item in list`, payload discard `_` inside enum variant patterns, first `String` helper builtins, `Unit`, first `std::io` / `std::fs` text-file slice, minimal `std::path` wrapper with path joining, parent lookup, file-name/stem extraction, extension extraction, and absolute-path classification, Path-aware text-file helpers, directory listing, directory creation, recursive directory creation, single-file removal, empty-directory removal, single-file copy, metadata predicates plus `PathStatus` grouping, `std::env::get_var`, `std::env::args`, `std::time::now_unix_millis`, `std::test` scalar assertions, `std::option` / `std::result` value helpers, `std::list` / `std::map` collection helpers, scalar-only v1 equality policy, line-comment-preserving deterministic `muga fmt`, `muga build` default artifact emission with written/reused reporting, `muga build --format json` artifact status output, explicit artifact emission JSON output, dependency/source/artifact hash plus regeneration-command JSON context for artifact diagnostics, `check --built` / `run --built` default artifact consumption, local path dependency metadata, unchanged build artifact reuse, `.mgb` package-local source-hash metadata, public interface hash stability for implementation-only changes, dependency-level package builds, local lockfile/archive hardening, targeted diagnostics, command-output contracts, initial `muga syntax --format json`, entry-aware `check --format json`, initial CLI JSON diagnostic entry source context, artifact-backed check JSON entry package, artifact-root, and concrete artifact-file context, initial `muga metadata --format json`, initial `muga workspace --format json`, initial `muga hover --format json`, initial `muga completions --format json`, initial `muga definition --format json`, initial `muga references --format json`, minimal `muga doc` with public source comments, CLI-first `muga new` app template plus lib/test templates, the initial `muga test` / `@test` workflow, `muga test --format json`, `muga run --format json`, `muga explain <diagnostic-code>`, the concrete JSON-backed editor workflow smoke test, the artifact/cache explanation design and `muga why-rebuild` implementation with text output, JSON output, stale dependency-interface coverage, implementation dependency-interface set-change hash context, lockfile metadata, and local archive-cache metadata coverage in [artifact-cache-explanations.md](artifact-cache-explanations.md), representative composite artifact-backed dependency API coverage without source-body fallback, `.mgi` public interface hash stability coverage across implementation-only edits and source-span movement, `.mgb` structural validation and bytecode merge coverage for control-flow-heavy independently generated dependency implementations, `muga build` reuse output and lockfile update behavior coverage for local path/local archive dependencies after implementation-only edits, public signature edits, and malformed lockfiles, recursive annotation diagnostics with concrete parameter/return signature suggestions for direct and mutual recursion, runtime call-context related notes for `run` and `test` diagnostics, source-spanned `R021` diagnostics for failed `std::test` scalar assertions, runtime/debug v1 boundary documentation that keeps stack context in `related` notes and artifact next-actions in `regenerationCommand` context, release-neutral benchmark health checks in [benchmark-health-checks.md](benchmark-health-checks.md), fuzzing and malformed-input planning in [fuzzing-malformed-input-plan.md](fuzzing-malformed-input-plan.md), release-neutral installation and onboarding in [installation-and-onboarding.md](installation-and-onboarding.md), example-driven learning in [muga-by-example.md](muga-by-example.md), future registry security design in [registry-security-design.md](registry-security-design.md), edition and semantic feature-set fingerprint policy in [edition-feature-fingerprint-policy.md](edition-feature-fingerprint-policy.md), and the runnable local-dependency report sample have landed. Package-mode public signatures now have representative coverage for every v1-supported public type shape through in-memory and persisted interfaces. The stdlib package docs and samples review now covers `std::io`, `std::fs`, `std::path`, `std::env`, `std::cli`, `std::time`, `std::string`, `std::fmt`, and the first `std::json` slice, including artifact-backed execution samples where useful. The release gate and GitHub Actions are aligned through [release-gate-alignment.md](release-gate-alignment.md), with workflows invoking `scripts/v1-release-gate.sh` directly. The first `std::json` package contract is implemented from [std-json-first-slice.md](std-json-first-slice.md) and audited in [std-json-implementation-audit.md](std-json-implementation-audit.md). The first `pub opaque type` interface slice, metadata-only `OpaqueHandleFacts` / `paramMode` interface slice, consuming-parameter checker, read-only and write-mode `std::fs::File` runtime handles, post-file-handle selection, scalar `eprint` / `eprintln` program stderr channel, integrated `report_app` workflow, statement-form `using` cleanup with nested unwind hardening, first pure `std::cli` helpers, CLI-first app template refresh, typed scalar `std::cli` parsing helpers, post-typed-cli selection, JSON value accessor helpers, the JSON config workflow sample, result mapping refresh, `std::string` text assembly helpers, required and composite `std::json` helpers, nested JSON config workflow refresh, post-nested selection, scalar array projection helpers, the post-json-array-projection selection, direct JSON scalar-array object-field helpers, the post-direct-json-array-field selection, repeated `std::cli` option value helpers, the post-repeated-cli-option selection, JSON path helpers, the post-json-path selection, typed JSON path scalar projection helpers, typed JSON path collection projection helpers, the post-typed-json-path-collection selection, `std::config::load_json_or[T]`, the generated `muga new --template config-app` template, strict `json::decode[T](value)`, the post-required-decoder adoption selection, structural `Option[T]`, recursive `List[T]`, typed `Map[String, T]`, concrete enum decoder targets, JSON/config schema polish, `@json(rename: "...")`, `@json(deny_unknown_fields)`, `@json(alias: "...")`, field-level `@validate(...)`, post-validation schema export selection, JSON/config schema export design, `muga schema --format json` schema export implementation, post-schema-export typed JSON encoding selection, typed JSON encoding design, typed JSON encoding implementation, post-typed-JSON-encoding full CLI parser schema design selection, CLI parser schema design, first CLI parser schema implementation, generated `config-app` CLI schema adoption, post-config-app CLI schema adoption gap selection, generated config-app usage adoption, post-config-app usage adoption gap selection, first `@cli(...)` field metadata implementation, generated config-app CLI metadata adoption, post-config-app CLI metadata adoption gap selection, strict CLI parser schema design, strict CLI parser schema implementation, post-strict CLI parser adoption gap selection, strict CLI tool sample adoption, post-strict CLI tool sample adoption gap selection, generated cli-tool template adoption, post-generated cli-tool template adoption gap selection, strict CLI manual help adoption, post-strict CLI manual help adoption gap selection, strict CLI no-default usage helper design, strict CLI no-default usage helper implementation, CLI command metadata design, CLI command metadata implementation, post-CLI command metadata adoption gap selection, CLI short option metadata design, CLI short option metadata implementation, and post-CLI short option metadata adoption gap selection, CLI positional field metadata design, CLI positional field metadata implementation, and post-CLI positional field metadata adoption gap selection, and built-in CLI help policy design, built-in CLI help helper implementation, post-built-in CLI help helper adoption gap selection, and parse-integrated CLI help workflow design, parse-integrated CLI help workflow implementation, and post-parse-integrated CLI help workflow adoption gap selection, compact CLI short option syntax design, compact CLI short option syntax implementation, post-compact CLI short option syntax adoption gap selection, and CLI subcommand metadata design, first enum metadata plumbing, strict command enum schemas/runtime dispatch in [cli-subcommand-metadata.md](cli-subcommand-metadata.md), and strict CLI sample/template command-tree adoption in [post-cli-subcommand-schema-adoption-gap-selection.md](post-cli-subcommand-schema-adoption-gap-selection.md), and wrapper-record root/global CLI option design, first `@cli(subcommand)` metadata plumbing, and wrapper schema/runtime parse/help support, and strict CLI sample/template wrapper adoption in [cli-wrapper-root-options.md](cli-wrapper-root-options.md), schema-backed generated app completion generation in [cli-schema-shell-completions.md](cli-schema-shell-completions.md), generated app completion onboarding and packaging hooks in [post-cli-schema-shell-completion-adoption-gap-selection.md](post-cli-schema-shell-completion-adoption-gap-selection.md), shell-agnostic JSON completion specs in [cli-completion-json-spec.md](cli-completion-json-spec.md), and richer nested generated-app completion traversal are implemented/documented. The next implementation-facing step should choose between TOML/config discovery, richer dynamic value source metadata, or installer integration now that completion packaging has shell, JSON, and nested traversal coverage.

Reasoning:

Update: static CLI completion value-source metadata and non-mutating completion
package emission have now landed after nested completion traversal. The active
next choice is TOML/config discovery versus continued release-channel polish;
dynamic completion producers and shell-profile installation remain deferred
until Muga has an execution and host-effect policy for completion callbacks.

- The package-aware semantic boundary is now real enough for `check`; v1 risk has moved from name resolution/flattening to execution and artifact workflow closure.
- The reference VM now consumes MIR-lowered bytecode with binding/local identity metadata and slot-backed locals, so persisted dependency implementations can be keyed by compiler-owned identity instead of display-name lookup.
- `.mgi` should remain a public-signature artifact, `.mgc` should remain a check-cache proof, and `.mgb` should remain a separate implementation/execution artifact that stores bytecode bodies generated through MIR rather than overloading either existing format or persisting dependency source.
- Artifact-backed `run` fails loudly when required dependency execution artifacts are missing, stale, hash-mismatched, structurally invalid, or inconsistent with loaded interfaces. It should continue to avoid silently falling back to dependency source bodies under `--artifact-root`.
- `muga.toml` should not name an artifact root yet. The manifest currently owns `[package] name/source` and local path dependency roots; adding build/cache configuration before lockfiles and a package-aware project driver would make ordinary project `check` and `run` semantics ambiguous.
- Control-flow MIR, native lowering, broad stdlib effects, and wildcard-heavy or catch-all matching should remain out of the v1 path unless they become necessary to make artifact-backed execution correct.

## Requirement Decisions For The Next Slice
Closed before coding artifact-backed execution:

- [x] Keep `.mgi` as the public interface artifact.
- [x] Keep `.mgc` as the check-cache proof keyed by entry source and dependency interface hashes.
- [x] Add `.mgb` as the separate implementation/execution artifact; store MIR-lowered bytecode bodies, not source bodies, and do not overload `.mgi` or `.mgc` for executable code.
- [x] Keep `--artifact-root` explicit for v1; do not add `muga.toml` artifact-root configuration before lockfiles and a package-aware project driver.
- [x] Artifact-backed `run` must reject missing, stale, hash-mismatched, structurally invalid, or mismatched execution artifacts instead of falling back to dependency source bodies.
- [x] Default `run` without `--artifact-root` should remain source-compatible while v1 artifact execution is introduced.
- [x] Control-flow MIR and native lowering are deferred unless expression-shaped MIR cannot correctly represent the first execution artifact.

Earlier enum/result decisions remain settled:

- [x] `Result[T, E]` landed first as a compiler-known enum-like standard type.
- [x] The enum declaration syntax is `enum Name[T, E] { Variant | Variant(Type) }`.
- [x] The MVP supports zero-payload and one-payload variants only.
- [x] Variant constructors and patterns are always qualified as `Enum::Variant`.
- [x] Match patterns must be exhaustive with no wildcard in the MVP.
- [x] Package-mode enum declarations use `PackageItemId`.
- [x] Public enum declarations appear in in-memory package interface summaries.
- [x] Use prefix `try expr` for the implemented `Result` propagation form; if chain propagation is added later, use postfix keyword `expr.try`, not postfix `expr?`.

## Implementation Plan And Estimate
Estimates are in focused engineering days for someone already familiar with this codebase. They include tests and documentation, not just code edits.
| Slice | Scope | Main files | Estimate | Risk |
|---|---|---|---:|---|
| 1. Enum/ADT internal model | Generalize the Option-specific representation into an enum-like internal model, without changing source behavior. AST/typed HIR/MIR pattern shape, runtime enum value shape, compiler-known enum metadata, and generic two-variant bytecode/runtime branching are in place. | `src/typing.rs`, `src/typed_hir.rs`, `src/mir.rs`, `src/bytecode.rs`, `src/runtime.rs`, `tests/examples.rs` | Done | Low |
| 2. `Result[T, E]` standard type | Add compiler-known `Result::Ok`, `Result::Err`, and exhaustive `Result` match. Reuse the known enum metadata table and generic runtime enum value shape; the later `Result` propagation slice covers `try expr`. | `src/known_enum.rs`, `src/parser.rs`, `src/typing.rs`, `src/mir.rs`, `src/bytecode.rs`, `src/runtime.rs`, `src/typed_hir.rs` | Done | Medium |
| 3. Enum declaration syntax MVP | Parse and typecheck user-defined enum declarations with optional unconstrained type parameters and zero/one-payload variants. Add runtime representation and typed HIR/interface summaries. | parser/AST/typechecker/HIR/bytecode/runtime/package/typed HIR/tests | Done | High |
| 4. Enum integration hardening | Expand diagnostics, package visibility cases, interface stale checks, and compatibility coverage after the MVP is green. | package/interface/typed HIR/tests/docs | Done | Medium |
| 5. Package interface persistence format | Serialize public records/functions/enums and resolved type identities in a deterministic v2 text format with stable artifact package/item IDs. Load the format back into `PackageInterfaceGraph` and validate the reloaded summaries. | `src/interface.rs`, `tests/examples.rs` | Done | Medium |
| 6. Interface hashes and loaded-interface validation | Add interface hashes, artifact path conventions, and a typed checking path that validates against loaded interface summaries. | `src/interface.rs`, `src/lib.rs`, tests | Done | Medium |
| 7. Downstream checking without dependency bodies | Load dependency interfaces as the checking boundary, synthesize or otherwise expose only public signatures, and avoid reading dependency implementation bodies for downstream checks. | `src/package.rs`, `src/interface.rs`, `src/lib.rs`, tests | Done | High |
| 8. Interface artifact discovery | Teach package checking to find persisted interface artifacts from an explicit interface root and reject missing/hash-mismatched/stale artifacts. | `src/interface.rs`, `src/package.rs`, `src/lib.rs`, tests | Done | High |
| 9. Package cache keys and invalidation | Define source/interface/dependency hash inputs, persist checked-package metadata, reject missing/stale cache artifacts, and keep cache-backed checking aligned with body checking. | `src/cache.rs`, `src/package.rs`, `src/lib.rs`, tests | Done | High |
| 10. CLI artifact-root checking | Expose a narrow CLI path for artifact-backed checking using `.mgi` and `.mgc` artifacts. | `src/main.rs`, `src/lib.rs`, tests/docs | Done | Medium |
| 11. CLI artifact generation | Add CLI/library artifact generation for `.mgi` and `.mgc`, and verify generated artifacts drive `check --artifact-root`. | `src/main.rs`, `src/lib.rs`, `src/interface.rs`, tests/docs | Done | Medium |
| 12. Combined artifact emission | Keep artifact roots explicit on the CLI and add `emit-artifacts` to write reachable `.mgi` plus entry `.mgc` in one command. | `src/main.rs`, `src/lib.rs`, tests/docs | Done | Low |
| 13. Transitive interface artifact reuse | Persist direct dependencies in `.mgi`, load transitive public-signature type interfaces, and include the loaded interface set in `.mgc` keys. | `src/interface.rs`, `src/package.rs`, `src/cache.rs`, tests/docs | Done | High |
| 14. Unflattened package graph loader | Return package files plus package/module/item/export metadata before flattening so resolver/typechecker migration has a stable input. | `src/package.rs`, tests/docs | Done | Medium |
| 15. Package-aware checking without flattening | Done for the current package checking surface: library-only package-aware boundary checking, source/module signature collection, retained module resolver/typecheck outputs, package-wide typed HIR aggregation, default package `check` and `compile_typed_path`, direct interface-backed dependency signatures/graph metadata, and removal of the interface-stub flattened typed path now run over the unflattened package graph while keeping artifact semantics explicit. Remaining v1 work is dependency-body-free execution and workflow hardening. | package/resolver/typing/lib/tests | Done | High |
| 16. MIR/runtime identity foundation | Route package-aware typed HIR through MIR into bytecode with explicit body nodes, binding/package-item identity, assignment mode, `NameRef` local identity, slot-backed runtime locals, entrypoint identity, and synthetic local metadata. | `src/mir.rs`, `src/bytecode.rs`, `src/runtime.rs`, `src/lib.rs`, tests/docs | Done | Medium |
| 17. Dependency-body-free execution | Added an explicit artifact-backed `run` path that validates `.mgi` / `.mgc`, loads separate MIR-lowered bytecode `.mgb` dependency implementation artifacts, and executes package dependencies without reading source files from the dependency source tree. `emit-artifacts` writes every artifact needed by this path. | `src/main.rs`, `src/lib.rs`, `src/cache.rs`, `src/interface.rs`, `src/implementation_artifact.rs`, tests/docs | Done | Medium |
| 18. V1 package workflow hardening | Document and test the explicit artifact workflow end to end, including transitive `.mgb` execution, independent `.mgi`/`.mgb` identity remapping, private item id reservation during bytecode merge, broader missing/stale/mismatched artifact diagnostics, default source-compatible execution, and sample package/project commands. | `README.md`, `ROADMAP.md`, `docs/*`, `tests/examples.rs`, samples | Done | Medium |
| 19. User-defined generic records/functions | Add explicit declaration type parameters for records and functions, call-site inference for ordinary use, and persisted package-interface support for generic public signatures without bounds, specialization, or polymorphic recursion. | parser/types/typing/package/interface/tests/docs | Done | High |
| 20. Generic surface hardening | Added docs/samples and targeted diagnostics for generic arity, ambiguous generic record literals, package-interface round trips, and stale generic signature checks. Keep expanding edge cases as the package/API surface grows. | docs/samples/tests/typing/interface | Done | Medium |
| 21. Result propagation | Implemented prefix `try expr` propagation for `Result`, including exact type rules, MIR/bytecode lowering, source diagnostics, artifact-backed dependency execution, nested control-flow coverage, and closure return behavior. | `src/token.rs`, `src/parser.rs`, `src/typing.rs`, `src/typed_hir.rs`, `src/mir.rs`, `src/bytecode.rs`, `tests/examples.rs`, specs/docs | Done | High |
| 22. First `String` helper builtins | Add low-risk string helpers that do not require byte-versus-character length semantics: `is_empty`, `contains`, `trim`, `starts_with`, `ends_with`, `replace`, and `split`, with runtime, typing, samples, and artifact-backed dependency coverage. `replace("", new)` is a no-op and `split("")` returns `[self]`. | `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 23. Fallible string parse helpers | Add `String.parse_int(): Result[Int, String]` and `String.parse_bool(): Result[Bool, String]` so practical code can exercise `try` with builtin fallible APIs while postponing richer parse error types. | `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 24. Explicit string character count | Add `String.char_count(): Int` as Unicode scalar-value counting, with runtime, typing, sample, source tests, and artifact-backed dependency coverage. This avoids overloading `String.len()` before byte/character/grapheme semantics are settled. | `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 25. Explicit string scalar slicing | Add `String.slice_chars(start: Int, count: Int): Result[String, String]` as zero-based Unicode scalar-value slicing, returning `Result::Err("invalid slice range")` for negative or out-of-range slices. Cover source, Unicode, type errors, samples, and artifact-backed `try` execution. | `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 26. String receiver diagnostic hardening | Keep string-helper receiver inference for unannotated parameters while reporting targeted `T006` diagnostics for concrete non-`String` receivers across unary, predicate, transform, slicing, and parse helpers. | `src/typing.rs`, `tests/examples.rs`, docs | Done | Low |
| 27. `try` expression diagnostic hardening | Report targeted `T023` diagnostics for obvious non-`Result` `try` operands while preserving expected-type inference for `try Result::Ok(...)` and similar constructor-heavy expressions. | `src/typing.rs`, `tests/examples.rs`, docs | Done | Low |
| 28. Explicit scalar formatting helpers | Add `to_string` for `Int`, `Bool`, and `String` plus `String.concat(other): String` as the first formatting primitive, without implicit conversion, interpolation, template formatting, or builder APIs. Cover source, ambiguity diagnostics, samples, and artifact-backed dependency execution. | `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 29. `Unit` value/type foundation | Add `Unit` and `()` so effect-only APIs can return `Result[Unit, E]` without inventing meaningless success `Bool`/`Int` values. Persist `Unit` through public package interfaces and `.mgb` bytecode artifacts. | parser/types/typing/typed HIR/MIR/bytecode/runtime/interface/artifacts/tests/docs | Done | Low |
| 30. Minimal std text IO package slice | Add compiler-provided virtual `std::io` and `std::fs` packages with transparent `IOError`, `fs::read_text`, and `fs::write_text`. Implement runtime text IO through internal builtins called by package functions, and cover direct package execution plus emitted `.mgi/.mgb` artifact execution. | `src/std_package.rs`, `src/package.rs`, `src/prelude.rs`, `src/resolver.rs`, `src/typing.rs`, `src/runtime.rs`, tests/docs | Done | Medium |
| 31. Explicit string byte length | Add `String.byte_len(): Int` as UTF-8 byte-size counting, distinct from scalar-value `char_count()` and fallible scalar slicing. Cover source, receiver diagnostics, samples/docs, and artifact-backed dependency execution. | `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 32. Minimal std path package slice | Add compiler-provided virtual `std::path` with transparent `Path`, `path::from_string`, and `path::as_string`. Cover direct package execution, explicit record literals, missing-import diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/package.rs`, `src/package_signature.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 33. Path-aware std fs text helpers | Add `fs::read_text_path` and `fs::write_text_path` as non-overloaded `std::path::Path` bridges while preserving existing `String` path APIs. Cover source, type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 34. Minimal std env package slice | Add compiler-provided virtual `std::env` with `env::get_var(name): Option[String]`. Cover present/missing environment variables without mutating process env, missing-import diagnostics, type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 35. Minimal std time package slice | Add compiler-provided virtual `std::time` with transparent `UnixMillis` and `time::now_unix_millis(): UnixMillis`. Cover source range checks, explicit record literals, missing-import diagnostics, argument-count diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 36. Process args std env slice | Add `env::args(): List[String]` plus CLI/library runtime plumbing for arguments passed after `--`. Cover source execution, default empty args, CLI default and artifact-backed `run`, non-`run` separator rejection, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/main.rs`, `src/lib.rs`, `src/runtime.rs`, `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `tests/examples.rs`, samples/docs | Done | Medium |
| 37. Path metadata std fs slice | Add `fs::exists_path(path::Path): Bool`, `fs::is_file_path(path::Path): Bool`, and `fs::is_dir_path(path::Path): Bool` as non-throwing metadata predicates. Cover source execution, missing path behavior, Path type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 38. Path directory creation std fs slice | Add `fs::create_dir_path(path::Path): Result[Unit, io::IOError]` as a non-recursive one-directory creation API. Cover success, existing-directory error shape, Path type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 39. Path join std path slice | Add `path::join(base: Path, child: String): Path` as the first path construction helper beyond transparent conversion. Cover direct package execution, child type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 40. Path directory listing std fs slice | Add `fs::read_dir_path(path::Path): Result[List[path::Path], io::IOError]` as a deterministic direct-directory listing API. Cover sorted success output, missing directory error shape, Path type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 41. Path file-name std path slice | Add `path::file_name(path: Path): Option[String]` as the first path inspection helper for directory-listing workflows. Cover `Some`, `None`, Path type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 42. Path parent std path slice | Add `path::parent(path: Path): Option[Path]` as the first parent traversal helper for directory-listing workflows. Cover `Some`, single-component `None`, Path type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 43. Path extension std path slice | Add `path::extension(path: Path): Option[String]` as a path classification helper for directory-listing workflows. Cover `Some`, missing-extension `None`, Path type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 44. Path file-stem std path slice | Add `path::file_stem(path: Path): Option[String]` as the counterpart to `path::extension`. Cover `Some`, empty-path `None`, Path type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 45. Recursive directory creation std fs slice | Add `fs::create_dir_all_path(path::Path): Result[Unit, io::IOError]` as the recursive counterpart to `create_dir_path`. Cover nested directory creation, existing-directory success, Path type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 46. Path absolute classification std path slice | Add `path::is_absolute(path: Path): Bool` as a pure host-path classification helper. Cover absolute and relative path results, Path type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 47. Single-file removal std fs slice | Add `fs::remove_file_path(path::Path): Result[Unit, io::IOError]` as the narrowest filesystem removal API. Cover successful removal, missing-file error shape, Path type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 48. Empty-directory removal std fs slice | Add `fs::remove_dir_path(path::Path): Result[Unit, io::IOError]` as non-recursive directory removal. Cover successful removal, non-empty directory error shape, Path type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 49. Single-file copy std fs slice | Add `io::PathPairError` and `fs::copy_file_path(from: path::Path, to: path::Path): Result[Unit, io::PathPairError]` as the first two-path filesystem operation. Cover success, overwrite behavior, missing-source error fields, Path type mismatch diagnostics, emitted `.mgi/.mgb` artifact execution, and a runnable package sample. | `src/std_package.rs`, `src/prelude.rs`, `src/typing.rs`, `src/runtime.rs`, `tests/examples.rs`, samples/docs | Done | Low |
| 50. Short-circuit Boolean operators | Add `and` / `or` as Bool-only keyword operators with `and` tighter than `or`, left-to-right evaluation, short-circuit bytecode lowering, type diagnostics, source docs, and runtime tests proving skipped right operands are not evaluated. | `src/token.rs`, `src/lexer.rs`, `src/parser.rs`, `src/typing.rs`, `src/typed_hir.rs`, `src/mir.rs`, `src/bytecode.rs`, `tests/examples.rs`, specs/docs | Done | Low |
| 51. `else if` syntax | Preserve existing `if` semantics while allowing `else if` as readable nested-if sugar for statements and expressions. Keep parser and diagnostics simple. | `src/parser.rs`, `tests/examples.rs`, specs/docs | Done | Low |
| 52. explicit `return expr` | Add an explicit early function exit statement that returns from the nearest named or anonymous function, is forbidden at top level, and type-checks against the function result type. | parser/AST/typed HIR/MIR/bytecode/typechecker/runtime/docs/tests | Done | Medium |
| 53. `break` / `continue` for loops | Add explicit loop control statements for `while` and future loop forms, reject them outside loops, and preserve nearest-loop behavior through nested functions and blocks. | parser/AST/typed HIR/MIR/bytecode/typechecker/runtime/docs/tests | Done | Medium |
| 54. `for item in list` | Add the first `for` form for `List[T]` only, with a fresh immutable loop binding, no iterator protocol, and lowering that preserves current explicit loop-control behavior. | parser/AST/typed HIR/MIR/bytecode/typechecker/runtime/docs/tests | Done | Medium |
| 55. payload discard `_` in enum patterns | Allow `_` only as a payload discard inside qualified enum variant patterns, while keeping broad catch-all `_ =>` matching deferred. | parser/AST/typed HIR/MIR/bytecode/typechecker/runtime/docs/tests | Done | Low |
| 56. minimal `muga build` command | Add the first package build command over the existing explicit artifact workflow, writing `.mgi` / `.mgc` / `.mgb` artifacts to a default project build directory without adding dependency manifests, lockfiles, or incremental reuse yet. | CLI/lib/artifact workflow/docs/tests | Done | Medium |
| 57. default build artifact consumption | Add an explicit `check` / `run` CLI convenience for consuming `.muga/build` artifacts, keeping default source-compatible behavior unchanged and keeping artifact-root configuration out of `muga.toml`. | CLI/lib/docs/tests | Done | Medium |
| 58. local path dependency metadata | Add the first manifest-level local dependency declarations so source imports can stay logical while package roots can live outside one source tree. Do not add lockfiles, registries, version solving, or automatic artifact reuse in this slice. | package manifest/loader/docs/tests | Done | Medium |
| 59. package-level artifact reuse | Start automatic reuse of package interface/implementation artifacts over the local dependency graph, keeping stale/missing artifacts as explicit rebuild decisions and avoiding lockfiles, registries, publishing, or artifact-root manifest configuration. | package/artifact driver/docs/tests | Done | High |
| 60. package-local rebuild input metadata | Add enough per-package source/dependency input metadata to make future rebuild skips and parallel package builds package-local rather than whole-graph text comparisons. Do not add lockfiles, registries, publishing, or artifact-root manifest configuration in this slice. | package/artifact driver/docs/tests | Done | High |
| 61. public interface hash stability | Keep public interface hashes stable for implementation-only changes so `.mgi` reuse can stay package-local and downstream rebuilds are not forced by private bodies. | interface artifact/docs/tests | Done | High |
| 62. parallel package builds | Build independent package artifacts concurrently over an acyclic dependency graph while preserving deterministic artifact paths, result ordering, and diagnostics. Do not add lockfiles, registries, publishing, or artifact-root manifest configuration in this slice. | package/artifact driver/tests/docs | Done | High |
| 63. local path lockfile metadata | Add the first reproducible dependency metadata so local and future non-local dependencies can be recorded by source descriptor and content hash shape. Local path entries use `source_hash` as rebuild/review metadata, not published package identity. Do not add registries, publishing, package archives, lockfile enforcement, or artifact-root manifest configuration in this slice. | manifest/resolver/docs/tests | Done | High |
| 64. lockfile validation and update policy | Use generated `muga.lock` metadata to detect malformed, duplicate, graph-inconsistent, unsupported, or stale local dependency entries deterministically while preserving source-compatible local path workflows. Well-formed stale metadata is updated; malformed or unsupported metadata fails with `PK026`. Do not add registries, publishing, package archives, non-local dependency forms, version solving, or artifact-root manifest configuration. | manifest/resolver/docs/tests | Done | High |
| 65. canonical package archive content hash | Define and implement the first local, deterministic published-package content input over `muga.toml` and sorted source files so future URL/Git/registry dependencies can share the same `sha256:<hex>` identity. Do not add registries, network fetching, signing, broad publishing workflows, or artifact-root manifest configuration. | package/archive/docs/tests | Done | High |
| 66. deterministic package archive emission | Emit or materialize a deterministic local package archive/manifest artifact from the canonical content input, preserving stable source ordering and the `sha256:<hex>` identity. Do not add registries, network fetching, version solving, signing, install workflows, broad publishing infrastructure, or artifact-root manifest configuration. | package/archive/CLI/docs/tests | Done | High |
| 67. package archive readback validation | Parse local `.mgp` archive bytes and validate their `sha256:<hex>` identity plus manifest/source layout so future URL/Git/registry fetch paths can share the same verifier. Do not add registries, network fetching, version solving, signing, install workflows, broad publishing infrastructure, or artifact-root manifest configuration. | package/archive/docs/tests | Done | High |
| 68. local package archive materialization | Use the validated `.mgp` bytes to materialize or inspect a local package source tree/cache entry as the first archive consumption step. Keep URL/Git fetching, registries, version solving, signing, publishing workflows, and artifact-root manifest configuration deferred. | package/archive/docs/tests | Done | High |
| 69. local archive dependency/cache consumption | Connect the validated local archive materialization output to a narrow dependency-consumption or cache path while keeping the archive verifier as the trust boundary. Do not add URL/Git fetching, registries, version solving, signing, publishing workflows, or artifact-root manifest configuration. | package/archive/manifest/docs/tests | Done | High |
| 70. local archive dependency hardening | Add focused diagnostics and lockfile/cache edge-case coverage for local archive dependencies before widening to URL/Git fetching, registries, version solving, signing, publishing workflows, or artifact-root manifest configuration. | package/archive/manifest/docs/tests | Done | Medium |
| 71. local archive workflow sample ergonomics | Add a clear local archive dependency sample or CLI/doc workflow that demonstrates emitting an `.mgp`, copying the hash into `muga.toml`, building, and reusing `.muga/packages` without adding registries, network fetching, signing, broad publishing workflows, or artifact-root manifest configuration. | docs/samples/CLI/tests | Done | Medium |
| 72. local project build reuse diagnostics | Make package artifact reuse and stale rebuild decisions easier to inspect by tightening diagnostics/tests around `.mgi` / `.mgb` / `.mgc` reuse, stale dependency artifacts, and lockfile/cache state. Do not add registries, network fetching, signing, broad publishing workflows, URL/Git dependency resolution, full incremental rebuild planning, or artifact-root manifest configuration. | package/artifact driver/docs/tests | Done | Medium |
| 73. contextual generic record literal hardening | Propagate known generic record field expected types into contextual values like empty lists, `Map.empty()`, and `Option::None`, so annotations that already fix record type arguments do not still produce ambiguity diagnostics. | typing/docs/tests | Done | Medium |
| 74. stale generic interface artifact diagnostics | Keep stale generic package interface validation useful when discovered through `.mgi` artifacts by adding artifact-root context and concrete regeneration-command suggestions. | interface/docs/tests | Done | Medium |
| 75. remaining stdlib error diagnostic hardening | Pick one remaining diagnostics gap around `std::io::IOError` or `std::io::PathPairError` usage and close it with targeted tests/docs before broad new language features. Full `std::io::IOError` and `std::io::PathPairError` source spellings now suggest `import std::io` plus `io::...`. | package/docs/tests | Done | Medium |
| 76. invalid try placement diagnostic cleanup | Keep invalid `try Result::Ok(...)` / `try Result::Err(...)` placements focused on the `T023` enclosing-return problem by suppressing redundant constructor expected-type noise. | typing/tests/docs | Done | Medium |
| 77. `to_string` ambiguity diagnostic guidance | Make ambiguous `to_string` receiver errors point users at the three supported receiver annotations: `Int`, `Bool`, or `String`. | typing/tests/docs | Done | Low |
| 78. `print` / `println` ambiguity diagnostic guidance | Make ambiguous `print` / `println` argument errors point users at the three supported argument annotations: `Int`, `Bool`, or `String`. | typing/tests/docs | Done | Low |
| 79. `len` / `is_empty` ambiguity diagnostic guidance | Make ambiguous direct `len` / `is_empty` argument errors point users at the supported collection or string annotations. | typing/tests/docs | Done | Low |
| 80. list-only ambiguity diagnostic guidance | Make ambiguous list indexing and `for` iterable errors point users at the supported `List[T]` annotation. | typing/tests/docs | Done | Low |
| 81. complete current E005 ambiguity guidance | Ensure every current `E005` ambiguity diagnostic includes targeted annotation guidance, including unresolved function signatures and remaining collection/string receiver cases. | typing/tests/docs | Done | Medium |
| 82. CLI/spec package workflow alignment | Align CLI usage and mini v1 spec with current `check` / `run --built` program-argument behavior, implemented local dependency forms, and minimal local lockfile support. | CLI/tests/docs | Done | Medium |
| 83. `--built` default artifact diagnostic guidance | Add direct `muga build <entry>` guidance to package artifact failures reached through `check --built` / `run --built`, with tests for missing interface artifacts, missing/stale check caches, and missing/stale implementation artifacts. | lib/CLI/tests/docs | Done | Medium |
| 84. artifact-backed run check-cache diagnostics | Cover missing/stale `.mgc` diagnostics reached through `run --artifact-root` and `run --built`, and make check-cache diagnostics point at `muga emit-check-cache` as the focused regeneration command. | cache/CLI/tests/docs | Done | Medium |
| 85. `.mgb` implementation artifact diagnostic context | Include the concrete `.mgb` file path and package context in implementation artifact diagnostics for stale interface hashes, dependency hash mismatches, parse/hash failures, and invalid bytecode structure, while preserving existing regeneration guidance. | implementation artifact/tests/docs | Done | Medium |
| 86. v1 release boundary hardening | Define the narrow v1 promise, feature freeze, sample policy, diagnostic policy, artifact workflow policy, and release gate. Move invalid future snippets out of `samples/`, fix stale package execution wording, and make CI/release workflows run CLI artifact smoke checks plus offline package verification. | docs/CI/samples | Done | Medium |
| 87. v1 RC readiness verification | Make every `docs/v1-release-checklist.md` completion item evidence-backed through tests, CI, docs, or scripts while preserving the v1 feature freeze. Add release-readiness tests and a local release-gate script. | docs/tests/scripts/CI | Done | Medium |
| 88. v1 work queue management | Maintain a long-running checkbox queue for v1 boundary work, hardening candidates, explicit scope decisions, and post-v1 backlog so implementation can continue without tying progress to release timing. | docs | Done | Low |
| 89. initial `muga test` workflow | Add compiler-recognized `@test` functions, script/package discovery, `Unit` / `Result[Unit, E]` validation, runtime execution, CLI summaries, focused tests, and docs. | parser/runtime/lib/CLI/tests/docs | Done | Medium |
| 90. scalar test assertions | Add the first `test::assert_true`, `test::assert_eq_int`, `test::assert_eq_bool`, and `test::assert_eq_string` helpers, with diagnostics and package coverage. | std package/runtime/tests/docs | Done | Medium |
| 91. deterministic `muga fmt` | Add deterministic formatting for v1 source files, including a CI-friendly `--check` mode and line-comment preservation. | parser/formatter/CLI/tests/docs | Done | High |
| 92. Option/Result helpers | Add a narrow value-transforming helper surface for `Option` and `Result`, with specs, docs, runtime coverage, and inference diagnostics. | std package/runtime/tests/docs | Done | Medium |
| 93. narrow List/Map helpers | Add a small collection-helper surface that avoids iterator protocols and structural equality while documenting allocation and inference behavior. | std package/runtime/tests/docs | Done | Medium |
| 94. equality policy | Document the scalar-only v1 equality policy and add regression coverage that structural values are rejected. | specs/docs/tests | Done | Medium |
| 95. entry-aware check JSON | Add entry path and `file://` URI metadata to `muga check --format json` while preserving the existing diagnostic object shape and human output. | CLI/docs/tests | Done | Low |
| 96. minimal `muga doc` | Add Markdown docs generated from `.mgi` public records, enums, functions, and item-level public source comments. | CLI/docs/tests | Done | Medium |
| 97. minimal `muga new` | Add app, lib, and test manifest project templates while refusing non-empty target directories. | CLI/tools/docs/tests | Done | Medium |
| 98. initial metadata JSON | Add `muga metadata --format json` for package/module/item/export metadata plus public interface docs and rendered types, preserving the current language surface. | CLI/tools/docs/tests | Done | Medium |
| 99. initial hover JSON | Add `muga hover --format json --line --column` for declaration hover data with public docs and signatures. | CLI/tools/docs/tests | Done | Medium |
| 100. initial completions JSON | Add `muga completions --format json` for visible package/interface completions with import aliases plus public docs and signatures. | CLI/tools/docs/tests | Done | Medium |
| 101. initial definition JSON | Add `muga definition --format json --line --column` for go-to-definition data over import aliases, local bindings, and package/interface item references. | CLI/tools/docs/tests | Done | Medium |
| 102. initial references JSON | Add `muga references --format json --line --column` for find references data over import aliases, local bindings, and package/interface item references in the entry module. | CLI/tools/docs/tests | Done | Medium |
| 103. initial workspace JSON | Add `muga workspace --format json` for loaded packages, module source files, default artifact root, and dependency edges reachable from the entrypoint. | CLI/tools/docs/tests | Done | Medium |
| 104. syntax JSON | Add `muga syntax --format json` for single-file lex/parse diagnostics and faster editor feedback. | CLI/tools/docs/tests | Done | Low |
| 105. entry diagnostic context | Add entry source context to CLI JSON compiler diagnostics so each diagnostic carries a directly usable source path and `file://` URI. | CLI/tools/docs/tests | Done | Low |
| 106. package/artifact-root diagnostic context | Add entry package and artifact-root context entries for artifact-backed check JSON diagnostics. | CLI/tools/docs/tests | Done | Low |
| 107. artifact-file diagnostic context | Add concrete artifact-file context entries for JSON diagnostics that already know a specific `.mgi`, `.mgc`, or `.mgb` path. | CLI/tools/docs/tests | Done | Medium |
| 108. build JSON output | Add JSON contracts for `muga build` artifact status lines so tooling can consume build output without parsing human text. | CLI/tools/docs/tests | Done | Medium |
| 109. artifact emission JSON output | Add JSON contracts for artifact emission commands so tooling can consume explicit artifact output without parsing human text. | CLI/tools/docs/tests | Done | Medium |
| 110. artifact diagnostic hash context | Add dependency hash, source hash, and regeneration-command context for artifact diagnostics where the compiler already computes that data. | CLI/diagnostics/docs/tests | Done | Medium |
| 111. test JSON output | Add JSON `muga test` output after assertion diagnostics and test stdout capture are stable. | CLI/test/docs/tests | Done | Medium |
| 112. run JSON output | Add JSON `run` output with explicit program stdout, stderr, and main-result separation. | CLI/runtime/docs/tests | Done | Medium |
| 113. runnable package examples | Add more runnable package examples that show local dependencies, artifact-backed execution, `Result` errors, text-file IO, and small reusable APIs. | samples/docs/tests | Done | Low |
| 114. diagnostic explain command | Add `muga explain <diagnostic-code>` using the documented diagnostic index so users and tools can resolve error-code guidance without scraping docs. | CLI/docs/tests | Done | Low |
| 115. JSON-backed editor workflow prototype | Broaden the JSON-backed LSP/editor prototype only around a concrete workflow that uses syntax/check/metadata/hover/completions/definition/references/workspace/run/test JSON without scraping human output. | CLI/tools/docs/tests | Done | Medium |
| 116. artifact/cache explanation design | Design broader artifact/cache explanation output, such as a future `muga why-rebuild`, before editor or agent tools depend on rebuild reasoning. | docs/CLI/tools/tests | Done | Medium |
| 117. initial why-rebuild JSON | Implement initial read-only `muga why-rebuild --format json` output over local package graphs and `.mgi` / `.mgc` / `.mgb` artifact state, keeping artifact/cache explanations non-mutating. | CLI/tools/tests/docs | Done | Medium |
| 118. why-rebuild stale dependency coverage | Broaden `muga why-rebuild` coverage for stale dependency-interface hashes in implementation and check-cache explanations. | CLI/tools/tests/docs | Done | Medium |
| 119. why-rebuild lockfile metadata coverage | Broaden `muga why-rebuild` coverage for local path/archive lock metadata without mutating `muga.lock`. | CLI/tools/tests/docs | Done | Medium |
| 120. why-rebuild archive-cache metadata coverage | Broaden `muga why-rebuild` coverage for local archive cache metadata before editor or agent tools depend on rebuild reasoning. | CLI/tools/tests/docs | Done | Medium |
| 121. why-rebuild human text output | Add compact human text output for `muga why-rebuild` while keeping machine consumers on `--format json`. | CLI/tools/tests/docs | Done | Medium |
| 122. runtime call-context diagnostics | Add `related` call-context notes for nested function call sites and entry/test execution while preserving existing text and JSON diagnostic contracts. | runtime/CLI/tests/docs | Done | Medium |
| 123. test assertion source diagnostics | Add source-spanned `R021` diagnostics for failed `std::test` scalar assertions while preserving `tests[].message`. | runtime/CLI/tests/docs | Done | Medium |
| 124. runtime/debug remaining follow-up | Close remaining runtime/debug reporting by treating `related` call-context notes plus `R021` assertion diagnostics as the v1 runtime stack context and existing `regenerationCommand` context as artifact next-action guidance. | runtime/CLI/tests/docs | Done | Medium |
| 125. lightweight benchmark health checks | Add release-neutral ignored benchmark health tests and a wrapper script for compiler stages, package artifact reuse, and representative String/List/Map runtime paths without public performance claims. | tests/scripts/docs | Done | Medium |
| 126. fuzzing and malformed-input plans | Add fuzzing and malformed-input test plans for parser, package archive, lockfile, interface, check-cache, and implementation artifacts. | docs/tests | Done | Medium |
| 127. installation and onboarding docs | Document installation and onboarding paths such as `cargo install`, version checks, quickstarts, and later binary-release expectations without treating them as release triggers. | docs/tests | Done | Low |
| 128. Muga by Example learning path | Draft example-driven learning material that progresses from bindings and records to `Result`, packages, tests, local dependencies, and artifact-backed builds. | docs/tests | Done | Medium |
| 129. registry security design | Preserve the `.mgp` hash foundation and design future registry security around signing, provenance, lockfile enforcement, cache validation, and malicious-package handling before remote fetching. | docs/tests | Done | Medium |
| 130. edition and feature fingerprint policy | Document an edition or semantic feature-set fingerprint policy before syntax or semantic changes need backward-compatible migration. | docs/tests | Done | Medium |
| 131. package/artifact diagnostic context audit | Audit package/artifact diagnostics for missing dependency hash, source hash, and regeneration-command JSON context; add focused tests for any gap. | diagnostics/tests/docs | Done | Medium |
| 132. artifact-backed dependency API coverage | Add more artifact-backed execution coverage for representative dependency APIs that combine stdlib packages, `try`, generic records/functions, enums, and transitive dependencies. | package/runtime/tests/docs | Done | Medium |
| 133. public interface hash stability audit | Audit `.mgi` public interface hash stability after implementation-only edits and source-span movement across records, enums, generic functions, stdlib-backed signatures, and transitive public types. | package/tests/docs | Done | Medium |
| 134. implementation artifact structural audit | Audit `.mgb` structural validation and bytecode merge behavior for control-flow-heavy dependency bodies, private package items, and independently generated artifacts. | package/runtime/tests/docs | Done | Medium |
| 135. build reuse and lockfile update audit | Audit `muga build` reuse output and lockfile update behavior for local path and local archive dependencies after dependency implementation-only edits, public signature edits, and malformed lockfiles. | package/build/tests/docs | Done | Medium |
| 136. remaining diagnostic actionability audit | Add focused diagnostics/tests for any remaining ambiguity or expected-type failure that still leaves users without a clear annotation/import/visibility/artifact-regeneration action. | diagnostics/tests/docs | Done | Medium |
| 137. public signature round-trip audit | Review package-mode public signatures and ensure every v1-supported public type shape round-trips through in-memory and persisted interfaces. | package/interface/tests/docs | Done | Medium |
| 138. stdlib docs and samples review | Review stdlib package docs and samples for `std::io`, `std::fs`, `std::path`, `std::env`, and `std::time`, including artifact-backed execution samples where useful. | docs/samples/tests | Done | Medium |
| 139. release gate alignment audit | Keep the release gate and GitHub Actions aligned whenever local gate changes, including `scripts/v1-release-gate.sh` and CI workflow commands. | scripts/CI/docs/tests | Done | Medium |
| 140. shell completions and doctor tool audit | Add minimal command-line shell completions and a `muga doctor` environment check if they remain tool-only. | CLI/docs/tests | Done | Medium |
| 141. first std::json slice design | Decide the first `std::json` slice only after Result/helper ergonomics, scalar/collection mapping, schema evolution, and diagnostics are documented. | docs/spec/tests | Done | Medium |
| 142. first std::json implementation | Implement only the documented first `std::json` package contract, without schema generation, HTTP/RPC, `Float`, `Decimal`, `Bytes`, streaming APIs, or resource handles. | std package/runtime/tests/docs | Done | Medium |
| 143. first std::json implementation audit | Audit the implemented first `std::json` slice against docs, samples, artifact-backed behavior, and release-readiness evidence before broadening any standard-library surface. | docs/tests/samples | Done | Low |
| 144. post-json stdlib boundary selection | Choose the next narrow stdlib/API boundary only after documenting a contract and checking deferred surfaces. | docs | Done | Low |
| 145. opaque resource handle boundary design | Design the opaque resource-handle contract before stdout/stderr handles, file handles, process APIs, HTTP/SSE/WebSocket/RPC, streaming APIs, `Bytes`, buffers, or schema/client generation. | docs/spec/tests | Done | Medium |
| 146. first opaque type interface slice plan | Plan the smallest parser/resolver/typechecker/interface slice for `pub opaque type` names without adding runtime-backed handle values or effectful APIs. | docs/spec/tests | Done | Medium |
| 147. first opaque type interface slice implementation | Implement parser/AST, package identity, nominal public type checking, `.mgi` persistence, editor/doc tooling, downstream loaded-interface checking, and rejecting coverage for `pub opaque type` names without runtime-backed handle values or effectful APIs. | parser/package/interface/CLI/tests/docs | Done | Medium |
| 148. opaque handle capability and close metadata plan | Design the next metadata boundary for runtime-backed opaque handles: capability facts, consuming parameters, explicit close behavior, `.mgi` representation, diagnostics, and the first candidate stdlib handle API. | docs/spec/tests | Done | Medium |
| 149. opaque handle metadata interface implementation | Implement opaque handle capability facts and consuming parameter modes in package interfaces, docs/editor tooling, and public hashes without source syntax, runtime-backed handle values, or new stdlib APIs. | interface/package/CLI/tests/docs | Done | Medium |
| 150. consuming parameter dataflow checker | Reject obvious use-after-consume for bindings passed to loaded-interface `consume` parameters, using synthetic/compiler-provided fixtures before source syntax or runtime handle APIs. | typing/package/tests/docs | Done | Medium |
| 151. first runtime file handle implementation design | Design the first runtime-backed `std::fs::File` handle slice after metadata and consuming diagnostics: runtime slot representation, explicit close/stale diagnostics, artifact-backed behavior, and the deliberately narrow text-handle API. | docs/runtime/std/tests | Done | Medium |
| 152. first read-only runtime file handle implementation | Implement compiler-provided `std::fs::File` with read-only `open_text`, `read_text_from`, and consuming `close`, using VM-local handle slots and artifact-backed execution coverage without write modes or broad resource APIs. | runtime/std_package/typing/tests/docs | Done | Large |
| 153. post-file-handle resource surface selection | Audit the read-only `std::fs::File` implementation evidence and decide the next resource-handle boundary before adding write modes, `Bytes`, buffering, stdout/stderr handles, async IO, streams, process APIs, or network APIs. | docs/runtime/std/tests | Done | Medium |
| 154. program stderr output channel | Implement scalar `eprint` / `eprintln` as prelude builtins, add a separate runtime stderr buffer, populate run/test JSON stderr fields, and keep stdout/stderr handles deferred. | runtime/prelude/typing/CLI/tests/docs | Done | Medium |
| 155. text output file handle design | Design write-mode text file handles after stderr: open/create/append mode names, `write_text_to`, `flush`, close/flush failure behavior, wrong-mode errors, and artifact-backed coverage. | docs/std/runtime/tests | Done | Medium |
| 156. text output file handle implementation | Implement `create_text`, `append_text`, `write_text_to`, and `flush` for `std::fs::File` using runtime access modes, recoverable wrong-mode `io::IOError` values, close-time flush behavior, and artifact-backed coverage. | std_package/prelude/typing/runtime/tests/docs | Done | Large |
| 157. integrated practical report workflow | Refresh a focused runnable report-app/sample workflow to combine args/env, stdout/stderr, text-file handle writes, JSON, `Result`, tests, local dependencies, and `run --built` coverage without broadening the language surface. | samples/tests/docs | Done | Medium |
| 158. post-report adoption gap selection | Audit the practical workflow evidence and choose the next adoption slice before adding broader language/runtime features. | docs/tests | Done | Medium |
| 159. lexical resource cleanup design | Design the first `using`/lexical-cleanup contract for runtime-backed resources: syntax, body/cleanup error behavior, explicit close interaction, use-after-cleanup diagnostics, formatter rules, and artifact-backed coverage. | docs/parser/typing/runtime/tests | Done | Large |
| 160. lexical resource cleanup implementation | Implement the first statement-form `using` cleanup path for runtime-backed opaque handles, including parser/formatter/typechecker/lowering/runtime behavior, explicit-close diagnostics, source/artifact/`run --built` tests, and `report_app` refresh. | parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done | Large |
| 161. nested cleanup unwind hardening | Ensure nested `using` attempts active outer cleanups when an inner acquisition/body transfer or cleanup failure exits the scope, while preserving the first cleanup error under the current one-error `Result[T, E]` model. | bytecode/tests/docs | Done | Medium |
| 162. post-using adoption gap selection | Audit the `using`-backed report workflow and choose the next narrow practical API boundary before adding `Bytes`, formatting templates, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 163. first std::cli helper slice | Add a pure `std::cli` package over explicit `List[String]` values for positional defaults, long flags, and long options, with source/artifact coverage and `report_app` refresh. | std package/tests/docs/samples | Done | Medium |
| 164. post-std-cli adoption gap selection | Audit the `std::cli`-refreshed report workflow and choose the next practical API boundary before adding `Bytes`, formatting templates, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 165. CLI-first app template refresh | Refresh `muga new --template app` so the generated project imports `std::env` / `std::cli`, handles a positional or `--name` argument, prints a greeting, returns it from `main`, and runs from source plus built artifacts. | project template/tests/docs | Done | Medium |
| 166. post-template adoption gap selection | Audit the CLI-first generated app and choose the next practical API boundary before adding richer CLI parsers, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 167. typed scalar std::cli parsing helpers | Add pure `std::cli` helpers that parse positional and long-option values as `Int` or `Bool`, returning `Result[Option[T], String]` or `Result[T, String]` defaults while preserving existing `--` and `--name=value` behavior. | std package/tests/docs/samples | Done | Medium |
| 168. post-typed-cli adoption gap selection | Audit typed CLI parsing helpers and choose the next practical API boundary before adding full CLI parser schemas, config-file loading, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 169. JSON value accessor helpers | Add pure `std::json` helpers for typed `Value` extraction and common object-field access/defaults, returning `json::Error` for wrong shapes while keeping config-file loading, schema decoding, TOML, and host effects deferred. | std package/tests/docs/samples | Done | Medium |
| 170. post-json-accessor adoption gap selection | Audit JSON accessors and choose the next practical API boundary before adding config-file loading, schema decoding, full CLI parser schemas, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 171. JSON config workflow sample | Add a manifest project sample that reads JSON config with existing `std::fs` / `std::path` / `std::json`, applies typed `std::cli` overrides from `std::env::args`, and covers source, emitted-artifact, and `run --built --format=json` execution without adding `std::config` or schema decoding. | samples/tests/docs | Done | Medium |
| 172. post-config-workflow adoption gap selection | Audit the config workflow sample and choose the next practical adoption/API boundary before adding `std::config`, TOML, schema decoding, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 173. config workflow result mapping refresh | Refresh the config workflow sample to use existing `std::result::map_err` for IO/JSON error normalization at the app boundary, removing local one-off wrappers while preserving source, artifact-backed, shape-error, and built-run coverage. | samples/tests/docs | Done | Small |
| 174. post-result-mapping adoption gap selection | Audit the result-mapped config workflow and choose the next practical adoption/API boundary before adding `std::config`, TOML, schema decoding, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 175. first std::string text assembly helpers | Add pure `std::string::concat_all(parts: List[String])` and `std::string::join(parts: List[String], separator: String)` helpers with sample, source/artifact coverage, docs/spec updates, and a `config_app` refresh while preserving explicit `to_string()` conversions. | std package/tests/docs/samples | Done | Medium |
| 176. post-string-assembly adoption gap selection | Audit `std::string` text assembly helpers and the refreshed config workflow before choosing the next practical adoption/API boundary ahead of formatting templates, interpolation, `std::fmt`, builders, `std::config`, schema decoding, full CLI parser schemas, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 177. JSON required object-field helpers | Add pure `std::json::object_string_required`, `object_int_required`, and `object_bool_required` helpers that return `json::Error` for missing required object fields while preserving existing wrong-shape behavior, sample/docs updates, and source plus artifact-backed coverage. | std package/tests/docs/samples | Done | Medium |
| 178. post-required-json-field adoption gap selection | Audit the required JSON object-field helpers and choose the next practical adoption/API boundary before adding `std::config`, TOML, schema decoding, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 179. JSON composite object-field helpers | Add pure `std::json::object_array`, `object_array_or`, `object_array_required`, `object_object`, `object_object_or`, and `object_object_required` helpers with source/artifact coverage, sample/docs/spec updates, and no JSON path or schema decoding contract. | std package/tests/docs/samples | Done | Medium |
| 180. post-composite-json-field adoption gap selection | Audit scalar plus composite JSON field helpers and choose the next practical adoption/API boundary before adding JSON paths, schema decoding, `std::config`, TOML, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 181. nested JSON config workflow refresh | Refresh `samples/projects/config_app` with nested JSON config fields, composite `std::json` object-field helper extraction, source/artifact/shape-error/`run --built --format=json` coverage, and docs/sample-review updates without adding `std::config` or schema decoding. | samples/tests/docs | Done | Medium |
| 182. post-nested-json-config adoption gap selection | Audit the nested JSON config workflow and choose the next practical adoption/API boundary before adding JSON paths, schema decoding, `std::config`, TOML, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 183. JSON scalar array projection helpers | Add pure `std::json::array_strings`, `array_ints`, and `array_bools` helpers that convert `List[json::Value]` to typed scalar lists with index-specific `json::Error`s, source/artifact coverage, docs/spec updates, and a `config_app` refresh. | std package/tests/docs/samples | Done | Medium |
| 184. post-json-array-projection adoption gap selection | Audit scalar array projection helpers and choose the next practical adoption/API boundary before adding direct object-field scalar-array helper matrices, JSON paths, schema decoding, `std::config`, TOML, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 185. direct JSON scalar-array object-field helpers | Add pure `std::json::object_string_array*`, `object_int_array*`, and `object_bool_array*` helpers with field-aware index errors, source/artifact coverage, docs/spec updates, and a `config_app` refresh. | std package/tests/docs/samples | Done | Medium |
| 186. post-direct-json-array-field adoption gap selection | Audit the direct scalar-array object-field helpers and choose the next practical adoption/API boundary before adding JSON paths, schema decoding, `std::config`, TOML, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 187. repeated CLI option value helpers | Add pure `std::cli::option_values` and `option_values_or` helpers for repeated long-option string values, refresh `config_app` with repeated `--tag` overrides, and cover source/artifact/docs/spec behavior without adding a full CLI parser schema. | std package/tests/docs/samples | Done | Medium |
| 188. post-repeated-cli-option adoption gap selection | Audit repeated CLI option values and choose the next practical adoption/API boundary before adding JSON paths, schema decoding, `std::config`, TOML, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 189. JSON path helpers | Add pure `std::json::PathSegment`, `json::at`, and `json::at_required` helpers for nested object/array traversal with path-aware missing and shape diagnostics, source/artifact/sample/docs/spec coverage, and no schema decoding or JSONPath parser. | std package/tests/docs/samples | Done | Medium |
| 190. post-json-path adoption gap selection | Audit JSON path helper adoption and choose the next practical adoption/API boundary before adding typed path projection helpers, schema decoding, `std::config`, TOML, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 191. typed JSON path scalar projection helpers | Add pure `std::json::at_string*`, `at_int*`, and `at_bool*` helpers with path-aware terminal scalar errors, source/artifact/sample/docs/spec coverage, and no typed array/object path matrix, schema decoding, `std::config`, TOML, or JSONPath parser. | std package/tests/docs/samples | Done | Medium |
| 192. post-typed-json-path-scalar adoption gap selection | Audit typed JSON path scalar helpers and choose the next practical adoption/API boundary before adding typed array/object path helpers, schema decoding, `std::config`, TOML, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 193. typed JSON path collection projection helpers | Add pure `std::json::at_array*`, `at_object*`, `at_string_array*`, `at_int_array*`, and `at_bool_array*` helpers with path-aware terminal collection and scalar-array item errors, source/artifact/sample/docs/spec coverage, and no schema decoding, `std::config`, TOML, or JSONPath parser. | std package/tests/docs/samples | Done | Medium |
| 194. post-typed-json-path-collection adoption gap selection | Audit typed JSON path collection helpers and choose the next practical adoption/API boundary before adding schema decoding, `std::config`, TOML, full CLI parser schemas, generated config app templates, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 195. JSON schema decoding design | Design the first JSON-to-record decoding boundary after typed path helpers, including API shape, `.mgi` record/enum schema source, defaults, unknown-field policy, nested path diagnostics, supported first target types, and explicit deferrals before implementing schema decoding or `std::config`. | docs/tests | Done | Large |
| 196. default-overlay JSON schema decoder implementation | Implement compiler-owned `json::decode_or[T](value, fallback)` for concrete non-generic record overlays over `String`, `Int`, `Bool`, scalar lists, `Map[String, json::Value]`, and nested supported records, with schema payloads that survive artifact-backed execution and a `config_app` refresh. | typing/MIR/bytecode/runtime/std_package/tests/docs/samples | Done | Large |
| 197. post-json-schema-decoder adoption gap selection | Audit `json::decode_or[T]` adoption and choose the next practical config/API boundary before required `json::decode[T]`, `std::config`, TOML, full CLI parser schemas, generated config app templates, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 198. `std::config` JSON default loading design | Design the first `std::config` boundary around `config::load_json_or[T](path, fallback)`, public config error types, compiler-lowered decoder schemas, artifact-backed execution, diagnostics, sample refresh, and explicit deferrals for TOML, config discovery, generated templates, full CLI parser schemas, and broader host effects. | docs/tests | Done | Medium |
| 199. `std::config` JSON default loader implementation | Implement compiler-owned `config::load_json_or[T](path, fallback)` with public `std::config` types, schema payload lowering, runtime read/parse/decode/error mapping, implementation artifact persistence, `config_app` refresh, and source/artifact/shape-error/`run --built --format=json` coverage. | std package/typing/MIR/bytecode/runtime/artifacts/tests/docs/samples | Done | Large |
| 200. post-std-config-json-loader adoption gap selection | Audit the implemented `std::config::load_json_or[T]` slice and choose the next practical config/API boundary before TOML, required `json::decode[T]`, generated config app templates, full CLI parser schemas, formatting templates, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 201. generated config app template | Add `muga new --template config-app` with `src/main/main.muga` plus `config/settings.json`, using `std::config::load_json_or[T]`, typed/repeated CLI overrides, public config error mapping, and source/artifact/`run --built` coverage. | project template/CLI/tests/docs | Done | Medium |
| 202. post-generated-config-app-template adoption gap selection | Audit the generated config app template and choose the next practical config/API boundary before TOML, required `json::decode[T]`, full CLI parser schemas, formatting templates, broader decoder targets, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 203. required `json::decode[T]` design | Design strict `json::decode[T](value)` with expected Result target type policy, missing-field diagnostics, no-fallback record decoding, schema payload/artifact behavior, source/artifact coverage, and explicit deferrals for TOML, broader decoder targets, full CLI schemas, formatting templates, and host effects. | docs/tests | Done | Medium |
| 204. required `json::decode[T]` implementation | Implement compiler-owned strict `json::decode[T](value)` with virtual std signature, expected-target typing, required schema lowering through typed HIR/MIR/bytecode/artifacts, strict runtime record decoding, diagnostics, source/artifact/`run --built` coverage, and docs/spec updates. | std package/typing/HIR/MIR/bytecode/runtime/artifacts/tests/docs | Done | Large |
| 205. post-required-json-decoder adoption gap selection | Audit strict `json::decode[T](value)` adoption and choose the next practical JSON/config/API boundary before TOML, broader decoder target types, full CLI parser schemas, formatting templates, config discovery, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 206. broader JSON decoder target design | Design the next `json::decode_or[T]` / `json::decode[T]` / `config::load_json_or[T]` target expansion, including `Option[T]`, typed `Map[String, T]`, enum representations, null/missing behavior, default-overlay interactions, diagnostics, artifact schema payloads, and explicit deferrals for TOML, CLI parser schemas, formatting templates, config discovery, and host effects. | docs/tests | Done | Large |
| 207. structural JSON decoder target implementation | Implement the selected structural decoder target expansion: `Option[T]`, recursive `List[T]`, typed `Map[String, T]`, null/missing/default-overlay semantics, schema artifact payloads, runtime strict/default decoding, source/artifact/`run --built` coverage, and docs/spec updates while keeping enum decoding and TOML deferred. | json_decode/typing/runtime/artifacts/tests/docs | Done | Large |
| 208. post-structural-json-decoder adoption gap selection | Audit the implemented structural decoder target expansion and choose the next practical JSON/config/API boundary before enum decoding, TOML, full CLI parser schemas, formatting templates, config discovery, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 209. structural config workflow refresh | Refresh `samples/projects/config_app`, `muga new --template config-app`, generated `config/settings.json`, docs, and tests to use structural typed settings (`Option[T]`, nested records, `List[Record]`, typed `Map[String, T]`) instead of manual `json::Value` metadata access while preserving CLI > config > defaults behavior and artifact/`run --built` coverage. | project template/samples/tests/docs | Done | Medium |
| 210. post-structural-config-workflow adoption gap selection | Audit the refreshed structural config workflow and choose the next practical JSON/config/API boundary before enum decoding, TOML, full CLI parser schemas, formatting templates, config discovery, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 211. enum JSON/config decoder implementation | Implement concrete user enum decoding for `json::decode_or[T]`, strict `json::decode[T]`, and `config::load_json_or[T]`, including zero-payload string tags, one-payload single-key objects, path-aware diagnostics, schema/artifact payloads, source/artifact/`run --built` coverage, and docs while deferring generic enum decoding, tag attributes, TOML, full CLI schemas, and host effects. | json_decode/typing/runtime/artifacts/tests/docs | Done | Large |
| 212. post-enum JSON/config decoder adoption gap selection | Audit the implemented enum decoder and choose the next practical JSON/config/API boundary before field/variant schema polish, TOML, full CLI parser schemas, formatting templates, config discovery, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 213. JSON/config schema polish design | Design the field/variant wire-name and schema-policy slice for JSON/config decoding, including attribute syntax, rename/alias/strictness scope, diagnostics, artifact payloads, package-interface compatibility, and explicit deferrals for TOML, full CLI schemas, schema generation, generic decoding, and host effects. | docs/tests | Done | Large |
| 214. JSON/config field and variant rename implementation | Implement `@json(rename: "...")` on record fields and enum variants, including parser/AST metadata, duplicate effective wire-name diagnostics, decoder schema wire names, package-interface and `.mgb` persistence, runtime path-aware decoding, source/artifact/`run --built` coverage, and docs while deferring aliases, strict unknown fields, validation attributes, TOML, full CLI schemas, and host effects. | parser/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done | Large |
| 215. post-rename JSON/config adoption gap selection | Audit the implemented `@json(rename: "...")` schema-polish slice and choose the next practical JSON/config/API boundary before aliases, strict unknown fields, validation attributes, TOML, full CLI schemas, schema generation, generic decoding, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 216. JSON/config strict unknown-field policy design | Design record-level strict unknown-field policy for JSON/config decoding, including opt-in syntax, accepted wire-key set, nested record behavior, diagnostics, artifact payloads, package-interface compatibility, and explicit deferrals for aliases, validation attributes, TOML, full CLI schemas, schema generation, generic decoding, and host effects. | docs/tests | Done | Large |
| 217. JSON/config strict unknown-field policy implementation | Implement record-level `@json(deny_unknown_fields)` for JSON/config decoding, including parser/AST record attributes, formatter support, typing metadata, decoder schema strictness flags, package-interface and `.mgb` persistence, runtime path-aware unknown-key rejection, source/artifact/`run --built` coverage, docs, and release readiness. | parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done | Large |
| 218. post-strict JSON/config adoption gap selection | Audit the implemented `@json(deny_unknown_fields)` schema-polish slice and choose the next practical JSON/config/API boundary before aliases, validation attributes, TOML, full CLI schemas, schema generation, generic decoding, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 219. JSON/config alias metadata design | Design field and enum-variant alias metadata for JSON/config decoding, including syntax, multiple-alias representation, duplicate and conflict policy, strict accepted-key interaction, package-interface compatibility, artifact payloads, runtime diagnostics, source/artifact coverage, and explicit deferrals for validation attributes, TOML, full CLI schemas, schema generation, generic decoding, and host effects. | docs/tests | Done | Large |
| 220. JSON/config alias metadata implementation | Implement field and enum-variant `@json(alias: "...")` metadata, including parser validation, formatter support, accepted-name duplicate diagnostics, decoder schema alias lists, package-interface and `.mgb` persistence, runtime alias matching and ambiguity errors, source/artifact/`run --built` coverage, docs, and release readiness. | parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done | Large |
| 221. post-alias JSON/config adoption gap selection | Audit the implemented `@json(alias: "...")` compatibility slice and choose the next practical JSON/config/API boundary before validation attributes, TOML, full CLI parser schemas, schema/client generation, generic decoding, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 222. JSON/config validation attribute design | Design the smallest validation attribute surface for JSON/config decoding, including syntax namespace, first scalar validators, error accumulation policy, path-aware diagnostics, package-interface and `.mgb` persistence, source/artifact coverage, and explicit deferrals for TOML, full CLI schemas, schema/client generation, custom validators, cross-field validation, generic decoding, and host effects. | docs/tests | Done | Large |
| 223. JSON/config validation attribute implementation | Implement field-level `@validate(...)` metadata for record fields, including typed attribute argument parsing, scalar string/int validators, type compatibility diagnostics, formatter support, package-interface and `.mgb` persistence, runtime path-aware validation errors, source/artifact/`run --built` coverage, docs, and release readiness. | parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done | Large |
| 224. post-validation JSON/config adoption gap selection | Audit the implemented validation attribute slice and choose the next practical JSON/config/API boundary before TOML, full CLI parser schemas, schema/client generation, generic decoding, `Bytes`, process APIs, network APIs, streams, or broader host effects. | docs/tests | Done | Medium |
| 225. JSON/config schema export design | Design the first schema export boundary for concrete public JSON/config contracts, including dialect choice, CLI/library API shape, type/attribute mapping, alias and strictness representation, package-interface support, diagnostics, tests, and explicit deferrals for TOML, full CLI schemas, full client generation, generic decoding, JSON encoding, broader validators, and host effects. | docs/tests | Done | Large |
| 226. JSON/config schema export implementation | Implement `muga schema --format json` for concrete public record/enum contracts, including JSON Schema Draft 2020-12 rendering with `x-muga` extensions, required/overlay decode modes, package/type selection, source/interface package coverage, unsupported-target diagnostics, docs, and release readiness. | schema/CLI/package/interface/tests/docs | Done | Large |
| 227. post-schema-export JSON/config adoption gap selection | Audit the implemented schema export slice and choose the next practical JSON/config/API boundary before TOML, full CLI parser schemas, full client generation, generic decoding, JSON encoding, broader validators, or host effects. | docs/tests | Done | Medium |
| 228. typed JSON encoding design | Design compiler-owned typed JSON encoding for concrete public record/enum contracts, including API names, supported targets, canonical wire-name output, option/null policy, enum payload shape, validation-on-encode policy, source/interface artifact behavior, diagnostics, tests, and explicit deferrals. | docs/tests | Done | Large |
| 229. typed JSON encoding implementation | Implement `json::to_value[T](value)` and `json::encode_typed[T](value)` for concrete JSON/config data contracts, including std package signatures, type checking, schema payloads, MIR/bytecode/artifacts, runtime conversion, validation-on-encode, source/interface/`run --built` coverage, docs, and release readiness. | typing/std_package/mir/bytecode/artifacts/runtime/tests/docs | Done | Large |
| 230. post-typed JSON encoding adoption gap selection | Audit the implemented typed JSON encoding slice and choose the next practical data/API boundary before TOML, full CLI parser schemas, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 231. full CLI parser schema design | Design a typed CLI parser schema boundary for concrete app settings, including API shape, metadata, field/flag/option/list semantics, validation reuse, usage/help generation, diagnostics, artifact behavior, config-app interaction, and explicit deferrals for TOML, full client generation, generic encoding/decoding, broader validators, config discovery automation, and host effects. | docs/tests | Done | Large |
| 232. first CLI parser schema implementation | Implement compiler-owned `cli::parse_or[T](args, defaults)` and `cli::usage_for[T](program, defaults)` for concrete non-generic record overlays, including `std::cli::Error`, supported/preserved field handling, argument parsing, validation, typed HIR/MIR/bytecode/artifacts/runtime behavior, source/artifact/`run --built` tests, docs, and release readiness. | std_package/typing/mir/bytecode/artifacts/runtime/tests/docs | Done | Large |
| 233. post-CLI parser schema adoption gap selection | Audit the first CLI parser schema implementation and choose the next practical adoption/API boundary before TOML, full client generation, generic encoding/decoding, broader validators, config discovery automation, or host-effect APIs. | docs/tests | Done | Medium |
| 234. generated config-app CLI schema adoption | Refresh `samples/projects/config_app`, `muga new --template config-app`, generated settings/docs/tests, and release readiness to replace manual settings override code with `cli::parse_or[T]` while preserving explicit config-path lookup, source/artifact/`run --built` behavior, and simple app-boundary error strings. | project template/samples/tests/docs | Done | Medium |
| 235. post-config-app CLI schema adoption gap selection | Audit the generated config-app CLI schema adoption and choose the next practical language/API gap before TOML, `@cli(...)`, dedicated `CliSchema`, config discovery automation, strict no-default parsing, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 236. generated config-app usage adoption | Refresh `samples/projects/config_app`, `muga new --template config-app`, generated settings/docs/tests, and release readiness to expose `cli::usage_for[T]` through a `--help` path while keeping `--config` explicit and preserving source/artifact/`run --built` behavior. | project template/samples/tests/docs | Done | Medium |
| 237. post-config-app usage adoption gap selection | Audit generated config-app usage adoption and choose the next practical language/API gap before TOML, `@cli(...)`, dedicated `CliSchema`, config discovery automation, strict no-default parsing, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 238. first `@cli(...)` field metadata design | Design field-level CLI metadata for option names, aliases, help text, hidden fields, parser/usage behavior, duplicate-name validation, JSON/validation interaction, interface/artifact compatibility, and explicit deferrals for positionals, subcommands, short flags, env vars, config discovery, strict parsing, TOML, and client generation. | docs/tests | Done | Large |
| 239. first `@cli(...)` field metadata implementation | Implement field-level `@cli(name: "...", alias: "...", help: "...", hidden)` with dedicated `CliSchema`, parser/formatter/type-checking, package-interface and `.mgb` persistence, runtime parser/usage behavior, source/artifact/`run --built` coverage, docs, and release readiness. | parser/formatter/typing/interfaces/cli_schema/runtime/artifacts/tests/docs | Done | Large |
| 240. generated config-app CLI metadata adoption | Refresh `samples/projects/config_app`, `muga new --template config-app`, docs, and tests to use `@cli(name: "tag", alias: "tags")` and field help text in generated settings while preserving explicit `--config`, CLI > config > defaults precedence, source/artifact behavior, and simple app-boundary errors. | project template/samples/tests/docs | Done | Medium |
| 241. post-config-app CLI metadata adoption gap selection | Audit generated config-app CLI metadata adoption and choose the next practical language/API gap before TOML, config discovery automation, strict no-default parsing, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 242. strict CLI parser schema design | Design strict `cli::parse[T](args)` over the existing CLI schema foundation for command-line-only records with required fields, missing-field diagnostics, generated usage expectations, validation behavior, and artifact/schema compatibility before implementation. | docs/tests | Done | Large |
| 243. strict CLI parser schema implementation | Implement compiler-owned `cli::parse[T](args)` with expected-result type inference, `MissingArgument`, strict target validation, absent `Bool`/`Option`/`List` synthesis, validation, runtime parsing, `CliSchema` artifact reuse, source/artifact/`run --built` tests, docs, and release readiness. | std_package/typing/mir/bytecode/artifacts/runtime/tests/docs | Done | Large |
| 244. post-strict CLI parser adoption gap selection | Audit strict `cli::parse[T](args)` adoption and choose the next practical CLI/config/API gap before TOML, config discovery automation, no-default usage helpers, combined short flags, attached values, subcommands, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 245. strict CLI tool sample adoption | Add a checked-in CLI-only manifest project that uses `cli::parse[T](args)` for required options, enums, lists, options, validation, and recoverable app-boundary error messages, with source/artifact/`run --built` tests and docs before adding a `cli-tool` template or no-default usage helper. | samples/tests/docs | Done | Medium |
| 246. post-strict CLI tool sample adoption gap selection | Audit the strict CLI tool sample and choose generated `muga new --template cli-tool` adoption before no-default usage helpers, TOML, config discovery automation, combined short flags, attached values, subcommands, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 247. generated cli-tool template adoption | Add a `muga new --template cli-tool` starter that mirrors the strict CLI tool sample through generated project files, CLI template parsing, usage/completions, source/build/`run --built` tests, docs, and release readiness before adding no-default usage helpers or broader CLI semantics. | project template/tests/docs | Done | Medium |
| 248. post-generated cli-tool template adoption gap selection | Audit the generated cli-tool template and choose strict CLI manual help adoption before no-default usage helpers, source-level type arguments, command metadata, TOML, config discovery automation, combined short flags, attached values, subcommands, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 249. strict CLI manual help adoption | Add explicit `--help` branches and deterministic usage text to `samples/projects/cli_tool` and the generated `cli-tool` template, with source/build/`run --built` tests, docs, and release readiness before adding no-default usage generation or broader CLI semantics. | samples/project template/tests/docs | Done | Medium |
| 250. post-strict CLI manual help adoption gap selection | Audit the manual help adoption and choose the next practical CLI/config/API gap before no-default usage helpers, source-level type arguments, command metadata, combined short flags, attached values, subcommands, TOML, config discovery automation, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 251. strict CLI no-default usage helper design | Design a generated strict usage helper and type-anchor policy for no-default CLI records, including API name, inference/diagnostics, field/alias/help/validation rendering, artifact behavior, explicit deferrals, and release-readiness coverage before implementation. | docs/tests | Done | Large |
| 252. strict CLI no-default usage helper implementation | Implement `cli::usage_for_required[T](program)` with minimal explicit call type arguments, schema lowering/runtime/artifact support, source/build/`run --built` tests, strict sample/template adoption, docs, and release readiness before command metadata, combined short flags, attached values, subcommands, TOML, config discovery automation, full client generation, or host-effect APIs. | parser/typing/std_package/mir/bytecode/runtime/artifacts/tests/docs | Done | Large |
| 253. post-strict CLI no-default usage helper adoption gap selection | Audit generated strict usage adoption and choose record-level CLI command metadata before combined short flags, attached values, subcommands, TOML, config discovery automation, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 254. CLI command metadata design | Design record-level `@cli(about: "...")` command summaries in generated usage, including syntax, validation, rendering, interface/artifact persistence, sample/template adoption, and explicit deferrals before combined short flags, attached values, subcommands, TOML, config discovery automation, full client generation, or host-effect APIs. | docs/tests | Done | Medium |
| 255. CLI command metadata implementation | Implement record-level `@cli(about: "...")` across parser validation, typing, package signatures/interfaces, `CliSchema`, typed HIR/MIR/bytecode/artifacts, runtime usage rendering, strict sample/template adoption, tests, and docs before combined short flags, attached values, subcommands, TOML, config discovery automation, full client generation, or host-effect APIs. | parser/typing/interfaces/runtime/artifacts/samples/tests/docs | Done | Large |
| 256. post-CLI command metadata adoption gap selection | Audit generated command summary adoption and choose the next practical CLI/config/API gap before combined short flags, attached values, subcommands, TOML, config discovery automation, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 257. CLI short option metadata design | Design field-level `@cli(short: "x")` metadata for typed CLI schemas, including syntax, validation, duplicate/conflict rules, parser behavior, usage rendering, interface/artifact persistence, app-owned `-h` help checks, and explicit deferrals before implementation. | docs/tests | Done | Large |
| 258. CLI short option metadata implementation | Implement field-level `@cli(short: "...")` across parser validation, formatting, typing, package signatures/interfaces, `CliSchema`, artifacts, runtime parsing and usage rendering, starter/sample adoption, tests, and docs before combined short flags, attached values, built-in help branching, positionals, subcommands, TOML, config discovery automation, shell completion generation, full client generation, or host-effect APIs. | parser/formatter/typing/interfaces/runtime/artifacts/samples/tests/docs | Done | Large |
| 259. post-CLI short option metadata adoption gap selection | Audit generated short option adoption and choose typed CLI positional field metadata design before combined short flags, attached values, built-in help branching, subcommands, TOML, config discovery automation, shell completion generation, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 260. CLI positional field metadata design | Design typed positional field metadata for CLI schemas, including public syntax, ordering and duplicate rules, supported first field types, option/positional conflict policy, `--` behavior, generated usage layout, interface/artifact compatibility, diagnostics, tests, and explicit deferrals before implementation. | docs/tests | Done | Large |
| 261. CLI positional field metadata implementation | Implement field-level `@cli(positional: N)` across parser validation, formatting, typing, package signatures/interfaces, `CliSchema`, artifacts, runtime parsing and usage rendering, starter/sample adoption, tests, and docs before combined short flags, attached values, built-in help branching, subcommands, TOML, config discovery automation, shell completion generation, full client generation, or host-effect APIs. | parser/formatter/typing/interfaces/runtime/artifacts/samples/tests/docs | Done | Large |
| 262. post-CLI positional field metadata adoption gap selection | Audit positional metadata adoption in samples/templates/docs and choose the next CLI ergonomics slice before combined short flags, attached values, built-in help branching, subcommands, TOML, config discovery automation, shell completion generation, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 263. built-in CLI help policy design | Design the first built-in CLI help policy after positional metadata, including API shape, generated help rendering, parse/usage integration, `--` behavior, app-owned printing/status boundaries, artifacts, tests, and explicit deferrals before implementation. | docs/tests | Done | Medium |
| 264. built-in CLI help helpers implementation | Implement `cli::help_requested(args)`, `cli::help_for[T](program, defaults)`, and `cli::help_for_required[T](program)` with help-name conflict diagnostics, schema lowering, artifact-backed rendering, source/build/`run --built` coverage, and strict/config template adoption before parse-integrated help result enums, subcommands, shell completion generation, TOML/config discovery automation, full client generation, or host-effect APIs. | typing/std_package/mir/bytecode/runtime/artifacts/samples/tests/docs | Done | Large |
| 265. post-built-in CLI help helper adoption gap selection | Audit built-in help helper adoption in samples/templates/docs and choose the next CLI/config ergonomics slice before parse-integrated help result enums, combined short flags, attached values, subcommands, shell completion generation, TOML/config discovery automation, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 266. parse-integrated CLI help workflow design | Design a typed CLI request workflow that combines help detection and strict/overlay parsing without runtime-owned printing/exits, covering public API shape, `Result`/help composition, source type anchors, schema lowering, artifacts, templates, diagnostics, and explicit deferrals before implementation. | docs/tests | Done | Large |
| 267. parse-integrated CLI help workflow implementation | Implement `cli::Request[T]`, `cli::parse_request[T]`, and `cli::parse_request_or[T]` across std_package, typing schema lowering, typed HIR, MIR, bytecode, runtime, artifacts, source/build/`run --built` tests, docs, and generated strict/config template adoption before runtime-owned printing/exits, subcommands, shell completion generation, TOML/config discovery automation, full client generation, or host-effect APIs. | typing/std_package/mir/bytecode/runtime/artifacts/samples/tests/docs | Done | Large |
| 268. post-parse-integrated CLI help workflow adoption gap selection | Audit request-workflow adoption in samples/templates/docs and choose compact CLI short option syntax design before runtime-owned printing/exits, subcommands, shell completion generation, TOML/config discovery automation, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 269. compact CLI short option syntax design | Design parser behavior for combined bool short flags and attached short option values, including `-abc`, `-ovalue`, `-abovalue`, `-o=value`, `-abo=value`, `--` behavior, unknown short diagnostics, missing values, repeated list merging, and explicit deferrals before implementation. | docs/tests | Done | Medium |
| 270. compact CLI short option syntax implementation | Implement compact short token parsing in the runtime parser for `cli::parse`, `cli::parse_or`, `cli::parse_request`, and `cli::parse_request_or`, with source/artifact/request workflow tests, docs, and release readiness before subcommands, shell completion generation, TOML/config discovery automation, or runtime-owned printing/exits. | runtime/tests/docs | Done | Medium |
| 271. post-compact CLI short option syntax adoption gap selection | Audit compact short syntax adoption in samples/templates/docs and choose CLI subcommand metadata design before implementation, shell completion generation, TOML/config discovery automation, runtime-owned printing/exits, full client generation, generic encoding/decoding, broader validators, or host-effect APIs. | docs/tests | Done | Medium |
| 272. CLI subcommand metadata design | Design schema-backed command trees for multi-action CLI tools, including public syntax, enum-vs-record target shape, root/global vs local options, nested help/request behavior, usage rendering, artifact compatibility, compact short option interaction, and future completion generation needs before implementation. | docs/tests | Done | High |
| 273. CLI subcommand enum metadata plumbing | Implement enum declaration and variant `@cli(...)` metadata across parser validation, formatting, type-checking diagnostics, typed HIR, package signatures, `.mgi` v10 persistence, package interface round-trip tests, docs, and release readiness before strict command schema lowering or runtime dispatch. | parser/formatter/typing/typed_hir/package_signature/interfaces/tests/docs | Done | Medium |
| 274. CLI subcommand strict schema implementation | Implement strict command enum schemas across typed schema lowering, MIR, bytecode, `.mgb`, runtime dispatch/help rendering, source/artifact/`run --built` tests, and docs before wrapper-record root/global options, generated shell completions, TOML/config discovery automation, or runtime-owned printing/exits. | typing/mir/bytecode/runtime/artifacts/tests/docs | Done | Large |
| 275. CLI subcommand adoption audit | Audit strict command enum schema adoption and refresh `samples/projects/cli_tool` plus generated `muga new --template cli-tool` starters to expose `run` / `inspect` command trees before wrapper-record root/global options, generated shell completions, TOML/config discovery automation, or runtime-owned printing/exits. | docs/tests/samples/templates | Done | Medium |
| 276. CLI wrapper-record root/global options design | Design how root/global options compose with command enums before implementing `tool --global run ...`, including dispatch order, help layout, artifact schema shape, overlay/default interactions, and diagnostics. | docs/tests | Done | High |
| 277. CLI wrapper-record subcommand metadata plumbing | Implement parser, formatter, type-checker, typed HIR, package signature, and `.mgi` support for field-level `@cli(subcommand)` on wrapper records before schema lowering, runtime dispatch, artifacts, or sample/template adoption. | parser/formatter/typing/typed_hir/package_signature/interfaces/tests/docs | Done | Medium |
| 278. CLI wrapper-record schema and runtime support | Lower wrapper records into `CliSchema` with nested subcommand schemas, persist schema/artifact payloads, parse root/global options before command tokens, and render wrapper root help before strict sample/template adoption. | typing/cli_schema/mir/bytecode/runtime/artifacts/tests/docs | Done | Large |
| 279. CLI wrapper-record sample/template adoption | Refresh the strict CLI sample and generated `cli-tool` starter with a minimal root/global option using wrapper records, preserving command dispatch, help, compact shorts, artifact-backed execution, and recoverable `cli::Error` mapping. | samples/templates/tests/docs | Done | Medium |
| 280. CLI schema-backed shell completion design | Design generated shell completion output from wrapper, command, leaf option, alias, short-option, enum-value, and positional `CliSchema` data before implementing a user-facing completion generator. | docs/tests | Done | High |
| 281. CLI schema-backed shell completion implementation | Implement `muga cli-completions <bash|zsh|fish> --program <name> --type <Type> ...` using source and artifact `CliSchema` loading, render bash/zsh/fish scripts for wrapper root options, command aliases, leaf options, enum values, help flags, and positional fallback, and cover source, artifact-root, and `--built` generated `cli-tool` workflows. | main/typing/cli_schema/runtime/artifacts/tests/docs | Done | Large |
| 282. CLI schema-backed shell completion adoption audit | Audit generated `cli-tool` completion installation and distribution docs after the generator implementation, then choose install documentation, generated project packaging hooks, shell-agnostic JSON completion specs, richer nested traversal, TOML/config discovery, richer value sources, or installer integration. | docs/tests/samples/templates | Done | Medium |
| 283. CLI generated app shell completion onboarding | Add generated `cli-tool` README completion commands plus first-project onboarding docs for source and `--built` app completion generation while keeping shell installation user/package-manager controlled. | templates/docs/tests | Done | Medium |
| 284. CLI generated app completion packaging hook | Add a generated `scripts/generate-completions.sh` helper for `cli-tool` starters that writes bash, zsh, and fish scripts into `completions/` without running the app or mutating shell profiles. | templates/tests/docs | Done | Medium |
| 285. CLI completion JSON spec design | Define the shell-agnostic generated-app completion JSON contract over existing `CliSchema` facts, including recursive wrapper, command, record, option, positional, candidate, and target metadata before dynamic value sources or installer behavior. | docs/tests | Done | Medium |
| 286. CLI completion JSON spec implementation | Implement `muga cli-completions --format json --program <name> --type <Type> ...` for source, `--artifact-root`, and `--built` workflows, sharing schema loading with shell renderers and covering JSON output plus shell/json argument validation. | main/tests/docs | Done | Medium |
| 287. CLI completion nested command traversal | Extend generated app completion renderers to track recursive command-scope transitions for bash, zsh, and fish so nested command enum payloads can reach leaf options, value candidates, and help flags while preserving recursive JSON output. | main/tests/docs | Done | Medium |
| 288. CLI completion value-source metadata | Implement `@cli(value_source: "file"|"directory")` for String-like CLI values, carry it through source, package signatures, interfaces, `CliSchema`, MIR, artifacts, JSON completion specs, and bash/zsh/fish option-value completion before TOML/config discovery or installer integration. | parser/typing/interfaces/cli_schema/main/tests/docs | Done | Medium |
| 289. CLI completion installer integration | Implement non-mutating completion package emission as `muga emit-cli-completions --format json --output-dir <dir> --program <name> --type <Type> ...`, writing bash, zsh, fish, and `.completions.json` files with text or JSON metadata output before shell-profile installation, package-manager-specific installers, TOML/config discovery, or dynamic completion producers. | main/templates/tests/docs | Done | Medium |
| 290. Generated config-app path discovery | Add explicit config path discovery to `samples/projects/config_app` and generated `config-app` starters as `--config` > `MUGA_CONFIG_PATH` > generated JSON default, with help text, source/build tests, onboarding docs, and release-readiness evidence before TOML parsing or package resource lookup. | samples/templates/tests/docs | Done | Medium |
| 291. Workspace manifest metadata | Extend `muga workspace --format json` with manifest path, project root, source root, resource root, root package path, direct dependencies, and dependency source/resource roots so editors, CI, wrappers, and installers can derive config/resource paths before runtime package resource lookup. | package/main/tests/docs | Done | Medium |
| 292. Generated config-app run helper | Add a generated config-app `README.md` and `scripts/run-with-config.sh` helper that uses `MUGA_BIN`, `MUGA_CONFIG_PATH`, and the generated `config/settings.json` to run from any current directory without runtime-owned config discovery. | templates/tests/docs | Done | Medium |
| 293. Package resource archives | Add `[package] resources = "resources"` inclusion for package content hashes, deterministic `.mgp` resource entries, archive validation/materialization, local archive dependency cache validation, workspace `resourceRoot` metadata, and malformed-input coverage before runtime resource lookup or installed layouts. | package/main/tests/docs | Done | Medium |
| 294. Runtime package resource lookup | Add read-only `std::fs::read_resource_text(package_path, resource_path)` and `std::fs::read_resource_bytes(package_path, resource_path)` over manifest-declared resources for source trees, package tests, local archive dependency caches, and explicit built-artifact runs without returning host paths; expose only opaque `std::bytes::Bytes` size/empty inspection. | std_package/typing/runtime/lib/tests/docs | Done | Medium |
| 295. Installed app bundles | Add `emit-app-bundle --source-free`, `run-app-bundle`, `install-app --replace-owned`, `uninstall-app`, `emit-app-completions`, `emit-app-archive`, and `unpack-app-archive` for optional source-free bundles, source-free artifact execution/completions, dependency trees, user-chosen launcher/metadata placement, guarded owned updates/uninstalls, and deterministic `.mga` transport without shell-profile mutation. | lib/main/tests/docs | Done | Medium |
| 296. `.mgi` API diff gate | Add a persisted-interface API diff library, `muga api-diff` text/JSON CLI output, compatible/source-compatible/breaking/unknown classification, `--fail-on` thresholds, fixture coverage, and a release-gate smoke check before registry publishing. | api_diff/main/tests/docs/scripts | Done | Medium |
| 297. App archive hash validation | Make `unpack-app-archive` require the generated `*-sha256-<hash>.mga` filename or an explicit expected hash and validate archive bytes before writing files, documenting the local distribution integrity boundary without adding signing, registry, or package-manager policy. | lib/tests/docs | Done | Medium |
| 298. App archive verification CLI | Add `verify-app-archive [--format text|json] [--expected-hash sha256:<hex>] <archive-file>` so CI, package managers, and recipients can validate `.mga` hash-bearing filenames or explicit hashes, bytes, and entry headers without choosing an output directory or writing files. Reuse that explicit-hash path for renamed archive unpacking. | lib/main/tests/docs | Done | Medium |
| 299. App bundle install/archive preflight | Validate app-bundle metadata and `.muga/build` artifacts before `install-app` writes launcher metadata or `emit-app-archive` writes `.mga` bytes, keeping broken bundles out of PATH and transport handoff. | lib/tests/docs | Done | Medium |
| 300. Package archive verification/unpack CLI | Add `verify-package-archive [--format text|json] [--expected-hash sha256:<hex>] <archive-file>` and `unpack-package-archive [--format text|json] [--expected-hash sha256:<hex>] --output-dir <dir> <archive-file>` plus `.mgp`/`.mga` release-gate archive verification and unpack/run/install smoke so recipients, CI, and future package managers can validate hash-bearing filenames or explicit hashes, bytes, manifest/source/resource entries, JSON errors, and local source materialization without mutating caches. | package/lib/main/tests/docs/scripts | Done | Medium |
| 301. Read-only binary file reads | Add `std::fs::read_bytes`, `std::fs::read_bytes_path`, and `bytes::at` so CLI tools can inspect local binary files through opaque `Bytes` before binary writes, streams, codecs, mutable buffers, or broader cryptographic APIs. | std_package/typing/runtime/tests/docs | Done | Medium |
| 302. Bytes SHA-256 hash | Add `std::hash::sha256_hex(bytes)` over opaque `Bytes` so file/resource verification tools can compute lowercase SHA-256 hex without adding streaming hash state, HMAC, signatures, KDFs, or broader cryptographic APIs. | std_package/typing/runtime/tests/docs | Done | Medium |
| 303. Bytes/hash stdlib sample adoption | Add a runnable `std_hash` sample that reads local bytes, inspects a byte, hashes the payload, and runs from source plus emitted artifacts, updating sample docs and review evidence without adding codecs, buffers, or broader crypto. | samples/tests/docs | Done | Low |
| 304. Generated app package helper | Add `README.md` and `scripts/package-app.sh` to `muga new --template app` so first projects can emit source-free bundles, run them, archive `.mga`, and verify archives without adding shell-profile mutation or registry policy. | templates/tests/docs | Done | Low |
| 305. Filesystem rename helper | Add `std::fs::rename_path(from, to): Result[Unit, io::PathPairError]` as a one-step path rename/move helper with source/artifact tests, sample coverage, and docs without recursive copy/delete fallback, directory-copy semantics, cross-device fallback, or broader mutation policy. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 306. Filesystem file-size helper | Add `std::fs::file_size_path(path): Result[Int, io::IOError]` as a scalar byte-length metadata helper with source/artifact tests, sample coverage, and docs before public metadata records, timestamps, permissions, symlink policy, or recursive directory sizing. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 307. Path extension replacement helper | Add `std::path::with_extension(path, new_extension): Path` as a pure output/sidecar path helper with source/artifact tests, sample coverage, and docs before canonicalization, symlink policy, host path resolution, or broader path normalization. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 308. Path file-name replacement helper | Add `std::path::with_file_name(path, new_file_name): Path` as a pure sibling output path helper with source/artifact tests, sample coverage, and docs before strict path component validation, canonicalization, symlink policy, or host path resolution. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 309. Environment current directory helper | Add `std::env::current_dir(): Result[path::Path, io::IOError]` as an explicit process-current-directory read with source/artifact tests, sample coverage, and docs before temp-file allocation, canonicalization, project-root lookup, runtime-owned config discovery, or process execution. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 310. Filesystem canonicalize path helper | Add `std::fs::canonicalize_path(target_path): Result[path::Path, io::IOError]` as recoverable existing-path host resolution with source/artifact tests, sample coverage, and docs before pure normalization, project-root lookup, unique temp-file policy, or symlink-specific controls. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 311. Path prefix stripping helper | Add `std::path::strip_prefix(path, base): Option[path::Path]` as a pure component-aware relative path helper with source/artifact tests, sample coverage, and docs before lexical normalization, symlink policy, sandbox containment, or host path resolution. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 312. Environment temporary directory helper | Add `std::env::temp_dir(): Result[path::Path, io::IOError]` as an explicit host temporary-directory convention read with source/artifact tests, sample coverage, and docs before unique temp-file allocation, cleanup policy, sandbox containment, or process execution. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 313. Path lexical normalize helper | Add `std::path::normalize(path): path::Path` as a pure lexical cleanup helper with source/artifact tests, sample coverage, and docs before symlink policy, strict path validation, sandbox containment, or host path resolution. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 314. Filesystem modified Unix milliseconds helper | Add `std::fs::modified_unix_millis_path(target_path): Result[time::UnixMillis, io::IOError]` as a narrow last-modified timestamp helper with source/artifact tests, sample coverage, and docs before public metadata records, accessed/created timestamps, permissions, symlink policy, or filesystem watches. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 315. generated report-app template | Add generated `muga new --template report-app` as a single-project file-processing starter with data fixture, report writer, helper script, source/build/`run --built` coverage, and docs before broader metadata/recursive filesystem APIs. | templates/tests/docs | Done | Low |
| 316. generated lib/test README onboarding | Add generated `README.md` files to `muga new --template lib` and `muga new --template test` so every first-project template carries local check/test/doc/build commands without adding new CLI surface. | templates/tests/docs | Done | Low |
| 317. muga new template discovery | Add `muga new --list-templates [--format json]` so users and setup tools can discover starter names, aliases, and descriptions before choosing a generated project template. | main/templates/tests/docs | Done | Low |
| 318. top-level CLI help | Add successful `muga --help`, `muga -h`, and `muga help` usage output so first-run users can discover commands without relying on error output. | main/tests/docs | Done | Low |
| 319. command-specific CLI help | Add `muga help <command>` usage filtering for known commands while reusing the canonical usage contract. | main/tests/docs | Done | Low |
| 320. Filesystem file metadata record | Add public `std::fs::FileMetadata` plus `file_metadata_path(file_path)` as a regular-file record that bundles byte size and modified time while leaving all-path metadata policy deferred. | std_package/tests/docs/samples | Done | Low |
| 321. generated report-app FileMetadata adoption | Refresh `muga new --template report-app` to use `fs::file_metadata_path` for byte-size reporting instead of deriving size from the read string. | templates/tests/docs | Done | Low |
| 322. Binary file write helpers | Add full-file `std::fs::write_bytes` and `write_bytes_path` over opaque `Bytes` so resource/local binary workflows can materialize bytes without adding buffers, codecs, streams, or binary handles. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 323. Muga by Example binary write adoption | Surface the new `std_fs_write_bytes` sample and binary write design in the example-driven learning path so users can discover byte round trips without reading release-readiness tests. | docs/tests | Done | Low |
| 324. Resource bytes export sample adoption | Add `samples/projects/resource_export` as a manifest resource byte export workflow that reads declared binary resources, hashes them, writes them to an explicit temporary output, verifies the round trip, and cleans up without new API surface. | samples/tests/docs | Done | Low |
| 325. Resource export source-free gate | Add source-free app-bundle coverage for `samples/projects/resource_export` in examples, Muga by Example, and the canonical release gate so binary resources are verified through bundle-local artifacts before registry or installer policy. | tests/scripts/docs | Done | Low |
| 326. Filesystem path status record | Add `std::fs::PathStatus` and `path_status(path::Path)` as a plain record grouping over `exists_path`, `is_file_path`, and `is_dir_path`, with source/artifact tests, sample adoption, and docs before rich all-path metadata, symlink classification, permissions, or directory sizing. | std_package/tests/docs/samples | Done | Low |
| 327. Generated cli-tool package helper | Add `scripts/package-cli-tool.sh` to generated `cli-tool` starters so users can emit a source-free bundle, run it, emit app completions from bundle interfaces, archive `.mga`, and verify the archive without shell-profile mutation or registry policy. | templates/tests/docs | Done | Low |
| 328. Generated config-app package helper | Add `scripts/package-config-app.sh` to generated `config-app` starters so typed JSON config apps can emit a source-free bundle, run it with `MUGA_CONFIG_PATH`, emit app completions, archive `.mga`, and verify the archive before TOML or runtime-owned discovery. | templates/tests/docs | Done | Low |
| 329. Generated report-app package helper | Add `scripts/package-report-app.sh` to generated `report-app` starters so file-processing projects can emit a source-free bundle, run it against the generated data fixture, archive `.mga`, and verify the archive without installer policy. | templates/tests/docs | Done | Low |
| 330. Generated resource-export template | Add `muga new --template resource-export` so users can generate a manifest-resource binary export starter with source/built runs, source-free bundle packaging, `.mga` archive verification, and no new runtime API. | templates/tests/docs | Done | Low |
| 331. Generated package-app template | Add `muga new --template package-app` as an app plus local library starter with source/built runs, workspace JSON, source-free bundle packaging, `.mga` verification, and no workspace manifest or registry policy. | templates/tests/docs | Done | Low |
| 332. Filesystem path info record | Add public `std::fs::PathKind`, `std::fs::PathInfo`, `path_kind(path::Path)`, and `path_info(path::Path)` as a pure grouping over `PathStatus` before host-error-backed all-path metadata, symlink policy, permissions, or directory sizing. | std_package/tests/docs/samples | Done | Low |
| 333. Muga by Example PathInfo adoption | Add the `std_fs_metadata` sample to the example-driven learning path so users can discover typed `PathInfo` path classification without reading release-readiness tests. | docs/tests | Done | Low |
| 334. Filesystem path metadata record | Add public `std::fs::PathMetadata` and `path_metadata_path(path::Path)` as host-error-backed existing-path kind/status/modified metadata before size-bearing all-path metadata, permissions, symlink classification, or recursive directory sizing. | std_package/tests/docs/samples | Done | Low |
| 335. Resource export PathMetadata adoption | Use `fs::path_metadata_path` in the resource export sample and generated starter to verify the materialized payload as an existing file before broader all-path metadata, installer, or registry policy. | samples/templates/tests/docs | Done | Low |
| 336. Filesystem path size metadata record | Add public `std::fs::PathSizeMetadata` and `path_size_metadata_path(path::Path)` with optional regular-file size for existing paths before recursive directory sizing, permissions, symlink classification, or owner metadata. | std_package/tests/docs/samples | Done | Low |
| 337. Filesystem recursive directory listing helper | Add `std::fs::read_dir_recursive_path(root_path): Result[List[path::Path], io::IOError]` as a deterministic read-only descendant traversal helper before recursive directory size metadata, recursive removal, directory copy, globbing, symlink classification, or sandbox policy. | std_package/tests/docs/samples | Done | Low |
| 338. Filesystem directory size metadata record | Add public `std::fs::DirectorySizeMetadata` and `directory_size_metadata_path(root_path)` as a deterministic read-only recursive byte/count aggregate before destructive recursive operations, globbing, public symlink classification, or sandbox policy. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 339. Filesystem recursive directory removal helper | Add `std::fs::remove_dir_all_path(dir_path): Result[Unit, io::IOError]` as the first destructive recursive directory helper without trash/recycle-bin policy, globbing, or sandbox containment. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 340. Filesystem recursive directory copy helper | Add `std::fs::copy_dir_all_path(from, to): Result[Unit, io::PathPairError]` as a no-overwrite recursive directory copy helper before merge/overwrite policy, rollback, host-rename acceleration, globbing, or sandbox containment. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 341. Filesystem recursive directory move helper | Add `std::fs::move_dir_all_path(from, to): Result[Unit, io::PathPairError]` as a no-overwrite copy-then-remove recursive directory move helper before host-rename acceleration, rollback, merge/overwrite policy, globbing, or sandbox containment. | std_package/typing/runtime/tests/docs/samples | Done | Low |
| 342. Standard formatting helpers | Add pure `std::fmt::{repeat,pad_left,pad_right,truncate_chars,format_values}` over explicit `String` values before language interpolation, localization, terminal display width, or builders. | std_package/tests/docs/samples | Done | Low |
| 343. Installed app inventory | Add non-mutating `list-installed-apps [--format text|json] --output-dir <bin-dir>` over Muga install ownership metadata, reporting ready/drift states before shell-profile mutation, package-manager policy, or registry publishing. | lib/main/tests/docs/scripts | Done | Low |
| 344. Generated package helper install hooks | Add optional `MUGA_INSTALL_DIR` install/list handoff to generated package helper scripts after archive verification, reusing `install-app --replace-owned` and `list-installed-apps` without shell-profile mutation. | templates/tests/docs | Done | Low |
| 345. Archive emission JSON output | Add `--format json` for `emit-package-archive` and `emit-app-archive`, reporting archive path/hash/package or program/file metadata plus structured error output for CI and packagers without changing text output or installer policy. | main/tests/docs | Done | Low |
| 346. Archive unpack JSON output | Add `--format json` for `unpack-package-archive` and `unpack-app-archive`, reporting restored root/file/hash metadata plus structured `archive`/`outputDir` errors for CI and packagers without changing text output or installer policy. | main/tests/docs | Done | Low |
| 347. App bundle emission JSON output | Add `--format json` for `emit-app-bundle`, reporting bundle root/entry/launcher/source-mode/artifact/file metadata plus structured `entry`/`outputDir` errors for CI and packagers without changing text output or installer policy. | main/tests/docs | Done | Low |
| 348. App install/uninstall JSON output | Add `--format json` for `install-app` and `uninstall-app`, reporting launcher/metadata/program/file metadata plus structured bundle/output-dir errors for CI and packagers without changing text output or shell-profile policy. | main/tests/docs | Done | Low |
| 349. Completion package emission JSON output | Add `--format json` for `emit-cli-completions` and `emit-app-completions`, reporting entry/bundle/output-dir/program/target/file metadata plus structured errors for CI and packagers without changing text output or shell-profile policy. | main/tests/docs | Done | Low |
| 350. Required JSON config loader | Add compiler-owned `std::config::load_json[T](path): Result[T, config::Error]` with required decode semantics and artifact-backed `LoadJsonConfigRequired` before TOML, config discovery, process APIs, or broader config frameworks. | std package/typing/MIR/bytecode/runtime/artifacts/tests/docs | Done | Low |
| 351. std::config package sample adoption | Add a runnable `samples/packages/app/std_config` sample plus artifact-backed coverage that demonstrates strict `config::load_json` and default-overlay `config::load_json_or` side by side before broader config frameworks. | samples/tests/docs | Done | Low |
## Current Handoff
- Installed-app bundles can be source-backed or `--source-free`, dependency-aware, runnable without copied source files through `run-app-bundle`, installable into a user-chosen bin dir, inventory-readable through `list-installed-apps`, completion-capable from bundle interfaces, and archivable as `.mga`; bundle emission, completion emission, install/uninstall, and archive emission/unpack now have JSON metadata for package and app archives; `emit-app-bundle --source-free`, bundle README handoff, generated app/`cli-tool`/`config-app`/`report-app` package helpers with optional `MUGA_INSTALL_DIR` install/list, generated lib/test README onboarding, `muga new --list-templates`, top-level and command-specific `muga help`, install/archive preflight, `.mga` verify/unpack validation, `.mgp` package archive verification/unpack, package archives preserve binary resources, minimal runtime `Bytes` local/resource reads plus full-file writes, resource export sample/source-free gate adoption, generated `resource-export` and `package-app` templates, resource export `PathMetadata` verification, strict `std::config::load_json`, `std_config` package sample adoption, Muga by Example adoption, SHA-256 hex, bytes/hash sample adoption, one-step `rename_path`, scalar `file_size_path`/`modified_unix_millis_path`, regular-file `FileMetadata`, path-status/kind/info metadata, existing-path `PathMetadata`, optional-size `PathSizeMetadata`, read-only recursive `read_dir_recursive_path`, recursive `DirectorySizeMetadata`, recursive `remove_dir_all_path`, no-overwrite recursive `copy_dir_all_path`, copy-then-remove recursive `move_dir_all_path`, pure `std::fmt` formatting helpers, generated `report-app` adoption, pure `path::normalize`, pure `path::with_file_name`/`path::with_extension`/`strip_prefix`, explicit `env::current_dir`/`env::temp_dir`, existing-path `fs::canonicalize_path`, `.mgi` API diff CLI/gate coverage, and generated `muga new --template report-app` are done; next, keep language interpolation, TOML, config discovery, process/network APIs, binary streams/codecs/handles, broader cryptographic APIs, shell-profile installation, registry publishing, dynamic producers, and broader multi-project/workspace policy separate until needed.
## Resume Checklist
1. [ ] Read [docs/strategy-and-implementation-plan.md](strategy-and-implementation-plan.md) and [ROADMAP.md](../ROADMAP.md).
2. [ ] Read this file.
3. [ ] Read [docs/internal/identity-model.md](internal/identity-model.md) before changing resolver/typechecker/HIR/MIR/runtime identity flow.
4. [ ] Read [docs/practical-language-readiness.md](practical-language-readiness.md) before starting broad stdlib, resource, performance, concurrency, service, or API/schema work.
5. [ ] Read [spec/007-concurrency-draft.md](../spec/007-concurrency-draft.md) before changing task, channel, scheduler, cancellation, or async IO behavior.
6. [ ] Keep artifact roots explicit on the CLI; do not add `muga.toml` artifact-root config until lockfiles and package-aware project build state exist; do not reintroduce flattened AST/HIR or dependency source-body lookup as the long-term package boundary.
7. [ ] Run `cargo test` before editing if the previous state is unknown; after every compiler-core change, also verify `target/debug/muga check samples/println_sum.muga`, `target/debug/muga samples/println_sum.muga`, `target/debug/muga samples/packages/app/main/main.muga`, and `target/debug/muga samples/projects/my_service/src/main/main.muga`.
