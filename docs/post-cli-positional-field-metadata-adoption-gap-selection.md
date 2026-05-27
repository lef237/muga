Status: built-in CLI help policy design completed

# Post-CLI Positional Field Metadata Adoption Gap Selection

Field-level `@cli(positional: N)` metadata is implemented across parser
validation, typing, package signatures, `.mgi` interfaces, `CliSchema`,
artifacts, runtime parsing, usage rendering, source/build tests, and the strict
`cli-tool` template/sample.

## Current Adoption Result

Muga can now model the common `tool input --flag` command shape without manual
`std::cli::positional` glue:

- typed strict parsing accepts primary operands through
  `@cli(positional: N)`;
- generated usage includes an ordered `Arguments:` section;
- source, emitted artifacts, and `run --built` preserve positional metadata;
- the generated `cli-tool` template and checked-in `samples/projects/cli_tool`
  use a positional target operand while keeping options for count, action,
  dry-run, tags, and owner.

The remaining visible CLI friction is now help plumbing. Applications still
write the same `--help` / `-h` branch and append a help option line manually:

```muga
if cli::has_flag(args, "help") or cli::has_short_flag(args, "h") {
  usage = usage_text()
  printed = println(usage)
  return Result::Ok(usage)
}
```

That is acceptable as an app-owned escape hatch, but it is the next
boilerplate users see when starting a practical command-line tool.

## Goals

Short-Term Goal: design a small built-in help policy that removes duplicated
`--help` / `-h` checks from generated strict CLI tools without hiding control
flow or process-status decisions.

Medium-Term Goal: keep generated help text, command summaries, positional
operands, short options, aliases, validation markers, hidden fields, and future
subcommand/help behavior under one documented CLI schema contract.

Long-Term Goal: make Muga CLIs publishable with a small typed command record
and predictable generated help, preparing the surface for subcommands and shell
completion generation.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| Built-in CLI help policy design | Directly removes the next repeated starter/template branch now that usage rendering includes positionals and options. Lets Muga decide API shape, output, `Result`, and status semantics before implementation. | Needs careful boundaries so the compiler/runtime does not silently exit or print without app consent. | Select |
| Implement built-in help immediately | Small code surface if it only checks `--help` / `-h`. | Public behavior would be easy to get wrong: whether help is an error, success, printed output, returned text, or process exit must be designed first. | Reject for this slice |
| Combined short flags such as `-abc` | Familiar for clusters of boolean flags. | Only polishes existing options; value-taking options and short clusters need ambiguity rules. | Defer |
| Attached short values such as `-ofile` | Familiar for some CLIs. | Lower impact than help; interacts with combined flags and value disambiguation. | Defer |
| Subcommands | High value for larger tools. | Needs nested schemas, global/local options, per-command help, and dispatch policy. Built-in help should be settled first. | Defer |
| Shell completion generation | Strong distribution value after schema metadata. | More useful after help and subcommand policy are explicit. | Defer |
| Custom positional labels or option+positional dual fields | Useful polish for some CLIs. | First adoption did not require them; dual fields need precedence and duplicate usage rules. | Defer |
| TOML/config discovery automation | Important for config-heavy apps. | Orthogonal to CLI help and broader than this immediate CLI ergonomics gap. | Defer |

## Selected Slice

The built-in CLI help policy is now designed in
[cli-built-in-help-policy.md](cli-built-in-help-policy.md). The next slice
should implement `cli::help_requested(args)`, `cli::help_for[T](program,
defaults)`, and `cli::help_for_required[T](program)` before parse-integrated
help result enums or runtime-owned printing/exits.

The design settles:

- public API shape, for example whether help is a `cli::help_requested(args)`
  helper, a parser mode, a `Result` variant, or a generated helper;
- whether generated usage includes `-h, --help` automatically, only through an
  opt-in flag, or through a separate usage helper;
- whether help is returned as a value, printed by the app, printed by runtime,
  or represented as a recoverable parse result;
- how strict `cli::parse[T]`, overlay `cli::parse_or[T]`, `usage_for`, and
  `usage_for_required` should share behavior;
- how `--` affects help detection;
- how hidden fields, aliases, short options, positionals, and future subcommands
  render in help;
- source, artifact, and `run --built` compatibility;
- explicit deferrals for process exit/status APIs, subcommands, shell
  completions, combined short flags, attached short values, TOML/config
  discovery, generic encoding/decoding, broader validators, and host-effect
  APIs.

## Recommended Order

1. Done: implement field-level CLI names, aliases, help, and hidden fields.
2. Done: implement strict required-option parsing.
3. Done: add the checked-in strict CLI sample and generated `cli-tool`
   starter.
4. Done: replace duplicated strict usage text with
   `cli::usage_for_required[T](program)`.
5. Done: implement record-level `@cli(about: "...")` command summaries.
6. Done: implement field-level `@cli(short: "x")` metadata.
7. Done: audit CLI short option metadata adoption.
8. Done: design and implement typed CLI positional field metadata.
9. Done: audit CLI positional field metadata adoption.
10. Done: design built-in CLI help policy in
    [cli-built-in-help-policy.md](cli-built-in-help-policy.md).
11. Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata.
12. Later: revisit combined short flags, attached values, subcommands, shell
    completion generation, custom labels, option+positional dual fields, TOML,
    config discovery automation, full client generation, generic
    encoding/decoding, broader validators, or host-effect APIs after help
    policy is explicit.
