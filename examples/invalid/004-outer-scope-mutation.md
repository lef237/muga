# Invalid Example 004: Outer-Scope Mutation

```txt
mut total = 0

fn add_total(x: Int) {
  total = total + x   // error: updating an outer-scope mutable binding is prohibited
}
```

Expected failure:

- `total` is mutable, but it belongs to the outer scope
- v1 only allows updates to mutable bindings in the current scope
