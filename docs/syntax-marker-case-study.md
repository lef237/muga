# Syntax Marker Case Study

Status: design note. This document is not a specification and does not define implemented behavior.

This note explains why Muga prefers giving each symbol one primary conceptual role.

The goal is not to criticize another language or language community. The examples below use hypothetical Muga-like syntax to show a readability pressure that Muga should avoid when designing future features.

## 1. The Problem

Compact punctuation can be efficient for experienced users, but it can also make the same marker carry several different concepts depending on context.

The following is pointer-style pseudocode, not current Muga syntax.

It shows the kind of design Muga should avoid. The example deliberately mixes a few small tasks:

- read an integer and compute the next value
- add a delta to a counter
- increment a counter by one
- compute a bit mask

```muga
record Counter {
  value: Int
}

fn add_delta(counter: *Counter, delta: *Int): *Counter {
  next_value = counter.value + *delta
  next = Counter {
    value: next_value
  }
  &next
}

fn inc(counter: *Counter): *Counter {
  one = 1
  add_delta(counter, &one)
}

count = 3
delta = 4
counter = Counter {
  value: 0
}

flags = 12
allowed = 4

count_ref: *Int = &count
next_count = *count_ref + 1
product = count * 2
mask = flags & allowed

next_counter = add_delta(&counter, &delta)
incremented_counter = inc(next_counter)
result = incremented_counter.value + next_count + product + mask
```

In this fragment:

- `*Int` and `*Counter` make `*` part of pointer type syntax
- `*count_ref` and `*delta` make `*` a value operation
- `count * 2` makes `*` multiplication
- `&count` makes `&` address creation
- `flags & allowed` makes `&` bitwise AND
- `counter: *Counter` puts the same marker into first-parameter API shape
- `delta: *Int` puts the same marker into parameter API shape
- `: *Counter` puts the same marker into return type shape
- final `&next` returns an address-like value
- `add_delta(&counter, &delta)` puts the paired marker at the call site

For a new reader, the same marker switching between first-parameter type, parameter type, return type, value access, arithmetic, address creation, and bit operations increases the amount of context they must hold in their head.

The pressure is stronger in function signatures because the symbol appears in both API shape and implementation details:

```muga
fn add_delta(counter: *Counter, delta: *Int): *Counter
next_counter = add_delta(&counter, &delta)
```

A reader has to map:

- first input type is pointer-like
- parameter type is pointer-like
- return type is pointer-like
- call site passes an address-like value
- implementation body may dereference values or return an address-like value

This is compact, but the relationship between API design and call-site behavior is not visually self-explanatory.

Muga should avoid that style when designing new syntax.

## 2. Muga Principle

Muga should prefer:

- one primary conceptual role per symbol
- explicit words when punctuation would become overloaded
- stable syntax over dense syntax
- local readability over compactness
- diagnostics that can explain syntax without many context-dependent exceptions

This does not mean punctuation is banned.

It means punctuation should stay conceptually stable.

Examples already aligned with this rule:

```muga
fn inc(x: Int): Int {
  x + 1
}
```

Here:

- `:` marks type annotation positions
- `+` is numeric addition

```muga
package app::main

import util::numbers

fn main(): Int {
  numbers::inc_twice(10)
}
```

Here:

- `::` marks package-qualified names
- it is not also used for field access or arithmetic

```muga
user.name
user.display_name()
```

Here:

- `expr.name` is field access
- `expr.name(...)` is chained-call surface syntax
- function-valued record fields are not allowed in v1, so this syntax does not also mean field-function call

## 3. Comparable Muga Direction

If Muga later adds pointer-like, reference-like, ownership, or borrowing concepts, prefer a design where type names and value operations remain visibly distinct.

The current reference draft prefers non-escaping read-only `ref T` for ordinary borrowed parameters. Muga should not introduce `Ref[T]` as a second spelling for the same concept.

The comparable Muga-shaped version of the earlier example keeps the same small tasks and uses Muga naming:

```muga
record Counter {
  value: Int
}

count = 3
delta = 4
counter = Counter {
  value: 0
}

flags = 12
allowed = 4

next_count = count + 1
product = count * 2
mask = flags.bit_and(allowed)

fn add_delta(counter: ref Counter, delta: Int): Counter {
  next_value = counter.value + delta
  counter.with(value: next_value)
}

fn inc(counter: ref Counter): Counter {
  counter.add_delta(1)
}

next_counter = counter.add_delta(delta)
incremented_counter = next_counter.inc()
result = incremented_counter.value + next_count + product + mask
```

This is longer than dense punctuation, but it is easier to read:

- `Counter`, `counter`, `count`, and `delta` are introduced before use
- `Int`, `Counter`, and `ref Counter` remain type syntax rather than symbolic pointer syntax
- the call site does not need address syntax such as `&counter`
- the implementation body does not need dereference syntax such as `*counter`
- field access still reads as `counter.value`
- `counter.with(...)` keeps updates non-destructive
- `flags.bit_and(allowed)` keeps bit operations named instead of reusing `&`
- `counter.add_delta(delta)` and `next_counter.inc()` keep the chain readable because each step is named
- the final `result` expression has the same shape without address or dereference syntax

The main benefit is that pointer-like representation does not leak into ordinary value flow.

The caller writes `counter.add_delta(delta)` rather than manufacturing address-like values with `&counter` and `&delta`. The callee signature says `counter: ref Counter`, so the borrowing relationship is still visible where the API is defined. Inside the function, reads use ordinary field access such as `counter.value`, not explicit dereference syntax. The return type is `Counter`, so the caller receives an ordinary value instead of having to reason about what a returned `*Counter` points to.

This fits Muga's chained-call style: a chain is encouraged when each step is a small, named transformation.

More explicit ordinary-call equivalent:

```muga
next_counter = add_delta(counter, delta)
incremented_counter = inc(next_counter)
result = incremented_counter.value + next_count + product + mask
```

This is also acceptable, but Muga should generally prefer the chained form when it reads naturally.

### Comparison Map

The examples above are intended to line up concept by concept:

| Avoid | Prefer | Reason |
| --- | --- | --- |
| `count_ref: *Int = &count` and `*count_ref` | `next_count = count + 1` | ordinary reads should not need address and dereference markers |
| `count * 2` | `count * 2` | `*` stays available for multiplication |
| `fn add_delta(counter: *Counter, delta: *Int): *Counter` | `fn add_delta(counter: ref Counter, delta: Int): Counter` | borrowed input and returned value are visibly different concepts |
| final `&next` | `counter.with(value: next_value)` | ordinary transformations return values instead of addresses |
| `counter.value + *delta` | `counter.value + delta` | field access stays stable and `delta` stays an ordinary value |
| `fn inc(counter: *Counter): *Counter` | `fn inc(counter: ref Counter): Counter` | the same borrowing rule applies to receiver-style APIs without pointer return syntax |
| `add_delta(&counter, &delta)` | `add_delta(counter, delta)` or `counter.add_delta(delta)` | call sites should pass values without address punctuation for ordinary borrowing |
| `inc(next_counter)` | `inc(next_counter)` or `next_counter.inc()` | the increment step has the same shape as any other named transformation |
| `flags & allowed` | `flags.bit_and(allowed)` | bit operations should not reuse address syntax |
| `result = incremented_counter.value + next_count + product + mask` | `result = incremented_counter.value + next_count + product + mask` | after the noisy setup is removed, the final data use can stay structurally identical |

`record.with(...)` and chained calls are implemented today. `ref T` and any `bit_and` operation are design directions, not implemented v1 behavior.

If Muga later adds bit operations, a named operation is clearer than spending `&` on another unrelated meaning:

```muga
flags = 12
allowed = 4
mask = flags.bit_and(allowed)
```

The important style rule is not "avoid chains".

Muga should encourage chains when each step stays in the same conceptual flow.

Good chain:

```muga
final_counter = counter.inc().add_delta(10).inc()
```

This reads as a sequence of named `Counter -> Counter` transformations.

Be more careful when a chain crosses abstraction boundaries:

```muga
counter.inc().add_delta(10).inc().value.println()
```

This combines state transformation, field extraction, and output in one expression. That may be acceptable in small examples, but it is not the style Muga should use to explain core design rules.

Prefer:

```muga
final_counter = counter.inc().add_delta(10).inc()
result = final_counter.value
println(result)
```

Prefer:

- name intermediate values
- split transformation logic into small functions
- keep borrowed parameters visible in function signatures
- use chaining when it reads as a clear sequence of named operations
- break the chain when it switches from transformation to extraction or side effect

These examples are not final syntax. They illustrate the design constraint: avoid reusing one symbol for unrelated concepts.

For the current borrow direction, see [spec/010-references-draft.md](../spec/010-references-draft.md).

## 4. Design Checklist

Before adding a new symbol or operator, ask:

1. What is the symbol's primary conceptual role?
2. Does this reuse an existing symbol for an unrelated concept?
3. Would a beginner understand the expression without knowing several context-specific meanings?
4. Could a keyword or named function be clearer?
5. Can diagnostics explain the syntax in one sentence?
6. Does this keep parser and typechecker rules simple?

If the answer is unclear, prefer a more explicit form first.

Muga can add shorter sugar later, once the concept is stable.
