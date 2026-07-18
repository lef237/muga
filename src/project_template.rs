use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{diagnostic::Diagnostic, package, span::Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectTemplate {
    App,
    Lib,
    Test,
    ConfigApp,
    CliTool,
    ReportApp,
    ResourceExport,
    PackageApp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectTemplateInfo {
    pub name: &'static str,
    pub aliases: &'static [&'static str],
    pub description: &'static str,
}

const PROJECT_TEMPLATE_INFOS: &[ProjectTemplateInfo] = &[
    ProjectTemplateInfo {
        name: "app",
        aliases: &[],
        description: "CLI-first app starter",
    },
    ProjectTemplateInfo {
        name: "lib",
        aliases: &["library"],
        description: "Library package starter",
    },
    ProjectTemplateInfo {
        name: "test",
        aliases: &["tests"],
        description: "Package with a passing std::test example",
    },
    ProjectTemplateInfo {
        name: "config-app",
        aliases: &["config_app", "config"],
        description: "Typed JSON config app with CLI overrides",
    },
    ProjectTemplateInfo {
        name: "cli-tool",
        aliases: &["cli_tool", "cli"],
        description: "Strict CLI tool with subcommands and completions",
    },
    ProjectTemplateInfo {
        name: "report-app",
        aliases: &["report_app", "report"],
        description: "File-processing report app starter",
    },
    ProjectTemplateInfo {
        name: "resource-export",
        aliases: &["resource_export", "resource"],
        description: "Binary resource export app starter",
    },
    ProjectTemplateInfo {
        name: "package-app",
        aliases: &[
            "package_app",
            "local-dependency",
            "local_dependency",
            "local",
        ],
        description: "App plus local library package starter",
    },
];

pub fn project_template_infos() -> &'static [ProjectTemplateInfo] {
    PROJECT_TEMPLATE_INFOS
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectTemplateOutput {
    pub root: PathBuf,
    pub entry: PathBuf,
}

pub fn create_project_template(
    root: &Path,
    template: ProjectTemplate,
) -> Result<ProjectTemplateOutput, Vec<Diagnostic>> {
    ensure_new_project_root(root)?;
    let package_name = inferred_package_name(root);
    if template == ProjectTemplate::PackageApp {
        return create_package_app_template(root, &package_name);
    }

    let manifest = manifest_for_template(template, &package_name);
    let (entry_relative, files) = template_files(template);
    let entry = root.join(entry_relative);
    let render_context = TemplateRenderContext::new(&package_name);

    write_template_file(&root.join("muga.toml"), &manifest)?;
    for file in files {
        let text = render_template_text(file.text, &render_context);
        write_template_file(&root.join(file.relative), &text)?;
    }

    Ok(ProjectTemplateOutput {
        root: root.to_path_buf(),
        entry,
    })
}

fn manifest_for_template(template: ProjectTemplate, package_name: &str) -> String {
    let mut manifest = format!(
        "[package]\nname = \"{}\"\nlanguage_revision = {}\nsource = \"src\"\n",
        escape_manifest_string(package_name),
        package::SUPPORTED_LANGUAGE_REVISION
    );
    if template == ProjectTemplate::ResourceExport {
        manifest.push_str("resources = \"resources\"\n");
    }
    manifest
}

fn create_package_app_template(
    root: &Path,
    package_name: &str,
) -> Result<ProjectTemplateOutput, Vec<Diagnostic>> {
    let render_context = TemplateRenderContext::new(package_name);
    let entry_relative = "app/src/main/main.muga";
    let entry = root.join(entry_relative);

    for file in package_app_template_files() {
        let text = render_template_text(file.text, &render_context);
        write_template_file(&root.join(file.relative), &text)?;
    }

    Ok(ProjectTemplateOutput {
        root: root.to_path_buf(),
        entry,
    })
}

struct TemplateRenderContext<'a> {
    package_name: &'a str,
    app_package_name: String,
    shared_package_name: String,
}

impl<'a> TemplateRenderContext<'a> {
    fn new(package_name: &'a str) -> Self {
        Self {
            package_name,
            app_package_name: format!("{package_name}_app"),
            shared_package_name: format!("{package_name}_shared"),
        }
    }
}

fn render_template_text(text: &str, context: &TemplateRenderContext<'_>) -> String {
    text.replace(
        "{{package_name}}",
        &escape_muga_string(context.package_name),
    )
    .replace(
        "{{app_package_name}}",
        &escape_muga_string(&context.app_package_name),
    )
    .replace(
        "{{shared_package_name}}",
        &escape_muga_string(&context.shared_package_name),
    )
    .replace(
        "{{language_revision}}",
        &package::SUPPORTED_LANGUAGE_REVISION.to_string(),
    )
}

fn ensure_new_project_root(root: &Path) -> Result<(), Vec<Diagnostic>> {
    if root.exists() {
        if !root.is_dir() {
            return Err(vec![
                Diagnostic::new(
                    "PK027",
                    format!(
                        "project target `{}` already exists and is not a directory",
                        root.display()
                    ),
                    Span::default(),
                )
                .with_suggestion("choose a new project directory"),
            ]);
        }
        let mut entries = fs::read_dir(root).map_err(|error| {
            vec![Diagnostic::new(
                "PK027",
                format!(
                    "failed to read project target `{}`: {error}",
                    root.display()
                ),
                Span::default(),
            )]
        })?;
        if entries.next().is_some() {
            return Err(vec![
                Diagnostic::new(
                    "PK027",
                    format!(
                        "project target `{}` already exists and is not empty",
                        root.display()
                    ),
                    Span::default(),
                )
                .with_suggestion("choose an empty directory or a new project path"),
            ]);
        }
        return Ok(());
    }

    fs::create_dir_all(root).map_err(|error| {
        vec![Diagnostic::new(
            "PK027",
            format!(
                "failed to create project target `{}`: {error}",
                root.display()
            ),
            Span::default(),
        )]
    })
}

fn write_template_file(path: &Path, text: &str) -> Result<(), Vec<Diagnostic>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            vec![Diagnostic::new(
                "PK027",
                format!(
                    "failed to create template directory `{}`: {error}",
                    parent.display()
                ),
                Span::default(),
            )]
        })?;
    }
    fs::write(path, text).map_err(|error| {
        vec![Diagnostic::new(
            "PK027",
            format!(
                "failed to write template file `{}`: {error}",
                path.display()
            ),
            Span::default(),
        )]
    })
}

struct TemplateFile {
    relative: &'static str,
    text: &'static str,
}

fn package_app_template_files() -> &'static [TemplateFile] {
    &[
        TemplateFile {
            relative: "README.md",
            text: r#"# Package App

Generated app plus local library package starter.

## Layout

- `app/` is the runnable package.
- `shared/` is a local library dependency.
- `app/muga.toml` depends on `shared/` with:

```toml
[dependencies]
{{shared_package_name}} = { path = "../shared" }
```

## Run

```sh
muga run app/src/main/main.muga
muga run app/src/main/main.muga -- Ada
muga run app/src/main/main.muga -- --name=Ada
muga workspace --format json app/src/main/main.muga
muga build app/src/main/main.muga
muga run --built app/src/main/main.muga -- --name=Ada
```

## Package

Create a source-free app bundle with the local dependency included as bundle
artifacts, run it, archive it as `.mga`, verify the generated archive, and
optionally install/list it when `MUGA_INSTALL_DIR` points at an explicit bin
directory:

```sh
sh scripts/package-package-app.sh
```

The helper uses `MUGA_BIN` when set, or `muga` from `PATH` otherwise. Set
`MUGA_PROGRAM`, `MUGA_BUNDLE_DIR`, `MUGA_ARCHIVE_DIR`, or `MUGA_INSTALL_DIR` to
override packaging paths or explicitly install/list the generated launcher.
Muga does not edit shell startup files.
"#,
        },
        TemplateFile {
            relative: "app/muga.toml",
            text: r#"[package]
name = "{{app_package_name}}"
language_revision = {{language_revision}}
source = "src"

[dependencies]
{{shared_package_name}} = { path = "../shared" }
"#,
        },
        TemplateFile {
            relative: "app/src/main/main.muga",
            text: r#"import {{shared_package_name}}::greetings
import std::cli
import std::env
import std::string

fn selected_name(args: List[String]): String {
  cli::option_or(args, "name", cli::positional_or(args, 0, "Muga"))
}

fn render_result(name: String): String {
  greeting = greetings::build(name)
  string::join(["package-app", greetings::render(greeting)], "|")
}

fn main(): String {
  result = render_result(selected_name(env::args()))
  printed = println(result)
  result
}
"#,
        },
        TemplateFile {
            relative: "shared/muga.toml",
            text: r#"[package]
name = "{{shared_package_name}}"
language_revision = {{language_revision}}
source = "src"
"#,
        },
        TemplateFile {
            relative: "shared/src/greetings/main.muga",
            text: r#"import std::string

pub record Greeting {
  name: String
  message: String
}

pub fn build(name: String): Greeting {
  Greeting { name: name, message: "hello ".concat(name) }
}

pub fn render(greeting: Greeting): String {
  string::join([greeting.name, greeting.message], "|")
}
"#,
        },
        TemplateFile {
            relative: "scripts/package-package-app.sh",
            text: r#"#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
project_dir=$(CDPATH= cd "$script_dir/.." && pwd)
MUGA_BIN=${MUGA_BIN:-muga}
program=${MUGA_PROGRAM:-package-app}
bundle_dir=${MUGA_BUNDLE_DIR:-"$project_dir/dist/$program"}
archive_dir=${MUGA_ARCHIVE_DIR:-"$project_dir/dist/app-archives"}
entry="$project_dir/app/src/main/main.muga"

"$MUGA_BIN" emit-app-bundle --source-free --output-dir "$bundle_dir" --program "$program" "$entry"
"$MUGA_BIN" run-app-bundle "$bundle_dir" -- --name=Ada
archive_output=$("$MUGA_BIN" emit-app-archive --archive-root "$archive_dir" --program "$program" "$bundle_dir")
printf '%s\n' "$archive_output"
archive_path=$(printf '%s\n' "$archive_output" | sed -n 's/^archive[[:space:]]//p' | tail -n 1)
if [ -z "$archive_path" ]; then
  archive_path=$(printf '%s\n' "$archive_output" | sed -n '/\.mga$/p' | tail -n 1)
fi
if [ -z "$archive_path" ]; then
  printf '%s\n' "package helper could not find emitted archive path" >&2
  exit 1
fi
"$MUGA_BIN" verify-app-archive "$archive_path"
if [ "${MUGA_INSTALL_DIR:-}" != "" ]; then
  "$MUGA_BIN" install-app --replace-owned --output-dir "$MUGA_INSTALL_DIR" --program "$program" "$bundle_dir"
  "$MUGA_BIN" list-installed-apps --output-dir "$MUGA_INSTALL_DIR"
fi
"#,
        },
    ]
}

fn template_files(template: ProjectTemplate) -> (&'static str, &'static [TemplateFile]) {
    match template {
        ProjectTemplate::PackageApp => unreachable!("package-app is generated by a custom layout"),
        ProjectTemplate::App => (
            "src/main/main.muga",
            &[
                TemplateFile {
                    relative: "src/main/main.muga",
                    text: r#"import std::cli
import std::env

fn name_from_args(args: List[String]): String {
  cli::option_or(args, "name", cli::positional_or(args, 0, "Muga"))
}

fn main(): String {
  name = name_from_args(env::args())
  message = "hello ".concat(name)
  printed = println(message)
  message
}
"#,
                },
                TemplateFile {
                    relative: "README.md",
                    text: r#"# App

Generated CLI-first app starter.

## Run

```sh
muga run src/main/main.muga
muga run src/main/main.muga -- Ada
muga run src/main/main.muga -- --name=Ada
muga build src/main/main.muga
muga run --built src/main/main.muga -- --name=Ada
```

## Package

Create a source-free app bundle, run it from bundle artifacts, archive it as
`.mga`, and verify the generated archive:

```sh
sh scripts/package-app.sh
```

The helper uses `MUGA_BIN` when set, or `muga` from `PATH` otherwise. Set
`MUGA_PROGRAM`, `MUGA_BUNDLE_DIR`, or `MUGA_ARCHIVE_DIR` to override defaults,
or `MUGA_INSTALL_DIR` to additionally install and list the launcher in an
explicit bin directory. The bundle directory must be absent or empty because
`emit-app-bundle` does not overwrite app bundles.
Muga does not edit shell startup files.
"#,
                },
                TemplateFile {
                    relative: "scripts/package-app.sh",
                    text: r#"#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
project_dir=$(CDPATH= cd "$script_dir/.." && pwd)
MUGA_BIN=${MUGA_BIN:-muga}
program=${MUGA_PROGRAM:-app}
bundle_dir=${MUGA_BUNDLE_DIR:-"$project_dir/bundle"}
archive_dir=${MUGA_ARCHIVE_DIR:-"$project_dir/app-archives"}
entry="$project_dir/src/main/main.muga"

"$MUGA_BIN" emit-app-bundle --source-free --output-dir "$bundle_dir" --program "$program" "$entry"
"$MUGA_BIN" run-app-bundle "$bundle_dir" -- --name=Ada
archive_output=$("$MUGA_BIN" emit-app-archive --archive-root "$archive_dir" --program "$program" "$bundle_dir")
printf '%s\n' "$archive_output"
archive_path=$(printf '%s\n' "$archive_output" | sed -n 's/^archive[[:space:]]//p' | tail -n 1)
if [ -z "$archive_path" ]; then
  archive_path=$(printf '%s\n' "$archive_output" | sed -n '/\.mga$/p' | tail -n 1)
fi
if [ -z "$archive_path" ]; then
  printf '%s\n' "package helper could not find emitted archive path" >&2
  exit 1
fi
"$MUGA_BIN" verify-app-archive "$archive_path"
if [ "${MUGA_INSTALL_DIR:-}" != "" ]; then
  "$MUGA_BIN" install-app --replace-owned --output-dir "$MUGA_INSTALL_DIR" --program "$program" "$bundle_dir"
  "$MUGA_BIN" list-installed-apps --output-dir "$MUGA_INSTALL_DIR"
fi
"#,
                },
            ],
        ),
        ProjectTemplate::Lib => (
            "src/lib/main.muga",
            &[
                TemplateFile {
                    relative: "src/lib/main.muga",
                    text: r#"pub fn value(): Int {
  1
}
"#,
                },
                TemplateFile {
                    relative: "README.md",
                    text: r#"# Library

Generated library package starter.

## Check

```sh
muga check src/lib/main.muga
muga doc src/lib/main.muga
muga build src/lib/main.muga
muga check --built src/lib/main.muga
```
"#,
                },
            ],
        ),
        ProjectTemplate::Test => (
            "src/main/main.muga",
            &[
                TemplateFile {
                    relative: "src/main/main.muga",
                    text: r#"import std::test

pub fn answer(): Int {
  42
}

@test
fn answer_is_42(): Result[Unit, String] {
  test::assert_eq_int(answer(), 42)
}
"#,
                },
                TemplateFile {
                    relative: "README.md",
                    text: r#"# Tests

Generated test package starter.

## Run

```sh
muga test src/main/main.muga
muga test --format json src/main/main.muga
muga doc src/main/main.muga
```
"#,
                },
            ],
        ),
        ProjectTemplate::ConfigApp => (
            "src/main/main.muga",
            &[
                TemplateFile {
                    relative: "src/main/main.muga",
                    text: r#"import std::cli
import std::config
import std::env
import std::path
import std::result
import std::string

pub record Owner {
  name: Option[String]
  team: Option[String]
}

pub record Server {
  name: String
  port: Int
}

pub record Settings {
  @cli(help: "Application display name")
  name: String
  @cli(help: "HTTP listen port")
  port: Int
  @cli(help: "Enable verbose logging")
  verbose: Bool
  @cli(name: "tag", alias: "tags", help: "Tag value")
  tags: List[String]
  owner: Owner
  servers: List[Server]
  limits: Map[String, Int]
}

fn default_owner(): Owner {
  name: Option[String] = Option::None
  team: Option[String] = Option::None
  Owner { name: name, team: team }
}

fn default_servers(): List[Server] {
  [Server { name: "local", port: 8080 }]
}

fn default_limits(): Map[String, Int] {
  limits: Map[String, Int] = Map.empty()
  limits.insert("workers", 1).insert("retries", 1)
}

fn default_settings(): Settings {
  tags: List[String] = []
  Settings { name: "Muga", port: 8080, verbose: false, tags: tags, owner: default_owner(), servers: default_servers(), limits: default_limits() }
}

fn config_error_kind_name(kind: config::ErrorKind): String {
  match kind {
    config::ErrorKind::Read => "Read"
    config::ErrorKind::Parse => "Parse"
    config::ErrorKind::Decode => "Decode"
  }
}

fn config_error_message(error: config::Error): String {
  string::concat_all(["config ", config_error_kind_name(error.kind), " ", error.offset.to_string(), ": ", error.message])
}

fn discovered_config_path(): String {
  match env::get_var("MUGA_CONFIG_PATH") {
    Option::Some(value) => value
    Option::None => "config/settings.json"
  }
}

fn cli_error_kind_name(kind: cli::ErrorKind): String {
  match kind {
    cli::ErrorKind::UnknownArgument => "UnknownArgument"
    cli::ErrorKind::MissingArgument => "MissingArgument"
    cli::ErrorKind::MissingValue => "MissingValue"
    cli::ErrorKind::InvalidValue => "InvalidValue"
    cli::ErrorKind::Validation => "Validation"
    cli::ErrorKind::UnsupportedTarget => "UnsupportedTarget"
  }
}

fn cli_error_message(error: cli::Error): String {
  string::concat_all(["cli ", cli_error_kind_name(error.kind), " ", error.argument, ": ", error.message])
}

fn config_value_follows(args: List[String], index: Int): Bool {
  match args.get(index + 1) {
    Option::Some(value) => !value.starts_with("--")
    Option::None => false
  }
}

fn settings_args(args: List[String]): List[String] {
  mut out: List[String] = []
  mut index = 0
  mut skip_next = false
  for arg in args {
    if skip_next {
      skip_next = false
    } else {
      if arg == "--" {
        return out
      }
      if arg == "--config" {
        skip_next = config_value_follows(args, index)
      } else if arg.starts_with("--config=") {
      } else {
        out = out.push(arg)
      }
    }
    index = index + 1
  }
  out
}

fn option_string(value: Option[String], fallback: String): String {
  match value {
    Option::Some(text) => text
    Option::None => fallback
  }
}

fn first_server_port(servers: List[Server]): Int {
  match servers.get(0) {
    Option::Some(server) => server.port
    Option::None => 0
  }
}

fn limit_or(limits: Map[String, Int], key: String, fallback: Int): Int {
  match limits.get(key) {
    Option::Some(value) => value
    Option::None => fallback
  }
}

fn render_settings(settings: Settings): String {
  owner = option_string(settings.owner.name, "unknown")
  team = option_string(settings.owner.team, "none")
  first_port = first_server_port(settings.servers)
  workers = limit_or(settings.limits, "workers", 0)
  string::join([settings.name, settings.port.to_string(), settings.verbose.to_string(), settings.tags.len().to_string(), owner, team, settings.servers.len().to_string(), first_port.to_string(), workers.to_string()], "|")
}

fn emit_usage(usage: String): Result[String, String] {
  printed = println(usage)
  Result::Ok(usage)
}

fn run_config(args: List[String]): Result[String, String] {
  config_path = path::from_string(cli::option_or(args, "config", discovered_config_path()))
  configured = try result::map_err(config::load_json_or(config_path, default_settings()), config_error_message)
  settings = try result::map_err(cli::parse_or(settings_args(args), configured), cli_error_message)
  rendered = render_settings(settings)
  printed = println(string::concat_all(["config ", rendered]))
  Result::Ok(rendered)
}

fn main(): Result[String, String] {
  args = env::args()
  request = try result::map_err(cli::parse_request_or(settings_args(args), "config-app", default_settings()), cli_error_message)
  match request {
    cli::Request::Help(usage) => emit_usage(string::concat_all([usage, "\n  --config <Path>  default: $MUGA_CONFIG_PATH or config/settings.json"]))
    cli::Request::Parsed(_) => run_config(args)
  }
}
"#,
                },
                TemplateFile {
                    relative: "config/settings.json",
                    text: r#"{
  "name": "Ada",
  "port": 4040,
  "verbose": false,
  "tags": ["tool", "service"],
  "owner": {
    "name": "ops",
    "team": null
  },
  "servers": [
    {
      "name": "api",
      "port": 9000
    },
    {
      "name": "worker",
      "port": 9001
    }
  ],
  "limits": {
    "workers": 4,
    "retries": 2
  }
}
"#,
                },
                TemplateFile {
                    relative: "README.md",
                    text: r#"# Config App

This generated app loads typed settings from JSON, then applies CLI overrides.

Run with an explicit config path:

```sh
muga run src/main/main.muga -- --config config/settings.json --port=5050
```

Run with the generated config helper:

```sh
sh scripts/run-with-config.sh --tag ops
```

The helper sets `MUGA_CONFIG_PATH` to this project's `config/settings.json`
unless the environment already provides a value. It uses `MUGA_BIN` when set,
or `muga` from `PATH` otherwise.

Inspect project metadata for wrappers and editor tooling:

```sh
muga workspace --format json src/main/main.muga
```

Build and run against default artifacts:

```sh
muga build src/main/main.muga
muga run --built src/main/main.muga -- --config config/settings.json
```

Package a source-free app bundle, generate app completions, archive it as
`.mga`, verify the archive, and optionally install/list it when
`MUGA_INSTALL_DIR` points at an explicit bin directory:

```sh
sh scripts/package-config-app.sh
```
"#,
                },
                TemplateFile {
                    relative: "scripts/run-with-config.sh",
                    text: r#"set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
project_dir=$(CDPATH= cd "$script_dir/.." && pwd)
MUGA_BIN=${MUGA_BIN:-muga}
MUGA_CONFIG_PATH=${MUGA_CONFIG_PATH:-"$project_dir/config/settings.json"}
export MUGA_CONFIG_PATH

"$MUGA_BIN" run "$project_dir/src/main/main.muga" -- "$@"
"#,
                },
                TemplateFile {
                    relative: "scripts/package-config-app.sh",
                    text: r#"#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
project_dir=$(CDPATH= cd "$script_dir/.." && pwd)
MUGA_BIN=${MUGA_BIN:-muga}
program=${MUGA_PROGRAM:-config-app}
bundle_dir=${MUGA_BUNDLE_DIR:-"$project_dir/dist/$program"}
archive_dir=${MUGA_ARCHIVE_DIR:-"$project_dir/dist/app-archives"}
completions_dir=${MUGA_COMPLETIONS_DIR:-"$project_dir/dist/completions"}
config_path=${MUGA_CONFIG_PATH:-"$project_dir/config/settings.json"}
entry="$project_dir/src/main/main.muga"

"$MUGA_BIN" emit-app-bundle --source-free --output-dir "$bundle_dir" --program "$program" "$entry"
MUGA_CONFIG_PATH="$config_path" "$MUGA_BIN" run-app-bundle "$bundle_dir" -- --tag packaged
"$MUGA_BIN" emit-app-completions --format json --output-dir "$completions_dir" --program "$program" --type Settings "$bundle_dir"
archive_output=$("$MUGA_BIN" emit-app-archive --archive-root "$archive_dir" --program "$program" "$bundle_dir")
printf '%s\n' "$archive_output"
archive_path=$(printf '%s\n' "$archive_output" | sed -n 's/^archive[[:space:]]//p' | tail -n 1)
if [ -z "$archive_path" ]; then
  archive_path=$(printf '%s\n' "$archive_output" | sed -n '/\.mga$/p' | tail -n 1)
fi
if [ -z "$archive_path" ]; then
  printf '%s\n' "package helper could not find emitted archive path" >&2
  exit 1
fi
"$MUGA_BIN" verify-app-archive "$archive_path"
if [ "${MUGA_INSTALL_DIR:-}" != "" ]; then
  "$MUGA_BIN" install-app --replace-owned --output-dir "$MUGA_INSTALL_DIR" --program "$program" "$bundle_dir"
  "$MUGA_BIN" list-installed-apps --output-dir "$MUGA_INSTALL_DIR"
fi
"#,
                },
            ],
        ),
        ProjectTemplate::ReportApp => (
            "src/main/main.muga",
            &[
                TemplateFile {
                    relative: "src/main/main.muga",
                    text: r#"import std::cli
import std::env
import std::fs
import std::io
import std::path
import std::result
import std::string

fn io_error_message(error: io::IOError): String {
  string::concat_all([error.operation, " ", error.path, ": ", error.message])
}

fn default_output_path(source_path: String): String {
  path::as_string(path::with_extension(path::from_string(source_path), "summary.txt"))
}

fn selected_input_path(args: List[String]): String {
  cli::positional_or(args, 0, "data/daily.txt")
}

fn output_path(args: List[String], input: String): String {
  cli::positional_or(args, 1, default_output_path(input))
}

fn summary_line(source_text: String): String {
  "daily: ".concat(source_text.trim())
}

fn render_report(input: String, output: String, source_text: String, metadata: fs::FileMetadata): String {
  summary = summary_line(source_text)
  string::concat_all(["summary: ", summary, "\nsource: ", input, "\nbytes: ", metadata.size.to_string(), "\noutput: ", output, "\n"])
}

fn main(): Result[String, String] {
  args = env::args()
  input = selected_input_path(args)
  output = output_path(args, input)
  metadata = try result::map_err(fs::file_metadata_path(path::from_string(input)), io_error_message)
  source_text = try result::map_err(fs::read_text(input), io_error_message)
  report = render_report(input, output, source_text, metadata)
  wrote = try result::map_err(fs::write_text(output, report), io_error_message)
  printed = println(report)
  Result::Ok(summary_line(source_text))
}
"#,
                },
                TemplateFile {
                    relative: "data/daily.txt",
                    text: r#"launch metrics healthy
"#,
                },
                TemplateFile {
                    relative: "README.md",
                    text: r#"# Report App

Generated file-processing report starter.

## Run

Run from this project directory:

```sh
muga run src/main/main.muga
muga run src/main/main.muga -- data/daily.txt data/custom-summary.txt
muga build src/main/main.muga
muga run --built src/main/main.muga -- data/daily.txt data/built-summary.txt
```

Run from any current directory through the helper:

```sh
sh scripts/run-report.sh
sh scripts/run-report.sh data/daily.txt data/script-summary.txt
```

The helper changes to this project root before running so relative data paths
resolve to the generated `data/` directory. It uses `MUGA_BIN` when set, or
`muga` from `PATH` otherwise.

## Package

Create a source-free app bundle, run it against the generated data fixture,
archive the bundle as `.mga`, verify the generated archive, and optionally
install/list it when `MUGA_INSTALL_DIR` points at an explicit bin directory:

```sh
sh scripts/package-report-app.sh
```
"#,
                },
                TemplateFile {
                    relative: "scripts/run-report.sh",
                    text: r#"#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
project_dir=$(CDPATH= cd "$script_dir/.." && pwd)
MUGA_BIN=${MUGA_BIN:-muga}

cd "$project_dir"
"$MUGA_BIN" run "$project_dir/src/main/main.muga" -- "$@"
"#,
                },
                TemplateFile {
                    relative: "scripts/package-report-app.sh",
                    text: r#"#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
project_dir=$(CDPATH= cd "$script_dir/.." && pwd)
MUGA_BIN=${MUGA_BIN:-muga}
program=${MUGA_PROGRAM:-report-app}
bundle_dir=${MUGA_BUNDLE_DIR:-"$project_dir/dist/$program"}
archive_dir=${MUGA_ARCHIVE_DIR:-"$project_dir/dist/app-archives"}
entry="$project_dir/src/main/main.muga"
input_path=${MUGA_REPORT_INPUT:-"$project_dir/data/daily.txt"}
output_path=${MUGA_REPORT_OUTPUT:-"$project_dir/dist/package-summary.txt"}

"$MUGA_BIN" emit-app-bundle --source-free --output-dir "$bundle_dir" --program "$program" "$entry"
"$MUGA_BIN" run-app-bundle "$bundle_dir" -- "$input_path" "$output_path"
archive_output=$("$MUGA_BIN" emit-app-archive --archive-root "$archive_dir" --program "$program" "$bundle_dir")
printf '%s\n' "$archive_output"
archive_path=$(printf '%s\n' "$archive_output" | sed -n 's/^archive[[:space:]]//p' | tail -n 1)
if [ -z "$archive_path" ]; then
  archive_path=$(printf '%s\n' "$archive_output" | sed -n '/\.mga$/p' | tail -n 1)
fi
if [ -z "$archive_path" ]; then
  printf '%s\n' "package helper could not find emitted archive path" >&2
  exit 1
fi
"$MUGA_BIN" verify-app-archive "$archive_path"
if [ "${MUGA_INSTALL_DIR:-}" != "" ]; then
  "$MUGA_BIN" install-app --replace-owned --output-dir "$MUGA_INSTALL_DIR" --program "$program" "$bundle_dir"
  "$MUGA_BIN" list-installed-apps --output-dir "$MUGA_INSTALL_DIR"
fi
"#,
                },
            ],
        ),
        ProjectTemplate::ResourceExport => (
            "src/main/main.muga",
            &[
                TemplateFile {
                    relative: "src/main/main.muga",
                    text: r#"import std::bytes
import std::cli
import std::env
import std::fs
import std::hash
import std::io
import std::path
import std::string

fn selected_output_path(args: List[String]): path::Path {
  path::from_string(cli::positional_or(args, 0, "dist/payload.bin"))
}

fn ensure_parent(output: path::Path): Result[Unit, io::IOError] {
  match path::parent(output) {
    Option::Some(parent) => fs::create_dir_all_path(parent)
    Option::None => Result::Ok(())
  }
}

fn kind_name(kind: fs::PathKind): String {
  match kind {
    fs::PathKind::Missing => "missing"
    fs::PathKind::File => "file"
    fs::PathKind::Directory => "directory"
    fs::PathKind::Other => "other"
  }
}

fn render_result(size: Int, metadata: fs::PathMetadata, digest: String, output: path::Path): String {
  string::join([size.to_string(), kind_name(metadata.kind), metadata.status.is_file.to_string(), digest, path::as_string(output)], "|")
}

fn main(): Result[String, io::IOError] {
  output = selected_output_path(env::args())
  data = try fs::read_resource_bytes("{{package_name}}", "static/payload.bin")
  digest = hash::sha256_hex(data)
  made_parent = try ensure_parent(output)
  wrote = try fs::write_bytes_path(output, data)
  metadata = try fs::path_metadata_path(output)
  written = try fs::read_bytes_path(output)
  Result::Ok(render_result(bytes::size(written), metadata, digest, output))
}
"#,
                },
                TemplateFile {
                    relative: "resources/static/payload.bin",
                    text: r#"Muga resource payload
"#,
                },
                TemplateFile {
                    relative: "README.md",
                    text: r#"# Resource Export

Generated binary resource export app starter.

## Run

Run from this project directory:

```sh
muga run src/main/main.muga
muga run src/main/main.muga -- dist/custom-payload.bin
muga build src/main/main.muga
muga run --built src/main/main.muga -- dist/built-payload.bin
```

The app declares `[package] resources = "resources"`, reads
`resources/static/payload.bin` as package-owned `Bytes`, writes it to the
selected output path, verifies file metadata, reads it back, and returns
`size|kind|is_file|sha256|output`.

## Package

Create a source-free app bundle, run it against the bundled resource, archive
the bundle as `.mga`, verify the generated archive, and optionally install/list
it when `MUGA_INSTALL_DIR` points at an explicit bin directory:

```sh
sh scripts/package-resource-export.sh
```
"#,
                },
                TemplateFile {
                    relative: "scripts/package-resource-export.sh",
                    text: r#"#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
project_dir=$(CDPATH= cd "$script_dir/.." && pwd)
MUGA_BIN=${MUGA_BIN:-muga}
program=${MUGA_PROGRAM:-resource-export}
bundle_dir=${MUGA_BUNDLE_DIR:-"$project_dir/dist/$program"}
archive_dir=${MUGA_ARCHIVE_DIR:-"$project_dir/dist/app-archives"}
entry="$project_dir/src/main/main.muga"
output_path=${MUGA_RESOURCE_OUTPUT:-"$project_dir/dist/package-payload.bin"}

"$MUGA_BIN" emit-app-bundle --source-free --output-dir "$bundle_dir" --program "$program" "$entry"
"$MUGA_BIN" run-app-bundle "$bundle_dir" -- "$output_path"
archive_output=$("$MUGA_BIN" emit-app-archive --archive-root "$archive_dir" --program "$program" "$bundle_dir")
printf '%s\n' "$archive_output"
archive_path=$(printf '%s\n' "$archive_output" | sed -n 's/^archive[[:space:]]//p' | tail -n 1)
if [ -z "$archive_path" ]; then
  archive_path=$(printf '%s\n' "$archive_output" | sed -n '/\.mga$/p' | tail -n 1)
fi
if [ -z "$archive_path" ]; then
  printf '%s\n' "package helper could not find emitted archive path" >&2
  exit 1
fi
"$MUGA_BIN" verify-app-archive "$archive_path"
if [ "${MUGA_INSTALL_DIR:-}" != "" ]; then
  "$MUGA_BIN" install-app --replace-owned --output-dir "$MUGA_INSTALL_DIR" --program "$program" "$bundle_dir"
  "$MUGA_BIN" list-installed-apps --output-dir "$MUGA_INSTALL_DIR"
fi
"#,
                },
            ],
        ),
        ProjectTemplate::CliTool => (
            "src/main/main.muga",
            &[
                TemplateFile {
                    relative: "src/main/main.muga",
                    text: r#"import std::cli
import std::env
import std::result
import std::string

pub enum Action {
  Audit
  Apply
}

@cli(about: "Run a typed strict CLI tool")
pub record Root {
  @cli(name: "profile", short: "p", help: "Execution profile")
  profile: Option[String]
  @cli(subcommand)
  command: Command
}

@cli(about: "Run a typed strict CLI tool")
pub enum Command {
  @cli(name: "run", alias: "r", about: "Run the main action")
  Run(RunCommand)
  @cli(name: "inspect", alias: "i", about: "Inspect one target")
  Inspect(InspectCommand)
}

@cli(about: "Run the main action")
pub record RunCommand {
  @cli(positional: 1, help: "Target resource name")
  @validate(non_empty)
  target: String
  @cli(name: "count", short: "c", help: "Number of items to process")
  @validate(min: 1, max: 10)
  count: Int
  @cli(short: "a", help: "Command action")
  action: Action
  @cli(name: "dry-run", short: "d", help: "Preview changes without applying them")
  dry_run: Bool
  @cli(name: "tag", short: "T", alias: "tags", help: "Tag filter")
  tags: List[String]
  @cli(short: "o", help: "Optional owner")
  owner: Option[String]
}

@cli(about: "Inspect one target")
pub record InspectCommand {
  @cli(positional: 1, help: "Target resource name")
  @validate(non_empty)
  target: String
  @cli(short: "v", help: "Show verbose inspection details")
  verbose: Bool
}

fn action_text(action: Action): String {
  match action {
    Action::Audit => "Audit"
    Action::Apply => "Apply"
  }
}

fn option_text(value: Option[String]): String {
  match value {
    Option::Some(text) => text
    Option::None => "none"
  }
}

fn tags_text(tags: List[String]): String {
  if tags.len() == 0 {
    "none"
  } else {
    string::join(tags, ",")
  }
}

fn render_run(command: RunCommand): String {
  string::join(["run", command.target, command.count.to_string(), action_text(command.action), command.dry_run.to_string(), tags_text(command.tags), option_text(command.owner)], "|")
}

fn render_inspect(command: InspectCommand): String {
  string::join(["inspect", command.target, command.verbose.to_string()], "|")
}

fn render_command(command: Command): String {
  match command {
    Command::Run(run_options) => render_run(run_options)
    Command::Inspect(inspect_command) => render_inspect(inspect_command)
  }
}

fn render_root(root: Root): String {
  rendered = render_command(root.command)
  match root.profile {
    Option::Some(profile) => string::join(["profile", profile, rendered], "|")
    Option::None => rendered
  }
}

fn cli_error_kind_name(kind: cli::ErrorKind): String {
  match kind {
    cli::ErrorKind::UnknownArgument => "UnknownArgument"
    cli::ErrorKind::MissingArgument => "MissingArgument"
    cli::ErrorKind::MissingValue => "MissingValue"
    cli::ErrorKind::InvalidValue => "InvalidValue"
    cli::ErrorKind::Validation => "Validation"
    cli::ErrorKind::UnsupportedTarget => "UnsupportedTarget"
  }
}

fn cli_error_message(error: cli::Error): String {
  string::concat_all(["cli ", cli_error_kind_name(error.kind), " ", error.argument, ": ", error.message])
}

fn emit_usage(usage: String): Result[String, String] {
  printed = println(usage)
  Result::Ok(usage)
}

fn run_command(root: Root): Result[String, String] {
  rendered = render_root(root)
  printed = println(string::concat_all(["cli-tool ", rendered]))
  Result::Ok(rendered)
}

fn main(): Result[String, String] {
  args = env::args()
  request = try result::map_err(cli::parse_request[Root](args, "cli-tool"), cli_error_message)
  match request {
    cli::Request::Help(usage) => emit_usage(usage)
    cli::Request::Parsed(root) => run_command(root)
  }
}
"#,
                },
                TemplateFile {
                    relative: "README.md",
                    text: r#"# cli-tool

Generated strict CLI tool starter.

## Run

```bash
muga run src/main/main.muga -- --help
muga run src/main/main.muga -- --profile dev run service --count 3 --action Audit --dry-run --tag ops --owner Kai
muga build src/main/main.muga
muga run --built src/main/main.muga -- inspect service --verbose
```

## Shell Completions

Generate app completions from the `Root` CLI schema. The command prints the
script to stdout, so redirect it to the path your shell or package manager
expects.

```bash
muga cli-completions bash --program cli-tool --type Root src/main/main.muga
muga cli-completions zsh --program cli-tool --type Root --built src/main/main.muga
muga cli-completions fish --program cli-tool --type Root src/main/main.muga
```

To generate bash, zsh, fish, and a shell-agnostic JSON completion spec into
`completions/`, including `cli-tool.completions.json`, run:

```bash
muga emit-cli-completions --format json --output-dir completions --program cli-tool --type Root src/main/main.muga
sh scripts/generate-completions.sh
```

## Package

Create a source-free app bundle, generate completions from bundle interfaces,
archive the bundle as `.mga`, verify the generated archive, and optionally
install/list it when `MUGA_INSTALL_DIR` points at an explicit bin directory:

```bash
sh scripts/package-cli-tool.sh
```
"#,
                },
                TemplateFile {
                    relative: "scripts/generate-completions.sh",
                    text: r#"#!/usr/bin/env sh
set -eu

entry="${1:-src/main/main.muga}"
out_dir="${2:-completions}"

muga emit-cli-completions --format json --output-dir "$out_dir" --program cli-tool --type Root "$entry"
"#,
                },
                TemplateFile {
                    relative: "scripts/package-cli-tool.sh",
                    text: r#"#!/usr/bin/env sh
set -eu

script_dir=$(CDPATH= cd "$(dirname "$0")" && pwd)
project_dir=$(CDPATH= cd "$script_dir/.." && pwd)
MUGA_BIN=${MUGA_BIN:-muga}
program=${MUGA_PROGRAM:-cli-tool}
bundle_dir=${MUGA_BUNDLE_DIR:-"$project_dir/dist/$program"}
archive_dir=${MUGA_ARCHIVE_DIR:-"$project_dir/dist/app-archives"}
completions_dir=${MUGA_COMPLETIONS_DIR:-"$project_dir/dist/completions"}
entry="$project_dir/src/main/main.muga"

"$MUGA_BIN" emit-app-bundle --source-free --output-dir "$bundle_dir" --program "$program" "$entry"
"$MUGA_BIN" run-app-bundle "$bundle_dir" -- run service --count 3 --action Audit
"$MUGA_BIN" emit-app-completions --format json --output-dir "$completions_dir" --program "$program" --type Root "$bundle_dir"
archive_output=$("$MUGA_BIN" emit-app-archive --archive-root "$archive_dir" --program "$program" "$bundle_dir")
printf '%s\n' "$archive_output"
archive_path=$(printf '%s\n' "$archive_output" | sed -n 's/^archive[[:space:]]//p' | tail -n 1)
if [ -z "$archive_path" ]; then
  archive_path=$(printf '%s\n' "$archive_output" | sed -n '/\.mga$/p' | tail -n 1)
fi
if [ -z "$archive_path" ]; then
  printf '%s\n' "package helper could not find emitted archive path" >&2
  exit 1
fi
"$MUGA_BIN" verify-app-archive "$archive_path"
if [ "${MUGA_INSTALL_DIR:-}" != "" ]; then
  "$MUGA_BIN" install-app --replace-owned --output-dir "$MUGA_INSTALL_DIR" --program "$program" "$bundle_dir"
  "$MUGA_BIN" list-installed-apps --output-dir "$MUGA_INSTALL_DIR"
fi
"#,
                },
            ],
        ),
    }
}

fn inferred_package_name(root: &Path) -> String {
    let raw = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("muga_app");
    let mut name = String::new();
    let mut last_was_underscore = false;
    for ch in raw.chars() {
        let next = if ch.is_ascii_alphanumeric() || ch == '_' {
            ch.to_ascii_lowercase()
        } else {
            '_'
        };
        if next == '_' {
            if !last_was_underscore {
                name.push(next);
            }
            last_was_underscore = true;
        } else {
            name.push(next);
            last_was_underscore = false;
        }
    }
    let name = name.trim_matches('_').to_string();
    if name.is_empty() {
        return "muga_app".to_string();
    }
    if name
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
    {
        name
    } else {
        format!("muga_{name}")
    }
}

fn escape_manifest_string(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn escape_muga_string(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            ch => escaped.push(ch),
        }
    }
    escaped
}
