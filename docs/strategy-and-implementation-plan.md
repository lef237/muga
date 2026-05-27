# Strategy And Implementation Plan

Status: strategy note. This is not a language specification and does not
override [ROADMAP.md](../ROADMAP.md). Use it as the context-free plan for why
the next design and implementation slices are ordered the way they are.

Purpose: record Muga's comparative positioning, product direction, feature
sequencing, measurement stance, and non-goals in one place. If conversation
context is lost, this document should be enough to recover the strategic
direction before resuming implementation.

## How To Resume Work

When starting from no context, read these in order:

1. this document for the strategic plan and feature order
2. [ROADMAP.md](../ROADMAP.md) for the current implementation priority
3. [docs/implementation-resume-plan.md](implementation-resume-plan.md) for the
   implementation ledger and concrete test expectations
4. the relevant focused spec before changing a language area

Do not start a broad feature only because it appears useful. Check the phase
order below, the current roadmap priority, and the reconsideration rules in
[docs/practical-language-readiness.md](practical-language-readiness.md).
The current default is Core Capability Acceleration: prefer a thin,
end-to-end implementation of a practical core capability over another narrow
polish slice when the core capability fits Muga's explicit-effects model and
can be covered through type checking, runtime, artifacts, samples, and tests.
Before starting memory-management, resource-handle, concurrency, schema,
testing, or service-runtime work, also check that document's cross-cutting
design boundaries.

## North Star

Muga should be a small, readable, compiler-first application language for
fast edit-check-run loops, explicit effects, stable package contracts, and
agent-friendly tooling.

Runtime and compile-time performance are part of the final product goal. Muga
should eventually support production programs with fast feedback during
development and optimized/native builds that can compete with Rust or C++ on
representative value-heavy, collection-heavy, and service workloads. That is a
roadmap target, not a current public claim. Muga should not optimize benchmark
scores at the cost of readability, local reasoning, package-interface
stability, or practical application workflows.

The compiler-speed ambition is deliberately stronger than "Go-like." On
representative Muga projects, the target is to beat Go-style fast compilation
for syntax/check and warm incremental builds, and to compete with the fastest
mainstream compiled languages for cold builds. Treat that as a product design
constraint: every broad feature must justify its cost against parse, resolve,
typecheck, package-interface loading, artifact reuse, and future daemon/watch
latency.

Preserve these strengths:

- local type inference instead of whole-program inference
- immutable-by-default bindings and source-level value semantics
- implementation-managed ordinary memory with explicit resource lifetime
  boundaries for OS- and runtime-backed values
- records, enums, functions, modules, and package interfaces as the main tools
- `Option[T]` for absence and `Result[T, E]` for recoverable failure
- explicit control flow for early return, resource lifetime, and concurrency
- package artifacts that let downstream packages avoid dependency body reads
- no hidden framework conventions for service endpoints or generated clients

## Comparative Positioning

Muga should not try to beat every language by copying every feature.

Relative to Rust, Muga should target application and service code where Rust's
ownership model, borrow reasoning, async runtime choices, and compile times can
become too heavy for the problem. Muga should keep the source language readable
and value-oriented, then recover performance through compiler/runtime work.

Relative to Go, Muga should aim to be faster on day-to-day compiler feedback,
not merely comparable. The core benchmark should be edit-check-run latency:
single-file syntax checks, package checks that reuse dependency interfaces,
warm incremental builds, and later watch/daemon responses. Muga should pair
that with simple tooling, clear concurrency, and deployment simplicity, while
offering stronger modeling tools: `Option`, `Result`, enums, exhaustive
`match`, explicit package interfaces, and future schema/client generation from
typed contracts.

Relative to Java and C#, Muga should not initially compete on ecosystem size or
framework breadth. It should compete on lightweight distribution, explicit
contracts, fast feedback, low ceremony, and generated adapters that avoid
class-heavy framework conventions.

Relative to dynamic scripting languages, Muga should keep readable source and
low ceremony while giving the compiler enough static information to validate
package boundaries, errors, public contracts, and future service APIs.

## Measurement Stance

Strict external benchmarking is intentionally deferred. Current Muga still has
a moving language surface, a reference VM, and evolving artifact workflows. A
strict comparison against Go, Rust, Java, or C# too early would mostly measure
immature implementation choices and could pull design work toward narrow
micro-optimizations.

Use lightweight health checks now:

- syntax/check/build feedback should stay fast enough for edit-check-run work
- artifact-backed `check` and `run` must not read dependency implementation
  bodies
- private body changes should not change public interface hashes
- public signature changes must invalidate the right downstream artifacts
- check/build times should not regress catastrophically on representative
  samples
- diagnostics and future LSP paths should keep response time visible

Introduce stricter measurement only when the relevant layer exists:

- project build and artifact reuse: measure hit/miss behavior and rebuild scope
- compiler feedback speed: measure syntax/check/build latency across warm and
  cold project states against Go and other fast compilers before adding a daemon
  or watch mode
- control-flow MIR and native backend: measure execution speed and allocation
  behavior
- service runtime: measure latency, throughput, memory, cancellation, and idle
  connection scaling
- public release comparisons: use representative CLI, package, and service
  programs rather than isolated microbenchmarks

The rule is: keep performance claims falsifiable, but do not let premature
benchmarks define the language.

## Core Capability Acceleration

The previous implementation cadence deliberately favored small, audit-heavy
slices. That kept quality high, but it also delayed practical core features.
Going forward, context-free resumes should use this priority unless a
maintainer explicitly redirects the work:

1. **Process spine**: add the smallest `std::process` surface that can run a
   child command, capture status/stdout/stderr, and accept explicit cwd/env
   options through `Result`, without shell-profile or build-script magic.
2. **Structured task spine**: implement task groups, spawn/join, failure
   propagation, cancellation, and timeout boundaries before channels or broad
   async IO.
3. **Service IO spine**: add the narrow socket/HTTP pieces needed for a small
   JSON service only after resource-handle and task rules can express
   cancellation, shutdown, and backpressure.
4. **Performance spine**: introduce control-flow MIR and runtime
   representation work when it unlocks measurable compiler/runtime progress;
   keep native backend claims behind benchmarks.
5. **Distribution spine**: use the existing `.mgp` / `.mga` archive,
   verification, and source-free bundle foundation to move toward publish and
   install workflows after local trust boundaries remain stable.

Each accelerated slice should still be vertical: public contract, parser or
typechecker impact if any, runtime behavior, package/interface/artifact
behavior, sample or template adoption, focused tests, and release-readiness
evidence. Acceleration means choosing more consequential slices, not accepting
unclear semantics or untested shortcuts.

## Phase Plan

### Current Phase Progress

Update this table at the end of each completed slice so a future session can
see the next safe step without reconstructing prior context.

| Area | Status | Next Step |
|---|---|---|
| Small explicit syntax | `and` / `or`, `else if`, explicit `return expr`, `break` / `continue`, `for item in list` for `List[T]`, and payload discard `_` in enum patterns are implemented. | Keep broad catch-all matching, iterator protocols, and larger pattern syntax deferred. |
| Package build productization | `muga build` writes `.mgi` / `.mgc` / `.mgb` artifacts to `.muga/build`, preserves unchanged generated artifacts, reports written/reused artifact status, `.mgb` records package-local source hashes, `.mgi` public hashes stay stable across implementation-only edits, stale generic `.mgi` artifact diagnostics include artifact-root context and regeneration commands, `.mgb` implementation artifact diagnostics include the concrete artifact path plus package context, full `std::io::...` source type spellings suggest `import std::io` plus `io::...`, invalid `try Result::Ok(...)` placements avoid redundant constructor expected-type noise, all current `E005` ambiguity diagnostics include targeted annotation guidance, recursive annotation diagnostics suggest direct-recursion parameter/return annotations and mutual-recursion explicit signatures, builds same-level package artifacts concurrently over an acyclic dependency graph, `check --built` / `run --built` consume that default directory explicitly, `run --built` program arguments are covered and CLI usage no longer implies `check` accepts them, missing/stale default artifacts under `--built` point at `muga build <entry>`, artifact-backed run has missing/stale `.mgc` check-cache diagnostic coverage and check-cache diagnostics mention `muga emit-check-cache`, local path dependencies resolve through `muga.toml`, manifest builds write/validate local path/archive `muga.lock` metadata, library content hashing computes the first `sha256:<hex>` package identity input including declared resources, `emit-package-archive` writes deterministic `.mgp` source/resource archives and can print a pasteable local archive dependency snippet, library readback validates `.mgp` bytes and expected hashes, library materialization writes validated `.mgp` bytes to absent or empty local source/resource trees, local `.mgp` archive dependencies consume `.muga/packages` cache entries keyed by hash with malformed form/cache/lockfile edge cases covered, `std::fs::read_resource_text` reads manifest-declared UTF-8 resources for source, test, archive dependency, and built-artifact runs, the mini v1 spec is aligned with those implemented package workflow pieces, `docs/v1-release-checklist.md` now defines the feature freeze/release gate, post-v1 snippets are outside runnable `samples/`, CI exercises the built-artifact smoke path plus offline package verification, and `tests/release_readiness.rs` plus `scripts/v1-release-gate.sh` make v1 RC readiness evidence-backed. | Work through the Core Acceleration Queue in `docs/implementation-resume-plan.md`, starting with `std::process`, while preserving package/artifact guarantees. |
| Trust and maintenance | Initial `conformance/` suite skeleton is wired into `cargo test` and release readiness, with valid, rejecting, and package artifact fixtures tied to the mini spec and split specs. `muga syntax --format json` emits single-file lex/parse diagnostics for faster editor feedback, `muga check --format json` emits schema-versioned diagnostic JSON with entry path and `file://` URI metadata, CLI JSON compiler diagnostics include entry source context in `diagnostics[].context`, artifact-backed check JSON diagnostics include entry package, artifact-root, concrete artifact-file context, artifact/source/dependency hash context, and regeneration-command context when available, `muga explain <diagnostic-code>` prints documented diagnostic catalog entries and stable diagnostic-code family guidance from `errors.md`, `muga run --format json` emits schema-versioned run results with captured program stdout/stderr, returned `main` values, and compiler/runtime diagnostics, `muga test --format json` emits schema-versioned test results with pass/fail status, failure messages, captured per-test stdout/stderr, summary counts, and pre-run compiler diagnostics, runtime diagnostics attach `related` call-context notes for nested function calls and entry/test execution, failed `std::test` scalar assertions add source-spanned `R021` diagnostics at the user assertion call, runtime/debug v1 reporting keeps stack context in `related` notes and artifact next-actions in `regenerationCommand` context, release-neutral benchmark health checks cover compiler stages, package artifact reuse, and representative String/List/Map runtime paths, malformed-input planning now covers parser/syntax, `.mgp` package archives, local `muga.lock`, `.mgi` interfaces, `.mgc` check caches, and `.mgb` implementation artifacts, install/onboarding docs now cover `cargo install`, local checkout installs, `muga --version`, first generated projects, and later binary-release expectations without treating them as release triggers, example-driven learning now covers bindings, records, `Result`, packages, tests, local dependencies, and artifact-backed builds, registry security design now preserves the `.mgp` hash foundation and scopes future signing, provenance, lockfile enforcement, cache validation, and malicious-package handling before remote fetching, edition/feature fingerprint policy now scopes future language-edition and semantic feature-set inputs for package artifacts before backward-compatible migration is needed, `muga build --format json` emits schema-versioned artifact root, artifact kind, path, URI, and written/reused status data, `muga emit-artifacts --format json`, `muga emit-interface --format json`, and `muga emit-check-cache --format json` emit schema-versioned explicit artifact root, artifact kind, path, and URI data, `muga why-rebuild` emits compact human text output and `muga why-rebuild --format json` emits non-mutating artifact/cache explanations for `.mgi`, `.mgc`, `.mgb`, manifest lockfile metadata, and local archive-cache metadata states with focused stale dependency-interface and implementation dependency-interface set-change hash coverage, representative artifact-backed dependency API coverage now combines stdlib packages, `try`, generic records/functions, enums, and transitive dependencies without source-body fallback, `.mgi` public interface hash stability is now covered across implementation-only edits, source-span movement, generic records/functions, stdlib-backed signatures, and transitive public types, `src/api_diff.rs` and `muga api-diff` now implement the first `.mgi` API diff library and CLI comparator for compatible/source-compatible/breaking/unknown public changes, `.mgb` structural validation and bytecode merge behavior now has representative coverage for control-flow-heavy dependency bodies, private package items, and independently generated artifacts, `muga build` reuse output and lockfile update behavior now have focused coverage for local path and local archive dependencies after implementation-only edits, public signature edits, archive content updates, and malformed lockfiles, recursive annotation diagnostics now point direct recursion at parameter/return annotations and mutual recursion at explicit signatures for every function in the group, package-mode public signatures now have representative coverage for every v1-supported public type shape through in-memory and persisted interfaces, `muga metadata --format json` emits package/module/item/export metadata plus public interface docs and rendered types for editor, LSP, CI, and agent consumers, `muga workspace --format json` emits loaded packages, module source files, the default artifact root, and dependency edges reachable from an entrypoint, `muga hover --format json` emits declaration hover data with public docs and signatures, `muga completions --format json` emits visible package/interface completions with import aliases plus public docs and signatures, `muga definition --format json` emits go-to-definition data for import aliases, local bindings, and package/interface item references, `muga references --format json` emits declaration plus entry-module references for the same initial target set, the concrete JSON-backed editor workflow smoke test composes syntax/check/workspace/metadata/hover/completions/definition/references/run/test JSON without scraping human output, `muga doc` emits Markdown from public `.mgi`-backed records/enums/functions plus item-level public source comments, `muga new` creates app, lib, test, config app, strict CLI tool, and report app manifest project templates, `samples/projects/report_app` demonstrates a runnable local path dependency workflow with text-file IO, `Result` error handling, reusable APIs, and artifact-backed execution coverage, `docs/diagnostics-and-output.md` defines the current human and JSON command-output contracts, `docs/mgi-api-diff.md` defines `.mgi` API diff inputs, identity, compatibility classifications, deprecation handling, library comparison, CLI output, and JSON output, `docs/standard-library-review-rules.md` defines the review gate for future `std` slices, `docs/stdlib-package-samples-review.md` records the stdlib package docs and samples review for `std::io`, `std::fs`, `std::path`, `std::env`, `std::cli`, `std::time`, `std::string`, `std::fmt`, and the first `std::json` slice, including artifact-backed execution samples, `docs/release-gate-alignment.md` records the release gate and GitHub Actions alignment around `scripts/v1-release-gate.sh`, `docs/artifact-cache-explanations.md` defines the `muga why-rebuild` artifact/cache explanation contract, `docs/benchmark-health-checks.md` defines the local benchmark health-check contract, `docs/fuzzing-malformed-input-plan.md` defines the fuzzing and malformed-input plan, `docs/installation-and-onboarding.md` defines release-neutral installation and first-run onboarding, `docs/muga-by-example.md` defines the runnable learning path, `docs/registry-security-design.md` defines the future registry trust boundary, `docs/edition-feature-fingerprint-policy.md` defines the future edition and feature-set fingerprint boundary, the first `muga test` / `@test` workflow is implemented, `std::test` exposes scalar assertion helpers such as `test::assert_eq_int`, `std::option` / `std::result` expose narrow value helpers, `std::string` exposes narrow text assembly helpers, `std::fmt` exposes narrow text layout helpers, `std::list` / `std::map` expose narrow collection helpers, the v1 equality policy is documented as scalar-only, and `muga fmt --check` formats v1 source files while preserving line comments. | Choose the next narrow stdlib/API boundary only after documenting a contract and checking deferred surfaces. |
| Opaque resources | Interface-only `pub opaque type` names are implemented for package mode, `.mgi`, docs, metadata, hover, completions, definition, references, and downstream loaded-interface checking. The metadata-only `OpaqueHandleFacts` / `paramMode` interface slice now persists capability facts, close-function identity, and parameter modes in `.mgi` v5, includes them in public hashes, and exposes them through metadata, hover/completion metadata, and docs. The consuming-parameter checker rejects direct same-scope use-after-consume for loaded-interface `consume` parameters. The first runtime-backed file-handle boundary and read-only `std::fs::File` implementation are done, [post-file-handle-resource-surface-selection.md](post-file-handle-resource-surface-selection.md) selected a program stderr channel instead of another handle family, scalar `eprint` / `eprintln` now implement that channel, text output file handles are implemented from [text-output-file-handles.md](text-output-file-handles.md), `report_app` now demonstrates args/env, stdout/stderr, text-file handle writes through `using`, JSON run output, `Result`, local dependencies, artifact-backed execution, and `run --built`, [lexical-resource-cleanup.md](lexical-resource-cleanup.md) records the implemented first statement-form `using` slice with nested cleanup unwind hardening, minimal pure `std::cli` helpers are implemented over explicit `List[String]` values for positional and option lookup with coverage in std package source, samples, `report_app`, and examples tests, the CLI-first generated app template uses `std::env` and `std::cli` and is covered by project-template source, generated-project tests, and onboarding examples, the implemented typed scalar `std::cli` parsing helpers are covered by code, samples, and tests, the implemented JSON value accessor helpers in `std::json` return `json::Error` for wrong shapes, `samples/projects/config_app` now implements the JSON config workflow with `std::config`, `std::json`, `std::cli`, `std::result::map_err`, source, emitted-artifact, config shape-error, CLI > config > defaults, and `run --built --format=json` coverage, [post-config-workflow-adoption-gap-selection.md](post-config-workflow-adoption-gap-selection.md) selects a `std::result::map_err` refresh for app-boundary error normalization, that refresh is implemented in `config_app`, the implemented narrow pure `std::string` text assembly helpers (`string::concat_all` / `string::join`) are covered by code, samples, and tests, the implemented narrow `std::json` required object-field helpers are covered by code, samples, and tests, the implemented narrow `std::json` array/object field helpers are covered by code, samples, and tests, `samples/projects/config_app` implements a nested JSON config workflow with composite/typed helpers for `tags`, owner metadata, servers, and limits, pure `std::json` scalar array projection helpers and direct scalar-array object-field helpers are covered by code, samples, and tests, implemented repeated `std::cli` option value helpers and first-slice JSON path helpers are covered by code, samples, and tests, implemented typed JSON path scalar projection helpers and typed JSON path collection projection helpers are covered by code, samples, and tests, [json-schema-decoding.md](json-schema-decoding.md) records JSON schema decoding design and implements compiler-owned `json::decode_or[T](value, fallback)` as the first decoder, [std-config-json-loading.md](std-config-json-loading.md) implements `std::config::load_json_or[T]`, and [json-required-decoding.md](json-required-decoding.md) implements strict `json::decode[T](value)`. | Implement the smallest CLI parser schema overlay before TOML, full client generation, generic encoding/decoding, broader validators, config discovery automation, or host effects. |
| Service platform | Not implemented beyond narrow `std::fs` / `std::path` / `std::env` / `std::cli` / `std::time` slices, but now part of Core Capability Acceleration. | Start with process/resource/task spines before HTTP; do not jump straight to a web framework. |
| Structured concurrency | Draft only, but promoted from distant backlog to core acceleration target. | Design and implement a thin task-group spine with explicit spawn/join/failure/cancellation before channels or hidden async. |
| Native performance backend | Deferred as a backend claim, but the performance path is a core acceleration target. | Start with control-flow MIR and benchmark health evidence before native backend work. |

The `std::config` JSON default loading design in
[std-config-json-loading.md](std-config-json-loading.md) keeps the first
config/API boundary minimal before TOML, generated config templates, full CLI
parser schemas, or broader host effects.
The implemented `std::json` value accessor helpers and their tests still
record the original config-file loading deferral that led through schema
decoding and into this focused `std::config` slice before wider `Bytes` or
process APIs.
[std-config-json-loading.md](std-config-json-loading.md) now records the
implemented
`std::config::load_json_or[T](path, fallback)`, strict
`std::config::load_json[T](path)`, the public `config::Error` shape,
compiler-lowered schema payloads, `LoadJsonConfig` /
`LoadJsonConfigRequired` artifact behavior, and `config_app` coverage. The
generated config app template now packages that workflow for `muga new`.
The generated `muga new --template config-app` starter is implemented before
TOML, required decoding, full CLI parser schemas, formatting templates, broader
decoder targets, or broader host effects.
The generated-template adoption path is now carried by
[json-required-decoding.md](json-required-decoding.md), which selects required
`json::decode[T](value)` before TOML, broader decoder target types, full CLI
parser schemas, formatting templates, config discovery, or broader platform
APIs.
[json-required-decoding.md](json-required-decoding.md) records that strict
decoder implementation: expected `Result[T, json::Error]` target typing,
path-aware missing-field errors, ignored unknown fields, no-fallback schema
lowering, and artifact-safe `DecodeJsonRequired` payloads.
[json-decoder-target-expansion.md](json-decoder-target-expansion.md) implements
the decoder expansion for `Option[T]`, recursive `List[T]`, typed
`Map[String, T]`, and concrete non-generic enums across `json::decode_or[T]`,
`json::decode[T]`, `config::load_json_or[T]`, and `config::load_json[T]`, with
generic decoding, field/variant schema polish, and TOML still deferred.
The implemented `config_app` sample and generated `config-app` starter carry
the structural config workflow with `Option[String]`, nested records,
`List[Record]`, and typed `Map[String, Int]` settings before TOML, full CLI
parser schemas, formatting templates, config discovery, or broader platform
APIs.
The decoder expansion implements enum JSON/config decoder support, using
zero-payload string tags and one-payload single-key objects before generic enum
decoding, field/variant schema polish, TOML, full CLI parser schemas,
formatting templates, config discovery, or broader platform APIs.
[json-config-schema-polish.md](json-config-schema-polish.md) implements
`@json(rename: "...")` on record fields and enum variants before aliases,
validation attributes, TOML, full CLI schemas, schema generation, generic
decoding, or broader platform APIs.
[json-config-strict-unknown-fields.md](json-config-strict-unknown-fields.md)
implements record-level `@json(deny_unknown_fields)`, accepted wire-key
semantics, path-aware unknown-key errors, `.mgi` record flags, and `RF` decoder
artifact tokens before aliases, validation attributes, TOML, full CLI schemas,
schema generation, generic decoding, or broader platform APIs.
[json-config-alias-metadata.md](json-config-alias-metadata.md) implements
repeated `@json(alias: "...")` arguments inside a single field/variant `@json(...)`
attribute, accepted-name conflict checks, strict unknown-field integration, and
`RG`/`EG` artifact tokens.
[json-config-validation-attributes.md](json-config-validation-attributes.md)
implements the post-alias trust slice: field-level `@validate(...)` metadata
with scalar string/int validators, path-aware validation errors, `.mgi` v8
metadata, and `RV` decoder artifact tokens.
[json-config-schema-export.md](json-config-schema-export.md) implements the
post-validation adoption slice: `muga schema --format json` for JSON Schema Draft 2020-12
output with Muga
`x-muga` extensions, required/overlay decode modes, concrete public record/enum
scope, validation keywords, alias metadata, loaded-interface package coverage,
and explicit deferrals.
[json-typed-encoding.md](json-typed-encoding.md) implements typed JSON encoding
with compiler-owned `json::to_value[T](value)` plus
`json::encode_typed[T](value)`, canonical primary wire-name output, omitted
optional record fields, enum output matching decode/schema export,
validation-on-encode, artifact schema behavior, and the post-schema-export
bidirectional contract slice.
[cli-parser-schema.md](cli-parser-schema.md) selects and implements the first
compiler-owned `cli::parse_or[T](args, defaults)` and
`cli::usage_for[T](program, defaults)` typed CLI schema boundary for concrete
non-generic record overlays, preserving CLI > config > defaults precedence while
deferring TOML, strict no-default parsing, subcommands, short flags, config
discovery automation, full client generation, and host-effect APIs.
The generated `config-app` sample and project template use `cli::parse_or[T]`
for CLI > config > defaults settings overlays and expose `cli::usage_for[T]`
through a `--help` path before TOML, `@cli(...)`, dedicated `CliSchema`, config
discovery automation, strict no-default parsing, full client generation, or
host-effect APIs.
[cli-field-metadata.md](cli-field-metadata.md) records first `@cli(...)` field
metadata implementation and generated config-app metadata adoption, including
field help plus `--tag` / `--tags`, before TOML, config discovery automation,
strict no-default parsing, full client generation, or host-effect APIs.
[config-path-discovery.md](config-path-discovery.md) implements the first
explicit config path discovery slice for generated config apps:
`--config` remains highest precedence, `MUGA_CONFIG_PATH` supplies a deployment
default, and the generated JSON file remains the final fallback before TOML,
package resource lookup, service manifests, or runtime-owned config precedence.
[workspace-manifest-metadata.md](workspace-manifest-metadata.md) implements the
next project tooling slice by exposing manifest roots, source roots, resource
roots, direct dependencies, and dependency source/resource roots in
`muga workspace --format json` before runtime package resource lookup and then
installed-app resource layouts.
[config-app-run-helper.md](config-app-run-helper.md) applies the config path
policy in generated projects with a local README, `scripts/run-with-config.sh`,
and `scripts/package-config-app.sh` before runtime-owned config discovery,
TOML parsing, or shell-profile mutation.
[package-resource-archives.md](package-resource-archives.md) implements
manifest-declared text/binary package resource inclusion for content hashes,
`.mgp` archives, materialization, and local archive dependency caches.
[runtime-package-resource-lookup.md](runtime-package-resource-lookup.md)
implements read-only `std::fs::read_resource_text(package, path)` lookup over
those manifest-declared resources for source, test, archive dependency, and
explicit built-artifact runs before installed launchers.
[binary-file-read.md](binary-file-read.md) implements local
`std::fs::read_bytes` / `read_bytes_path` plus `bytes::at` inspection over
opaque `Bytes`, and [binary-file-write.md](binary-file-write.md) implements
full-file `std::fs::write_bytes` / `write_bytes_path` without adding binary
handles, buffers, codecs, or streams.
[installed-app-bundles.md](installed-app-bundles.md) implements the first
non-mutating app bundle layout with optional source-free output, JSON bundle
metadata, bundle-local dependencies, and `bin/<program>` launcher plus
`run-app-bundle` artifact execution, JSON-capable
`install-app --replace-owned` wrapper/ownership metadata placement plus
`list-installed-apps` inventory, generated helper install/list hooks, and
JSON-capable `uninstall-app` metadata-backed
removal for manifest projects, `emit-app-completions` package emission from
bundle interfaces, and deterministic `.mga` archive transport with JSON archive
and unpack metadata before
shell-profile installer mutation, registry publishing, binary streams/codecs,
broader cryptographic APIs, or package-manager-specific installers.
[cli-field-metadata.md](cli-field-metadata.md) implements the first field-level
CLI metadata slice: `@cli(name: "...", alias: "...", help: "...", hidden)` and
a dedicated `CliSchema` artifact boundary, keeping CLI option contracts
separate from JSON wire metadata before TOML, config discovery automation,
strict parsing, full client generation, or host-effect APIs.
[strict-cli-parser-schema.md](strict-cli-parser-schema.md) implements
strict `cli::parse[T](args)` for command-line-only records with required
options before TOML, config discovery automation, combined short flags, attached
values, subcommands, full client generation, generic encoding/decoding, broader
validators, or host-effect APIs.
compiler-owned `cli::parse[T](args)` with expected-result type inference,
`MissingArgument` errors, absent `Bool`/`Option`/`List` synthesis, strict
unsupported-field rejection, and no new no-default usage helper before
TOML, config discovery, combined short flags, attached values, subcommands, or host-effect APIs.
The checked-in strict CLI tool sample at
`samples/projects/cli_tool/src/main/main.muga` adopts the strict parser through
a root command, typed subcommands, generated help, compact short options, and
completion coverage before TOML, config discovery, full client generation,
generic encoding/decoding, broader validators, or host-effect APIs.
Generated `muga new --template cli-tool` adoption is implemented from that
sample shape, including source/build/`run --built`, generated README,
completion helper, and packaging helper coverage.
[strict-cli-no-default-usage.md](strict-cli-no-default-usage.md) implements
`cli::usage_for_required[T](program)` with explicit call type arguments,
source/artifact coverage, strict sample/template adoption, and the replacement
for the historical strict CLI manual help duplication.
[cli-command-metadata.md](cli-command-metadata.md) documents
`@cli(about: "...")` generated usage summaries before short options,
subcommands, TOML, config discovery automation, full client generation, or
host-effect APIs.
[cli-short-option-metadata.md](cli-short-option-metadata.md) implements exact
short-option syntax, parser behavior, generated usage rendering, app-owned
`cli::has_short_flag(args, "h")`, and interface/artifact-compatible schema
payloads.
[post-cli-short-option-metadata-adoption-gap-selection.md](post-cli-short-option-metadata-adoption-gap-selection.md)
selects typed CLI positional field metadata design next, so typed command
records can model primary operands before combined short flags, attached
values, built-in help branching, subcommands, TOML, config discovery
automation, shell completion generation, full client generation, or
host-effect APIs.
[cli-positional-field-metadata.md](cli-positional-field-metadata.md) selects
field-level `@cli(positional: N)` with explicit 1-based indexes, generated
`Arguments:` usage, parser behavior, interface/artifact compatibility, and
source/artifact/template coverage.
[post-cli-positional-field-metadata-adoption-gap-selection.md](post-cli-positional-field-metadata-adoption-gap-selection.md)
selects the built-in CLI help policy in
[cli-built-in-help-policy.md](cli-built-in-help-policy.md), which led to
`cli::help_requested` and generated help helpers after typed positional
operands landed.
[cli-built-in-help-policy.md](cli-built-in-help-policy.md) implements
`cli::help_requested`, `cli::help_for`, and `cli::help_for_required`, including
`--`-aware detection, schema-backed help rendering, opt-in help-name conflict
diagnostics, artifact-backed execution, and generated config/strict CLI
template adoption before parse-integrated help result enums, combined short
flags, attached values, subcommands, shell completion generation, TOML/config
discovery automation, full client generation, or host-effect APIs.
[post-built-in-cli-help-helper-adoption-gap-selection.md](post-built-in-cli-help-helper-adoption-gap-selection.md)
selected parse-integrated CLI help workflow design, so generated starters can
match a typed help-or-parsed request while runtime-owned printing/exits remain
deferred.
[parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md)
implements `cli::Request[T]`, `cli::parse_request[T]`, and
`cli::parse_request_or[T]` across strict/config starters before
runtime-owned printing/exits, subcommands, shell completions, TOML/config
discovery automation, full client generation, or host-effect APIs.
[post-parse-integrated-cli-help-workflow-adoption-gap-selection.md](post-parse-integrated-cli-help-workflow-adoption-gap-selection.md)
audits request workflow adoption and selects compact CLI short option syntax
design next, keeping subcommands, shell completions, config discovery, and
runtime-owned printing/exits deferred.
[compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md)
implements `-abc`, `-ofile`, and `-abo=value` over existing short metadata in
the runtime parser.
[post-compact-cli-short-option-syntax-adoption-gap-selection.md](post-compact-cli-short-option-syntax-adoption-gap-selection.md)
audits compact short syntax adoption and selected CLI subcommand metadata
design.
[cli-subcommand-metadata.md](cli-subcommand-metadata.md) implements enum/variant
metadata plus strict command enum schemas through source validation, `.mgi`
package interfaces, `.mgb` schema payloads, recursive runtime dispatch/help,
artifact-backed execution, and `run --built` before wrapper-record root/global
options, generated app shell completions, TOML/config discovery automation, or
runtime-owned printing/exits.
[post-cli-subcommand-schema-adoption-gap-selection.md](post-cli-subcommand-schema-adoption-gap-selection.md)
adopts that command-tree shape in `samples/projects/cli_tool` and generated
`muga new --template cli-tool` starters with `run` / `inspect` subcommands,
generated root/leaf help, compact short options, validation, artifact-backed
execution, and recoverable `cli::Error` mapping.
[cli-wrapper-root-options.md](cli-wrapper-root-options.md) implements strict
wrapper records with exactly one `@cli(subcommand)` field as the root/global
option shape, including schema/artifact lowering, runtime root-option parsing
before command dispatch, wrapper root help, and source/artifact/`run --built`
coverage. The strict sample and generated `cli-tool` starter now adopt a `Root`
wrapper with `--profile` / `-p`.
[cli-schema-shell-completions.md](cli-schema-shell-completions.md) implements
`muga cli-completions <bash|zsh|fish> --program <name> --type <Type> ...` as a
separate generated-app completion command driven by `CliSchema` across source,
`--artifact-root`, and `--built` workflows; after-command global options,
overlay/default wrappers, and config discovery remain deferred.
[post-cli-schema-shell-completion-adoption-gap-selection.md](post-cli-schema-shell-completion-adoption-gap-selection.md)
implements the first packaging/install adoption step through onboarding docs, a
generated `cli-tool` README, and `scripts/generate-completions.sh` while keeping
shell installation user-owned. [cli-completion-json-spec.md](cli-completion-json-spec.md)
implements `muga cli-completions --format json --program <name> --type <Type>
...` as the shell-agnostic recursive completion contract over the same
`CliSchema` facts, and shell renderers now use recursive scope transitions for
nested command trees. [cli-completion-value-sources.md](cli-completion-value-sources.md)
adds `@cli(value_source: "file"|"directory")` so path-valued String options and
positionals carry static completion metadata through source, interfaces,
artifacts, JSON, and shell renderers before future TOML/config discovery,
dynamic completion producers, or installer integration.
[cli-completion-installer-integration.md](cli-completion-installer-integration.md)
implements the non-mutating installer integration slice as
`muga emit-cli-completions --format json --output-dir <dir> --program <name>
--type <Type> ...`, writing bash, zsh, fish, and `.completions.json` files plus
machine-readable file metadata for user-owned or package-manager-owned
placement while keeping shell-profile edits deferred.

### Phase 1: Small Syntax That Preserves Readability

Add small, explicit syntax before broad abstraction systems.

Recommended order:

1. `and` / `or` short-circuit Boolean operators
2. `else if`
3. explicit `return expr`
4. `break` and `continue` for `while`
5. `for item in list` for `List[T]` only
6. payload discard `_` in enum patterns, without broad catch-all matching

Constraints:

- keep `and` / `or` Bool-only, left-to-right, and short-circuiting
- make `return` return from the nearest named or anonymous function, never from
  top-level script execution
- make `break` / `continue` target only the nearest loop in the same function
- keep the first `for` implementation limited to `List[T]` and immutable loop
  item bindings
- allow `_` only as a payload discard inside qualified one-payload enum variant
  patterns
- keep catch-all `_ =>` deferred because it can hide public enum evolution
- do not add iterator protocols for the first `for` implementation

### Phase 2: Productize Package Builds

Turn explicit artifacts into the normal development workflow.

Recommended order:

1. `muga build`
2. default project artifact directory, such as `.muga/build`
3. explicit `check` / `run` consumption of the default build directory
4. local path dependencies
5. unchanged package-level interface and implementation artifact reuse
6. package-local rebuild input metadata
7. public interface hash stability for implementation-only changes
8. parallel package builds over an acyclic dependency graph
9. lockfile and content-hash dependency resolution; the local-path metadata, validation/update policy, and first canonical content-hash helper have landed, while full non-local enforcement remains deferred
10. package archive format/emission/readback validation plus local materialization, local archive dependency cache consumption, cache/lockfile edge-case hardening, pasteable local archive dependency snippets, visible local build reuse/stale diagnostics, contextual generic record literal hardening, and focused ambiguity diagnostic guidance; next continue remaining v1 diagnostics hardening before publish/install workflows

Constraints:

- preserve `.mgi` as the public interface artifact
- preserve `.mgc` as the check-cache proof
- preserve `.mgb` as the implementation artifact
- never silently fall back to dependency source bodies in artifact-backed builds
- keep artifact-root configuration out of `muga.toml` until dependency
  declarations and lockfiles make project-level build state meaningful
- keep source imports logical; URLs, hashes, and filesystem paths belong in
  manifests and lockfiles, not `.muga` files

This is the main path toward compile feedback that is faster than Go-like fast
rebuilds without giving up stronger typed contracts.

### Phase 3: Limited Receiver Overload For Dot Chains

Before adding protocols or traits, consider a deliberately small overload
mechanism keyed only by receiver type.

Candidate shape:

```muga
fn len(self: String): Int {
  self.char_count()
}

fn len[T](self: List[T]): Int {
  // implementation omitted
}
```

Rules to preserve:

- overload resolution may use only the first argument / receiver type
- return-type overload is not allowed
- implicit conversions are not allowed
- protocol-based dot lookup is not part of this phase
- same receiver type plus same visible function name is an ambiguity error
- resolved receiver overload data must be representable in `.mgi`

Reason: dot chains are central to Muga readability. Limited receiver overload
can support natural names like `len`, `is_empty`, and `to_string` across common
types without importing the full complexity of traits, typeclasses, or dynamic
dispatch.

### Phase 4: Opaque Types And Resource Handles

Add representation hiding before broad IO, database, socket, or service APIs.
The boundary design lives in
[opaque-resource-handles.md](opaque-resource-handles.md). The first
interface-only `pub opaque type` slice is implemented, and the metadata-only
`OpaqueHandleFacts` / `paramMode` interface slice now fixes the next non-runtime
boundary, the consuming checker now covers direct same-scope uses after
loaded-interface `consume` parameters, and the first runtime file-handle design
and read-only `std::fs::File` implementation are done. The
post-file-handle selection in
[post-file-handle-resource-surface-selection.md](post-file-handle-resource-surface-selection.md)
chose scalar `eprint` / `eprintln`, now implemented as a program stderr channel
rather than stdout/stderr handles. Text output file handles are implemented from
[text-output-file-handles.md](text-output-file-handles.md), and broader
runtime-backed handle values and effectful APIs remain deferred.

Recommended order:

1. `pub opaque type Name` for runtime/native handle names
2. package-interface support for public opaque names
3. capability, consuming-parameter, explicit close, and diagnostic metadata in
   interfaces and tooling
4. consuming-parameter checking
5. runtime representation for compiler-provided opaque handles
6. `pub opaque record Name { ... }` only when ordinary Muga data needs a hidden
   representation

Examples:

```muga
package std::fs

pub opaque type File

pub fn open(path: path::Path): Result[File, io::IOError]
pub fn write(file: File, text: String): Result[Unit, io::IOError]
```

Constraints:

- importing packages can name an opaque type but cannot construct it or access
  its representation
- opaque runtime handles should define ownership, close/drop behavior,
  task-boundary behavior, and cancellation behavior
- per-field visibility should remain deferred unless concrete code needs a
  partially transparent public record

### Phase 5: Structured Resource Lifetime

Add explicit resource lifetime syntax after opaque handles have a stable
interface representation.

Preferred candidate:

```muga
using file = try fs::open(path) {
  try file.write(text)
}
```

Rules to decide before implementation:

- whether `using` binds only opaque resource handles or any value with a close
  operation
- how close failures are represented
- what happens when both the body and close operation fail
- whether the block value is returned directly or wrapped in `Result`
- how cancellation interacts with close

Constraints:

- do not use `with`; it already belongs to record update
- do not hide resource lifetime behind implicit exceptions
- keep lifetime lexical and visible
- keep the first slice narrow, even if it handles only compiler-known resource
  families

### Phase 6: Equality, Hashing, And Collections

Set and arbitrary Map keys need equality/hash support beyond the scalar-only v1 equality policy first.

Recommended order:

1. keep v1 equality limited to `Int`, `Bool`, and `String`
2. define hash support for built-in scalar types before hash-based collections
3. derive structural equality/hash for records and enums only when all fields
   or payloads support it
4. persist that support in package interfaces
5. add `Set[T]`
6. allow broader `Map[K, V]` keys
7. add explicit `map { ... }` literals

Constraints:

- no user-defined operator overload
- no implicit dynamic equality for unsupported values
- no structural equality for v1 records, enums, `Option`, `Result`, lists, or maps
- plain `{ ... }` must not become map syntax because it already carries block
  and record-literal roles

### Phase 7: Web And Service Platform

Grow Web support from explicit lower-level contracts rather than a hidden
framework.

Recommended order:

1. `Bytes`, `Buffer`, and `StringBuilder`
2. `std::json` parse/encode with explicit `Result` errors
3. URL and header types
4. `std::http` request and response data types
5. explicit server/listener/resource APIs
6. schema generation from `.mgi`
7. OpenAPI or RPC adapter packages
8. generated TypeScript clients or other external bindings
9. logging, metrics, config, and process APIs

Constraints:

- a plain `pub fn` is a package function, not automatically an endpoint
- packages opt into HTTP/RPC through explicit adapter APIs
- generators consume `.mgi`, not private source bodies
- unsupported public types must make generation fail clearly
- `Option`, `Result`, `List`, `Map`, enums, records, and opaque types need
  explicit external representation rules

### Phase 8: Structured Concurrency

Keep structured concurrency as the primary model, not async function coloring.

Recommended order:

1. source-nameable `Task[T]` as an opaque type
2. `group { ... }`
3. `spawn expr`
4. `task.join()`
5. failure and cancellation rules
6. task-boundary capture rules
7. typed channels
8. `select`-style coordination
9. timeouts and deadlines
10. cancellation-aware nonblocking IO

Important decision: `join()` must not behave like a hidden exception. Its
failure mode should be visible through `Result` or an explicit enum such as a
future `JoinResult[T]`.

Constraints:

- child tasks cannot outlive their group
- immutable captures should be easy
- mutable captures across task boundaries should be rejected or made explicit
- cancellation should be structured and downward-propagating
- channels and select come after task groups

### Phase 9: Performance Backend

Runtime speed work should follow stable semantics.

Recommended order:

1. control-flow-oriented MIR
2. explicit locals, temporaries, moves, branches, and terminators
3. efficient String/List/Map representations
4. copy elision
5. destructive-update lowering when a value is uniquely owned
6. escape analysis and stack allocation
7. inlining and specialization for hot generic or higher-order functions
8. Cranelift native backend as the likely first native target

Constraints:

- keep source-level value semantics
- do not add source-level references as the ordinary performance answer
- measure allocation and copy behavior once MIR/backend work begins

## Tooling Direction

Muga should be easy for both people and coding agents to use.

Prioritize:

- conformance fixtures tied to the specs
- stable machine-readable diagnostics
- `.mgi` API compatibility diffing
- formatter
- LSP with fast local feedback
- package graph inspection through `muga metadata`
- visible package/interface completions through `muga completions`
- go-to-definition through `muga definition`
- artifact/cache explanation commands
- generated docs from `.mgi`
- minimal project scaffolding
- test runner and sample runner

Recommended order: the concrete JSON-backed editor workflow has landed on top
of entry-aware JSON output, `muga metadata` package facts, declaration hovers,
visible completions, go-to-definition, references, workspace metadata, run JSON,
and test JSON. `muga why-rebuild` now implements compact human text output, and
`muga why-rebuild --format json` implements the non-mutating JSON surface from
[artifact-cache-explanations.md](artifact-cache-explanations.md), including
local archive-cache metadata. Minimal
`muga new --list-templates` plus app/lib/test/config-app/cli-tool/report-app/resource-export/package-app scaffolding, public source comments in generated docs,
top-level and command-specific `muga help`, static shell completions, and `muga doctor` have landed as tool-only adoption
surfaces. The first `std::json` slice is implemented from
[std-json-first-slice.md](std-json-first-slice.md) and audited in
[std-json-implementation-audit.md](std-json-implementation-audit.md),
preserving Result ergonomics, scalar/collection mapping, schema evolution, and
diagnostics. Further expansion should not expand into schema generation,
HTTP/RPC, `Float`, `Decimal`, `Bytes`, streaming APIs, or resource handles.
The post-JSON stdlib/API boundary selection is recorded in
[post-json-stdlib-boundary-selection.md](post-json-stdlib-boundary-selection.md):
design opaque resource handles before stdout/stderr handles, file handles,
process APIs, HTTP/SSE/WebSocket/RPC, streaming APIs, `Bytes`, buffers, or
schema/client generation. The resource-handle boundary itself is defined in
[opaque-resource-handles.md](opaque-resource-handles.md). The follow-up
selection is recorded in
[post-file-handle-resource-surface-selection.md](post-file-handle-resource-surface-selection.md):
scalar `eprint` / `eprintln` implement the program stderr channel, and
[text-output-file-handles.md](text-output-file-handles.md) defines the implemented
write-mode file handle slice while standard-stream handles and broad IO remain
deferred.
Service-facing generators should still wait until schema and resource/runtime
boundaries are explicit.

Agent-friendly tooling is a strategic advantage. The compiler should expose
structured data instead of forcing tools to scrape human text.

## Maintenance And Trust Direction

Muga's long-term shape depends on evidence and compatibility, not just feature
count.

Prioritize:

- conformance tests that can be run by future compiler versions
- API compatibility checks over `.mgi` public interfaces
- standard-library review rules before broadening `std`
- runtime and test failure reports with source spans and useful call context
- lightweight health benchmarks before public performance claims
- install, version, and quickstart docs that do not force release timing
- `.mgp` archive hashing as the local foundation for later signing,
  provenance, registry trust, and lockfile enforcement in
  [registry-security-design.md](registry-security-design.md)
- edition or feature-set fingerprints in
  [edition-feature-fingerprint-policy.md](edition-feature-fingerprint-policy.md)
  before backward-compatible migration is needed

Remote registries, binary distribution channels, strict benchmark thresholds,
and edition migrations should wait until the local package/artifact contract is
stable enough that those promises can be maintained.

The modern-language gap decision pass in
[modern-language-gap-decisions-2026-05-22.md](modern-language-gap-decisions-2026-05-22.md)
adds a stricter ordering rule: conformance, JSON diagnostics, command-output
contracts, `.mgi` API diffing, stdlib review rules, package metadata, and
artifact/cache explanations such as
[artifact-cache-explanations.md](artifact-cache-explanations.md), fuzzing and
malformed-input planning, release-neutral onboarding in
[installation-and-onboarding.md](installation-and-onboarding.md),
example-driven learning in [muga-by-example.md](muga-by-example.md), and
registry security design in
[registry-security-design.md](registry-security-design.md), and edition
feature-set fingerprint policy in
[edition-feature-fingerprint-policy.md](edition-feature-fingerprint-policy.md)
come before broad syntax, registry, native backend, or service runtime
expansion.

## Features To Avoid For Now

Keep these out unless a later concrete design note proves they are needed:

- broad trait, protocol, interface, or typeclass systems
- protocol inheritance, blanket implementations, specialization, and protocol
  objects
- user-defined operator overload
- return-type overload
- implicit conversions
- implicit throwing exceptions
- `async fn` / `await` as the primary concurrency model
- source-level references, borrowing syntax, raw pointers, or pointer identity
- runtime metaprogramming as a core abstraction mechanism
- dynamic `Any` as the normal interop path
- wildcard-heavy pattern matching and early catch-all enum patterns
- hidden web framework conventions that turn ordinary public functions into
  network endpoints
- property access with hidden IO

## Decision Checklist

Before adding a feature, require clear answers:

1. Does it preserve local readability?
2. Does it keep control flow and effects visible?
3. Can it be represented in `.mgi`?
4. Can unchanged dependencies avoid body reads?
5. Does it avoid whole-program inference?
6. Does it fit source-level value semantics?
7. Does it keep dot chain meaning predictable?
8. Does it need strict benchmarking now, or only a health check?
9. Does it improve real programs enough to justify parser, typechecker,
   diagnostic, and tooling cost?

If the answer is uncertain, prefer a narrow library API, explicit package
adapter, or focused design note over a new language feature.
