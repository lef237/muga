# Valid Example 006: Mutual Recursion With Explicit Signatures

```txt
fn is_even(n: Int): Bool {
  if n == 0 {
    true
  } else {
    is_odd(n - 1)
  }
}

fn is_odd(n: Int): Bool {
  if n == 0 {
    false
  } else {
    is_even(n - 1)
  }
}
```

Why this is valid:

- both functions have explicit signatures
- the recursive group can be typed before either body is checked
