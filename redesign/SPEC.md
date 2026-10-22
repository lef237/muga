# Muga Language And Tool Specification (Redesign Draft)

Status: draft v0, assembled on 2026-10-23 from [DECISIONS.md](./DECISIONS.md).

How to read this draft:

- A rule that cites a decision (D-nn) is settled.
- **Provisional** marks a spelling or detail chosen only to make the examples
  concrete. It may change without a new decision entry.
- **Open** marks a question that needs a maintainer decision. All open
  questions are listed in [Open Questions](#open-questions).

## 1. Program Structure

A program is a directory of `.muga` files, and each file is a module (D-14).
The file passed to `muga run` is the entry module and must declare `main`.

```muga
import config

fn main(files: ReadFiles, out: Stdout): Result[Unit] {
  let c = config.load(files)?
  out.println("name: ${c.name}")
  ok(unit)
}
```

- `import name` loads `name.muga` from the same directory. Its items are used
  as `name.item`. Subdirectories are Open (O-21).
- There are no visibility modifiers. Every top-level item is visible to the
  modules that import its module.
- There are no external packages, manifests, or lockfiles in v1.
- The top level contains only `fn`, `record`, `enum`, `test`, and immutable
  constants (D-25). Which expressions a constant may use is Open (O-18).
- An import cycle is an error (Provisional).

## 2. Lexical Structure

- Comments run from `//` to the end of the line. There are no block comments
  (D-25).
- Statements end at line breaks. There are no semicolons (Provisional).
- Literals:
  - `Int`: `42`
  - `Float`: `1.5`, with digits on both sides of the point (Provisional)
  - `Bool`: `true`, `false`
  - `String`: `"..."`, with the escapes `\"`, `\\`, `\n`, `\t`, and `\$`
    (Provisional), and interpolation `${expr}` (D-18)
- Naming: values and functions use `snake_case`, and types and variants use
  `PascalCase` (Provisional). Whether the compiler enforces this is Open
  (O-11).

## 3. Types

| Type | Meaning |
|---|---|
| `Int` | 64-bit signed integer. Overflow stops the program (D-13, D-10). |
| `Float` | 64-bit IEEE 754 floating point (D-13). |
| `Bool` | `true` or `false`. |
| `String` | Immutable UTF-8 text. |
| `Unit` | The type of expressions with no useful value. Its value spelling is Open (O-3). |
| `List[T]` | An ordered list. |
| `Map[K, V]` | A key-value map. Key types, iteration order, and literal syntax are Open (O-2). |
| `Option[T]` | `Some(value)` or `None`. |
| `Result[T]` | A success value of type `T`, or a failure carrying `Error` (D-12). |
| `Error` | The built-in error value (D-12). |
| `record`, `enum` | User-defined nominal types (D-20, D-21). |
| function types | Values created with `fn`. Their type syntax is Open (O-5). |
| capability types | `ReadFiles`, `WriteFiles`, `Command`, `Env`, `Stdin`, `Stdout`, `Stderr`, `Args` (names Provisional, D-03). |

- There is no null. Absence is expressed with `Option`.
- There are no implicit conversions. Convert explicitly, for example with
  `count.to_float()` (D-13).
- All values have value semantics. No value can be changed through another
  name (D-07).

## 4. Bindings

```muga
let name = "muga"
let mut total = 0
total = total + 1
```

Rules (D-17):

- `let` declares an immutable binding, and `let mut` a mutable one.
- `x = e` reassigns `x`. This is allowed only when `x` was declared with
  `let mut`, and the type of `e` must equal the type of `x`.
- Declaring a name that is already visible in an enclosing scope is an error
  (no shadowing).
- An unused binding is an error. A `let mut` binding that is never reassigned
  is an error.
- Every binding is initialized at its declaration. There is no `let x` without
  a value.
- Function parameters are immutable bindings.
- A local variable's type is inferred from its initializer. Whether an
  explicit annotation such as `let x: Int = 1` is allowed is Open (O-23).

## 5. Functions

### 5.1 Declarations

```muga
fn area(w: Int, h: Int): Int {
  w * h
}
```

- Parameter types and return types are required (D-25). Whether a function
  that returns `Unit` may omit `: Unit` is Open (O-6).
- A block evaluates to its last expression. `return expr` exits early.
- There is no overloading. Within a module, a name refers to one function,
  except that `self` functions are told apart by the type of `self` (5.2).

### 5.2 `self` Functions And Call Style

A function whose first parameter is named `self` can only be called with a
dot. Every other function can only be called directly (D-09).

```muga
fn is_adult(self: User): Bool {
  self.age >= 18
}

u.is_adult()    // ok
is_adult(u)     // error: call it as u.is_adult()
max(1, 2)       // ok
1.max(2)        // error: call it as max(1, 2)
```

- `read(self: ReadFiles, ...)` and `read(self: Stdin, ...)` can coexist,
  because calls are resolved by the static type of the value before the dot.
  There is no dynamic dispatch.
- Which types a project may declare `self` functions for, and how calls
  resolve across modules, is Open (O-10).

### 5.3 Generic Functions

```muga
fn first[T](items: List[T]): Option[T] {
  items.get(0)
}

let config = json.decode[Config](text)?
```

- Type arguments are inferred at the call site. When they cannot be
  inferred, they are written explicitly in brackets (D-06).
- There are no constraints. Which operations, such as `==`, are allowed on
  type parameters is Open (O-22).
- Records and enums cannot have type parameters (D-06).

### 5.4 Anonymous Functions

```muga
let adults = users.filter(fn(u) { u.age >= 18 })
```

- Anonymous functions use the `fn` keyword (D-19). Parameter types are
  inferred from the expected function type. Parameter annotations are written
  only when no expected type exists, such as when an anonymous function is
  bound to a local variable (Provisional).
- Anonymous functions capture values when they are created. Reassigning a
  captured variable inside an anonymous function is an error, so state is
  never shared through a closure (Provisional, follows from D-07).

## 6. Records

```muga
record User { name: String, age: Int }

let u = User { name: "a", age: 30 }
let older = u.with(age: u.age + 1)
```

- A record literal lists every field.
- `u.name` reads a field. Assigning to a field (`u.age = 31`) is an error. The
  suggested fix is `u = u.with(age: 31)` on a `let mut` binding (D-07).
- There are no tuples. Use a record to return several values (D-20).

## 7. Enums And `match`

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

- A variant carries no data, one value, or named fields. `Rect(Float, Float)`
  is an error that suggests named fields (D-21).
- `match` must be exhaustive. Its arms have the form `pattern => expr`.
- `match` is the only way to branch on variants. There is no `if let`
  (D-21).
- `_` matches anything (Provisional). Match guards are Open (O-24).
- Whether variants are written qualified (`Shape.Circle`) or unqualified
  (`Circle`) is Open (O-4).

## 8. Expressions And Control Flow

- `if cond { a } else { b }` is an expression. `cond` must be a `Bool`;
  nothing else counts as true or false.
- Loops (D-25):
  - `for x in list`
  - `for i in 0..n`, where the range excludes `n` (Provisional)
  - `for entry in map`, with `entry.key` and `entry.value` (D-20)
  - `while cond`
- Loops evaluate to `Unit`. Whether `break` and `continue` exist is Open
  (O-9).
- Operators:
  - `+ - * / %` require both operands to have the same numeric type (D-13).
    Integer division and remainder semantics are Open (O-17).
  - `== != < <= > >=` compare values of the same type.
  - `&& || !` operate on `Bool`.
  - `+` on `String` is an error that suggests interpolation (D-18).
- Discarding a value that is not `Unit` is an error (D-17). The syntax for an
  intentional discard is Open (O-7).
- Interpolation `${expr}` accepts `Int`, `Float`, `Bool`, and `String`
  (Provisional). Other types are Open (O-16).

## 9. Errors And Stopping

### 9.1 `Result` And `?`

```muga
fn load(files: ReadFiles): Result[Config] {
  let text = files.read("config.json").context("cannot read the config file")?
  json.decode[Config](text)
}
```

- `Result[T]` fails with `Error` (D-12). Values are built with `ok(value)` and
  `err("message")` (Provisional, O-4).
- `expr?` inside a function that returns `Result` returns the failure from
  the function (D-11).
- `?` on an `Option` is an error. Convert first with
  `opt.or_err("message")?` (D-25).
- `.context("message")` adds a context entry to a failing `Result`.
- An `Error` has a message, a kind, and a context chain. The kinds are
  `NotFound`, `PermissionDenied`, `InvalidData`, and `Other` (Provisional).

### 9.2 Stopping On Bugs

- Out-of-range list indexing, `Int` overflow, and division by zero stop the
  program (D-10). Whether an explicit `panic` or `unwrap` exists is Open
  (O-14).
- When the program stops, it writes the message, the source location, and
  the call stack to standard error. The exit status is Open (O-8).
- Code cannot catch a stop. The runtime still releases resources, such as
  open files.

## 10. Capabilities

### 10.1 Capability Values

- Capabilities are ordinary values (D-03). They can be passed as arguments,
  stored in records and lists, and captured by anonymous functions.
- User code cannot create a capability. Only the runtime (through `main`) and
  tests (through fakes, section 11) can (D-25). A top-level constant can
  never hold one.
- Narrowing returns a new capability with less authority. The original is
  unchanged:
  - `files.only("data/")` limits file access to a subtree. Path semantics,
    such as relative paths, `..`, and symbolic links, are Open (O-15).
  - `commands.only("git")` limits command execution to one program
    (Provisional).

### 10.2 `main`

```muga
fn main(files: ReadFiles, git: Command, out: Stdout): Result[Unit] {
  ...
}
```

- Every parameter of `main` must have a capability type. Each type may appear
  at most once, in any order, and the runtime supplies the values by type
  (Provisional).
- `main` returns `Result[Unit]` (Provisional, O-6). On `Err`, the runtime
  writes the error and its context chain to standard error and exits with
  status 1 (D-25).

### 10.3 Granting

```text
$ muga run tool.muga --read data/ --write out/ --exec git --env HOME -- arg1 arg2
```

| Capability | Flag (Provisional) | Granted by default |
|---|---|---|
| `ReadFiles` | `--read <path>` (repeatable) | no |
| `WriteFiles` | `--write <path>` (repeatable) | no |
| `Command` | `--exec <program>` (repeatable) | no |
| `Env` | `--env <NAME>` (repeatable) | no |
| `Stdin`, `Stdout`, `Stderr`, `Args` | none | yes |

- If `main` requires a capability that was not granted, the program does not
  start. The error lists each missing capability and its flag (D-04).
- Arguments after `--` reach the program through `Args`.
- There is no network capability in v1 (D-05).
- The methods of each capability (for example `read`, `write`, `list`, and
  `run`) belong to the standard library API, which is Open (O-1).

## 11. Tests

```muga
fn count_lines(files: ReadFiles, path: String): Result[Int] {
  let text = files.read(path)?
  ok(text.lines().len())
}

test "counts lines" {
  let files = ReadFiles.memory(...)   // Provisional, O-20
  assert_eq(count_lines(files, "a.txt")?, 2)
}
```

- `test "name" { ... }` is a top-level item in any file (D-16).
- A test body behaves like a function that returns `Result[Unit]`, so `?`
  works, and a failure fails the test (Provisional).
- `assert(cond)` and `assert_eq(actual, expected)` check results
  (Provisional).
- Fake capabilities, such as in-memory files or captured output, can only be
  created inside test blocks (D-16). Their API is Open (O-20).

## 12. Tools

| Command | Purpose |
|---|---|
| `muga run <file> [grants] [-- args]` | Check and run a program (D-04, D-15). |
| `muga check <file or dir>` | Report diagnostics without running. |
| `muga test [file or dir]` | Run `test` blocks. |
| `muga fmt [file or dir]` | Format source files. The formatting rules are Open (O-12). |

Each diagnostic is text with a code, a location, an explanation, and, when
possible, a suggested fix (D-15). The exact format is Open (O-13). The
compiler recognizes habits from other languages and suggests the Muga form
(D-08, D-17). The cases include:

| Written | Suggested Muga form |
|---|---|
| reassigning a `let` binding | declare it with `let mut` |
| `const x = ...` | `let x = ...` |
| `var x = ...`, `x := ...` | `let mut x = ...` |
| `null`, `nil`, `undefined` | `Option` with `None` |
| `"a" + b` | `"a${b}"` |
| `(a, b)` tuple | a record |
| `x => ...`, `\|x\| ...` | `fn(x) { ... }` |
| `if let` | `match` |
| prefix `try expr` | `expr?` |
| `v.push(1)` with the result discarded | `v = v.push(1)` |
| `f(x)` for a `self` function, `x.f()` for an ordinary function | the declared call style |
| `Rect(Float, Float)` variant | `Rect { w: Float, h: Float }` |

## 13. Implementation Constraints

- Rust, edition 2024, with `unsafe_code = "forbid"`, `warnings = "deny"`, and
  Clippy lints denied.
- No external crates.
- Pipeline: lexer, parser, name resolution, type checking, typed intermediate
  representation, tree-walking evaluator (D-23).
- Inside the implementation, errors propagate as values (`Result`). No
  non-local exit may skip cleanup. Resource handles carry generation numbers,
  so a stale handle fails safely instead of reaching another resource. These
  follow the runtime lessons in VISION.md.
- Nesting and recursion depth have limits that produce diagnostics instead of
  crashes (Provisional).
- Target size: 10,000 to 20,000 lines of Rust, excluding tests (VISION.md).

## Open Questions

Resolve these one at a time with the maintainer (see README.md), and record
each answer in DECISIONS.md. The task suite in PLAN.md decides which ones are
needed for v1.

| ID | Question |
|---|---|
| O-1 | The standard library API: the methods of each capability, and the functions on `List`, `Map`, `String`, `Option`, `Result`, and `json`. Settle them from the task suite. |
| O-2 | `Map`: literal syntax, allowed key types, iteration order (insertion order is proposed). |
| O-3 | The spelling of the `Unit` value (`unit` in the examples). |
| O-4 | Constructor and pattern spellings for `Option` and `Result` (`ok` vs `Ok`, `Some`/`None`), and whether variants are qualified (`Shape.Circle`). |
| O-5 | The syntax of function types, such as `fn(Int): Bool`. |
| O-6 | Whether `: Unit` may be omitted, and the exact allowed signatures of `main`. |
| O-7 | The syntax for intentionally discarding a non-`Unit` value, such as `let _ = expr`. |
| O-8 | Exit statuses for stopping on a bug and for `Err`, and whether programs can choose their exit status. |
| O-9 | Whether `break` and `continue` exist. |
| O-10 | Where `self` functions may be declared (only for the project's own types?) and how dot calls resolve across modules. |
| O-11 | Whether naming conventions are compiler errors. |
| O-12 | Formatter rules (indentation, line width, trailing commas). |
| O-13 | The diagnostic format, code scheme, and message language. |
| O-14 | Whether an explicit `panic` or `unwrap` exists. |
| O-15 | Capability path semantics: relative paths, `..`, symbolic links, and the operations `WriteFiles` covers (create, delete, rename). Also the arguments and environment of `Command`, and the shape of `Env`. |
| O-16 | Which types interpolation accepts, and how values are converted to text. |
| O-17 | Integer division and remainder with negative operands; `Float` formatting, NaN, and infinity. |
| O-18 | Which expressions top-level constants may use. |
| O-19 | The version number of the first redesign release. This is the maintainer's call under RELEASING.md. |
| O-20 | The API of test fakes, such as `ReadFiles.memory(...)` and capturing `Stdout`. |
| O-21 | Whether modules can live in subdirectories. |
| O-22 | Which operations are allowed on type parameters, such as `==`. |
| O-23 | Whether local variables may carry type annotations. |
| O-24 | Whether `match` supports guards. |
