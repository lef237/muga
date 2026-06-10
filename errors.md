# Error Catalog v1

This document defines the expected diagnostic categories for the v1 split specification. The wording may vary by implementation, but the category and trigger condition should remain stable.

## Diagnostic Stability Policy

Muga v1 treats diagnostics as part of the user-facing contract. Exact prose may improve, but v1 diagnostics should keep:

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
| `E` | core resolver/typechecker errors that correspond to the original v1 examples |
| `T` | detailed typing, enum, generic, collection, and control-flow checks |
| `R` | runtime/VM errors |
| `PK` | package, manifest, interface, cache, archive, and artifact workflow errors |
| `FMT` | formatter input and file IO errors |

When adding a new public diagnostic code or changing the trigger for an existing code, update this catalog and add or adjust a focused test.

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

Referenced examples:

- [examples/invalid/001-immutable-update.md](./examples/invalid/001-immutable-update.md)

## E002: Duplicate Binding In Current Scope

Trigger:

- `mut x = e` where `x` already exists in the current scope
- `fn f(...) { ... }` where `f` already exists in the current scope
- duplicate parameter names within one parameter list

Recommended message:

```txt
duplicate binding `x` in the current scope
```

Referenced examples:

- [examples/invalid/002-duplicate-mutable-binding.md](./examples/invalid/002-duplicate-mutable-binding.md)

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

Referenced examples:

- [examples/invalid/003-shadowing-in-block.md](./examples/invalid/003-shadowing-in-block.md)

## E004: Outer-Scope Mutation Prohibited

Trigger:

- `x = e` in an inner function scope where `x` resolves to a mutable binding in an outer function scope

Recommended message:

```txt
cannot update outer-scope mutable binding `x` in v1
```

Referenced examples:

- [examples/invalid/004-outer-scope-mutation.md](./examples/invalid/004-outer-scope-mutation.md)

## E005: Annotation Required

Trigger:

- a function parameter type is not uniquely inferable
- a function return type is not uniquely inferable

Recommended message:

```txt
type annotation required because inference is not unique
```

Referenced examples:

- [examples/invalid/005-ambiguous-identity.md](./examples/invalid/005-ambiguous-identity.md)

## E006: Recursive Function Requires Annotation

Trigger:

- a directly recursive function has neither an annotated parameter nor an explicit return type

Recommended message:

```txt
recursive function requires at least one parameter or return type annotation
```

Referenced examples:

- [examples/invalid/006-unannotated-recursion.md](./examples/invalid/006-unannotated-recursion.md)

## E007: Mutual Recursion Requires Explicit Signatures

Trigger:

- a mutually recursive function group lacks explicit signatures

Recommended message:

```txt
mutually recursive functions require explicit signatures in v1
```

Referenced examples:

- [examples/invalid/007-unannotated-mutual-recursion.md](./examples/invalid/007-unannotated-mutual-recursion.md)

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
record fields may not have function type in v1
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

## Required V1 Guidance

These diagnostic behaviors are part of the v1 release gate even when the exact code is not one of the original `E001`-`E012` examples.

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

## Future Feature Syntax

Syntax reserved for post-v1 features should fail as unsupported or invalid v1 syntax. It should not be documented or tested as runnable sample source until the feature is implemented. Examples include `group`, `spawn`, channels, optional chaining, postfix Result propagation, broad catch-all matching, references, and call-site type arguments.
