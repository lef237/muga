# Invalid Example 009: Ambiguous Higher-Order Parameter

```txt
fn apply(x, f) {
  f(x)   // error: neither the argument type nor the result type is uniquely inferable
}
```

Expected failure:

- `f` is called as a function, but its full function type is not determined locally
- v1 still requires annotation when local inference does not determine a unique function type
