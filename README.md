# muga

"Muga" is a Japanese term meaning "selflessness" or "transcendence of self," referring to a state of being beyond personal limitations or free from self-centered thinking.

This programming language borrows that idea for a small language focused on simple rules, readable code, and low syntactic overhead.

This repository currently contains a v1 specification draft and an early Rust implementation.

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
# => 10
```

Only validate the front end (parse, name resolution, typing) without executing:

```bash
cargo run -- check samples/println_sum.muga
# => ok
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
- statements are separated by newlines and comments use `#`
- source-level type annotations may use `Int`, `Bool`, `String`, nominal record types, and function types such as `A -> B`
- type inference is local-only
- type inference is locally bidirectional inside one function body, including some higher-order parameters
- receiver-style functions use a record type as the first parameter, and `self` is only a conventional parameter name
- `expr.name` is field access and `expr.name(...)` is a chained call
- `record.with(field: expr, ...)` is a record-only non-destructive update
- records use nominal data declarations together with record literals
- record fields may not have function type
- higher-order functions are allowed
- function types use `->`

## Documentation

- Canonical draft: [mini-language-spec-v1.md](./mini-language-spec-v1.md)
- Split specification:
  - [spec/001-core-language.md](./spec/001-core-language.md)
  - [spec/002-name-resolution.md](./spec/002-name-resolution.md)
  - [spec/003-typing.md](./spec/003-typing.md)
  - [spec/004-functions.md](./spec/004-functions.md)
  - [spec/005-records.md](./spec/005-records.md)
- Error catalog: [errors.md](./errors.md)

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
- `println` is available as a prelude builtin
- `println(x)` prints `Int`, `Bool`, or `String` on one line and returns the same value
- `record`, field access, `record.with` update, chained UFCS-style calls, and arrow function type annotations are implemented
- local bidirectional inference for some higher-order parameters and anonymous functions is implemented
- explicit receiver-style distinction is not implemented yet

## Planned Priority

The remaining work around records, dot syntax, and receiver-style calls is currently prioritized as follows:

1. explicit resolution rules for receiver-parameter style
2. pipe syntax if it becomes necessary later
3. broader chain sugar extensions after the core model is stable

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
- [samples/mixed_chain_pipeline.muga](./samples/mixed_chain_pipeline.muga) (runnable sample that mixes UFCS calls, record update, and field access)
- [samples/higher_order_functions.muga](./samples/higher_order_functions.muga) (runnable sample for higher-order functions with minimal annotations)
- [samples/higher_order_local_inference.muga](./samples/higher_order_local_inference.muga) (runnable sample for locally inferred higher-order parameters and anonymous functions)
- [samples/higher_order_explicit_arrow.muga](./samples/higher_order_explicit_arrow.muga) (runnable sample for explicit arrow annotations on callbacks)

Sample note:

- In [samples/mixed_chain_pipeline.muga](./samples/mixed_chain_pipeline.muga), `10.start().inc().inc().value.double()` has the same meaning as `double(inc(inc(start(10))).value)`. Both chain style and ordinary call style are valid.

Higher-order annotation guide:

- Omit an arrow annotation when the callback type is uniquely determined inside the same function body, as in [samples/higher_order_functions.muga](./samples/higher_order_functions.muga) and [samples/higher_order_local_inference.muga](./samples/higher_order_local_inference.muga).
- Keep an arrow annotation when local inference is still ambiguous, or when you want the callback contract to be obvious at the declaration site, as in [samples/higher_order_explicit_arrow.muga](./samples/higher_order_explicit_arrow.muga).
- `public API` is future-facing guidance for functions that will eventually be exposed across files or packages. Muga does not have `pub` or a module system yet, so this is not enforced today, but explicit annotations will likely be preferred at those boundaries for readability and fast interface checking.

## License

Licensed under the [MIT License](./LICENSE.txt).
