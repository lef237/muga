# Modern Language Gap Inventory 2026-05-22

Status: research inventory only. This is not a roadmap, language
specification, or implementation commitment. The decision pass over this
inventory is [modern-language-gap-decisions-2026-05-22.md](modern-language-gap-decisions-2026-05-22.md).

Purpose: preserve a broad list of capabilities and ecosystem practices that
modern popular languages expose as of 2026-05-22, so later work can decide
which pieces fit Muga's philosophy. This file intentionally lists more than
Muga should implement. Use it as a source inventory before updating
`ROADMAP.md`, `docs/practical-language-readiness.md`, or
`docs/implementation-resume-plan.md`.

## Source Snapshot

Popularity and adoption signals checked:

- [Stack Overflow Developer Survey 2025](https://survey.stackoverflow.co/2025/technology): Rust, Gleam, Elixir, and Zig are highly admired; VS Code, GitHub, Docker, Vite, Cargo, and `uv` show the importance of integrated tooling and developer experience.
- [GitHub Octoverse 2025](https://github.blog/news-insights/octoverse/octoverse-a-new-developer-joins-github-every-second-as-ai-leads-typescript-to-1/): TypeScript and Python are the top GitHub languages in 2025; new developers and AI-assisted workflows are now mainstream signals.
- [RedMonk Programming Language Rankings January 2026](https://redmonk.com/sogrady/2026/04/14/language-rankings-1-26/): long-term adoption remains concentrated around JavaScript, Python, Java, PHP, C#, C/C++, TypeScript, Ruby, Go, Swift, Kotlin, Dart, and Rust tiers.
- [PYPL May 2026](https://pypl.github.io/PYPL.html): tutorial search interest remains a useful leading indicator for onboarding demand.
- [TIOBE May 2026 reporting](https://www.techrepublic.com/article/tiobe-index-language-rankings/?post_id=3965858) and [TIOBE index methodology pages](https://www.tiobe.com/tiobe-index/): search/index visibility still favors Python, C, Java, C++, C#, JavaScript, SQL, R, and mature ecosystems.
- [JetBrains State of Developer Ecosystem 2025](https://devecosystem-2025.jetbrains.com/): tool and ecosystem surveys should be treated as biased but useful signals for IDE, AI, and language-workflow expectations.

Feature and toolchain references checked:

- Rust: [Cargo Reference](https://doc.rust-lang.org/cargo/reference/), [Cargo dependency resolver](https://doc.rust-lang.org/stable/cargo/reference/resolver.html), [Rust Edition Guide](https://doc.rust-lang.org/edition-guide/), [rustfmt](https://rust-lang.github.io/rustfmt/), [Clippy](https://doc.rust-lang.org/stable/clippy/index.html)
- Go: [command documentation](https://go.dev/doc/cmd), [fuzzing](https://go.dev/doc/security/fuzz/), [vulnerability management](https://go.dev/doc/security/vuln/), [benchmark guidance](https://go.dev/wiki/Benchmarks), [module layout](https://go.dev/doc/modules/layout)
- TypeScript: [project references and build mode](https://www.typescriptlang.org/docs/handbook/project-references.html), compiler options, declaration-file ecosystem
- Swift: [Swift 6 announcement](https://www.swift.org/blog/announcing-swift-6/), [Swift Package Manager](https://docs.swift.org/package-manager/PackageDescription/PackageDescription.html), package ecosystem notes
- Kotlin: [coroutines guide](https://kotlinlang.org/docs/coroutines-guide.html), [Kotlin Multiplatform docs](https://kotlinlang.org/docs/multiplatform/get-started.html), Gradle/KSP docs
- .NET/C#: [nullable reference types](https://learn.microsoft.com/en-us/dotnet/csharp/nullable-references), [Native AOT](https://learn.microsoft.com/en-us/dotnet/core/deploying/native-aot/), dependency injection, logging, testing, analyzers
- Java: [virtual threads](https://docs.oracle.com/en/java/javase/25/core/virtual-threads.html), [records](https://dev.java/learn/records/), [pattern matching](https://dev.java/learn/pattern-matching/), sealed-class specs
- Dart, Gleam, Elixir, Zig: official docs and ecosystem pages around null safety, package metadata, BEAM/OTP, formatters, package managers, cross-compilation, and external interop

## How To Read This Inventory

Use the following rough fit labels during the next decision pass:

- `Strong fit`: aligns with small readable source, explicit effects, package
  interfaces, deterministic artifacts, and agent-friendly tooling.
- `Likely fit`: useful, but needs a narrow design before implementation.
- `Needs design`: potentially valuable but could distort the language if added
  casually.
- `Post-v1`: useful later, but not needed for the v1 compatibility boundary.
- `Poor fit`: conflicts with Muga's current spine unless a future design note
  proves otherwise.

The labels below are preliminary inventory notes, not decisions.

## 1. Adoption And Product Shape

- Language tour and "Muga by Example" path from first expression to packages,
  errors, tests, and artifact-backed builds. Preliminary fit: `Strong fit`.
- A one-command first-run experience: install, `muga new`, `muga run`, `muga
  test`, and `muga doc` working on a generated project. Preliminary fit:
  `Strong fit`.
- Official package website or package index with docs, README rendering,
  examples, health signals, ownership, license, and vulnerability notices.
  Preliminary fit: `Post-v1`.
- Clear positioning by use case: scripting/automation, package-oriented CLIs,
  typed services, embedded tools, or teaching. Preliminary fit: `Strong fit`.
- Official style guide and API design guide, similar in role to Rust API
  guidelines, Go effective style, Swift API guidelines, and Kotlin style
  conventions. Preliminary fit: `Strong fit`.
- Community governance: contribution guide, RFC/design-note process, security
  policy, code of conduct, release notes, compatibility policy, and triage
  labels. Preliminary fit: `Strong fit`.
- Beginner and advanced docs split: tutorial, language reference, package
  guide, compiler architecture, standard-library guide, and migration guide.
  Preliminary fit: `Strong fit`.
- Interactive playground, browser examples, or hosted snippet runner.
  Preliminary fit: `Likely fit`.
- AI-agent usage guide: stable commands, JSON outputs, no hidden prompts,
  machine-readable task graph, and safe edit/test loops. Preliminary fit:
  `Strong fit`.

## 2. Core Language Surface

- Minimal static attributes for tooling, starting with `@test`,
  `@deprecated`, and possibly `@external_name`. Preliminary fit: `Strong fit`
  if attributes remain static metadata.
- Named arguments for long or same-typed call sites. Preliminary fit:
  `Needs design`; label compatibility and `.mgi` representation are the hard
  parts.
- Default arguments. Preliminary fit: `Needs design`; they can simplify APIs
  but complicate `.mgi` compatibility, overload-like resolution, and generated
  docs.
- Destructuring for records and tuples if tuples are ever added. Preliminary
  fit: `Needs design`; useful, but not yet justified by current record surface.
- Small match refinements: guards, `if` conditions on arms, and nested record
  patterns. Preliminary fit: `Needs design`; keep broad catch-all matching
  deferred.
- Exhaustiveness diagnostics with stable related notes. Preliminary fit:
  `Strong fit`.
- Range syntax for `Int`, `String`, `List`, and slicing. Preliminary fit:
  `Needs design`; must preserve explicit Unicode scalar/byte/grapheme policy.
- String interpolation or formatting templates. Preliminary fit:
  `Needs design`; should wait for `std::fmt`, escaping rules, and builder
  policy.
- `T?` and `?.` for Option-only ergonomics. Preliminary fit: `Post-v1`;
  keep separate from `Result` propagation.
- Postfix Result propagation such as `expr?`. Preliminary fit: `Poor fit`
  under current direction; use `try expr`, and maybe visible `expr.try` later.
- Pipe/operator features. Preliminary fit: `Needs design`; method-like calls
  and ordinary functions may already cover the ergonomic need.
- User-defined operators. Preliminary fit: `Poor fit` for v1 because it
  complicates readability, diagnostics, and package interfaces.
- Macros and compile-time code rewriting. Preliminary fit: `Poor fit` for the
  current spine; external code generation from `.mgi` is a safer first path.
- Runtime reflection as a core mechanism. Preliminary fit: `Poor fit`.
- Compile-time constants and simple build-time configuration. Preliminary fit:
  `Likely fit` if explicit and package-interface-visible.
- Module aliases, re-exports, public/private/package visibility ergonomics.
  Preliminary fit: `Strong fit`; already partly present, but docs and examples
  need hardening.
- Friend/internal visibility or test-only visibility. Preliminary fit:
  `Needs design`; useful for tests, risky for stable package boundaries.

## 3. Type System And Static Semantics

- First-class conformance tests for all accepted and rejected v1 syntax.
  Preliminary fit: `Strong fit`.
- Public API compatibility model over `.mgi`: breaking/non-breaking diff,
  deprecation, removed items, changed signatures, record fields, enum variants,
  and error type evolution. Preliminary fit: `Strong fit`.
- Stable machine-readable diagnostics. Preliminary fit: `Strong fit`.
- Warning/lint framework separate from hard errors. Preliminary fit:
  `Strong fit`.
- Dead code, unused import, unused binding, unreachable code, and unused public
  export lints. Preliminary fit: `Strong fit`.
- Style and idiom lints, similar to Clippy or analyzers. Preliminary fit:
  `Likely fit`.
- Nullability: keep `Option[T]`, no universal null. Preliminary fit:
  `Strong fit` for the current design.
- Type aliases. Preliminary fit: `Likely fit`; useful for public API clarity
  but must preserve canonical `.mgi` identity.
- Newtype or opaque public type. Preliminary fit: `Strong fit` for resource
  handles and domain types.
- Protocols/traits/typeclasses. Preliminary fit: `Post-v1`; only after
  concrete stdlib duplication proves the need.
- Generic constraints/bounds. Preliminary fit: `Post-v1`; likely needed before
  broad generic collection helpers, but not before the package boundary is
  stable.
- Associated types / higher-kinded types. Preliminary fit: `Poor fit` for the
  near term.
- Variance and subtyping. Preliminary fit: `Poor fit`; likely too much
  complexity for Muga's local-readability goal.
- Gradual typing or `Any`. Preliminary fit: `Poor fit` as a default; only
  explicit external adapters should carry dynamic values.
- Effect typing beyond `Result`. Preliminary fit: `Needs design`; could help
  IO/concurrency but risks becoming too heavy.
- Data-race capability facts for task/resource boundaries. Preliminary fit:
  `Strong fit` later, as small opaque-resource facts rather than borrow syntax.
- Public-signature inference for `pub fn`. Preliminary fit: `Post-v1` unless
  `.mgi` compatibility rules are settled first.
- Exhaustive enum evolution policy. Preliminary fit: `Strong fit`; required
  for API diff and compatibility docs.
- Semantic feature-set or edition fingerprint in package artifacts.
  Preliminary fit: `Strong fit`.

## 4. Error Handling And Diagnostics

- Error index with every diagnostic code, examples, and fixes. Preliminary fit:
  `Strong fit`.
- `muga explain <code>` for local explanation of diagnostics. Preliminary fit:
  `Strong fit`.
- JSON diagnostics from CLI and library. Preliminary fit: `Strong fit`.
- Suggested fixes and code actions that are precise enough for editors and AI
  agents. Preliminary fit: `Strong fit`.
- Multi-span diagnostics with declaration-site notes and package/artifact
  context. Preliminary fit: `Strong fit`; already partly present.
- Runtime error source spans and call context. Preliminary fit: `Strong fit`.
- Stack traces for named and anonymous functions. Preliminary fit: `Strong fit`
  after runtime call frames are stable.
- Error context chaining for `Result`, such as adding path/operation context.
  Preliminary fit: `Likely fit`.
- Rich standard error types per stdlib package instead of raw strings.
  Preliminary fit: `Strong fit`.
- Panic/abort mechanism. Preliminary fit: `Needs design`; useful for fatal
  bugs, but should not replace `Result`.
- Assertions separate from recoverable errors. Preliminary fit: `Strong fit`.
- Diagnostic snapshot tests and UI tests. Preliminary fit: `Strong fit`.

## 5. Testing And Verification

- `muga test` with `@test`, `Unit` or `Result[Unit, E]` return values, and
  structured failures. Preliminary fit: `Strong fit`.
- Scalar assertion helpers for `Int`, `Bool`, and `String`. Preliminary fit:
  `Strong fit`.
- Equality policy before structural assertions. Preliminary fit: `Strong fit`.
- Doctests or runnable code blocks in docs. Preliminary fit: `Likely fit`.
- Golden/snapshot tests for CLI output and diagnostics. Preliminary fit:
  `Strong fit`.
- Property-based testing. Preliminary fit: `Likely fit`; useful after random,
  shrinking, and display policy exist.
- Fuzz testing for parser, archive readers, diagnostics, and stdlib parsers.
  Preliminary fit: `Strong fit` for compiler implementation; `Post-v1` for
  user-facing language support.
- Coverage reporting. Preliminary fit: `Likely fit`.
- Benchmarks integrated with test runner. Preliminary fit: `Likely fit`;
  public thresholds should wait.
- Mutation testing. Preliminary fit: `Post-v1`.
- Conformance-suite layout separate from examples and release readiness tests.
  Preliminary fit: `Strong fit`.
- Cross-version compatibility fixtures for `.mgi`, `.mgb`, `.mgc`, `.mgp`,
  and `muga.lock`. Preliminary fit: `Strong fit`.
- Static analyzer tests for lints. Preliminary fit: `Strong fit`.
- Race or concurrency tests once task groups exist. Preliminary fit: `Post-v1`.

## 6. Standard Library Surface

- `std::test` assertion package. Preliminary fit: `Strong fit`.
- `std::option` helpers: `is_some`, `is_none`, `map`, `and_then`, `value_or`.
  Preliminary fit: `Strong fit`.
- `std::result` helpers: `is_ok`, `is_err`, `map`, `map_err`, `and_then`,
  `value_or`, maybe `context`. Preliminary fit: `Strong fit`.
- Collection helpers: `std::list` now provides `map`, `filter`, `fold`, `any`,
  and `all`; `std::map` now provides `keys` and `values`. `List.contains` and
  `Map.entries` are deferred because v1 equality is scalar-only and entries
  need a public record shape before they are worth exposing.
- `Set[T]`. Preliminary fit: `Post-v1`; needs equality/hash policy.
- Arbitrary `Map` keys. Preliminary fit: `Post-v1`; needs equality/hash
  policy.
- Ordered map or deterministic map iteration. Preliminary fit: `Needs design`;
  critical for reproducible tools if maps expose iteration.
- `Bytes` and binary buffers. Preliminary fit: `Strong fit` before network,
  hashing, and binary IO.
- `Buffer` and `StringBuilder`. Preliminary fit: `Strong fit` before broad
  formatting and high-throughput IO.
- `Duration`, `Instant`, and monotonic clock APIs. Preliminary fit:
  `Strong fit`.
- `DateTime` and timezone handling. Preliminary fit: `Post-v1`; UTC-first
  policy should come first.
- `Float` / `F64`, NaN policy, equality, ordering, and JSON encoding.
  Preliminary fit: `Strong fit` before JSON/scientific work, but needs careful
  spec.
- `Decimal`. Preliminary fit: `Post-v1`; useful for money, not a core scalar.
- `Random` with deterministic seedable RNG and secure RNG split. Preliminary
  fit: `Likely fit`.
- `Regex`. Preliminary fit: `Post-v1`; needs dependency and Unicode policy.
- `JSON` parse/encode with explicit data types and `Result`. Preliminary fit:
  `Strong fit` after helper ergonomics.
- `TOML` for manifests/config. Preliminary fit: `Likely fit`.
- `YAML`. Preliminary fit: `Post-v1`; complexity and edge cases are high.
- `CSV`. Preliminary fit: `Likely fit` for practical data exchange.
- `Hash` / checksum APIs. Preliminary fit: `Likely fit`; package hashing
  already needs the concept internally.
- `Crypto`. Preliminary fit: `Post-v1`; avoid designing crypto casually.
- Compression archives. Preliminary fit: `Post-v1`.
- `path`, `fs`, `io` resource handles instead of only one-shot helpers.
  Preliminary fit: `Strong fit`, but after opaque-resource rules.
- Stdout/stderr handles and structured output. Preliminary fit: `Likely fit`.
- `process` spawning and exit status. Preliminary fit: `Post-v1`; must handle
  resource lifetime, cancellation, and security.
- `env` current directory, executable path, args, vars. Preliminary fit:
  `Likely fit`; some pieces already exist.
- `log`, `metrics`, and `trace` packages. Preliminary fit: `Strong fit` for
  services, after structured context policy.
- `config` package with explicit sources. Preliminary fit: `Likely fit`.
- CLI argument parser package. Preliminary fit: `Likely fit`.
- URL, URI, and percent-encoding package. Preliminary fit: `Likely fit`.
- HTTP client/server. Preliminary fit: `Post-v1`; requires resource handles,
  bytes, cancellation, and backpressure.
- TLS. Preliminary fit: `Post-v1`; likely external dependency boundary.
- Database drivers. Preliminary fit: `Post-v1`; should be package ecosystem,
  not core.

## 7. Concurrency, Services, And Runtime Effects

- Opaque resource handle model with copy/send/share/close capabilities.
  Preliminary fit: `Strong fit`.
- Lexical cleanup (`using`) with deterministic close and explicit failure
  behavior. Preliminary fit: `Strong fit` after resources.
- Structured task groups. Preliminary fit: `Strong fit` after resources and
  runtime support.
- `spawn`, `join`, cancellation propagation, and join-result type.
  Preliminary fit: `Strong fit` if explicit.
- Channels. Preliminary fit: `Post-v1`; after structured tasks.
- `select`, deadlines, and timeouts. Preliminary fit: `Post-v1`.
- Async IO scheduler. Preliminary fit: `Post-v1`.
- Avoid hidden async suspension in ordinary calls. Preliminary fit: `Strong
  fit` as a constraint.
- Actor model / supervision tree, inspired by BEAM languages. Preliminary fit:
  `Needs design`; appealing for services, but likely higher-level runtime work.
- Graceful shutdown hooks. Preliminary fit: `Strong fit` for service runtime.
- Backpressure primitives for streams. Preliminary fit: `Post-v1`.
- Stream abstraction for bytes/messages. Preliminary fit: `Post-v1`.
- Data-race diagnostics. Preliminary fit: `Strong fit` once tasks exist.
- Runtime cancellation boundary documentation. Preliminary fit: `Strong fit`.

## 8. Compiler Architecture And Compile Speed

- Measured parse/check/build latency budgets for small, medium, and package
  graph projects. Preliminary fit: `Strong fit`.
- `muga check --watch` or compiler daemon for editor-speed feedback.
  Preliminary fit: `Likely fit`.
- Package-level incremental rebuild planning beyond unchanged artifact
  preservation. Preliminary fit: `Strong fit` after v1.
- Fine-grained incremental checking inside a package. Preliminary fit:
  `Post-v1`; complex but important for large projects.
- Stable dependency graph metadata command, e.g. `muga metadata`.
  Preliminary fit: `Strong fit`.
- Artifact/cache explanation command, e.g. `muga why-rebuild`.
  Preliminary fit: `Strong fit`.
- Build timing report, similar in spirit to Cargo build timings.
  Preliminary fit: `Likely fit`.
- Memory usage measurement in compiler tests. Preliminary fit: `Likely fit`.
- Parallel parsing/checking for independent package modules. Preliminary fit:
  `Likely fit`.
- Control-flow MIR. Preliminary fit: `Strong fit` after v1.
- Native backend. Preliminary fit: `Post-v1`; likely after MIR and benchmark
  health checks.
- Wasm backend or hosted WASM runtime. Preliminary fit: `Post-v1`; useful for
  playground and embedding.
- Debug info and source maps for bytecode/native/WASM. Preliminary fit:
  `Strong fit` once backends mature.
- Deterministic builds across OSes. Preliminary fit: `Strong fit`.
- Cross-compilation strategy. Preliminary fit: `Post-v1`; important if native
  backend or single-binary distribution becomes a goal.
- Build profiles: debug, release, test, bench. Preliminary fit: `Likely fit`.
- Compiler recovery after parse/type errors to produce multiple diagnostics.
  Preliminary fit: `Strong fit`.
- IDE-safe partial analysis. Preliminary fit: `Strong fit` for LSP.
- Internal query system or build graph engine. Preliminary fit: `Post-v1`;
  useful only when full incremental work starts.

## 9. CLI And Tooling Commands

- `muga fmt` and `muga fmt --check`. Preliminary fit: `Strong fit`.
- `muga lint` or `muga check --lint`. Preliminary fit: `Strong fit`.
- `muga test`. Preliminary fit: `Strong fit`.
- `muga bench`. Preliminary fit: `Likely fit`.
- `muga doc`. Preliminary fit: `Strong fit`.
- `muga new`. Preliminary fit: `Strong fit`.
- `muga metadata` for package graph and artifact facts. Preliminary fit:
  `Strong fit`.
- `muga explain <diagnostic-code>`. Preliminary fit: `Strong fit`.
- `muga tree` for dependency graph. Preliminary fit: `Likely fit`.
- `muga why <package/artifact>` or `why-rebuild`. Preliminary fit:
  `Likely fit`.
- `muga add`, `muga update`, `muga vendor`, `muga publish`, `muga yank`.
  Preliminary fit: `Post-v1`; wait for remote dependency model.
- `muga doctor` for environment, path, toolchain, and cache checks.
  Preliminary fit: `Likely fit`.
- `muga clean` scoped to build artifacts and caches. Preliminary fit:
  `Likely fit`.
- `muga repl` or scratch runner. Preliminary fit: `Needs design`; useful for
  teaching, but Muga is package/artifact-first.
- `muga format-json-diagnostics` is not needed if diagnostics can be emitted as
  JSON directly.
- Shell completions for CLI. Preliminary fit: `Likely fit`.
- Exit-code policy for all commands. Preliminary fit: `Strong fit`.

## 10. Editor, LSP, And AI-Agent Experience

- Official syntax highlighting grammar. Preliminary fit: `Strong fit`.
- LSP hover types and public docs. Preliminary fit: `Strong fit`.
- Go-to definition and references across packages through `.mgi`.
  Preliminary fit: `Strong fit`.
- Completion from visible package exports and local bindings. Preliminary fit:
  `Strong fit`.
- Rename refactor inside a package. Preliminary fit: `Likely fit`.
- Cross-package rename with API-diff warnings. Preliminary fit: `Post-v1`.
- Code actions for imports, annotations, artifact regeneration, and simple
  diagnostic fixes. Preliminary fit: `Strong fit`.
- Inlay hints for inferred types. Preliminary fit: `Likely fit`.
- Formatter integration and format-on-save. Preliminary fit: `Strong fit`.
- Test discovery and run/debug individual tests. Preliminary fit: `Strong fit`
  after `muga test`.
- Debug adapter protocol integration. Preliminary fit: `Post-v1`.
- AI-agent-safe structured outputs: diagnostics, package metadata, build
  results, test results, docs index, and API diff. Preliminary fit:
  `Strong fit`.
- Stable, documented file formats for tools. Preliminary fit: `Strong fit`.
- `llms.txt` or machine-readable docs index. Preliminary fit: `Likely fit`.

## 11. Package, Registry, And Supply Chain

- Workspace model for multiple related packages sharing build state.
  Preliminary fit: `Strong fit`.
- Dev/test/bench dependencies separated from normal dependencies.
  Preliminary fit: `Likely fit`.
- Optional dependencies or feature flags. Preliminary fit: `Needs design`;
  useful, but feature unification can become complex.
- Platform-specific dependencies. Preliminary fit: `Post-v1`.
- Version solver with SemVer ranges. Preliminary fit: `Post-v1`.
- Lockfile as source of truth for remote bytes. Preliminary fit: `Post-v1`.
- Source replacement, vendoring, or local mirrors. Preliminary fit: `Post-v1`.
- Package yanking and deprecation. Preliminary fit: `Post-v1`.
- Package ownership transfer and scoped names. Preliminary fit: `Post-v1`.
- Package signing and provenance attestations. Preliminary fit: `Post-v1`.
- Vulnerability database and `muga audit`. Preliminary fit: `Post-v1`.
- License metadata and policy checks. Preliminary fit: `Likely fit`.
- SBOM generation. Preliminary fit: `Post-v1`.
- Reproducible package archive verification. Preliminary fit: `Strong fit`;
  `.mgp` hashing already moves in this direction.
- Publish dry run and package contents listing. Preliminary fit: `Strong fit`
  if/when publish exists.
- Package score/quality signals: docs present, tests present, license,
  supported Muga version, recent compatibility. Preliminary fit: `Post-v1`.
- Registry search should not be a trust root. Preliminary fit: `Strong fit` as
  a design principle.
- Arbitrary build scripts. Preliminary fit: `Needs design` leaning `Poor fit`;
  they harm reproducibility unless tightly sandboxed.

## 12. Documentation And API Generation

- Public API docs generated from `.mgi` plus source comments. Preliminary fit:
  `Strong fit`.
- Doc comments syntax and rendering rules. Preliminary fit: `Strong fit`.
- Public examples attached to functions/types. Preliminary fit: `Likely fit`.
- Doctests for examples. Preliminary fit: `Likely fit`.
- Searchable docs index. Preliminary fit: `Likely fit`.
- Deprecated API rendering. Preliminary fit: `Strong fit`.
- API compatibility diff report in docs. Preliminary fit: `Likely fit`.
- Schema generation from `.mgi`: JSON Schema, OpenAPI, client stubs.
  Preliminary fit: `Post-v1`.
- Generated docs for package graphs. Preliminary fit: `Likely fit`.
- Versioned docs by package version. Preliminary fit: `Post-v1`.
- Migration guides for editions or breaking changes. Preliminary fit:
  `Strong fit` once editions exist.
- Standard-library cookbook. Preliminary fit: `Strong fit`.

## 13. Runtime, Deployment, And Operations

- Single binary distribution of the compiler. Preliminary fit: `Strong fit`.
- Binary releases for macOS/Linux/Windows. Preliminary fit: `Post-v1`, release
  timing remains maintainer-controlled.
- Self-contained application binaries after native backend. Preliminary fit:
  `Post-v1`.
- Bytecode artifact runner with stable versioning. Preliminary fit:
  `Strong fit` for current architecture.
- Debug symbols/source maps for runtime artifacts. Preliminary fit: `Likely
  fit`.
- Stack traces and structured crash reports. Preliminary fit: `Strong fit`.
- CPU profiler and allocation profiler. Preliminary fit: `Post-v1`.
- Tracing hooks for compiler and runtime. Preliminary fit: `Likely fit`.
- Runtime metrics for services. Preliminary fit: `Post-v1`.
- Structured logging package. Preliminary fit: `Strong fit` for services.
- Configuration sources: files, environment, CLI args. Preliminary fit:
  `Likely fit`.
- Secrets policy. Preliminary fit: `Post-v1`.
- Graceful shutdown for services. Preliminary fit: `Post-v1`.
- Container image guidance. Preliminary fit: `Post-v1`.
- WASI/WASM deployment. Preliminary fit: `Post-v1`.
- Embedded runtime or host embedding API. Preliminary fit: `Needs design`.

## 14. Interoperability

- C ABI interop. Preliminary fit: `Post-v1`; must preserve value semantics and
  resource safety.
- JavaScript/TypeScript interop or Wasm host adapters. Preliminary fit:
  `Post-v1`.
- CLI/process interop first. Preliminary fit: `Likely fit`.
- JSON/OpenAPI interop. Preliminary fit: `Strong fit` after schema rules.
- Database protocol adapters. Preliminary fit: `Post-v1`.
- Foreign package metadata adapters. Preliminary fit: `Post-v1`.
- Embedding Muga in Rust/Go/Swift host programs. Preliminary fit:
  `Needs design`.
- Stable ABI between Muga packages. Preliminary fit: `Post-v1`; `.mgi` is the
  current source-level contract.
- External function declarations. Preliminary fit: `Needs design`; useful but
  risk-prone.
- Generated clients from `.mgi`. Preliminary fit: `Post-v1`.

## 15. Performance And Quality Measurement

- Compile-time health dashboard: parse, resolve, typecheck, package load,
  artifact validation, build graph. Preliminary fit: `Strong fit`.
- Runtime microbenchmarks for scalar, string, list, map, enum, closure,
  function call, match, `try`, and IO paths. Preliminary fit: `Likely fit`.
- End-to-end benchmarks for package builds and artifact-backed run.
  Preliminary fit: `Strong fit`.
- Noisy benchmark quarantine and benchmark selection rules. Preliminary fit:
  `Strong fit`.
- Public performance claims only after stable workloads and enough history.
  Preliminary fit: `Strong fit`.
- Compiler memory benchmarks. Preliminary fit: `Likely fit`.
- Fuzzing parser/archive/lockfile readers. Preliminary fit: `Strong fit`.
- Static code-quality metrics for compiler implementation. Preliminary fit:
  `Likely fit`.
- CI matrix across OS/toolchain versions. Preliminary fit: `Strong fit`.
- Release-gate timing budgets. Preliminary fit: `Likely fit` after workload
  history.

## 16. Security And Trust

- Security policy and vulnerability disclosure process. Preliminary fit:
  `Strong fit`.
- Threat model for package archives, lockfiles, registry metadata, generated
  code, and build artifacts. Preliminary fit: `Strong fit`.
- Archive validation fuzzing. Preliminary fit: `Strong fit`.
- Cache collision and poisoning tests. Preliminary fit: `Strong fit`.
- Registry signing/provenance. Preliminary fit: `Post-v1`.
- Dependency vulnerability database. Preliminary fit: `Post-v1`.
- `muga audit`. Preliminary fit: `Post-v1`.
- Sandboxed build steps if build scripts ever exist. Preliminary fit:
  `Needs design`.
- Reproducible builds. Preliminary fit: `Strong fit`.
- Supply-chain metadata: license, source, checksum, publisher, namespace,
  deprecation/yank. Preliminary fit: `Post-v1`.
- Compiler/runtime hardening for untrusted source and artifacts. Preliminary
  fit: `Strong fit`.
- Safe defaults for network/process APIs. Preliminary fit: `Strong fit` when
  those APIs are added.

## 17. Compatibility, Editions, And Migration

- Language edition or semantic feature-set in artifacts. Preliminary fit:
  `Strong fit`.
- `muga fix` for mechanical migrations. Preliminary fit: `Post-v1`.
- Deprecation metadata in `.mgi`. Preliminary fit: `Strong fit`.
- Compatibility guide per release. Preliminary fit: `Strong fit`.
- API diff classifications. Preliminary fit: `Strong fit`.
- Source compatibility tests across editions. Preliminary fit: `Post-v1`.
- Package minimum Muga version field. Preliminary fit: `Strong fit`.
- Standard-library versioning policy. Preliminary fit: `Strong fit`.
- Compatibility lint for future-deprecated constructs. Preliminary fit:
  `Likely fit`.
- Migration docs and examples. Preliminary fit: `Strong fit`.

## 18. Domains And Ecosystem Bridges

- CLI applications: argument parsing, env, path/fs, process, logging,
  packaging. Preliminary fit: `Strong fit`.
- Typed backend services: JSON, HTTP, resource handles, structured
  concurrency, logging/metrics/tracing, config, shutdown. Preliminary fit:
  `Post-v1` after lower layers.
- Data scripts: CSV, JSON, decimal/float, date/time, notebooks or REPL.
  Preliminary fit: `Needs design`.
- Scientific/ML: arrays, linear algebra, GPU, Python interop. Preliminary fit:
  `Poor fit` for near term.
- Frontend/web UI: WASM, DOM bindings, JS interop. Preliminary fit: `Post-v1`.
- Mobile: native UI integration, package bridges. Preliminary fit: `Poor fit`
  for near term.
- Embedded: no-std profile, allocator control, cross-compilation. Preliminary
  fit: `Post-v1` only if native backend changes priorities.
- Education: playground, examples, friendly errors, course docs. Preliminary
  fit: `Strong fit`.

## 19. Muga-Specific Gaps Already Evident From This Pass

These are not final recommendations, but they are the most repeated gaps across
modern language ecosystems and current Muga documents:

- A conformance-suite layout beyond examples and release readiness tests.
- Stable JSON diagnostics and stable command output contracts.
- `muga fmt`, LSP, `muga test`, `muga doc`, and `muga new`.
- API compatibility diffing over `.mgi`.
- Standard-library review rules before adding broad APIs.
- Doc comments and generated public API docs.
- Package/workspace metadata commands for tools and AI agents.
- Build timing and cache explanation commands.
- Runtime error call context and eventual stack traces.
- Lightweight benchmark health checks without public performance promises.
- Fuzzing for parser, package archive, lockfile, and artifact readers.
- `Bytes`, `Buffer`, `StringBuilder`, `Duration`, `Instant`, `Float`, and
  explicit JSON mapping.
- Resource handle capabilities and lexical cleanup before broad IO/process/net.
- Supply-chain threat model before remote registries.
- Edition/feature-set fingerprints before syntax evolution.
- Installation/onboarding docs that stay separate from release prompting.
- Example-driven learning path, not only reference docs.

## Decision Pass

The recurring gaps are classified in
[modern-language-gap-decisions-2026-05-22.md](modern-language-gap-decisions-2026-05-22.md)
into:

1. v1 validation/support work that does not widen the language surface.
2. optional pre-v1 usability work that should be documented in the v1 checklist
   before implementation.
3. post-v1 platform work.
4. deliberate non-goals.

Use that decision file, not this inventory alone, when updating the main
roadmap or implementation resume plan.
