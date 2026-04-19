# Invalid Example 010: Ambiguous Print Callback

```txt
fn show(x: Int, f) {
  print(f(x))   # error: `print` accepts several concrete types, so the callback result is not uniquely inferable
}
```

Expected failure:

- `x: Int` constrains the callback argument type
- the callback result is still ambiguous because `print` accepts `Int`, `Bool`, or `String`
- v1 therefore still requires an explicit arrow annotation here
