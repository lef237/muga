Status: CLI positional field metadata design selected

# Post-CLI Short Option Metadata Adoption Gap Selection

Field-level `@cli(short: "x")` metadata is implemented and adopted in the
strict CLI sample, generated `cli-tool` starter, and `std_cli_schema` package
sample. Typed CLI records can now express long names, aliases, short names,
hidden fields, help text, validation, command summaries, and deterministic
generated usage through one compiler-owned `CliSchema`.

The remaining CLI ergonomics gap is no longer ordinary option spelling. It is
that practical command-line tools still need natural positional arguments such
as `tool input.muga --format json`, but typed CLI schemas currently model only
named options. Muga already has pure `std::cli::positional` lookup helpers for
manual code, but parser/usage/artifact behavior cannot yet be driven from the
record schema.

## Current Adoption Result

- `samples/projects/cli_tool` and generated `muga new --template cli-tool`
  both accept short options for required fields, optional fields, repeated
  fields, enum fields, and app-owned `-h` help checks.
- `samples/packages/app/std_cli_schema` verifies short option metadata through
  `cli::parse_or[T]`, `cli::usage_for[T]`, emitted artifacts, and hidden-field
  behavior.
- Short option names are validated as single ASCII letters, duplicate short
  names are rejected in one schema, and generated usage renders
  `-x, --long` option cells deterministically.
- `.mgi` interfaces, package signatures, typed HIR, MIR, `CliSchema`, and
  schema artifacts preserve short names while old payloads without short
  metadata remain readable.
- The strict sample and generated starter still cannot express a required
  input path or optional output path as typed positionals without falling back
  to manual `std::cli::positional` glue outside the schema.

## Goals

Short-Term Goal: let typed CLI records model the primary operands users expect
to pass without option names, while keeping generated usage honest and stable.

Medium-Term Goal: keep named options, short options, aliases, positionals,
validation, command summaries, hidden fields, interfaces, artifacts, and future
completion metadata in one `CliSchema` instead of splitting app glue from
schema-driven parsing.

Long-Term Goal: make Muga CLI tools publishable from typed records alone:
parser behavior, generated help, validation, templates, artifacts, future shell
completions, and future subcommand metadata should all share one public command
contract.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| Typed CLI positional field metadata design | Unlocks the common `tool input --flag` command shape; reuses the existing field metadata, validation, usage, interface, and artifact pipeline; prepares subcommands and shell completions better than parser-only short syntax tweaks. | Needs ordering, required/optional/list policy, usage layout, `--` behavior, and conflict rules with named option metadata. | Select |
| Combined short flags such as `-abc` | Familiar for clusters of boolean flags and now naturally follows short option metadata. | Only improves spelling compactness for existing options; value-taking fields make ambiguity and diagnostics more subtle. | Defer |
| Attached short values such as `-ofile` | Familiar for some Unix-style tools. | Less necessary because `-o value` and `-o=value` already work; ambiguity overlaps with combined short flags. | Defer |
| Built-in `--help` / `-h` command framework | Removes explicit starter branches and could standardize help output. | Needs exit/status policy and application framework semantics; positionals should first be represented in usage. | Defer |
| Subcommands | High value for larger tools. | Requires nested schemas, dispatch, global vs local options, and per-command usage; positionals are the smaller prerequisite. | Defer |
| Shell completion generation | Strong ecosystem value after short names. | More complete after positionals and possibly subcommands are present in the schema. | Defer |
| TOML or config discovery automation | Valuable for config-backed apps. | Orthogonal to CLI operand modeling and should reuse the same schema after command shape improves. | Defer |
| Use only manual `std::cli::positional` helpers | Already possible for custom code. | Does not feed generated usage, validation, templates, interfaces, artifacts, or future tooling. | Reject as the next schema slice |
| Full client generation, generic encoding/decoding, broader validators, or host-effect APIs | Important for the broader platform. | Larger than the immediate CLI adoption gap and still benefit from a complete command contract later. | Defer |

## Selected Slice

Design typed CLI positional field metadata before implementation.

The design should settle:

- public syntax, likely field-level metadata such as `@cli(positional: ...)`,
  without committing parser behavior before the syntax is documented;
- ordering rules, duplicate-position diagnostics, and how declaration order
  interacts with explicit position indexes;
- supported first field types for strict parsing and config/default overlays,
  especially required scalar/enum fields, `Option[T]`, and `List[T]`;
- whether a field may combine positional metadata with `name`, `short`, or
  `alias`, or whether the first slice keeps positionals and options separate;
- `--` behavior, mixed option/positional parsing, repeated positionals, and
  missing/extra argument diagnostics;
- generated `usage_for[T]` and `usage_for_required[T]` layout, including
  operand labels and validation markers;
- package signature, `.mgi`, typed HIR, `CliSchema`, schema artifact, and
  `run --built` compatibility behavior;
- explicit deferrals for combined short flags, attached short values, built-in
  help branching, subcommands, TOML, config discovery automation, shell
  completion generation, full client generation, generic encoding/decoding,
  broader validators, and host-effect APIs.

## Recommended Order

1. Done: implement field-level CLI names, aliases, help, and hidden fields.
2. Done: implement strict required-option parsing.
3. Done: add the checked-in strict CLI sample and generated `cli-tool`
   starter.
4. Done: replace duplicated strict help text with
   `cli::usage_for_required[T](program)`.
5. Done: implement record-level `@cli(about: "...")` command summaries.
6. Done: implement field-level `@cli(short: "x")` metadata and generated
   sample/template adoption.
7. Done: audit CLI short option metadata adoption.
8. Done: design typed CLI positional field metadata in
   [cli-positional-field-metadata.md](cli-positional-field-metadata.md).
9. Done: implement CLI positional field metadata.
10. Done: audit CLI positional field metadata adoption.
11. Done: design built-in CLI help policy in
   [cli-built-in-help-policy.md](cli-built-in-help-policy.md).
12. Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata.
13. Later: revisit combined short flags, attached short values, built-in help
   branching, subcommands, TOML, config discovery automation, shell completion
   generation, full client generation, generic encoding/decoding, broader
   validators, or host-effect APIs after positional semantics are explicit.
