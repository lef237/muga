# Invalid Example 005: Ambiguous Identity Function

```txt
fn id(x) {
  x   // error: the parameter type and return type are not uniquely inferable
}
```

Expected failure:

- `x` is unconstrained
- v1 requires annotation when inference cannot determine a unique type
