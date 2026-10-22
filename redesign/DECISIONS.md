# Muga Redesign Decisions

Status: settled. The maintainer made each decision one question at a time.
Do not reopen a decision without explicit maintainer approval. Record an
approved change as a new entry that supersedes the old one.

Decision dates: D-01 to D-16 on 2026-10-22, D-17 to D-25 on 2026-10-23.

## D-01: Safety Is The Top Priority

- **Decision:** The promise kept above all others is that code written by AI
  can be reviewed and then run safely with narrowed authority. Speed only
  needs to match a scripting language.
- **Alternatives:**
  - Top-tier native performance: the legacy direction. It makes a native backend a v1
    prerequisite, which was the biggest reason the legacy project was hard to
    finish.
  - AI authoring accuracy above all: rejects any mechanism that adds
    ceremony, including capability parameters.
  - Learnability for beginners: puts both safety and speed second.
- **Why:** Safety gives the narrowest scope that can be finished, and it
  matches the lesson about runtime memory safety in VISION.md.
- **Consequence:** A native backend is outside v1.

## D-02: Use Case Is CLI And Automation Tools

- **Decision:** v1 targets small to medium tools that work with files, JSON,
  external commands, standard input and output, and environment variables.
- **Alternatives:**
  - An embeddable host language (minimal core, but nothing works standalone).
  - Web API servers (needs networking, concurrency, and performance from day
    one).
  - Pure data transformation like jq or CUE (too narrow).
- **Why:** The most common case where AI writes code that a person runs on
  their own machine. Capabilities matter most here, and concurrency is not
  needed.

## D-03: Authority Is Passed As Capability Values

- **Decision:** `main` receives capabilities as parameters. Any function that
  performs I/O takes a capability as a parameter. A capability can be
  narrowed before it is passed, for example `files.only("data/")`.
- **Alternatives:**
  - Effect annotations such as `fn f() uses Fs`: needs an effect type system,
    and makes it hard to narrow authority to a path.
  - Runtime flags only: signatures would not show authority, which breaks
    promise 2.
- **Why:** Capabilities are ordinary values and types, so no new type-system
  machinery is needed. Narrowing is a plain function call. Tests can pass
  fake capabilities (D-16).

```muga
fn load(files: ReadFiles, path: String): Result[Config] {
  let text = files.read(path)?
  json.decode[Config](text)
}

fn main(files: ReadFiles, out: Stdout): Result[Unit] {
  let config = load(files.only("data/"), "data/config.json")?
  out.println(config.name)
  ok(unit)
}
```

## D-04: Grants Come From Run-Time Flags

- **Decision:** The parameters of `main` are the request. The person running
  the program grants scopes with flags such as `--read data/` and
  `--exec git`. If a grant is missing, the program does not start, and the
  error lists the needed flags. Standard input, output, and error, and the
  program arguments, are always granted.
- **Alternatives:**
  - Permissions in a project configuration file: reviewable, but needs a
    configuration file specification.
  - Interactive prompts: poor fit for CI and coding agents.
  - Automatically granting whatever `main` declares: safety would rest only
    on someone reading `main`.
- **Why:** No configuration file and a small implementation.

```text
$ muga run tool.muga
error: main in tool.muga requires these capabilities:
  files: ReadFiles   --read <path>
  git:   Command     --exec git

$ muga run tool.muga --read data/ --exec git
```

## D-05: No Networking In v1

- **Decision:** No network capability or HTTP library. When HTTP is needed,
  run an external command such as `curl` through `--exec curl`.
- **Alternatives:** An HTTP client with per-host grants (needs a TLS
  dependency and more code). An HTTP client and server (needs concurrency).
- **Why:** Keeps the implementation free of external crates.

## D-06: Generic Functions Only

- **Decision:** Users can write functions with type parameters, such as
  `fn first[T](items: List[T]): Option[T]`. There are no constraints or
  traits. User-defined records and enums cannot have type parameters.
- **Alternatives:** Generic functions and generic types. Built-in generics
  only, as in early Go.
- **Why:** The type checker needs generic functions internally anyway (for
  `map`, `filter`, and similar). Exposing them adds little. Generic types
  would add more types and more kinds of type errors.

## D-07: Value Semantics With Reassignable Mutable Bindings

- **Decision:** All values are immutable. Updating a value produces a new
  value, as in `names = names.push(x)` and `user.with(age: user.age + 1)`.
  Only bindings declared mutable can be reassigned (D-17).
- **Alternatives:**
  - In-place updates on mutable bindings, as with Swift structs: needs a
    "function that mutates its receiver" concept.
  - Fully immutable, as in Elm: procedural CLI code becomes awkward.
- **Why:** Values never alias, so a value passed to a function can never
  change behind the caller's back. Ordinary loops with accumulators still
  work.

## D-08: One Way To Write It, Enforced By The Grammar

- **Decision:** The grammar accepts exactly one spelling for each construct.
  Habits from other languages, such as `let x =` followed by reassignment,
  `null`, or a prefix `try`, are rejected with diagnostics that show the Muga
  spelling.
- **Alternatives:**
  - Accept several spellings and let the formatter or `lint --fix`
    canonicalize them (the legacy approach): both the grammar and the rewrite
    rules need maintenance.
  - Restrict the grammar and also reject any layout that differs from
    formatter output: tedious for hand editing.
- **Why:** The parser and formatter stay small. An AI agent fixes the code
  after one diagnostic.

## D-09: The Declaration Decides The Call Style

- **Decision:** A function whose first parameter is named `self` can only be
  called as `x.f(...)`. Every other function can only be called as `f(...)`.
  There is no inheritance and no dynamic dispatch.
- **Alternatives:**
  - Ordinary calls only: chains become deep nesting, and same-named
    functions need module qualification.
  - Dot calls only: produces unnatural forms like `1.max(2)` and
    `"hi".println()`.
- **Why:** Capability calls read naturally (`files.read(path)`). Names are
  distinguished by the type of `self`, so `files.read` and `stdin.read` do
  not clash. The result looks like methods in most languages.

```muga
fn is_adult(self: User): Bool {
  self.age >= 18
}

let adults = users.filter(fn(u) { u.is_adult() })
let big = max(1, 2)

is_adult(u)   // error: call it as u.is_adult()
```

## D-10: Bugs Stop The Program And Cannot Be Caught

- **Decision:** Out-of-range list access, integer overflow, division by zero,
  and other bugs stop the program immediately. The program prints the
  message, the location, and the call stack. Code has no way to catch this.
  The runtime performs cleanup.
- **Alternatives:**
  - No stopping at all, with every fallible operation returning `Option` or
    `Result`: indexing and arithmetic become heavy.
  - Stopping that callers can catch (`recover`): brings back the invisible
    control flow of exceptions.
- **Why:** A clean split. Expected failures are `Result` values, and bugs
  stop the program.

## D-11: Errors Propagate With Postfix `?`

- **Decision:** `expr?` returns the error from the enclosing function when
  `expr` is a failure.
- **Alternatives:** Prefix `try`, as in legacy Muga, Swift, and Zig. It is
  more visible in review, but chaining a call on its result needs
  parentheses or a temporary.
- **Why:** Chains with dot calls (D-09), as in `files.read(p)?.trim()`. AI
  agents write `?` naturally. The legacy roadmap even planned a diagnostic for
  agents that wrote postfix `?`.

## D-12: One Built-In `Error` Type

- **Decision:** `Result[T]` always fails with the built-in `Error`, which
  carries a message, a kind, and a chain of context. `?` works everywhere
  without conversion. Code that needs to branch on failure kinds returns its
  own enum instead of a `Result`.
- **Alternatives:** Arbitrary error types (`Result[T, E]`). Without traits
  there is no automatic error conversion, so `?` breaks whenever error types
  differ.
- **Why:** Enough for CLI tools, and similar to Rust's `anyhow`.

```muga
fn load(files: ReadFiles): Result[Config] {
  let text = files.read("config.json").context("cannot read the config file")?
  json.decode[Config](text)
}

fn check_age(age: Int): Result[Int] {
  if age < 0 {
    return err("age is negative")
  }
  ok(age)
}
```

## D-13: `Int` And `Float`, Both 64-Bit

- **Decision:** Two numeric types. There are no implicit conversions, so
  conversions are explicit, as in `count.to_float()`. `Int` overflow stops
  the program (D-10).
- **Alternatives:**
  - Arbitrary-precision `Int`: with no external crates this needs a
    hand-written bignum, and it is slower.
  - Adding `Decimal`: more types and conversion rules.
- **Why:** Familiar from most languages, and small to implement.

## D-14: One File Is One Module, With No Visibility Modifiers

- **Decision:** `import config` loads `config.muga` from the same directory,
  and its items are used as `config.load(...)`. There is no `pub`; every
  top-level item is visible within the project. v1 has no external packages,
  manifests, or lockfiles.
- **Alternatives:**
  - Single-file programs only: tools past a few hundred lines get hard to
    write.
  - A `pub` modifier: little benefit while there are no external packages.
- **Why:** Packages, manifests, artifact formats, and lockfiles were a large
  part of the legacy complexity.

## D-15: Tools Are `run`, `check`, `test`, And `fmt`

- **Decision:** Only these four commands. Diagnostics are text with
  suggested fixes, which AI agents read directly.
- **Alternatives:**
  - `run` and `check` only: no test runner or formatter.
  - The four commands plus JSON diagnostics and a language server: the scope
    grows a lot.
- **Why:** The legacy tooling surface was a large share of its code. JSON
  output, a language server, documentation generation, and build artifacts
  wait until after v1.

## D-16: Tests Are `test` Blocks In The Same File

- **Decision:** `test "name" { ... }` blocks sit next to the code they test.
  Fake capabilities, such as in-memory files, can only be created inside
  tests.
- **Alternatives:** Separate `_test` files with `test_` functions, as in Go.
- **Why:** The code and its tests can be read together in review. Capability
  values (D-03) make I/O code testable without touching the real system.

## D-17: Variables Are Declared With `let` And `let mut`

- **Decision:**
  - `let` declares an immutable binding. `let mut` declares a mutable one.
  - Reassignment (`x = e`) is allowed only for `let mut` bindings.
  - Shadowing is not allowed. A name means one thing within its scope.
  - Unused variables are errors. A `let mut` binding that is never
    reassigned is an error. Discarding a non-`Unit` value is an error.
  - There is no deferred initialization. Write
    `let x = if c { a } else { b }` instead.
  - `const`, `var`, and `:=` are rejected with suggested fixes.
- **History:** Legacy Muga used `x = 1` and `mut x = 1` without a
  declaration keyword. The maintainer chose that to avoid confusion: the
  declaration keywords mean different things in different languages.

  | Keyword | JavaScript / TypeScript | Swift | Rust | Kotlin | Go |
  |---|---|---|---|---|---|
  | `let` | reassignable | not reassignable | not reassignable | none | none |
  | `const` | not reassignable | none | compile-time constant | none | compile-time constant |
  | `var` | legacy, discouraged | reassignable | none | reassignable | reassignable |
  | `mut` | none | none | `let mut` is reassignable | none | none |

- **Alternatives:**
  - `let` and `var`, as in Swift: JavaScript users see `var` as deprecated.
  - `const` and `let`, as in JavaScript: a Rust or Swift user's `let`
    silently becomes reassignable, and the longer keyword is the safe one.
  - Keep the legacy keyword-free form: a typo in a reassignment silently
    creates a new binding, and a declaration looks exactly like a
    reassignment.
  - `let` and `mut` as two separate keywords: no language uses `mut` alone
    as a declaration, so AI agents would likely write `let mut` anyway.
- **Why:** Every misreading fails as a compile error with a suggested fix.
  None of them silently changes meaning:

  | Written by habit | Result |
  |---|---|
  | `let x = 0` then `x = 1` (JavaScript) | error; fix: `let mut x = 0` |
  | `const x = 1` (JavaScript) | error: Muga has no `const`; fix: `let x = 1` |
  | `var x = 0` (JavaScript, Swift, Kotlin, Go) | error; fix: `let mut x = 0` |
  | `v.push(1)` to mutate in place (Rust) | error: the returned value is discarded; fix: `v = v.push(1)` |
  | `let text = text.trim()` (Rust shadowing) | error: `text` is already declared; choose a new name |

- **Principle established:** Misreadings must fail loudly. Use this principle
  for future syntax decisions.

```muga
let name = "muga"
let mut total = 0
for x in items {
  total = total + x
}
```

## D-18: Strings Use `${...}` Interpolation Only

- **Decision:** `"Hello, ${user.name}"` is the only way to build strings from
  values. `+` between strings is an error with a suggested fix. Lists of
  strings are joined with `join`.
- **Alternatives:**
  - `{...}` interpolation: clashes with the braces of JSON text, which CLI
    tools write often, so those braces would need escaping as `{{` and `}}`.
  - Both `${...}` and `+`: breaks D-08.
  - `+` only: numbers need `to_string()`, and long messages are hard to read.
- **Why:** No clash with JSON. Familiar from JavaScript, TypeScript, Kotlin,
  Dart, and shells.

## D-19: Anonymous Functions Are `fn(u) { ... }`

- **Decision:** Anonymous functions use the same `fn` keyword as
  declarations. Parameter types are inferred from context.
- **Alternatives:**
  - `|u| ...`, as in Rust: short, but unfamiliar to JavaScript users.
  - `u => ...`, as in JavaScript: in JavaScript, `u => { u.age }` returns
    nothing, but in Muga a block returns its last expression. The same
    shape would mean something different.
- **Why:** Every function starts with `fn`, and the form collides with no
  other language's meaning.

```muga
let names = users
  .filter(fn(u) { u.age >= 18 })
  .map(fn(u) { u.name })
```

## D-20: No Tuples; Records Only

- **Decision:** Records are the only way to group values. Multiple return
  values use a record. Iterating a map yields entries with `.key` and
  `.value`.
- **Alternatives:** Tuples with destructuring, such as `let (a, b) = f()`.
- **Why:** One way to group data. Named fields are readable at the call
  site, where `.0` and `.1` are not.

## D-21: Enum Variants Have Three Forms, And `match` Is The Only Branch

- **Decision:** A variant carries no data (`Empty`), one value
  (`Circle(Float)`), or named fields (`Rect { w: Float, h: Float }`). Several
  unnamed values, as in `Rect(Float, Float)`, are rejected with a suggestion
  to name them. The only way to branch on variants is an exhaustive `match`.
  There is no `if let`.
- **Alternatives:**
  - No data or named fields only: `Option` would become `Some { value: x }`.
  - Allow every Rust form: tuple-like variants would bring tuples back, which
    D-20 rejected.
- **Why:** Consistent with D-20, while `Option` and `Result` stay natural.

```muga
enum Shape {
  Circle(Float),
  Rect { w: Float, h: Float },
  Empty,
}

fn area(s: Shape): Float {
  match s {
    Circle(r) => 3.14 * r * r,
    Rect { w, h } => w * h,
    Empty => 0.0,
  }
}
```

## D-22: Replace The Implementation In The Same Repository

- **Decision:** Rewrite `main` from scratch. The legacy implementation stays
  available through the `v0.6.0` tag and a `legacy` branch. The `legacy`
  branch is created from the latest `main` at replacement time, because
  `main` has unreleased work after `v0.6.0`.
- **Alternatives:**
  - A new repository: the release pipeline and the crates.io name would need
    to be carried over.
  - A parallel `next/` directory: two implementations to carry until the
    switch.
- **Why:** Keeps the `muga` crate name, the release process, and CI. The
  `0.x` series allows breaking changes.

## D-23: A Tree-Walking Interpreter

- **Decision:** v1 runs programs by walking the type-checked tree. The typed
  intermediate representation is kept separate, so a bytecode VM can replace
  the evaluator later.
- **Alternatives:**
  - A bytecode VM: several times faster, but more code and harder debugging.
  - Generating Rust code: native speed, but needs a Rust toolchain at run
    time, slows the edit-run loop, and returns to the speed-first direction
    that D-01 dropped.
- **Why:** The smallest implementation. CLI tools mostly wait on I/O.

## D-24: v1 Is Done When AI Solves The Task Suite

- **Decision:** Prepare about ten representative CLI tasks. v1 is done when
  an AI agent, given only the one-page guide, solves them with the minimum
  necessary grants. Features no task needs are not part of v1.
- **Alternatives:**
  - A feature checklist: scope tends to grow during implementation.
  - Rewriting the maintainer's own tools: the tasks would be biased toward
    one person's needs.
- **Why:** The legacy project had no stopping point, because "done" meant
  "the features are complete".

## D-25: Rules Derived From The Decisions Above

These rules follow from earlier decisions. They were shown to the maintainer,
who raised no objections.

- Function parameter types and return types are required. Local variable
  types are inferred (promise 2).
- A block evaluates to its last expression. `return` exits early (D-19).
- Loops are `for x in list`, `for i in 0..n`, and `while`. There is no C-style
  `for` (D-08).
- `?` does not apply to `Option`. Convert first with `opt.or_err("message")?`
  (D-12).
- When `main` returns an `Err`, the error and its context chain are written
  to standard error, and the process exits with status 1 (D-12).
- Capabilities are ordinary values, so they can be grouped in a record to cut
  down on parameter passing (D-03).
- The top level holds only immutable constants. Only the runtime and tests
  can create capabilities (D-01).
- Comments use `//` only (D-08).
