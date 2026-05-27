use std::{collections::BTreeSet, fs, path::Path};

#[test]
fn v1_release_checklist_has_evidence_for_each_completion_item() {
    let checklist = read("docs/v1-release-checklist.md");
    assert!(
        !checklist.contains("- [ ]"),
        "v1 release checklist still has unchecked completion items"
    );
    let completed_items = checklist.matches("- [x]").count();
    let evidence_items = checklist.matches("Evidence:").count();
    assert_eq!(
        completed_items, evidence_items,
        "each checked v1 release completion item must carry evidence"
    );
    for required in [
        "Evidence:",
        "scripts/v1-release-gate.sh",
        "tests/release_readiness.rs",
        ".github/workflows/ci.yml",
        ".github/workflows/release.yml",
    ] {
        assert!(
            checklist.contains(required),
            "v1 release checklist missing evidence marker `{required}`"
        );
    }
}

#[test]
fn samples_do_not_contain_post_v1_planned_snippets() {
    let sample_files = files_with_extension(Path::new("samples"), "muga");
    assert!(
        !sample_files.is_empty(),
        "sample tree should contain muga files"
    );

    for path in &sample_files {
        let display = path.display().to_string();
        assert!(
            !display.contains("planned_"),
            "post-v1 planned snippets do not belong under samples: {display}"
        );
        assert!(
            !display.contains("concurrency"),
            "post-v1 concurrency snippets do not belong under samples: {display}"
        );
    }

    for snippet in [
        "docs/design-snippets/concurrency/group.muga",
        "docs/design-snippets/concurrency/channels.muga",
    ] {
        assert!(
            Path::new(snippet).is_file(),
            "post-v1 design snippet should live under docs: {snippet}"
        );
    }
}

#[test]
fn docs_do_not_reference_removed_planned_samples() {
    for path in documentation_files() {
        let text = read(&path);
        for forbidden in [
            "samples/planned_concurrency",
            "planned_concurrency_group.muga",
            "planned_concurrency_channels.muga",
            "runs a package graph by flattening imported packages",
        ] {
            assert!(
                !text.contains(forbidden),
                "{} still references stale wording `{forbidden}`",
                path.display()
            );
        }
    }
}

#[test]
fn documentation_guide_keeps_readme_human_sized() {
    let readme = read("README.md");
    let guide = read("docs/README.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    assert!(
        readme.lines().count() <= 450,
        "README should stay short enough to remain a human-facing entry point"
    );
    assert!(
        readme.contains("docs/README.md"),
        "README must point readers to the documentation guide"
    );
    assert!(
        !readme.contains("post-json-stdlib-boundary-selection.md"),
        "README should not inline historical decision-log indexes"
    );
    assert!(
        implementation_resume.lines().count() <= 1_900,
        "implementation resume should not grow by retaining completed one-off checklists"
    );
    assert!(
        !implementation_resume
            .contains("Test Plan For The Completed Local Archive Dependency Slice"),
        "completed one-off archive checklists should not stay in the active resume handoff"
    );

    for required in [
        "human-oriented entry point",
        "Reading Order",
        "Current Direction",
        "Audit Result",
        "Decision Logs",
        "Keep README short and user-facing",
        "shell-agnostic JSON completion spec",
        "reuses `CliSchema`",
        "Core Capability Acceleration",
        "std::process",
        "References are not retention proof",
        "Rust tests, Muga samples, and executable CLI contracts",
    ] {
        assert!(
            guide.contains(required),
            "documentation guide missing `{required}`"
        );
    }
}

#[test]
fn conformance_suite_is_wired_into_release_readiness() {
    let readme = read("conformance/README.md");
    let harness = read("tests/conformance.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let checklist = read("docs/v1-release-checklist.md");

    for required in [
        "mini-language-spec-v1.md",
        "spec/001-core-language.md",
        "spec/002-name-resolution.md",
        "spec/003-typing.md",
        "spec/004-functions.md",
        "spec/005-records.md",
        "spec/006-packages.md",
    ] {
        assert!(
            readme.contains(required),
            "conformance README must tie fixtures to `{required}`"
        );
    }

    for root in [
        "conformance/v1/valid",
        "conformance/v1/rejecting",
        "conformance/v1/package-artifacts",
    ] {
        let files = files_with_extension(Path::new(root), "muga");
        assert!(
            !files.is_empty(),
            "conformance root has no fixtures: {root}"
        );
    }

    for required in [
        "valid_conformance_programs_run",
        "rejecting_conformance_programs_report_expected_codes",
        "package_artifact_conformance_runs_without_dependency_source_fallback",
        "expect-main",
        "expect-error",
    ] {
        assert!(
            harness.contains(required),
            "conformance harness missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("v1 release checklist", checklist.as_str()),
    ] {
        assert!(
            text.contains("conformance"),
            "{label} must mention the conformance suite"
        );
    }
}

#[test]
fn diagnostic_code_prefixes_used_by_source_are_documented() {
    let errors = read("errors.md");
    let mut prefixes = BTreeSet::new();
    for path in files_with_extension(Path::new("src"), "rs") {
        let source = read(&path);
        collect_diagnostic_prefixes(&source, &mut prefixes);
    }

    for prefix in prefixes {
        assert!(
            errors.contains(&format!("| `{prefix}` |")),
            "errors.md must document diagnostic prefix `{prefix}`"
        );
    }
}

#[test]
fn diagnostic_json_and_command_output_contract_are_documented() {
    let readme = read_primary_docs();
    let errors = read("errors.md");
    let contract = read("docs/diagnostics-and-output.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");
    let diagnostic = read("src/diagnostic.rs");

    assert!(
        readme.contains("docs/diagnostics-and-output.md"),
        "README must link to the diagnostics and command-output contract"
    );
    assert!(
        errors.contains("docs/diagnostics-and-output.md"),
        "errors.md must point to the machine-readable diagnostic contract"
    );

    for required in [
        "muga check --format json",
        "muga --help",
        "muga help <command>",
        "muga run --format json",
        "muga test --format json",
        "muga build --format json",
        "muga why-rebuild --format json",
        "muga api-diff --format json",
        "muga emit-cli-completions --format json",
        "muga emit-app-completions --format json",
        "muga emit-app-bundle --format json",
        "muga install-app --format json",
        "muga uninstall-app --format json",
        "muga emit-package-archive --format json",
        "muga emit-app-archive --format json",
        "muga verify-app-archive --format json",
        "muga verify-package-archive --format json",
        "muga unpack-app-archive --format json",
        "muga unpack-package-archive --format json",
        "muga list-installed-apps --format json",
        "--expected-hash",
        "muga emit-artifacts --format json",
        "muga emit-interface --format json",
        "muga emit-check-cache --format json",
        "muga fmt --check",
        "would format<TAB><path>",
        "\"schemaVersion\":1",
        "\"command\":\"check\"",
        "\"command\":\"run\"",
        "\"command\":\"test\"",
        "\"command\":\"build\"",
        "\"command\":\"why-rebuild\"",
        "\"command\":\"api-diff\"",
        "\"command\":\"emit-cli-completions\"",
        "\"command\":\"emit-app-completions\"",
        "\"command\":\"emit-app-bundle\"",
        "\"command\":\"install-app\"",
        "\"command\":\"uninstall-app\"",
        "\"command\":\"emit-package-archive\"",
        "\"command\":\"emit-app-archive\"",
        "\"command\":\"verify-app-archive\"",
        "\"command\":\"verify-package-archive\"",
        "\"command\":\"unpack-app-archive\"",
        "\"command\":\"unpack-package-archive\"",
        "\"command\":\"list-installed-apps\"",
        "\"command\":\"emit-artifacts\"",
        "\"command\":\"emit-interface\"",
        "\"command\":\"emit-check-cache\"",
        "\"entry\"",
        "\"archive\"",
        "\"root\"",
        "\"launcher\"",
        "\"sourceMode\"",
        "\"replaceOwned\"",
        "\"outputDir\"",
        "\"files\"",
        "\"archiveRoot\"",
        "\"entryPackage\"",
        "\"dependencySnippet\"",
        "\"program\"",
        "\"path\"",
        "\"uri\"",
        "file://",
        "\"status\":\"ok\"",
        "\"status\":\"error\"",
        "\"status\":\"passed\"",
        "\"status\":\"failed\"",
        "\"diagnostics\"",
        "\"tests\"",
        "\"summary\"",
        "\"hash\"",
        "\"files\"",
        "\"passed\"",
        "\"failed\"",
        "\"stdout\"",
        "\"stderr\"",
        "\"mainResult\"",
        "\"artifactRoot\"",
        "\"lockfile\"",
        "\"archiveCache\"",
        "\"metadataHash\"",
        "\"artifacts\"",
        "\"packages\"",
        "\"state\"",
        "\"reason\"",
        "\"selection\"",
        "\"sourceKind\"",
        "\"missing\"",
        "\"fresh\"",
        "\"stale\"",
        "\"hashMismatch\"",
        "\"invalid\"",
        "\"code\"",
        "\"severity\"",
        "\"span\"",
        "\"related\"",
        "\"suggestions\"",
        "\"context\"",
        "\"kind\": \"source\"",
        "\"role\": \"entry\"",
        "\"kind\": \"package\"",
        "\"kind\": \"artifactRoot\"",
        "\"kind\": \"artifactFile\"",
        "\"kind\": \"artifactHash\"",
        "\"kind\": \"regenerationCommand\"",
        "\"artifactKind\"",
        "\"hashKind\"",
        "\"packagePath\"",
        "\"artifactKind\":\"interface\"",
        "\"hashKind\":\"dependencyInterface\"",
        "\"hashKind\":\"lockfile\"",
        "\"hashKind\":\"archiveCache\"",
        "\"sourceKind\":\"path\"",
        "\"sourceKind\":\"archive\"",
        "\"kind\":\"archiveCache\"",
        "\"sourceUri\"",
        "\"kind\":\"artifactHash\",\"role\":\"expected\",\"hashKind\":\"source\"",
        "dependency interface set changed",
        "\"command\":\"muga emit-check-cache --artifact-root <dir> <entry>\"",
        "\"status\":\"written\"",
        "\"reused\"",
        "\"check-input\"",
        "\"default-build\"",
        "\"dependency-interface\"",
        "\"check-cache\"",
        "\"implementation\"",
        "call-context notes",
        "R021",
        "user assertion call",
        "diagnostics[].related",
        "tests[].diagnostics",
        "tests[].stderr",
        "diagnostics[].context",
        "eprint",
        "eprintln",
    ] {
        assert!(
            contract.contains(required),
            "diagnostics/output contract missing `{required}`"
        );
    }

    for required in [
        "diagnostic_json_includes_stable_structured_fields",
        "diagnostic_json_includes_artifact_file_context_when_available",
        "cli_check_json_reports_success_contract_on_stdout",
        "cli_check_json_reports_diagnostic_contract_on_stdout",
        "cli_check_json_reports_package_and_artifact_context_for_artifact_backed_diagnostics",
        "cli_check_json_reports_hash_and_regeneration_context_for_stale_check_cache",
        "cli_run_reports_dependency_interface_set_changed_implementation_artifact_context",
        "cli_run_json_reports_stdout_and_main_result_on_stdout",
        "cli_run_text_writes_program_stderr_to_stderr",
        "cli_run_json_reports_program_stderr_on_stdout",
        "cli_run_json_reports_null_main_result_without_main",
        "cli_run_json_reports_diagnostic_contract_on_stdout",
        "cli_run_json_reports_runtime_call_context_on_stdout",
        "cli_run_json_passes_program_args_after_separator",
        "cli_test_json_reports_success_contract_on_stdout",
        "cli_test_json_reports_per_test_stderr_on_stdout",
        "cli_test_json_reports_failure_contract_on_stdout",
        "cli_test_json_reports_assertion_failure_source_context_on_stdout",
        "cli_test_json_reports_diagnostic_contract_on_stdout",
        "cli_test_json_reports_runtime_call_context_on_stdout",
        "runtime_errors_include_call_context_related_notes",
        "cli_build_json_reports_artifact_status_contract",
        "cli_build_json_reports_diagnostic_contract_on_stdout",
        "cli_why_rebuild_json_reports_fresh_artifact_states",
        "cli_why_rebuild_json_reports_missing_explicit_artifacts",
        "cli_why_rebuild_json_reports_stale_source_artifacts",
        "cli_why_rebuild_json_reports_stale_dependency_interface_artifacts",
        "cli_why_rebuild_json_reports_dependency_interface_set_changed_implementation_artifact",
        "cli_why_rebuild_json_reports_stale_local_path_lockfile_metadata",
        "cli_why_rebuild_json_reports_fresh_local_archive_lockfile_metadata",
        "cli_why_rebuild_json_reports_invalid_and_hash_mismatched_artifacts",
        "cli_emit_artifacts_json_reports_artifact_contract",
        "cli_emit_interface_json_reports_filtered_artifact_contract",
        "cli_emit_check_cache_json_reports_artifact_contract",
        "cli_emit_check_cache_json_reports_diagnostic_contract_on_stdout",
        "cli_emit_package_archive_writes_archive_and_hash",
        "cli_emit_and_unpack_app_archive_round_trips_bundle_launcher",
        "verify-app-archive",
        "verify-package-archive",
    ] {
        assert!(
            examples.contains(required),
            "examples test suite must cover `{required}`"
        );
    }

    for required in [
        "OutputFormat::Json",
        "check_json_output",
        "run_json_output",
        "print_run_outcome",
        "stderr_text",
        "test_json_output",
        "push_test_case_json",
        "test_status_label",
        "build_json_output",
        "push_build_artifact_json",
        "build_artifact_kind",
        "why_rebuild_json_output",
        "completion_package_json_output",
        "completion_package_diagnostic_json_output",
        "app_bundle_emit_json_output",
        "app_bundle_emit_diagnostic_json_output",
        "app_bundle_source_mode_label",
        "app_install_json_output",
        "app_install_diagnostic_json_output",
        "app_uninstall_json_output",
        "app_uninstall_diagnostic_json_output",
        "package_archive_emit_json_output",
        "package_archive_emit_diagnostic_json_output",
        "app_archive_emit_json_output",
        "app_archive_emit_diagnostic_json_output",
        "app_archive_verify_json_output",
        "app_archive_diagnostic_json_output",
        "package_archive_verify_json_output",
        "package_archive_diagnostic_json_output",
        "app_archive_unpack_json_output",
        "app_archive_unpack_diagnostic_json_output",
        "package_archive_unpack_json_output",
        "package_archive_unpack_diagnostic_json_output",
        "expected_archive_hash",
        "push_artifact_cache_lockfile_json",
        "push_artifact_cache_archive_cache_json",
        "push_artifact_cache_explanation_json",
        "artifact_emission_json_output",
        "push_emitted_artifact_json",
        "artifact_emission_extra_diagnostic_context",
        "entry_source_diagnostic_context",
        "check_extra_diagnostic_context",
        "artifact_root_diagnostic_context",
        "--format json",
    ] {
        assert!(
            cli.contains(required),
            "CLI missing JSON support `{required}`"
        );
    }
    for required in [
        "pub enum DiagnosticContext",
        "Package {",
        "ArtifactRoot {",
        "ArtifactFile {",
        "ArtifactHash {",
        "RegenerationCommand {",
        "artifact_file_context",
        "artifact_hash_context",
        "regeneration_command_context",
        "diagnostics_json_array_with_context",
    ] {
        assert!(
            diagnostic.contains(required),
            "diagnostic JSON support missing `{required}`"
        );
    }
}

#[test]
fn runtime_debug_reporting_boundary_is_documented_and_covered() {
    let contract = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");

    for required in [
        "Schema version 1 does not expose a separate `stackTrace` field",
        "diagnostics[].related",
        "call-context notes",
        "R021",
        "user assertion call",
        "regenerationCommand",
    ] {
        assert!(
            contract.contains(required),
            "diagnostics/output contract missing runtime-debug boundary `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("runtime/debug") || text.contains("Runtime/debug"),
            "{label} must mention the runtime/debug reporting boundary"
        );
        assert!(
            text.contains("call-context") && text.contains("R021"),
            "{label} must keep call context and assertion diagnostics tied together"
        );
        assert!(
            text.contains("regenerationCommand"),
            "{label} must keep artifact regeneration next actions documented"
        );
        assert!(
            text.contains("lightweight benchmark") || text.contains("benchmark health"),
            "{label} must preserve the completed benchmark health-check handoff"
        );
        assert!(
            text.contains("fuzzing") && text.contains("malformed-input"),
            "{label} must move the next trust slice to fuzzing plans"
        );
    }

    for required in ["RuntimeCallFrame", "add_test_assertion_diagnostic", "R021"] {
        assert!(
            runtime.contains(required),
            "runtime implementation missing `{required}`"
        );
    }

    for required in [
        "runtime_errors_include_call_context_related_notes",
        "cli_run_json_reports_runtime_call_context_on_stdout",
        "cli_test_json_reports_runtime_call_context_on_stdout",
        "cli_test_json_reports_assertion_failure_source_context_on_stdout",
        "cli_check_json_reports_hash_and_regeneration_context_for_stale_check_cache",
        "cli_run_reports_missing_package_check_artifact",
        "cli_run_built_reports_missing_default_check_cache_artifact",
        "cli_why_rebuild_json_reports_stale_source_artifacts",
    ] {
        assert!(
            examples.contains(required),
            "examples test suite must cover runtime/debug reporting boundary `{required}`"
        );
    }
}

#[test]
fn benchmark_health_checks_are_documented_and_covered() {
    let readme = read_primary_docs();
    let checklist = read("docs/v1-release-checklist.md");
    let docs = read("docs/benchmark-health-checks.md");
    let script = read("scripts/benchmark-health-check.sh");
    let health_tests = read("tests/benchmark_health.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");

    for (label, text) in [
        ("README", readme.as_str()),
        ("v1 checklist", checklist.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("benchmark-health-checks.md"),
            "{label} must link benchmark health-check docs"
        );
        assert!(
            text.contains("public performance claims")
                || text.contains("public performance claim")
                || (text.contains("public") && text.contains("performance")),
            "{label} must keep benchmark health checks release-neutral"
        );
    }

    for required in [
        "scripts/benchmark-health-check.sh",
        "cargo test --locked --test benchmark_health -- --ignored --nocapture",
        "release-neutral",
        "not public performance claims",
        "compiler stages",
        "package artifact reuse",
        "String/List/Map",
        "benchmark-health",
    ] {
        assert!(
            docs.contains(required),
            "benchmark health-check docs missing `{required}`"
        );
    }

    for required in [
        "benchmark health checks are release-neutral local measurements",
        "cargo test --locked --test benchmark_health -- --ignored --nocapture",
    ] {
        assert!(
            script.contains(required),
            "benchmark health-check script missing `{required}`"
        );
    }

    for required in [
        "compiler_stage_health_check_reports_elapsed_times",
        "package_artifact_reuse_health_check_reports_elapsed_times",
        "representative_runtime_health_check_reports_elapsed_times",
        "#[ignore = \"manual benchmark health check; run scripts/benchmark-health-check.sh\"]",
        "benchmark-health",
        "compile_typed_path",
        "compile_mir_path",
        "compile_bytecode_path",
        "build_package_artifacts",
        "run_path_against_default_build_artifacts",
        "samples/string_helpers.muga",
        "samples/packages/app/std_list/main.muga",
        "samples/packages/app/std_map/main.muga",
    ] {
        assert!(
            health_tests.contains(required),
            "benchmark health integration test missing `{required}`"
        );
    }
}

#[test]
fn performance_and_concurrency_goals_are_ordered_in_strategy_docs() {
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let practical = read("docs/practical-language-readiness.md");

    for required in [
        "fast feedback during",
        "optimized/native builds",
        "compete with Rust or C++",
        "compiler-speed ambition",
        "beat Go-style fast compilation",
        "syntax/check/build feedback",
        "compiler feedback speed",
        "against Go and other fast compilers",
    ] {
        assert!(
            strategy.contains(required),
            "strategy plan must preserve the performance goal `{required}`"
        );
    }

    for required in [
        "fast edit-check-run loop",
        "beat Go-style compiler feedback",
        "watch/daemon",
        "Go and other fast compilers",
        "Rust/C++-class performance",
        "performance and concurrency spine",
        "fast syntax/check/build feedback",
        "full incremental project artifact reuse",
        "structured concurrency",
        "cancellation-aware IO",
    ] {
        assert!(
            roadmap.contains(required),
            "roadmap must order performance/concurrency work around `{required}`"
        );
    }

    for required in [
        "final performance ambition",
        "compiler-speed ambition",
        "compete with Rust or C++",
        "day-to-day compile feedback than Go",
        "fast edit-check-run",
        "cold, warm, and",
        "full incremental package/project artifact reuse",
        "control-flow-oriented MIR",
        "native backend",
        "representative benchmark suites",
        "structured concurrency",
        "typed channels",
    ] {
        assert!(
            practical.contains(required),
            "practical readiness must preserve performance/concurrency step `{required}`"
        );
    }
}

#[test]
fn generated_lockfile_cleanup_script_is_dedicated_and_safe() {
    let gitignore = read(".gitignore");
    let scripts_readme = read("scripts/README.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let script = read("scripts/trash-generated-muga-locks.sh");

    for required in [
        "set -euo pipefail",
        "samples/projects/cli_tool/muga.lock",
        "samples/projects/resource_export/muga.lock",
        "samples/projects/*/muga.lock",
        "refusing unexpected generated lock path",
        "refusing to trash non-file path",
        "refusing to trash non-muga.lock path",
        "command -v trash",
        "trash \"$lock_path\"",
        "--dry-run",
        "--quiet",
    ] {
        assert!(
            script.contains(required),
            "generated lockfile cleanup script missing `{required}`"
        );
    }
    assert!(
        !script.contains("rm ") && !script.contains("rm\t"),
        "generated lockfile cleanup must use trash instead of rm"
    );
    assert!(
        gitignore.contains("samples/projects/*/muga.lock"),
        "generated sample muga.lock files must stay ignored"
    );
    for (label, text) in [
        ("scripts README", scripts_readme.as_str()),
        ("implementation resume", implementation_resume.as_str()),
    ] {
        assert!(
            text.contains("scripts/trash-generated-muga-locks.sh")
                && text.contains("samples/projects/cli_tool/muga.lock")
                && text.contains("samples/projects/resource_export/muga.lock"),
            "{label} must make generated muga.lock cleanup discoverable"
        );
        assert!(
            text.contains("--dry-run"),
            "{label} must mention dry-run cleanup discovery"
        );
        assert!(
            text.contains("samples/projects/*/muga.lock") || text.contains("generated locks"),
            "{label} must mention ignored generated sample locks"
        );
    }
}

#[test]
fn fuzzing_malformed_input_plan_is_documented_and_covered() {
    let readme = read_primary_docs();
    let checklist = read("docs/v1-release-checklist.md");
    let plan = read("docs/fuzzing-malformed-input-plan.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");
    let examples = read("tests/examples.rs");
    let conformance = read("tests/conformance.rs");
    let lib = read("src/lib.rs");
    let package = read("src/package.rs");
    let interface = read("src/interface.rs");
    let cache = read("src/cache.rs");
    let implementation_artifact = read("src/implementation_artifact.rs");

    for (label, text) in [
        ("README", readme.as_str()),
        ("v1 checklist", checklist.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("fuzzing-malformed-input-plan.md"),
            "{label} must link the fuzzing and malformed-input plan"
        );
        assert!(
            text.contains("install") || text.contains("onboarding") || text.contains("quickstart"),
            "{label} must move the next trust slice to installation/onboarding docs"
        );
    }

    for required in [
        "Parser And Single-File Syntax",
        "Package Archive `.mgp`",
        "App Bundle Archive `.mga`",
        "Local `muga.lock`",
        "Package Interface `.mgi`",
        "Check-Cache `.mgc`",
        "Implementation Artifact `.mgb`",
        "Panic is a bug",
        "diagnostic, not crash",
        "`~/tmp/`",
        "deterministic regression test",
        "no source-body fallback",
        "validate_package_archive_bytes",
        "verify_app_bundle_archive",
        "verify_app_bundle_archive_with_expected_hash",
        "read_verified_app_bundle_archive",
        "read_verified_app_bundle_archive_with_expected_hash",
        "expected_app_bundle_archive_hash_from_path",
        "validate_app_bundle_expected_archive_hash",
        "validate_lockfile_text",
        "PackageInterfaceGraph::read_persisted_file",
        "read_package_check_artifact",
        "implementation_artifact::read_persisted_file",
        "Future `cargo fuzz`",
    ] {
        assert!(
            plan.contains(required),
            "fuzzing/malformed-input plan missing `{required}`"
        );
    }

    for required in [
        "validate_package_archive_bytes",
        "materialize_package_archive_bytes",
        "validate_lockfile_text",
    ] {
        assert!(
            package.contains(required),
            "package reader surface missing `{required}`"
        );
    }
    for required in [
        "verify_app_bundle_archive",
        "verify_app_bundle_archive_with_expected_hash",
        "read_verified_app_bundle_archive",
        "read_verified_app_bundle_archive_with_expected_hash",
        "parse_app_bundle_archive_bytes",
        "expected_app_bundle_archive_hash_from_path",
        "validate_app_bundle_expected_archive_hash",
    ] {
        assert!(
            lib.contains(required),
            "app bundle archive reader surface missing `{required}`"
        );
    }
    assert!(
        interface.contains("pub fn read_persisted_file"),
        "interface reader surface missing persisted-file reader"
    );
    assert!(
        cache.contains("pub fn read_package_check_artifact")
            && cache.contains("pub fn validate_package_check_artifact"),
        "check-cache reader surface must expose read and validate helpers"
    );
    assert!(
        implementation_artifact.contains("pub fn read_persisted_file")
            && implementation_artifact.contains("pub fn read_persisted_artifacts"),
        "implementation artifact reader surface must expose persisted readers"
    );

    for required in [
        "invalid_examples_fail_frontend",
        "package_archive_readback_rejects_hash_mismatch",
        "package_archive_readback_rejects_malformed_entries",
        "package_archive_materialization_rejects_non_empty_destination_without_writes",
        "package_archive_materialization_rejects_unsafe_manifest_source_roots",
        "package_archive_materialization_rejects_unsafe_manifest_resource_roots",
        "app_archive_readback_rejects_malformed_entries_without_writes",
        "app_archive_unpack_rejects_non_empty_output_without_writes",
        "build_rejects_malformed_local_path_dependency_lockfile",
        "build_rejects_malformed_local_archive_dependency_lockfile",
        "package_check_reports_hash_mismatched_interface_artifact",
        "typed_hir_rejects_stale_package_interface_signatures",
        "package_cache_rejects_stale_checked_artifact",
        "package_cache_rejects_stale_dependency_interface_artifact",
        "cli_check_json_reports_hash_and_regeneration_context_for_stale_check_cache",
        "implementation_artifact_rejects_invalid_bytecode_symbol_ref",
        "implementation_artifact_rejects_invalid_bytecode_local_count",
        "implementation_artifact_rejects_invalid_bytecode_jump_target",
        "cli_why_rebuild_json_reports_invalid_and_hash_mismatched_artifacts",
    ] {
        assert!(
            examples.contains(required),
            "examples suite must keep malformed-input regression anchor `{required}`"
        );
    }
    assert!(
        conformance.contains("rejecting_conformance_programs_report_expected_codes"),
        "conformance suite must keep rejecting-program regression coverage"
    );
}

#[test]
fn installation_onboarding_paths_are_documented_and_covered() {
    let readme = read_primary_docs();
    let checklist = read("docs/v1-release-checklist.md");
    let docs = read("docs/installation-and-onboarding.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");
    let cargo = read("Cargo.toml");
    let cli = read("src/main.rs");
    let examples = read("tests/examples.rs");

    for (label, text) in [
        ("README", readme.as_str()),
        ("v1 checklist", checklist.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("installation-and-onboarding.md"),
            "{label} must link installation/onboarding docs"
        );
        assert!(
            text.contains("release trigger") || text.contains("release-neutral"),
            "{label} must keep onboarding separate from release timing"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("Muga by Example") || text.contains("example-driven"),
            "{label} must move the next trust slice to example-driven learning material"
        );
    }

    for required in [
        "Rust 1.95",
        "cargo install muga",
        "cargo install --path . --locked",
        "cargo run --locked -- --version",
        "muga --version",
        "muga --help",
        "muga help <command>",
        "muga version",
        "muga new --template app ~/tmp/muga-hello",
        "muga run ~/tmp/muga-hello/src/main/main.muga",
        "muga run ~/tmp/muga-hello/src/main/main.muga -- Ada",
        "muga check ~/tmp/muga-hello/src/main/main.muga",
        "muga fmt --check ~/tmp/muga-hello/src/main/main.muga",
        "muga build ~/tmp/muga-hello/src/main/main.muga",
        "muga check --built ~/tmp/muga-hello/src/main/main.muga",
        "muga run --built ~/tmp/muga-hello/src/main/main.muga -- --name=Ada",
        "muga why-rebuild --built ~/tmp/muga-hello/src/main/main.muga",
        "MUGA_PROGRAM=hello sh ~/tmp/muga-hello/scripts/package-app.sh",
        "scripts/package-app.sh",
        "MUGA_BUNDLE_DIR",
        "MUGA_ARCHIVE_DIR",
        "verify-app-archive [--format text|json] [--expected-hash sha256:<hex>] <archive>",
        "`--expected-hash sha256:<hex>`",
        "hello Muga",
        "`--name=Ada`",
        "muga test --format json ~/tmp/muga-test/src/main/main.muga",
        "muga doc ~/tmp/muga-test/src/main/main.muga",
        "`~/tmp/`",
        "Release-Neutral Boundaries",
        "release triggers",
        "binary release",
    ] {
        assert!(
            docs.contains(required),
            "installation/onboarding docs missing `{required}`"
        );
    }

    for required in [
        "cargo install --path . --locked",
        "muga --version",
        "muga --help",
        "muga new --template app ~/tmp/muga-hello",
        "muga run ~/tmp/muga-hello/src/main/main.muga -- Ada",
        "muga run --built ~/tmp/muga-hello/src/main/main.muga -- --name=Ada",
        "MUGA_PROGRAM=hello sh ~/tmp/muga-hello/scripts/package-app.sh",
        "docs/installation-and-onboarding.md",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    assert!(
        cargo.contains("rust-version = \"1.95\""),
        "Cargo.toml must keep the documented Rust version check aligned"
    );
    for required in [
        "Mode::Version",
        "Mode::Help",
        "command_usage",
        "usage_invocation_topic",
        "env!(\"CARGO_PKG_VERSION\")",
        "muga --version",
        "muga help",
        "--help",
    ] {
        assert!(
            cli.contains(required),
            "CLI version command missing `{required}`"
        );
    }
    assert!(
        examples.contains("cli_version_reports_package_version"),
        "examples suite must cover CLI version output"
    );
    assert!(
        examples.contains("cli_help_reports_usage"),
        "examples suite must cover CLI top-level help output"
    );
    assert!(
        examples.contains("unknown help topic"),
        "examples suite must cover unknown CLI help topics"
    );
}

#[test]
fn muga_by_example_learning_path_is_documented_and_covered() {
    let readme = read_primary_docs();
    let checklist = read("docs/v1-release-checklist.md");
    let docs = read("docs/muga-by-example.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");
    let project_template = read("src/project_template.rs");

    for (label, text) in [
        ("README", readme.as_str()),
        ("v1 checklist", checklist.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("muga-by-example.md"),
            "{label} must link Muga by Example docs"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("registry-security-design.md"),
            "{label} must link the registry security design"
        );
        assert!(
            text.contains("edition-feature-fingerprint-policy.md"),
            "{label} must link the edition/feature fingerprint policy"
        );
        assert_hands_off_to_tooling_completions_doctor(label, text);
    }

    for required in [
        "bindings",
        "records",
        "`Result`",
        "packages",
        "tests",
        "local dependencies",
        "artifact-backed builds",
        "samples/println_sum.muga",
        "samples/record_user.muga",
        "samples/record_with_update.muga",
        "samples/result_try.muga",
        "samples/string_parse_int.muga",
        "samples/packages/app/main/main.muga",
        "samples/projects/local_path_app/src/main/main.muga",
        "samples/projects/report_app/src/main/main.muga",
        "samples/projects/resource_export/src/main/main.muga",
        "muga new --template package-app ~/tmp/muga-example-package",
        "sh scripts/package-package-app.sh",
        "samples/packages/app/std_fs_metadata/main.muga",
        "samples/packages/app/std_fs_path_metadata/main.muga",
        "samples/packages/app/std_fs_path_size_metadata/main.muga",
        "samples/packages/app/std_fs_read_dir_recursive/main.muga",
        "PathInfo",
        "PathKind",
        "path_info",
        "PathMetadata",
        "path_metadata_path",
        "PathSizeMetadata",
        "path_size_metadata_path",
        "samples/packages/app/std_fs_write_bytes/main.muga",
        "samples/packages/app/artifact_facade/main.muga",
        "resource-bytes-export-sample.md",
        "muga run samples/projects/resource_export/src/main/main.muga",
        "muga run --built samples/projects/resource_export/src/main/main.muga",
        "muga new --template resource-export ~/tmp/muga-example-resource",
        "binary-file-write.md",
        "write_bytes_path",
        "muga new --template test ~/tmp/muga-example-test",
        "muga test --format json ~/tmp/muga-example-test/src/main/main.muga",
        "muga doc ~/tmp/muga-example-test/src/main/main.muga",
        "muga metadata --format json samples/packages/app/main/main.muga",
        "muga workspace --format json samples/packages/app/main/main.muga",
        "muga build samples/packages/app/artifact_facade/main.muga",
        "muga check --built samples/packages/app/artifact_facade/main.muga",
        "muga run --built samples/packages/app/artifact_facade/main.muga",
        "muga why-rebuild --built samples/packages/app/artifact_facade/main.muga",
        "muga emit-artifacts --artifact-root ~/tmp/muga-example-artifacts",
        "muga emit-package-archive --archive-root ~/tmp/muga-example-archives --dependency-snippet",
        "muga verify-package-archive <archive>",
        "muga unpack-package-archive",
        "--expected-hash sha256:<hex>",
        "`~/tmp/`",
        "release-neutral",
        "release trigger",
    ] {
        assert!(
            docs.contains(required),
            "Muga by Example missing `{required}`"
        );
    }

    for sample in [
        "samples/println_sum.muga",
        "samples/record_user.muga",
        "samples/record_with_update.muga",
        "samples/result_try.muga",
        "samples/string_parse_int.muga",
        "samples/packages/app/main/main.muga",
        "samples/projects/local_path_app/src/main/main.muga",
        "samples/projects/report_app/src/main/main.muga",
        "samples/projects/resource_export/src/main/main.muga",
        "samples/packages/app/std_fs_metadata/main.muga",
        "samples/packages/app/std_fs_path_metadata/main.muga",
        "samples/packages/app/std_fs_path_size_metadata/main.muga",
        "samples/packages/app/std_fs_read_dir_recursive/main.muga",
        "samples/packages/app/std_fs_write_bytes/main.muga",
        "samples/packages/app/artifact_facade/main.muga",
    ] {
        assert!(
            Path::new(sample).is_file(),
            "Muga by Example references missing runnable sample: {sample}"
        );
    }

    for required in ["ProjectTemplate::Test", "@test", "test::assert_eq_int"] {
        assert!(
            project_template.contains(required),
            "generated test template missing `{required}`"
        );
    }
}

#[test]
fn registry_security_design_is_documented_and_covered() {
    let readme = read_primary_docs();
    let checklist = read("docs/v1-release-checklist.md");
    let design = read("docs/registry-security-design.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");
    let spec = read("spec/006-packages.md");
    let package = read("src/package.rs");
    let lib = read("src/lib.rs");
    let examples = read("tests/examples.rs");
    let artifact_cache = read("docs/artifact-cache-explanations.md");
    let fuzzing = read("docs/fuzzing-malformed-input-plan.md");

    for (label, text) in [
        ("README", readme.as_str()),
        ("v1 checklist", checklist.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("registry-security-design.md"),
            "{label} must link registry security design docs"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("edition-feature-fingerprint-policy.md"),
            "{label} must hand off to edition/feature fingerprint policy docs"
        );
        assert_hands_off_to_tooling_completions_doctor(label, text);
    }

    for required in [
        "package_content_hash",
        "write_package_archive",
        "muga emit-package-archive",
        "validate_package_archive_bytes",
        "read_package_archive",
        "materialize_package_archive_bytes",
        "materialize_package_archive",
        ".muga/packages/<package>-sha256-<hash>",
        "muga.lock",
        "muga why-rebuild --format json",
        "Content identity is authoritative",
        "Registries are naming and discovery services",
        "not a trust root",
        "signing",
        "provenance",
        "lockfile enforcement",
        "cache validation",
        "malicious-package",
        "trust-on-first-use",
        "dependency confusion",
        "typosquatting",
        "package takeover",
        "yanked packages",
        "muga audit",
        "SBOM",
        "URL, Git, or registry dependency declarations",
        "network fetching",
        "release trigger",
        "sha256:<hex>",
        "registry metadata compromise",
        "mirror or CDN",
        "Git tags moved",
        "cache poisoning",
        "malformed lockfiles",
        "path escapes",
        "source/resource",
        "publisher signatures",
        "registry signatures",
        "provenance attestations",
    ] {
        assert!(
            design.contains(required),
            "registry security design missing `{required}`"
        );
    }

    for required in [
        "package_content_hash",
        "write_package_archive",
        "validate_package_archive_bytes",
        "read_package_archive",
        "materialize_package_archive_bytes",
        "materialize_package_archive",
        "unpack_package_archive",
        "validate_lockfile_text",
        "package_archive_dependency_cache_root",
    ] {
        assert!(
            package.contains(required),
            "package trust-boundary source missing `{required}`"
        );
    }
    for required in [
        "write_package_archive",
        "read_package_archive",
        "validate_package_archive_bytes",
        "materialize_package_archive",
        "materialize_package_archive_bytes",
        "unpack_package_archive",
        "unpack_package_archive_with_expected_hash",
        "explain_package_artifact_cache",
    ] {
        assert!(
            lib.contains(required),
            "public API surface missing `{required}`"
        );
    }

    for required in [
        "package_archive_readback_validates_hash_and_entries",
        "package_archive_readback_rejects_hash_mismatch",
        "manifest_local_archive_dependency_rejects_hash_mismatch",
        "manifest_local_archive_dependency_rejects_stale_cache_hash",
        "manifest_local_archive_dependency_materializes_declared_resources_and_validates_cache_hash",
        "manifest_local_archive_dependency_rejects_cache_file_collision",
        "build_rejects_malformed_local_archive_dependency_lockfile",
        "cli_emit_package_archive_dependency_snippet_drives_local_archive_workflow",
        "unpack-package-archive",
        "cli_why_rebuild_json_reports_fresh_local_archive_lockfile_metadata",
    ] {
        assert!(
            examples.contains(required),
            "examples suite must keep registry security foundation anchor `{required}`"
        );
    }

    for required in [
        "artifactRoot",
        "lockfile",
        "archiveCache",
        "metadataHash",
        "\"sourceKind\": \"archive\"",
        "archiveCache[].metadataHash[].hashKind",
        "muga why-rebuild [--format text|json]",
    ] {
        assert!(
            artifact_cache.contains(required),
            "artifact/cache explanation docs missing `{required}`"
        );
    }

    for required in [
        "Package Archive `.mgp`",
        "Local `muga.lock`",
        "validate_package_archive_bytes",
        "materialize_package_archive_bytes",
        "validate_lockfile_text",
        "manifest_local_archive_dependency_rejects_stale_cache_hash",
        "package_archive_materialization_rejects_unsafe_manifest_resource_roots",
        "manifest_local_archive_dependency_rejects_cache_file_collision",
    ] {
        assert!(
            fuzzing.contains(required),
            "fuzzing plan must keep registry foundation malformed-input anchor `{required}`"
        );
    }

    for required in [
        "future registry security design",
        "`.mgp` hash as the package identity",
        "naming/discovery services rather than trust roots",
        "This slice deliberately does not yet include",
    ] {
        assert!(
            spec.contains(required),
            "package spec must keep registry security boundary `{required}`"
        );
    }
}

#[test]
fn edition_feature_fingerprint_policy_is_documented_and_covered() {
    let readme = read_primary_docs();
    let checklist = read("docs/v1-release-checklist.md");
    let policy = read("docs/edition-feature-fingerprint-policy.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");
    let spec = read("spec/006-packages.md");
    let mgi_api_diff = read("docs/mgi-api-diff.md");
    let interface = read("src/interface.rs");
    let cache = read("src/cache.rs");
    let implementation_artifact = read("src/implementation_artifact.rs");
    let package = read("src/package.rs");
    let examples = read("tests/examples.rs");

    for (label, text) in [
        ("README", readme.as_str()),
        ("v1 checklist", checklist.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
        ("package spec", spec.as_str()),
        ("mgi api diff", mgi_api_diff.as_str()),
    ] {
        assert!(
            text.contains("edition-feature-fingerprint-policy.md"),
            "{label} must link the edition/feature fingerprint policy"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert_hands_off_to_tooling_completions_doctor(label, text);
    }

    for required in [
        "muga-package-interface-v11",
        "muga-package-check-v1",
        "muga-package-implementation-bytecode-v1",
        "lockfile_version = 1",
        "muga_version",
        "source_fingerprint_input_from_entry",
        "PackageCheckCacheKey",
        "PackageImplementationArtifact",
        "public_api_hash",
        "recheck_fingerprint",
        "language edition",
        "semantic feature set",
        "compiler semantic version",
        "Prelude/std semantic version",
        "Artifact schema version",
        "muga.lock",
        ".mgp",
        ".mgi",
        ".mgc",
        ".mgb",
        "muga why-rebuild --format json",
        "regenerationCommand",
        "muga fix",
        "unsupported edition",
        "unsupported semantic feature",
        "fail closed",
        "release trigger",
        "named arguments",
        "schema versioning",
        "semantic_features",
    ] {
        assert!(
            policy.contains(required),
            "edition/feature fingerprint policy missing `{required}`"
        );
    }

    for required in [
        "PERSISTED_INTERFACE_HEADER",
        "muga-package-interface-v11",
        "stable_hash",
    ] {
        assert!(
            interface.contains(required),
            "interface artifact source missing `{required}`"
        );
    }
    for required in [
        "PERSISTED_CHECK_HEADER",
        "muga-package-check-v1",
        "PackageCheckCacheKey",
        "source_fingerprint_input_from_entry",
    ] {
        assert!(
            cache.contains(required),
            "check-cache source missing `{required}`"
        );
    }
    for required in [
        "PERSISTED_IMPLEMENTATION_HEADER",
        "muga-package-implementation-bytecode-v1",
        "PackageImplementationArtifact",
        "interface_hash",
        "source_hash",
        "dependency_interfaces",
    ] {
        assert!(
            implementation_artifact.contains(required),
            "implementation artifact source missing `{required}`"
        );
    }
    for required in [
        "source_fingerprint_input_from_entry",
        "manifest_lockfile_text",
        "lockfile_version = 1",
        "muga_version",
        "validate_lockfile_text",
    ] {
        assert!(
            package.contains(required),
            "package source missing `{required}`"
        );
    }

    for required in [
        "edition or semantic feature-set fingerprint differs",
        "unknown",
        "fail closed",
    ] {
        assert!(
            mgi_api_diff.contains(required),
            "API diff design missing edition/fingerprint compatibility rule `{required}`"
        );
    }
    for required in [
        "cli_check_json_reports_hash_and_regeneration_context_for_stale_check_cache",
        "package_cache_rejects_stale_dependency_interface_artifact",
        "cli_run_reports_dependency_interface_mismatched_implementation_artifact",
        "cli_run_reports_dependency_interface_set_changed_implementation_artifact_context",
        "cli_why_rebuild_json_reports_invalid_and_hash_mismatched_artifacts",
    ] {
        assert!(
            examples.contains(required),
            "examples suite must keep artifact fingerprint diagnostic anchor `{required}`"
        );
    }
}

#[test]
fn representative_artifact_dependency_api_coverage_is_documented_and_covered() {
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");
    let examples = read("tests/examples.rs");

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("representative")
                && text.contains("artifact-backed")
                && text.contains("source-body fallback"),
            "{label} must document representative artifact-backed dependency API coverage"
        );
    }

    for required in [
        "artifact_run_covers_representative_dependency_api_without_source",
        "std__path.mgb",
        "util__parse.mgb",
        "model__score.mgb",
        "api__reports.mgb",
    ] {
        assert!(
            examples.contains(required),
            "examples suite must keep representative artifact dependency API coverage anchor `{required}`"
        );
    }
}

#[test]
fn public_interface_hash_stability_audit_is_documented_and_covered() {
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");
    let examples = read("tests/examples.rs");
    let interface = read("src/interface.rs");

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("public interface hash stability")
                && text.contains("source-span movement"),
            "{label} must document the public interface hash stability audit"
        );
        assert_hands_off_to_tooling_completions_doctor(label, text);
    }

    for required in [
        "package_interface_hash_stays_stable_for_representative_public_shapes",
        "stable_hash_for_package",
        "persisted_artifact_text",
        "Report[Envelope[score::Score]]",
        "score::Stage[score::Score]",
        "io::IOError",
        "path::Path",
    ] {
        assert!(
            examples.contains(required),
            "examples suite must keep public interface hash stability anchor `{required}`"
        );
    }

    for required in [
        "stable_hash_for_package",
        "persisted_artifact_text",
        "PersistedInterfaceBodyShape::Hash",
        "format_interface_span",
    ] {
        assert!(
            interface.contains(required),
            "interface source must keep hash/span separation anchor `{required}`"
        );
    }
}

#[test]
fn implementation_artifact_structural_audit_is_documented_and_covered() {
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");
    let examples = read("tests/examples.rs");
    let implementation_artifact = read("src/implementation_artifact.rs");
    let bytecode = read("src/bytecode.rs");

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains(".mgb")
                && text.contains("structural validation")
                && text.contains("bytecode merge"),
            "{label} must document the implementation artifact structural audit"
        );
        assert_hands_off_to_tooling_completions_doctor(label, text);
    }

    for required in [
        "artifact_run_merges_independent_control_flow_dependency_implementations",
        "bytecode_program_has_control_flow",
        "write_control_flow_implementation_provider",
        "compile_bytecode_path_for_run_against_artifact_root",
        "pkg fn adjust",
        "try parse_limit",
        "while current < 6",
    ] {
        assert!(
            examples.contains(required),
            "examples suite must keep implementation artifact structural audit anchor `{required}`"
        );
    }

    for required in [
        "read_persisted_artifacts_reserving_program_items",
        "next_private_package_item_id",
        "next_package_item_id_in_program",
        "validate_artifact_structure",
        "validate_chunk_structure",
    ] {
        assert!(
            implementation_artifact.contains(required),
            "implementation artifact source must keep structural audit anchor `{required}`"
        );
    }

    for required in [
        "pub fn merge",
        "canonicalize_package_function_refs",
        "JumpIfFalse",
    ] {
        assert!(
            bytecode.contains(required),
            "bytecode source must keep merge audit anchor `{required}`"
        );
    }
}

#[test]
fn build_reuse_lockfile_update_audit_is_documented_and_covered() {
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");
    let examples = read("tests/examples.rs");
    let package = read("src/package.rs");
    let cli = read("src/main.rs");

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("muga build")
                && text.contains("reuse output")
                && text.contains("lockfile update behavior")
                && text.contains("local path")
                && text.contains("local archive")
                && text.contains("implementation-only edits")
                && text.contains("public signature edits")
                && text.contains("malformed lockfiles"),
            "{label} must document the build reuse and lockfile update audit"
        );
        assert_hands_off_to_tooling_completions_doctor(label, text);
    }

    for required in [
        "cli_build_reports_reuse_and_refreshes_local_path_lockfile_after_dependency_edits",
        "cli_build_refreshes_local_archive_lockfile_after_archive_dependency_update",
        "build_rejects_malformed_local_path_dependency_lockfile",
        "build_rejects_malformed_local_archive_dependency_lockfile",
        "build_output_has_status_for_file",
        "lockfile_source_hash",
        "lockfile_archive_hash",
    ] {
        assert!(
            examples.contains(required),
            "examples suite must keep build reuse/lockfile audit anchor `{required}`"
        );
    }

    for required in [
        "manifest_lockfile_text",
        "validate_lockfile_text",
        "local_dependency_source_hash",
        "ProjectDependencySource::Archive",
    ] {
        assert!(
            package.contains(required),
            "package source must keep lockfile update/validation anchor `{required}`"
        );
    }

    for required in ["build_json_output", "build_artifact_status"] {
        assert!(
            cli.contains(required),
            "CLI source must keep build output status anchor `{required}`"
        );
    }
}

#[test]
fn recursive_annotation_diagnostic_actionability_is_documented_and_covered() {
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");
    let errors = read("errors.md");
    let examples = read("tests/examples.rs");
    let typing = read("src/typing.rs");

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        let lower_text = text.to_lowercase();
        assert!(
            lower_text.contains("recursive annotation diagnostics")
                && lower_text.contains("parameter")
                && lower_text.contains("return")
                && lower_text.contains("explicit")
                && lower_text.contains("signatures"),
            "{label} must document recursive annotation diagnostic actionability"
        );
        assert_hands_off_to_tooling_completions_doctor(label, text);
    }

    for required in [
        "directly recursive functions",
        "mutually recursive functions",
        "parameter type annotation",
        "explicit return type",
        "explicit return types",
    ] {
        assert!(
            errors.contains(required),
            "errors.md must keep recursive annotation guidance `{required}`"
        );
    }

    for required in [
        "direct_recursion_requires_actionable_annotation_suggestion",
        "mutual_recursion_requires_actionable_signature_suggestion",
    ] {
        assert!(
            examples.contains(required),
            "examples suite must keep recursive annotation actionability test `{required}`"
        );
    }

    for required in [
        "add a parameter type annotation or an explicit return type to the recursive function",
        "add parameter type annotations and an explicit return type to each function in the mutually recursive group",
    ] {
        assert!(
            typing.contains(required),
            "typing source must keep recursive annotation suggestion `{required}`"
        );
    }
}

#[test]
fn public_signature_round_trip_audit_is_documented_and_covered() {
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");
    let examples = read("tests/examples.rs");
    let interface = read("src/interface.rs");

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        let lower_text = text.to_lowercase();
        assert!(
            lower_text.contains("package-mode public signatures")
                && lower_text.contains("v1-supported public type shape")
                && lower_text.contains("in-memory and persisted interfaces"),
            "{label} must document the public signature round-trip audit"
        );
        assert_hands_off_to_tooling_completions_doctor(label, text);
    }

    for required in [
        "package_public_signatures_round_trip_representative_type_shapes",
        "assert_representative_public_signature_shapes",
        "PackageRecord",
        "PackageEnum",
        "GenericParam",
        "Function",
        "std::io",
        "std::path",
        "check_package_aware_path_against_loaded_interfaces",
    ] {
        assert!(
            examples.contains(required),
            "examples suite must keep public signature round-trip audit anchor `{required}`"
        );
    }

    for required in [
        "canonical_public_signature_type",
        "public_type_items_by_package",
        "PackageItemKind::Record",
        "PackageItemKind::Enum",
    ] {
        assert!(
            interface.contains(required),
            "interface source must keep public signature canonicalization anchor `{required}`"
        );
    }
}

#[test]
fn stdlib_package_docs_and_samples_review_is_documented_and_covered() {
    let readme = read_primary_docs();
    let review = read("docs/stdlib-package-samples-review.md");
    let guide = read("docs/muga-by-example.md");
    let rules = read("docs/standard-library-review-rules.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let roadmap = read("ROADMAP.md");
    let examples = read("tests/examples.rs");
    let std_package = read("src/std_package.rs");

    assert!(
        readme.contains("docs/stdlib-package-samples-review.md"),
        "README must link to the stdlib package docs and samples review"
    );
    assert!(
        guide.contains("stdlib-package-samples-review.md"),
        "Muga by Example must link to the stdlib package samples review"
    );
    assert!(
        rules.contains("stdlib-package-samples-review.md"),
        "standard-library review rules must link to the completed sample review"
    );

    for required in [
        "stdlib package docs and samples review",
        "std::io",
        "std::fs",
        "std::path",
        "std::env",
        "std::cli",
        "std::time",
        "std::string",
        "std::fmt",
        "std::json",
        "std::bytes",
        "std::hash",
        "artifact-backed execution samples",
        "samples/packages/app/std_io/main.muga",
        "samples/packages/app/std_env/main.muga",
        "samples/packages/app/std_env_args/main.muga",
        "samples/packages/app/std_env_current_dir/main.muga",
        "samples/packages/app/std_env_temp_dir/main.muga",
        "samples/packages/app/std_cli/main.muga",
        "samples/packages/app/std_cli_schema/main.muga",
        "samples/packages/app/std_time/main.muga",
        "samples/packages/app/std_string/main.muga",
        "samples/packages/app/std_fmt/main.muga",
        "samples/packages/app/std_json/main.muga",
        "samples/packages/app/std_hash/main.muga",
        "samples/packages/app/std_path_with_file_name/main.muga",
        "samples/packages/app/std_path_normalize/main.muga",
        "samples/packages/app/std_path_with_extension/main.muga",
        "samples/packages/app/std_fs_rename/main.muga",
        "samples/packages/app/std_fs_move_dir_all/main.muga",
        "samples/packages/app/std_fs_file_size/main.muga",
        "samples/packages/app/std_fs_modified_time/main.muga",
        "samples/packages/app/std_fs_file_metadata/main.muga",
        "samples/packages/app/std_fs_path_metadata/main.muga",
        "samples/packages/app/std_fs_path_size_metadata/main.muga",
        "samples/packages/app/std_fs_read_dir_recursive/main.muga",
        "samples/packages/app/std_fs_canonicalize/main.muga",
        "samples/projects/report_app",
        "samples/projects/resource_export",
        "standard_fs_artifact_run_exposes_error_fields_without_direct_io_import",
        "standard_fs_move_dir_all_artifact_run_uses_emitted_std_implementations",
        "standard_fs_rename_artifact_run_uses_emitted_std_implementations",
        "standard_fs_file_size_artifact_run_uses_emitted_std_implementations",
        "standard_fs_modified_unix_millis_artifact_run_uses_emitted_std_implementations",
        "standard_fs_file_metadata_artifact_run_uses_emitted_std_implementations",
        "standard_fs_path_status_returns_public_record",
        "standard_fs_path_info_returns_kind_and_status",
        "standard_fs_path_metadata_artifact_run_uses_emitted_std_implementations",
        "standard_fs_path_size_metadata_artifact_run_uses_emitted_std_implementations",
        "standard_fs_read_dir_recursive_artifact_run_uses_emitted_std_implementations",
        "standard_fs_canonicalize_artifact_run_uses_emitted_std_implementations",
        "standard_env_args_artifact_run_uses_program_arguments",
        "standard_env_current_dir_artifact_run_uses_emitted_std_implementations",
        "standard_env_temp_dir_artifact_run_uses_emitted_std_implementations",
        "standard_cli_artifact_run_uses_emitted_std_implementations",
        "package_std_cli_schema_sample_runs",
        "package_std_cli_schema_sample_runs_against_emitted_artifacts",
        "standard_time_artifact_run_uses_emitted_std_implementations",
        "standard_string_artifact_run_uses_emitted_std_implementations",
        "standard_fmt_artifact_run_uses_emitted_std_implementations",
        "standard_json_artifact_run_uses_emitted_std_implementations",
        "standard_hash_sha256_hex_hashes_read_bytes_for_source_and_built_runs",
        "package_std_hash_sample_runs_against_emitted_artifacts",
        "manifest_resource_export_project_sample_runs_against_emitted_artifacts",
    ] {
        assert!(
            review.contains(required),
            "stdlib package samples review missing `{required}`"
        );
    }

    for sample in [
        "samples/packages/app/std_io/main.muga",
        "samples/packages/app/std_path/main.muga",
        "samples/packages/app/std_path_join/main.muga",
        "samples/packages/app/std_path_normalize/main.muga",
        "samples/packages/app/std_path_file_name/main.muga",
        "samples/packages/app/std_path_with_file_name/main.muga",
        "samples/packages/app/std_path_parent/main.muga",
        "samples/packages/app/std_path_extension/main.muga",
        "samples/packages/app/std_path_file_stem/main.muga",
        "samples/packages/app/std_path_with_extension/main.muga",
        "samples/packages/app/std_path_is_absolute/main.muga",
        "samples/packages/app/std_fs_path/main.muga",
        "samples/packages/app/std_fs_read_dir/main.muga",
        "samples/packages/app/std_fs_read_dir_recursive/main.muga",
        "samples/packages/app/std_fs_metadata/main.muga",
        "samples/packages/app/std_fs_create_dir/main.muga",
        "samples/packages/app/std_fs_create_dir_all/main.muga",
        "samples/packages/app/std_fs_remove_file/main.muga",
        "samples/packages/app/std_fs_remove_dir/main.muga",
        "samples/packages/app/std_fs_copy_file/main.muga",
        "samples/packages/app/std_fs_move_dir_all/main.muga",
        "samples/packages/app/std_fs_rename/main.muga",
        "samples/packages/app/std_fs_file_size/main.muga",
        "samples/packages/app/std_fs_modified_time/main.muga",
        "samples/packages/app/std_fs_file_metadata/main.muga",
        "samples/packages/app/std_fs_path_metadata/main.muga",
        "samples/packages/app/std_fs_canonicalize/main.muga",
        "samples/packages/app/std_env/main.muga",
        "samples/packages/app/std_env_args/main.muga",
        "samples/packages/app/std_env_current_dir/main.muga",
        "samples/packages/app/std_env_temp_dir/main.muga",
        "samples/packages/app/std_cli/main.muga",
        "samples/packages/app/std_cli_schema/main.muga",
        "samples/packages/app/std_time/main.muga",
        "samples/packages/app/std_string/main.muga",
        "samples/packages/app/std_fmt/main.muga",
        "samples/packages/app/std_json/main.muga",
        "samples/packages/app/std_hash/main.muga",
        "samples/projects/resource_export/src/main/main.muga",
    ] {
        assert!(Path::new(sample).is_file(), "missing sample `{sample}`");
        assert!(
            readme.contains(sample),
            "README sample list must include `{sample}`"
        );
    }

    for required in [
        "package_std_io_sample_runs",
        "package_std_path_sample_runs",
        "package_std_path_with_file_name_sample_runs",
        "package_std_path_normalize_sample_runs",
        "package_std_path_with_extension_sample_runs",
        "package_std_fs_path_sample_runs",
        "package_std_fs_move_dir_all_sample_runs",
        "package_std_fs_rename_sample_runs",
        "package_std_fs_file_size_sample_runs",
        "package_std_fs_modified_time_sample_runs",
        "package_std_fs_file_metadata_sample_runs",
        "package_std_fs_metadata_sample_runs",
        "package_std_fs_path_metadata_sample_runs",
        "package_std_fs_path_size_metadata_sample_runs",
        "package_std_fs_read_dir_recursive_sample_runs",
        "package_std_fs_canonicalize_sample_runs",
        "package_std_env_sample_runs",
        "package_std_env_args_sample_runs",
        "package_std_env_current_dir_sample_runs",
        "package_std_env_temp_dir_sample_runs",
        "package_std_cli_sample_runs",
        "package_std_cli_schema_sample_runs",
        "package_std_cli_schema_sample_runs_against_emitted_artifacts",
        "package_std_time_sample_runs",
        "package_std_string_sample_runs",
        "package_std_fmt_sample_runs",
        "package_std_json_sample_runs",
        "package_std_hash_sample_runs",
        "package_std_hash_sample_runs_against_emitted_artifacts",
        "manifest_resource_export_project_sample_runs",
        "manifest_resource_export_project_sample_runs_against_emitted_artifacts",
        "standard_path_artifact_run_uses_emitted_std_implementations",
        "standard_path_with_file_name_artifact_run_uses_emitted_std_implementations",
        "standard_path_normalize_artifact_run_uses_emitted_std_implementations",
        "standard_path_with_extension_artifact_run_uses_emitted_std_implementations",
        "standard_fs_path_artifact_run_uses_emitted_std_implementations",
        "standard_fs_read_dir_artifact_run_uses_emitted_std_implementations",
        "standard_fs_read_dir_recursive_artifact_run_uses_emitted_std_implementations",
        "standard_fs_move_dir_all_artifact_run_uses_emitted_std_implementations",
        "standard_fs_rename_artifact_run_uses_emitted_std_implementations",
        "standard_fs_file_size_artifact_run_uses_emitted_std_implementations",
        "standard_fs_modified_unix_millis_artifact_run_uses_emitted_std_implementations",
        "standard_fs_file_metadata_artifact_run_uses_emitted_std_implementations",
        "standard_fs_path_status_returns_public_record",
        "standard_fs_path_info_returns_kind_and_status",
        "standard_fs_path_metadata_artifact_run_uses_emitted_std_implementations",
        "standard_fs_path_size_metadata_artifact_run_uses_emitted_std_implementations",
        "standard_fs_canonicalize_artifact_run_uses_emitted_std_implementations",
        "standard_fs_artifact_run_exposes_error_fields_without_direct_io_import",
        "standard_env_artifact_run_uses_emitted_std_implementations",
        "standard_env_args_artifact_run_uses_program_arguments",
        "standard_env_current_dir_artifact_run_uses_emitted_std_implementations",
        "standard_env_temp_dir_artifact_run_uses_emitted_std_implementations",
        "standard_cli_artifact_run_uses_emitted_std_implementations",
        "standard_time_artifact_run_uses_emitted_std_implementations",
        "standard_string_artifact_run_uses_emitted_std_implementations",
        "standard_fmt_artifact_run_uses_emitted_std_implementations",
        "standard_json_artifact_run_uses_emitted_std_implementations",
    ] {
        assert!(
            examples.contains(required),
            "examples suite must keep stdlib docs/sample review anchor `{required}`"
        );
    }

    for required in [
        "package std::io",
        "pub record IOError",
        "pub record PathPairError",
        "package std::path",
        "pub fn join(base: Path, child: String): Path",
        "pub fn normalize(path: Path): Path",
        "pub fn with_file_name(path: Path, new_file_name: String): Path",
        "pub fn with_extension(path: Path, new_extension: String): Path",
        "package std::fs",
        "pub fn read_dir_path(dir_path: path::Path): Result[List[path::Path], io::IOError]",
        "pub fn read_dir_recursive_path(root_path: path::Path): Result[List[path::Path], io::IOError]",
        "pub fn copy_file_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]",
        "pub fn move_dir_all_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]",
        "pub fn rename_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]",
        "pub fn file_size_path(file_path: path::Path): Result[Int, io::IOError]",
        "pub fn modified_unix_millis_path(target_path: path::Path): Result[time::UnixMillis, io::IOError]",
        "pub record FileMetadata",
        "pub fn file_metadata_path(file_path: path::Path): Result[FileMetadata, io::IOError]",
        "pub record PathStatus",
        "pub enum PathKind",
        "pub record PathInfo",
        "pub record PathMetadata",
        "pub fn path_status(target_path: path::Path): PathStatus",
        "pub fn path_kind(target_path: path::Path): PathKind",
        "pub fn path_info(target_path: path::Path): PathInfo",
        "pub fn path_metadata_path(target_path: path::Path): Result[PathMetadata, io::IOError]",
        "pub fn canonicalize_path(target_path: path::Path): Result[path::Path, io::IOError]",
        "package std::env",
        "pub fn args(): List[String]",
        "pub fn current_dir(): Result[path::Path, io::IOError]",
        "pub fn temp_dir(): Result[path::Path, io::IOError]",
        "package std::cli",
        "pub fn positional(args: List[String], index: Int): Option[String]",
        "pub fn option_or(args: List[String], name: String, default_value: String): String",
        "package std::time",
        "pub fn now_unix_millis(): UnixMillis",
        "package std::string",
        "pub fn concat_all(parts: List[String]): String",
        "pub fn join(parts: List[String], separator: String): String",
        "package std::fmt",
        "pub fn repeat(text: String, count: Int): String",
        "pub fn pad_left(text: String, width: Int, fill: String): String",
        "pub fn pad_right(text: String, width: Int, fill: String): String",
        "pub fn truncate_chars(text: String, max_chars: Int): String",
        "package std::json",
        "pub enum Value",
        "pub fn parse(text: String): Result[Value, Error]",
        "pub fn encode(value: Value): Result[String, Error]",
        "package std::bytes",
        "pub opaque type Bytes",
        "pub fn at(bytes: Bytes, index: Int): Option[Int]",
        "package std::hash",
        "pub fn sha256_hex(bytes: bytes::Bytes): String",
    ] {
        assert!(
            std_package.contains(required),
            "std package source missing reviewed public surface `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("stdlib package docs and samples review")
                && text.contains("std::io")
                && text.contains("std::fs")
                && text.contains("std::path")
                && text.contains("std::env")
                && text.contains("std::cli")
                && text.contains("std::time")
                && text.contains("std::string")
                && text.contains("std::fmt")
                && text.contains("std::json")
                && text.contains("artifact-backed execution samples"),
            "{label} must document the completed stdlib package docs/sample review"
        );
        assert_hands_off_to_tooling_completions_doctor(label, text);
    }
}

#[test]
fn muga_explain_scope_is_documented() {
    let readme = read_primary_docs();
    let errors = read("errors.md");
    let contract = read("docs/diagnostics-and-output.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let roadmap = read("ROADMAP.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");

    for required in [
        "muga explain E001",
        "`muga explain <diagnostic-code>`",
        "`errors.md` catalog entry",
        "diagnostic family",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "The CLI command `muga explain <diagnostic-code>`",
        "matching catalog",
        "stable prefix",
        "| `T` | detailed typing",
    ] {
        assert!(errors.contains(required), "errors.md missing `{required}`");
    }

    for required in [
        "`muga explain <diagnostic-code>`",
        "matching `errors.md` catalog",
        "stable diagnostic-code prefixes",
        "diagnostic message, related notes, suggestions",
    ] {
        assert!(
            contract.contains(required),
            "command-output contract missing `{required}`"
        );
    }

    for required in [
        "`muga explain <diagnostic-code>`",
        "`errors.md` diagnostic guidance",
        "stable diagnostic-code families",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 release checklist", checklist.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("muga explain <diagnostic-code>"),
            "{label} missing `muga explain`"
        );
        assert!(
            text.contains("errors.md") && text.contains("diagnostic"),
            "{label} must tie `muga explain` to diagnostic guidance"
        );
    }
    assert!(
        decisions.contains("[x] `muga explain <diagnostic-code>`"),
        "modern gap decisions must mark `muga explain` complete"
    );

    for required in [
        "cli_explain_reports_catalog_entry_on_stdout",
        "cli_explain_reports_known_family_for_uncataloged_code",
        "cli_explain_rejects_unknown_diagnostic_code_on_stderr",
        "cli_explain_rejects_json_format_on_stderr",
    ] {
        assert!(
            examples.contains(required),
            "examples test suite must cover `{required}`"
        );
    }

    for required in [
        "Mode::Explain",
        "diagnostic_explanation",
        "diagnostic_catalog_entry",
        "diagnostic_family_area",
        "ERROR_CATALOG",
        "muga explain <diagnostic-code>",
    ] {
        assert!(
            cli.contains(required),
            "CLI missing `muga explain` support `{required}`"
        );
    }
}

#[test]
fn muga_syntax_scope_is_documented() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let contract = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");
    let lib = read("src/lib.rs");

    for required in [
        "muga syntax --format json path/to/file.muga",
        "faster editor feedback",
        "lexes and parses",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "`muga syntax --format json <entry>`",
        "faster editor feedback",
        "diagnostics[].context",
        "does not run resolver, typechecker",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "`muga syntax --format json <entry>`",
        "\"command\": \"syntax\"",
        "lexes and parses one source file",
        "does not run resolver, typechecker",
    ] {
        assert!(
            contract.contains(required),
            "command-output contract missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 release checklist", checklist.as_str()),
    ] {
        assert!(
            text.contains("muga syntax"),
            "{label} missing `muga syntax`"
        );
        assert!(
            text.contains("LSP") || text.contains("editor"),
            "{label} must tie syntax diagnostics to editor tooling"
        );
    }

    assert!(
        examples.contains("cli_syntax_json_reports_fast_parse_feedback_for_editor_tools"),
        "examples test suite must cover `muga syntax`"
    );
    for required in [
        "Mode::Syntax",
        "syntax requires --format json",
        "syntax_check_path",
    ] {
        assert!(
            cli.contains(required),
            "CLI missing `muga syntax` support `{required}`"
        );
    }
    assert!(
        lib.contains("pub fn syntax_check_path"),
        "library must expose the syntax check path used by the CLI"
    );
}

#[test]
fn mgi_api_diff_design_is_documented() {
    let readme = read_primary_docs();
    let design = read("docs/mgi-api-diff.md");
    let api_diff = read("src/api_diff.rs");
    let main = read("src/main.rs");
    let api_diff_tests = read("tests/api_diff.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let checklist = read("docs/v1-release-checklist.md");

    assert!(
        readme.contains("docs/mgi-api-diff.md"),
        "README must link to the `.mgi` API diff design"
    );

    for required in [
        "PackageInterfaceGraph",
        "muga api-diff",
        "--old-artifact-root",
        "--new-artifact-root",
        "--fail-on breaking",
        "public function",
        "record",
        "enum",
        "implementation-only",
        "public interface hashes",
        "Compatible",
        "Source-Compatible",
        "Breaking",
        "Unknown",
        "deprecation",
        "muga::api_diff::diff_package_interfaces",
        "PackageApiDiff",
        "PackageApiDiffSummary",
        "schemaVersion",
        "\"command\": \"api-diff\"",
        "should not read source bodies",
    ] {
        assert!(
            design.contains(required),
            ".mgi API diff design missing `{required}`"
        );
    }

    for required in [
        "pub fn diff_package_interfaces",
        "PackageApiDiffStatus",
        "PackageApiDiffClassification",
        "PackageApiDiffSummary",
        "stable_hash_for_package",
        "record-field-type-changed",
        "enum-variant",
        "function-parameter-mode-relaxed",
        "opaque-handle-fact-changed",
    ] {
        assert!(
            api_diff.contains(required),
            ".mgi API diff implementation missing `{required}`"
        );
    }

    for required in [
        "Mode::ApiDiff",
        "\"api-diff\" => Mode::ApiDiff",
        "api_diff_from_cli",
        "api_diff_fail_on",
        "parse_api_diff_fail_on",
        "api_diff_text_output",
        "api_diff_json_output",
        "--fail-on unknown|breaking|source-compatible",
        "api-diff requires --old-artifact-root",
        "api-diff requires --new-artifact-root",
        "api-diff requires --package",
        "muga api-diff [--format text|json] [--fail-on unknown|breaking|source-compatible] --old-artifact-root <dir> --new-artifact-root <dir> --package <package>",
    ] {
        assert!(
            main.contains(required),
            ".mgi API diff CLI implementation missing `{required}`"
        );
    }

    for required in [
        "package_api_diff_reports_compatible_for_span_only_changes",
        "package_api_diff_reports_source_compatible_public_additions_and_renames",
        "package_api_diff_reports_breaking_public_shape_changes",
        "package_api_diff_fails_closed_for_unknown_opaque_handle_facts",
        "cli_api_diff_reports_text_and_json_from_artifacts",
        "cli_api_diff_validates_arguments_and_reports_json_diagnostics",
        "cli_api_diff_reports_compatible_for_persisted_implementation_only_edits",
        "cli_api_diff_reports_record_enum_and_generic_changes_from_artifacts",
    ] {
        assert!(
            api_diff_tests.contains(required),
            ".mgi API diff tests missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("strategy plan", strategy.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("v1 release checklist", checklist.as_str()),
    ] {
        assert!(
            text.contains("mgi-api-diff.md"),
            "{label} must point to the `.mgi` API diff design"
        );
    }
}

#[test]
fn standard_library_review_rules_are_documented() {
    let readme = read_primary_docs();
    let rules = read("docs/standard-library-review-rules.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let checklist = read("docs/v1-release-checklist.md");

    assert!(
        readme.contains("docs/standard-library-review-rules.md"),
        "README must link to the standard-library review rules"
    );

    for required in [
        "Review Checklist",
        "Result[T, E]",
        "Option[T]",
        "public error types",
        "property access must not perform hidden IO",
        "opaque resource handles",
        ".mgi",
        "artifact-backed",
        "io::IOError",
        "io::PathPairError",
        "std::fs",
        "std::path",
        "std::env",
        "std::time",
        "std::test",
        "diagnostics",
        "Deferred Until Separate Design",
    ] {
        assert!(
            rules.contains(required),
            "standard-library review rules missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("strategy plan", strategy.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("v1 release checklist", checklist.as_str()),
    ] {
        assert!(
            text.contains("standard-library-review-rules.md"),
            "{label} must point to the standard-library review rules"
        );
    }
}

#[test]
fn muga_test_scope_is_documented() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let function_spec = read("spec/004-functions.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");

    for required in [
        "muga test path/to/file.muga",
        "`muga test` for compiler-recognized `@test` functions",
        "std::test",
        "test::assert_eq_int",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "compiler-recognized `@test` metadata",
        "`@test` on named function declarations",
        "std::test",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "## 12. Static Test Attribute",
        "muga test",
        "Unit",
        "Result[Unit, E]",
        "test::assert_true",
        "test::assert_eq_int",
        "reflection",
    ] {
        assert!(
            function_spec.contains(required),
            "function spec missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
    ] {
        assert!(
            text.contains("muga test")
                && text.contains("@test")
                && text.contains("test::assert_eq_int"),
            "{label} must track the implemented muga test scope"
        );
    }

    for required in [
        "muga_test_runs_unit_and_result_tests",
        "muga_test_reports_result_err_failure",
        "muga_test_rejects_invalid_test_function_shape",
        "muga_test_rejects_unknown_and_misplaced_attributes",
        "muga_test_discovers_package_tests",
        "muga_test_assertion_helpers_report_scalar_failures",
        "muga_test_assertion_helpers_are_type_checked",
        "cli_test_reports_pass_and_failure_summary",
        "cli_test_reports_assertion_failure_message",
        "cli_test_json_reports_success_contract_on_stdout",
        "cli_test_json_reports_failure_contract_on_stdout",
        "cli_test_json_reports_assertion_failure_source_context_on_stdout",
        "cli_test_json_reports_diagnostic_contract_on_stdout",
    ] {
        assert!(
            examples.contains(required),
            "examples test suite missing `{required}`"
        );
    }

    for required in [
        "Mode::Test",
        "print_test_outcome",
        "test_json_output",
        "muga::test_path",
    ] {
        assert!(cli.contains(required), "CLI missing `{required}`");
    }
}

#[test]
fn muga_doc_scope_is_documented() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let contract = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");
    let lib = read("src/lib.rs");
    let doc = read("src/doc.rs");
    let interface = read("src/interface.rs");
    let package = read("src/package.rs");

    for required in [
        "muga doc path/to/package/main.muga",
        "`muga doc` emits Markdown documentation",
        "public package records, enums",
        "functions from the same public interface graph",
        "Public source comments written as `///`",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "`muga doc <entry>`",
        "public package interface records, enums, opaque types, functions, and item-level public source comments",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "`muga doc <entry>`",
        "Markdown documentation",
        "same public interface",
        "`.mgi` artifacts",
        "public source comments",
    ] {
        assert!(
            contract.contains(required),
            "command-output contract missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 release checklist", checklist.as_str()),
    ] {
        assert!(text.contains("muga doc"), "{label} missing `muga doc`");
        assert!(
            text.contains(".mgi"),
            "{label} must keep `muga doc` tied to `.mgi` interfaces"
        );
        assert!(
            text.contains("public source comments"),
            "{label} missing public source comment docs"
        );
    }

    assert!(
        examples.contains("cli_doc_emits_public_interface_markdown"),
        "examples test suite must cover `muga doc`"
    );
    assert!(
        examples.contains("package_interface_persists_public_doc_comments_without_hashing_them"),
        "examples test suite must cover persisted public doc comments"
    );
    for required in ["Mode::Doc", "muga doc <source-file>"] {
        assert!(
            cli.contains(required),
            "CLI missing `muga doc` support `{required}`"
        );
    }
    assert!(
        lib.contains("render_package_docs"),
        "library must expose `render_package_docs`"
    );
    for required in [
        "PackageInterfaceGraph",
        "Generated from public package interfaces.",
        "push_doc_comments",
        "pub record",
        "pub enum",
        "pub fn",
    ] {
        assert!(doc.contains(required), "doc renderer missing `{required}`");
    }
    for required in [
        "doc_comments",
        "push_doc_comment_lines",
        "parse_doc_comments",
        "Some(\"doc\") => continue",
    ] {
        assert!(
            interface.contains(required),
            "interface persistence missing `{required}`"
        );
    }
    assert!(
        package.contains("attach_doc_comments_from_source"),
        "package loader must attach public source comments"
    );
}

#[test]
fn muga_new_scope_is_documented() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let contract = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");
    let lib = read("src/lib.rs");
    let project_template = read("src/project_template.rs");

    for required in [
        "muga new --list-templates",
        "muga new --template app path/to/project",
        "`muga new --list-templates [--format json]`",
        "`muga new [--template app|lib|test|config-app|cli-tool|report-app|resource-export|package-app] <project-dir>`",
        "app, library, package-with-test, config app, strict CLI tool, report app, resource export, or local package app",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "`muga new --list-templates [--format json]`",
        "`muga new [--template app|lib|test|config-app|cli-tool|report-app|resource-export|package-app] <project-dir>`",
        "app, library package, package with tests, config app, strict CLI tool, report app, resource export app, or app plus local library starter",
        "std::config::load_json_or[T]",
        "strict `cli::parse[T]`",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "`muga new --list-templates [--format json]`",
        "`template<TAB><name><TAB><description>`",
        "`muga new [--template app|lib|test|config-app|cli-tool|report-app|resource-export|package-app] <project-dir>`",
        "`created<TAB><project-dir>`",
        "`entry<TAB><entry-path>`",
    ] {
        assert!(
            contract.contains(required),
            "command-output contract missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 release checklist", checklist.as_str()),
    ] {
        assert!(text.contains("muga new"), "{label} missing `muga new`");
        assert!(
            text.contains("app") && text.contains("lib") && text.contains("test"),
            "{label} must describe the template scope"
        );
    }

    for required in [
        "cli_new_creates_app_lib_and_test_templates",
        "cli_new_creates_cli_tool_template",
        "cli_new_creates_report_app_template",
        "cli_new_creates_resource_export_template",
        "cli_new_creates_package_app_template",
        "cli_new_lists_project_templates",
        "cli_new_rejects_non_empty_target",
        "--list-templates",
        "template\\tapp\\tCLI-first app starter",
        "config/settings.json",
        "--template=config-app",
        "--template=cli-tool",
        "--template=report-app",
        "--template=resource-export",
        "--template=package-app",
        "Generated library package starter.",
        "Generated test package starter.",
    ] {
        assert!(
            examples.contains(required),
            "examples test suite must cover `muga new` `{required}`"
        );
    }

    for required in [
        "Mode::New",
        "muga new --list-templates [--format json]",
        "muga new [--template app|lib|test|config-app|cli-tool|report-app|resource-export|package-app] <project-dir>",
        "parse_new_template",
        "print_project_template_list",
        "project_template_list_json_output",
        "--list-templates",
        "--template",
        "config-app",
        "cli-tool",
        "report-app",
        "resource-export",
        "package-app",
    ] {
        assert!(
            cli.contains(required),
            "CLI missing `muga new` support `{required}`"
        );
    }

    assert!(
        lib.contains("create_project_template"),
        "library must expose `create_project_template`"
    );
    assert!(
        lib.contains("project_template_infos"),
        "library must expose `project_template_infos`"
    );
    for required in [
        "ProjectTemplate",
        "ProjectTemplateInfo",
        "project_template_infos",
        "ConfigApp",
        "ReportApp",
        "ResourceExport",
        "PackageApp",
        "File-processing report app starter",
        "Binary resource export app starter",
        "App plus local library package starter",
        "src/main/main.muga",
        "src/lib/main.muga",
        "config/settings.json",
        "data/daily.txt",
        "resources/static/payload.bin",
        "app/src/main/main.muga",
        "shared/src/greetings/main.muga",
        "Generated library package starter.",
        "Generated test package starter.",
        "answer_is_42",
        "already exists and is not empty",
    ] {
        assert!(
            project_template.contains(required),
            "project template implementation missing `{required}`"
        );
    }
}

#[test]
fn muga_metadata_scope_is_documented() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let contract = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");
    let doc = read("src/doc.rs");

    for required in [
        "muga metadata --format json path/to/package/main.muga",
        "package/module/item/export metadata",
        "public interface docs and rendered types",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "`muga metadata --format json <entry>`",
        "package/module/item/export metadata",
        "public interface docs and rendered types",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "`muga metadata --format json <entry>`",
        "\"command\": \"metadata\"",
        "\"entryPackage\"",
        "\"publicInterface\"",
    ] {
        assert!(
            contract.contains(required),
            "command-output contract missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 release checklist", checklist.as_str()),
    ] {
        assert!(
            text.contains("muga metadata"),
            "{label} missing `muga metadata`"
        );
        assert!(
            text.contains("LSP") || text.contains("editor"),
            "{label} must tie metadata to editor tooling"
        );
    }

    assert!(
        examples.contains("cli_metadata_json_reports_package_graph_for_editor_tools"),
        "examples test suite must cover `muga metadata`"
    );
    for required in [
        "Mode::Metadata",
        "metadata_json_output",
        "push_public_interface_json",
        "metadata requires --format json",
    ] {
        assert!(
            cli.contains(required),
            "CLI missing `muga metadata` support `{required}`"
        );
    }
    assert!(
        doc.contains("pub fn render_type_info"),
        "doc type renderer must be available to metadata JSON"
    );
}

#[test]
fn muga_workspace_scope_is_documented() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let contract = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");

    for required in [
        "muga workspace --format json path/to/package/main.muga",
        "workspace metadata",
        "dependency edges",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "`muga workspace --format json <entry>`",
        "workspace metadata",
        "dependency edges",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "`muga workspace --format json <entry>`",
        "\"command\": \"workspace\"",
        "\"artifactRoot\"",
        "\"sourceFile\"",
        "\"dependencyEdges\"",
    ] {
        assert!(
            contract.contains(required),
            "command-output contract missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 release checklist", checklist.as_str()),
    ] {
        assert!(
            text.contains("muga workspace"),
            "{label} missing `muga workspace`"
        );
        assert!(
            text.contains("LSP") || text.contains("editor"),
            "{label} must tie workspace metadata to editor tooling"
        );
    }

    assert!(
        examples.contains("cli_workspace_json_reports_loaded_packages_for_editor_tools"),
        "examples test suite must cover `muga workspace`"
    );
    for required in [
        "Mode::Workspace",
        "workspace_json_output",
        "workspace requires --format json",
        "push_workspace_module_json",
    ] {
        assert!(
            cli.contains(required),
            "CLI missing `muga workspace` support `{required}`"
        );
    }
}

#[test]
fn muga_hover_scope_is_documented() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let contract = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");

    for required in [
        "muga hover --format json --line 2 --column 12 path/to/package/main.muga",
        "declaration hover",
        "public docs and signatures",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "`muga hover --format json --line <line> --column <column> <entry>`",
        "declaration hover",
        "public docs and signatures",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "`muga hover --format json --line <line> --column <column> <entry>`",
        "\"command\": \"hover\"",
        "\"signature\"",
        "\"docComments\"",
    ] {
        assert!(
            contract.contains(required),
            "command-output contract missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 release checklist", checklist.as_str()),
    ] {
        assert!(text.contains("muga hover"), "{label} missing `muga hover`");
        assert!(
            text.contains("LSP") || text.contains("editor"),
            "{label} must tie hover to editor tooling"
        );
    }

    assert!(
        examples.contains("cli_hover_json_reports_public_declaration_for_editor_tools"),
        "examples test suite must cover `muga hover`"
    );
    for required in [
        "Mode::Hover",
        "hover_json_output",
        "hover requires --line",
        "render_hover_signature",
    ] {
        assert!(
            cli.contains(required),
            "CLI missing `muga hover` support `{required}`"
        );
    }
}

#[test]
fn muga_completions_scope_is_documented() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let contract = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");

    for required in [
        "muga completions --format json path/to/package/main.muga",
        "visible package/interface completions",
        "public docs and signatures",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "`muga completions --format json <entry>`",
        "visible package/interface completions",
        "public docs and signatures",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "`muga completions --format json <entry>`",
        "\"command\": \"completions\"",
        "\"completions\"",
        "\"label\"",
        "\"detail\"",
    ] {
        assert!(
            contract.contains(required),
            "command-output contract missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 release checklist", checklist.as_str()),
    ] {
        assert!(
            text.contains("muga completions"),
            "{label} missing `muga completions`"
        );
        assert!(
            text.contains("LSP") || text.contains("editor"),
            "{label} must tie completions to editor tooling"
        );
    }

    assert!(
        examples.contains("cli_completions_json_reports_visible_package_items_for_editor_tools"),
        "examples test suite must cover `muga completions`"
    );
    for required in [
        "Mode::Completions",
        "completions_json_output",
        "completions requires --format json",
        "push_completion_item_json",
    ] {
        assert!(
            cli.contains(required),
            "CLI missing `muga completions` support `{required}`"
        );
    }
}

#[test]
fn tooling_shell_completions_and_doctor_are_documented_and_tool_only() {
    let readme = read_primary_docs();
    let contract = read("docs/diagnostics-and-output.md");
    let onboarding = read("docs/installation-and-onboarding.md");
    let tool_doc = read("docs/shell-completions-and-doctor.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let roadmap = read("ROADMAP.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");

    for required in [
        "muga shell-completions bash",
        "muga doctor --format json",
        "docs/shell-completions-and-doctor.md",
        "tool-only",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "`muga doctor [--format text|json]`",
        "`muga shell-completions <bash|zsh|fish>`",
        "\"command\":\"doctor\"",
        "tool-only",
    ] {
        assert!(
            contract.contains(required),
            "command-output contract missing `{required}`"
        );
    }

    for required in [
        "muga doctor",
        "muga shell-completions bash",
        "shell-completions-and-doctor.md",
        "tool-only",
    ] {
        assert!(
            onboarding.contains(required),
            "onboarding missing `{required}`"
        );
    }

    for required in [
        "muga shell-completions <bash|zsh|fish>",
        "muga doctor [--format text|json]",
        "tool-only",
        "do not parse or check Muga",
        "\"command\": \"doctor\"",
        "first\n`std::json` implementation follows",
    ] {
        assert!(
            tool_doc.contains(required),
            "shell completions / doctor doc missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 release checklist", checklist.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert_hands_off_to_tooling_completions_doctor(label, text);
        assert_hands_off_to_std_json_design(label, text);
    }

    for required in [
        "cli_doctor_reports_environment_checks",
        "cli_doctor_json_reports_environment_checks",
        "cli_shell_completions_reports_static_scripts_on_stdout",
        "cli_shell_completions_rejects_unknown_shell",
    ] {
        assert!(
            examples.contains(required),
            "examples test suite missing `{required}`"
        );
    }

    for required in [
        "Mode::Doctor",
        "Mode::ShellCompletions",
        "doctor_json_output",
        "doctor_text_output",
        "shell_completion_script",
        "is_supported_shell_completion_shell",
        "muga shell-completions <bash|zsh|fish>",
    ] {
        assert!(
            cli.contains(required),
            "CLI missing shell completions / doctor support `{required}`"
        );
    }
}

#[test]
fn std_json_first_slice_design_and_boundary_are_documented() {
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let design = read("docs/std-json-first-slice.md");
    let audit = read("docs/std-json-implementation-audit.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");

    for required in [
        "does not implement the package",
        "pub enum Value",
        "pub enum Number",
        "pub enum ErrorKind",
        "pub record Error",
        "pub fn parse(text: String): Result[Value, Error]",
        "pub fn encode(value: Value): Result[String, Error]",
        "pub fn number_as_int(number: Number): Result[Int, Error]",
        "Result Ergonomics",
        "Scalar And Collection Mapping",
        "Schema Evolution",
        "Diagnostics",
        "Number::Raw",
        "DuplicateKey",
        "sorts keys lexicographically",
        "NestingLimitExceeded",
        "artifact-backed execution",
        "must not expand into schema generation",
        "`Float`, `Decimal`, `Bytes`, streaming APIs, or resource handles",
        "128 nested arrays or objects",
    ] {
        assert!(
            design.contains(required),
            "std json design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 release checklist", checklist.as_str()),
        ("standard library review rules", stdlib_rules.as_str()),
    ] {
        assert!(
            text.contains("std-json-first-slice.md"),
            "{label} must link the first std::json slice design"
        );
        assert_std_json_first_slice_boundary(label, text);
    }

    for required in [
        "std-json-first-slice.md",
        "standard_json_encode_escapes_strings",
        "standard_json_parse_exposes_error_offset",
        "standard_json_encode_rejects_invalid_raw_number",
        "standard_json_encode_reports_nesting_limit",
        "standard_json_artifact_run_uses_emitted_std_implementations",
        "schema generation",
        "HTTP/RPC",
        "Float",
        "Decimal",
        "Bytes",
        "resource handles",
        "Result ergonomics",
        "scalar/collection mapping",
        "schema evolution",
        "diagnostics",
    ] {
        assert!(
            audit.contains(required),
            "std json implementation audit missing `{required}`"
        );
    }

    assert!(
        implementation_resume.contains("| 142. first std::json implementation |")
            && implementation_resume.contains("| std package/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 143. first std::json implementation audit |")
            && implementation_resume.contains("| docs/tests/samples | Done |")
            && implementation_resume.contains("| 144. post-json stdlib boundary selection |")
            && implementation_resume.contains("| docs | Done |")
            && implementation_resume.contains("| 145. opaque resource handle boundary design |")
            && implementation_resume.contains("| docs/spec/tests | Done |")
            && implementation_resume.contains("| 146. first opaque type interface slice plan |")
            && implementation_resume.contains("| docs/spec/tests | Done |")
            && implementation_resume
                .contains("| 147. first opaque type interface slice implementation |")
            && implementation_resume.contains("| parser/package/interface/CLI/tests/docs | Done |")
            && implementation_resume
                .contains("| 148. opaque handle capability and close metadata plan |")
            && implementation_resume.contains("| docs/spec/tests | Done |")
            && implementation_resume
                .contains("| 149. opaque handle metadata interface implementation |")
            && implementation_resume.contains("| interface/package/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 150. consuming parameter dataflow checker |")
            && implementation_resume.contains("| typing/package/tests/docs | Done |")
            && implementation_resume
                .contains("| 151. first runtime file handle implementation design |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume
                .contains("| 152. first read-only runtime file handle implementation |")
            && implementation_resume.contains("| runtime/std_package/typing/tests/docs | Done |")
            && implementation_resume
                .contains("| 153. post-file-handle resource surface selection |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume.contains("| 154. program stderr output channel |")
            && implementation_resume.contains("| runtime/prelude/typing/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 155. text output file handle design |")
            && implementation_resume.contains("| docs/std/runtime/tests | Done |")
            && implementation_resume.contains("| 156. text output file handle implementation |")
            && implementation_resume
                .contains("| std_package/prelude/typing/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation resume plan must mark std::json, opaque interface, and metadata implementation done, then hand off to consuming diagnostics"
    );
}

#[test]
fn post_json_stdlib_boundary_selection_is_documented() {
    let selection = read("docs/post-json-stdlib-boundary-selection.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");

    for required in [
        "opaque resource-handle design",
        "not a new effectful runtime API",
        "stdout/stderr handles",
        "file handles",
        "process APIs",
        "HTTP/SSE/WebSocket/RPC",
        "streaming APIs",
        "`Bytes`",
        "buffers",
        "schema/client generation",
        "List.contains",
        "Map.entries",
        "scalar-only equality",
        "pub opaque type",
        ".mgi",
        "Result",
        "ownership",
        "close",
        "clone/share",
        "cancellation",
    ] {
        assert!(
            selection.contains(required),
            "post-json stdlib boundary selection missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("standard library review rules", stdlib_rules.as_str()),
    ] {
        assert!(
            text.contains("post-json-stdlib-boundary-selection.md")
                && text.contains("opaque resource")
                && text.contains("stdout/stderr handles")
                && text.contains("process")
                && text.contains("HTTP")
                && text.contains("Bytes"),
            "{label} must link and preserve the post-json stdlib boundary selection"
        );
    }

    assert!(
        implementation_resume.contains("| 144. post-json stdlib boundary selection |")
            && implementation_resume.contains("| docs | Done |")
            && implementation_resume.contains("| 145. opaque resource handle boundary design |")
            && implementation_resume.contains("| docs/spec/tests | Done |")
            && implementation_resume.contains("| 146. first opaque type interface slice plan |")
            && implementation_resume.contains("| docs/spec/tests | Done |")
            && implementation_resume
                .contains("| 147. first opaque type interface slice implementation |")
            && implementation_resume.contains("| parser/package/interface/CLI/tests/docs | Done |")
            && implementation_resume
                .contains("| 148. opaque handle capability and close metadata plan |")
            && implementation_resume.contains("| docs/spec/tests | Done |")
            && implementation_resume
                .contains("| 149. opaque handle metadata interface implementation |")
            && implementation_resume.contains("| interface/package/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 150. consuming parameter dataflow checker |")
            && implementation_resume.contains("| typing/package/tests/docs | Done |")
            && implementation_resume
                .contains("| 151. first runtime file handle implementation design |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume
                .contains("| 152. first read-only runtime file handle implementation |")
            && implementation_resume.contains("| runtime/std_package/typing/tests/docs | Done |")
            && implementation_resume
                .contains("| 153. post-file-handle resource surface selection |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume.contains("| 154. program stderr output channel |")
            && implementation_resume.contains("| runtime/prelude/typing/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 155. text output file handle design |")
            && implementation_resume.contains("| docs/std/runtime/tests | Done |")
            && implementation_resume.contains("| 156. text output file handle implementation |")
            && implementation_resume
                .contains("| std_package/prelude/typing/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation resume plan must mark post-json boundary, opaque interface, and metadata implementation done, then queue consuming diagnostics"
    );
}

#[test]
fn opaque_resource_handle_boundary_is_documented() {
    let design = read("docs/opaque-resource-handles.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let mgi_api_diff = read("docs/mgi-api-diff.md");
    let value_semantics = read("spec/011-value-semantics.md");
    let package_spec = read("spec/006-packages.md");

    for required in [
        "does not implement",
        "pub opaque type Name",
        "Source Contract",
        "Package Interface Contract",
        "API Diff Contract",
        "Resource Capability Defaults",
        "Consuming Operations",
        "Close And Cleanup",
        "Task Boundaries And Cancellation",
        "Runtime Diagnostics",
        "not copyable",
        "not cloneable",
        "not shareable across tasks",
        "not sendable across tasks",
        "consume",
        "Result[Unit, io::IOError]",
        "use after close",
        "stale runtime slot",
        "First Implementation Slice",
        "Interface Slice Plan",
        "parser and AST support",
        "package item identity",
        "nominal `TypeInfo`",
        "persist opaque type entries in `.mgi`",
        "construction",
        "field access",
        "match patterns",
        "structural equality",
        "formatting",
        "runtime-backed handle values and effectful stdlib packages deferred",
    ] {
        assert!(
            design.contains(required),
            "opaque resource-handle design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("standard library review rules", stdlib_rules.as_str()),
        ("mgi api diff", mgi_api_diff.as_str()),
        ("value semantics spec", value_semantics.as_str()),
        ("package spec", package_spec.as_str()),
    ] {
        assert!(
            text.contains("opaque-resource-handles.md")
                && text.contains("opaque")
                && text.contains("resource"),
            "{label} must link and preserve the opaque resource-handle boundary"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("modern gap decisions", decisions.as_str()),
    ] {
        assert!(
            text.contains("first opaque type interface")
                || text.contains("`pub opaque type` interface")
                || text.contains("pub opaque type"),
            "{label} must hand off to the first opaque type interface slice"
        );
    }

    assert!(
        implementation_resume.contains("| 145. opaque resource handle boundary design |")
            && implementation_resume.contains("| docs/spec/tests | Done |")
            && implementation_resume.contains("| 146. first opaque type interface slice plan |")
            && implementation_resume.contains("| docs/spec/tests | Done |")
            && implementation_resume
                .contains("| 147. first opaque type interface slice implementation |")
            && implementation_resume.contains("| parser/package/interface/CLI/tests/docs | Done |")
            && implementation_resume
                .contains("| 148. opaque handle capability and close metadata plan |")
            && implementation_resume.contains("| docs/spec/tests | Done |")
            && implementation_resume
                .contains("| 149. opaque handle metadata interface implementation |")
            && implementation_resume.contains("| interface/package/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 150. consuming parameter dataflow checker |")
            && implementation_resume.contains("| typing/package/tests/docs | Done |")
            && implementation_resume
                .contains("| 151. first runtime file handle implementation design |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume
                .contains("| 152. first read-only runtime file handle implementation |")
            && implementation_resume.contains("| runtime/std_package/typing/tests/docs | Done |")
            && implementation_resume
                .contains("| 153. post-file-handle resource surface selection |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume.contains("| 154. program stderr output channel |")
            && implementation_resume.contains("| runtime/prelude/typing/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 155. text output file handle design |")
            && implementation_resume.contains("| docs/std/runtime/tests | Done |")
            && implementation_resume.contains("| 156. text output file handle implementation |")
            && implementation_resume
                .contains("| std_package/prelude/typing/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation resume plan must mark resource-handle design, interface implementation, and metadata implementation done, then queue consuming diagnostics"
    );
}

#[test]
fn opaque_type_interface_slice_is_implemented_and_covered() {
    let parser = read("src/parser.rs");
    let package = read("src/package.rs");
    let package_signature = read("src/package_signature.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let interface = read("src/interface.rs");
    let cli = read("src/main.rs");
    let docs = read("src/doc.rs");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let design = read("docs/opaque-resource-handles.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let package_spec = read("spec/006-packages.md");
    let command_contract = read("docs/diagnostics-and-output.md");
    let api_diff = read("docs/mgi-api-diff.md");

    for required in [
        "parse_opaque_type_decl_with_visibility",
        "opaque type declarations must be public",
        "`pub opaque type` is only allowed in package mode",
    ] {
        assert!(parser.contains(required), "parser missing `{required}`");
    }

    for required in [
        "PackageItemKind::OpaqueType",
        "mangle_opaque_type_name_for_visibility",
        "opaque type `{name}` is not visible",
    ] {
        assert!(package.contains(required), "package missing `{required}`");
    }

    for required in [
        "PackageOpaqueTypeSignature",
        "validate_opaque_arg_count",
        "TypeInfo::PackageOpaque",
    ] {
        assert!(
            package_signature.contains(required),
            "package signatures missing `{required}`"
        );
    }

    for required in [
        "Type::Opaque",
        "predeclare_opaque_types",
        "opaque type `{}` expects exactly 0 type arguments",
        "opaque type `{}` cannot be constructed",
    ] {
        assert!(typing.contains(required), "typing missing `{required}`");
    }

    for required in [
        "Stmt::OpaqueType",
        "PackageOpaque",
        "package_opaque_items_by_symbol",
    ] {
        assert!(
            typed_hir.contains(required),
            "typed HIR missing `{required}`"
        );
    }

    for required in [
        "PackageInterfaceOpaqueType",
        "\"opaque-type\"",
        "\"PackageOpaque\"",
        "opaque_type_by_name",
    ] {
        assert!(
            interface.contains(required),
            "interface missing `{required}`"
        );
    }

    for required in [
        "opaqueTypes",
        "opaqueType",
        "render_hover_opaque_type_signature",
    ] {
        assert!(cli.contains(required), "CLI missing `{required}`");
    }

    assert!(
        docs.contains("push_opaque_type_docs") && docs.contains("pub opaque type"),
        "docs renderer must include public opaque types"
    );

    for required in [
        "package_aware_typed_program_preserves_public_opaque_types",
        "package_aware_checking_rejects_non_public_opaque_type_declarations",
        "package_aware_checking_rejects_opaque_type_arguments_and_equality",
        "package_aware_checking_rejects_opaque_value_operations",
        "downstream_package_can_name_loaded_opaque_type_signature",
        "json_backed_editor_workflow_uses_existing_command_contracts",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing opaque coverage `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("opaque resource design", design.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("package spec", package_spec.as_str()),
        ("command-output contract", command_contract.as_str()),
        ("mgi API diff", api_diff.as_str()),
    ] {
        assert!(
            text.contains("pub opaque type") && text.contains("opaque"),
            "{label} must document the implemented opaque type interface slice"
        );
    }

    assert!(
        implementation_resume.contains("| 147. first opaque type interface slice implementation |")
            && implementation_resume.contains("| parser/package/interface/CLI/tests/docs | Done |")
            && implementation_resume
                .contains("| 148. opaque handle capability and close metadata plan |")
            && implementation_resume.contains("| docs/spec/tests | Done |")
            && implementation_resume
                .contains("| 149. opaque handle metadata interface implementation |")
            && implementation_resume.contains("| interface/package/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 150. consuming parameter dataflow checker |")
            && implementation_resume.contains("| typing/package/tests/docs | Done |")
            && implementation_resume
                .contains("| 151. first runtime file handle implementation design |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume
                .contains("| 152. first read-only runtime file handle implementation |")
            && implementation_resume.contains("| runtime/std_package/typing/tests/docs | Done |")
            && implementation_resume
                .contains("| 153. post-file-handle resource surface selection |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume.contains("| 154. program stderr output channel |")
            && implementation_resume.contains("| runtime/prelude/typing/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 155. text output file handle design |")
            && implementation_resume.contains("| docs/std/runtime/tests | Done |")
            && implementation_resume.contains("| 156. text output file handle implementation |")
            && implementation_resume
                .contains("| std_package/prelude/typing/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation queue must mark opaque type interface and metadata implementation done and queue consuming diagnostics"
    );
}

#[test]
fn opaque_handle_capability_close_metadata_plan_is_documented() {
    let design = read("docs/opaque-resource-handles.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let api_diff = read("docs/mgi-api-diff.md");

    for required in [
        "Capability And Close Metadata Plan",
        "OpaqueHandleFacts",
        "`runtimeBacked = true`",
        "`copyable = false`",
        "`cloneable = false`",
        "`sendable = false`",
        "`shareable = false`",
        "`closeable = false`",
        "Consuming Parameters",
        "`borrow`",
        "`consume`",
        "`paramMode`",
        "use-after-consume",
        "Close Function Metadata",
        "`Result[Unit, E]`",
        "Interface And API Diff Rules",
        "changing a parameter from `borrow` to `consume` is breaking",
        "First Candidate API",
        "pub opaque type File",
        "pub fn open_text(path: path::Path): Result[File, io::IOError]",
        "pub fn close(file: File): Result[Unit, io::IOError]",
        "Only `close` should consume",
        "Implementation Order",
    ] {
        assert!(
            design.contains(required),
            "opaque handle metadata plan missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
    ] {
        assert!(
            text.contains("OpaqueHandleFacts")
                && text.contains("consuming")
                && text.contains("parameter")
                && text.contains("metadata")
                && (text.contains("std::fs::File")
                    || text.contains("future `std::fs::File`")
                    || text.contains("file handles")),
            "{label} must preserve the opaque handle metadata handoff"
        );
    }

    assert!(
        api_diff.contains("opaque type capability")
            && api_diff.contains("consuming-parameter metadata"),
        "mgi API diff design must reserve opaque capability and consuming-parameter metadata"
    );

    assert!(
        implementation_resume.contains("| 148. opaque handle capability and close metadata plan |")
            && implementation_resume.contains("| docs/spec/tests | Done |")
            && implementation_resume
                .contains("| 149. opaque handle metadata interface implementation |")
            && implementation_resume.contains("| interface/package/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 150. consuming parameter dataflow checker |")
            && implementation_resume.contains("| typing/package/tests/docs | Done |")
            && implementation_resume
                .contains("| 151. first runtime file handle implementation design |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume
                .contains("| 152. first read-only runtime file handle implementation |")
            && implementation_resume.contains("| runtime/std_package/typing/tests/docs | Done |")
            && implementation_resume
                .contains("| 153. post-file-handle resource surface selection |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume.contains("| 154. program stderr output channel |")
            && implementation_resume.contains("| runtime/prelude/typing/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 155. text output file handle design |")
            && implementation_resume.contains("| docs/std/runtime/tests | Done |")
            && implementation_resume.contains("| 156. text output file handle implementation |")
            && implementation_resume
                .contains("| std_package/prelude/typing/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation queue must mark opaque handle metadata implementation done and queue consuming diagnostics"
    );
}

#[test]
fn opaque_handle_metadata_interface_slice_is_implemented_and_covered() {
    let interface = read("src/interface.rs");
    let package_signature = read("src/package_signature.rs");
    let cli = read("src/main.rs");
    let docs = read("src/doc.rs");
    let examples = read("tests/examples.rs");
    let command_contract = read("docs/diagnostics-and-output.md");
    let package_spec = read("spec/006-packages.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let design = read("docs/opaque-resource-handles.md");
    let api_diff = read("docs/mgi-api-diff.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "muga-package-interface-v11",
        "muga-package-interface-v5",
        "muga-package-interface-v4",
        "OpaqueHandleFacts",
        "PackageInterfaceParamMode",
        "format_opaque_handle_facts",
        "parse_opaque_handle_facts",
        "parse_param_mode",
        "closeFunction",
        "runtimeBacked",
        "PackageInterfaceParamMode::Borrow",
        "PackageInterfaceParamMode::Consume",
        "expected.mode == PackageInterfaceParamMode::Borrow",
    ] {
        assert!(
            interface.contains(required),
            "interface metadata implementation missing `{required}`"
        );
    }

    for required in [
        "mode: PackageInterfaceParamMode",
        "param.mode",
        "PackageInterfaceParamMode::Borrow",
    ] {
        assert!(
            package_signature.contains(required),
            "package signature metadata propagation missing `{required}`"
        );
    }

    for required in [
        "push_opaque_handle_facts_json",
        "handleFacts",
        "paramMode",
        "paramModes",
        "push_completion_metadata_json",
        "push_hover_metadata_json",
    ] {
        assert!(
            cli.contains(required),
            "CLI metadata output missing `{required}`"
        );
    }

    for required in [
        "render_opaque_handle_facts",
        "handleFacts",
        "paramMode",
        "PackageInterfaceParamMode",
    ] {
        assert!(
            docs.contains(required),
            "docs metadata output missing `{required}`"
        );
    }

    for required in [
        "package_interface_hash_and_round_trip_include_opaque_handle_metadata",
        "package_aware_typed_program_preserves_public_opaque_types",
        "json_backed_editor_workflow_uses_existing_command_contracts",
        "muga-package-interface-v11",
        "handleFacts",
        "runtimeBacked",
        "paramMode",
        "borrow",
        "PackageInterfaceParamMode::Consume",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing metadata coverage `{required}`"
        );
    }

    for (label, text) in [
        ("command contract", command_contract.as_str()),
        ("package spec", package_spec.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("opaque resource design", design.as_str()),
        ("mgi API diff", api_diff.as_str()),
    ] {
        assert!(
            (text.contains("handleFacts") || text.contains("OpaqueHandleFacts"))
                && text.contains("paramMode"),
            "{label} must document opaque handle metadata interface output"
        );
    }

    assert!(
        implementation_resume.contains("| 149. opaque handle metadata interface implementation |")
            && implementation_resume.contains("| interface/package/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 150. consuming parameter dataflow checker |")
            && implementation_resume.contains("| typing/package/tests/docs | Done |")
            && implementation_resume
                .contains("| 151. first runtime file handle implementation design |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume
                .contains("| 152. first read-only runtime file handle implementation |")
            && implementation_resume.contains("| runtime/std_package/typing/tests/docs | Done |")
            && implementation_resume
                .contains("| 153. post-file-handle resource surface selection |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume.contains("| 154. program stderr output channel |")
            && implementation_resume.contains("| runtime/prelude/typing/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 155. text output file handle design |")
            && implementation_resume.contains("| docs/std/runtime/tests | Done |")
            && implementation_resume.contains("| 156. text output file handle implementation |")
            && implementation_resume
                .contains("| std_package/prelude/typing/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation queue must mark opaque handle metadata implementation done and queue consuming diagnostics"
    );
}

#[test]
fn consuming_parameter_dataflow_checker_is_implemented_and_covered() {
    let typing = read("src/typing.rs");
    let examples = read("tests/examples.rs");
    let errors = read("errors.md");
    let design = read("docs/opaque-resource-handles.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let package_spec = read("spec/006-packages.md");
    let mini_spec = read("mini-language-spec-v1.md");

    for required in [
        "PackageInterfaceParamMode::Consume",
        "package_function_param_modes",
        "consumed_bindings",
        "param_modes_for_callee",
        "check_consumed_binding_use",
        "mark_consumed_argument",
        "clear_consumed_binding",
        "T026",
        "binding `{name}` was consumed and cannot be used again",
    ] {
        assert!(
            typing.contains(required),
            "typing consume checker missing `{required}`"
        );
    }

    for required in [
        "loaded_interface_consume_param_rejects_use_after_consume",
        "PackageInterfaceParamMode::Consume",
        "binding `token` was consumed and cannot be used again",
        "compile_typed_path_against_loaded_interfaces",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing consuming checker coverage `{required}`"
        );
    }

    for (label, text) in [
        ("error catalog", errors.as_str()),
        ("opaque resource design", design.as_str()),
        ("package spec", package_spec.as_str()),
    ] {
        assert!(
            text.contains("T026") && text.contains("use-after-consume") && text.contains("consume"),
            "{label} must document consuming parameter diagnostics"
        );
    }

    assert!(
        !mini_spec.contains("consuming-use diagnostics, and resource-handle stdlib APIs"),
        "mini spec must not keep consuming-use diagnostics in the unimplemented list"
    );

    assert!(
        implementation_resume.contains("| 150. consuming parameter dataflow checker |")
            && implementation_resume.contains("| typing/package/tests/docs | Done |")
            && implementation_resume
                .contains("| 151. first runtime file handle implementation design |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume
                .contains("| 152. first read-only runtime file handle implementation |")
            && implementation_resume.contains("| runtime/std_package/typing/tests/docs | Done |")
            && implementation_resume
                .contains("| 153. post-file-handle resource surface selection |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume.contains("| 154. program stderr output channel |")
            && implementation_resume.contains("| runtime/prelude/typing/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 155. text output file handle design |")
            && implementation_resume.contains("| docs/std/runtime/tests | Done |")
            && implementation_resume.contains("| 156. text output file handle implementation |")
            && implementation_resume
                .contains("| std_package/prelude/typing/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation queue must mark consuming diagnostics done and queue runtime file-handle design"
    );
}

#[test]
fn first_runtime_file_handle_design_is_documented() {
    let design = read("docs/opaque-resource-handles.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");

    for required in [
        "First Runtime File Handle Implementation Design",
        "read-only",
        "`write_text_to` remains deferred",
        "{ family, slot, generation }",
        "family =\n  \"std::fs::File\"",
        "Open { path, file }",
        "Closed { generation }",
        "runtimeBacked=true",
        "closeFunction=std::fs::close",
        "open_text(path: path::Path): Result[File, io::IOError]",
        "read_text_from(file: File): Result[String, io::IOError]",
        "close(file: File): Result[Unit, io::IOError]",
        "T026",
        "stale slot",
        "wrong family",
        "double close",
        "run --built",
        "fresh handle",
        "table per VM run",
        "write modes",
        "stdout/stderr handles",
    ] {
        assert!(
            design.contains(required),
            "first runtime file-handle design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
    ] {
        assert!(
            text.contains("std::fs::File")
                && text.contains("read-only")
                && (text.contains("open_text") || text.contains("runtime")),
            "{label} must preserve the first runtime file-handle design handoff"
        );
    }

    assert!(
        implementation_resume.contains("| 151. first runtime file handle implementation design |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume
                .contains("| 152. first read-only runtime file handle implementation |")
            && implementation_resume.contains("| runtime/std_package/typing/tests/docs | Done |")
            && implementation_resume
                .contains("| 153. post-file-handle resource surface selection |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume.contains("| 154. program stderr output channel |")
            && implementation_resume.contains("| runtime/prelude/typing/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 155. text output file handle design |")
            && implementation_resume.contains("| docs/std/runtime/tests | Done |")
            && implementation_resume.contains("| 156. text output file handle implementation |")
            && implementation_resume
                .contains("| std_package/prelude/typing/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation queue must mark first runtime file-handle design done and queue read-only implementation"
    );
}

#[test]
fn read_only_runtime_file_handle_is_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let runtime = read("src/runtime.rs");
    let typing = read("src/typing.rs");
    let interface = read("src/interface.rs");
    let package_signature = read("src/package_signature.rs");
    let examples = read("tests/examples.rs");
    let errors = read("errors.md");
    let design = read("docs/opaque-resource-handles.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");

    for required in [
        "pub opaque type File",
        "pub fn open_text(file_path: path::Path): Result[File, io::IOError]",
        "pub fn read_text_from(file: File): Result[String, io::IOError]",
        "pub fn close(file: File): Result[Unit, io::IOError]",
        "FS_OPEN_TEXT_BUILTIN",
        "FS_READ_TEXT_FROM_BUILTIN",
        "FS_CLOSE_BUILTIN",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing `{required}`"
        );
    }

    for required in ["StdFsOpenText", "StdFsReadTextFrom", "StdFsClose"] {
        assert!(prelude.contains(required), "prelude missing `{required}`");
        assert!(typing.contains(required), "typing missing `{required}`");
    }

    for required in [
        "RuntimeHandle",
        "RuntimeHandles",
        "STD_FS_FILE_HANDLE_FAMILY",
        "open_std_fs_file",
        "read_std_fs_file_text",
        "close_std_fs_file",
        "R022",
    ] {
        assert!(runtime.contains(required), "runtime missing `{required}`");
    }

    for required in [
        "package_opaque_handle_facts",
        "runtime_backed: true",
        "close_function: std_fs_close_item",
        "PackageInterfaceParamMode::Consume",
    ] {
        assert!(
            interface.contains(required),
            "interface metadata missing `{required}`"
        );
    }

    for required in ["package_param_mode", "\"close\"", "\"file\"", "Consume"] {
        assert!(
            package_signature.contains(required),
            "package signature metadata missing `{required}`"
        );
    }

    for required in [
        "standard_fs_file_handle_open_read_close_runs_as_virtual_package",
        "standard_fs_file_handle_open_missing_file_returns_io_error",
        "standard_fs_file_handle_close_consumes_binding",
        "standard_fs_file_handle_alias_after_close_is_runtime_error",
        "standard_fs_file_handle_artifact_run_uses_fresh_handle_table",
        "standard_fs_file_handle_interface_exposes_handle_facts_and_close_mode",
    ] {
        assert!(
            examples.contains(required),
            "examples must cover `{required}`"
        );
    }

    assert!(
        errors.contains("R022")
            && errors.contains("Runtime Resource Handles")
            && errors.contains("stale")
            && errors.contains("already-closed"),
        "errors.md must document runtime handle diagnostics"
    );

    for (label, text) in [
        ("opaque resource design", design.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
    ] {
        assert!(
            text.contains("std::fs::File")
                && text.contains("read-only")
                && text.contains("resource")
                && (text.contains("implemented") || text.contains("implementation")),
            "{label} must record the read-only runtime file-handle implementation"
        );
    }

    assert!(
        implementation_resume
            .contains("| 152. first read-only runtime file handle implementation |")
            && implementation_resume.contains("| runtime/std_package/typing/tests/docs | Done |")
            && implementation_resume
                .contains("| 153. post-file-handle resource surface selection |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume.contains("| 154. program stderr output channel |")
            && implementation_resume.contains("| runtime/prelude/typing/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 155. text output file handle design |")
            && implementation_resume.contains("| docs/std/runtime/tests | Done |")
            && implementation_resume.contains("| 156. text output file handle implementation |")
            && implementation_resume
                .contains("| std_package/prelude/typing/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation queue must mark read-only std::fs::File done and queue resource-surface selection"
    );
}

#[test]
fn post_file_handle_resource_surface_selection_is_documented() {
    let selection = read("docs/post-file-handle-resource-surface-selection.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let opaque_design = read("docs/opaque-resource-handles.md");

    for required in [
        "Audit Of The Implemented File Handle",
        "Selection Criteria",
        "Candidates Compared",
        "Program stderr channel",
        "eprint",
        "eprintln",
        "not stdout/stderr handles",
        "Write-mode text file handles",
        "`Bytes` value type",
        "Lexical cleanup `using`",
        "Process APIs",
        "HTTP / SSE / WebSocket / RPC",
        "Selected Surface",
        "populate the existing `muga run --format json` `stderr` field",
        "text output file handles",
        "create_text",
        "append_text",
        "write_text_to",
        "flush",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "standard-stream handles",
        "structured concurrency",
        "schema and client generation",
    ] {
        assert!(
            selection.contains(required),
            "post-file-handle selection missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("standard library review rules", stdlib_rules.as_str()),
        ("opaque resource design", opaque_design.as_str()),
    ] {
        assert!(
            text.contains("post-file-handle-resource-surface-selection.md")
                && text.contains("stderr")
                && text.contains("handles"),
            "{label} must link and preserve the post-file-handle selection"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
    ] {
        assert!(
            text.contains("eprint") && text.contains("eprintln"),
            "{label} must name the selected scalar stderr builtins"
        );
    }

    assert!(
        implementation_resume.contains("| 153. post-file-handle resource surface selection |")
            && implementation_resume.contains("| docs/runtime/std/tests | Done |")
            && implementation_resume.contains("| 154. program stderr output channel |")
            && implementation_resume.contains("| runtime/prelude/typing/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 155. text output file handle design |")
            && implementation_resume.contains("| docs/std/runtime/tests | Done |")
            && implementation_resume.contains("| 156. text output file handle implementation |")
            && implementation_resume
                .contains("| std_package/prelude/typing/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation queue must mark post-file-handle selection done and queue program stderr"
    );
}

#[test]
fn program_stderr_output_channel_is_implemented_and_covered() {
    let prelude = read("src/prelude.rs");
    let runtime = read("src/runtime.rs");
    let typing = read("src/typing.rs");
    let cli = read("src/main.rs");
    let lib = read("src/lib.rs");
    let examples = read("tests/examples.rs");
    let contract = read("docs/diagnostics-and-output.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let post_file_selection = read("docs/post-file-handle-resource-surface-selection.md");

    for required in [
        "BuiltinId::Eprint",
        "BuiltinId::Eprintln",
        "name: \"eprint\"",
        "name: \"eprintln\"",
        "Builtin(eprint)",
        "Builtin(eprintln)",
    ] {
        assert!(prelude.contains(required), "prelude missing `{required}`");
    }

    for required in [
        "stderr_text",
        "stderr.borrow().clone()",
        "BuiltinId::Eprint",
        "BuiltinId::Eprintln",
        "`eprint` accepts only Int, Bool, or String",
        "`eprintln` accepts only Int, Bool, or String",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing stderr implementation `{required}`"
        );
    }

    for required in [
        "BuiltinId::Eprint",
        "BuiltinId::Eprintln",
        "before calling `print`, `println`, `eprint`, or `eprintln`",
    ] {
        assert!(typing.contains(required), "typing missing `{required}`");
    }

    for required in [
        "stderr_text",
        "push_test_case_json",
        "print_run_outcome",
        "eprint!(\"{}\", outcome.stderr_text)",
    ] {
        assert!(cli.contains(required), "CLI missing `{required}`");
    }

    assert!(
        lib.contains("pub stderr_text: String") && lib.contains("stderr_text: outcome.stderr_text"),
        "library test results must expose per-test stderr"
    );

    for required in [
        "builtin_eprint_eprintln_capture_stderr_and_return_argument",
        "builtin_eprintln_rejects_non_scalar_argument",
        "cli_run_text_writes_program_stderr_to_stderr",
        "cli_run_json_reports_program_stderr_on_stdout",
        "cli_test_json_reports_per_test_stderr_on_stdout",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing stderr coverage `{required}`"
        );
    }

    for (label, text) in [
        ("command contract", contract.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("post-file selection", post_file_selection.as_str()),
    ] {
        assert!(
            text.contains("eprint") && text.contains("eprintln") && text.contains("stderr"),
            "{label} must document the scalar stderr channel"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("post-file selection", post_file_selection.as_str()),
    ] {
        assert!(
            text.contains("stdout/stderr handles"),
            "{label} must keep standard-stream handles deferred"
        );
    }

    assert!(
        contract.contains("tests[].stderr")
            && contract.contains("Text-mode `muga run` writes that captured program stderr"),
        "command-output contract must define run and test stderr reporting"
    );
    assert!(
        implementation_resume.contains("| 154. program stderr output channel |")
            && implementation_resume.contains("| runtime/prelude/typing/CLI/tests/docs | Done |")
            && implementation_resume.contains("| 155. text output file handle design |")
            && implementation_resume.contains("| docs/std/runtime/tests | Done |")
            && implementation_resume.contains("| 156. text output file handle implementation |")
            && implementation_resume
                .contains("| std_package/prelude/typing/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation queue must mark program stderr done and queue text output file handle design"
    );
}

#[test]
fn text_output_file_handle_design_is_documented() {
    let design = read("docs/text-output-file-handles.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let opaque_design = read("docs/opaque-resource-handles.md");
    let post_file_selection = read("docs/post-file-handle-resource-surface-selection.md");

    for required in [
        "Status: implemented",
        "create_text",
        "append_text",
        "write_text_to",
        "flush",
        "open_text",
        "read_text_from",
        "close",
        "One public `File` with runtime access mode",
        "Separate `TextReader` and `TextWriter` opaque types",
        "open_text(path, mode)",
        "stdout/stderr handles",
        "Read | Write | Append",
        "wrong-mode operations return `Result::Err(io::IOError)`",
        "close` attempts `flush` first",
        "only `close(file)` has `paramMode=consume`",
        "Artifact-backed `run`",
        "binary `Bytes`",
        "standard-stream handles",
        "process APIs",
        "async IO",
    ] {
        assert!(
            design.contains(required),
            "text output file handle design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("standard library review rules", stdlib_rules.as_str()),
        ("opaque resource design", opaque_design.as_str()),
        ("post-file selection", post_file_selection.as_str()),
    ] {
        assert!(
            text.contains("text-output-file-handles.md")
                && (text.contains("text output file handle")
                    || text.contains("Text output file handle")),
            "{label} must link and preserve the text output file handle design"
        );
    }

    assert!(
        implementation_resume.contains("| 155. text output file handle design |")
            && implementation_resume.contains("| docs/std/runtime/tests | Done |")
            && implementation_resume.contains("| 156. text output file handle implementation |")
            && implementation_resume
                .contains("| std_package/prelude/typing/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation queue must mark text output handle implementation done and queue integrated workflow"
    );
}

#[test]
fn text_output_file_handles_are_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let runtime = read("src/runtime.rs");
    let typing = read("src/typing.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/text-output-file-handles.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");

    for required in [
        "FS_CREATE_TEXT_BUILTIN",
        "FS_APPEND_TEXT_BUILTIN",
        "FS_WRITE_TEXT_TO_BUILTIN",
        "FS_FLUSH_BUILTIN",
        "pub fn create_text(file_path: path::Path): Result[File, io::IOError]",
        "pub fn append_text(file_path: path::Path): Result[File, io::IOError]",
        "pub fn write_text_to(file: File, text: String): Result[Unit, io::IOError]",
        "pub fn flush(file: File): Result[Unit, io::IOError]",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing text output handle item `{required}`"
        );
    }

    for required in [
        "StdFsCreateText",
        "StdFsAppendText",
        "StdFsWriteTextTo",
        "StdFsFlush",
    ] {
        assert!(prelude.contains(required), "prelude missing `{required}`");
        assert!(typing.contains(required), "typing missing `{required}`");
    }

    for required in [
        "StdFsFileMode",
        "create_std_fs_file",
        "append_std_fs_file",
        "write_std_fs_file_text",
        "flush_std_fs_file",
        "wrong_mode_io_error",
        "close_std_fs_file",
        "io::Write::flush",
        "io_error_value(\"write_text_to\"",
        "io_error_value(\"flush\"",
        "io_error_value(\"close\"",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing text output handle implementation `{required}`"
        );
    }

    for required in [
        "standard_fs_file_handle_create_write_flush_close_runs_as_virtual_package",
        "standard_fs_file_handle_append_text_preserves_existing_content",
        "standard_fs_file_handle_write_to_read_handle_returns_io_error",
        "standard_fs_file_handle_read_from_write_handle_returns_io_error",
        "standard_fs_file_handle_close_consumes_write_binding",
        "standard_fs_file_handle_artifact_run_can_write_text",
        "write_text_to function should be exported",
        "flush function should be exported",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing text output handle coverage `{required}`"
        );
    }

    assert!(
        design.contains("Status: implemented")
            && design.contains("create_text")
            && design.contains("write_text_to")
            && design.contains("stdout/stderr handles"),
        "text output handle design must record implemented scope and deferred standard streams"
    );

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
    ] {
        assert!(
            text.contains("text-output-file-handles.md")
                && text.contains("implemented")
                && text.contains("stdout/stderr handles"),
            "{label} must record implemented text output handles and deferred standard streams"
        );
    }

    assert!(
        implementation_resume.contains("| 156. text output file handle implementation |")
            && implementation_resume
                .contains("| std_package/prelude/typing/runtime/tests/docs | Done |")
            && implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation queue must mark text output handle implementation done and queue integrated workflow"
    );
}

#[test]
fn integrated_practical_report_workflow_is_implemented_and_covered() {
    let app = read("samples/projects/report_app/src/main/main.muga");
    let shared = read("samples/projects/report_shared/src/reports/main.muga");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let by_example = read("docs/muga-by-example.md");
    let v1_checklist = read("docs/v1-release-checklist.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let onboarding = read("docs/installation-and-onboarding.md");

    for required in [
        "import std::cli",
        "import std::env",
        "import std::fs",
        "import std::io",
        "fn default_output_path(): path::Path",
        "fn ensure_parent(file_path: path::Path): Result[Unit, io::IOError]",
        "fn write_report(file_path: path::Path, text: String): Result[Unit, io::IOError]",
        "fs::create_text",
        "fs::write_text_to",
        "fs::flush",
        "using file = try fs::create_text",
        "env::args()",
        "cli::positional_or(args, 0",
        "cli::positional_or(args, 1",
        "println(summary)",
        "eprintln(\"wrote report \"",
        "fn main(): Result[String, io::IOError]",
    ] {
        assert!(app.contains(required), "report app missing `{required}`");
    }

    for required in [
        "pub fn render_summary(summary: String, source_path: String): String",
        "summary: ",
        "source: ",
    ] {
        assert!(
            shared.contains(required),
            "report shared package missing `{required}`"
        );
    }

    for required in [
        "manifest_report_project_sample_runs",
        "manifest_report_project_sample_runs_against_emitted_artifacts",
        "manifest_report_project_sample_json_built_run_writes_report",
        ".arg(\"--built\")",
        ".arg(\"--format=json\")",
        "daily:ready\\\\n",
        "wrote report ",
        "Result::Ok(daily:ready|read_text|2)",
        "summary: daily:ready",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing integrated report workflow coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("Muga by example", by_example.as_str()),
        ("v1 checklist", v1_checklist.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("installation onboarding", onboarding.as_str()),
    ] {
        assert!(
            text.contains("report_app")
                && text.contains("args/env")
                && text.contains("stdout/stderr")
                && text.contains("text-file handle writes")
                && text.contains("run --built"),
            "{label} must document the integrated report workflow"
        );
    }

    assert!(
        implementation_resume.contains("| 157. integrated practical report workflow |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume.contains("| 158. post-report adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation queue must mark integrated report workflow done and queue post-report adoption selection"
    );
}

#[test]
fn lexical_resource_cleanup_design_is_documented() {
    let design = read("docs/lexical-resource-cleanup.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");

    for required in [
        "Status: implemented first slice",
        "using file = try fs::create_text(output)",
        "`using` is a statement, not an expression",
        "runtime-backed opaque handle",
        "OpaqueHandleFacts",
        "close function must return `Result[Unit, E]`",
        "same cleanup error",
        "cleanup failure wins when both the body and cleanup fail",
        "first cleanup error observed",
        "active outer cleanups are",
        "Passing the binding to its close function inside the block is rejected",
        "muga fmt",
        ".mgi",
        "artifact-backed execution and `run --built`",
        "standard-stream handles",
        "Bytes",
    ] {
        assert!(
            design.contains(required),
            "lexical cleanup design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
    ] {
        assert!(
            text.contains("lexical-resource-cleanup.md")
                && text.contains("statement-form `using`")
                && text.contains("stdout/stderr handles")
                && text.contains("Bytes"),
            "{label} must reference the selected lexical cleanup design"
        );
    }

    assert!(
        implementation_resume.contains("| 159. lexical resource cleanup design |")
            && implementation_resume.contains("| docs/parser/typing/runtime/tests | Done |")
            && implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation queue must mark lexical cleanup design done and queue implementation"
    );
}

#[test]
fn lexical_resource_cleanup_is_implemented_and_covered() {
    let token = read("src/token.rs");
    let parser = read("src/parser.rs");
    let formatter = read("src/formatter.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let mir = read("src/mir.rs");
    let bytecode = read("src/bytecode.rs");
    let examples = read("tests/examples.rs");
    let report_app = read("samples/projects/report_app/src/main/main.muga");
    let errors = read("errors.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in ["Using", "parse_using_stmt", "UsingStmt"] {
        assert!(
            token.contains(required) || parser.contains(required) || typed_hir.contains(required),
            "using implementation missing `{required}`"
        );
    }
    for required in [
        "format_expr(&stmt.value",
        "check_using_stmt",
        "resolve_using_cleanup",
        "using_cleanups",
        "compile_using_stmt",
        "emit_cleanup_call",
        "emit_cleanup_unwind_sequence",
        "emit_cleanup_call_ignoring_error",
        "emit_scope_unwind_to(chunk, 0)",
    ] {
        assert!(
            formatter.contains(required)
                || typing.contains(required)
                || mir.contains(required)
                || bytecode.contains(required),
            "lexical cleanup implementation missing `{required}`"
        );
    }
    for required in [
        "muga_fmt_formats_using_statement",
        "standard_fs_using_file_handle_writes_and_closes",
        "standard_fs_using_file_handle_closes_on_try_return",
        "standard_fs_nested_using_closes_outer_on_inner_acquisition_failure",
        "compile_bytecode_nested_using_cleanup_error_branch_unwinds_outer_cleanup",
        "using_rejects_non_handle_binding",
        "using_rejects_explicit_close_inside_block",
        "using_binding_is_not_visible_after_block",
        "standard_fs_file_handle_artifact_run_can_write_text",
    ] {
        assert!(examples.contains(required), "missing test `{required}`");
    }
    assert!(report_app.contains("using file = try fs::create_text"));
    assert!(errors.contains("T027") && errors.contains("using"));
    assert!(
        implementation_resume.contains("| 160. lexical resource cleanup implementation |")
            && implementation_resume
                .contains("| parser/formatter/typing/MIR/bytecode/runtime/tests/docs | Done |"),
        "implementation queue must mark lexical cleanup implementation done"
    );
}

#[test]
fn std_cli_first_slice_is_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let package = read("src/package.rs");
    let package_signature = read("src/package_signature.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_cli/main.muga");
    let report_app = read("samples/projects/report_app/src/main/main.muga");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let spec = read("spec/003-typing.md");
    let mini_spec = read("mini-language-spec-v1.md");

    for required in [
        "pub const CLI_PACKAGE: &str = \"std::cli\"",
        "package std::cli",
        "pub fn positional(args: List[String], index: Int): Option[String]",
        "pub fn positional_or(args: List[String], index: Int, default_value: String): String",
        "pub fn has_flag(args: List[String], name: String): Bool",
        "pub fn option(args: List[String], name: String): Option[String]",
        "pub fn option_or(args: List[String], name: String, default_value: String): String",
        "pub fn option_values(args: List[String], name: String): List[String]",
        "pub fn option_values_or(args: List[String], name: String, default_value: List[String]): List[String]",
        "pub fn positional_int(args: List[String], index: Int): Result[Option[Int], String]",
        "pub fn positional_int_or(args: List[String], index: Int, default_value: Int): Result[Int, String]",
        "pub fn option_int(args: List[String], name: String): Result[Option[Int], String]",
        "pub fn option_int_or(args: List[String], name: String, default_value: Int): Result[Int, String]",
        "pub fn positional_bool(args: List[String], index: Int): Result[Option[Bool], String]",
        "pub fn positional_bool_or(args: List[String], index: Int, default_value: Bool): Result[Bool, String]",
        "pub fn option_bool(args: List[String], name: String): Result[Option[Bool], String]",
        "pub fn option_bool_or(args: List[String], name: String, default_value: Bool): Result[Bool, String]",
    ] {
        assert!(
            std_package.contains(required),
            "std::cli package source missing `{required}`"
        );
    }

    for text in [package.as_str(), package_signature.as_str()] {
        assert!(
            text.contains("import std::cli") && text.contains("cli::..."),
            "missing std::cli import guidance"
        );
    }

    for required in [
        "package_std_cli_sample_runs",
        "standard_cli_positionals_flags_and_equal_options_run_as_virtual_package",
        "standard_cli_separate_options_and_missing_values_return_options",
        "standard_cli_double_dash_stops_option_parsing",
        "standard_cli_typed_scalar_helpers_run_as_virtual_package",
        "standard_cli_typed_scalar_helpers_report_parse_errors",
        "standard_cli_repeated_option_values_run_as_virtual_package",
        "standard_cli_typed_scalar_artifact_run_uses_emitted_std_implementations",
        "standard_cli_annotation_without_import_suggests_import",
        "standard_cli_artifact_run_uses_emitted_std_implementations",
        "artifact.txt|report.txt|fallback",
        "std__cli.mgb",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing std::cli coverage `{required}`"
        );
    }

    for required in [
        "import std::cli",
        "cli::positional_or(args, 0",
        "cli::positional_or(args, 1",
    ] {
        assert!(
            report_app.contains(required),
            "report_app missing `{required}`"
        );
    }
    assert!(
        !report_app.contains("fn arg_path_or_default"),
        "report_app should use std::cli instead of its old local argument helper"
    );

    for required in [
        "import std::cli",
        "cli::positional_or",
        "cli::has_flag",
        "cli::option_or",
        "cli::option_values",
        "cli::option_values_or",
        "cli::option_int_or",
        "cli::option_bool_or",
    ] {
        assert!(
            sample.contains(required),
            "std_cli sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("typing spec", spec.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("std::cli")
                && text.contains("Int")
                && text.contains("Bool")
                && text.contains("parse"),
            "{label} must document the typed std::cli parsing helpers"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("typing spec", spec.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("std::cli")
                && text.contains("List[String]")
                && text.contains("positional")
                && text.contains("option"),
            "{label} must document the implemented std::cli slice"
        );
    }

    assert!(
        implementation_resume.contains("| 163. first std::cli helper slice |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume.contains("| 164. post-std-cli adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 165. CLI-first app template refresh |")
            && implementation_resume.contains("| project template/tests/docs | Done |")
            && implementation_resume.contains("| 166. post-template adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 167. typed scalar std::cli parsing helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume.contains("| 168. post-typed-cli adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 169. JSON value accessor helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume.contains("| 170. post-json-accessor adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 171. JSON config workflow sample |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 172. post-config-workflow adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 173. config workflow result mapping refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 174. post-result-mapping adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 175. first std::string text assembly helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 176. post-string-assembly adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 177. JSON required object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 178. post-required-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 179. JSON composite object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 180. post-composite-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 181. nested JSON config workflow refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 182. post-nested-json-config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 183. JSON scalar array projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 184. post-json-array-projection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 185. direct JSON scalar-array object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |"),
        "implementation queue must mark std::cli and typed CLI parsing done"
    );
}

#[test]
fn cli_first_app_template_is_implemented_and_covered() {
    let project_template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let onboarding = read("docs/installation-and-onboarding.md");
    let by_example = read("docs/muga-by-example.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let checklist = read("docs/v1-release-checklist.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "import std::cli",
        "import std::env",
        "fn name_from_args(args: List[String]): String",
        "cli::option_or(args, \"name\", cli::positional_or(args, 0, \"Muga\"))",
        "name_from_args(env::args())",
        "printed = println(message)",
        "fn main(): String",
        "relative: \"README.md\"",
        "relative: \"scripts/package-app.sh\"",
        "emit-app-bundle --source-free",
        "verify-app-archive",
        "MUGA_BUNDLE_DIR",
        "MUGA_ARCHIVE_DIR",
        "MUGA_INSTALL_DIR",
        "list-installed-apps",
    ] {
        assert!(
            project_template.contains(required),
            "app template source missing `{required}`"
        );
    }

    for required in [
        "cli_new_creates_app_lib_and_test_templates",
        "hello Muga\\nhello Muga\\n",
        "hello Ada\\nhello Ada\\n",
        "hello Grace\\nhello Grace\\n",
        "hello Lin\\nhello Lin\\n",
        "Generated CLI-first app starter.",
        "scripts/package-app.sh",
        "MUGA_PROGRAM",
        "app_package_run",
        "app_install_package_run",
        "emit-app-archive",
        "verify-app-archive",
        "ready\\thello-install",
        "hello-sha256-",
        ".arg(\"fmt\")",
        ".arg(\"--check\")",
        ".arg(\"build\")",
        ".arg(\"--built\")",
        ".arg(\"--name=Grace\")",
        ".arg(\"--name\")",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing generated app coverage `{required}`"
        );
    }

    for (label, text) in [
        ("onboarding", onboarding.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("v1 checklist", checklist.as_str()),
    ] {
        assert!(
            text.contains("std::env")
                && text.contains("std::cli")
                && text.contains("hello Muga")
                && text.contains("--name"),
            "{label} must document the CLI-first generated app template"
        );
    }
    for (label, text) in [
        ("onboarding", onboarding.as_str()),
        ("Muga by Example", by_example.as_str()),
    ] {
        assert!(
            text.contains("scripts/package-app.sh") && text.contains("source-free"),
            "{label} must document the generated app package helper"
        );
    }

    assert!(
        implementation_resume.contains("| 165. CLI-first app template refresh |")
            && implementation_resume.contains("| project template/tests/docs | Done |")
            && implementation_resume.contains("| 166. post-template adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 167. typed scalar std::cli parsing helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume.contains("| 168. post-typed-cli adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 169. JSON value accessor helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume.contains("| 304. Generated app package helper |")
            && implementation_resume.contains("| templates/tests/docs | Done |")
            && implementation_resume.contains("| 170. post-json-accessor adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 171. JSON config workflow sample |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 172. post-config-workflow adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 173. config workflow result mapping refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 174. post-result-mapping adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 175. first std::string text assembly helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 176. post-string-assembly adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 177. JSON required object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 178. post-required-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 179. JSON composite object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 180. post-composite-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 181. nested JSON config workflow refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 182. post-nested-json-config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 183. JSON scalar array projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 184. post-json-array-projection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 185. direct JSON scalar-array object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |"),
        "implementation queue must mark the CLI-first app template, next selection, and typed CLI parsing done"
    );
}

#[test]
fn json_config_workflow_sample_is_implemented_and_covered() {
    let sample = read("samples/projects/config_app/src/main/main.muga");
    let manifest = read("samples/projects/config_app/muga.toml");
    let config = read("samples/projects/config_app/config/settings.json");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");

    for required in ["name = \"config_app\"", "source = \"src\""] {
        assert!(
            manifest.contains(required),
            "config app manifest missing `{required}`"
        );
    }

    for required in [
        "\"name\": \"Ada\"",
        "\"port\": 4040",
        "\"verbose\": false",
        "\"tags\": [\"tool\", \"service\"]",
        "\"owner\": {",
        "\"team\": null",
        "\"servers\": [",
        "\"port\": 9000",
        "\"limits\": {",
        "\"workers\": 4",
    ] {
        assert!(
            config.contains(required),
            "config app JSON fixture missing `{required}`"
        );
    }

    for required in [
        "import std::config",
        "import std::path",
        "import std::env",
        "import std::cli",
        "import std::result",
        "record Owner",
        "record Server",
        "record Settings",
        "tags: List[String]",
        "owner: Owner",
        "servers: List[Server]",
        "limits: Map[String, Int]",
        "fn config_error_message(error: config::Error): String",
        "fn cli_error_message(error: cli::Error): String",
        "fn settings_args(args: List[String]): List[String]",
        "result::map_err(config::load_json_or(config_path, default_settings()), config_error_message)",
        "cli::option_or(args, \"config\"",
        "cli::parse_or(settings_args(args), configured)",
        "settings.tags.len().to_string()",
        "option_string(settings.owner.name, \"unknown\")",
        "settings.servers.len().to_string()",
        "limit_or(settings.limits, \"workers\", 0)",
        "Result::Ok(rendered)",
    ] {
        assert!(
            sample.contains(required),
            "config app sample missing `{required}`"
        );
    }
    assert!(
        !sample.contains("import std::json")
            && !sample.contains("Map[String, json::Value]")
            && !sample.contains("json::at"),
        "config app sample should use structural settings instead of manual JSON helpers"
    );
    for removed in [
        "fn json_value_result",
        "fn json_string_result",
        "fn json_int_result",
        "fn json_bool_result",
    ] {
        assert!(
            !sample.contains(removed),
            "config app sample should use result::map_err instead of `{removed}`"
        );
    }

    for required in [
        "manifest_config_project_sample_runs_with_cli_overrides",
        "manifest_config_project_sample_runs_against_emitted_artifacts",
        "manifest_config_project_sample_reports_config_shape_errors",
        "manifest_config_project_sample_reports_cli_parse_errors",
        "manifest_config_project_sample_json_built_run_applies_cli_overrides",
        "Result::Ok(Grace|9090|true|1|ops|none|2|9000|4)",
        "Result::Ok(Ada|4040|false|2|ops|none|2|9000|4)",
        "Result::Err(config Decode -1: expected JSON Int at path .servers[0].port)",
        "Result::Err(cli UnknownArgument --unknown: unknown CLI option `--unknown`)",
        "Result::Ok(Kai|5050|true|1|unknown|none|1|8080|1)",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing config workflow coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
    ] {
        assert!(
            text.contains("samples/projects/config_app")
                && text.contains("JSON config")
                && text.contains("workflow")
                && text.contains("std::result::map_err")
                && (text.contains("std::fs") || text.contains("std::config"))
                && text.contains("std::json")
                && text.contains("std::cli")
                && text.contains("CLI > config > defaults"),
            "{label} must document the JSON config workflow sample"
        );
    }

    assert!(
        implementation_resume.contains("| 171. JSON config workflow sample |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 172. post-config-workflow adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 173. config workflow result mapping refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 174. post-result-mapping adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 175. first std::string text assembly helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 176. post-string-assembly adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 177. JSON required object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 178. post-required-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 179. JSON composite object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 180. post-composite-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 181. nested JSON config workflow refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 182. post-nested-json-config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 183. JSON scalar array projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 184. post-json-array-projection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 185. direct JSON scalar-array object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |"),
        "implementation queue must mark config workflow done and queue the next selection"
    );
}

#[test]
fn post_config_workflow_adoption_gap_selection_is_documented() {
    let selection = read("docs/post-config-workflow-adoption-gap-selection.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");

    for required in [
        "Status: Result error mapping refresh implemented",
        "`std::result::map_err`",
        "`samples/projects/config_app`",
        "app-boundary error normalization",
        "result::map_err(..., io_error_message)",
        "result::map_err(..., json_error_message)",
        "remove local one-off JSON result wrapper functions",
        "CLI > config > defaults",
        "`run --built --format=json`",
        "Implemented First Slice",
        "json_value_result",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Add common error unions or implicit error conversion",
        "Add `std::config` or TOML",
        "Record/schema decoding from JSON",
        "Full CLI parser schema with usage/help",
        "Formatting templates or interpolation",
        "`Bytes`, process APIs, network APIs, or streams",
    ] {
        assert!(
            selection.contains(required),
            "post-config-workflow selection missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
    ] {
        assert!(
            text.contains("post-config-workflow-adoption-gap-selection.md")
                && text.contains("std::result::map_err")
                && text.contains("app-boundary error")
                && text.contains("std::config")
                && text.contains("schema decoding")
                && text.contains("process APIs"),
            "{label} must reference the selected post-config-workflow adoption gap"
        );
    }

    assert!(
        implementation_resume.contains("| 172. post-config-workflow adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 173. config workflow result mapping refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 174. post-result-mapping adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 175. first std::string text assembly helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 176. post-string-assembly adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 177. JSON required object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 178. post-required-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 179. JSON composite object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 180. post-composite-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 181. nested JSON config workflow refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 182. post-nested-json-config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 183. JSON scalar array projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 184. post-json-array-projection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 185. direct JSON scalar-array object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |"),
        "implementation queue must mark post-config-workflow selection done and queue the result mapping refresh"
    );
}

#[test]
fn std_string_text_assembly_helpers_are_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let package = read("src/package.rs");
    let package_signature = read("src/package_signature.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_string/main.muga");
    let config_sample = read("samples/projects/config_app/src/main/main.muga");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let spec = read("spec/003-typing.md");
    let mini_spec = read("mini-language-spec-v1.md");

    for required in [
        "pub const STRING_PACKAGE: &str = \"std::string\"",
        "STRING_PACKAGE => Some(STRING_FILES)",
        "package std::string",
        "pub fn concat_all(parts: List[String]): String",
        "pub fn join(parts: List[String], separator: String): String",
        "out = out.concat(part)",
        "out = out.concat(separator).concat(part)",
    ] {
        assert!(
            std_package.contains(required),
            "std::string package source missing `{required}`"
        );
    }

    for (label, text) in [
        ("package resolver", package.as_str()),
        ("package signature", package_signature.as_str()),
    ] {
        assert!(
            text.contains("import std::string") && text.contains("string::..."),
            "{label} must suggest importing std::string for missing string alias"
        );
    }

    for required in [
        "standard_string_helpers_run_as_virtual_package",
        "standard_string_helpers_report_type_mismatches",
        "standard_string_missing_import_suggests_import",
        "standard_string_artifact_run_uses_emitted_std_implementations",
        "std__string.mgi",
        "std__string.mgb",
        "Ada|7|false",
        "config|Ada|4040",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing std::string coverage `{required}`"
        );
    }

    for required in [
        "package app::std_string",
        "import std::string",
        "empty: List[String] = []",
        "string::join(empty, \",\")",
        "8080.to_string()",
        "string::concat_all([prefix, \"config \", rendered])",
    ] {
        assert!(
            sample.contains(required),
            "std_string sample missing `{required}`"
        );
    }

    for required in [
        "import std::string",
        "string::concat_all([\"config \", config_error_kind_name(error.kind)",
        "string::join([settings.name, settings.port.to_string(), settings.verbose.to_string(), settings.tags.len().to_string(), owner, team, settings.servers.len().to_string(), first_port.to_string(), workers.to_string()], \"|\")",
        "println(string::concat_all([\"config \", rendered]))",
    ] {
        assert!(
            config_sample.contains(required),
            "config app sample missing std::string usage `{required}`"
        );
    }
    assert!(
        !config_sample.contains(".concat("),
        "config app should assemble text through std::string helpers"
    );

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("typing spec", spec.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("std::string")
                && text.contains("string::concat_all")
                && text.contains("string::join")
                && text.contains("List[String]")
                && text.contains("to_string"),
            "{label} must document the implemented std::string text assembly helpers"
        );
    }

    assert!(
        implementation_resume.contains("| 175. first std::string text assembly helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 176. post-string-assembly adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 177. JSON required object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 178. post-required-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 179. JSON composite object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 180. post-composite-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 181. nested JSON config workflow refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 182. post-nested-json-config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 183. JSON scalar array projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 184. post-json-array-projection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 185. direct JSON scalar-array object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |"),
        "implementation queue must mark std::string helpers done and queue the next selection"
    );
}

#[test]
fn std_fmt_text_layout_helpers_are_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let package = read("src/package.rs");
    let package_signature = read("src/package_signature.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fmt/main.muga");
    let design = read("docs/std-fmt-text-layout.md");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let by_example = read("docs/muga-by-example.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "pub const FMT_PACKAGE: &str = \"std::fmt\"",
        "FMT_PACKAGE => Some(FMT_FILES)",
        "package std::fmt",
        "pub fn repeat(text: String, count: Int): String",
        "pub fn pad_left(text: String, width: Int, fill: String): String",
        "pub fn pad_right(text: String, width: Int, fill: String): String",
        "pub fn truncate_chars(text: String, max_chars: Int): String",
        "pub enum FormatError",
        "MissingValue(Int)",
        "UnclosedPlaceholder(Int)",
        "UnexpectedClose(Int)",
        "pub fn format_values(template: String, values: List[String]): Result[String, FormatError]",
        "while index < count",
        "first_fill_scalar",
        "fill.slice_chars(0, 1)",
        "fn char_at(text: String, index: Int): String",
        "values.get(index)",
        "text.char_count()",
    ] {
        assert!(
            std_package.contains(required),
            "std::fmt package source missing `{required}`"
        );
    }

    for (label, text) in [
        ("package resolver", package.as_str()),
        ("package signature", package_signature.as_str()),
    ] {
        assert!(
            text.contains("import std::fmt") && text.contains("fmt::..."),
            "{label} must suggest importing std::fmt for missing fmt alias"
        );
    }

    for required in [
        "package app::std_fmt",
        "import std::fmt",
        "import std::string",
        "fmt::repeat(\"-\", 3)",
        "fmt::pad_left(\"Muga\", 7, \".\")",
        "fmt::pad_right(\"go\", 5, \"*\")",
        "fmt::truncate_chars(\"abcdef\", 4)",
        "fmt::format_values(\"{} {{ok}} {}\", [\"Muga\", \"fmt\"])",
        "string::join([ruler, left, right, clipped, empty, rendered], \"|\")",
    ] {
        assert!(
            sample.contains(required),
            "std_fmt sample missing `{required}`"
        );
    }

    for required in [
        "package_std_fmt_sample_runs",
        "standard_fmt_layout_helpers_run_as_virtual_package",
        "standard_fmt_format_values_runs_as_virtual_package",
        "standard_fmt_format_values_reports_template_errors",
        "standard_fmt_helpers_report_type_mismatches",
        "standard_fmt_missing_import_suggests_import",
        "standard_fmt_artifact_run_uses_emitted_std_implementations",
        "std__fmt.mgi",
        "std__fmt.mgb",
        "---|...Muga|go***|abcd||Muga {ok} fmt",
        "ababab|00é|猫xx|éc||Muga",
        "Hello, Ada. Use {} for placeholders.",
        "missing:1|unclosed:0|close:4",
        "ok..",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing std::fmt coverage `{required}`"
        );
    }

    for required in [
        "Status: `std::fmt` is implemented",
        "Short-term",
        "Medium-term",
        "Long-term",
        "Final goal",
        "Public Shape",
        "pub fn repeat(text: String, count: Int): String",
        "pub fn pad_left(text: String, width: Int, fill: String): String",
        "pub fn pad_right(text: String, width: Int, fill: String): String",
        "pub fn truncate_chars(text: String, max_chars: Int): String",
        "pub fn format_values(template: String, values: List[String]): Result[String, FormatError]",
        "zero or negative counts",
        "first scalar of `fill`",
        "Unicode scalar values",
        "`{{` writes a literal `{`, and `}}` writes a literal `}`",
        "Missing values, unclosed placeholders, and stray `}` braces",
        "Candidates Compared",
        "Add explicit text-layout helpers",
        "Add explicit `fmt::format_values(template, values)`",
        "Add language interpolation syntax",
        "Add mutable format builders",
        "Deferred Policy",
        "Validation",
        "std_fmt_text_layout_helpers_are_implemented_and_covered",
    ] {
        assert!(
            design.contains(required),
            "std::fmt design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("std-fmt-text-layout.md")
                || text.contains("std_fmt/main.muga")
                || text.contains("std::fmt")
                || text.contains("fmt::repeat"),
            "{label} must surface std::fmt text layout helpers"
        );
    }

    assert!(
        implementation_resume.contains("| 342. Standard formatting helpers |")
            && implementation_resume
                .contains("std::fmt::{repeat,pad_left,pad_right,truncate_chars,format_values}")
            && implementation_resume.contains("pure `std::fmt` formatting helpers"),
        "implementation queue must cover std::fmt text layout"
    );
}

#[test]
fn std_json_accessor_helpers_are_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_json/main.muga");
    let std_json_contract = read("docs/std-json-first-slice.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let rules = read("docs/standard-library-review-rules.md");
    let audit = read("docs/std-json-implementation-audit.md");
    let checklist = read("docs/v1-release-checklist.md");
    let spec = read("spec/003-typing.md");
    let mini_spec = read("mini-language-spec-v1.md");

    for required in [
        "pub fn as_bool(value: Value): Result[Bool, Error]",
        "pub fn as_string(value: Value): Result[String, Error]",
        "pub fn as_number(value: Value): Result[Number, Error]",
        "pub fn as_int(value: Value): Result[Int, Error]",
        "pub fn as_array(value: Value): Result[List[Value], Error]",
        "pub fn as_object(value: Value): Result[Map[String, Value], Error]",
        "pub fn object_get(value: Value, key: String): Result[Option[Value], Error]",
        "pub fn object_bool(value: Value, key: String): Result[Option[Bool], Error]",
        "pub fn object_bool_or(value: Value, key: String, default_value: Bool): Result[Bool, Error]",
        "pub fn object_bool_required(value: Value, key: String): Result[Bool, Error]",
        "pub fn object_string(value: Value, key: String): Result[Option[String], Error]",
        "pub fn object_string_or(value: Value, key: String, default_value: String): Result[String, Error]",
        "pub fn object_string_required(value: Value, key: String): Result[String, Error]",
        "pub fn object_int(value: Value, key: String): Result[Option[Int], Error]",
        "pub fn object_int_or(value: Value, key: String, default_value: Int): Result[Int, Error]",
        "pub fn object_int_required(value: Value, key: String): Result[Int, Error]",
    ] {
        assert!(
            std_package.contains(required),
            "std::json package source missing accessor `{required}`"
        );
    }

    for required in [
        "standard_json_value_accessors_run_as_virtual_package",
        "standard_json_required_object_fields_report_missing_errors",
        "standard_json_value_accessors_report_shape_errors",
        "standard_json_accessor_artifact_run_uses_emitted_std_implementations",
        "missing JSON object field `missing_name`",
        "expected JSON Number for object field `port`",
        "expected JSON Object",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing JSON accessor coverage `{required}`"
        );
    }

    for required in [
        "json::object_string_required",
        "json::object_int_required",
        "json::object_bool_required",
        "json::object_string_or",
        "json::encode",
    ] {
        assert!(
            sample.contains(required),
            "std_json sample missing accessor usage `{required}`"
        );
    }

    for (label, text) in [
        ("std json contract", std_json_contract.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("review rules", rules.as_str()),
        ("implementation audit", audit.as_str()),
        ("v1 checklist", checklist.as_str()),
        ("typing spec", spec.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("std::json")
                && text.contains("json::Error")
                && (text.contains("accessor") || text.contains("object-field")),
            "{label} must document JSON value accessor helpers"
        );
    }

    assert!(
        implementation_resume.contains("| 169. JSON value accessor helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume.contains("| 170. post-json-accessor adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 171. JSON config workflow sample |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 172. post-config-workflow adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 173. config workflow result mapping refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 174. post-result-mapping adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 175. first std::string text assembly helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 176. post-string-assembly adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 177. JSON required object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 178. post-required-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 179. JSON composite object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 180. post-composite-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 181. nested JSON config workflow refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 182. post-nested-json-config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 183. JSON scalar array projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 184. post-json-array-projection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 185. direct JSON scalar-array object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |"),
        "implementation queue must mark JSON accessors done and queue the next selection"
    );
}

#[test]
fn std_json_required_object_field_helpers_are_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_json/main.muga");
    let contract = read("docs/std-json-first-slice.md");
    let audit = read("docs/std-json-implementation-audit.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let spec = read("spec/003-typing.md");
    let mini_spec = read("mini-language-spec-v1.md");

    for required in [
        "fn missing_field_error(key: String): Error",
        "message: \"missing JSON object field `\".concat(key).concat(\"`\")",
        "pub fn object_string_required(value: Value, key: String): Result[String, Error]",
        "pub fn object_int_required(value: Value, key: String): Result[Int, Error]",
        "pub fn object_bool_required(value: Value, key: String): Result[Bool, Error]",
        "Result::Err(missing_field_error(key))",
    ] {
        assert!(
            std_package.contains(required),
            "std::json package source missing required-field helper `{required}`"
        );
    }

    for required in [
        "standard_json_required_object_fields_report_missing_errors",
        "standard_json_accessor_artifact_run_uses_emitted_std_implementations",
        "missing JSON object field `missing_name`",
        "missing JSON object field `missing_port`",
        "missing JSON object field `missing_enabled`",
        "expected JSON Number for object field `name`",
        "object_string_required(parsed, \"name\")",
        "object_bool_required(parsed, \"enabled\")",
        "object_int_required(parsed, \"count\")",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing required-field coverage `{required}`"
        );
    }

    for required in [
        "json::object_string_required",
        "json::object_int_required",
        "json::object_bool_required",
        "json::object_string_or",
        "default|Ada|42|true",
    ] {
        assert!(
            sample.contains(required) || examples.contains(required),
            "std_json sample or test missing `{required}`"
        );
    }

    for (label, text) in [
        ("contract", contract.as_str()),
        ("audit", audit.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("typing spec", spec.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("std::json") && text.contains("required"),
            "{label} must document std::json required object-field helpers"
        );
    }

    assert!(
        implementation_resume.contains("| 177. JSON required object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 178. post-required-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 179. JSON composite object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 180. post-composite-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 181. nested JSON config workflow refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 182. post-nested-json-config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 183. JSON scalar array projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 184. post-json-array-projection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 185. direct JSON scalar-array object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |"),
        "implementation queue must mark JSON required field helpers done and queue the next selection"
    );
}

#[test]
fn std_json_composite_object_field_helpers_are_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_json/main.muga");
    let contract = read("docs/std-json-first-slice.md");
    let audit = read("docs/std-json-implementation-audit.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let spec = read("spec/003-typing.md");
    let mini_spec = read("mini-language-spec-v1.md");

    for required in [
        "pub fn object_array(value: Value, key: String): Result[Option[List[Value]], Error]",
        "pub fn object_array_or(value: Value, key: String, default_value: List[Value]): Result[List[Value], Error]",
        "pub fn object_array_required(value: Value, key: String): Result[List[Value], Error]",
        "pub fn object_object(value: Value, key: String): Result[Option[Map[String, Value]], Error]",
        "pub fn object_object_or(value: Value, key: String, default_value: Map[String, Value]): Result[Map[String, Value], Error]",
        "pub fn object_object_required(value: Value, key: String): Result[Map[String, Value], Error]",
        "Value::Array(parsed) => Result::Ok(Option::Some(parsed))",
        "Value::Object(parsed) => Result::Ok(Option::Some(parsed))",
        "field_shape_error(key, \"Array\")",
        "field_shape_error(key, \"Object\")",
    ] {
        assert!(
            std_package.contains(required),
            "std::json package source missing composite-field helper `{required}`"
        );
    }

    for required in [
        "standard_json_composite_object_fields_run_as_virtual_package",
        "standard_json_accessor_artifact_run_uses_emitted_std_implementations",
        "object_array_required(parsed, \"items\")",
        "object_object_required(parsed, \"meta\")",
        "object_array_or(parsed, \"missing_items\"",
        "object_object_or(parsed, \"missing_meta\"",
        "missing JSON object field `missing_array`",
        "expected JSON Array for object field `meta`",
        "expected JSON Object for object field `items`",
        "Result::Ok(Ada|true|3|2|2|1|core|2)",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing composite-field coverage `{required}`"
        );
    }

    for required in [
        "json::object_array_required",
        "json::object_array_or",
        "json::object_object_required",
        "json::object_object_or",
        "2|0|1|0",
    ] {
        assert!(
            sample.contains(required) || examples.contains(required),
            "std_json sample or tests missing `{required}`"
        );
    }

    for (label, text) in [
        ("contract", contract.as_str()),
        ("audit", audit.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("typing spec", spec.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("std::json")
                && (text.contains("composite") || text.contains("array/object")),
            "{label} must document std::json composite object-field helpers"
        );
    }

    assert!(
        implementation_resume.contains("| 179. JSON composite object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 180. post-composite-json-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 181. nested JSON config workflow refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 182. post-nested-json-config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 183. JSON scalar array projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 184. post-json-array-projection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 185. direct JSON scalar-array object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |"),
        "implementation queue must mark JSON composite field helpers done and queue the next selection"
    );
}

#[test]
fn nested_json_config_workflow_refresh_is_implemented_and_covered() {
    let sample = read("samples/projects/config_app/src/main/main.muga");
    let config = read("samples/projects/config_app/config/settings.json");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let audit = read("docs/std-json-implementation-audit.md");

    for required in [
        "\"tags\": [\"tool\", \"service\"]",
        "\"owner\": {",
        "\"team\": null",
        "\"servers\": [",
        "\"limits\": {",
    ] {
        assert!(
            config.contains(required),
            "nested config fixture missing `{required}`"
        );
    }

    for required in [
        "tags: List[String]",
        "owner: Owner",
        "servers: List[Server]",
        "limits: Map[String, Int]",
        "config::load_json_or(config_path, default_settings())",
        "cli::parse_or(settings_args(args), configured)",
        "settings.tags.len().to_string()",
        "settings.servers.len().to_string()",
        "option_string(settings.owner.name, \"unknown\")",
        "first_server_port(settings.servers)",
        "limit_or(settings.limits, \"workers\", 0)",
    ] {
        assert!(
            sample.contains(required),
            "config app nested refresh missing `{required}`"
        );
    }

    for required in [
        "Result::Ok(Grace|9090|true|1|ops|none|2|9000|4)",
        "Result::Ok(Ada|4040|false|2|ops|none|2|9000|4)",
        "Result::Err(config Decode -1: expected JSON Int at path .servers[0].port)",
        "Result::Ok(Kai|5050|true|1|unknown|none|1|8080|1)",
        "config Kai|5050|true|1|unknown|none|1|8080|1",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing nested config coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("std json audit", audit.as_str()),
    ] {
        assert!(
            (text.contains("nested JSON config workflow")
                || text.contains("typed JSON config workflow"))
                && text.contains("std::json")
                && text.contains("tags")
                && text.contains("metadata")
                && text.contains("schema decoding"),
            "{label} must document the implemented nested JSON config workflow refresh"
        );
    }

    assert!(
        implementation_resume.contains("| 181. nested JSON config workflow refresh |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 182. post-nested-json-config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 183. JSON scalar array projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 184. post-json-array-projection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 185. direct JSON scalar-array object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |"),
        "implementation queue must mark nested config refresh done and queue the next selection"
    );
}

#[test]
fn std_cli_repeated_option_values_are_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let std_cli_sample = read("samples/packages/app/std_cli/main.muga");
    let config_app = read("samples/projects/config_app/src/main/main.muga");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let spec = read("spec/003-typing.md");
    let mini_spec = read("mini-language-spec-v1.md");

    for required in [
        "fn option_values_from_args(args: List[String], marker: String, equals: String): List[String]",
        "pub fn option_values(args: List[String], name: String): List[String]",
        "pub fn option_values_or(args: List[String], name: String, default_value: List[String]): List[String]",
        "if arg == \"--\" {",
        "option_value_for_arg(args, index, arg, marker, equals)",
        "out = match option_value_for_arg(args, index, arg, marker, equals)",
        "Option::Some(value) => out.push(value)",
        "values.is_empty()",
    ] {
        assert!(
            std_package.contains(required),
            "std::cli implementation missing repeated option helper evidence `{required}`"
        );
    }

    for required in [
        "standard_cli_repeated_option_values_run_as_virtual_package",
        "cli::option_values(args, \"tag\")",
        "cli::option_values_or(args, \"absent\", [\"default\"])",
        "2|tool|service|default|0|0",
        "artifact.txt|report.txt|fallback",
        "Result::Ok(input.txt|--literal|true|out.txt|2|true|2|tool|service|default)",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing repeated std::cli option coverage `{required}`"
        );
    }

    for required in [
        "cli::option_values(args, \"tag\")",
        "cli::option_values_or(args, \"missing\", [\"default\"])",
    ] {
        assert!(
            std_cli_sample.contains(required),
            "std_cli sample missing repeated option helper usage `{required}`"
        );
    }

    for required in [
        "tags: List[String]",
        "cli::parse_or(settings_args(args), configured)",
    ] {
        assert!(
            config_app.contains(required),
            "config_app missing repeated list-setting override evidence `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume", implementation_resume.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
    ] {
        assert!(
            text.contains("std::cli")
                && text.contains("option value")
                && (text.contains("schema decoding") || text.contains("full CLI parser")),
            "{label} must document repeated std::cli option values and deferred broader surfaces"
        );
    }

    for required in [
        "pub fn option_values(args: List[String], name: String): List[String]",
        "pub fn option_values_or(args: List[String], name: String, default_value: List[String]): List[String]",
        "encounter order",
        "typed repeated parsing",
    ] {
        assert!(
            spec.contains(required),
            "typing spec missing repeated std::cli option behavior `{required}`"
        );
    }

    assert!(
        mini_spec.contains("std::cli")
            && mini_spec.contains("single/repeated long-option lookup")
            && mini_spec.contains("List[String]"),
        "mini spec must mention repeated std::cli long-option lookup"
    );

    assert!(
        implementation_resume.contains("| 187. repeated CLI option value helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 188. post-repeated-cli-option adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 189. JSON path helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume.contains("| 190. post-json-path adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 191. typed JSON path scalar projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 192. post-typed-json-path-scalar adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 193. typed JSON path collection projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 194. post-typed-json-path-collection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 195. JSON schema decoding design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 196. default-overlay JSON schema decoder implementation |")
            && implementation_resume
                .contains("| typing/MIR/bytecode/runtime/std_package/tests/docs/samples | Done |"),
        "implementation queue must mark typed JSON path collection helpers done and queue the next selection"
    );
}

#[test]
fn std_json_path_helpers_are_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let std_json_sample = read("samples/packages/app/std_json/main.muga");
    let config_app = read("samples/projects/config_app/src/main/main.muga");
    let contract = read("docs/std-json-first-slice.md");
    let audit = read("docs/std-json-implementation-audit.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let spec = read("spec/003-typing.md");
    let mini_spec = read("mini-language-spec-v1.md");

    for required in [
        "pub enum PathSegment",
        "Field(String)",
        "Index(Int)",
        "fn append_path_field(path: String, key: String): String",
        "fn append_path_index(path: String, index: Int): String",
        "fn path_shape_error(path: String, expected: String): Error",
        "fn missing_path_error(path: String): Error",
        "fn render_path(path: List[PathSegment]): String",
        "fn at_from_index(value: Value, path: List[PathSegment], index: Int, rendered: String): Result[Option[Value], Error]",
        "pub fn at(value: Value, path: List[PathSegment]): Result[Option[Value], Error]",
        "pub fn at_required(value: Value, path: List[PathSegment]): Result[Value, Error]",
        "expected JSON \".concat(expected).concat(\" at path \")",
        "missing JSON value at path ",
    ] {
        assert!(
            std_package.contains(required),
            "std::json implementation missing path helper evidence `{required}`"
        );
    }

    for required in [
        "standard_json_path_helpers_run_as_virtual_package",
        "json::at_required(parsed, [json::PathSegment::Field(\"meta\")",
        "json::PathSegment::Index(0)",
        "json::at(parsed, [json::PathSegment::Field(\"meta\")",
        "expected JSON Object at path .items.bad",
        "missing JSON value at path .meta.missing",
        "Result::Ok(Ada|true|3|2|2|1|core|2)",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing JSON path helper coverage `{required}`"
        );
    }

    for required in [
        "json::at_required(parsed",
        "json::PathSegment::Field(\"meta\")",
        "json::PathSegment::Index(0)",
        "core|1",
    ] {
        assert!(
            std_json_sample.contains(required) || examples.contains(required),
            "std_json sample missing JSON path helper usage `{required}`"
        );
    }

    for removed in [
        "fn metadata_owner(settings: Settings): String",
        "json::at(metadata",
        "json::PathSegment::Field(\"owner\")",
        "owner = metadata_owner(settings)",
    ] {
        assert!(
            !config_app.contains(removed),
            "config app sample should no longer need manual JSON path helper usage `{removed}`"
        );
    }

    for (label, text) in [
        ("contract", contract.as_str()),
        ("audit", audit.as_str()),
        ("implementation resume", implementation_resume.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("typing spec", spec.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("std::json")
                && text.contains("JSON path")
                && text.contains("schema decoding"),
            "{label} must document JSON path helpers and deferred schema decoding"
        );
    }

    assert!(
        implementation_resume.contains("| 189. JSON path helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume.contains("| 190. post-json-path adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 191. typed JSON path scalar projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 192. post-typed-json-path-scalar adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 193. typed JSON path collection projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 194. post-typed-json-path-collection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 195. JSON schema decoding design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 196. default-overlay JSON schema decoder implementation |")
            && implementation_resume
                .contains("| typing/MIR/bytecode/runtime/std_package/tests/docs/samples | Done |"),
        "implementation queue must mark typed JSON path collection helpers done and queue the next selection"
    );
}

#[test]
fn std_json_path_scalar_projection_helpers_are_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let std_json_sample = read("samples/packages/app/std_json/main.muga");
    let config_app = read("samples/projects/config_app/src/main/main.muga");
    let contract = read("docs/std-json-first-slice.md");
    let audit = read("docs/std-json-implementation-audit.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let spec = read("spec/003-typing.md");
    let mini_spec = read("mini-language-spec-v1.md");

    for required in [
        "fn at_string_value(value: Value, path: List[PathSegment]): Result[String, Error]",
        "fn at_int_value(value: Value, path: List[PathSegment]): Result[Int, Error]",
        "fn at_bool_value(value: Value, path: List[PathSegment]): Result[Bool, Error]",
        "pub fn at_string(value: Value, path: List[PathSegment]): Result[Option[String], Error]",
        "pub fn at_string_or(value: Value, path: List[PathSegment], default_value: String): Result[String, Error]",
        "pub fn at_string_required(value: Value, path: List[PathSegment]): Result[String, Error]",
        "pub fn at_int(value: Value, path: List[PathSegment]): Result[Option[Int], Error]",
        "pub fn at_int_or(value: Value, path: List[PathSegment], default_value: Int): Result[Int, Error]",
        "pub fn at_int_required(value: Value, path: List[PathSegment]): Result[Int, Error]",
        "pub fn at_bool(value: Value, path: List[PathSegment]): Result[Option[Bool], Error]",
        "pub fn at_bool_or(value: Value, path: List[PathSegment], default_value: Bool): Result[Bool, Error]",
        "pub fn at_bool_required(value: Value, path: List[PathSegment]): Result[Bool, Error]",
        "path_shape_error(render_path(path), \"String\")",
        "path_shape_error(render_path(path), \"Int\")",
        "path_shape_error(render_path(path), \"Bool\")",
    ] {
        assert!(
            std_package.contains(required),
            "std::json implementation missing typed path scalar helper evidence `{required}`"
        );
    }

    for required in [
        "standard_json_path_scalar_helpers_run_as_virtual_package",
        "json::at_string_required(parsed",
        "json::at_string_or(parsed",
        "json::at_int_required(parsed",
        "json::at_bool_required(parsed",
        "json::at_string(parsed",
        "json::at_int(parsed",
        "expected JSON String at path .meta.count",
        "expected JSON Int at path .meta.ratio",
        "expected JSON Object at path .items.bad",
        "Result::Ok(core|fallback|3|true|missing|missing|",
        "Result::Ok(Ada|true|3|2|2|1|core|2)",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing typed JSON path scalar coverage `{required}`"
        );
    }

    for required in [
        "json::at_string_required(parsed",
        "json::at_int_required(parsed",
    ] {
        assert!(
            std_json_sample.contains(required),
            "std_json sample missing typed path scalar usage `{required}`"
        );
    }

    for removed in [
        "fn metadata_owner(settings: Settings): String",
        "json::at_string_or(metadata",
        "json::PathSegment::Field(\"owner\")",
    ] {
        assert!(
            !config_app.contains(removed),
            "config app sample should no longer need manual typed path scalar usage `{removed}`"
        );
    }

    for (label, text) in [
        ("contract", contract.as_str()),
        ("audit", audit.as_str()),
        ("implementation resume", implementation_resume.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("typing spec", spec.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("std::json")
                && text.contains("typed JSON path scalar")
                && text.contains("schema decoding"),
            "{label} must document typed JSON path scalar helpers and deferred schema decoding"
        );
    }

    assert!(
        implementation_resume.contains("| 191. typed JSON path scalar projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 192. post-typed-json-path-scalar adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 193. typed JSON path collection projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 194. post-typed-json-path-collection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 195. JSON schema decoding design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 196. default-overlay JSON schema decoder implementation |")
            && implementation_resume
                .contains("| typing/MIR/bytecode/runtime/std_package/tests/docs/samples | Done |"),
        "implementation queue must mark typed JSON path collection helpers done and queue the next selection"
    );
}

#[test]
fn std_json_path_collection_projection_helpers_are_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let std_json_sample = read("samples/packages/app/std_json/main.muga");
    let contract = read("docs/std-json-first-slice.md");
    let audit = read("docs/std-json-implementation-audit.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let spec = read("spec/003-typing.md");
    let mini_spec = read("mini-language-spec-v1.md");

    for required in [
        "fn at_array_value(value: Value, path: List[PathSegment]): Result[List[Value], Error]",
        "fn at_object_value(value: Value, path: List[PathSegment]): Result[Map[String, Value], Error]",
        "fn at_string_array_value(value: Value, path: List[PathSegment]): Result[List[String], Error]",
        "fn at_int_array_value(value: Value, path: List[PathSegment]): Result[List[Int], Error]",
        "fn at_bool_array_value(value: Value, path: List[PathSegment]): Result[List[Bool], Error]",
        "pub fn at_array(value: Value, path: List[PathSegment]): Result[Option[List[Value]], Error]",
        "pub fn at_array_or(value: Value, path: List[PathSegment], default_value: List[Value]): Result[List[Value], Error]",
        "pub fn at_array_required(value: Value, path: List[PathSegment]): Result[List[Value], Error]",
        "pub fn at_object(value: Value, path: List[PathSegment]): Result[Option[Map[String, Value]], Error]",
        "pub fn at_object_or(value: Value, path: List[PathSegment], default_value: Map[String, Value]): Result[Map[String, Value], Error]",
        "pub fn at_object_required(value: Value, path: List[PathSegment]): Result[Map[String, Value], Error]",
        "pub fn at_string_array(value: Value, path: List[PathSegment]): Result[Option[List[String]], Error]",
        "pub fn at_string_array_or(value: Value, path: List[PathSegment], default_value: List[String]): Result[List[String], Error]",
        "pub fn at_string_array_required(value: Value, path: List[PathSegment]): Result[List[String], Error]",
        "pub fn at_int_array(value: Value, path: List[PathSegment]): Result[Option[List[Int]], Error]",
        "pub fn at_int_array_or(value: Value, path: List[PathSegment], default_value: List[Int]): Result[List[Int], Error]",
        "pub fn at_int_array_required(value: Value, path: List[PathSegment]): Result[List[Int], Error]",
        "pub fn at_bool_array(value: Value, path: List[PathSegment]): Result[Option[List[Bool]], Error]",
        "pub fn at_bool_array_or(value: Value, path: List[PathSegment], default_value: List[Bool]): Result[List[Bool], Error]",
        "pub fn at_bool_array_required(value: Value, path: List[PathSegment]): Result[List[Bool], Error]",
        "path_shape_error(render_path(path), \"Array\")",
        "path_shape_error(render_path(path), \"Object\")",
        "path.push(PathSegment::Index(index))",
    ] {
        assert!(
            std_package.contains(required),
            "std::json package source missing collection path helper `{required}`"
        );
    }

    for required in [
        "standard_json_path_collection_helpers_run_as_virtual_package",
        "json::at_string_array_required(parsed",
        "json::at_string_array_or(parsed",
        "json::at_int_array_required(parsed",
        "json::at_bool_array_required(parsed",
        "json::at_object_required(parsed",
        "json::at_array_required(parsed",
        "json::at_array_or(parsed",
        "json::at_string_array(parsed",
        "json::at_array(parsed",
        "expected JSON Array at path .meta.obj",
        "expected JSON String at path .meta.bad_tags[1]",
        "expected JSON Int at path .meta.ratio_counts[1]",
        "expected JSON Object at path .items.bad",
        "Result::Ok(2|1|2|2|1|1|0|missing|missing|",
        "Result::Ok(Ada|true|3|2|2|1|core|2)",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing typed JSON path collection coverage `{required}`"
        );
    }

    for required in [
        "json::at_string_array_required(parsed",
        "json::PathSegment::Field(\"labels\")",
        "Result::Ok(default|Ada|42|true|2|2|0|2|0|core|1|2|",
    ] {
        assert!(
            std_json_sample.contains(required) || examples.contains(required),
            "std_json sample or examples missing collection path usage `{required}`"
        );
    }

    for (label, text) in [
        ("contract", contract.as_str()),
        ("audit", audit.as_str()),
        ("implementation resume", implementation_resume.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("typing spec", spec.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("std::json")
                && text.contains("typed JSON path collection")
                && text.contains("schema decoding"),
            "{label} must document typed JSON path collection helpers and deferred schema decoding"
        );
    }

    assert!(
        implementation_resume.contains("| 193. typed JSON path collection projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 194. post-typed-json-path-collection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 195. JSON schema decoding design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 196. default-overlay JSON schema decoder implementation |")
            && implementation_resume
                .contains("| typing/MIR/bytecode/runtime/std_package/tests/docs/samples | Done |"),
        "implementation queue must mark typed JSON path collection helpers done and queue the next selection"
    );
}

#[test]
fn json_schema_decoding_design_is_documented() {
    let design = read("docs/json-schema-decoding.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let audit = read("docs/std-json-implementation-audit.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");

    for required in [
        "Status: default-overlay JSON schema decoder implemented",
        "json::decode_or[T](value, fallback)",
        "pub fn decode_or[T](value: json::Value, fallback: T): Result[T, json::Error]",
        "pub fn decode[T](value: json::Value): Result[T, json::Error]",
        "compiler-recognized",
        "Supported First Target Types",
        "Map[String, json::Value]",
        "records whose fields recursively use only the supported first target types",
        "public shape is available to the package-aware checker",
        "unknown JSON object fields are ignored",
        "expected JSON Object at path <root>",
        "expected JSON String at path .tags[1]",
        "Schema Source",
        "loaded `.mgi` public interfaces",
        "serializable decoder schema",
        "Runtime reflective builtin",
        "Generated source helper functions",
        "Decoder builder API",
        "`std::config` with JSON/TOML discovery",
        "Non-Goals",
        "Implementation Plan",
        "config_app",
    ] {
        assert!(
            design.contains(required),
            "JSON schema decoding design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("implementation resume", implementation_resume.as_str()),
        ("std json audit", audit.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
    ] {
        assert!(
            text.contains("json-schema-decoding.md")
                && text.contains("json::decode_or[T](value, fallback)")
                && text.contains("std::config"),
            "{label} must reference the JSON schema decoding design"
        );
    }

    assert!(
        implementation_resume.contains("| 195. JSON schema decoding design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 196. default-overlay JSON schema decoder implementation |")
            && implementation_resume
                .contains("| typing/MIR/bytecode/runtime/std_package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 197. post-json-schema-decoder adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 198. `std::config` JSON default loading design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 199. `std::config` JSON default loader implementation |")
            && implementation_resume.contains(
                "| std package/typing/MIR/bytecode/runtime/artifacts/tests/docs/samples | Done |"
            ),
        "implementation queue must mark JSON schema decoding design done and queue decode_or implementation"
    );
}

#[test]
fn json_schema_decoder_implementation_is_covered() {
    let std_package = read("src/std_package.rs");
    let json_decode = read("src/json_decode.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let mir = read("src/mir.rs");
    let bytecode = read("src/bytecode.rs");
    let runtime = read("src/runtime.rs");
    let implementation_artifact = read("src/implementation_artifact.rs");
    let examples = read("tests/examples.rs");
    let config_app = read("samples/projects/config_app/src/main/main.muga");
    let design = read("docs/json-schema-decoding.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");

    for required in [
        "pub fn decode_or[T](value: Value, fallback: T): Result[T, Error]",
        "json::decode_or requires compiler schema lowering",
    ] {
        assert!(
            std_package.contains(required),
            "std::json package missing `{required}`"
        );
    }

    for required in [
        "pub enum JsonDecodeSchema",
        "JsonObjectMap",
        "artifact_text",
        "from_artifact_text",
    ] {
        assert!(
            json_decode.contains(required),
            "json decode schema module missing `{required}`"
        );
    }

    for required in [
        "TypedJsonDecodeSchemaInfo",
        "std_json_decode_or_bindings",
        "check_std_json_decode_or_call",
        "json_decode_schema_for_type",
        "Map[String, json::Value]",
    ] {
        assert!(
            typing.contains(required),
            "typechecker missing decode_or implementation evidence `{required}`"
        );
    }

    assert!(
        typed_hir.contains("json_decode_schema: Option<JsonDecodeSchema>")
            && typed_hir.contains("json_decode_schema_for_call"),
        "typed HIR must carry decode_or schemas"
    );
    assert!(
        mir.contains("JsonDecodeOr")
            && mir.contains("lower_json_decode_schema")
            && mir.contains("package_item_symbol"),
        "MIR lowering must carry decode_or schemas and package record names"
    );
    assert!(
        bytecode.contains("DecodeJson") && bytecode.contains("schema.map_symbols"),
        "bytecode must carry and merge decode_or schemas"
    );
    assert!(
        implementation_artifact.contains("\"DecodeJson\"")
            && implementation_artifact.contains("from_artifact_text")
            && implementation_artifact.contains("validate_symbols"),
        "implementation artifacts must persist decode_or schemas"
    );
    assert!(
        runtime.contains("decode_json_value")
            && runtime.contains("decode_json_record")
            && runtime.contains("expected JSON {expected} at path"),
        "runtime must decode schema payloads with path diagnostics"
    );

    for required in [
        "standard_json_decode_or_record_overlay_runs",
        "standard_json_decode_or_reports_nested_path_errors",
        "standard_json_decode_or_rejects_unsupported_targets",
        "standard_json_decode_or_artifact_run_uses_schema_payload",
        "expected JSON String at path .tags[1]",
    ] {
        assert!(examples.contains(required), "examples missing `{required}`");
    }
    assert!(
        config_app.contains("config::load_json_or(config_path, default_settings())")
            && !config_app.contains("fn settings_from_config"),
        "config_app must use compiler-owned schema decoding instead of hand-written field extraction"
    );

    for required in [
        "Status: default-overlay JSON schema decoder implemented",
        "Done: lower `decode_or` calls with schema payloads",
        "Done: implement runtime decoding for the supported first target set",
        "Next: audit adoption before adding required `decode[T]`",
    ] {
        assert!(design.contains(required), "design missing `{required}`");
    }

    assert!(
        implementation_resume
            .contains("| 196. default-overlay JSON schema decoder implementation |")
            && implementation_resume
                .contains("| typing/MIR/bytecode/runtime/std_package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 197. post-json-schema-decoder adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 198. `std::config` JSON default loading design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 199. `std::config` JSON default loader implementation |")
            && implementation_resume.contains(
                "| std package/typing/MIR/bytecode/runtime/artifacts/tests/docs/samples | Done |"
            ),
        "implementation queue must mark decode_or implementation done and queue adoption audit"
    );

    for (label, text) in [
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("strategy", strategy.as_str()),
    ] {
        assert!(
            text.contains("json::decode_or[T](value, fallback)")
                && text.contains("implemented")
                && text.contains("std::config"),
            "{label} must document implemented decode_or and remaining config deferrals"
        );
    }
}

#[test]
fn std_config_json_loading_design_is_documented() {
    let design = read("docs/std-config-json-loading.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let audit = read("docs/std-json-implementation-audit.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");

    for required in [
        "Status: std::config JSON default loader implemented",
        "pub fn load_json_or[T](file_path: path::Path, fallback: T): Result[T, Error]",
        "pub fn load_json[T](file_path: path::Path): Result[T, Error]",
        "pub enum ErrorKind",
        "Read",
        "Parse",
        "Decode",
        "pub record Error",
        "raw_code: Option[Int]",
        "json::decode_or[T](value, fallback)",
        "CLI > config > defaults",
        "compiler-recognized at direct call sites",
        "reuses the `json::decode_or[T]` schema machinery",
        "serializable decoder schema",
        "emitted `.mgb` artifacts",
        "Supported Target Types",
        "Map[String, json::Value]",
        "expected JSON String at path .tags[1]",
        "config::load_json_or[T](path, fallback): Result[T, config::Error]",
        "Return nested source errors",
        "Ambient discovery",
        "TOML support",
        "Required `json::decode[T]`",
        "Non-Goals",
        "Implementation Plan",
        "Done: add `std::config` to the virtual std package set",
        "Done: add `config::load_json[T]` with required decode semantics",
        "Next: keep TOML, config discovery, process APIs, network APIs, streams",
    ] {
        assert!(
            design.contains(required),
            "std::config JSON loading design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("implementation resume", implementation_resume.as_str()),
        ("std json audit", audit.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
    ] {
        assert!(
            text.contains("std-config-json-loading.md")
                && text.contains("std::config")
                && text.contains("config::load_json_or"),
            "{label} must reference the selected std::config JSON loading design"
        );
    }

    assert!(
        implementation_resume.contains("| 198. `std::config` JSON default loading design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 199. `std::config` JSON default loader implementation |")
            && implementation_resume.contains(
                "| std package/typing/MIR/bytecode/runtime/artifacts/tests/docs/samples | Done |"
            ),
        "implementation queue must mark std::config design done and queue implementation"
    );
}

#[test]
fn std_config_json_loader_implementation_is_covered() {
    let std_package = read("src/std_package.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let mir = read("src/mir.rs");
    let bytecode = read("src/bytecode.rs");
    let runtime = read("src/runtime.rs");
    let implementation_artifact = read("src/implementation_artifact.rs");
    let examples = read("tests/examples.rs");
    let config_app = read("samples/projects/config_app/src/main/main.muga");
    let std_config_sample = read("samples/packages/app/std_config/main.muga");
    let design = read("docs/std-config-json-loading.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let audit = read("docs/std-json-implementation-audit.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");

    for required in [
        "pub enum ErrorKind",
        "pub record Error",
        "pub fn load_json_or[T](file_path: path::Path, fallback: T): Result[T, Error]",
        "pub fn load_json[T](file_path: path::Path): Result[T, Error]",
        "config::load_json_or requires compiler schema lowering",
        "config::load_json requires compiler schema lowering",
    ] {
        assert!(
            std_package.contains(required),
            "std::config package missing `{required}`"
        );
    }

    for required in [
        "std_config_load_json_or_bindings",
        "std_config_load_json_bindings",
        "check_std_config_load_json_or_call",
        "check_std_config_load_json_call",
        "config_load_json_schemas",
        "config_required_load_json_schemas",
        "config::load_json_or",
        "config::load_json",
    ] {
        assert!(
            typing.contains(required),
            "typechecker missing config loader implementation evidence `{required}`"
        );
    }

    assert!(
        typed_hir.contains("config_load_json_schema: Option<JsonDecodeSchema>")
            && typed_hir
                .contains("config_required_load_json_schema: Option<Box<JsonDecodeSchema>>")
            && typed_hir.contains("config_load_json_schema_for_call"),
        "typed HIR must carry config loader schemas"
    );
    assert!(
        mir.contains("ConfigLoadJsonOr")
            && mir.contains("ConfigLoadJson")
            && mir.contains("config_required_load_json_schema")
            && mir.contains("config_load_json_schema"),
        "MIR lowering must carry config loader schemas"
    );
    assert!(
        bytecode.contains("LoadJsonConfig")
            && bytecode.contains("LoadJsonConfigRequired")
            && bytecode.contains("schema.map_symbols"),
        "bytecode must carry and merge config loader schemas"
    );
    assert!(
        implementation_artifact.contains("\"LoadJsonConfig\"")
            && implementation_artifact.contains("\"LoadJsonConfigRequired\"")
            && implementation_artifact.contains("invalid JSON config decoder schema")
            && implementation_artifact.contains("invalid required JSON config decoder schema")
            && implementation_artifact.contains("JSON config decoder schema"),
        "implementation artifacts must persist and validate config loader schemas"
    );
    assert!(
        runtime.contains("ConfigErrorKind")
            && runtime.contains("config_error_value")
            && runtime.contains("expect_path_value")
            && runtime.contains("JsonParser::new(&text).parse()")
            && runtime.contains("decode_json_value(program, schema"),
        "runtime must implement read/parse/decode config loading"
    );
    assert!(
        runtime.contains("decode_json_value_required(program, schema"),
        "runtime must implement read/parse/decode config loading"
    );

    for required in [
        "standard_config_load_json_or_record_runs",
        "standard_config_load_json_or_reports_decode_path_errors",
        "standard_config_load_json_or_reports_parse_errors",
        "standard_config_load_json_or_rejects_unsupported_targets",
        "standard_config_load_json_or_artifact_run_uses_schema_payload",
        "standard_config_load_json_record_runs",
        "standard_config_load_json_reports_required_decode_errors",
        "standard_config_load_json_rejects_unsupported_targets",
        "standard_config_load_json_artifact_run_uses_schema_payload",
        "package_std_config_sample_runs",
        "package_std_config_sample_runs_against_emitted_artifacts",
        "Result::Ok(Ada|9090)",
        "Result::Ok(Ada|api|9090|2|prod || ops|localhost|8080|1|dev)",
        "Decode|-1|missing required JSON field at path .port",
        "LoadJsonConfigRequired",
        "manifest_config_project_sample_reports_config_shape_errors",
        "Result::Err(config Decode -1: expected JSON Int at path .servers[0].port)",
    ] {
        assert!(examples.contains(required), "examples missing `{required}`");
    }

    assert!(
        config_app.contains("import std::config")
            && config_app.contains("config::load_json_or(config_path, default_settings())")
            && !config_app.contains("fn read_config")
            && !config_app.contains("fs::read_text_path")
            && !config_app.contains("import std::json"),
        "config_app must use std::config loader instead of local read/parse/decode plumbing"
    );

    assert!(
        std_config_sample.contains("config::load_json(path::from_string")
            && std_config_sample.contains("config::load_json_or(path::from_string")
            && std_config_sample.contains("samples/packages/app/std_config/required.json")
            && std_config_sample.contains("samples/packages/app/std_config/overlay.json"),
        "std::config package sample must demonstrate required and default-overlay loaders"
    );

    for required in [
        "Status: std::config JSON default loader implemented",
        "Done: add `std::config` to the virtual std package set",
        "Done: reuse `JsonDecodeSchema` validation",
        "Done: implement runtime text read, JSON parse, schema decode",
        "Done: persist and validate the new bytecode instruction",
        "Done: refresh `samples/projects/config_app`",
        "Done: add `config::load_json[T]` with required decode semantics",
        "LoadJsonConfigRequired",
        "Next: keep TOML, config discovery, process APIs, network APIs, streams",
    ] {
        assert!(design.contains(required), "design missing `{required}`");
    }

    assert!(
        implementation_resume.contains("| 199. `std::config` JSON default loader implementation |")
            && implementation_resume.contains(
                "| std package/typing/MIR/bytecode/runtime/artifacts/tests/docs/samples | Done |"
            )
            && implementation_resume
                .contains("| 200. post-std-config-json-loader adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 350. Required JSON config loader |")
            && implementation_resume.contains("std::config::load_json[T](path)")
            && implementation_resume.contains("LoadJsonConfigRequired")
            && implementation_resume.contains("| 351. std::config package sample adoption |"),
        "implementation queue must mark std::config implementation done and queue adoption audit"
    );

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("std json audit", audit.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
    ] {
        assert!(
            text.contains("std-config-json-loading.md")
                && text.contains("std::config")
                && text.contains("load_json_or")
                && text.contains("load_json")
                && text.contains("implemented"),
            "{label} must document the implemented std::config JSON loader"
        );
    }
}

#[test]
fn generated_config_app_template_is_implemented_and_covered() {
    let project_template = read("src/project_template.rs");
    let cli = read("src/main.rs");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let onboarding = read("docs/installation-and-onboarding.md");
    let by_example = read("docs/muga-by-example.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let contract = read("docs/diagnostics-and-output.md");

    for required in [
        "ProjectTemplate::ConfigApp",
        "config/settings.json",
        "import std::config",
        "record Owner",
        "record Server",
        "record Settings",
        "owner: Owner",
        "servers: List[Server]",
        "limits: Map[String, Int]",
        "config_error_message(error: config::Error)",
        "config::load_json_or(config_path, default_settings())",
        "cli::parse_or(settings_args(args), configured)",
        "option_string(settings.owner.name, \"unknown\")",
        "limit_or(settings.limits, \"workers\", 0)",
        "\"team\": null",
        "\"workers\": 4",
    ] {
        assert!(
            project_template.contains(required),
            "config app template implementation missing `{required}`"
        );
    }
    assert!(
        !project_template.contains("import std::json")
            && !project_template.contains("Map[String, json::Value]")
            && !project_template.contains("json::at"),
        "config app template should use structural settings instead of manual JSON helpers"
    );

    for required in [
        "\"config-app\" | \"config_app\" | \"config\"",
        "muga new [--template app|lib|test|config-app|cli-tool|report-app|resource-export|package-app] <project-dir>",
        "app lib test config-app",
        "expected `app`, `lib`, `test`, `config-app`, `cli-tool`, `report-app`, `resource-export`, or `package-app`",
    ] {
        assert!(cli.contains(required), "CLI missing `{required}`");
    }

    for required in [
        "--template=config-app",
        "config/settings.json",
        "config::load_json_or(config_path, default_settings())",
        "cli::parse_or(settings_args(args), configured)",
        "config Grace|9090|true|1|ops|none|2|9000|4",
        "Result::Ok(Ada|5050|false|1|ops|none|2|9000|4)",
        "config_build",
    ] {
        assert!(
            examples.contains(required),
            "examples test suite missing generated config app coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("installation onboarding", onboarding.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("command-output contract", contract.as_str()),
    ] {
        assert!(
            text.contains("config-app")
                && text.contains("std::config")
                && text.contains("load_json_or"),
            "{label} must document the generated config app template"
        );
    }

    assert!(
        implementation_resume.contains("| 201. generated config app template |")
            && implementation_resume.contains("| project template/CLI/tests/docs | Done |")
            && implementation_resume
                .contains("| 202. post-generated-config-app-template adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 203. required `json::decode[T]` design |")
            && implementation_resume.contains("Design strict `json::decode[T](value)`"),
        "implementation queue must mark generated config app template done and choose the required decoder design"
    );
}

#[test]
fn required_json_decoding_design_is_documented() {
    let design = read("docs/json-required-decoding.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let std_json_audit = read("docs/std-json-implementation-audit.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");

    for required in [
        "Status: required JSON decoder implemented",
        "pub fn decode[T](value: json::Value): Result[T, json::Error]",
        "Type Target Policy",
        "expected return type",
        "decoded: Result[Settings, json::Error] = json::decode(value)",
        "settings: Settings = try json::decode(value)",
        "If the expected type is absent",
        "Supported Targets",
        "concrete non-generic records",
        "Decode Semantics",
        "Record decoding is strict",
        "missing fields return `json::Error`",
        "json::ErrorKind::UnexpectedToken",
        "unknown object fields remain ignored",
        "Lowering And Artifacts",
        "typing records required decoder schemas separately",
        "bytecode emits a new required JSON decode instruction",
        "Implementation Status",
        "DecodeJsonRequired",
        "Required Coverage",
        "unsupported-target rejecting tests",
        "inference diagnostics for unannotated calls",
        "artifact-backed execution and `run --built` coverage",
        "Non-Goals",
        "Implementation Plan",
        "Done: add `json::decode[T]`",
        "Next: audit adoption",
    ] {
        assert!(
            design.contains(required),
            "required JSON decoder design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("implementation resume", implementation_resume.as_str()),
        ("std json audit", std_json_audit.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
    ] {
        assert!(
            text.contains("json-required-decoding.md")
                && text.contains("json::decode[T](value)")
                && text.contains("json::Error"),
            "{label} must reference the required JSON decoder design"
        );
    }

    assert!(
        implementation_resume.contains("| 203. required `json::decode[T]` design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 204. required `json::decode[T]` implementation |")
            && implementation_resume.contains(
                "| std package/typing/HIR/MIR/bytecode/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 205. post-required-json-decoder adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 206. broader JSON decoder target design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 207. structural JSON decoder target implementation |")
            && implementation_resume
                .contains("| json_decode/typing/runtime/artifacts/tests/docs | Done |"),
        "implementation queue must mark required JSON decoder implementation done and queue broader decoder target design"
    );
}

#[test]
fn required_json_decoder_implementation_is_covered() {
    let std_package = read("src/std_package.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let mir = read("src/mir.rs");
    let bytecode = read("src/bytecode.rs");
    let runtime = read("src/runtime.rs");
    let artifact = read("src/implementation_artifact.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/json-required-decoding.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let std_json_audit = read("docs/std-json-implementation-audit.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");

    for required in [
        "pub fn decode[T](value: Value): Result[T, Error]",
        "json::decode requires compiler schema lowering",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing required json decoder evidence `{required}`"
        );
    }

    for required in [
        "std_json_decode_bindings",
        "json_required_decode_schemas",
        "check_std_json_decode_call",
        "type annotation required because `json::decode` has no fallback value",
        "Result[T, json::Error]",
        "json::decode",
    ] {
        assert!(
            typing.contains(required),
            "typing missing required json decoder evidence `{required}`"
        );
    }

    for required in [
        "json_required_decode_schema",
        "json_required_decode_schemas",
    ] {
        assert!(
            typed_hir.contains(required),
            "typed HIR missing required json decoder evidence `{required}`"
        );
    }

    for required in ["JsonDecode(JsonDecodeExpr)", "json_required_decode_schema"] {
        assert!(
            mir.contains(required),
            "MIR missing required json decoder evidence `{required}`"
        );
    }

    assert!(
        bytecode.contains("DecodeJsonRequired"),
        "bytecode must carry a distinct required JSON decode instruction"
    );

    for required in [
        "DecodeJsonRequired",
        "invalid required JSON decoder schema",
        "required JSON decoder schema",
    ] {
        assert!(
            artifact.contains(required),
            "implementation artifact missing required json decoder evidence `{required}`"
        );
    }

    for required in [
        "DecodeJsonRequired",
        "decode_json_value_required",
        "decode_json_record_required",
        "missing required JSON field at path",
        "decode_json_value_required(program, &field.schema",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing required json decoder evidence `{required}`"
        );
    }

    for required in [
        "standard_json_decode_required_record_runs",
        "standard_json_decode_reports_missing_required_field_path",
        "standard_json_decode_requires_expected_target",
        "standard_json_decode_rejects_unsupported_targets",
        "standard_json_decode_requires_json_error_boundary",
        "standard_json_decode_artifact_run_uses_schema_payload",
        "fn decode_settings(value: json::Value): Result[Settings, json::Error]",
        "settings: Settings = try json::decode(parsed)",
        "message(json::decode(parsed))",
        "Result::Ok(missing required JSON field at path .port)",
        ".arg(\"--built\")",
    ] {
        assert!(
            examples.contains(required),
            "examples missing required json decoder coverage `{required}`"
        );
    }

    for required in [
        "Status: required JSON decoder implemented",
        "json::decode[T](value)",
        "Implementation Status",
        "DecodeJsonRequired",
        "Done: add `json::decode[T]`",
        "Next: audit adoption",
    ] {
        assert!(
            design.contains(required),
            "required JSON decoder docs missing implementation evidence `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("implementation resume", implementation_resume.as_str()),
        ("std json audit", std_json_audit.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
    ] {
        assert!(
            text.contains("json::decode[T](value)") && text.contains("json-required-decoding.md"),
            "{label} must document the implemented required JSON decoder"
        );
    }

    assert!(
        mini_spec.contains("json::decode[T](value)") && mini_spec.contains("json::decode_or[T]"),
        "mini spec must document the implemented required JSON decoder surface"
    );

    assert!(
        implementation_resume.contains("| 204. required `json::decode[T]` implementation |")
            && implementation_resume.contains(
                "| std package/typing/HIR/MIR/bytecode/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 205. post-required-json-decoder adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 206. broader JSON decoder target design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 207. structural JSON decoder target implementation |")
            && implementation_resume
                .contains("| json_decode/typing/runtime/artifacts/tests/docs | Done |"),
        "implementation queue must mark required JSON decoder implementation done and queue broader decoder target design"
    );
}

#[test]
fn json_decoder_target_expansion_design_is_documented() {
    let design = read("docs/json-decoder-target-expansion.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let std_json_audit = read("docs/std-json-implementation-audit.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");

    for required in [
        "Status: structural and concrete enum JSON decoder targets implemented",
        "Option[T]",
        "recursive `List[T]`",
        "typed `Map[String, T]`",
        "json::decode_or[T]",
        "json::decode[T]",
        "config::load_json_or[T]",
        "concrete non-generic user enums",
        "Current Boundary",
        "Selected First Expansion",
        "Option Semantics",
        "Missing record field",
        "Present `null`",
        "fallback field value",
        "Option[Option[T]]",
        "List Semantics",
        "Typed Map Semantics",
        "Enum Semantics",
        "zero-payload variants decode from a string tag",
        "one-payload variants decode from a single-key object",
        "Lowering And Artifacts",
        "O <schema>",
        "L <schema>",
        "MT <schema>",
        "E <type_symbol>",
        "Implementation Result",
        "Diagnostics",
        "Required Coverage",
        "Non-Goals",
        "Implementation Plan",
        "Done: design and implement JSON/config schema polish",
    ] {
        assert!(
            design.contains(required),
            "JSON decoder target expansion design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("implementation resume", implementation_resume.as_str()),
        ("std json audit", std_json_audit.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
    ] {
        assert!(
            text.contains("json-decoder-target-expansion.md")
                && text.contains("Option[T]")
                && text.contains("Map[String, T]"),
            "{label} must reference the selected JSON decoder target expansion design"
        );
    }

    assert!(
        implementation_resume.contains("| 206. broader JSON decoder target design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 207. structural JSON decoder target implementation |")
            && implementation_resume
                .contains("| json_decode/typing/runtime/artifacts/tests/docs | Done |")
            && implementation_resume
                .contains("| 208. post-structural-json-decoder adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 209. structural config workflow refresh |")
            && implementation_resume.contains("| project template/samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 210. post-structural-config-workflow adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 211. enum JSON/config decoder implementation |")
            && implementation_resume
                .contains("| json_decode/typing/runtime/artifacts/tests/docs | Done |")
            && implementation_resume
                .contains("| 212. post-enum JSON/config decoder adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 213. JSON/config schema polish design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 214. JSON/config field and variant rename implementation |")
            && implementation_resume.contains(
                "| parser/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            ),
        "implementation queue must mark rename implementation done"
    );
}

#[test]
fn structural_json_decoder_target_implementation_is_covered() {
    let json_decode = read("src/json_decode.rs");
    let typing = read("src/typing.rs");
    let mir = read("src/mir.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/json-decoder-target-expansion.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let typing_spec = read("spec/003-typing.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let std_config = read("docs/std-config-json-loading.md");
    let std_json_audit = read("docs/std-json-implementation-audit.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");

    for required in [
        "Option(Box<JsonDecodeSchema>)",
        "List(Box<JsonDecodeSchema>)",
        "TypedStringMap(Box<JsonDecodeSchema>)",
        "\"O\"",
        "\"L\"",
        "\"MT\"",
        "map_symbols",
        "validate_symbols",
    ] {
        assert!(
            json_decode.contains(required),
            "JSON decoder schema implementation missing `{required}`"
        );
    }

    for required in [
        "Type::Option(item)",
        "matches!(item, Type::Option(_))",
        "JsonDecodeSchema::Option",
        "JsonDecodeSchema::List",
        "JsonDecodeSchema::TypedStringMap",
        "Map[String, T]",
        "Map[String, json::Value]",
    ] {
        assert!(
            typing.contains(required),
            "typing implementation missing structural decoder support `{required}`"
        );
    }

    for required in [
        "JsonDecodeSchema::Option",
        "JsonDecodeSchema::List",
        "JsonDecodeSchema::TypedStringMap",
    ] {
        assert!(
            mir.contains(required),
            "MIR lowering missing structural decoder schema `{required}`"
        );
    }

    for required in [
        "decode_json_option",
        "decode_json_option_required",
        "decode_json_list",
        "decode_json_typed_string_map",
        "is_json_null",
        "option_some_payload",
        "JsonDecodeSchema::Option",
        "JsonDecodeSchema::List",
        "JsonDecodeSchema::TypedStringMap",
    ] {
        assert!(
            runtime.contains(required),
            "runtime implementation missing structural decoder support `{required}`"
        );
    }

    for required in [
        "standard_json_decode_structural_targets_run",
        "standard_json_decode_or_structural_targets_overlay_run",
        "standard_config_load_json_or_structural_targets_runs",
        "standard_json_decode_structural_targets_report_path_errors",
        "standard_json_decode_structural_artifact_run_uses_schema_payload",
        "standard_json_decode_rejects_deferred_structural_targets",
        "Option[Option[String]]",
        "Map[Int, String]",
    ] {
        assert!(
            examples.contains(required),
            "examples coverage missing structural decoder case `{required}`"
        );
    }

    for (label, text) in [
        ("design", design.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("typing spec", typing_spec.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("std config docs", std_config.as_str()),
        ("std json audit", std_json_audit.as_str()),
        ("stdlib review", stdlib_review.as_str()),
    ] {
        assert!(
            text.contains("Option[T]")
                && text.contains("recursive `List[T]`")
                && text.contains("Map[String, T]")
                && text.contains("json::decode[T]")
                && text.contains("config::load_json_or[T]"),
            "{label} must document implemented structural JSON decoder targets"
        );
    }

    assert!(
        implementation_resume.contains("| 207. structural JSON decoder target implementation |")
            && implementation_resume
                .contains("| json_decode/typing/runtime/artifacts/tests/docs | Done |")
            && implementation_resume
                .contains("| 208. post-structural-json-decoder adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 209. structural config workflow refresh |")
            && implementation_resume.contains("| project template/samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 210. post-structural-config-workflow adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 211. enum JSON/config decoder implementation |")
            && implementation_resume
                .contains("| json_decode/typing/runtime/artifacts/tests/docs | Done |")
            && implementation_resume
                .contains("| 212. post-enum JSON/config decoder adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 213. JSON/config schema polish design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 214. JSON/config field and variant rename implementation |")
            && implementation_resume.contains(
                "| parser/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            ),
        "implementation queue must mark rename implementation done"
    );
}

#[test]
fn structural_config_workflow_refresh_is_implemented_and_covered() {
    let sample = read("samples/projects/config_app/src/main/main.muga");
    let config = read("samples/projects/config_app/config/settings.json");
    let template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let std_json_audit = read("docs/std-json-implementation-audit.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");

    for required in [
        "record Owner",
        "name: Option[String]",
        "team: Option[String]",
        "record Server",
        "servers: List[Server]",
        "limits: Map[String, Int]",
        "config::load_json_or(config_path, default_settings())",
        "option_string(settings.owner.name, \"unknown\")",
        "first_server_port(settings.servers)",
        "limit_or(settings.limits, \"workers\", 0)",
        "cli::parse_or(settings_args(args), configured)",
        "fn settings_args(args: List[String]): List[String]",
    ] {
        assert!(
            sample.contains(required),
            "config app structural refresh missing `{required}`"
        );
        assert!(
            template.contains(required),
            "generated config app template missing `{required}`"
        );
    }

    for removed in ["import std::json", "Map[String, json::Value]", "json::at"] {
        assert!(
            !sample.contains(removed) && !template.contains(removed),
            "config app structural refresh should remove `{removed}`"
        );
    }

    for required in [
        "\"owner\": {",
        "\"team\": null",
        "\"servers\": [",
        "\"name\": \"api\"",
        "\"port\": 9000",
        "\"limits\": {",
        "\"workers\": 4",
        "\"retries\": 2",
    ] {
        assert!(
            config.contains(required) && template.contains(required),
            "structural config fixture/template missing `{required}`"
        );
    }

    for required in [
        "cli_new_creates_app_lib_and_test_templates",
        "manifest_config_project_sample_runs_with_cli_overrides",
        "manifest_config_project_sample_runs_against_emitted_artifacts",
        "manifest_config_project_sample_reports_config_shape_errors",
        "manifest_config_project_sample_reports_cli_parse_errors",
        "manifest_config_project_sample_json_built_run_applies_cli_overrides",
        "std__config.mgb",
        "std__cli.mgb",
        "std__string.mgb",
        "Result::Ok(Grace|9090|true|1|ops|none|2|9000|4)",
        "Result::Ok(Ada|4040|false|2|ops|none|2|9000|4)",
        "Result::Err(config Decode -1: expected JSON Int at path .servers[0].port)",
        "Result::Ok(Kai|5050|true|1|unknown|none|1|8080|1)",
    ] {
        assert!(
            examples.contains(required),
            "examples coverage missing structural config workflow `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("std json audit", std_json_audit.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
    ] {
        assert!(
            text.contains("structural config workflow")
                && text.contains("Option[String]")
                && text.contains("List[Record]")
                && text.contains("Map[String, Int]")
                && text.contains("config-app"),
            "{label} must document the implemented structural config workflow refresh"
        );
    }

    assert!(
        implementation_resume.contains("| 209. structural config workflow refresh |")
            && implementation_resume.contains("| project template/samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 210. post-structural-config-workflow adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 211. enum JSON/config decoder implementation |")
            && implementation_resume
                .contains("| json_decode/typing/runtime/artifacts/tests/docs | Done |")
            && implementation_resume
                .contains("| 212. post-enum JSON/config decoder adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 213. JSON/config schema polish design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 214. JSON/config field and variant rename implementation |")
            && implementation_resume.contains(
                "| parser/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            ),
        "implementation queue must mark rename implementation done"
    );
}

#[test]
fn enum_json_config_decoder_implementation_is_covered() {
    let json_decode = read("src/json_decode.rs");
    let typing = read("src/typing.rs");
    let mir = read("src/mir.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/json-decoder-target-expansion.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let typing_spec = read("spec/003-typing.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let std_config = read("docs/std-config-json-loading.md");
    let std_json_audit = read("docs/std-json-implementation-audit.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");

    for required in [
        "JsonDecodeVariantSchema",
        "Enum {",
        "\"E\"",
        "parse_bool_token",
        "decoder variant",
        "validate_symbols",
    ] {
        assert!(
            json_decode.contains(required),
            "JSON decoder schema missing enum support `{required}`"
        );
    }

    for required in [
        "Type::Enum(enum_name, args)",
        "JsonDecodeSchema::Enum",
        "JsonDecodeVariantSchema",
        "enum_variant_payload_type",
        "concrete non-generic enums",
    ] {
        assert!(
            typing.contains(required),
            "typing implementation missing enum decoder support `{required}`"
        );
    }

    for required in [
        "JsonDecodeSchema::Enum",
        "JsonDecodeVariantSchema",
        "package_item_symbol",
    ] {
        assert!(
            mir.contains(required),
            "MIR lowering missing enum decoder support `{required}`"
        );
    }

    for required in [
        "decode_json_enum",
        "decode_json_enum_object",
        "json_decode_unknown_enum_variant_error",
        "json_decode_enum_payload_fallback",
        "expected single-key JSON Object",
        "String or Object",
        "JsonDecodeSchema::Enum",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing enum decoder support `{required}`"
        );
    }

    for required in [
        "standard_json_decode_enum_targets_run",
        "standard_json_decode_or_enum_targets_overlay_run",
        "standard_json_decode_enum_targets_report_path_errors",
        "standard_json_decode_enum_artifact_run_uses_schema_payload",
        "standard_config_load_json_or_enum_targets_runs",
        "unknown JSON enum variant `Paused` at path .mode",
        "expected JSON Int at path .action.Scale",
        "Box[String]",
    ] {
        assert!(
            examples.contains(required),
            "examples coverage missing enum decoder case `{required}`"
        );
    }

    for (label, text) in [
        ("design", design.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("typing spec", typing_spec.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("std config docs", std_config.as_str()),
        ("std json audit", std_json_audit.as_str()),
        ("stdlib review", stdlib_review.as_str()),
    ] {
        assert!(
            text.contains("concrete")
                && text.contains("enum")
                && text.contains("zero-payload")
                && text.contains("one-payload")
                && text.contains("generic enum"),
            "{label} must document implemented enum JSON/config decoder support"
        );
    }

    assert!(
        implementation_resume.contains("| 211. enum JSON/config decoder implementation |")
            && implementation_resume
                .contains("| json_decode/typing/runtime/artifacts/tests/docs | Done |")
            && implementation_resume
                .contains("| 212. post-enum JSON/config decoder adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 213. JSON/config schema polish design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 214. JSON/config field and variant rename implementation |")
            && implementation_resume.contains(
                "| parser/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            ),
        "implementation queue must mark rename implementation done"
    );
}

#[test]
fn json_config_schema_polish_design_is_documented() {
    let design = read("docs/json-config-schema-polish.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let std_json_audit = read("docs/std-json-implementation-audit.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");

    for required in [
        "Status: JSON/config field and variant rename implemented",
        "@json(rename: \"...\")",
        "record fields and enum variants",
        "Candidates Compared",
        "Separate attributes",
        "Automatic case conversion",
        "External schema declarations",
        "Runtime helper transforms",
        "Implemented Surface",
        "duplicate effective wire names",
        "path-aware",
        ".server_host",
        "Metadata Pipeline",
        "package interfaces",
        "Artifact Tokens",
        "RA <type_symbol>",
        "EA <type_symbol>",
        "Diagnostics",
        "Deferred Work",
        "Implementation Plan",
        "Done: implement parser/AST/typechecker metadata",
        "Done: audit post-rename adoption",
        "Done: design record-level strict unknown-field policy",
        "Done: implement record-level `@json(deny_unknown_fields)`",
        "Done: audit post-strict unknown-field adoption",
        "Done: design `@json(alias: \"...\")` metadata",
        "Done: implement alias metadata",
    ] {
        assert!(
            design.contains(required),
            "JSON/config schema polish design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("std json audit", std_json_audit.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
    ] {
        assert!(
            text.contains("json-config-schema-polish.md")
                && text.contains("@json(rename")
                && text.contains("record fields and enum variants")
                && text.contains("TOML"),
            "{label} must reference the JSON/config schema polish design"
        );
    }

    assert!(
        implementation_resume.contains("| 213. JSON/config schema polish design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 214. JSON/config field and variant rename implementation |")
            && implementation_resume.contains(
                "| parser/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 215. post-rename JSON/config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 216. JSON/config strict unknown-field policy design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 217. JSON/config strict unknown-field policy implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 218. post-strict JSON/config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 219. JSON/config alias metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 220. JSON/config alias metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            ),
        "implementation queue must mark rename and strict unknown-field follow-up selection done"
    );
}

#[test]
fn json_config_strict_unknown_field_policy_design_is_documented() {
    let design = read("docs/json-config-strict-unknown-fields.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let std_json_audit = read("docs/std-json-implementation-audit.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let schema_polish = read("docs/json-config-schema-polish.md");

    for required in [
        "Status: JSON/config strict unknown-field policy implemented",
        "@json(deny_unknown_fields)",
        "@json(rename: \"...\")",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Syntax Decision",
        "Candidates Compared",
        "Record-level `@json(deny_unknown_fields)`",
        "Make all record decoders strict by default",
        "Add separate strict decode functions or call options",
        "Field-level unknown-key markers",
        "Attribute Validation",
        "Accepted-Key Semantics",
        "Runtime Behavior",
        "json::decode_or[T]",
        "json::decode[T]",
        "config::load_json_or[T]",
        "unexpected JSON field",
        "Metadata Pipeline",
        "json_deny_unknown_fields",
        "Package Interface Format",
        "record JSON flags",
        "Artifact Tokens",
        "RF <type_symbol> <flags> <field_count>",
        "Tests",
        "malformed artifacts reject unknown `RF` flag bits",
        "Deferred Work",
        "Implementation Plan",
        "Done: implement the strict unknown-field policy",
        "Done: audit post-strict unknown-field adoption",
        "Done: design `@json(alias: \"...\")`",
        "Done: implement alias metadata",
    ] {
        assert!(
            design.contains(required),
            "JSON/config strict unknown-field design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("std json audit", std_json_audit.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("schema polish", schema_polish.as_str()),
    ] {
        assert!(
            text.contains("json-config-strict-unknown-fields.md")
                && text.contains("@json(deny_unknown_fields)")
                && text.contains("RF")
                && text.contains("aliases")
                && text.contains("TOML"),
            "{label} must reference the JSON/config strict unknown-field design"
        );
    }

    assert!(
        implementation_resume
            .contains("| 216. JSON/config strict unknown-field policy design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 217. JSON/config strict unknown-field policy implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 218. post-strict JSON/config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 219. JSON/config alias metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 220. JSON/config alias metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            ),
        "implementation queue must mark strict unknown-field implementation and post-strict selection done"
    );
}

#[test]
fn json_config_strict_unknown_field_policy_implementation_is_covered() {
    let ast = read("src/ast.rs");
    let parser = read("src/parser.rs");
    let formatter = read("src/formatter.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let package_signature = read("src/package_signature.rs");
    let package_rewrite = read("src/package.rs");
    let interface = read("src/interface.rs");
    let json_decode = read("src/json_decode.rs");
    let mir = read("src/mir.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/json-config-strict-unknown-fields.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let std_config = read("docs/std-config-json-loading.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");

    for required in [
        "pub value: Option<AttributeArgumentValue>",
        "pub attributes: Vec<Attribute>",
        "pub struct RecordDecl",
    ] {
        assert!(
            ast.contains(required),
            "AST missing strict unknown-field attribute support `{required}`"
        );
    }

    for required in [
        "deny_unknown_fields",
        "validate_record_attributes",
        "json_attribute_is_deny_unknown_fields",
        "record declarations support only",
        "record fields support only",
        "enum variants support only",
    ] {
        assert!(
            parser.contains(required),
            "parser missing strict unknown-field handling `{required}`"
        );
    }

    assert!(
        formatter.contains("format_attributes(&stmt.attributes")
            && formatter.contains("if let Some(value) = &argument.value"),
        "formatter must preserve record-level flag attributes and string attributes"
    );

    for required in [
        "json_deny_unknown_fields: bool",
        "json_deny_unknown_fields_from_attributes",
        "deny_unknown_fields: record.json_deny_unknown_fields",
    ] {
        assert!(
            typing.contains(required),
            "typing missing strict unknown-field metadata `{required}`"
        );
    }

    for required in [
        "json_deny_unknown_fields: bool",
        "json_deny_unknown_fields_from_attributes",
    ] {
        assert!(
            typed_hir.contains(required) && package_signature.contains(required),
            "typed HIR and package signatures must preserve strict unknown-field metadata `{required}`"
        );
    }

    assert!(
        package_rewrite.contains("attributes: record.attributes.clone()"),
        "package rewrite must preserve record-level JSON attributes"
    );

    for required in [
        "muga-package-interface-v6",
        "\"muga-package-interface-v5\"",
        "json_deny_unknown_fields: bool",
        "record_json_flags",
        "record JSON flags",
        "json_deny_unknown_fields: record.json_deny_unknown_fields",
        "record.json_deny_unknown_fields == interface.json_deny_unknown_fields",
    ] {
        assert!(
            interface.contains(required),
            "package interface missing strict unknown-field persistence `{required}`"
        );
    }

    for required in [
        "deny_unknown_fields: bool",
        "\"RF\"",
        "record JSON flags",
        "invalid record JSON flags",
        "record_artifact_rejects_unknown_json_flag_bits",
    ] {
        assert!(
            json_decode.contains(required),
            "JSON decoder schema missing strict unknown-field artifact support `{required}`"
        );
    }

    assert!(
        mir.contains("deny_unknown_fields: *deny_unknown_fields"),
        "MIR lowering must preserve strict unknown-field schema flags"
    );

    for required in [
        "reject_unknown_json_record_fields",
        "unexpected JSON field",
        "deny_unknown_fields",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing strict unknown-field behavior `{required}`"
        );
    }

    for required in [
        "standard_json_decode_strict_unknown_fields_run",
        "standard_config_load_json_or_reports_strict_unknown_fields",
        "package_interfaces_preserve_json_rename_metadata_without_source",
        "unexpected JSON field `retry_count` at path .retry_count",
        "unexpected JSON field `portt` at path .primary.portt",
        "unexpected JSON field `extra` at path .extra",
        "run_path_against_artifact_root",
    ] {
        assert!(
            examples.contains(required),
            "examples coverage missing strict unknown-field case `{required}`"
        );
    }

    for (label, text) in [
        ("design", design.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("std config", std_config.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
    ] {
        assert!(
            text.contains("@json(deny_unknown_fields)")
                && text.contains("unknown")
                && text.contains("TOML"),
            "{label} must document implemented strict unknown-field metadata"
        );
    }

    assert!(
        implementation_resume
            .contains("| 217. JSON/config strict unknown-field policy implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 218. post-strict JSON/config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 219. JSON/config alias metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 220. JSON/config alias metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            ),
        "implementation queue must mark strict unknown-field implementation done and queue alias design"
    );
}

#[test]
fn json_config_alias_metadata_design_is_documented() {
    let design = read("docs/json-config-alias-metadata.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let std_json_audit = read("docs/std-json-implementation-audit.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let schema_polish = read("docs/json-config-schema-polish.md");
    let strict_design = read("docs/json-config-strict-unknown-fields.md");

    for required in [
        "Status: JSON/config alias metadata implemented",
        "@json(alias: \"...\")",
        "@json(rename: \"...\")",
        "@json(deny_unknown_fields)",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Syntax Decision",
        "Syntax Candidates Compared",
        "Single `@json(rename: \"...\", alias: \"...\", alias: \"...\")` attribute",
        "Repeated `@json(alias: \"...\")` attributes",
        "`@json(aliases: [\"a\", \"b\"])` list argument",
        "Accepted-Name Semantics",
        "Decode Conflict Policy",
        "json::decode_or[T]",
        "config::load_json_or[T]",
        "Metadata Pipeline",
        "json_aliases: Vec<Symbol>",
        "Package Interface Format",
        "muga-package-interface-v11",
        "Artifact Tokens",
        "RG <type_symbol>",
        "EG <type_symbol>",
        "Tests",
        "malformed alias artifact payloads are rejected",
        "Deferred Work",
        "Implementation Plan",
        "Done: design alias syntax",
        "Done: implement the smallest alias metadata slice",
    ] {
        assert!(
            design.contains(required),
            "JSON/config alias metadata design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("std json audit", std_json_audit.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("schema polish", schema_polish.as_str()),
        ("strict unknown-field design", strict_design.as_str()),
    ] {
        assert!(
            text.contains("json-config-alias-metadata.md")
                && text.contains("@json(alias")
                && text.contains("validation")
                && text.contains("TOML"),
            "{label} must reference the JSON/config alias metadata design"
        );
    }

    assert!(
        implementation_resume.contains("| 219. JSON/config alias metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 220. JSON/config alias metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            ),
        "implementation queue must mark alias metadata design and implementation done"
    );
}

#[test]
fn json_config_alias_metadata_implementation_is_covered() {
    let parser = read("src/parser.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let package_signature = read("src/package_signature.rs");
    let json_decode = read("src/json_decode.rs");
    let mir = read("src/mir.rs");
    let runtime = read("src/runtime.rs");
    let interface = read("src/interface.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/json-config-alias-metadata.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let schema_polish = read("docs/json-config-schema-polish.md");
    let strict_design = read("docs/json-config-strict-unknown-fields.md");
    let std_config = read("docs/std-config-json-loading.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");

    for required in [
        "alias",
        "JSON rename and alias values must be non-empty",
        "JSON rename may be specified only once",
        "json_attribute_is_json_schema_metadata",
    ] {
        assert!(
            parser.contains(required),
            "parser missing JSON alias metadata handling `{required}`"
        );
    }

    for required in [
        "json_aliases: Vec<Symbol>",
        "json_aliases_from_attributes",
        "duplicate JSON field wire name",
        "duplicate JSON enum variant wire name",
        "aliases: field.json_aliases.clone()",
        "aliases: variant.json_aliases.clone()",
    ] {
        assert!(
            typing.contains(required),
            "typing missing JSON alias metadata handling `{required}`"
        );
    }

    for required in ["json_aliases: Vec<String>", "json_aliases_from_attributes"] {
        assert!(
            typed_hir.contains(required) && package_signature.contains(required),
            "typed HIR and package signatures must preserve JSON aliases `{required}`"
        );
    }

    for required in [
        "aliases: Vec<Symbol>",
        "\"RG\"",
        "\"EG\"",
        "record field alias count",
        "enum variant alias count",
        "invalid decoder field alias symbol",
        "alias_artifacts_reject_malformed_payloads",
    ] {
        assert!(
            json_decode.contains(required),
            "JSON decoder schema missing alias artifact support `{required}`"
        );
    }

    assert!(
        mir.contains("aliases: field")
            && mir.contains("aliases: variant")
            && mir.contains("source_symbol(*alias)"),
        "MIR lowering must preserve JSON alias symbols"
    );

    for required in [
        "json_object_field_for_decode",
        "json_decode_field_accepts_name",
        "json_decode_variant_accepts_name",
        "multiple JSON fields match",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing JSON alias decoding behavior `{required}`"
        );
    }

    for required in [
        "muga-package-interface-v11",
        "\"muga-package-interface-v7\"",
        "\"muga-package-interface-v6\"",
        "json_aliases: Vec<String>",
        "field JSON alias count",
        "enum variant JSON alias count",
        "json_aliases: field.json_aliases.clone()",
        "field.json_aliases == expected.json_aliases",
        "variant.json_aliases == expected.json_aliases",
    ] {
        assert!(
            interface.contains(required),
            "package interface missing JSON alias persistence `{required}`"
        );
    }

    for required in [
        "standard_json_decode_json_alias_metadata_runs",
        "standard_json_decode_json_alias_rejects_duplicate_accepted_names",
        "package_interfaces_preserve_json_rename_metadata_without_source",
        "multiple JSON fields match `host` at path .host",
        "muga-package-interface-v11",
        "run_path_against_artifact_root",
    ] {
        assert!(
            examples.contains(required),
            "examples coverage missing JSON alias case `{required}`"
        );
    }

    for (label, text) in [
        ("design", design.as_str()),
        ("schema polish", schema_polish.as_str()),
        ("strict unknown-field design", strict_design.as_str()),
        ("std config", std_config.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
    ] {
        assert!(
            text.contains("@json(alias")
                && text.contains("implemented")
                && text.contains("validation")
                && text.contains("TOML"),
            "{label} must document implemented JSON alias metadata"
        );
    }

    assert!(
        implementation_resume
            .contains("| 220. JSON/config alias metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 221. post-alias JSON/config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |"),
        "implementation queue must mark alias metadata implementation done and post-alias selection done"
    );
}

#[test]
fn json_config_validation_attribute_implementation_is_covered() {
    let design = read("docs/json-config-validation-attributes.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let parser = read("src/parser.rs");
    let formatter = read("src/formatter.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let package_signature = read("src/package_signature.rs");
    let interface = read("src/interface.rs");
    let json_decode = read("src/json_decode.rs");
    let mir = read("src/mir.rs");
    let runtime = read("src/runtime.rs");
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let errors = read("errors.md");

    for required in [
        "Status: JSON/config validation attributes implemented",
        "@validate(...)",
        "@json(rename: \"...\")",
        "@json(alias: \"...\")",
        "@json(deny_unknown_fields)",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Syntax Decision",
        "Syntax Candidates Compared",
        "Field-level `@validate(...)`",
        "Put validators inside `@json(...)`",
        "First Validator Set",
        "`String`: `non_empty`, `min_len: Int`, `max_len: Int`",
        "`Int`: `min: Int`, `max: Int`",
        "Runtime Semantics",
        "json::ErrorKind::Validation",
        "Metadata Pipeline",
        "JsonDecodeValidationRule",
        "Package Interface Format",
        "muga-package-interface-v11",
        "Artifact Tokens",
        "RV <type_symbol>",
        "Tests",
        "malformed validation artifact payloads are rejected",
        "Deferred Work",
        "Implementation Plan",
        "Done: design the validation attribute syntax",
        "Done: implement the smallest validation attribute slice",
        "Done: audit the post-validation JSON/config adoption gap",
        "Done: design JSON/config schema export",
        "Done: implement the smallest schema export slice",
        "Done: audit the post-schema-export JSON/config adoption gap",
        "Done: design typed JSON encoding",
        "Done: implement the smallest typed JSON encoding slice",
    ] {
        assert!(
            design.contains(required),
            "JSON/config validation attribute implementation docs missing `{required}`"
        );
    }

    for required in [
        "AttributeArgumentValue",
        "parse_attribute_argument_value",
        "validate_validate_attribute_arguments",
        "validate_record_field_attributes",
        "attribute `@validate` is allowed only on record fields",
    ] {
        assert!(
            parser.contains(required),
            "parser missing validation attribute support `{required}`"
        );
    }
    assert!(
        formatter.contains("AttributeArgumentValue::Int"),
        "formatter must preserve integer validation attribute arguments"
    );
    for required in [
        "json_validation: Vec<JsonDecodeValidationRule>",
        "json_validation_from_attributes",
        "check_json_validation_attributes",
        "JSON validation `min` may not be greater than `max`",
        "is not supported for field",
        "validation: field.json_validation.clone()",
    ] {
        assert!(
            typing.contains(required),
            "typing missing validation metadata and checks `{required}`"
        );
    }
    for required in [
        "json_validation: Vec<JsonDecodeValidationRule>",
        "json_validation_from_attributes",
    ] {
        assert!(
            typed_hir.contains(required) && package_signature.contains(required),
            "typed HIR and package signatures must preserve validation metadata `{required}`"
        );
    }
    for required in [
        "muga-package-interface-v11",
        "\"muga-package-interface-v7\"",
        "json_validation: Vec<JsonDecodeValidationRule>",
        "field JSON validation count",
        "invalid field JSON validation list",
        "field.json_validation == expected.json_validation",
    ] {
        assert!(
            interface.contains(required),
            "package interface missing validation persistence `{required}`"
        );
    }
    for required in [
        "JsonDecodeValidationRule",
        "\"RV\"",
        "record field validation count",
        "record_validation_artifact_round_trips",
        "validation_artifacts_reject_malformed_payloads",
    ] {
        assert!(
            json_decode.contains(required),
            "JSON decoder schema missing validation artifact support `{required}`"
        );
    }
    assert!(
        mir.contains("validation: field.validation.clone()"),
        "MIR lowering must preserve validation metadata"
    );
    for required in [
        "JsonErrorKind::Validation",
        "validate_json_decoded_field",
        "validate_json_rule",
        "validation failed at path",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing validation decode behavior `{required}`"
        );
    }
    assert!(
        std_package.contains("Validation"),
        "std::json::ErrorKind must expose Validation"
    );
    assert!(
        errors.contains("T028") && errors.contains("invalid JSON/config validation metadata"),
        "error catalog must document validation metadata diagnostics"
    );
    for required in [
        "standard_json_decode_validation_attributes_run",
        "standard_json_decode_validation_attributes_reject_bad_targets_and_rules",
        "standard_config_load_json_or_reports_validation_errors",
        "package_interfaces_preserve_json_validation_metadata_without_source",
        "cli_run_built_preserves_json_validation_metadata",
        "muga-package-interface-v11",
        "RV 1 1 1",
    ] {
        assert!(
            examples.contains(required) || json_decode.contains(required),
            "tests missing validation coverage `{required}`"
        );
    }

    for (label, text) in [("README", readme.as_str()), ("ROADMAP", roadmap.as_str())] {
        assert!(
            text.contains("json-config-validation-attributes.md")
                && text.contains("@validate")
                && text.contains("TOML")
                && text.contains("full CLI"),
            "{label} must reference the JSON/config validation attribute implementation"
        );
    }

    assert!(
        implementation_resume
            .contains("| 222. JSON/config validation attribute design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 223. JSON/config validation attribute implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume.contains("| 224. post-validation JSON/config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 225. JSON/config schema export design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 226. JSON/config schema export implementation |")
            && implementation_resume.contains("| schema/CLI/package/interface/tests/docs | Done |")
            && implementation_resume
                .contains("| 227. post-schema-export JSON/config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 228. typed JSON encoding design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 229. typed JSON encoding implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/artifacts/runtime/tests/docs | Done |"
            ),
        "implementation queue must mark validation, schema export, post-schema-export selection, and typed JSON encoding design done, then queue implementation"
    );
    assert!(
        implementation_resume.contains("Next recommended slice: implement CLI subcommand metadata")
            && !implementation_resume.contains(
                "The next implementation-facing step should implement JSON/config alias metadata"
            ),
        "implementation snapshot must point at CLI parser schema implementation next"
    );
}

#[test]
fn json_config_schema_export_implementation_is_covered() {
    let design = read("docs/json-config-schema-export.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let std_json_audit = read("docs/std-json-implementation-audit.md");
    let schema_export = read("src/schema_export.rs");
    let lib = read("src/lib.rs");
    let main = read("src/main.rs");
    let examples = read("tests/examples.rs");
    let errors = read("errors.md");

    for required in [
        "Status: JSON/config schema export implemented",
        "JSON Schema Draft 2020-12",
        "$schema",
        "https://json-schema.org/draft/2020-12/schema",
        "$id",
        "$defs",
        "x-muga",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Format Decision",
        "CLI/API Decision",
        "muga schema --format json",
        "--decode-mode required|overlay",
        "Source And Interface Scope",
        "public concrete record and enum",
        "The implementation adds `T029`",
        "Type Mapping",
        "std::json::Value",
        "Record Mapping",
        "additionalProperties: false",
        "x-muga.validation",
        "Enum Mapping",
        "oneOf",
        "Definition Identity",
        "Candidates Compared",
        "Muga-native schema JSON only",
        "OpenAPI schema component output first",
        "Embed schemas into `metadata --format json`",
        "Tests",
        "unsupported generic or opaque targets",
        "Deferred Work",
        "Implementation Plan",
        "Done: implement the smallest schema export slice",
        "Done: audit the post-schema-export JSON/config adoption gap",
        "Done: design typed JSON encoding",
        "Done: implement the smallest typed JSON encoding slice",
    ] {
        assert!(
            design.contains(required),
            "JSON/config schema export implementation docs missing `{required}`"
        );
    }

    for required in [
        "JSON_SCHEMA_DRAFT_2020_12",
        "SchemaDecodeMode",
        "SchemaExportOptions",
        "render_json_config_schema_for_interfaces",
        "Self::Overlay => \"overlay\"",
        "definition_key",
        "field_wire_name",
        "variant_wire_name",
        "apply_string_validation",
        "apply_int_validation",
        "schema export does not support generic record",
        "schema export does not support generic enum",
        "type `{}` is not supported by JSON/config schema export",
    ] {
        assert!(
            schema_export.contains(required),
            "schema exporter missing implementation evidence `{required}`"
        );
    }
    for required in [
        "pub mod schema_export",
        "pub use schema_export::{SchemaDecodeMode, SchemaExportOptions}",
        "render_json_config_schema_for_check",
        "loaded_interfaces.graph.package_by_path(package).is_some()",
    ] {
        assert!(
            lib.contains(required),
            "library API missing schema export evidence `{required}`"
        );
    }
    for required in [
        "Mode::Schema",
        "parse_schema_decode_mode",
        "schema requires --format json",
        "accepts at most one --package",
        "--type is only supported with `schema`, `cli-completions`, `emit-cli-completions`, or `emit-app-completions`",
        "--decode-mode is only supported with `schema`",
        "muga schema --format json",
    ] {
        assert!(
            main.contains(required),
            "CLI missing schema export evidence `{required}`"
        );
    }
    assert!(
        errors.contains("T029")
            && errors.contains("unsupported JSON/config schema export targets or field types"),
        "error catalog must document schema export diagnostics"
    );
    for required in [
        "cli_schema_json_exports_record_contract",
        "cli_schema_json_exports_overlay_mode_without_required_fields",
        "cli_schema_json_exports_enum_contract",
        "cli_schema_json_exports_dependency_package_from_interfaces",
        "cli_schema_json_rejects_unsupported_generic_target",
    ] {
        assert!(
            examples.contains(required),
            "examples coverage missing schema export case `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("std json audit", std_json_audit.as_str()),
    ] {
        assert!(
            text.contains("json-config-schema-export.md")
                && text.contains("JSON Schema Draft 2020-12")
                && text.contains("muga schema --format json")
                && text.contains("TOML"),
            "{label} must reference the JSON/config schema export implementation"
        );
    }

    assert!(
        implementation_resume.contains("| 225. JSON/config schema export design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 226. JSON/config schema export implementation |")
            && implementation_resume.contains("| schema/CLI/package/interface/tests/docs | Done |")
            && implementation_resume
                .contains("| 227. post-schema-export JSON/config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 228. typed JSON encoding design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 229. typed JSON encoding implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/artifacts/runtime/tests/docs | Done |"
            ),
        "implementation queue must mark schema export design, implementation, post-schema-export selection, and typed JSON encoding design done, then typed JSON encoding implementation done"
    );
    assert!(
        implementation_resume.contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation snapshot must point at generated config-app CLI schema adoption next"
    );
}

#[test]
fn json_typed_encoding_design_is_documented() {
    let design = read("docs/json-typed-encoding.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let schema_export_design = read("docs/json-config-schema-export.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let std_json_audit = read("docs/std-json-implementation-audit.md");
    let std_package = read("src/std_package.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let mir = read("src/mir.rs");
    let bytecode = read("src/bytecode.rs");
    let implementation_artifact = read("src/implementation_artifact.rs");
    let json_decode = read("src/json_decode.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");

    for required in [
        "Status: typed JSON encoding implemented",
        "compiler-owned typed JSON encoding boundary",
        "json::to_value[T](value)",
        "json::encode_typed[T](value)",
        "json::encode(value: json::Value)",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Selected API",
        "Supported First Targets",
        "Option[Option[T]]",
        "std::json::Value",
        "Mapping Semantics",
        "record field `Option::None`",
        "Attribute Semantics",
        "@json(rename: \"...\")",
        "@json(alias: \"...\")",
        "@json(deny_unknown_fields)",
        "@validate(...)",
        "Validation-on-encode",
        "Option Policy",
        "Enum Policy",
        "Schema And Artifacts",
        "JsonDecodeSchema",
        "JsonContractSchema",
        "Diagnostics",
        "`json::to_value` supports only",
        "Implemented Coverage",
        "artifact-backed execution and `run --built` coverage",
        "Candidates Compared",
        "Overload existing `json::encode` for typed values",
        "Runtime reflective builtin",
        "Non-Goals",
        "Implementation Plan",
        "Done: select compiler-owned `json::to_value[T](value)`",
        "Done: implement the smallest typed JSON encoding slice",
    ] {
        assert!(
            design.contains(required),
            "typed JSON encoding design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("schema export design", schema_export_design.as_str()),
        ("strategy", strategy.as_str()),
        ("std json audit", std_json_audit.as_str()),
    ] {
        assert!(
            text.contains("json-typed-encoding.md")
                && text.contains("json::to_value")
                && text.contains("typed JSON encoding")
                && text.contains("TOML"),
            "{label} must reference the typed JSON encoding design"
        );
    }

    for required in [
        "pub fn to_value[T](value: T): Result[Value, Error]",
        "pub fn encode_typed[T](value: T): Result[String, Error]",
    ] {
        assert!(
            std_package.contains(required),
            "std::json package signature missing typed encoding helper `{required}`"
        );
    }
    for required in [
        "std_json_to_value_bindings",
        "std_json_encode_typed_bindings",
        "json_to_value_schemas",
        "json_encode_typed_schemas",
        "check_std_json_to_value_call",
        "check_std_json_encode_typed_call",
        "json_encode_schema_for_type",
        "JsonDecodeSchema::JsonValue",
    ] {
        assert!(
            typing.contains(required),
            "typing missing typed JSON encoding evidence `{required}`"
        );
    }
    for required in ["json_to_value_schema", "json_encode_typed_schema"] {
        assert!(
            typed_hir.contains(required),
            "typed HIR missing typed JSON encoding schema evidence `{required}`"
        );
    }
    for required in [
        "JsonToValueExpr",
        "JsonEncodeTypedExpr",
        "Expr::JsonToValue",
        "Expr::JsonEncodeTyped",
    ] {
        assert!(
            mir.contains(required),
            "MIR missing typed JSON encoding lowering evidence `{required}`"
        );
    }
    for required in ["Instruction::JsonToValue", "Instruction::JsonEncodeTyped"] {
        assert!(
            bytecode.contains(required)
                && implementation_artifact.contains("JsonToValue")
                && implementation_artifact.contains("JsonEncodeTyped"),
            "bytecode/artifacts missing typed JSON encoding instruction evidence `{required}`"
        );
    }
    for required in ["JsonValue", "\"V\""] {
        assert!(
            json_decode.contains(required),
            "JSON schema payloads missing raw json::Value encoding evidence `{required}`"
        );
    }
    for required in [
        "typed_value_to_json_value",
        "typed_record_to_json_object",
        "typed_enum_to_json_value",
        "validate_json_decoded_field(field, field_value",
        "Instruction::JsonEncodeTyped",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing typed JSON encoding behavior `{required}`"
        );
    }
    for required in [
        "standard_json_encode_typed_record_runs",
        "standard_json_to_value_record_runs",
        "standard_json_encode_typed_reports_validation_errors",
        "standard_json_encode_typed_artifact_run_uses_schema_payload",
        "standard_json_encode_typed_interface_artifact_run_uses_schema_payload",
        "standard_json_encode_typed_rejects_unsupported_targets",
    ] {
        assert!(
            examples.contains(required),
            "examples missing typed JSON encoding coverage `{required}`"
        );
    }

    assert!(
        implementation_resume.contains("| 228. typed JSON encoding design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 229. typed JSON encoding implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/artifacts/runtime/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 230. post-typed JSON encoding adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 231. full CLI parser schema design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 232. first CLI parser schema implementation |")
            && implementation_resume.contains(
                "| std_package/typing/mir/bytecode/artifacts/runtime/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 233. post-CLI parser schema adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 234. generated config-app CLI schema adoption |")
            && implementation_resume.contains("| project template/samples/tests/docs | Done |"),
        "implementation queue must mark typed JSON encoding design, CLI parser schema design, CLI parser implementation, and post-CLI parser selection done, then queue generated config-app adoption"
    );
    assert!(
        implementation_resume.contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation snapshot must point at generated config-app CLI schema adoption next"
    );
}

#[test]
fn cli_parser_schema_design_is_documented() {
    let design = read("docs/cli-parser-schema.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let json_typed = read("docs/json-typed-encoding.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let std_json_audit = read("docs/std-json-implementation-audit.md");

    for required in [
        "Status: full CLI parser schema design selected",
        "Implementation status: first CLI parser schema slice implemented",
        "cli::parse_or[T](args, defaults)",
        "cli::usage_for[T](program, defaults)",
        "CLI > config > defaults",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Selected Public API",
        "ErrorKind",
        "UnknownArgument",
        "MissingValue",
        "InvalidValue",
        "Validation",
        "UnsupportedTarget",
        "Supported First Targets",
        "Preserved overlay-only field types",
        "Rejected targets",
        "Strict no-default parsing",
        "Naming And Metadata",
        "@cli(name: \"server-host\")",
        "@cli(positional: 1)",
        "Do not reuse `@json(alias: \"...\")`",
        "Argument Semantics",
        "--name value",
        "--name=false",
        "UnknownArgument",
        "Positionals",
        "Enums",
        "Lists",
        "Validation",
        "Generated Usage",
        "usage_for(program, defaults)",
        "Schema And Artifacts",
        "CliSchema",
        "Diagnostics",
        "Candidates Compared",
        "`cli::parse_or[T](args, defaults)` overlay parser",
        "`cli::usage_for[T](program, defaults)`",
        "Strict `cli::parse[T](args)`",
        "Explicit schema records built by users",
        "Runtime reflection",
        "Reuse `@json(alias)`",
        "Full subcommands",
        "Environment variable and config discovery integration",
        "Multi-error accumulation",
        "Short flags",
        "Non-Goals",
        "Implemented First Slice",
        "schema payload propagation through typed HIR, MIR, bytecode, `.mgb`",
        "Implementation Plan",
        "Done: implement the smallest CLI parser schema slice",
        "Done: audit the first implementation",
        "Done: refresh the config-app sample and template",
        "Done: expose generated config-app usage",
        "Done: design strict `cli::parse[T](args)`",
        "strict-cli-parser-schema.md",
        "MissingArgument",
        "Done: audit strict CLI parser adoption",
        "Done: implement a checked-in strict CLI tool sample",
        "Done: audit strict CLI tool sample adoption",
        "Done: implement generated `muga new --template cli-tool` adoption",
        "Done: audit generated cli-tool template adoption",
        "Done: implement strict CLI manual help adoption",
    ] {
        assert!(
            design.contains(required),
            "CLI parser schema design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("typed encoding design", json_typed.as_str()),
        ("strategy", strategy.as_str()),
        ("std json audit", std_json_audit.as_str()),
    ] {
        assert!(
            text.contains("cli-parser-schema.md")
                && text.contains("cli::parse_or")
                && text.contains("cli::usage_for")
                && text.contains("TOML"),
            "{label} must reference the CLI parser schema design"
        );
    }

    assert!(
        implementation_resume.contains("| 231. full CLI parser schema design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 232. first CLI parser schema implementation |")
            && implementation_resume.contains(
                "| std_package/typing/mir/bytecode/artifacts/runtime/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 233. post-CLI parser schema adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 234. generated config-app CLI schema adoption |")
            && implementation_resume.contains("| project template/samples/tests/docs | Done |")
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark CLI parser schema design, implementation, and post-CLI parser selection done, then queue generated config-app adoption"
    );
    assert!(
        implementation_resume.contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation snapshot must point at generated config-app CLI schema adoption next"
    );
}

#[test]
fn cli_parser_schema_implementation_is_covered() {
    let std_package = read("src/std_package.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let mir = read("src/mir.rs");
    let bytecode = read("src/bytecode.rs");
    let artifact = read("src/implementation_artifact.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/cli-parser-schema.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");

    for required in [
        "pub const CLI_ERROR_KIND_MANGLED_NAME",
        "pub const CLI_ERROR_MANGLED_NAME",
        "pub enum ErrorKind",
        "pub record Error",
        "pub fn parse_or[T](args: List[String], defaults: T): Result[T, Error]",
        "pub fn usage_for[T](program: String, defaults: T): String",
    ] {
        assert!(
            std_package.contains(required),
            "std::cli package source missing CLI schema evidence `{required}`"
        );
    }

    for required in [
        "cli_parse_or_schemas",
        "cli_usage_for_schemas",
        "std_cli_parse_or_bindings",
        "std_cli_usage_for_bindings",
        "check_std_cli_parse_or_call",
        "check_std_cli_usage_for_call",
        "cli_schema_for_type",
        "cli_field_schema_for_type",
    ] {
        assert!(
            typing.contains(required),
            "typing missing CLI schema lowering evidence `{required}`"
        );
    }

    for (label, text, required) in [
        ("typed HIR", typed_hir.as_str(), "cli_parse_or_schema"),
        ("typed HIR", typed_hir.as_str(), "cli_usage_for_schema"),
        ("MIR", mir.as_str(), "CliParseOrExpr"),
        ("MIR", mir.as_str(), "CliUsageForExpr"),
        ("bytecode", bytecode.as_str(), "Instruction::CliParseOr"),
        ("bytecode", bytecode.as_str(), "Instruction::CliUsageFor"),
        ("artifact", artifact.as_str(), "invalid CLI parser schema"),
        ("artifact", artifact.as_str(), "invalid CLI usage schema"),
        ("runtime", runtime.as_str(), "enum CliErrorKind"),
        ("runtime", runtime.as_str(), "fn cli_parse_or"),
        ("runtime", runtime.as_str(), "fn cli_usage_for"),
        ("runtime", runtime.as_str(), "Instruction::CliParseOr"),
        ("runtime", runtime.as_str(), "Instruction::CliUsageFor"),
    ] {
        assert!(
            text.contains(required),
            "{label} missing CLI parser implementation evidence `{required}`"
        );
    }

    for required in [
        "standard_cli_parse_or_record_overlay_runs",
        "standard_cli_parse_or_reports_recoverable_errors",
        "standard_cli_usage_for_record_runs",
        "standard_cli_parse_or_artifact_run_uses_schema_payload",
        ".arg(\"--built\")",
        "standard_cli_parse_or_rejects_unsupported_targets",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing CLI parser schema coverage `{required}`"
        );
    }

    for (label, text) in [
        ("CLI schema design", design.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
    ] {
        assert!(
            text.contains("cli::parse_or")
                && text.contains("cli::usage_for")
                && text.contains("CLI parser schema")
                && text.contains("first"),
            "{label} must document the implemented CLI parser schema slice"
        );
    }

    assert!(
        implementation_resume.contains("| 232. first CLI parser schema implementation |")
            && implementation_resume.contains(
                "| std_package/typing/mir/bytecode/artifacts/runtime/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 233. post-CLI parser schema adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 234. generated config-app CLI schema adoption |")
            && implementation_resume.contains("| project template/samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 235. post-config-app CLI schema adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark CLI parser implementation, post-implementation audit, and generated config-app adoption done"
    );
}

#[test]
fn generated_config_app_cli_schema_adoption_is_implemented_and_covered() {
    let sample = read("samples/projects/config_app/src/main/main.muga");
    let template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let by_example = read("docs/muga-by-example.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");

    for required in [
        "fn cli_error_message(error: cli::Error): String",
        "fn settings_args(args: List[String]): List[String]",
        "config_value_follows(args, index)",
        "cli::option_or(args, \"config\"",
        "cli::parse_or(settings_args(args), configured)",
        "cli::ErrorKind::UnknownArgument",
        "cli::ErrorKind::UnsupportedTarget",
        "Result::Ok(rendered)",
    ] {
        assert!(
            sample.contains(required),
            "config app sample missing CLI schema adoption evidence `{required}`"
        );
        assert!(
            template.contains(required),
            "generated config app template missing CLI schema adoption evidence `{required}`"
        );
    }

    for removed in [
        "fn apply_args",
        "cli::option_int_or(args, \"port\"",
        "cli::option_bool_or(args, \"verbose\"",
        "cli::option_values_or(args, \"tag\", settings.tags)",
    ] {
        assert!(
            !sample.contains(removed) && !template.contains(removed),
            "config app CLI schema adoption should remove `{removed}`"
        );
    }

    for required in [
        "cli_new_creates_app_lib_and_test_templates",
        "manifest_config_project_sample_runs_with_cli_overrides",
        "manifest_config_project_sample_reports_cli_parse_errors",
        "manifest_config_project_sample_json_built_run_applies_cli_overrides",
        "cli::parse_or(settings_args(args), configured)",
        "--tags=ops",
        "--tag=runtime",
        "Result::Ok(Grace|9090|true|1|ops|none|2|9000|4)",
        "Result::Ok(Kai|5050|true|1|unknown|none|1|8080|1)",
    ] {
        assert!(
            examples.contains(required),
            "examples coverage missing config-app CLI schema adoption `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
    ] {
        assert!(
            text.contains("config-app")
                && text.contains("cli::parse_or")
                && text.contains("CLI > config > defaults"),
            "{label} must document config-app CLI schema adoption"
        );
    }

    assert!(
        implementation_resume.contains("| 234. generated config-app CLI schema adoption |")
            && implementation_resume.contains("| project template/samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 235. post-config-app CLI schema adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 236. generated config-app usage adoption |")
            && implementation_resume.contains("| project template/samples/tests/docs | Done |")
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark generated config-app CLI schema adoption and its follow-up audit done"
    );
}

#[test]
fn generated_config_app_usage_adoption_is_implemented_and_covered() {
    let sample = read("samples/projects/config_app/src/main/main.muga");
    let template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let by_example = read("docs/muga-by-example.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");

    for required in [
        "cli::parse_request_or(settings_args(args), \"config-app\", default_settings())",
        "cli::Request::Help(usage)",
        "cli::Request::Parsed(_)",
        "fn emit_usage(usage: String): Result[String, String]",
        "fn run_config(args: List[String]): Result[String, String]",
        "--config <Path>  default:",
    ] {
        assert!(
            sample.contains(required),
            "config app sample missing usage adoption evidence `{required}`"
        );
        assert!(
            template.contains(required),
            "generated config app template missing usage adoption evidence `{required}`"
        );
    }

    for required in [
        "cli_new_creates_app_lib_and_test_templates",
        "manifest_config_project_sample_reports_usage",
        "manifest_config_project_sample_json_built_run_applies_cli_overrides",
        "Usage: config-app [options]",
        "--config <Path>  default: $MUGA_CONFIG_PATH or config/settings.json",
        "--config <Path>  default: $MUGA_CONFIG_PATH or samples/projects/config_app/config/settings.json",
    ] {
        assert!(
            examples.contains(required),
            "examples coverage missing generated config-app usage adoption `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
    ] {
        assert!(
            text.contains("config-app")
                && (text.contains("cli::usage_for") || text.contains("cli::help_for"))
                && text.contains("--help"),
            "{label} must document generated config-app usage adoption"
        );
    }

    assert!(
        implementation_resume.contains("| 236. generated config-app usage adoption |")
            && implementation_resume.contains("| project template/samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 237. post-config-app usage adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 238. first `@cli(...)` field metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 239. first `@cli(...)` field metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/cli_schema/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark generated config-app usage adoption done and queue the next audit"
    );
}

#[test]
fn generated_config_app_path_discovery_is_documented_and_covered() {
    let design = read("docs/config-path-discovery.md");
    let sample = read("samples/projects/config_app/src/main/main.muga");
    let template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let diagnostics = read("docs/diagnostics-and-output.md");
    let onboarding = read("docs/installation-and-onboarding.md");
    let std_config = read("docs/std-config-json-loading.md");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: generated config apps now support explicit environment-backed config",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "--config <path>",
        "MUGA_CONFIG_PATH",
        "config/settings.json",
        "CLI setting fields override the decoded config file",
        "Candidates Compared",
        "`--config` > `MUGA_CONFIG_PATH` > generated JSON path",
        "Implement TOML parsing first",
        "Defer",
        "Done: add `discovered_config_path()`",
        "Next: defer TOML",
    ] {
        assert!(
            design.contains(required),
            "config path discovery doc missing `{required}`"
        );
    }

    for required in [
        "fn discovered_config_path(): String",
        "env::get_var(\"MUGA_CONFIG_PATH\")",
        "cli::option_or(args, \"config\", discovered_config_path())",
        "$MUGA_CONFIG_PATH or",
    ] {
        assert!(
            sample.contains(required),
            "config app sample missing path discovery evidence `{required}`"
        );
        assert!(
            template.contains(required),
            "generated config app template missing path discovery evidence `{required}`"
        );
    }

    for required in [
        "manifest_config_project_sample_uses_env_config_path_default",
        ".env(\"MUGA_CONFIG_PATH\"",
        "fn discovered_config_path(): String",
        "--config <Path>  default: $MUGA_CONFIG_PATH or config/settings.json",
        "--config <Path>  default: $MUGA_CONFIG_PATH or samples/projects/config_app/config/settings.json",
    ] {
        assert!(
            examples.contains(required),
            "examples missing config path discovery coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("diagnostics", diagnostics.as_str()),
        ("onboarding", onboarding.as_str()),
        ("std config", std_config.as_str()),
    ] {
        assert!(
            text.contains("config-path-discovery.md") || text.contains("MUGA_CONFIG_PATH"),
            "{label} must surface config path discovery"
        );
    }

    assert!(
        implementation_resume.contains("| 290. Generated config-app path discovery |")
            && implementation_resume.contains("samples/templates/tests/docs | Done")
            && implementation_resume.contains(
                "Next recommended slice: design installed-app resource layout and launcher boundary"
            ),
        "implementation queue must cover generated config path discovery"
    );
}

#[test]
fn workspace_manifest_metadata_is_documented_and_covered() {
    let design = read("docs/workspace-manifest-metadata.md");
    let package = read("src/package.rs");
    let cli = read("src/main.rs");
    let examples = read("tests/examples.rs");
    let diagnostics = read("docs/diagnostics-and-output.md");
    let editor_workflow = read("docs/editor-json-workflow.md");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `muga workspace --format json` now reports manifest roots",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "\"project\"",
        "\"manifest\"",
        "\"sourceRoot\"",
        "\"directDependencies\"",
        "\"sourceKind\"",
        "Candidates Compared",
        "Extend `workspace --format json` with manifest metadata",
        "Implement TOML config parsing",
        "Add runtime package resource lookup",
        "Reject",
        "Done: add `project_manifest_metadata_from_entry()`",
        "\"resourceRoot\"",
        "Done: evaluate package resource inclusion",
    ] {
        assert!(
            design.contains(required),
            "workspace manifest metadata doc missing `{required}`"
        );
    }

    for required in [
        "pub struct ProjectManifestMetadata",
        "pub struct ProjectManifestDependencyMetadata",
        "pub fn project_manifest_metadata_from_entry",
        "project_manifest_dependency_metadata",
        "resource_root",
    ] {
        assert!(
            package.contains(required),
            "package module missing manifest metadata evidence `{required}`"
        );
    }

    for required in [
        "push_workspace_project_json",
        "push_workspace_project_dependency_json",
        "ProjectManifestMetadata",
        "ProjectManifestDependencyMetadata",
        "direct_dependencies",
        "source_kind.as_str()",
        "push_optional_path_ref_json",
    ] {
        assert!(
            cli.contains(required),
            "CLI workspace JSON missing manifest metadata evidence `{required}`"
        );
    }

    for required in [
        "project_manifest_metadata_reports_roots_and_dependency_sources",
        r#"\"packagePath\":\"workspace_app\""#,
        r#"\"directDependencies\":[\"workspace_shared\"]"#,
        r#"\"resourceRoot\":{\"path\":\""#,
        r#"\"sourceKind\":\"path\""#,
        r#"\"source\":\"../shared\""#,
    ] {
        assert!(
            examples.contains(required),
            "examples missing workspace manifest metadata coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("practical", practical.as_str()),
        ("diagnostics", diagnostics.as_str()),
        ("editor workflow", editor_workflow.as_str()),
    ] {
        assert!(
            text.contains("workspace-manifest-metadata.md")
                || text.contains("manifest root")
                || text.contains("source root"),
            "{label} must surface workspace manifest metadata"
        );
    }

    assert!(
        implementation_resume.contains("| 291. Workspace manifest metadata |")
            && implementation_resume.contains("package/main/tests/docs | Done")
            && implementation_resume.contains(
                "Next recommended slice: design installed-app resource layout and launcher boundary"
            ),
        "implementation queue must cover workspace manifest metadata"
    );
}

#[test]
fn generated_config_app_run_helper_is_documented_and_covered() {
    let design = read("docs/config-app-run-helper.md");
    let template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let config_path = read("docs/config-path-discovery.md");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let onboarding = read("docs/installation-and-onboarding.md");
    let by_example = read("docs/muga-by-example.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: generated `config-app` projects now include non-mutating",
        "scripts/run-with-config.sh",
        "scripts/package-config-app.sh",
        "MUGA_CONFIG_PATH",
        "MUGA_BIN",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Candidates Compared",
        "Add generated `scripts/run-with-config.sh`",
        "Add generated `scripts/package-config-app.sh`",
        "MUGA_INSTALL_DIR",
        "explicit-bin install/list",
        "source-free app bundle",
        "app completion package emission",
        "Generate a jq-based workspace JSON wrapper",
        "Reject",
        "Done: add `README.md`",
        "Done: add `scripts/package-config-app.sh`",
        "Done: package resource inclusion is implemented",
        "Done: read-only runtime resource lookup",
    ] {
        assert!(
            design.contains(required),
            "config app run helper doc missing `{required}`"
        );
    }

    for required in [
        "relative: \"README.md\"",
        "relative: \"scripts/run-with-config.sh\"",
        "relative: \"scripts/package-config-app.sh\"",
        "MUGA_BIN=${MUGA_BIN:-muga}",
        "MUGA_CONFIG_PATH=${MUGA_CONFIG_PATH:-\"$project_dir/config/settings.json\"}",
        "\"$MUGA_BIN\" run \"$project_dir/src/main/main.muga\" -- \"$@\"",
        "MUGA_CONFIG_PATH=\"$config_path\" \"$MUGA_BIN\" run-app-bundle \"$bundle_dir\" -- --tag packaged",
        "\"$MUGA_BIN\" emit-app-completions --format json --output-dir \"$completions_dir\" --program \"$program\" --type Settings \"$bundle_dir\"",
        "\"$MUGA_BIN\" install-app --replace-owned --output-dir \"$MUGA_INSTALL_DIR\" --program \"$program\" \"$bundle_dir\"",
        "\"$MUGA_BIN\" list-installed-apps --output-dir \"$MUGA_INSTALL_DIR\"",
    ] {
        assert!(
            template.contains(required),
            "config-app template missing run helper evidence `{required}`"
        );
    }

    for required in [
        "sh scripts/run-with-config.sh --tag ops",
        "sh scripts/package-config-app.sh",
        "generated config helper script should run",
        "generated config package script should run",
        "MUGA_BIN",
        "MUGA_CONFIG_PATH",
        "MUGA_INSTALL_DIR",
        "config Ada|6061|false|2|ops|none|2|9000|4",
        "config Ada|4040|false|1|ops|none|2|9000|4",
        "dist/completions/config-app.completions.json",
    ] {
        assert!(
            examples.contains(required),
            "examples missing config-app run helper coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("practical", practical.as_str()),
        ("onboarding", onboarding.as_str()),
        ("muga by example", by_example.as_str()),
        ("config path discovery", config_path.as_str()),
    ] {
        assert!(
            text.contains("config-app-run-helper.md")
                || text.contains("scripts/run-with-config.sh"),
            "{label} must surface generated config app run helper"
        );
    }

    assert!(
        implementation_resume.contains("| 292. Generated config-app run helper |")
            && implementation_resume.contains("| 328. Generated config-app package helper |")
            && implementation_resume.contains("templates/tests/docs | Done")
            && implementation_resume.contains(
                "Next recommended slice: design installed-app resource layout and launcher boundary"
            ),
        "implementation queue must cover generated config app run helper"
    );
}

#[test]
fn package_resource_archives_are_documented_and_covered() {
    let design = read("docs/package-resource-archives.md");
    let package = read("src/package.rs");
    let main = read("src/main.rs");
    let examples = read("tests/examples.rs");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let packages_spec = read("spec/006-packages.md");
    let registry = read("docs/registry-security-design.md");
    let fuzzing = read("docs/fuzzing-malformed-input-plan.md");
    let diagnostics = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: manifest-declared package resources, including binary files",
        "[package] resources = \"resources\"",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "Candidates Compared",
        "Manifest-declared `resources = \"resources\"`",
        "Binary resource archive bytes",
        "Non-mutating package archive verification",
        "Runtime package resource lookup",
        "Non-Goals",
        "Validation",
        "Next",
    ] {
        assert!(
            design.contains(required),
            "package resource archives doc missing `{required}`"
        );
    }

    for required in [
        "resource_root: Option<PathBuf>",
        "pub resources: Vec<PackageArchiveResourceEntry>",
        "pub struct PackageArchiveResourceEntry",
        "pub struct PackageArchiveVerifyOutput",
        "pub fn verify_package_archive",
        "pub fn verify_package_archive_with_expected_hash",
        "expected_package_archive_hash_from_path",
        "validate_manifest_resource_dir",
        "validate_package_archive_resource_path",
        "collect_package_resource_files",
        "package_archive_manifest_resource_dir",
        "resource\\t{}\\t{}\\n",
        "package archive resource entries require [package] resources",
        "package_archive_dependency_cache_content_hash",
    ] {
        assert!(
            package.contains(required),
            "package module missing resource archive evidence `{required}`"
        );
    }

    for required in ["resourceRoot", "push_optional_path_ref_json"] {
        assert!(
            main.contains(required),
            "workspace JSON missing resource root evidence `{required}`"
        );
    }
    for required in [
        "Mode::VerifyPackageArchive",
        "Mode::UnpackPackageArchive",
        "verify-package-archive",
        "unpack-package-archive",
        "--expected-hash",
        "unpack-package-archive requires --output-dir",
        "package_archive_verify_json_output",
        "package_archive_unpack_json_output",
        "package_archive_unpack_diagnostic_json_output",
        "packageArchive",
    ] {
        assert!(
            main.contains(required),
            "package archive verify CLI missing `{required}`"
        );
    }

    for required in [
        "package_content_hash_covers_manifest_declared_resources",
        "package_archive_emission_includes_manifest_declared_resources",
        "package_archive_preserves_binary_manifest_resources",
        "cli_verify_package_archive_validates_hash_from_filename",
        "package_archive_readback_rejects_malformed_entries",
        "package_archive_materialization_rejects_unsafe_manifest_resource_roots",
        "manifest_local_archive_dependency_materializes_declared_resources_and_validates_cache_hash",
        "standard_fs_read_resource_bytes_reads_manifest_entry_resources_for_source_and_built_runs",
        "standard_fs_read_resource_bytes_reads_archive_dependency_resources_from_cache",
        "resource entries require [package] resources",
        r#"\"resourceRoot\":{\"path\":\""#,
    ] {
        assert!(
            examples.contains(required),
            "examples missing resource archive coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("practical", practical.as_str()),
        ("packages spec", packages_spec.as_str()),
        ("registry", registry.as_str()),
        ("fuzzing", fuzzing.as_str()),
        ("diagnostics", diagnostics.as_str()),
    ] {
        assert!(
            text.contains("package-resource-archives.md")
                || text.contains("resourceRoot")
                || text.contains("source/resource"),
            "{label} must surface package resource archives"
        );
    }

    assert!(
        implementation_resume.contains("| 293. Package resource archives |")
            && implementation_resume.contains("package/main/tests/docs | Done")
            && implementation_resume.contains("| 300. Package archive verification/unpack CLI |")
            && implementation_resume.contains(
                "Next recommended slice: design installed-app resource layout and launcher boundary"
            ),
        "implementation queue must cover package resource archives"
    );
}

#[test]
fn runtime_package_resource_lookup_is_documented_and_covered() {
    let design = read("docs/runtime-package-resource-lookup.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let lib = read("src/lib.rs");
    let examples = read("tests/examples.rs");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let packages_spec = read("spec/006-packages.md");
    let typing_spec = read("spec/003-typing.md");
    let package_archives = read("docs/package-resource-archives.md");
    let registry = read("docs/registry-security-design.md");
    let fuzzing = read("docs/fuzzing-malformed-input-plan.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: read-only runtime package resource lookup is implemented",
        "std::fs::read_resource_text(package_path, resource_path)",
        "std::fs::read_resource_bytes(package_path, resource_path)",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "Candidates Compared",
        "`fs::read_resource_text(package, path)`",
        "`fs::read_resource_bytes(package, path)` plus opaque `std::bytes::Bytes`",
        "`fs::resource_path(package, path): path::Path`",
        "Non-Goals",
        "Validation",
        "Next",
    ] {
        assert!(
            design.contains(required),
            "runtime package resource lookup doc missing `{required}`"
        );
    }

    for required in [
        "FS_READ_RESOURCE_TEXT_BUILTIN",
        "FS_READ_RESOURCE_BYTES_BUILTIN",
        "BYTES_PACKAGE",
        "pub fn read_resource_text(package_path: String, resource_path: String)",
        "pub fn read_resource_bytes(package_path: String, resource_path: String)",
        "pub opaque type Bytes",
        "pub fn size(bytes: Bytes): Int",
        "pub fn empty(bytes: Bytes): Bool",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing resource lookup evidence `{required}`"
        );
    }

    for required in [
        "StdFsReadResourceText",
        "StdFsReadResourceBytes",
        "StdBytesSize",
        "StdBytesIsEmpty",
        "FS_READ_RESOURCE_TEXT_BUILTIN",
        "FS_READ_RESOURCE_BYTES_BUILTIN",
        "Builtin(__muga_std_fs_read_resource_text)",
        "Builtin(__muga_std_fs_read_resource_bytes)",
        "Builtin(__muga_std_bytes_size)",
        "Builtin(__muga_std_bytes_is_empty)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing resource lookup evidence `{required}`"
        );
    }

    for required in [
        "check_std_fs_read_resource_text_builtin",
        "check_std_fs_read_resource_bytes_builtin",
        "check_std_bytes_unary_builtin",
        "Type::Builtin(BuiltinId::StdFsReadResourceText)",
        "Type::Builtin(BuiltinId::StdFsReadResourceBytes)",
        "expected 2 arguments",
        "Type::Result(Box::new(Type::String), Box::new(error_ty))",
        "Type::Result(Box::new(bytes_ty), Box::new(error_ty))",
        "bytes::Bytes",
    ] {
        assert!(
            typing.contains(required),
            "typing missing resource lookup evidence `{required}`"
        );
    }

    for required in [
        "run_with_args_and_package_resources",
        "run_package_function_with_args_and_package_resources",
        "package_resource_roots",
        "read_package_resource_text",
        "read_package_resource_bytes",
        "Value::Bytes",
        "validate_runtime_resource_path",
        "resource_display_path",
        "canonicalize",
        "read_resource_text",
        "read_resource_bytes",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing resource lookup evidence `{required}`"
        );
    }

    for required in [
        "package_resource_roots_from_entry",
        "project_manifest_metadata_from_entry",
        "run_discovered_tests_with_package_resources",
        "run_with_args_and_package_resources",
    ] {
        assert!(
            lib.contains(required),
            "library run plumbing missing resource lookup evidence `{required}`"
        );
    }

    for required in [
        "standard_fs_read_resource_text_reads_manifest_entry_resources_for_source_and_built_runs",
        "standard_fs_read_resource_text_reads_archive_dependency_resources_from_cache",
        "standard_fs_read_resource_text_reports_invalid_paths_and_missing_roots",
        "standard_fs_read_resource_text_is_available_to_package_tests",
        "standard_fs_read_resource_bytes_reads_manifest_entry_resources_for_source_and_built_runs",
        "standard_fs_read_resource_bytes_reads_archive_dependency_resources_from_cache",
        "read_resource_text",
        "read_resource_bytes",
        "bytes::size",
        "bytes::empty",
        "InvalidInput",
        "NotFound",
    ] {
        assert!(
            examples.contains(required),
            "examples missing runtime resource lookup coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("practical", practical.as_str()),
        ("packages spec", packages_spec.as_str()),
        ("typing spec", typing_spec.as_str()),
        ("package archives", package_archives.as_str()),
        ("registry", registry.as_str()),
        ("fuzzing", fuzzing.as_str()),
    ] {
        assert!(
            text.contains("runtime-package-resource-lookup.md")
                || text.contains("read_resource_text")
                || text.contains("runtime resource lookup"),
            "{label} must surface runtime package resource lookup"
        );
    }

    assert!(
        implementation_resume.contains("| 294. Runtime package resource lookup |")
            && implementation_resume.contains("std_package/typing/runtime/lib/tests/docs | Done")
            && implementation_resume.contains(
                "Next recommended slice: design installed-app resource layout and launcher boundary"
            ),
        "implementation queue must cover runtime package resource lookup"
    );
}

#[test]
fn binary_file_reads_are_documented_and_covered() {
    let design = read("docs/binary-file-read.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let typing_spec = read("spec/003-typing.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: read-only binary file reads are implemented",
        "std::fs::read_bytes(path)",
        "std::fs::read_bytes_path(path::Path)",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub fn read_bytes(path: String): Result[bytes::Bytes, io::IOError]",
        "pub fn read_bytes_path(file_path: path::Path): Result[bytes::Bytes, io::IOError]",
        "pub fn at(bytes: Bytes, index: Int): Option[Int]",
        "Candidates Compared",
        "`fs::read_bytes(path)` and `fs::read_bytes_path(path::Path)`",
        "`bytes::at(bytes, index): Option[Int]`",
        "Non-Goals",
        "Validation",
    ] {
        assert!(
            design.contains(required),
            "binary file read doc missing `{required}`"
        );
    }

    for required in [
        "FS_READ_BYTES_BUILTIN",
        "BYTES_AT_BUILTIN",
        "pub fn read_bytes(path: String): Result[bytes::Bytes, io::IOError]",
        "pub fn read_bytes_path(file_path: path::Path): Result[bytes::Bytes, io::IOError]",
        "pub fn at(bytes: Bytes, index: Int): Option[Int]",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing binary read evidence `{required}`"
        );
    }

    for required in [
        "StdFsReadBytes",
        "StdBytesAt",
        "FS_READ_BYTES_BUILTIN",
        "BYTES_AT_BUILTIN",
        "Builtin(__muga_std_fs_read_bytes)",
        "Builtin(__muga_std_bytes_at)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing binary read evidence `{required}`"
        );
    }

    for required in [
        "check_std_fs_read_bytes_builtin",
        "check_std_bytes_at_builtin",
        "Type::Builtin(BuiltinId::StdFsReadBytes)",
        "Type::Builtin(BuiltinId::StdBytesAt)",
        "Type::Result(Box::new(bytes_ty), Box::new(error_ty))",
        "Type::Option(Box::new(Type::Int))",
        "bytes::at",
    ] {
        assert!(
            typing.contains(required),
            "typing missing binary read evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdFsReadBytes",
        "BuiltinId::StdBytesAt",
        "fs::read(&path)",
        "io_error_value(\"read_bytes\"",
        "usize::try_from(index)",
        "option_some(Value::Int",
        "option_none",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing binary read evidence `{required}`"
        );
    }

    for required in [
        "standard_fs_read_bytes_reads_file_and_indexes_bytes_for_source_and_built_runs",
        "fs::read_bytes",
        "fs::read_bytes_path",
        "bytes::at",
        "Result::Ok(3|0|255|none|3)",
    ] {
        assert!(
            examples.contains(required),
            "examples missing binary read coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("typing spec", typing_spec.as_str()),
        ("strategy", strategy.as_str()),
    ] {
        assert!(
            text.contains("binary-file-read.md")
                || text.contains("read_bytes")
                || text.contains("binary writes/streams"),
            "{label} must surface binary file reads"
        );
    }

    assert!(
        implementation_resume.contains("| 301. Read-only binary file reads |")
            && implementation_resume.contains("std_package/typing/runtime/tests/docs | Done")
            && implementation_resume.contains("bytes::at"),
        "implementation queue must cover read-only binary file reads"
    );
}

#[test]
fn binary_file_writes_are_documented_and_covered() {
    let design = read("docs/binary-file-write.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_write_bytes/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let typing_spec = read("spec/003-typing.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::write_bytes(path, data)` and",
        "`std::fs::write_bytes_path(file_path, data)` are implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub fn write_bytes(path: String, data: bytes::Bytes): Result[Unit, io::IOError]",
        "pub fn write_bytes_path(file_path: path::Path, data: bytes::Bytes): Result[Unit, io::IOError]",
        "operation = \"write_bytes\"",
        "Candidates Compared",
        "`fs::write_bytes(path, data)` plus `fs::write_bytes_path(file_path, data)`",
        "Non-Goals",
        "Validation",
        "standard_fs_write_bytes_writes_binary_file_for_source_and_built_runs",
        "standard_fs_write_bytes_missing_parent_returns_io_error",
        "package_std_fs_write_bytes_sample_runs",
    ] {
        assert!(
            design.contains(required),
            "binary file write doc missing `{required}`"
        );
    }

    for required in [
        "FS_WRITE_BYTES_BUILTIN",
        "pub fn write_bytes(path: String, data: bytes::Bytes): Result[Unit, io::IOError]",
        "pub fn write_bytes_path(file_path: path::Path, data: bytes::Bytes): Result[Unit, io::IOError]",
        "__muga_std_fs_write_bytes(path, data)",
        "__muga_std_fs_write_bytes(path::as_string(file_path), data)",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing binary write evidence `{required}`"
        );
    }

    for required in [
        "StdFsWriteBytes",
        "FS_WRITE_BYTES_BUILTIN",
        "Builtin(__muga_std_fs_write_bytes)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing binary write evidence `{required}`"
        );
    }

    for required in [
        "check_std_fs_write_bytes_builtin",
        "Type::Builtin(BuiltinId::StdFsWriteBytes)",
        "self.check_expr_with_expected(&expr.args[1], Some(bytes_ty))",
        "Type::Result(Box::new(Type::Unit), Box::new(error_ty))",
    ] {
        assert!(
            typing.contains(required),
            "typing missing binary write evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdFsWriteBytes",
        "Value::Bytes(data)",
        "fs::write(&path, data)",
        "io_error_value(\"write_bytes\"",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing binary write evidence `{required}`"
        );
    }

    for required in [
        "standard_fs_write_bytes_writes_binary_file_for_source_and_built_runs",
        "standard_fs_write_bytes_missing_parent_returns_io_error",
        "fs::write_bytes",
        "fs::write_bytes_path",
        "Result::Ok(3|3)",
        "Result::Ok(write_bytes)",
    ] {
        assert!(
            examples.contains(required),
            "examples missing binary write coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_write_bytes",
        "import std::bytes",
        "import std::env",
        "import std::fs",
        "fs::write_bytes_path(target, data)",
        "fs::read_bytes_path(target)",
        "fs::remove_file_path(target)",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_write_bytes sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("typing spec", typing_spec.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("stdlib review", stdlib_review.as_str()),
    ] {
        assert!(
            text.contains("binary-file-write.md")
                || text.contains("write_bytes")
                || text.contains("std_fs_write_bytes"),
            "{label} must surface binary file writes"
        );
    }

    assert!(
        implementation_resume.contains("| 322. Binary file write helpers |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("full-file writes"),
        "implementation queue must cover binary file writes"
    );
}

#[test]
fn bytes_sha256_hash_is_documented_and_covered() {
    let design = read("docs/bytes-sha256-hash.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let package = read("src/package.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_hash/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let typing_spec = read("spec/003-typing.md");
    let binary_read = read("docs/binary-file-read.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::hash::sha256_hex(bytes)` is implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub fn sha256_hex(bytes: bytes::Bytes): String",
        "64 lowercase hexadecimal characters",
        "Candidates Compared",
        "`hash::sha256_hex(bytes): String`",
        "Non-Goals",
        "Validation",
    ] {
        assert!(
            design.contains(required),
            "bytes SHA-256 hash doc missing `{required}`"
        );
    }

    for required in [
        "HASH_PACKAGE",
        "HASH_SHA256_HEX_BUILTIN",
        "package std::hash",
        "pub fn sha256_hex(bytes: bytes::Bytes): String",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing SHA-256 evidence `{required}`"
        );
    }

    for required in [
        "StdHashSha256Hex",
        "HASH_SHA256_HEX_BUILTIN",
        "Builtin(__muga_std_hash_sha256_hex)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing SHA-256 evidence `{required}`"
        );
    }

    for required in [
        "check_std_hash_sha256_hex_builtin",
        "Type::Builtin(BuiltinId::StdHashSha256Hex)",
        "hash::sha256_hex",
        "self.apply_expected(Type::String, expected, expr.span)",
    ] {
        assert!(
            typing.contains(required),
            "typing missing SHA-256 evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdHashSha256Hex",
        "__muga_std_hash_sha256_hex",
        "crate::package::sha256_hex(&bytes)",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing SHA-256 evidence `{required}`"
        );
    }

    assert!(
        package.contains("pub(crate) fn sha256_hex(input: &[u8]) -> String"),
        "package SHA-256 helper must be reusable by std::hash"
    );

    for required in [
        "standard_hash_sha256_hex_hashes_read_bytes_for_source_and_built_runs",
        "package_std_hash_sample_runs",
        "import std::hash",
        "hash::sha256_hex(bytes)",
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
    ] {
        assert!(
            examples.contains(required),
            "examples missing SHA-256 coverage `{required}`"
        );
    }

    for required in [
        "package app::std_hash",
        "import std::bytes",
        "import std::fs",
        "import std::hash",
        "fs::read_bytes",
        "bytes::at",
        "hash::sha256_hex(data)",
    ] {
        assert!(
            sample.contains(required),
            "std_hash sample missing SHA-256 bytes usage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("typing spec", typing_spec.as_str()),
        ("binary read", binary_read.as_str()),
    ] {
        assert!(
            text.contains("bytes-sha256-hash.md")
                || text.contains("sha256_hex")
                || text.contains("broader cryptographic APIs"),
            "{label} must surface bytes SHA-256 hashing"
        );
    }

    assert!(
        implementation_resume.contains("| 302. Bytes SHA-256 hash |")
            && implementation_resume.contains("std_package/typing/runtime/tests/docs | Done")
            && implementation_resume.contains("std::hash::sha256_hex"),
        "implementation queue must cover bytes SHA-256 hashing"
    );
}

#[test]
fn resource_bytes_export_sample_is_documented_and_covered() {
    let design = read("docs/resource-bytes-export-sample.md");
    let manifest = read("samples/projects/resource_export/muga.toml");
    let sample = read("samples/projects/resource_export/src/main/main.muga");
    let resource = read("samples/projects/resource_export/resources/static/payload.bin");
    let examples = read("tests/examples.rs");
    let by_example = read("docs/muga-by-example.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `samples/projects/resource_export` is implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Selected Shape",
        "fs::path_metadata_path",
        "Candidates Compared",
        "Project sample over existing APIs",
        "Generated resource-export starter",
        "New resource export helper",
        "Binary stream/resource handles",
        "Docs-only mention",
        "Validation",
        "manifest_resource_export_project_sample_runs",
        "manifest_resource_export_project_sample_runs_against_emitted_artifacts",
        "manifest_resource_export_source_free_bundle_runs_without_sources",
        "cli_new_creates_resource_export_template",
        "resource_bytes_export_sample_is_documented_and_covered",
    ] {
        assert!(
            design.contains(required),
            "resource export design missing `{required}`"
        );
    }

    for required in [
        "name = \"resource_export\"",
        "source = \"src\"",
        "resources = \"resources\"",
    ] {
        assert!(
            manifest.contains(required),
            "resource export manifest missing `{required}`"
        );
    }

    for required in [
        "import std::bytes",
        "import std::cli",
        "import std::env",
        "import std::fs",
        "import std::hash",
        "fs::read_resource_bytes(\"resource_export\", \"static/payload.bin\")",
        "hash::sha256_hex(data)",
        "fs::write_bytes_path(target, data)",
        "metadata = try fs::path_metadata_path(target)",
        "fs::read_bytes_path(target)",
        "fs::remove_file_path(target)",
        "fn kind_name(kind: fs::PathKind): String",
        "cli::positional_or(env::args(), 0, path::as_string(default_path))",
    ] {
        assert!(
            sample.contains(required),
            "resource export sample missing `{required}`"
        );
    }

    assert_eq!(resource, "muga-resource\n");

    for required in [
        "samples/projects/resource_export/src/main/main.muga",
        "muga run samples/projects/resource_export/src/main/main.muga",
        "muga build samples/projects/resource_export/src/main/main.muga",
        "muga run --built samples/projects/resource_export/src/main/main.muga",
        "muga emit-app-bundle --format json --source-free --output-dir ~/tmp/muga-resource-export-bundle",
        "muga run-app-bundle ~/tmp/muga-resource-export-bundle",
        "muga new --template resource-export ~/tmp/muga-example-resource",
        "sh scripts/package-resource-export.sh",
        "resource-bytes-export-sample.md",
        "Result::Ok(14|file|true|e54f8e906eaac9d311ba74b926b071faee0dc5a0036dd5a5e3c2b23b55f39728)",
    ] {
        assert!(
            by_example.contains(required),
            "Muga by Example missing resource export evidence `{required}`"
        );
    }

    for required in [
        "manifest_resource_export_project_sample_runs",
        "manifest_resource_export_project_sample_runs_against_emitted_artifacts",
        "manifest_resource_export_source_free_bundle_runs_without_sources",
        "cli_new_creates_resource_export_template",
        "fs::read_resource_bytes(\\\"asset_export\\\", \\\"static/payload.bin\\\")",
        "fs::path_metadata_path(output)",
        "scripts/package-resource-export.sh",
        "dist/resource-export/.muga/app-bundle",
        "MUGA_INSTALL_DIR",
        "e54f8e906eaac9d311ba74b926b071faee0dc5a0036dd5a5e3c2b23b55f39728",
    ] {
        assert!(
            examples.contains(required),
            "examples missing resource export coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("implementation resume", implementation_resume.as_str()),
    ] {
        assert!(
            text.contains("samples/projects/resource_export")
                || text.contains("resource-bytes-export-sample.md"),
            "{label} must surface resource export sample adoption"
        );
    }

    assert!(
        implementation_resume.contains("| 324. Resource bytes export sample adoption |")
            && implementation_resume.contains("| 330. Generated resource-export template |")
            && implementation_resume.contains("| 335. Resource export PathMetadata adoption |")
            && implementation_resume.contains("samples/tests/docs | Done")
            && implementation_resume.contains("manifest resource byte export")
            && implementation_resume.contains("resource export `PathMetadata` verification"),
        "implementation queue must cover resource bytes export sample adoption"
    );
}

#[test]
fn generated_package_app_template_is_implemented_and_covered() {
    let design = read("docs/generated-package-app-template.md");
    let project_template = read("src/project_template.rs");
    let cli = read("src/main.rs");
    let examples = read("tests/examples.rs");
    let onboarding = read("docs/installation-and-onboarding.md");
    let by_example = read("docs/muga-by-example.md");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let diagnostics = read("docs/diagnostics-and-output.md");
    let practical = read("docs/practical-language-readiness.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: implemented",
        "Short-term",
        "Medium-term",
        "Long-term",
        "Final goal",
        "Candidates Compared",
        "`package-app` generated starter",
        "Rich all-path metadata record",
        "Recursive directory operations",
        "Formatting/interpolation or `std::fmt`",
        "TOML/config discovery",
        "Shell-profile installation or registry publishing",
        "Selected Shape",
        "`muga new --template package-app <dir>`",
        "MUGA_INSTALL_DIR",
        "`app/muga.toml`",
        "`shared/muga.toml`",
        "`scripts/package-package-app.sh`",
        "Validation",
        "cli_new_creates_package_app_template",
        "generated_package_app_template_is_implemented_and_covered",
    ] {
        assert!(
            design.contains(required),
            "generated package-app design missing `{required}`"
        );
    }

    for required in [
        "PackageApp",
        "ProjectTemplate::PackageApp",
        "package_app_template_files",
        "App plus local library package starter",
        "relative: \"app/muga.toml\"",
        "relative: \"app/src/main/main.muga\"",
        "relative: \"shared/muga.toml\"",
        "relative: \"shared/src/greetings/main.muga\"",
        "relative: \"scripts/package-package-app.sh\"",
        "{{shared_package_name}} = { path = \"../shared\" }",
        "import {{shared_package_name}}::greetings",
        "pub record Greeting",
        "emit-app-bundle --source-free --output-dir \"$bundle_dir\" --program \"$program\" \"$entry\"",
        "run-app-bundle \"$bundle_dir\" -- --name=Ada",
        "verify-app-archive \"$archive_path\"",
        "list-installed-apps --output-dir \"$MUGA_INSTALL_DIR\"",
    ] {
        assert!(
            project_template.contains(required),
            "package-app template implementation missing `{required}`"
        );
    }

    for required in [
        "muga new [--template app|lib|test|config-app|cli-tool|report-app|resource-export|package-app]",
        "\"package-app\" | \"package_app\" | \"local-dependency\" | \"local_dependency\" | \"local\"",
        "app lib test config-app cli-tool report-app resource-export package-app",
        "expected `app`, `lib`, `test`, `config-app`, `cli-tool`, `report-app`, `resource-export`, or `package-app`",
    ] {
        assert!(cli.contains(required), "CLI missing `{required}`");
    }

    for required in [
        "cli_new_creates_package_app_template",
        "--template=package-app",
        "local_stack_app",
        "local_stack_shared = { path = \\\"../shared\\\" }",
        "app/src/main/main.muga",
        "shared/src/greetings/main.muga",
        "muga workspace --format json",
        "scripts/package-package-app.sh",
        "dist/package-app/.muga/app-bundle",
        "local_stack_shared__greetings.mgb",
        "MUGA_INSTALL_DIR",
    ] {
        assert!(
            examples.contains(required),
            "examples missing package-app coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("diagnostics", diagnostics.as_str()),
        ("onboarding", onboarding.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("docs README", docs_readme.as_str()),
        ("practical", practical.as_str()),
        ("strategy", strategy.as_str()),
        ("implementation resume", implementation_resume.as_str()),
    ] {
        assert!(
            text.contains("package-app")
                || text.contains("generated-package-app-template.md")
                || text.contains("local package app"),
            "{label} must surface generated package-app adoption"
        );
    }

    assert!(
        implementation_resume.contains("| 331. Generated package-app template |")
            && implementation_resume.contains("| templates/tests/docs | Done |")
            && implementation_resume
                .contains("generated `resource-export` and `package-app` templates"),
        "implementation queue must mark generated package-app template done"
    );
}

#[test]
fn fs_rename_path_is_documented_and_covered() {
    let design = read("docs/fs-rename-path.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_rename/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::rename_path(from_path, to_path)` is implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub fn rename_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]",
        "operation = \"rename\"",
        "Candidates Compared",
        "`fs::rename_path(from, to)`",
        "Non-Goals",
        "Validation",
        "standard_fs_rename_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_rename_sample_runs",
    ] {
        assert!(
            design.contains(required),
            "filesystem rename doc missing `{required}`"
        );
    }

    for required in [
        "FS_RENAME_BUILTIN",
        "__muga_std_fs_rename",
        "pub fn rename_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing filesystem rename evidence `{required}`"
        );
    }

    for required in [
        "StdFsRename",
        "FS_RENAME_BUILTIN",
        "Builtin(__muga_std_fs_rename)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing filesystem rename evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdFsRename",
        "check_std_fs_copy_file_builtin",
        "std_io_path_pair_error_type",
    ] {
        assert!(
            typing.contains(required),
            "typing missing filesystem rename evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdFsRename",
        "expect_two_string_args(args, span, \"__muga_std_fs_rename\")",
        "fs::rename(&from_path, &to_path)",
        "\"rename\", &from_path, &to_path, &error",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing filesystem rename evidence `{required}`"
        );
    }

    for required in [
        "standard_fs_rename_path_moves_file_as_virtual_package",
        "standard_fs_rename_path_missing_source_returns_path_pair_error",
        "standard_fs_rename_path_type_mismatch_reports_expected_path",
        "standard_fs_rename_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_rename_sample_runs",
        "fs::rename_path",
        "__muga_std_fs_rename",
    ] {
        assert!(
            examples.contains(required),
            "examples missing filesystem rename coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_rename",
        "import std::fs",
        "fs::rename_path(source, target)",
        "Result::Err(error) => error.operation",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_rename sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
    ] {
        assert!(
            text.contains("fs-rename-path.md")
                || text.contains("std_fs_rename")
                || text.contains("rename_path"),
            "{label} must surface filesystem rename"
        );
    }

    assert!(
        implementation_resume.contains("| 305. Filesystem rename helper |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("one-step `rename_path`"),
        "implementation queue must cover filesystem rename"
    );
}

#[test]
fn fs_file_size_path_is_documented_and_covered() {
    let design = read("docs/fs-file-size.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_file_size/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::file_size_path(file_path)` is implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub fn file_size_path(file_path: path::Path): Result[Int, io::IOError]",
        "Directories and other non-file paths return",
        "operation = \"file_size\"",
        "Candidates Compared",
        "`fs::file_size_path(path): Result[Int, io::IOError]`",
        "Non-Goals",
        "Validation",
        "standard_fs_file_size_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_file_size_sample_runs",
    ] {
        assert!(
            design.contains(required),
            "filesystem file-size doc missing `{required}`"
        );
    }

    for required in [
        "FS_FILE_SIZE_BUILTIN",
        "__muga_std_fs_file_size",
        "pub fn file_size_path(file_path: path::Path): Result[Int, io::IOError]",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing filesystem file-size evidence `{required}`"
        );
    }

    for required in [
        "StdFsFileSize",
        "FS_FILE_SIZE_BUILTIN",
        "Builtin(__muga_std_fs_file_size)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing filesystem file-size evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdFsFileSize",
        "check_std_fs_file_size_builtin",
        "Type::Result(Box::new(Type::Int), Box::new(error_ty))",
        "std_io_error_type",
    ] {
        assert!(
            typing.contains(required),
            "typing missing filesystem file-size evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdFsFileSize",
        "expect_string_arg(args, span, \"__muga_std_fs_file_size\")",
        "fs::metadata(&path)",
        "metadata.is_file()",
        "metadata.len()",
        "io_error_value(\"file_size\"",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing filesystem file-size evidence `{required}`"
        );
    }

    for required in [
        "standard_fs_file_size_path_runs_as_virtual_package",
        "standard_fs_file_size_path_missing_file_returns_io_error",
        "standard_fs_file_size_path_directory_returns_io_error",
        "standard_fs_file_size_path_type_mismatch_reports_expected_path",
        "standard_fs_file_size_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_file_size_sample_runs",
        "fs::file_size_path",
        "__muga_std_fs_file_size",
    ] {
        assert!(
            examples.contains(required),
            "examples missing filesystem file-size coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_file_size",
        "import std::fs",
        "fs::file_size_path(file_path)",
        "Result::Ok(size) => size.to_string()",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_file_size sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
    ] {
        assert!(
            text.contains("fs-file-size.md")
                || text.contains("std_fs_file_size")
                || text.contains("file_size_path"),
            "{label} must surface filesystem file size"
        );
    }

    assert!(
        implementation_resume.contains("| 306. Filesystem file-size helper |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("scalar `file_size_path`"),
        "implementation queue must cover filesystem file size"
    );
}

#[test]
fn fs_modified_unix_millis_path_is_documented_and_covered() {
    let design = read("docs/fs-modified-unix-millis.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_modified_time/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::modified_unix_millis_path(target_path)` is implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub fn modified_unix_millis_path(target_path: path::Path): Result[time::UnixMillis, io::IOError]",
        "milliseconds after the Unix epoch",
        "operation = \"modified_unix_millis\"",
        "define an all-path public metadata record",
        "Candidates Compared",
        "`fs::modified_unix_millis_path(path): Result[time::UnixMillis, io::IOError]`",
        "Non-Goals",
        "Validation",
        "standard_fs_modified_unix_millis_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_modified_time_sample_runs",
    ] {
        assert!(
            design.contains(required),
            "filesystem modified-time doc missing `{required}`"
        );
    }

    for required in [
        "FS_MODIFIED_UNIX_MILLIS_BUILTIN",
        "__muga_std_fs_modified_unix_millis",
        "import std::time",
        "pub fn modified_unix_millis_path(target_path: path::Path): Result[time::UnixMillis, io::IOError]",
        "time::UnixMillis",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing filesystem modified-time evidence `{required}`"
        );
    }

    for required in [
        "StdFsModifiedUnixMillis",
        "FS_MODIFIED_UNIX_MILLIS_BUILTIN",
        "Builtin(__muga_std_fs_modified_unix_millis)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing filesystem modified-time evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdFsModifiedUnixMillis",
        "check_std_fs_modified_unix_millis_builtin",
        "Type::Result(Box::new(Type::Int), Box::new(error_ty))",
        "std_io_error_type",
    ] {
        assert!(
            typing.contains(required),
            "typing missing filesystem modified-time evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdFsModifiedUnixMillis",
        "expect_string_arg(args, span, \"__muga_std_fs_modified_unix_millis\")",
        "metadata.modified()",
        "duration_since(UNIX_EPOCH)",
        "io_error_value(",
        "\"modified_unix_millis\"",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing filesystem modified-time evidence `{required}`"
        );
    }

    for required in [
        "standard_fs_modified_unix_millis_path_returns_timestamp_record",
        "standard_fs_modified_unix_millis_path_missing_file_returns_io_error",
        "standard_fs_modified_unix_millis_path_type_mismatch_reports_expected_path",
        "standard_fs_modified_unix_millis_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_modified_time_sample_runs",
        "fs::modified_unix_millis_path",
        "__muga_std_fs_modified_unix_millis",
    ] {
        assert!(
            examples.contains(required),
            "examples missing filesystem modified-time coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_modified_time",
        "import std::fs",
        "import std::path",
        "fs::modified_unix_millis_path(target)",
        "Result::Ok(modified) => if modified.value > 0",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_modified_time sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
    ] {
        assert!(
            text.contains("fs-modified-unix-millis.md")
                || text.contains("std_fs_modified_time")
                || text.contains("modified_unix_millis_path"),
            "{label} must surface filesystem modified time"
        );
    }

    assert!(
        implementation_resume.contains("| 314. Filesystem modified Unix milliseconds helper |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("modified_unix_millis_path"),
        "implementation queue must cover filesystem modified time"
    );
}

#[test]
fn fs_file_metadata_record_is_documented_and_covered() {
    let design = read("docs/fs-file-metadata-record.md");
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_file_metadata/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::FileMetadata` and",
        "`std::fs::file_metadata_path(file_path)` are implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub record FileMetadata",
        "size: Int",
        "modified: time::UnixMillis",
        "pub fn file_metadata_path(file_path: path::Path): Result[FileMetadata, io::IOError]",
        "operation = \"file_size\"",
        "operation = \"modified_unix_millis\"",
        "Candidates Compared",
        "`fs::file_metadata_path(path): Result[FileMetadata, io::IOError]`",
        "Non-Goals",
        "Validation",
        "standard_fs_file_metadata_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_file_metadata_sample_runs",
    ] {
        assert!(
            design.contains(required),
            "filesystem file metadata doc missing `{required}`"
        );
    }

    for required in [
        "pub record FileMetadata",
        "size: Int",
        "modified: time::UnixMillis",
        "pub fn file_metadata_path(file_path: path::Path): Result[FileMetadata, io::IOError]",
        "size = try file_size_path(file_path)",
        "modified = try modified_unix_millis_path(file_path)",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing filesystem file metadata evidence `{required}`"
        );
    }

    for required in [
        "standard_fs_file_metadata_path_returns_public_record",
        "standard_fs_file_metadata_path_missing_file_returns_io_error",
        "standard_fs_file_metadata_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_file_metadata_sample_runs",
        "fs::file_metadata_path",
        "fs::FileMetadata",
    ] {
        assert!(
            examples.contains(required),
            "examples missing filesystem file metadata coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_file_metadata",
        "import std::fs",
        "import std::path",
        "fs::file_metadata_path(file_path)",
        "metadata.size.to_string()",
        "metadata.modified.value > 0",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_file_metadata sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("fs-file-metadata-record.md")
                || text.contains("std_fs_file_metadata")
                || text.contains("file_metadata_path")
                || text.contains("FileMetadata"),
            "{label} must surface filesystem file metadata"
        );
    }

    assert!(
        implementation_resume.contains("| 320. Filesystem file metadata record |")
            && implementation_resume.contains("std_package/tests/docs/samples | Done")
            && implementation_resume.contains("regular-file `FileMetadata`"),
        "implementation queue must cover filesystem file metadata"
    );
}

#[test]
fn fs_path_status_record_is_documented_and_covered() {
    let design = read("docs/fs-path-status.md");
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_metadata/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::PathStatus` and",
        "`std::fs::path_status(target_path)` are",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub record PathStatus",
        "exists: Bool",
        "is_file: Bool",
        "is_dir: Bool",
        "pub fn path_status(target_path: path::Path): PathStatus",
        "PathStatus { exists: false, is_file: false, is_dir: false }",
        "Candidates Compared",
        "`fs::path_status(path): PathStatus`",
        "Non-Goals",
        "Validation",
        "standard_fs_path_status_returns_public_record",
        "standard_fs_path_status_type_mismatch_reports_expected_path",
        "standard_fs_metadata_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_metadata_sample_runs",
    ] {
        assert!(
            design.contains(required),
            "filesystem path status doc missing `{required}`"
        );
    }

    for required in [
        "pub record PathStatus",
        "exists: Bool",
        "is_file: Bool",
        "is_dir: Bool",
        "pub fn path_status(target_path: path::Path): PathStatus",
        "exists: exists_path(target_path)",
        "is_file: is_file_path(target_path)",
        "is_dir: is_dir_path(target_path)",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing filesystem path status evidence `{required}`"
        );
    }

    for required in [
        "standard_fs_path_status_returns_public_record",
        "standard_fs_path_status_type_mismatch_reports_expected_path",
        "standard_fs_metadata_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_metadata_sample_runs",
        "fs::path_status",
        "fs::PathStatus",
    ] {
        assert!(
            examples.contains(required),
            "examples missing filesystem path status coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_metadata",
        "import std::fs",
        "import std::path",
        "fs::path_status(source)",
        "source_status.is_file",
        "folder_status.is_dir",
        "missing_status.exists",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_metadata sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("fs-path-status.md")
                || text.contains("std_fs_metadata")
                || text.contains("path_status")
                || text.contains("PathStatus"),
            "{label} must surface filesystem path status"
        );
    }

    assert!(
        implementation_resume.contains("| 326. Filesystem path status record |")
            && implementation_resume.contains("std_package/tests/docs/samples | Done")
            && (implementation_resume.contains("path-status `PathStatus`")
                || implementation_resume.contains("path-status/kind/info metadata")),
        "implementation queue must cover filesystem path status"
    );
}

#[test]
fn fs_path_info_is_documented_and_covered() {
    let design = read("docs/fs-path-info.md");
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_metadata/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::PathKind`, `std::fs::PathInfo`,",
        "`std::fs::path_kind(target_path)`, and `std::fs::path_info(target_path)` are",
        "Short-term",
        "Medium-term",
        "Long-term",
        "Final goal",
        "Public Shape",
        "pub enum PathKind",
        "Missing",
        "File",
        "Directory",
        "Other",
        "pub record PathInfo",
        "status: PathStatus",
        "kind: PathKind",
        "pub fn path_kind(target_path: path::Path): PathKind",
        "pub fn path_info(target_path: path::Path): PathInfo",
        "Candidates Compared",
        "`PathKind` plus `PathInfo` over existing predicates",
        "Result[PathInfo, io::IOError]",
        "Validation",
        "standard_fs_path_info_returns_kind_and_status",
        "standard_fs_path_info_type_mismatch_reports_expected_path",
        "standard_fs_metadata_artifact_run_uses_emitted_std_implementations",
        "fs_path_info_is_documented_and_covered",
    ] {
        assert!(
            design.contains(required),
            "filesystem path info doc missing `{required}`"
        );
    }

    for required in [
        "pub enum PathKind",
        "Missing",
        "File",
        "Directory",
        "Other",
        "pub record PathInfo",
        "status: PathStatus",
        "kind: PathKind",
        "fn path_kind_from_status(status: PathStatus): PathKind",
        "pub fn path_kind(target_path: path::Path): PathKind",
        "pub fn path_info(target_path: path::Path): PathInfo",
        "PathInfo { status: status, kind: kind }",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing filesystem path info evidence `{required}`"
        );
    }

    for required in [
        "standard_fs_path_info_returns_kind_and_status",
        "standard_fs_path_info_type_mismatch_reports_expected_path",
        "standard_fs_metadata_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_metadata_sample_runs",
        "fs::PathKind::Missing",
        "fs::PathKind::File",
        "fs::PathKind::Directory",
        "fs::PathKind::Other",
        "fs::PathInfo",
        "fs::path_kind",
        "fs::path_info",
        "file/true/true/directory/missing/false",
        "file/true/directory/true/missing/false",
    ] {
        assert!(
            examples.contains(required),
            "examples missing filesystem path info coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_metadata",
        "import std::fs",
        "import std::path",
        "fn kind_name(kind: fs::PathKind): String",
        "source_status: fs::PathStatus = fs::path_status(source)",
        "source_info = fs::path_info(source)",
        "folder_info = fs::path_info(folder)",
        "missing_info = fs::path_info(missing)",
        "missing_status = missing_info.status",
        "kind_name(source_info.kind)",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_metadata sample missing filesystem path info evidence `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("fs-path-info.md")
                || text.contains("PathKind")
                || text.contains("PathInfo")
                || text.contains("path_info"),
            "{label} must surface filesystem path info"
        );
    }

    assert!(
        implementation_resume.contains("| 332. Filesystem path info record |")
            && implementation_resume.contains("std_package/tests/docs/samples | Done")
            && implementation_resume.contains("std::fs::PathKind")
            && implementation_resume.contains("std::fs::PathInfo")
            && implementation_resume.contains("path-status/kind/info metadata"),
        "implementation queue must cover filesystem path info"
    );
}

#[test]
fn fs_path_metadata_record_is_documented_and_covered() {
    let design = read("docs/fs-path-metadata.md");
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_path_metadata/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let muga_by_example = read("docs/muga-by-example.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::PathMetadata` and",
        "`std::fs::path_metadata_path(target_path)` are implemented",
        "Short-term",
        "Medium-term",
        "Long-term",
        "Final goal",
        "Public Shape",
        "pub record PathMetadata",
        "status: PathStatus",
        "kind: PathKind",
        "modified: time::UnixMillis",
        "pub fn path_metadata_path(target_path: path::Path): Result[PathMetadata, io::IOError]",
        "`path_metadata_path` first calls `modified_unix_millis_path(target_path)`",
        "operation = \"modified_unix_millis\"",
        "Candidates Compared",
        "`PathMetadata` with `status`, `kind`, and `modified`",
        "Add `size` to `PathMetadata`",
        "one-shot runtime metadata builtin",
        "Validation",
        "standard_fs_path_metadata_path_returns_public_record",
        "standard_fs_path_metadata_path_missing_path_returns_io_error",
        "standard_fs_path_metadata_path_type_mismatch_reports_expected_path",
        "standard_fs_path_metadata_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_path_metadata_sample_runs",
        "fs_path_metadata_record_is_documented_and_covered",
    ] {
        assert!(
            design.contains(required),
            "filesystem path metadata doc missing `{required}`"
        );
    }

    for required in [
        "pub record PathMetadata",
        "status: PathStatus",
        "kind: PathKind",
        "modified: time::UnixMillis",
        "pub fn path_metadata_path(target_path: path::Path): Result[PathMetadata, io::IOError]",
        "modified = try modified_unix_millis_path(target_path)",
        "info = path_info(target_path)",
        "PathMetadata {",
        "status: info.status",
        "kind: info.kind",
        "modified: modified",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing filesystem path metadata evidence `{required}`"
        );
    }

    for required in [
        "package_std_fs_path_metadata_sample_runs",
        "standard_fs_path_metadata_path_returns_public_record",
        "standard_fs_path_metadata_path_missing_path_returns_io_error",
        "standard_fs_path_metadata_path_type_mismatch_reports_expected_path",
        "standard_fs_path_metadata_artifact_run_uses_emitted_std_implementations",
        "fs::PathMetadata",
        "fs::path_metadata_path",
        "modified_unix_millis|",
        "file/true/directory/true/false",
    ] {
        assert!(
            examples.contains(required),
            "examples missing filesystem path metadata coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_path_metadata",
        "import std::fs",
        "import std::path",
        "fn kind_name(kind: fs::PathKind): String",
        "source_metadata: fs::PathMetadata = try fs::path_metadata_path(source)",
        "folder_metadata = try fs::path_metadata_path(folder)",
        "source_metadata.modified.value.to_string()",
        "kind_name(source_metadata.kind)",
        "source_metadata.status.is_file",
        "folder_metadata.status.is_dir",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_path_metadata sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("Muga by Example", muga_by_example.as_str()),
    ] {
        assert!(
            text.contains("fs-path-metadata.md")
                || text.contains("std_fs_path_metadata")
                || text.contains("PathMetadata")
                || text.contains("path_metadata_path"),
            "{label} must surface filesystem path metadata"
        );
    }

    assert!(
        implementation_resume.contains("| 334. Filesystem path metadata record |")
            && implementation_resume.contains("std_package/tests/docs/samples | Done")
            && implementation_resume.contains("std::fs::PathMetadata")
            && implementation_resume.contains("path_metadata_path(path::Path)")
            && implementation_resume.contains("existing-path `PathMetadata`"),
        "implementation queue must cover filesystem path metadata"
    );
}

#[test]
fn fs_path_size_metadata_record_is_documented_and_covered() {
    let design = read("docs/fs-path-size-metadata.md");
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_path_size_metadata/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let muga_by_example = read("docs/muga-by-example.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::PathSizeMetadata` and",
        "`std::fs::path_size_metadata_path(target_path)` are implemented",
        "Short-term",
        "Medium-term",
        "Long-term",
        "Final goal",
        "Public Shape",
        "pub record PathSizeMetadata",
        "status: PathStatus",
        "kind: PathKind",
        "modified: time::UnixMillis",
        "size: Option[Int]",
        "pub fn path_size_metadata_path(target_path: path::Path): Result[PathSizeMetadata, io::IOError]",
        "`path_size_metadata_path` first calls `path_metadata_path(target_path)`",
        "Option::Some(bytes)",
        "Option::None",
        "Candidates Compared",
        "Add `PathSizeMetadata` with `size: Option[Int]`",
        "Add `size` to `PathMetadata`",
        "Recursive directory size metadata",
        "Validation",
        "standard_fs_path_size_metadata_path_returns_public_record",
        "standard_fs_path_size_metadata_path_missing_path_returns_io_error",
        "standard_fs_path_size_metadata_path_type_mismatch_reports_expected_path",
        "standard_fs_path_size_metadata_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_path_size_metadata_sample_runs",
        "fs_path_size_metadata_record_is_documented_and_covered",
    ] {
        assert!(
            design.contains(required),
            "filesystem path size metadata doc missing `{required}`"
        );
    }

    for required in [
        "pub record PathSizeMetadata",
        "size: Option[Int]",
        "pub fn path_size_metadata_path(target_path: path::Path): Result[PathSizeMetadata, io::IOError]",
        "metadata = try path_metadata_path(target_path)",
        "size: Option[Int] = if metadata.status.is_file",
        "file_size = try file_size_path(target_path)",
        "Option::Some(file_size)",
        "PathSizeMetadata {",
        "status: metadata.status",
        "kind: metadata.kind",
        "modified: metadata.modified",
        "size: size",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing filesystem path size metadata evidence `{required}`"
        );
    }

    for required in [
        "package_std_fs_path_size_metadata_sample_runs",
        "standard_fs_path_size_metadata_path_returns_public_record",
        "standard_fs_path_size_metadata_path_missing_path_returns_io_error",
        "standard_fs_path_size_metadata_path_type_mismatch_reports_expected_path",
        "standard_fs_path_size_metadata_artifact_run_uses_emitted_std_implementations",
        "fs::PathSizeMetadata",
        "fs::path_size_metadata_path",
        "modified_unix_millis|",
        "file/true/8/directory/true/none/false",
    ] {
        assert!(
            examples.contains(required),
            "examples missing filesystem path size metadata coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_path_size_metadata",
        "import std::fs",
        "import std::path",
        "fn kind_name(kind: fs::PathKind): String",
        "fn size_text(size: Option[Int]): String",
        "source_metadata: fs::PathSizeMetadata = try fs::path_size_metadata_path(source)",
        "folder_metadata = try fs::path_size_metadata_path(folder)",
        "source_metadata.modified.value.to_string()",
        "size_text(source_metadata.size)",
        "size_text(folder_metadata.size)",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_path_size_metadata sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("Muga by Example", muga_by_example.as_str()),
    ] {
        assert!(
            text.contains("fs-path-size-metadata.md")
                || text.contains("std_fs_path_size_metadata")
                || text.contains("PathSizeMetadata")
                || text.contains("path_size_metadata_path"),
            "{label} must surface filesystem path size metadata"
        );
    }

    assert!(
        implementation_resume.contains("| 336. Filesystem path size metadata record |")
            && implementation_resume.contains("std_package/tests/docs/samples | Done")
            && implementation_resume.contains("std::fs::PathSizeMetadata")
            && implementation_resume.contains("path_size_metadata_path(path::Path)")
            && implementation_resume.contains("optional-size `PathSizeMetadata`"),
        "implementation queue must cover filesystem path size metadata"
    );
}

#[test]
fn fs_read_dir_recursive_is_documented_and_covered() {
    let design = read("docs/fs-read-dir-recursive.md");
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_read_dir_recursive/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let muga_by_example = read("docs/muga-by-example.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::read_dir_recursive_path(root_path)` is implemented",
        "Short-term",
        "Medium-term",
        "Long-term",
        "Final goal",
        "Public Shape",
        "pub fn read_dir_recursive_path(root_path: path::Path): Result[List[path::Path], io::IOError]",
        "The returned list contains descendants of `root_path`, not `root_path` itself.",
        "__muga_std_fs_read_dir_recursive(path::as_string(root_path))",
        "The runtime traversal sorts each directory's direct children",
        "host directory-entry metadata",
        "Candidates Compared",
        "Add `read_dir_recursive_path(root_path)`",
        "Add recursive directory size metadata first",
        "Add recursive remove or directory copy first",
        "Add glob/walk pattern matching",
        "Deferred Policy",
        "Validation",
        "package_std_fs_read_dir_recursive_sample_runs",
        "standard_fs_read_dir_recursive_path_returns_descendants",
        "standard_fs_read_dir_recursive_path_missing_dir_returns_io_error",
        "standard_fs_read_dir_recursive_path_does_not_recurse_into_symlink_dirs",
        "standard_fs_read_dir_recursive_path_type_mismatch_reports_expected_path",
        "standard_fs_read_dir_recursive_artifact_run_uses_emitted_std_implementations",
        "fs_read_dir_recursive_is_documented_and_covered",
    ] {
        assert!(
            design.contains(required),
            "filesystem recursive read dir doc missing `{required}`"
        );
    }

    for required in [
        "FS_READ_DIR_RECURSIVE_BUILTIN",
        "pub fn read_dir_recursive_path(root_path: path::Path): Result[List[path::Path], io::IOError]",
        "__muga_std_fs_read_dir_recursive(path::as_string(root_path))",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing recursive read dir evidence `{required}`"
        );
    }

    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");

    for required in [
        "StdFsReadDirRecursive",
        "FS_READ_DIR_RECURSIVE_BUILTIN",
        "Builtin(__muga_std_fs_read_dir_recursive)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing recursive read dir evidence `{required}`"
        );
    }

    assert!(
        typing.contains("BuiltinId::StdFsReadDir | BuiltinId::StdFsReadDirRecursive"),
        "typing must type recursive read dir like direct read dir"
    );

    for required in [
        "fn read_dir_recursive_paths(root_path: &str)",
        "collect_read_dir_recursive_paths(root_path, &mut paths)",
        "children.sort_by(|left, right| left.0.cmp(&right.0))",
        "BuiltinId::StdFsReadDirRecursive",
        "read_dir_recursive_paths(&path)",
        "io_error_value(\"read_dir\", &error_path, &error)",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing recursive read dir evidence `{required}`"
        );
    }

    for required in [
        "package_std_fs_read_dir_recursive_sample_runs",
        "standard_fs_read_dir_recursive_path_returns_descendants",
        "standard_fs_read_dir_recursive_path_missing_dir_returns_io_error",
        "standard_fs_read_dir_recursive_path_does_not_recurse_into_symlink_dirs",
        "standard_fs_read_dir_recursive_path_type_mismatch_reports_expected_path",
        "standard_fs_read_dir_recursive_artifact_run_uses_emitted_std_implementations",
        "fs::read_dir_recursive_path",
        "std-fs-read-dir-recursive",
        "read_dir",
    ] {
        assert!(
            examples.contains(required),
            "examples missing recursive read dir coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_read_dir_recursive",
        "import std::fs",
        "import std::path",
        "fn relative_or_empty(base: path::Path, items: List[path::Path], index: Int): String",
        "entries = try fs::read_dir_recursive_path(root)",
        "path::strip_prefix(value, base)",
        "entries.len().to_string()",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_read_dir_recursive sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("Muga by Example", muga_by_example.as_str()),
    ] {
        assert!(
            text.contains("fs-read-dir-recursive.md")
                || text.contains("std_fs_read_dir_recursive")
                || text.contains("read_dir_recursive_path")
                || text.contains("recursive directory listing"),
            "{label} must surface recursive directory listing"
        );
    }

    assert!(
        implementation_resume.contains("| 337. Filesystem recursive directory listing helper |")
            && implementation_resume.contains("std_package/tests/docs/samples | Done")
            && implementation_resume.contains("read_dir_recursive_path(root_path)")
            && implementation_resume.contains("read-only recursive `read_dir_recursive_path`"),
        "implementation queue must cover recursive directory listing"
    );
}

#[test]
fn fs_directory_size_metadata_is_documented_and_covered() {
    let design = read("docs/fs-directory-size-metadata.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_directory_size_metadata/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let muga_by_example = read("docs/muga-by-example.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::DirectorySizeMetadata` and",
        "`std::fs::directory_size_metadata_path(root_path)` are implemented",
        "Short-term",
        "Medium-term",
        "Long-term",
        "Final goal",
        "Public Shape",
        "pub record DirectorySizeMetadata",
        "size: Int",
        "file_count: Int",
        "directory_count: Int",
        "other_count: Int",
        "pub fn directory_size_metadata_path(root_path: path::Path): Result[DirectorySizeMetadata, io::IOError]",
        "__muga_std_fs_directory_size_metadata(path::as_string(root_path))",
        "count descendants of `root_path`",
        "operation = \"directory_size_metadata\"",
        "Candidates Compared",
        "Add `DirectorySizeMetadata` with byte and count fields",
        "Return only `Result[Int, io::IOError]`",
        "Compose `read_dir_recursive_path` with `path_size_metadata_path`",
        "Add destructive recursive operations first",
        "Deferred Policy",
        "Validation",
        "package_std_fs_directory_size_metadata_sample_runs",
        "standard_fs_directory_size_metadata_path_returns_public_record",
        "standard_fs_directory_size_metadata_path_missing_dir_returns_io_error",
        "standard_fs_directory_size_metadata_path_file_returns_io_error",
        "standard_fs_directory_size_metadata_path_counts_symlinks_as_other",
        "standard_fs_directory_size_metadata_path_type_mismatch_reports_expected_path",
        "standard_fs_directory_size_metadata_artifact_run_uses_emitted_std_implementations",
        "fs_directory_size_metadata_is_documented_and_covered",
    ] {
        assert!(
            design.contains(required),
            "filesystem directory size metadata doc missing `{required}`"
        );
    }

    for required in [
        "FS_DIRECTORY_SIZE_METADATA_MANGLED_NAME",
        "FS_DIRECTORY_SIZE_METADATA_VISIBLE_NAME_IN_FS",
        "FS_DIRECTORY_SIZE_METADATA_BUILTIN",
        "pub record DirectorySizeMetadata",
        "file_count: Int",
        "directory_count: Int",
        "other_count: Int",
        "pub fn directory_size_metadata_path(root_path: path::Path): Result[DirectorySizeMetadata, io::IOError]",
        "__muga_std_fs_directory_size_metadata(path::as_string(root_path))",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing filesystem directory size metadata evidence `{required}`"
        );
    }

    for required in [
        "StdFsDirectorySizeMetadata",
        "FS_DIRECTORY_SIZE_METADATA_BUILTIN",
        "Builtin(__muga_std_fs_directory_size_metadata)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing filesystem directory size metadata evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdFsDirectorySizeMetadata",
        "check_std_fs_directory_size_metadata_builtin",
        "std_fs_directory_size_metadata_type",
        "FS_DIRECTORY_SIZE_METADATA_VISIBLE_NAME_IN_FS",
        "Type::Result(Box::new(metadata_ty), Box::new(error_ty))",
    ] {
        assert!(
            typing.contains(required),
            "typing missing filesystem directory size metadata evidence `{required}`"
        );
    }

    for required in [
        "struct DirectorySizeMetadataRaw",
        "enum DirectorySizeEntryKind",
        "fn read_directory_size_metadata",
        "collect_directory_size_metadata(root_path, &mut metadata)",
        "fn collect_directory_size_metadata",
        "children.sort_by(|left, right| left.0.cmp(&right.0))",
        "file_type.is_dir()",
        "file_type.is_file()",
        "increment_directory_size_count",
        "add_directory_size_value",
        "fn directory_size_metadata_value",
        "FS_DIRECTORY_SIZE_METADATA_MANGLED_NAME",
        "BuiltinId::StdFsDirectorySizeMetadata",
        "read_directory_size_metadata(&path)",
        "\"directory_size_metadata\"",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing filesystem directory size metadata evidence `{required}`"
        );
    }

    for required in [
        "package_std_fs_directory_size_metadata_sample_runs",
        "standard_fs_directory_size_metadata_path_returns_public_record",
        "standard_fs_directory_size_metadata_path_missing_dir_returns_io_error",
        "standard_fs_directory_size_metadata_path_file_returns_io_error",
        "standard_fs_directory_size_metadata_path_counts_symlinks_as_other",
        "standard_fs_directory_size_metadata_path_type_mismatch_reports_expected_path",
        "standard_fs_directory_size_metadata_artifact_run_uses_emitted_std_implementations",
        "fs::DirectorySizeMetadata",
        "fs::directory_size_metadata_path",
        "directory_size_metadata|",
        "Result::Ok(8/2/1/0)",
        "Result::Ok(10/2/1/2)",
        "Result::Ok(14|2|1|0)",
    ] {
        assert!(
            examples.contains(required),
            "examples missing filesystem directory size metadata coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_directory_size_metadata",
        "import std::fs",
        "import std::io",
        "import std::path",
        "metadata: fs::DirectorySizeMetadata = try fs::directory_size_metadata_path(root)",
        "metadata.size.to_string()",
        "metadata.file_count.to_string()",
        "metadata.directory_count.to_string()",
        "metadata.other_count.to_string()",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_directory_size_metadata sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("Muga by Example", muga_by_example.as_str()),
    ] {
        assert!(
            text.contains("fs-directory-size-metadata.md")
                || text.contains("std_fs_directory_size_metadata")
                || text.contains("DirectorySizeMetadata")
                || text.contains("directory_size_metadata_path"),
            "{label} must surface filesystem directory size metadata"
        );
    }

    assert!(
        implementation_resume.contains("| 338. Filesystem directory size metadata record |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("std::fs::DirectorySizeMetadata")
            && implementation_resume.contains("directory_size_metadata_path(root_path)")
            && implementation_resume.contains("recursive `DirectorySizeMetadata`"),
        "implementation queue must cover filesystem directory size metadata"
    );
}

#[test]
fn fs_remove_dir_all_is_documented_and_covered() {
    let design = read("docs/fs-remove-dir-all.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_remove_dir_all/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let muga_by_example = read("docs/muga-by-example.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::remove_dir_all_path(dir_path)` is implemented",
        "Short-term",
        "Medium-term",
        "Long-term",
        "Final goal",
        "Public Shape",
        "pub fn remove_dir_all_path(dir_path: path::Path): Result[Unit, io::IOError]",
        "operation = \"remove_dir_all\"",
        "__muga_std_fs_remove_dir_all(path::as_string(dir_path))",
        "intentionally destructive",
        "Candidates Compared",
        "Add `remove_dir_all_path(dir_path)`",
        "Add recursive directory copy first",
        "Add trash/recycle-bin deletion",
        "Add glob/pattern deletion",
        "Deferred Policy",
        "Validation",
        "package_std_fs_remove_dir_all_sample_runs",
        "standard_fs_remove_dir_all_path_removes_non_empty_tree",
        "standard_fs_remove_dir_all_path_missing_dir_returns_io_error",
        "standard_fs_remove_dir_all_path_file_returns_io_error",
        "standard_fs_remove_dir_all_path_type_mismatch_reports_expected_path",
        "standard_fs_remove_dir_all_artifact_run_uses_emitted_std_implementations",
        "fs_remove_dir_all_is_documented_and_covered",
    ] {
        assert!(
            design.contains(required),
            "filesystem recursive remove dir doc missing `{required}`"
        );
    }

    for required in [
        "FS_REMOVE_DIR_ALL_BUILTIN",
        "__muga_std_fs_remove_dir_all",
        "pub fn remove_dir_all_path(dir_path: path::Path): Result[Unit, io::IOError]",
        "__muga_std_fs_remove_dir_all(path::as_string(dir_path))",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing recursive remove dir evidence `{required}`"
        );
    }

    for required in [
        "StdFsRemoveDirAll",
        "FS_REMOVE_DIR_ALL_BUILTIN",
        "Builtin(__muga_std_fs_remove_dir_all)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing recursive remove dir evidence `{required}`"
        );
    }

    assert!(
        typing.contains("| BuiltinId::StdFsRemoveDirAll")
            && typing.contains("check_std_fs_unit_path_builtin"),
        "typing must type recursive remove dir like other one-path Unit fs builtins"
    );

    for required in [
        "BuiltinId::StdFsRemoveDirAll",
        "expect_string_arg(args, span, \"__muga_std_fs_remove_dir_all\")",
        "fs::remove_dir_all(&path)",
        "io_error_value(\"remove_dir_all\", &path, &error)",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing recursive remove dir evidence `{required}`"
        );
    }

    for required in [
        "package_std_fs_remove_dir_all_sample_runs",
        "standard_fs_remove_dir_all_path_removes_non_empty_tree",
        "standard_fs_remove_dir_all_path_missing_dir_returns_io_error",
        "standard_fs_remove_dir_all_path_file_returns_io_error",
        "standard_fs_remove_dir_all_path_type_mismatch_reports_expected_path",
        "standard_fs_remove_dir_all_artifact_run_uses_emitted_std_implementations",
        "fs::remove_dir_all_path",
        "remove_dir_all|",
        "Result::Ok(false)",
    ] {
        assert!(
            examples.contains(required),
            "examples missing recursive remove dir coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_remove_dir_all",
        "created = try fs::create_dir_all_path(nested)",
        "written = try fs::write_text_path(payload, \"payload\")",
        "removed = try fs::remove_dir_all_path(root)",
        "Result::Ok(fs::exists_path(root).to_string())",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_remove_dir_all sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("Muga by Example", muga_by_example.as_str()),
    ] {
        assert!(
            text.contains("fs-remove-dir-all.md")
                || text.contains("std_fs_remove_dir_all")
                || text.contains("remove_dir_all_path"),
            "{label} must surface recursive remove dir"
        );
    }

    assert!(
        implementation_resume.contains("| 339. Filesystem recursive directory removal helper |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("std::fs::remove_dir_all_path")
            && implementation_resume.contains("recursive `remove_dir_all_path`"),
        "implementation queue must cover recursive remove dir"
    );
}

#[test]
fn fs_copy_dir_all_is_documented_and_covered() {
    let design = read("docs/fs-copy-dir-all.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_copy_dir_all/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let muga_by_example = read("docs/muga-by-example.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::copy_dir_all_path(from_path, to_path)` is implemented",
        "Short-term",
        "Medium-term",
        "Long-term",
        "Final goal",
        "Public Shape",
        "pub fn copy_dir_all_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]",
        "operation = \"copy_dir_all\"",
        "__muga_std_fs_copy_dir_all(path::as_string(from_path), path::as_string(to_path))",
        "destination root must not already exist",
        "must not be the source directory or inside the source directory",
        "Symlinks and other",
        "special entries return `io::PathPairError`",
        "does not roll back partially copied directories",
        "Candidates Compared",
        "Add no-overwrite `copy_dir_all_path(from, to)`",
        "Add merge/overwrite copy",
        "Add host-rename/cross-device move fallback",
        "Add glob/pattern copy",
        "Deferred Policy",
        "Validation",
        "package_std_fs_copy_dir_all_sample_runs",
        "standard_fs_copy_dir_all_path_copies_directory_tree",
        "standard_fs_copy_dir_all_path_missing_source_returns_path_pair_error",
        "standard_fs_copy_dir_all_path_existing_target_returns_path_pair_error",
        "standard_fs_copy_dir_all_path_rejects_destination_inside_source",
        "standard_fs_copy_dir_all_path_type_mismatch_reports_expected_path",
        "standard_fs_copy_dir_all_artifact_run_uses_emitted_std_implementations",
        "fs_copy_dir_all_is_documented_and_covered",
    ] {
        assert!(
            design.contains(required),
            "filesystem recursive copy dir doc missing `{required}`"
        );
    }

    for required in [
        "FS_COPY_DIR_ALL_BUILTIN",
        "__muga_std_fs_copy_dir_all",
        "pub fn copy_dir_all_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]",
        "__muga_std_fs_copy_dir_all(path::as_string(from_path), path::as_string(to_path))",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing recursive copy dir evidence `{required}`"
        );
    }

    for required in [
        "StdFsCopyDirAll",
        "FS_COPY_DIR_ALL_BUILTIN",
        "Builtin(__muga_std_fs_copy_dir_all)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing recursive copy dir evidence `{required}`"
        );
    }

    assert!(
        typing.contains("| BuiltinId::StdFsCopyDirAll")
            && typing.contains("check_std_fs_copy_file_builtin"),
        "typing must type recursive copy dir like other two-path Unit fs builtins"
    );

    for required in [
        "BuiltinId::StdFsCopyDirAll",
        "expect_two_string_args(args, span, \"__muga_std_fs_copy_dir_all\")",
        "copy_dir_all_paths(&from_path, &to_path)",
        "path_pair_error_value(",
        "\"copy_dir_all\"",
        "fn reject_copy_dir_target_inside_source",
        "fs::create_dir(to_root)",
        "fs::copy(&entry.from_path, &entry.to_path)",
        "io::ErrorKind::Unsupported",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing recursive copy dir evidence `{required}`"
        );
    }

    for required in [
        "package_std_fs_copy_dir_all_sample_runs",
        "standard_fs_copy_dir_all_path_copies_directory_tree",
        "standard_fs_copy_dir_all_path_missing_source_returns_path_pair_error",
        "standard_fs_copy_dir_all_path_existing_target_returns_path_pair_error",
        "standard_fs_copy_dir_all_path_rejects_destination_inside_source",
        "standard_fs_copy_dir_all_path_type_mismatch_reports_expected_path",
        "standard_fs_copy_dir_all_artifact_run_uses_emitted_std_implementations",
        "fs::copy_dir_all_path",
        "copy_dir_all|",
        "Result::Ok(true/true)",
    ] {
        assert!(
            examples.contains(required),
            "examples missing recursive copy dir coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_copy_dir_all",
        "fn copied_payload_summary(base: path::Path, target: path::Path): String",
        "fs::copy_dir_all_path(source, target)",
        "fs::remove_dir_all_path(base)",
        "text.concat(\"|\").concat(removed.to_string()).concat(\"|\").concat(fs::exists_path(base).to_string())",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_copy_dir_all sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("Muga by Example", muga_by_example.as_str()),
    ] {
        assert!(
            text.contains("fs-copy-dir-all.md")
                || text.contains("std_fs_copy_dir_all")
                || text.contains("copy_dir_all_path"),
            "{label} must surface recursive copy dir"
        );
    }

    assert!(
        implementation_resume.contains("| 340. Filesystem recursive directory copy helper |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("std::fs::copy_dir_all_path")
            && implementation_resume.contains("no-overwrite recursive `copy_dir_all_path`"),
        "implementation queue must cover recursive copy dir"
    );
}

#[test]
fn fs_move_dir_all_is_documented_and_covered() {
    let design = read("docs/fs-move-dir-all.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_move_dir_all/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let muga_by_example = read("docs/muga-by-example.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::move_dir_all_path(from_path, to_path)` is implemented",
        "Short-term",
        "Medium-term",
        "Long-term",
        "Final goal",
        "Public Shape",
        "pub fn move_dir_all_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]",
        "operation = \"move_dir_all\"",
        "__muga_std_fs_move_dir_all(path::as_string(from_path), path::as_string(to_path))",
        "destination root must not already exist",
        "destination must not be the",
        "source directory or inside the source directory",
        "Symlinks and other special entries",
        "does not roll back a partially copied target",
        "does not remove the target if deleting the source fails",
        "Candidates Compared",
        "Add copy-then-remove `move_dir_all_path(from, to)`",
        "Use host `rename` first with fallback",
        "Add transactional rollback",
        "Add merge/overwrite move",
        "Deferred Policy",
        "Validation",
        "package_std_fs_move_dir_all_sample_runs",
        "standard_fs_move_dir_all_path_moves_directory_tree",
        "standard_fs_move_dir_all_path_missing_source_returns_path_pair_error",
        "standard_fs_move_dir_all_path_existing_target_returns_path_pair_error",
        "standard_fs_move_dir_all_path_rejects_destination_inside_source",
        "standard_fs_move_dir_all_path_type_mismatch_reports_expected_path",
        "standard_fs_move_dir_all_artifact_run_uses_emitted_std_implementations",
        "fs_move_dir_all_is_documented_and_covered",
    ] {
        assert!(
            design.contains(required),
            "filesystem recursive move dir doc missing `{required}`"
        );
    }

    for required in [
        "FS_MOVE_DIR_ALL_BUILTIN",
        "__muga_std_fs_move_dir_all",
        "pub fn move_dir_all_path(from_path: path::Path, to_path: path::Path): Result[Unit, io::PathPairError]",
        "__muga_std_fs_move_dir_all(path::as_string(from_path), path::as_string(to_path))",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing recursive move dir evidence `{required}`"
        );
    }

    for required in [
        "StdFsMoveDirAll",
        "FS_MOVE_DIR_ALL_BUILTIN",
        "Builtin(__muga_std_fs_move_dir_all)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing recursive move dir evidence `{required}`"
        );
    }

    assert!(
        typing.contains("| BuiltinId::StdFsMoveDirAll")
            && typing.contains("check_std_fs_copy_file_builtin"),
        "typing must type recursive move dir like other two-path Unit fs builtins"
    );

    for required in [
        "BuiltinId::StdFsMoveDirAll",
        "expect_two_string_args(args, span, \"__muga_std_fs_move_dir_all\")",
        "move_dir_all_paths(&from_path, &to_path)",
        "path_pair_error_value(",
        "\"move_dir_all\"",
        "fn reject_move_dir_target_inside_source",
        "copy_dir_all_paths_after_target_check(from_path, to_path)?",
        "fs::remove_dir_all(from_path)",
        "directory move destination must not be the source or inside the source",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing recursive move dir evidence `{required}`"
        );
    }

    for required in [
        "package_std_fs_move_dir_all_sample_runs",
        "standard_fs_move_dir_all_path_moves_directory_tree",
        "standard_fs_move_dir_all_path_missing_source_returns_path_pair_error",
        "standard_fs_move_dir_all_path_existing_target_returns_path_pair_error",
        "standard_fs_move_dir_all_path_rejects_destination_inside_source",
        "standard_fs_move_dir_all_path_type_mismatch_reports_expected_path",
        "standard_fs_move_dir_all_artifact_run_uses_emitted_std_implementations",
        "fs::move_dir_all_path",
        "move_dir_all|",
        "Result::Ok(true/false/true)",
    ] {
        assert!(
            examples.contains(required),
            "examples missing recursive move dir coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_move_dir_all",
        "fn moved_payload_summary(base: path::Path, source: path::Path, target: path::Path): String",
        "fs::move_dir_all_path(source, target)",
        "fs::remove_dir_all_path(base)",
        "text.concat(\"|\").concat(source_exists.to_string()).concat(\"|\").concat(target_exists.to_string()).concat(\"|\").concat(fs::exists_path(base).to_string())",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_move_dir_all sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("Muga by Example", muga_by_example.as_str()),
    ] {
        assert!(
            text.contains("fs-move-dir-all.md")
                || text.contains("std_fs_move_dir_all")
                || text.contains("move_dir_all_path"),
            "{label} must surface recursive move dir"
        );
    }

    assert!(
        implementation_resume.contains("| 341. Filesystem recursive directory move helper |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("std::fs::move_dir_all_path")
            && implementation_resume.contains("copy-then-remove recursive `move_dir_all_path`"),
        "implementation queue must cover recursive move dir"
    );
}

#[test]
fn fs_canonicalize_path_is_documented_and_covered() {
    let design = read("docs/fs-canonicalize-path.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_fs_canonicalize/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::fs::canonicalize_path(target_path)` is implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub fn canonicalize_path(target_path: path::Path): Result[path::Path, io::IOError]",
        "operation = \"canonicalize\"",
        "not valid Unicode",
        "host filesystem's canonicalization behavior",
        "not a pure lexical normalizer",
        "Candidates Compared",
        "`fs::canonicalize_path(path): Result[path::Path, io::IOError]`",
        "Non-Goals",
        "Validation",
        "standard_fs_canonicalize_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_canonicalize_sample_runs",
    ] {
        assert!(
            design.contains(required),
            "filesystem canonicalize doc missing `{required}`"
        );
    }

    for required in [
        "FS_CANONICALIZE_BUILTIN",
        "__muga_std_fs_canonicalize",
        "pub fn canonicalize_path(target_path: path::Path): Result[path::Path, io::IOError]",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing filesystem canonicalize evidence `{required}`"
        );
    }

    for required in [
        "StdFsCanonicalize",
        "FS_CANONICALIZE_BUILTIN",
        "Builtin(__muga_std_fs_canonicalize)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing filesystem canonicalize evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdFsCanonicalize",
        "check_std_fs_canonicalize_builtin",
        "Type::Result(Box::new(Type::String), Box::new(error_ty))",
        "std_io_error_type",
    ] {
        assert!(
            typing.contains(required),
            "typing missing filesystem canonicalize evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdFsCanonicalize",
        "fs::canonicalize(&path)",
        "path_buf_into_string(path, \"canonical path is not valid Unicode\")",
        "io_error_value(\"canonicalize\", &path, &error)",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing filesystem canonicalize evidence `{required}`"
        );
    }

    for required in [
        "standard_fs_canonicalize_path_resolves_existing_file",
        "standard_fs_canonicalize_path_missing_file_returns_io_error",
        "standard_fs_canonicalize_path_type_mismatch_reports_expected_path",
        "standard_fs_canonicalize_artifact_run_uses_emitted_std_implementations",
        "package_std_fs_canonicalize_sample_runs",
        "fs::canonicalize_path",
        "__muga_std_fs_canonicalize",
    ] {
        assert!(
            examples.contains(required),
            "examples missing filesystem canonicalize coverage `{required}`"
        );
    }

    for required in [
        "package app::std_fs_canonicalize",
        "import std::fs",
        "import std::path",
        "fs::canonicalize_path(source)",
        "path::file_name(resolved)",
        "Result::Err(error) => error.operation",
    ] {
        assert!(
            sample.contains(required),
            "std_fs_canonicalize sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
    ] {
        assert!(
            text.contains("fs-canonicalize-path.md")
                || text.contains("std_fs_canonicalize")
                || text.contains("canonicalize_path"),
            "{label} must surface filesystem canonicalize"
        );
    }

    assert!(
        implementation_resume.contains("| 310. Filesystem canonicalize path helper |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("existing-path `fs::canonicalize_path`"),
        "implementation queue must cover filesystem canonicalize"
    );
}

#[test]
fn path_normalize_is_documented_and_covered() {
    let design = read("docs/path-normalize.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_path_normalize/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::path::normalize(path)` is implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub fn normalize(path: Path): Path",
        "preserves leading `..` components",
        "does not touch the filesystem",
        "must not be used as a sandbox containment check",
        "Candidates Compared",
        "`path::normalize(path): Path`",
        "Non-Goals",
        "Validation",
        "standard_path_normalize_artifact_run_uses_emitted_std_implementations",
        "package_std_path_normalize_sample_runs",
    ] {
        assert!(
            design.contains(required),
            "path normalize doc missing `{required}`"
        );
    }

    for required in [
        "PATH_NORMALIZE_BUILTIN",
        "__muga_std_path_normalize",
        "pub fn normalize(path: Path): Path",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing path normalize evidence `{required}`"
        );
    }

    for required in [
        "StdPathNormalize",
        "PATH_NORMALIZE_BUILTIN",
        "Builtin(__muga_std_path_normalize)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing path normalize evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdPathNormalize",
        "check_std_path_normalize_builtin",
        "self.check_expr_with_expected(&expr.args[0], Some(Type::String))",
    ] {
        assert!(
            typing.contains(required),
            "typing missing path normalize evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdPathNormalize",
        "expect_string_arg(args, span, \"__muga_std_path_normalize\")",
        "normalize_path_lexically_for_std",
        "Component::ParentDir",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing path normalize evidence `{required}`"
        );
    }

    for required in [
        "standard_path_normalize_removes_dot_and_internal_parent_components",
        "standard_path_normalize_preserves_leading_parent_components",
        "standard_path_normalize_collapsed_relative_path_returns_dot",
        "standard_path_normalize_type_mismatch_reports_expected_path",
        "standard_path_normalize_artifact_run_uses_emitted_std_implementations",
        "package_std_path_normalize_sample_runs",
        "path::normalize",
        "__muga_std_path_normalize",
    ] {
        assert!(
            examples.contains(required),
            "examples missing path normalize coverage `{required}`"
        );
    }

    for required in [
        "package app::std_path_normalize",
        "import std::path",
        "path::normalize(source)",
        "path::as_string(normalized)",
    ] {
        assert!(
            sample.contains(required),
            "std_path_normalize sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
    ] {
        assert!(
            text.contains("path-normalize.md")
                || text.contains("std_path_normalize")
                || text.contains("path::normalize"),
            "{label} must surface path normalize"
        );
    }

    assert!(
        implementation_resume.contains("| 313. Path lexical normalize helper |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("pure `path::normalize`"),
        "implementation queue must cover path normalize"
    );
}

#[test]
fn path_with_file_name_is_documented_and_covered() {
    let design = read("docs/path-with-file-name.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_path_with_file_name/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::path::with_file_name(path, new_file_name)` is implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub fn with_file_name(path: Path, new_file_name: String): Path",
        "does not touch the",
        "filesystem",
        "does not reject separators in",
        "Candidates Compared",
        "`path::with_file_name(path, new_file_name): Path`",
        "Non-Goals",
        "Validation",
        "standard_path_with_file_name_artifact_run_uses_emitted_std_implementations",
        "package_std_path_with_file_name_sample_runs",
    ] {
        assert!(
            design.contains(required),
            "path with_file_name doc missing `{required}`"
        );
    }

    for required in [
        "PATH_WITH_FILE_NAME_BUILTIN",
        "__muga_std_path_with_file_name",
        "pub fn with_file_name(path: Path, new_file_name: String): Path",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing path with_file_name evidence `{required}`"
        );
    }

    for required in [
        "StdPathWithFileName",
        "PATH_WITH_FILE_NAME_BUILTIN",
        "Builtin(__muga_std_path_with_file_name)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing path with_file_name evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdPathWithFileName",
        "check_std_path_with_file_name_builtin",
        "self.check_expr_with_expected(&expr.args[0], Some(Type::String))",
        "self.check_expr_with_expected(&expr.args[1], Some(Type::String))",
    ] {
        assert!(
            typing.contains(required),
            "typing missing path with_file_name evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdPathWithFileName",
        "expect_two_string_args(args, span, \"__muga_std_path_with_file_name\")",
        ".with_file_name(file_name)",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing path with_file_name evidence `{required}`"
        );
    }

    for required in [
        "standard_path_with_file_name_runs_as_virtual_package",
        "standard_path_with_file_name_replaces_single_component",
        "standard_path_with_file_name_type_mismatch_reports_expected_path",
        "standard_path_with_file_name_name_type_mismatch_reports_expected_string",
        "standard_path_with_file_name_artifact_run_uses_emitted_std_implementations",
        "package_std_path_with_file_name_sample_runs",
        "path::with_file_name",
        "__muga_std_path_with_file_name",
    ] {
        assert!(
            examples.contains(required),
            "examples missing path with_file_name coverage `{required}`"
        );
    }

    for required in [
        "package app::std_path_with_file_name",
        "import std::path",
        "path::with_file_name(source, \"summary.txt\")",
        "path::as_string(output)",
    ] {
        assert!(
            sample.contains(required),
            "std_path_with_file_name sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
    ] {
        assert!(
            text.contains("path-with-file-name.md")
                || text.contains("std_path_with_file_name")
                || text.contains("with_file_name"),
            "{label} must surface path with_file_name"
        );
    }

    assert!(
        implementation_resume.contains("| 308. Path file-name replacement helper |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("pure `path::with_file_name`"),
        "implementation queue must cover path with_file_name"
    );
}

#[test]
fn path_with_extension_is_documented_and_covered() {
    let design = read("docs/path-with-extension.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_path_with_extension/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::path::with_extension(path, new_extension)` is implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub fn with_extension(path: Path, new_extension: String): Path",
        "An empty `new_extension` removes the final extension",
        "does not touch the filesystem",
        "Candidates Compared",
        "`path::with_extension(path, new_extension): Path`",
        "Non-Goals",
        "Validation",
        "standard_path_with_extension_artifact_run_uses_emitted_std_implementations",
        "package_std_path_with_extension_sample_runs",
    ] {
        assert!(
            design.contains(required),
            "path with_extension doc missing `{required}`"
        );
    }

    for required in [
        "PATH_WITH_EXTENSION_BUILTIN",
        "__muga_std_path_with_extension",
        "pub fn with_extension(path: Path, new_extension: String): Path",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing path with_extension evidence `{required}`"
        );
    }

    for required in [
        "StdPathWithExtension",
        "PATH_WITH_EXTENSION_BUILTIN",
        "Builtin(__muga_std_path_with_extension)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing path with_extension evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdPathWithExtension",
        "check_std_path_with_extension_builtin",
        "self.check_expr_with_expected(&expr.args[0], Some(Type::String))",
        "self.check_expr_with_expected(&expr.args[1], Some(Type::String))",
    ] {
        assert!(
            typing.contains(required),
            "typing missing path with_extension evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdPathWithExtension",
        "expect_two_string_args(args, span, \"__muga_std_path_with_extension\")",
        ".with_extension(extension)",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing path with_extension evidence `{required}`"
        );
    }

    for required in [
        "standard_path_with_extension_runs_as_virtual_package",
        "standard_path_with_extension_empty_extension_strips_extension",
        "standard_path_with_extension_type_mismatch_reports_expected_path",
        "standard_path_with_extension_extension_type_mismatch_reports_expected_string",
        "standard_path_with_extension_artifact_run_uses_emitted_std_implementations",
        "package_std_path_with_extension_sample_runs",
        "path::with_extension",
        "__muga_std_path_with_extension",
    ] {
        assert!(
            examples.contains(required),
            "examples missing path with_extension coverage `{required}`"
        );
    }

    for required in [
        "package app::std_path_with_extension",
        "import std::path",
        "path::with_extension(source, \"json\")",
        "path::as_string(output)",
    ] {
        assert!(
            sample.contains(required),
            "std_path_with_extension sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
    ] {
        assert!(
            text.contains("path-with-extension.md")
                || text.contains("std_path_with_extension")
                || text.contains("with_extension"),
            "{label} must surface path with_extension"
        );
    }

    assert!(
        implementation_resume.contains("| 307. Path extension replacement helper |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("path::with_extension"),
        "implementation queue must cover path with_extension"
    );
}

#[test]
fn path_strip_prefix_is_documented_and_covered() {
    let design = read("docs/path-strip-prefix.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_path_strip_prefix/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::path::strip_prefix(path, base)` is implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub fn strip_prefix(path: Path, base: Path): Option[Path]",
        "If `path` and `base` are equal",
        "does not touch the filesystem",
        "Candidates Compared",
        "`path::strip_prefix(path, base): Option[Path]`",
        "Non-Goals",
        "Validation",
        "standard_path_strip_prefix_artifact_run_uses_emitted_std_implementations",
        "package_std_path_strip_prefix_sample_runs",
    ] {
        assert!(
            design.contains(required),
            "path strip_prefix doc missing `{required}`"
        );
    }

    for required in [
        "PATH_STRIP_PREFIX_BUILTIN",
        "__muga_std_path_strip_prefix",
        "pub fn strip_prefix(path: Path, base: Path): Option[Path]",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing path strip_prefix evidence `{required}`"
        );
    }

    for required in [
        "StdPathStripPrefix",
        "PATH_STRIP_PREFIX_BUILTIN",
        "Builtin(__muga_std_path_strip_prefix)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing path strip_prefix evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdPathStripPrefix",
        "check_std_path_strip_prefix_builtin",
        "self.check_expr_with_expected(&expr.args[0], Some(Type::String))",
        "self.check_expr_with_expected(&expr.args[1], Some(Type::String))",
    ] {
        assert!(
            typing.contains(required),
            "typing missing path strip_prefix evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdPathStripPrefix",
        "expect_two_string_args(args, span, \"__muga_std_path_strip_prefix\")",
        ".strip_prefix(std::path::Path::new(&base))",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing path strip_prefix evidence `{required}`"
        );
    }

    for required in [
        "standard_path_strip_prefix_returns_relative_path",
        "standard_path_strip_prefix_non_prefix_returns_none",
        "standard_path_strip_prefix_equal_paths_returns_empty_path",
        "standard_path_strip_prefix_path_type_mismatch_reports_expected_path",
        "standard_path_strip_prefix_base_type_mismatch_reports_expected_path",
        "standard_path_strip_prefix_artifact_run_uses_emitted_std_implementations",
        "package_std_path_strip_prefix_sample_runs",
        "path::strip_prefix",
        "__muga_std_path_strip_prefix",
    ] {
        assert!(
            examples.contains(required),
            "examples missing path strip_prefix coverage `{required}`"
        );
    }

    for required in [
        "package app::std_path_strip_prefix",
        "import std::path",
        "path::strip_prefix(source, base)",
        "path::as_string(relative)",
    ] {
        assert!(
            sample.contains(required),
            "std_path_strip_prefix sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
    ] {
        assert!(
            text.contains("path-strip-prefix.md")
                || text.contains("std_path_strip_prefix")
                || text.contains("strip_prefix"),
            "{label} must surface path strip_prefix"
        );
    }

    assert!(
        implementation_resume.contains("| 311. Path prefix stripping helper |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume
                .contains("path::with_file_name`/`path::with_extension`/`strip_prefix"),
        "implementation queue must cover path strip_prefix"
    );
}

#[test]
fn env_current_dir_is_documented_and_covered() {
    let design = read("docs/env-current-dir.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_env_current_dir/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::env::current_dir()` is implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub fn current_dir(): Result[path::Path, io::IOError]",
        "operation = \"current_dir\"",
        "path = \".\"",
        "not valid Unicode",
        "does not canonicalize",
        "Candidates Compared",
        "`env::current_dir(): Result[path::Path, io::IOError]`",
        "Non-Goals",
        "Validation",
        "standard_env_current_dir_artifact_run_uses_emitted_std_implementations",
        "package_std_env_current_dir_sample_runs",
    ] {
        assert!(
            design.contains(required),
            "environment current-dir doc missing `{required}`"
        );
    }

    for required in [
        "ENV_CURRENT_DIR_BUILTIN",
        "__muga_std_env_current_dir",
        "pub fn current_dir(): Result[path::Path, io::IOError]",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing environment current-dir evidence `{required}`"
        );
    }

    for required in [
        "StdEnvCurrentDir",
        "ENV_CURRENT_DIR_BUILTIN",
        "Builtin(__muga_std_env_current_dir)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing environment current-dir evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdEnvCurrentDir",
        "check_std_env_current_dir_builtin",
        "Type::Result(Box::new(Type::String), Box::new(error_ty))",
        "std_io_error_type",
    ] {
        assert!(
            typing.contains(required),
            "typing missing environment current-dir evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdEnvCurrentDir",
        "process_env::current_dir()",
        "path_buf_into_string(path, \"current directory is not valid Unicode\")",
        "io_error_value(\"current_dir\", \".\", &error)",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing environment current-dir evidence `{required}`"
        );
    }

    for required in [
        "standard_env_current_dir_returns_process_current_dir",
        "standard_env_current_dir_rejects_arguments",
        "standard_env_current_dir_artifact_run_uses_emitted_std_implementations",
        "package_std_env_current_dir_sample_runs",
        "env::current_dir",
        "__muga_std_env_current_dir",
    ] {
        assert!(
            examples.contains(required),
            "examples missing environment current-dir coverage `{required}`"
        );
    }

    for required in [
        "package app::std_env_current_dir",
        "import std::env",
        "import std::path",
        "env::current_dir()",
        "path::is_absolute(dir)",
        "Result::Err(error) => error.operation",
    ] {
        assert!(
            sample.contains(required),
            "std_env_current_dir sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
    ] {
        assert!(
            text.contains("env-current-dir.md")
                || text.contains("std_env_current_dir")
                || text.contains("env::current_dir"),
            "{label} must surface environment current_dir"
        );
    }

    assert!(
        implementation_resume.contains("| 309. Environment current directory helper |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("explicit `env::current_dir`"),
        "implementation queue must cover environment current_dir"
    );
}

#[test]
fn env_temp_dir_is_documented_and_covered() {
    let design = read("docs/env-temp-dir.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let sample = read("samples/packages/app/std_env_temp_dir/main.muga");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let stdlib_rules = read("docs/standard-library-review-rules.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: `std::env::temp_dir()` is implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        "pub fn temp_dir(): Result[path::Path, io::IOError]",
        "operation = \"temp_dir\"",
        "path = \".\"",
        "not valid Unicode",
        "does not create a directory",
        "Candidates Compared",
        "`env::temp_dir(): Result[path::Path, io::IOError]`",
        "Non-Goals",
        "Validation",
        "standard_env_temp_dir_artifact_run_uses_emitted_std_implementations",
        "package_std_env_temp_dir_sample_runs",
    ] {
        assert!(
            design.contains(required),
            "environment temp-dir doc missing `{required}`"
        );
    }

    for required in [
        "ENV_TEMP_DIR_BUILTIN",
        "__muga_std_env_temp_dir",
        "pub fn temp_dir(): Result[path::Path, io::IOError]",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing environment temp-dir evidence `{required}`"
        );
    }

    for required in [
        "StdEnvTempDir",
        "ENV_TEMP_DIR_BUILTIN",
        "Builtin(__muga_std_env_temp_dir)",
    ] {
        assert!(
            prelude.contains(required),
            "prelude missing environment temp-dir evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdEnvTempDir",
        "check_std_env_temp_dir_builtin",
        "Type::Result(Box::new(Type::String), Box::new(error_ty))",
        "std_io_error_type",
    ] {
        assert!(
            typing.contains(required),
            "typing missing environment temp-dir evidence `{required}`"
        );
    }

    for required in [
        "BuiltinId::StdEnvTempDir",
        "process_env::temp_dir()",
        "\"temporary directory path is not valid Unicode\"",
        "io_error_value(\"temp_dir\", \".\", &error)",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing environment temp-dir evidence `{required}`"
        );
    }

    for required in [
        "standard_env_temp_dir_returns_process_temp_dir",
        "standard_env_temp_dir_rejects_arguments",
        "standard_env_temp_dir_artifact_run_uses_emitted_std_implementations",
        "package_std_env_temp_dir_sample_runs",
        "env::temp_dir",
        "__muga_std_env_temp_dir",
    ] {
        assert!(
            examples.contains(required),
            "examples missing environment temp-dir coverage `{required}`"
        );
    }

    for required in [
        "package app::std_env_temp_dir",
        "import std::env",
        "env::temp_dir()",
        "Result::Ok(_) => \"ok\"",
        "Result::Err(error) => error.kind",
    ] {
        assert!(
            sample.contains(required),
            "std_env_temp_dir sample missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("stdlib rules", stdlib_rules.as_str()),
    ] {
        assert!(
            text.contains("env-temp-dir.md")
                || text.contains("std_env_temp_dir")
                || text.contains("env::temp_dir"),
            "{label} must surface environment temp_dir"
        );
    }

    assert!(
        implementation_resume.contains("| 312. Environment temporary directory helper |")
            && implementation_resume
                .contains("std_package/typing/runtime/tests/docs/samples | Done")
            && implementation_resume.contains("explicit `env::current_dir`/`env::temp_dir`"),
        "implementation queue must cover environment temp_dir"
    );
}

#[test]
fn installed_app_bundles_are_documented_and_covered() {
    let design = read("docs/installed-app-bundles.md");
    let lib = read("src/lib.rs");
    let main = read("src/main.rs");
    let project_template = read("src/project_template.rs");
    let package = read("src/package.rs");
    let examples = read("tests/examples.rs");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let packages_spec = read("spec/006-packages.md");
    let runtime_resources = read("docs/runtime-package-resource-lookup.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: the first non-mutating installed-app layout is implemented",
        "muga emit-app-bundle [--format text|json] [--source-free] --output-dir <dir> [--program <name>] <source-file>",
        "muga run-app-bundle [--format text|json] <bundle-dir> [-- <program-arg>...]",
        "muga install-app [--format text|json] [--replace-owned] --output-dir <bin-dir> [--program <name>] <bundle-dir>",
        "muga list-installed-apps [--format text|json] --output-dir <bin-dir>",
        "muga uninstall-app [--format text|json] --output-dir <bin-dir> --program <name>",
        "muga emit-app-completions [--format text|json] --output-dir <dir> [--program <name>] --type <type> [--package <package>] <bundle-dir>",
        "muga emit-app-archive [--format text|json] --archive-root <dir> [--program <name>] <bundle-dir>",
        "muga verify-app-archive [--format text|json] [--expected-hash sha256:<hex>] <archive-file>",
        "muga unpack-app-archive [--format text|json] [--expected-hash sha256:<hex>] --output-dir <dir> <archive-file>",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Implemented Contract",
        ".muga/app-bundle",
        "Source-backed bundle plus launcher",
        "Source-free bundle runner",
        "Source-free emitted bundle",
        "Machine-readable bundle emission",
        "Dependency-aware source tree bundle",
        "Non-mutating install wrapper",
        "Metadata-backed uninstall",
        "Machine-readable install/uninstall",
        "Non-mutating installed-app inventory",
        "Generated helper install/list hook",
        "Source-free completion package emission",
        "Archive-only app bundle",
        "Hash-bearing app archive filename",
        "Non-mutating app archive verification",
        "Machine-readable archive emission",
        "Machine-readable archive unpack",
        "Pre-install/pre-archive bundle validation",
        "generated `README.md` is part of the handoff surface",
        "muga emit-app-archive",
        "validates that transport without writing files",
        "validates that the bundle metadata",
        "validates the archive bytes",
        "the encoded `sha256:<hex>`",
        "invalidMetadata",
        "launcherMismatch",
        "Drift is reported as data",
        "Non-Goals",
        "Validation",
        "Next",
    ] {
        assert!(
            design.contains(required),
            "installed app bundle doc missing `{required}`"
        );
    }

    for required in [
        "pub struct AppBundleOutput",
        "pub struct AppBundleInstallOutput",
        "pub struct AppBundleUninstallOutput",
        "pub enum InstalledAppState",
        "pub struct InstalledAppEntry",
        "pub struct InstalledAppInventoryOutput",
        "pub struct AppBundleInterfaceOutput",
        "pub struct AppBundleArchiveOutput",
        "pub struct AppBundleArchiveVerifyOutput",
        "pub struct AppBundleArchiveUnpackOutput",
        "pub metadata: PathBuf",
        "pub fn emit_app_bundle",
        "pub fn emit_source_free_app_bundle",
        "AppBundleSourceMode::SourceFree",
        "pub fn compile_bytecode_app_bundle",
        "pub fn run_app_bundle_with_args",
        "pub fn install_app_bundle",
        "pub fn install_app_bundle_replace_owned",
        "pub fn uninstall_app_bundle",
        "pub fn list_installed_app_bundles",
        "pub fn read_app_bundle_interfaces",
        "pub fn app_bundle_program",
        "pub fn write_app_bundle_archive",
        "pub fn verify_app_bundle_archive",
        "pub fn verify_app_bundle_archive_with_expected_hash",
        "pub fn unpack_app_bundle_archive",
        "read_verified_app_bundle_archive",
        "read_verified_app_bundle_archive_with_expected_hash",
        "validate_app_bundle_expected_archive_hash",
        "validate_app_bundle_artifacts",
        "APP_BUNDLE_ARCHIVE_HEADER",
        "APP_BUNDLE_METADATA_FILE",
        "expected_app_bundle_archive_hash_from_path",
        "app archive hash mismatch",
        "AppBundleCopyMode::MugaSources",
        "app_bundle_artifact_paths",
        "discover_app_bundle_program",
        "app_bundle_launcher_text",
        "app_bundle_readme_text",
        "app_bundle_metadata_text",
        "read_app_bundle_entry_package",
        "run-app-bundle",
        "app_bundle_install_launcher_text",
        "app_bundle_install_metadata_path",
        "app_bundle_install_metadata_text",
        "validate_app_bundle_install_metadata",
        "parse_app_bundle_install_metadata_text",
        "installed_app_entry_from_metadata",
        "InstalledAppState::LauncherMismatch",
        "parse_app_bundle_manifest_string",
        "app install metadata",
        "does not match this launcher",
        "failed to remove app install launcher",
        "muga-installed-app-v1",
        "make_app_bundle_launcher_executable",
        "app_bundle_dependency_roots",
        ".muga",
        "bundle-deps",
        "MUGA_BIN",
        "PK031",
    ] {
        assert!(
            lib.contains(required),
            "library missing app bundle evidence `{required}`"
        );
    }

    for required in [
        "MUGA_INSTALL_DIR",
        "install-app --replace-owned --output-dir \"$MUGA_INSTALL_DIR\"",
        "list-installed-apps --output-dir \"$MUGA_INSTALL_DIR\"",
    ] {
        assert!(
            project_template.contains(required),
            "project templates missing install/list helper evidence `{required}`"
        );
    }

    assert!(
        package.contains("pub fn project_manifest_metadata_from_root"),
        "package metadata must be readable from a source-free bundle root"
    );

    for required in [
        "Mode::EmitAppBundle",
        "Mode::EmitAppCompletions",
        "Mode::RunAppBundle",
        "Mode::InstallApp",
        "Mode::UninstallApp",
        "Mode::ListInstalledApps",
        "Mode::EmitAppArchive",
        "Mode::UnpackAppArchive",
        "Mode::VerifyAppArchive",
        "\"emit-app-bundle\"",
        "\"emit-app-completions\"",
        "\"--source-free\"",
        "\"--replace-owned\"",
        "\"run-app-bundle\"",
        "\"install-app\"",
        "\"uninstall-app\"",
        "\"list-installed-apps\"",
        "\"emit-app-archive\"",
        "\"unpack-app-archive\"",
        "\"verify-app-archive\"",
        "emit-app-bundle requires --output-dir",
        "emit-app-completions requires --output-dir",
        "emit-app-completions requires --type",
        "--source-free is only supported with `emit-app-bundle`",
        "--replace-owned is only supported with `install-app`",
        "install-app requires --output-dir",
        "uninstall-app requires --output-dir",
        "uninstall-app requires --program",
        "list-installed-apps requires --output-dir",
        "emit-app-archive requires --archive-root",
        "unpack-app-archive requires --output-dir",
        "failed to emit app bundle",
        "failed to emit app completion package",
        "run_app_bundle_with_args",
        "failed to install app bundle",
        "failed to uninstall app",
        "failed to list installed apps",
        "failed to emit app archive",
        "failed to unpack app archive",
        "failed to verify app archive",
        "app_bundle_emit_json_output",
        "app_bundle_emit_diagnostic_json_output",
        "app_install_json_output",
        "app_install_diagnostic_json_output",
        "app_uninstall_json_output",
        "app_uninstall_diagnostic_json_output",
        "app_archive_emit_json_output",
        "app_archive_emit_diagnostic_json_output",
        "app_archive_verify_json_output",
        "app_archive_unpack_json_output",
        "app_archive_unpack_diagnostic_json_output",
        "installed_apps_json_output",
        "muga emit-app-bundle [--format text|json] [--source-free] --output-dir <dir> [--program <name>] <source-file>",
        "muga emit-app-completions [--format text|json] --output-dir <dir> [--program <name>] --type <type> [--package <package>] <bundle-dir>",
        "muga run-app-bundle [--format text|json] <bundle-dir> [-- <program-arg>...]",
        "muga install-app [--format text|json] [--replace-owned] --output-dir <bin-dir> [--program <name>] <bundle-dir>",
        "muga list-installed-apps [--format text|json] --output-dir <bin-dir>",
        "muga uninstall-app [--format text|json] --output-dir <bin-dir> --program <name>",
        "muga emit-app-archive [--format text|json] --archive-root <dir> [--program <name>] <bundle-dir>",
        "muga verify-app-archive [--format text|json] [--expected-hash <sha256>] <archive-file>",
        "--expected-hash",
        "muga unpack-app-archive [--format text|json] [--expected-hash <sha256>] --output-dir <dir> <archive-file>",
    ] {
        assert!(
            main.contains(required),
            "CLI missing app bundle evidence `{required}`"
        );
    }

    for required in [
        "cli_emit_app_bundle_writes_source_backed_layout_and_launcher",
        "emit_app_bundle_reports_bundle_local_artifact_paths",
        "cli_emit_app_bundle_writes_dependency_aware_layout_and_launcher",
        "cli_emit_app_bundle_rejects_output_inside_dependency_source_root",
        "cli_install_app_writes_non_mutating_launcher_for_bundle",
        "cli_list_installed_apps_reports_owned_launchers",
        "cli_install_and_archive_reject_broken_app_bundle_without_writes",
        "cli_emit_app_bundle_source_free_uses_artifacts_without_bundle_sources",
        "cli_emit_app_completions_writes_package_from_source_free_bundle",
        "cli_emit_and_unpack_app_archive_round_trips_bundle_launcher",
        "cli_unpack_app_archive_validates_hash_from_filename",
        "cli_emit_app_archive_rejects_archive_root_inside_bundle",
        "bin/bundle-tool",
        ".muga/bundle-deps/shared",
        ".muga/app-bundle",
        "muga install-app --output-dir <bin-dir> --program bundle-tool .",
        "muga list-installed-apps --output-dir <bin-dir>",
        "muga emit-app-completions --format json --output-dir <completion-dir> --type <Type> .",
        "muga emit-app-archive --archive-root <archive-dir> --program bundle-tool .",
        "muga verify-app-archive <archive-file>",
        "muga unpack-app-archive [--format text|json] [--expected-hash sha256:<hex>] --output-dir <bundle-dir> <archive-file>",
        "Use `--replace-owned` when updating a launcher already installed by Muga",
        "!bundle.join(\"muga.lock\").exists()",
        ".muga/installed-apps/installed-tool.toml",
        "metadata-only-bin",
        "--replace-owned",
        "uninstall-app",
        "removed",
        "updated|Ada",
        "ready\\tlisted-tool",
        "ready\\thello-install",
        "MUGA_INSTALL_DIR",
        "\\\"command\\\":\\\"list-installed-apps\\\"",
        "\\\"command\\\":\\\"emit-app-bundle\\\"",
        "\\\"command\\\":\\\"emit-app-completions\\\"",
        "\\\"command\\\":\\\"install-app\\\"",
        "\\\"command\\\":\\\"uninstall-app\\\"",
        "\\\"command\\\":\\\"emit-app-archive\\\"",
        "\\\"command\\\":\\\"unpack-app-archive\\\"",
        "launcherMismatch\\tlisted-tool",
        "no installed apps",
        "does not match this launcher",
        "format = \\\"muga-installed-app-v1\\\"",
        "bundle_launcher = ",
        "installed-bin/installed-tool",
        "source-free-tool",
        "emit-app-completions",
        "verify-app-archive",
        "\\\"command\\\":\\\"verify-app-archive\\\"",
        "status\\tok",
        "missing package implementation artifact",
        "cli-tool.completions.json",
        "Result::Ok(archived|Ada)",
        "Result::Ok(base|shared|Ada)",
        "Result::Ok(bundle|Ada)",
        "fetch or emit the app archive again",
    ] {
        assert!(
            examples.contains(required),
            "examples missing app bundle coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("practical", practical.as_str()),
        ("packages spec", packages_spec.as_str()),
        ("runtime resources", runtime_resources.as_str()),
    ] {
        assert!(
            text.contains("installed-app-bundles.md") || text.contains("emit-app-bundle"),
            "{label} must surface installed app bundles"
        );
    }

    assert!(
        implementation_resume.contains("| 295. Installed app bundles |")
            && implementation_resume.contains("| 343. Installed app inventory |")
            && implementation_resume.contains("| 344. Generated package helper install hooks |")
            && implementation_resume.contains("| 345. Archive emission JSON output |")
            && implementation_resume.contains("lib/main/tests/docs | Done")
            && implementation_resume.contains("emit-app-bundle --source-free")
            && implementation_resume.contains("list-installed-apps")
            && implementation_resume.contains("package archives preserve binary resources"),
        "implementation queue must cover installed app bundles"
    );
}

#[test]
fn cli_field_metadata_design_is_documented() {
    let design = read("docs/cli-field-metadata.md");
    let cli_parser_schema = read("docs/cli-parser-schema.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");

    for required in [
        "Status: first CLI field metadata implemented",
        "`name: \"long-option\"`",
        "repeated `alias: \"long-option\"`",
        "`help: \"text\"`",
        "`hidden`",
        "Long-option tokens must match `[A-Za-z][A-Za-z0-9_-]*`",
        "`@json(alias: \"...\")` remains JSON/config input compatibility metadata",
        "Duplicate And Conflict Rules",
        "Parser Behavior",
        "Usage Behavior",
        "Metadata Pipeline",
        "Introduce a dedicated Rust-side `CliSchema`",
        "Suggested first artifact shape",
        "Package-interface text can keep legacy field lines unchanged",
        "Candidates Compared",
        "Field-level `@cli(name, alias, help, hidden)` plus dedicated `CliSchema`",
        "Field-level `@cli(...)` but continue overloading `JsonDecodeSchema`",
        "Reuse `@json(rename)` and `@json(alias)` for CLI",
        "Tests",
        "Deferred Work",
        "Done: implement parser/formatter/typing/interface/typed-HIR/MIR/bytecode/",
        "Done: refresh `config-app`",
        "Next: re-audit before TOML",
    ] {
        assert!(
            design.contains(required),
            "CLI field metadata design missing `{required}`"
        );
    }

    assert!(
        cli_parser_schema.contains("cli-field-metadata.md")
            && cli_parser_schema.contains("dedicated")
            && cli_parser_schema.contains("CliSchema")
            && cli_parser_schema.contains("implementation boundary"),
        "CLI parser schema doc must point at the selected CLI metadata design"
    );

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
    ] {
        assert!(
            text.contains("cli-field-metadata.md")
                && text.contains("@cli")
                && text.contains("CliSchema")
                && text.contains("TOML"),
            "{label} must reference the CLI field metadata design"
        );
    }

    assert!(
        implementation_resume.contains("| 238. first `@cli(...)` field metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 239. first `@cli(...)` field metadata implementation |")
            && implementation_resume.contains("| parser/formatter/typing/interfaces/cli_schema/runtime/artifacts/tests/docs | Done |")
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark CLI metadata design and implementation done"
    );
}

#[test]
fn cli_field_metadata_implementation_is_covered() {
    let parser = read("src/parser.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let package_signature = read("src/package_signature.rs");
    let interface = read("src/interface.rs");
    let cli_schema = read("src/cli_schema.rs");
    let mir = read("src/mir.rs");
    let bytecode = read("src/bytecode.rs");
    let implementation_artifact = read("src/implementation_artifact.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/cli-field-metadata.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "validate_cli_attribute_arguments",
        "is_cli_long_option_token",
        "duplicate `@cli` attribute",
        "attribute `@cli` is allowed only on record declarations, enum declarations, record fields, and enum variants",
    ] {
        assert!(
            parser.contains(required),
            "parser missing CLI field metadata handling `{required}`"
        );
    }

    for required in [
        "TypedCliSchemaInfo",
        "cli_parse_or_schemas: Vec<TypedCliSchemaInfo>",
        "cli_usage_for_schemas: Vec<TypedCliSchemaInfo>",
        "cli_name_from_attributes",
        "duplicate CLI option name",
        "field `{}` has `@cli(...)` metadata",
        "CliFieldSchema",
    ] {
        assert!(
            typing.contains(required),
            "typing missing CLI field metadata handling `{required}`"
        );
    }

    for required in [
        "cli_name: Option<String>",
        "cli_aliases: Vec<String>",
        "cli_help: Option<String>",
        "cli_hidden: bool",
    ] {
        assert!(
            typed_hir.contains(required) && package_signature.contains(required),
            "typed HIR and package signatures must preserve CLI metadata `{required}`"
        );
    }

    for required in [
        "muga-package-interface-v11",
        "\"muga-package-interface-v8\"",
        "invalid field CLI alias count",
        "field CLI flags",
        "field.cli_hidden == expected.cli_hidden",
    ] {
        assert!(
            interface.contains(required),
            "package interface missing CLI metadata persistence `{required}`"
        );
    }

    for required in [
        "pub struct CliSchema",
        "pub struct CliFieldSchema",
        "pub enum CliValueSchema",
        "\"CR\"",
        "\"CE\"",
        "cli_schema_artifact_round_trips",
        "cli_schema_artifacts_reject_unknown_flag_bits",
    ] {
        assert!(
            cli_schema.contains(required),
            "CLI schema artifact support missing `{required}`"
        );
    }

    assert!(
        mir.contains("fn lower_cli_schema") && mir.contains("CliValueSchema"),
        "MIR lowering must preserve dedicated CLI schemas"
    );
    assert!(
        bytecode.contains("Instruction::CliParseOr") && bytecode.contains("CliSchema"),
        "bytecode must carry dedicated CLI schemas"
    );
    assert!(
        implementation_artifact.contains("CliSchema::from_artifact_text")
            && implementation_artifact.contains("CliUsageFor"),
        "implementation artifacts must persist dedicated CLI schemas"
    );
    for required in [
        "validate_cli_parsed_field",
        "cli_visible_option_fields",
        "aliases: ",
        "cli_field_by_option_name",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing CLI metadata behavior `{required}`"
        );
    }

    for required in [
        "standard_cli_field_metadata_parse_and_usage_runs",
        "standard_cli_field_metadata_rejects_invalid_contracts",
        "standard_cli_parse_or_artifact_run_uses_schema_payload",
        "aliases: --server-host",
        "muga-package-interface-v11",
    ] {
        assert!(
            examples.contains(required),
            "examples coverage missing CLI metadata case `{required}`"
        );
    }

    assert!(
        design.contains("Status: first CLI field metadata implemented")
            && design.contains(
                "Done: implement parser/formatter/typing/interface/typed-HIR/MIR/bytecode/"
            )
            && design.contains("Done: refresh `config-app`")
            && design.contains("Next: re-audit before TOML"),
        "CLI metadata design doc must record implementation completion"
    );
    assert!(
        implementation_resume.contains("| 239. first `@cli(...)` field metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/cli_schema/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume.contains("| 240. generated config-app CLI metadata adoption |")
            && implementation_resume.contains("| project template/samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 241. post-config-app CLI metadata adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 242. strict CLI parser schema design |")
            && implementation_resume.contains("| 243. strict CLI parser schema implementation |")
            && implementation_resume
                .contains("| std_package/typing/mir/bytecode/artifacts/runtime/tests/docs | Done |")
            && implementation_resume
                .contains("| 244. post-strict CLI parser adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 245. strict CLI tool sample adoption |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 246. post-strict CLI tool sample adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 247. generated cli-tool template adoption |")
            && implementation_resume.contains("| project template/tests/docs | Done |")
            && implementation_resume
                .contains("| 248. post-generated cli-tool template adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 249. strict CLI manual help adoption |")
            && implementation_resume.contains("| samples/project template/tests/docs | Done |")
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark CLI metadata adoption audit done and queue strict parser design"
    );
}

#[test]
fn generated_config_app_cli_metadata_adoption_is_implemented_and_covered() {
    let sample = read("samples/projects/config_app/src/main/main.muga");
    let template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let by_example = read("docs/muga-by-example.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let cli_metadata_design = read("docs/cli-field-metadata.md");

    for required in [
        "@cli(help: \"Application display name\")",
        "@cli(help: \"HTTP listen port\")",
        "@cli(help: \"Enable verbose logging\")",
        "@cli(name: \"tag\", alias: \"tags\", help: \"Tag value\")",
    ] {
        assert!(
            sample.contains(required),
            "config app sample missing CLI metadata adoption `{required}`"
        );
        assert!(
            template.contains(required),
            "generated config app template missing CLI metadata adoption `{required}`"
        );
    }

    for required in [
        "cli_new_creates_app_lib_and_test_templates",
        "manifest_config_project_sample_runs_with_cli_overrides",
        "manifest_config_project_sample_reports_usage",
        "manifest_config_project_sample_json_built_run_applies_cli_overrides",
        "--tag=ops",
        "--tag=runtime",
        "--tag <String>  repeatable",
        "aliases: --tags",
        "Application display name",
        "HTTP listen port",
        "Enable verbose logging",
        "Tag value",
    ] {
        assert!(
            examples.contains(required),
            "examples coverage missing generated config-app CLI metadata `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("CLI metadata design", cli_metadata_design.as_str()),
    ] {
        assert!(
            text.contains("config-app")
                && text.contains("@cli")
                && text.contains("--tag")
                && text.contains("TOML"),
            "{label} must document generated config-app CLI metadata adoption"
        );
    }

    assert!(
        implementation_resume.contains("| 240. generated config-app CLI metadata adoption |")
            && implementation_resume.contains("| project template/samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 241. post-config-app CLI metadata adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 242. strict CLI parser schema design |")
            && implementation_resume.contains("| 243. strict CLI parser schema implementation |")
            && implementation_resume.contains(
                "| std_package/typing/mir/bytecode/artifacts/runtime/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 244. post-strict CLI parser adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 245. strict CLI tool sample adoption |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 246. post-strict CLI tool sample adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 247. generated cli-tool template adoption |")
            && implementation_resume.contains("| project template/tests/docs | Done |")
            && implementation_resume
                .contains("| 248. post-generated cli-tool template adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 249. strict CLI manual help adoption |")
            && implementation_resume.contains("| samples/project template/tests/docs | Done |")
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark generated config-app CLI metadata adoption done"
    );
}

#[test]
fn strict_cli_parser_schema_design_is_documented() {
    let design = read("docs/strict-cli-parser-schema.md");
    let cli_parser_schema = read("docs/cli-parser-schema.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");

    for required in [
        "Status: strict CLI parser schema implemented",
        "pub fn parse[T](args: List[String]): Result[T, Error]",
        "MissingArgument",
        "derive `T` only from an expected `Result[T, cli::Error]` target",
        "Do not add source-level call type arguments in this slice",
        "Required field types",
        "Synthesized-when-absent field types",
        "`Bool` defaults to `false`",
        "`Option::None`",
        "`[]`",
        "Rejected strict targets",
        "Hidden strict fields must be able to synthesize an absent value",
        "UnknownArgument",
        "MissingValue",
        "InvalidValue",
        "Validation",
        "Do not add a strict no-default usage helper in this slice",
        "source-level call type arguments",
        "typed HIR, MIR, bytecode, and `.mgb`",
        "`.mgi` interfaces",
        "Candidates Compared",
        "`cli::parse[T](args)` with expected-result type inference",
        "Reuse `cli::parse_or[T](args, defaults)` with placeholder defaults",
        "Add source-level call type arguments now",
        "schema witness or type-token",
        "Synthesize `Bool=false`, `Option::None`, and `[]`",
        "Implementation Status",
        "Done: implement `cli::parse[T](args)`",
        "Done: audit strict CLI parser adoption",
        "Done: implement a checked-in strict CLI tool sample",
        "Done: audit strict CLI tool sample adoption",
        "Done: implement generated `muga new --template cli-tool` adoption",
        "Done: audit generated cli-tool template adoption",
        "Done: implement strict CLI manual help adoption",
    ] {
        assert!(
            design.contains(required),
            "strict CLI parser schema design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("CLI parser schema", cli_parser_schema.as_str()),
    ] {
        assert!(
            text.contains("strict-cli-parser-schema.md")
                && text.contains("cli::parse[T]")
                && text.contains("MissingArgument"),
            "{label} must document the strict CLI parser schema design"
        );
    }

    assert!(
        implementation_resume.contains("| 242. strict CLI parser schema design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 243. strict CLI parser schema implementation |")
            && implementation_resume.contains(
                "| std_package/typing/mir/bytecode/artifacts/runtime/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 244. post-strict CLI parser adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 245. strict CLI tool sample adoption |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 246. post-strict CLI tool sample adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 247. generated cli-tool template adoption |")
            && implementation_resume.contains("| project template/tests/docs | Done |")
            && implementation_resume
                .contains("| 248. post-generated cli-tool template adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 249. strict CLI manual help adoption |")
            && implementation_resume.contains("| samples/project template/tests/docs | Done |")
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark strict parser design done and queue implementation"
    );
}

#[test]
fn strict_cli_parser_schema_implementation_is_covered() {
    let std_package = read("src/std_package.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let mir = read("src/mir.rs");
    let bytecode = read("src/bytecode.rs");
    let artifact = read("src/implementation_artifact.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let strict_design = read("docs/strict-cli-parser-schema.md");
    let cli_design = read("docs/cli-parser-schema.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let release_checklist = read("docs/v1-release-checklist.md");

    for required in [
        "MissingArgument",
        "pub fn parse[T](args: List[String]): Result[T, Error]",
        "cli::parse requires compiler schema lowering",
    ] {
        assert!(
            std_package.contains(required),
            "std::cli package source missing strict parser evidence `{required}`"
        );
    }

    for required in [
        "std_cli_parse_bindings",
        "check_std_cli_parse_call",
        "type annotation required because `cli::parse` has no default value",
        "cli_parse_schemas",
        "strict parsing cannot preserve unsupported fields",
        "strict parsing cannot require hidden options",
        "CliValueSchema::EnumList",
    ] {
        assert!(
            typing.contains(required),
            "typing missing strict CLI parser evidence `{required}`"
        );
    }

    for (label, text, required) in [
        ("typed HIR", typed_hir.as_str(), "cli_parse_schema"),
        ("typed HIR", typed_hir.as_str(), "cli_parse_schemas"),
        ("MIR", mir.as_str(), "CliParseExpr"),
        ("MIR", mir.as_str(), "Expr::CliParse"),
        ("MIR", mir.as_str(), "lower_cli_schema"),
        ("bytecode", bytecode.as_str(), "Instruction::CliParse"),
        ("artifact", artifact.as_str(), "CliParse"),
        (
            "artifact",
            artifact.as_str(),
            "invalid strict CLI parser schema",
        ),
        ("artifact", artifact.as_str(), "strict CLI parser schema"),
        ("runtime", runtime.as_str(), "Instruction::CliParse"),
        ("runtime", runtime.as_str(), "fn cli_parse("),
        ("runtime", runtime.as_str(), "CliErrorKind::MissingArgument"),
        ("runtime", runtime.as_str(), "cli_synthesized_absent_value"),
        ("runtime", runtime.as_str(), "CliValueSchema::EnumList"),
    ] {
        assert!(
            text.contains(required),
            "{label} missing strict CLI parser implementation evidence `{required}`"
        );
    }

    for required in [
        "standard_cli_parse_required_record_runs",
        "standard_cli_parse_reports_required_and_synthesized_absent_values",
        "standard_cli_parse_artifact_run_uses_schema_payload",
        "standard_cli_parse_rejects_unsupported_targets",
        "MissingArgument:--name",
        "strict parsing cannot preserve unsupported fields",
        "strict parsing cannot require hidden options",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing strict CLI parser coverage `{required}`"
        );
    }

    for required in [
        "Status: strict CLI parser schema implemented",
        "Implementation Status",
        "Done: implement `cli::parse[T](args)`",
        "Done: audit strict CLI parser adoption",
        "Done: implement a checked-in strict CLI tool sample",
        "Done: audit strict CLI tool sample adoption",
        "Done: implement generated `muga new --template cli-tool` adoption",
        "Done: audit generated cli-tool template adoption",
        "Done: implement strict CLI manual help adoption",
    ] {
        assert!(
            strict_design.contains(required),
            "strict CLI parser design doc missing implementation status `{required}`"
        );
    }

    assert!(
        cli_design.contains("Done: implement `cli::parse[T](args)`")
            && cli_design.contains("Done: audit strict CLI parser adoption")
            && cli_design.contains("Done: implement a checked-in strict CLI tool sample")
            && cli_design.contains("Done: audit strict CLI tool sample adoption")
            && cli_design
                .contains("Done: implement generated `muga new --template cli-tool` adoption")
            && cli_design.contains("Done: audit generated cli-tool template adoption")
            && cli_design.contains("Done: implement strict CLI manual help adoption"),
        "CLI parser schema doc must record strict parser and sample adoption completion"
    );

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("cli::parse[T]") && text.contains("MissingArgument"),
            "{label} must document strict `cli::parse[T]` and MissingArgument"
        );
    }

    assert!(
        release_checklist.contains("strict `cli::parse[T]`")
            && release_checklist.contains("no-default usage"),
        "release checklist must distinguish implemented strict parse from deferred no-default usage"
    );

    assert!(
        implementation_resume.contains("| 243. strict CLI parser schema implementation |")
            && implementation_resume.contains(
                "| std_package/typing/mir/bytecode/artifacts/runtime/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 244. post-strict CLI parser adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 245. strict CLI tool sample adoption |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 246. post-strict CLI tool sample adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 247. generated cli-tool template adoption |")
            && implementation_resume.contains("| project template/tests/docs | Done |")
            && implementation_resume
                .contains("| 248. post-generated cli-tool template adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 249. strict CLI manual help adoption |")
            && implementation_resume.contains("| samples/project template/tests/docs | Done |")
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark strict parser implementation done and queue the adoption audit"
    );
}

#[test]
fn strict_cli_tool_sample_adoption_is_implemented_and_covered() {
    let manifest = read("samples/projects/cli_tool/muga.toml");
    let sample = read("samples/projects/cli_tool/src/main/main.muga");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let by_example = read("docs/muga-by-example.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let cli_design = read("docs/cli-parser-schema.md");
    let strict_design = read("docs/strict-cli-parser-schema.md");

    for required in ["name = \"cli_tool\"", "source = \"src\""] {
        assert!(
            manifest.contains(required),
            "strict CLI tool manifest missing `{required}`"
        );
    }

    for required in [
        "import std::cli",
        "import std::env",
        "import std::result",
        "pub enum Action",
        "pub record Root",
        "@cli(name: \"profile\", short: \"p\", help: \"Execution profile\")",
        "profile: Option[String]",
        "@cli(subcommand)",
        "command: Command",
        "pub enum Command",
        "@cli(name: \"run\", alias: \"r\", about: \"Run the main action\")",
        "Run(RunCommand)",
        "@cli(name: \"inspect\", alias: \"i\", about: \"Inspect one target\")",
        "Inspect(InspectCommand)",
        "pub record RunCommand",
        "pub record InspectCommand",
        "@cli(name: \"dry-run\"",
        "@cli(name: \"tag\", short: \"T\", alias: \"tags\"",
        "@validate(non_empty)",
        "@validate(min: 1, max: 10)",
        "tags: List[String]",
        "owner: Option[String]",
        "cli::parse_request[Root](args, \"cli-tool\")",
        "cli::Request::Help(usage)",
        "cli::Request::Parsed(root)",
        "Result[String, String]",
    ] {
        assert!(
            sample.contains(required),
            "strict CLI tool sample missing `{required}`"
        );
    }

    for required in [
        "manifest_cli_tool_project_sample_runs_with_required_options",
        "manifest_cli_tool_project_sample_reports_cli_parse_errors",
        "manifest_cli_tool_project_sample_runs_against_emitted_artifacts",
        "manifest_cli_tool_project_sample_json_built_run_uses_strict_parse",
        "Result::Ok(profile|dev|run|service|3|Apply|true|ops,prod|Kai)",
        "Result::Err(cli MissingArgument <target>",
        "Result::Err(cli Validation --count",
        "Result::Err(cli UnknownArgument deploy",
        "std__cli.mgb",
        "std__env.mgb",
        "std__result.mgb",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing strict CLI tool sample coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("CLI parser schema", cli_design.as_str()),
        ("strict parser design", strict_design.as_str()),
    ] {
        assert!(
            text.contains("samples/projects/cli_tool")
                && (text.contains("cli::parse[T]") || text.contains("cli::parse_request[T]"))
                && text.contains("strict CLI"),
            "{label} must document strict CLI tool sample adoption"
        );
    }

    assert!(
        implementation_resume.contains("| 245. strict CLI tool sample adoption |")
            && implementation_resume.contains("| samples/tests/docs | Done |")
            && implementation_resume
                .contains("| 246. post-strict CLI tool sample adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 247. generated cli-tool template adoption |")
            && implementation_resume.contains("| project template/tests/docs | Done |")
            && implementation_resume
                .contains("| 248. post-generated cli-tool template adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 249. strict CLI manual help adoption |")
            && implementation_resume.contains("| samples/project template/tests/docs | Done |")
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark strict CLI tool sample adoption done and queue generated cli-tool template adoption"
    );
}

#[test]
fn generated_cli_tool_template_adoption_is_implemented_and_covered() {
    let project_template = read("src/project_template.rs");
    let cli = read("src/main.rs");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let by_example = read("docs/muga-by-example.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let cli_design = read("docs/cli-parser-schema.md");
    let strict_design = read("docs/strict-cli-parser-schema.md");

    for required in [
        "CliTool",
        "ProjectTemplate::CliTool",
        "pub enum Action",
        "pub record Root",
        "@cli(name: \"profile\", short: \"p\", help: \"Execution profile\")",
        "profile: Option[String]",
        "@cli(subcommand)",
        "command: Command",
        "pub enum Command",
        "@cli(name: \"run\", alias: \"r\", about: \"Run the main action\")",
        "Run(RunCommand)",
        "@cli(name: \"inspect\", alias: \"i\", about: \"Inspect one target\")",
        "Inspect(InspectCommand)",
        "pub record RunCommand",
        "pub record InspectCommand",
        "@cli(name: \"dry-run\"",
        "@cli(name: \"tag\", alias: \"tags\"",
        "@validate(non_empty)",
        "@validate(min: 1, max: 10)",
        "cli::parse_request[Root](args, \"cli-tool\")",
        "cli::Request::Help(usage)",
        "cli::Request::Parsed(root)",
        "Result[String, String]",
    ] {
        assert!(
            project_template.contains(required),
            "cli-tool template source missing `{required}`"
        );
    }

    for required in [
        "muga new [--template app|lib|test|config-app|cli-tool|report-app|resource-export|package-app]",
        "\"cli-tool\" | \"cli_tool\" | \"cli\"",
        "app lib test config-app cli-tool report-app resource-export package-app",
        "expected `app`, `lib`, `test`, `config-app`, `cli-tool`, `report-app`, `resource-export`, or `package-app`",
    ] {
        assert!(
            cli.contains(required),
            "CLI missing cli-tool support `{required}`"
        );
    }

    for required in [
        "cli_new_creates_cli_tool_template",
        "--template=cli-tool",
        "name = \\\"strict_tool\\\"",
        "Result::Ok(run|service|3|Apply|true|ops,prod|Kai)",
        "Result::Err(cli MissingArgument <target>",
        "Result::Err(cli Validation --count",
        "\\\"stdout\\\":\\\"cli-tool profile|prod|run|batch|5|Apply|true|ops,prod|Kai\\\\n\\\"",
        "\\\"mainResult\\\":\\\"Result::Ok(profile|prod|run|batch|5|Apply|true|ops,prod|Kai)\\\"",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing generated cli-tool coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("CLI parser schema", cli_design.as_str()),
        ("strict parser design", strict_design.as_str()),
    ] {
        assert!(
            text.contains("muga new --template cli-tool")
                && (text.contains("cli::parse[T]") || text.contains("cli::parse_request[T]"))
                && (text.contains("implemented") || text.contains("Done")),
            "{label} must document generated cli-tool template adoption"
        );
    }

    assert!(
        implementation_resume.contains("| 247. generated cli-tool template adoption |")
            && implementation_resume.contains("| project template/tests/docs | Done |")
            && implementation_resume
                .contains("| 248. post-generated cli-tool template adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 249. strict CLI manual help adoption |")
            && implementation_resume.contains("| samples/project template/tests/docs | Done |")
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark generated cli-tool template adoption done and queue its audit"
    );
}

#[test]
fn generated_report_app_template_is_implemented_and_covered() {
    let project_template = read("src/project_template.rs");
    let cli = read("src/main.rs");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let by_example = read("docs/muga-by-example.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let design = read("docs/generated-report-app-template.md");

    for required in [
        "ReportApp",
        "ProjectTemplate::ReportApp",
        "Generated file-processing report starter.",
        "data/daily.txt",
        "scripts/run-report.sh",
        "scripts/package-report-app.sh",
        "path::with_extension(path::from_string(source_path), \"summary.txt\")",
        "fs::file_metadata_path(path::from_string(input))",
        "fs::read_text(input)",
        "fs::write_text(output, report)",
        "metadata.size.to_string()",
        "MUGA_REPORT_INPUT",
        "MUGA_REPORT_OUTPUT",
        "MUGA_INSTALL_DIR",
        "emit-app-bundle --source-free",
        "verify-app-archive",
        "list-installed-apps --output-dir \"$MUGA_INSTALL_DIR\"",
        "Result::Ok(summary_line(source_text))",
    ] {
        assert!(
            project_template.contains(required),
            "report-app template source missing `{required}`"
        );
    }

    for required in [
        "muga new [--template app|lib|test|config-app|cli-tool|report-app|resource-export|package-app]",
        "\"report-app\" | \"report_app\" | \"report\"",
        "app lib test config-app cli-tool report-app resource-export package-app",
        "expected `app`, `lib`, `test`, `config-app`, `cli-tool`, `report-app`, `resource-export`, or `package-app`",
    ] {
        assert!(
            cli.contains(required),
            "CLI missing report-app `{required}`"
        );
    }

    for required in [
        "cli_new_creates_report_app_template",
        "--template=report-app",
        "data/daily.txt",
        "data/built-summary.txt",
        "scripts/run-report.sh",
        "scripts/package-report-app.sh",
        "MUGA_INSTALL_DIR",
        "generated report-app package helper should run",
        "dist/report-app/.muga/app-bundle",
        "Result::Ok(daily: launch metrics healthy)",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing report-app coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("practical readiness", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("generated report app design", design.as_str()),
    ] {
        assert!(
            text.contains("muga new --template report-app")
                && text.contains("report")
                && (text.contains("run --built") || text.contains("built-artifact")),
            "{label} must document generated report-app template adoption"
        );
    }

    assert!(
        implementation_resume.contains("| 315. generated report-app template |")
            && implementation_resume.contains("| templates/tests/docs | Done |")
            && implementation_resume.contains("generated `muga new --template report-app`")
            && implementation_resume
                .contains("| 321. generated report-app FileMetadata adoption |")
            && implementation_resume.contains("| 329. Generated report-app package helper |"),
        "implementation queue must mark generated report-app template done"
    );
}

#[test]
fn strict_cli_manual_help_adoption_is_implemented_and_covered() {
    let sample = read("samples/projects/cli_tool/src/main/main.muga");
    let project_template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let by_example = read("docs/muga-by-example.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let cli_design = read("docs/cli-parser-schema.md");
    let strict_design = read("docs/strict-cli-parser-schema.md");

    for (label, text) in [
        ("sample", sample.as_str()),
        ("project template", project_template.as_str()),
    ] {
        for required in [
            "cli::parse_request[Root](args, \"cli-tool\")",
            "cli::Request::Help(usage)",
            "cli::Request::Parsed(root)",
            "fn emit_usage(usage: String): Result[String, String]",
            "fn run_command(root: Root): Result[String, String]",
        ] {
            assert!(
                text.contains(required),
                "{label} missing strict CLI manual help evidence `{required}`"
            );
        }
    }

    for required in [
        "manifest_cli_tool_project_sample_reports_generated_usage",
        "cli_new_creates_cli_tool_template",
        "standard_cli_usage_for_required_record_runs",
        "standard_cli_usage_for_required_artifact_run_uses_schema_payload",
        ".arg(\"--help\")",
        "built_help",
        "Usage: cli-tool [global-options] <command> [args]",
        "Usage: cli-tool run [options] <target>",
        "Result::Ok(Usage: cli-tool [global-options] <command> [args]",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing strict CLI manual help coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("CLI parser schema", cli_design.as_str()),
        ("strict parser design", strict_design.as_str()),
    ] {
        assert!(
            text.contains("strict CLI manual help")
                || (text.contains("manual")
                    && text.contains("--help")
                    && text.contains("cli-tool")),
            "{label} must document strict CLI manual help adoption"
        );
    }

    assert!(
        by_example.contains("muga run samples/projects/cli_tool/src/main/main.muga -- --help")
            && by_example.contains("cli::help_for_required[Root]"),
        "Muga by Example must show the strict CLI help path"
    );
    assert!(
        implementation_resume.contains("| 249. strict CLI manual help adoption |")
            && implementation_resume.contains("| samples/project template/tests/docs | Done |")
            && implementation_resume
                .contains("| 250. post-strict CLI manual help adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 251. strict CLI no-default usage helper design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 252. strict CLI no-default usage helper implementation |")
            && implementation_resume.contains(
                "| parser/typing/std_package/mir/bytecode/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark strict CLI manual help adoption done and queue its audit"
    );
}

#[test]
fn strict_cli_no_default_usage_helper_design_is_documented() {
    let design = read("docs/strict-cli-no-default-usage.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let cli_design = read("docs/cli-parser-schema.md");
    let strict_design = read("docs/strict-cli-parser-schema.md");

    for required in [
        "Status: strict CLI no-default usage helper implemented",
        "pub fn usage_for_required[T](program: String): String",
        "cli::usage_for_required[Command](\"cli-tool\")",
        "cli::usage_for[T](program, defaults)",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Public API",
        "Type Anchor Policy",
        "Source Call Type Arguments",
        "Usage Rendering Contract",
        "Usage: cli-tool [options]",
        "--target <String>  required; non-empty; Target resource name",
        "--count <Int>  required; range: 1..10; Number of items to process",
        "--action <Action>  required; values: Audit, Apply; Command action",
        "--dry-run[=<Bool>]  Preview changes without applying them",
        "--tag <String>  repeatable; aliases: --tags; Tag filter",
        "--owner <String>  Optional owner",
        "do not include app-owned `--help` text",
        "Schema And Artifacts",
        "CliUsageRequired",
        "Candidates Compared",
        "`cli::usage_for_required[T](program)` with explicit call type argument",
        "Overload `cli::usage_for[T](program)` by arity",
        "Infer `T` from expected `String` result",
        "Require a fake default record",
        "schema witness or type-token value",
        "Non-Goals",
        "explicit call type arguments for ordinary user-defined generic functions",
        "Implementation Plan",
        "Done: implement `cli::usage_for_required[T](program)`",
        "Done: implement CLI short option metadata. Done: audit CLI short option metadata adoption. Done: design CLI positional field metadata in [cli-positional-field-metadata.md](cli-positional-field-metadata.md). Done: implement CLI positional field metadata. Done: audit CLI positional field metadata adoption. Done: design built-in CLI help policy in [cli-built-in-help-policy.md](cli-built-in-help-policy.md). Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata",
    ] {
        assert!(
            design.contains(required),
            "strict CLI no-default usage design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("CLI parser schema", cli_design.as_str()),
        ("strict parser design", strict_design.as_str()),
    ] {
        assert!(
            text.contains("strict-cli-no-default-usage.md") && text.contains("usage_for_required"),
            "{label} must surface the strict CLI no-default usage helper design"
        );
    }

    assert!(
        implementation_resume.contains("| 251. strict CLI no-default usage helper design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 252. strict CLI no-default usage helper implementation |")
            && implementation_resume.contains(
                "| parser/typing/std_package/mir/bytecode/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume.contains(
                "| 253. post-strict CLI no-default usage helper adoption gap selection |"
            )
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 254. CLI command metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 255. CLI command metadata implementation |")
            && implementation_resume.contains(
                "| parser/typing/interfaces/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark no-default usage design done and queue implementation"
    );
}

#[test]
fn strict_cli_no_default_usage_helper_implementation_is_covered() {
    let ast = read("src/ast.rs");
    let parser = read("src/parser.rs");
    let formatter = read("src/formatter.rs");
    let std_package = read("src/std_package.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let mir = read("src/mir.rs");
    let bytecode = read("src/bytecode.rs");
    let artifact = read("src/implementation_artifact.rs");
    let runtime = read("src/runtime.rs");
    let sample = read("samples/projects/cli_tool/src/main/main.muga");
    let project_template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let design = read("docs/strict-cli-no-default-usage.md");
    let by_example = read("docs/muga-by-example.md");

    for (label, text, required) in [
        ("AST", ast.as_str(), "type_args: Vec<TypeExpr>"),
        (
            "parser",
            parser.as_str(),
            "call_type_args_are_followed_by_lparen",
        ),
        (
            "parser",
            parser.as_str(),
            "expected `(` after call type arguments",
        ),
        ("formatter", formatter.as_str(), "format_call_type_args"),
        (
            "std::cli package",
            std_package.as_str(),
            "pub fn usage_for_required[T](program: String): String",
        ),
        (
            "typing",
            typing.as_str(),
            "std_cli_usage_for_required_bindings",
        ),
        (
            "typing",
            typing.as_str(),
            "check_std_cli_usage_for_required_call",
        ),
        (
            "typing",
            typing.as_str(),
            "explicit call type arguments are currently supported only for `cli::usage_for_required`, `cli::help_for_required`, and `cli::parse_request`",
        ),
        (
            "typed HIR",
            typed_hir.as_str(),
            "cli_usage_for_required_schema",
        ),
        ("MIR", mir.as_str(), "CliUsageForRequiredExpr"),
        ("MIR", mir.as_str(), "Expr::CliUsageForRequired"),
        (
            "bytecode",
            bytecode.as_str(),
            "Instruction::CliUsageForRequired",
        ),
        ("artifact", artifact.as_str(), "CliUsageForRequired"),
        ("runtime", runtime.as_str(), "cli_usage_for_required"),
        (
            "runtime",
            runtime.as_str(),
            "cli_required_usage_annotations",
        ),
        ("runtime", runtime.as_str(), "non-empty"),
        ("runtime", runtime.as_str(), "range: {min}..{max}"),
    ] {
        assert!(
            text.contains(required),
            "{label} missing strict CLI no-default usage implementation evidence `{required}`"
        );
    }

    for (label, text) in [
        ("sample", sample.as_str()),
        ("project template", project_template.as_str()),
    ] {
        assert!(
            text.contains("cli::parse_request[Root](args, \"cli-tool\")")
                && text.contains("cli::Request::Help(usage)")
                && !text.contains("Number of items to process (1..10)")
                && !text.contains("Audit or Apply"),
            "{label} must use generated strict usage instead of duplicated manual option text"
        );
    }

    for required in [
        "standard_cli_usage_for_required_record_runs",
        "standard_cli_usage_for_required_artifact_run_uses_schema_payload",
        "standard_cli_usage_for_required_rejects_invalid_type_anchors",
        "cli::usage_for_required[Command](\"app\")",
        "<target>  required; non-empty; Target resource name",
        "--count <Int>  required; range: 1..10; Number of items to process",
        "--action <Action>  required; values: Audit, Apply; Command action",
        "explicit call type arguments are currently supported only for `cli::usage_for_required`, `cli::help_for_required`, and `cli::parse_request`",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing strict usage helper coverage `{required}`"
        );
    }

    assert!(
        design.contains("Status: strict CLI no-default usage helper implemented")
            && readme.contains("cli::usage_for_required[T](program)")
            && roadmap.contains("implements `cli::usage_for_required[T](program)`")
            && by_example.contains("cli::help_for_required[Root]")
            && implementation_resume
                .contains("| 252. strict CLI no-default usage helper implementation |")
            && implementation_resume.contains(
                "| parser/typing/std_package/mir/bytecode/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume.contains(
                "| 253. post-strict CLI no-default usage helper adoption gap selection |"
            )
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 254. CLI command metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 255. CLI command metadata implementation |")
            && implementation_resume.contains(
                "| parser/typing/interfaces/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "docs and implementation queue must mark strict usage helper implementation done and queue adoption audit"
    );
}

#[test]
fn cli_command_metadata_design_is_documented() {
    let design = read("docs/cli-command-metadata.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");

    for required in [
        "Status: CLI command metadata implemented",
        "@cli(about: \"Inspect and apply changes to a target resource\")",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Public Syntax",
        "record declarations may have at most one `@cli` attribute",
        "the only record-level argument in this slice is `about`",
        "Usage Rendering",
        "Usage: cli-tool [options]",
        "Run typed strict CLI commands",
        "Options:",
        "Schema And Artifacts",
        "`CliSchema` now carries `about: Option<Symbol>`",
        "package signatures and `.mgi` interfaces persist metadata",
        "old interfaces and old schema payloads without command metadata remain",
        "Candidates Compared",
        "Record-level `@cli(about: \"...\")`",
        "Reuse source doc comments as command descriptions",
        "Add `about` to `usage_for(program, defaults, about)` arguments",
        "Non-Goals",
        "Implementation Plan",
        "Done: implement `@cli(about: \"...\")`",
        "Done: implement CLI short option metadata. Done: audit CLI short option metadata adoption. Done: design CLI positional field metadata in [cli-positional-field-metadata.md](cli-positional-field-metadata.md). Done: implement CLI positional field metadata. Done: audit CLI positional field metadata adoption. Done: design built-in CLI help policy in [cli-built-in-help-policy.md](cli-built-in-help-policy.md). Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata",
    ] {
        assert!(
            design.contains(required),
            "CLI command metadata design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
    ] {
        assert!(
            text.contains("cli-command-metadata.md") && text.contains("@cli(about"),
            "{label} must surface the CLI command metadata design"
        );
    }

    assert!(
        implementation_resume.contains("| 254. CLI command metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 255. CLI command metadata implementation |")
            && implementation_resume.contains(
                "| parser/typing/interfaces/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 256. post-CLI command metadata adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 257. CLI short option metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 258. CLI short option metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark CLI command metadata implemented and queue adoption audit"
    );
}

#[test]
fn cli_command_metadata_implementation_is_covered() {
    let parser = read("src/parser.rs");
    let typing = read("src/typing.rs");
    let package_signature = read("src/package_signature.rs");
    let interface = read("src/interface.rs");
    let typed_hir = read("src/typed_hir.rs");
    let mir = read("src/mir.rs");
    let cli_schema = read("src/cli_schema.rs");
    let runtime = read("src/runtime.rs");
    let sample = read("samples/projects/cli_tool/src/main/main.muga");
    let cli_schema_sample = read("samples/packages/app/std_cli_schema/main.muga");
    let project_template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let design = read("docs/cli-command-metadata.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");

    for required in [
        "cli_attribute_is_record_metadata",
        "cli_attribute_is_field_metadata",
        "record declarations support only `@cli(about: \\\"...\\\")`",
        "record fields support only `@cli(name:",
        "CLI about may be specified only once",
    ] {
        assert!(
            parser.contains(required),
            "parser missing CLI command metadata evidence `{required}`"
        );
    }

    for required in [
        "cli_about: Option<Symbol>",
        "cli_about_from_attributes",
        "about: record.cli_about",
    ] {
        assert!(
            typing.contains(required),
            "typing missing CLI command metadata evidence `{required}`"
        );
    }

    for (label, text, required) in [
        (
            "package signatures",
            package_signature.as_str(),
            "cli_about: Option<String>",
        ),
        ("interface", interface.as_str(), "cli_about: Option<String>"),
        (
            "interface",
            interface.as_str(),
            "\"invalid record CLI about marker\"",
        ),
        (
            "interface",
            interface.as_str(),
            "record.cli_about == interface.cli_about",
        ),
        ("typed HIR", typed_hir.as_str(), "cli_about: Option<String>"),
        ("MIR", mir.as_str(), "about: schema.about.map"),
        (
            "CLI schema",
            cli_schema.as_str(),
            "pub about: Option<Symbol>",
        ),
        ("CLI schema", cli_schema.as_str(), "\"CA\""),
        (
            "runtime",
            runtime.as_str(),
            "if let Some(about) = schema.about",
        ),
    ] {
        assert!(
            text.contains(required),
            "{label} missing CLI command metadata evidence `{required}`"
        );
    }

    for (label, text) in [
        ("strict sample", sample.as_str()),
        ("std_cli_schema sample", cli_schema_sample.as_str()),
        ("project template", project_template.as_str()),
    ] {
        assert!(
            text.contains("@cli(about:"),
            "{label} must adopt CLI command metadata"
        );
    }

    for required in [
        "standard_cli_usage_for_required_record_runs",
        "standard_cli_usage_for_required_artifact_run_uses_schema_payload",
        "package_std_cli_schema_sample_runs_against_emitted_artifacts",
        "muga_test_rejects_unknown_and_misplaced_attributes",
        "Run a typed strict CLI tool",
        "Run artifact-backed commands",
        "Result::Ok(api|compat|2|blue|green|true|Batch|secret|true|true|true)",
        "record declarations support only `@cli(about: \\\"...\\\")`",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing CLI command metadata coverage `{required}`"
        );
    }

    assert!(
        design.contains("Status: CLI command metadata implemented")
            && readme.contains("@cli(about: \"...\")")
            && roadmap.contains("@cli(about: \"...\")")
            && implementation_resume.contains("| 255. CLI command metadata implementation |")
            && implementation_resume.contains(
                "| parser/typing/interfaces/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 256. post-CLI command metadata adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 257. CLI short option metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 258. CLI short option metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "docs and implementation queue must mark CLI command metadata implementation done"
    );
}

#[test]
fn cli_short_option_metadata_design_is_documented() {
    let design = read("docs/cli-short-option-metadata.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");

    for required in [
        "Status: CLI short option metadata implemented",
        "@cli(short: \"t\", help: \"Target resource name\")",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Public Syntax",
        "short` is field-level only",
        "one-character ASCII alphabetic string",
        "Parser Behavior",
        "-t value",
        "-t=value",
        "bare short `Bool` flags",
        "combined short flags such as `-abc`",
        "Usage Rendering",
        "-t, --target <String>",
        "App-Owned Help",
        "cli::has_short_flag(args, \"h\")",
        "Schema And Artifacts",
        "`CliFieldSchema` should carry `short: Option<Symbol>`",
        "old interfaces and old schema payloads without short metadata remain readable",
        "Candidates Compared",
        "Long aliases only",
        "Reserve `-h` globally",
        "Non-Goals",
        "Implementation Plan",
        "Done: implement `@cli(short: \"...\")`",
        "Done: audit CLI short option metadata adoption",
        "Done: design CLI positional field metadata in",
        "Done: audit CLI positional field metadata adoption",
        "Done: design built-in CLI help policy in",
        "Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata",
    ] {
        assert!(
            design.contains(required),
            "CLI short option metadata design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
    ] {
        assert!(
            text.contains("cli-short-option-metadata.md")
                || text.contains("@cli(short")
                || text.contains("CLI short option metadata"),
            "{label} must surface the CLI short option metadata design"
        );
    }

    assert!(
        implementation_resume.contains("| 257. CLI short option metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 258. CLI short option metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 259. post-CLI short option metadata adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 260. CLI positional field metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 261. CLI positional field metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark CLI short option metadata design done and queue implementation"
    );
}

#[test]
fn cli_short_option_metadata_implementation_is_covered() {
    let parser = read("src/parser.rs");
    let typing = read("src/typing.rs");
    let cli_schema = read("src/cli_schema.rs");
    let typed_hir = read("src/typed_hir.rs");
    let mir = read("src/mir.rs");
    let interface = read("src/interface.rs");
    let package_signature = read("src/package_signature.rs");
    let runtime = read("src/runtime.rs");
    let std_package = read("src/std_package.rs");
    let sample = read("samples/projects/cli_tool/src/main/main.muga");
    let schema_sample = read("samples/packages/app/std_cli_schema/main.muga");
    let project_template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/cli-short-option-metadata.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");

    for (label, text, required) in [
        (
            "parser",
            parser.as_str(),
            "CLI short option names require string literals",
        ),
        ("parser", parser.as_str(), "is_cli_short_option_token"),
        ("typing", typing.as_str(), "cli_short_names"),
        ("typing", typing.as_str(), "duplicate CLI short option name"),
        ("typing", typing.as_str(), "short: field.cli_short"),
        (
            "CliSchema",
            cli_schema.as_str(),
            "pub short: Option<Symbol>",
        ),
        ("CliSchema", cli_schema.as_str(), "\"CS\""),
        (
            "CliSchema",
            cli_schema.as_str(),
            "cli_schema_artifacts_read_old_payloads_without_short_metadata",
        ),
        ("typed HIR", typed_hir.as_str(), "cli_short"),
        ("MIR", mir.as_str(), "short: field.short"),
        ("interface", interface.as_str(), "field.cli_short.is_some()"),
        ("interface", interface.as_str(), "\"short\".to_string()"),
        ("package signature", package_signature.as_str(), "cli_short"),
        ("runtime", runtime.as_str(), "cli_field_by_short_name"),
        ("runtime", runtime.as_str(), "cli_usage_option_cell"),
        (
            "runtime",
            runtime.as_str(),
            "cli_arg_looks_like_option_marker",
        ),
        (
            "std::cli package",
            std_package.as_str(),
            "pub fn has_short_flag(args: List[String], name: String): Bool",
        ),
    ] {
        assert!(
            text.contains(required),
            "{label} missing CLI short option implementation evidence `{required}`"
        );
    }

    for (label, text) in [
        ("cli-tool sample", sample.as_str()),
        ("std_cli_schema sample", schema_sample.as_str()),
        ("project template", project_template.as_str()),
    ] {
        assert!(
            text.contains("@cli(short:") && text.contains("cli::parse_request[Root]")
                || label == "std_cli_schema sample"
                    && text.contains("@cli(name: \"host\", short: \"H\""),
            "{label} must adopt CLI short option metadata"
        );
    }

    for required in [
        "standard_cli_short_option_metadata_parse_and_usage_runs",
        "standard_cli_short_option_metadata_artifact_run_uses_schema_payload",
        "duplicate CLI short option name `h`",
        "CLI short option names must be one ASCII letter",
        "-H, --host <String>",
        "MissingValue:-H",
        "UnknownArgument:-x",
        "cli::has_short_flag([\"-h\"",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing CLI short option coverage `{required}`"
        );
    }

    assert!(
        design.contains("Status: CLI short option metadata implemented")
            && design.contains("Done: implement `@cli(short: \"...\")`")
            && design.contains("Done: audit CLI short option metadata adoption")
            && design.contains("Done: design CLI positional field metadata in")
            && design.contains("Done: audit CLI positional field metadata adoption")
            && design.contains("Done: design built-in CLI help policy in")
            && design.contains("Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata")
            && readme.contains("CLI short option metadata implementation")
            && roadmap.contains("implements field-level `@cli(short: \"x\")`")
            && stdlib_review.contains("standard_cli_short_option_metadata_parse_and_usage_runs")
            && practical.contains("cli::has_short_flag")
            && implementation_resume.contains("| 258. CLI short option metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 259. post-CLI short option metadata adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 260. CLI positional field metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 261. CLI positional field metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "docs and implementation queue must mark CLI short option metadata implementation done"
    );
}

#[test]
fn post_cli_short_option_metadata_adoption_gap_selection_is_documented() {
    let selection = read("docs/post-cli-short-option-metadata-adoption-gap-selection.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let short_design = read("docs/cli-short-option-metadata.md");

    for required in [
        "Status: CLI positional field metadata design selected",
        "Field-level `@cli(short: \"x\")` metadata is implemented",
        "Current Adoption Result",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Candidates Compared",
        "Typed CLI positional field metadata design",
        "Combined short flags such as `-abc`",
        "Attached short values such as `-ofile`",
        "Built-in `--help` / `-h` command framework",
        "Subcommands",
        "Shell completion generation",
        "TOML or config discovery automation",
        "Use only manual `std::cli::positional` helpers",
        "Selected Slice",
        "Design typed CLI positional field metadata before implementation",
        "public syntax",
        "ordering rules",
        "supported first field types",
        "`--` behavior",
        "generated `usage_for[T]` and `usage_for_required[T]` layout",
        "Recommended Order",
        "Done: audit CLI short option metadata adoption",
        "Done: design typed CLI positional field metadata in",
        "Done: implement CLI positional field metadata",
        "Done: audit CLI positional field metadata adoption",
        "Done: design built-in CLI help policy in",
        "Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata",
    ] {
        assert!(
            selection.contains(required),
            "post-CLI short option metadata adoption selection missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
    ] {
        assert!(
            text.contains("post-cli-short-option-metadata-adoption-gap-selection.md")
                && text.contains("typed CLI positional field metadata"),
            "{label} must surface the post-CLI short option metadata adoption audit"
        );
    }

    assert!(
        short_design.contains("Done: audit CLI short option metadata adoption")
            && implementation_resume
                .contains("| 259. post-CLI short option metadata adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 260. CLI positional field metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 261. CLI positional field metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 262. post-CLI positional field metadata adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 263. built-in CLI help policy design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 264. built-in CLI help helpers implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark CLI short option adoption audit done and queue positional metadata design"
    );
}

#[test]
fn cli_positional_field_metadata_design_is_documented() {
    let design = read("docs/cli-positional-field-metadata.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let selection = read("docs/post-cli-short-option-metadata-adoption-gap-selection.md");

    for required in [
        "Status: CLI positional field metadata implemented",
        "@cli(positional: 1, help: \"Input source file\")",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Public Syntax",
        "positive 1-based integer literal",
        "indexes must be contiguous from `1`",
        "final positional index and captures all remaining positional operands",
        "may combine with `help`",
        "may not combine with `name`, `short`, `alias`, or `hidden`",
        "Supported First Types",
        "zero-payload concrete enums",
        "`List[T]` for the supported scalar/enum shapes, only as the final positional",
        "Parser Behavior",
        "`--` stops option parsing",
        "dash-leading tokens before `--` remain option markers",
        "MissingArgument",
        "UnknownArgument",
        "Usage Rendering",
        "Arguments:",
        "<input-path>",
        "[label...]",
        "Schema And Artifacts",
        "`CliFieldSchema` should carry `position: Option<u32>`",
        "old interfaces and old schema payloads without positional metadata",
        "readable as `position = None`",
        "Candidates Compared",
        "`@cli(positional: 1)` with explicit 1-based indexes",
        "`@cli(positional)` marker ordered by declaration",
        "`@cli(position: 0)` zero-based indexes",
        "Allow `name`, `short`, or `alias` together with `positional`",
        "Use only manual `std::cli::positional` helpers",
        "Diagnostics",
        "Non-Goals",
        "Implementation Plan",
        "Done: implement `@cli(positional: N)`",
        "Done: audit CLI positional field metadata adoption",
        "Done: design built-in CLI help policy in",
        "Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata",
    ] {
        assert!(
            design.contains(required),
            "CLI positional field metadata design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("selection", selection.as_str()),
    ] {
        assert!(
            text.contains("cli-positional-field-metadata.md")
                && text.contains("CLI positional field metadata")
                || text.contains("@cli(positional"),
            "{label} must surface the CLI positional field metadata design"
        );
    }

    assert!(
        implementation_resume.contains("| 260. CLI positional field metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 261. CLI positional field metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 262. post-CLI positional field metadata adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 263. built-in CLI help policy design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 264. built-in CLI help helpers implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark CLI positional design done and queue implementation"
    );
}

#[test]
fn cli_positional_field_metadata_implementation_is_covered() {
    let parser = read("src/parser.rs");
    let typing = read("src/typing.rs");
    let cli_schema = read("src/cli_schema.rs");
    let typed_hir = read("src/typed_hir.rs");
    let mir = read("src/mir.rs");
    let interface = read("src/interface.rs");
    let package_signature = read("src/package_signature.rs");
    let runtime = read("src/runtime.rs");
    let sample = read("samples/projects/cli_tool/src/main/main.muga");
    let project_template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/cli-positional-field-metadata.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");

    for (label, text, required) in [
        (
            "parser",
            parser.as_str(),
            "CLI positional indexes require integer literals",
        ),
        (
            "parser",
            parser.as_str(),
            "CLI positional may be specified only once",
        ),
        ("typing", typing.as_str(), "cli_positions"),
        ("typing", typing.as_str(), "duplicate CLI positional index"),
        ("typing", typing.as_str(), "CLI positional List field"),
        (
            "CliSchema",
            cli_schema.as_str(),
            "pub position: Option<u32>",
        ),
        ("CliSchema", cli_schema.as_str(), "\"CP\""),
        (
            "CliSchema",
            cli_schema.as_str(),
            "CLI positional indexes must be positive",
        ),
        ("typed HIR", typed_hir.as_str(), "cli_position"),
        ("MIR", mir.as_str(), "position: field.position"),
        ("interface", interface.as_str(), "\"position\".to_string()"),
        (
            "package signature",
            package_signature.as_str(),
            "cli_position_from_attributes",
        ),
        ("runtime", runtime.as_str(), "cli_assign_positionals"),
        ("runtime", runtime.as_str(), "cli_positional_argument_label"),
        ("runtime", runtime.as_str(), "cli_visible_positional_fields"),
        ("runtime", runtime.as_str(), "cli_bool_literal"),
    ] {
        assert!(
            text.contains(required),
            "{label} missing CLI positional implementation evidence `{required}`"
        );
    }

    for (label, text) in [
        ("cli-tool sample", sample.as_str()),
        ("project template", project_template.as_str()),
    ] {
        assert!(
            text.contains("@cli(positional: 1") && text.contains("parse_request[Root]"),
            "{label} must adopt CLI positional field metadata"
        );
    }

    for required in [
        "standard_cli_positional_field_metadata_parse_and_usage_runs",
        "standard_cli_positional_field_metadata_artifact_run_uses_schema_payload",
        "standard_cli_positional_field_metadata_rejects_invalid_contracts",
        "MissingArgument:<input-path>",
        "Usage: app [options] <input-path> [output-path] [labels...]",
        "duplicate CLI positional index `1`",
        "CLI positional List field `labels`",
        "may not combine `positional` with `name`, `short`, `alias`, or `hidden`",
        "not supported by CLI positional parsing",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing CLI positional coverage `{required}`"
        );
    }

    assert!(
        design.contains("Status: CLI positional field metadata implemented")
            && design.contains("Done: implement `@cli(positional: N)`")
            && design.contains("Done: audit CLI positional field metadata adoption")
            && design.contains("Done: design built-in CLI help policy in")
            && design.contains("Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata")
            && readme.contains("CLI positional field metadata implementation")
            && roadmap.contains("implements field-level `@cli(positional: N)`")
            && stdlib_review
                .contains("standard_cli_positional_field_metadata_parse_and_usage_runs")
            && practical.contains("implements `@cli(positional: N)` typed operands")
            && implementation_resume
                .contains("| 261. CLI positional field metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 262. post-CLI positional field metadata adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 263. built-in CLI help policy design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 264. built-in CLI help helpers implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "docs and implementation queue must mark CLI positional metadata implementation done"
    );
}

#[test]
fn post_cli_positional_field_metadata_adoption_gap_selection_is_documented() {
    let selection = read("docs/post-cli-positional-field-metadata-adoption-gap-selection.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let positional_design = read("docs/cli-positional-field-metadata.md");

    for required in [
        "Status: built-in CLI help policy design completed",
        "Field-level `@cli(positional: N)` metadata is implemented",
        "Current Adoption Result",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Candidates Compared",
        "Built-in CLI help policy design",
        "Implement built-in help immediately",
        "Combined short flags such as `-abc`",
        "Attached short values such as `-ofile`",
        "Subcommands",
        "Shell completion generation",
        "Custom positional labels or option+positional dual fields",
        "TOML/config discovery automation",
        "Selected Slice",
        "The built-in CLI help policy is now designed",
        "cli::help_for_required[T](program)",
        "public API shape",
        "generated usage includes `-h, --help`",
        "`--` affects help detection",
        "Recommended Order",
        "Done: audit CLI positional field metadata adoption",
        "Done: design built-in CLI help policy in",
        "Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata",
    ] {
        assert!(
            selection.contains(required),
            "post-CLI positional field metadata adoption selection missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
    ] {
        assert!(
            text.contains("post-cli-positional-field-metadata-adoption-gap-selection.md")
                && (text.contains("built-in CLI help policy")
                    || (text.contains("cli-built-in-help-policy.md")
                        && text.contains("cli::help_requested"))),
            "{label} must surface the post-CLI positional metadata adoption audit"
        );
    }

    assert!(
        positional_design.contains("Done: audit CLI positional field metadata adoption")
            && positional_design.contains("Done: design built-in CLI help policy in")
            && positional_design.contains("Done: implement built-in CLI help helpers. Done: audit built-in CLI help helper adoption. Done: design parse-integrated CLI help workflow in [parse-integrated-cli-help-workflow.md](parse-integrated-cli-help-workflow.md). Done: implement parse-integrated CLI help workflow. Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata")
            && implementation_resume
                .contains("| 262. post-CLI positional field metadata adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 263. built-in CLI help policy design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 264. built-in CLI help helpers implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark positional metadata adoption audit done and queue built-in help policy design"
    );
}

#[test]
fn cli_built_in_help_policy_design_is_documented() {
    let design = read("docs/cli-built-in-help-policy.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let positional_selection =
        read("docs/post-cli-positional-field-metadata-adoption-gap-selection.md");

    for required in [
        "Status: built-in CLI help helpers implemented",
        "pub fn help_requested(args: List[String]): Bool",
        "pub fn help_for[T](program: String, defaults: T): String",
        "pub fn help_for_required[T](program: String): String",
        "cli::help_requested(args)",
        "cli::help_for_required[Command](\"cli-tool\")",
        "`--` stops help detection",
        "app-owned printing/status decisions",
        "the runtime does not print help automatically",
        "the runtime does not exit the program automatically",
        "Generated help should preserve the existing usage rendering",
        "-h, --help  Show this help",
        "reserves `--help` and `-h`",
        "Schema And Artifacts",
        "help_for_required` requires exactly one explicit concrete non-generic record",
        "Candidates Compared",
        "`cli::help_requested` plus generated `help_for` / `help_for_required`",
        "Parse-integrated `cli::parse_with_help[T]`",
        "Runtime auto-print and exit",
        "Treat help as `cli::ErrorKind::Help`",
        "Reserve `-h` globally",
        "Keep only manual `has_flag` / `has_short_flag` help branches",
        "Diagnostics",
        "Non-Goals",
        "Done: implement `cli::help_requested`, `cli::help_for`,",
        "Done: audit built-in CLI help helper adoption",
        "Done: design parse-integrated CLI help workflow in",
        "Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata",
    ] {
        assert!(
            design.contains(required),
            "built-in CLI help policy design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("positional selection", positional_selection.as_str()),
    ] {
        assert!(
            text.contains("cli-built-in-help-policy.md")
                && text.contains("cli::help_requested")
                && text.contains("cli::help_for_required"),
            "{label} must surface the built-in CLI help policy design"
        );
    }

    assert!(
        implementation_resume.contains("| 263. built-in CLI help policy design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 264. built-in CLI help helpers implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 265. post-built-in CLI help helper adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 266. parse-integrated CLI help workflow design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 267. parse-integrated CLI help workflow implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark built-in CLI help helpers done and queue parse-integrated workflow design"
    );
}

#[test]
fn cli_built_in_help_helpers_are_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let mir = read("src/mir.rs");
    let bytecode = read("src/bytecode.rs");
    let artifact = read("src/implementation_artifact.rs");
    let runtime = read("src/runtime.rs");
    let project_template = read("src/project_template.rs");
    let config_sample = read("samples/projects/config_app/src/main/main.muga");
    let cli_tool_sample = read("samples/projects/cli_tool/src/main/main.muga");
    let examples = read("tests/examples.rs");
    let design = read("docs/cli-built-in-help-policy.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let muga_by_example = read("docs/muga-by-example.md");

    for (label, text, required) in [
        (
            "std::cli package",
            std_package.as_str(),
            "pub fn help_requested(args: List[String]): Bool",
        ),
        (
            "std::cli package",
            std_package.as_str(),
            "pub fn help_for[T](program: String, defaults: T): String",
        ),
        (
            "std::cli package",
            std_package.as_str(),
            "pub fn help_for_required[T](program: String): String",
        ),
        ("typing", typing.as_str(), "std_cli_help_for_bindings"),
        (
            "typing",
            typing.as_str(),
            "std_cli_help_for_required_bindings",
        ),
        ("typing", typing.as_str(), "check_std_cli_help_for_call"),
        (
            "typing",
            typing.as_str(),
            "check_std_cli_help_for_required_call",
        ),
        ("typing", typing.as_str(), "validate_cli_help_schema"),
        ("typing", typing.as_str(), "reserves `--help` and `-h`"),
        ("typed HIR", typed_hir.as_str(), "cli_help_for_schema"),
        (
            "typed HIR",
            typed_hir.as_str(),
            "cli_help_for_required_schema",
        ),
        ("MIR", mir.as_str(), "CliHelpForExpr"),
        ("MIR", mir.as_str(), "CliHelpForRequiredExpr"),
        ("MIR", mir.as_str(), "Expr::CliHelpFor"),
        ("bytecode", bytecode.as_str(), "Instruction::CliHelpFor"),
        (
            "bytecode",
            bytecode.as_str(),
            "Instruction::CliHelpForRequired",
        ),
        (
            "artifact",
            artifact.as_str(),
            "ins\\tCliHelpFor\\t{}\\t{}\\n",
        ),
        (
            "artifact",
            artifact.as_str(),
            "ins\\tCliHelpForRequired\\t{}\\t{}\\n",
        ),
        ("artifact", artifact.as_str(), "invalid CLI help schema"),
        (
            "artifact",
            artifact.as_str(),
            "invalid strict CLI help schema",
        ),
        ("runtime", runtime.as_str(), "Instruction::CliHelpFor"),
        ("runtime", runtime.as_str(), "cli_help_for("),
        ("runtime", runtime.as_str(), "cli_help_for_required("),
        ("runtime", runtime.as_str(), "cli_append_help_option"),
    ] {
        assert!(
            text.contains(required),
            "{label} missing built-in CLI help implementation evidence `{required}`"
        );
    }

    for (label, text) in [
        ("project template", project_template.as_str()),
        ("config-app sample", config_sample.as_str()),
        ("cli-tool sample", cli_tool_sample.as_str()),
    ] {
        assert!(
            text.contains("cli::Request::Help(usage)")
                && (text.contains("cli::parse_request_or(")
                    || text.contains("cli::parse_request[Root]")),
            "{label} must adopt typed CLI request help workflow"
        );
    }

    for required in [
        "standard_cli_help_for_record_runs",
        "standard_cli_help_for_required_record_runs",
        "standard_cli_help_for_required_artifact_run_uses_schema_payload",
        "standard_cli_help_for_required_rejects_invalid_contracts",
        "cli::help_requested([\"--\", \"--help\", \"-h\"])",
        "reserves `--help` and `-h`",
        "-h, --help  Show this help",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing built-in CLI help coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("Muga by Example", muga_by_example.as_str()),
    ] {
        assert!(
            text.contains("cli::help_requested")
                && text.contains("cli::help_for")
                && text.contains("cli::help_for_required"),
            "{label} must surface implemented built-in CLI help helpers"
        );
    }

    assert!(
        design.contains("Status: built-in CLI help helpers implemented")
            && design.contains("Done: implement `cli::help_requested`")
            && design.contains("Done: audit built-in CLI help helper adoption")
            && design.contains("Done: design parse-integrated CLI help workflow in")
            && design.contains("Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata")
            && implementation_resume.contains("| 264. built-in CLI help helpers implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 265. post-built-in CLI help helper adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 266. parse-integrated CLI help workflow design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 267. parse-integrated CLI help workflow implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume.contains(
                "Next recommended slice: implement CLI subcommand metadata"
            ),
        "docs and implementation queue must mark built-in CLI help helper implementation done"
    );
}

#[test]
fn post_built_in_cli_help_helper_adoption_gap_selection_is_documented() {
    let selection = read("docs/post-built-in-cli-help-helper-adoption-gap-selection.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let help_policy = read("docs/cli-built-in-help-policy.md");

    for required in [
        "Status: parse-integrated CLI help workflow design selected",
        "Built-in CLI help helpers are implemented",
        "Current Adoption Result",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Candidates Compared",
        "Parse-integrated CLI help workflow design",
        "Implement parse-integrated help immediately",
        "Combined short flags such as `-abc`",
        "Attached short values such as `-ofile`",
        "Subcommands",
        "Shell completion generation",
        "TOML/config discovery automation",
        "Runtime auto-print and exit",
        "Keep only low-level helpers",
        "Selected Slice",
        "Design a parse-integrated CLI help workflow before implementation",
        "generic `cli::Request[T]`",
        "app-owned printing and status decisions",
        "Recommended Order",
        "Done: audit built-in CLI help helper adoption",
        "Done: design parse-integrated CLI help workflow in",
        "Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata",
    ] {
        assert!(
            selection.contains(required),
            "post-built-in CLI help helper adoption selection missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
    ] {
        assert!(
            text.contains("post-built-in-cli-help-helper-adoption-gap-selection.md")
                && text.contains("parse-integrated CLI help workflow"),
            "{label} must surface the post-built-in CLI help helper adoption audit"
        );
    }

    assert!(
        help_policy.contains("Done: audit built-in CLI help helper adoption")
            && help_policy.contains("Done: design parse-integrated CLI help workflow in")
            && help_policy.contains("Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata")
            && implementation_resume
                .contains("| 265. post-built-in CLI help helper adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 266. parse-integrated CLI help workflow design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 267. parse-integrated CLI help workflow implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume.contains(
                "Next recommended slice: implement CLI subcommand metadata"
            ),
        "implementation queue must mark built-in help adoption audit done and queue parse-integrated help workflow implementation"
    );
}

#[test]
fn parse_integrated_cli_help_workflow_design_is_documented() {
    let design = read("docs/parse-integrated-cli-help-workflow.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let selection = read("docs/post-built-in-cli-help-helper-adoption-gap-selection.md");

    for required in [
        "Status: parse-integrated CLI help workflow implemented",
        "pub enum Request[T]",
        "Help(String)",
        "Parsed(T)",
        "pub fn parse_request[T](args: List[String], program: String): Result[Request[T], Error]",
        "pub fn parse_request_or[T](args: List[String], program: String, defaults: T): Result[Request[T], Error]",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Public API",
        "The argument order follows parsing",
        "Help therefore wins over unrelated parse errors before `--`",
        "Schema And Artifacts",
        "allow explicit call type arguments for `parse_request[T]`",
        "Diagnostics",
        "`cli::parse_request` requires exactly 1 explicit record type argument",
        "Candidates Compared",
        "`Request[T]` plus `parse_request[T]` / `parse_request_or[T]`",
        "`parse_with_help[T]` / `parse_or_with_help[T]` names",
        "Return `Result[T, cli::Error]` where help is `ErrorKind::Help`",
        "Runtime auto-print and exit",
        "Non-Goals",
        "Implementation Plan",
        "Done: implement `cli::Request[T]`, `cli::parse_request[T]`, and",
        "Done: audit parse-integrated CLI help workflow adoption in",
        "Done: design compact CLI short option syntax in",
        "Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata",
    ] {
        assert!(
            design.contains(required),
            "parse-integrated CLI help workflow design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("post-built-in selection", selection.as_str()),
    ] {
        assert!(
            text.contains("parse-integrated-cli-help-workflow.md")
                && text.contains("parse_request")
                && text.contains("parse_request_or"),
            "{label} must surface the parse-integrated CLI help workflow design"
        );
    }

    assert!(
        selection.contains("Done: design parse-integrated CLI help workflow in")
            && selection.contains("Done: audit parse-integrated CLI help workflow adoption. Done: design compact CLI short option syntax in [compact-cli-short-option-syntax.md](compact-cli-short-option-syntax.md). Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata")
            && implementation_resume.contains("| 266. parse-integrated CLI help workflow design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 267. parse-integrated CLI help workflow implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume.contains(
                "| 268. post-parse-integrated CLI help workflow adoption gap selection |"
            )
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 269. compact CLI short option syntax design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 270. compact CLI short option syntax implementation |")
            && implementation_resume.contains("| runtime/tests/docs | Done |")
            && implementation_resume.contains(
                "Next recommended slice: implement CLI subcommand metadata"
            ),
        "implementation queue must mark parse-integrated help workflow design done and queue implementation"
    );
}

#[test]
fn parse_integrated_cli_help_workflow_is_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let mir = read("src/mir.rs");
    let bytecode = read("src/bytecode.rs");
    let artifact = read("src/implementation_artifact.rs");
    let runtime = read("src/runtime.rs");
    let project_template = read("src/project_template.rs");
    let config_sample = read("samples/projects/config_app/src/main/main.muga");
    let cli_tool_sample = read("samples/projects/cli_tool/src/main/main.muga");
    let examples = read("tests/examples.rs");
    let design = read("docs/parse-integrated-cli-help-workflow.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let by_example = read("docs/muga-by-example.md");

    for (label, text, required) in [
        (
            "std::cli package",
            std_package.as_str(),
            "pub enum Request[T]",
        ),
        (
            "std::cli package",
            std_package.as_str(),
            "pub fn parse_request[T](args: List[String], program: String): Result[Request[T], Error]",
        ),
        (
            "std::cli package",
            std_package.as_str(),
            "pub fn parse_request_or[T](args: List[String], program: String, defaults: T): Result[Request[T], Error]",
        ),
        ("typing", typing.as_str(), "std_cli_parse_request_bindings"),
        (
            "typing",
            typing.as_str(),
            "std_cli_parse_request_or_bindings",
        ),
        (
            "typing",
            typing.as_str(),
            "check_std_cli_parse_request_call",
        ),
        (
            "typing",
            typing.as_str(),
            "check_std_cli_parse_request_or_call",
        ),
        (
            "typing",
            typing.as_str(),
            "`cli::parse_request` requires exactly 1 explicit record type argument",
        ),
        ("typed HIR", typed_hir.as_str(), "cli_parse_request_schema"),
        (
            "typed HIR",
            typed_hir.as_str(),
            "cli_parse_request_or_schema",
        ),
        ("MIR", mir.as_str(), "CliParseRequestExpr"),
        ("MIR", mir.as_str(), "CliParseRequestOrExpr"),
        (
            "bytecode",
            bytecode.as_str(),
            "Instruction::CliParseRequest",
        ),
        (
            "bytecode",
            bytecode.as_str(),
            "Instruction::CliParseRequestOr",
        ),
        (
            "artifact",
            artifact.as_str(),
            "ins\\tCliParseRequest\\t{}\\t{}\\n",
        ),
        (
            "artifact",
            artifact.as_str(),
            "ins\\tCliParseRequestOr\\t{}\\t{}\\n",
        ),
        ("runtime", runtime.as_str(), "Instruction::CliParseRequest"),
        ("runtime", runtime.as_str(), "cli_request_help"),
        ("runtime", runtime.as_str(), "cli_help_requested"),
    ] {
        assert!(
            text.contains(required),
            "{label} missing parse-integrated CLI help implementation evidence `{required}`"
        );
    }

    for (label, text, required) in [
        (
            "project template",
            project_template.as_str(),
            "cli::parse_request[Root](args, \"cli-tool\")",
        ),
        (
            "project template",
            project_template.as_str(),
            "cli::parse_request_or(settings_args(args), \"config-app\", default_settings())",
        ),
        (
            "config-app sample",
            config_sample.as_str(),
            "cli::parse_request_or(settings_args(args), \"config-app\", default_settings())",
        ),
        (
            "cli-tool sample",
            cli_tool_sample.as_str(),
            "cli::parse_request[Root](args, \"cli-tool\")",
        ),
        (
            "config-app sample",
            config_sample.as_str(),
            "cli::Request::Help(usage)",
        ),
        (
            "cli-tool sample",
            cli_tool_sample.as_str(),
            "cli::Request::Parsed(root)",
        ),
    ] {
        assert!(
            text.contains(required),
            "{label} missing typed request adoption evidence `{required}`"
        );
    }

    for required in [
        "standard_cli_parse_request_required_record_runs",
        "standard_cli_parse_request_or_record_overlay_runs",
        "standard_cli_parse_request_artifact_run_uses_schema_payload",
        "standard_cli_parse_request_rejects_invalid_contracts",
        "cli_new_creates_cli_tool_template",
        "cli::parse_request[Root]",
        "cli::parse_request_or(settings_args(args)",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing parse-integrated CLI request coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("Muga by Example", by_example.as_str()),
    ] {
        assert!(
            text.contains("cli::parse_request") && text.contains("cli::Request"),
            "{label} must surface implemented parse-integrated CLI request workflow"
        );
    }

    assert!(
        design.contains("Status: parse-integrated CLI help workflow implemented")
            && design.contains("Done: implement `cli::Request[T]`")
            && design.contains("Done: audit parse-integrated CLI help workflow adoption in")
            && design.contains("Done: design compact CLI short option syntax in")
            && design.contains("Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata")
            && implementation_resume
                .contains("| 267. parse-integrated CLI help workflow implementation |")
            && implementation_resume.contains(
                "| typing/std_package/mir/bytecode/runtime/artifacts/samples/tests/docs | Done |"
            )
            && implementation_resume.contains(
                "| 268. post-parse-integrated CLI help workflow adoption gap selection |"
            )
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 269. compact CLI short option syntax design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 270. compact CLI short option syntax implementation |")
            && implementation_resume.contains("| runtime/tests/docs | Done |")
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "docs and queue must mark parse-integrated CLI help workflow implementation done"
    );
}

#[test]
fn post_parse_integrated_cli_help_workflow_adoption_gap_selection_is_documented() {
    let selection = read("docs/post-parse-integrated-cli-help-workflow-adoption-gap-selection.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let design = read("docs/parse-integrated-cli-help-workflow.md");

    for required in [
        "Status: compact CLI short option syntax design selected",
        "Current Adoption Result",
        "cli::parse_request[Root](args, \"cli-tool\")",
        "cli::parse_request_or(settings_args(args), \"config-app\", default_settings())",
        "cli::Request::Help(String)",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Candidates Compared",
        "Compact CLI short option syntax design",
        "Implement compact short options immediately",
        "Extend request workflow into every stdlib sample",
        "Runtime auto-print and exit on help",
        "Subcommands",
        "Shell completion generation for generated apps",
        "TOML/config discovery automation",
        "Selected Slice",
        "Design compact CLI short option syntax before implementation",
        "`-abc`",
        "`-ovalue`",
        "`-abovalue`",
        "`-o=value`",
        "`-abo=value`",
        "runtime parser/diagnostics change",
        "Recommended Order",
        "Done: audit parse-integrated CLI help workflow adoption here",
        "Done: design compact CLI short option syntax in",
        "Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata",
    ] {
        assert!(
            selection.contains(required),
            "post-parse-integrated CLI help workflow adoption selection missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
    ] {
        assert!(
            text.contains("post-parse-integrated-cli-help-workflow-adoption-gap-selection.md")
                && text.contains("compact CLI short option syntax"),
            "{label} must surface the post-parse-integrated CLI help workflow adoption audit"
        );
    }

    assert!(
        design.contains("Done: audit parse-integrated CLI help workflow adoption in")
            && design.contains("Done: design compact CLI short option syntax in")
            && design.contains("Done: implement compact CLI short option syntax. Done: audit compact CLI short option syntax adoption. Next: design CLI subcommand metadata")
            && implementation_resume.contains(
                "| 268. post-parse-integrated CLI help workflow adoption gap selection |"
            )
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 269. compact CLI short option syntax design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 270. compact CLI short option syntax implementation |")
            && implementation_resume.contains("| runtime/tests/docs | Done |")
            && implementation_resume
                .contains("Next recommended slice: implement CLI subcommand metadata"),
        "implementation queue must mark request workflow adoption audit done and queue compact short option syntax design"
    );
}

#[test]
fn compact_cli_short_option_syntax_design_is_documented() {
    let design = read("docs/compact-cli-short-option-syntax.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let selection = read("docs/post-parse-integrated-cli-help-workflow-adoption-gap-selection.md");

    for required in [
        "Status: compact CLI short option syntax implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Scope",
        "cli::parse[T](args)",
        "cli::parse_or[T](args, defaults)",
        "cli::parse_request[T](args, program)",
        "cli::parse_request_or[T](args, program, defaults)",
        "Accepted Forms",
        "New compact forms",
        "`-abc`",
        "`-ofile`",
        "`-abo=value`",
        "Token Rules",
        "Without `=`",
        "With `=`",
        "`Bool`, `Option[Bool]`, and `List[Bool]` count as bare-bool fields",
        "Diagnostics",
        "unknown CLI option `-x`",
        "missing value for `-o`",
        "Help And Request Workflow Boundary",
        "`cli::help_requested(args)` and the request helpers continue to recognize exact",
        "Candidates Compared",
        "One compact grammar for combined bool flags and attached values",
        "Combined bool flags only",
        "Attached values only",
        "Treat `-vfalse` as `-v=false`",
        "Schema metadata for cluster behavior",
        "Extend built-in help detection to compact `-h...` tokens",
        "Non-Goals",
        "Implementation Plan",
        "Done: implement compact short token parsing in the runtime parser",
        "Done: audit compact CLI short option syntax adoption in",
        "Done: design CLI subcommand metadata in",
        "Done: implement first enum/variant CLI subcommand metadata plumbing",
        "Done: implement strict command enum schemas and runtime dispatch/help",
        "Done: audit strict command enum schema adoption",
        "Done: design wrapper-record root/global CLI options in",
        "Done: implement `@cli(subcommand)` metadata plumbing in",
        "Done: implement wrapper schema lowering and runtime parse/help",
        "Done: adopt a minimal global option in the strict CLI sample/template",
    ] {
        assert!(
            design.contains(required),
            "compact CLI short option syntax design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
    ] {
        assert!(
            text.contains("compact-cli-short-option-syntax.md") && text.contains("-abc"),
            "{label} must surface compact CLI short option syntax design"
        );
    }

    assert!(
        selection.contains("Done: design compact CLI short option syntax in")
            && implementation_resume.contains("| 269. compact CLI short option syntax design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 270. compact CLI short option syntax implementation |")
            && implementation_resume.contains("| runtime/tests/docs | Done |")
            && implementation_resume
                .contains("| 271. post-compact CLI short option syntax adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 272. CLI subcommand metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 273. CLI subcommand enum metadata plumbing |")
            && implementation_resume.contains(
                "| parser/formatter/typing/typed_hir/package_signature/interfaces/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 274. CLI subcommand strict schema implementation |")
            && implementation_resume.contains("| typing/mir/bytecode/runtime/artifacts/tests/docs | Done |")
            && implementation_resume
                .contains("| 275. CLI subcommand adoption audit |")
            && implementation_resume.contains("| docs/tests/samples/templates | Done |")
            && implementation_resume
                .contains("| 276. CLI wrapper-record root/global options design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 277. CLI wrapper-record subcommand metadata plumbing |"),
        "implementation queue must mark compact short option syntax design done and queue implementation"
    );
}

#[test]
fn compact_cli_short_option_syntax_is_implemented_and_covered() {
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/compact-cli-short-option-syntax.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");

    for required in [
        "cli_parse_short_token",
        "cli_short_run_starts_with_known_short",
        "cli_parse_exact_short_token",
        "cli_parse_compact_short_run",
        "cli_parse_short_run_with_explicit_value",
        "cli_parse_and_merge_field",
        "CliErrorKind::UnknownArgument",
        "CliErrorKind::MissingValue",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing compact short option implementation evidence `{required}`"
        );
    }

    for required in [
        "standard_cli_compact_short_option_syntax_runs",
        "cli::parse_request[Command]",
        "\"-abn3\"",
        "\"-oout.txt\"",
        "\"-vv=false\"",
        "\"-abo=log.txt\"",
        "\"-dc3\"",
        "\"-aApply\"",
        "\"-Tops\"",
        "\"-oKai\"",
        "MissingValue:-o",
        "UnknownArgument:-x",
        "UnknownArgument:-zoo",
        "InvalidValue:-n",
        "Help:Usage: compact",
        "\"-uartifact\"",
        "\"-p9000\"",
        "standard_cli_short_option_metadata_artifact_run_uses_schema_payload",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing compact short option coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
    ] {
        assert!(
            text.contains("compact-cli-short-option-syntax.md")
                && text.contains("implements")
                && text.contains("-abo=value"),
            "{label} must surface implemented compact short option syntax"
        );
    }

    assert!(
        design.contains("Status: compact CLI short option syntax implemented")
            && design.contains("Done: implement compact short token parsing in the runtime parser")
            && design.contains("post-compact-cli-short-option-syntax-adoption-gap-selection.md")
            && design.contains("Done: design CLI subcommand metadata in")
            && design.contains("Done: implement first enum/variant CLI subcommand metadata plumbing")
            && design.contains("Done: implement strict command enum schemas and runtime dispatch/help")
            && design.contains("Done: audit strict command enum schema adoption")
            && design.contains("Done: design wrapper-record root/global CLI options in")
            && design.contains("Done: implement `@cli(subcommand)` metadata plumbing in")
            && design.contains("Done: implement wrapper schema lowering and runtime parse/help")
            && design.contains("Done: adopt a minimal global option in the strict CLI sample/template")
            && implementation_resume
                .contains("| 270. compact CLI short option syntax implementation |")
            && implementation_resume.contains("| runtime/tests/docs | Done |")
            && implementation_resume
                .contains("| 271. post-compact CLI short option syntax adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 272. CLI subcommand metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 273. CLI subcommand enum metadata plumbing |")
            && implementation_resume.contains(
                "| parser/formatter/typing/typed_hir/package_signature/interfaces/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 274. CLI subcommand strict schema implementation |")
            && implementation_resume.contains("| typing/mir/bytecode/runtime/artifacts/tests/docs | Done |")
            && implementation_resume
                .contains("| 275. CLI subcommand adoption audit |")
            && implementation_resume.contains("| docs/tests/samples/templates | Done |")
            && implementation_resume
                .contains("| 276. CLI wrapper-record root/global options design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 277. CLI wrapper-record subcommand metadata plumbing |"),
        "implementation queue must mark compact short option syntax implementation done and queue adoption audit"
    );
}

#[test]
fn post_compact_cli_short_option_syntax_adoption_gap_selection_is_documented() {
    let selection = read("docs/post-compact-cli-short-option-syntax-adoption-gap-selection.md");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");

    for required in [
        "Status: CLI subcommand metadata implemented; sample/template adoption implemented",
        "Current Adoption Result",
        "cli::parse[T](args)",
        "cli::parse_or[T](args, defaults)",
        "cli::parse_request[T](args, program)",
        "cli::parse_request_or[T](args, program, defaults)",
        "samples/projects/cli_tool",
        "`-dc3`",
        "`-aApply`",
        "`-Tops`",
        "`-oKai`",
        "no `CliSchema`, `.mgi`, `.mgb`, project-template, or standard-library API",
        "The remaining CLI adoption gap is now command shape rather than option shape",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Candidates Compared",
        "CLI subcommand metadata design",
        "Implement subcommands immediately",
        "Generated app shell completion generation",
        "TOML/config discovery automation",
        "Runtime-owned printing, exits, or process status API",
        "Rich help polish",
        "Full client generation, generic encoding/decoding, broader validators, or host-effect APIs",
        "Selected Slice",
        "Design CLI subcommand metadata before implementation",
        "enum of command-record payloads",
        "root/global options",
        "subcommand help",
        "CliSchema",
        "how compact short option syntax continues",
        "command schema",
        "Recommended Order",
        "Done: audit compact short option syntax adoption here",
        "Done: design CLI subcommand metadata in",
        "cli-subcommand-metadata.md",
        "Done: implement first enum/variant CLI subcommand metadata plumbing",
        "Done: implement strict command enum schemas and runtime dispatch/help",
        "Done: audit strict command enum schema adoption",
        "post-cli-subcommand-schema-adoption-gap-selection.md",
        "Done: design wrapper-record root/global CLI options in",
        "Done: implement `@cli(subcommand)` metadata plumbing in",
        "Done: implement wrapper schema lowering and runtime parse/help",
        "Done: adopt a minimal global option in the strict CLI sample/template",
    ] {
        assert!(
            selection.contains(required),
            "post-compact CLI short option syntax adoption selection missing `{required}`"
        );
    }

    for required in [
        "manifest_cli_tool_project_sample_runs_with_required_options",
        "cli_new_creates_cli_tool_template",
        "\"-dc3\"",
        "\"-aApply\"",
        "\"-Tops\"",
        "\"-Tprod\"",
        "\"-oKai\"",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing compact CLI adoption evidence `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
    ] {
        assert!(
            text.contains("post-compact-cli-short-option-syntax-adoption-gap-selection.md")
                && text.contains("CLI subcommand metadata"),
            "{label} must surface the post-compact CLI short option syntax adoption audit"
        );
    }

    assert!(
        implementation_resume
            .contains("| 271. post-compact CLI short option syntax adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 272. CLI subcommand metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 273. CLI subcommand enum metadata plumbing |")
            && implementation_resume.contains(
                "| parser/formatter/typing/typed_hir/package_signature/interfaces/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 274. CLI subcommand strict schema implementation |")
            && implementation_resume.contains("| typing/mir/bytecode/runtime/artifacts/tests/docs | Done |")
            && implementation_resume
                .contains("| 275. CLI subcommand adoption audit |")
            && implementation_resume.contains("| docs/tests/samples/templates | Done |")
            && implementation_resume
                .contains("| 276. CLI wrapper-record root/global options design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 277. CLI wrapper-record subcommand metadata plumbing |"),
        "implementation queue must mark post-compact adoption audit and first subcommand metadata plumbing done"
    );
}

#[test]
fn cli_subcommand_metadata_design_is_documented() {
    let design = read("docs/cli-subcommand-metadata.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let compact = read("docs/compact-cli-short-option-syntax.md");
    let selection = read("docs/post-compact-cli-short-option-syntax-adoption-gap-selection.md");

    for required in [
        "Status: strict CLI command enum schemas implemented",
        "enum-backed command trees",
        "@cli(about: \"Project maintenance tool\")",
        "pub enum Command",
        "@cli(name: \"build\", alias: \"b\", about: \"Build package artifacts\")",
        "Build(BuildCommand)",
        "Config(ConfigCommand)",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Selected Public Shape",
        "a concrete non-generic enum",
        "every command variant carries exactly one payload",
        "every payload is either a supported concrete command record or another",
        "every command variant has explicit `@cli(name: \"...\")` metadata",
        "Rejected command enum targets",
        "Public Metadata",
        "Enum declarations support only root or branch summaries",
        "Enum variants support command metadata",
        "`alias: \"b\"` is invoked as `tool b`, not `tool -b`",
        "Strict And Overlay Helper Scope",
        "cli::parse[T](args)",
        "cli::parse_request[T](args, program)",
        "cli::usage_for_required[T](program)",
        "cli::help_for_required[T](program)",
        "Overlay/default helpers remain record-only in this slice",
        "Root And Global Options",
        "@cli(subcommand)",
        "Parsing Semantics",
        "Exact `--help` or exact `-h` before a command token requests help",
        "Compact short option syntax applies only inside the selected leaf record schema",
        "Diagnostics",
        "missing CLI command `<command>`",
        "unknown CLI command `deploy`",
        "Usage And Help Rendering",
        "Usage: tool <command> [args]",
        "Commands:",
        "Usage: tool build [options] <entry>",
        "Schema And Artifacts",
        "pub struct CliSchema",
        "pub commands: Vec<CliCommandVariantSchema>",
        "pub struct CliCommandVariantSchema",
        "new artifact token family, for example `CC`",
        "Implemented metadata plumbing",
        "Implemented strict schema/runtime support",
        "muga-package-interface-v11",
        "standard_cli_subcommand_parse_request_runs",
        "standard_cli_subcommand_parse_request_artifact_run_uses_schema_payload",
        "standard_cli_subcommand_schema_rejects_invalid_contracts",
        "Candidates Compared",
        "Concrete enum of command-record payloads",
        "Record with a subcommand field",
        "Function table or annotated command functions",
        "Support overlay/default command enums immediately",
        "Root/global options in first subcommand slice",
        "Non-Goals",
        "Implementation Plan",
        "Done: design CLI subcommand metadata here",
        "Done: implement the first enum metadata plumbing",
        "Done: implement strict command enum schemas",
        "Done: audit strict command enum schema adoption",
        "post-cli-subcommand-schema-adoption-gap-selection.md",
        "Done: design wrapper-record root/global options in",
        "Done: implement `@cli(subcommand)` parser/formatter/type-checker metadata",
        "Done: implement wrapper schema lowering and runtime parse/help",
        "Done: adopt a minimal global option in the strict CLI sample/template",
    ] {
        assert!(
            design.contains(required),
            "CLI subcommand metadata design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("compact design", compact.as_str()),
        ("post-compact selection", selection.as_str()),
    ] {
        assert!(
            text.contains("cli-subcommand-metadata.md") && text.contains("CLI subcommand metadata"),
            "{label} must surface CLI subcommand metadata design"
        );
    }

    assert!(
        implementation_resume.contains("CLI subcommand metadata design is recorded in")
            && implementation_resume.contains("cli-subcommand-metadata.md")
            && implementation_resume.contains("| 272. CLI subcommand metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 273. CLI subcommand enum metadata plumbing |")
            && implementation_resume.contains(
                "| parser/formatter/typing/typed_hir/package_signature/interfaces/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 274. CLI subcommand strict schema implementation |")
            && implementation_resume
                .contains("| typing/mir/bytecode/runtime/artifacts/tests/docs | Done |")
            && implementation_resume
                .contains("| 275. CLI subcommand adoption audit |")
            && implementation_resume.contains("| docs/tests/samples/templates | Done |")
            && implementation_resume
                .contains("| 276. CLI wrapper-record root/global options design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 277. CLI wrapper-record subcommand metadata plumbing |"),
        "implementation queue must mark subcommand metadata design, first plumbing, and strict schemas done and queue adoption"
    );
}

#[test]
fn cli_subcommand_enum_metadata_plumbing_is_covered() {
    let ast = read("src/ast.rs");
    let parser = read("src/parser.rs");
    let formatter = read("src/formatter.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let package = read("src/package.rs");
    let package_signature = read("src/package_signature.rs");
    let interface = read("src/interface.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/cli-subcommand-metadata.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    assert!(
        ast.contains("pub attributes: Vec<Attribute>"),
        "enum declarations must preserve attributes in the AST"
    );

    for required in [
        "validate_enum_attributes",
        "cli_attribute_is_command_variant_metadata",
        "enum declarations support only `@cli(about:",
        "enum variants support only `@cli(name:",
    ] {
        assert!(
            parser.contains(required),
            "parser missing CLI subcommand enum metadata handling `{required}`"
        );
    }

    assert!(
        formatter.contains("format_attributes(&stmt.attributes"),
        "formatter must preserve enum CLI attributes"
    );
    assert!(
        package.contains("attributes: enumeration.attributes.clone()"),
        "package rewriting must preserve enum CLI attributes"
    );

    for required in [
        "CLI command variant `{}` in enum `{}` requires",
        "duplicate CLI command name `{}` in enum `{}`",
    ] {
        assert!(
            typing.contains(required),
            "typing missing CLI command metadata diagnostic `{required}`"
        );
    }

    for required in [
        "cli_about: Option<String>",
        "cli_name: Option<String>",
        "cli_aliases: Vec<String>",
        "cli_hidden: bool",
    ] {
        assert!(
            typed_hir.contains(required) && package_signature.contains(required),
            "typed HIR and package signatures must preserve command metadata `{required}`"
        );
    }

    for required in [
        "muga-package-interface-v11",
        "\"muga-package-interface-v9\"",
        "enum variant CLI alias count",
        "invalid enum variant CLI alias",
        "variant.cli_hidden == expected.cli_hidden",
    ] {
        assert!(
            interface.contains(required),
            "package interface missing CLI command metadata persistence `{required}`"
        );
    }

    for required in [
        "package_interfaces_preserve_cli_subcommand_metadata_without_source",
        "muga-package-interface-v11",
        "duplicate CLI command name `b`",
    ] {
        assert!(
            examples.contains(required),
            "examples missing CLI subcommand metadata coverage `{required}`"
        );
    }

    assert!(
        design.contains("Implemented metadata plumbing")
            && design.contains("muga-package-interface-v11")
            && implementation_resume.contains("| 273. CLI subcommand enum metadata plumbing |")
            && implementation_resume
                .contains("| 274. CLI subcommand strict schema implementation |"),
        "docs must record first subcommand metadata plumbing and the strict schema slice"
    );
}

#[test]
fn cli_subcommand_strict_schema_implementation_is_covered() {
    let cli_schema = read("src/cli_schema.rs");
    let typing = read("src/typing.rs");
    let mir = read("src/mir.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/cli-subcommand-metadata.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "CliCommandVariantSchema",
        "commands: Vec<CliCommandVariantSchema>",
        "\"CC\"",
        "parse_command_artifact_tokens",
        "CLI_COMMAND_HIDDEN_FLAG",
        "cli_command_schema_artifact_round_trips",
    ] {
        assert!(
            cli_schema.contains(required),
            "CLI schema missing command schema artifact support `{required}`"
        );
    }

    for required in [
        "cli_command_schema_for_enum",
        "command variant `{qualified}` requires",
        "must carry a record or command enum payload",
        "defaults/overlays cannot represent command enum dispatch",
        "payload: Box::new(payload)",
    ] {
        assert!(
            typing.contains(required),
            "typing missing strict command enum schema support `{required}`"
        );
    }

    for required in [
        "CliCommandVariantSchema",
        "payload: Box::new(self.lower_cli_schema(&command.payload))",
    ] {
        assert!(
            mir.contains(required),
            "MIR lowering missing command schema support `{required}`"
        );
    }

    for required in [
        "cli_schema_is_command",
        "cli_parse_command",
        "cli_parse_command_request_outcome",
        "cli_help_for_command",
        "cli_command_by_name",
        "missing required CLI command `<command>`",
        "unknown CLI command `{token}`",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing command schema dispatch/help support `{required}`"
        );
    }

    for required in [
        "standard_cli_subcommand_parse_request_runs",
        "standard_cli_subcommand_parse_request_artifact_run_uses_schema_payload",
        "standard_cli_subcommand_schema_rejects_invalid_contracts",
        "run_path_against_artifact_root",
        ".arg(\"--built\")",
    ] {
        assert!(
            examples.contains(required),
            "examples missing strict CLI subcommand coverage `{required}`"
        );
    }

    assert!(
        design.contains("Status: strict CLI command enum schemas implemented")
            && design.contains("Implemented strict schema/runtime support")
            && implementation_resume
                .contains("| 274. CLI subcommand strict schema implementation |")
            && implementation_resume
                .contains("| typing/mir/bytecode/runtime/artifacts/tests/docs | Done |")
            && implementation_resume.contains("| 275. CLI subcommand adoption audit |")
            && implementation_resume.contains("| docs/tests/samples/templates | Done |"),
        "docs must record implemented strict command enum schemas and the next adoption audit"
    );
}

#[test]
fn cli_subcommand_schema_adoption_audit_is_covered() {
    let sample = read("samples/projects/cli_tool/src/main/main.muga");
    let project_template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let selection = read("docs/post-cli-subcommand-schema-adoption-gap-selection.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let by_example = read("docs/muga-by-example.md");
    let design = read("docs/cli-subcommand-metadata.md");

    for (label, text) in [
        ("sample", sample.as_str()),
        ("project template", project_template.as_str()),
    ] {
        for required in [
            "pub record Root",
            "@cli(name: \"profile\", short: \"p\", help: \"Execution profile\")",
            "@cli(subcommand)",
            "command: Command",
            "pub enum Command",
            "@cli(name: \"run\", alias: \"r\", about: \"Run the main action\")",
            "Run(RunCommand)",
            "@cli(name: \"inspect\", alias: \"i\", about: \"Inspect one target\")",
            "Inspect(InspectCommand)",
            "pub record RunCommand",
            "pub record InspectCommand",
            "cli::parse_request[Root](args, \"cli-tool\")",
            "Command::Run(run_options)",
            "Command::Inspect(inspect_command)",
        ] {
            assert!(
                text.contains(required),
                "{label} missing CLI subcommand adoption evidence `{required}`"
            );
        }
    }

    for required in [
        "cli_new_creates_cli_tool_template",
        "Usage: cli-tool [global-options] <command> [args]",
        "Usage: cli-tool run [options] <target>",
        "Result::Ok(profile|dev|run|service|3|Apply|true|ops,prod|Kai)",
        "Result::Ok(inspect|service|true)",
        "Result::Err(cli UnknownArgument deploy",
        "\\\"stdout\\\":\\\"cli-tool profile|prod|run|batch|5|Apply|true|ops,prod|Kai\\\\n\\\"",
        "\\\"mainResult\\\":\\\"Result::Ok(profile|prod|run|batch|5|Apply|true|ops,prod|Kai)\\\"",
    ] {
        assert!(
            examples.contains(required),
            "examples missing CLI subcommand sample/template adoption coverage `{required}`"
        );
    }

    for required in [
        "Status: strict CLI tool sample and template subcommand adoption implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Candidates Compared",
        "Adopt subcommands in `samples/projects/cli_tool` and the generated `cli-tool` template",
        "Command::Run(RunCommand)",
        "Command::Inspect(InspectCommand)",
        "wrapper-record root/global options",
        "Recommended Order",
        "Done: design wrapper-record root/global CLI options in",
        "Done: implement `@cli(subcommand)` parser/formatter/type-checker metadata",
        "Done: implement wrapper schema lowering and runtime parse/help",
        "Done: adopt a minimal global option in the strict CLI sample/template",
    ] {
        assert!(
            selection.contains(required),
            "CLI subcommand adoption selection missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("CLI subcommand design", design.as_str()),
    ] {
        assert!(
            text.contains("post-cli-subcommand-schema-adoption-gap-selection.md")
                && text.contains("cli-tool")
                && (text.contains("run") || text.contains("inspect")),
            "{label} must surface CLI subcommand sample/template adoption"
        );
    }

    assert!(
        implementation_resume.contains("| 275. CLI subcommand adoption audit |")
            && implementation_resume.contains("| docs/tests/samples/templates | Done |")
            && implementation_resume
                .contains("| 276. CLI wrapper-record root/global options design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 277. CLI wrapper-record subcommand metadata plumbing |"),
        "implementation queue must mark CLI subcommand adoption done and queue root/global option design"
    );
}

#[test]
fn cli_wrapper_root_options_design_is_documented() {
    let parser = read("src/parser.rs");
    let formatter = read("src/formatter.rs");
    let typing = read("src/typing.rs");
    let typed_hir = read("src/typed_hir.rs");
    let package_signature = read("src/package_signature.rs");
    let interface = read("src/interface.rs");
    let cli_schema = read("src/cli_schema.rs");
    let mir = read("src/mir.rs");
    let implementation_artifact = read("src/implementation_artifact.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/cli-wrapper-root-options.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let by_example = read("docs/muga-by-example.md");
    let subcommand_design = read("docs/cli-subcommand-metadata.md");
    let subcommand_adoption = read("docs/post-cli-subcommand-schema-adoption-gap-selection.md");
    let compact_design = read("docs/compact-cli-short-option-syntax.md");
    let compact_adoption =
        read("docs/post-compact-cli-short-option-syntax-adoption-gap-selection.md");

    for required in [
        "Status: CLI wrapper-record root/global option sample/template adoption implemented",
        "@cli(subcommand)",
        "pub record Root",
        "pub enum Command",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "Selected Public Shape",
        "Wrapper Field Rules",
        "Helper Scope",
        "Parsing Semantics",
        "[\"--verbose\", \"--profile\", \"dev\", \"run\", ...]",
        "Root/global options are not accepted after the command",
        "Usage: tool [global-options] <command> [args]",
        "Global Options:",
        "Schema And Artifacts",
        "CliSubcommandSchema",
        "pub subcommand: Option<CliSubcommandSchema>",
        "`CW`",
        "cli_subcommand: bool",
        "muga-package-interface-v11",
        "Implemented Metadata Plumbing",
        "Implemented Schema And Runtime",
        "Implemented Sample And Template Adoption",
        "@cli(name: \"profile\", short: \"p\", help: \"Execution profile\")",
        "cli::parse_request[Root](args, \"cli-tool\")",
        "Done: adopt a minimal `--profile` / `-p` global option",
        "cli_wrapper_subcommand_field_metadata_plumbing_is_covered",
        "cli_wrapper_subcommand_field_metadata_rejects_invalid_contracts",
        "standard_cli_wrapper_parse_request_runs",
        "standard_cli_wrapper_parse_request_artifact_and_built_runs_use_schema_payload",
        "Candidates Compared",
        "Wrapper record with `@cli(subcommand)` field",
        "Support overlay/default wrapper helpers now",
        "Non-Goals",
        "Done: design wrapper-record root/global options here",
        "Done: implement parser/formatter/type-checker support",
        "Done: lower wrapper schemas through `CliSchema`",
        "Done: design schema-backed generated shell completions",
        "cli-schema-shell-completions.md",
        "Done: implement `muga cli-completions <bash|zsh|fish> --program <name>",
        "Next: audit generated-project shell completion adoption",
    ] {
        assert!(
            design.contains(required),
            "CLI wrapper root-option design missing `{required}`"
        );
    }

    for required in [
        "subcommand_count",
        "CLI subcommand is a flag and does not take a value",
        "record fields support only `@cli(name:",
        "subcommand",
    ] {
        assert!(
            parser.contains(required),
            "parser missing wrapper subcommand metadata support `{required}`"
        );
    }
    assert!(
        formatter.contains("argument.value"),
        "formatter must preserve bare @cli(subcommand) markers"
    );
    for required in [
        "cli_subcommand: bool",
        "cli_subcommand_from_attributes",
        "cli_subcommand_argument_from_attributes",
        "may contain exactly one `@cli(subcommand)` field",
        "may not combine `subcommand` with `name`, `short`, `positional`, `value_source`, `alias`, `help`, or `hidden` metadata",
        "must have a concrete command enum type",
    ] {
        assert!(
            typing.contains(required),
            "typing missing wrapper subcommand metadata support `{required}`"
        );
    }
    for required in ["cli_subcommand: bool", "cli_subcommand_from_attributes"] {
        assert!(
            typed_hir.contains(required) && package_signature.contains(required),
            "typed HIR and package signatures must preserve wrapper subcommand metadata `{required}`"
        );
    }
    for required in [
        "muga-package-interface-v11",
        "\"muga-package-interface-v10\"",
        "field.cli_subcommand",
        "cli_subcommand: bool",
        "flags & !3",
    ] {
        assert!(
            interface.contains(required),
            "package interface missing wrapper subcommand metadata persistence `{required}`"
        );
    }
    for required in [
        "pub struct CliSubcommandSchema",
        "pub subcommand: Option<CliSubcommandSchema>",
        "\"CW\"",
        "parse_wrapper_artifact_tokens",
        "cli_wrapper_schema_artifact_round_trips",
    ] {
        assert!(
            cli_schema.contains(required),
            "CLI schema missing wrapper schema support `{required}`"
        );
    }
    for required in ["subcommand: schema", "CliSubcommandSchema"] {
        assert!(
            mir.contains(required),
            "MIR lowering missing wrapper schema support `{required}`"
        );
    }
    for required in ["CliSchema::from_artifact_text", "schema.artifact_text()"] {
        assert!(
            implementation_artifact.contains(required),
            "implementation artifacts must persist wrapper CLI schemas through `{required}`"
        );
    }
    for required in [
        "cli_schema_is_wrapper",
        "cli_parse_wrapper",
        "cli_parse_global_options",
        "cli_parse_wrapper_request",
        "cli_help_for_wrapper_required",
        "Global Options:",
        "does not support wrapper record schemas",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing wrapper parse/help support `{required}`"
        );
    }
    for required in [
        "cli_wrapper_subcommand_field_metadata_plumbing_is_covered",
        "cli_wrapper_subcommand_field_metadata_rejects_invalid_contracts",
        "standard_cli_wrapper_parse_request_runs",
        "standard_cli_wrapper_parse_request_artifact_and_built_runs_use_schema_payload",
        "cli_new_creates_cli_tool_template",
        "manifest_cli_tool_project_sample_runs_with_required_options",
        "manifest_cli_tool_project_sample_reports_generated_usage",
        "manifest_cli_tool_project_sample_runs_against_emitted_artifacts",
        "manifest_cli_tool_project_sample_json_built_run_uses_strict_parse",
        "muga-package-interface-v11",
        "cli_subcommand",
        "may contain exactly one `@cli(subcommand)` field",
        "Usage: tool [global-options] <command> [args]",
        "\"--built\"",
    ] {
        assert!(
            examples.contains(required),
            "examples missing wrapper subcommand coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("CLI subcommand design", subcommand_design.as_str()),
        ("post-subcommand adoption", subcommand_adoption.as_str()),
        ("compact CLI design", compact_design.as_str()),
        ("post-compact adoption", compact_adoption.as_str()),
    ] {
        assert!(
            text.contains("cli-wrapper-root-options.md")
                && (text.contains("@cli(subcommand)") || text.contains("root/global option")),
            "{label} must surface wrapper-record root/global option design"
        );
    }

    assert!(
        implementation_resume
            .contains("CLI wrapper-record root/global options design is recorded in")
            && implementation_resume.contains("cli-wrapper-root-options.md")
            && implementation_resume
                .contains("| 276. CLI wrapper-record root/global options design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 277. CLI wrapper-record subcommand metadata plumbing |")
            && implementation_resume.contains(
                "| parser/formatter/typing/typed_hir/package_signature/interfaces/tests/docs | Done |"
            )
            && implementation_resume.contains("| 278. CLI wrapper-record schema and runtime support |")
            && implementation_resume.contains("| typing/cli_schema/mir/bytecode/runtime/artifacts/tests/docs | Done |")
            && implementation_resume.contains("| 279. CLI wrapper-record sample/template adoption |")
            && implementation_resume.contains("| samples/templates/tests/docs | Done |")
            && implementation_resume.contains("| 280. CLI schema-backed shell completion design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 281. CLI schema-backed shell completion implementation |")
            && implementation_resume.contains("| main/typing/cli_schema/runtime/artifacts/tests/docs | Done |")
            && implementation_resume.contains("| 282. CLI schema-backed shell completion adoption audit |")
            && implementation_resume.contains("| docs/tests/samples/templates | Done |")
            && implementation_resume.contains("| 283. CLI generated app shell completion onboarding |")
            && implementation_resume.contains("| templates/docs/tests | Done |")
            && implementation_resume.contains("| 284. CLI generated app completion packaging hook |")
            && implementation_resume.contains("| templates/tests/docs | Done |")
            && implementation_resume.contains(
                "Shell-agnostic JSON completion specs are implemented in"
            )
            && implementation_resume.contains("| 285. CLI completion JSON spec design |")
            && implementation_resume.contains("| 286. CLI completion JSON spec implementation |")
            && implementation_resume.contains("| 287. CLI completion nested command traversal |")
            && implementation_resume
                .contains("Next recommended slice: design installed-app resource layout and launcher boundary"),
        "implementation queue must mark shell completion packaging, JSON specs, and nested traversal done"
    );
}

#[test]
fn cli_schema_shell_completion_design_is_documented() {
    let design = read("docs/cli-schema-shell-completions.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let by_example = read("docs/muga-by-example.md");
    let shell_doc = read("docs/shell-completions-and-doctor.md");
    let wrapper_design = read("docs/cli-wrapper-root-options.md");
    let subcommand_design = read("docs/cli-subcommand-metadata.md");

    for required in [
        "Status: CLI schema-backed shell completion implementation landed for generated",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "muga cli-completions <bash|zsh|fish> --program <name> --type <Type>",
        "--package <package>",
        "--artifact-root <dir>|--built",
        "muga cli-completions fish --program cli-tool --type Root",
        "`--type` is required",
        "CliSchema",
        "Root",
        "Command",
        "@cli(subcommand)",
        "Implemented Slice",
        "source-backed checking",
        "explicit artifact roots through `--artifact-root <dir>`",
        "default built artifacts through `--built`",
        "command names plus command aliases",
        "Bool option value candidates",
        "Global Options:",
        "hidden",
        "`--help`",
        "`-h`",
        "true` / `false`",
        "enum-valued options",
        "fields whose types are unsupported by the completion schema are omitted",
        "compact short clusters",
        "muga shell-completions",
        "muga completions --format json",
        "Candidates Compared",
        "Select: separate command selected",
        "Generate only JSON completion specs first",
        "Non-Goals",
        "Done: implement `muga cli-completions <bash|zsh|fish> --program <name>",
        "Done: add shell-agnostic JSON completion specs",
        "Done: implement richer nested command traversal",
        "Done: add non-mutating completion package emission",
        "Next: evaluate TOML/config discovery",
    ] {
        assert!(
            design.contains(required),
            "CLI schema shell completion design missing `{required}`"
        );
    }

    assert!(
        shell_doc.contains("does not inspect the")
            && shell_doc.contains("current project, package manifest, source tree")
            && design.contains("separate from `muga shell-completions`"),
        "app completions must remain separate from static muga tool completions"
    );

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("wrapper design", wrapper_design.as_str()),
        ("subcommand design", subcommand_design.as_str()),
    ] {
        assert!(
            text.contains("cli-schema-shell-completions.md")
                && text.contains("muga cli-completions <bash|zsh|fish>")
                && text.contains("CliSchema"),
            "{label} must surface schema-backed generated shell completion design"
        );
    }

    assert!(
        implementation_resume.contains("CLI schema-backed shell completion design is recorded in")
            && implementation_resume.contains("cli-schema-shell-completions.md")
            && implementation_resume.contains("| 280. CLI schema-backed shell completion design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 281. CLI schema-backed shell completion implementation |")
            && implementation_resume
                .contains("| main/typing/cli_schema/runtime/artifacts/tests/docs | Done |")
            && implementation_resume
                .contains("| 282. CLI schema-backed shell completion adoption audit |")
            && implementation_resume.contains("| docs/tests/samples/templates | Done |")
            && implementation_resume
                .contains("| 283. CLI generated app shell completion onboarding |")
            && implementation_resume.contains("| templates/docs/tests | Done |")
            && implementation_resume
                .contains("| 284. CLI generated app completion packaging hook |")
            && implementation_resume.contains("| templates/tests/docs | Done |")
            && implementation_resume
                .contains("Shell-agnostic JSON completion specs are implemented in")
            && implementation_resume.contains("| 285. CLI completion JSON spec design |")
            && implementation_resume.contains("| 286. CLI completion JSON spec implementation |")
            && implementation_resume.contains("| 287. CLI completion nested command traversal |")
            && implementation_resume.contains(
                "Next recommended slice: design installed-app resource layout and launcher boundary"
            ),
        "implementation queue must mark shell completion packaging, JSON specs, and nested traversal done"
    );
}

#[test]
fn cli_schema_shell_completion_implementation_is_covered() {
    let main = read("src/main.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/cli-schema-shell-completions.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let diagnostics = read("docs/diagnostics-and-output.md");
    let shell_doc = read("docs/shell-completions-and-doctor.md");

    for required in [
        "Mode::CliCompletions",
        "\"cli-completions\" => Mode::CliCompletions",
        "cli_completion_script",
        "cli_completion_schema_for_check",
        "cli_completion_field_has_explicit_metadata",
        "bash_cli_completion_script",
        "zsh_cli_completion_script",
        "fish_cli_completion_script",
        "completion_shell",
        "completion_program",
        "CompletionCommandTransition",
        "push_cli_completion_command_scopes",
        "bash_command_transition_cases",
        "zsh_command_transition_cases",
        "fish_scope_condition",
        "--program",
        "SHELL_COMPLETION_COMMANDS",
        "SHELL_COMPLETION_OPTIONS",
        "cli-completions requires --program",
        "cli-completions requires --type",
    ] {
        assert!(
            main.contains(required),
            "main CLI completion implementation missing `{required}`"
        );
    }

    for required in [
        "cli_schema_shell_completions_report_generated_app_scripts",
        "cli_schema_completions_omit_unsupported_default_fields",
        "cli_schema_completions_traverse_nested_command_scopes",
        "cli_schema_shell_completions_use_artifacts_and_built_workflows",
        "cli_schema_shell_completions_validate_cli_arguments",
        "--artifact-root",
        "--built",
        "Audit Apply",
        "complete -F _cli_tool_completion cli-tool",
        "#compdef cli-tool",
        "complete -c cli-tool",
        "powershell",
    ] {
        assert!(
            examples.contains(required),
            "examples missing CLI completion implementation coverage `{required}`"
        );
    }

    for required in [
        "Implemented Slice",
        "source-backed checking",
        "explicit artifact roots through `--artifact-root <dir>`",
        "default built artifacts through `--built`",
        "command names plus command aliases",
        "Bool option value candidates",
        "shell renderers now traverse command scopes recursively",
        "fields whose types are unsupported by the completion schema are omitted",
        "Done: implement `muga cli-completions <bash|zsh|fish> --program <name>",
        "Done: add shell-agnostic JSON completion specs",
        "Done: implement richer nested command traversal",
        "Done: add non-mutating completion package emission",
        "Next: evaluate TOML/config discovery",
    ] {
        assert!(
            design.contains(required),
            "CLI completion implementation doc missing `{required}`"
        );
    }

    assert!(
        diagnostics.contains("muga cli-completions <bash|zsh|fish>")
            && diagnostics.contains("deterministic generated-app shell completion script")
            && diagnostics.contains("diagnostics")
            && shell_doc.contains("Generated Muga app completions are intentionally separate")
            && shell_doc.contains("cli-schema-shell-completions.md"),
        "command-output and static shell docs must describe generated app completions separately"
    );

    assert!(
        implementation_resume
            .contains("CLI schema-backed shell completion implementation is in place")
            && implementation_resume
                .contains("| 281. CLI schema-backed shell completion implementation |")
            && implementation_resume
                .contains("| main/typing/cli_schema/runtime/artifacts/tests/docs | Done |")
            && implementation_resume
                .contains("| 282. CLI schema-backed shell completion adoption audit |")
            && implementation_resume.contains("| docs/tests/samples/templates | Done |")
            && implementation_resume
                .contains("| 283. CLI generated app shell completion onboarding |")
            && implementation_resume.contains("| templates/docs/tests | Done |")
            && implementation_resume
                .contains("| 284. CLI generated app completion packaging hook |")
            && implementation_resume.contains("| templates/tests/docs | Done |"),
        "implementation queue must cover completed CLI completion packaging"
    );
}

#[test]
fn post_cli_schema_shell_completion_adoption_gap_selection_is_documented() {
    let audit = read("docs/post-cli-schema-shell-completion-adoption-gap-selection.md");
    let onboarding = read("docs/installation-and-onboarding.md");
    let project_template = read("src/project_template.rs");
    let examples = read("tests/examples.rs");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let practical = read("docs/practical-language-readiness.md");
    let by_example = read("docs/muga-by-example.md");
    let cli_completion_design = read("docs/cli-schema-shell-completions.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: CLI schema-backed shell completion adoption audit completed; install docs, generated cli-tool README, packaging hook, shell-agnostic JSON completion spec, nested traversal, value sources, and non-mutating completion package emission implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "muga cli-completions",
        "CliSchema",
        "installation-and-onboarding.md",
        "generated `cli-tool` README",
        "Candidates Compared",
        "Add install documentation plus a generated `cli-tool` README",
        "Select",
        "Generate completion files during `muga new --template cli-tool`",
        "shell-agnostic JSON completion specs",
        "Automatic shell-profile installation",
        "Reject for v1",
        "Selected Slice",
        "scripts/generate-completions.sh",
        "Done: add generated `cli-tool` README and onboarding docs",
        "Done: add a generated project packaging hook",
        "Done: add shell-agnostic JSON completion specs",
        "Done: implement richer nested command traversal",
        "Done: add static file/directory value-source metadata",
        "Done: add non-mutating completion package emission",
        "Next: evaluate TOML/config discovery",
    ] {
        assert!(
            audit.contains(required),
            "post CLI schema shell completion adoption audit missing `{required}`"
        );
    }

    for required in [
        "Generated Muga apps use a separate schema-backed completion command",
        "muga new --template cli-tool ~/tmp/muga-cli",
        "muga cli-completions fish --program cli-tool --type Root ~/tmp/muga-cli/src/main/main.muga",
        "muga emit-cli-completions --format json --output-dir ~/tmp/muga-cli/completions --program cli-tool --type Root ~/tmp/muga-cli/src/main/main.muga",
        "muga cli-completions zsh --program cli-tool --type Root --built ~/tmp/muga-cli/src/main/main.muga",
        "sh scripts/generate-completions.sh",
        "sh scripts/package-cli-tool.sh",
        "cli-schema-shell-completions.md",
    ] {
        assert!(
            onboarding.contains(required),
            "onboarding docs missing generated app completion evidence `{required}`"
        );
    }

    for required in [
        "relative: \"README.md\"",
        "Generated strict CLI tool starter.",
        "muga cli-completions bash --program cli-tool --type Root src/main/main.muga",
        "muga cli-completions zsh --program cli-tool --type Root --built src/main/main.muga",
        "muga cli-completions fish --program cli-tool --type Root src/main/main.muga",
        "muga emit-cli-completions --format json --output-dir completions --program cli-tool --type Root src/main/main.muga",
        "relative: \"scripts/generate-completions.sh\"",
        "sh scripts/generate-completions.sh",
        "relative: \"scripts/package-cli-tool.sh\"",
        "sh scripts/package-cli-tool.sh",
        "MUGA_INSTALL_DIR",
    ] {
        assert!(
            project_template.contains(required),
            "generated cli-tool template missing completion README evidence `{required}`"
        );
    }

    for required in [
        "generated_readme",
        "generated cli-tool README missing",
        "muga cli-completions bash --program cli-tool --type Root src/main/main.muga",
        "muga cli-completions zsh --program cli-tool --type Root --built src/main/main.muga",
        "generated_completion_script",
        "generated_package_script",
        "scripts/generate-completions.sh",
        "scripts/package-cli-tool.sh",
        "MUGA_INSTALL_DIR",
        "muga emit-cli-completions --format json --output-dir \\\"$out_dir\\\" --program cli-tool --type Root \\\"$entry\\\"",
        "emit-app-bundle --source-free --output-dir \\\"$bundle_dir\\\" --program \\\"$program\\\" \\\"$entry\\\"",
        "emit-app-completions --format json --output-dir \\\"$completions_dir\\\" --program \\\"$program\\\" --type Root \\\"$bundle_dir\\\"",
        "completions/cli-tool.bash",
        "completions/cli-tool.completions.json",
        "dist/completions/cli-tool.completions.json",
    ] {
        assert!(
            examples.contains(required),
            "examples missing generated cli-tool README coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("practical readiness", practical.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("CLI completion design", cli_completion_design.as_str()),
    ] {
        assert!(
            text.contains("post-cli-schema-shell-completion-adoption-gap-selection.md")
                && text.contains("generated `cli-tool` README"),
            "{label} must surface generated app completion onboarding adoption"
        );
    }

    assert!(
        implementation_resume
            .contains("CLI schema-backed shell completion adoption audit is recorded in")
            && implementation_resume
                .contains("post-cli-schema-shell-completion-adoption-gap-selection.md")
            && implementation_resume
                .contains("| 282. CLI schema-backed shell completion adoption audit |")
            && implementation_resume.contains("| docs/tests/samples/templates | Done |")
            && implementation_resume
                .contains("| 283. CLI generated app shell completion onboarding |")
            && implementation_resume.contains("| templates/docs/tests | Done |")
            && implementation_resume
                .contains("| 284. CLI generated app completion packaging hook |")
            && implementation_resume.contains("| templates/tests/docs | Done |")
            && implementation_resume
                .contains("Shell-agnostic JSON completion specs are implemented in")
            && implementation_resume.contains("| 285. CLI completion JSON spec design |")
            && implementation_resume.contains("| 286. CLI completion JSON spec implementation |")
            && implementation_resume.contains("| 287. CLI completion nested command traversal |")
            && implementation_resume.contains(
                "Next recommended slice: design installed-app resource layout and launcher boundary"
            ),
        "implementation queue must mark generated app completion packaging, JSON specs, and nested traversal done"
    );
}

#[test]
fn cli_completion_json_spec_implementation_is_covered() {
    let design = read("docs/cli-completion-json-spec.md");
    let main = read("src/main.rs");
    let examples = read("tests/examples.rs");
    let diagnostics = read("docs/diagnostics-and-output.md");
    let shell_design = read("docs/cli-schema-shell-completions.md");
    let shell_doc = read("docs/shell-completions-and-doctor.md");
    let onboarding = read("docs/installation-and-onboarding.md");
    let post_adoption = read("docs/post-cli-schema-shell-completion-adoption-gap-selection.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let docs_readme = read("docs/README.md");
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");

    for required in [
        "Status: shell-agnostic generated-app completion spec implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "muga cli-completions --format json --program cli-tool --type Root",
        "does not accept a shell",
        "\"schemaVersion\": 1",
        "\"command\": \"cli-completions\"",
        "`completion` is recursive",
        "`record`: a leaf option/positional schema",
        "`command`: an enum-backed command tree",
        "`wrapper`: a root record with options plus one `@cli(subcommand)` field",
        "`takesValue`",
        "`repeatable`",
        "`fallback: \"file\"`",
        "Unsupported record fields without explicit `@cli(...)` metadata are omitted",
        "Value schema `kind` values are `string`, `int`, `bool`, `option`, `list`, and",
        "Candidates Compared",
        "Add `muga cli-completions --format json`",
        "Extend editor `muga completions --format json`",
        "Implement richer nested traversal first",
        "Done: use this JSON contract to improve nested command traversal",
        "Done: add non-mutating completion package emission",
        "Next: evaluate TOML/config discovery",
    ] {
        assert!(
            design.contains(required),
            "CLI completion JSON spec missing `{required}`"
        );
    }

    for required in [
        "Mode::CliCompletions",
        "\"cli-completions\" => Mode::CliCompletions",
        "Mode::CliCompletions => match cli_completion_output(&cli)",
        "OutputFormat::Json => cli_completion_json(cli)",
        "cli_completion_json_output",
        "push_cli_completion_schema_json",
        "push_cli_completion_options_json",
        "push_cli_completion_positionals_json",
        "push_cli_completion_commands_json",
        "push_cli_completion_value_json",
        "cli_completion_field_has_explicit_metadata",
        "cli_completion_value_is_repeatable",
        "cli-completions --format json does not accept a shell",
        "muga cli-completions --format json --program <name> --type <type>",
    ] {
        assert!(
            main.contains(required),
            "main CLI completion JSON implementation missing `{required}`"
        );
    }

    for required in [
        "cli_schema_completion_json_reports_shell_agnostic_contract",
        "cli_schema_completions_omit_unsupported_default_fields",
        "cli_schema_completions_traverse_nested_command_scopes",
        ".arg(\"--format\")",
        ".arg(\"json\")",
        "completion",
        "wrapper",
        "subcommand",
        "command",
        "target",
        "fallback",
        "tags",
        "repeatable",
        "cli-completions --format json does not accept a shell",
    ] {
        assert!(
            examples.contains(required),
            "examples missing CLI completion JSON coverage `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("diagnostics", diagnostics.as_str()),
        ("shell design", shell_design.as_str()),
        ("shell/static doc", shell_doc.as_str()),
        ("onboarding", onboarding.as_str()),
        ("post completion adoption", post_adoption.as_str()),
    ] {
        assert!(
            text.contains("cli-completion-json-spec.md")
                || text.contains("muga cli-completions --format json")
                || text.contains("shell-agnostic JSON completion spec"),
            "{label} must surface the shell-agnostic CLI completion JSON contract"
        );
    }

    assert!(
        diagnostics.contains("`cli-completions` JSON field rules:")
            && diagnostics.contains("\"completion\":{\"kind\":\"wrapper\"")
            && shell_design.contains("Done: add shell-agnostic JSON completion specs")
            && post_adoption.contains("Done: add shell-agnostic JSON completion specs"),
        "command-output and completion docs must describe the JSON completion contract"
    );

    assert!(
        implementation_resume.contains("Shell-agnostic JSON completion specs are implemented in")
            && implementation_resume.contains("cli-completion-json-spec.md")
            && implementation_resume.contains("| 285. CLI completion JSON spec design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 286. CLI completion JSON spec implementation |")
            && implementation_resume.contains("| main/tests/docs | Done |")
            && implementation_resume.contains("| 287. CLI completion nested command traversal |")
            && implementation_resume.contains(
                "Next recommended slice: design installed-app resource layout and launcher boundary"
            ),
        "implementation queue must mark CLI completion JSON specs and nested traversal done"
    );
}

#[test]
fn cli_completion_value_source_metadata_is_covered() {
    let design = read("docs/cli-completion-value-sources.md");
    let json_design = read("docs/cli-completion-json-spec.md");
    let shell_design = read("docs/cli-schema-shell-completions.md");
    let parser = read("src/parser.rs");
    let typing = read("src/typing.rs");
    let cli_schema = read("src/cli_schema.rs");
    let interface = read("src/interface.rs");
    let main = read("src/main.rs");
    let examples = read("tests/examples.rs");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let roadmap = read("ROADMAP.md");
    let docs_readme = read("docs/README.md");

    for required in [
        "Status: static filesystem value-source metadata implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "@cli(value_source: \"file\"",
        "@cli(value_source: \"file\"|\"directory\")",
        "`value_source` accepts only `\"file\"` and `\"directory\"`",
        "`String`, `Option[String]`, or `List[String]`",
        "`valueSource`",
        "bash uses `compgen -f` or `compgen -d`",
        "zsh uses `_files` or `_files -/`",
        "fish keeps file values",
        "Candidates Compared",
        "Field-level `@cli(value_source: \"file\"|\"directory\")`",
        "Done: validate `@cli(value_source: \"file\"|\"directory\")`",
        "Done: add non-mutating completion package emission",
        "Next: evaluate TOML/config discovery",
    ] {
        assert!(
            design.contains(required),
            "CLI completion value source design missing `{required}`"
        );
    }

    for required in [
        "value_source",
        "CLI value sources support only `file` or `directory`",
        "CLI value source may be specified only once",
    ] {
        assert!(
            parser.contains(required),
            "parser missing CLI value source validation `{required}`"
        );
    }

    for required in [
        "CliValueSource",
        "cli_value_source_from_attributes",
        "cli_value_source_argument_from_attributes",
        "cli_value_source_allowed_for_schema",
        "requires a String value type",
        "value_source: field.cli_value_source",
    ] {
        assert!(
            typing.contains(required),
            "typing missing CLI value source plumbing `{required}`"
        );
    }

    for required in [
        "pub enum CliValueSource",
        "File",
        "Directory",
        "artifact_token",
        "from_artifact_token",
        "\"CV\"",
        "value_source: Option<CliValueSource>",
    ] {
        assert!(
            cli_schema.contains(required),
            "CliSchema missing value source artifact support `{required}`"
        );
    }

    for required in [
        "cli_value_source",
        "\"value_source\"",
        "CliValueSource::from_artifact_token",
    ] {
        assert!(
            interface.contains(required),
            "interface missing value source persistence `{required}`"
        );
    }

    for required in [
        "value_source: Option<muga::cli_schema::CliValueSource>",
        "push_cli_completion_value_source_json",
        "cli_completion_value_source_allowed_for_schema",
        "valueSource",
        "compgen_flag",
        "_files -/",
        "__fish_complete_directories",
    ] {
        assert!(
            main.contains(required),
            "main missing value source completion rendering `{required}`"
        );
    }

    for required in [
        "cli_schema_completions_report_value_sources",
        "cli_value_source_metadata_requires_string_values",
        "cli_value_source_metadata_rejects_invalid_attribute_values",
        "valueSource",
        "compgen -f",
        "compgen -d",
        "__fish_complete_directories",
        "--artifact-root",
        "requires a String value type",
    ] {
        assert!(
            examples.contains(required),
            "examples missing CLI value source coverage `{required}`"
        );
    }

    assert!(
        json_design.contains("`valueSource`: `\"file\"`, `\"directory\"`, or `null`")
            && shell_design.contains("static file/directory sources")
            && implementation_resume.contains("| 288. CLI completion value-source metadata |")
            && implementation_resume
                .contains("Static CLI completion value-source metadata is implemented")
            && roadmap.contains("static file/directory value-source data")
            && docs_readme.contains("cli-completion-value-sources.md"),
        "docs must surface CLI completion value source metadata"
    );
}

#[test]
fn cli_completion_installer_integration_is_covered() {
    let design = read("docs/cli-completion-installer-integration.md");
    let main = read("src/main.rs");
    let examples = read("tests/examples.rs");
    let project_template = read("src/project_template.rs");
    let diagnostics = read("docs/diagnostics-and-output.md");
    let onboarding = read("docs/installation-and-onboarding.md");
    let docs_readme = read("docs/README.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let shell_design = read("docs/cli-schema-shell-completions.md");
    let json_design = read("docs/cli-completion-json-spec.md");
    let value_sources = read("docs/cli-completion-value-sources.md");
    let post_adoption = read("docs/post-cli-schema-shell-completion-adoption-gap-selection.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "Status: non-mutating generated-app completion package emission implemented",
        "Short-Term Goal",
        "Medium-Term Goal",
        "Long-Term Goal",
        "Final Goal",
        "muga emit-cli-completions [--format text|json] --output-dir <dir> --program <name> --type <Type>",
        "muga emit-app-completions [--format text|json] --output-dir <dir> [--program <name>] --type <Type>",
        "`--artifact-root`, and `--built`",
        "<program>.bash",
        "_<program>",
        "<program>.fish",
        "<program>.completions.json",
        "written<TAB><output-dir>/<program>.bash",
        "never edits shell profiles",
        "Candidates Compared",
        "Add `muga emit-cli-completions --output-dir ...`",
        "Add `muga emit-app-completions --output-dir ...`",
        "Add machine-readable completion package emission",
        "Add shell-profile installation",
        "Defer",
        "Done: refresh the generated `cli-tool` packaging hook",
        "scripts/package-cli-tool.sh",
        "source-free bundle emission, bundle execution, app completion package emission",
        "Done: add `muga emit-app-completions [--format text|json] --output-dir <dir> [--program <name>]",
        "Next: evaluate TOML/config discovery",
    ] {
        assert!(
            design.contains(required),
            "CLI completion installer integration doc missing `{required}`"
        );
    }

    for required in [
        "Mode::EmitCliCompletions",
        "\"emit-cli-completions\" => Mode::EmitCliCompletions",
        "Mode::EmitAppCompletions",
        "\"emit-app-completions\" => Mode::EmitAppCompletions",
        "emit_cli_completion_package",
        "emit_app_completion_package",
        "completion_package_json_output",
        "completion_package_diagnostic_json_output",
        "cli_completion_package_file_stem",
        "read_app_bundle_interfaces",
        "--output-dir",
        "emit-cli-completions requires --output-dir",
        "emit-app-completions requires --output-dir",
        "format!(\"{file_stem}.bash\")",
        "format!(\"_{file_stem}\")",
        "format!(\"{file_stem}.fish\")",
        "format!(\"{file_stem}.completions.json\")",
        "written\\t{}",
    ] {
        assert!(
            main.contains(required),
            "main missing CLI completion installer integration evidence `{required}`"
        );
    }

    for required in [
        "emit_cli_completions_writes_shell_and_json_package",
        "cli_emit_app_completions_writes_package_from_source_free_bundle",
        ".arg(\"emit-cli-completions\")",
        ".arg(\"emit-app-completions\")",
        "\\\"command\\\":\\\"emit-cli-completions\\\"",
        "\\\"command\\\":\\\"emit-app-completions\\\"",
        "cli-tool.bash",
        "_cli-tool",
        "cli-tool.fish",
        "cli-tool.completions.json",
        "emit-cli-completions requires --output-dir",
    ] {
        assert!(
            examples.contains(required),
            "examples missing CLI completion package coverage `{required}`"
        );
    }

    for required in [
        "muga emit-cli-completions --format json --output-dir completions --program cli-tool --type Root src/main/main.muga",
        "muga emit-cli-completions --format json --output-dir \"$out_dir\" --program cli-tool --type Root \"$entry\"",
        "relative: \"scripts/package-cli-tool.sh\"",
        "\"$MUGA_BIN\" emit-app-completions --format json --output-dir \"$completions_dir\" --program \"$program\" --type Root \"$bundle_dir\"",
        "\"$MUGA_BIN\" list-installed-apps --output-dir \"$MUGA_INSTALL_DIR\"",
        ".completions.json",
    ] {
        assert!(
            project_template.contains(required),
            "generated cli-tool template missing completion package evidence `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("docs README", docs_readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("strategy", strategy.as_str()),
        ("practical", practical.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("diagnostics", diagnostics.as_str()),
        ("onboarding", onboarding.as_str()),
        ("shell design", shell_design.as_str()),
        ("JSON design", json_design.as_str()),
        ("value-source design", value_sources.as_str()),
        ("post adoption", post_adoption.as_str()),
    ] {
        assert!(
            text.contains("cli-completion-installer-integration.md")
                || text.contains("muga emit-cli-completions")
                || text.contains("emit-cli-completions"),
            "{label} must surface CLI completion installer integration"
        );
    }

    assert!(
        diagnostics.contains("muga emit-cli-completions [--format text|json] --output-dir <dir>")
            && onboarding
                .contains("muga emit-cli-completions --format json --output-dir ~/tmp/muga-cli/completions")
            && onboarding
                .contains("muga emit-app-completions --format json --output-dir ~/tmp/muga-cli/app-completions")
            && implementation_resume.contains("| 289. CLI completion installer integration |")
            && implementation_resume.contains("main/templates/tests/docs | Done")
            && implementation_resume.contains(
                "Next recommended slice: design installed-app resource layout and launcher boundary"
            ),
        "docs and implementation queue must cover completion package emission"
    );
}

#[test]
fn json_config_rename_metadata_implementation_is_covered() {
    let ast = read("src/ast.rs");
    let parser = read("src/parser.rs");
    let typing = read("src/typing.rs");
    let json_decode = read("src/json_decode.rs");
    let mir = read("src/mir.rs");
    let runtime = read("src/runtime.rs");
    let interface = read("src/interface.rs");
    let typed_hir = read("src/typed_hir.rs");
    let package_signature = read("src/package_signature.rs");
    let package_rewrite = read("src/package.rs");
    let examples = read("tests/examples.rs");
    let design = read("docs/json-config-schema-polish.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let mini_spec = read("mini-language-spec-v1.md");
    let std_config = read("docs/std-config-json-loading.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");

    for required in [
        "AttributeArgument",
        "arguments: Vec<AttributeArgument>",
        "attributes: Vec<Attribute>",
    ] {
        assert!(
            ast.contains(required),
            "AST missing JSON rename metadata `{required}`"
        );
    }

    for required in [
        "validate_json_attribute_arguments",
        "validate_record_field_attributes",
        "validate_enum_variant_attributes",
        "attribute `@json` is allowed only on record declarations, record fields, and enum variants",
        "JSON rename and alias values must be non-empty",
    ] {
        assert!(
            parser.contains(required),
            "parser missing JSON rename attribute handling `{required}`"
        );
    }

    for required in [
        "json_rename: Option<Symbol>",
        "duplicate JSON field wire name",
        "duplicate JSON enum variant wire name",
        "wire_name: field.json_rename",
        "wire_name: variant.json_rename",
    ] {
        assert!(
            typing.contains(required),
            "typing missing JSON rename metadata handling `{required}`"
        );
    }

    for required in [
        "wire_name: Option<Symbol>",
        "\"RA\"",
        "\"EA\"",
        "record field wire symbol",
        "enum variant wire symbol",
        "invalid decoder field wire symbol",
    ] {
        assert!(
            json_decode.contains(required),
            "JSON decoder artifact schema missing rename support `{required}`"
        );
    }

    assert!(
        mir.contains("wire_name") && mir.contains("source_symbol(wire_name)"),
        "MIR lowering must preserve JSON rename wire symbols"
    );

    for required in [
        "json_decode_wire_name",
        "json_object_field_for_decode",
        "json_decode_variant_by_name",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing JSON rename decoding behavior `{required}`"
        );
    }

    for required in [
        "json_rename: Option<String>",
        "json_aliases: Vec<String>",
        "field.len() < 4",
        "variant.len() < 4",
        "json_rename: field.json_rename.clone()",
        "json_rename: variant.json_rename.clone()",
    ] {
        assert!(
            interface.contains(required),
            "package interface missing JSON rename persistence `{required}`"
        );
    }

    assert!(
        typed_hir.contains("json_rename_from_attributes")
            && package_signature.contains("json_rename_from_attributes")
            && package_rewrite.contains("attributes: field.attributes.clone()")
            && package_rewrite.contains("attributes: variant.attributes.clone()"),
        "typed HIR, package signatures, and package rewrite must preserve JSON rename metadata"
    );

    for required in [
        "standard_json_decode_json_rename_metadata_runs",
        "standard_json_decode_json_rename_rejects_duplicate_wire_names",
        "package_interfaces_preserve_json_rename_metadata_without_source",
        "expected JSON Int at path .next_action.scale",
        "unknown JSON enum variant `Auto` at path .run_mode",
        "run_path_against_artifact_root",
    ] {
        assert!(
            examples.contains(required),
            "examples coverage missing JSON rename case `{required}`"
        );
    }

    for (label, text) in [
        ("design", design.as_str()),
        ("mini spec", mini_spec.as_str()),
        ("std config", std_config.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
    ] {
        assert!(
            text.contains("@json(rename")
                && text.contains("record fields and enum variants")
                && text.contains("TOML"),
            "{label} must document implemented JSON rename metadata"
        );
    }

    assert!(
        implementation_resume
            .contains("| 214. JSON/config field and variant rename implementation |")
            && implementation_resume.contains(
                "| parser/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 215. post-rename JSON/config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 216. JSON/config strict unknown-field policy design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 217. JSON/config strict unknown-field policy implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            )
            && implementation_resume
                .contains("| 218. post-strict JSON/config adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 219. JSON/config alias metadata design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 220. JSON/config alias metadata implementation |")
            && implementation_resume.contains(
                "| parser/formatter/typing/interfaces/json_decode/runtime/artifacts/tests/docs | Done |"
            ),
        "implementation queue must mark JSON rename implementation done and queue alias design"
    );
}

#[test]
fn std_json_scalar_array_projection_helpers_are_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let config_app = read("samples/projects/config_app/src/main/main.muga");
    let std_json_sample = read("samples/packages/app/std_json/main.muga");
    let contract = read("docs/std-json-first-slice.md");
    let audit = read("docs/std-json-implementation-audit.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let spec = read("spec/003-typing.md");
    let mini_spec = read("mini-language-spec-v1.md");

    for required in [
        "fn array_item_shape_error(index: Int, expected: String): Error",
        "pub fn array_strings(values: List[Value]): Result[List[String], Error]",
        "pub fn array_ints(values: List[Value]): Result[List[Int], Error]",
        "pub fn array_bools(values: List[Value]): Result[List[Bool], Error]",
        "expected JSON \".concat(expected).concat(\" for array item at index \")",
        "converted: Result[String, Error] = match value",
        "converted: Result[Int, Error] = match value",
        "converted: Result[Bool, Error] = match value",
        "Result::Err(array_item_shape_error(index, \"Int\"))",
    ] {
        assert!(
            std_package.contains(required),
            "std::json implementation missing scalar array projection evidence `{required}`"
        );
    }

    for required in [
        "standard_json_scalar_array_projections_run_as_virtual_package",
        "json::array_strings(names_values)",
        "json::array_ints(ports_values)",
        "json::array_bools(flags_values)",
        "expected JSON String for array item at index 1",
        "expected JSON Int for array item at index 0",
        "expected JSON Bool for array item at index 0",
        "Result::Ok(Ada|true|3|2|2|1|core|2)",
        "Result::Ok(default|Ada|42|true|2|2|0|2|0|",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing scalar array projection coverage `{required}`"
        );
    }

    for required in [
        "tags: List[String]",
        "config::load_json_or(config_path, default_settings())",
        "settings.tags.len().to_string()",
    ] {
        assert!(
            config_app.contains(required),
            "config app sample missing scalar list config usage `{required}`"
        );
    }

    for required in [
        "json::object_int_array_required(parsed, \"items\")",
        "json::object_string_array_required(parsed, \"tags\")",
    ] {
        assert!(
            std_json_sample.contains(required),
            "std_json sample missing scalar array field helper usage `{required}`"
        );
    }

    for (label, text) in [
        ("contract", contract.as_str()),
        ("audit", audit.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("typing spec", spec.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("std::json")
                && (text.contains("scalar array projection") || text.contains("array_strings"))
                && text.contains("schema decoding"),
            "{label} must document scalar array projection helpers and deferred schema decoding"
        );
    }

    assert!(
        implementation_resume.contains("| 183. JSON scalar array projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 184. post-json-array-projection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 185. direct JSON scalar-array object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |"),
        "implementation queue must mark scalar array projections done and queue the post-json-array-projection selection"
    );
}

#[test]
fn std_json_scalar_array_object_field_helpers_are_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");
    let config_app = read("samples/projects/config_app/src/main/main.muga");
    let std_json_sample = read("samples/packages/app/std_json/main.muga");
    let contract = read("docs/std-json-first-slice.md");
    let audit = read("docs/std-json-implementation-audit.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let stdlib_review = read("docs/stdlib-package-samples-review.md");
    let by_example = read("docs/muga-by-example.md");
    let spec = read("spec/003-typing.md");
    let mini_spec = read("mini-language-spec-v1.md");

    for required in [
        "fn array_field_item_shape_error(key: String, index: Int, expected: String): Error",
        "fn field_array_strings(key: String, values: List[Value]): Result[List[String], Error]",
        "fn field_array_ints(key: String, values: List[Value]): Result[List[Int], Error]",
        "fn field_array_bools(key: String, values: List[Value]): Result[List[Bool], Error]",
        "pub fn object_string_array(value: Value, key: String): Result[Option[List[String]], Error]",
        "pub fn object_string_array_or(value: Value, key: String, default_value: List[String]): Result[List[String], Error]",
        "pub fn object_string_array_required(value: Value, key: String): Result[List[String], Error]",
        "pub fn object_int_array(value: Value, key: String): Result[Option[List[Int]], Error]",
        "pub fn object_int_array_or(value: Value, key: String, default_value: List[Int]): Result[List[Int], Error]",
        "pub fn object_int_array_required(value: Value, key: String): Result[List[Int], Error]",
        "pub fn object_bool_array(value: Value, key: String): Result[Option[List[Bool]], Error]",
        "pub fn object_bool_array_or(value: Value, key: String, default_value: List[Bool]): Result[List[Bool], Error]",
        "pub fn object_bool_array_required(value: Value, key: String): Result[List[Bool], Error]",
        "expected JSON \".concat(expected).concat(\" for object field `\")",
        "field_array_strings(key, values)",
        "field_array_ints(key, values)",
        "field_array_bools(key, values)",
    ] {
        assert!(
            std_package.contains(required),
            "std::json implementation missing scalar array field helper evidence `{required}`"
        );
    }

    for required in [
        "standard_json_scalar_array_field_helpers_run_as_virtual_package",
        "json::object_string_array_required(parsed, \"tags\")",
        "json::object_int_array_required(parsed, \"ports\")",
        "json::object_bool_array_required(parsed, \"flags\")",
        "json::object_string_array_or(parsed, \"missing_tags\", fallback_tags)",
        "json::object_bool_array(parsed, \"missing_flags\")",
        "expected JSON String for object field `bad_tags` array item at index 1",
        "expected JSON Int for object field `bad_ports` array item at index 0",
        "expected JSON Bool for object field `bad_flags` array item at index 0",
        "expected JSON Array for object field `shape`",
        "Result::Ok(Ada|true|3|2|2|1|core|2)",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing scalar array field helper coverage `{required}`"
        );
    }

    for required in [
        "tags: List[String]",
        "config::load_json_or(config_path, default_settings())",
    ] {
        assert!(
            config_app.contains(required),
            "config app sample missing scalar array field helper usage `{required}`"
        );
    }

    for required in [
        "json::object_int_array_required(parsed, \"items\")",
        "json::object_string_array_required(parsed, \"tags\")",
    ] {
        assert!(
            std_json_sample.contains(required),
            "std_json sample missing scalar array field helper usage `{required}`"
        );
    }

    for (label, text) in [
        ("contract", contract.as_str()),
        ("audit", audit.as_str()),
        ("implementation resume", implementation_resume.as_str()),
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("stdlib review", stdlib_review.as_str()),
        ("Muga by Example", by_example.as_str()),
        ("typing spec", spec.as_str()),
        ("mini spec", mini_spec.as_str()),
    ] {
        assert!(
            text.contains("std::json")
                && (text.contains("scalar-array object-field")
                    || text.contains("object_string_array"))
                && text.contains("schema decoding"),
            "{label} must document scalar array object-field helpers and deferred schema decoding"
        );
    }

    assert!(
        implementation_resume.contains("| 185. direct JSON scalar-array object-field helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 186. post-direct-json-array-field adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 187. repeated CLI option value helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 188. post-repeated-cli-option adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 189. JSON path helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume.contains("| 190. post-json-path adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 191. typed JSON path scalar projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 192. post-typed-json-path-scalar adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 193. typed JSON path collection projection helpers |")
            && implementation_resume.contains("| std package/tests/docs/samples | Done |")
            && implementation_resume
                .contains("| 194. post-typed-json-path-collection adoption gap selection |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume.contains("| 195. JSON schema decoding design |")
            && implementation_resume.contains("| docs/tests | Done |")
            && implementation_resume
                .contains("| 196. default-overlay JSON schema decoder implementation |")
            && implementation_resume
                .contains("| typing/MIR/bytecode/runtime/std_package/tests/docs/samples | Done |"),
        "implementation queue must mark typed JSON path collection helpers done and queue the next selection"
    );
}

#[test]
fn std_json_first_slice_is_implemented_and_covered() {
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let runtime = read("src/runtime.rs");
    let typing = read("src/typing.rs");
    let package = read("src/package.rs");
    let package_signature = read("src/package_signature.rs");
    let examples = read("tests/examples.rs");
    let review = read("docs/stdlib-package-samples-review.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");

    for required in [
        "JSON_PACKAGE",
        "JSON_VALUE_MANGLED_NAME",
        "JSON_NUMBER_MANGLED_NAME",
        "JSON_ERROR_KIND_MANGLED_NAME",
        "JSON_ERROR_MANGLED_NAME",
        "package std::json",
        "pub enum Value",
        "pub enum Number",
        "pub enum ErrorKind",
        "pub record Error",
        "pub fn parse(text: String): Result[Value, Error]",
        "pub fn encode(value: Value): Result[String, Error]",
        "pub fn number_as_int(number: Number): Result[Int, Error]",
        "pub fn int(value: Int): Value",
        "JSON_PARSE_BUILTIN",
        "JSON_ENCODE_BUILTIN",
        "JSON_NUMBER_AS_INT_BUILTIN",
    ] {
        assert!(
            std_package.contains(required),
            "std package missing first std::json implementation `{required}`"
        );
    }

    for required in ["StdJsonParse", "StdJsonEncode", "StdJsonNumberAsInt"] {
        assert!(
            prelude.contains(required),
            "prelude missing std::json builtin `{required}`"
        );
    }

    for required in [
        "JSON_NESTING_LIMIT",
        "struct JsonParser",
        "fn encode_json_value",
        "fn json_number_as_int",
        "DuplicateKey",
        "NumberOutOfRange",
        "NestingLimitExceeded",
        "sort_by",
    ] {
        assert!(
            runtime.contains(required),
            "runtime missing std::json implementation evidence `{required}`"
        );
    }

    for required in [
        "check_std_json_parse_builtin",
        "check_std_json_single_value_builtin",
        "std_json_expected_return",
    ] {
        assert!(
            typing.contains(required),
            "typing missing std::json builtin checker `{required}`"
        );
    }

    for (label, text) in [
        ("package diagnostics", package.as_str()),
        ("package signature diagnostics", package_signature.as_str()),
    ] {
        assert!(
            text.contains("import std::json"),
            "{label} must suggest importing std::json"
        );
    }

    for required in [
        "package_std_json_sample_runs",
        "standard_json_parse_objects_arrays_and_scalars",
        "standard_json_encode_sorts_object_keys",
        "standard_json_encode_escapes_strings",
        "standard_json_parse_reports_data_error_kinds",
        "standard_json_parse_exposes_error_offset",
        "standard_json_number_as_int_validates_raw_numbers",
        "standard_json_encode_rejects_invalid_raw_number",
        "standard_json_parse_reports_nesting_limit",
        "standard_json_encode_reports_nesting_limit",
        "standard_json_annotation_without_import_suggests_import",
        "standard_json_parse_type_mismatch_reports_expected_string",
        "standard_json_artifact_run_uses_emitted_std_implementations",
    ] {
        assert!(
            examples.contains(required),
            "examples suite missing std::json coverage `{required}`"
        );
    }

    assert!(
        Path::new("samples/packages/app/std_json/main.muga").is_file(),
        "std::json package sample should exist"
    );
    assert!(
        review.contains("std::json")
            && review.contains("samples/packages/app/std_json/main.muga")
            && review.contains("standard_json_artifact_run_uses_emitted_std_implementations"),
        "stdlib samples review must cover std::json"
    );
    assert!(
        implementation_resume.contains("| 142. first std::json implementation |")
            && implementation_resume.contains("| std package/runtime/tests/docs | Done |"),
        "implementation queue must mark std::json implementation done"
    );
}

#[test]
fn muga_definition_scope_is_documented() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let contract = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");

    for required in [
        "muga definition --format json --line 4 --column 8 path/to/package/main.muga",
        "go-to-definition",
        "package/module/item ids",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "`muga definition --format json --line <line> --column <column> <entry>`",
        "go-to-definition",
        "package/interface item references",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "`muga definition --format json --line <line> --column <column> <entry>`",
        "\"command\": \"definition\"",
        "\"definition\"",
        "\"bindingKind\"",
        "\"selectionSpan\"",
    ] {
        assert!(
            contract.contains(required),
            "command-output contract missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 release checklist", checklist.as_str()),
    ] {
        assert!(
            text.contains("muga definition"),
            "{label} missing `muga definition`"
        );
        assert!(
            text.contains("LSP") || text.contains("editor"),
            "{label} must tie definition to editor tooling"
        );
    }

    assert!(
        examples.contains("cli_definition_json_reports_local_and_package_targets_for_editor_tools"),
        "examples test suite must cover `muga definition`"
    );
    for required in [
        "Mode::Definition",
        "definition_json_output",
        "definition requires --line",
        "push_definition_target_json",
    ] {
        assert!(
            cli.contains(required),
            "CLI missing `muga definition` support `{required}`"
        );
    }
}

#[test]
fn muga_references_scope_is_documented() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let contract = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");

    for required in [
        "muga references --format json --line 4 --column 8 path/to/package/main.muga",
        "find references",
        "entry module",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "`muga references --format json --line <line> --column <column> <entry>`",
        "find references",
        "entry module",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "`muga references --format json --line <line> --column <column> <entry>`",
        "\"command\": \"references\"",
        "\"target\"",
        "\"references\"",
        "\"kind\": \"reference\"",
    ] {
        assert!(
            contract.contains(required),
            "command-output contract missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 release checklist", checklist.as_str()),
    ] {
        assert!(
            text.contains("muga references"),
            "{label} missing `muga references`"
        );
        assert!(
            text.contains("LSP") || text.contains("editor"),
            "{label} must tie references to editor tooling"
        );
    }

    assert!(
        examples.contains("cli_references_json_reports_entry_module_references_for_editor_tools"),
        "examples test suite must cover `muga references`"
    );
    for required in [
        "Mode::References",
        "references_json_output",
        "references requires --line",
        "push_reference_location_json",
    ] {
        assert!(
            cli.contains(required),
            "CLI missing `muga references` support `{required}`"
        );
    }
}

#[test]
fn json_backed_editor_workflow_is_documented_and_covered() {
    let readme = read_primary_docs();
    let workflow = read("docs/editor-json-workflow.md");
    let contract = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let roadmap = read("ROADMAP.md");
    let examples = read("tests/examples.rs");

    assert!(
        Path::new("docs/editor-json-workflow.md").is_file(),
        "editor JSON workflow document should exist"
    );

    for required in [
        "muga syntax --format json",
        "muga check --format json",
        "muga workspace --format json",
        "muga metadata --format json",
        "muga hover --format json",
        "muga completions --format json",
        "muga definition --format json",
        "muga references --format json",
        "muga run --format json",
        "muga test --format json",
        "without scraping human output",
        "json_backed_editor_workflow_uses_existing_command_contracts",
    ] {
        assert!(
            workflow.contains(required),
            "editor JSON workflow doc missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("diagnostics contract", contract.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("v1 release checklist", checklist.as_str()),
    ] {
        assert!(
            text.contains("editor-json-workflow.md"),
            "{label} must link to the editor JSON workflow"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("strategy plan", strategy.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("json_backed_editor_workflow_uses_existing_command_contracts")
                || text.contains("concrete JSON-backed editor workflow")
                || text.contains("concrete editor adapter flow"),
            "{label} must track the concrete editor workflow smoke coverage"
        );
    }

    assert!(
        decisions.contains("[x] Broaden the JSON-backed LSP/editor prototype"),
        "modern gap decisions must mark the concrete editor workflow complete"
    );
    assert!(
        examples.contains("json_backed_editor_workflow_uses_existing_command_contracts"),
        "examples test suite must cover the concrete editor JSON workflow"
    );
}

#[test]
fn artifact_cache_explanation_design_is_documented() {
    let readme = read_primary_docs();
    let design = read("docs/artifact-cache-explanations.md");
    let contract = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let roadmap = read("ROADMAP.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");

    assert!(
        Path::new("docs/artifact-cache-explanations.md").is_file(),
        "artifact/cache explanation design document should exist"
    );

    for required in [
        "muga why-rebuild",
        "--format text|json",
        "--format json",
        "artifactRoot",
        "lockfile",
        "archiveCache",
        "metadataHash",
        "artifactFile",
        "artifactHash",
        "regenerationCommand",
        "dependency-interface set changes",
        "missing",
        "fresh",
        "stale",
        "hashMismatch",
        "invalid",
        "non-mutating",
        "does not read dependency implementation source",
        "state<TAB>kind<TAB>package",
    ] {
        assert!(
            design.contains(required),
            "artifact/cache explanation design missing `{required}`"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("diagnostics contract", contract.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 release checklist", checklist.as_str()),
        ("roadmap", roadmap.as_str()),
    ] {
        assert!(
            text.contains("artifact-cache-explanations.md"),
            "{label} must link to the artifact/cache explanation design"
        );
    }

    assert!(
        decisions.contains("[x] Add artifact/cache explanation design"),
        "modern gap decisions must mark artifact/cache explanation design complete"
    );
    assert!(
        implementation_resume.contains("Initial read-only `muga why-rebuild --format json`")
            && implementation_resume
                .contains("`muga why-rebuild` now has compact human text output")
            && strategy.contains("`muga why-rebuild` emits compact human text output"),
        "planning docs must track the implemented why-rebuild text and JSON slices"
    );
    for required in [
        "cli_why_rebuild_json_reports_fresh_artifact_states",
        "cli_why_rebuild_json_reports_missing_explicit_artifacts",
        "cli_why_rebuild_json_reports_stale_source_artifacts",
        "cli_why_rebuild_json_reports_stale_dependency_interface_artifacts",
        "cli_why_rebuild_json_reports_dependency_interface_set_changed_implementation_artifact",
        "cli_why_rebuild_json_reports_stale_local_path_lockfile_metadata",
        "cli_why_rebuild_json_reports_fresh_local_archive_lockfile_metadata",
        "cli_why_rebuild_json_reports_invalid_and_hash_mismatched_artifacts",
        "cli_why_rebuild_text_reports_fresh_artifact_states",
        "cli_why_rebuild_text_reports_missing_explicit_artifacts",
        "cli_why_rebuild_text_reports_lockfile_and_archive_cache_metadata",
    ] {
        assert!(
            examples.contains(required),
            "examples test suite missing `{required}`"
        );
    }
    for required in [
        "Mode::WhyRebuild",
        "why_rebuild_text_output",
        "why_rebuild_json_output",
        "muga::explain_package_artifact_cache",
        "muga why-rebuild [--format text|json]",
    ] {
        assert!(cli.contains(required), "CLI missing `{required}`");
    }
}

#[test]
fn core_capability_acceleration_priority_is_documented() {
    let roadmap = read("ROADMAP.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let guide = read("docs/README.md");

    for (label, text) in [
        ("ROADMAP", roadmap.as_str()),
        ("strategy plan", strategy.as_str()),
        ("practical readiness", practical.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("documentation guide", guide.as_str()),
    ] {
        assert!(
            text.contains("Core Capability Acceleration"),
            "{label} must make the new priority explicit for context-free resumes"
        );
    }

    for required in [
        "`std::process` spine",
        "Structured task spine",
        "Service IO spine",
        "Performance spine",
        "Distribution spine",
    ] {
        assert!(
            implementation_resume.contains(required)
                || roadmap.contains(required)
                || strategy.contains(required),
            "core acceleration docs missing `{required}`"
        );
    }

    assert!(
        roadmap.contains("The v1 surface is feature-frozen.")
            && roadmap.contains("That still protects the small source model"),
        "ROADMAP must preserve the small v1 model while changing implementation priority"
    );
    assert!(
        guide.contains("References are not retention proof")
            && guide.contains("Rust tests, Muga samples, and executable CLI contracts")
            && guide.contains("Delete historical files"),
        "documentation guide must record the docs cleanup policy"
    );
}

#[test]
fn muga_fmt_scope_is_documented() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let contract = read("docs/diagnostics-and-output.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let examples = read("tests/examples.rs");
    let cli = read("src/main.rs");

    for required in ["muga fmt --check", "line comments", "preserving"] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in ["muga fmt [--check]", "line comments"] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "muga fmt <entry>",
        "muga fmt --check <entry>",
        "line comments",
    ] {
        assert!(
            contract.contains(required),
            "command-output contract missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
    ] {
        assert!(
            text.contains("muga fmt") && text.contains("--check") && text.contains("comment"),
            "{label} must track the implemented muga fmt scope"
        );
    }

    for required in [
        "muga_fmt_formats_source_deterministically",
        "muga_fmt_formats_package_items_and_attributes",
        "muga_fmt_preserves_line_comments",
        "muga_fmt_allows_comment_markers_inside_strings",
        "cli_fmt_check_reports_unformatted_source_without_writing",
        "cli_fmt_writes_formatted_source",
        "cli_fmt_writes_comment_preserving_source",
        "cli_fmt_preserves_manifest_inferred_package_shape",
    ] {
        assert!(
            examples.contains(required),
            "examples test suite missing `{required}`"
        );
    }

    for required in [
        "Mode::Fmt",
        "muga::format_path",
        "muga::check_format_path",
        "--check",
    ] {
        assert!(cli.contains(required), "CLI missing `{required}`");
    }
}

#[test]
fn option_result_helpers_are_documented_and_covered() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let typing_spec = read("spec/003-typing.md");
    let option_spec = read("spec/008-collections.md");
    let result_spec = read("spec/013-enums-results.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let std_package = read("src/std_package.rs");
    let examples = read("tests/examples.rs");

    for required in [
        "std::option",
        "std::result",
        "option::map",
        "option::and_then",
        "option::value_or",
        "result::map",
        "result::map_err",
        "result::and_then",
        "result::value_or",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "std::option",
        "std::result",
        "value transformation without propagation syntax",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "pub fn is_some[T](value: Option[T]): Bool",
        "pub fn map[T, U](value: Option[T], f: T -> U): Option[U]",
        "pub fn map_err[T, E, F](value: Result[T, E], f: E -> F): Result[T, F]",
    ] {
        assert!(
            typing_spec.contains(required),
            "typing spec missing `{required}`"
        );
    }

    for required in ["option::is_some(option)", "option::map(option, f)"] {
        assert!(
            option_spec.contains(required),
            "collection spec missing `{required}`"
        );
    }
    for required in ["result::is_ok(result)", "result::map_err(result, f)"] {
        assert!(
            result_spec.contains(required),
            "result spec missing `{required}`"
        );
    }

    for required in [
        "OPTION_PACKAGE",
        "RESULT_PACKAGE",
        "pub fn value_or[T](value: Option[T], fallback: T): T",
        "pub fn value_or[T, E](value: Result[T, E], fallback: T): T",
    ] {
        assert!(
            std_package.contains(required),
            "std package source missing `{required}`"
        );
    }

    for required in [
        "package_std_option_sample_runs",
        "package_std_result_sample_runs",
        "standard_option_artifact_run_uses_emitted_std_implementations",
        "standard_result_artifact_run_uses_emitted_std_implementations",
        "standard_option_result_helpers_type_check_callbacks",
    ] {
        assert!(
            examples.contains(required),
            "examples test suite missing `{required}`"
        );
    }

    for sample in [
        "samples/packages/app/std_option/main.muga",
        "samples/packages/app/std_result/main.muga",
    ] {
        assert!(Path::new(sample).is_file(), "missing sample `{sample}`");
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
    ] {
        assert!(
            text.contains("std::option")
                && text.contains("std::result")
                && text.contains("std::list")
                && text.contains("std::map"),
            "{label} must track completed Option/Result helpers and collection helpers"
        );
    }
}

#[test]
fn list_map_helpers_are_documented_and_covered() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let typing_spec = read("spec/003-typing.md");
    let collection_spec = read("spec/008-collections.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let std_package = read("src/std_package.rs");
    let prelude = read("src/prelude.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");

    for required in [
        "std::list",
        "std::map",
        "list::map",
        "list::filter",
        "list::fold",
        "list::any",
        "list::all",
        "map::keys",
        "map::values",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "`std::list` helpers",
        "`std::map` helpers",
        "narrow collection transformations and key/value extraction",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "pub fn map[T, U](items: List[T], f: T -> U): List[U]",
        "pub fn filter[T](items: List[T], predicate: T -> Bool): List[T]",
        "pub fn fold[T, U](items: List[T], initial: U, f: (U, T) -> U): U",
        "pub fn keys[K, V](items: Map[K, V]): List[K]",
        "pub fn values[K, V](items: Map[K, V]): List[V]",
    ] {
        assert!(
            typing_spec.contains(required),
            "typing spec missing `{required}`"
        );
    }

    for required in [
        "list::map(items, f)",
        "list::fold(items, initial, f)",
        "map::keys(self: Map[K, V]): List[K]",
        "map::values(self: Map[K, V]): List[V]",
        "`List.contains` remains deferred",
        "map::entries` remains deferred",
    ] {
        assert!(
            collection_spec.contains(required),
            "collection spec missing `{required}`"
        );
    }

    for required in [
        "LIST_PACKAGE",
        "MAP_PACKAGE",
        "MAP_KEYS_BUILTIN",
        "MAP_VALUES_BUILTIN",
        "pub fn all[T](items: List[T], predicate: T -> Bool): Bool",
        "pub fn keys[K, V](items: Map[K, V]): List[K]",
    ] {
        assert!(
            std_package.contains(required),
            "std package source missing `{required}`"
        );
    }

    for required in ["StdMapKeys", "StdMapValues"] {
        assert!(prelude.contains(required), "prelude missing `{required}`");
        assert!(runtime.contains(required), "runtime missing `{required}`");
    }

    for required in [
        "package_std_list_sample_runs",
        "package_std_map_sample_runs",
        "standard_list_artifact_run_uses_emitted_std_implementations",
        "standard_map_artifact_run_uses_emitted_std_implementations",
        "standard_list_helpers_type_check_callbacks",
    ] {
        assert!(
            examples.contains(required),
            "examples test suite missing `{required}`"
        );
    }

    for sample in [
        "samples/packages/app/std_list/main.muga",
        "samples/packages/app/std_map/main.muga",
    ] {
        assert!(Path::new(sample).is_file(), "missing sample `{sample}`");
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 checklist", checklist.as_str()),
    ] {
        assert!(
            text.contains("std::list") && text.contains("std::map") && text.contains("equality"),
            "{label} must track completed collection helpers and the scalar-only equality boundary"
        );
    }
}

#[test]
fn equality_policy_is_documented_and_covered() {
    let readme = read_primary_docs();
    let mini_spec = read("mini-language-spec-v1.md");
    let typing_spec = read("spec/003-typing.md");
    let value_spec = read("spec/011-value-semantics.md");
    let enum_spec = read("spec/013-enums-results.md");
    let function_spec = read("spec/004-functions.md");
    let collection_spec = read("spec/008-collections.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let practical = read("docs/practical-language-readiness.md");
    let decisions = read("docs/modern-language-gap-decisions-2026-05-22.md");
    let strategy = read("docs/strategy-and-implementation-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let std_rules = read("docs/standard-library-review-rules.md");
    let typing = read("src/typing.rs");
    let runtime = read("src/runtime.rs");
    let examples = read("tests/examples.rs");

    for required in [
        "scalar-only",
        "`Int`, `Bool`, and `String`",
        "structural equality remains deferred",
    ] {
        assert!(readme.contains(required), "README missing `{required}`");
    }

    for required in [
        "scalar-only `==` / `!=` equality for `Int`, `Bool`, and `String`",
        "structural equality",
    ] {
        assert!(
            mini_spec.contains(required),
            "mini spec missing `{required}`"
        );
    }

    for required in [
        "The v1 equality policy is intentionally scalar-only",
        "Structural equality is not part of v1",
        "std::test` package follows the same policy",
        "They require an explicit spec update",
    ] {
        assert!(
            typing_spec.contains(required),
            "typing spec missing `{required}`"
        );
    }

    for required in [
        "v1 equality is value equality only for `Int`, `Bool`, and `String`",
        "unsupported values must be rejected statically",
        "aggregate equality expands beyond the v1 scalar-only policy",
    ] {
        assert!(
            value_spec.contains(required),
            "value semantics spec missing `{required}`"
        );
    }

    for required in [
        "`Option[T]`, `Result[T, E]`, and user-defined enums do not support `==` or `!=` in v1",
        "There is no generic structural `assert_eq` in v1",
        "v1 equality policy is scalar-only",
    ] {
        assert!(
            enum_spec.contains(required)
                || function_spec.contains(required)
                || collection_spec.contains(required),
            "split specs missing `{required}`"
        );
    }

    for (label, text) in [
        ("implementation resume plan", implementation_resume.as_str()),
        ("practical readiness", practical.as_str()),
        ("modern gap decisions", decisions.as_str()),
        ("strategy plan", strategy.as_str()),
        ("v1 checklist", checklist.as_str()),
        ("standard library review rules", std_rules.as_str()),
    ] {
        assert!(
            text.contains("scalar-only") && text.contains("equality"),
            "{label} must track the scalar-only equality policy"
        );
    }

    for required in [
        "equality is allowed only for Int, Bool, and String",
        "Type::Int | Type::Bool | Type::String | Type::Unknown(_)",
    ] {
        assert!(
            typing.contains(required),
            "typing source missing `{required}`"
        );
    }

    for required in [
        "BinaryOp::EqEq, Value::Int",
        "BinaryOp::EqEq, Value::Bool",
        "BinaryOp::EqEq, Value::String",
        "BinaryOp::BangEq, Value::String",
    ] {
        assert!(runtime.contains(required), "runtime missing `{required}`");
    }

    assert!(
        examples.contains("structural_equality_is_rejected_by_v1_policy"),
        "examples test suite must cover structural equality rejection"
    );
}

#[test]
fn release_docs_and_workflows_cover_v1_gate() {
    let readme = read_primary_docs();
    let roadmap = read("ROADMAP.md");
    let implementation_resume = read("docs/implementation-resume-plan.md");
    let checklist = read("docs/v1-release-checklist.md");
    let alignment = read("docs/release-gate-alignment.md");
    let releasing = read("RELEASING.md");
    let ci = read(".github/workflows/ci.yml");
    let release = read(".github/workflows/release.yml");
    let script = read("scripts/v1-release-gate.sh");

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("RELEASING", releasing.as_str()),
    ] {
        assert!(
            text.contains("docs/v1-release-checklist.md"),
            "{label} must point to the v1 release checklist"
        );
    }

    for (label, text) in [
        ("README", readme.as_str()),
        ("ROADMAP", roadmap.as_str()),
        ("implementation resume plan", implementation_resume.as_str()),
        ("v1 checklist", checklist.as_str()),
        ("RELEASING", releasing.as_str()),
    ] {
        assert!(
            text.contains("release-gate-alignment.md"),
            "{label} must point to the release gate alignment notes"
        );
    }

    for required in [
        "The v1 release priority is to keep the promise narrow",
        "explicit package artifact workflow remains the v1 package boundary",
        "Plain `check` and `run` remain source-compatible",
        "`check --built` and `run --built` consume",
        "`.mgi`",
        "`.mgc`",
        "`.mgb`",
    ] {
        assert!(
            readme.contains(required),
            "README must document the narrow artifact workflow detail `{required}`"
        );
    }

    assert!(
        roadmap.contains("The v1 surface is feature-frozen."),
        "ROADMAP must keep the v1 surface feature-frozen"
    );
    for required in [
        "v1 release boundary hardening",
        "v1 RC readiness verification",
        "keep the v1 feature freeze intact",
        "release gate and GitHub Actions aligned",
    ] {
        assert!(
            implementation_resume.contains(required),
            "implementation resume plan must record v1 hardening detail `{required}`"
        );
    }

    for required in [
        "Canonical Gate",
        "GitHub Actions Contract",
        "scripts/v1-release-gate.sh",
        "scripts/v1-release-gate.sh --with-publish-dry-run",
        ".github/workflows/ci.yml",
        ".github/workflows/release.yml",
        "local gate changes",
    ] {
        assert!(
            alignment.contains(required),
            "release gate alignment doc missing `{required}`"
        );
    }

    for command in [
        "cargo fmt --check",
        "scripts/clippy-check.sh",
        "cargo test --locked",
        "cargo build --locked",
        "mkdir -p \"$gate_tmp\"",
        "target/debug/muga build samples/packages/app/artifact_facade/main.muga",
        "target/debug/muga check --built samples/packages/app/artifact_facade/main.muga",
        "target/debug/muga run --built samples/packages/app/artifact_facade/main.muga",
        "target/debug/muga api-diff --old-artifact-root samples/packages/app/artifact_facade/.muga/build --new-artifact-root samples/packages/app/artifact_facade/.muga/build --package app::artifact_facade --fail-on breaking",
        "target/debug/muga emit-package-archive --archive-root \"$gate_tmp/package-archives\" samples/projects/local_path_shared/src/logging/main.muga",
        "target/debug/muga verify-package-archive \"$package_archive_path\"",
        "target/debug/muga verify-package-archive --expected-hash \"$package_archive_hash\" \"$package_archive_renamed\"",
        "target/debug/muga unpack-package-archive --expected-hash \"$package_archive_hash\" --output-dir \"$gate_tmp/renamed-unpacked-package\" \"$package_archive_renamed\"",
        "target/debug/muga check \"$gate_tmp/renamed-unpacked-package/src/logging/main.muga\"",
        "cp -R samples/projects/my_service \"$gate_tmp/my_service\"",
        "target/debug/muga emit-app-bundle --source-free --output-dir \"$gate_tmp/app-bundle\" --program release-gate \"$gate_tmp/my_service/src/main/main.muga\"",
        "target/debug/muga emit-app-archive --archive-root \"$gate_tmp/app-archives\" --program release-gate \"$gate_tmp/app-bundle\"",
        "target/debug/muga verify-app-archive \"$app_archive_path\"",
        "target/debug/muga verify-app-archive --expected-hash \"$app_archive_hash\" \"$app_archive_renamed\"",
        "target/debug/muga unpack-app-archive --expected-hash \"$app_archive_hash\" --output-dir \"$gate_tmp/renamed-unpacked-app\" \"$app_archive_renamed\"",
        "target/debug/muga unpack-app-archive --output-dir \"$gate_tmp/unpacked-app\" \"$app_archive_path\"",
        "target/debug/muga run-app-bundle \"$gate_tmp/unpacked-app\"",
        "target/debug/muga install-app --output-dir \"$gate_tmp/installed-bin\" --program release-gate \"$gate_tmp/unpacked-app\"",
        "target/debug/muga list-installed-apps --output-dir \"$gate_tmp/installed-bin\"",
        "MUGA_BIN=\"$PWD/target/debug/muga\" \"$gate_tmp/installed-bin/release-gate\"",
        "target/debug/muga uninstall-app --output-dir \"$gate_tmp/installed-bin\" --program release-gate",
        "cp -R samples/projects/resource_export \"$gate_tmp/resource_export\"",
        "target/debug/muga emit-app-bundle --source-free --output-dir \"$gate_tmp/resource-export-bundle\" --program resource-export \"$gate_tmp/resource_export/src/main/main.muga\"",
        "target/debug/muga run-app-bundle \"$gate_tmp/resource-export-bundle\" -- \"$gate_tmp/resource-export-payload.bin\"",
        "cargo package --locked --allow-dirty --offline --list",
        "cargo package --locked --allow-dirty --offline",
    ] {
        assert!(
            script.contains(command),
            "release gate script missing command `{command}`"
        );
        assert!(
            checklist.contains(command) && alignment.contains(command),
            "release gate docs missing command `{command}`"
        );
    }

    assert!(
        ci.contains("run: scripts/v1-release-gate.sh"),
        "CI workflow must invoke the canonical offline release gate script"
    );
    assert!(
        release.contains("scripts/v1-release-gate.sh --with-publish-dry-run"),
        "release workflow must invoke the canonical gate with publish dry run"
    );
    let dry_run_index = release
        .find("scripts/v1-release-gate.sh --with-publish-dry-run")
        .expect("release workflow should run publish dry run gate");
    let publish_index = release
        .find("cargo publish --locked")
        .expect("release workflow should publish after the dry run gate");
    assert!(
        dry_run_index < publish_index,
        "release workflow must run the publish dry-run gate before publishing"
    );
    assert!(
        script.contains("--with-publish-dry-run"),
        "release gate script must expose an explicit publish dry-run option"
    );
    assert!(
        script.contains("cargo publish --dry-run --locked"),
        "release gate script must keep the crates.io publish dry run behind the option"
    );
}

#[test]
fn clippy_policy_is_configured_and_release_gated() {
    let manifest = read("Cargo.toml");
    let clippy_config = read("clippy.toml");
    let clippy_script = read("scripts/clippy-check.sh");
    let gate = read("scripts/v1-release-gate.sh");
    let checklist = read("docs/v1-release-checklist.md");
    let alignment = read("docs/release-gate-alignment.md");

    for required in [
        "[lints.rust]",
        "warnings = \"deny\"",
        "[lints.clippy]",
        "all = \"deny\"",
        "dbg_macro = \"deny\"",
        "todo = \"deny\"",
        "unimplemented = \"deny\"",
    ] {
        assert!(
            manifest.contains(required),
            "Cargo.toml must keep lint policy marker `{required}`"
        );
    }
    assert!(
        clippy_config.contains("msrv = \"1.95.0\""),
        "clippy.toml must pin the Clippy MSRV to the Rust release toolchain"
    );
    assert!(
        clippy_script.contains("cargo clippy --locked --all-targets --all-features -- -D warnings"),
        "Clippy sub-gate must cover locked dependency resolution, every target, every feature, and warning denial"
    );
    assert!(
        gate.contains("scripts/clippy-check.sh"),
        "release gate must call the canonical Clippy sub-gate"
    );
    for (label, text) in [
        ("v1 release checklist", checklist.as_str()),
        ("release gate alignment doc", alignment.as_str()),
    ] {
        assert!(
            text.contains("scripts/clippy-check.sh")
                && text
                    .contains("cargo clippy --locked --all-targets --all-features -- -D warnings")
                && text.contains("clippy.toml"),
            "{label} must document the hardened Clippy sub-gate and MSRV policy"
        );
    }
}

fn assert_hands_off_to_tooling_completions_doctor(label: &str, text: &str) {
    assert!(
        text.contains("shell completions")
            && text.contains("muga doctor")
            && text.contains("tool-only"),
        "{label} must hand off to tooling-only shell completions / muga doctor"
    );
}

fn assert_hands_off_to_std_json_design(label: &str, text: &str) {
    assert!(
        text.contains("std::json")
            && text.contains("Result")
            && text.contains("scalar/collection")
            && text.contains("schema evolution")
            && text.contains("diagnostics"),
        "{label} must hand off to the documented std::json design prerequisites"
    );
}

fn assert_std_json_first_slice_boundary(label: &str, text: &str) {
    assert!(
        text.contains("std::json")
            && text.contains("Result")
            && text.contains("scalar/collection")
            && text.contains("schema evolution")
            && text.contains("diagnostics")
            && (text.contains("schema generation")
                || (text.contains("schema") && text.contains("generation")))
            && text.contains("HTTP")
            && text.contains("Float")
            && text.contains("Decimal")
            && text.contains("Bytes")
            && text.contains("resource handles"),
        "{label} must document the first std::json slice boundary"
    );
}

fn documentation_files() -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    for root in [
        ".github",
        "docs",
        "spec",
        "README.md",
        "ROADMAP.md",
        "RELEASING.md",
        "errors.md",
        "mini-language-spec-v1.md",
    ] {
        let path = Path::new(root);
        if path.is_file() {
            files.push(path.to_path_buf());
        } else {
            files.extend(files_with_extensions(path, &["md", "yml", "yaml"]));
        }
    }
    files
}

fn files_with_extension(root: &Path, extension: &str) -> Vec<std::path::PathBuf> {
    files_with_extensions(root, &[extension])
}

fn files_with_extensions(root: &Path, extensions: &[&str]) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    collect_files(root, extensions, &mut files);
    files.sort();
    files
}

fn collect_files(root: &Path, extensions: &[&str], out: &mut Vec<std::path::PathBuf>) {
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()));
    for entry in entries {
        let path = entry
            .unwrap_or_else(|error| {
                panic!("failed to read entry under {}: {error}", root.display())
            })
            .path();
        if path.is_dir() {
            collect_files(&path, extensions, out);
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extensions.contains(&extension))
        {
            out.push(path);
        }
    }
}

fn collect_diagnostic_prefixes(source: &str, out: &mut BTreeSet<String>) {
    let bytes = source.as_bytes();
    let mut index = 0;
    while index + 4 < bytes.len() {
        if bytes[index] == b'"' {
            let mut end = index + 1;
            while end < bytes.len() && bytes[end] != b'"' {
                end += 1;
            }
            if end < bytes.len() {
                let token = &source[index + 1..end];
                if let Some(prefix) = diagnostic_prefix(token) {
                    out.insert(prefix.to_string());
                }
                index = end;
            }
        }
        index += 1;
    }
}

fn diagnostic_prefix(token: &str) -> Option<&str> {
    let split = token
        .char_indices()
        .find_map(|(index, ch)| ch.is_ascii_digit().then_some(index))?;
    let (prefix, digits) = token.split_at(split);
    if prefix.is_empty()
        || digits.len() != 3
        || !prefix.chars().all(|ch| ch.is_ascii_uppercase())
        || !digits.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }
    Some(prefix)
}

fn read_primary_docs() -> String {
    // README is intentionally a short landing page; release-readiness evidence
    // may live in the documentation guide or the detailed docs it indexes.
    let mut text = String::new();
    for path in documentation_files() {
        text.push_str("\n\n# ");
        text.push_str(&path.display().to_string());
        text.push('\n');
        text.push_str(&read(path));
    }
    text
}

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}
