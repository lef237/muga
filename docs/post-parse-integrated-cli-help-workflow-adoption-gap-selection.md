Status: compact CLI short option syntax design selected

# Post-Parse-Integrated CLI Help Workflow Adoption Gap Selection

The parse-integrated request workflow is implemented across the standard
`std::cli` package surface, source execution, implementation artifacts, built
execution, generated `cli-tool` projects, and generated `config-app` projects.

## Current Adoption Result

Generated strict and config starters now parse help and command/config values
through one typed request contract:

- strict CLI tools use `cli::parse_request[Root](args, "cli-tool")` once root
  wrapper options are adopted;
- config apps use
  `cli::parse_request_or(settings_args(args), "config-app", default_settings())`;
- help remains a successful `cli::Request::Help(String)` value, so apps still
  own printing, return values, and future process-status decisions;
- low-level `cli::parse[T]`, `cli::parse_or[T]`, `cli::help_requested`,
  `cli::help_for[T]`, and `cli::help_for_required[T]` remain documented and
  covered for custom workflows;
- source, emitted artifacts, and `run --built` preserve the same request
  behavior.

No additional template migration is needed in this slice. The only visible
config-app boilerplate that remains is the app-specific `--config` bridge and
help line, which belongs to a future config discovery/TOML policy rather than
the request workflow itself.

## Goals

Short-Term Goal: audit that the request workflow removed generated help/parser
branching without hiding effects, then choose the next small CLI ergonomics
slice.

Medium-Term Goal: keep Muga-generated CLI tools close to familiar command-line
conventions while preserving the existing typed schema and artifact contracts.

Long-Term Goal: make larger Muga CLIs publishable without abandoning explicit
schema data, app-owned effects, and source/artifact parity.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| Compact CLI short option syntax design | Directly improves the `@cli(short: "...")` surface already implemented. Combined bool flags such as `-abc` and attached short values such as `-ofile` are common CLI expectations, and both can be specified as runtime parser behavior without new schema metadata. | Needs careful ambiguity rules for bool clusters, value-taking options, `=`, repeated list fields, unknown short names, `--`, and diagnostics. | Select |
| Implement compact short options immediately | Small likely code change in the runtime parser. | The combined/attached interaction is exactly the subtle part; design should fix the accepted grammar before behavior changes. | Reject for this slice |
| Extend request workflow into every stdlib sample | Would increase visible usage of `cli::Request[T]`. | `samples/packages/app/std_cli_schema` intentionally demonstrates low-level `parse_or` and `usage_for`; replacing it would reduce API coverage rather than improve starters. | Reject |
| Runtime auto-print and exit on help | Smallest generated app code. | Hides IO and process-status policy before Muga has a stable process exit API. | Reject |
| Subcommands | High value for larger tools and a natural follow-up to typed requests. | Needs nested schemas, global vs local options, dispatch shape, per-command help, artifact representation, and completion behavior. Compact short syntax is a smaller prerequisite polish. | Defer |
| Shell completion generation for generated apps | Strong distribution value because CLI schemas already exist. | More useful after short-token grammar and subcommand shape are settled. | Defer |
| TOML/config discovery automation | Would remove the config-app `settings_args` and manual `--config` help line. | Broader policy surface involving config file naming, precedence, discovery roots, and likely TOML support. | Defer |
| Custom positional labels or option+positional dual fields | Useful presentation polish for some tools. | Lower impact than compact short token support and can reuse the same usage-rendering foundation later. | Defer |
| Full client generation, generic encoding/decoding, broader validators, or host-effect APIs | Important for mature ecosystems. | Larger than the immediate CLI ergonomics gap and likely depends on subcommands/config policy. | Defer |

## Selected Slice

Design compact CLI short option syntax before implementation.

The design should settle:

- how `-abc` expands when every short option accepts a bare bool value;
- how `-ovalue`, `-abovalue`, `-o=value`, and `-abo=value` attach values to
  the final short option;
- whether `Bool`, `Option[Bool]`, and `List[Bool]` all count as bare-bool
  cluster fields;
- how repeated list fields merge when a short option appears multiple times in
  one compact token;
- how unknown short names and missing values report the offending token;
- how explicit `--` continues to stop option parsing;
- how negative positional arguments and value-looking tokens remain governed by
  the existing option-marker rules;
- why this is a runtime parser/diagnostics change, not a new `CliSchema`
  metadata shape.

The design must keep existing accepted forms working:

- long options: `--name value` and `--name=value`;
- exact short options: `-n value` and `-n=value`;
- bare bool short options: `-v`;
- explicit bool values: `-v=false` and `-v false`;
- positional parsing after `--`.

## Recommended Order

1. Done: implement schema-backed parsing, usage, help, metadata, artifacts, and
   starter adoption.
2. Done: design and implement parse-integrated CLI help workflow in
   [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md).
3. Done: audit parse-integrated CLI help workflow adoption here.
4. Done: design compact CLI short option syntax in
   [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md).
5. Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata.
6. Later: revisit subcommands, shell completion generation, TOML/config
   discovery automation, runtime-owned printing/exits, custom labels, full
   client generation, generic encoding/decoding, broader validators, and
   host-effect APIs after compact short option behavior is specified.
