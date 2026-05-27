# Config Path Discovery

Status: generated config apps now support explicit environment-backed config
path discovery.

The JSON config loader intentionally stays small: `config::load_json_or[T]`
loads exactly one caller-selected JSON file and preserves CLI > config >
defaults in ordinary Muga code. The next practical gap was not TOML syntax; it
was reducing the need to pass `--config` for every local run while keeping the
path precedence visible and testable.

## Goals

Short-Term Goal: let generated `config-app` projects find a config path from an
environment variable when `--config` is not provided.

Medium-Term Goal: make config-backed generated apps easier to run from scripts,
service wrappers, and local shells without hiding precedence inside the runtime.

Long-Term Goal: keep future TOML, package resource lookup, and service
manifest work layered on explicit path selection rather than ambient magic.

Final Goal: make Muga practical for small operational tools: typed defaults,
JSON config files, CLI overrides, and a simple deployment-time config path
should work without a framework.

## Selected Precedence

Generated config apps use this path order:

1. `--config <path>` or `--config=<path>`
2. `MUGA_CONFIG_PATH`
3. the generated project default `config/settings.json`

The checked-in sample uses the same policy with its repository-relative fixture
as the final fallback:

```muga
fn discovered_config_path(): String {
  match env::get_var("MUGA_CONFIG_PATH") {
    Option::Some(value) => value
    Option::None => "config/settings.json"
  }
}

config_path = path::from_string(cli::option_or(args, "config", discovered_config_path()))
```

Help text reports the policy as:

```text
--config <Path>  default: $MUGA_CONFIG_PATH or config/settings.json
```

This is discovery of the config path only. The settings precedence remains
unchanged: CLI setting fields override the decoded config file, and the config
file overlays typed defaults through `config::load_json_or[T]`.
Generated projects now include the local helpers documented in
[config-app-run-helper.md](config-app-run-helper.md), which apply this policy
from run and package scripts without changing runtime semantics.

## Candidates Compared

| Candidate | Benefit | Cost / Risk | Decision |
|---|---|---|---|
| `--config` > `MUGA_CONFIG_PATH` > generated JSON path | Immediate deployment value; uses existing `std::env`; keeps path policy visible in generated source and tests. | Adds one environment variable convention to generated apps. | Select |
| Implement TOML parsing first | Familiar config format. | Requires parser policy, table/array/date/number semantics, format-specific diagnostics, and artifact/schema compatibility work. | Defer |
| Add `config::load_default_json_or[T]` | Shorter app code. | Hides path discovery and precedence in a compiler-owned generic helper before project resource semantics exist. | Defer |
| Make `muga run` change the current directory to the project root | Makes relative config paths work from anywhere. | Breaks host process expectations and source-compatible run behavior. | Reject |
| Add package resource lookup | Useful for installed apps. | Needs manifest-declared archive resources and a read-only runtime contract. | Done in [runtime-package-resource-lookup.md](runtime-package-resource-lookup.md) |

## Non-Goals

This slice does not add:

- TOML/YAML/JSON5 parsing;
- automatic current-directory mutation;
- installed app resource layouts;
- service manifests;
- automatic CLI/config merging outside generated source;
- runtime-owned config precedence.

## Implementation Plan

1. Done: add `discovered_config_path()` to the checked-in config sample and the
   generated `config-app` template.
2. Done: keep `--config` as the highest-precedence path.
3. Done: use `MUGA_CONFIG_PATH` only when `--config` is absent.
4. Done: update help text, onboarding docs, and tests for source and generated
   config apps.
5. Next: defer TOML until the JSON config workflow, generated templates, and
   package/resource policy justify a format expansion.
