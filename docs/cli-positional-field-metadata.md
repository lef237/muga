Status: CLI positional field metadata implemented

# CLI Positional Field Metadata

Typed CLI records can now express long option names, aliases, short options,
hidden fields, help text, validation, command summaries, strict parsing,
overlay parsing, generated usage, package interfaces, and artifact-backed
execution. The remaining basic command-shape gap is positional operands:
practical tools often read `tool input.muga --format json`, not only
`tool --input input.muga --format json`.

This slice adds the smallest schema-owned positional surface:

```muga
@cli(about: "Inspect a source file")
pub record Command {
  @cli(positional: 1, help: "Input source file")
  input: String

  @cli(positional: 2, help: "Optional output path")
  output: Option[String]

  @cli(short: "f", help: "Output format")
  format: Format
}
```

## Goals

Short-Term Goal: let strict CLI tools and config overlays parse primary
operands through the same typed schema that already parses named options.

Medium-Term Goal: keep positionals, named options, short options, aliases,
validation, command summaries, hidden fields, usage, interfaces, artifacts, and
future completion metadata in one `CliSchema`.

Long-Term Goal: make Muga command-line tools publishable from a single typed
record contract, so parser behavior, generated help, validation, templates,
artifact-backed execution, future shell completions, and future subcommands do
not require duplicate app-specific glue.

## Public Syntax

Allow `positional: N` inside a field-level `@cli(...)` attribute, where `N` is
a positive 1-based integer literal:

```muga
pub record Command {
  @cli(positional: 1, help: "Input path")
  input_path: String

  @cli(positional: 2, help: "Output path")
  output_path: Option[String]
}
```

Rules:

- `positional` is field-level only; record-level `@cli(...)` continues to
  support only `about`;
- a field may specify at most one `positional`;
- the value must be a positive integer literal, starting at `1`;
- positional indexes are unique in one CLI schema;
- indexes must be contiguous from `1` for every non-list positional field;
- a positional `List[T]` field must have the final positional index and captures all remaining positional operands;
- in the first implementation, a positional field may combine with `help` but
  may not combine with `name`, `short`, `alias`, or `hidden`;
- `@json(...)` wire-name metadata does not affect positional labels or accepted
  operands;
- `@validate(...)` continues to validate parsed positional values.

Use the source field name, normalized the same way as default CLI long option
names, as the generated operand label. For example, `input_path` renders as
`<input-path>`. A future `label` metadata key can be considered only after the
basic positional contract is implemented and audited.

## Supported First Types

The first positional slice should support the same scalar value shapes already
understood by typed CLI option parsing:

- `String`;
- `Int`;
- `Bool`, parsed from explicit `true` / `false` text rather than from presence;
- zero-payload concrete enums using the same value tags as CLI option values;
- `Option[T]` for the supported scalar/enum shapes;
- `List[T]` for the supported scalar/enum shapes, only as the final positional.

Unsupported field types remain unsupported even when annotated with
`positional`. The diagnostic should say that the field is not supported by CLI
positional parsing rather than falling back to a generic record/field error.

## Parser Behavior

`cli::parse[T](args)` and `cli::parse_or[T](args, defaults)` should parse named
options exactly as they do today, then assign remaining positional operands by
position index.

Parsing rules:

- long options and short options may appear before, between, or after
  positional operands;
- option values consumed by `--name value` or `-n value` are not positional
  operands;
- `--` stops option parsing and treats all following tokens as positional
  operands, including dash-leading values;
- dash-leading tokens before `--` remain option markers. Unknown markers still
  report `UnknownArgument` instead of becoming positionals, so typos such as
  `--imput` are not silently accepted as input paths;
- values that need to start with `-` can be passed after `--`;
- a missing required positional in strict parsing reports `MissingArgument`
  with the generated operand label such as `<input-path>`;
- an extra positional when no final `List[T]` positional exists reports
  `UnknownArgument` with the raw unexpected token;
- in `cli::parse_or[T](args, defaults)`, absent positional fields preserve the
  default value, while supplied `List[T]` positionals replace the default list
  with the collected operands.

Strict parsing keeps the existing absent-field synthesis:

- absent `Option[T]` positional fields become `Option::None`;
- absent `List[T]` positional fields become an empty list;
- absent required scalar/enum positional fields are `MissingArgument` errors.

## Usage Rendering

Generated usage should include positional operands in the usage line and an
`Arguments:` section when a schema has visible positionals:

```text
Usage: inspect [options] <input-path> [output-path]

Inspect a source file

Arguments:
  <input-path>  required; Input source file
  [output-path]  Optional output path

Options:
  -f, --format <Format>  values: Json, Text; Output format
```

Formatting rules:

- required scalar/enum positionals render as `<label>`;
- `Option[T]` positionals render as `[label]`;
- final `List[T]` positionals render as `[label...]`;
- positionals are ordered by their explicit numeric index, not record
  declaration order;
- validation markers and help text follow the same ordering used for option
  fields;
- positionals are never hidden in the first slice, so they always render;
- if a schema has no positionals, current usage output remains unchanged.

## Schema And Artifacts

`CliFieldSchema` should carry `position: Option<u32>` or an equivalent
positive integer representation.

Implementation behavior:

- parser and formatter preserve field-level `@cli(positional: N)`;
- typing stores positional metadata on record fields and validates duplicate
  indexes, contiguous non-list indexes, final-list policy, supported field
  types, and invalid combinations with `name`, `short`, `alias`, or `hidden`;
- package signatures and `.mgi` interfaces persist positional metadata for
  public records so downstream packages can lower CLI schemas without provider
  source;
- typed HIR, MIR, bytecode, `.mgb` implementation artifacts, and runtime
  parser/usage rendering preserve positions;
- old interfaces and old schema payloads without positional metadata remain
  readable as `position = None`;
- malformed new schema/interface payloads reject before execution.

Prefer an additive schema trailer for positional metadata, similar to the short
option metadata trailer, rather than inserting a fixed token into the middle of
the existing recursive field payload. That keeps old payloads readable and
reduces ambiguity with nested CLI value schema tokens.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| `@cli(positional: 1)` with explicit 1-based indexes | Stable under record field reordering, readable for users, easy to validate for duplicates and gaps, and fits existing field metadata. | Requires explicit numbering and a contiguity rule. | Select |
| `@cli(positional)` marker ordered by declaration | Shortest syntax for simple commands. | Reordering fields silently changes CLI behavior, and gaps/list capture policy become implicit. | Reject |
| `@cli(position: 0)` zero-based indexes | Matches internal list indexing and existing pure `cli::positional(args, 0)` helper. | User-facing command syntax should not expose internal indexing; `position: 1` reads as the first argument. | Reject |
| Allow `name`, `short`, or `alias` together with `positional` | Could let one field accept both operand and option forms. | Creates precedence and duplicate-usage ambiguity; can be added later if real tools need it. | Defer |
| Add a separate `label` metadata key immediately | Allows prettier metavars. | Not required for parsing, and source-field-derived labels are deterministic. | Defer |
| Use only manual `std::cli::positional` helpers | Already possible. | Does not feed generated usage, validation, interfaces, artifacts, templates, or future completions. | Reject |
| Implement combined short flags or attached short values first | Small parser convenience after short options. | Does not unlock the common primary-operand command shape. | Defer |
| Implement subcommands first | High value for larger CLIs. | Needs positional and option behavior inside nested command schemas first. | Defer |
| Built-in help branching first | Reduces starter boilerplate. | Generated help should know about positionals before help policy is centralized. | Defer |
| TOML, config discovery, shell completion generation, full client generation, generic encoding/decoding, broader validators, or host-effect APIs | Important later surfaces. | Larger or orthogonal to the immediate CLI operand gap. | Defer |

## Diagnostics

The implementation should add targeted diagnostics for:

- record-level `@cli(positional: ...)`;
- non-integer or non-positive positional values;
- duplicate positional indexes;
- gaps in non-list positional indexes;
- a positional `List[T]` that is not the final positional;
- combining `positional` with `name`, `short`, `alias`, or `hidden`;
- unsupported positional field types;
- missing required positionals at runtime;
- extra positional operands at runtime.

Runtime errors should preserve the user-facing argument spelling:
`<input-path>` for missing required positionals and the raw token for extra
positional operands.

## Non-Goals

This design does not add:

- combined short flags such as `-abc`;
- attached short values such as `-ofile`;
- fields that are both positional and named options;
- custom positional labels;
- hidden positionals;
- variadic positionals except one final `List[T]`;
- subcommands or nested command dispatch;
- built-in help branching, exits, or process status APIs;
- TOML, config discovery automation, shell completion generation, full client
  generation, generic encoding/decoding, broader validators, or host-effect
  APIs.

## Implementation Plan

1. Done: field-level CLI names, aliases, help, and hidden fields.
2. Done: strict required-option parsing.
3. Done: generated strict usage with
   `cli::usage_for_required[T](program)`.
4. Done: record-level `@cli(about: "...")` command summaries.
5. Done: field-level `@cli(short: "x")` metadata and short-option parsing.
6. Done: audit CLI short option metadata adoption in
   [post-cli-short-option-metadata-adoption-gap-selection.md](post-cli-short-option-metadata-adoption-gap-selection.md).
7. Done: implement `@cli(positional: N)` across parser validation,
   formatting, typing, package signatures/interfaces, `CliSchema`, artifacts,
   runtime parsing and usage rendering, sample/template adoption, tests, and
   docs.
8. Done: audit CLI positional field metadata adoption in
   [post-cli-positional-field-metadata-adoption-gap-selection.md](post-cli-positional-field-metadata-adoption-gap-selection.md).
9. Done: design built-in CLI help policy in
   [cli-built-in-help-policy.md](cli-built-in-help-policy.md).
10. Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata.
11. Later: revisit combined short flags, attached short values, option+positional
   dual fields, custom labels, built-in help branching, subcommands, TOML,
   config discovery automation, shell completion generation, full client
   generation, generic encoding/decoding, broader validators, or host-effect
   APIs.
