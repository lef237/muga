# Muga Style

## Canonical Call Syntax

Named functions with one or more value arguments use chained-call syntax. The
first argument becomes the receiver and the remaining arguments keep their
order:

```muga
value.transform()
value.combine(other)
value.package::transform(other)
```

These are the canonical spellings of the corresponding ordinary calls
`transform(value)`, `combine(value, other)`, and
`package::transform(value, other)`.

Ordinary-call syntax remains canonical for:

- zero-argument named functions, such as `now()`
- calls through function values, such as `callback(value)`
- enum variant constructors, such as `Result::Ok(value)`

Run `muga lint <source-file>` to check this rule. `muga check` continues to
accept both call forms because ordinary calls remain part of the language.
