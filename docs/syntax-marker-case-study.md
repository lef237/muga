# Syntax Marker Case Study

Status: design note. This document is not a specification and does not define implemented behavior.

This note explains why Muga prefers giving each symbol one primary conceptual role.

The goal is not to criticize another language. The examples below show a real readability pressure that Muga should avoid when designing future syntax.

## 1. The Problem

Some languages use compact punctuation heavily.

That can be efficient for experienced users, but it can also make the same marker carry several different concepts depending on context.

Example:

```go
type Counter struct {
    value int
}

func (c *Counter) Inc(delta *int) {
    c.value += *delta
}

func NewCounter(initial int) *Counter {
    return &Counter{value: initial}
}

var x int
var p *int

p = &x
*p = 20

counter := NewCounter(0)
counter.Inc(&x)

product := x * 2
mask := flags & allowed
```

In this fragment:

- `(c *Counter)` uses `*` in a receiver type
- `delta *int` uses `*` in a parameter type
- `NewCounter(...) *Counter` uses `*` in a return type
- `*int` uses `*` as part of a pointer type
- `*delta` and `*p` use `*` as dereference operations
- `x * 2` uses `*` as multiplication
- `return &Counter{...}` uses `&` to create an address from a composite value
- `&x` uses `&` as address creation
- `flags & allowed` uses `&` as bitwise AND

These forms are learnable, and they are useful in the language that uses them. The issue for Muga is different: Muga wants code to be understood locally with low symbolic load.

For a new reader, the same marker switching between receiver type, parameter type, return type, value access, arithmetic, address creation, and bit operations increases the amount of context they must hold in their head.

The pressure is stronger in function signatures because the symbol appears in both API shape and implementation details:

```go
func (c *Counter) Inc(delta *int)
func NewCounter(initial int) *Counter
counter.Inc(&x)
```

A reader has to map:

- receiver type is pointer-like
- parameter type is pointer-like
- return type is pointer-like
- call site passes an address-like value

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

## 3. Hypothetical Bad Muga Design

The following is not current Muga syntax.

It shows the kind of design Muga should avoid:

```muga
value: *Int = &count
next = *value + 1
mask = flags & allowed

fn inc(self: *Counter, delta: *Int): *Counter {
  *self.value = *self.value + *delta
  self
}

counter = inc(&counter, &count)
```

Problem:

- `*Int` would make `*` a type constructor
- `*value` would make `*` a value operation
- `*` would already be multiplication
- `&count` would make `&` address creation
- `flags & allowed` would make `&` bitwise AND
- `self: *Counter` would put the same marker into receiver-style API shape
- `delta: *Int` would put the same marker into parameter API shape
- `: *Counter` would put the same marker into return type API shape
- `inc(&counter, &count)` would put the paired marker at the call site

This is compact, but it puts several unrelated jobs onto the same visual markers.

That is not the direction Muga should take.

## 4. Better Muga Direction

If Muga later adds pointer-like, reference-like, ownership, or borrowing concepts, prefer a design where type names and value operations remain visibly distinct.

The current reference draft prefers non-escaping read-only `ref T` for ordinary borrowed parameters. Muga should not introduce `Ref[T]` as a second spelling for the same concept.

Preferred borrowed-parameter direction:

```muga
mask = flags.bit_and(allowed)

fn add_delta(counter: Counter, delta: Int): Counter {
  next_value = counter.value + delta
  counter.with(value: next_value)
}

fn next_value(counter: ref Counter, delta: Int): Int {
  counter.value + delta
}

fn inc(counter: ref Counter, delta: Int): Counter {
  updated_value = counter.next_value(delta)
  counter.with(value: updated_value)
}

counter = counter.inc(count)
```

This is longer than dense punctuation, but it is easier to read:

- `ref Counter` clearly marks a borrowed parameter
- the call site does not need address syntax such as `&counter`
- field access still reads as `counter.value`
- `counter.with(...)` keeps updates non-destructive
- `updated_value` makes the data flow explicit
- `counter.next_value(...)` keeps the chain readable because the step is named
- `flags.bit_and(allowed)` clearly performs a bit operation

This fits Muga's chained-call style: a chain is encouraged when each step is a small, named transformation.

More explicit equivalent:

```muga
mask = bit_and(flags, allowed)

fn next_value(counter: ref Counter, delta: Int): Int {
  counter.value + delta
}

fn inc(counter: ref Counter, delta: Int): Counter {
  updated_value = next_value(counter, delta)
  counter.with(value: updated_value)
}

counter = inc(counter, count)
```

This is also acceptable, but Muga should generally prefer the chained form when it reads naturally.

The important style rule is to avoid hiding several operations inside one nested expression.

Avoid dense chains that combine too many concerns:

```muga
counter = counter.with(value: counter.value + count).render().println()
```

This uses chaining, but it is not the desired style because it combines update construction, rendering, and output in one expression.

Prefer:

- name intermediate values
- split transformation logic into small functions
- keep borrowed parameters visible in function signatures
- use chaining when it reads as a clear sequence of named operations

These examples are not final syntax. They illustrate the design constraint: avoid reusing one symbol for unrelated concepts.

For the current borrow direction, see [spec/010-references-draft.md](../spec/010-references-draft.md).

## 5. Design Checklist

Before adding a new symbol or operator, ask:

1. What is the symbol's primary conceptual role?
2. Does this reuse an existing symbol for an unrelated concept?
3. Would a beginner understand the expression without knowing several context-specific meanings?
4. Could a keyword or named function be clearer?
5. Can diagnostics explain the syntax in one sentence?
6. Does this keep parser and typechecker rules simple?

If the answer is unclear, prefer a more explicit form first.

Muga can add shorter sugar later, once the concept is stable.
