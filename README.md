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
cargo run -- samples/print_sum.muga
```

Expected output (the first line is `print`, the second line is the return value of `main`):

```
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
cargo run -- check samples/print_sum.muga
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
  print(1 + 2)
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

### Invalid

- [examples/invalid/001-immutable-update.md](./examples/invalid/001-immutable-update.md)
- [examples/invalid/002-duplicate-mutable-binding.md](./examples/invalid/002-duplicate-mutable-binding.md)
- [examples/invalid/003-shadowing-in-block.md](./examples/invalid/003-shadowing-in-block.md)
- [examples/invalid/004-outer-scope-mutation.md](./examples/invalid/004-outer-scope-mutation.md)
- [examples/invalid/005-ambiguous-identity.md](./examples/invalid/005-ambiguous-identity.md)
- [examples/invalid/006-unannotated-recursion.md](./examples/invalid/006-unannotated-recursion.md)
- [examples/invalid/007-unannotated-mutual-recursion.md](./examples/invalid/007-unannotated-mutual-recursion.md)
- [examples/invalid/008-invalid-record-update.md](./examples/invalid/008-invalid-record-update.md)

## Rust Implementation

- parsing, name resolution, type checking, HIR lowering, bytecode compilation, and the VM runtime are being implemented
- HIR and bytecode names are managed through symbol interning
- `check` only validates the front end
- `run` passes through the front end, lowers to HIR, compiles to bytecode, and executes the result
- `run` prints the return value when a zero-argument `main()` exists
- `print` is available as a prelude builtin
- `print(x)` prints `Int`, `Bool`, or `String` on one line and returns the same value
- `record`, field access, `record.with` update, and chained UFCS-style calls are implemented
- explicit receiver-style distinction and arrow function type annotations are not implemented yet

## Planned Priority

The remaining work around records, dot syntax, and receiver-style calls is currently prioritized as follows:

1. explicit resolution rules for receiver-parameter style
2. function types in parameter annotations and higher-order function annotation syntax
3. pipe syntax if it becomes necessary later
4. broader chain sugar extensions after the core model is stable

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
- [samples/print_sum.muga](./samples/print_sum.muga)
- [samples/closure_capture.muga](./samples/closure_capture.muga)
- [samples/record_field_access.muga](./samples/record_field_access.muga) (runnable sample for `record` and field access)
- [samples/record_counter_loop.muga](./samples/record_counter_loop.muga) (runnable sample for mutable bindings and `record.with(...)`)
- [samples/nested_record_access.muga](./samples/nested_record_access.muga) (runnable sample for nested record access)
- [samples/record_with_update.muga](./samples/record_with_update.muga) (runnable sample for `record`, field access, and `record.with(...)`)
- [samples/method_chain_user.muga](./samples/method_chain_user.muga) (runnable sample for chained UFCS-style calls)
- [samples/number_chain.muga](./samples/number_chain.muga) (runnable sample for chaining plain functions on `Int`)
- [samples/print_chain.muga](./samples/print_chain.muga) (runnable sample for chaining through builtin `print`)
- [samples/mixed_chain_pipeline.muga](./samples/mixed_chain_pipeline.muga) (runnable sample that mixes UFCS calls, record update, and field access)
- [samples/planned_record_user.muga](./samples/planned_record_user.muga) (planned syntax sample for `record`, receiver-style functions, and dot syntax)
- [samples/planned_higher_order_functions.muga](./samples/planned_higher_order_functions.muga) (planned syntax sample for `->` function types and higher-order functions)

Sample note:

- In [samples/mixed_chain_pipeline.muga](./samples/mixed_chain_pipeline.muga), `10.start().inc().inc().value.double()` has the same meaning as `double(inc(inc(start(10))).value)`. Both chain style and ordinary call style are valid.
