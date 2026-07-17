# Error Catalog

This document defines the expected diagnostic categories for the current split
specification. The wording may vary by implementation, but the category and
trigger condition should remain stable within each release.

## Diagnostic Stability Policy

Muga treats diagnostics as part of the user-facing contract. Exact prose may improve, but diagnostics should keep:

- stable code families
- a primary source span when source is available
- related notes when a previous declaration, field declaration, package item, or artifact path explains the failure
- suggestions when a user can fix the issue with an annotation, import, visibility modifier, or artifact regeneration command

Current code families:

| Prefix | Area |
|---|---|
| `L` | lexing |
| `P` | parsing |
| `N` | unresolved names |
| `E` | core resolver/typechecker errors from the original catalog |
| `T` | detailed typing, enum, generic, collection, and control-flow checks |
| `R` | runtime/VM errors |
| `PK` | package, manifest, interface, cache, archive, and artifact workflow errors |
| `FMT` | formatter input and file IO errors |

When adding a new public diagnostic code or changing the trigger for an existing code, update this catalog and add or adjust a focused test.

### Planned Warning And Lint Contract

The current diagnostic model emits errors only; machine-readable output uses
`severity: "error"` for every diagnostic. Diagnostics should gain a
first-class severity and lint policy rather than encoding warnings as prose or
one-off command behavior.

The first warning candidates are:

- unused imports, local bindings, and parameters
- unreachable statements or expressions
- a `Result` value is discarded where failure is likely to be accidental

A similar-name lint is not a baseline requirement. First use the ordinary
unused warnings and real-program evidence to determine whether misspelled
updates routinely escape detection. Only if they do, consider a narrow warning
for a plain `name = value` introduction that differs by a small edit from an
earlier mutable binding in the same function. It should not compare every
identifier in scope, and it must remain configurable because intentional pairs
such as `user` / `users` or `item` / `items` are common.

The lint design must define stable lint identifiers, default levels,
allow/warn/deny configuration, a command-line `--deny-warnings`-style CI mode,
suppression scope, and JSON severity. Warnings must not change whether a program
is accepted unless their configured level is `deny`. The exact configuration
syntax is not yet committed.

The machine-readable diagnostic schema and CLI output envelope are defined by
the CLI `--format json` implementations and pinned by Rust tests. Tools should
use that JSON contract where available instead of scraping display text.
The CLI command `muga explain <diagnostic-code>` prints the matching catalog
entry below when one exists, or the documented diagnostic family for newer
detailed codes that share a stable prefix.

## E001: Immutable Update

Trigger:

- `x = e` where `x` is any immutable name in the current scope

This includes:

- immutable local bindings
- function bindings
- parameter bindings

Recommended message:

```txt
cannot update immutable binding `x`
```

Referenced fixtures:

- [conformance/current/rejecting/name-resolution/immutable_update.muga](./conformance/current/rejecting/name-resolution/immutable_update.muga)

## E002: Duplicate Binding In Current Scope

Trigger:

- `mut x = e` where `x` already exists in the current scope
- `fn f(...) { ... }` where `f` already exists in the current scope
- duplicate parameter names within one parameter list

Recommended message:

```txt
duplicate binding `x` in the current scope
```

Referenced fixtures:

- [conformance/current/rejecting/name-resolution/duplicate_mutable_binding.muga](./conformance/current/rejecting/name-resolution/duplicate_mutable_binding.muga)

## E003: Shadowing Prohibited

Trigger:

- introducing a new binding whose name already exists in an enclosing scope

This includes:

- `mut x = e` in an inner scope when `x` exists outside
- `x = e` in an inner scope when it would otherwise introduce a new immutable binding that collides with an enclosing immutable name
- function declarations and parameters that reuse an enclosing name

Recommended message:

```txt
shadowing is prohibited for `x`
```

Referenced fixtures:

- [conformance/current/rejecting/name-resolution/shadowing_in_block.muga](./conformance/current/rejecting/name-resolution/shadowing_in_block.muga)

## E004: Outer-Scope Mutation Prohibited

Trigger:

- `x = e` in an inner function scope where `x` resolves to a mutable binding in an outer function scope

Recommended message:

```txt
cannot update outer-scope mutable binding `x`
```

Referenced fixtures:

- [conformance/current/rejecting/name-resolution/outer_scope_mutation.muga](./conformance/current/rejecting/name-resolution/outer_scope_mutation.muga)

## E005: Annotation Required

Trigger:

- a function parameter type is not uniquely inferable
- a function return type is not uniquely inferable

Recommended message:

```txt
type annotation required because inference is not unique
```

Referenced fixtures:

- [conformance/current/rejecting/typing/ambiguous_identity.muga](./conformance/current/rejecting/typing/ambiguous_identity.muga)
- [conformance/current/rejecting/typing/ambiguous_higher_order_parameter.muga](./conformance/current/rejecting/typing/ambiguous_higher_order_parameter.muga)
- [conformance/current/rejecting/typing/ambiguous_println_callback.muga](./conformance/current/rejecting/typing/ambiguous_println_callback.muga)

## E006: Recursive Function Requires Annotation

Trigger:

- a directly recursive function has neither an annotated parameter nor an explicit return type

Recommended message:

```txt
recursive function requires at least one parameter or return type annotation
```

Referenced fixtures:

- [conformance/current/rejecting/typing/unannotated_recursion.muga](./conformance/current/rejecting/typing/unannotated_recursion.muga)

## E007: Mutual Recursion Requires Explicit Signatures

Trigger:

- a mutually recursive function group lacks explicit signatures

Recommended message:

```txt
mutually recursive functions require explicit signatures
```

Referenced fixtures:

- [conformance/current/rejecting/typing/unannotated_mutual_recursion.muga](./conformance/current/rejecting/typing/unannotated_mutual_recursion.muga)

## E008: Unknown Field

Trigger:

- `expr.name` where the static type of `expr` has no field `name`

Recommended message:

```txt
unknown field `name`
```

## E009: Invalid Record Literal

Trigger:

- a record literal omits a required field
- a record literal repeats a field
- a record literal contains an extra field
- a record literal field value has the wrong type

Recommended message:

```txt
invalid record literal for `TypeName`
```

## E010: Invalid Chained Dot Call

Trigger:

- `expr.name(args...)` where no applicable receiver-style or UFCS-style function resolution succeeds

Recommended message:

```txt
cannot resolve chained call `name`
```

## E011: Function-Valued Record Field Prohibited

Trigger:

- a record field is declared with a function type

Recommended message:

```txt
record fields may not have function type
```

## E012: Invalid Record Update

Trigger:

- `expr.with(...)` where `expr` does not have a record type
- a record update mentions an unknown field
- a record update repeats the same field name
- a record update supplies a value of the wrong type for a field

Recommended message:

```txt
invalid record update
```

Referenced fixtures:

- [conformance/current/rejecting/typing/invalid_record_update.muga](./conformance/current/rejecting/typing/invalid_record_update.muga)

## E013: Mutable Capture Across `spawn`

Trigger:

- an identifier inside a `spawn` operand resolves to a `mut` binding declared
  outside that `spawn` operand, for reads as well as writes

Recommended message:

```txt
cannot capture mutable binding `name` across `spawn`
```

Required guidance:

- attach a related note pointing at the mutable binding declaration
- suggest binding an immutable copy before `spawn` or passing the value in
  through a function argument

Referenced fixtures:

- [conformance/current/rejecting/name-resolution/spawn_mut_capture.muga](./conformance/current/rejecting/name-resolution/spawn_mut_capture.muga)

## Required Diagnostic Guidance

These diagnostic behaviors are part of the current language contract and are
checked by the release-quality gate even when the exact code is not one of the
original `E001`-`E012` examples. The contract changes together with its
specifications and current conformance fixtures.

### Ambiguity

Ambiguity diagnostics must say what annotation makes the program unique.

Required guidance:

- ambiguous `print` / `println` / `eprint` / `eprintln`: suggest annotating as `Int`, `Bool`, or `String`
- ambiguous `to_string`: suggest annotating as `Int`, `Bool`, or `String`
- ambiguous `len` / `is_empty`: suggest the supported `String`, `List[T]`, or `Map[K, V]` receiver shapes as applicable
- ambiguous `get`: suggest `List[T]` or `Map[K, V]`
- ambiguous `contains`: suggest `String` or `Map[K, V]`
- ambiguous `insert` / `remove`: suggest `Map[K, V]`
- ambiguous list indexing or `for item in list`: suggest `List[T]`
- ambiguous function signatures: suggest adding parameter and return type annotations until the signature is unique
- directly recursive functions: suggest adding a parameter type annotation or explicit return type
- mutually recursive functions: suggest adding parameter type annotations and explicit return types to every function in the group

### Imports And Standard Packages

Source uses import aliases, not full package paths inside type names. Diagnostics for source spellings such as `std::io::IOError` or `std::io::PathPairError` must point users toward:

```muga
import std::io
```

and then the local alias form:

```muga
io::IOError
io::PathPairError
```

## T026: Use After Consume

`T026` reports use-after-consume for a binding that has been passed directly to
a loaded-interface parameter marked `consume`.

Required guidance:

- point at the later use of the consumed binding
- attach a related note to the consuming call
- suggest avoiding any use of the binding after passing it to a consuming
  parameter

## T027: Invalid Using Cleanup

`T027` reports invalid `using` lexical cleanup. It covers non-handle
initializers, non-closeable opaque handles, invisible or malformed close
metadata, and explicit consuming calls such as `fs::close(file)` inside a
`using` block.

Required guidance:

- state that `using` requires a runtime-backed closeable opaque handle
- require a close function with a consuming handle parameter and
  `Result[Unit, E]` return type matching the enclosing function error
- attach a related note to the `using` binding when rejecting an explicit
  consume inside the block
- suggest letting `using` close the handle automatically

## T028: Invalid JSON/Config Validation Metadata

`T028` reports invalid JSON/config validation metadata on record fields. It
covers validators used with unsupported field types, negative string length
bounds, impossible minimum/maximum ranges, and conflicting duplicate validators.

Required guidance:

- name the unsupported or invalid validator
- point at the field attribute argument
- attach a related note for the earlier bound or duplicate validator when
  helpful
- suggest using string validators on `String` / `Option[String]` and numeric
  bounds on `Int` / `Option[Int]`

## T029: Unsupported JSON/Config Schema Export Target

`T029` reports unsupported JSON/config schema export targets or field types. It
covers attempts to export generic user records/enums, private or missing
targets, opaque handles, functions, maps with non-string keys, and public data
shapes containing types the first schema export slice cannot represent.

Required guidance:

- name the unsupported target or type
- point at the target declaration or field span when source is available
- suggest exporting a concrete public record or enum composed of `String`,
  `Int`, `Bool`, `Option`, `List`, `Map[String, T]`, `std::json::Value`, and
  supported concrete public records/enums

## T030: `spawn` Outside `group`

`T030` reports a `spawn` expression that is not inside the body of an
enclosing `group` expression in the same function. Function boundaries reset
the group context, including `fn` expression bodies; a `spawn` operand does
not.

Required guidance:

- point at the `spawn` expression
- say that `spawn` is allowed only inside a `group` block

Referenced fixtures:

- [conformance/current/rejecting/typing/spawn_outside_group.muga](./conformance/current/rejecting/typing/spawn_outside_group.muga)

## R022: Invalid Runtime Resource Handle

`R022` reports invalid runtime-backed opaque handle use that source checking did
not prove impossible, such as a stale, already-closed, or wrong-family
`std::fs::File` handle reaching the VM.

Required guidance:

- keep recoverable host IO failures in public `Result` error values
- use a hard runtime diagnostic for stale or already-closed handle aliases
- mention the invalid handle state in the primary message

## Package And Artifact Workflows

Artifact-backed commands are deliberately explicit. Diagnostics for package artifacts must:

- include the relevant package path when available
- include the concrete artifact path when available
- reject missing, stale, hash-mismatched, structurally invalid, wrong-package, or dependency-interface-mismatched artifacts
- avoid silently reading dependency implementation source bodies after an artifact failure
- suggest `muga build <entry>` for default `.muga/build` failures reached through `--built`
- suggest the focused explicit command where useful, such as `muga emit-interface`, `muga emit-check-cache`, or `muga emit-artifacts`

## Package Lockfiles And Archives

Manifest, lockfile, and `.mgp` archive diagnostics must fail loudly rather than silently rewriting unsafe state. This includes malformed lockfiles, unsupported dependency forms, missing archive hashes, path/archive form mistakes, stale archive caches, cache path collisions, package-name mismatches, archive hash mismatches, non-canonical archive layout, source-root escapes, non-UTF-8 paths, duplicate entries, and non-source archive entries.

Lockfile `muga_version` values must be `MAJOR.MINOR.PATCH` versions (pre-release and build metadata are accepted but ignored by comparison). A lockfile recorded by a newer compiler than the running one is rejected as `PK026` and left unmodified rather than reinterpreted or rewritten; the diagnostic must name both versions and suggest upgrading muga or deliberately deleting `muga.lock` and rebuilding. Same-or-older recorded versions are accepted and refreshed by the next successful build.

## Future Feature Syntax

Syntax reserved for future features should fail as unsupported or invalid syntax. It should not be documented or tested as runnable sample source until the feature is implemented. Examples include channels, `select`, optional chaining, postfix Result propagation, broad catch-all matching, references, and call-site type arguments. `group`, `spawn`, and `std::task::join` are implemented structured task group syntax, not future syntax; their diagnostics are `T030` and `E013` above.

# Lint diagnostics

## L001: named function call should use chained-call syntax

A named function with one or more value arguments was called using ordinary
call syntax. Move the first argument before the function name and use it as the
chain receiver. Zero-argument named functions, calls through function values,
and enum variant constructors keep ordinary-call syntax.

## L002: failed to write lint fix

`muga lint --fix` could not write a rewritten source file. Check the reported
path, permissions, and available storage, then run the command again.

## L003: enum constructor should use ordinary-call syntax

An enum constructor was written as a chained call. Move the receiver into the
constructor argument list. Enum construction is the canonical exception to
the named-function chained-call rule.
