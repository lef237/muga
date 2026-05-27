# Generated Report App Template

Status: implemented. This records why `muga new --template report-app` is a
single-project starter instead of a direct copy of the checked-in
`samples/projects/report_app` local-dependency workflow.

## Goal

Turn the existing report workflow into something a new user can generate,
run, build, and re-run from artifacts without copying repository sample files.
The template should demonstrate practical file input, sidecar output,
recoverable `Result` error handling, a small run helper, and a source-free
package helper while avoiding new manifest or installer policy.

## Options

| Option | Benefit | Cost | Decision |
|---|---|---|---|
| Copy the checked-in `report_app` plus `report_shared` projects | Shows local dependency boundaries exactly as the sample does | Too much report-specific structure for the first report starter; local dependency teaching now belongs to `package-app`. | Defer |
| Generate a single-project report app | Smallest runnable starter; works with current `muga new`, `muga run`, `muga build`, and `muga run --built`; easy to document and test | Does not teach local dependency packaging by itself | Implement |
| Adopt the regular-file `FileMetadata` record | Uses the narrow accepted metadata API in the first report starter without changing the generated workflow. | Still does not teach all-path metadata or directory policy. | Implement after `FileMetadata` landed |
| Add generated `scripts/package-report-app.sh` | Reuses source-free app bundles and `.mga` archive verification to make the report starter distributable without broadening `muga new`. | Adds one generated script and keeps package-manager/install policy outside the template. | Implement |
| Add all-path filesystem metadata records first | Useful for later richer report tooling | Freezes directory, symlink, permission, and timestamp policy before the first-project path needs it | Defer |
| Add recursive directory copy/remove first | Useful for larger tools | Higher host-effect and safety policy cost; not needed for a first report app | Defer |

## Implemented Shape

`muga new --template report-app <project-dir>` generates:

- `src/main/main.muga`, using `std::env`, `std::cli`, `std::fs`,
  `std::path`, `std::result`, and `std::string`.
- `data/daily.txt`, a small input fixture.
- `README.md`, with source and built-artifact run commands.
- `scripts/run-report.sh`, which changes to the project root and respects
  `MUGA_BIN` without editing shell startup files.
- `scripts/package-report-app.sh`, which emits a source-free app bundle, runs
  it against `data/daily.txt`, archives the bundle as `.mga`, and verifies the
  archive. `MUGA_REPORT_INPUT` and `MUGA_REPORT_OUTPUT` can override the
  package-smoke input and output paths, and `MUGA_INSTALL_DIR` can explicitly
  install/list the generated launcher.

The generated program reads `data/daily.txt` by default, derives a sidecar
`data/daily.summary.txt` path with `path::with_extension`, reads regular-file
metadata with `fs::file_metadata_path`, writes a rendered summary with
`fs::write_text`, prints the summary, and returns
`Result[String, String]`.

## Deferrals

Keep broader workspace policy, project-root runtime discovery, package resource
defaults, recursive directory operations, all-path metadata records, binary
writes/streams/codecs, process APIs, shell-profile mutation, and registry
publishing as separate decisions. The checked-in `samples/projects/report_app`
remains the rich report workflow path; `package-app` is the generated local
dependency teaching path; this template is the first-project report starter.

## Validation

Coverage lives in `cli_new_creates_report_app_template`: generation,
manifest-name inference, source contents, README/helper contents, `fmt --check`,
source run from the project root, build, `run --built`, and helper execution
from another current directory, plus source-free package helper execution.
Release readiness locks CLI parsing, static shell completion labels, docs,
implementation-resume evidence, and this design record.
