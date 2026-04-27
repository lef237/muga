# Language Design Reference

Status: design note. This is not a specification and does not define implemented behavior.

This document lists languages Muga should keep in view when making design decisions, then summarizes design patterns that Muga should either reject, constrain, or design explicitly.

The goal is not to rank languages or criticize them. Each language succeeds under different constraints. The purpose here is to keep Muga's own direction clear: simple syntax, strong static typing, minimal annotations, fast compilation, function-centered design, and readable code.

## 1. Reference Languages

These languages are useful references for Muga because they are widely used, admired, influential, or deliberately designed around a clear philosophy.

| Language | Useful reference point for Muga |
|---|---|
| Rust | strong static typing, `Option`, enums, traits, explicit ownership, performance without a GC |
| Go | small language surface, fast builds, simple concurrency, practical tooling |
| Ruby | natural readability, low visual noise, expressive blocks, programmer-oriented API design |
| Python | readability, low ceremony, strong standard-library culture, productive scripting |
| TypeScript | inference-first typed ergonomics on top of a flexible programming model |
| C# | pragmatic mainstream language evolution, async, LINQ, tooling, strong IDE feedback |
| Kotlin | null-safety, concise syntax, extension functions, sealed types |
| Zig | low-level performance, explicit allocation, no hidden control flow as a design principle |
| Swift | readable modern syntax, optionals, enums with associated values, strong API design culture |
| OCaml | local type inference, modules, explicit interfaces, algebraic data types, fast separate compilation lessons |
| F# | practical ML-family syntax, discriminated unions, pipelines, type inference in a mainstream ecosystem |
| ReScript | fast typed JavaScript workflow, inferred types, readable output, curated type-system surface |
| Elixir | lightweight concurrency, pattern matching, pipeline style, fault-tolerant runtime culture |
| Erlang | lightweight processes, message passing, fault-tolerance model, BEAM runtime lessons |
| Lisp / Clojure | expression-oriented programming, data-as-code, powerful abstraction pressure |
| Gleam | typed functional programming on BEAM, small surface, friendly error messages |
| MoonBit | modern ML-style design, fast tooling goals, WebAssembly-oriented direction |
| Nim | expressive syntax, systems capability, metaprogramming, Python-like readability goals |
| Elm | simple functional UI architecture, strong static guarantees, no uncontrolled runtime exceptions as a design goal |
| V | fast compilation goals, simple syntax, Go-like practicality |
| Crystal | Ruby-like readability with static typing and native compilation |
| Lua | small language core, lightweight implementation, embeddability, practical data-description style |
| Pony | actor-model concurrency, data-race freedom, no null, AOT native compilation as a high-bar safety reference |

Muga does not need to copy any one of these languages. It should instead combine a small number of compatible ideas:

- Go-like compile-speed ambition
- Rust-like explicitness around absence and errors
- Ruby-like readability and low visual noise
- TypeScript-like annotation-light ergonomics
- Kotlin/Swift-like null-safety direction
- OCaml/ReScript-like separation between local inference and explicit module/package interfaces
- Elixir-like lightweight concurrency ergonomics, without requiring dynamic typing
- Erlang/Pony-like attention to concurrency safety and runtime behavior
- Lua-like small-core discipline
- Zig-like explicitness around hidden behavior

These references should be used selectively.

Reference does not mean adoption. Muga should borrow specific strengths without inheriting each language's full feature bundle.

Examples:

- borrow readability and low visual noise, but keep static analysis strong
- borrow inference-first ergonomics, but keep package interfaces explicit and cacheable
- borrow lightweight concurrency ergonomics, but keep the model compatible with static typing and native compilation
- borrow small-core discipline, but still provide enough language structure for practical web and CLI programs

## 2. Inference And Package Interfaces

Muga should infer as much as it can locally.

This means a function like this should be accepted when its body uniquely determines the types:

```muga
pub fn inc(x) {
  x + 1
}
```

The defining package can infer the public signature:

```txt
pub fn inc(Int): Int
```

Then it stores that resolved signature in a package interface.

Downstream packages should typecheck against that interface, not against the dependency's full source body.

That gives Muga both:

- annotation-light source code
- fast separate compilation

What Muga should **not** use as the default model is whole-program inference.

Whole-program inference means the compiler may infer a function's type by looking at all call sites across the whole program:

```muga
pub fn id(x) {
  x
}

a = id(1)
b = id("name")
```

This can be powerful, but it makes package boundaries harder to cache. A change in one downstream package can affect what an upstream public function appears to mean, or force the compiler to re-open many dependency bodies.

Muga's preferred rule:

- infer inside the current function, module, or package when the result is unique
- store resolved public signatures in package interfaces
- do not infer public APIs from arbitrary downstream call sites
- require annotations only when local inference cannot determine one stable signature

## 3. Muga Design Positions

This section is intentionally direct about Muga's own direction.

Direct language does not imply disrespect toward other languages. Many successful languages make different choices productively. The point here is to make Muga's constraints explicit.

### 3.1 No classes

Muga will not introduce classes.

This means:

- no `class` declaration
- no member-owned methods
- no instance-variable model
- no constructor system tied to classes
- no class inheritance

Muga's object-like ergonomics should come from records, functions, modules, visibility, and chained-call syntax.

Preferred direction:

- data is declared with `record`
- behavior is declared with functions
- method-like calls are surface syntax over functions
- encapsulation is handled by modules and visibility, not class ownership

Reason:

- class-centered design would move Muga away from its function-centered core
- class ownership would make `record` less clearly a data-only construct
- class methods would compete with UFCS-style function calls
- Muga wants small data types, explicit functions, and predictable name resolution
- method syntax should remain call-site ergonomics, not a semantic ownership model

This is not a negative judgment on class-based languages. Kotlin, Swift, C#, Ruby, Python, and TypeScript all use class-like constructs productively. Muga's choice is different: classes are not part of the language model.

### 3.2 No class inheritance

Because Muga has no classes, class inheritance is out of scope.

Preferred direction:

- composition
- functions
- records
- future protocols/traits/interfaces if needed
- future enums or sum types for closed variants

Reason:

- inheritance can mix code reuse, subtyping, construction, overriding, and state layout
- it can complicate local reasoning and compiler implementation
- it can make APIs harder to evolve when used as the default reuse pattern

### 3.3 No universal null

Muga should not make every type implicitly nullable.

Preferred direction:

- ordinary `T` means a value is present
- `Option[T]` means a value may be absent
- safe collection lookup returns `Option[T]`

Reason:

- absence should be visible in the type
- callers should handle missing values deliberately
- this avoids a large class of null-reference errors

### 3.4 Constrain hidden control flow

Muga should constrain features where a simple-looking expression can perform non-obvious control flow.

Muga should not make the following ordinary default mechanisms:

- implicit throwing exceptions
- property access that may execute arbitrary user code
- implicit conversions
- user-defined operator overloading in v1
- hidden async suspension

Reason:

- Muga should be easy to read locally
- fast compilers benefit from fewer implicit semantic paths
- non-obvious behavior makes performance and error handling harder to understand

This does not rule out every future form of error handling, operator customization, or asynchronous IO. It means those features must be explicit enough that ordinary code remains locally readable.

### 3.5 No global mutable state by default

Muga should make shared mutable state explicit and uncommon.

Preferred direction:

- immutable by default
- `mut` is local and explicit
- cross-task mutation should be restricted or explicit
- concurrency should prefer message passing or structured task ownership

Reason:

- mutable global state makes reasoning, testing, and concurrency harder
- it makes safe high-performance parallelism harder

### 3.6 No package-only encapsulation

Muga should not make package visibility the only encapsulation boundary.

Preferred direction:

- package is the import/build/cache unit
- module/file is the default encapsulation unit
- `pkg` exposes within the package
- `pub` exposes across package boundaries

Reason:

- package-only privacy makes small abstractions leak across unrelated files
- Muga should support fine-grained encapsulation without forcing many tiny directories

### 3.7 No raw-string internal state by default

Muga should not use raw strings as the default representation for closed internal states.

Preferred direction:

- use records for known shapes
- use future enums or sum types for finite states
- use `String` for external text and external data keys

Reason:

- raw-string internal APIs give up useful static checking
- spelling mistakes become runtime bugs
- closed states should be explicit in the type system

### 3.8 No whole-program inference by default

Muga should infer aggressively inside a local body, but should not use whole-program inference as the default compilation model.

Preferred direction:

- local inference inside functions
- inferred public signatures may be stored in package interfaces
- downstream packages read interfaces rather than rechecking dependency bodies

Reason:

- users should not write unnecessary annotations
- compilation should stay fast and cache-friendly
- package boundaries need stable interfaces

### 3.9 No default dynamic or `Any` escape hatch

Muga should not make untyped dynamic values the ordinary way to escape the type system.

Preferred direction:

- use concrete types for ordinary values
- use `Option[T]` for possible absence
- use future enums or sum types for finite variants
- use future generic abstractions for reusable typed APIs
- reserve any dynamic interop escape hatch for explicit boundary APIs if needed

Reason:

- Muga wants static types to remain trustworthy
- broad dynamic escape hatches make diagnostics and optimization weaker
- web and CLI development still benefit from type precision at boundaries

### 3.10 No ambiguous dot syntax

Muga should keep dot syntax stable.

Current examples:

- `expr.name` means field access
- `expr.name(...)` means chained call
- `record.with(...)` means non-destructive record update

Muga v1 therefore does not allow function-valued record fields.

Reason:

- `expr.name` should not sometimes mean field access and sometimes a method lookup
- `expr.name(...)` should not mean a field-function call
- stable dot syntax keeps parsing, resolution, and reading simple

### 3.11 No package-level wildcard behavior by default

Muga should keep package imports explicit.

Preferred direction:

- imported packages have local aliases
- package-qualified access uses `alias::name`
- alias conflicts are errors
- implicit re-exports are not part of the default model

Reason:

- explicit imports keep dependency edges clear
- package interfaces stay easier to cache
- diagnostics can point to one concrete source of a name

### 3.12 No async function coloring as the primary concurrency model

Muga should not make the entire language revolve around `async fn` and `await` as the first concurrency model.

Preferred direction:

- structured task groups
- lightweight `spawn`
- explicit `join`
- typed channels later
- async IO details after the task model is stable

Reason:

- ordinary function signatures should stay simple
- concurrency lifetimes should be structured
- the model should fit both VM execution and future native compilation

This does not forbid future async-specific APIs.

### 3.13 Stable syntax over overloaded forms

Muga should keep common syntax forms stable.

Current examples:

- `expr.name` means field access
- `expr.name(...)` means chained call
- `record.with(...)` means non-destructive record update
- `alias::name` means package-qualified access

Reason:

- stable syntax lowers reader burden
- stable syntax simplifies parser and resolver design
- ambiguity is expensive in both implementation and documentation

### 3.14 No runtime metaprogramming as a core mechanism

Muga should not rely on runtime reflection, monkey patching, or dynamic metaprogramming for ordinary abstraction.

Reason:

- it reduces what static analysis can prove
- it makes fast separate compilation harder
- it makes code harder to read and optimize

Compile-time generation may be considered later, but it should not define the v1 core.

## 4. Practical Design Rule For Muga

When deciding whether to add a feature, ask:

1. Does this keep ordinary code shorter and clearer?
2. Does this preserve local reasoning?
3. Does this keep static types useful without requiring many annotations?
4. Does this keep the parser, resolver, and typechecker fast?
5. Does this work with package interfaces and incremental compilation?
6. Does this fit a function-centered language without classes?

If a feature fails several of these checks, it should be deferred even if another successful language has it.

## 5. References

- [Stack Overflow Developer Survey 2025: admired and desired languages](https://survey.stackoverflow.co/2025/technology/)
- [JetBrains State of Developer Ecosystem 2025](https://blog.jetbrains.com/research/2025/10/state-of-developer-ecosystem-2025)
- [Ruby: About Ruby](https://www.ruby-lang.org/en/about/)
- [Go FAQ: object-oriented language and inheritance](https://tip.golang.org/doc/faq)
- [Rust Book: object-oriented characteristics and inheritance](https://doc.rust-lang.org/stable/book/ch18-01-what-is-oo.html)
- [OCaml compiler frontend: type inference and module signatures](https://ocaml.org/docs/compiler-frontend)
- [F# discriminated unions](https://learn.microsoft.com/dotnet/fsharp/language-reference/discriminated-unions/)
- [ReScript language manual: introduction](https://rescript-lang.org/docs/manual/v12.0.0/introduction)
- [Erlang reference manual: processes](https://www.erlang.org/docs/17/reference_manual/processes)
- [Lua: about](https://www.lua.org/about.html)
- [Pony: what makes Pony different](https://www.ponylang.io/discover/what-makes-pony-different/)
- [Kotlin null safety](https://kotlinlang.org/docs/null-safety.html)
- [Zig overview: no hidden control flow](https://ziglang.org/learn/overview/)
