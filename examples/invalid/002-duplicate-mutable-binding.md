# Invalid Example 002: Duplicate Mutable Binding

```txt
mut y = 1
mut y = 2   # error: duplicate binding in the current scope
```

Expected failure:

- `mut` always introduces a new binding
- the second declaration reuses the same name in the same scope
