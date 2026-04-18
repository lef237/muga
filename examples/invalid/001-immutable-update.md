# Invalid Example 001: Immutable Update

```txt
x = 1
x = 2   # error: x is immutable in the current scope
```

Expected failure:

- `x = 1` creates an immutable binding
- the second assignment attempts to update that immutable binding
