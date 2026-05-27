# Modern Language Gap Decisions 2026-05-22

Status: classification pass over
[modern-language-gap-inventory-2026-05-22.md](modern-language-gap-inventory-2026-05-22.md).
This is not a language specification and does not decide release timing.

Purpose: turn the broad inventory into implementation memory. The goal is to
make future sessions pick the right class of work without re-litigating every
modern-language feature. Items here still need focused specs and tests before
implementation.

## Decision Principles

Use these principles when choosing from the inventory:

1. Preserve Muga's small readable source model before adding expressive syntax.
2. Treat `.mgi` as the public contract for packages, docs, API diffing, schema
   generation, and editor tooling.
3. Prefer explicit `Option`, `Result`, resource handles, and package metadata
   over hidden exceptions, ambient state, or runtime reflection.
4. Make compiler and tool output structured enough for humans, editors, CI, and
   coding agents.
5. Treat compile speed, cache behavior, diagnostics, and reproducibility as
   product features, not afterthoughts.
6. Add broad IO, process, network, registry, and service APIs only after the
   lower-level ownership, resource, cancellation, and trust models are explicit.
7. Keep release timing and version bumps as maintainer decisions; roadmap items
   are implementation guidance, not release prompts.

## V1 Validation And Support Work

These items fit Muga strongly and can be done before v1 without widening the
source language. They should validate the current surface, improve tool
contracts, or make existing package/artifact behavior easier to trust.

- [x] Add a conformance-suite layout tied to the mini spec and split specs,
  separate from runnable samples and release-readiness checks.
- [x] Define stable JSON diagnostics for CLI/library/LSP/agent use, including
  code, severity, primary span, related spans, suggestions, package/artifact
  context, and human message.
- [x] Add entry path and `file://` URI metadata to `muga check --format json`
  so editor, CI, LSP, and agent consumers can map command diagnostics without
  scraping CLI arguments.
- [x] Add entry source context to CLI JSON diagnostics so each diagnostic can
  carry a directly usable source path and `file://` URI.
- [x] Add entry package and artifact-root context to artifact-backed
  `check --format json` diagnostics when available.
- [x] Add concrete artifact-file context to JSON diagnostics that know a
  specific `.mgi`, `.mgc`, or `.mgb` path.
- [x] Define stable command-output contracts for `check`, `run`, `build`,
  artifact commands, and future JSON modes.
- [x] Design `.mgi` API compatibility diffing for public functions, records,
  enums, deprecations, and implementation-only changes.
- [x] Write standard-library review rules before broadening `std`: no hidden
  IO in property access, recoverable effects return `Result`, absence is
  `Option`, public error types are explicit, and runtime-backed values use
  opaque resources.
- [x] Add doc-comment and public API documentation rules for item-level
  `///` comments on public records, enums, opaque types, and functions.
- [x] Add initial package metadata for editor and agent tools through
  `muga metadata --format json`, exposing package/module/item/export metadata
  plus public interface docs and rendered types.
- [x] Add initial workspace metadata for editor and agent tools through
  `muga workspace --format json`, exposing loaded packages, module source
  files, default artifact root, and dependency edges reachable from an
  entrypoint.
- [x] Add initial faster parse feedback for editor and agent tools through
  `muga syntax --format json`, exposing single-file lex/parse diagnostics
  without resolver, typechecker, import loading, or artifact checks.
- [ ] Add broader workspace/project design only after the entry-reachable
  editor/LSP contract needs more than one entrypoint.
- [x] Add artifact/cache explanation design, such as a future `muga why-rebuild`
  or `muga why-artifact` command. The design lives in
  [artifact-cache-explanations.md](artifact-cache-explanations.md) and keeps
  the first surface non-mutating with `--format json` for tooling.
- [x] Implement initial read-only `muga why-rebuild --format json` output for
  `.mgi`, `.mgc`, and `.mgb` artifact states, with focused fresh, missing,
  stale, hashMismatch, invalid, `--built`, and explicit `--artifact-root`
  coverage.
- [x] Add lightweight benchmark health checks for compiler stages, package
  artifact reuse, and representative String/List/Map runtime paths, without
  public performance claims. See
  [benchmark-health-checks.md](benchmark-health-checks.md).
- [x] Add fuzzing and malformed-input test plans for parser, package archive,
  lockfile, interface, check-cache, and implementation artifacts. See
  [fuzzing-malformed-input-plan.md](fuzzing-malformed-input-plan.md).
- [x] Add runtime diagnostic call-context related notes for nested function
  calls and entry/test execution.
- [x] Add source-spanned diagnostics for failed `std::test` scalar assertions.
- [x] Close runtime/debug failure reporting for v1 by keeping stack context in
  `related` call-context notes, failed scalar assertions in source-spanned
  `R021` diagnostics, and artifact next-actions in `regenerationCommand`
  context instead of adding a separate stack-trace schema.
- [x] Document install, version-check, and quickstart paths while keeping
  release timing separate. See
  [installation-and-onboarding.md](installation-and-onboarding.md). This is a
  release-neutral onboarding surface, not a release trigger.
- [x] Draft the "Muga by Example" learning path from bindings through records,
  `Result`, packages, tests, local dependencies, and artifact-backed builds.
  See [muga-by-example.md](muga-by-example.md).
- [x] Preserve the `.mgp` hash foundation and design future registry security
  around signing, provenance, lockfile enforcement, cache validation, and
  malicious-package handling before remote fetching. See
  [registry-security-design.md](registry-security-design.md).
- [x] Record language edition or semantic feature-set fingerprint policy for
  package artifacts before incompatible syntax or semantic changes need it.
  See
  [edition-feature-fingerprint-policy.md](edition-feature-fingerprint-policy.md).

Recommended first slice from this group:

1. conformance-suite layout
2. JSON diagnostics schema and command-output contract
3. `.mgi` API-diff design
4. standard-library review rules

These four reduce future ambiguity the most while changing little or no user
source syntax.

## Optional Pre-V1 Usability Work

These items fit Muga, but they add user-visible tools or small language/stdlib
surface. If they move into v1 scope, update the v1 checklist, mini spec or
split specs, samples, diagnostics, and focused tests first.

- [x] `muga test` with static `@test` metadata and `Unit` /
  `Result[Unit, E]` test returns.
- [x] Scalar test assertions for `Int`, `Bool`, and `String`, including
  `test::assert_true`, `test::assert_eq_int`, `test::assert_eq_bool`, and
  `test::assert_eq_string`.
- [x] Deterministic `muga fmt` with `--check` for v1 source files, including
  line-comment preservation.
- [x] Top-level and command-specific `muga --help` / `muga help` usage output
  for first-run CLI discovery.
- [x] Minimal `muga doc` generated from `.mgi` public records, enums,
  functions, and item-level public source comments.
- [x] `muga new` template discovery plus templates for app, library package,
  package with tests, config-aware app, strict CLI tool, and file-processing
  report app.
- [x] Initial `muga metadata --format json` package facts for LSP/editor and
  agent consumers.
- [x] Initial `muga hover --format json` declaration hover data with public
  docs and signatures.
- [x] Initial `muga completions --format json` visible package/interface
  completions with import aliases plus public docs and signatures.
- [x] Initial `muga definition --format json` go-to-definition data for import
  aliases, local bindings, and package/interface item references.
- [x] Initial `muga references --format json` find references data for import
  aliases, local bindings, and package/interface item references in the entry
  module.
- [x] Initial `muga workspace --format json` workspace metadata for loaded
  packages, module source files, default artifact root, and dependency edges
  reachable from an entrypoint.
- [x] Initial `muga syntax --format json` single-file parse feedback for
  editor and LSP tooling.
- [x] Initial CLI JSON diagnostic entry source context for editor and LSP
  tooling.
- [x] Initial package/artifact-root JSON diagnostic context for artifact-backed
  check diagnostics.
- [x] Initial concrete artifact-file JSON diagnostic context for `.mgi`,
  `.mgc`, and `.mgb` diagnostics.
- [x] Initial `muga build --format json` artifact status output for `.mgi`,
  `.mgc`, and `.mgb` build products.
- [x] Initial `muga emit-artifacts --format json`,
  `muga emit-interface --format json`, and
  `muga emit-check-cache --format json` output for explicit artifact emission
  commands.
- [x] Initial dependency hash, source hash, artifact hash, and
  regeneration-command context for artifact diagnostics.
- [x] Initial `muga test --format json` output with structured test results,
  captured per-test stdout/stderr, summary counts, and pre-run compiler
  diagnostics.
- [x] Initial `muga run --format json` output with captured program
  stdout/stderr, returned `main` values, and compiler/runtime diagnostics.
- [x] Broaden the JSON-backed LSP/editor prototype only when there is a
  concrete workflow to validate after entry-aware JSON diagnostics, metadata
  facts, declaration hovers, visible completions, definition targets,
  entry-module reference results, entry-reachable workspace metadata,
  single-file parse feedback, run/test results, and entry
  source/package/artifact-root/artifact-file/hash/regeneration diagnostic
  context plus build and artifact-emission JSON output are available.
- [x] `muga explain <diagnostic-code>` after the diagnostic schema and error
  index are stable.
- [x] Option/Result helper packages with value-transforming `std::option` and
  `std::result` functions.
- [x] Narrow `std::list` / `std::map` helpers for allocation-explicit list
  transforms and map key/value extraction.
- [x] Scalar-only v1 equality policy documented for `Int`, `Bool`, and
  `String`, with structural equality deferred.
- [x] More runnable package examples showing `Result`, local dependencies,
  text-file IO, artifact-backed execution, and reusable APIs.
- [x] Representative artifact-backed dependency API coverage now combines
  stdlib packages, `try`, generic records/functions, enums, and transitive
  dependencies without source-body fallback.
- [x] `.mgi` public interface hash stability is covered after
  implementation-only edits and source-span movement across records, enums,
  generic functions, stdlib-backed signatures, and transitive public types.
- [x] `.mgb` structural validation and bytecode merge behavior is covered for
  control-flow-heavy dependency bodies, private package items, and independently
  generated artifacts.
- [x] `muga build` reuse output and lockfile update behavior is covered for
  local path and local archive dependencies after implementation-only edits,
  public signature edits, archive content updates, and malformed lockfiles.
- [x] recursive annotation diagnostics now suggest parameter/return annotations
  for direct recursion and explicit signatures for every function in mutually
  recursive groups.
- [x] Package-mode public signatures now have representative coverage for every
  v1-supported public type shape through in-memory and persisted interfaces,
  including same-package and imported public type identities.
- [x] Minimal command-line shell completions and a `muga doctor` environment
  check if they remain tool-only.
- [x] First `std::json` slice design after documenting `Result` ergonomics,
  scalar/collection mapping, schema evolution, and diagnostics. See
  [std-json-first-slice.md](std-json-first-slice.md).
- [x] First `std::json` package implementation from that contract, with
  schema generation, HTTP/RPC, `Float`, `Decimal`, `Bytes`, streaming APIs, and
  resource handles still deferred.
- [x] First `std::json` implementation audit against docs, samples,
  artifact-backed behavior, and release-readiness evidence. See
  [std-json-implementation-audit.md](std-json-implementation-audit.md).
- [x] Post-JSON stdlib/API boundary selection. See
  [post-json-stdlib-boundary-selection.md](post-json-stdlib-boundary-selection.md):
  design opaque resource handles before stdout/stderr handles, file handles,
  process APIs, HTTP/SSE/WebSocket/RPC, streaming APIs, `Bytes`, buffers, or
  schema/client generation.
- [x] Opaque resource-handle boundary design. See
  [opaque-resource-handles.md](opaque-resource-handles.md): `pub opaque type`
  names now have `.mgi` identity and tooling visibility, while capability
  defaults, consuming-operation metadata, explicit close semantics,
  task-boundary/cancellation rules, and runtime diagnostic rules remain required
  before runtime-backed handle values or broad effectful APIs.
- [x] First `pub opaque type` interface slice. Public opaque names can be parsed
  in package mode, typechecked in signatures, persisted in `.mgi`, surfaced to
  docs/editor JSON tooling, and consumed from loaded interfaces without adding
  runtime-backed handle values.
- [x] Opaque handle capability and close metadata plan. The next implementation
  boundary is metadata-only: persist `OpaqueHandleFacts`, consuming parameter
  modes, explicit close metadata, API-diff/hash rules, and use-after-consume
  diagnostics before adding any runtime-backed handle values.
- [x] Opaque handle metadata interface slice. `.mgi` v5 now persists
  `OpaqueHandleFacts`, close-function identity, and `paramMode`, includes them
  in public hashes, and exposes them through metadata, hover/completion
  metadata, and docs without source syntax or runtime-backed handle values.
- [x] Consuming-parameter checker. Loaded-interface parameters marked `consume`
  now reject direct same-scope use-after-consume with `T026`, using a synthetic
  opaque-handle fixture before source syntax or runtime-backed handle values.
- [x] First runtime file-handle design. The first runtime-backed handle slice is
  read-only `std::fs::File`: VM-local `{family, slot, generation}` handles,
  `open_text`, `read_text_from`, consuming `close`, hard stale/wrong-family
  diagnostics, and no write modes, `Bytes`, streams, or stdout/stderr handles.
- [x] First read-only runtime file-handle implementation. `std::fs::File` now
  has `open_text`, `read_text_from`, and consuming `close` backed by VM-local
  runtime slots, `.mgi` handle facts and consume-mode metadata, source and
  artifact-backed execution coverage, recoverable `io::IOError` host failures,
  and hard `R022` stale/closed-handle diagnostics.
- [x] Post-file-handle resource-surface selection. The audit and selection in
  [post-file-handle-resource-surface-selection.md](post-file-handle-resource-surface-selection.md)
  chose a program stderr channel through scalar `eprint` / `eprintln`, not
  stdout/stderr handles, write-mode file handles, `Bytes`, streams,
  process APIs, or network APIs.
- [x] Program stderr output channel. Scalar `eprint` / `eprintln` now capture
  program stderr separately from stdout, text-mode `run` writes it to process
  stderr on success, and `run` / `test` JSON expose the captured stderr.
- [x] Text output file handle implementation. The implementation from
  [text-output-file-handles.md](text-output-file-handles.md) keeps one public
  `std::fs::File` type with runtime read/write/append modes, adds
  `create_text`, `append_text`, `write_text_to`, and `flush`, keeps only
  `close` consuming, treats wrong-mode operations as recoverable `io::IOError`
  values, and covers source plus artifact-backed execution.

Recommended order:

1. The practical `report_app` workflow sample now covers args/env,
   stdout/stderr, text-file handle writes, JSON, `Result`, tests, local
   dependencies, and `run --built`; [lexical-resource-cleanup.md](lexical-resource-cleanup.md)
   records the implemented statement-form `using` cleanup path for
   runtime-backed opaque handles. Minimal pure `std::cli` helpers are now
   implemented over explicit `List[String]` values for positional and option
   lookup, with behavior covered by std package source, samples, `report_app`,
   and examples tests. The CLI-first generated app template uses `std::env` and `std::cli`
   and is covered by project-template source, generated-project tests, and
   onboarding examples before richer CLI parsers or broader host effects. The
   implemented typed scalar `std::cli` parsing helpers are covered by code,
   samples, and tests for `Int` and `Bool` before full CLI parser schemas,
   config-file loading, process
   APIs, network APIs, or broader host effects.
   the implemented JSON value accessor helpers in `std::json`, returning
   `json::Error` for wrong shapes, preserve the post-typed-cli path before
   config-file loading, schema decoding, full CLI parser schemas, process APIs,
   network APIs, or broader host effects. Do not broaden schema generation,
   HTTP/RPC,
   `Float`, `Decimal`, `Bytes`, streaming APIs, stdout/stderr handles, process
   APIs, or network APIs as incidental `std::json` or stdlib growth. The
   implemented `samples/projects/config_app` JSON config workflow sample
   carries the post-json-accessor path by composing existing `std::config`,
   `std::path`, `std::json`, `std::env`, `std::cli`, and
   `std::result::map_err` with CLI > config > defaults precedence before TOML,
   broader schema tooling, full CLI parser schemas, process APIs, network APIs,
   or broader host effects. The
   [post-config-workflow-adoption-gap-selection.md](post-config-workflow-adoption-gap-selection.md)
   selection chooses the implemented `config_app` refresh that uses existing
   `std::result::map_err` for app-boundary error normalization before new error
   unions, `std::config`, TOML, schema decoding, full CLI parser schemas,
   formatting templates, process APIs, network APIs, or broader host effects.
   The implemented narrow pure `std::string` text assembly helpers
   (`string::concat_all` / `string::join`) carry the post-result-mapping path
   with explicit `to_string` conversion before formatting templates,
   interpolation, `std::fmt`, builders, broader config/schema work, full CLI
   parser schemas, process APIs, network APIs, or broader host effects.
   The implemented narrow `std::json` required object-field helpers carry the
   post-string-assembly path before broader `std::config`, TOML, schema
   tooling, full CLI parser schemas, formatting templates, interpolation,
   `std::fmt`, builders, process APIs, network APIs, or broader host effects.
   The implemented narrow `std::json` array/object field helpers carry the
   post-required-json-field path before JSON paths, broader config/schema work,
   TOML, full CLI parser schemas, formatting templates, process APIs, network
   APIs, or broader host effects.
   The implemented nested JSON config workflow refresh for
   `samples/projects/config_app` carries the post-composite-json-field path by
   using composite/typed `std::json` helpers for `tags`, owner metadata,
   servers, and limits before JSON paths, broader schema decoding,
   `std::config` expansion, TOML, full CLI parser schemas, formatting
   templates, process APIs, network APIs, or broader host effects.
   The implemented pure `std::json` scalar array projection helpers carry the
   post-nested-json-config path before JSON paths, schema decoding, broader
   object-field matrices, `std::config` expansion, TOML, full CLI parser
   schemas, formatting templates, process APIs, network APIs, or broader host
   effects.
   The implemented direct `std::json` scalar-array object-field helpers carry
   the post-json-array-projection path before JSON paths, schema decoding,
   `std::config` expansion, TOML, full CLI parser schemas, formatting
   templates, process APIs, network APIs, or broader host effects.
   The post-direct-json-array-field path is now carried by the implemented
   repeated `std::cli` option value helpers before JSON paths, schema decoding,
   `std::config`, TOML, full CLI parser schemas, formatting templates, process
   APIs, network APIs, or broader host effects.
   The post-repeated-cli-option path is now carried by the implemented JSON
   path helpers before schema decoding, `std::config`, TOML, full CLI parser
   schemas, formatting templates, process APIs, network APIs, or broader host
   effects.
   The post-json-path path is now carried by the implemented typed JSON path scalar
   projection helpers before typed array/object path helpers, schema decoding,
   `std::config`, TOML, full CLI parser schemas, formatting templates, process
   APIs, network APIs, or broader host effects.
   The post-typed-json-path-scalar path is now carried by the implemented typed JSON path collection
   projection helpers before schema decoding, `std::config`, TOML, full CLI
   parser schemas, generated config app templates, formatting templates,
   process APIs, network APIs, or broader host effects.
   The JSON schema decoding design path is now carried by
   [json-schema-decoding.md](json-schema-decoding.md) before implementing
   required `json::decode`, broader `std::config`, TOML, full CLI parser
   schemas, generated config app templates, formatting templates, process APIs,
   network APIs, or broader host effects.
   The [json-schema-decoding.md](json-schema-decoding.md) design selects and implements a
   compiler-owned `json::decode_or[T](value, fallback)` default-overlay decoder
   before required `json::decode[T]`, `std::config`, TOML, generated config app
   templates, full CLI parser schemas, process APIs, network APIs, or broader
   host effects.
   The post-JSON-schema-decoder path is now carried by the minimal
   `std::config` JSON default loading design before TOML, required
   `json::decode[T]`, generated config app templates, full CLI parser schemas,
   process APIs, network APIs, or broader host effects.
   The [std-config-json-loading.md](std-config-json-loading.md) design and
   implementation fixes that public API, `config::Error` shape,
   compiler-lowered schema payloads, `LoadJsonConfig` /
   `LoadJsonConfigRequired` artifact persistence, runtime error mapping, and
   `config_app` coverage. The implemented helpers are
   `config::load_json_or[T](path, fallback)` and `config::load_json[T](path)`.
   The generated `muga new --template config-app` starter is implemented before
   TOML, required JSON decoding, full CLI parser schemas, formatting templates,
   broader decoder targets, process APIs, network APIs, or broader host effects.
   The generated-template adoption path is now carried by
   [json-required-decoding.md](json-required-decoding.md), which implements
   required `json::decode[T](value)` before TOML, broader decoder target types,
   full CLI parser schemas, formatting templates, config discovery, process
   APIs, network APIs, or broader host effects.
   [json-required-decoding.md](json-required-decoding.md) defines and
   implements that strict decoder with expected `Result[T, json::Error]`
   target typing, path-aware missing-field errors, ignored unknown fields,
   no-fallback schema lowering, and artifact-safe `DecodeJsonRequired`
   payloads.
   [json-decoder-target-expansion.md](json-decoder-target-expansion.md)
   implements the decoder target expansion for `Option[T]`, recursive
   `List[T]`, typed `Map[String, T]`, and concrete non-generic enums across
   `json::decode_or[T]`, `json::decode[T]`, `config::load_json_or[T]`, and
   `config::load_json[T]`, with generic decoding, field/variant schema polish,
   and TOML still deferred.
   The implemented `config_app` sample and generated `config-app` starter carry
   the structural config workflow with `Option[String]`, nested records,
   `List[Record]`, and typed `Map[String, Int]` settings before TOML, full CLI
   parser schemas, formatting templates, config discovery, process APIs,
   network APIs, or broader host effects.
   The decoder expansion implements enum JSON/config decoder support, using
   zero-payload string tags and one-payload single-key objects before generic
   enum decoding, field/variant schema polish, TOML, full CLI parser schemas,
   formatting templates, config discovery, process APIs, network APIs, or
   broader host effects.
   [json-config-schema-polish.md](json-config-schema-polish.md) implements
   `@json(rename: "...")` on record fields and enum variants before aliases,
   validation attributes, TOML, full CLI schemas, schema generation, generic
   decoding, or broader host effects.
   [json-config-strict-unknown-fields.md](json-config-strict-unknown-fields.md)
   implements record-level `@json(deny_unknown_fields)`, accepted wire-key
   semantics, path-aware unknown-key errors, `.mgi` record flags, and `RF`
   decoder artifact tokens before aliases, validation attributes, TOML, full
   CLI schemas, schema generation, generic decoding, or broader host effects.
   [json-config-alias-metadata.md](json-config-alias-metadata.md) implements
   repeated `@json(alias: "...")` arguments inside a single field/variant `@json(...)`
   attribute, accepted-name conflict checks, strict unknown-field integration,
   `.mgi`/`.mgb` metadata, and recoverable ambiguity errors.
   The
   release gate and GitHub Actions are now
   aligned through
   `scripts/v1-release-gate.sh`. The stdlib package docs and samples review now
   covers `std::io`, `std::fs`, `std::path`, `std::env`, `std::cli`,
   `std::time`, `std::string`, `std::fmt`, and the first `std::json` slice, including
   artifact-backed execution samples where
   useful. Keep the current scalar-only equality policy in force while this
   hardening material is maintained.

Do not start named arguments, `using` expressions/multiple bindings, range
syntax, interpolation, `T?`, or `?.` just because the tools above exist. Those
remain separate syntax decisions.

## Post-V1 Platform Work

These items are valuable for making Muga practical and popular, but they should
wait until the v1 package/artifact contract is stable or until a focused design
note moves one item forward.

- [ ] Opaque resource handles with copy/send/share/close capability facts.
- [x] First-slice lexical cleanup with statement-form `using` for
  runtime-backed opaque handles.
- [ ] `Bytes`, `Buffer`, `StringBuilder`, `Duration`, `Instant`, `Float/F64`,
  and eventually `Decimal`.
- [ ] JSON, TOML, CSV, URL/URI, hashing, random, logging, metrics, tracing, and
  configuration packages.
- [ ] Process APIs, HTTP client/server, TLS, SSE, WebSocket, RPC, schema
  generation, and client generation.
- [ ] Structured concurrency with task groups, `spawn`, `join`, cancellation,
  channels, timeouts, `select`, and async IO.
- [ ] Full package workspace mode, dev/test/bench dependencies, version solver,
  remote URL/Git/registry dependencies, source replacement, vendoring, package
  yanking, publishing, signing, provenance, vulnerability database, `muga
  audit`, and SBOM generation.
- [ ] Full incremental package rebuild planning and later fine-grained
  per-package incremental checking.
- [ ] Control-flow MIR, native backend, profiler support, debug symbols, source
  maps, cross-compilation, WASM/playground runtime, and self-contained
  application binaries.
- [ ] Debug adapter integration, cross-package rename, versioned docs, package
  website, package health scores, and registry search.
- [ ] Edition migration tooling such as `muga fix`, after edition policy exists.

These should be managed as platform phases, not mixed into the v1 feature
boundary by default.

## Syntax Candidates To Keep Out By Default

These can be revisited later, but they should not be implemented before v1
without an explicit design note and checklist/spec/test updates.

- named arguments
- default arguments
- record destructuring and tuple-like syntax
- match guards and nested record patterns
- range/slicing syntax
- string interpolation/templates
- Option shorthand `T?` and Option-only `?.`
- protocols/traits/typeclasses and generic bounds
- external function declarations
- REPL/scratch runner if it conflicts with package/artifact-first behavior

The likely order, if syntax resumes after v1, is:

1. static attributes beyond `@test`
2. named arguments
3. opaque resource type declarations and `using`
4. small pattern refinements
5. range/slicing syntax
6. interpolation/templates
7. Option-only shorthand/chaining
8. protocols/bounds only after stdlib duplication proves the need

## Deliberate Non-Goals For The Current Direction

These conflict with Muga's current design spine unless a later design note
overturns the decision with concrete evidence.

- universal null or implicitly nullable values
- implicit throwing exceptions as the recoverable-error model
- postfix `expr?` for `Result` propagation
- runtime reflection as a core abstraction mechanism
- macros or compile-time code rewriting as ordinary abstraction
- user-defined operators and broad operator overloading
- overloaded dispatch and return-type overload
- dynamic `Any` as the normal interop path
- source-level references, borrowing syntax, raw pointers, or pointer identity
- hidden async suspension in ordinary calls
- classes, inheritance, constructors tied to class hierarchy, or hidden
  instance-variable state
- property access that performs hidden IO
- arbitrary unsandboxed build scripts
- scientific/ML, mobile UI, or embedded/no-std focus before Muga's package,
  tooling, and service foundations are stable

## Next Implementation Memory

When starting the next non-feature-planning session, prefer this order:

1. Build the conformance-suite skeleton and wire it into release readiness.
2. Define JSON diagnostics and command-output contracts.
3. Implement first `.mgi` API diff library comparison, CLI wrapper, and compatibility
   classifications. Done in [mgi-api-diff.md](mgi-api-diff.md).
4. Write stdlib review rules and link them from the v1 checklist. Done in
   [standard-library-review-rules.md](standard-library-review-rules.md).
5. Add the initial `muga test` / `@test` workflow. Done.
6. Add scalar assertions for `muga test`. Done.
7. Add line-comment-preserving deterministic `muga fmt` for v1 source files.
   Done.
8. Then choose the next optional pre-v1 usability slice.

If a proposed task is not in the first two sections, confirm that it is not
widening the v1 surface by accident.
