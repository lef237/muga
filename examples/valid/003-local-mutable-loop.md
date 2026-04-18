# Valid Example 003: Local Mutable Loop

```txt
fn sum_to(n: Int) {
  mut i = 0
  mut acc = 0

  while i < n {
    acc = acc + i
    i = i + 1
  }

  acc
}
```

Why this is valid:

- `i` and `acc` are mutable bindings local to the function scope
- all updates stay within the scope where those bindings were introduced
