# V1 Release Checklist

Status: v1 boundary and release-readiness checklist. This document defines what Muga v1 promises, what it deliberately does not promise, and the gates that must pass before a v1 release candidate or final release.

Passing this checklist is a quality signal, not a requirement to publish the
next crate as v1. While the language, package, or artifact specifications are
still expected to change, release version and timing remain separate maintainer
decisions. Keep using this gate to prevent regressions in the intended v1
workflow.

Use this document after [ROADMAP.md](../ROADMAP.md), [mini-language-spec-v1.md](../mini-language-spec-v1.md), and [docs/implementation-resume-plan.md](implementation-resume-plan.md). If this checklist conflicts with an older roadmap note, prefer the narrower v1 promise here and update the older note.

## V1 Promise

Muga v1 is the first stable small-core release. It promises:

- the closed v1 grammar and semantics documented in [mini-language-spec-v1.md](../mini-language-spec-v1.md) and the implemented split specs
- source-compatible `check` and `run` for scripts, package files, and manifest projects
- immutable-by-default local reasoning, local type inference, records, enums, functions, closures, generic records/functions, `List`, `Map`, `Option`, `Result`, and visible `try expr`
- package mode with module-private default visibility, `pkg`, `pub`, explicit imports, and no package top-level execution
- `.mgi` public interface artifacts, `.mgc` package check-cache artifacts, and `.mgb` MIR-lowered bytecode implementation artifacts
- explicit artifact-backed `check` and `run` through `--artifact-root`, plus default `.muga/build` consumption through `check --built` and `run --built`
- local path dependencies and local `.mgp` archive dependencies with deterministic source/resource hashing, minimal local lockfile metadata, and read-only runtime resource lookup for declared UTF-8 resources
- clear diagnostics for the supported v1 workflow, especially ambiguity, visibility, stale artifact, missing artifact, hash mismatch, and invalid artifact cases

## Feature Freeze

These are not v1 blockers and should not be started before v1 unless they are required to fix a concrete v1 regression:

- public-signature inference for `pub fn`
- artifact-root configuration in `muga.toml`
- URL/Git/registry dependency forms, remote fetching, registry publishing workflows, package signing, and full published-package lockfile enforcement
- shell-profile installer mutation and package-manager-owned install workflows
- full incremental project artifact reuse
- resource-handle growth beyond the current `std::fs::File` text handle and
  statement-form `using` cleanup slice, `Bytes`, stdout/stderr handles,
  process APIs, HTTP, SSE, WebSocket, RPC, service runtime, schema generation,
  and client generation
- richer `std::cli` parsing beyond the current pure helpers,
  `cli::parse_or[T]`, strict `cli::parse[T]`, and overlay usage generation,
  including short flags, subcommands, no-default usage helpers, config
  discovery, and richer usage diagnostics
- control-flow MIR, native backend, optimizer work, and backend benchmarking claims
- traits, protocols, typeclasses, overloaded dispatch, user-defined operators, references or borrowing syntax, broad wildcard matching, map literals, `Set[T]`, arbitrary `Map` keys, call-site type arguments, iterator protocols, named arguments, `using` expressions or multiple bindings, range/slicing syntax, string interpolation, `T?`, `?.`, `expr?`, and `expr.try`
- concurrency syntax such as `group`, `spawn`, `join`, channels, `select`, `async`, or `await`

The initial `muga test` workflow with a minimal compiler-recognized `@test`
attribute is implemented, and `std::test` now provides scalar assertion
helpers. The `std::option` and `std::result` value-helper packages are also
implemented, along with narrow `std::list` and `std::map` helper packages. The
v1 equality policy is documented as scalar-only for `Int`, `Bool`, and `String`.
Any further helper additions, structural equality, structural assertions,
`List.contains`, or `Map.entries` must update the specs, docs, samples, and
focused tests before they are treated as part of the v1 surface.

Tooling additions that preserve the accepted source language may also be built
before v1. The deterministic `muga fmt` is implemented with `--check` and line
comment preservation, top-level and command-specific `muga --help` / `muga help`
prints usage for first-run CLI discovery, `muga syntax --format json` emits single-file
lex/parse diagnostics for faster editor feedback, and `muga check --format json`
includes entry path plus `file://` URI metadata for editor, CI, LSP, and agent
consumers. CLI JSON compiler diagnostics also include entry source context in
`diagnostics[].context`, and artifact-backed check diagnostics include entry
package, artifact-root, and concrete artifact-file context when available.
`muga doc`
generates Markdown from public package interface records, enums, functions, and
item-level public source comments stored in `.mgi`. `muga new` lists available
templates and creates a CLI-first app template using `std::env` / `std::cli`
that prints `hello Muga` by default and accepts `--name`, plus lib, test,
config app, strict CLI tool, and report app project templates.
`muga metadata --format json` emits
package/module/item/export metadata plus public interface docs and rendered
types for editor, LSP, CI, and agent consumers. `muga workspace --format json`
emits loaded packages, module source files, the default artifact root, and
dependency edges reachable from an entrypoint. `muga hover --format json`
emits declaration hover data with public docs and signatures.
`muga completions --format json` emits visible package/interface completions
with import aliases plus public docs and signatures.
`muga definition --format json` emits go-to-definition data for import aliases,
local bindings, and package/interface item references.
`muga references --format json` emits declaration plus entry-module references
for the same initial target set. `docs/editor-json-workflow.md` documents the
concrete adapter sequence across syntax, check, workspace, metadata, hover,
completions, definition, references, run, and test JSON.
`muga build --format json` emits structured
artifact root, artifact kind, path, URI, and written/reused status data.
`muga emit-artifacts --format json`, `muga emit-interface --format json`, and
`muga emit-check-cache --format json` emit structured explicit artifact output.
Artifact diagnostics include structured hash and regeneration-command context
where available. `muga test --format json` emits structured test results,
captured per-test stdout/stderr, summary counts, and pre-run compiler
diagnostics. `muga run --format json` emits captured program stdout/stderr,
the returned `main` value when present, and
compiler/runtime diagnostics. `samples/projects/report_app` covers a runnable
local path dependency workflow with args/env, stdout/stderr, text-file handle writes,
JSON run output, `Result` error handling, reusable APIs, artifact-backed
execution, and `run --built`. `muga explain <diagnostic-code>`
prints the documented diagnostic catalog entry or stable diagnostic-code family
from `errors.md`.
They should use the v1 grammar and package/interface facts, must not silently
change artifact semantics, and must not be treated as release triggers.

Maintenance additions may also be built before v1 when they validate the
current surface: conformance fixtures, machine-readable diagnostics, `.mgi`
API-diff design, library comparison, and CLI wrapper in [mgi-api-diff.md](mgi-api-diff.md), standard-library review
rules in [standard-library-review-rules.md](standard-library-review-rules.md),
artifact/cache explanation command coverage in
[artifact-cache-explanations.md](artifact-cache-explanations.md), benchmark
health checks in [benchmark-health-checks.md](benchmark-health-checks.md),
fuzzing and malformed-input planning in
[fuzzing-malformed-input-plan.md](fuzzing-malformed-input-plan.md), runtime
failure reports, install and onboarding docs in
[installation-and-onboarding.md](installation-and-onboarding.md), tool-only
shell completions and `muga doctor` in
[shell-completions-and-doctor.md](shell-completions-and-doctor.md), and
example-driven education in [muga-by-example.md](muga-by-example.md).
Keep binary distribution,
remote registry trust/signing/provenance, edition migration, and strict public
performance benchmark thresholds separate from v1 release timing unless a
concrete v1 workflow requires them.
Future registry trust boundaries are scoped in
[registry-security-design.md](registry-security-design.md) while remote
fetching remains deferred.
Future edition and semantic feature-set fingerprint boundaries are scoped in
[edition-feature-fingerprint-policy.md](edition-feature-fingerprint-policy.md)
while edition migration remains deferred.

Use [modern-language-gap-decisions-2026-05-22.md](./modern-language-gap-decisions-2026-05-22.md)
to classify new ideas from the modern-language inventory. Before v1, prefer
validation/support work that preserves the current source language: conformance
layout, JSON diagnostics, stable command-output contracts, `.mgi` API-diff
design, library comparison, and CLI wrapper in [mgi-api-diff.md](mgi-api-diff.md), stdlib review rules in
[standard-library-review-rules.md](standard-library-review-rules.md),
doc-comment/API-doc rules, package metadata, artifact/cache explanations in
[artifact-cache-explanations.md](artifact-cache-explanations.md), benchmark
health checks in [benchmark-health-checks.md](benchmark-health-checks.md),
fuzzing plans in
[fuzzing-malformed-input-plan.md](fuzzing-malformed-input-plan.md), runtime
failure context, install and onboarding docs in
[installation-and-onboarding.md](installation-and-onboarding.md), tool-only
shell completions and `muga doctor` in
[shell-completions-and-doctor.md](shell-completions-and-doctor.md), and
example-driven learning material in [muga-by-example.md](muga-by-example.md).
The first `std::json` slice is scoped in
[std-json-first-slice.md](std-json-first-slice.md) and implemented within that
package contract after documenting `Result` ergonomics, scalar/collection
mapping, schema evolution, and diagnostics; the pure value/object-field
accessor follow-up returning `json::Error` is scoped by the implemented
`std::json` package source, sample, examples, and release-readiness coverage.
It must not expand into config-file loading, schema generation, HTTP/RPC,
`Float`, `Decimal`, `Bytes`, streaming APIs, or broader resource handles.

Additional syntax is outside the frozen v1 surface by default. `@test` is
admitted only as static metadata for `muga test`, and the first statement-form
`using` cleanup slice is admitted only for runtime-backed opaque handles with
compiler-known close metadata. Named arguments, `using` expressions/multiple
bindings, range/slicing syntax, pattern-matching refinements, interpolation,
`T?`, and `?.` remain deferred unless the v1 checklist, specs, diagnostics,
formatter rules, samples, and focused tests are deliberately updated first.

AI agents should not proactively recommend publishing, tagging, or cutting a
release until the v1 completion criteria are satisfied, unless the maintainer
explicitly asks for release preparation.

## Sample Policy

Before v1:

- every `.muga` file under `samples/` must be a runnable entrypoint, a support source file for a runnable entrypoint, or an intentionally rejected fixture covered by an automated test path
- future-looking snippets that are not valid v1 source must live under `docs/design-snippets/`, not under `samples/`
- README sample links must distinguish runnable samples from design snippets
- representative package samples must continue to cover source-compatible `run`, explicit artifact-backed `run`, and `run --built`

## Diagnostic Policy

Diagnostics are part of the v1 contract. Exact wording may evolve, but these properties must hold:

- each public diagnostic has a stable code family documented in [errors.md](../errors.md)
- ambiguity diagnostics tell users what annotation or import makes the program unambiguous
- package/interface/cache/implementation artifact diagnostics include the relevant package or artifact path when available, and JSON diagnostics include concrete artifact-file, artifact-hash, and regeneration-command context when that data is known
- artifact-backed commands fail loudly on missing, stale, hash-mismatched, structurally invalid, or wrong-package artifacts
- artifact-backed commands must not silently read dependency implementation source bodies after an artifact failure
- `--built` failures point at `muga build <entry>`; explicit artifact-root failures point at the focused `emit-*` command where that is more actionable
- source spelling mistakes such as `std::io::IOError` in type annotations point users at `import std::io` plus the local `io::IOError` form

## Artifact Workflow Policy

The v1 package boundary is explicit:

- `.mgi` stores the public package interface and is the typed contract for downstream checking
- `.mgc` stores the package check-cache proof for an entry package and must not grow into an executable body store
- `.mgb` stores MIR-lowered bytecode implementation bodies for artifact-backed execution
- plain `check` and `run` remain source-compatible and do not silently consume `.muga/build`
- `check --built` and `run --built` explicitly consume `.muga/build`
- `check --artifact-root` and `run --artifact-root` explicitly consume the supplied artifact root
- local `.mgp` archive dependencies are a local deterministic archive workflow, not a registry or publishing workflow
- future registry security, signing, provenance, full lockfile enforcement, and malicious-package handling are design-only in [registry-security-design.md](registry-security-design.md)
- future edition selectors, semantic feature-set fingerprints, and migration tooling are design-only in [edition-feature-fingerprint-policy.md](edition-feature-fingerprint-policy.md)

## Release Gate

The standard release-quality gate for v1 preparation is:

```bash
scripts/v1-release-gate.sh
scripts/v1-release-gate.sh --with-publish-dry-run
```

The first command is the offline local/CI gate. The second command adds `cargo publish --dry-run --locked`; it may contact crates.io and should be run manually when preparing a publish.
The local script is the canonical command list. GitHub Actions should invoke
that script instead of duplicating the gate commands; see
[release-gate-alignment.md](release-gate-alignment.md).

The scripted gate runs:

```bash
cargo fmt --check
scripts/clippy-check.sh
cargo test --locked
cargo build --locked
mkdir -p "$gate_tmp"
target/debug/muga check samples/println_sum.muga
target/debug/muga samples/println_sum.muga
target/debug/muga build samples/packages/app/artifact_facade/main.muga
target/debug/muga check --built samples/packages/app/artifact_facade/main.muga
target/debug/muga run --built samples/packages/app/artifact_facade/main.muga
target/debug/muga api-diff --old-artifact-root samples/packages/app/artifact_facade/.muga/build --new-artifact-root samples/packages/app/artifact_facade/.muga/build --package app::artifact_facade --fail-on breaking
target/debug/muga emit-package-archive --archive-root "$gate_tmp/package-archives" samples/projects/local_path_shared/src/logging/main.muga
target/debug/muga verify-package-archive "$package_archive_path"
target/debug/muga verify-package-archive --expected-hash "$package_archive_hash" "$package_archive_renamed"
target/debug/muga unpack-package-archive --expected-hash "$package_archive_hash" --output-dir "$gate_tmp/renamed-unpacked-package" "$package_archive_renamed"
target/debug/muga check "$gate_tmp/renamed-unpacked-package/src/logging/main.muga"
cp -R samples/projects/my_service "$gate_tmp/my_service"
target/debug/muga emit-app-bundle --source-free --output-dir "$gate_tmp/app-bundle" --program release-gate "$gate_tmp/my_service/src/main/main.muga"
target/debug/muga emit-app-archive --archive-root "$gate_tmp/app-archives" --program release-gate "$gate_tmp/app-bundle"
target/debug/muga verify-app-archive "$app_archive_path"
target/debug/muga verify-app-archive --expected-hash "$app_archive_hash" "$app_archive_renamed"
target/debug/muga unpack-app-archive --expected-hash "$app_archive_hash" --output-dir "$gate_tmp/renamed-unpacked-app" "$app_archive_renamed"
target/debug/muga unpack-app-archive --output-dir "$gate_tmp/unpacked-app" "$app_archive_path"
target/debug/muga run-app-bundle "$gate_tmp/unpacked-app"
target/debug/muga install-app --output-dir "$gate_tmp/installed-bin" --program release-gate "$gate_tmp/unpacked-app"
target/debug/muga list-installed-apps --output-dir "$gate_tmp/installed-bin"
MUGA_BIN="$PWD/target/debug/muga" "$gate_tmp/installed-bin/release-gate"
target/debug/muga uninstall-app --output-dir "$gate_tmp/installed-bin" --program release-gate
cp -R samples/projects/resource_export "$gate_tmp/resource_export"
target/debug/muga emit-app-bundle --source-free --output-dir "$gate_tmp/resource-export-bundle" --program resource-export "$gate_tmp/resource_export/src/main/main.muga"
target/debug/muga run-app-bundle "$gate_tmp/resource-export-bundle" -- "$gate_tmp/resource-export-payload.bin"
cargo package --locked --allow-dirty --offline --list
cargo package --locked --allow-dirty --offline
```

`scripts/clippy-check.sh` runs
`cargo clippy --locked --all-targets --all-features -- -D warnings`; the crate
also keeps deny-by-default Rust/Clippy lint policy in `Cargo.toml` and the
pinned Clippy MSRV in `clippy.toml`.

CI must run the offline package/app archive verification and CLI smoke checks by invoking
`scripts/v1-release-gate.sh`. The release workflow must run
`scripts/v1-release-gate.sh --with-publish-dry-run` before publishing.

## Completion Criteria

Muga v1 is release-ready when:

- [x] the feature freeze above is still intact
  Evidence: `tests/release_readiness.rs` checks stale post-v1 sample references, the conformance suite layout, and the release-readiness docs point at this checklist; any new language feature should also require changing ROADMAP/checklist text and focused tests.
- [x] `samples/` contains only runnable entrypoints, their support source files, or intentionally rejected fixtures
  Evidence: `tests/release_readiness.rs` rejects planned/future snippet paths under `samples/` and asserts post-v1 concurrency snippets live under `docs/design-snippets/`.
- [x] `errors.md` describes the stable diagnostic code families and v1 diagnostic guarantees
  Evidence: `tests/release_readiness.rs` scans diagnostic code prefixes used by `src/*.rs`, requires each prefix to be documented in `errors.md`, and checks that the JSON diagnostic/output contract is documented and covered by CLI tests.
- [x] README documents the narrow v1 artifact workflow and points to this checklist
  Evidence: `tests/release_readiness.rs` checks README, ROADMAP, and RELEASING references to this checklist.
- [x] ROADMAP and implementation-resume notes agree that the active v1 work is hardening, not broad feature expansion
  Evidence: ROADMAP calls the v1 surface feature-frozen, and `docs/implementation-resume-plan.md` records v1 release boundary hardening as the completed slice.
- [x] CI runs formatting, linting, tests, CLI smoke checks, and offline package/app archive verification
  Evidence: `.github/workflows/ci.yml` invokes `scripts/v1-release-gate.sh`, and `tests/release_readiness.rs` checks the canonical script command list plus [release-gate-alignment.md](release-gate-alignment.md).
- [x] the offline release gate passes locally, and the publish dry run is explicit before tagging
  Evidence: `scripts/v1-release-gate.sh` passed for this readiness slice, `.github/workflows/release.yml` invokes `scripts/v1-release-gate.sh --with-publish-dry-run` before publishing, and the option adds the network publish dry run for the final tag-time check.
