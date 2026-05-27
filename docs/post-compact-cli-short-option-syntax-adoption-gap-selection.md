Status: CLI subcommand metadata implemented; sample/template adoption implemented

# Post-Compact CLI Short Option Syntax Adoption Gap Selection

Compact CLI short option syntax is implemented in the shared typed CLI parser,
so every schema-backed parser entrypoint now accepts the new forms without new
metadata:

- `cli::parse[T](args)`;
- `cli::parse_or[T](args, defaults)`;
- `cli::parse_request[T](args, program)`;
- `cli::parse_request_or[T](args, program, defaults)`.

## Current Adoption Result

The compact syntax is adopted through the parser rather than through generated
source rewrites:

- `samples/projects/cli_tool` and the generated `cli-tool` starter already
  declare short names for `count`, `action`, `dry-run`, `tag`, and `owner`;
- source and `run --built` execution now cover compact invocations such as
  `-dc3`, `-aApply`, `-Tops`, and `-oKai`;
- the lower-level examples cover combined bool clusters, attached scalar and
  enum values, repeated list shorts, explicit compact final values, help
  request precedence, artifact-backed schema payloads, `--` boundaries, and
  compatibility with existing multi-character unknown short diagnostics;
- no `CliSchema`, `.mgi`, `.mgb`, project-template, or standard-library API
  migration is needed for this slice because compact syntax is deterministic
  parser behavior over existing `@cli(short: "...")` metadata.

The remaining CLI adoption gap is now command shape rather than option shape:
Muga can model a polished single command, but larger tools still need nested
commands such as `tool build`, `tool check`, or `tool config set` without
falling back to ad hoc string dispatch.

## Goals

Short-Term Goal: confirm compact short syntax is visible in generated CLI
workflow coverage, then choose the next schema-backed CLI ergonomics slice.

Medium-Term Goal: let one typed command contract drive parsing, generated
usage/help, artifacts, future shell completions, and command dispatch for
multi-action tools.

Long-Term Goal: make Muga-generated CLIs practical enough for real project
tooling while preserving typed schemas, explicit effects, deterministic
artifacts, and app-owned output/status policy.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| CLI subcommand metadata design | Unlocks multi-action tools, lets generated help model `tool <command> ...`, and gives shell completion generation a stable command tree to consume. A design slice can settle syntax, dispatch shape, root/global options, nested usage, artifacts, and compatibility before widening runtime behavior. | Larger than another option parser tweak; needs careful choices around enum-vs-record command representation, global vs local options, per-command positionals, and request/help return shapes. | Select |
| Implement subcommands immediately | High user-facing value if the shape is obvious. | The public syntax and artifact contract are not obvious; implementing first would freeze weak choices around dispatch and help. | Reject for this slice |
| Generated app shell completion generation | Strong distribution value because typed CLI schemas now include names, aliases, shorts, positionals, help, and command summaries. | Completion script shape is more valuable after subcommands and root/global options are represented; otherwise a first generator would need compatibility churn. | Defer |
| TOML/config discovery automation | Valuable for config-heavy apps and would remove some generated `config-app` glue. | Orthogonal to command dispatch and likely needs its own format/discovery/precedence policy. | Defer |
| Runtime-owned printing, exits, or process status API | Familiar for CLI frameworks and would reduce app boilerplate. | Conflicts with the current explicit-effect boundary; Muga still needs a stable process-status contract before hiding output or exits in parser helpers. | Reject |
| Rich help polish such as examples, footers, option groups, or custom headings | Improves documentation quality for published commands. | Lower leverage than representing multi-command structure; can layer onto the same usage renderer after subcommands. | Defer |
| Full client generation, generic encoding/decoding, broader validators, or host-effect APIs | Important for a mature ecosystem. | Larger and less directly connected to the now-complete typed CLI option surface. | Defer |

## Selected Slice

Design CLI subcommand metadata before implementation.

The design should settle:

- whether the public command target is an enum of command-record payloads, a
  record with a subcommand field, or another schema shape;
- how command names, aliases, summaries, hidden commands, and per-command help
  are declared;
- how root/global options interact with local subcommand options and
  positionals;
- how `cli::parse_request[T]` and `cli::parse_request_or[T]` report root help,
  subcommand help, missing subcommands, unknown subcommands, and parsed command
  values;
- generated usage text for root command lists and nested command usage;
- `CliSchema`, package interface, `.mgi`, `.mgb`, and old-artifact
  compatibility behavior;
- how compact short option syntax continues to apply only inside the selected
  command schema;
- which pieces of shell completion generation become easier once subcommand
  metadata is stable.

## Recommended Order

1. Done: implement field names, aliases, help, hidden fields, command
   summaries, strict/overlay parsing, short options, positionals, built-in help
   helpers, parse-integrated request helpers, and compact short option syntax.
2. Done: audit compact short option syntax adoption here.
3. Done: design CLI subcommand metadata in
   [cli-subcommand-metadata.md](cli-subcommand-metadata.md).
4. Done: implement first enum/variant CLI subcommand metadata plumbing through
   source validation and `.mgi` package interfaces.
5. Done: implement strict command enum schemas and runtime dispatch/help.
6. Done: audit strict command enum schema adoption and refresh generated
   samples/templates in
   [post-cli-subcommand-schema-adoption-gap-selection.md](post-cli-subcommand-schema-adoption-gap-selection.md).
7. Done: design wrapper-record root/global CLI options in
   [cli-wrapper-root-options.md](cli-wrapper-root-options.md).
8. Done: implement `@cli(subcommand)` metadata plumbing in
   [cli-wrapper-root-options.md](cli-wrapper-root-options.md).
9. Done: implement wrapper schema lowering and runtime parse/help for root
   options.
10. Done: adopt a minimal global option in the strict CLI sample/template.
11. Done: design schema-backed generated shell completions in
   [cli-schema-shell-completions.md](cli-schema-shell-completions.md).
12. Done: implement `muga cli-completions <bash|zsh|fish> --program <name>
   --type <Type> ...`.
13. Next: audit generated-project shell completion adoption, including install
   docs, packaging hooks, JSON completion specs, and richer nested traversal.
14. Later: revisit TOML/config discovery automation, richer help polish, process status APIs,
   runtime-owned printing/exits, full client generation, generic
   encoding/decoding, broader validators, and host-effect APIs.
