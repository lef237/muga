# Muga Roadmap

Keep this file short. Language details belong in [spec-v1.md](./spec-v1.md)
and [spec/](./spec/). Executable behavior should be shown by Rust tests and
Muga samples.

## Current State

Muga currently has:

- lexer, parser, resolver, typechecker, typed HIR, MIR lowering, bytecode, and a
  reference VM runtime
- `check`, `run`, `test`, `fmt`, `doc`, `build`, artifact, archive, bundle,
  completion, metadata, workspace, hover, definition, and references commands
- package interfaces and artifacts through `.mgi`, `.mgc`, and `.mgb`
- local path and local archive dependencies with lockfile metadata
- app and package archive workflows through `.mga` and `.mgp`
- core v1 language surface: immutable-by-default bindings, `mut`, records,
  enums, functions, closures, local inference, `Option`, `Result`, `try`,
  `match`, `for`, `break`, `continue`, `return`, `Unit`, package imports, and
  statement-form `using`
- standard package slices for `std::io`, `std::fs`, `std::path`, `std::env`,
  `std::cli`, `std::time`, `std::bytes`, `std::hash`, `std::string`,
  `std::fmt`, `std::list`, `std::map`, `std::option`, `std::result`,
  `std::json`, `std::config`, and `std::test`

Run the local quality gate with:

```bash
scripts/v1-release-gate.sh
```

## Active Priority

The next implementation priority is **Core Capability Acceleration**.

Start with `std::process` as the first external-effect spine:

- child command execution
- captured status, stdout, and stderr
- explicit cwd and env options
- public process error records
- source and artifact-backed execution
- one runnable sample under `samples/packages/app/`
- focused Rust tests in the existing examples/conformance style

After that, grow practical capability in this order:

1. structured task groups: explicit `spawn` / `join`, scoped lifetimes,
   failure propagation, cancellation, and timeout boundaries
2. service IO: socket and minimal HTTP/JSON workflows after resource and task
   semantics can express shutdown and backpressure
3. performance path: control-flow MIR, runtime representation work, and
   benchmark evidence before native backend claims
4. distribution path: build on `.mgp` / `.mga`, source-free bundles,
   verification, and install inventory before registry publishing

## Stability Rules

- Keep the v1 source model small.
- Prefer code, samples, conformance fixtures, and Rust tests over long design
  prose.
- Add prose only when a rule is not clear from code or samples.
- Keep `samples/` runnable. Invalid examples belong under `examples/invalid/`,
  `conformance/v1/rejecting/`, or `spec/snippets/`.
- Do not make normal `check` / `run` silently depend on built artifacts.
- Keep artifact-backed package execution hard-failing without dependency source
  fallback.
- Keep public `pub fn` signatures explicit until public-signature inference is
  deliberately implemented.
- Keep diagnostics stable, actionable, and source/artifact-aware.

## Deferred

These remain out of the active slice unless a concrete implementation need
forces the decision:

- classes, inheritance, traits, protocols, typeclasses, overloaded dispatch
- source-level references, mutable references, pointer syntax, or borrowing
- postfix `expr?`, future `expr.try`, `T?`, and optional chaining
- broad wildcard pattern matching, nested patterns, guards, and multi-payload
  enum variants
- map literals, `Set[T]`, arbitrary `Map` keys, broad collection APIs, and
  iterator protocols
- URL/Git/registry dependencies, remote fetching, publishing workflows, package
  signing, SBOMs, and full published-package lockfile enforcement
- binary streams, codecs, broader cryptographic APIs, service runtime APIs,
  async IO, and native backend work

## Short Version

Keep Muga useful by adding narrow, executable vertical slices. The next slice is
`std::process`; prove it with Muga samples, artifact-backed execution, and Rust
tests.
