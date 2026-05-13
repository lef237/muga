# Implementation Resume Plan

Status: current implementation ledger for 2026-05-13 after adding package-aware module body checking, stable interface artifact identities, package-aware default checking/typed compilation, removal of the legacy interface-stub typed path, an initial MIR bytecode boundary, slot-backed runtime locals keyed by lowered local identity, and explicit `.mgb` implementation artifacts for artifact-backed package execution.

Purpose: if prior conversation context is lost, read this file after [ROADMAP.md](../ROADMAP.md). It records what the repository currently implements, what was verified, and the concrete test plan for the next code slice.

## V1 Route To Preserve

The current phase is foundation closure for v1, not open-ended compiler idealization. Package-aware checking, persisted interfaces, explicit check artifacts, package-wide typed HIR aggregation, MIR-backed bytecode generation, and slot-backed runtime locals are already in place. The next work should turn that foundation into a complete v1 package workflow.

Recommended order:

1. Close the current MIR/runtime identity foundation at the reference-VM boundary. Do enough to keep execution keyed by lowered local identity and semantic package identity; do not start control-flow MIR or native lowering before v1 unless a concrete artifact-backed execution bug requires it.
2. Harden dependency-body-free package execution. Artifact-backed `check` avoids dependency implementation bodies; artifact-backed `run` now consumes `.mgb` implementation artifacts without dependency source-tree fallback.
3. Keep the artifact root explicit on the CLI. Extend the explicit workflow before adding any `muga.toml` artifact-root configuration.
4. Preserve `.mgi` as the public interface artifact, `.mgc` as the check-cache proof, and `.mgb` as the package implementation artifact. Do not stretch `.mgc` into an executable body store.
5. Keep missing, stale, or hash-mismatched execution artifacts as hard errors under `--artifact-root`; do not silently read dependency source bodies in artifact-backed execution.
6. Continue hardening samples, README/spec notes, and diagnostics. Keep generic records/functions, wildcard enum patterns, `try expr`, native backend work, broad stdlib effects, and full incremental reuse deferred unless one is required to finish the v1 package/artifact path.

## Verification Snapshot

- [x] `cargo fmt --check`, `git diff --check`, `cargo check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test --locked` passed after the latest MIR/runtime identity slice.
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
- [x] blocks and final-expression function bodies.
- [x] `if` statements, `if` expressions, and `while` loops.
- [x] integer overflow and division-by-zero runtime diagnostics.

### Functions And Inference

- [x] named `fn` declarations.
- [x] recursive and mutually recursive functions with the current annotation rules.
- [x] anonymous function expressions.
- [x] closure capture.
- [x] higher-order functions.
- [x] local bidirectional inference for the implemented cases.
- [x] function type annotations with `->`.
- [ ] user-defined generic functions are not implemented.

### Records And Calls

- [x] `record` declarations.
- [x] nominal record literals.
- [x] field access with `expr.name`.
- [x] non-destructive record update with `expr.with(...)`.
- [x] chained UFCS-style calls with `expr.name(...)`.
- [x] package-qualified chained calls with `expr.alias::name(...)`.
- [x] typed HIR preserves direct, chained, qualified chained, builtin, value, and package-item call targets.
- [x] AST, HIR, and typed HIR preserve compiler-known enum match arms as enum-variant-shaped patterns: enum name, variant name, and optional payload binding.

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
- [x] `muga emit-artifacts` writes reachable `.mgb` package implementation artifacts alongside reachable `.mgi` interfaces and the entry `.mgc` check cache.
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
- [x] explicit artifact-backed package execution reads dependency implementation bodies from `.mgb` artifacts and does not fall back to dependency source files.

### Diagnostics

- [x] `Diagnostic` supports primary span, related notes, suggestions, and replacements.
- [x] simple diagnostics still display as one line.
- [x] duplicate declarations can point at previous declarations.
- [x] package visibility diagnostics can point at private declarations and suggest `pkg` or `pub`.
- [x] record literal/update and field diagnostics include declaration-site context in selected cases.
- [x] user enum diagnostics cover unknown enum/variant constructor references, generic expected-type failures, constructor arity, missing arms, duplicate arms, and foreign arms.
- [ ] cross-package diagnostics for persisted interfaces and caches are not implemented.

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
- [x] user-defined enum runtime values use the same generic `EnumValue` display shape.
- [x] typed HIR and in-memory package interface summaries preserve public user enum declarations and public signatures containing user enum types.
- [x] imported package enum constructors and patterns such as `alias::Enum::Variant` are covered.
- [x] public, `pkg`, and module-private enum visibility cases are covered.
- [x] in-memory package interface validation catches stale enum identity, type parameter, variant, and payload mismatches.
- [ ] map literals are deferred.
- [ ] arbitrary map key types are deferred.
- [ ] `Set[T]` is deferred.
- [ ] error propagation syntax is not implemented.

## Architecture Facts To Keep In Mind

- The VM/bytecode path is the current execution backend and should remain a reference backend.
- typed HIR is the semantic boundary for package interfaces and MIR lowering.
- The default compile APIs and bytecode backend now consume an initial expression-shaped MIR with explicit entry/function bodies, body terminators, hoisted body-local function definitions, typed HIR binding/package-item identity, and typed assignment update mode. Bytecode/runtime name references carry optional binding identity, lowered local identity, and display symbols; runtime environments are slot-backed by `LocalId`; and package function item references resolve to the defining function binding at bytecode lowering. MIR is now the only backend-facing IR while it is matured toward a control-flow-oriented backend IR.
- `Option[T]` and `Result[T, E]` remain compiler-known enum-like types for now; user-defined enums use a parallel source-level enum model.
- `match` supports compiler-known `Option[T]` / `Result[T, E]` and user-defined enums; match patterns are represented internally as enum variant patterns.
- Runtime enum-like values use a generic enum-value representation.
- `Map` runtime storage is a simple vector of key/value entries, which is correct for semantics but not a final performance representation.
- Package interfaces now have a deterministic v2 text format with stable artifact package/item IDs and file round-trip helpers.
- Loaded package interface summaries can now act as the downstream dependency boundary for typed checking.
- A library API can discover dependency `.mgi` artifacts from an explicit interface root for typed checking.
- Interface artifacts now record direct dependencies, and artifact discovery follows those dependencies so public signatures can mention types from transitive packages without reading dependency bodies.
- A library API can compute package check cache keys and validate `.mgc` artifacts against source plus loaded dependency interface hashes.
- CLI `check --artifact-root` can consume `.mgi` and `.mgc` artifacts without reading dependency implementation bodies.
- CLI `emit-interface` and `emit-check-cache` can produce the artifacts consumed by `check --artifact-root`, with `.mgc` emission gated by a successful package-aware artifact check.
- CLI `emit-interface` can emit all reachable interfaces without manually naming each dependency package.
- CLI `emit-artifacts` emits reachable `.mgi` interfaces, reachable `.mgb` implementation artifacts, and the entry `.mgc` check cache in one explicit artifact-root workflow.
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
- Project-mode artifact-root config is intentionally deferred until dependency declarations, lockfiles, and a package-aware project driver exist.
- Full incremental artifact reuse is still not implemented.

## Recommended Next Implementation

The next implementation theme is v1 artifact workflow hardening while keeping artifact roots explicit on the CLI. The current package-aware path validates package boundary rules, builds source/module signatures from the unflattened graph, runs module body checks, backs default package `check` and package typed compilation, exposes per-module plus package-wide typed HIR without flattening, and lowers execution through an initial expression-shaped MIR before bytecode generation. Artifact-backed `check` consumes `.mgi` and `.mgc` artifacts without dependency implementation bodies; artifact-backed `run` consumes `.mgi`, `.mgc`, and `.mgb` artifacts without reading dependency implementation source files from the source tree.

Reasoning:

- The package-aware semantic boundary is now real enough for `check`; v1 risk has moved from name resolution/flattening to execution and artifact workflow closure.
- The reference VM now consumes MIR-lowered bytecode with binding/local identity metadata and slot-backed locals, so persisted dependency implementations can be keyed by compiler-owned identity instead of display-name lookup.
- `.mgi` should remain a public-signature artifact, `.mgc` should remain a check-cache proof, and `.mgb` should remain a separate implementation/execution artifact rather than overloading either existing format.
- Artifact-backed `run` fails loudly when required dependency execution artifacts are missing, stale, or inconsistent with loaded interfaces. It should continue to avoid silently falling back to dependency source bodies under `--artifact-root`.
- `muga.toml` should not name an artifact root yet. The manifest currently owns only `[package] name/source`; adding build/cache configuration before dependency declarations and lockfiles would make ordinary project `check` and `run` semantics ambiguous.
- Control-flow MIR, native lowering, broad stdlib effects, `try expr`, wildcard-heavy matching, and generic records/functions should remain out of the v1 path unless they become necessary to make artifact-backed execution correct.

## Requirement Decisions For The Next Slice

Closed before coding artifact-backed execution:

- [x] Keep `.mgi` as the public interface artifact.
- [x] Keep `.mgc` as the check-cache proof keyed by entry source and dependency interface hashes.
- [x] Add `.mgb` as the separate implementation/execution artifact; do not overload `.mgi` or `.mgc` for executable code.
- [x] Keep `--artifact-root` explicit for v1; do not add `muga.toml` artifact-root configuration before dependency declarations, lockfiles, and a package-aware project driver.
- [x] Artifact-backed `run` must reject missing, stale, or mismatched execution artifacts instead of falling back to dependency source bodies.
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
- [x] Prefer `try expr` over postfix `?` if Result propagation sugar is added later.

## Implementation Plan And Estimate

Estimates are in focused engineering days for someone already familiar with this codebase. They include tests and documentation, not just code edits.

| Slice | Scope | Main files | Estimate | Risk |
|---|---|---|---:|---|
| 1. Enum/ADT internal model | Generalize the Option-specific representation into an enum-like internal model, without changing source behavior. AST/typed HIR/MIR pattern shape, runtime enum value shape, compiler-known enum metadata, and generic two-variant bytecode/runtime branching are in place. | `src/typing.rs`, `src/typed_hir.rs`, `src/mir.rs`, `src/bytecode.rs`, `src/runtime.rs`, `tests/examples.rs` | Done | Low |
| 2. `Result[T, E]` standard type | Add compiler-known `Result::Ok`, `Result::Err`, and exhaustive `Result` match. No propagation sugar yet. Reuse the known enum metadata table and generic runtime enum value shape. | `src/known_enum.rs`, `src/parser.rs`, `src/typing.rs`, `src/mir.rs`, `src/bytecode.rs`, `src/runtime.rs`, `src/typed_hir.rs` | Done | Medium |
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
| 17. Dependency-body-free execution | Added an explicit artifact-backed `run` path that validates `.mgi` / `.mgc`, loads separate `.mgb` dependency implementation artifacts, and executes package dependencies without reading source files from the dependency source tree. `emit-artifacts` writes every artifact needed by this path. | `src/main.rs`, `src/lib.rs`, `src/cache.rs`, `src/interface.rs`, `src/implementation_artifact.rs`, tests/docs | Done | Medium |
| 18. V1 package workflow hardening | Document and test the explicit artifact workflow end to end, including broader missing/stale artifact diagnostics, default source-compatible execution, and sample package/project commands. | `README.md`, `ROADMAP.md`, `docs/*`, `tests/examples.rs`, samples | 1-2 days | Medium |
| 19. Error propagation design | Specify `try expr` propagation for `Result`, including exact type rules and desugaring. Keep this post-v1 unless v1 error-handling docs require the syntax decision. | spec docs first, then parser/typechecker/HIR/runtime | Deferred | High |

The safest immediate code slice is now artifact workflow hardening: keep the current expression-shaped MIR/reference VM path, broaden `.mgb` diagnostics and samples, and defer automatic project artifact reuse until dependency declarations and lockfiles exist.

## Test Plan For The Next Code Slice

Tests around these behavioral anchors are now partially covered. Keep expanding them before treating the full artifact workflow as v1-ready.

Artifact-backed execution:

- [x] `emit_artifacts_writes_interface_check_and_execution_artifacts`
- [x] `run_with_artifact_root_executes_dependency_without_dependency_source_body`
- [x] `run_with_artifact_root_rejects_missing_dependency_execution_artifact`
- [x] `run_with_artifact_root_rejects_stale_dependency_execution_artifact`
- [x] `run_with_artifact_root_rejects_execution_artifact_interface_hash_mismatch`
- [x] `run_with_artifact_root_does_not_fall_back_to_dependency_source`
- [ ] `default_run_without_artifact_root_remains_source_compatible`
- [ ] `artifact_backed_run_does_not_change_default_run_behavior`

Already-covered package-aware checking anchors:

- [x] `package_aware_checking_preserves_public_import_resolution`
- [x] `package_aware_checking_rejects_private_cross_package_references`
- [x] `package_aware_checking_reports_package_qualified_type_errors`
- [x] `package_aware_checking_reuses_unflattened_package_graph`
- [x] `package_aware_typed_program_preserves_public_interface_items`
- [x] `package_aware_checking_exposes_module_type_outputs`
- [x] `package_module_typed_hir_lowering_preserves_package_binding_identity`
- [x] `package_aware_checking_can_use_loaded_interface_signatures_without_dependency_source`
- [x] `package_aware_checking_can_use_cached_interface_artifacts_without_dependency_source`
- [x] `package_signature_environment_preserves_same_package_type_identities`
- [x] `package_signature_environment_resolves_imported_public_types`
- [x] `package_signature_environment_preserves_generic_enum_signatures`
- [x] `package_signature_environment_rejects_generic_enum_arity_mismatch`
- [x] `package_module_signature_environment_tracks_module_visibility`
- [x] `package_module_signature_environment_tracks_imported_exports`
- [x] `package_module_typechecking_uses_signature_environment_for_body_errors`
- [x] `artifact_workflow_rejects_missing_artifacts_without_source_fallback`
- [x] `interface_artifact_checking_handles_independently_generated_package_ids`

Compatibility:

- [x] `default_cli_check_accepts_package_entry`
- [x] `artifact_generation_does_not_change_default_run_behavior`

## Definition Of Done For The Next Code Slice

- [ ] Existing `cargo test` remains green.
- [x] Existing package-body checking remains source-compatible.
- [x] No `muga.toml` artifact-root field is added before dependency declarations and lockfiles.
- [x] Artifact-root behavior remains explicit through CLI flags.
- [x] Default CLI checking/execution remains unchanged when no artifact root is provided.
- [x] Artifact-backed package execution can run dependencies from artifacts without reading dependency implementation source files from the source tree.
- [x] Artifact-backed package execution rejects missing, stale, or interface-mismatched execution artifacts without source fallback.
- [x] Docs are updated in `README.md`, `ROADMAP.md`, relevant `spec/*.md`, and this file.

## Resume Checklist

When resuming implementation:

1. [ ] Run `cargo test`.
2. [ ] Read this file.
3. [ ] Read [ROADMAP.md](../ROADMAP.md).
4. [ ] Read [docs/internal/identity-model.md](internal/identity-model.md) before changing resolver/typechecker/HIR/MIR/runtime identity flow.
5. [ ] Keep artifact roots explicit on the CLI; do not add `muga.toml` artifact-root config until dependency declarations and lockfiles exist.
6. [ ] Do not reintroduce flattened AST/HIR or dependency source-body lookup as the long-term package boundary.
7. [ ] After every compiler-core change, verify at least:

```bash
cargo test
target/debug/muga check samples/println_sum.muga
target/debug/muga samples/println_sum.muga
target/debug/muga samples/packages/app/main/main.muga
target/debug/muga samples/projects/my_service/src/main/main.muga
```
