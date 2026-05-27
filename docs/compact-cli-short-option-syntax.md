Status: compact CLI short option syntax implemented

# Compact CLI Short Option Syntax

Muga already supports field-level `@cli(short: "x")` metadata for typed CLI
schemas, including exact short forms such as `-x`, `-x value`, and `-x=value`.
The remaining short-option ergonomics gap is compact token syntax: familiar
forms such as `-abc`, `-ofile`, and `-abo=value`.

This slice implements compact short-token behavior in the compiler-owned typed
CLI schema parsers without adding new schema metadata.

## Goals

Short-Term Goal: implement combined bool short flags and attached short values
without changing existing exact short option behavior.

Medium-Term Goal: make Muga CLIs feel normal to users who expect Unix-style
short options while keeping generated usage, help, request workflows, and
artifacts on the existing `CliSchema` contract.

Long-Term Goal: keep the short-option grammar stable before subcommands and
generated app shell completions depend on CLI schemas.

Final Goal: make Muga command-line tools practical enough to publish while
preserving explicit data contracts and app-owned effects.

## Scope

The compact syntax applies to compiler-owned typed schema parsing:

- `cli::parse[T](args)`;
- `cli::parse_or[T](args, defaults)`;
- `cli::parse_request[T](args, program)`;
- `cli::parse_request_or[T](args, program, defaults)`.

It does not add any new `@cli(...)` metadata, interface payloads, `.mgi`
payloads, `CliSchema` fields, or `.mgb` instruction schema payloads. Existing
short names remain the source of truth.

## Accepted Forms

Existing accepted forms stay valid:

- long separated values: `--name value`;
- long inline values: `--name=value`;
- exact short separated values: `-n value`;
- exact short inline values: `-n=value`;
- exact bare bool short flags: `-v`;
- exact explicit bool values: `-v=false` and `-v false`;
- `--` stops option parsing and leaves later tokens as positionals.

New compact forms:

- combined bare-bool flags: `-abc`;
- attached values: `-ofile`;
- bool prefix plus attached value: `-abofile`;
- explicit value on the final short: `-o=value`;
- bool prefix plus explicit final value: `-abo=value`.

## Token Rules

A compact short token starts with one `-`, is not `--`, and has at least one
character after the dash.

The runtime parser should split the token after the leading dash on the first
`=`:

- `-abc` has short run `abc` and no explicit value;
- `-abo=value` has short run `abo` and explicit value `value`.

Short names continue to be one ASCII letter because `@cli(short: "...")`
already validates that contract. The parser therefore walks the short run as
ASCII characters and resolves each character through existing schema short
metadata.

### Without `=`

For tokens without `=`:

1. Walk the short run from left to right.
2. If the current short field accepts a bare bool value, merge `true` for that
   field and continue.
3. If the current short field does not accept a bare bool value and there are
   remaining characters in the token, treat the remaining suffix as that
   field's attached value and stop walking the token.
4. If the current short field does not accept a bare bool value and there is no
   remaining suffix, fall back to the existing separated-value behavior:
   consume the next token only when it does not look like an option marker;
   otherwise report `MissingValue`.

Examples:

| Token | Schema sketch | Meaning |
|---|---|---|
| `-abc` | `a: Bool`, `b: Bool`, `c: Bool` | `a=true`, `b=true`, `c=true` |
| `-vvv` | `v: List[Bool]` | `v=[true, true, true]` |
| `-abofile` | `a: Bool`, `b: Bool`, `o: String` | `a=true`, `b=true`, `o="file"` |
| `-n3` | `n: Int` | `n=3` |
| `-o-file` | `o: String` | `o="-file"` |
| `-abn 3` | `a: Bool`, `b: Bool`, `n: Int` | `a=true`, `b=true`, `n=3` |

`Bool`, `Option[Bool]`, and `List[Bool]` count as bare-bool fields. They do not
consume an attached suffix without `=`. For example, `-vfalse` is parsed as
`-v -f -a -l -s -e` where those short names exist, or reports the first
unknown short. Users who want an explicit bool value should write `-v=false` or
the exact separated form `-v false`.

### With `=`

For tokens with `=`:

1. Every short before the final short in the run must accept a bare bool value.
2. The explicit value after `=` is parsed as the final short field's value.
3. The final short may be any supported field type, including `Bool`,
   `Option[Bool]`, or `List[Bool]`.

Examples:

| Token | Schema sketch | Meaning |
|---|---|---|
| `-o=report.txt` | `o: String` | `o="report.txt"` |
| `-abo=report.txt` | `a: Bool`, `b: Bool`, `o: String` | `a=true`, `b=true`, `o="report.txt"` |
| `-vv=false` | `v: List[Bool]` | `v=[true, false]` |
| `-abn=3` | `a: Bool`, `b: Bool`, `n: Int` | `a=true`, `b=true`, `n=3` |

If a non-final short does not accept a bare bool value, the parser reports
`MissingValue` for that short. This keeps value-taking options from silently
swallowing later compact flags.

## Diagnostics

Diagnostics should preserve the existing public `cli::Error` surface:

- unknown compact short names report `UnknownArgument`;
- missing separated values report `MissingValue`;
- invalid attached or explicit values report `InvalidValue`;
- validation failures continue to report `Validation`.

The `argument` field should identify the smallest actionable option spelling:

- `-x` for an unknown short `x` inside `-abx`;
- `-o` for a missing value in `-abo`;
- `-n` for an invalid attached Int in `-abnxyz`.

The diagnostic message should name that same spelling, for example:

```text
unknown CLI option `-x`
```

and:

```text
missing value for `-o`
```

## Help And Request Workflow Boundary

`cli::help_requested(args)` and the request helpers continue to recognize exact
`--help` and exact `-h` before `--`. This slice does not make `-hV` a help
request because the low-level help predicate intentionally has no schema
context. Request helpers already reserve short `h` in their schemas, so exact
`-h` remains the supported built-in help spelling.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| One compact grammar for combined bool flags and attached values | Handles `-abc`, `-ofile`, and `-abo=value` together, avoiding two incompatible short-token policies. Requires only runtime parser/tests/docs because short metadata already exists. | Needs explicit disambiguation between bool clusters and value-taking attached suffixes. | Select |
| Combined bool flags only | Very small and safe for `Bool` fields. | Leaves `-ofile` unsupported even though attached values are the other common short-token expectation. | Reject |
| Attached values only | Helps value options such as `-oout.txt`. | Leaves `-abc` unsupported and makes future bool clusters ambiguous. | Reject |
| Treat `-vfalse` as `-v=false` | Convenient for bool fields. | Conflicts with the cluster grammar and short names `f`, `a`, `l`, `s`, `e`; users already have explicit `-v=false`. | Reject |
| Schema metadata for cluster behavior | Could allow per-field policy. | Adds schema surface for what should be deterministic token syntax. | Reject |
| Extend built-in help detection to compact `-h...` tokens | Familiar in some CLIs. | `cli::help_requested` has no schema context and would make short `h` globally magical outside request helpers. | Defer |
| Subcommands or shell completion generation first | High value for larger tools. | Both depend on stable CLI token behavior; compact short syntax is a smaller prerequisite. | Defer |

## Non-Goals

This design does not add:

- new `@cli(...)` metadata;
- short aliases separate from the primary short name;
- short names for positional fields or subcommands;
- compact long-option syntax;
- schema-aware `cli::help_requested` behavior for `-h...`;
- subcommands or nested command dispatch;
- shell completion generation for generated apps;
- TOML/config discovery automation;
- runtime-owned printing/exits or process-status APIs;
- full client generation, generic encoding/decoding, broader validators, or
  host-effect APIs.

## Implementation Plan

1. Done: implement field-level `@cli(short: "...")` metadata and exact short
   option parsing.
2. Done: audit parse-integrated CLI help workflow adoption in
   [post-parse-integrated-cli-help-workflow-adoption-gap-selection.md](post-parse-integrated-cli-help-workflow-adoption-gap-selection.md).
3. Done: design compact CLI short option syntax here.
4. Done: implement compact short token parsing in the runtime parser with
   source/artifact/request workflow coverage.
5. Done: audit compact CLI short option syntax adoption in
   [post-compact-cli-short-option-syntax-adoption-gap-selection.md](post-compact-cli-short-option-syntax-adoption-gap-selection.md).
6. Done: design CLI subcommand metadata in
   [cli-subcommand-metadata.md](cli-subcommand-metadata.md).
7. Done: implement first enum/variant CLI subcommand metadata plumbing through
   source validation and `.mgi` package interfaces.
8. Done: implement strict command enum schemas and runtime dispatch/help.
9. Done: audit strict command enum schema adoption and refresh generated CLI
   samples/templates in
   [post-cli-subcommand-schema-adoption-gap-selection.md](post-cli-subcommand-schema-adoption-gap-selection.md).
10. Done: design wrapper-record root/global CLI options in
    [cli-wrapper-root-options.md](cli-wrapper-root-options.md).
11. Done: implement `@cli(subcommand)` metadata plumbing in
    [cli-wrapper-root-options.md](cli-wrapper-root-options.md).
12. Done: implement wrapper schema lowering and runtime parse/help for root
    options.
13. Done: adopt a minimal global option in the strict CLI sample/template.
14. Done: design schema-backed generated shell completions in
    [cli-schema-shell-completions.md](cli-schema-shell-completions.md).
15. Done: implement `muga cli-completions <bash|zsh|fish> --program <name>
    --type <Type> ...`.
16. Next: audit generated-project shell completion adoption, including install
    docs, packaging hooks, JSON completion specs, and richer nested traversal.
17. Later: revisit TOML/config
   discovery automation, runtime-owned printing/exits, and broader host-effect
   APIs after compact short option syntax is implemented and audited.
