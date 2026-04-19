# Invalid Example 008: Invalid Record Update

```txt
record User {
  name: String
  age: Int
}

user = User {
  name: "Ada"
  age: 20
}

bad = user.with(height: 170)   # error: `height` is not a field of `User`
```

Expected failure:

- `with(...)` is only valid for declared record fields
- `height` is not declared on `User`
