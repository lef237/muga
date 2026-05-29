# Muga

Muga is an experimental programming language for small, readable application
programs. This repository contains the Rust compiler/runtime implementation,
language notes, examples, and conformance tests.

The project is currently moving toward v1. The intended v1 shape is narrow:
source-compatible `check` and `run`, local type inference, immutable-by-default
bindings, explicit package artifacts, and clear diagnostics.

## Install

Install the published command:

```bash
cargo install muga
```

Install this checkout:

```bash
cargo install --path . --locked
```

Run from the checkout without installing:

```bash
cargo run --locked -- --version
cargo run --locked -- samples/println_sum.muga
```

## Quickstart

Create and run a small app:

```bash
muga new --template app hello-muga
muga run hello-muga/src/main/main.muga
muga check hello-muga/src/main/main.muga
```

The generated project contains a `muga.toml` manifest and source files under
`src/`. Try the available starters with:

```bash
muga new --list-templates
```

## A Small Program

```muga
fn sum_to(n: Int) {
  mut i = 0
  mut acc = 0

  while i < n {
    acc = acc + i
    i = i + 1
  }

  acc
}

fn main(): Int {
  println(sum_to(5))
}
```

Run it with:

```bash
muga run samples/println_sum.muga
```

## Common Commands

```bash
muga --help
muga doctor
muga explain E001
muga syntax --format json path/to/file.muga
muga check path/to/file.muga
muga run path/to/file.muga -- arg1 arg2
muga test path/to/file.muga
muga fmt path/to/file.muga
muga doc path/to/package/main.muga
muga build path/to/package/main.muga
```

Many commands also support `--format json` for editor and tooling workflows.

## Language Snapshot

- Bindings are immutable by default; use `mut` for mutation.
- There is no `let`.
- Type inference is local-first; write annotations where inference would be
  ambiguous.
- Data is modeled with nominal `record` and `enum` declarations.
- `Option[T]` and `Result[T, E]` are explicit, with `match` and prefix
  `try expr`.
- Packages use `package`, `import`, `pub`, and manifest files.
- Package artifact files use `.mgi`, `.mgc`, and `.mgb`.

For the compact language overview, start with [spec-v1.md](./spec-v1.md).
Detailed topic specs live in [spec/](./spec/).

## Repository Map

- [samples/](./samples/): runnable Muga programs and package examples.
- [examples/valid/](./examples/valid/): small accepted examples.
- [examples/invalid/](./examples/invalid/): examples that should be rejected.
- [conformance/](./conformance/): conformance fixtures and release checks.
- [errors.md](./errors.md): diagnostic catalog.
- [ROADMAP.md](./ROADMAP.md): current implementation direction.
- [RELEASING.md](./RELEASING.md): release process notes.

## Development

Run the test suite:

```bash
cargo test --locked
```

Run the local release-quality gate:

```bash
scripts/v1-release-gate.sh
```

Run benchmark health checks:

```bash
scripts/benchmark-health-check.sh
```
