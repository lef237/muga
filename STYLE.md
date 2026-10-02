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
- named calls with explicit call-site type arguments, until chained call-site
  type arguments are supported
- calls through function values, such as `callback(value)`
- enum variant constructors, such as `Result::Ok(value)`

Run `muga lint <source-file>` to check this rule. `muga check` continues to
accept both call forms because ordinary calls remain part of the language.
Run `muga lint --fix <source-file>` to rewrite eligible calls and format the
changed source files while preserving line comments.

Suppress one lint code for calls that begin on the next physical line with an
explicit comment:

```muga
// muga-lint: allow-next-line S001 -- arguments are intentionally symmetric
result = equals(expected, actual)
```

The suppression applies only to the named code and only to the immediately
following line. Both `muga lint` and `muga lint --fix` honor it. A reason after
`--` is optional but recommended.

## Lint Levels

`muga lint` also reports warning lints for likely mistakes: unused imports,
bindings, and parameters, unreachable code, and discarded `Result` values
(`W001` to `W005`). Warnings do not fail the command; the style lints `S001` and
`S003` do. Change levels with `--allow`, `--warn`, and `--deny`, and use
`--deny-warnings` in CI:

```bash
muga lint --deny-warnings path/to/main.muga
muga lint --allow W003 --deny W005 path/to/main.muga
```

Start a binding or parameter name with `_`, or write `_ = expr`, when a value
is intentionally unused. The full contract and every lint code are listed in
[errors.md](./errors.md#warning-and-lint-contract).
