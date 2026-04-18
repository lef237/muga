# Invalid Example 006: Recursive Function Without Required Annotation

```txt
fn fact(n) {
  if n == 0 {
    1
  } else {
    n * fact(n - 1)   # error: recursive function has no annotated parameter or return type
  }
}
```

Expected failure:

- the function is directly recursive
- v1 requires at least one annotated parameter or an explicit return type
