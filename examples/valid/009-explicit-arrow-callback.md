# Valid Example 009: Explicit Arrow Callback

```txt
fn show(x: Int, f: Int -> String): String {
  print(f(x))
}
```

Why this is valid:

- the arrow annotation states the callback contract explicitly
- this style is useful when local inference would otherwise be weak or when the function is meant to be a clear interface
