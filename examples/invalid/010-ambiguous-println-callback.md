# Invalid Example 010: Ambiguous Println Callback

```txt
fn show(x: Int, f) {
  println(f(x))   // error: `println` accepts several concrete types, so the callback result is not uniquely inferable
}
```

Expected failure:

- `x: Int` constrains the callback argument type
- the callback result is still ambiguous because `println` accepts `Int`, `Bool`, or `String`
- v1 therefore still requires an explicit arrow annotation here
