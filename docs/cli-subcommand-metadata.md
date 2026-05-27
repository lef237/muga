Status: strict CLI command enum schemas implemented; sample/template adoption implemented

# CLI Subcommand Metadata

Muga's schema-backed CLI surface now covers the core shape of a polished single
command: typed strict and overlay parsing, generated usage/help, command
summaries, aliases, hidden fields, positionals, short options, compact short
tokens, parse-integrated help requests, source/artifact parity, and generated
starter adoption.

The newly implemented practical CLI shape is command dispatch. Real tools
rarely stay as one command. They need typed command trees such as `tool build`,
`tool check`, and `tool config set` without falling back to ad hoc string
matching after `std::env::args()`.

This design selects enum-backed command trees for the first subcommand metadata
implementation:

```muga
@cli(about: "Project maintenance tool")
pub enum Command {
  @cli(name: "build", alias: "b", about: "Build package artifacts")
  Build(BuildCommand)

  @cli(name: "check", about: "Check sources without running")
  Check(CheckCommand)

  @cli(name: "config", about: "Inspect or update configuration")
  Config(ConfigCommand)
}

@cli(about: "Build package artifacts")
pub record BuildCommand {
  @cli(positional: 1, help: "Entry source file")
  entry: String

  @cli(short: "j", help: "Parallel build jobs")
  jobs: Option[Int]
}

pub enum ConfigCommand {
  @cli(name: "get", about: "Print one configuration value")
  Get(ConfigGetCommand)

  @cli(name: "set", about: "Set one configuration value")
  Set(ConfigSetCommand)
}
```

`cli::parse_request[Command](args, "tool")` returns
`cli::Request::Parsed(Command::Build(value))` for `["build", ...]`, returns
root help for `["--help"]`, and returns subcommand help for
`["build", "--help"]`.

## Goals

Short-Term Goal: preserve command-tree metadata through source, formatting,
typed HIR, package signatures, and package interfaces before touching runtime
dispatch, bytecode schema payloads, or generated starter code.

Medium-Term Goal: let strict Muga CLI tools use one typed command enum for
dispatch, generated root help, per-command help, source execution, built
artifacts, and future shell completion generation.

Long-Term Goal: make typed command contracts the source of truth for command
names, aliases, summaries, visibility, payload parsing, validation, diagnostics,
artifacts, examples, completion scripts, and later richer help polish.

Final Goal: make Muga command-line tools practical enough to publish and
recommend while preserving explicit effects, deterministic artifacts, and
app-owned output/status policy.

## Selected Public Shape

The first subcommand implementation extends strict schema-backed helpers to
accept either a supported record target, as today, or a concrete non-generic
command enum target.

Supported command enum targets:

- a concrete non-generic enum;
- at least one variant;
- every command variant carries exactly one payload;
- every payload is either a supported concrete command record or another
  supported command enum;
- every command variant has explicit `@cli(name: "...")` metadata;
- every command name and alias is unique within its sibling command level.

Rejected command enum targets:

- generic enums;
- zero-payload variants;
- variants with unsupported payload types;
- variants without `@cli(name: "...")`;
- duplicate command names or aliases at the same level;
- command records that fail existing strict CLI schema rules.

The return value stays the user-defined enum. No generated dispatch function or
runtime callback table is introduced. Application code remains ordinary pattern
matching:

```muga
request = try cli::parse_request[Command](args, "tool")
match request {
  cli::Request::Help(help) => println(help)
  cli::Request::Parsed(command) => match command {
    Command::Build(build) => run_build(build)
    Command::Check(check) => run_check(check)
    Command::Config(config) => run_config(config)
  }
}
```

## Public Metadata

Extend `@cli(...)` to enum declarations and enum variants.

Enum declarations support only root or branch summaries:

```muga
@cli(about: "Project maintenance tool")
pub enum Command { ... }
```

Enum variants support command metadata:

```muga
@cli(name: "build", alias: "b", about: "Build package artifacts")
Build(BuildCommand)
```

Rules:

- `name` is required for command variants in the first implementation;
- `alias` may repeat;
- `about` is optional and is used in parent command lists;
- `hidden` is optional, parses the command, and omits it from parent command
  lists;
- command names and aliases use the same token grammar as long options:
  ASCII letter first, then ASCII letters, digits, `_`, or `-`;
- command names and aliases are positional command tokens, not options, so
  `alias: "b"` is invoked as `tool b`, not `tool -b`;
- no command-level `short` metadata is added in this slice;
- command-level metadata is independent from `@json(rename: "...")`, because
  JSON tags and CLI command tokens have different compatibility and help
  expectations.

Variant command names are explicit instead of inferred from variant names. That
avoids freezing a PascalCase-to-kebab-case policy and keeps command spelling a
source-compatible contract.

Record-level `@cli(about: "...")` remains the detailed description for a leaf
command's own help. If the payload record omits `about`, detailed command help
may fall back to the selected variant `about` line.

## Strict And Overlay Helper Scope

The first implementation should support command enum targets for strict
helpers:

- `cli::parse[T](args)`;
- `cli::parse_request[T](args, program)`;
- `cli::usage_for_required[T](program)`;
- `cli::help_for_required[T](program)`.

Overlay/default helpers remain record-only in this slice:

- `cli::parse_or[T](args, defaults)`;
- `cli::parse_request_or[T](args, program, defaults)`;
- `cli::usage_for[T](program, defaults)`;
- `cli::help_for[T](program, defaults)`.

The reason is type-safety, not parser difficulty. An enum default has only one
active variant, so it cannot provide defaults for every sibling command payload
without adding a second defaults registry shape. Mixing strict parsing for some
commands and overlay parsing for the default command would make the same helper
change requiredness based on the selected subcommand. Keep command trees strict
until a separate root/global config design justifies an overlay command model.

If an overlay helper targets a command enum, type checking should reject it with
a targeted unsupported-target diagnostic instead of silently falling back to
record behavior.

## Root And Global Options

The first command enum target has no root/global user options. Root command
levels support only command selection and built-in request help.

This deliberately defers wrappers such as:

```muga
pub record Root {
  @cli(short: "v")
  verbose: Bool

  @cli(subcommand)
  command: Command
}
```

That wrapper shape is the right future direction for global options because it
would return both root options and the selected command in one typed value. It
is not selected for the first implementation because it introduces a new
field-level `subcommand` marker, root option precedence, defaults interaction,
and help layout rules at the same time as command dispatch.

Applications that need global configuration before this extension should keep
using a record payload repeated where needed, or use explicit app-owned
preprocessing around `std::env::args()`.

## Parsing Semantics

For a command enum schema:

1. Parse the current command level from left to right.
2. Exact `--help` or exact `-h` before a command token requests help for the
   current command level.
3. The first non-help token before `--` must match a visible or hidden command
   name or alias at the current level.
4. If the matched payload is a command record, parse the remaining tokens with
   the existing strict record parser.
5. If the matched payload is another command enum, recursively parse the next
   command level.
6. `--` before a required command token stops option-like scanning and leaves no
   command token for the current level, so the parser reports a missing
   command.

Help behavior is schema-aware only inside request helpers:

- `cli::parse[Command](["--help"])` reports an unknown argument, preserving the
  lower-level parser boundary;
- `cli::parse_request[Command](["--help"], "tool")` returns root help;
- `cli::parse_request[Command](["build", "--help"], "tool")` returns build
  help;
- `cli::parse_request[Command](["config", "set", "--help"], "tool")` returns
  nested `config set` help.

`cli::help_requested(args)` remains the existing low-level exact-match helper.
It is intentionally not changed to understand command schemas.

Compact short option syntax applies only inside the selected leaf record schema.
For example, `tool build -vj4 entry.muga` is parsed by the `BuildCommand`
record schema after `build` is selected. Command tokens themselves are never
clustered short options.

## Diagnostics

Use the existing public `cli::Error` shape:

- missing command: `ErrorKind::MissingArgument`, `argument: "<command>"`;
- unknown command token: `ErrorKind::UnknownArgument`, `argument` set to the
  token as written;
- missing record field after command dispatch: existing `MissingArgument`;
- unknown option inside the selected record: existing `UnknownArgument`;
- invalid record value: existing `InvalidValue`;
- validation failure: existing `Validation`;
- malformed or unsupported command schema at runtime: `UnsupportedTarget`.

Example messages:

```text
missing CLI command `<command>`
unknown CLI command `deploy`
unknown CLI option `--release`
```

Type-checking should diagnose schema-shape mistakes before runtime:

```text
`cli::parse_request` command enum variant `Deploy` requires `@cli(name: "...")`
```

```text
duplicate CLI command name `build` in enum `Command`
```

```text
`cli::parse_request` command variant `Command::Info` must carry a record or command enum payload
```

## Usage And Help Rendering

Root help should render command structure before options:

```text
Usage: tool <command> [args]

Project maintenance tool

Commands:
  build   Build package artifacts
  check   Check sources without running
  config  Inspect or update configuration

Options:
  -h, --help  Show this help
```

Leaf command help should reuse existing record help with the selected command
path as the program name:

```text
Usage: tool build [options] <entry>

Build package artifacts

Arguments:
  <entry>  required; Entry source file

Options:
  -j, --jobs <Int>  Parallel build jobs
  -h, --help        Show this help
```

Nested branch help should render another command list:

```text
Usage: tool config <command> [args]

Commands:
  get  Print one configuration value
  set  Set one configuration value

Options:
  -h, --help  Show this help
```

Determinism rules:

- command order follows enum variant declaration order;
- hidden commands parse but do not render in `Commands:`;
- aliases render after the command summary as `aliases: ...`;
- a command list with no visible commands renders `(none)` only if all commands
  are hidden;
- detailed leaf help uses existing option/argument ordering and compact short
  parser behavior.

## Schema And Artifacts

The implemented internal CLI schema keeps the existing record fields and adds a
command-variant list. A schema with no commands is the existing record schema;
a schema with commands is a command enum schema:

```rust
pub struct CliSchema {
    pub type_name: Symbol,
    pub package_item: Option<PackageItemId>,
    pub about: Option<Symbol>,
    pub fields: Vec<CliFieldSchema>,
    pub commands: Vec<CliCommandVariantSchema>,
}

pub struct CliCommandVariantSchema {
    pub variant_name: Symbol,
    pub command_name: Symbol,
    pub aliases: Vec<Symbol>,
    pub about: Option<Symbol>,
    pub hidden: bool,
    pub payload: Box<CliSchema>,
}
```

The existing record schema payload should remain exactly readable. New command
schemas should use a new artifact token family, for example `CC`, while keeping
the current `CR` record token for compatibility.

Required persistence changes:

- AST/parser: allow `@cli(about: "...")` on enum declarations and
  `@cli(name: "...", alias: "...", about: "...", hidden)` on enum variants;
- package signatures and `.mgi`: persist enum-level and variant-level CLI
  metadata for public command enums;
- typed schema lowering: produce command-bearing `CliSchema` values for
  supported command enum targets and nested command enum payloads;
- typed HIR, MIR, bytecode, and `.mgb`: carry the generalized `CliSchema`;
- runtime: dispatch commands recursively and render root/branch/leaf help;
- artifact validation: reject malformed command payloads before execution.

Implemented metadata plumbing:

- AST/parser/formatter accept and preserve enum-level `@cli(about: "...")`;
- parser/type checking accept variant command metadata, require variant
  `@cli(name: "...")` when any command metadata is present, and reject duplicate
  sibling command names or aliases;
- typed HIR and package signatures preserve enum and variant command metadata;
- `.mgi` package interfaces use `muga-package-interface-v11` and preserve
  enum-level `cli about` plus variant `cli name/aliases/about/hidden` metadata;
- v10 and older readable interfaces load wrapper-field CLI metadata as absent
  when that newer marker is not present.

Implemented strict schema/runtime support:

- `CliSchema` now carries `commands: Vec<CliCommandVariantSchema>` and persists
  command schemas with the `CC` artifact token family while keeping existing
  `CR` record schema payloads readable;
- strict helpers accept concrete non-generic command enum targets whose variants
  all have explicit command names and record or nested-command payloads;
- overlay/default helpers remain record-only and reject command enum targets;
- MIR, bytecode schema payloads, implementation artifacts, and artifact-backed
  execution preserve nested command schemas;
- runtime parsing dispatches command names and aliases recursively, parses the
  selected leaf record with existing strict record semantics, and returns the
  user enum value;
- `cli::parse_request[T](args, program)` returns root, branch, or leaf help for
  exact `--help` / `-h` at the relevant command level;
- root and branch help render `Usage: tool <command> [args]`, visible command
  lists, aliases, summaries, and the built-in `-h, --help` row;
- source, artifact-backed execution, invalid schema diagnostics, command-schema
  artifact round trips, and hidden-command help omission are covered by
  `standard_cli_subcommand_parse_request_runs`,
  `standard_cli_subcommand_parse_request_artifact_run_uses_schema_payload`, and
  `standard_cli_subcommand_schema_rejects_invalid_contracts`.

Compatibility behavior:

- old record-only `CR` schema payloads remain valid;
- old interfaces without enum CLI metadata remain loadable, but command enum
  parsing rejects variants that lack required `@cli(name: "...")`;
- older runtimes will reject new `CC` command-schema payloads as unsupported
  artifact data, which is acceptable for a new language/runtime feature;
- command metadata changes are public-interface changes for public enums and
  should affect interface hashes.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| Concrete enum of command-record payloads | Matches Muga's existing enum dispatch and record parser model. The parsed value is the user's command enum, so app code uses normal exhaustive `match`. Nested command enums model `tool config set` without new dispatch syntax. | Requires enum/variant CLI metadata, recursive schema payloads, and request-help routing. | Select |
| Record with a subcommand field | Models root/global options naturally and returns one record containing globals plus the command. | Requires a new `@cli(subcommand)` field marker, precedence rules, defaults behavior, and a new mixed record/command schema at the same time as first command dispatch. | Defer |
| Record with a string command field and manual dispatch | Small compiler change. | Loses typed dispatch, generated command help, exhaustive matching, and future completion generation. | Reject |
| Function table or annotated command functions | Feels close to some CLI frameworks. | Introduces callback registration and effect/order questions before Muga has function metadata for runtime discovery. | Reject |
| Infer command names from variant identifiers | Less annotation in simple cases. | Freezes case conversion and rename compatibility too early; `BuildArtifact` to `build-artifact` is a policy decision, not a parser fact. | Defer |
| Require strict helpers only for first command enum support | Keeps default/overlay semantics coherent and still unlocks practical multi-action tools. | Config-heavy command trees still need explicit app-owned preprocessing or repeated payload defaults. | Select |
| Support overlay/default command enums immediately | Could combine config apps and subcommands. | A single enum default cannot provide defaults for every sibling payload; mixed strict/overlay behavior would surprise users. | Reject for first slice |
| Root/global options in first subcommand slice | Common in mature CLIs. | Widens the public type shape and help/parse precedence rules; better as a wrapper-record design after command enums are proven. | Defer |
| Built-in `help` subcommand | Familiar for `tool help build`. | Adds another command dispatch policy beyond exact `--help`/`-h`; the existing request enum is enough for the first slice. | Defer |
| Shell completion generation first | Distribution value. | Completion scripts need a stable command tree. | Defer until subcommand schema exists |

## Non-Goals

This design does not add:

- root/global user options;
- `@cli(subcommand)` record fields;
- overlay/default command enum parsing;
- inferred command names;
- short command aliases such as `tool -b`;
- command groups, categories, examples, footers, or custom headings;
- built-in `tool help ...` command routing;
- runtime-owned printing, process exits, or process status APIs;
- shell completion generation;
- TOML/config discovery automation;
- full client generation, generic encoding/decoding, broader validators, or
  broader host-effect APIs.

## Implementation Plan

1. Done: implement schema-backed record CLI parsing, usage/help, field
   metadata, command summaries, short options, positionals, request helpers,
   compact short token syntax, source/artifact parity, and generated starter
   adoption.
2. Done: audit compact short option syntax adoption in
   [post-compact-cli-short-option-syntax-adoption-gap-selection.md](post-compact-cli-short-option-syntax-adoption-gap-selection.md).
3. Done: design CLI subcommand metadata here.
4. Done: implement the first enum metadata plumbing across AST/parser,
   formatter, type-checking diagnostics, typed HIR, package signatures, `.mgi`
   v10 persistence, tests, and docs.
5. Done: implement strict command enum schemas across typed
   schema lowering, package signatures, `.mgi`, typed HIR, MIR, bytecode,
   `.mgb`, runtime dispatch/help rendering, source/artifact/`run --built`
   tests, and docs.
6. Done: audit strict command enum schema adoption and refresh the checked-in
   strict CLI sample plus generated `cli-tool` template in
   [post-cli-subcommand-schema-adoption-gap-selection.md](post-cli-subcommand-schema-adoption-gap-selection.md).
7. Done: design wrapper-record root/global options in
   [cli-wrapper-root-options.md](cli-wrapper-root-options.md).
8. Done: implement `@cli(subcommand)` parser/formatter/type-checker metadata
   plumbing in [cli-wrapper-root-options.md](cli-wrapper-root-options.md).
9. Done: implement wrapper schema lowering and runtime parse/help for root
   options in [cli-wrapper-root-options.md](cli-wrapper-root-options.md).
10. Done: adopt a minimal global option in the strict CLI sample/template.
11. Done: design schema-backed generated shell completions in
   [cli-schema-shell-completions.md](cli-schema-shell-completions.md).
12. Done: implement `muga cli-completions <bash|zsh|fish> --program <name>
   --type <Type> ...`.
13. Next: audit generated-project shell completion adoption, including install
   docs, packaging hooks, JSON completion specs, and richer nested traversal.
14. Later: revisit TOML/config discovery automation, richer help polish,
   process status APIs, runtime-owned printing/exits, full client generation,
   generic encoding/decoding, broader validators, and host-effect APIs.
