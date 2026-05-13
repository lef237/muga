# Implementation Resume Plan

Status: current implementation ledger for 2026-05-13.

Purpose: if prior conversation context is lost, read this file after [ROADMAP.md](../ROADMAP.md). It records what the repository currently implements, what was verified, and the concrete test plan for the next code slice.

## Verification Snapshot

- [x] `cargo test` passed after transitive interface artifact reuse support: 226 tests, 0 failures.
- [x] `cargo clippy --all-targets -- -D warnings` passed after transitive interface artifact reuse support.
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
- [x] package enum constructor call targets carry enum `PackageItemId` when the enum comes from package mode.
- [x] package interfaces have a deterministic v1 text format with file write/read helpers.
- [x] persisted package interface round-trip preserves direct dependency metadata, public records, functions, enums, `TypeInfo`, item identity, enum variants, and payload types.
- [x] persisted package interfaces include deterministic content hashes and reject hash mismatches.
- [x] package interface artifact path naming is deterministic for package paths.
- [x] typed package compilation can validate against loaded package interface summaries.
- [x] loaded package interfaces can be used as the dependency boundary for downstream typed checking without reading dependency implementation bodies.
- [x] package interface artifacts can be discovered from an explicit interface root for downstream typed checking.
- [x] interface artifact discovery follows transitive `.mgi` dependencies needed by public signatures.
- [x] missing and hash-mismatched interface artifacts are rejected with regeneration guidance.
- [x] package check cache keys include entry package source hashes and loaded direct/transitive dependency interface hashes.
- [x] missing or stale `.mgc` package check artifacts are rejected with regeneration guidance.
- [x] `muga check --artifact-root <dir>` consumes `.mgi` and `.mgc` artifacts without reading dependency implementation bodies.
- [x] `muga emit-interface` and `muga emit-check-cache` write `.mgi` and `.mgc` artifacts for explicit artifact-backed checks.
- [x] `muga emit-interface` emits all reachable package interfaces when `--package` is omitted, or one selected package when `--package` is supplied.
- [ ] default CLI checking/execution still uses package flattening and dependency source loading.

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
- typed HIR is the intended semantic boundary for future MIR, package interfaces, and native backend work.
- The current bytecode backend still lowers from the older untyped HIR, not typed HIR.
- `Option[T]` and `Result[T, E]` remain compiler-known enum-like types for now; user-defined enums use a parallel source-level enum model.
- `match` supports compiler-known `Option[T]` / `Result[T, E]` and user-defined enums; match patterns are represented internally as enum variant patterns.
- Runtime enum-like values use a generic enum-value representation.
- `Map` runtime storage is a simple vector of key/value entries, which is correct for semantics but not a final performance representation.
- Package interfaces now have a deterministic v1 text format and file round-trip helpers.
- Loaded package interface summaries can now act as the downstream dependency boundary for typed checking.
- A library API can discover dependency `.mgi` artifacts from an explicit interface root for typed checking.
- Interface artifacts now record direct dependencies, and artifact discovery follows those dependencies so public signatures can mention types from transitive packages without reading dependency bodies.
- A library API can compute package check cache keys and validate `.mgc` artifacts against source plus loaded dependency interface hashes.
- CLI `check --artifact-root` can consume `.mgi` and `.mgc` artifacts without reading dependency implementation bodies.
- CLI `emit-interface` and `emit-check-cache` can produce the artifacts consumed by `check --artifact-root`.
- CLI `emit-interface` can emit all reachable interfaces without manually naming each dependency package.
- CLI `emit-artifacts` emits reachable `.mgi` interfaces and the entry `.mgc` check cache in one explicit artifact-root workflow.
- Default CLI package checking and execution still read and flatten dependency bodies.
- Project-mode artifact-root config is intentionally deferred until dependency declarations, lockfiles, and a package-aware project driver exist.
- Full incremental artifact reuse and package-aware checking without flattening are still not implemented.

## Recommended Next Implementation

The next implementation theme is package-aware checking without flattening while keeping artifact roots explicit on the CLI.

Reasoning:

- `List[T]`, `Option[T]`, and `Map[K, V]` now cover the first collection slice.
- `Map.get` and `List.get` already depend on `Option[T]`, so the compiler has a working enum-like subset.
- `Result[T, E]` now proves the same enum metadata, runtime value, typed HIR, and package-interface path for a second generic enum.
- General IO, process, HTTP, time, and concurrency APIs can use explicit `Result` before any propagation sugar is added.
- User-defined enum declarations, runtime values, typed HIR, and in-memory public interface summaries are now represented.
- Enum integration hardening now covers diagnostics, imported constructors/patterns, visibility errors, package enum call-target identity, and stale interface validation.
- Persisted package interfaces now round-trip record/function/enum identity, type parameters, variants, payload types, public signatures, and source spans.
- Loaded package interfaces can now be used for downstream signature/type checking without dependency implementation bodies.
- Interface artifacts can now be discovered from an explicit root, with missing/hash-mismatched artifacts rejected before checking.
- Interface artifacts now persist direct dependency paths, and artifact loading follows those paths for transitive public-signature type dependencies.
- Package check cache keys now include entry source content and loaded direct/transitive dependency interface hashes.
- CLI artifact-backed checking can now consume existing `.mgi` and `.mgc` artifacts.
- CLI artifact generation can now produce `.mgi` and `.mgc` for the explicit workflow.
- `muga emit-artifacts` now combines reachable interface emission and entry check-cache emission.
- `muga.toml` should not name an artifact root yet. The manifest currently owns only `[package] name/source`; adding build/cache configuration before dependency declarations and lockfiles would make ordinary project `check` semantics ambiguous.
- The remaining boundary pieces are real artifact storage/reuse, dependency/lockfile-driven project configuration, and eventually making interface-backed checking the normal package path.

## Requirement Decisions For The Next Slice

Closed before coding the next feature:

- [x] `Result[T, E]` lands first as a compiler-known enum-like standard type.
- [x] The enum declaration syntax is `enum Name[T, E] { Variant | Variant(Type) }`.
- [x] Variant declarations use newline or comma boundaries, matching record fields.
- [x] The MVP supports zero-payload and one-payload variants only.
- [x] Multi-field variants and named-field variants are deferred.
- [x] Variant constructors and patterns are always qualified as `Enum::Variant`.
- [x] Match patterns must be exhaustive with no wildcard in the MVP.
- [x] Package-mode enum declarations use `PackageItemId`.
- [x] Public enum declarations appear in in-memory package interface summaries.
- [x] Prefer `try expr` over postfix `?` if Result propagation sugar is added.

Current recommendation:

- Use `enum` as the declaration keyword with optional unconstrained type parameters.
- Keep variant names qualified in expressions and patterns, matching current `Option::Some` and `Option::None`.
- MVP variants should support zero or one unnamed payload. This covers `Option[T]` and `Result[T, E]`.
- Keep wildcard patterns deferred until basic exhaustiveness diagnostics are solid.
- Keep `Option[T]` canonical; keep `T?` reserved and unimplemented.
- Keep explicit `Result[T, E]` construction and `match` as the baseline before propagation sugar.
- If propagation sugar is added, prefer `try expr` because it is visible and avoids overloading `?`.
- Avoid adding broad stdlib effects until `Result` behavior is stable.

Recommended source shape:

```muga
enum Result[T, E] {
  Ok(T)
  Err(E)
}

fn value_or_zero(value: Result[Int, String]): Int {
  match value {
    Result::Ok(x) => x
    Result::Err(message) => 0
  }
}
```

## Implementation Plan And Estimate

Estimates are in focused engineering days for someone already familiar with this codebase. They include tests and documentation, not just code edits.

| Slice | Scope | Main files | Estimate | Risk |
|---|---|---|---:|---|
| 1. Enum/ADT internal model | Generalize the Option-specific representation into an enum-like internal model, without changing source behavior. AST/HIR/typed HIR pattern shape, runtime enum value shape, compiler-known enum metadata, and generic two-variant bytecode/runtime branching are in place. | `src/typing.rs`, `src/typed_hir.rs`, `src/hir.rs`, `src/bytecode.rs`, `src/runtime.rs`, `tests/examples.rs` | Done | Low |
| 2. `Result[T, E]` standard type | Add compiler-known `Result::Ok`, `Result::Err`, and exhaustive `Result` match. No propagation sugar yet. Reuse the known enum metadata table and generic runtime enum value shape. | `src/known_enum.rs`, `src/parser.rs`, `src/typing.rs`, `src/hir.rs`, `src/bytecode.rs`, `src/runtime.rs`, `src/typed_hir.rs` | Done | Medium |
| 3. Enum declaration syntax MVP | Parse and typecheck user-defined enum declarations with optional unconstrained type parameters and zero/one-payload variants. Add runtime representation and typed HIR/interface summaries. | parser/AST/typechecker/HIR/bytecode/runtime/package/typed HIR/tests | Done | High |
| 4. Enum integration hardening | Expand diagnostics, package visibility cases, interface stale checks, and compatibility coverage after the MVP is green. | package/interface/typed HIR/tests/docs | Done | Medium |
| 5. Package interface persistence format | Serialize public records/functions/enums and resolved type identities in a deterministic v1 text format. Load the format back into `PackageInterfaceGraph` and validate the reloaded summaries. | `src/interface.rs`, `tests/examples.rs` | Done | Medium |
| 6. Interface hashes and loaded-interface validation | Add interface hashes, artifact path conventions, and a typed checking path that validates against loaded interface summaries. | `src/interface.rs`, `src/lib.rs`, tests | Done | Medium |
| 7. Downstream checking without dependency bodies | Load dependency interfaces as the checking boundary, synthesize or otherwise expose only public signatures, and avoid reading dependency implementation bodies for downstream checks. | `src/package.rs`, `src/interface.rs`, `src/lib.rs`, tests | Done | High |
| 8. Interface artifact discovery | Teach package checking to find persisted interface artifacts from an explicit interface root and reject missing/hash-mismatched/stale artifacts. | `src/interface.rs`, `src/package.rs`, `src/lib.rs`, tests | Done | High |
| 9. Package cache keys and invalidation | Define source/interface/dependency hash inputs, persist checked-package metadata, reject missing/stale cache artifacts, and keep cache-backed checking aligned with body checking. | `src/cache.rs`, `src/package.rs`, `src/lib.rs`, tests | Done | High |
| 10. CLI artifact-root checking | Expose a narrow CLI path for artifact-backed checking using `.mgi` and `.mgc` artifacts. | `src/main.rs`, `src/lib.rs`, tests/docs | Done | Medium |
| 11. CLI artifact generation | Add CLI/library artifact generation for `.mgi` and `.mgc`, and verify generated artifacts drive `check --artifact-root`. | `src/main.rs`, `src/lib.rs`, `src/interface.rs`, tests/docs | Done | Medium |
| 12. Combined artifact emission | Keep artifact roots explicit on the CLI and add `emit-artifacts` to write reachable `.mgi` plus entry `.mgc` in one command. | `src/main.rs`, `src/lib.rs`, tests/docs | Done | Low |
| 13. Transitive interface artifact reuse | Persist direct dependencies in `.mgi`, load transitive public-signature type interfaces, and include the loaded interface set in `.mgc` keys. | `src/interface.rs`, `src/package.rs`, `src/cache.rs`, tests/docs | Done | High |
| 14. Package-aware checking without flattening | Start replacing package flattening with package-aware checking boundaries while keeping artifact semantics explicit. | package/resolver/typing/lib/tests | 4-8 days | High |
| 15. Error propagation design | Specify `try expr` propagation for `Result`, including exact type rules and desugaring. Implement only after user-defined enum identity is stable. | spec docs first, then parser/typechecker/HIR/runtime | 2-4 days | High |

The safest immediate code slice is now Slice 14: begin moving package checking away from flattening while preserving the explicit `.mgi` / `.mgc` workflow. This may require a design checkpoint before larger edits.

## Test Plan For The Next Code Slice

Add tests around these behavioral anchors before enabling artifact-backed package checking by default.

Package-aware checking:

- `package_aware_checking_preserves_public_import_resolution`
- `package_aware_checking_rejects_private_cross_package_references`
- `package_aware_checking_reports_package_qualified_type_errors`
- `artifact_workflow_rejects_missing_or_stale_artifacts_without_source_fallback`

Compatibility:

- `default_cli_check_keeps_existing_body_based_behavior`
- `artifact_generation_does_not_change_default_run_behavior`

## Definition Of Done For The Next Code Slice

- [ ] Existing `cargo test` remains green.
- [ ] Existing package-body checking remains source-compatible.
- [ ] No `muga.toml` artifact-root field is added before dependency declarations and lockfiles.
- [ ] Artifact-root behavior remains explicit through CLI flags.
- [ ] Default CLI checking/execution remains unchanged when no artifact root is provided.
- [ ] Docs are updated in `README.md`, `ROADMAP.md`, relevant `spec/*.md`, and this file.

## Resume Checklist

When resuming implementation:

1. [ ] Run `cargo test`.
2. [ ] Read this file.
3. [ ] Read [ROADMAP.md](../ROADMAP.md).
4. [ ] Read [spec/013-enums-results.md](../spec/013-enums-results.md).
5. [ ] Keep artifact roots explicit on the CLI; do not add `muga.toml` artifact-root config until dependency declarations and lockfiles exist.
6. [ ] Keep package flattening unchanged for normal execution unless the task explicitly changes package checking.
7. [ ] After every compiler-core change, verify at least:

```bash
cargo test
target/debug/muga check samples/println_sum.muga
target/debug/muga samples/println_sum.muga
target/debug/muga samples/packages/app/main/main.muga
target/debug/muga samples/projects/my_service/src/main/main.muga
```
