# Muga

"Muga（無我）" is a Japanese term meaning "selflessness" or "transcendence of self," referring to a state of being beyond personal limitations or free from self-centered thinking.

This programming language incorporates the concept of muga, featuring a simple and intuitive syntax designed to immerse developers in coding while letting go of self-consciousness. 

Muga emphasizes both code aesthetics and efficiency, providing an environment where developers can freely express their creative ideas.

## Project Status

Muga is under active development in the `0.x` series. The language is improved
continuously through small releases; features, corrections, redesigns, and
removals are chosen because they improve the current language, not because they
must fit a predetermined `1.0.0` feature list.

Releases normally advance the patch component (`Z`) one step at a time, for
example `0.6.0` to `0.6.1`. Whether to advance the minor component (`Y`) is a
separate maintainer decision. Version `1.0.0` will name the point at which
continued experience shows that Muga no longer needs foundational redesign and
is ready for a long-lived compatibility promise. See
[RELEASING.md](./RELEASING.md) for the versioning policy and
[ROADMAP.md](./ROADMAP.md) for current work and the independent 1.0 readiness
criteria.

## Why Muga

Muga is a quiet, statically typed language for building self-contained tools
and reliable services, designed for local reasoning and machine-assisted
development.

- **Small surface, one spelling per operation.** No classes, inheritance,
  traits, or overloading. Records hold data, ordinary functions define
  behavior, and dot calls are just function calls — so any line of code can
  be read locally, without hunting for hidden dispatch.
- **Safe defaults with little ceremony.** Bindings are immutable unless
  marked `mut`, shadowing is rejected, and local type inference keeps
  annotations to where they actually help.
- **Errors are values, not surprises.** `Option[T]` and `Result[T, E]` with
  exhaustive `match` and prefix `try` — no implicit exceptions, no invisible
  control flow.
- **Value semantics everywhere.** Ordinary code never sees pointers,
  references, or ownership syntax; updates return new values.
- **Structured concurrency by construction.** `group { ... }` and `spawn`
  make task lifetimes lexical: child tasks can never outlive their group.
- **Tooling is part of the language.** `check`, `run`, `test`, `fmt`, `doc`,
  `build`, `explain`, and editor queries (`hover`, `definition`,
  `completions`, …) ship in one binary, and most commands speak
  `--format json` for editors, CI, and coding agents.
- **Honest package boundaries.** Public interfaces are explicit, and
  artifact-backed execution (`.mgi` / `.mgc` / `.mgb`) never silently falls
  back to reading dependency source bodies.

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
cd hello-muga
muga run src/main/main.muga
muga check src/main/main.muga
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
muga lint path/to/file.muga
muga lint --fix path/to/file.muga
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
- Structured task groups use `group { ... }` scopes, `spawn expr`, and
  `std::task` joins; child tasks never outlive their group.
- Packages use `package`, `import`, `pub`, and manifest files.
- Package artifact files use `.mgi`, `.mgc`, and `.mgb`.

For the compact language overview, start with [LANGUAGE.md](./LANGUAGE.md).
Detailed topic specs live in [spec/](./spec/).

## Repository Map

- [samples/](./samples/): runnable Muga programs and package examples that
  teach the language.
- [STYLE.md](./STYLE.md): canonical source style and lint-enforced conventions.
- [conformance/](./conformance/): fixtures that pin accepted programs,
  rejected diagnostics, and artifact-backed execution.
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
scripts/release-gate.sh
```

Run benchmark health checks:

```bash
scripts/benchmark-health-check.sh
```
