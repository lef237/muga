# Valid Example 004: Inferred Parameter Type

```txt
fn double(x) {
  x * 2
}
```

Why this is valid:

- `x` is uniquely constrained by integer multiplication in the v1 typing model
- no explicit annotation is needed
