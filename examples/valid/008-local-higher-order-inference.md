# Valid Example 008: Local Higher-Order Inference

```txt
fn apply(x: Int, f): Int {
  f(x)
}
```

Why this is valid:

- `x: Int` constrains the argument type of `f`
- `: Int` constrains the result type of `f(x)`
- the checker can infer `f: Int -> Int` using only information inside the same function
