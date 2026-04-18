# Valid Example 001: Basic Bindings

```txt
x = 1
mut total = 0
total = total + x
```

Why this is valid:

- `x = 1` introduces a new immutable binding
- `mut total = 0` introduces a new mutable binding
- `total = total + x` updates the mutable binding in the same scope
