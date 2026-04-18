# Valid Example 002: Read From Outer Scope

```txt
base = 10

fn plus_base(x: Int) {
  x + base
}
```

Why this is valid:

- `base` is read from an outer scope
- the function does not attempt to update `base`
