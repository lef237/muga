# Muga

Muga is a compiler-first programming language project named after the Japanese idea of muga, often translated as selflessness. Its current design emphasizes readable local reasoning through immutable-by-default bindings, local type inference, value semantics, records plus functions, and predictable package boundaries.

This repository tracks the language design, examples, and the current Rust compiler/runtime implementation as Muga moves toward v1.

## Installation

Install the published command with Cargo:

```bash
cargo install muga
```

Run a source file with:

```bash
muga path/to/file.muga
muga check path/to/file.muga
muga emit-interface --artifact-root path/to/artifacts --package util::numbers path/to/package/main.muga
muga emit-check-cache --artifact-root path/to/artifacts path/to/package/main.muga
muga check --artifact-root path/to/artifacts path/to/package/main.muga
```

## Quickstart

Prerequisites: a recent Rust toolchain (edition 2024, so Rust 1.85 or later).

Clone the repository and run one of the bundled samples:

```bash
git clone https://github.com/lef237/muga.git
cd muga
cargo run -- samples/println_sum.muga
```

Expected output (the first line is `println`, the second line is the return value of `main`):

```text
10
10
```

Try another sample that chains function calls:

```bash
cargo run -- samples/number_chain.muga
# => 4
```

Only validate the front end (parse, name resolution, typing) without executing:

```bash
cargo run -- check samples/println_sum.muga
# => ok
```

Package mode is also available through a file entrypoint:

```bash
cargo run -- check samples/packages/app/main/main.muga
cargo run -- samples/packages/app/main/main.muga
```

For artifact-backed package checking, first emit dependency `.mgi` interface files, then emit the entry package `.mgc` check cache file:

```bash
cargo run -- emit-interface --artifact-root path/to/artifacts --package util::numbers samples/packages/app/main/main.muga
cargo run -- emit-check-cache --artifact-root path/to/artifacts path/to/package/main.muga
cargo run -- check --artifact-root path/to/artifacts path/to/package/main.muga
```

Run your own file by pointing `cargo run` at any `.muga` source. `run` is the default subcommand, so it can be omitted:

```bash
cargo run -- run path/to/file.muga
cargo run -- path/to/file.muga
```

A program may either define a zero-argument `main()` or run top-level statements directly. When `main()` exists, its return value is printed after execution:

```muga
fn main(): Int {
  println(1 + 2)
}
```

For more entry points, browse the [Samples](#samples) section below.

## Language Shape

- no `let`; bindings are immutable by default and `mut` opts into mutation
- `x = e` is resolved statically as either a new immutable binding or an update to an existing mutable binding
- shadowing and mutation across function boundaries are rejected
- type inference is local-first; annotations are required only when inference is ambiguous or intentionally bounded by the current implementation
- comments use `//`; statements are newline-separated
- data uses nominal `record` declarations; behavior uses functions
- `expr.name` is field access, `expr.name(...)` is chained-call syntax, and `expr.with(...)` is record update
- classes, inheritance, function-valued record fields, traits, protocols, typeclasses, overloaded dispatch, and ordinary source-level references are out of scope for v1
- source values use value semantics; the implementation may share immutable storage internally when that is not observable
- recoverable failures use explicit `Result[T, E]`; possible future propagation sugar is documented as `try expr`, not postfix `?`

## Documentation

- Language overview: [mini-language-spec-v1.md](./mini-language-spec-v1.md)
- Split specification:
  - [spec/001-core-language.md](./spec/001-core-language.md)
  - [spec/002-name-resolution.md](./spec/002-name-resolution.md)
  - [spec/003-typing.md](./spec/003-typing.md)
  - [spec/004-functions.md](./spec/004-functions.md)
  - [spec/005-records.md](./spec/005-records.md)
  - [spec/006-packages.md](./spec/006-packages.md) (draft)
  - [spec/007-concurrency-draft.md](./spec/007-concurrency-draft.md) (draft)
  - [spec/008-collections.md](./spec/008-collections.md) (draft)
  - [spec/009-generics.md](./spec/009-generics.md) (draft)
  - [spec/010-references-draft.md](./spec/010-references-draft.md) (decision note)
  - [spec/011-value-semantics.md](./spec/011-value-semantics.md) (draft)
  - [spec/012-protocols-deferred.md](./spec/012-protocols-deferred.md) (decision note)
  - [spec/013-enums-results.md](./spec/013-enums-results.md) (draft)
- Error catalog: [errors.md](./errors.md)
- Implementation roadmap and next priority: [ROADMAP.md](./ROADMAP.md)
- Implementation ledger, resume checklist, and next-slice test plan: [docs/implementation-resume-plan.md](./docs/implementation-resume-plan.md)
- Ideal compiler architecture: [docs/ideal-compiler-architecture.md](./docs/ideal-compiler-architecture.md)
- Language design reference: [docs/language-design-reference.md](./docs/language-design-reference.md)
- Syntax marker case study: [docs/syntax-marker-case-study.md](./docs/syntax-marker-case-study.md)
- Compiler identity note: [docs/internal/identity-model.md](./docs/internal/identity-model.md)

## Examples

### Valid

- [examples/valid/001-basic-bindings.md](./examples/valid/001-basic-bindings.md)
- [examples/valid/002-read-from-outer-scope.md](./examples/valid/002-read-from-outer-scope.md)
- [examples/valid/003-local-mutable-loop.md](./examples/valid/003-local-mutable-loop.md)
- [examples/valid/004-inferred-parameter-type.md](./examples/valid/004-inferred-parameter-type.md)
- [examples/valid/005-recursive-function.md](./examples/valid/005-recursive-function.md)
- [examples/valid/006-mutual-recursion.md](./examples/valid/006-mutual-recursion.md)
- [examples/valid/007-record-with-update.md](./examples/valid/007-record-with-update.md)
- [examples/valid/008-local-higher-order-inference.md](./examples/valid/008-local-higher-order-inference.md)
- [examples/valid/009-explicit-arrow-callback.md](./examples/valid/009-explicit-arrow-callback.md)

### Invalid

- [examples/invalid/001-immutable-update.md](./examples/invalid/001-immutable-update.md)
- [examples/invalid/002-duplicate-mutable-binding.md](./examples/invalid/002-duplicate-mutable-binding.md)
- [examples/invalid/003-shadowing-in-block.md](./examples/invalid/003-shadowing-in-block.md)
- [examples/invalid/004-outer-scope-mutation.md](./examples/invalid/004-outer-scope-mutation.md)
- [examples/invalid/005-ambiguous-identity.md](./examples/invalid/005-ambiguous-identity.md)
- [examples/invalid/006-unannotated-recursion.md](./examples/invalid/006-unannotated-recursion.md)
- [examples/invalid/007-unannotated-mutual-recursion.md](./examples/invalid/007-unannotated-mutual-recursion.md)
- [examples/invalid/008-invalid-record-update.md](./examples/invalid/008-invalid-record-update.md)
- [examples/invalid/009-ambiguous-higher-order-parameter.md](./examples/invalid/009-ambiguous-higher-order-parameter.md)
- [examples/invalid/010-ambiguous-println-callback.md](./examples/invalid/010-ambiguous-println-callback.md)

## Rust Implementation

Implemented:

- lexer, parser, resolver, typechecker, HIR lowering, bytecode compilation, and VM runtime
- `check` for front-end validation and `run` for VM execution
- `print` / `println` prelude builtins for `Int`, `Bool`, and `String`
- records, field access, `record.with(...)`, chained calls, package-qualified chained calls, arrow function types, local binding annotations, and local bidirectional inference for selected higher-order cases
- `List[T]`, `Option[T]`, `Result[T, E]`, and `Map[K, V]` type expressions
- list literals, direct list indexing, `len`, `is_empty`, `push`, `get`, and `set`
- `Option::Some`, `Option::None`, `Result::Ok`, `Result::Err`, and exhaustive `match` for `Option` and `Result`
- user-defined `enum` declarations with optional unconstrained type parameters, zero-payload and one-payload variants, qualified construction/patterns, exhaustive `match`, VM execution, typed HIR, and in-memory package interface summaries
- `Map.empty`, `contains`, `get`, `insert`, and `remove` for `Int`, `Bool`, and `String` keys
- file-based package mode with `package`, `import`, `pkg`, `pub`, `as`, module-private top-level items, and `alias::Name`
- minimal `muga.toml` project mode with `[package] name/source`
- typed HIR with resolved call shape, call origin, expression types, local binding identity, and package item identity
- in-memory package interface summaries for public records/enums/functions plus validation of public package references against those summaries
- hardened enum diagnostics, package enum visibility checks, imported `alias::Enum::Variant` constructors/patterns, and package enum call-target identity
- deterministic v1 package interface text persistence with content hashes, file write/read helpers, artifact path naming, round-trip validation, and loaded-interface validation for public records/enums/functions
- downstream typed checking can use loaded package interfaces or discovered `.mgi` artifacts without reading dependency implementation bodies
- package check cache keys combine entry package source content with dependency interface hashes, and `.mgc` check artifacts are rejected when missing or stale
- `muga check --artifact-root <dir>` validates package entries against `.mgi` and `.mgc` artifacts without reading dependency implementation bodies
- `muga emit-interface` and `muga emit-check-cache` write `.mgi` and `.mgc` artifacts for the explicit artifact-backed package workflow
- structured diagnostics with related notes and suggestions in selected resolver, typechecker, record, and package errors

Not implemented yet:

- user-defined generic records and generic functions
- map literals, `Set[T]`, arbitrary `Map` key types, and broad collection APIs
- public-signature inference for `pub fn`; public functions currently need explicit signatures
- project-mode artifact-root config, dependency declarations, registries, full incremental package artifact reuse, MIR, and native code generation
- error propagation syntax such as `try expr`

## Planned Priority

The next implementation slice is project-mode artifact-root config and full package artifact reuse.

After that, the priority moves to package checking without flattening, package caching, MIR, and native backend work. The detailed breakdown lives in [ROADMAP.md](./ROADMAP.md).

## Samples

- [samples/sum_to.muga](./samples/sum_to.muga)
- [samples/println_sum.muga](./samples/println_sum.muga)
- [samples/inferred_types.muga](./samples/inferred_types.muga) (runnable sample showing that parameter and return type annotations can be omitted when inference succeeds)
- [samples/no_main.muga](./samples/no_main.muga) (runnable sample showing that `main()` is optional — top-level statements run directly)
- [samples/closure_capture.muga](./samples/closure_capture.muga)
- [samples/record_field_access.muga](./samples/record_field_access.muga) (runnable sample for `record` and field access)
- [samples/record_counter_loop.muga](./samples/record_counter_loop.muga) (runnable sample for mutable bindings and `record.with(...)`)
- [samples/nested_record_access.muga](./samples/nested_record_access.muga) (runnable sample for nested record access)
- [samples/record_with_update.muga](./samples/record_with_update.muga) (runnable sample for `record`, field access, and `record.with(...)`)
- [samples/record_user.muga](./samples/record_user.muga) (runnable sample for record declarations, receiver-shaped parameters, and chained calls)
- [samples/method_chain_user.muga](./samples/method_chain_user.muga) (runnable sample for chained UFCS-style calls)
- [samples/number_chain.muga](./samples/number_chain.muga) (runnable sample for chaining plain functions on `Int`)
- [samples/println_chain.muga](./samples/println_chain.muga) (runnable sample for chaining through builtin `println`)
- [samples/print_then_println.muga](./samples/print_then_println.muga) (runnable sample for mixing `print` and `println`)
- [samples/mixed_chain_pipeline.muga](./samples/mixed_chain_pipeline.muga) (runnable sample that mixes UFCS calls, record update, and field access)
- [samples/higher_order_functions.muga](./samples/higher_order_functions.muga) (runnable sample for higher-order functions with minimal annotations)
- [samples/higher_order_local_inference.muga](./samples/higher_order_local_inference.muga) (runnable sample for locally inferred higher-order parameters and anonymous functions)
- [samples/higher_order_explicit_arrow.muga](./samples/higher_order_explicit_arrow.muga) (runnable sample for explicit arrow annotations on callbacks)
- [samples/packages/app/main/main.muga](./samples/packages/app/main/main.muga) (runnable package entrypoint that imports `util::numbers` and `util::users`, and demonstrates `expr.alias::name(...)` chained calls)
- [samples/packages/app/split_main/main.muga](./samples/packages/app/split_main/main.muga) (runnable package sample where the entry package is split across multiple files)
- [samples/packages/app/alias_demo/main.muga](./samples/packages/app/alias_demo/main.muga) (runnable package sample that uses `import ... as ...` to avoid alias collisions)
- [samples/packages/app/enum_demo/main.muga](./samples/packages/app/enum_demo/main.muga) (runnable package sample that exports and consumes a public generic enum)
- [samples/projects/my_service/src/main/main.muga](./samples/projects/my_service/src/main/main.muga) (runnable manifest project sample where package declarations are inferred from `muga.toml` and directories)

Planned concurrency draft samples:

- [samples/planned_concurrency_group.muga](./samples/planned_concurrency_group.muga) (recommended Phase 1 direction: `group` / `spawn` / `join`)
- [samples/planned_concurrency_channels.muga](./samples/planned_concurrency_channels.muga) (later-phase extension after the structured task core is stable)

Sample note:

- In [samples/mixed_chain_pipeline.muga](./samples/mixed_chain_pipeline.muga), `10.start().inc().inc().value.double()` has the same meaning as `double(inc(inc(start(10))).value)`. Both chain style and ordinary call style are valid.

Higher-order annotation guide:

- Omit an arrow annotation when the callback type is uniquely determined inside the same function body, as in [samples/higher_order_functions.muga](./samples/higher_order_functions.muga) and [samples/higher_order_local_inference.muga](./samples/higher_order_local_inference.muga).
- Keep an arrow annotation when local inference is still ambiguous, or when you want the callback contract to be obvious at the declaration site, as in [samples/higher_order_explicit_arrow.muga](./samples/higher_order_explicit_arrow.muga).
- Current `pub fn` declarations require explicit signatures. The design direction is to infer public signatures in the defining package and store resolved signatures in package interfaces.

Package alias note:

- `import company::analytics::numbers` gives the default local alias `numbers`.
- If two imports would produce the same alias, the file is rejected with `PK007`.
- Use `as` to disambiguate, as shown in [samples/packages/app/alias_demo/main.muga](./samples/packages/app/alias_demo/main.muga).

Package layout note:

- Muga's package draft uses `directory = package` and `file = module`.
- Source files import logical package paths such as `my_service::users`, not filesystem paths such as `../users`.
- In manifest project mode, `name = "my_service"` and `source = "src"` let `src/users/` map to `my_service::users` without nesting another `my_service/` directory under `src/`.
- Without a nearby `muga.toml`, a package file must start with an explicit `package ...` declaration before it can use `import`, `pub`, or `pkg`.
- The target distribution model is manifest-based and should use cached package interfaces for fast rebuilds. The compiler library and CLI can emit and consume `.mgi` and `.mgc` artifacts for explicit artifact-backed checks, but project-mode artifact-root config and automatic artifact reuse are not implemented yet.
- See [spec/006-packages.md](./spec/006-packages.md) for the large-project layout and distribution model.

## License

Licensed under the [MIT License](./LICENSE.txt).
