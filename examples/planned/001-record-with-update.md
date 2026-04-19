# Planned Example 001: Non-Destructive Record Update

```txt
record User {
  name: String
  age: Int
}

user = User {
  name: "Ada"
  age: 20
}

older = user.with(age: user.age + 1)
```

Why this is planned:

- `user` has record type `User`
- `age` is a declared field of `User`
- the replacement expression `user.age + 1` has type `Int`
- `with(...)` creates a new `User` value and preserves unspecified fields
