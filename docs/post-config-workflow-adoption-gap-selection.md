Status: Result error mapping refresh implemented

# Post-Config-Workflow Adoption Gap Selection

The JSON config workflow sample proves that current `std::fs`, `std::path`,
`std::json`, `std::env`, and `std::cli` pieces can build a practical settings
flow. It also exposes the next immediate friction: recoverable IO and JSON
errors must be normalized into the app's `Result[String, String]` boundary.

Muga already has `std::result::map_err`, but `samples/projects/config_app`
currently uses several local `json_*_result` wrappers instead. The best next
slice is to refresh the practical sample to use the existing `std::result`
helper surface before inventing a `std::config` package, error unions,
schema decoding, or a broader formatting/error API.

## Short-Term Goal

Refresh `samples/projects/config_app/src/main/main.muga`:

- import `std::result`;
- use `result::map_err(..., io_error_message)` for filesystem reads;
- use `result::map_err(..., json_error_message)` for JSON parse and
  object-field extraction;
- remove local one-off JSON result wrapper functions;
- preserve CLI > config > defaults behavior, source execution,
  emitted-artifact execution, config shape-error coverage, and
  `run --built --format=json` coverage.

The first slice should not add a new error type, postfix propagation syntax,
implicit error conversion, a `std::config` package, TOML, schema decoding,
formatting templates, `Bytes`, process APIs, network APIs, or streams.

## Medium-Term Goal

Use the refreshed sample to decide whether existing `std::result` helpers are
enough for app-boundary error normalization:

- if `map_err` keeps the sample clear, keep teaching explicit error mapping
  and defer new error abstractions;
- if repeated mappings still dominate, design a small common error convention
  or app-local helper pattern before broad stdlib expansion;
- if field extraction remains the noisy part after error mapping, revisit
  record/schema decoding from `.mgi` public interfaces;
- if string assembly remains the noisy part, evaluate formatting templates or
  builder APIs separately from config loading.

## Long-Term Goal

Keep Muga's practical app path explicit and teachable:

- applications can normalize errors at their boundaries without hidden
  exceptions or implicit conversions;
- config, package, and service tools can share `Result` conventions;
- richer config/schema tooling can build on ordinary functions first;
- process, network, and streaming APIs can introduce their own error contracts
  only after resource and cancellation rules are designed.

## Candidates Compared

| Candidate | Practical value | Risk | Decision |
|---|---|---|---|
| Refresh config workflow with `std::result::map_err` | Directly removes repeated local wrappers; demonstrates an existing v1 helper in a real workflow; no new syntax, std API, host effects, or compatibility surface | Only improves ergonomics where a single error-mapping function is enough | Select next |
| Add common error unions or implicit error conversion | Reduces app-boundary boilerplate across IO, JSON, CLI, and future APIs | Changes propagation semantics and error contracts; likely affects `try expr`, diagnostics, docs, and user expectations | Defer |
| Add `std::config` or TOML | Useful for app settings | Requires format choice, parser/dependency policy, discovery rules, precedence policy, diagnostics, and compatibility contract | Defer |
| Record/schema decoding from JSON | Removes manual field extraction | Requires `.mgi` schema mapping, default/missing-field policy, versioning, nested diagnostics, and compatibility rules | Defer |
| Full CLI parser schema with usage/help | Helps polished tools and subcommands | Requires schema representation, repeated values, generated help text, validation diagnostics, and compatibility policy | Defer |
| Formatting templates or interpolation | Reduces `.concat(...)` noise in samples | Adds syntax or broad formatting APIs before escaping, localization, builders, and formatter rules are settled | Defer |
| `Bytes`, process APIs, network APIs, or streams | Needed for platform work | Requires resource, encoding, cancellation, buffering, security, and runtime contracts beyond this app ergonomics gap | Defer |

## Selected Slice

Use the existing `std::result::map_err` helper in the JSON config workflow
sample and document the pattern.

This is intentionally an adoption cleanup, not a new language feature. It
keeps error conversion explicit and proves whether the existing `Result` helper
surface is enough before broader error or config abstractions are designed.

## Implemented First Slice

`samples/projects/config_app/src/main/main.muga` now imports `std::result` and
uses `result::map_err` for every app-boundary error conversion:

- `fs::read_text_path(...)` maps `io::IOError` through `io_error_message`;
- `json::parse(...)` maps `json::Error` through `json_error_message`;
- `json::object_string_or`, `json::object_int_or`, and
  `json::object_bool_or` map shape errors through `json_error_message`;
- the local `json_value_result`, `json_string_result`, `json_int_result`, and
  `json_bool_result` wrappers are removed;
- existing source, emitted-artifact, config shape-error, and
  `run --built --format=json` tests continue to cover the workflow.

## Recommended Order

1. Record this selection and add release-readiness evidence.
2. Refresh `samples/projects/config_app/src/main/main.muga` to import
   `std::result` and replace local JSON result wrappers with `result::map_err`.
3. Keep the existing source, emitted-artifact, shape-error, and
   `run --built --format=json` tests passing.
4. Update README, ROADMAP, practical-readiness, Muga by Example, and stdlib
   review docs to mention the `std::result::map_err` pattern.
5. Run the release gate before choosing `std::config`, TOML, schema decoding,
   full CLI parser schemas, formatting templates, `Bytes`, process APIs, or
   network APIs.
