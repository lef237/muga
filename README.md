# Muga

"Muga" is a Japanese term meaning "selflessness" or "transcendence of self," referring to a state of being beyond personal limitations or free from self-centered thinking.

This programming language borrows that idea for a small language focused on simple rules, readable code, and low syntactic overhead.

This repository currently contains a v1 specification draft and an early Rust implementation.

## Installation

Install the published command with Cargo:

```bash
cargo install muga
```

Run a source file with:

```bash
muga path/to/file.muga
muga check path/to/file.muga
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

Run your own file by pointing `cargo run` at any `.muga` source. `run` is the default subcommand, so it can be omitted:

```bash
cargo run -- run path/to/file.muga
cargo run -- path/to/file.muga
```

A minimal program needs a zero-argument `main()` — its return value is printed after execution:

```muga
fn main(): Int {
  println(1 + 2)
}
```

For more entry points, browse the [Samples](#samples) section below.

## Current Direction

- no `let`
- immutable by default
- `mut` introduces mutable bindings
- `x = e` creates a new immutable binding when `x` is undefined in the current scope
- `x = e` updates an existing mutable binding when `x` already resolves to a mutable name in the current scope
- `x = e` is an error when `x` already resolves to an immutable name in the current scope
- shadowing is prohibited
- an inner block in the same function may update an enclosing mutable binding
- updating an outer-scope binding across a function boundary is prohibited
- type annotations are omitted by default and only required when inference is not possible
- statements are separated by newlines and comments use `//`
- source-level type annotations may use `Int`, `Bool`, `String`, nominal record types, and function types such as `A -> B`
- type inference is local-only
- type inference is locally bidirectional inside one function body, including some higher-order parameters
- Muga does not introduce classes; data uses `record`, behavior uses functions, and method-like calls are surface syntax
- receiver-style functions use a record type as the first parameter, and `self` is only a conventional parameter name
- `expr.name` is field access, while `expr.name(...)` and `expr.alias::name(...)` are chained calls
- `record.with(field: expr, ...)` is a record-only non-destructive update
- records use nominal data declarations together with record literals
- record fields may not have function type
- higher-order functions are allowed
- function types use `->`
- v1 generics are drafted for generic type expressions, generic records, and generic functions
- v1 does not introduce trait, interface, protocol, typeclass, or overloaded dispatch declarations
- collection design is drafted around `List[T]` first, then `Option[T]` and `Map[K, V]`
- source-level values use value semantics; the implementation may share immutable storage internally when that is not observable
- explicit source-level references such as `ref T`, `mut ref T`, and `&value` are not planned for ordinary Muga code
- write-oriented APIs should prefer value-returning updates, builder/buffer types, or resource handles

## Documentation

- Canonical draft: [mini-language-spec-v1.md](./mini-language-spec-v1.md)
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
- Error catalog: [errors.md](./errors.md)
- Implementation roadmap: [ROADMAP.md](./ROADMAP.md)
- Current next steps: [docs/current-next-steps.md](./docs/current-next-steps.md)
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

- parsing, name resolution, type checking, HIR lowering, bytecode compilation, and the VM runtime are being implemented
- HIR and bytecode names are managed through symbol interning
- `check` only validates the front end
- `run` passes through the front end, lowers to HIR, compiles to bytecode, and executes the result
- `run` prints the return value when a zero-argument `main()` exists
- `print` and `println` are available as prelude builtins
- `print(x)` writes `Int`, `Bool`, or `String` without a trailing newline and returns the same value
- `println(x)` writes `Int`, `Bool`, or `String` with a trailing newline and returns the same value
- `record`, field access, `record.with` update, chained UFCS-style calls, and arrow function type annotations are implemented
- local bidirectional inference for some higher-order parameters and anonymous functions is implemented
- file-based package mode with `package`, `import`, `pkg`, `pub`, module-private top-level items, and `alias::Name` is implemented
- minimal `muga.toml` project mode with `[package] name/source` and inferred package paths is implemented
- current package implementation still requires fully annotated `pub fn`; the design direction is to allow inferred public signatures once package interfaces can store them
- generics, generic collection types, list literals, `Option[T]`, and `Map[K, V]` are design drafts and not implemented yet
- explicit source-level references, mutable references, and explicit dereference syntax are not planned for ordinary Muga code
- typed HIR preserves resolved call shape and ordinary/chained/package-qualified call origin
- dependency declarations, registries, package interfaces, and package caching are not implemented yet

## Planned Priority

The current recommended implementation order is:

1. start measuring compile-time costs and keep benchmarking throughout the compiler work
2. expose resolver/typechecker identity data for typed HIR
3. fix package-aware symbol identity, then introduce typed HIR
4. replace package flattening with real package interfaces and caching
5. split compiler MIR from the VM path, then add a fast native backend

The detailed breakdown lives in [ROADMAP.md](./ROADMAP.md).

```bash
cargo run -- check path/to/file.muga
cargo run -- run path/to/file.muga
```

`run` can be omitted:

```bash
cargo run -- path/to/file.muga
```

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
- [samples/projects/my_service/src/main/main.muga](./samples/projects/my_service/src/main/main.muga) (runnable manifest project sample where package declarations are inferred from `muga.toml` and directories)

Planned concurrency draft samples:

- [samples/planned_concurrency_group.muga](./samples/planned_concurrency_group.muga) (recommended Phase 1 direction: `group` / `spawn` / `join`)
- [samples/planned_concurrency_channels.muga](./samples/planned_concurrency_channels.muga) (later-phase extension after the structured task core is stable)

Sample note:

- In [samples/mixed_chain_pipeline.muga](./samples/mixed_chain_pipeline.muga), `10.start().inc().inc().value.double()` has the same meaning as `double(inc(inc(start(10))).value)`. Both chain style and ordinary call style are valid.

Higher-order annotation guide:

- Omit an arrow annotation when the callback type is uniquely determined inside the same function body, as in [samples/higher_order_functions.muga](./samples/higher_order_functions.muga) and [samples/higher_order_local_inference.muga](./samples/higher_order_local_inference.muga).
- Keep an arrow annotation when local inference is still ambiguous, or when you want the callback contract to be obvious at the declaration site, as in [samples/higher_order_explicit_arrow.muga](./samples/higher_order_explicit_arrow.muga).
- In the package design, `pub fn` should also be inference-first when its public signature is uniquely inferable. The generated package interface stores the resolved signature so downstream packages can stay fast without rechecking dependency bodies.

Package alias note:

- `import company::analytics::numbers` gives the default local alias `numbers`.
- If two imports would produce the same alias, the file is rejected with `PK007`.
- Use `as` to disambiguate, as shown in [samples/packages/app/alias_demo/main.muga](./samples/packages/app/alias_demo/main.muga).

Package layout note:

- Muga's package draft uses `directory = package` and `file = module`.
- Source files import logical package paths such as `my_service::users`, not filesystem paths such as `../users`.
- In manifest project mode, `name = "my_service"` and `source = "src"` let `src/users/` map to `my_service::users` without nesting another `my_service/` directory under `src/`.
- The future distribution model is manifest-based and should use cached package interfaces for fast rebuilds.
- See [spec/006-packages.md](./spec/006-packages.md) for the large-project layout and distribution model.

## License

Licensed under the [MIT License](./LICENSE.txt).
