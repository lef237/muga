Status: first CLI field metadata implemented

# CLI Field Metadata

`cli::parse_or[T]` and `cli::usage_for[T]` now make generated config-style
apps practical without hand-written per-field argument parsing. The first
field-level command-line metadata slice is implemented: record fields can set
CLI-specific long-option names, aliases, usage text, and hidden options without
changing JSON/config contracts.

This document records the first `@cli(...)` metadata slice for record fields.
It keeps the existing explicit app boundary: programs still decide when to call
`env::args()`, how to handle `--help`, where config files come from, and how to
map `cli::Error` into their public result type.

## Goals

Short-Term Goal: let concrete settings records customize long option names,
aliases, usage help text, and hidden fields while preserving the existing
`cli::parse_or[T]` overlay behavior.

Medium-Term Goal: keep JSON/config, validation, CLI parsing, usage generation,
interfaces, and artifacts aligned enough for generated starters and small real
tools to work from source or `run --built`.

Long-Term Goal: make Muga practical and adoptable by letting one typed public
data contract drive config files, command-line options, generated help,
validation, schema export, future TOML, and future client/tool generation
without duplicate glue code.

## Selected Syntax

Use one field-level `@cli(...)` attribute:

```muga
record Settings {
  @json(rename: "server_host")
  @cli(name: "host", alias: "server-host", help: "Server host name")
  host: String

  @cli(name: "tag", alias: "tags", help: "Filter tag")
  tags: List[String]

  @cli(hidden)
  debug_token: Option[String]
}
```

The first supported arguments are:

- `name: "long-option"`: replaces the field's primary CLI option name;
- repeated `alias: "long-option"`: accepted additional long option names;
- `help: "text"`: usage/help text for visible fields;
- `hidden`: parseable by `cli::parse_or[T]` but omitted from `cli::usage_for[T]`.

Rules:

- `@cli(...)` is allowed only on record fields in this slice.
- A field may have at most one `@cli` attribute.
- `name` and `help` may appear at most once.
- `alias` may appear zero or more times.
- `hidden` is a flag and may not have a value.
- `name` and `alias` values are long-option tokens without leading dashes.
- Long-option tokens must match `[A-Za-z][A-Za-z0-9_-]*`.
- `help` values may contain spaces and punctuation, but not tabs, newlines, or
  carriage returns.
- Empty `@cli(...)`, empty string values, leading `--`, short-flag spellings
  such as `-v`, and values containing whitespace are rejected.

Do not add `@cli(positional: N)` in this slice. Positionals, short flags,
subcommands, environment variables, config discovery, and required strict
parsing need separate policy decisions.

## Naming Semantics

The primary CLI option name for a field is:

1. `@cli(name: "...")`, when present;
2. otherwise `@json(rename: "...")`, when present;
3. otherwise the Muga field name.

Aliases are accepted only from `@cli(alias: "...")`.
`@json(alias: "...")` remains JSON/config input compatibility metadata and is
not accepted as a CLI alias.

`@cli(name: "...")` changes only the CLI surface. It does not change the record
field name, JSON/config primary wire name, JSON aliases, schema export property
name, or typed JSON encoding output.

For zero-payload enum field values, the first slice continues to parse enum tags
from the enum variant's JSON primary tag, meaning `@json(rename: "...")` on an
enum variant is still the enum value spelling. CLI-specific enum variant aliases
remain deferred.

## Duplicate And Conflict Rules

Type checking should reject ambiguous CLI accepted-name sets for every record
that uses CLI metadata or is lowered for `cli::parse_or[T]` /
`cli::usage_for[T]`:

- one field cannot repeat an alias;
- an alias cannot equal that field's primary CLI name;
- two exposed fields in one record cannot share any primary or alias name;
- a field annotated with `@cli(...)` must have a CLI-supported field type in the
  current parser slice;
- hidden fields still participate in duplicate checks because they remain
  parseable.

The diagnostic should point at the conflicting `@cli` argument when source spans
are available and include a related note for the previous accepted name.

No app-level names are reserved in the schema. A generated app may still treat
`--help` or `--config` as app-level options before calling `cli::parse_or[T]`.
That precedence stays explicit in app code.

## Parser Behavior

`cli::parse_or[T](args, defaults)` should accept the primary option name and all
CLI aliases:

- `--host api`, `--host=api`, and `--server-host api` all target the same field
  in the example above;
- repeated scalar options keep existing last-value-wins behavior;
- repeated list options append in argument order after replacing the default
  list on the first supplied item;
- using both a primary name and an alias for the same scalar field is not an
  error; normal last-value-wins behavior applies;
- bool, option, list, enum, `--`, unknown argument, missing value, invalid
  value, and validation behavior otherwise remains the same as the first CLI
  parser schema slice.

Runtime error `argument` strings should use the spelling supplied by the user,
for example `--server-host`, not the canonical primary option name.

`@validate(...)` runs after parsing exactly as it does today. Hidden fields are
not exempt from validation.

## Usage Behavior

`cli::usage_for[T](program, defaults)` remains deterministic plain text:

- fields are ordered by record declaration order;
- unsupported unannotated fields are preserved from defaults and omitted;
- `@cli(hidden)` fields are omitted;
- visible fields show the primary long option;
- aliases are shown in declaration order as `aliases: --a, --b`;
- list fields still show `repeatable`;
- displayable defaults are shown as `default: ...`;
- `help` text is appended after aliases/repeatability/default markers.

Example shape:

```text
Usage: config-app [options]

Options:
  --host <String>  aliases: --server-host  default: "localhost"  Server host name
  --tag <String>  repeatable  aliases: --tags  default: []  Filter tag
```

The exact spacing can stay compact like the current implementation, but the
ordering of name, metavar, repeatable marker, aliases, default, and help text
should be stable enough for tests.

## Metadata Pipeline

The implementation should add explicit CLI metadata beside JSON metadata:

- parser/AST: no new grammar node is required; `@cli` uses existing attribute
  arguments;
- typing: record fields carry `cli_name`, `cli_aliases`, `cli_help`, and
  `cli_hidden` metadata after validation;
- package signatures and typed HIR: preserve CLI metadata for public records and
  loaded-interface checking;
- package interfaces: persist CLI metadata in a new interface version so
  downstream packages can lower CLI schemas without provider source;
- MIR and bytecode: `cli::parse_or` and `cli::usage_for` carry a dedicated
  `CliSchema`, not a JSON decoder schema;
- `.mgb` implementation artifacts: persist the `CliSchema` payload and reject
  malformed payloads before execution;
- runtime: parse options, format usage, and validate values from `CliSchema`.

## Schema And Artifact Decision

Introduce a dedicated Rust-side `CliSchema` for the implementation slice.

The first parser implementation reused `JsonDecodeSchema` because CLI-only
metadata did not exist yet. That reuse is no longer the right boundary once the
schema carries help text, hidden flags, and CLI aliases. JSON and CLI contracts
overlap in field types and validation rules, but they differ in accepted names,
visibility, usage formatting, app-level options, and future positional or
subcommand behavior.

The implementation may still reuse existing `JsonDecodeValidationRule` and
scalar/list/enum parsing helpers internally. It should not encode CLI names,
aliases, help text, or hidden flags as JSON wire metadata.

Suggested first artifact shape:

- `CR <type_symbol> <field_count> ...` for record CLI schemas;
- each field stores source field symbol, primary option symbol, alias count and
  alias symbols, CLI flag bits, optional help symbol, validation rules, and a
  CLI value-shape token;
- flags use bit `1` for `hidden`, and unknown flag bits are malformed
  artifacts;
- help text is stored as a symbol so `.mgb` schema text remains whitespace
  tokenized;
- value-shape tokens cover the already-supported field types: `S`, `I`, `B`,
  `O`, `LS`, `LI`, `LB`, and zero-payload enum tags.

Package-interface text can keep legacy field lines unchanged when no CLI
metadata exists. Fields with CLI metadata should use an extended v9 field line
after JSON rename/alias/validation data, preserving v8 and older interfaces as
empty CLI metadata.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| Field-level `@cli(name, alias, help, hidden)` plus dedicated `CliSchema` | Solves the visible generated-help gaps, keeps JSON names separate, and creates the correct artifact boundary before positional/subcommand work. | Larger implementation than extending the JSON schema payload; needs interface v9 and `.mgb` CLI payload validation. | Select |
| Field-level `@cli(...)` but continue overloading `JsonDecodeSchema` | Fastest way to ship aliases/help by reusing existing wire-name and alias fields. | Makes CLI-only help/hidden data part of a JSON decoder type, blurs artifact semantics, and contradicts the earlier boundary that a dedicated schema is needed once CLI metadata exists. | Reject |
| Reuse `@json(rename)` and `@json(alias)` for CLI | No new syntax for names. | Payload compatibility aliases would accidentally change command-line behavior, and there is no place for help or hidden fields. | Reject |
| Repeated `@cli(alias: "...")` attributes | Reads naturally one alias per line. | Requires relaxing the established one-attribute-per-namespace field rule and complicates formatting. | Reject |
| Add `@cli(positional: N)` now | Important for CLI-only commands. | Requires positional parsing, duplicate index policy, usage sections, and conflict behavior with long options. | Defer |
| Add short flags such as `@cli(short: "v")` now | Familiar for command-line users. | Needs `-v`, `-abc`, value attachment, collisions, and usage formatting rules. | Defer |
| Record-level command metadata | Useful for program names, descriptions, subcommands, and grouped help. | Generated apps already pass program names explicitly, and subcommands need a larger command model. | Defer |
| Config discovery and automatic precedence | Could make generated config apps shorter. | Should build on settled field metadata and explicit app-level option policy. | Defer |
| Strict `cli::parse[T](args)` with required options | Useful for CLI-only tools. | Required/missing semantics and help markers are clearer after names/help metadata lands. | Defer |

## Tests

The implementation slice covers:

- parser accepts `@cli(name: "...", alias: "...", help: "...", hidden)` on
  record fields;
- parser rejects `@cli` on records, enum variants, functions, locals, and
  parameters;
- parser rejects duplicate `name`, duplicate `help`, value-bearing `hidden`,
  empty attributes, invalid option tokens, and help strings with tabs/newlines;
- formatter preserves `@cli(...)` attributes;
- type checking rejects duplicate primary/alias accepted names within a field
  and across fields;
- `@cli(...)` on unsupported field types reports a targeted unsupported CLI
  field diagnostic;
- `cli::parse_or[T]` accepts primary names and aliases from source execution;
- `cli::parse_or[T]` reports runtime errors using the supplied option spelling;
- `cli::usage_for[T]` shows aliases and help text and omits hidden fields;
- `@json(rename)` still drives the default CLI primary name when `@cli(name)` is
  absent;
- `@json(alias)` is not accepted as a CLI alias;
- `@validate(...)` still runs after parsing through a primary name or alias;
- package interfaces preserve CLI metadata without provider source;
- artifact-backed execution and `run --built` preserve parse and usage
  behavior;
- malformed CLI schema artifact payloads are rejected.

## Deferred Work

- positional arguments;
- short flags and combined short flags;
- subcommands and command groups;
- record-level command descriptions and examples;
- enum variant CLI aliases distinct from JSON aliases;
- environment-variable metadata;
- config discovery and automatic CLI/config/default precedence;
- strict required parsing without defaults;
- TOML/YAML/JSON5 loading;
- full client generation, OpenAPI/RPC, process/network APIs, `Bytes`, streams,
  or broader host effects.

## Implementation Plan

1. Done: implement `cli::parse_or[T]` and `cli::usage_for[T]` over concrete
   record overlays.
2. Done: refresh generated config apps to use `cli::parse_or[T]`.
3. Done: expose generated config-app usage through `--help`.
4. Done: audit post-config-app usage adoption and select CLI field metadata.
5. Done: design first `@cli(...)` field metadata in this document.
6. Done: implement parser/formatter/typing/interface/typed-HIR/MIR/bytecode/
   artifact/runtime support for field-level `@cli(...)` and dedicated
   `CliSchema`.
7. Done: refresh `config-app` to use `@cli(name: "tag", alias: "tags")` and
   field help text.
8. Next: re-audit before TOML, config discovery automation, strict parsing,
   full client generation, broader validators, or host APIs.
