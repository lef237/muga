Status: parse-integrated CLI help workflow design selected

# Post-Built-In CLI Help Helper Adoption Gap Selection

Built-in CLI help helpers are implemented across the virtual `std::cli`
package, typing schema lowering, typed HIR, MIR, bytecode, `.mgb`
implementation artifacts, runtime rendering, source/build/`run --built`
coverage, generated templates, and checked-in project samples.

## Current Adoption Result

Muga now has one schema-owned help contract for the two starter CLI shapes:

- `cli::help_requested(args)` recognizes exact `--help` and `-h` before `--`;
- `cli::help_for[T](program, defaults)` renders overlay/config help from the
  same schema as `cli::parse_or[T]`;
- `cli::help_for_required[T](program)` renders strict no-default help from the
  same schema as `cli::parse[T]`;
- help rendering includes command summaries, positional operands, short
  options, aliases, validation markers, defaults where relevant, hidden-field
  omission, and the built-in `-h, --help` row;
- help helper targets reject opt-in schema conflicts with `--help` and `-h`;
- `samples/projects/config_app`, `samples/projects/cli_tool`, and generated
  `muga new --template config-app` / `cli-tool` starters use the helpers.

The remaining CLI boilerplate is now control-flow shape, not help text
generation. Practical apps still write an app-owned branch before parsing:

```muga
if cli::help_requested(args) {
  help = usage_text()
  printed = println(help)
  return Result::Ok(help)
}

parsed: Result[Command, cli::Error] = cli::parse(args)
```

That explicit branch is a good low-level escape hatch, but generated starters
should eventually be able to express "help or parsed command" as one typed
request workflow while preserving application-owned printing and status
decisions.

## Goals

Short-Term Goal: design a parse-integrated help workflow that removes the
remaining repeated help-before-parse branch from generated command-line apps
without introducing runtime-owned printing or process exits.

Medium-Term Goal: keep strict parsing, overlay/config parsing, generated help,
recoverable CLI errors, source/artifact execution, and starter templates under
one typed request contract.

Long-Term Goal: make publishable Muga CLIs feel framework-small while staying
ordinary Muga code: users should match a typed request, print or return help
when they choose, and handle parse errors explicitly.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| Parse-integrated CLI help workflow design | Directly addresses the next visible starter boilerplate after built-in help helpers. Can preserve app-owned printing/status by returning a typed request such as help text vs parsed command. Establishes strict and overlay naming before implementation. | Needs careful generic enum/API shape, error boundary, source call type argument policy, artifact schema lowering, and template migration rules. | Select |
| Implement parse-integrated help immediately | Would quickly shrink starter code. | Public API choices are subtle: strict vs overlay helper names, request enum shape, interaction with `Result`, and whether help is success all need design first. | Reject for this slice |
| Combined short flags such as `-abc` | Familiar CLI polish for boolean flags. | Lower practical leverage than removing control-flow boilerplate; value-taking short options still need ambiguity policy. | Defer |
| Attached short values such as `-ofile` | Useful for Unix-style compact options. | Less urgent because `-o value` and `-o=value` already work; overlaps with combined short flag parsing. | Defer |
| Subcommands | High value for larger tools. | Needs nested schemas, dispatch, global vs local options, and per-command help. A parse-integrated request shape should be settled first. | Defer |
| Shell completion generation | Strong distribution value for schema-backed CLIs. | More useful after help workflow and subcommand shape are explicit. | Defer |
| TOML/config discovery automation | High value for config-heavy apps. | Orthogonal to help control flow and likely needs its own `std::config` policy. Keep it next-tier after CLI request shape. | Defer |
| Runtime auto-print and exit | Smallest app code and familiar host behavior. | Violates Muga's current explicit-effect boundary and blocks applications from choosing stdout/stderr, return values, or future status codes. | Reject |
| Keep only low-level helpers | Zero implementation risk and remains flexible. | Leaves generated starters with repeated branching even after help text generation was standardized. | Reject |
| Full client generation, generic encoding/decoding, broader validators, or host-effect APIs | Important for the larger platform. | Larger than the immediate CLI workflow gap and benefit from a cleaner command request contract later. | Defer |

## Selected Slice

Design a parse-integrated CLI help workflow before implementation.

That design is now recorded in
[parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md)
and has been implemented as `cli::Request[T]`, `cli::parse_request[T]`, and
`cli::parse_request_or[T]`.

The design should settle:

- public API shape and names for strict records and overlay/config records;
- whether the returned type is a generic `cli::Request[T]`, a `Result`
  carrying help separately, or separate strict/overlay enums;
- how `Help(String)` should compose with `Result` and existing `cli::Error`;
- whether source call type arguments are required for strict no-default records,
  inferred from expected result type, or anchored another way;
- how generated help text is carried through typed HIR, MIR, bytecode, `.mgb`
  artifacts, and `run --built`;
- how templates should match on help vs parsed values while retaining
  app-owned printing and status decisions;
- whether `cli::help_for` / `cli::help_for_required` remain public low-level
  helpers for custom workflows;
- explicit deferrals for runtime-owned printing/exiting, custom help flags,
  subcommands, combined short flags, attached short values, shell completion
  generation, TOML/config discovery, full client generation, generic
  encoding/decoding, broader validators, and host-effect APIs.

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
10. Done: design built-in CLI help policy.
11. Done: implement built-in CLI help helpers.
12. Done: audit built-in CLI help helper adoption.
13. Done: design parse-integrated CLI help workflow in
    [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md).
14. Done: implement parse-integrated CLI help workflow.
15. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata.
16. Later: revisit combined short flags, attached values, subcommands, shell
    completion generation, TOML/config discovery automation, full client
    generation, generic encoding/decoding, broader validators, or host-effect
    APIs after the request workflow is explicit.
