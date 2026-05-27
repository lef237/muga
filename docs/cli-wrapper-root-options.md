Status: CLI wrapper-record root/global option sample/template adoption implemented

# CLI Wrapper Root Options

Strict command enum schemas make multi-command tools practical, but they do
not yet model root or global options such as:

```bash
tool --verbose --profile dev run app/main.muga
```

The next CLI shape should return both the global options and the selected typed
command without asking application code to pre-scan `env::args()`.

## Goals

Short-Term Goal: design a wrapper-record shape that composes existing strict
record fields with an existing strict command enum field.

Medium-Term Goal: let generated tools express common root options such as
`--verbose`, `--profile`, `--format`, or `--config` once, while keeping command
payloads typed and local.

Long-Term Goal: make one typed CLI schema cover root options, command dispatch,
leaf options, generated root/leaf help, diagnostics, artifacts, and later shell
completion generation.

Final Goal: make Muga-generated CLIs feel publishable and conventional without
weakening typed app-boundary control over output, errors, and process status.

## Selected Public Shape

Use a concrete non-generic record wrapper with exactly one field marked
`@cli(subcommand)`. The marked field must have a supported concrete command
enum type.

```muga
@cli(about: "Project maintenance tool")
pub record Root {
  @cli(short: "v", help: "Show verbose output")
  verbose: Bool

  @cli(help: "Profile name")
  profile: Option[String]

  @cli(subcommand)
  command: Command
}

@cli(about: "Project commands")
pub enum Command {
  @cli(name: "run", alias: "r", about: "Run an entry file")
  Run(RunCommand)

  @cli(name: "inspect", alias: "i", about: "Inspect an entry file")
  Inspect(InspectCommand)
}
```

`cli::parse_request[Root](args, "tool")` returns
`cli::Request::Parsed(Root { verbose: true, profile: Option::Some("dev"),
command: Command::Run(value) })` for
`["--verbose", "--profile", "dev", "run", ...]`.

## Wrapper Field Rules

- `@cli(subcommand)` is a field-level marker.
- A wrapper record may contain exactly one `@cli(subcommand)` field.
- The marked field type must be a supported concrete non-generic command enum.
- The marked field is not rendered as an option or argument.
- The marked field may not also specify `name`, `alias`, `short`, `help`,
  `hidden`, or `positional`.
- The marked field may not be `Option[T]`, `List[T]`, or another structural
  wrapper; the command enum itself controls required command dispatch.
- Other wrapper fields are strict root/global options using the existing field
  metadata, validation, bool synthesis, optional synthesis, and list behavior.
- Positional root fields are deferred for the first implementation because
  `tool input run ...` is ambiguous and less common than root options.

## Helper Scope

The first implementation supports wrapper records only in strict helpers:

- `cli::parse[T](args)`;
- `cli::parse_request[T](args, program)`;
- `cli::usage_for_required[T](program)`;
- `cli::help_for_required[T](program)`.

Overlay/default helpers remain unsupported for wrapper records:

- `cli::parse_or[T](args, defaults)`;
- `cli::parse_request_or[T](args, program, defaults)`;
- `cli::usage_for[T](program, defaults)`;
- `cli::help_for[T](program, defaults)`.

The reason is the same as command enum overlays: a single default wrapper value
contains one active command variant and cannot provide defaults for every
sibling command payload. A future config-aware command model can add a separate
defaults registry after strict wrapper behavior is stable.

## Parsing Semantics

For a wrapper record:

1. Parse root/global option tokens from the start of the argument list using
   the wrapper record fields except the `@cli(subcommand)` field.
2. Stop root option parsing at the first non-option command token.
3. Exact `--help` or exact `-h` before a command token requests wrapper root
   help.
4. `--` before a command token stops root option scanning and leaves no command
   token, so the parser reports a missing command.
5. Parse the remaining tokens with the nested command enum schema.
6. Once a command token has been selected, later tokens belong to the selected
   command payload. Root/global options are not accepted after the command.

Examples:

| Args | Result |
|---|---|
| `["--verbose", "run", "app/main.muga"]` | root `verbose = true`, command `Run(...)` |
| `["run", "--verbose"]` | `--verbose` is parsed by `RunCommand`, not the wrapper |
| `["--help", "run"]` | root help, command ignored |
| `["run", "--help"]` | `run` leaf help |
| `["--", "run"]` | missing command |

Compact short option syntax applies at the current parsing scope. Before the
command token, `-vpdev run ...` may parse root shorts if the wrapper has `v`
and `p`; after the command token, compact shorts parse only against the
selected leaf record.

## Diagnostics

Type-checking should reject invalid wrapper contracts before runtime:

```text
`cli::parse_request` wrapper record `Root` may contain exactly one `@cli(subcommand)` field
```

```text
`cli::parse_request` wrapper field `Root::command` must have a concrete command enum type
```

```text
`cli::parse_request` wrapper field `Root::command` cannot combine `subcommand` with `short`
```

Runtime parsing should reuse public `cli::Error` values:

- missing command uses `MissingArgument` with `argument: "<command>"`;
- unknown command uses `UnknownArgument` with the token as written;
- missing required root option uses the existing root option error;
- unknown root option before the command uses the root parser error;
- unknown option after command dispatch uses the selected command payload
  error.

## Help Rendering

Wrapper root help should show global options and command choices without
mixing them with leaf options:

```text
Usage: tool [global-options] <command> [args]

Project maintenance tool

Commands:
  run      aliases: r  Run an entry file
  inspect  aliases: i  Inspect an entry file

Global Options:
  -v, --verbose[=<Bool>]  Show verbose output
  --profile <String>      Profile name
  -h, --help              Show this help
```

Leaf help stays local to the selected command:

```text
Usage: tool run [options] <entry>

Run an entry file

Arguments:
  <entry>  required; Entry source file

Options:
  -h, --help  Show this help
```

Global options are intentionally accepted only before the command in the first
implementation. That keeps generated help truthful and prevents a root option
from silently stealing a leaf option with the same name.

## Schema And Artifacts

Keep command enum schemas distinct from wrapper record schemas.

Add a nested subcommand schema to `CliSchema` instead of overloading the
existing `commands` field:

```rust
pub struct CliSchema {
    pub type_name: Symbol,
    pub package_item: Option<PackageItemId>,
    pub about: Option<Symbol>,
    pub fields: Vec<CliFieldSchema>,
    pub commands: Vec<CliCommandVariantSchema>,
    pub subcommand: Option<CliSubcommandSchema>,
}

pub struct CliSubcommandSchema {
    pub field_name: Symbol,
    pub schema: Box<CliSchema>,
}
```

Invariants:

- a plain record schema has fields and no commands/subcommand;
- a command enum schema has commands and no fields/subcommand;
- a wrapper record schema has root fields, no direct commands, and one nested
  subcommand schema;
- wrapper root fields exclude the marker field.

Wrapper schemas use the `CW` artifact token, which stores the record field
payload plus a length-prefixed nested command schema. Existing `CR` record
schema payloads and `CC` command schema payloads remain readable.

Package signatures and `.mgi` interfaces persist one new record-field metadata
bit: `cli_subcommand: bool`. Current `.mgi` files use
`muga-package-interface-v11`; v10 and older interfaces load this field as
`false`.

## Implemented Metadata Plumbing

The first implementation slice covers the public metadata path:

- parser validation accepts `@cli(subcommand)` only as field-level CLI
  metadata and rejects duplicate marker arguments or marker values;
- `muga fmt` preserves the bare marker spelling;
- type checking rejects more than one subcommand marker per wrapper record,
  rejects marker combinations with `name`, `short`, `positional`, `alias`,
  `help`, or `hidden`, and requires a concrete non-generic command enum field
  whose command schema is otherwise valid;
- typed HIR and package signatures expose `cli_subcommand: bool`;
- `.mgi` package interfaces persist field CLI flags through
  `muga-package-interface-v11` while still reading legacy v10 interfaces;
- examples coverage is anchored by
  `cli_wrapper_subcommand_field_metadata_plumbing_is_covered` and
  `cli_wrapper_subcommand_field_metadata_rejects_invalid_contracts`.

## Implemented Schema And Runtime

The second implementation slice connects wrapper records to executable CLI
schemas:

- `CliSchema` now carries `subcommand: Option<CliSubcommandSchema>` so wrapper
  records keep root fields separate from the nested command enum schema instead
  of overloading `commands`;
- wrapper schemas persist through `CW` `.mgb` payloads and lower through MIR,
  bytecode, and implementation artifacts alongside existing `CR` and `CC`
  payloads;
- strict helpers lower wrapper records for `cli::parse[T]`,
  `cli::parse_request[T]`, `cli::usage_for_required[T]`, and
  `cli::help_for_required[T]`, while default/overlay helpers reject wrapper
  records at type checking;
- runtime parsing scans root/global options before the command token, preserves
  `--verbose run` as a bare Bool root option plus command dispatch, treats
  `--` before a command as a missing command, and leaves after-command options
  to the selected command payload;
- wrapper root help renders `Usage: tool [global-options] <command> [args]`,
  `Commands:`, and `Global Options:` sections, while leaf help remains scoped
  to the selected command payload;
- source, artifact-backed, and `run --built` coverage is anchored by
  `standard_cli_wrapper_parse_request_runs` and
  `standard_cli_wrapper_parse_request_artifact_and_built_runs_use_schema_payload`.

## Implemented Sample And Template Adoption

The checked-in `samples/projects/cli_tool` project and generated
`muga new --template cli-tool` starter now use a `Root` wrapper record with:

- `@cli(name: "profile", short: "p", help: "Execution profile")`
  `profile: Option[String]` as the minimal root/global option;
- `@cli(subcommand) command: Command` for the existing `run` / `inspect`
  command tree;
- `cli::parse_request[Root](args, "cli-tool")` so root help, root options,
  selected command parsing, and recoverable `cli::Error` mapping stay in one
  typed request flow.

The adoption deliberately chose `--profile` / `-p` over a second global
`--verbose` option because `InspectCommand` already owns `-v`, and a profile is
a realistic cross-command value option that demonstrates root option value
consumption without changing normal output when omitted. The source, generated
template, artifact-backed execution, built JSON execution, and help checks are
covered by `cli_new_creates_cli_tool_template`,
`manifest_cli_tool_project_sample_runs_with_required_options`,
`manifest_cli_tool_project_sample_reports_generated_usage`,
`manifest_cli_tool_project_sample_runs_against_emitted_artifacts`, and
`manifest_cli_tool_project_sample_json_built_run_uses_strict_parse`.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| Wrapper record with `@cli(subcommand)` field | Returns global options and selected command in one typed value; reuses record fields, command enums, validation, artifacts, and app `match`; matches common CLI framework shape. | Requires one new field marker, wrapper parse split, wrapper help layout, schema/artifact extension, and diagnostics. | Select |
| App-owned pre-scan plus command enum parse | No compiler/runtime change. | Duplicates option parsing, hides global options from generated help, weakens artifact/schema completeness, and makes templates less useful. | Reject |
| Add root options directly to command enums | Avoids wrapper record syntax. | Enums have no fields; adding fields to enum declarations would be a broader language change and awkward for app code. | Reject |
| Special `Global` command variant | Uses existing command enum machinery. | Makes global options look like a command, cannot combine with every command, and breaks help expectations. | Reject |
| Allow root options before and after subcommands | Familiar in some CLI frameworks. | Ambiguous when root and leaf options share names; complicates generated help and diagnostics. | Defer |
| Support overlay/default wrapper helpers now | Could combine config defaults with command trees. | A single default value cannot represent all command payload defaults; strict wrappers are simpler and safer. | Defer |
| Implement shell completions before wrapper records | Valuable for distribution. | Completion generation benefits from knowing global option scope; wrapper design should come first. | Defer |

## Non-Goals

This design does not add:

- root positional operands before commands;
- root/global options accepted after the command token;
- overlay/default command wrapper parsing;
- inferred command names;
- command groups, examples, footers, or custom help sections;
- shell completion generation;
- TOML/config discovery automation;
- runtime-owned printing, exits, or process status APIs.

## Implementation Plan

1. Done: implement strict command enum schemas and runtime dispatch/help in
   [cli-subcommand-metadata.md](cli-subcommand-metadata.md).
2. Done: adopt command enums in `samples/projects/cli_tool` and generated
   `cli-tool` starters in
   [post-cli-subcommand-schema-adoption-gap-selection.md](post-cli-subcommand-schema-adoption-gap-selection.md).
3. Done: design wrapper-record root/global options here.
4. Done: implement parser/formatter/type-checker support for
   `@cli(subcommand)` on record fields and preserve it through package
   signatures and interfaces.
5. Done: lower wrapper schemas through `CliSchema`, MIR, bytecode, artifacts,
   and runtime parse/help.
6. Done: adopt a minimal `--profile` / `-p` global option in the strict CLI
   sample and generated `cli-tool` template after source, artifact-backed,
   built, and help coverage is stable.
7. Done: design schema-backed generated shell completions over wrapper,
   command, and leaf schemas in
   [cli-schema-shell-completions.md](cli-schema-shell-completions.md).
8. Done: implement `muga cli-completions <bash|zsh|fish> --program <name>
   --type <Type> ...` for source, artifact-root, and `--built` workflows.
9. Next: audit generated-project shell completion adoption, including install
   docs, packaging hooks, JSON completion specs, and richer nested traversal.
10. Later: TOML/config discovery automation, richer help polish, and runtime
   process-status helpers.
