# Valid Example 005: Recursive Function With One Annotation

```txt
fn fact(n: Int) {
  if n == 0 {
    1
  } else {
    n * fact(n - 1)
  }
}
```

Why this is valid:

- the function is directly recursive
- one parameter annotation is present, which satisfies the v1 recursion rule
