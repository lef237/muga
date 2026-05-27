Status: strict CLI tool sample and template subcommand adoption implemented

# Post-CLI Subcommand Schema Adoption Gap Selection

Strict CLI command enum schemas are implemented. The remaining adoption gap was
whether users would see the new command-tree model only in isolated stdlib
tests, or in the generated starter they are most likely to copy.

## Current Adoption Result

- Strict command schemas accept concrete command enums, recursively dispatch
  subcommands, and return typed `cli::Request[Command]` values; the current
  `cli-tool` starter now reaches that command enum through a `Root` wrapper via
  `cli::parse_request[Root](args, "cli-tool")`.
- Source, artifact-root, and `run --built` coverage already prove nested command
  parsing, aliases, hidden commands, root help, branch help, and leaf help.
- `samples/projects/cli_tool` and `muga new --template cli-tool` now expose the
  practical command-tree shape directly:
  `Command::Run(RunCommand)` for the existing strict option workflow and
  `Command::Inspect(InspectCommand)` as a second leaf command.
- The existing compact short options, attached values, positional target,
  validation, list values, optional values, and recoverable `cli::Error`
  boundary remain visible inside the `run` leaf.

## Goals

Short-Term Goal: make the checked-in strict CLI sample and generated
`cli-tool` starter demonstrate enum-backed subcommands without adding new
language/runtime semantics.

Medium-Term Goal: use the sample/template as the acceptance fixture for future
CLI ergonomics such as root/global options, command completion generation,
TOML/config discovery, and richer help polish.

Long-Term Goal: make Muga-generated tools feel like practical multi-command
developer CLIs while preserving typed records/enums as the single source of
truth for parsing, help, diagnostics, artifacts, and app-boundary errors.

Final Goal: help Muga become useful and adoptable by making common real-world
CLI shapes simple, typed, generated, documented, and artifact-compatible.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| Adopt subcommands in `samples/projects/cli_tool` and the generated `cli-tool` template | Highest onboarding value; exercises the implemented command schema in the public starter; keeps existing option metadata in a leaf command; validates source/build/artifact workflows. | Changes sample invocation shape from `cli-tool <target>` to `cli-tool run <target>`. | Implemented |
| Keep subcommands only in focused stdlib tests | Lowest churn and preserves the old sample commands. | Users would not see the intended practical shape when running `muga new --template cli-tool`; adoption gap remains. | Reject |
| Add a new separate `multi-cli-tool` template | Avoids changing the existing starter. | Increases template surface before Muga has enough distinct starter personas; duplicates strict CLI boilerplate. | Defer |
| Implement wrapper-record root/global options first | Enables `tool --verbose run ...` and other mature CLI shapes. | Adds semantic design work beyond the completed command enum model; not required to make subcommands visible now. | Defer |
| Generate shell completions first | Useful distribution feature and benefits from command metadata. | Completion format and install workflow should follow a stable command-tree starter, not precede it. | Defer |
| Add runtime-owned printing/exits for help/errors | Could reduce starter code size. | Conflicts with Muga's current pure request/error boundary and would hide recoverable app behavior. | Reject for this slice |

## Implemented Slice

- Refresh `samples/projects/cli_tool/src/main/main.muga` to a root
  `Command` enum with `run`/`inspect` variants.
- Mirror that source in `muga new --template cli-tool`.
- Keep `RunCommand` as the strict field-metadata demonstration:
  positional target, compact short options, attached values, enum parsing,
  `Bool`, `List[String]`, `Option[String]`, and `@validate(...)`.
- Add `InspectCommand` as a compact second command with a positional target and
  a short verbose flag.
- Update source, generated-template, artifact-root, built JSON, root help, leaf
  help, missing argument, validation, and unknown command tests.
- Update onboarding docs and readiness checks so command-tree usage is the
  documented starter shape.

## Design Notes

- A later wrapper slice adds root/global options through a `Root` record without
  changing this command enum adoption result.
- The starter uses `cli::parse_request[Root]` rather than runtime-owned
  printing/exits, preserving explicit `Result[String, String]` app boundaries.
- `run` has alias `r` and `inspect` has alias `i`; aliases are command tokens,
  not short options.
- Leaf records keep compact short option behavior unchanged. For example,
  `cli-tool run service -dc3 -aApply -Tops -oKai` parses inside `RunCommand`.

## Recommended Order

1. Done: implement CLI subcommand metadata design in
   [cli-subcommand-metadata.md](cli-subcommand-metadata.md).
2. Done: implement enum/variant metadata plumbing.
3. Done: implement strict command enum schemas and runtime dispatch/help.
4. Done: adopt command enums in the strict CLI sample and generated `cli-tool`
   starter.
5. Done: design wrapper-record root/global CLI options in
   [cli-wrapper-root-options.md](cli-wrapper-root-options.md).
6. Done: implement `@cli(subcommand)` parser/formatter/type-checker metadata
   plumbing in [cli-wrapper-root-options.md](cli-wrapper-root-options.md).
7. Done: implement wrapper schema lowering and runtime parse/help for root
   options.
8. Done: adopt a minimal global option in the strict CLI sample/template.
9. Done: design schema-backed generated shell completions from the command and
   wrapper schemas in
   [cli-schema-shell-completions.md](cli-schema-shell-completions.md).
10. Done: implement `muga cli-completions <bash|zsh|fish> --program <name>
   --type <Type> ...` for source, artifact-root, and `--built` workflows.
11. Next: audit generated-project shell completion adoption, including install
   docs, packaging hooks, JSON completion specs, and richer nested traversal.
12. Later: revisit TOML/config discovery automation and runtime process-status
   helpers after command-tree ergonomics are stable.
