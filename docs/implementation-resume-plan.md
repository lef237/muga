# Implementation Resume Plan

Status: verified resume snapshot for 2026-05-12.

Purpose: if prior conversation context is lost, read this file first. It records what the repository currently implements, what was verified, and what should be implemented next.

## Verification Snapshot

- [x] `git status --short --branch` showed a clean worktree on `main...origin/main` before the enum metadata implementation.
- [x] `cargo test` passed after compiler-known `Result[T, E]` support: 159 tests, 0 failures.
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
- [x] public export lookup through `PackageExportGraph`.
- [x] in-memory package interface summaries for public records and functions.
- [x] interface summaries preserve public `TypeInfo`, package record identity, collection types, and compiler-known `Result` signatures.
- [x] typed package compilation validates references against generated in-memory interfaces.
- [ ] persisted package interface files are not implemented.
- [ ] downstream package checking does not yet consume stored interface artifacts.
- [ ] package flattening is still the execution/checking pipeline.

Important transition detail:

- typed HIR carries `PackageItemId` for package identifiers, calls, and package record types, but part of the lowering path still recovers this identity from flattened mangled names. This is acceptable for the current flattened backend, but it should be removed before package flattening is replaced.

### Diagnostics

- [x] `Diagnostic` supports primary span, related notes, suggestions, and replacements.
- [x] simple diagnostics still display as one line.
- [x] duplicate declarations can point at previous declarations.
- [x] package visibility diagnostics can point at private declarations and suggest `pkg` or `pub`.
- [x] record literal/update and field diagnostics include declaration-site context in selected cases.
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
- [ ] map literals are deferred.
- [ ] arbitrary map key types are deferred.
- [ ] `Set[T]` is deferred.
- [ ] user-defined enum/sum types are not implemented.
- [ ] error propagation syntax is not implemented.

## Architecture Facts To Keep In Mind

- The VM/bytecode path is the current execution backend and should remain a reference backend.
- typed HIR is the intended semantic boundary for future MIR, package interfaces, and native backend work.
- The current bytecode backend still lowers from the older untyped HIR, not typed HIR.
- `Option[T]` and `Result[T, E]` are implemented as compiler-known enum-like types rather than as user-defined enums.
- `match` currently supports compiler-known `Option[T]` and `Result[T, E]`; match patterns are represented internally as enum variant patterns.
- Runtime `Option` and `Result` values use a generic enum-value representation; typechecking and bytecode branching consume compiler-known enum metadata.
- `Map` runtime storage is a simple vector of key/value entries, which is correct for semantics but not a final performance representation.
- The public package interface model is in memory only; there is no serialized format, cache key, or incremental invalidation yet.

## Recommended Next Implementation

The next implementation theme is user-defined enum/sum-type declarations, before broad stdlib or persisted package-interface work.

Reasoning:

- `List[T]`, `Option[T]`, and `Map[K, V]` now cover the first collection slice.
- `Map.get` and `List.get` already depend on `Option[T]`, so the compiler has a working enum-like subset.
- `Result[T, E]` now proves the same enum metadata, runtime value, typed HIR, and package-interface path for a second generic enum.
- General IO, process, HTTP, time, and concurrency APIs can use explicit `Result` before any propagation sugar is added.
- Persisted package interfaces should not be frozen before user-defined enum identities and public enum declarations are represented.

## Requirement Decisions For The Next Slice

Decide these before coding the next feature:

- [x] `Result[T, E]` lands first as a compiler-known enum-like standard type.
- [ ] The exact enum declaration syntax.
- [ ] Whether the MVP supports only zero-payload and one-payload variants, or also multi-field/named-field variants.
- [ ] Whether variant constructors are always qualified as `Enum::Variant`.
- [ ] Whether match patterns must be exhaustive with no wildcard in the MVP.
- [ ] How enum identities are represented across local declarations and packages.
- [ ] How enum declarations appear in package interface summaries.
- [x] Prefer `try expr` over postfix `?` if Result propagation sugar is added.

Current recommendation:

- Use `enum` as the declaration keyword.
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
| 3. Enum declaration syntax MVP | Parse and typecheck user-defined enum declarations with zero/one-payload variants. Add runtime representation and typed HIR/interface summaries. | parser/AST/typechecker/HIR/bytecode/runtime/package/typed HIR/tests | 3-5 days | High |
| 4. Generic enum declarations | Type parameters on enum declarations and variant constructors. This may share work with generic records/functions. | parser/AST/typechecker/package/typed HIR/tests | 3-6 days | High |
| 5. Error propagation design | Specify `try expr` propagation for `Result`, including exact type rules and desugaring. Implement only after user-defined enum identity is stable. | spec docs first, then parser/typechecker/HIR/runtime | 2-4 days | High |
| 6. Package interface persistence | Serialize public records/functions/enums, type identities, and hashes. Start consuming stored summaries for downstream checking. | `src/package.rs`, `src/typed_hir.rs`, new interface/cache modules, tests | 5-10 days | High |

The safest immediate code slice is now Slice 3: add user-defined enum declaration syntax for zero-payload and one-payload variants, then route those declarations into the same match/type/runtime representation currently used by compiler-known `Option` and `Result`.

## Definition Of Done For The Next Code Slice

- [ ] Existing `cargo test` remains green.
- [ ] Existing Option, List, Map, package, and typed HIR behavior remains source-compatible.
- [ ] New user-defined enum behavior is covered in parser, typechecker, runtime, typed HIR, and package-interface tests as applicable.
- [ ] Exhaustiveness diagnostics include missing and duplicate variant coverage.
- [ ] Public signatures containing user-defined enum types are represented in package interface summaries.
- [ ] Docs are updated in `README.md`, `ROADMAP.md`, relevant `spec/*.md`, and this file.

## Resume Checklist

When resuming implementation:

1. [ ] Run `cargo test`.
2. [ ] Read this file.
3. [ ] Read [docs/current-next-steps.md](./current-next-steps.md).
4. [ ] Read [spec/013-enums-results.md](../spec/013-enums-results.md).
5. [ ] Confirm whether the intended next code slice is user-defined enum declaration MVP or a package-interface task.
6. [ ] Keep package flattening unchanged unless the task is explicitly package-interface persistence.
7. [ ] After every compiler-core change, verify at least:

```bash
cargo test
target/debug/muga samples/println_sum.muga
target/debug/muga samples/packages/app/main/main.muga
target/debug/muga samples/projects/my_service/src/main/main.muga
```
