# Standard Library Package Samples Review

Status: completed v1 hardening review. This document records the
stdlib package docs and samples review for `std::io`, `std::fs`, `std::path`,
`std::env`, `std::cli`, `std::time`, `std::string`, `std::fmt`, `std::json`,
`std::bytes`, and `std::hash` slices. It is evidence for the current v1
package surface, not a request to add broader IO, process, resource-handle,
schema, formatting, crypto, or time APIs.

## Review Result

The current package docs and runnable samples cover the implemented standard
library slices and keep their effect model visible:

- `std::io` provides public error data contracts for filesystem effects.
- `std::path` provides transparent path values and pure path helpers.
- `std::fs` provides one-shot text, directory, copy, rename, scalar file-size
  and modified-time metadata, regular-file `FileMetadata`, path status/info
  grouping, existing-path metadata, optional regular-file size metadata for
  existing paths, existing-path
  canonicalization, removal, and predicate helpers plus read-only
  binary/resource reads with explicit `Result` return types where effects can
  fail.
- `std::env` exposes environment lookup as `Option[String]`, program
  arguments as `List[String]`, and the current working directory as
  `Result[path::Path, io::IOError]`.
- `std::cli` exposes pure argument, repeated option value, and typed scalar
  parsing helpers over explicit `List[String]` values without reading host
  process state.
- `std::time` exposes the clock read through an explicitly named function that
  returns transparent `time::UnixMillis` data.
- `std::string` exposes pure text assembly helpers over explicit
  `List[String]` values without implicit conversion, formatting templates, or
  builders.
- `std::json` exposes parse/encode helpers, value/object-field accessors, JSON
  path helpers, and compiler-owned schema decoders through explicit
  `json::Value`, `json::Number`, `json::ErrorKind`, and `json::Error` data
  contracts, including required-field and composite-field helpers, with
  recoverable failures returned as `Result`.
- `std::bytes` exposes opaque binary data with size, empty, and zero-based byte
  inspection helpers, without mutable buffers or codecs.
- `std::hash` exposes a single SHA-256 hex helper over `Bytes`, without broader
  cryptographic APIs.

The runnable samples under `samples/packages/app/` are source-compatible
examples. The artifact-backed execution samples are represented by focused
`tests/examples.rs` tests that emit `.mgi` / `.mgb` artifacts and run against
the artifact root without depending on private dependency source bodies.

## Package Evidence

| Package | Public surface reviewed | Runnable samples | Artifact-backed coverage |
|---|---|---|---|
| `std::io` | `io::IOError`, `io::PathPairError` | `samples/packages/app/std_io/main.muga`; also used through `std::fs` samples | `standard_fs_artifact_run_exposes_error_fields_without_direct_io_import`, `standard_fs_artifact_run_uses_emitted_std_implementations`, `standard_fs_copy_file_artifact_run_uses_emitted_std_implementations`, `standard_fs_copy_dir_all_artifact_run_uses_emitted_std_implementations` |
| `std::path` | `path::Path`, `from_string`, `as_string`, `join`, `normalize`, `file_name`, `with_file_name`, `parent`, `strip_prefix`, `extension`, `file_stem`, `with_extension`, `is_absolute` | `std_path`, `std_path_join`, `std_path_normalize`, `std_path_file_name`, `std_path_with_file_name`, `std_path_parent`, `std_path_strip_prefix`, `std_path_extension`, `std_path_file_stem`, `std_path_with_extension`, `std_path_is_absolute` | `standard_path_artifact_run_uses_emitted_std_implementations`, `standard_path_join_artifact_run_uses_emitted_std_implementations`, `standard_path_normalize_artifact_run_uses_emitted_std_implementations`, `standard_path_file_name_artifact_run_uses_emitted_std_implementations`, `standard_path_with_file_name_artifact_run_uses_emitted_std_implementations`, `standard_path_parent_artifact_run_uses_emitted_std_implementations`, `standard_path_strip_prefix_artifact_run_uses_emitted_std_implementations`, `standard_path_extension_artifact_run_uses_emitted_std_implementations`, `standard_path_file_stem_artifact_run_uses_emitted_std_implementations`, `standard_path_with_extension_artifact_run_uses_emitted_std_implementations`, `standard_path_is_absolute_artifact_run_uses_emitted_std_implementations` |
| `std::fs` | `FileMetadata`, `PathStatus`, `PathKind`, `PathInfo`, `PathMetadata`, `PathSizeMetadata`, `DirectorySizeMetadata`, `read_text`, `read_bytes`, `write_text`, `write_bytes`, `read_text_path`, `read_bytes_path`, `write_text_path`, `write_bytes_path`, `read_resource_text`, `read_resource_bytes`, `open_text`, `create_text`, `append_text`, `read_text_from`, `write_text_to`, `flush`, `close`, `read_dir_path`, `read_dir_recursive_path`, `directory_size_metadata_path`, `canonicalize_path`, `create_dir_path`, `create_dir_all_path`, `remove_file_path`, `remove_dir_path`, `remove_dir_all_path`, `copy_file_path`, `copy_dir_all_path`, `move_dir_all_path`, `rename_path`, `file_size_path`, `modified_unix_millis_path`, `file_metadata_path`, `path_status`, `path_kind`, `path_info`, `path_metadata_path`, `path_size_metadata_path`, `exists_path`, `is_file_path`, `is_dir_path` | `std_fs_path`, `std_fs_read_dir`, `std_fs_read_dir_recursive`, `std_fs_directory_size_metadata`, `std_fs_metadata`, `std_fs_path_metadata`, `std_fs_path_size_metadata`, `std_fs_create_dir`, `std_fs_create_dir_all`, `std_fs_remove_file`, `std_fs_remove_dir`, `std_fs_remove_dir_all`, `std_fs_copy_file`, `std_fs_copy_dir_all`, `std_fs_move_dir_all`, `std_fs_rename`, `std_fs_file_size`, `std_fs_modified_time`, `std_fs_file_metadata`, `std_fs_write_bytes`, `std_fs_canonicalize`; `samples/projects/report_app` covers local dependency use and text-file handle writes; `samples/projects/resource_export` covers manifest resource byte export, path metadata verification, and cleanup; generated `report-app` covers single-project text read/write reporting; `samples/packages/app/std_hash/main.muga` covers local byte reads | `standard_fs_artifact_run_uses_emitted_std_implementations`, `standard_fs_path_artifact_run_uses_emitted_std_implementations`, `standard_fs_metadata_artifact_run_uses_emitted_std_implementations`, `standard_fs_path_status_returns_public_record`, `standard_fs_path_info_returns_kind_and_status`, `standard_fs_path_metadata_artifact_run_uses_emitted_std_implementations`, `standard_fs_path_size_metadata_artifact_run_uses_emitted_std_implementations`, `standard_fs_directory_size_metadata_artifact_run_uses_emitted_std_implementations`, `standard_fs_create_dir_artifact_run_uses_emitted_std_implementations`, `standard_fs_create_dir_all_artifact_run_uses_emitted_std_implementations`, `standard_fs_remove_file_artifact_run_uses_emitted_std_implementations`, `standard_fs_remove_dir_artifact_run_uses_emitted_std_implementations`, `standard_fs_remove_dir_all_artifact_run_uses_emitted_std_implementations`, `standard_fs_copy_file_artifact_run_uses_emitted_std_implementations`, `standard_fs_copy_dir_all_artifact_run_uses_emitted_std_implementations`, `standard_fs_move_dir_all_artifact_run_uses_emitted_std_implementations`, `standard_fs_rename_artifact_run_uses_emitted_std_implementations`, `standard_fs_file_size_artifact_run_uses_emitted_std_implementations`, `standard_fs_modified_unix_millis_artifact_run_uses_emitted_std_implementations`, `standard_fs_file_metadata_artifact_run_uses_emitted_std_implementations`, `standard_fs_canonicalize_artifact_run_uses_emitted_std_implementations`, `standard_fs_read_dir_artifact_run_uses_emitted_std_implementations`, `standard_fs_read_dir_recursive_artifact_run_uses_emitted_std_implementations`, `standard_fs_file_handle_artifact_run_can_write_text`, `standard_fs_read_bytes_reads_file_and_indexes_bytes_for_source_and_built_runs`, `standard_fs_write_bytes_writes_binary_file_for_source_and_built_runs`, `standard_fs_read_resource_bytes_reads_manifest_entry_resources_for_source_and_built_runs`, `manifest_report_project_sample_runs_against_emitted_artifacts`, `manifest_resource_export_project_sample_runs_against_emitted_artifacts` |
| `std::env` | `env::get_var`, `env::args`, `env::current_dir`, `env::temp_dir` | `samples/packages/app/std_env/main.muga`, `samples/packages/app/std_env_args/main.muga`, `samples/packages/app/std_env_current_dir/main.muga`, `samples/packages/app/std_env_temp_dir/main.muga`; `samples/projects/report_app` and `samples/projects/config_app` use program args in practical workflows | `standard_env_artifact_run_uses_emitted_std_implementations`, `standard_env_args_artifact_run_uses_program_arguments`, `standard_env_current_dir_artifact_run_uses_emitted_std_implementations`, `standard_env_temp_dir_artifact_run_uses_emitted_std_implementations`, `manifest_report_project_sample_json_built_run_writes_report`, `manifest_config_project_sample_json_built_run_applies_cli_overrides` |
| `std::cli` | `cli::positional`, `cli::positional_or`, `cli::has_flag`, `cli::has_short_flag`, `cli::option`, `cli::option_or`, `cli::option_values`, `cli::option_values_or`, typed `Int` / `Bool` helpers, compiler-owned `cli::parse_or[T]`, `cli::parse[T]`, `cli::usage_for[T]`, `cli::usage_for_required[T]`, `cli::help_requested`, `cli::help_for[T]`, `cli::help_for_required[T]`, `cli::Request[T]`, `cli::parse_request[T]`, `cli::parse_request_or[T]`, command enum schemas, `cli::ErrorKind`, and `cli::Error` | `samples/packages/app/std_cli/main.muga`, `samples/packages/app/std_cli_schema/main.muga`; `samples/projects/report_app` uses `cli::positional_or` for input/output defaults; `samples/projects/config_app` uses `cli::parse_or[T]` for typed settings overlays; `samples/projects/cli_tool` uses a strict command enum with `run` / `inspect` subcommands, generated root/leaf help, compact short options, and `@cli(positional: 1)` for leaf target operands | `package_std_cli_sample_runs`, `package_std_cli_schema_sample_runs`, `package_std_cli_schema_sample_runs_against_emitted_artifacts`, `standard_cli_artifact_run_uses_emitted_std_implementations`, `standard_cli_typed_scalar_artifact_run_uses_emitted_std_implementations`, `standard_cli_parse_or_artifact_run_uses_schema_payload`, `standard_cli_parse_or_record_overlay_runs`, `standard_cli_parse_or_reports_recoverable_errors`, `standard_cli_usage_for_record_runs`, `standard_cli_field_metadata_parse_and_usage_runs`, `standard_cli_short_option_metadata_parse_and_usage_runs`, `standard_cli_short_option_metadata_artifact_run_uses_schema_payload`, `standard_cli_positional_field_metadata_parse_and_usage_runs`, `standard_cli_positional_field_metadata_artifact_run_uses_schema_payload`, `standard_cli_field_metadata_artifact_run_uses_schema_payload`, `standard_cli_parse_or_rejects_unsupported_targets`, `standard_cli_parse_request_required_record_runs`, `standard_cli_parse_request_or_record_overlay_runs`, `standard_cli_parse_request_artifact_run_uses_schema_payload`, `standard_cli_parse_request_rejects_invalid_contracts`, `standard_cli_subcommand_parse_request_runs`, `standard_cli_subcommand_parse_request_artifact_run_uses_schema_payload`, `standard_cli_subcommand_schema_rejects_invalid_contracts`, `standard_cli_repeated_option_values_run_as_virtual_package`, `standard_cli_typed_scalar_helpers_run_as_virtual_package`, `standard_cli_typed_scalar_helpers_report_parse_errors`, `standard_cli_positionals_flags_and_equal_options_run_as_virtual_package`, `standard_cli_separate_options_and_missing_values_return_options`, `standard_cli_double_dash_stops_option_parsing`, `manifest_cli_tool_project_sample_runs_with_required_options`, `manifest_cli_tool_project_sample_reports_generated_usage`, `manifest_cli_tool_project_sample_json_built_run_uses_strict_parse`, `manifest_config_project_sample_runs_with_cli_overrides`, `manifest_config_project_sample_json_built_run_applies_cli_overrides` |
| `std::time` | `time::UnixMillis`, `time::now_unix_millis` | `samples/packages/app/std_time/main.muga` | `standard_time_artifact_run_uses_emitted_std_implementations` |
| `std::string` | `string::concat_all`, `string::join` over explicit `List[String]` values | `samples/packages/app/std_string/main.muga`; `samples/projects/config_app` uses these helpers for app-boundary messages and rendered settings | `standard_string_helpers_run_as_virtual_package`, `standard_string_artifact_run_uses_emitted_std_implementations`, `standard_string_helpers_report_type_mismatches`, `standard_string_missing_import_suggests_import`, `manifest_config_project_sample_runs_against_emitted_artifacts` |
| `std::fmt` | `fmt::repeat`, `fmt::pad_left`, `fmt::pad_right`, `fmt::truncate_chars`, `fmt::format_values` over explicit `String` values | `samples/packages/app/std_fmt/main.muga` | `standard_fmt_layout_helpers_run_as_virtual_package`, `standard_fmt_format_values_runs_as_virtual_package`, `standard_fmt_format_values_reports_template_errors`, `standard_fmt_artifact_run_uses_emitted_std_implementations`, `standard_fmt_helpers_report_type_mismatches`, `standard_fmt_missing_import_suggests_import` |
| `std::json` | `json::Value`, `json::Number`, `json::ErrorKind`, `json::Error`, `json::PathSegment`, `parse`, `encode`, `number_as_int`, `int`, value `as_*` helpers, scalar/composite object-field access/default/required helpers, scalar array projection helpers, direct scalar-array object-field helpers, `at`, `at_required`, typed scalar path projection helpers, typed collection path projection helpers, compiler-owned `decode_or[T]` / strict `decode[T]`, and compiler-owned `to_value[T]` / `encode_typed[T]` over structural `Option[T]`, recursive `List[T]`, typed `Map[String, T]`, `std::json::Value`, and concrete enum/record targets | `samples/packages/app/std_json/main.muga` covers explicit JSON value and path helper usage; `samples/projects/config_app` now consumes structural JSON through `std::config` without importing `std::json` | `standard_json_artifact_run_uses_emitted_std_implementations`, `standard_json_accessor_artifact_run_uses_emitted_std_implementations`, `standard_json_decode_or_artifact_run_uses_schema_payload`, `standard_json_decode_artifact_run_uses_schema_payload`, `standard_json_decode_structural_artifact_run_uses_schema_payload`, `standard_json_decode_enum_artifact_run_uses_schema_payload`, `standard_json_encode_typed_record_runs`, `standard_json_to_value_record_runs`, `standard_json_encode_typed_reports_validation_errors`, `standard_json_encode_typed_artifact_run_uses_schema_payload`, `standard_json_encode_typed_interface_artifact_run_uses_schema_payload`, `standard_json_encode_typed_rejects_unsupported_targets`, `standard_json_decode_required_record_runs`, `standard_json_decode_structural_targets_run`, `standard_json_decode_enum_targets_run`, `standard_json_decode_or_structural_targets_overlay_run`, `standard_json_decode_or_enum_targets_overlay_run`, `standard_json_decode_structural_targets_report_path_errors`, `standard_json_decode_enum_targets_report_path_errors`, `standard_json_decode_reports_missing_required_field_path`, `standard_json_decode_requires_expected_target`, `standard_json_decode_rejects_unsupported_targets`, `standard_json_decode_rejects_deferred_structural_targets`, `standard_json_value_accessors_run_as_virtual_package`, `standard_json_required_object_fields_report_missing_errors`, `standard_json_composite_object_fields_run_as_virtual_package`, `standard_json_path_helpers_run_as_virtual_package`, `standard_json_path_scalar_helpers_run_as_virtual_package`, `standard_json_path_collection_helpers_run_as_virtual_package`, `standard_json_scalar_array_projections_run_as_virtual_package`, `standard_json_scalar_array_field_helpers_run_as_virtual_package`, `standard_json_value_accessors_report_shape_errors`, `manifest_config_project_sample_runs_against_emitted_artifacts`, `manifest_config_project_sample_reports_config_shape_errors` |
| `std::bytes` | opaque `bytes::Bytes`, `bytes::size`, `bytes::empty`, `bytes::at` | `samples/packages/app/std_hash/main.muga` reads local bytes and inspects byte `0`; `samples/projects/resource_export/src/main/main.muga` reads declared resource bytes | `standard_fs_read_bytes_reads_file_and_indexes_bytes_for_source_and_built_runs`, `standard_fs_read_resource_bytes_reads_manifest_entry_resources_for_source_and_built_runs`, `standard_fs_read_resource_bytes_reads_archive_dependency_resources_from_cache`, `package_std_hash_sample_runs_against_emitted_artifacts`, `manifest_resource_export_project_sample_runs_against_emitted_artifacts` |
| `std::hash` | `hash::sha256_hex(bytes): String` | `samples/packages/app/std_hash/main.muga` computes a lowercase SHA-256 hex digest; `samples/projects/resource_export/src/main/main.muga` hashes declared resource bytes before writing them out | `standard_hash_sha256_hex_hashes_read_bytes_for_source_and_built_runs`, `package_std_hash_sample_runs`, `package_std_hash_sample_runs_against_emitted_artifacts`, `manifest_resource_export_project_sample_runs_against_emitted_artifacts` |
| `std::config` | `config::ErrorKind`, `config::Error`, compiler-owned `config::load_json_or[T](path, fallback)` and `config::load_json[T](path)` over the same structural and concrete enum JSON decoder target set | `samples/packages/app/std_config/main.muga` demonstrates strict required config and default-overlay config loading side by side; `samples/projects/config_app` loads typed JSON config through `std::config`, keeps CLI > config > defaults precedence in app code, discovers the default path through `MUGA_CONFIG_PATH` when `--config` is absent, and maps public config errors at the app boundary | `package_std_config_sample_runs`, `package_std_config_sample_runs_against_emitted_artifacts`, `standard_config_load_json_or_record_runs`, `standard_config_load_json_record_runs`, `standard_config_load_json_or_structural_targets_runs`, `standard_config_load_json_or_enum_targets_runs`, `standard_config_load_json_or_reports_decode_path_errors`, `standard_config_load_json_reports_required_decode_errors`, `standard_config_load_json_or_reports_parse_errors`, `standard_config_load_json_or_artifact_run_uses_schema_payload`, `standard_config_load_json_artifact_run_uses_schema_payload`, `manifest_config_project_sample_uses_env_config_path_default`, `manifest_config_project_sample_runs_against_emitted_artifacts`, `manifest_config_project_sample_reports_config_shape_errors`, `manifest_config_project_sample_json_built_run_applies_cli_overrides` |

## Review Notes

- The `std::io` sample constructs the public error records directly because
  `std::io` currently has data contracts, not standalone runtime functions.
- `samples/packages/app/std_path_with_file_name/main.muga` demonstrates pure
  sibling output path derivation without filesystem reads, path validation, or
  normalization.
- `samples/packages/app/std_path_normalize/main.muga` demonstrates pure lexical
  path cleanup without filesystem reads, symlink resolution, or sandbox
  containment policy.
- `samples/packages/app/std_path_strip_prefix/main.muga` demonstrates pure
  relative path derivation without filesystem reads, normalization, or sandbox
  containment policy.
- `samples/packages/app/std_path_with_extension/main.muga` demonstrates pure
  output/sidecar path derivation without filesystem reads, normalization, or
  symlink resolution.
- `muga new --template report-app` now applies `path::with_extension`,
  `fs::read_text`, and `fs::write_text` in a generated first-project report
  workflow with source and `run --built` coverage. `muga new --template
  package-app` now covers generated local dependency teaching, while
  `samples/projects/report_app` remains the richer report workflow sample.
- `samples/packages/app/std_fs_file_size/main.muga` demonstrates scalar file
  size metadata over an existing small binary payload without adding a public
  metadata record.
- `samples/packages/app/std_fs_modified_time/main.muga` demonstrates a narrow
  last-modified timestamp read through `time::UnixMillis` without adding
  all-path metadata records, accessed/created timestamps, permissions, or
  symlink policy.
- `samples/packages/app/std_fs_file_metadata/main.muga` demonstrates the
  regular-file `FileMetadata` record that groups file byte size and modified
  time while still leaving path-kind policy to the path metadata records.
- `samples/packages/app/std_fs_metadata/main.muga` demonstrates the
  `PathStatus`, `PathKind`, and `PathInfo` grouping layer over existing
  path-status predicates while still leaving host-error-backed all-path
  metadata records deferred.
- `samples/packages/app/std_fs_path_metadata/main.muga` demonstrates
  host-error-backed existing-path `PathMetadata` with kind/status and modified
  time while still leaving optional size and broader metadata policy to later
  records.
- `samples/packages/app/std_fs_path_size_metadata/main.muga` demonstrates
  `PathSizeMetadata` with optional regular-file byte size while still leaving
  recursive directory sizing to a dedicated aggregate API.
- `samples/packages/app/std_fs_read_dir_recursive/main.muga` demonstrates
  deterministic read-only descendant traversal while still leaving aggregation
  and destructive behavior to separate APIs, with directory copy, globbing, and
  symlink policy deferred.
- `samples/packages/app/std_fs_directory_size_metadata/main.muga` demonstrates
  deterministic recursive byte/count aggregation while still leaving destructive
  behavior to separate APIs and keeping globbing, public symlink
  classification, and sandbox policy deferred.
- `samples/packages/app/std_fs_remove_dir_all/main.muga` demonstrates
  recursive generated-tree cleanup while still leaving trash/recycle-bin
  integration, globbing, and sandbox policy deferred.
- `samples/packages/app/std_fs_copy_dir_all/main.muga` demonstrates
  no-overwrite recursive directory copy while still leaving merge/overwrite,
  metadata preservation, rollback, host-rename acceleration, globbing, and
  sandbox policy deferred.
- `samples/packages/app/std_fs_move_dir_all/main.muga` demonstrates
  no-overwrite copy-then-remove directory move while still leaving
  merge/overwrite, rollback, host-rename acceleration, and sandbox policy
  deferred.
- `samples/packages/app/std_fmt/main.muga` demonstrates pure formatting helpers
  while still leaving language interpolation, format specifiers, localization,
  and builders deferred.
- `samples/packages/app/std_fs_canonicalize/main.muga` demonstrates
  existing-path canonicalization as a recoverable filesystem effect without
  adding project-root lookup, config discovery, or pure normalization.
- `samples/packages/app/std_fs_rename/main.muga` keeps the rename helper sample
  deterministic by exercising the recoverable missing-source error path.
- `samples/packages/app/std_env_current_dir/main.muga` demonstrates ambient
  current-directory reads as `Result[path::Path, io::IOError]` without adding
  runtime-owned config discovery, project-root lookup, canonicalization, or
  symlink policy.
- `samples/packages/app/std_env_temp_dir/main.muga` demonstrates ambient
  temporary-directory reads as `Result[path::Path, io::IOError]` without adding
  unique temp-file allocation, cleanup, or sandbox policy.
- `samples/projects/config_app` is the JSON config workflow sample: it loads a
  typed JSON file with `std::config::load_json_or[T]`, keeps the path explicit
  through `std::path`, decodes optional owner fields, nested server records,
  `List[Server]`, and typed `Map[String, Int]` limits directly as ordinary
  records, maps public config errors with `std::result::map_err`, reads program
  args with `std::env`, applies typed `std::cli::parse_or[T]` settings
  overlays, assembles app text with `std::string`, and keeps explicit CLI >
  config > defaults precedence. The nested JSON config workflow refresh is
  implemented with `tags`, owner metadata, servers, and limits, using
  composite/typed helpers before adding the later `std::config` expansion,
  TOML, JSON paths, or broader schema decoding.
  The `std::json` scalar array projection slice is implemented with
  `array_strings`, `array_ints`, and `array_bools` before broader object-field
  matrices or schema decoding. The direct scalar-array object-field helper
  slice is implemented; the later structural refresh supersedes the old
  `config_app` manual `json::object_string_array_or` path. The repeated
  `std::cli` option value slice is also implemented and remains covered in
  `std_cli`; `config_app` now uses
  the full `cli::parse_or[T]` schema overlay for repeated `--tags` values
  instead of hand-written list override code. The JSON path helper slice is
  implemented with `json::PathSegment`, `json::at`, and
  `json::at_required`; current explicit path-helper usage lives in the
  `std_json` sample and focused tests after the structural `config_app`
  refresh. The typed JSON path scalar projection slice is implemented with
  `json::at_string*`, `json::at_int*`, and
  `json::at_bool*`; current `config_app` no longer needs manual nested metadata
  path lookup after structural config decoding. The typed JSON path collection
  projection helper slice is implemented with
  `json::at_array*`, `json::at_object*`, `json::at_string_array*`,
  `json::at_int_array*`, and `json::at_bool_array*` before adding schema
  decoding, `std::config`, TOML, or generated config app templates. The schema
  decoding design in
  [json-schema-decoding.md](json-schema-decoding.md) selects and implements compiler-owned
  `json::decode_or[T](value, fallback)` as the first implementation before
  required `json::decode[T]`, `std::config`, TOML, or generated config app
  templates. The minimal `std::config` JSON default loading design lands before
  TOML, generated config app templates, or full CLI parser schemas. The
  selected design and implementation in
  [std-config-json-loading.md](std-config-json-loading.md) fixes the public
  `config::Error` shape, artifact-safe schema lowering, `LoadJsonConfig`
  persistence, runtime read/parse/decode mapping, and `config_app` coverage.
  The generated `muga new --template config-app` starter is implemented before
  TOML, required decoding, full CLI parser schemas, formatting templates, or
  broader decoder targets.
  [json-required-decoding.md](json-required-decoding.md) selects and implements
  required `json::decode[T](value)` before TOML, broader decoder target types,
  full CLI parser schemas, formatting templates, or broader platform APIs.
  [json-required-decoding.md](json-required-decoding.md) defines and implements
  the strict decoder contract with expected `Result[T, json::Error]` targets,
  missing-field errors, ignored unknown fields, no-fallback schema lowering, and
  artifact-safe `DecodeJsonRequired` payloads.
  [json-decoder-target-expansion.md](json-decoder-target-expansion.md) implements
  the decoder target expansion for `Option[T]`, recursive `List[T]`, typed
  `Map[String, T]`, and concrete non-generic enums across
  `json::decode_or[T]`, `json::decode[T]`, and `config::load_json_or[T]`.
  The implemented `config_app` sample and generated `config-app` starter carry
  the structural config workflow with `Option[String]`, nested records,
  `List[Record]`, and typed `Map[String, Int]` settings before TOML, full CLI
  parser schemas, formatting templates, config discovery, or broader host
  effects.
  The decoder expansion implements enum JSON/config decoder support, using
  zero-payload string tags and one-payload single-key objects before generic
  enum decoding, field/variant schema polish, TOML, full CLI parser schemas,
  formatting templates, config discovery, or broader host effects.
  [json-config-schema-polish.md](json-config-schema-polish.md),
  [json-config-strict-unknown-fields.md](json-config-strict-unknown-fields.md),
  [json-config-alias-metadata.md](json-config-alias-metadata.md), and
  [json-config-validation-attributes.md](json-config-validation-attributes.md)
  now implement canonical wire names, strict record metadata, aliases, and
  validation attributes across source, package interfaces, decoder schemas,
  artifacts, and `run --built`.
  [json-config-schema-export.md](json-config-schema-export.md) implements
  `muga schema --format json`, and
  [json-typed-encoding.md](json-typed-encoding.md) implements
  `json::to_value[T]` / `json::encode_typed[T]` so the same concrete contract
  can decode, validate, export, and emit JSON.
  [cli-parser-schema.md](cli-parser-schema.md) selects and implements
  `cli::parse_or[T]` / `cli::usage_for[T]` as the post-typed-JSON app boundary.
  The generated `config-app` template and sample use `cli::parse_or[T]` for
  CLI > config > defaults settings overlays and expose `cli::usage_for[T]`
  through `--help` before TOML, full client generation, generic
  encoding/decoding, broader validators, config discovery automation, or host
  effects.
  [cli-field-metadata.md](cli-field-metadata.md) records first `@cli(...)`
  field metadata implementation and generated config-app metadata adoption.
  [cli-field-metadata.md](cli-field-metadata.md) implements field-level
  `@cli(name: "...", alias: "...", help: "...", hidden)` plus a dedicated
  `CliSchema`, before adding TOML/config discovery.
  [strict-cli-parser-schema.md](strict-cli-parser-schema.md) implements
  strict `cli::parse[T](args)` with expected-result type inference,
  `MissingArgument` errors, absent `Bool`/`Option`/`List` synthesis, strict
  unsupported-field rejection, source/artifact/`run --built` coverage, and no
  new no-default usage helper.
  The checked-in strict CLI tool sample at
  `samples/projects/cli_tool/src/main/main.muga` adopts the strict parser with
  a root command, subcommands, generated help, compact short options, and
  completion coverage before TOML, config discovery, full client generation,
  generic encoding/decoding, broader validators, or host effects.
  Generated `muga new --template cli-tool` adoption is implemented from that
  sample shape, including source/build/`run --built`, generated README,
  completion helper, and packaging helper coverage.
  [strict-cli-no-default-usage.md](strict-cli-no-default-usage.md) implements
  `cli::usage_for_required[T](program)` with explicit call type arguments,
  source/artifact coverage, strict sample/template adoption, and the
  replacement for the historical strict CLI manual help duplication.
  [cli-command-metadata.md](cli-command-metadata.md) documents
  `@cli(about: "...")` generated usage summaries before short options,
  subcommands, TOML, config discovery automation, full client generation, or
  host-effect APIs.
  [cli-short-option-metadata.md](cli-short-option-metadata.md) implements exact
  short-option syntax, parser behavior, generated usage rendering, app-owned
  `cli::has_short_flag(args, "h")`, and interface/artifact-compatible schema
  payloads.
  [post-cli-short-option-metadata-adoption-gap-selection.md](post-cli-short-option-metadata-adoption-gap-selection.md)
  selects typed CLI positional field metadata design next, so schema-driven
  CLIs can model primary operands before combined short flags, attached values,
  built-in help branching, subcommands, TOML, config discovery automation,
  shell completion generation, full client generation, or host effects.
  [cli-positional-field-metadata.md](cli-positional-field-metadata.md)
  implements `@cli(positional: N)` with explicit 1-based indexes, generated
  positional usage, source/interface/artifact persistence, and strict
  `cli-tool` template adoption.
  [post-cli-positional-field-metadata-adoption-gap-selection.md](post-cli-positional-field-metadata-adoption-gap-selection.md)
  selects the built-in CLI help policy in
  [cli-built-in-help-policy.md](cli-built-in-help-policy.md), which led to
  `cli::help_requested` and generated help helpers after typed positional
  operands landed.
  [cli-built-in-help-policy.md](cli-built-in-help-policy.md) implements
  `cli::help_requested`, `cli::help_for`, and `cli::help_for_required`, so
  generated config and strict CLI tools remove
  repeated `--help` / `-h` checks and manual help rows before parse-integrated
  help result enums, combined short flags, attached values, subcommands, shell
  completion generation, TOML/config discovery automation, full client
  generation, or host effects.
  [post-built-in-cli-help-helper-adoption-gap-selection.md](post-built-in-cli-help-helper-adoption-gap-selection.md)
  selected parse-integrated CLI help workflow design, so generated starters can
  match a typed help-or-parsed request while runtime-owned printing/exits remain
  deferred.
  [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md)
  implements `cli::Request[T]`, `cli::parse_request[T]`, and
  `cli::parse_request_or[T]` across strict/config starters before
  runtime-owned printing/exits, subcommands, shell completions, TOML/config
  discovery automation, full client generation, or host effects.
  [post-parse-integrated-cli-help-workflow-adoption-gap-selection.md](post-parse-integrated-cli-help-workflow-adoption-gap-selection.md)
  audits that adoption and selects compact CLI short option syntax design next
  before subcommands, shell completions, config discovery, and runtime-owned
  printing/exits.
  [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md)
  implements `-abc`, `-ofile`, and `-abo=value` as runtime parser behavior over
  existing short metadata.
  [post-compact-cli-short-option-syntax-adoption-gap-selection.md](post-compact-cli-short-option-syntax-adoption-gap-selection.md)
  audits compact short syntax adoption and selected CLI subcommand metadata
  design; [cli-subcommand-metadata.md](cli-subcommand-metadata.md) implements
  enum/variant metadata plus strict command enum schemas through source
  validation, `.mgi` package interfaces, `.mgb` schema payloads, recursive
  runtime dispatch/help, artifact-backed execution, and `run --built` before
  wrapper-record root/global options, generated app shell completions,
  TOML/config discovery, or runtime-owned printing/exits.
  [post-cli-subcommand-schema-adoption-gap-selection.md](post-cli-subcommand-schema-adoption-gap-selection.md)
  audits that implementation and refreshes `samples/projects/cli_tool` plus
  generated `muga new --template cli-tool` starters with `run` / `inspect`
  subcommands while preserving compact short options, validation, generated
  root/leaf help, artifact-backed execution, and recoverable `cli::Error`
  mapping.
  [cli-wrapper-root-options.md](cli-wrapper-root-options.md) implements strict
  wrapper records with one `@cli(subcommand)` field for root/global options,
  including schema/artifact lowering, runtime parse/help, source,
  artifact-backed, and `run --built` coverage, plus strict sample/generated
  `cli-tool` adoption with `--profile` / `-p`.
  [cli-schema-shell-completions.md](cli-schema-shell-completions.md) implements
  `muga cli-completions <bash|zsh|fish> --program <name> --type <Type> ...` as
  the `CliSchema`-backed generated app completion surface across source,
  `--artifact-root`, and `--built` workflows.
  [post-cli-schema-shell-completion-adoption-gap-selection.md](post-cli-schema-shell-completion-adoption-gap-selection.md)
  implements the first completion onboarding step through install docs and a
  generated `cli-tool` README, plus a `scripts/generate-completions.sh`
  packaging hook.
  [cli-completion-json-spec.md](cli-completion-json-spec.md),
  [cli-completion-value-sources.md](cli-completion-value-sources.md), and
  [cli-completion-installer-integration.md](cli-completion-installer-integration.md)
  cover the shell-agnostic JSON contract, static path value sources, and
  non-mutating completion package emission for generated CLIs.
  Historical JSON/config adoption selections remain part of this stdlib audit:
  [json-config-schema-polish.md](json-config-schema-polish.md) covers
  `@json(rename: "...")` on record fields and enum variants before TOML;
  [json-config-strict-unknown-fields.md](json-config-strict-unknown-fields.md)
  covers `@json(deny_unknown_fields)`, `RF` artifact tokens, aliases, and TOML
  deferrals;
  [json-config-alias-metadata.md](json-config-alias-metadata.md) covers
  `@json(alias: "...")`, validation follow-ups, and TOML deferrals.
- `samples/projects/report_app` is the integrated practical workflow for
  args/env, stdout/stderr, text-file handle writes, JSON run output, `Result`,
  local dependencies, artifact-backed execution, and `run --built`.
- The `std::fs` samples intentionally include recoverable failure paths as
  ordinary `Result` values. Destructive helper samples target missing paths or
  known sample-local paths and do not rely on broad cleanup behavior.
- The `std::env` samples avoid host-specific assertions where useful.
  `env::get_var` uses a deliberately missing variable, `env::args` documents
  the default empty argument list, and `env::temp_dir` only requires that the
  host can provide a representable temporary-directory path.
- The `std::cli` sample is host-independent because it passes an explicit
  argument list. It covers positional defaults, long flags, `--name=value`,
  repeated option values, typed `Int` / `Bool` parsing, and the `--` terminator
  without adding global parser state.
- The `std::time` sample checks only that the returned `UnixMillis` value is
  positive. It does not make a timing precision or performance claim.
- The `std::string` sample keeps non-string conversion explicit with
  `to_string()` and demonstrates empty-list joins without adding formatting
  templates, interpolation, builders, buffers, or localization policy.
- The `std::json` sample stays inside the documented package contract in
  [std-json-first-slice.md](std-json-first-slice.md) and the implemented
  accessor follow-up.
  It demonstrates parse/encode, scalar and composite object-field accessors,
  required-field helpers, direct scalar-array object-field helpers, and `Result`
  error flow without schema generation, HTTP/RPC, `Float`, `Decimal`, `Bytes`,
  streaming APIs, or resource handles.
  The implementation evidence is audited in
  [std-json-implementation-audit.md](std-json-implementation-audit.md).
- Broader filesystem operations, stdout/stderr handles, broader binary `Bytes`,
  recursive directory sizing, accessed/created timestamps, permissions,
  symlink classification, recursive removal, directory copy, resource handles, schema/client
  generation, richer CLI parsing/help, richer formatting templates,
  interpolation, builders, richer time APIs, and process execution remain deferred by
  [standard-library-review-rules.md](standard-library-review-rules.md).

## Maintenance Rule

When any of these public stdlib packages changes, update this review together
with README sample links, [muga-by-example.md](muga-by-example.md), and the
focused source plus artifact-backed tests in `tests/examples.rs`. The release
gate / GitHub Actions alignment for `scripts/v1-release-gate.sh` is now tracked
in [release-gate-alignment.md](release-gate-alignment.md); keep the next
standard-library pass from broadening beyond documented package contracts
without a separate design note.
