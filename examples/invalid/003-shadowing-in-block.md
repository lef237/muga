# Invalid Example 003: Shadowing In Block

```txt
flag = true
value = 1

if flag {
  value = 2   // error: shadowing of outer immutable binding is prohibited
} else {
  value
}
```

Expected failure:

- the inner block has no local `value`
- using `value = 2` there would introduce a new binding
- that new binding would shadow the outer `value`, which is forbidden in v1
