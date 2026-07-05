# Explicit References Decision Note

Status: not planned for ordinary Muga code.

This document records the decision not to pursue explicit source-level references as a normal language feature.

The active value and performance direction is defined in [011-value-semantics.md](./011-value-semantics.md).

## Decision

Muga should not plan these forms for ordinary source code:

- `ref T`
- `mut ref T`
- `&value`
- `*T`
- `*value`
- general pointer arithmetic
- general writable aliases

This applies to both read-oriented and write-oriented APIs.

## Rationale

Muga prioritizes:

- readability
- low syntactic overhead
- strong static typing
- fast compilation
- high runtime performance
- understandable concurrency

Explicit references add a second way to reason about most values. Read-only references are simpler than writable references, but even read-only references introduce questions about escaping, generics, function types, package interfaces, closures, and task capture.

Writable references are much heavier. They require rules for:

- exclusive access
- aliasing
- assignment through references
- interaction with immutable bindings
- interaction with field access
- interaction with closures
- interaction with structured concurrency
- diagnostics for conflicting access

Those rules may be worth it in a systems language. They are not currently a good fit for Muga's ordinary application-facing syntax.

## Preferred Alternatives

### Ordinary Data

Use value-returning updates:

```txt
next = user.with(age: user.age + 1)
items = items.push(item)
```

The compiler may lower these to in-place updates when a value is uniquely owned and the transformation is not observable.

### Repeated Construction

Use builder or buffer types:

```txt
mut builder = StringBuilder.new()
builder = builder.push("hello")
builder = builder.push(" world")
builder.to_string()
```

For byte or text buffers, use the same value-update style:

```txt
mut buf = Buffer.empty()
buf = buf.append("hello")
buf = buf.append(" world")
```

The source remains value-oriented, while the implementation can keep storage efficient.

### External Effects

Use resource or handle types:

```txt
writer = file_writer("out.txt")
writer.write("hello")
writer.flush()
```

The side effect belongs to the resource API. It does not require general mutable references in the language.

## Performance Position

Removing explicit references from ordinary source code does not block high performance.

The compiler/runtime may still use pointers internally for:

- passing large values without copying
- shared immutable storage
- resource handles
- stack allocation
- escape analysis
- scalar replacement
- copy elision
- destructive update lowering when a value is uniquely owned
- efficient native ABI decisions

The important distinction is:

- source code keeps value semantics
- implementation may choose reference-like representations internally

## Reconsideration Criteria

Explicit references should be reconsidered only if all of these become true:

1. Benchmarks show that internal sharing, copy elision, destructive update lowering, resource handles, and builder/buffer types are not enough.
2. The needed API cannot be expressed cleanly as a value-returning update or resource operation.
3. The proposed feature can remain small, readable, and local.
4. The diagnostics are understandable.
5. The feature does not require whole-program alias or lifetime inference.
6. The feature does not compromise structured concurrency safety.

Until then, explicit source-level references should remain out of ordinary Muga.
