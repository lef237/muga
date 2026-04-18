# Invalid Example 007: Mutual Recursion Without Signatures

```txt
fn is_even(n) {
  if n == 0 {
    true
  } else {
    is_odd(n - 1)   # error: mutually recursive group lacks explicit signatures
  }
}

fn is_odd(n) {
  if n == 0 {
    false
  } else {
    is_even(n - 1)  # error: mutually recursive group lacks explicit signatures
  }
}
```

Expected failure:

- `is_even` and `is_odd` form a mutually recursive group
- v1 requires explicit signatures for mutual recursion
