use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

const ERROR_CATALOG: &str = include_str!("../errors.md");

fn main() -> ExitCode {
    let cli = match Cli::parse(env::args().skip(1).collect()) {
        Ok(cli) => cli,
        Err(message) => {
            eprintln!("{message}");
            eprintln!("{}", usage());
            return ExitCode::from(2);
        }
    };

    match cli.mode {
        Mode::Help => {
            if cli.path.is_empty() {
                println!("{}", usage());
                ExitCode::SUCCESS
            } else {
                match command_usage(&cli.path) {
                    Some(help) => {
                        print!("{help}");
                        ExitCode::SUCCESS
                    }
                    None => {
                        eprintln!("unknown help topic `{}`", cli.path);
                        ExitCode::from(1)
                    }
                }
            }
        }
        Mode::Version => {
            println!("muga {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Mode::Explain => match diagnostic_explanation(&cli.path) {
            Some(explanation) => {
                print!("{explanation}");
                ExitCode::SUCCESS
            }
            None => {
                eprintln!("unknown diagnostic code `{}`", cli.path);
                ExitCode::from(1)
            }
        },
        Mode::Doctor => {
            let report = doctor_report();
            match cli.output_format {
                OutputFormat::Text => print!("{}", doctor_text_output(&report)),
                OutputFormat::Json => println!("{}", doctor_json_output(&report)),
            }
            ExitCode::SUCCESS
        }
        Mode::ShellCompletions => {
            print!(
                "{}",
                shell_completion_script(&cli.path).expect("validated shell completion shell")
            );
            ExitCode::SUCCESS
        }
        Mode::CliCompletions => match cli_completion_output(&cli) {
            Ok(output) => {
                print!("{output}");
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                for diagnostic in diagnostics {
                    eprintln!("{diagnostic}");
                }
                let completion_target = cli
                    .completion_shell
                    .as_deref()
                    .map(|shell| format!("{shell} completions"))
                    .unwrap_or_else(|| "JSON completion spec".to_string());
                eprintln!(
                    "failed to generate {completion_target} for `{}`",
                    cli.completion_program
                        .as_deref()
                        .expect("validated CLI completion program")
                );
                ExitCode::from(1)
            }
        },
        Mode::EmitCliCompletions => match emit_cli_completion_package(&cli) {
            Ok(output) => {
                match cli.output_format {
                    OutputFormat::Text => {
                        for path in output.files {
                            println!("written\t{}", path.display());
                        }
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            completion_package_json_output(
                                "emit-cli-completions",
                                "entry",
                                Path::new(&cli.path),
                                Path::new(
                                    cli.output_dir
                                        .as_deref()
                                        .expect("validated completion output directory"),
                                ),
                                &output,
                            )
                        );
                    }
                }
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                match cli.output_format {
                    OutputFormat::Text => {
                        for diagnostic in diagnostics {
                            eprintln!("{diagnostic}");
                        }
                        eprintln!(
                            "failed to emit completion package for `{}`",
                            cli.completion_program
                                .as_deref()
                                .expect("validated CLI completion program")
                        );
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            completion_package_diagnostic_json_output(
                                &CompletionPackageDiagnosticJsonInput {
                                    command: "emit-cli-completions",
                                    input_key: "entry",
                                    input_path: Path::new(&cli.path),
                                    output_dir: Path::new(
                                        cli.output_dir
                                            .as_deref()
                                            .expect("validated completion output directory"),
                                    ),
                                    program: cli.completion_program.as_deref(),
                                    package_name: cli.packages.first().map(String::as_str),
                                    type_name: cli.schema_type.as_deref(),
                                    status: "error",
                                },
                                &diagnostics,
                            )
                        );
                    }
                }
                ExitCode::from(1)
            }
        },
        Mode::EmitAppCompletions => match emit_app_completion_package(&cli) {
            Ok(output) => {
                match cli.output_format {
                    OutputFormat::Text => {
                        for path in output.files {
                            println!("written\t{}", path.display());
                        }
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            completion_package_json_output(
                                "emit-app-completions",
                                "bundle",
                                Path::new(&cli.path),
                                Path::new(
                                    cli.output_dir
                                        .as_deref()
                                        .expect("validated app completion output directory"),
                                ),
                                &output,
                            )
                        );
                    }
                }
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                match cli.output_format {
                    OutputFormat::Text => {
                        for diagnostic in diagnostics {
                            eprintln!("{diagnostic}");
                        }
                        eprintln!(
                            "failed to emit app completion package for `{}`",
                            cli.completion_program
                                .as_deref()
                                .unwrap_or("app bundle launcher")
                        );
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            completion_package_diagnostic_json_output(
                                &CompletionPackageDiagnosticJsonInput {
                                    command: "emit-app-completions",
                                    input_key: "bundle",
                                    input_path: Path::new(&cli.path),
                                    output_dir: Path::new(
                                        cli.output_dir
                                            .as_deref()
                                            .expect("validated app completion output directory"),
                                    ),
                                    program: cli.completion_program.as_deref(),
                                    package_name: cli.packages.first().map(String::as_str),
                                    type_name: cli.schema_type.as_deref(),
                                    status: "error",
                                },
                                &diagnostics,
                            )
                        );
                    }
                }
                ExitCode::from(1)
            }
        },
        Mode::EmitAppBundle => {
            let output_dir = cli
                .output_dir
                .expect("validated app bundle output directory");
            let result = if cli.source_free_app_bundle {
                muga::emit_source_free_app_bundle(
                    Path::new(&cli.path),
                    Path::new(&output_dir),
                    cli.completion_program.as_deref(),
                )
            } else {
                muga::emit_app_bundle(
                    Path::new(&cli.path),
                    Path::new(&output_dir),
                    cli.completion_program.as_deref(),
                )
            };
            match result {
                Ok(output) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for path in output.files {
                                println!("written\t{}", path.display());
                            }
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                app_bundle_emit_json_output(
                                    &output,
                                    app_bundle_source_mode_label(cli.source_free_app_bundle),
                                )
                            );
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                            eprintln!("failed to emit app bundle for `{}`", cli.path);
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                app_bundle_emit_diagnostic_json_output(
                                    Path::new(&cli.path),
                                    Path::new(&output_dir),
                                    app_bundle_source_mode_label(cli.source_free_app_bundle),
                                    "error",
                                    &diagnostics,
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::InstallApp => {
            let output_dir = cli.output_dir.expect("validated app install output dir");
            let result = if cli.replace_owned_install {
                muga::install_app_bundle_replace_owned(
                    Path::new(&cli.path),
                    Path::new(&output_dir),
                    cli.completion_program.as_deref(),
                )
            } else {
                muga::install_app_bundle(
                    Path::new(&cli.path),
                    Path::new(&output_dir),
                    cli.completion_program.as_deref(),
                )
            };
            match result {
                Ok(output) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for path in output.files {
                                println!("written\t{}", path.display());
                            }
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                app_install_json_output(
                                    &output,
                                    Path::new(&cli.path),
                                    Path::new(&output_dir),
                                    cli.replace_owned_install,
                                )
                            );
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                            eprintln!("failed to install app bundle from `{}`", cli.path);
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                app_install_diagnostic_json_output(
                                    Path::new(&cli.path),
                                    Path::new(&output_dir),
                                    cli.completion_program.as_deref(),
                                    cli.replace_owned_install,
                                    "error",
                                    &diagnostics,
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::UninstallApp => {
            let output_dir = cli.output_dir.expect("validated app uninstall output dir");
            let program = cli
                .completion_program
                .as_deref()
                .expect("validated app uninstall program");
            match muga::uninstall_app_bundle(Path::new(&output_dir), program) {
                Ok(output) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for path in output.files {
                                println!("removed\t{}", path.display());
                            }
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                app_uninstall_json_output(&output, Path::new(&output_dir))
                            );
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                            eprintln!("failed to uninstall app `{program}`");
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                app_uninstall_diagnostic_json_output(
                                    Path::new(&output_dir),
                                    program,
                                    "error",
                                    &diagnostics,
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::ListInstalledApps => {
            let output_dir = cli
                .output_dir
                .expect("validated installed app inventory output dir");
            match muga::list_installed_app_bundles(Path::new(&output_dir)) {
                Ok(output) => {
                    match cli.output_format {
                        OutputFormat::Text => print!("{}", installed_apps_text_output(&output)),
                        OutputFormat::Json => println!("{}", installed_apps_json_output(&output)),
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                            eprintln!("failed to list installed apps in `{output_dir}`");
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                installed_apps_diagnostic_json_output(
                                    Path::new(&output_dir),
                                    "error",
                                    &diagnostics,
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::EmitAppArchive => {
            let archive_root = cli.archive_root.expect("validated app archive root");
            match muga::write_app_bundle_archive(
                Path::new(&cli.path),
                Path::new(&archive_root),
                cli.completion_program.as_deref(),
            ) {
                Ok(output) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            println!("{}", output.path.display());
                            println!("{}", output.content_hash);
                        }
                        OutputFormat::Json => println!("{}", app_archive_emit_json_output(&output)),
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                            eprintln!("failed to emit app archive for `{}`", cli.path);
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                app_archive_emit_diagnostic_json_output(
                                    Path::new(&cli.path),
                                    Path::new(&archive_root),
                                    "error",
                                    &diagnostics,
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::UnpackAppArchive => {
            let output_dir = cli.output_dir.expect("validated app archive output dir");
            let result = match cli.expected_archive_hash.as_deref() {
                Some(expected_hash) => muga::unpack_app_bundle_archive_with_expected_hash(
                    Path::new(&cli.path),
                    expected_hash,
                    Path::new(&output_dir),
                ),
                None => {
                    muga::unpack_app_bundle_archive(Path::new(&cli.path), Path::new(&output_dir))
                }
            };
            match result {
                Ok(output) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for path in output.files {
                                println!("written\t{}", path.display());
                            }
                        }
                        OutputFormat::Json => {
                            println!("{}", app_archive_unpack_json_output(&output))
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                            eprintln!("failed to unpack app archive `{}`", cli.path);
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                app_archive_unpack_diagnostic_json_output(
                                    Path::new(&cli.path),
                                    Path::new(&output_dir),
                                    "error",
                                    &diagnostics,
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::VerifyAppArchive => {
            let result = match cli.expected_archive_hash.as_deref() {
                Some(expected_hash) => muga::verify_app_bundle_archive_with_expected_hash(
                    Path::new(&cli.path),
                    expected_hash,
                ),
                None => muga::verify_app_bundle_archive(Path::new(&cli.path)),
            };
            match result {
                Ok(output) => {
                    match cli.output_format {
                        OutputFormat::Text => print!("{}", app_archive_verify_text_output(&output)),
                        OutputFormat::Json => {
                            println!("{}", app_archive_verify_json_output(&output));
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                            eprintln!("failed to verify app archive `{}`", cli.path);
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                app_archive_diagnostic_json_output(
                                    Path::new(&cli.path),
                                    "verify-app-archive",
                                    "error",
                                    &diagnostics,
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::VerifyPackageArchive => {
            let result = match cli.expected_archive_hash.as_deref() {
                Some(expected_hash) => muga::verify_package_archive_with_expected_hash(
                    Path::new(&cli.path),
                    expected_hash,
                ),
                None => muga::verify_package_archive(Path::new(&cli.path)),
            };
            match result {
                Ok(output) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            print!("{}", package_archive_verify_text_output(&output))
                        }
                        OutputFormat::Json => {
                            println!("{}", package_archive_verify_json_output(&output));
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                            eprintln!("failed to verify package archive `{}`", cli.path);
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                package_archive_diagnostic_json_output(
                                    Path::new(&cli.path),
                                    "verify-package-archive",
                                    "error",
                                    &diagnostics,
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::UnpackPackageArchive => {
            let output_dir = cli
                .output_dir
                .expect("validated package archive output dir");
            let result = match cli.expected_archive_hash.as_deref() {
                Some(expected_hash) => muga::unpack_package_archive_with_expected_hash(
                    Path::new(&cli.path),
                    expected_hash,
                    Path::new(&output_dir),
                ),
                None => muga::unpack_package_archive(Path::new(&cli.path), Path::new(&output_dir)),
            };
            match result {
                Ok(output) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for path in output.files {
                                println!("written\t{}", path.display());
                            }
                        }
                        OutputFormat::Json => {
                            println!("{}", package_archive_unpack_json_output(&output));
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                            eprintln!("failed to unpack package archive `{}`", cli.path);
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                package_archive_unpack_diagnostic_json_output(
                                    Path::new(&cli.path),
                                    Path::new(&output_dir),
                                    "error",
                                    &diagnostics,
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::Syntax => match muga::syntax_check_path(Path::new(&cli.path)) {
            Ok(()) => {
                println!(
                    "{}",
                    command_diagnostic_json_output(Path::new(&cli.path), "syntax", "ok", &[])
                );
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                println!(
                    "{}",
                    command_diagnostic_json_output(
                        Path::new(&cli.path),
                        "syntax",
                        "error",
                        &diagnostics
                    )
                );
                ExitCode::from(1)
            }
        },
        Mode::Check => {
            let result = if cli.use_built_artifacts {
                muga::check_package_aware_path_against_default_build_artifacts(Path::new(&cli.path))
                    .map(|_| ())
            } else if let Some(artifact_root) = &cli.artifact_root {
                muga::check_package_aware_path_against_cached_artifact_root(
                    Path::new(&cli.path),
                    Path::new(artifact_root),
                )
                .map(|_| ())
            } else {
                muga::check_path(Path::new(&cli.path)).map(|_| ())
            };
            match result {
                Ok(_) => {
                    match cli.output_format {
                        OutputFormat::Text => println!("ok"),
                        OutputFormat::Json => {
                            println!("{}", check_json_output(Path::new(&cli.path), "ok", &[]));
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                check_json_output_with_context(
                                    Path::new(&cli.path),
                                    "error",
                                    &diagnostics,
                                    &check_extra_diagnostic_context(&cli),
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::Lint => match if cli.lint_fix {
            muga::fix_lint_path(Path::new(&cli.path)).map(Some)
        } else {
            muga::lint_path(Path::new(&cli.path)).map(|()| None)
        } {
            Ok(outcome) => {
                if let Some(outcome) = outcome {
                    for path in &outcome.changed_files {
                        println!("fixed\t{}", path.display());
                    }
                    println!("fixed {} call(s)", outcome.fixed_calls);
                } else {
                    println!("ok");
                }
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                for diagnostic in diagnostics {
                    eprintln!("{diagnostic}");
                }
                ExitCode::from(1)
            }
        },
        Mode::Metadata => match muga::check_package_aware_path(Path::new(&cli.path)) {
            Ok(check) => {
                println!("{}", metadata_json_output(Path::new(&cli.path), &check));
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                println!(
                    "{}",
                    command_diagnostic_json_output(
                        Path::new(&cli.path),
                        "metadata",
                        "error",
                        &diagnostics
                    )
                );
                ExitCode::from(1)
            }
        },
        Mode::Schema => {
            let check_result = if let Some(artifact_root) = &cli.artifact_root {
                muga::check_package_aware_path_against_cached_artifact_root(
                    Path::new(&cli.path),
                    Path::new(artifact_root),
                )
            } else {
                muga::check_package_aware_path(Path::new(&cli.path))
            };
            match check_result.and_then(|check| {
                muga::render_json_config_schema_for_check(
                    &check,
                    &muga::SchemaExportOptions {
                        package: cli.packages.first().cloned(),
                        type_name: cli.schema_type.clone(),
                        decode_mode: cli.schema_decode_mode,
                    },
                )
            }) {
                Ok(schema) => {
                    println!("{schema}");
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    println!(
                        "{}",
                        command_diagnostic_json_output(
                            Path::new(&cli.path),
                            "schema",
                            "error",
                            &diagnostics
                        )
                    );
                    ExitCode::from(1)
                }
            }
        }
        Mode::Workspace => match muga::check_package_aware_path(Path::new(&cli.path)) {
            Ok(check) => {
                match muga::package::default_build_artifact_root_from_entry(Path::new(&cli.path)) {
                    Ok(artifact_root) => {
                        let project = match muga::package::project_manifest_metadata_from_entry(
                            Path::new(&cli.path),
                        ) {
                            Ok(project) => project,
                            Err(diagnostics) => {
                                println!(
                                    "{}",
                                    command_diagnostic_json_output(
                                        Path::new(&cli.path),
                                        "workspace",
                                        "error",
                                        &diagnostics
                                    )
                                );
                                return ExitCode::from(1);
                            }
                        };
                        println!(
                            "{}",
                            workspace_json_output(
                                Path::new(&cli.path),
                                &artifact_root,
                                project.as_ref(),
                                &check
                            )
                        );
                        ExitCode::SUCCESS
                    }
                    Err(diagnostics) => {
                        println!(
                            "{}",
                            command_diagnostic_json_output(
                                Path::new(&cli.path),
                                "workspace",
                                "error",
                                &diagnostics
                            )
                        );
                        ExitCode::from(1)
                    }
                }
            }
            Err(diagnostics) => {
                println!(
                    "{}",
                    command_diagnostic_json_output(
                        Path::new(&cli.path),
                        "workspace",
                        "error",
                        &diagnostics
                    )
                );
                ExitCode::from(1)
            }
        },
        Mode::WhyRebuild => {
            let (artifact_root, selection) = if let Some(artifact_root) = &cli.artifact_root {
                (
                    PathBuf::from(artifact_root),
                    muga::ArtifactRootSelection::ArtifactRoot,
                )
            } else {
                match muga::default_build_artifact_root(Path::new(&cli.path)) {
                    Ok(artifact_root) => (artifact_root, muga::ArtifactRootSelection::Built),
                    Err(diagnostics) => {
                        match cli.output_format {
                            OutputFormat::Text => {
                                for diagnostic in diagnostics {
                                    eprintln!("{diagnostic}");
                                }
                            }
                            OutputFormat::Json => {
                                println!(
                                    "{}",
                                    command_diagnostic_json_output(
                                        Path::new(&cli.path),
                                        "why-rebuild",
                                        "error",
                                        &diagnostics
                                    )
                                );
                            }
                        }
                        return ExitCode::from(1);
                    }
                }
            };
            match muga::explain_package_artifact_cache(
                Path::new(&cli.path),
                &artifact_root,
                selection,
            ) {
                Ok(explanation) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            print!(
                                "{}",
                                why_rebuild_text_output(Path::new(&cli.path), &explanation)
                            );
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                why_rebuild_json_output(Path::new(&cli.path), &explanation)
                            );
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                command_diagnostic_json_output_with_context(
                                    Path::new(&cli.path),
                                    "why-rebuild",
                                    "error",
                                    &diagnostics,
                                    &why_rebuild_extra_diagnostic_context(
                                        Path::new(&cli.path),
                                        &artifact_root,
                                        selection,
                                    ),
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::ApiDiff => match api_diff_from_cli(&cli) {
            Ok(diff) => {
                match cli.output_format {
                    OutputFormat::Text => print!("{}", api_diff_text_output(&diff)),
                    OutputFormat::Json => println!("{}", api_diff_json_output(&diff)),
                }
                if cli
                    .api_diff_fail_on
                    .is_some_and(|fail_on| fail_on.matches(diff.status))
                {
                    ExitCode::from(1)
                } else {
                    ExitCode::SUCCESS
                }
            }
            Err(diagnostics) => {
                match cli.output_format {
                    OutputFormat::Text => {
                        for diagnostic in diagnostics {
                            eprintln!("{diagnostic}");
                        }
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            api_diff_diagnostic_json_output(
                                cli.packages.first().map(String::as_str),
                                &diagnostics,
                            )
                        );
                    }
                }
                ExitCode::from(1)
            }
        },
        Mode::Completions => match muga::check_package_aware_path(Path::new(&cli.path)) {
            Ok(check) => {
                println!("{}", completions_json_output(Path::new(&cli.path), &check));
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                println!(
                    "{}",
                    command_diagnostic_json_output(
                        Path::new(&cli.path),
                        "completions",
                        "error",
                        &diagnostics
                    )
                );
                ExitCode::from(1)
            }
        },
        Mode::Definition => match muga::check_package_aware_path(Path::new(&cli.path)) {
            Ok(check) => {
                let position = cli.position.expect("validated definition position");
                println!(
                    "{}",
                    definition_json_output(Path::new(&cli.path), &check, position)
                );
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                println!(
                    "{}",
                    command_diagnostic_json_output(
                        Path::new(&cli.path),
                        "definition",
                        "error",
                        &diagnostics
                    )
                );
                ExitCode::from(1)
            }
        },
        Mode::References => match muga::check_package_aware_path(Path::new(&cli.path)) {
            Ok(check) => {
                let position = cli.position.expect("validated references position");
                println!(
                    "{}",
                    references_json_output(Path::new(&cli.path), &check, position)
                );
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                println!(
                    "{}",
                    command_diagnostic_json_output(
                        Path::new(&cli.path),
                        "references",
                        "error",
                        &diagnostics
                    )
                );
                ExitCode::from(1)
            }
        },
        Mode::Hover => match muga::check_package_aware_path(Path::new(&cli.path)) {
            Ok(check) => {
                let position = cli.position.expect("validated hover position");
                println!(
                    "{}",
                    hover_json_output(Path::new(&cli.path), &check, position)
                );
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                println!(
                    "{}",
                    command_diagnostic_json_output(
                        Path::new(&cli.path),
                        "hover",
                        "error",
                        &diagnostics
                    )
                );
                ExitCode::from(1)
            }
        },
        Mode::Run => match if cli.use_built_artifacts {
            muga::run_path_against_default_build_artifacts_with_args(
                Path::new(&cli.path),
                &cli.program_args,
            )
        } else if let Some(artifact_root) = &cli.artifact_root {
            muga::run_path_against_artifact_root_with_args(
                Path::new(&cli.path),
                Path::new(artifact_root),
                &cli.program_args,
            )
        } else {
            muga::run_path_with_args(Path::new(&cli.path), &cli.program_args)
        } {
            Ok(outcome) => {
                match cli.output_format {
                    OutputFormat::Text => print_run_outcome(&outcome),
                    OutputFormat::Json => {
                        println!("{}", run_json_output(Path::new(&cli.path), &outcome));
                    }
                }
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                match cli.output_format {
                    OutputFormat::Text => {
                        for diagnostic in diagnostics {
                            eprintln!("{diagnostic}");
                        }
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            command_diagnostic_json_output(
                                Path::new(&cli.path),
                                "run",
                                "error",
                                &diagnostics
                            )
                        );
                    }
                }
                ExitCode::from(1)
            }
        },
        Mode::RunAppBundle => {
            match muga::run_app_bundle_with_args(Path::new(&cli.path), &cli.program_args) {
                Ok(outcome) => {
                    match cli.output_format {
                        OutputFormat::Text => print_run_outcome(&outcome),
                        OutputFormat::Json => {
                            println!("{}", run_json_output(Path::new(&cli.path), &outcome));
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                command_diagnostic_json_output(
                                    Path::new(&cli.path),
                                    "run-app-bundle",
                                    "error",
                                    &diagnostics
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::Test => match muga::test_path(Path::new(&cli.path)) {
            Ok(outcome) => {
                match cli.output_format {
                    OutputFormat::Text => print_test_outcome(&outcome),
                    OutputFormat::Json => {
                        println!("{}", test_json_output(Path::new(&cli.path), &outcome));
                    }
                }
                if outcome.failed_count() == 0 {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::from(1)
                }
            }
            Err(diagnostics) => {
                match cli.output_format {
                    OutputFormat::Text => {
                        for diagnostic in diagnostics {
                            eprintln!("{diagnostic}");
                        }
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            command_diagnostic_json_output(
                                Path::new(&cli.path),
                                "test",
                                "error",
                                &diagnostics
                            )
                        );
                    }
                }
                ExitCode::from(1)
            }
        },
        Mode::Fmt => {
            let result = if cli.format_check {
                muga::check_format_path(Path::new(&cli.path))
            } else {
                muga::format_path(Path::new(&cli.path))
            };
            match result {
                Ok(outcome) => {
                    if cli.format_check && outcome.changed {
                        println!("would format\t{}", outcome.path.display());
                        ExitCode::from(1)
                    } else if outcome.changed {
                        println!("formatted\t{}", outcome.path.display());
                        ExitCode::SUCCESS
                    } else {
                        println!("ok");
                        ExitCode::SUCCESS
                    }
                }
                Err(diagnostics) => {
                    for diagnostic in diagnostics {
                        eprintln!("{diagnostic}");
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::Doc => match muga::render_package_docs(Path::new(&cli.path)) {
            Ok(markdown) => {
                print!("{markdown}");
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                for diagnostic in diagnostics {
                    eprintln!("{diagnostic}");
                }
                ExitCode::from(1)
            }
        },
        Mode::New => {
            if cli.list_new_templates {
                match cli.output_format {
                    OutputFormat::Text => print_project_template_list(),
                    OutputFormat::Json => println!("{}", project_template_list_json_output()),
                }
                ExitCode::SUCCESS
            } else {
                match muga::create_project_template(Path::new(&cli.path), cli.new_template) {
                    Ok(output) => {
                        println!("created\t{}", output.root.display());
                        println!("entry\t{}", output.entry.display());
                        ExitCode::SUCCESS
                    }
                    Err(diagnostics) => {
                        for diagnostic in diagnostics {
                            eprintln!("{diagnostic}");
                        }
                        ExitCode::from(1)
                    }
                }
            }
        }
        Mode::EmitInterface => {
            let artifact_root = cli.artifact_root.expect("validated artifact root");
            match muga::write_package_interface_artifacts(
                Path::new(&cli.path),
                Path::new(&artifact_root),
                &cli.packages,
            ) {
                Ok(paths) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for path in paths {
                                println!("{}", path.display());
                            }
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                artifact_emission_json_output(
                                    Path::new(&cli.path),
                                    "emit-interface",
                                    Path::new(&artifact_root),
                                    &paths,
                                )
                            );
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                command_diagnostic_json_output_with_context(
                                    Path::new(&cli.path),
                                    "emit-interface",
                                    "error",
                                    &diagnostics,
                                    &artifact_emission_extra_diagnostic_context(
                                        Path::new(&cli.path),
                                        Path::new(&artifact_root),
                                    ),
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::EmitCheckCache => {
            let artifact_root = cli.artifact_root.expect("validated artifact root");
            match muga::write_package_check_cache_artifact_for_root(
                Path::new(&cli.path),
                Path::new(&artifact_root),
            ) {
                Ok(path) => {
                    match cli.output_format {
                        OutputFormat::Text => println!("{}", path.display()),
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                artifact_emission_json_output(
                                    Path::new(&cli.path),
                                    "emit-check-cache",
                                    Path::new(&artifact_root),
                                    std::slice::from_ref(&path),
                                )
                            );
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                command_diagnostic_json_output_with_context(
                                    Path::new(&cli.path),
                                    "emit-check-cache",
                                    "error",
                                    &diagnostics,
                                    &artifact_emission_extra_diagnostic_context(
                                        Path::new(&cli.path),
                                        Path::new(&artifact_root),
                                    ),
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::EmitArtifacts => {
            let artifact_root = cli.artifact_root.expect("validated artifact root");
            match muga::write_package_artifacts(Path::new(&cli.path), Path::new(&artifact_root)) {
                Ok(paths) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for path in paths {
                                println!("{}", path.display());
                            }
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                artifact_emission_json_output(
                                    Path::new(&cli.path),
                                    "emit-artifacts",
                                    Path::new(&artifact_root),
                                    &paths,
                                )
                            );
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                command_diagnostic_json_output_with_context(
                                    Path::new(&cli.path),
                                    "emit-artifacts",
                                    "error",
                                    &diagnostics,
                                    &artifact_emission_extra_diagnostic_context(
                                        Path::new(&cli.path),
                                        Path::new(&artifact_root),
                                    ),
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::EmitPackageArchive => {
            let archive_root = cli.archive_root.expect("validated archive root");
            match muga::write_package_archive(Path::new(&cli.path), Path::new(&archive_root)) {
                Ok(output) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            if cli.dependency_snippet {
                                print!("{}", package_archive_dependency_snippet(&output));
                            } else {
                                println!("{}", output.path.display());
                                println!("{}", output.content_hash);
                            }
                        }
                        OutputFormat::Json => {
                            println!("{}", package_archive_emit_json_output(&output));
                        }
                    }
                    ExitCode::SUCCESS
                }
                Err(diagnostics) => {
                    match cli.output_format {
                        OutputFormat::Text => {
                            for diagnostic in diagnostics {
                                eprintln!("{diagnostic}");
                            }
                        }
                        OutputFormat::Json => {
                            println!(
                                "{}",
                                package_archive_emit_diagnostic_json_output(
                                    Path::new(&cli.path),
                                    Path::new(&archive_root),
                                    "error",
                                    &diagnostics,
                                )
                            );
                        }
                    }
                    ExitCode::from(1)
                }
            }
        }
        Mode::Build => match muga::build_package_artifacts(Path::new(&cli.path)) {
            Ok(output) => {
                match cli.output_format {
                    OutputFormat::Text => {
                        for path in &output.artifacts {
                            let status = build_artifact_status(&output, path);
                            println!("{status}\t{}", path.display());
                        }
                    }
                    OutputFormat::Json => {
                        println!("{}", build_json_output(Path::new(&cli.path), &output));
                    }
                }
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                match cli.output_format {
                    OutputFormat::Text => {
                        for diagnostic in diagnostics {
                            eprintln!("{diagnostic}");
                        }
                    }
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            command_diagnostic_json_output_with_context(
                                Path::new(&cli.path),
                                "build",
                                "error",
                                &diagnostics,
                                &build_extra_diagnostic_context(Path::new(&cli.path)),
                            )
                        );
                    }
                }
                ExitCode::from(1)
            }
        },
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Help,
    Version,
    Explain,
    Doctor,
    ShellCompletions,
    CliCompletions,
    EmitCliCompletions,
    EmitAppCompletions,
    EmitAppBundle,
    InstallApp,
    UninstallApp,
    ListInstalledApps,
    EmitAppArchive,
    UnpackAppArchive,
    VerifyAppArchive,
    VerifyPackageArchive,
    UnpackPackageArchive,
    Syntax,
    Check,
    Lint,
    Run,
    RunAppBundle,
    Test,
    Fmt,
    Doc,
    Metadata,
    Schema,
    Workspace,
    WhyRebuild,
    ApiDiff,
    Completions,
    Definition,
    References,
    Hover,
    New,
    EmitInterface,
    EmitCheckCache,
    EmitArtifacts,
    EmitPackageArchive,
    Build,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Cli {
    mode: Mode,
    path: String,
    artifact_root: Option<String>,
    old_artifact_root: Option<String>,
    new_artifact_root: Option<String>,
    api_diff_fail_on: Option<ApiDiffFailOn>,
    expected_archive_hash: Option<String>,
    archive_root: Option<String>,
    output_dir: Option<String>,
    use_built_artifacts: bool,
    output_format: OutputFormat,
    format_check: bool,
    lint_fix: bool,
    source_free_app_bundle: bool,
    replace_owned_install: bool,
    new_template: muga::ProjectTemplate,
    list_new_templates: bool,
    dependency_snippet: bool,
    packages: Vec<String>,
    schema_type: Option<String>,
    schema_decode_mode: muga::SchemaDecodeMode,
    completion_shell: Option<String>,
    completion_program: Option<String>,
    position: Option<SourcePositionArg>,
    program_args: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SourcePositionArg {
    line: usize,
    column: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ApiDiffFailOn {
    Unknown,
    Breaking,
    SourceCompatible,
}

impl ApiDiffFailOn {
    fn matches(self, status: muga::api_diff::PackageApiDiffStatus) -> bool {
        match self {
            Self::Unknown => status == muga::api_diff::PackageApiDiffStatus::Unknown,
            Self::Breaking => matches!(
                status,
                muga::api_diff::PackageApiDiffStatus::Unknown
                    | muga::api_diff::PackageApiDiffStatus::Breaking
            ),
            Self::SourceCompatible => status != muga::api_diff::PackageApiDiffStatus::Compatible,
        }
    }
}

impl Cli {
    fn empty_for_mode(mode: Mode) -> Self {
        Self {
            mode,
            path: String::new(),
            artifact_root: None,
            old_artifact_root: None,
            new_artifact_root: None,
            api_diff_fail_on: None,
            expected_archive_hash: None,
            archive_root: None,
            output_dir: None,
            use_built_artifacts: false,
            output_format: OutputFormat::Text,
            format_check: false,
            lint_fix: false,
            source_free_app_bundle: false,
            replace_owned_install: false,
            new_template: muga::ProjectTemplate::App,
            list_new_templates: false,
            dependency_snippet: false,
            packages: Vec::new(),
            schema_type: None,
            schema_decode_mode: muga::SchemaDecodeMode::Required,
            completion_shell: None,
            completion_program: None,
            position: None,
            program_args: Vec::new(),
        }
    }

    fn parse(mut args: Vec<String>) -> Result<Self, String> {
        if args.is_empty() {
            return Err("missing source file".to_string());
        }

        if matches!(args.first().map(String::as_str), Some("--help" | "-h")) {
            if args.len() != 1 {
                return Err("help does not accept arguments".to_string());
            }
            return Ok(Self::empty_for_mode(Mode::Help));
        }

        if matches!(args.first().map(String::as_str), Some("help")) {
            if args.len() > 2 {
                return Err("help accepts at most one command".to_string());
            }
            let mut cli = Self::empty_for_mode(Mode::Help);
            if let Some(topic) = args.get(1) {
                cli.path = normalized_help_topic(topic).to_string();
            }
            return Ok(cli);
        }

        if matches!(
            args.first().map(String::as_str),
            Some("--version" | "-V" | "version")
        ) {
            if args.len() != 1 {
                return Err("version does not accept arguments".to_string());
            }
            return Ok(Self::empty_for_mode(Mode::Version));
        }

        let mode = if matches!(
            args.first().map(String::as_str),
            Some(
                "syntax"
                    | "explain"
                    | "doctor"
                    | "shell-completions"
                    | "cli-completions"
                    | "emit-cli-completions"
                    | "emit-app-completions"
                    | "emit-app-bundle"
                    | "install-app"
                    | "uninstall-app"
                    | "list-installed-apps"
                    | "emit-app-archive"
                    | "unpack-app-archive"
                    | "verify-app-archive"
                    | "verify-package-archive"
                    | "unpack-package-archive"
                    | "check"
                    | "lint"
                    | "run"
                    | "run-app-bundle"
                    | "test"
                    | "fmt"
                    | "doc"
                    | "metadata"
                    | "schema"
                    | "workspace"
                    | "why-rebuild"
                    | "api-diff"
                    | "completions"
                    | "definition"
                    | "references"
                    | "hover"
                    | "new"
                    | "emit-interface"
                    | "emit-check-cache"
                    | "emit-artifacts"
                    | "emit-package-archive"
                    | "build"
            )
        ) {
            match args.remove(0).as_str() {
                "explain" => Mode::Explain,
                "doctor" => Mode::Doctor,
                "shell-completions" => Mode::ShellCompletions,
                "cli-completions" => Mode::CliCompletions,
                "emit-cli-completions" => Mode::EmitCliCompletions,
                "emit-app-completions" => Mode::EmitAppCompletions,
                "emit-app-bundle" => Mode::EmitAppBundle,
                "install-app" => Mode::InstallApp,
                "uninstall-app" => Mode::UninstallApp,
                "list-installed-apps" => Mode::ListInstalledApps,
                "emit-app-archive" => Mode::EmitAppArchive,
                "unpack-app-archive" => Mode::UnpackAppArchive,
                "verify-app-archive" => Mode::VerifyAppArchive,
                "verify-package-archive" => Mode::VerifyPackageArchive,
                "unpack-package-archive" => Mode::UnpackPackageArchive,
                "syntax" => Mode::Syntax,
                "check" => Mode::Check,
                "lint" => Mode::Lint,
                "run" => Mode::Run,
                "run-app-bundle" => Mode::RunAppBundle,
                "test" => Mode::Test,
                "fmt" => Mode::Fmt,
                "doc" => Mode::Doc,
                "metadata" => Mode::Metadata,
                "schema" => Mode::Schema,
                "workspace" => Mode::Workspace,
                "why-rebuild" => Mode::WhyRebuild,
                "api-diff" => Mode::ApiDiff,
                "completions" => Mode::Completions,
                "definition" => Mode::Definition,
                "references" => Mode::References,
                "hover" => Mode::Hover,
                "new" => Mode::New,
                "emit-interface" => Mode::EmitInterface,
                "emit-check-cache" => Mode::EmitCheckCache,
                "emit-artifacts" => Mode::EmitArtifacts,
                "emit-package-archive" => Mode::EmitPackageArchive,
                "build" => Mode::Build,
                _ => unreachable!("mode was already checked"),
            }
        } else {
            Mode::Run
        };

        let mut artifact_root = None;
        let mut old_artifact_root = None;
        let mut new_artifact_root = None;
        let mut api_diff_fail_on = None;
        let mut expected_archive_hash = None;
        let mut archive_root = None;
        let mut output_dir = None;
        let mut use_built_artifacts = false;
        let mut output_format = OutputFormat::Text;
        let mut output_format_was_set = false;
        let mut format_check = false;
        let mut lint_fix = false;
        let mut source_free_app_bundle = false;
        let mut replace_owned_install = false;
        let mut new_template = muga::ProjectTemplate::App;
        let mut new_template_was_set = false;
        let mut list_new_templates = false;
        let mut dependency_snippet = false;
        let mut packages = Vec::new();
        let mut schema_type = None;
        let mut schema_decode_mode = muga::SchemaDecodeMode::Required;
        let mut schema_decode_mode_was_set = false;
        let mut completion_shell = None;
        let mut completion_program = None;
        let mut cli_completion_positionals = Vec::new();
        let mut position_line = None;
        let mut position_column = None;
        let mut path = None;
        let mut program_args = Vec::new();
        let mut saw_program_args_separator = false;
        let mut index = 0;
        while index < args.len() {
            let arg = &args[index];
            if arg == "--" {
                saw_program_args_separator = true;
                program_args.extend(args[index + 1..].iter().cloned());
                break;
            } else if arg == "--artifact-root" {
                index += 1;
                let Some(root) = args.get(index) else {
                    return Err("missing value for --artifact-root".to_string());
                };
                if artifact_root.replace(root.clone()).is_some() {
                    return Err("--artifact-root was provided more than once".to_string());
                }
            } else if let Some(root) = arg.strip_prefix("--artifact-root=") {
                if root.is_empty() {
                    return Err("missing value for --artifact-root".to_string());
                }
                if artifact_root.replace(root.to_string()).is_some() {
                    return Err("--artifact-root was provided more than once".to_string());
                }
            } else if arg == "--old-artifact-root" {
                index += 1;
                let Some(root) = args.get(index) else {
                    return Err("missing value for --old-artifact-root".to_string());
                };
                if root.is_empty() {
                    return Err("missing value for --old-artifact-root".to_string());
                }
                if old_artifact_root.replace(root.clone()).is_some() {
                    return Err("--old-artifact-root was provided more than once".to_string());
                }
            } else if let Some(root) = arg.strip_prefix("--old-artifact-root=") {
                if root.is_empty() {
                    return Err("missing value for --old-artifact-root".to_string());
                }
                if old_artifact_root.replace(root.to_string()).is_some() {
                    return Err("--old-artifact-root was provided more than once".to_string());
                }
            } else if arg == "--new-artifact-root" {
                index += 1;
                let Some(root) = args.get(index) else {
                    return Err("missing value for --new-artifact-root".to_string());
                };
                if root.is_empty() {
                    return Err("missing value for --new-artifact-root".to_string());
                }
                if new_artifact_root.replace(root.clone()).is_some() {
                    return Err("--new-artifact-root was provided more than once".to_string());
                }
            } else if let Some(root) = arg.strip_prefix("--new-artifact-root=") {
                if root.is_empty() {
                    return Err("missing value for --new-artifact-root".to_string());
                }
                if new_artifact_root.replace(root.to_string()).is_some() {
                    return Err("--new-artifact-root was provided more than once".to_string());
                }
            } else if arg == "--fail-on" {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing value for --fail-on".to_string());
                };
                if api_diff_fail_on
                    .replace(parse_api_diff_fail_on(value)?)
                    .is_some()
                {
                    return Err("--fail-on was provided more than once".to_string());
                }
            } else if let Some(value) = arg.strip_prefix("--fail-on=") {
                if value.is_empty() {
                    return Err("missing value for --fail-on".to_string());
                }
                if api_diff_fail_on
                    .replace(parse_api_diff_fail_on(value)?)
                    .is_some()
                {
                    return Err("--fail-on was provided more than once".to_string());
                }
            } else if arg == "--expected-hash" {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing value for --expected-hash".to_string());
                };
                if value.is_empty() {
                    return Err("missing value for --expected-hash".to_string());
                }
                if expected_archive_hash.replace(value.clone()).is_some() {
                    return Err("--expected-hash was provided more than once".to_string());
                }
            } else if let Some(value) = arg.strip_prefix("--expected-hash=") {
                if value.is_empty() {
                    return Err("missing value for --expected-hash".to_string());
                }
                if expected_archive_hash.replace(value.to_string()).is_some() {
                    return Err("--expected-hash was provided more than once".to_string());
                }
            } else if arg == "--archive-root" {
                index += 1;
                let Some(root) = args.get(index) else {
                    return Err("missing value for --archive-root".to_string());
                };
                if archive_root.replace(root.clone()).is_some() {
                    return Err("--archive-root was provided more than once".to_string());
                }
            } else if let Some(root) = arg.strip_prefix("--archive-root=") {
                if root.is_empty() {
                    return Err("missing value for --archive-root".to_string());
                }
                if archive_root.replace(root.to_string()).is_some() {
                    return Err("--archive-root was provided more than once".to_string());
                }
            } else if arg == "--output-dir" {
                index += 1;
                let Some(root) = args.get(index) else {
                    return Err("missing value for --output-dir".to_string());
                };
                if root.is_empty() {
                    return Err("missing value for --output-dir".to_string());
                }
                if output_dir.replace(root.clone()).is_some() {
                    return Err("--output-dir was provided more than once".to_string());
                }
            } else if let Some(root) = arg.strip_prefix("--output-dir=") {
                if root.is_empty() {
                    return Err("missing value for --output-dir".to_string());
                }
                if output_dir.replace(root.to_string()).is_some() {
                    return Err("--output-dir was provided more than once".to_string());
                }
            } else if arg == "--built" {
                if use_built_artifacts {
                    return Err("--built was provided more than once".to_string());
                }
                use_built_artifacts = true;
            } else if arg == "--check" {
                if format_check {
                    return Err("--check was provided more than once".to_string());
                }
                format_check = true;
            } else if arg == "--fix" {
                if lint_fix {
                    return Err("--fix was provided more than once".to_string());
                }
                lint_fix = true;
            } else if arg == "--source-free" {
                if source_free_app_bundle {
                    return Err("--source-free was provided more than once".to_string());
                }
                source_free_app_bundle = true;
            } else if arg == "--replace-owned" {
                if replace_owned_install {
                    return Err("--replace-owned was provided more than once".to_string());
                }
                replace_owned_install = true;
            } else if arg == "--format" {
                index += 1;
                let Some(format) = args.get(index) else {
                    return Err("missing value for --format".to_string());
                };
                output_format = parse_output_format(format)?;
                if output_format_was_set {
                    return Err("--format was provided more than once".to_string());
                }
                output_format_was_set = true;
            } else if let Some(format) = arg.strip_prefix("--format=") {
                if format.is_empty() {
                    return Err("missing value for --format".to_string());
                }
                output_format = parse_output_format(format)?;
                if output_format_was_set {
                    return Err("--format was provided more than once".to_string());
                }
                output_format_was_set = true;
            } else if arg == "--template" {
                index += 1;
                let Some(template) = args.get(index) else {
                    return Err("missing value for --template".to_string());
                };
                new_template = parse_new_template(template)?;
                if new_template_was_set {
                    return Err("--template was provided more than once".to_string());
                }
                new_template_was_set = true;
            } else if let Some(template) = arg.strip_prefix("--template=") {
                if template.is_empty() {
                    return Err("missing value for --template".to_string());
                }
                new_template = parse_new_template(template)?;
                if new_template_was_set {
                    return Err("--template was provided more than once".to_string());
                }
                new_template_was_set = true;
            } else if arg == "--list-templates" {
                if list_new_templates {
                    return Err("--list-templates was provided more than once".to_string());
                }
                list_new_templates = true;
            } else if arg == "--dependency-snippet" {
                if dependency_snippet {
                    return Err("--dependency-snippet was provided more than once".to_string());
                }
                dependency_snippet = true;
            } else if arg == "--package" {
                index += 1;
                let Some(package) = args.get(index) else {
                    return Err("missing value for --package".to_string());
                };
                packages.push(package.clone());
            } else if let Some(package) = arg.strip_prefix("--package=") {
                if package.is_empty() {
                    return Err("missing value for --package".to_string());
                }
                packages.push(package.to_string());
            } else if arg == "--type" {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing value for --type".to_string());
                };
                if value.is_empty() {
                    return Err("missing value for --type".to_string());
                }
                if schema_type.replace(value.clone()).is_some() {
                    return Err("--type was provided more than once".to_string());
                }
            } else if let Some(value) = arg.strip_prefix("--type=") {
                if value.is_empty() {
                    return Err("missing value for --type".to_string());
                }
                if schema_type.replace(value.to_string()).is_some() {
                    return Err("--type was provided more than once".to_string());
                }
            } else if arg == "--program" {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing value for --program".to_string());
                };
                if value.is_empty() {
                    return Err("missing value for --program".to_string());
                }
                if completion_program.replace(value.clone()).is_some() {
                    return Err("--program was provided more than once".to_string());
                }
            } else if let Some(value) = arg.strip_prefix("--program=") {
                if value.is_empty() {
                    return Err("missing value for --program".to_string());
                }
                if completion_program.replace(value.to_string()).is_some() {
                    return Err("--program was provided more than once".to_string());
                }
            } else if arg == "--decode-mode" {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing value for --decode-mode".to_string());
                };
                schema_decode_mode = parse_schema_decode_mode(value)?;
                if schema_decode_mode_was_set {
                    return Err("--decode-mode was provided more than once".to_string());
                }
                schema_decode_mode_was_set = true;
            } else if let Some(value) = arg.strip_prefix("--decode-mode=") {
                if value.is_empty() {
                    return Err("missing value for --decode-mode".to_string());
                }
                schema_decode_mode = parse_schema_decode_mode(value)?;
                if schema_decode_mode_was_set {
                    return Err("--decode-mode was provided more than once".to_string());
                }
                schema_decode_mode_was_set = true;
            } else if arg == "--line" {
                index += 1;
                let Some(line) = args.get(index) else {
                    return Err("missing value for --line".to_string());
                };
                if position_line
                    .replace(parse_position_value("--line", line)?)
                    .is_some()
                {
                    return Err("--line was provided more than once".to_string());
                }
            } else if let Some(line) = arg.strip_prefix("--line=") {
                if line.is_empty() {
                    return Err("missing value for --line".to_string());
                }
                if position_line
                    .replace(parse_position_value("--line", line)?)
                    .is_some()
                {
                    return Err("--line was provided more than once".to_string());
                }
            } else if arg == "--column" {
                index += 1;
                let Some(column) = args.get(index) else {
                    return Err("missing value for --column".to_string());
                };
                if position_column
                    .replace(parse_position_value("--column", column)?)
                    .is_some()
                {
                    return Err("--column was provided more than once".to_string());
                }
            } else if let Some(column) = arg.strip_prefix("--column=") {
                if column.is_empty() {
                    return Err("missing value for --column".to_string());
                }
                if position_column
                    .replace(parse_position_value("--column", column)?)
                    .is_some()
                {
                    return Err("--column was provided more than once".to_string());
                }
            } else if arg.starts_with('-') {
                return Err(format!("unknown option `{arg}`"));
            } else if mode == Mode::CliCompletions {
                cli_completion_positionals.push(arg.clone());
            } else if path.replace(arg.clone()).is_some() {
                return Err(if mode == Mode::Explain {
                    "too many diagnostic codes".to_string()
                } else if mode == Mode::ShellCompletions {
                    "too many shells".to_string()
                } else if mode == Mode::Doctor {
                    "doctor does not accept a source file".to_string()
                } else if mode == Mode::UninstallApp {
                    "uninstall-app does not accept an app bundle directory".to_string()
                } else if matches!(mode, Mode::EmitAppCompletions | Mode::RunAppBundle) {
                    "too many app bundle directories".to_string()
                } else {
                    "too many source files".to_string()
                });
            }
            index += 1;
        }

        if (old_artifact_root.is_some() || new_artifact_root.is_some()) && mode != Mode::ApiDiff {
            return Err(
                "--old-artifact-root and --new-artifact-root are only supported with `api-diff`"
                    .to_string(),
            );
        }
        if api_diff_fail_on.is_some() && mode != Mode::ApiDiff {
            return Err("--fail-on is only supported with `api-diff`".to_string());
        }
        if expected_archive_hash.is_some()
            && !matches!(
                mode,
                Mode::UnpackAppArchive
                    | Mode::VerifyAppArchive
                    | Mode::VerifyPackageArchive
                    | Mode::UnpackPackageArchive
            )
        {
            return Err(
                "--expected-hash is only supported with `unpack-app-archive`, `verify-app-archive`, `verify-package-archive`, or `unpack-package-archive`"
                    .to_string(),
            );
        }
        if mode == Mode::ApiDiff {
            if old_artifact_root.is_none() {
                return Err("api-diff requires --old-artifact-root".to_string());
            }
            if new_artifact_root.is_none() {
                return Err("api-diff requires --new-artifact-root".to_string());
            }
        }
        if artifact_root.is_some()
            && !matches!(
                mode,
                Mode::Check
                    | Mode::Run
                    | Mode::CliCompletions
                    | Mode::EmitCliCompletions
                    | Mode::Schema
                    | Mode::WhyRebuild
                    | Mode::EmitInterface
                    | Mode::EmitCheckCache
                    | Mode::EmitArtifacts
            )
        {
            return Err(
                "--artifact-root is only supported with `check`, `run`, `cli-completions`, `emit-cli-completions`, `schema`, `why-rebuild`, `emit-interface`, `emit-check-cache`, or `emit-artifacts`"
                    .to_string(),
            );
        }
        if archive_root.is_some()
            && !matches!(mode, Mode::EmitPackageArchive | Mode::EmitAppArchive)
        {
            return Err(
                "--archive-root is only supported with `emit-package-archive` or `emit-app-archive`"
                    .to_string(),
            );
        }
        if output_dir.is_some()
            && !matches!(
                mode,
                Mode::EmitCliCompletions
                    | Mode::EmitAppCompletions
                    | Mode::EmitAppBundle
                    | Mode::InstallApp
                    | Mode::UninstallApp
                    | Mode::ListInstalledApps
                    | Mode::UnpackAppArchive
                    | Mode::UnpackPackageArchive
            )
        {
            return Err(
                "--output-dir is only supported with `emit-cli-completions`, `emit-app-completions`, `emit-app-bundle`, `install-app`, `uninstall-app`, `list-installed-apps`, `unpack-app-archive`, or `unpack-package-archive`"
                    .to_string(),
            );
        }
        if use_built_artifacts && artifact_root.is_some() {
            return Err("--built cannot be combined with --artifact-root".to_string());
        }
        if use_built_artifacts
            && !matches!(
                mode,
                Mode::Check
                    | Mode::Run
                    | Mode::CliCompletions
                    | Mode::EmitCliCompletions
                    | Mode::WhyRebuild
            )
        {
            return Err(
                "--built is only supported with `check`, `run`, `cli-completions`, `emit-cli-completions`, or `why-rebuild`"
                    .to_string(),
            );
        }
        if output_format == OutputFormat::Json
            && !matches!(
                mode,
                Mode::Doctor
                    | Mode::Syntax
                    | Mode::Check
                    | Mode::Run
                    | Mode::RunAppBundle
                    | Mode::Test
                    | Mode::CliCompletions
                    | Mode::EmitCliCompletions
                    | Mode::EmitAppCompletions
                    | Mode::Metadata
                    | Mode::Schema
                    | Mode::Workspace
                    | Mode::WhyRebuild
                    | Mode::ApiDiff
                    | Mode::EmitAppBundle
                    | Mode::InstallApp
                    | Mode::UninstallApp
                    | Mode::EmitPackageArchive
                    | Mode::EmitAppArchive
                    | Mode::VerifyAppArchive
                    | Mode::VerifyPackageArchive
                    | Mode::UnpackAppArchive
                    | Mode::UnpackPackageArchive
                    | Mode::ListInstalledApps
                    | Mode::Completions
                    | Mode::Definition
                    | Mode::References
                    | Mode::Hover
                    | Mode::EmitInterface
                    | Mode::EmitCheckCache
                    | Mode::EmitArtifacts
                    | Mode::Build
                    | Mode::New
            )
        {
            return Err(
                "--format json is currently supported only with `doctor`, `syntax`, `check`, `run`, `run-app-bundle`, `test`, `cli-completions`, `emit-cli-completions`, `emit-app-completions`, `metadata`, `schema`, `workspace`, `why-rebuild`, `api-diff`, `emit-app-bundle`, `install-app`, `uninstall-app`, `emit-package-archive`, `emit-app-archive`, `verify-app-archive`, `verify-package-archive`, `unpack-app-archive`, `unpack-package-archive`, `list-installed-apps`, `completions`, `definition`, `references`, `hover`, `emit-interface`, `emit-check-cache`, `emit-artifacts`, `build`, or `new --list-templates`"
                    .to_string(),
            );
        }
        if mode == Mode::New && output_format == OutputFormat::Json && !list_new_templates {
            return Err("--format json is only supported with `new --list-templates`".to_string());
        }
        if mode == Mode::Syntax && output_format != OutputFormat::Json {
            return Err("syntax requires --format json".to_string());
        }
        if mode == Mode::Metadata && output_format != OutputFormat::Json {
            return Err("metadata requires --format json".to_string());
        }
        if mode == Mode::Schema && output_format != OutputFormat::Json {
            return Err("schema requires --format json".to_string());
        }
        if mode == Mode::Workspace && output_format != OutputFormat::Json {
            return Err("workspace requires --format json".to_string());
        }
        if mode == Mode::Completions && output_format != OutputFormat::Json {
            return Err("completions requires --format json".to_string());
        }
        if mode == Mode::Definition && output_format != OutputFormat::Json {
            return Err("definition requires --format json".to_string());
        }
        if mode == Mode::References && output_format != OutputFormat::Json {
            return Err("references requires --format json".to_string());
        }
        if mode == Mode::Hover && output_format != OutputFormat::Json {
            return Err("hover requires --format json".to_string());
        }
        if format_check && mode != Mode::Fmt {
            return Err("--check is only supported with `fmt`".to_string());
        }
        if lint_fix && mode != Mode::Lint {
            return Err("--fix is only supported with `lint`".to_string());
        }
        if source_free_app_bundle && mode != Mode::EmitAppBundle {
            return Err("--source-free is only supported with `emit-app-bundle`".to_string());
        }
        if replace_owned_install && mode != Mode::InstallApp {
            return Err("--replace-owned is only supported with `install-app`".to_string());
        }
        if new_template_was_set && mode != Mode::New {
            return Err("--template is only supported with `new`".to_string());
        }
        if list_new_templates && mode != Mode::New {
            return Err("--list-templates is only supported with `new`".to_string());
        }
        if list_new_templates && new_template_was_set {
            return Err("--list-templates cannot be combined with --template".to_string());
        }
        if matches!(
            mode,
            Mode::EmitInterface | Mode::EmitCheckCache | Mode::EmitArtifacts
        ) && artifact_root.is_none()
        {
            return Err(format!("{} requires --artifact-root", mode.name()));
        }
        if mode == Mode::EmitPackageArchive && archive_root.is_none() {
            return Err("emit-package-archive requires --archive-root".to_string());
        }
        if mode == Mode::EmitAppArchive && archive_root.is_none() {
            return Err("emit-app-archive requires --archive-root".to_string());
        }
        if !packages.is_empty()
            && !matches!(
                mode,
                Mode::EmitInterface
                    | Mode::Schema
                    | Mode::ApiDiff
                    | Mode::CliCompletions
                    | Mode::EmitAppCompletions
                    | Mode::EmitCliCompletions
            )
        {
            return Err(
                "--package is only supported with `emit-interface`, `schema`, `api-diff`, `cli-completions`, `emit-cli-completions`, or `emit-app-completions`"
                    .to_string(),
            );
        }
        if matches!(
            mode,
            Mode::Schema
                | Mode::ApiDiff
                | Mode::CliCompletions
                | Mode::EmitCliCompletions
                | Mode::EmitAppCompletions
        ) && packages.len() > 1
        {
            return Err(format!("{} accepts at most one --package", mode.name()));
        }
        if mode == Mode::ApiDiff && packages.is_empty() {
            return Err("api-diff requires --package".to_string());
        }
        if schema_type.is_some()
            && !matches!(
                mode,
                Mode::Schema
                    | Mode::CliCompletions
                    | Mode::EmitCliCompletions
                    | Mode::EmitAppCompletions
            )
        {
            return Err(
                "--type is only supported with `schema`, `cli-completions`, `emit-cli-completions`, or `emit-app-completions`"
                    .to_string(),
            );
        }
        if completion_program.is_some()
            && !matches!(
                mode,
                Mode::CliCompletions
                    | Mode::EmitCliCompletions
                    | Mode::EmitAppCompletions
                    | Mode::EmitAppBundle
                    | Mode::InstallApp
                    | Mode::UninstallApp
                    | Mode::EmitAppArchive
            )
        {
            return Err(
                "--program is only supported with `cli-completions`, `emit-cli-completions`, `emit-app-completions`, `emit-app-bundle`, `install-app`, `uninstall-app`, or `emit-app-archive`"
                    .to_string(),
            );
        }
        if mode == Mode::CliCompletions {
            match output_format {
                OutputFormat::Text => match cli_completion_positionals.as_slice() {
                    [] => return Err("missing shell".to_string()),
                    [shell] => {
                        if !is_supported_shell_completion_shell(shell) {
                            return Err(format!(
                                "unknown shell `{shell}`; expected `bash`, `zsh`, or `fish`"
                            ));
                        }
                        return Err("missing source file".to_string());
                    }
                    [shell, entry] => {
                        if !is_supported_shell_completion_shell(shell) {
                            return Err(format!(
                                "unknown shell `{shell}`; expected `bash`, `zsh`, or `fish`"
                            ));
                        }
                        completion_shell = Some(shell.clone());
                        path = Some(entry.clone());
                    }
                    _ => return Err("too many source files".to_string()),
                },
                OutputFormat::Json => match cli_completion_positionals.as_slice() {
                    [] => return Err("missing source file".to_string()),
                    [entry] => {
                        path = Some(entry.clone());
                    }
                    [shell, _] if is_supported_shell_completion_shell(shell) => {
                        return Err(
                            "cli-completions --format json does not accept a shell".to_string()
                        );
                    }
                    _ => return Err("too many source files".to_string()),
                },
            }
            if completion_program.is_none() {
                return Err("cli-completions requires --program".to_string());
            }
            if schema_type.is_none() {
                return Err("cli-completions requires --type".to_string());
            }
        }
        if mode == Mode::EmitCliCompletions {
            if completion_program.is_none() {
                return Err("emit-cli-completions requires --program".to_string());
            }
            if schema_type.is_none() {
                return Err("emit-cli-completions requires --type".to_string());
            }
            if output_dir.is_none() {
                return Err("emit-cli-completions requires --output-dir".to_string());
            }
        }
        if mode == Mode::EmitAppCompletions {
            if schema_type.is_none() {
                return Err("emit-app-completions requires --type".to_string());
            }
            if output_dir.is_none() {
                return Err("emit-app-completions requires --output-dir".to_string());
            }
        }
        if mode == Mode::EmitAppBundle && output_dir.is_none() {
            return Err("emit-app-bundle requires --output-dir".to_string());
        }
        if mode == Mode::InstallApp && output_dir.is_none() {
            return Err("install-app requires --output-dir".to_string());
        }
        if mode == Mode::UninstallApp {
            if output_dir.is_none() {
                return Err("uninstall-app requires --output-dir".to_string());
            }
            if completion_program.is_none() {
                return Err("uninstall-app requires --program".to_string());
            }
        }
        if mode == Mode::ListInstalledApps && output_dir.is_none() {
            return Err("list-installed-apps requires --output-dir".to_string());
        }
        if mode == Mode::UnpackAppArchive && output_dir.is_none() {
            return Err("unpack-app-archive requires --output-dir".to_string());
        }
        if mode == Mode::UnpackPackageArchive && output_dir.is_none() {
            return Err("unpack-package-archive requires --output-dir".to_string());
        }
        if schema_decode_mode_was_set && mode != Mode::Schema {
            return Err("--decode-mode is only supported with `schema`".to_string());
        }
        if (position_line.is_some() || position_column.is_some())
            && !matches!(mode, Mode::Definition | Mode::References | Mode::Hover)
        {
            return Err(
                "--line and --column are only supported with `definition`, `references`, or `hover`"
                    .to_string(),
            );
        }
        let position = if matches!(mode, Mode::Definition | Mode::References | Mode::Hover) {
            let missing_line = match mode {
                Mode::Definition => "definition requires --line",
                Mode::References => "references requires --line",
                Mode::Hover => "hover requires --line",
                _ => unreachable!("position is only parsed for definition, references, and hover"),
            };
            let missing_column = match mode {
                Mode::Definition => "definition requires --column",
                Mode::References => "references requires --column",
                Mode::Hover => "hover requires --column",
                _ => unreachable!("position is only parsed for definition, references, and hover"),
            };
            Some(SourcePositionArg {
                line: position_line.ok_or_else(|| missing_line.to_string())?,
                column: position_column.ok_or_else(|| missing_column.to_string())?,
            })
        } else {
            None
        };
        if dependency_snippet && mode != Mode::EmitPackageArchive {
            return Err(
                "--dependency-snippet is only supported with `emit-package-archive`".to_string(),
            );
        }
        if saw_program_args_separator && !matches!(mode, Mode::Run | Mode::RunAppBundle) {
            return Err(
                "program arguments after `--` are only supported with `run` or `run-app-bundle`"
                    .to_string(),
            );
        }

        let path = match path {
            Some(path) => {
                if mode == Mode::Doctor {
                    return Err("doctor does not accept a source file".to_string());
                }
                if mode == Mode::New && list_new_templates {
                    return Err("--list-templates does not accept a project directory".to_string());
                }
                if mode == Mode::ListInstalledApps {
                    return Err("list-installed-apps does not accept a source file".to_string());
                }
                path
            }
            None if mode == Mode::Doctor => String::new(),
            None if mode == Mode::New && list_new_templates => String::new(),
            None if mode == Mode::UninstallApp => String::new(),
            None if mode == Mode::ListInstalledApps => String::new(),
            None if mode == Mode::ApiDiff => String::new(),
            None => {
                return Err(if mode == Mode::New {
                    "missing project directory".to_string()
                } else if mode == Mode::Explain {
                    "missing diagnostic code".to_string()
                } else if mode == Mode::ShellCompletions
                    || (mode == Mode::CliCompletions && completion_shell.is_none())
                {
                    "missing shell".to_string()
                } else if matches!(
                    mode,
                    Mode::EmitAppCompletions
                        | Mode::InstallApp
                        | Mode::EmitAppArchive
                        | Mode::RunAppBundle
                ) {
                    "missing app bundle directory".to_string()
                } else if matches!(mode, Mode::UnpackAppArchive | Mode::VerifyAppArchive) {
                    "missing app archive file".to_string()
                } else if matches!(
                    mode,
                    Mode::VerifyPackageArchive | Mode::UnpackPackageArchive
                ) {
                    "missing package archive file".to_string()
                } else {
                    "missing source file".to_string()
                });
            }
        };
        if mode == Mode::ShellCompletions && !is_supported_shell_completion_shell(&path) {
            return Err(format!(
                "unknown shell `{path}`; expected `bash`, `zsh`, or `fish`"
            ));
        }
        if mode == Mode::ApiDiff && !path.is_empty() {
            return Err("api-diff does not accept a source file".to_string());
        }

        Ok(Self {
            mode,
            path,
            artifact_root,
            old_artifact_root,
            new_artifact_root,
            api_diff_fail_on,
            expected_archive_hash,
            archive_root,
            output_dir,
            use_built_artifacts,
            output_format,
            format_check,
            lint_fix,
            source_free_app_bundle,
            replace_owned_install,
            new_template,
            list_new_templates,
            dependency_snippet,
            packages,
            schema_type,
            schema_decode_mode,
            completion_shell,
            completion_program,
            position,
            program_args,
        })
    }
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Mode::Help => "help",
            Mode::Version => "version",
            Mode::Explain => "explain",
            Mode::Doctor => "doctor",
            Mode::ShellCompletions => "shell-completions",
            Mode::CliCompletions => "cli-completions",
            Mode::EmitCliCompletions => "emit-cli-completions",
            Mode::EmitAppCompletions => "emit-app-completions",
            Mode::EmitAppBundle => "emit-app-bundle",
            Mode::InstallApp => "install-app",
            Mode::UninstallApp => "uninstall-app",
            Mode::ListInstalledApps => "list-installed-apps",
            Mode::EmitAppArchive => "emit-app-archive",
            Mode::UnpackAppArchive => "unpack-app-archive",
            Mode::VerifyAppArchive => "verify-app-archive",
            Mode::VerifyPackageArchive => "verify-package-archive",
            Mode::UnpackPackageArchive => "unpack-package-archive",
            Mode::Syntax => "syntax",
            Mode::Check => "check",
            Mode::Lint => "lint",
            Mode::Run => "run",
            Mode::RunAppBundle => "run-app-bundle",
            Mode::Test => "test",
            Mode::Fmt => "fmt",
            Mode::Doc => "doc",
            Mode::Metadata => "metadata",
            Mode::Schema => "schema",
            Mode::Workspace => "workspace",
            Mode::WhyRebuild => "why-rebuild",
            Mode::ApiDiff => "api-diff",
            Mode::Completions => "completions",
            Mode::Definition => "definition",
            Mode::References => "references",
            Mode::Hover => "hover",
            Mode::New => "new",
            Mode::EmitInterface => "emit-interface",
            Mode::EmitCheckCache => "emit-check-cache",
            Mode::EmitArtifacts => "emit-artifacts",
            Mode::EmitPackageArchive => "emit-package-archive",
            Mode::Build => "build",
        }
    }
}

fn usage() -> &'static str {
    concat!(
        "usage: muga --help\n",
        "       muga help\n",
        "       muga --version\n",
        "       muga version\n",
        "       muga explain <diagnostic-code>\n",
        "       muga doctor [--format text|json]\n",
        "       muga shell-completions <bash|zsh|fish>\n",
        "       muga cli-completions <bash|zsh|fish> --program <name> --type <type> [--package <package>] [--artifact-root <dir>|--built] <source-file>\n",
        "       muga cli-completions --format json --program <name> --type <type> [--package <package>] [--artifact-root <dir>|--built] <source-file>\n",
        "       muga emit-cli-completions [--format text|json] --output-dir <dir> --program <name> --type <type> [--package <package>] [--artifact-root <dir>|--built] <source-file>\n",
        "       muga emit-app-completions [--format text|json] --output-dir <dir> [--program <name>] --type <type> [--package <package>] <bundle-dir>\n",
        "       muga syntax --format json <source-file>\n",
        "       muga check [--format text|json] [--artifact-root <dir>|--built] <source-file>\n",
        "       muga lint [--fix] <source-file>\n",
        "       muga run [--format text|json] [--artifact-root <dir>|--built] <source-file> [-- <program-arg>...]\n",
        "       muga run-app-bundle [--format text|json] <bundle-dir> [-- <program-arg>...]\n",
        "       muga test [--format text|json] <source-file>\n",
        "       muga fmt [--check] <source-file>\n",
        "       muga doc <source-file>\n",
        "       muga metadata --format json <source-file>\n",
        "       muga schema --format json [--package <package>] [--type <type>] [--decode-mode required|overlay] [--artifact-root <dir>] <source-file>\n",
        "       muga workspace --format json <source-file>\n",
        "       muga why-rebuild [--format text|json] [--artifact-root <dir>|--built] <source-file>\n",
        "       muga api-diff [--format text|json] [--fail-on unknown|breaking|source-compatible] --old-artifact-root <dir> --new-artifact-root <dir> --package <package>\n",
        "       muga verify-app-archive [--format text|json] [--expected-hash <sha256>] <archive-file>\n",
        "       muga verify-package-archive [--format text|json] [--expected-hash <sha256>] <archive-file>\n",
        "       muga completions --format json <source-file>\n",
        "       muga definition --format json --line <line> --column <column> <source-file>\n",
        "       muga references --format json --line <line> --column <column> <source-file>\n",
        "       muga hover --format json --line <line> --column <column> <source-file>\n",
        "       muga new --list-templates [--format json]\n",
        "       muga new [--template app|lib|test|config-app|cli-tool|report-app|resource-export|package-app] <project-dir>\n",
        "       muga [--format text|json] <source-file> [-- <program-arg>...]\n",
        "       muga build [--format text|json] <source-file>\n",
        "       muga emit-app-bundle [--format text|json] [--source-free] --output-dir <dir> [--program <name>] <source-file>\n",
        "       muga install-app [--format text|json] [--replace-owned] --output-dir <bin-dir> [--program <name>] <bundle-dir>\n",
        "       muga list-installed-apps [--format text|json] --output-dir <bin-dir>\n",
        "       muga uninstall-app [--format text|json] --output-dir <bin-dir> --program <name>\n",
        "       muga emit-app-archive [--format text|json] --archive-root <dir> [--program <name>] <bundle-dir>\n",
        "       muga unpack-app-archive [--format text|json] [--expected-hash <sha256>] --output-dir <dir> <archive-file>\n",
        "       muga emit-package-archive [--format text|json] --archive-root <dir> [--dependency-snippet] <source-file>\n",
        "       muga unpack-package-archive [--format text|json] [--expected-hash <sha256>] --output-dir <dir> <archive-file>\n",
        "       muga emit-artifacts [--format text|json] --artifact-root <dir> <source-file>\n",
        "       muga emit-interface [--format text|json] --artifact-root <dir> [--package <package>] <source-file>\n",
        "       muga emit-check-cache [--format text|json] --artifact-root <dir> <source-file>",
    )
}

fn normalized_help_topic(topic: &str) -> &str {
    match topic {
        "--help" | "-h" => "help",
        "--version" | "-V" => "version",
        _ => topic,
    }
}

fn command_usage(topic: &str) -> Option<String> {
    let topic = normalized_help_topic(topic);
    let mut output = String::new();
    for line in usage().lines() {
        let invocation = line
            .trim_start()
            .strip_prefix("usage: ")
            .unwrap_or_else(|| line.trim_start());
        if usage_invocation_topic(invocation) == Some(topic) {
            if output.is_empty() {
                output.push_str("usage: ");
            } else {
                output.push_str("       ");
            }
            output.push_str(invocation);
            output.push('\n');
        }
    }
    if output.is_empty() {
        None
    } else {
        Some(output)
    }
}

fn usage_invocation_topic(invocation: &str) -> Option<&str> {
    let command = invocation
        .strip_prefix("muga ")?
        .split_whitespace()
        .next()?;
    Some(normalized_help_topic(command))
}

fn diagnostic_explanation(code: &str) -> Option<String> {
    let code = code.trim().to_ascii_uppercase();
    if code.is_empty() {
        return None;
    }
    if let Some(entry) = diagnostic_catalog_entry(&code) {
        return Some(entry);
    }

    let prefix = diagnostic_code_prefix(&code)?;
    let area = diagnostic_family_area(prefix)?;
    let mut output = String::new();
    writeln!(&mut output, "{prefix}: {area}")
        .expect("writing diagnostic explanation should not fail");
    writeln!(&mut output).expect("writing diagnostic explanation should not fail");
    writeln!(&mut output, "Source: errors.md")
        .expect("writing diagnostic explanation should not fail");
    writeln!(&mut output).expect("writing diagnostic explanation should not fail");
    writeln!(
        &mut output,
        "No exact catalog entry is documented for `{code}` yet."
    )
    .expect("writing diagnostic explanation should not fail");
    writeln!(
        &mut output,
        "Use the diagnostic message, related notes, suggestions, and `errors.md` guidance for the actionable fix."
    )
    .expect("writing diagnostic explanation should not fail");
    Some(output)
}

fn diagnostic_catalog_entry(code: &str) -> Option<String> {
    let heading_prefix = format!("## {code}: ");
    let mut lines = ERROR_CATALOG.lines();
    while let Some(line) = lines.next() {
        let Some(title) = line.strip_prefix(&heading_prefix) else {
            continue;
        };
        let mut output = String::new();
        writeln!(&mut output, "{code}: {title}")
            .expect("writing diagnostic explanation should not fail");
        writeln!(&mut output).expect("writing diagnostic explanation should not fail");
        writeln!(&mut output, "Source: errors.md")
            .expect("writing diagnostic explanation should not fail");

        let mut body_started = false;
        for body_line in lines {
            if body_line.starts_with("## ") {
                break;
            }
            if !body_started && body_line.is_empty() {
                continue;
            }
            if !body_started {
                writeln!(&mut output).expect("writing diagnostic explanation should not fail");
                body_started = true;
            }
            writeln!(&mut output, "{body_line}")
                .expect("writing diagnostic explanation should not fail");
        }
        return Some(output);
    }
    None
}

fn diagnostic_code_prefix(code: &str) -> Option<&str> {
    let len = code
        .bytes()
        .take_while(|byte| byte.is_ascii_uppercase())
        .count();
    if len == 0 { None } else { Some(&code[..len]) }
}

fn diagnostic_family_area(prefix: &str) -> Option<&'static str> {
    for line in ERROR_CATALOG.lines() {
        let Some(rest) = line.trim().strip_prefix("| `") else {
            continue;
        };
        let Some((candidate, rest)) = rest.split_once("` | ") else {
            continue;
        };
        if candidate != prefix {
            continue;
        }
        let Some((area, _)) = rest.split_once(" |") else {
            continue;
        };
        return Some(area);
    }
    None
}

fn parse_new_template(value: &str) -> Result<muga::ProjectTemplate, String> {
    match value {
        "app" => Ok(muga::ProjectTemplate::App),
        "lib" | "library" => Ok(muga::ProjectTemplate::Lib),
        "test" | "tests" => Ok(muga::ProjectTemplate::Test),
        "config-app" | "config_app" | "config" => Ok(muga::ProjectTemplate::ConfigApp),
        "cli-tool" | "cli_tool" | "cli" => Ok(muga::ProjectTemplate::CliTool),
        "report-app" | "report_app" | "report" => Ok(muga::ProjectTemplate::ReportApp),
        "resource-export" | "resource_export" | "resource" => {
            Ok(muga::ProjectTemplate::ResourceExport)
        }
        "package-app" | "package_app" | "local-dependency" | "local_dependency" | "local" => {
            Ok(muga::ProjectTemplate::PackageApp)
        }
        _ => Err(format!(
            "unknown project template `{value}`; expected `app`, `lib`, `test`, `config-app`, `cli-tool`, `report-app`, `resource-export`, or `package-app`"
        )),
    }
}

fn print_project_template_list() {
    for template in muga::project_template_infos() {
        println!("template\t{}\t{}", template.name, template.description);
    }
}

fn project_template_list_json_output() -> String {
    let mut output = String::new();
    output.push_str("{\"schemaVersion\":1,\"command\":\"new\",\"status\":\"ok\",\"templates\":[");
    let mut needs_comma = false;
    for template in muga::project_template_infos() {
        if needs_comma {
            output.push(',');
        } else {
            needs_comma = true;
        }
        output.push_str("{\"name\":");
        push_json_string(&mut output, template.name);
        output.push_str(",\"aliases\":[");
        for (index, alias) in template.aliases.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            push_json_string(&mut output, alias);
        }
        output.push_str("],\"description\":");
        push_json_string(&mut output, template.description);
        output.push('}');
    }
    output.push_str("]}");
    output
}

fn parse_output_format(value: &str) -> Result<OutputFormat, String> {
    match value {
        "text" => Ok(OutputFormat::Text),
        "json" => Ok(OutputFormat::Json),
        _ => Err(format!(
            "unknown output format `{value}`; expected `text` or `json`"
        )),
    }
}

fn parse_api_diff_fail_on(value: &str) -> Result<ApiDiffFailOn, String> {
    match value {
        "unknown" => Ok(ApiDiffFailOn::Unknown),
        "breaking" => Ok(ApiDiffFailOn::Breaking),
        "source-compatible" | "sourceCompatible" => Ok(ApiDiffFailOn::SourceCompatible),
        _ => Err(format!(
            "unknown api-diff --fail-on value `{value}`; expected `unknown`, `breaking`, or `source-compatible`"
        )),
    }
}

fn parse_schema_decode_mode(value: &str) -> Result<muga::SchemaDecodeMode, String> {
    match value {
        "required" => Ok(muga::SchemaDecodeMode::Required),
        "overlay" => Ok(muga::SchemaDecodeMode::Overlay),
        _ => Err(format!(
            "unknown schema decode mode `{value}`; expected `required` or `overlay`"
        )),
    }
}

fn parse_position_value(option: &str, value: &str) -> Result<usize, String> {
    let position = value
        .parse::<usize>()
        .map_err(|_| format!("{option} must be a positive integer"))?;
    if position == 0 {
        return Err(format!("{option} must be a positive integer"));
    }
    Ok(position)
}

const SHELL_COMPLETION_COMMANDS: &[&str] = &[
    "help",
    "version",
    "explain",
    "doctor",
    "shell-completions",
    "cli-completions",
    "emit-cli-completions",
    "emit-app-completions",
    "emit-app-bundle",
    "install-app",
    "uninstall-app",
    "list-installed-apps",
    "emit-app-archive",
    "unpack-app-archive",
    "verify-app-archive",
    "verify-package-archive",
    "unpack-package-archive",
    "syntax",
    "check",
    "run",
    "run-app-bundle",
    "test",
    "fmt",
    "doc",
    "metadata",
    "schema",
    "workspace",
    "why-rebuild",
    "completions",
    "definition",
    "references",
    "hover",
    "new",
    "build",
    "emit-package-archive",
    "emit-artifacts",
    "emit-interface",
    "emit-check-cache",
];

const SHELL_COMPLETION_OPTIONS: &[&str] = &[
    "--help",
    "-h",
    "--version",
    "-V",
    "--format",
    "--artifact-root",
    "--built",
    "--archive-root",
    "--expected-hash",
    "--output-dir",
    "--dependency-snippet",
    "--package",
    "--type",
    "--program",
    "--decode-mode",
    "--line",
    "--column",
    "--template",
    "--list-templates",
    "--check",
    "--replace-owned",
];

#[derive(Clone, Debug, PartialEq, Eq)]
struct DoctorReport {
    checks: Vec<DoctorCheck>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DoctorCheck {
    name: &'static str,
    status: DoctorCheckStatus,
    message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DoctorCheckStatus {
    Ok,
    Warn,
}

impl DoctorReport {
    fn status(&self) -> &'static str {
        if self
            .checks
            .iter()
            .any(|check| check.status == DoctorCheckStatus::Warn)
        {
            "warn"
        } else {
            "ok"
        }
    }
}

impl DoctorCheckStatus {
    fn as_str(self) -> &'static str {
        match self {
            DoctorCheckStatus::Ok => "ok",
            DoctorCheckStatus::Warn => "warn",
        }
    }
}

fn doctor_report() -> DoctorReport {
    let mut checks = Vec::new();
    checks.push(doctor_ok(
        "version",
        format!("muga {}", env!("CARGO_PKG_VERSION")),
    ));
    checks.push(match env::current_exe() {
        Ok(path) => doctor_path_check("executable", &path, "current executable"),
        Err(error) => doctor_warn(
            "executable",
            format!("current executable unavailable: {error}"),
        ),
    });
    checks.push(match env::current_dir() {
        Ok(path) => doctor_directory_check("cwd", &path, "current directory"),
        Err(error) => doctor_warn("cwd", format!("current directory unavailable: {error}")),
    });
    checks.push(match home_directory() {
        Some(path) => doctor_directory_check("home", &path, "home directory"),
        None => doctor_warn("home", "HOME is not set".to_string()),
    });
    checks.push(doctor_directory_check(
        "temp",
        &env::temp_dir(),
        "temporary directory",
    ));
    checks.push(doctor_path_env_check());
    DoctorReport { checks }
}

fn doctor_path_check(name: &'static str, path: &Path, label: &str) -> DoctorCheck {
    match fs::metadata(path) {
        Ok(_) => doctor_ok(name, format!("{label}: {}", path.display())),
        Err(error) => doctor_warn(name, format!("{label} is not readable: {error}")),
    }
}

fn doctor_directory_check(name: &'static str, path: &Path, label: &str) -> DoctorCheck {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => {
            doctor_ok(name, format!("{label}: {}", path.display()))
        }
        Ok(_) => doctor_warn(
            name,
            format!("{label} is not a directory: {}", path.display()),
        ),
        Err(error) => doctor_warn(name, format!("{label} is not readable: {error}")),
    }
}

fn doctor_path_env_check() -> DoctorCheck {
    let Some(paths) = env::var_os("PATH") else {
        return doctor_warn("path", "PATH is not set".to_string());
    };
    let count = env::split_paths(&paths)
        .filter(|path| !path.as_os_str().is_empty())
        .count();
    if count == 0 {
        doctor_warn("path", "PATH has no entries".to_string())
    } else {
        doctor_ok("path", format!("PATH has {count} entries"))
    }
}

fn home_directory() -> Option<PathBuf> {
    env::var_os("HOME")
        .filter(|path| !path.as_os_str().is_empty())
        .or_else(|| env::var_os("USERPROFILE").filter(|path| !path.as_os_str().is_empty()))
        .map(PathBuf::from)
}

fn doctor_ok(name: &'static str, message: String) -> DoctorCheck {
    DoctorCheck {
        name,
        status: DoctorCheckStatus::Ok,
        message,
    }
}

fn doctor_warn(name: &'static str, message: String) -> DoctorCheck {
    DoctorCheck {
        name,
        status: DoctorCheckStatus::Warn,
        message,
    }
}

fn doctor_text_output(report: &DoctorReport) -> String {
    let mut output = String::new();
    writeln!(&mut output, "doctor\tstatus\t{}", report.status())
        .expect("writing doctor text to String should not fail");
    for check in &report.checks {
        writeln!(
            &mut output,
            "{}\t{}\t{}",
            check.status.as_str(),
            check.name,
            sanitize_text_field(&check.message)
        )
        .expect("writing doctor text to String should not fail");
    }
    output
}

fn doctor_json_output(report: &DoctorReport) -> String {
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"doctor\",\"status\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing doctor JSON to String should not fail");
    push_json_string(&mut output, report.status());
    output.push_str(",\"diagnostics\":[],\"checks\":[");
    for (index, check) in report.checks.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push('{');
        output.push_str("\"name\":");
        push_json_string(&mut output, check.name);
        output.push_str(",\"status\":");
        push_json_string(&mut output, check.status.as_str());
        output.push_str(",\"message\":");
        push_json_string(&mut output, &check.message);
        output.push('}');
    }
    output.push_str("]}");
    output
}

fn is_supported_shell_completion_shell(value: &str) -> bool {
    matches!(value, "bash" | "zsh" | "fish")
}

fn shell_completion_script(shell: &str) -> Option<String> {
    match shell {
        "bash" => Some(bash_shell_completion_script()),
        "zsh" => Some(zsh_shell_completion_script()),
        "fish" => Some(fish_shell_completion_script()),
        _ => None,
    }
}

#[derive(Clone, Debug)]
struct CliCompletionTarget {
    package_path: String,
    type_name: String,
    schema: muga::cli_schema::CliSchema,
    symbols: muga::symbol::SymbolTable,
}

#[derive(Clone, Debug)]
struct CompletionOption {
    longs: Vec<String>,
    short: Option<String>,
    help: Option<String>,
    takes_value: bool,
    value_source: Option<muga::cli_schema::CliValueSource>,
    values: Vec<String>,
}

#[derive(Clone, Debug)]
struct CompletionCommandCandidate {
    token: String,
    about: Option<String>,
}

#[derive(Clone, Debug)]
struct CompletionScope {
    key: String,
    path: Vec<Vec<String>>,
    options: Vec<CompletionOption>,
    commands: Vec<CompletionCommandCandidate>,
}

#[derive(Clone, Debug)]
struct CompletionCommandTransition {
    from_scope: String,
    token: String,
    to_scope: String,
}

#[derive(Clone, Debug)]
struct CliCompletionModel {
    program: String,
    root: CompletionScope,
    command_scopes: Vec<CompletionScope>,
    command_transitions: Vec<CompletionCommandTransition>,
}

#[derive(Clone, Debug)]
struct CliCompletionPackageOutput {
    program: String,
    target_package: String,
    target_type: String,
    files: Vec<PathBuf>,
}

fn cli_completion_output(cli: &Cli) -> Result<String, Vec<muga::diagnostic::Diagnostic>> {
    match cli.output_format {
        OutputFormat::Text => cli_completion_script(cli),
        OutputFormat::Json => cli_completion_json(cli),
    }
}

fn cli_completion_model_for_cli(
    cli: &Cli,
) -> Result<(CliCompletionTarget, CliCompletionModel), Vec<muga::diagnostic::Diagnostic>> {
    let check = if cli.use_built_artifacts {
        muga::check_package_aware_path_against_default_build_artifacts(Path::new(&cli.path))
    } else if let Some(artifact_root) = &cli.artifact_root {
        muga::check_package_aware_path_against_cached_artifact_root(
            Path::new(&cli.path),
            Path::new(artifact_root),
        )
    } else {
        muga::check_package_aware_path(Path::new(&cli.path))
    }?;
    let target = cli_completion_schema_for_check(
        &check,
        cli.packages.first().map(String::as_str),
        cli.schema_type
            .as_deref()
            .expect("validated CLI completion type"),
    )?;
    let model = cli_completion_model(
        cli.completion_program
            .as_deref()
            .expect("validated CLI completion program"),
        &target.schema,
        &target.symbols,
    );
    Ok((target, model))
}

fn cli_completion_model_for_app_bundle(
    cli: &Cli,
    program: &str,
) -> Result<(CliCompletionTarget, CliCompletionModel), Vec<muga::diagnostic::Diagnostic>> {
    let bundle = muga::read_app_bundle_interfaces(Path::new(&cli.path))?;
    let signatures = muga::package_signature::PackageSignatureEnvironment::from_interfaces(
        &bundle.interfaces,
        bundle.symbols,
    );
    let target = cli_completion_schema_for_signatures(
        "emit-app-completions",
        &signatures,
        &bundle.entry_package,
        cli.packages.first().map(String::as_str),
        cli.schema_type
            .as_deref()
            .expect("validated app completion type"),
    )?;
    let model = cli_completion_model(program, &target.schema, &target.symbols);
    Ok((target, model))
}

fn cli_completion_script(cli: &Cli) -> Result<String, Vec<muga::diagnostic::Diagnostic>> {
    let (_target, model) = cli_completion_model_for_cli(cli)?;
    match cli
        .completion_shell
        .as_deref()
        .expect("validated CLI completion shell")
    {
        "bash" => Ok(bash_cli_completion_script(&model)),
        "zsh" => Ok(zsh_cli_completion_script(&model)),
        "fish" => Ok(fish_cli_completion_script(&model)),
        _ => unreachable!("validated CLI completion shell"),
    }
}

fn cli_completion_json(cli: &Cli) -> Result<String, Vec<muga::diagnostic::Diagnostic>> {
    let (target, model) = cli_completion_model_for_cli(cli)?;
    Ok(cli_completion_json_output(
        "cli-completions",
        "entry",
        Path::new(&cli.path),
        &target,
        &model,
    ))
}

fn emit_cli_completion_package(
    cli: &Cli,
) -> Result<CliCompletionPackageOutput, Vec<muga::diagnostic::Diagnostic>> {
    let (target, model) = cli_completion_model_for_cli(cli)?;
    let output_dir = Path::new(
        cli.output_dir
            .as_deref()
            .expect("validated completion output directory"),
    );
    fs::create_dir_all(output_dir).map_err(|error| {
        vec![cli_completion_error(format!(
            "`emit-cli-completions` cannot create output directory `{}`: {error}",
            output_dir.display()
        ))]
    })?;

    let files = write_cli_completion_package(
        "emit-cli-completions",
        "cli-completions",
        "entry",
        output_dir,
        Path::new(&cli.path),
        &target,
        &model,
    )?;
    Ok(CliCompletionPackageOutput {
        program: model.program.clone(),
        target_package: target.package_path.clone(),
        target_type: target.type_name.clone(),
        files,
    })
}

fn emit_app_completion_package(
    cli: &Cli,
) -> Result<CliCompletionPackageOutput, Vec<muga::diagnostic::Diagnostic>> {
    let program =
        muga::app_bundle_program(Path::new(&cli.path), cli.completion_program.as_deref())?;
    let (target, model) = cli_completion_model_for_app_bundle(cli, &program)?;
    let output_dir = Path::new(
        cli.output_dir
            .as_deref()
            .expect("validated app completion output directory"),
    );
    fs::create_dir_all(output_dir).map_err(|error| {
        vec![cli_completion_error(format!(
            "`emit-app-completions` cannot create output directory `{}`: {error}",
            output_dir.display()
        ))]
    })?;

    let files = write_cli_completion_package(
        "emit-app-completions",
        "emit-app-completions",
        "bundle",
        output_dir,
        Path::new(&cli.path),
        &target,
        &model,
    )?;
    Ok(CliCompletionPackageOutput {
        program: model.program.clone(),
        target_package: target.package_path.clone(),
        target_type: target.type_name.clone(),
        files,
    })
}

fn write_cli_completion_package(
    command_name: &str,
    json_command: &str,
    input_key: &str,
    output_dir: &Path,
    input_path: &Path,
    target: &CliCompletionTarget,
    model: &CliCompletionModel,
) -> Result<Vec<PathBuf>, Vec<muga::diagnostic::Diagnostic>> {
    let file_stem = cli_completion_package_file_stem(&model.program);
    let json = cli_completion_json_output(json_command, input_key, input_path, target, model);
    let files = [
        (
            format!("{file_stem}.bash"),
            bash_cli_completion_script(model),
        ),
        (format!("_{file_stem}"), zsh_cli_completion_script(model)),
        (
            format!("{file_stem}.fish"),
            fish_cli_completion_script(model),
        ),
        (format!("{file_stem}.completions.json"), json),
    ];

    let mut written = Vec::with_capacity(files.len());
    for (file_name, contents) in files {
        let path = output_dir.join(file_name);
        muga::durable_write::replace_file(&path, contents).map_err(|error| {
            vec![cli_completion_error(format!(
                "`{command_name}` cannot write `{}`: {error}",
                path.display()
            ))]
        })?;
        written.push(path);
    }
    Ok(written)
}

fn cli_completion_package_file_stem(program: &str) -> String {
    let mut stem = String::new();
    let mut last_was_underscore = false;
    for ch in program.chars() {
        let next = if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
            ch
        } else {
            '_'
        };
        if next == '_' {
            if !last_was_underscore {
                stem.push(next);
            }
            last_was_underscore = true;
        } else {
            stem.push(next);
            last_was_underscore = false;
        }
    }
    let stem = stem.trim_matches('_');
    if stem.is_empty() {
        "completion".to_string()
    } else {
        stem.to_string()
    }
}

fn cli_completion_schema_for_check(
    check: &muga::PackageAwareCheck,
    package: Option<&str>,
    type_name: &str,
) -> Result<CliCompletionTarget, Vec<muga::diagnostic::Diagnostic>> {
    let default_package = check
        .packages
        .package_graph
        .package(check.packages.entry_package)
        .map(|package| package.path.as_str())
        .unwrap_or("");
    cli_completion_schema_for_signatures(
        "cli-completions",
        &check.signatures,
        default_package,
        package,
        type_name,
    )
}

fn cli_completion_schema_for_signatures(
    command_name: &str,
    signatures: &muga::package_signature::PackageSignatureEnvironment,
    default_package: &str,
    package: Option<&str>,
    type_name: &str,
) -> Result<CliCompletionTarget, Vec<muga::diagnostic::Diagnostic>> {
    let package_path = package.unwrap_or(default_package);
    let Some(package_id) = signatures
        .package_paths
        .iter()
        .find_map(|(id, path)| (path == package_path).then_some(*id))
    else {
        return Err(vec![cli_completion_error(format!(
            "`{command_name}` cannot find package `{package_path}`"
        ))]);
    };
    let record_item = signatures
        .records
        .iter()
        .find(|record| record.package == package_id && record.name == type_name)
        .map(|record| record.item);
    let enum_item = signatures
        .enums
        .iter()
        .find(|enumeration| enumeration.package == package_id && enumeration.name == type_name)
        .map(|enumeration| enumeration.item);
    let mut symbols = signatures.symbols.clone();
    let mut diagnostics = Vec::new();
    let mut visiting = HashSet::new();
    let schema = match (record_item, enum_item) {
        (Some(record_item), None) => cli_completion_record_schema(
            signatures,
            &mut symbols,
            record_item,
            &mut visiting,
            &mut diagnostics,
        ),
        (None, Some(enum_item)) => cli_completion_command_schema(
            signatures,
            &mut symbols,
            enum_item,
            &mut visiting,
            &mut diagnostics,
        ),
        (Some(_), Some(_)) => {
            diagnostics.push(cli_completion_error(format!(
                "`{command_name}` found both a record and enum named `{type_name}` in `{package_path}`"
            )));
            None
        }
        (None, None) => {
            diagnostics.push(cli_completion_error(format!(
                "`{command_name}` cannot find type `{type_name}` in package `{package_path}`"
            )));
            None
        }
    };
    if diagnostics.is_empty() {
        Ok(CliCompletionTarget {
            package_path: package_path.to_string(),
            type_name: type_name.to_string(),
            schema: schema.expect("schema exists when diagnostics are empty"),
            symbols,
        })
    } else {
        Err(diagnostics)
    }
}

fn cli_completion_record_schema(
    signatures: &muga::package_signature::PackageSignatureEnvironment,
    symbols: &mut muga::symbol::SymbolTable,
    item: muga::identity::PackageItemId,
    visiting: &mut HashSet<muga::identity::PackageItemId>,
    diagnostics: &mut Vec<muga::diagnostic::Diagnostic>,
) -> Option<muga::cli_schema::CliSchema> {
    let Some(record) = signatures.record(item) else {
        diagnostics.push(cli_completion_error(
            "`cli-completions` cannot inspect requested record",
        ));
        return None;
    };
    if !record.type_params.is_empty() {
        diagnostics.push(cli_completion_error(format!(
            "`cli-completions` supports only concrete non-generic record targets; found `{}`",
            record.name
        )));
        return None;
    }

    let wrapper_subcommand = record.fields.iter().find(|field| field.cli_subcommand);
    let mut fields = Vec::new();
    let mut subcommand = None;
    let mut valid = true;
    for field in &record.fields {
        if field.cli_subcommand {
            let Some(enum_item) =
                cli_completion_enum_item_for_type(signatures, symbols, record.package, &field.ty)
            else {
                diagnostics.push(cli_completion_error(format!(
                    "`cli-completions` wrapper field `{}::{}` must have a concrete command enum type",
                    record.name, field.name
                )));
                valid = false;
                continue;
            };
            let Some(schema) = cli_completion_command_schema(
                signatures,
                symbols,
                enum_item,
                visiting,
                diagnostics,
            ) else {
                valid = false;
                continue;
            };
            subcommand = Some(muga::cli_schema::CliSubcommandSchema {
                field_name: symbols.intern(&field.name),
                schema: Box::new(schema),
            });
            continue;
        }
        if wrapper_subcommand.is_some() && field.cli_position.is_some() {
            diagnostics.push(cli_completion_error(format!(
                "`cli-completions` wrapper record `{}` does not support root positional field `{}` before a subcommand",
                record.name, field.name
            )));
            valid = false;
            continue;
        }
        let Some(value) =
            cli_completion_value_schema_for_type(signatures, symbols, record.package, &field.ty)
        else {
            if !cli_completion_field_has_explicit_metadata(field) {
                continue;
            }
            diagnostics.push(cli_completion_error(format!(
                "`cli-completions` does not support field `{}`; use a String, Int, Bool, Option[String|Int|Bool], List[String|Int|Bool], or zero-payload enum field",
                field.name
            )));
            valid = false;
            continue;
        };
        if field.cli_hidden && !cli_completion_schema_can_synthesize_absent_value(&value) {
            diagnostics.push(cli_completion_error(format!(
                "`cli-completions` hidden field `{}` must be optional, Bool, or a supported List",
                field.name
            )));
            valid = false;
            continue;
        }
        if field.cli_value_source.is_some()
            && !cli_completion_value_source_allowed_for_schema(&value)
        {
            diagnostics.push(cli_completion_error(format!(
                "`cli-completions` value source field `{}` must have a String, Option[String], or List[String] value type",
                field.name
            )));
            valid = false;
            continue;
        }
        fields.push(muga::cli_schema::CliFieldSchema {
            name: symbols.intern(&field.name),
            option_name: symbols.intern(
                field
                    .cli_name
                    .as_deref()
                    .or(field.json_rename.as_deref())
                    .unwrap_or(&field.name),
            ),
            short: field
                .cli_short
                .as_deref()
                .map(|short| symbols.intern(short)),
            position: field.cli_position,
            value_source: field.cli_value_source,
            aliases: field
                .cli_aliases
                .iter()
                .map(|alias| symbols.intern(alias))
                .collect(),
            help: field.cli_help.as_deref().map(|help| symbols.intern(help)),
            hidden: field.cli_hidden,
            validation: field.json_validation.clone(),
            value,
        });
    }
    if !valid {
        return None;
    }
    Some(muga::cli_schema::CliSchema {
        type_name: symbols.intern(&record.name),
        package_item: Some(record.item),
        about: record
            .cli_about
            .as_deref()
            .map(|about| symbols.intern(about)),
        fields,
        commands: Vec::new(),
        subcommand,
    })
}

fn cli_completion_field_has_explicit_metadata(
    field: &muga::package_signature::PackageFieldSignature,
) -> bool {
    field.cli_name.is_some()
        || field.cli_short.is_some()
        || field.cli_position.is_some()
        || field.cli_value_source.is_some()
        || !field.cli_aliases.is_empty()
        || field.cli_help.is_some()
        || field.cli_hidden
        || field.cli_subcommand
}

fn cli_completion_command_schema(
    signatures: &muga::package_signature::PackageSignatureEnvironment,
    symbols: &mut muga::symbol::SymbolTable,
    item: muga::identity::PackageItemId,
    visiting: &mut HashSet<muga::identity::PackageItemId>,
    diagnostics: &mut Vec<muga::diagnostic::Diagnostic>,
) -> Option<muga::cli_schema::CliSchema> {
    let Some(enumeration) = signatures.enumeration(item) else {
        diagnostics.push(cli_completion_error(
            "`cli-completions` cannot inspect requested enum",
        ));
        return None;
    };
    if !enumeration.type_params.is_empty() {
        diagnostics.push(cli_completion_error(format!(
            "`cli-completions` supports only concrete non-generic command enum targets; found `{}`",
            enumeration.name
        )));
        return None;
    }
    if !visiting.insert(item) {
        diagnostics.push(cli_completion_error(format!(
            "`cli-completions` cannot derive a recursive command schema for `{}`",
            enumeration.name
        )));
        return None;
    }
    let mut valid = true;
    if enumeration.variants.is_empty() {
        diagnostics.push(cli_completion_error(format!(
            "`cli-completions` command enum `{}` must have at least one variant",
            enumeration.name
        )));
        valid = false;
    }
    let mut command_names = HashMap::new();
    let mut commands = Vec::new();
    for variant in &enumeration.variants {
        let Some(command_name) = variant.cli_name.as_deref() else {
            diagnostics.push(cli_completion_error(format!(
                "`cli-completions` command variant `{}::{}` requires `@cli(name: \"...\")`",
                enumeration.name, variant.name
            )));
            valid = false;
            continue;
        };
        for name in
            std::iter::once(command_name).chain(variant.cli_aliases.iter().map(String::as_str))
        {
            if command_names
                .insert(name.to_string(), variant.span)
                .is_some()
            {
                diagnostics.push(cli_completion_error(format!(
                    "duplicate CLI command name `{name}` in enum `{}`",
                    enumeration.name
                )));
                valid = false;
            }
        }
        let Some(payload_ty) = variant.payload.as_ref() else {
            diagnostics.push(cli_completion_error(format!(
                "`cli-completions` command variant `{}::{}` must carry a record or command enum payload",
                enumeration.name, variant.name
            )));
            valid = false;
            continue;
        };
        let payload = match (
            cli_completion_record_item_for_type(
                signatures,
                symbols,
                enumeration.package,
                payload_ty,
            ),
            cli_completion_enum_item_for_type(signatures, symbols, enumeration.package, payload_ty),
        ) {
            (Some(record_item), None) => cli_completion_record_schema(
                signatures,
                symbols,
                record_item,
                visiting,
                diagnostics,
            ),
            (None, Some(enum_item)) => {
                cli_completion_command_schema(signatures, symbols, enum_item, visiting, diagnostics)
            }
            _ => {
                diagnostics.push(cli_completion_error(format!(
                    "`cli-completions` command variant `{}::{}` has unsupported payload type",
                    enumeration.name, variant.name
                )));
                valid = false;
                None
            }
        };
        let Some(payload) = payload else {
            valid = false;
            continue;
        };
        commands.push(muga::cli_schema::CliCommandVariantSchema {
            variant_name: symbols.intern(&variant.name),
            command_name: symbols.intern(command_name),
            aliases: variant
                .cli_aliases
                .iter()
                .map(|alias| symbols.intern(alias))
                .collect(),
            about: variant
                .cli_about
                .as_deref()
                .map(|about| symbols.intern(about)),
            hidden: variant.cli_hidden,
            payload: Box::new(payload),
        });
    }
    visiting.remove(&item);
    if !valid {
        return None;
    }
    Some(muga::cli_schema::CliSchema {
        type_name: symbols.intern(&enumeration.name),
        package_item: Some(enumeration.item),
        about: enumeration
            .cli_about
            .as_deref()
            .map(|about| symbols.intern(about)),
        fields: Vec::new(),
        commands,
        subcommand: None,
    })
}

fn cli_completion_record_item_for_type(
    signatures: &muga::package_signature::PackageSignatureEnvironment,
    symbols: &muga::symbol::SymbolTable,
    current_package: muga::identity::PackageId,
    ty: &muga::types::TypeInfo,
) -> Option<muga::identity::PackageItemId> {
    match ty {
        muga::types::TypeInfo::PackageRecord { item, args, .. } if args.is_empty() => Some(*item),
        muga::types::TypeInfo::Record(symbol, args) if args.is_empty() => {
            let name = symbols.resolve(*symbol);
            signatures
                .records
                .iter()
                .find(|record| record.package == current_package && record.name == name)
                .map(|record| record.item)
        }
        _ => None,
    }
}

fn cli_completion_enum_item_for_type(
    signatures: &muga::package_signature::PackageSignatureEnvironment,
    symbols: &muga::symbol::SymbolTable,
    current_package: muga::identity::PackageId,
    ty: &muga::types::TypeInfo,
) -> Option<muga::identity::PackageItemId> {
    match ty {
        muga::types::TypeInfo::PackageEnum { item, args, .. } if args.is_empty() => Some(*item),
        muga::types::TypeInfo::Enum { symbol, args } if args.is_empty() => {
            let name = symbols.resolve(*symbol);
            signatures
                .enums
                .iter()
                .find(|enumeration| {
                    enumeration.package == current_package && enumeration.name == name
                })
                .map(|enumeration| enumeration.item)
        }
        _ => None,
    }
}

fn cli_completion_value_schema_for_type(
    signatures: &muga::package_signature::PackageSignatureEnvironment,
    symbols: &mut muga::symbol::SymbolTable,
    current_package: muga::identity::PackageId,
    ty: &muga::types::TypeInfo,
) -> Option<muga::cli_schema::CliValueSchema> {
    match ty {
        muga::types::TypeInfo::String => Some(muga::cli_schema::CliValueSchema::String),
        muga::types::TypeInfo::Int => Some(muga::cli_schema::CliValueSchema::Int),
        muga::types::TypeInfo::Bool => Some(muga::cli_schema::CliValueSchema::Bool),
        muga::types::TypeInfo::Option(item) => match item.as_ref() {
            muga::types::TypeInfo::String => Some(muga::cli_schema::CliValueSchema::Option(
                Box::new(muga::cli_schema::CliValueSchema::String),
            )),
            muga::types::TypeInfo::Int => Some(muga::cli_schema::CliValueSchema::Option(Box::new(
                muga::cli_schema::CliValueSchema::Int,
            ))),
            muga::types::TypeInfo::Bool => Some(muga::cli_schema::CliValueSchema::Option(
                Box::new(muga::cli_schema::CliValueSchema::Bool),
            )),
            other => cli_completion_enum_value_schema(signatures, symbols, current_package, other)
                .map(|schema| muga::cli_schema::CliValueSchema::Option(Box::new(schema))),
        },
        muga::types::TypeInfo::List(item) => match item.as_ref() {
            muga::types::TypeInfo::String => Some(muga::cli_schema::CliValueSchema::StringList),
            muga::types::TypeInfo::Int => Some(muga::cli_schema::CliValueSchema::IntList),
            muga::types::TypeInfo::Bool => Some(muga::cli_schema::CliValueSchema::BoolList),
            other => {
                let schema =
                    cli_completion_enum_value_schema(signatures, symbols, current_package, other)?;
                if let muga::cli_schema::CliValueSchema::Enum {
                    type_name,
                    package_item,
                    variants,
                } = schema
                {
                    Some(muga::cli_schema::CliValueSchema::EnumList {
                        type_name,
                        package_item,
                        variants,
                    })
                } else {
                    None
                }
            }
        },
        other => cli_completion_enum_value_schema(signatures, symbols, current_package, other),
    }
}

fn cli_completion_enum_value_schema(
    signatures: &muga::package_signature::PackageSignatureEnvironment,
    symbols: &mut muga::symbol::SymbolTable,
    current_package: muga::identity::PackageId,
    ty: &muga::types::TypeInfo,
) -> Option<muga::cli_schema::CliValueSchema> {
    let item = cli_completion_enum_item_for_type(signatures, symbols, current_package, ty)?;
    let enumeration = signatures.enumeration(item)?;
    if !enumeration.type_params.is_empty() {
        return None;
    }
    let mut variants = Vec::new();
    for variant in &enumeration.variants {
        if variant.payload.is_some() {
            return None;
        }
        variants.push(muga::cli_schema::CliEnumVariantSchema {
            name: symbols.intern(&variant.name),
            tag: symbols.intern(variant.json_rename.as_deref().unwrap_or(&variant.name)),
        });
    }
    Some(muga::cli_schema::CliValueSchema::Enum {
        type_name: symbols.intern(&enumeration.name),
        package_item: Some(enumeration.item),
        variants,
    })
}

fn cli_completion_schema_can_synthesize_absent_value(
    schema: &muga::cli_schema::CliValueSchema,
) -> bool {
    matches!(
        schema,
        muga::cli_schema::CliValueSchema::Bool
            | muga::cli_schema::CliValueSchema::Option(_)
            | muga::cli_schema::CliValueSchema::StringList
            | muga::cli_schema::CliValueSchema::IntList
            | muga::cli_schema::CliValueSchema::BoolList
            | muga::cli_schema::CliValueSchema::EnumList { .. }
    )
}

fn cli_completion_value_source_allowed_for_schema(
    schema: &muga::cli_schema::CliValueSchema,
) -> bool {
    match schema {
        muga::cli_schema::CliValueSchema::String | muga::cli_schema::CliValueSchema::StringList => {
            true
        }
        muga::cli_schema::CliValueSchema::Option(item) => {
            matches!(item.as_ref(), muga::cli_schema::CliValueSchema::String)
        }
        _ => false,
    }
}

fn cli_completion_error(message: impl Into<String>) -> muga::diagnostic::Diagnostic {
    muga::diagnostic::Diagnostic::new("T006", message, muga::span::Span::default())
}

fn cli_completion_model(
    program: &str,
    schema: &muga::cli_schema::CliSchema,
    symbols: &muga::symbol::SymbolTable,
) -> CliCompletionModel {
    let root_schema = cli_completion_command_source_schema(schema);
    let root = CompletionScope {
        key: "root".to_string(),
        path: Vec::new(),
        options: cli_completion_options(schema, symbols),
        commands: cli_completion_commands(root_schema, symbols),
    };
    let mut command_scopes = Vec::new();
    let mut command_transitions = Vec::new();
    push_cli_completion_command_scopes(
        root_schema,
        symbols,
        "root",
        &[],
        &mut command_scopes,
        &mut command_transitions,
    );
    CliCompletionModel {
        program: program.to_string(),
        root,
        command_scopes,
        command_transitions,
    }
}

fn cli_completion_command_source_schema(
    schema: &muga::cli_schema::CliSchema,
) -> &muga::cli_schema::CliSchema {
    schema
        .subcommand
        .as_ref()
        .map(|subcommand| subcommand.schema.as_ref())
        .unwrap_or(schema)
}

fn push_cli_completion_command_scopes(
    schema: &muga::cli_schema::CliSchema,
    symbols: &muga::symbol::SymbolTable,
    parent_scope: &str,
    parent_path: &[Vec<String>],
    command_scopes: &mut Vec<CompletionScope>,
    command_transitions: &mut Vec<CompletionCommandTransition>,
) {
    for command in schema.commands.iter().filter(|command| !command.hidden) {
        let tokens = cli_completion_command_tokens(command, symbols);
        let scope_key = format!("s{}", command_scopes.len());
        for token in &tokens {
            command_transitions.push(CompletionCommandTransition {
                from_scope: parent_scope.to_string(),
                token: token.clone(),
                to_scope: scope_key.clone(),
            });
        }
        let mut path = parent_path.to_vec();
        path.push(tokens);
        let command_schema = cli_completion_command_source_schema(&command.payload);
        command_scopes.push(CompletionScope {
            key: scope_key.clone(),
            path: path.clone(),
            options: cli_completion_options(&command.payload, symbols),
            commands: cli_completion_commands(command_schema, symbols),
        });
        push_cli_completion_command_scopes(
            command_schema,
            symbols,
            &scope_key,
            &path,
            command_scopes,
            command_transitions,
        );
    }
}

fn cli_completion_options(
    schema: &muga::cli_schema::CliSchema,
    symbols: &muga::symbol::SymbolTable,
) -> Vec<CompletionOption> {
    schema
        .fields
        .iter()
        .filter(|field| !field.hidden && field.position.is_none())
        .map(|field| {
            let mut longs = Vec::with_capacity(1 + field.aliases.len());
            longs.push(symbols.resolve(field.option_name).to_string());
            longs.extend(
                field
                    .aliases
                    .iter()
                    .map(|alias| symbols.resolve(*alias).to_string()),
            );
            CompletionOption {
                longs,
                short: field.short.map(|short| symbols.resolve(short).to_string()),
                help: field.help.map(|help| symbols.resolve(help).to_string()),
                takes_value: !cli_completion_accepts_bare_bool(&field.value),
                value_source: field.value_source,
                values: cli_completion_value_candidates(&field.value, symbols),
            }
        })
        .collect()
}

fn cli_completion_commands(
    schema: &muga::cli_schema::CliSchema,
    symbols: &muga::symbol::SymbolTable,
) -> Vec<CompletionCommandCandidate> {
    schema
        .commands
        .iter()
        .filter(|command| !command.hidden)
        .flat_map(|command| {
            let about = command
                .about
                .map(|about| symbols.resolve(about).to_string());
            cli_completion_command_tokens(command, symbols)
                .into_iter()
                .map(move |token| CompletionCommandCandidate {
                    token,
                    about: about.clone(),
                })
        })
        .collect()
}

fn cli_completion_command_tokens(
    command: &muga::cli_schema::CliCommandVariantSchema,
    symbols: &muga::symbol::SymbolTable,
) -> Vec<String> {
    let mut tokens = Vec::with_capacity(1 + command.aliases.len());
    tokens.push(symbols.resolve(command.command_name).to_string());
    tokens.extend(
        command
            .aliases
            .iter()
            .map(|alias| symbols.resolve(*alias).to_string()),
    );
    tokens
}

fn cli_completion_accepts_bare_bool(schema: &muga::cli_schema::CliValueSchema) -> bool {
    match schema {
        muga::cli_schema::CliValueSchema::Bool | muga::cli_schema::CliValueSchema::BoolList => true,
        muga::cli_schema::CliValueSchema::Option(item) => {
            matches!(item.as_ref(), muga::cli_schema::CliValueSchema::Bool)
        }
        _ => false,
    }
}

fn cli_completion_value_candidates(
    schema: &muga::cli_schema::CliValueSchema,
    symbols: &muga::symbol::SymbolTable,
) -> Vec<String> {
    match schema {
        muga::cli_schema::CliValueSchema::Bool | muga::cli_schema::CliValueSchema::BoolList => {
            vec!["true".to_string(), "false".to_string()]
        }
        muga::cli_schema::CliValueSchema::Option(item)
            if matches!(item.as_ref(), muga::cli_schema::CliValueSchema::Bool) =>
        {
            vec!["true".to_string(), "false".to_string()]
        }
        muga::cli_schema::CliValueSchema::Enum { variants, .. }
        | muga::cli_schema::CliValueSchema::EnumList { variants, .. } => variants
            .iter()
            .map(|variant| symbols.resolve(variant.tag).to_string())
            .collect(),
        muga::cli_schema::CliValueSchema::Option(item) => {
            cli_completion_value_candidates(item, symbols)
        }
        _ => Vec::new(),
    }
}

fn cli_completion_json_output(
    command: &str,
    input_key: &str,
    input_path: &Path,
    target: &CliCompletionTarget,
    model: &CliCompletionModel,
) -> String {
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing CLI completion JSON to String should not fail");
    push_json_string(&mut output, command);
    output.push(',');
    push_json_string(&mut output, input_key);
    output.push(':');
    output.push_str(&entry_json_object(input_path));
    output.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"program\":");
    push_json_string(&mut output, &model.program);
    output.push_str(",\"target\":{\"package\":");
    push_json_string(&mut output, &target.package_path);
    output.push_str(",\"type\":");
    push_json_string(&mut output, &target.type_name);
    output.push_str("},\"completion\":");
    push_cli_completion_schema_json(&mut output, &target.schema, &target.symbols);
    output.push('}');
    output
}

fn completion_package_json_output(
    command: &str,
    input_key: &str,
    input_path: &Path,
    output_dir: &Path,
    package: &CliCompletionPackageOutput,
) -> String {
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing completion package JSON to String should not fail");
    push_json_string(&mut output, command);
    output.push_str(",\"status\":\"ok\",\"diagnostics\":[],");
    push_json_string(&mut output, input_key);
    output.push(':');
    push_path_ref_json(&mut output, input_path);
    output.push_str(",\"outputDir\":");
    push_path_ref_json(&mut output, output_dir);
    output.push_str(",\"program\":");
    push_json_string(&mut output, &package.program);
    output.push_str(",\"target\":{\"package\":");
    push_json_string(&mut output, &package.target_package);
    output.push_str(",\"type\":");
    push_json_string(&mut output, &package.target_type);
    output.push_str("},\"files\":[");
    for (index, path) in package.files.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_path_ref_json(&mut output, path);
    }
    output.push_str("]}");
    output
}

struct CompletionPackageDiagnosticJsonInput<'a> {
    command: &'a str,
    input_key: &'a str,
    input_path: &'a Path,
    output_dir: &'a Path,
    program: Option<&'a str>,
    package_name: Option<&'a str>,
    type_name: Option<&'a str>,
    status: &'a str,
}

fn completion_package_diagnostic_json_output(
    input: &CompletionPackageDiagnosticJsonInput<'_>,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    let input_context = if input.input_key == "entry" {
        entry_source_diagnostic_context(input.input_path)
    } else {
        artifact_root_diagnostic_context("app-bundle", input.input_path)
    };
    let context = [
        input_context,
        artifact_root_diagnostic_context("completion-output", input.output_dir),
    ];
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing completion package diagnostic JSON to String should not fail");
    push_json_string(&mut output, input.command);
    output.push(',');
    push_json_string(&mut output, input.input_key);
    output.push(':');
    push_path_ref_json(&mut output, input.input_path);
    output.push_str(",\"outputDir\":");
    push_path_ref_json(&mut output, input.output_dir);
    output.push_str(",\"program\":");
    push_optional_json_string(&mut output, input.program);
    output.push_str(",\"target\":{\"package\":");
    push_optional_json_string(&mut output, input.package_name);
    output.push_str(",\"type\":");
    push_optional_json_string(&mut output, input.type_name);
    output.push_str("},\"status\":");
    push_json_string(&mut output, input.status);
    output.push_str(",\"diagnostics\":");
    output.push_str(&muga::diagnostic::diagnostics_json_array_with_context(
        diagnostics,
        &context,
    ));
    output.push('}');
    output
}

fn push_cli_completion_schema_json(
    output: &mut String,
    schema: &muga::cli_schema::CliSchema,
    symbols: &muga::symbol::SymbolTable,
) {
    output.push('{');
    output.push_str("\"kind\":");
    let kind = if schema.subcommand.is_some() {
        "wrapper"
    } else if schema.commands.is_empty() {
        "record"
    } else {
        "command"
    };
    push_json_string(output, kind);
    output.push_str(",\"type\":");
    push_json_string(output, symbols.resolve(schema.type_name));
    output.push_str(",\"about\":");
    push_optional_symbol_json(output, schema.about, symbols);
    output.push_str(",\"options\":");
    push_cli_completion_options_json(output, schema, symbols);
    output.push_str(",\"positionals\":");
    push_cli_completion_positionals_json(output, schema, symbols);
    output.push_str(",\"commands\":");
    push_cli_completion_commands_json(output, schema, symbols);
    output.push_str(",\"subcommand\":");
    if let Some(subcommand) = &schema.subcommand {
        output.push('{');
        output.push_str("\"field\":");
        push_json_string(output, symbols.resolve(subcommand.field_name));
        output.push_str(",\"schema\":");
        push_cli_completion_schema_json(output, &subcommand.schema, symbols);
        output.push('}');
    } else {
        output.push_str("null");
    }
    output.push('}');
}

fn push_cli_completion_options_json(
    output: &mut String,
    schema: &muga::cli_schema::CliSchema,
    symbols: &muga::symbol::SymbolTable,
) {
    output.push('[');
    let mut first = true;
    for field in schema
        .fields
        .iter()
        .filter(|field| !field.hidden && field.position.is_none())
    {
        if !first {
            output.push(',');
        }
        first = false;
        push_cli_completion_field_json(output, field, symbols, false);
    }
    output.push(']');
}

fn push_cli_completion_positionals_json(
    output: &mut String,
    schema: &muga::cli_schema::CliSchema,
    symbols: &muga::symbol::SymbolTable,
) {
    let mut fields = schema
        .fields
        .iter()
        .filter(|field| !field.hidden && field.position.is_some())
        .collect::<Vec<_>>();
    fields.sort_by_key(|field| field.position.unwrap_or(u32::MAX));
    output.push('[');
    for (index, field) in fields.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_cli_completion_field_json(output, field, symbols, true);
    }
    output.push(']');
}

fn push_cli_completion_field_json(
    output: &mut String,
    field: &muga::cli_schema::CliFieldSchema,
    symbols: &muga::symbol::SymbolTable,
    positional: bool,
) {
    output.push('{');
    output.push_str("\"field\":");
    push_json_string(output, symbols.resolve(field.name));
    if positional {
        output.push_str(",\"index\":");
        write!(output, "{}", field.position.unwrap_or(0))
            .expect("writing CLI completion JSON to String should not fail");
        output.push_str(",\"fallback\":");
        match field
            .value_source
            .unwrap_or(muga::cli_schema::CliValueSource::File)
        {
            muga::cli_schema::CliValueSource::File => push_json_string(output, "file"),
            muga::cli_schema::CliValueSource::Directory => push_json_string(output, "directory"),
        }
    } else {
        output.push_str(",\"names\":[");
        let mut names = Vec::with_capacity(1 + field.aliases.len());
        names.push(format!("--{}", symbols.resolve(field.option_name)));
        names.extend(
            field
                .aliases
                .iter()
                .map(|alias| format!("--{}", symbols.resolve(*alias))),
        );
        for (index, name) in names.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            push_json_string(output, name);
        }
        output.push(']');
        output.push_str(",\"short\":");
        if let Some(short) = field.short {
            push_json_string(output, &format!("-{}", symbols.resolve(short)));
        } else {
            output.push_str("null");
        }
        output.push_str(",\"takesValue\":");
        output.push_str(if cli_completion_accepts_bare_bool(&field.value) {
            "false"
        } else {
            "true"
        });
        output.push_str(",\"repeatable\":");
        output.push_str(if cli_completion_value_is_repeatable(&field.value) {
            "true"
        } else {
            "false"
        });
    }
    output.push_str(",\"help\":");
    push_optional_symbol_json(output, field.help, symbols);
    output.push_str(",\"valueSource\":");
    push_cli_completion_value_source_json(output, field.value_source);
    output.push_str(",\"value\":");
    push_cli_completion_value_json(output, &field.value, symbols);
    output.push_str(",\"candidates\":");
    push_string_array_json(
        output,
        &cli_completion_value_candidates(&field.value, symbols),
    );
    output.push('}');
}

fn push_cli_completion_value_source_json(
    output: &mut String,
    value_source: Option<muga::cli_schema::CliValueSource>,
) {
    match value_source {
        Some(muga::cli_schema::CliValueSource::File) => push_json_string(output, "file"),
        Some(muga::cli_schema::CliValueSource::Directory) => push_json_string(output, "directory"),
        None => output.push_str("null"),
    }
}

fn push_cli_completion_commands_json(
    output: &mut String,
    schema: &muga::cli_schema::CliSchema,
    symbols: &muga::symbol::SymbolTable,
) {
    output.push('[');
    let mut first = true;
    for command in schema.commands.iter().filter(|command| !command.hidden) {
        if !first {
            output.push(',');
        }
        first = false;
        output.push('{');
        output.push_str("\"name\":");
        push_json_string(output, symbols.resolve(command.command_name));
        output.push_str(",\"aliases\":[");
        for (index, alias) in command.aliases.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            push_json_string(output, symbols.resolve(*alias));
        }
        output.push_str("],\"about\":");
        push_optional_symbol_json(output, command.about, symbols);
        output.push_str(",\"schema\":");
        push_cli_completion_schema_json(output, &command.payload, symbols);
        output.push('}');
    }
    output.push(']');
}

fn push_cli_completion_value_json(
    output: &mut String,
    value: &muga::cli_schema::CliValueSchema,
    symbols: &muga::symbol::SymbolTable,
) {
    output.push('{');
    match value {
        muga::cli_schema::CliValueSchema::String => {
            output.push_str("\"kind\":\"string\"");
        }
        muga::cli_schema::CliValueSchema::Int => {
            output.push_str("\"kind\":\"int\"");
        }
        muga::cli_schema::CliValueSchema::Bool => {
            output.push_str("\"kind\":\"bool\"");
        }
        muga::cli_schema::CliValueSchema::Option(item) => {
            output.push_str("\"kind\":\"option\",\"item\":");
            push_cli_completion_value_json(output, item, symbols);
        }
        muga::cli_schema::CliValueSchema::StringList => {
            output.push_str("\"kind\":\"list\",\"item\":{\"kind\":\"string\"}");
        }
        muga::cli_schema::CliValueSchema::IntList => {
            output.push_str("\"kind\":\"list\",\"item\":{\"kind\":\"int\"}");
        }
        muga::cli_schema::CliValueSchema::BoolList => {
            output.push_str("\"kind\":\"list\",\"item\":{\"kind\":\"bool\"}");
        }
        muga::cli_schema::CliValueSchema::Enum {
            type_name,
            variants,
            ..
        } => {
            output.push_str("\"kind\":\"enum\",\"type\":");
            push_json_string(output, symbols.resolve(*type_name));
            output.push_str(",\"values\":");
            push_cli_completion_enum_values_json(output, variants, symbols);
        }
        muga::cli_schema::CliValueSchema::EnumList {
            type_name,
            variants,
            ..
        } => {
            output.push_str("\"kind\":\"list\",\"item\":{\"kind\":\"enum\",\"type\":");
            push_json_string(output, symbols.resolve(*type_name));
            output.push_str(",\"values\":");
            push_cli_completion_enum_values_json(output, variants, symbols);
            output.push('}');
        }
    }
    output.push('}');
}

fn push_cli_completion_enum_values_json(
    output: &mut String,
    variants: &[muga::cli_schema::CliEnumVariantSchema],
    symbols: &muga::symbol::SymbolTable,
) {
    output.push('[');
    for (index, variant) in variants.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push('{');
        output.push_str("\"name\":");
        push_json_string(output, symbols.resolve(variant.name));
        output.push_str(",\"completion\":");
        push_json_string(output, symbols.resolve(variant.tag));
        output.push('}');
    }
    output.push(']');
}

fn push_optional_symbol_json(
    output: &mut String,
    symbol: Option<muga::symbol::Symbol>,
    symbols: &muga::symbol::SymbolTable,
) {
    if let Some(symbol) = symbol {
        push_json_string(output, symbols.resolve(symbol));
    } else {
        output.push_str("null");
    }
}

fn cli_completion_value_is_repeatable(value: &muga::cli_schema::CliValueSchema) -> bool {
    matches!(
        value,
        muga::cli_schema::CliValueSchema::StringList
            | muga::cli_schema::CliValueSchema::IntList
            | muga::cli_schema::CliValueSchema::BoolList
            | muga::cli_schema::CliValueSchema::EnumList { .. }
    )
}

fn bash_cli_completion_script(model: &CliCompletionModel) -> String {
    let function_name = shell_identifier(&format!("{}_completion", model.program));
    let value_skip_cases = bash_scope_value_skip_cases(model);
    let transition_cases = bash_command_transition_cases(model);
    let value_cases = bash_scope_value_completion_cases(model);
    let scope_cases = bash_scope_candidate_cases(model);
    format!(
        r#"# {program} bash completion generated by muga cli-completions
_{function_name}() {{
  local cur prev scope candidates expect_value
  COMPREPLY=()
  cur="${{COMP_WORDS[COMP_CWORD]}}"
  prev="${{COMP_WORDS[COMP_CWORD-1]}}"
  scope="root"
  expect_value=0

  for word in "${{COMP_WORDS[@]:1:COMP_CWORD-1}}"; do
    if [ "$expect_value" = 1 ]; then
      expect_value=0
      continue
    fi
    case "$scope:$word" in
{value_skip_cases}
    esac
    case "$scope:$word" in
{transition_cases}
    esac
  done

{value_cases}

  case "$scope" in
{scope_cases}
  esac

  COMPREPLY=( $(compgen -W "$candidates" -- "$cur") )
}}
complete -F _{function_name} {program_word}
"#,
        program = model.program,
        function_name = function_name,
        value_skip_cases = value_skip_cases,
        transition_cases = transition_cases,
        value_cases = value_cases,
        scope_cases = scope_cases,
        program_word = shell_word(&model.program),
    )
}

fn bash_scope_value_completion_cases(model: &CliCompletionModel) -> String {
    let mut cases = Vec::new();
    for scope in all_completion_scopes(model) {
        for option in &scope.options {
            if option.values.is_empty() && option.value_source.is_none() {
                continue;
            }
            let patterns = completion_option_patterns(option)
                .into_iter()
                .map(|pattern| bash_case_pattern(&format!("{}:{pattern}", scope.key)))
                .collect::<Vec<_>>()
                .join("|");
            if option.values.is_empty() {
                let compgen_flag = match option.value_source {
                    Some(muga::cli_schema::CliValueSource::File) => "-f",
                    Some(muga::cli_schema::CliValueSource::Directory) => "-d",
                    None => continue,
                };
                cases.push(format!(
                    "  case \"$scope:$prev\" in\n    {patterns}) COMPREPLY=( $(compgen {compgen_flag} -- \"$cur\") ); return 0 ;;\n  esac",
                    patterns = patterns,
                    compgen_flag = compgen_flag,
                ));
            } else {
                let values = shell_single_quote(&shell_words(&option.values));
                cases.push(format!(
                    "  case \"$scope:$prev\" in\n    {patterns}) COMPREPLY=( $(compgen -W {values} -- \"$cur\") ); return 0 ;;\n  esac",
                    patterns = patterns,
                    values = values,
                ));
            }
        }
    }
    cases.join("\n")
}

fn bash_scope_value_skip_cases(model: &CliCompletionModel) -> String {
    let mut cases = Vec::new();
    for scope in all_completion_scopes(model) {
        cases.extend(
            scope
                .options
                .iter()
                .filter(|option| option.takes_value)
                .flat_map(completion_option_patterns)
                .map(|pattern| {
                    format!(
                        "      {}) expect_value=1; continue ;;",
                        bash_case_pattern(&format!("{}:{pattern}", scope.key))
                    )
                }),
        );
    }
    if cases.is_empty() {
        "      __muga_no_value_options__) ;;".to_string()
    } else {
        cases.join("\n")
    }
}

fn bash_command_transition_cases(model: &CliCompletionModel) -> String {
    let cases = model
        .command_transitions
        .iter()
        .map(|transition| {
            format!(
                "      {}) scope={}; continue ;;",
                bash_case_pattern(&format!("{}:{}", transition.from_scope, transition.token)),
                shell_word(&transition.to_scope)
            )
        })
        .collect::<Vec<_>>();
    if cases.is_empty() {
        "      __muga_no_command_transitions__) ;;".to_string()
    } else {
        cases.join("\n")
    }
}

fn bash_scope_candidate_cases(model: &CliCompletionModel) -> String {
    all_completion_scopes(model)
        .into_iter()
        .map(|scope| {
            let candidates = shell_single_quote(&shell_words(&completion_scope_candidates(scope)));
            format!(
                "    {}) candidates={candidates} ;;",
                bash_case_pattern(&scope.key)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn zsh_cli_completion_script(model: &CliCompletionModel) -> String {
    let function_name = shell_identifier(&format!("{}_completion", model.program));
    let value_skip_cases = zsh_scope_value_skip_cases(model);
    let transition_cases = zsh_command_transition_cases(model);
    let value_cases = zsh_scope_value_completion_cases(model);
    let scope_cases = zsh_scope_candidate_cases(model);
    format!(
        r#"#compdef {program_word}
# {program} zsh completion generated by muga cli-completions
_{function_name}() {{
  local scope prev expect_value word
  scope="root"
  prev="${{words[CURRENT-1]}}"
  expect_value=0

  for word in "${{words[@]:1:CURRENT-2}}"; do
    if [[ "$expect_value" == 1 ]]; then
      expect_value=0
      continue
    fi
    case "$scope:$word" in
{value_skip_cases}
    esac
    case "$scope:$word" in
{transition_cases}
    esac
  done

{value_cases}

  case "$scope" in
{scope_cases}
  esac
}}
_{function_name} "$@"
"#,
        program = model.program,
        program_word = shell_word(&model.program),
        function_name = function_name,
        value_skip_cases = value_skip_cases,
        transition_cases = transition_cases,
        value_cases = value_cases,
        scope_cases = scope_cases,
    )
}

fn zsh_scope_value_completion_cases(model: &CliCompletionModel) -> String {
    let mut cases = Vec::new();
    for scope in all_completion_scopes(model) {
        for option in &scope.options {
            if option.values.is_empty() && option.value_source.is_none() {
                continue;
            }
            let patterns = completion_option_patterns(option)
                .into_iter()
                .map(|pattern| zsh_case_pattern(&format!("{}:{pattern}", scope.key)))
                .collect::<Vec<_>>()
                .join("|");
            if option.values.is_empty() {
                let file_command = match option.value_source {
                    Some(muga::cli_schema::CliValueSource::File) => "_files",
                    Some(muga::cli_schema::CliValueSource::Directory) => "_files -/",
                    None => continue,
                };
                cases.push(format!(
                    "  case \"$scope:$prev\" in\n    ({patterns})\n      {file_command}\n      return\n      ;;\n  esac",
                    patterns = patterns,
                    file_command = file_command,
                ));
            } else {
                let entries = zsh_describe_entries(
                    &option
                        .values
                        .iter()
                        .map(|value| (value.clone(), None))
                        .collect::<Vec<_>>(),
                );
                cases.push(format!(
                    "  case \"$scope:$prev\" in\n    ({patterns})\n      local -a option_values\n      option_values=(\n{entries}\n      )\n      _describe -t values '{program} value' option_values\n      return\n      ;;\n  esac",
                    patterns = patterns,
                    entries = entries,
                    program = zsh_single_quote_text(&model.program),
                ));
            }
        }
    }
    cases.join("\n")
}

fn zsh_scope_value_skip_cases(model: &CliCompletionModel) -> String {
    let mut cases = Vec::new();
    for scope in all_completion_scopes(model) {
        cases.extend(
            scope
                .options
                .iter()
                .filter(|option| option.takes_value)
                .flat_map(completion_option_patterns)
                .map(|pattern| {
                    format!(
                        "      ({}) expect_value=1; continue ;;",
                        zsh_case_pattern(&format!("{}:{pattern}", scope.key))
                    )
                }),
        );
    }
    if cases.is_empty() {
        "      (__muga_no_value_options__) ;;".to_string()
    } else {
        cases.join("\n")
    }
}

fn zsh_command_transition_cases(model: &CliCompletionModel) -> String {
    let cases = model
        .command_transitions
        .iter()
        .map(|transition| {
            format!(
                "      ({}) scope={}; continue ;;",
                zsh_case_pattern(&format!("{}:{}", transition.from_scope, transition.token)),
                shell_word(&transition.to_scope)
            )
        })
        .collect::<Vec<_>>();
    if cases.is_empty() {
        "      (__muga_no_command_transitions__) ;;".to_string()
    } else {
        cases.join("\n")
    }
}

fn zsh_scope_candidate_cases(model: &CliCompletionModel) -> String {
    all_completion_scopes(model)
        .into_iter()
        .map(|scope| {
            let entries = zsh_describe_entries(&completion_scope_descriptions(scope));
            format!(
                "    ({})\n      local -a scope_candidates\n      scope_candidates=(\n{entries}\n      )\n      _describe -t values '{program} argument' scope_candidates\n      ;;",
                zsh_case_pattern(&scope.key),
                entries = entries,
                program = zsh_single_quote_text(&model.program),
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn fish_cli_completion_script(model: &CliCompletionModel) -> String {
    let program_word = shell_word(&model.program);
    let mut out = format!(
        "# {} fish completion generated by muga cli-completions\n",
        model.program
    );
    for scope in all_completion_scopes(model) {
        let condition = fish_scope_condition(scope);
        push_fish_options(&mut out, &program_word, &condition, &scope.options);
        writeln!(
            &mut out,
            "complete -c {program_word}{condition} -l help -s h -d 'Show help'"
        )
        .expect("writing fish completion should not fail");
        if !scope.commands.is_empty() {
            let command_entries = scope
                .commands
                .iter()
                .map(|command| match &command.about {
                    Some(about) => format!("{}\\t{}", command.token, about),
                    None => command.token.clone(),
                })
                .collect::<Vec<_>>()
                .join(" ");
            writeln!(
                &mut out,
                "complete -c {program_word}{condition} -f -a {} -d 'Command'",
                shell_single_quote(&command_entries)
            )
            .expect("writing fish completion should not fail");
        }
    }
    out
}

fn fish_scope_condition(scope: &CompletionScope) -> String {
    let mut conditions = scope
        .path
        .iter()
        .map(|tokens| format!("__fish_seen_subcommand_from {}", tokens.join(" ")))
        .collect::<Vec<_>>();
    let command_tokens = scope
        .commands
        .iter()
        .map(|command| command.token.as_str())
        .collect::<Vec<_>>();
    if !command_tokens.is_empty() {
        conditions.push(format!(
            "not __fish_seen_subcommand_from {}",
            command_tokens.join(" ")
        ));
    }
    if conditions.is_empty() {
        String::new()
    } else {
        format!(" -n {}", shell_single_quote(&conditions.join("; and ")))
    }
}

fn push_fish_options(
    out: &mut String,
    program_word: &str,
    condition: &str,
    options: &[CompletionOption],
) {
    for option in options {
        for (index, long) in option.longs.iter().enumerate() {
            let mut line = format!("complete -c {program_word}{condition} -l {long}");
            if index == 0
                && let Some(short) = &option.short
            {
                write!(&mut line, " -s {short}").expect("writing fish option should not fail");
            }
            if option.takes_value {
                line.push_str(" -r");
            }
            if matches!(
                option.value_source,
                Some(muga::cli_schema::CliValueSource::Directory)
            ) && option.values.is_empty()
            {
                line.push_str(" -f -a '(__fish_complete_directories)'");
            }
            if !option.values.is_empty() {
                write!(
                    &mut line,
                    " -a {}",
                    shell_single_quote(&shell_words(&option.values))
                )
                .expect("writing fish option values should not fail");
            }
            if let Some(help) = &option.help {
                write!(&mut line, " -d {}", shell_single_quote(help))
                    .expect("writing fish option help should not fail");
            }
            out.push_str(&line);
            out.push('\n');
        }
    }
}

fn completion_scope_candidates(scope: &CompletionScope) -> Vec<String> {
    let mut candidates = Vec::new();
    candidates.extend(scope.commands.iter().map(|command| command.token.clone()));
    for option in &scope.options {
        candidates.extend(option.longs.iter().map(|long| format!("--{long}")));
        if let Some(short) = &option.short {
            candidates.push(format!("-{short}"));
        }
    }
    candidates.push("--help".to_string());
    candidates.push("-h".to_string());
    candidates
}

fn completion_scope_descriptions(scope: &CompletionScope) -> Vec<(String, Option<String>)> {
    let mut entries = Vec::new();
    entries.extend(
        scope
            .commands
            .iter()
            .map(|command| (command.token.clone(), command.about.clone())),
    );
    for option in &scope.options {
        entries.extend(
            option
                .longs
                .iter()
                .map(|long| (format!("--{long}"), option.help.clone())),
        );
        if let Some(short) = &option.short {
            entries.push((format!("-{short}"), option.help.clone()));
        }
    }
    entries.push(("--help".to_string(), Some("Show help".to_string())));
    entries.push(("-h".to_string(), Some("Show help".to_string())));
    entries
}

fn all_completion_scopes(model: &CliCompletionModel) -> Vec<&CompletionScope> {
    let mut scopes = Vec::with_capacity(1 + model.command_scopes.len());
    scopes.push(&model.root);
    scopes.extend(model.command_scopes.iter());
    scopes
}

fn completion_option_patterns(option: &CompletionOption) -> Vec<String> {
    let mut patterns = option
        .longs
        .iter()
        .map(|long| format!("--{long}"))
        .collect::<Vec<_>>();
    if let Some(short) = &option.short {
        patterns.push(format!("-{short}"));
    }
    patterns
}

fn shell_words(words: &[String]) -> String {
    words.join(" ")
}

fn shell_word(value: &str) -> String {
    if value
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b'/'))
    {
        value.to_string()
    } else {
        shell_single_quote(value)
    }
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r#"'\''"#))
}

fn shell_identifier(value: &str) -> String {
    let mut out = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() || out.as_bytes()[0].is_ascii_digit() {
        out.insert(0, '_');
    }
    out
}

fn bash_case_pattern(value: &str) -> String {
    value.replace('\\', r"\\").replace(')', r"\)")
}

fn zsh_case_pattern(value: &str) -> String {
    value.replace('\\', r"\\").replace(')', r"\)")
}

fn zsh_single_quote_text(value: &str) -> String {
    value.replace('\'', "'\\''")
}

fn zsh_describe_entries(entries: &[(String, Option<String>)]) -> String {
    entries
        .iter()
        .map(|(value, description)| match description {
            Some(description) => format!(
                "    '{}:{}'",
                zsh_single_quote_text(value),
                zsh_single_quote_text(description)
            ),
            None => format!("    '{}'", zsh_single_quote_text(value)),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn bash_shell_completion_script() -> String {
    let commands = SHELL_COMPLETION_COMMANDS.join(" ");
    let options = SHELL_COMPLETION_OPTIONS.join(" ");
    format!(
        r#"# muga bash completion
_muga() {{
  local cur prev commands shells options
  COMPREPLY=()
  cur="${{COMP_WORDS[COMP_CWORD]}}"
  prev="${{COMP_WORDS[COMP_CWORD-1]}}"
  commands="{commands}"
  shells="bash zsh fish"
  options="{options}"

  case "$prev" in
    muga)
      COMPREPLY=( $(compgen -W "$commands --help -h --version -V" -- "$cur") )
      return 0
      ;;
    shell-completions)
      COMPREPLY=( $(compgen -W "$shells" -- "$cur") )
      return 0
      ;;
    --format)
      COMPREPLY=( $(compgen -W "text json" -- "$cur") )
      return 0
      ;;
    --template)
      COMPREPLY=( $(compgen -W "app lib test config-app cli-tool report-app resource-export package-app" -- "$cur") )
      return 0
      ;;
  esac

  COMPREPLY=( $(compgen -W "$options" -- "$cur") )
}}
complete -F _muga muga
"#
    )
}

fn zsh_shell_completion_script() -> String {
    let commands = SHELL_COMPLETION_COMMANDS
        .iter()
        .map(|command| format!("    '{command}:muga {command}'"))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"#compdef muga
# muga zsh completion
_muga() {{
  local -a commands shells
  commands=(
{commands}
  )
  shells=(
    'bash:bash completion'
    'zsh:zsh completion'
    'fish:fish completion'
  )

  _arguments -C \
    '(-h --help)'{{-h,--help}}'[print help]' \
    '(-V --version)'{{-V,--version}}'[print version]' \
    '1:command:->command' \
    '*::argument:->argument'

  case "$state" in
    command)
      _describe -t commands 'muga command' commands
      ;;
    argument)
      case "${{words[2]}}" in
        shell-completions)
          _describe -t shells 'shell' shells
          ;;
        *)
          _files
          ;;
      esac
      ;;
  esac
}}
_muga "$@"
"#
    )
}

fn fish_shell_completion_script() -> String {
    let commands = SHELL_COMPLETION_COMMANDS.join(" ");
    let options = SHELL_COMPLETION_OPTIONS.join(" ");
    format!(
        r#"# muga fish completion
complete -c muga -f -n '__fish_use_subcommand' -a '{commands}' -d 'Muga command'
complete -c muga -f -n '__fish_seen_subcommand_from shell-completions' -a 'bash zsh fish' -d 'Shell'
complete -c muga -l format -a 'text json' -d 'Output format'
complete -c muga -l template -a 'app lib test config-app cli-tool report-app resource-export package-app' -d 'Project template'
complete -c muga -s h -l help -d 'Print help'
complete -c muga -s V -l version -d 'Print version'
complete -c muga -l artifact-root -r -d 'Artifact root'
complete -c muga -l archive-root -r -d 'Archive root'
complete -c muga -l output-dir -r -d 'Output directory'
complete -c muga -l package -r -d 'Package path'
complete -c muga -l line -r -d 'Source line'
complete -c muga -l column -r -d 'Source column'
complete -c muga -l built -d 'Use default build artifacts'
complete -c muga -l dependency-snippet -d 'Print dependency snippet'
complete -c muga -l check -d 'Check formatting without writing'
# Options: {options}
"#
    )
}

fn check_json_output(
    entry_path: &Path,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    check_json_output_with_context(entry_path, status, diagnostics, &[])
}

fn check_json_output_with_context(
    entry_path: &Path,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
    extra_context: &[muga::diagnostic::DiagnosticContext],
) -> String {
    command_diagnostic_json_output_with_context(
        entry_path,
        "check",
        status,
        diagnostics,
        extra_context,
    )
}

fn command_diagnostic_json_output(
    entry_path: &Path,
    command: &str,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    command_diagnostic_json_output_with_context(entry_path, command, status, diagnostics, &[])
}

fn command_diagnostic_json_output_with_context(
    entry_path: &Path,
    command: &str,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
    extra_context: &[muga::diagnostic::DiagnosticContext],
) -> String {
    let mut context = Vec::with_capacity(1 + extra_context.len());
    context.push(entry_source_diagnostic_context(entry_path));
    context.extend_from_slice(extra_context);
    format!(
        "{{\"schemaVersion\":{},\"command\":\"{}\",\"entry\":{},\"status\":\"{}\",\"diagnostics\":{}}}",
        muga::diagnostic::JSON_SCHEMA_VERSION,
        command,
        entry_json_object(entry_path),
        status,
        muga::diagnostic::diagnostics_json_array_with_context(diagnostics, &context)
    )
}

fn app_bundle_source_mode_label(source_free: bool) -> &'static str {
    if source_free {
        "sourceFree"
    } else {
        "sourceBacked"
    }
}

fn app_bundle_emit_json_output(output: &muga::AppBundleOutput, source_mode: &str) -> String {
    let mut json = String::new();
    write!(
        &mut json,
        "{{\"schemaVersion\":{},\"command\":\"emit-app-bundle\",\"root\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing app bundle emit JSON to String should not fail");
    push_path_ref_json(&mut json, &output.root);
    json.push_str(",\"entry\":");
    push_path_ref_json(&mut json, &output.entry);
    json.push_str(",\"launcher\":");
    push_path_ref_json(&mut json, &output.launcher);
    json.push_str(",\"program\":");
    push_json_string(&mut json, &output.program);
    json.push_str(",\"sourceMode\":");
    push_json_string(&mut json, source_mode);
    json.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"artifacts\":[");
    for (index, path) in output.artifacts.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        push_path_ref_json(&mut json, path);
    }
    json.push_str("],\"files\":[");
    for (index, path) in output.files.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        push_path_ref_json(&mut json, path);
    }
    json.push_str("]}");
    json
}

fn app_bundle_emit_diagnostic_json_output(
    entry_path: &Path,
    output_dir: &Path,
    source_mode: &str,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    let context = [
        entry_source_diagnostic_context(entry_path),
        artifact_root_diagnostic_context("bundle-output", output_dir),
    ];
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"emit-app-bundle\",\"entry\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing app bundle emit diagnostic JSON to String should not fail");
    push_path_ref_json(&mut output, entry_path);
    output.push_str(",\"outputDir\":");
    push_path_ref_json(&mut output, output_dir);
    output.push_str(",\"sourceMode\":");
    push_json_string(&mut output, source_mode);
    output.push_str(",\"status\":");
    push_json_string(&mut output, status);
    output.push_str(",\"diagnostics\":");
    output.push_str(&muga::diagnostic::diagnostics_json_array_with_context(
        diagnostics,
        &context,
    ));
    output.push('}');
    output
}

fn package_archive_dependency_snippet(output: &muga::PackageArchiveOutput) -> String {
    format!(
        "[dependencies]\n{} = {{ archive = \"{}\", hash = \"{}\" }}\n",
        manifest_dependency_key(&output.package_name),
        escape_manifest_string(&output.path.display().to_string()),
        escape_manifest_string(&output.content_hash)
    )
}

fn package_archive_emit_json_output(output: &muga::PackageArchiveOutput) -> String {
    let mut json = String::new();
    write!(
        &mut json,
        "{{\"schemaVersion\":{},\"command\":\"emit-package-archive\",\"entryPackage\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing package archive emit JSON to String should not fail");
    push_json_string(&mut json, &output.package_name);
    json.push_str(",\"archive\":");
    push_path_ref_json(&mut json, &output.path);
    json.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"hash\":");
    push_json_string(&mut json, &output.content_hash);
    json.push_str(",\"dependencySnippet\":");
    push_json_string(&mut json, &package_archive_dependency_snippet(output));
    json.push('}');
    json
}

fn package_archive_emit_diagnostic_json_output(
    entry_path: &Path,
    archive_root: &Path,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    let context = [
        entry_source_diagnostic_context(entry_path),
        artifact_root_diagnostic_context("archive-output", archive_root),
    ];
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"emit-package-archive\",\"entry\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing package archive emit diagnostic JSON to String should not fail");
    push_path_ref_json(&mut output, entry_path);
    output.push_str(",\"archiveRoot\":");
    push_path_ref_json(&mut output, archive_root);
    output.push_str(",\"status\":");
    push_json_string(&mut output, status);
    output.push_str(",\"diagnostics\":");
    output.push_str(&muga::diagnostic::diagnostics_json_array_with_context(
        diagnostics,
        &context,
    ));
    output.push('}');
    output
}

fn package_archive_unpack_json_output(
    output: &muga::PackageArchiveMaterializationOutput,
) -> String {
    let mut json = String::new();
    write!(
        &mut json,
        "{{\"schemaVersion\":{},\"command\":\"unpack-package-archive\",\"root\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing package archive unpack JSON to String should not fail");
    push_path_ref_json(&mut json, &output.root);
    json.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"hash\":");
    push_json_string(&mut json, &output.content_hash);
    json.push_str(",\"files\":[");
    for (index, path) in output.files.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        push_path_ref_json(&mut json, path);
    }
    json.push_str("]}");
    json
}

fn package_archive_unpack_diagnostic_json_output(
    archive_path: &Path,
    output_dir: &Path,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    let context = [
        package_archive_diagnostic_context(archive_path),
        artifact_root_diagnostic_context("unpack-output", output_dir),
    ];
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"unpack-package-archive\",\"archive\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing package archive unpack diagnostic JSON to String should not fail");
    push_path_ref_json(&mut output, archive_path);
    output.push_str(",\"outputDir\":");
    push_path_ref_json(&mut output, output_dir);
    output.push_str(",\"status\":");
    push_json_string(&mut output, status);
    output.push_str(",\"diagnostics\":");
    output.push_str(&muga::diagnostic::diagnostics_json_array_with_context(
        diagnostics,
        &context,
    ));
    output.push('}');
    output
}

fn app_archive_emit_json_output(output: &muga::AppBundleArchiveOutput) -> String {
    let mut json = String::new();
    write!(
        &mut json,
        "{{\"schemaVersion\":{},\"command\":\"emit-app-archive\",\"archive\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing app archive emit JSON to String should not fail");
    push_path_ref_json(&mut json, &output.path);
    json.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"program\":");
    push_json_string(&mut json, &output.program);
    json.push_str(",\"hash\":");
    push_json_string(&mut json, &output.content_hash);
    json.push_str(",\"files\":[");
    for (index, path) in output.files.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        push_path_ref_json(&mut json, path);
    }
    json.push_str("]}");
    json
}

fn app_archive_emit_diagnostic_json_output(
    bundle_dir: &Path,
    archive_root: &Path,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    let context = [
        artifact_root_diagnostic_context("app-bundle", bundle_dir),
        artifact_root_diagnostic_context("archive-output", archive_root),
    ];
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"emit-app-archive\",\"bundle\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing app archive emit diagnostic JSON to String should not fail");
    push_path_ref_json(&mut output, bundle_dir);
    output.push_str(",\"archiveRoot\":");
    push_path_ref_json(&mut output, archive_root);
    output.push_str(",\"status\":");
    push_json_string(&mut output, status);
    output.push_str(",\"diagnostics\":");
    output.push_str(&muga::diagnostic::diagnostics_json_array_with_context(
        diagnostics,
        &context,
    ));
    output.push('}');
    output
}

fn app_archive_unpack_json_output(output: &muga::AppBundleArchiveUnpackOutput) -> String {
    let mut json = String::new();
    write!(
        &mut json,
        "{{\"schemaVersion\":{},\"command\":\"unpack-app-archive\",\"root\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing app archive unpack JSON to String should not fail");
    push_path_ref_json(&mut json, &output.root);
    json.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"files\":[");
    for (index, path) in output.files.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        push_path_ref_json(&mut json, path);
    }
    json.push_str("]}");
    json
}

fn app_archive_unpack_diagnostic_json_output(
    archive_path: &Path,
    output_dir: &Path,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    let context = [
        app_archive_diagnostic_context(archive_path),
        artifact_root_diagnostic_context("unpack-output", output_dir),
    ];
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"unpack-app-archive\",\"archive\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing app archive unpack diagnostic JSON to String should not fail");
    push_path_ref_json(&mut output, archive_path);
    output.push_str(",\"outputDir\":");
    push_path_ref_json(&mut output, output_dir);
    output.push_str(",\"status\":");
    push_json_string(&mut output, status);
    output.push_str(",\"diagnostics\":");
    output.push_str(&muga::diagnostic::diagnostics_json_array_with_context(
        diagnostics,
        &context,
    ));
    output.push('}');
    output
}

fn app_archive_diagnostic_json_output(
    archive_path: &Path,
    command: &str,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    let context = [app_archive_diagnostic_context(archive_path)];
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing app archive diagnostic JSON to String should not fail");
    push_json_string(&mut output, command);
    output.push_str(",\"archive\":");
    push_path_ref_json(&mut output, archive_path);
    output.push_str(",\"status\":");
    push_json_string(&mut output, status);
    output.push_str(",\"diagnostics\":");
    output.push_str(&muga::diagnostic::diagnostics_json_array_with_context(
        diagnostics,
        &context,
    ));
    output.push('}');
    output
}

fn app_archive_diagnostic_context(archive_path: &Path) -> muga::diagnostic::DiagnosticContext {
    muga::diagnostic::DiagnosticContext::artifact_file(
        "archive",
        "appArchive",
        archive_path.display().to_string(),
        file_uri_for_path(archive_path),
    )
}

fn package_archive_diagnostic_json_output(
    archive_path: &Path,
    command: &str,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    let context = [package_archive_diagnostic_context(archive_path)];
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing package archive diagnostic JSON to String should not fail");
    push_json_string(&mut output, command);
    output.push_str(",\"archive\":");
    push_path_ref_json(&mut output, archive_path);
    output.push_str(",\"status\":");
    push_json_string(&mut output, status);
    output.push_str(",\"diagnostics\":");
    output.push_str(&muga::diagnostic::diagnostics_json_array_with_context(
        diagnostics,
        &context,
    ));
    output.push('}');
    output
}

fn package_archive_diagnostic_context(archive_path: &Path) -> muga::diagnostic::DiagnosticContext {
    muga::diagnostic::DiagnosticContext::artifact_file(
        "archive",
        "packageArchive",
        archive_path.display().to_string(),
        file_uri_for_path(archive_path),
    )
}

fn app_install_json_output(
    output: &muga::AppBundleInstallOutput,
    bundle_dir: &Path,
    output_dir: &Path,
    replace_owned: bool,
) -> String {
    let mut json = String::new();
    write!(
        &mut json,
        "{{\"schemaVersion\":{},\"command\":\"install-app\",\"status\":\"ok\",\"diagnostics\":[],\"bundle\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing app install JSON to String should not fail");
    push_path_ref_json(&mut json, bundle_dir);
    json.push_str(",\"outputDir\":");
    push_path_ref_json(&mut json, output_dir);
    json.push_str(",\"launcher\":");
    push_path_ref_json(&mut json, &output.launcher);
    json.push_str(",\"metadata\":");
    push_path_ref_json(&mut json, &output.metadata);
    json.push_str(",\"program\":");
    push_json_string(&mut json, &output.program);
    write!(&mut json, ",\"replaceOwned\":{replace_owned},\"files\":[")
        .expect("writing app install JSON to String should not fail");
    for (index, path) in output.files.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        push_path_ref_json(&mut json, path);
    }
    json.push_str("]}");
    json
}

fn app_install_diagnostic_json_output(
    bundle_dir: &Path,
    output_dir: &Path,
    program: Option<&str>,
    replace_owned: bool,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    let context = [
        artifact_root_diagnostic_context("app-bundle", bundle_dir),
        artifact_root_diagnostic_context("install-output", output_dir),
    ];
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"install-app\",\"bundle\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing app install diagnostic JSON to String should not fail");
    push_path_ref_json(&mut output, bundle_dir);
    output.push_str(",\"outputDir\":");
    push_path_ref_json(&mut output, output_dir);
    output.push_str(",\"program\":");
    push_optional_json_string(&mut output, program);
    write!(&mut output, ",\"replaceOwned\":{replace_owned},\"status\":")
        .expect("writing app install diagnostic JSON to String should not fail");
    push_json_string(&mut output, status);
    output.push_str(",\"diagnostics\":");
    output.push_str(&muga::diagnostic::diagnostics_json_array_with_context(
        diagnostics,
        &context,
    ));
    output.push('}');
    output
}

fn app_uninstall_json_output(output: &muga::AppBundleUninstallOutput, output_dir: &Path) -> String {
    let mut json = String::new();
    write!(
        &mut json,
        "{{\"schemaVersion\":{},\"command\":\"uninstall-app\",\"status\":\"ok\",\"diagnostics\":[],\"outputDir\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing app uninstall JSON to String should not fail");
    push_path_ref_json(&mut json, output_dir);
    json.push_str(",\"launcher\":");
    push_path_ref_json(&mut json, &output.launcher);
    json.push_str(",\"metadata\":");
    push_path_ref_json(&mut json, &output.metadata);
    json.push_str(",\"program\":");
    push_json_string(&mut json, &output.program);
    json.push_str(",\"files\":[");
    for (index, path) in output.files.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        push_path_ref_json(&mut json, path);
    }
    json.push_str("]}");
    json
}

fn app_uninstall_diagnostic_json_output(
    output_dir: &Path,
    program: &str,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    let context = [artifact_root_diagnostic_context(
        "install-output",
        output_dir,
    )];
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"uninstall-app\",\"outputDir\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing app uninstall diagnostic JSON to String should not fail");
    push_path_ref_json(&mut output, output_dir);
    output.push_str(",\"program\":");
    push_json_string(&mut output, program);
    output.push_str(",\"status\":");
    push_json_string(&mut output, status);
    output.push_str(",\"diagnostics\":");
    output.push_str(&muga::diagnostic::diagnostics_json_array_with_context(
        diagnostics,
        &context,
    ));
    output.push('}');
    output
}

fn entry_source_diagnostic_context(entry_path: &Path) -> muga::diagnostic::DiagnosticContext {
    muga::diagnostic::DiagnosticContext::source(
        "entry",
        entry_path.display().to_string(),
        file_uri_for_path(entry_path),
    )
}

fn run_json_output(entry_path: &Path, outcome: &muga::runtime::RunOutcome) -> String {
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"run\",\"entry\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing run JSON to String should not fail");
    output.push_str(&entry_json_object(entry_path));
    output.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"stdout\":");
    push_json_string(&mut output, &outcome.output_text);
    output.push_str(",\"stderr\":");
    push_json_string(&mut output, &outcome.stderr_text);
    output.push_str(",\"mainResult\":");
    if let Some(value) = &outcome.main_result {
        push_json_string(&mut output, &value.to_string());
    } else {
        output.push_str("null");
    }
    output.push('}');
    output
}

fn test_json_output(entry_path: &Path, outcome: &muga::TestRunOutcome) -> String {
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"test\",\"entry\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing test JSON to String should not fail");
    output.push_str(&entry_json_object(entry_path));
    output.push_str(",\"status\":");
    push_json_string(
        &mut output,
        if outcome.failed_count() == 0 {
            "ok"
        } else {
            "error"
        },
    );
    output.push_str(",\"diagnostics\":[],\"tests\":[");
    let context = [entry_source_diagnostic_context(entry_path)];
    for (index, test) in outcome.tests.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_test_case_json(&mut output, test, &context);
    }
    write!(
        &mut output,
        "],\"summary\":{{\"passed\":{},\"failed\":{}}}}}",
        outcome.passed_count(),
        outcome.failed_count()
    )
    .expect("writing test JSON to String should not fail");
    output
}

fn push_test_case_json(
    output: &mut String,
    test: &muga::TestCaseResult,
    context: &[muga::diagnostic::DiagnosticContext],
) {
    output.push('{');
    output.push_str("\"name\":");
    push_json_string(output, &test.name);
    output.push_str(",\"status\":");
    push_json_string(output, test_status_label(test.status));
    output.push_str(",\"message\":");
    if let Some(message) = &test.message {
        push_json_string(output, message);
    } else {
        output.push_str("null");
    }
    output.push_str(",\"diagnostics\":");
    output.push_str(&muga::diagnostic::diagnostics_json_array_with_context(
        &test.diagnostics,
        context,
    ));
    output.push_str(",\"stdout\":");
    push_json_string(output, &test.output_text);
    output.push_str(",\"stderr\":");
    push_json_string(output, &test.stderr_text);
    output.push('}');
}

fn test_status_label(status: muga::TestStatus) -> &'static str {
    match status {
        muga::TestStatus::Passed => "passed",
        muga::TestStatus::Failed => "failed",
    }
}

fn check_extra_diagnostic_context(cli: &Cli) -> Vec<muga::diagnostic::DiagnosticContext> {
    let entry_path = Path::new(&cli.path);
    let mut context = Vec::new();
    if let Ok(Some(package_path)) = muga::package::entry_package_path_from_entry(entry_path) {
        context.push(muga::diagnostic::DiagnosticContext::package(
            "entry",
            package_path,
        ));
    }
    if let Some(artifact_root) = &cli.artifact_root {
        context.push(artifact_root_diagnostic_context(
            "check-input",
            Path::new(artifact_root),
        ));
    } else if cli.use_built_artifacts
        && let Ok(artifact_root) = muga::default_build_artifact_root(entry_path)
    {
        context.push(artifact_root_diagnostic_context(
            "default-build",
            &artifact_root,
        ));
    }
    context
}

fn artifact_root_diagnostic_context(
    role: &str,
    path: &Path,
) -> muga::diagnostic::DiagnosticContext {
    muga::diagnostic::DiagnosticContext::artifact_root(
        role,
        path.display().to_string(),
        file_uri_for_path(path),
    )
}

fn build_extra_diagnostic_context(entry_path: &Path) -> Vec<muga::diagnostic::DiagnosticContext> {
    let mut context = Vec::new();
    if let Ok(Some(package_path)) = muga::package::entry_package_path_from_entry(entry_path) {
        context.push(muga::diagnostic::DiagnosticContext::package(
            "entry",
            package_path,
        ));
    }
    if let Ok(artifact_root) = muga::default_build_artifact_root(entry_path) {
        context.push(artifact_root_diagnostic_context(
            "default-build",
            &artifact_root,
        ));
    }
    context
}

fn why_rebuild_extra_diagnostic_context(
    entry_path: &Path,
    artifact_root: &Path,
    selection: muga::ArtifactRootSelection,
) -> Vec<muga::diagnostic::DiagnosticContext> {
    let mut context = Vec::new();
    if let Ok(Some(package_path)) = muga::package::entry_package_path_from_entry(entry_path) {
        context.push(muga::diagnostic::DiagnosticContext::package(
            "entry",
            package_path,
        ));
    }
    let role = match selection {
        muga::ArtifactRootSelection::Built => "default-build",
        muga::ArtifactRootSelection::ArtifactRoot => "check-input",
    };
    context.push(artifact_root_diagnostic_context(role, artifact_root));
    context
}

fn why_rebuild_text_output(
    entry_path: &Path,
    explanation: &muga::ArtifactCacheExplanation,
) -> String {
    let mut output = String::new();
    if let Some(lockfile) = &explanation.lockfile {
        push_why_rebuild_text_line(
            &mut output,
            entry_path,
            &explanation.artifact_root,
            WhyRebuildTextLine {
                state: lockfile.state.as_str(),
                kind: "lockfile",
                package_path: "-",
                path: &lockfile.path,
                reason: &lockfile.reason,
                commands: &lockfile.regeneration_commands,
            },
        );
    }
    for cache in &explanation.archive_caches {
        push_why_rebuild_text_line(
            &mut output,
            entry_path,
            &explanation.artifact_root,
            WhyRebuildTextLine {
                state: cache.state.as_str(),
                kind: "archiveCache",
                package_path: &cache.package_path,
                path: &cache.path,
                reason: &cache.reason,
                commands: &cache.regeneration_commands,
            },
        );
    }
    for artifact in &explanation.artifacts {
        push_why_rebuild_text_line(
            &mut output,
            entry_path,
            &explanation.artifact_root,
            WhyRebuildTextLine {
                state: artifact.state.as_str(),
                kind: artifact.artifact_kind.as_str(),
                package_path: &artifact.package_path,
                path: &artifact.path,
                reason: &artifact.reason,
                commands: &artifact.regeneration_commands,
            },
        );
    }
    output
}

struct WhyRebuildTextLine<'a> {
    state: &'a str,
    kind: &'a str,
    package_path: &'a str,
    path: &'a Path,
    reason: &'a str,
    commands: &'a [muga::ArtifactCacheRegenerationCommand],
}

fn push_why_rebuild_text_line(
    output: &mut String,
    entry_path: &Path,
    artifact_root: &Path,
    line: WhyRebuildTextLine<'_>,
) {
    write!(
        output,
        "{}\t{}\t{}\t{}\t{}",
        sanitize_text_field(line.state),
        sanitize_text_field(line.kind),
        sanitize_text_field(line.package_path),
        sanitize_text_field(&line.path.display().to_string()),
        sanitize_text_field(line.reason),
    )
    .expect("writing why-rebuild text to String should not fail");
    if !line.commands.is_empty() {
        write!(
            output,
            "\trun: {}",
            line.commands
                .iter()
                .map(|command| why_rebuild_command_text(command, entry_path, artifact_root))
                .collect::<Vec<_>>()
                .join("; ")
        )
        .expect("writing why-rebuild text to String should not fail");
    }
    output.push('\n');
}

fn why_rebuild_command_text(
    command: &muga::ArtifactCacheRegenerationCommand,
    entry_path: &Path,
    artifact_root: &Path,
) -> String {
    command
        .command
        .replace("<dir>", &artifact_root.display().to_string())
        .replace("<entry>", &entry_path.display().to_string())
}

fn sanitize_text_field(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if matches!(ch, '\t' | '\n' | '\r') {
                ' '
            } else {
                ch
            }
        })
        .collect()
}

fn why_rebuild_json_output(
    entry_path: &Path,
    explanation: &muga::ArtifactCacheExplanation,
) -> String {
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"why-rebuild\",\"entry\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing why-rebuild JSON to String should not fail");
    output.push_str(&entry_json_object(entry_path));
    output.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"artifactRoot\":");
    push_artifact_root_selection_json(
        &mut output,
        &explanation.artifact_root,
        explanation.artifact_root_selection,
    );
    output.push_str(",\"lockfile\":");
    if let Some(lockfile) = &explanation.lockfile {
        push_artifact_cache_lockfile_json(&mut output, lockfile);
    } else {
        output.push_str("null");
    }
    output.push_str(",\"archiveCache\":[");
    for (index, cache) in explanation.archive_caches.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_artifact_cache_archive_cache_json(&mut output, cache);
    }
    output.push(']');
    output.push_str(",\"packages\":[");
    for (index, package) in explanation.packages.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push('{');
        output.push_str("\"path\":");
        push_json_string(&mut output, &package.path);
        output.push_str(",\"role\":");
        push_json_string(&mut output, package.role.as_str());
        output.push('}');
    }
    output.push_str("],\"artifacts\":[");
    for (index, artifact) in explanation.artifacts.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_artifact_cache_explanation_json(&mut output, artifact);
    }
    output.push_str("]}");
    output
}

fn api_diff_from_cli(
    cli: &Cli,
) -> Result<muga::api_diff::PackageApiDiff, Vec<muga::diagnostic::Diagnostic>> {
    let package = cli
        .packages
        .first()
        .expect("validated api-diff package argument");
    let old_root = cli
        .old_artifact_root
        .as_deref()
        .expect("validated old artifact root");
    let new_root = cli
        .new_artifact_root
        .as_deref()
        .expect("validated new artifact root");
    let mut symbols = muga::symbol::SymbolTable::default();
    let old = muga::interface::PackageInterfaceGraph::read_persisted_artifacts(
        Path::new(old_root),
        std::slice::from_ref(package),
        &mut symbols,
    )?;
    let new = muga::interface::PackageInterfaceGraph::read_persisted_artifacts(
        Path::new(new_root),
        std::slice::from_ref(package),
        &mut symbols,
    )?;
    Ok(muga::api_diff::diff_package_interfaces(
        &old, &new, package, &symbols,
    ))
}

fn api_diff_text_output(diff: &muga::api_diff::PackageApiDiff) -> String {
    let mut output = String::new();
    writeln!(
        &mut output,
        "status\t{}\nsummary\tcompatible={}\tsourceCompatible={}\tbreaking={}\tunknown={}",
        diff.status.as_str(),
        diff.summary.compatible,
        diff.summary.source_compatible,
        diff.summary.breaking,
        diff.summary.unknown
    )
    .expect("writing api-diff text to String should not fail");
    for change in &diff.changes {
        writeln!(
            &mut output,
            "change\t{}\t{}\t{}\t{}",
            change.classification.as_str(),
            sanitize_text_field(&change.kind),
            sanitize_text_field(&change.path),
            sanitize_text_field(&change.message)
        )
        .expect("writing api-diff text to String should not fail");
    }
    output
}

fn api_diff_json_output(diff: &muga::api_diff::PackageApiDiff) -> String {
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"api-diff\",\"status\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing api-diff JSON to String should not fail");
    push_json_string(&mut output, diff.status.as_str());
    output.push_str(",\"package\":");
    push_json_string(&mut output, &diff.package);
    output.push_str(",\"summary\":{");
    write!(
        &mut output,
        "\"compatible\":{},\"sourceCompatible\":{},\"breaking\":{},\"unknown\":{}",
        diff.summary.compatible,
        diff.summary.source_compatible,
        diff.summary.breaking,
        diff.summary.unknown
    )
    .expect("writing api-diff summary JSON to String should not fail");
    output.push_str("},\"changes\":[");
    for (index, change) in diff.changes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push('{');
        output.push_str("\"classification\":");
        push_json_string(&mut output, change.classification.as_str());
        output.push_str(",\"kind\":");
        push_json_string(&mut output, &change.kind);
        output.push_str(",\"path\":");
        push_json_string(&mut output, &change.path);
        output.push_str(",\"message\":");
        push_json_string(&mut output, &change.message);
        output.push('}');
    }
    output.push_str("]}");
    output
}

fn api_diff_diagnostic_json_output(
    package: Option<&str>,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"api-diff\",\"status\":\"error\",\"package\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing api-diff diagnostic JSON to String should not fail");
    if let Some(package) = package {
        push_json_string(&mut output, package);
    } else {
        output.push_str("null");
    }
    output.push_str(",\"diagnostics\":");
    output.push_str(&muga::diagnostic::diagnostics_json_array(diagnostics));
    output.push('}');
    output
}

fn installed_apps_text_output(output: &muga::InstalledAppInventoryOutput) -> String {
    let mut text = String::new();
    if output.apps.is_empty() {
        writeln!(
            &mut text,
            "empty\t{}\tno installed apps",
            sanitize_text_field(&output.output_dir.display().to_string())
        )
        .expect("writing installed apps text to String should not fail");
        return text;
    }
    for app in &output.apps {
        writeln!(
            &mut text,
            "{}\t{}\t{}\t{}\t{}",
            app.state.as_str(),
            sanitize_text_field(&app.program),
            sanitize_text_field(&app.launcher.display().to_string()),
            sanitize_text_field(
                &app.bundle
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .unwrap_or_else(|| "-".to_string())
            ),
            sanitize_text_field(&app.reason)
        )
        .expect("writing installed apps text to String should not fail");
    }
    text
}

fn installed_apps_json_output(output: &muga::InstalledAppInventoryOutput) -> String {
    let mut json = String::new();
    write!(
        &mut json,
        "{{\"schemaVersion\":{},\"command\":\"list-installed-apps\",\"status\":\"ok\",\"diagnostics\":[],\"outputDir\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing installed apps JSON to String should not fail");
    push_path_ref_json(&mut json, &output.output_dir);
    json.push_str(",\"metadataDir\":");
    push_path_ref_json(&mut json, &output.metadata_dir);
    json.push_str(",\"apps\":[");
    for (index, app) in output.apps.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        push_installed_app_json(&mut json, app);
    }
    json.push_str("]}");
    json
}

fn push_installed_app_json(output: &mut String, app: &muga::InstalledAppEntry) {
    output.push('{');
    output.push_str("\"program\":");
    push_json_string(output, &app.program);
    output.push_str(",\"state\":");
    push_json_string(output, app.state.as_str());
    output.push_str(",\"reason\":");
    push_json_string(output, &app.reason);
    output.push_str(",\"launcher\":");
    push_path_ref_json(output, &app.launcher);
    output.push_str(",\"metadata\":");
    push_path_ref_json(output, &app.metadata);
    output.push_str(",\"bundle\":");
    push_optional_path_ref_json(output, app.bundle.as_deref());
    output.push_str(",\"bundleLauncher\":");
    push_optional_path_ref_json(output, app.bundle_launcher.as_deref());
    output.push('}');
}

fn installed_apps_diagnostic_json_output(
    output_dir: &Path,
    status: &str,
    diagnostics: &[muga::diagnostic::Diagnostic],
) -> String {
    let context = [muga::diagnostic::DiagnosticContext::artifact_root(
        "install-output",
        output_dir.display().to_string(),
        file_uri_for_path(output_dir),
    )];
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"list-installed-apps\",\"status\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing installed apps diagnostic JSON to String should not fail");
    push_json_string(&mut output, status);
    output.push_str(",\"outputDir\":");
    push_path_ref_json(&mut output, output_dir);
    output.push_str(",\"diagnostics\":");
    output.push_str(&muga::diagnostic::diagnostics_json_array_with_context(
        diagnostics,
        &context,
    ));
    output.push('}');
    output
}

fn app_archive_verify_text_output(output: &muga::AppBundleArchiveVerifyOutput) -> String {
    format!(
        "status\tok\narchive\t{}\nhash\t{}\nfiles\t{}\n",
        output.path.display(),
        output.content_hash,
        output.files.len()
    )
}

fn app_archive_verify_json_output(output: &muga::AppBundleArchiveVerifyOutput) -> String {
    let mut json = String::new();
    write!(
        &mut json,
        "{{\"schemaVersion\":{},\"command\":\"verify-app-archive\",\"archive\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing app archive verify JSON to String should not fail");
    push_path_ref_json(&mut json, &output.path);
    json.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"hash\":");
    push_json_string(&mut json, &output.content_hash);
    json.push_str(",\"files\":[");
    for (index, path) in output.files.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("{\"path\":");
        push_json_string(&mut json, &path.display().to_string());
        json.push('}');
    }
    json.push_str("]}");
    json
}

fn package_archive_verify_text_output(output: &muga::PackageArchiveVerifyOutput) -> String {
    format!(
        "status\tok\narchive\t{}\nhash\t{}\nmanifest\t{}\nsources\t{}\nresources\t{}\n",
        output.path.display(),
        output.content_hash,
        output.manifest,
        output.sources.len(),
        output.resources.len()
    )
}

fn package_archive_verify_json_output(output: &muga::PackageArchiveVerifyOutput) -> String {
    let mut json = String::new();
    write!(
        &mut json,
        "{{\"schemaVersion\":{},\"command\":\"verify-package-archive\",\"archive\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing package archive verify JSON to String should not fail");
    push_path_ref_json(&mut json, &output.path);
    json.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"hash\":");
    push_json_string(&mut json, &output.content_hash);
    json.push_str(",\"manifest\":{\"path\":");
    push_json_string(&mut json, &output.manifest);
    json.push_str("},\"sources\":[");
    for (index, path) in output.sources.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("{\"path\":");
        push_json_string(&mut json, path);
        json.push('}');
    }
    json.push_str("],\"resources\":[");
    for (index, path) in output.resources.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("{\"path\":");
        push_json_string(&mut json, path);
        json.push('}');
    }
    json.push_str("]}");
    json
}

fn push_artifact_root_selection_json(
    output: &mut String,
    path: &Path,
    selection: muga::ArtifactRootSelection,
) {
    output.push('{');
    output.push_str("\"path\":");
    push_json_string(output, &path.display().to_string());
    output.push_str(",\"uri\":");
    push_json_string(output, &file_uri_for_path(path));
    output.push_str(",\"selection\":");
    push_json_string(output, selection.as_str());
    output.push('}');
}

fn push_artifact_cache_lockfile_json(
    output: &mut String,
    lockfile: &muga::ArtifactCacheLockfileExplanation,
) {
    output.push('{');
    output.push_str("\"kind\":\"lockfile\",\"path\":");
    push_json_string(output, &lockfile.path.display().to_string());
    output.push_str(",\"uri\":");
    push_json_string(output, &file_uri_for_path(&lockfile.path));
    output.push_str(",\"state\":");
    push_json_string(output, lockfile.state.as_str());
    output.push_str(",\"reason\":");
    push_json_string(output, &lockfile.reason);
    output.push_str(",\"dependencies\":[");
    for (index, dependency) in lockfile.dependencies.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_artifact_cache_lockfile_dependency_json(output, dependency);
    }
    output.push_str("],\"metadataHash\":[");
    for (index, hash) in lockfile.hashes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_artifact_cache_hash_json(output, hash);
    }
    output.push_str("],\"regenerationCommand\":[");
    for (index, command) in lockfile.regeneration_commands.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_artifact_cache_regeneration_command_json(output, command);
    }
    output.push_str("]}");
}

fn push_artifact_cache_lockfile_dependency_json(
    output: &mut String,
    dependency: &muga::ArtifactCacheLockfileDependencyExplanation,
) {
    output.push('{');
    output.push_str("\"packagePath\":");
    push_json_string(output, &dependency.package_path);
    output.push_str(",\"sourceKind\":");
    push_json_string(output, &dependency.source_kind);
    output.push_str(",\"source\":");
    push_json_string(output, &dependency.source);
    output.push_str(",\"hashKind\":");
    push_json_string(output, &dependency.hash_kind);
    output.push_str(",\"hash\":");
    push_json_string(output, &dependency.hash);
    output.push_str(",\"dependencies\":[");
    for (index, package_path) in dependency.dependencies.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_json_string(output, package_path);
    }
    output.push_str("]}");
}

fn push_artifact_cache_archive_cache_json(
    output: &mut String,
    cache: &muga::ArtifactCacheArchiveCacheExplanation,
) {
    output.push('{');
    output.push_str("\"kind\":\"archiveCache\",\"packagePath\":");
    push_json_string(output, &cache.package_path);
    output.push_str(",\"path\":");
    push_json_string(output, &cache.path.display().to_string());
    output.push_str(",\"uri\":");
    push_json_string(output, &file_uri_for_path(&cache.path));
    output.push_str(",\"source\":");
    push_json_string(output, &cache.archive_path.display().to_string());
    output.push_str(",\"sourceUri\":");
    push_json_string(output, &file_uri_for_path(&cache.archive_path));
    output.push_str(",\"state\":");
    push_json_string(output, cache.state.as_str());
    output.push_str(",\"reason\":");
    push_json_string(output, &cache.reason);
    output.push_str(",\"metadataHash\":[");
    for (index, hash) in cache.hashes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_artifact_cache_hash_json(output, hash);
    }
    output.push_str("],\"regenerationCommand\":[");
    for (index, command) in cache.regeneration_commands.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_artifact_cache_regeneration_command_json(output, command);
    }
    output.push_str("]}");
}

fn push_artifact_cache_explanation_json(
    output: &mut String,
    artifact: &muga::ArtifactCacheArtifactExplanation,
) {
    output.push('{');
    output.push_str("\"artifactKind\":");
    push_json_string(output, artifact.artifact_kind.as_str());
    output.push_str(",\"packagePath\":");
    push_json_string(output, &artifact.package_path);
    output.push_str(",\"path\":");
    push_json_string(output, &artifact.path.display().to_string());
    output.push_str(",\"uri\":");
    push_json_string(output, &file_uri_for_path(&artifact.path));
    output.push_str(",\"state\":");
    push_json_string(output, artifact.state.as_str());
    output.push_str(",\"reason\":");
    push_json_string(output, &artifact.reason);
    output.push_str(",\"artifactFile\":");
    push_artifact_cache_file_json(output, artifact);
    output.push_str(",\"artifactHash\":[");
    for (index, hash) in artifact.hashes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_artifact_cache_hash_json(output, hash);
    }
    output.push_str("],\"regenerationCommand\":[");
    for (index, command) in artifact.regeneration_commands.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_artifact_cache_regeneration_command_json(output, command);
    }
    output.push_str("]}");
}

fn push_artifact_cache_file_json(
    output: &mut String,
    artifact: &muga::ArtifactCacheArtifactExplanation,
) {
    output.push('{');
    output.push_str("\"kind\":\"artifactFile\",\"role\":");
    push_json_string(output, artifact_cache_file_role(artifact.artifact_kind));
    output.push_str(",\"artifactKind\":");
    push_json_string(output, artifact.artifact_kind.as_str());
    output.push_str(",\"path\":");
    push_json_string(output, &artifact.path.display().to_string());
    output.push_str(",\"uri\":");
    push_json_string(output, &file_uri_for_path(&artifact.path));
    output.push('}');
}

fn artifact_cache_file_role(kind: muga::ArtifactCacheArtifactKind) -> &'static str {
    match kind {
        muga::ArtifactCacheArtifactKind::Interface => "interface",
        muga::ArtifactCacheArtifactKind::CheckCache => "check-cache",
        muga::ArtifactCacheArtifactKind::Implementation => "implementation",
    }
}

fn push_artifact_cache_hash_json(output: &mut String, hash: &muga::ArtifactCacheHashExplanation) {
    output.push('{');
    output.push_str("\"kind\":\"artifactHash\",\"role\":");
    push_json_string(output, &hash.role);
    output.push_str(",\"hashKind\":");
    push_json_string(output, &hash.hash_kind);
    if let Some(package_path) = &hash.package_path {
        output.push_str(",\"packagePath\":");
        push_json_string(output, package_path);
    }
    output.push_str(",\"value\":");
    push_json_string(output, &hash.value);
    output.push('}');
}

fn push_artifact_cache_regeneration_command_json(
    output: &mut String,
    command: &muga::ArtifactCacheRegenerationCommand,
) {
    output.push('{');
    output.push_str("\"kind\":\"regenerationCommand\",\"role\":");
    push_json_string(output, &command.role);
    output.push_str(",\"command\":");
    push_json_string(output, &command.command);
    output.push('}');
}

fn build_json_output(entry_path: &Path, build: &muga::PackageBuildOutput) -> String {
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"build\",\"entry\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing build JSON to String should not fail");
    output.push_str(&entry_json_object(entry_path));
    output.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"artifactRoot\":");
    push_path_ref_json(&mut output, &build.artifact_root);
    output.push_str(",\"artifacts\":[");
    for (index, path) in build.artifacts.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_build_artifact_json(&mut output, build, path);
    }
    output.push_str("]}");
    output
}

fn push_build_artifact_json(output: &mut String, build: &muga::PackageBuildOutput, path: &Path) {
    output.push('{');
    output.push_str("\"status\":");
    push_json_string(output, build_artifact_status(build, path));
    output.push_str(",\"artifactKind\":");
    push_json_string(output, build_artifact_kind(path));
    output.push_str(",\"path\":");
    push_json_string(output, &path.display().to_string());
    output.push_str(",\"uri\":");
    push_json_string(output, &file_uri_for_path(path));
    output.push('}');
}

fn build_artifact_status(build: &muga::PackageBuildOutput, path: &Path) -> &'static str {
    if build.reused_artifacts.iter().any(|reused| reused == path) {
        "reused"
    } else {
        "written"
    }
}

fn build_artifact_kind(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("mgi") => "interface",
        Some("mgc") => "checkCache",
        Some("mgb") => "implementation",
        _ => "unknown",
    }
}

fn artifact_emission_extra_diagnostic_context(
    entry_path: &Path,
    artifact_root: &Path,
) -> Vec<muga::diagnostic::DiagnosticContext> {
    let mut context = Vec::new();
    if let Ok(Some(package_path)) = muga::package::entry_package_path_from_entry(entry_path) {
        context.push(muga::diagnostic::DiagnosticContext::package(
            "entry",
            package_path,
        ));
    }
    context.push(artifact_root_diagnostic_context("output", artifact_root));
    context
}

fn artifact_emission_json_output(
    entry_path: &Path,
    command: &str,
    artifact_root: &Path,
    artifacts: &[PathBuf],
) -> String {
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing artifact emission JSON to String should not fail");
    push_json_string(&mut output, command);
    output.push_str(",\"entry\":");
    output.push_str(&entry_json_object(entry_path));
    output.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"artifactRoot\":");
    push_path_ref_json(&mut output, artifact_root);
    output.push_str(",\"artifacts\":[");
    for (index, path) in artifacts.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_emitted_artifact_json(&mut output, path);
    }
    output.push_str("]}");
    output
}

fn push_emitted_artifact_json(output: &mut String, path: &Path) {
    output.push('{');
    output.push_str("\"artifactKind\":");
    push_json_string(output, build_artifact_kind(path));
    output.push_str(",\"path\":");
    push_json_string(output, &path.display().to_string());
    output.push_str(",\"uri\":");
    push_json_string(output, &file_uri_for_path(path));
    output.push('}');
}

fn metadata_json_output(entry_path: &Path, check: &muga::PackageAwareCheck) -> String {
    let graph = &check.packages.package_graph;
    let interfaces = check.typed_program.package_interfaces();
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"metadata\",\"entry\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing metadata JSON to String should not fail");
    output.push_str(&entry_json_object(entry_path));
    output.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"entryPackage\":");
    push_package_ref_json(&mut output, graph.package(check.packages.entry_package));
    output.push_str(",\"entryModule\":");
    push_module_ref_json(&mut output, graph.module(check.packages.entry_module));
    output.push_str(",\"packages\":[");
    for (index, package) in graph.packages.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        let exports = check.packages.package_exports.package(package.id);
        let interface = interfaces
            .packages
            .iter()
            .find(|interface| interface.package == package.id);
        push_package_metadata_json(
            &mut output,
            package,
            graph,
            exports,
            interface,
            &check.typed_program.symbols,
        );
    }
    output.push_str("]}");
    output
}

fn workspace_json_output(
    entry_path: &Path,
    artifact_root: &Path,
    project: Option<&muga::package::ProjectManifestMetadata>,
    check: &muga::PackageAwareCheck,
) -> String {
    let graph = &check.packages.package_graph;
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"workspace\",\"entry\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing workspace JSON to String should not fail");
    output.push_str(&entry_json_object(entry_path));
    output.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"artifactRoot\":");
    push_path_ref_json(&mut output, artifact_root);
    output.push_str(",\"project\":");
    push_workspace_project_json(&mut output, project);
    output.push_str(",\"entryPackage\":");
    push_package_ref_json(&mut output, graph.package(check.packages.entry_package));
    output.push_str(",\"entryModule\":");
    push_module_ref_json(&mut output, graph.module(check.packages.entry_module));
    output.push_str(",\"packages\":[");
    for (index, package) in graph.packages.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_workspace_package_json(&mut output, package, check);
    }
    output.push_str("],\"dependencyEdges\":[");
    let mut needs_comma = false;
    for package in &graph.packages {
        for import in &package.imports {
            if needs_comma {
                output.push(',');
            } else {
                needs_comma = true;
            }
            push_workspace_dependency_edge_json(&mut output, package, import, graph);
        }
    }
    output.push_str("]}");
    output
}

fn push_workspace_project_json(
    output: &mut String,
    project: Option<&muga::package::ProjectManifestMetadata>,
) {
    let Some(project) = project else {
        output.push_str("null");
        return;
    };
    output.push('{');
    output.push_str("\"manifest\":");
    push_path_ref_json(output, &project.manifest_path);
    output.push_str(",\"root\":");
    push_path_ref_json(output, &project.root);
    output.push_str(",\"sourceRoot\":");
    push_path_ref_json(output, &project.source_root);
    output.push_str(",\"resourceRoot\":");
    push_optional_path_ref_json(output, project.resource_root.as_deref());
    output.push_str(",\"packagePath\":");
    push_json_string(output, &project.package_path);
    output.push_str(",\"directDependencies\":[");
    for (index, dependency) in project.direct_dependencies.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_json_string(output, dependency);
    }
    output.push_str("],\"dependencies\":[");
    for (index, dependency) in project.dependencies.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_workspace_project_dependency_json(output, dependency);
    }
    output.push_str("]}");
}

fn push_workspace_project_dependency_json(
    output: &mut String,
    dependency: &muga::package::ProjectManifestDependencyMetadata,
) {
    output.push('{');
    output.push_str("\"packagePath\":");
    push_json_string(output, &dependency.package_path);
    output.push_str(",\"root\":");
    push_path_ref_json(output, &dependency.root);
    output.push_str(",\"sourceRoot\":");
    push_path_ref_json(output, &dependency.source_root);
    output.push_str(",\"resourceRoot\":");
    push_optional_path_ref_json(output, dependency.resource_root.as_deref());
    output.push_str(",\"sourceKind\":");
    push_json_string(output, dependency.source_kind.as_str());
    output.push_str(",\"source\":");
    push_json_string(output, &dependency.source);
    output.push_str(",\"hash\":");
    if let Some(hash) = &dependency.hash {
        push_json_string(output, hash);
    } else {
        output.push_str("null");
    }
    output.push_str(",\"dependencies\":[");
    for (index, dependency) in dependency.dependencies.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_json_string(output, dependency);
    }
    output.push_str("]}");
}

fn push_workspace_package_json(
    output: &mut String,
    package: &muga::package::PackageInfo,
    check: &muga::PackageAwareCheck,
) {
    let graph = &check.packages.package_graph;
    let loaded_package = check
        .packages
        .packages
        .iter()
        .find(|loaded| loaded.path == package.path);
    output.push('{');
    write!(output, "\"id\":{}", package.id.as_u32())
        .expect("writing workspace JSON to String should not fail");
    output.push_str(",\"path\":");
    push_json_string(output, &package.path);
    output.push_str(",\"role\":");
    push_json_string(
        output,
        if package.id == check.packages.entry_package {
            "entry"
        } else {
            "dependency"
        },
    );
    output.push_str(",\"imports\":[");
    for (index, import) in package.imports.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_package_import_json(output, import);
    }
    output.push_str("],\"modules\":[");
    for (index, module_id) in package.modules.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        let module = graph.module(*module_id);
        let file = module.and_then(|module| {
            loaded_package.and_then(|loaded| {
                loaded
                    .files
                    .iter()
                    .find(|file| file.module_path == module.path)
            })
        });
        push_workspace_module_json(output, module, file, check);
    }
    output.push_str("]}");
}

fn push_workspace_module_json(
    output: &mut String,
    module: Option<&muga::package::PackageModuleInfo>,
    file: Option<&muga::package::LoadedPackageFile>,
    check: &muga::PackageAwareCheck,
) {
    output.push('{');
    output.push_str("\"module\":");
    push_module_ref_json(output, module);
    output.push_str(",\"sourceFile\":");
    if let Some(path) = file.and_then(|file| file.path.as_deref()) {
        push_path_ref_json(output, path);
    } else {
        output.push_str("null");
    }
    let source = file.map(|file| file.source.as_str()).unwrap_or("");
    write!(
        output,
        ",\"lineCount\":{},\"byteLength\":{},\"checked\":{}",
        source.lines().count(),
        source.len(),
        workspace_module_is_checked(module, check)
    )
    .expect("writing workspace JSON to String should not fail");
    output.push('}');
}

fn workspace_module_is_checked(
    module: Option<&muga::package::PackageModuleInfo>,
    check: &muga::PackageAwareCheck,
) -> bool {
    let Some(module) = module else {
        return false;
    };
    check.module_checks.iter().any(|module_check| {
        module_check.package == module.package && module_check.module == module.id
    })
}

fn push_workspace_dependency_edge_json(
    output: &mut String,
    package: &muga::package::PackageInfo,
    import: &muga::package::PackageImportInfo,
    graph: &muga::package::PackageSymbolGraph,
) {
    output.push('{');
    output.push_str("\"from\":");
    push_package_ref_json(output, Some(package));
    output.push_str(",\"to\":");
    push_package_ref_json(output, graph.package(import.package));
    output.push_str(",\"alias\":");
    push_json_string(output, &import.alias);
    output.push_str(",\"path\":");
    push_json_string(output, &import.path);
    output.push_str(",\"span\":");
    push_span_json(output, import.span);
    output.push('}');
}

fn push_package_metadata_json(
    output: &mut String,
    package: &muga::package::PackageInfo,
    graph: &muga::package::PackageSymbolGraph,
    exports: Option<&muga::interface::PackageExports>,
    interface: Option<&muga::interface::PackageInterface>,
    symbols: &muga::symbol::SymbolTable,
) {
    output.push('{');
    write!(output, "\"id\":{}", package.id.as_u32())
        .expect("writing metadata JSON to String should not fail");
    output.push_str(",\"path\":");
    push_json_string(output, &package.path);
    output.push_str(",\"imports\":[");
    for (index, import) in package.imports.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_package_import_json(output, import);
    }
    output.push_str("],\"modules\":[");
    for (index, module_id) in package.modules.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_module_ref_json(output, graph.module(*module_id));
    }
    output.push_str("],\"items\":[");
    for (index, item) in graph
        .items
        .iter()
        .filter(|item| item.package == package.id)
        .enumerate()
    {
        if index > 0 {
            output.push(',');
        }
        push_package_item_json(output, item);
    }
    output.push_str("],\"exports\":");
    push_package_exports_json(output, exports);
    output.push_str(",\"publicInterface\":");
    push_public_interface_json(output, interface, symbols);
    output.push('}');
}

fn push_package_import_json(output: &mut String, import: &muga::package::PackageImportInfo) {
    output.push('{');
    output.push_str("\"alias\":");
    push_json_string(output, &import.alias);
    write!(output, ",\"package\":{}", import.package.as_u32())
        .expect("writing metadata JSON to String should not fail");
    output.push_str(",\"path\":");
    push_json_string(output, &import.path);
    output.push_str(",\"span\":");
    push_span_json(output, import.span);
    output.push('}');
}

fn push_package_item_json(output: &mut String, item: &muga::package::PackageItemInfo) {
    output.push('{');
    write!(
        output,
        "\"id\":{},\"package\":{},\"module\":{}",
        item.id.as_u32(),
        item.package.as_u32(),
        item.module.as_u32()
    )
    .expect("writing metadata JSON to String should not fail");
    output.push_str(",\"name\":");
    push_json_string(output, &item.name);
    output.push_str(",\"kind\":");
    push_json_string(output, package_item_kind_label(item.kind));
    output.push_str(",\"visibility\":");
    push_json_string(output, visibility_label(item.visibility));
    output.push_str(",\"span\":");
    push_span_json(output, item.span);
    output.push_str(",\"mangledName\":");
    push_json_string(output, &item.mangled_name);
    output.push('}');
}

fn push_package_exports_json(
    output: &mut String,
    exports: Option<&muga::interface::PackageExports>,
) {
    output.push_str("{\"records\":");
    if let Some(exports) = exports {
        push_export_items_json(output, &exports.records);
        output.push_str(",\"enums\":");
        push_export_items_json(output, &exports.enums);
        output.push_str(",\"opaqueTypes\":");
        push_export_items_json(output, &exports.opaque_types);
        output.push_str(",\"functions\":");
        push_export_items_json(output, &exports.functions);
    } else {
        output.push_str("[],\"enums\":[],\"opaqueTypes\":[],\"functions\":[]");
    }
    output.push('}');
}

fn push_export_items_json(output: &mut String, items: &[muga::interface::PackageExportItem]) {
    output.push('[');
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push('{');
        write!(output, "\"item\":{}", item.item.as_u32())
            .expect("writing metadata JSON to String should not fail");
        output.push_str(",\"name\":");
        push_json_string(output, &item.name);
        output.push_str(",\"mangledName\":");
        push_json_string(output, &item.mangled_name);
        output.push_str(",\"span\":");
        push_span_json(output, item.span);
        output.push('}');
    }
    output.push(']');
}

fn push_public_interface_json(
    output: &mut String,
    interface: Option<&muga::interface::PackageInterface>,
    symbols: &muga::symbol::SymbolTable,
) {
    output.push('{');
    output.push_str("\"dependencies\":");
    if let Some(interface) = interface {
        push_string_array_json(output, &interface.dependencies);
        output.push_str(",\"records\":[");
        for (index, record) in interface.records.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            push_public_record_json(output, record, symbols);
        }
        output.push_str("],\"enums\":[");
        for (index, enumeration) in interface.enums.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            push_public_enum_json(output, enumeration, symbols);
        }
        output.push_str("],\"opaqueTypes\":[");
        for (index, opaque) in interface.opaque_types.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            push_public_opaque_type_json(output, opaque);
        }
        output.push_str("],\"functions\":[");
        for (index, function) in interface.functions.iter().enumerate() {
            if index > 0 {
                output.push(',');
            }
            push_public_function_json(output, function, symbols);
        }
        output.push(']');
    } else {
        output.push_str("[],\"records\":[],\"enums\":[],\"opaqueTypes\":[],\"functions\":[]");
    }
    output.push('}');
}

fn push_public_record_json(
    output: &mut String,
    record: &muga::interface::PackageInterfaceRecord,
    symbols: &muga::symbol::SymbolTable,
) {
    output.push('{');
    write!(output, "\"item\":{}", record.item.as_u32())
        .expect("writing metadata JSON to String should not fail");
    output.push_str(",\"name\":");
    push_json_string(output, &record.name);
    output.push_str(",\"docComments\":");
    push_string_array_json(output, &record.doc_comments);
    output.push_str(",\"typeParams\":");
    push_string_array_json(output, &record.type_params);
    output.push_str(",\"fields\":[");
    for (index, field) in record.fields.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push('{');
        output.push_str("\"name\":");
        push_json_string(output, &field.name);
        output.push_str(",\"type\":");
        push_type_json(output, &field.ty, symbols);
        output.push_str(",\"span\":");
        push_span_json(output, field.span);
        output.push('}');
    }
    output.push_str("],\"span\":");
    push_span_json(output, record.span);
    output.push('}');
}

fn push_public_enum_json(
    output: &mut String,
    enumeration: &muga::interface::PackageInterfaceEnum,
    symbols: &muga::symbol::SymbolTable,
) {
    output.push('{');
    write!(output, "\"item\":{}", enumeration.item.as_u32())
        .expect("writing metadata JSON to String should not fail");
    output.push_str(",\"name\":");
    push_json_string(output, &enumeration.name);
    output.push_str(",\"docComments\":");
    push_string_array_json(output, &enumeration.doc_comments);
    output.push_str(",\"typeParams\":");
    push_string_array_json(output, &enumeration.type_params);
    output.push_str(",\"variants\":[");
    for (index, variant) in enumeration.variants.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push('{');
        output.push_str("\"name\":");
        push_json_string(output, &variant.name);
        output.push_str(",\"payload\":");
        if let Some(payload) = &variant.payload {
            push_type_json(output, payload, symbols);
        } else {
            output.push_str("null");
        }
        output.push_str(",\"span\":");
        push_span_json(output, variant.span);
        output.push('}');
    }
    output.push_str("],\"span\":");
    push_span_json(output, enumeration.span);
    output.push('}');
}

fn push_public_opaque_type_json(
    output: &mut String,
    opaque: &muga::interface::PackageInterfaceOpaqueType,
) {
    output.push('{');
    write!(output, "\"item\":{}", opaque.item.as_u32())
        .expect("writing metadata JSON to String should not fail");
    output.push_str(",\"name\":");
    push_json_string(output, &opaque.name);
    output.push_str(",\"docComments\":");
    push_string_array_json(output, &opaque.doc_comments);
    output.push_str(",\"handleFacts\":");
    push_opaque_handle_facts_json(output, &opaque.handle_facts);
    output.push_str(",\"span\":");
    push_span_json(output, opaque.span);
    output.push('}');
}

fn push_opaque_handle_facts_json(output: &mut String, facts: &muga::interface::OpaqueHandleFacts) {
    output.push('{');
    write!(
        output,
        "\"runtimeBacked\":{},\"copyable\":{},\"cloneable\":{},\"sendable\":{},\"shareable\":{},\"structurallyComparable\":{},\"serializable\":{},\"closeable\":{}",
        facts.runtime_backed,
        facts.copyable,
        facts.cloneable,
        facts.sendable,
        facts.shareable,
        facts.structurally_comparable,
        facts.serializable,
        facts.closeable
    )
    .expect("writing metadata JSON to String should not fail");
    output.push_str(",\"closeFunction\":");
    if let Some(item) = facts.close_function {
        write!(output, "{}", item.as_u32())
            .expect("writing metadata JSON to String should not fail");
    } else {
        output.push_str("null");
    }
    output.push('}');
}

fn push_public_function_json(
    output: &mut String,
    function: &muga::interface::PackageInterfaceFunction,
    symbols: &muga::symbol::SymbolTable,
) {
    output.push('{');
    write!(output, "\"item\":{}", function.item.as_u32())
        .expect("writing metadata JSON to String should not fail");
    output.push_str(",\"name\":");
    push_json_string(output, &function.name);
    output.push_str(",\"docComments\":");
    push_string_array_json(output, &function.doc_comments);
    output.push_str(",\"typeParams\":");
    push_string_array_json(output, &function.type_params);
    output.push_str(",\"params\":[");
    for (index, param) in function.params.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push('{');
        output.push_str("\"name\":");
        push_json_string(output, &param.name);
        output.push_str(",\"type\":");
        push_type_json(output, &param.ty, symbols);
        output.push_str(",\"paramMode\":");
        push_json_string(output, param.mode.as_str());
        output.push_str(",\"span\":");
        push_span_json(output, param.span);
        output.push('}');
    }
    output.push_str("],\"returnType\":");
    push_type_json(output, &function.ret, symbols);
    output.push_str(",\"span\":");
    push_span_json(output, function.span);
    output.push('}');
}

fn push_package_ref_json(output: &mut String, package: Option<&muga::package::PackageInfo>) {
    if let Some(package) = package {
        output.push('{');
        write!(output, "\"id\":{}", package.id.as_u32())
            .expect("writing metadata JSON to String should not fail");
        output.push_str(",\"path\":");
        push_json_string(output, &package.path);
        output.push('}');
    } else {
        output.push_str("null");
    }
}

fn push_module_ref_json(output: &mut String, module: Option<&muga::package::PackageModuleInfo>) {
    if let Some(module) = module {
        output.push('{');
        write!(
            output,
            "\"id\":{},\"package\":{}",
            module.id.as_u32(),
            module.package.as_u32()
        )
        .expect("writing metadata JSON to String should not fail");
        output.push_str(",\"path\":");
        push_json_string(output, &module.path);
        output.push('}');
    } else {
        output.push_str("null");
    }
}

fn push_type_json(
    output: &mut String,
    ty: &muga::types::TypeInfo,
    symbols: &muga::symbol::SymbolTable,
) {
    push_json_string(output, &muga::doc::render_type_info(ty, symbols));
}

fn push_string_array_json(output: &mut String, values: &[String]) {
    output.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_json_string(output, value);
    }
    output.push(']');
}

fn push_span_json(output: &mut String, span: muga::span::Span) {
    write!(
        output,
        "{{\"start\":{{\"line\":{},\"column\":{}}},\"end\":{{\"line\":{},\"column\":{}}}}}",
        span.start.line, span.start.column, span.end.line, span.end.column
    )
    .expect("writing metadata JSON to String should not fail");
}

fn package_item_kind_label(kind: muga::package::PackageItemKind) -> &'static str {
    match kind {
        muga::package::PackageItemKind::Record => "record",
        muga::package::PackageItemKind::Enum => "enum",
        muga::package::PackageItemKind::OpaqueType => "opaqueType",
        muga::package::PackageItemKind::Function => "function",
    }
}

fn visibility_label(visibility: muga::ast::Visibility) -> &'static str {
    match visibility {
        muga::ast::Visibility::Private => "private",
        muga::ast::Visibility::Package => "package",
        muga::ast::Visibility::Public => "public",
    }
}

enum CompletionPublicItem<'a> {
    Record(&'a muga::interface::PackageInterfaceRecord),
    Enum(&'a muga::interface::PackageInterfaceEnum),
    OpaqueType(&'a muga::interface::PackageInterfaceOpaqueType),
    Function(&'a muga::interface::PackageInterfaceFunction),
}

fn completions_json_output(entry_path: &Path, check: &muga::PackageAwareCheck) -> String {
    let graph = &check.packages.package_graph;
    let interfaces = check.typed_program.package_interfaces();
    let entry_package = graph.package(check.packages.entry_package);
    let mut visible_packages = HashSet::new();
    visible_packages.insert(check.packages.entry_package);
    if let Some(package) = entry_package {
        for import in &package.imports {
            visible_packages.insert(import.package);
        }
    }

    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"completions\",\"entry\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing completions JSON to String should not fail");
    output.push_str(&entry_json_object(entry_path));
    output.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"completions\":[");

    let mut needs_comma = false;
    if let Some(package) = entry_package {
        for import in &package.imports {
            push_completion_separator(&mut output, &mut needs_comma);
            push_import_completion_json(&mut output, import, graph);
        }
    }

    for interface in &interfaces.packages {
        if !visible_packages.contains(&interface.package) {
            continue;
        }
        for record in &interface.records {
            push_completion_separator(&mut output, &mut needs_comma);
            push_completion_item_json(
                &mut output,
                interface,
                graph,
                CompletionPublicItem::Record(record),
                &check.typed_program.symbols,
            );
        }
        for enumeration in &interface.enums {
            push_completion_separator(&mut output, &mut needs_comma);
            push_completion_item_json(
                &mut output,
                interface,
                graph,
                CompletionPublicItem::Enum(enumeration),
                &check.typed_program.symbols,
            );
        }
        for opaque in &interface.opaque_types {
            push_completion_separator(&mut output, &mut needs_comma);
            push_completion_item_json(
                &mut output,
                interface,
                graph,
                CompletionPublicItem::OpaqueType(opaque),
                &check.typed_program.symbols,
            );
        }
        for function in &interface.functions {
            push_completion_separator(&mut output, &mut needs_comma);
            push_completion_item_json(
                &mut output,
                interface,
                graph,
                CompletionPublicItem::Function(function),
                &check.typed_program.symbols,
            );
        }
    }

    output.push_str("]}");
    output
}

fn push_completion_separator(output: &mut String, needs_comma: &mut bool) {
    if *needs_comma {
        output.push(',');
    } else {
        *needs_comma = true;
    }
}

fn push_import_completion_json(
    output: &mut String,
    import: &muga::package::PackageImportInfo,
    graph: &muga::package::PackageSymbolGraph,
) {
    output.push('{');
    output.push_str("\"label\":");
    push_json_string(output, &import.alias);
    output.push_str(",\"kind\":\"import\",\"detail\":");
    push_json_string(output, &format!("import {}", import.path));
    output.push_str(",\"package\":");
    push_package_ref_json(output, graph.package(import.package));
    output
        .push_str(",\"module\":null,\"item\":null,\"signature\":null,\"docComments\":[],\"span\":");
    push_span_json(output, import.span);
    output.push('}');
}

fn push_completion_item_json(
    output: &mut String,
    interface: &muga::interface::PackageInterface,
    graph: &muga::package::PackageSymbolGraph,
    public_item: CompletionPublicItem<'_>,
    symbols: &muga::symbol::SymbolTable,
) {
    let item_id = completion_item_id(&public_item);
    let item = graph.item(item_id);
    let signature = render_completion_signature(&public_item, symbols);
    output.push('{');
    output.push_str("\"label\":");
    push_json_string(output, completion_item_name(&public_item));
    output.push_str(",\"kind\":");
    push_json_string(output, completion_item_kind_label(&public_item));
    output.push_str(",\"detail\":");
    push_json_string(output, &signature);
    output.push_str(",\"package\":");
    push_package_ref_json(output, graph.package(interface.package));
    output.push_str(",\"module\":");
    if let Some(item) = item {
        push_module_ref_json(output, graph.module(item.module));
    } else {
        output.push_str("null");
    }
    write!(output, ",\"item\":{}", item_id.as_u32())
        .expect("writing completions JSON to String should not fail");
    output.push_str(",\"signature\":");
    push_json_string(output, &signature);
    output.push_str(",\"docComments\":");
    push_string_array_json(output, completion_doc_comments(&public_item));
    output.push_str(",\"metadata\":");
    push_completion_metadata_json(output, &public_item);
    output.push_str(",\"span\":");
    push_span_json(
        output,
        item.map_or_else(|| completion_item_span(&public_item), |item| item.span),
    );
    output.push('}');
}

fn push_completion_metadata_json(output: &mut String, item: &CompletionPublicItem<'_>) {
    match item {
        CompletionPublicItem::OpaqueType(opaque) => {
            output.push_str("{\"handleFacts\":");
            push_opaque_handle_facts_json(output, &opaque.handle_facts);
            output.push('}');
        }
        CompletionPublicItem::Function(function) => {
            output.push_str("{\"paramModes\":[");
            for (index, param) in function.params.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push('{');
                output.push_str("\"name\":");
                push_json_string(output, &param.name);
                output.push_str(",\"mode\":");
                push_json_string(output, param.mode.as_str());
                output.push('}');
            }
            output.push_str("]}");
        }
        CompletionPublicItem::Record(_) | CompletionPublicItem::Enum(_) => {
            output.push_str("{}");
        }
    }
}

fn completion_item_id(item: &CompletionPublicItem<'_>) -> muga::identity::PackageItemId {
    match item {
        CompletionPublicItem::Record(record) => record.item,
        CompletionPublicItem::Enum(enumeration) => enumeration.item,
        CompletionPublicItem::OpaqueType(opaque) => opaque.item,
        CompletionPublicItem::Function(function) => function.item,
    }
}

fn completion_item_name<'a>(item: &'a CompletionPublicItem<'_>) -> &'a str {
    match item {
        CompletionPublicItem::Record(record) => &record.name,
        CompletionPublicItem::Enum(enumeration) => &enumeration.name,
        CompletionPublicItem::OpaqueType(opaque) => &opaque.name,
        CompletionPublicItem::Function(function) => &function.name,
    }
}

fn completion_item_kind_label(item: &CompletionPublicItem<'_>) -> &'static str {
    match item {
        CompletionPublicItem::Record(_) => "record",
        CompletionPublicItem::Enum(_) => "enum",
        CompletionPublicItem::OpaqueType(_) => "opaqueType",
        CompletionPublicItem::Function(_) => "function",
    }
}

fn completion_item_span(item: &CompletionPublicItem<'_>) -> muga::span::Span {
    match item {
        CompletionPublicItem::Record(record) => record.span,
        CompletionPublicItem::Enum(enumeration) => enumeration.span,
        CompletionPublicItem::OpaqueType(opaque) => opaque.span,
        CompletionPublicItem::Function(function) => function.span,
    }
}

fn completion_doc_comments<'a>(item: &'a CompletionPublicItem<'_>) -> &'a [String] {
    match item {
        CompletionPublicItem::Record(record) => &record.doc_comments,
        CompletionPublicItem::Enum(enumeration) => &enumeration.doc_comments,
        CompletionPublicItem::OpaqueType(opaque) => &opaque.doc_comments,
        CompletionPublicItem::Function(function) => &function.doc_comments,
    }
}

fn render_completion_signature(
    item: &CompletionPublicItem<'_>,
    symbols: &muga::symbol::SymbolTable,
) -> String {
    match item {
        CompletionPublicItem::Record(record) => render_hover_record_signature(record, symbols),
        CompletionPublicItem::Enum(enumeration) => {
            render_hover_enum_signature(enumeration, symbols)
        }
        CompletionPublicItem::OpaqueType(opaque) => render_hover_opaque_type_signature(opaque),
        CompletionPublicItem::Function(function) => {
            render_hover_function_signature(function, symbols)
        }
    }
}

#[derive(Clone, Debug)]
struct DefinitionTarget {
    kind: &'static str,
    name: String,
    binding: Option<muga::identity::BindingId>,
    binding_kind: Option<muga::identity::BindingKind>,
    package: Option<muga::identity::PackageId>,
    module: Option<muga::identity::ModuleId>,
    item: Option<muga::identity::PackageItemId>,
    span: muga::span::Span,
}

fn definition_json_output(
    entry_path: &Path,
    check: &muga::PackageAwareCheck,
    position: SourcePositionArg,
) -> String {
    let graph = &check.packages.package_graph;
    let target = find_definition_target(check, position);
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"definition\",\"entry\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing definition JSON to String should not fail");
    output.push_str(&entry_json_object(entry_path));
    output.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"position\":");
    push_source_position_json(&mut output, position);
    output.push_str(",\"definition\":");
    if let Some(target) = target {
        push_definition_target_json(&mut output, &target, graph);
    } else {
        output.push_str("null");
    }
    output.push('}');
    output
}

fn find_definition_target(
    check: &muga::PackageAwareCheck,
    position: SourcePositionArg,
) -> Option<DefinitionTarget> {
    let graph = &check.packages.package_graph;
    let entry_package = graph.package(check.packages.entry_package);
    if let Some(import) = entry_package.and_then(|package| {
        package
            .imports
            .iter()
            .find(|import| span_contains_position(import.span, position))
    }) {
        return Some(import_definition_target(import));
    }

    let module_check = check
        .module_checks
        .iter()
        .find(|module_check| module_check.module == check.packages.entry_module)?;

    if let Some(identifier) = module_check
        .type_output
        .identifier_refs
        .iter()
        .filter(|identifier| span_contains_position(identifier.span, position))
        .min_by_key(|identifier| span_size_key(identifier.span))
        && let Some(target) = binding_definition_target(check, module_check, identifier.binding)
    {
        return Some(target);
    }

    if let Some(assignment) = module_check
        .type_output
        .assignment_targets
        .iter()
        .filter(|assignment| {
            binding_name_contains_position(
                assignment.name,
                assignment.span,
                &module_check.type_output.symbols,
                position,
            )
        })
        .min_by_key(|assignment| span_size_key(assignment.span))
        && let Some(target) = binding_definition_target(check, module_check, assignment.binding)
    {
        return Some(target);
    }

    if let Some(binding) = module_check
        .type_output
        .bindings
        .iter()
        .filter(|binding| {
            binding_name_contains_position(
                binding.symbol,
                binding.span,
                &module_check.type_output.symbols,
                position,
            )
        })
        .min_by_key(|binding| span_size_key(binding.span))
        && let Some(target) = binding_definition_target(check, module_check, binding.id)
    {
        return Some(target);
    }

    graph
        .items
        .iter()
        .find(|item| {
            item.module == check.packages.entry_module
                && item_header_contains_position(item, position)
        })
        .map(package_item_definition_target)
}

fn import_definition_target(import: &muga::package::PackageImportInfo) -> DefinitionTarget {
    DefinitionTarget {
        kind: "import",
        name: import.alias.clone(),
        binding: None,
        binding_kind: None,
        package: Some(import.package),
        module: None,
        item: None,
        span: import.span,
    }
}

fn binding_definition_target(
    check: &muga::PackageAwareCheck,
    module_check: &muga::PackageModuleCheck,
    binding_id: muga::identity::BindingId,
) -> Option<DefinitionTarget> {
    let binding = module_check
        .type_output
        .bindings
        .iter()
        .find(|binding| binding.id == binding_id)?;
    if let Some(item) = binding
        .package_item
        .and_then(|item| check.packages.package_graph.item(item))
    {
        return Some(package_item_definition_target(item));
    }
    Some(DefinitionTarget {
        kind: "binding",
        name: module_check
            .type_output
            .symbols
            .resolve(binding.symbol)
            .to_string(),
        binding: Some(binding.id),
        binding_kind: Some(binding.kind),
        package: Some(module_check.package),
        module: Some(module_check.module),
        item: None,
        span: binding.span,
    })
}

fn package_item_definition_target(item: &muga::package::PackageItemInfo) -> DefinitionTarget {
    DefinitionTarget {
        kind: package_item_kind_label(item.kind),
        name: item.name.clone(),
        binding: None,
        binding_kind: None,
        package: Some(item.package),
        module: Some(item.module),
        item: Some(item.id),
        span: item.span,
    }
}

fn push_definition_target_json(
    output: &mut String,
    target: &DefinitionTarget,
    graph: &muga::package::PackageSymbolGraph,
) {
    output.push('{');
    output.push_str("\"kind\":");
    push_json_string(output, target.kind);
    output.push_str(",\"name\":");
    push_json_string(output, &target.name);
    output.push_str(",\"binding\":");
    if let Some(binding) = target.binding {
        write!(output, "{}", binding.as_u32())
            .expect("writing definition JSON to String should not fail");
    } else {
        output.push_str("null");
    }
    output.push_str(",\"bindingKind\":");
    if let Some(kind) = target.binding_kind {
        push_json_string(output, binding_kind_label(kind));
    } else {
        output.push_str("null");
    }
    output.push_str(",\"package\":");
    push_package_ref_json(
        output,
        target.package.and_then(|package| graph.package(package)),
    );
    output.push_str(",\"module\":");
    push_module_ref_json(
        output,
        target.module.and_then(|module| graph.module(module)),
    );
    output.push_str(",\"item\":");
    if let Some(item) = target.item {
        write!(output, "{}", item.as_u32())
            .expect("writing definition JSON to String should not fail");
    } else {
        output.push_str("null");
    }
    output.push_str(",\"span\":");
    push_span_json(output, target.span);
    output.push_str(",\"selectionSpan\":");
    push_span_json(output, target.span);
    output.push('}');
}

#[derive(Clone, Debug)]
struct ReferenceLocation {
    kind: &'static str,
    name: String,
    binding: Option<muga::identity::BindingId>,
    package: Option<muga::identity::PackageId>,
    module: Option<muga::identity::ModuleId>,
    item: Option<muga::identity::PackageItemId>,
    span: muga::span::Span,
}

fn references_json_output(
    entry_path: &Path,
    check: &muga::PackageAwareCheck,
    position: SourcePositionArg,
) -> String {
    let graph = &check.packages.package_graph;
    let target = find_definition_target(check, position);
    let references = target
        .as_ref()
        .map(|target| collect_reference_locations(check, target))
        .unwrap_or_default();

    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"references\",\"entry\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing references JSON to String should not fail");
    output.push_str(&entry_json_object(entry_path));
    output.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"position\":");
    push_source_position_json(&mut output, position);
    output.push_str(",\"target\":");
    if let Some(target) = &target {
        push_definition_target_json(&mut output, target, graph);
    } else {
        output.push_str("null");
    }
    output.push_str(",\"references\":[");
    for (index, reference) in references.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        push_reference_location_json(&mut output, reference, graph);
    }
    output.push_str("]}");
    output
}

fn collect_reference_locations(
    check: &muga::PackageAwareCheck,
    target: &DefinitionTarget,
) -> Vec<ReferenceLocation> {
    let Some(module_check) = check
        .module_checks
        .iter()
        .find(|module_check| module_check.module == check.packages.entry_module)
    else {
        return Vec::new();
    };

    let mut references = Vec::new();
    let mut seen = HashSet::new();

    push_reference_once(
        &mut references,
        &mut seen,
        ReferenceLocation {
            kind: "declaration",
            name: target.name.clone(),
            binding: target.binding,
            package: target.package,
            module: target.module,
            item: target.item,
            span: target.span,
        },
    );

    if target.kind == "import" {
        collect_import_alias_references(check, module_check, target, &mut references, &mut seen);
        return references;
    }

    if let Some(binding) = target.binding {
        collect_binding_references(check, module_check, binding, &mut references, &mut seen);
    } else if let Some(item) = target.item {
        collect_package_item_references(check, module_check, item, &mut references, &mut seen);
    }

    if references.len() > 1 {
        references[1..].sort_by_key(|reference| {
            (
                reference.span.start.line,
                reference.span.start.column,
                reference.span.end.line,
                reference.span.end.column,
            )
        });
    }
    references
}

fn collect_import_alias_references(
    check: &muga::PackageAwareCheck,
    module_check: &muga::PackageModuleCheck,
    target: &DefinitionTarget,
    references: &mut Vec<ReferenceLocation>,
    seen: &mut HashSet<String>,
) {
    let alias_prefix = format!("{}::", target.name);
    for identifier in &module_check.type_output.identifier_refs {
        let name = module_check.type_output.symbols.resolve(identifier.name);
        if !name.starts_with(&alias_prefix) {
            continue;
        }
        let item = module_check
            .type_output
            .bindings
            .iter()
            .find(|binding| binding.id == identifier.binding)
            .and_then(|binding| binding.package_item);
        let item_info = item.and_then(|item| check.packages.package_graph.item(item));
        push_reference_once(
            references,
            seen,
            ReferenceLocation {
                kind: "reference",
                name: name.to_string(),
                binding: None,
                package: item_info.map(|item| item.package).or(target.package),
                module: item_info.map(|item| item.module),
                item,
                span: identifier.span,
            },
        );
    }
}

fn collect_binding_references(
    check: &muga::PackageAwareCheck,
    module_check: &muga::PackageModuleCheck,
    binding: muga::identity::BindingId,
    references: &mut Vec<ReferenceLocation>,
    seen: &mut HashSet<String>,
) {
    for identifier in &module_check.type_output.identifier_refs {
        if identifier.binding != binding {
            continue;
        }
        if let Some(reference) = reference_location_for_binding(
            check,
            module_check,
            binding,
            module_check
                .type_output
                .symbols
                .resolve(identifier.name)
                .to_string(),
            identifier.span,
            "reference",
        ) {
            push_reference_once(references, seen, reference);
        }
    }

    for assignment in &module_check.type_output.assignment_targets {
        if assignment.binding != binding || !assignment.is_update {
            continue;
        }
        if let Some(reference) = reference_location_for_binding(
            check,
            module_check,
            binding,
            module_check
                .type_output
                .symbols
                .resolve(assignment.name)
                .to_string(),
            assignment.span,
            "write",
        ) {
            push_reference_once(references, seen, reference);
        }
    }
}

fn collect_package_item_references(
    check: &muga::PackageAwareCheck,
    module_check: &muga::PackageModuleCheck,
    item: muga::identity::PackageItemId,
    references: &mut Vec<ReferenceLocation>,
    seen: &mut HashSet<String>,
) {
    for identifier in &module_check.type_output.identifier_refs {
        let binding = module_check
            .type_output
            .bindings
            .iter()
            .find(|binding| binding.id == identifier.binding);
        if binding.and_then(|binding| binding.package_item) != Some(item) {
            continue;
        }
        if let Some(item_info) = check.packages.package_graph.item(item) {
            push_reference_once(
                references,
                seen,
                ReferenceLocation {
                    kind: "reference",
                    name: module_check
                        .type_output
                        .symbols
                        .resolve(identifier.name)
                        .to_string(),
                    binding: None,
                    package: Some(item_info.package),
                    module: Some(item_info.module),
                    item: Some(item),
                    span: identifier.span,
                },
            );
        }
    }
}

fn reference_location_for_binding(
    check: &muga::PackageAwareCheck,
    module_check: &muga::PackageModuleCheck,
    binding_id: muga::identity::BindingId,
    name: String,
    span: muga::span::Span,
    kind: &'static str,
) -> Option<ReferenceLocation> {
    let binding = module_check
        .type_output
        .bindings
        .iter()
        .find(|binding| binding.id == binding_id)?;
    if let Some(item) = binding.package_item {
        let item_info = check.packages.package_graph.item(item)?;
        return Some(ReferenceLocation {
            kind,
            name,
            binding: None,
            package: Some(item_info.package),
            module: Some(item_info.module),
            item: Some(item),
            span,
        });
    }
    Some(ReferenceLocation {
        kind,
        name,
        binding: Some(binding_id),
        package: Some(module_check.package),
        module: Some(module_check.module),
        item: None,
        span,
    })
}

fn push_reference_once(
    references: &mut Vec<ReferenceLocation>,
    seen: &mut HashSet<String>,
    reference: ReferenceLocation,
) {
    let key = format!(
        "{}:{}:{}:{}:{}:{:?}:{:?}:{:?}",
        reference.kind,
        reference.span.start.line,
        reference.span.start.column,
        reference.span.end.line,
        reference.span.end.column,
        reference.binding.map(|binding| binding.as_u32()),
        reference.item.map(|item| item.as_u32()),
        reference.package.map(|package| package.as_u32()),
    );
    if seen.insert(key) {
        references.push(reference);
    }
}

fn push_reference_location_json(
    output: &mut String,
    reference: &ReferenceLocation,
    graph: &muga::package::PackageSymbolGraph,
) {
    output.push('{');
    output.push_str("\"kind\":");
    push_json_string(output, reference.kind);
    output.push_str(",\"name\":");
    push_json_string(output, &reference.name);
    output.push_str(",\"binding\":");
    if let Some(binding) = reference.binding {
        write!(output, "{}", binding.as_u32())
            .expect("writing references JSON to String should not fail");
    } else {
        output.push_str("null");
    }
    output.push_str(",\"package\":");
    push_package_ref_json(
        output,
        reference.package.and_then(|package| graph.package(package)),
    );
    output.push_str(",\"module\":");
    push_module_ref_json(
        output,
        reference.module.and_then(|module| graph.module(module)),
    );
    output.push_str(",\"item\":");
    if let Some(item) = reference.item {
        write!(output, "{}", item.as_u32())
            .expect("writing references JSON to String should not fail");
    } else {
        output.push_str("null");
    }
    output.push_str(",\"span\":");
    push_span_json(output, reference.span);
    output.push('}');
}

fn binding_name_contains_position(
    symbol: muga::symbol::Symbol,
    span: muga::span::Span,
    symbols: &muga::symbol::SymbolTable,
    position: SourcePositionArg,
) -> bool {
    let name = symbols.resolve(symbol);
    position.line == span.start.line
        && position.column >= span.start.column
        && position.column < span.start.column + name.chars().count()
}

fn binding_kind_label(kind: muga::identity::BindingKind) -> &'static str {
    match kind {
        muga::identity::BindingKind::Immutable => "immutable",
        muga::identity::BindingKind::Mutable => "mutable",
        muga::identity::BindingKind::Function => "function",
        muga::identity::BindingKind::Parameter => "parameter",
    }
}

fn span_contains_position(span: muga::span::Span, position: SourcePositionArg) -> bool {
    if position.line < span.start.line || position.line > span.end.line {
        return false;
    }
    if span.start.line == span.end.line {
        return position.line == span.start.line
            && position.column >= span.start.column
            && position.column < span.end.column;
    }
    if position.line == span.start.line {
        return position.column >= span.start.column;
    }
    if position.line == span.end.line {
        return position.column < span.end.column;
    }
    true
}

fn span_size_key(span: muga::span::Span) -> usize {
    (span.end.line.saturating_sub(span.start.line) * 100_000)
        + span.end.column.saturating_sub(span.start.column)
}

enum HoverPublicItem<'a> {
    Record(&'a muga::interface::PackageInterfaceRecord),
    Enum(&'a muga::interface::PackageInterfaceEnum),
    OpaqueType(&'a muga::interface::PackageInterfaceOpaqueType),
    Function(&'a muga::interface::PackageInterfaceFunction),
}

fn hover_json_output(
    entry_path: &Path,
    check: &muga::PackageAwareCheck,
    position: SourcePositionArg,
) -> String {
    let graph = &check.packages.package_graph;
    let interfaces = check.typed_program.package_interfaces();
    let item = graph.items.iter().find(|item| {
        item.module == check.packages.entry_module && item_header_contains_position(item, position)
    });
    let mut output = String::new();
    write!(
        &mut output,
        "{{\"schemaVersion\":{},\"command\":\"hover\",\"entry\":",
        muga::diagnostic::JSON_SCHEMA_VERSION
    )
    .expect("writing hover JSON to String should not fail");
    output.push_str(&entry_json_object(entry_path));
    output.push_str(",\"status\":\"ok\",\"diagnostics\":[],\"position\":");
    push_source_position_json(&mut output, position);
    output.push_str(",\"hover\":");
    if let Some(item) = item {
        let public_item = find_hover_public_item(&interfaces, item);
        push_hover_item_json(
            &mut output,
            item,
            graph,
            public_item,
            &check.typed_program.symbols,
        );
    } else {
        output.push_str("null");
    }
    output.push('}');
    output
}

fn item_header_contains_position(
    item: &muga::package::PackageItemInfo,
    position: SourcePositionArg,
) -> bool {
    item.span.start.line == position.line && item.span.start.column <= position.column
}

fn find_hover_public_item<'a>(
    interfaces: &'a muga::interface::PackageInterfaceGraph,
    item: &muga::package::PackageItemInfo,
) -> Option<HoverPublicItem<'a>> {
    let package = interfaces
        .packages
        .iter()
        .find(|package| package.package == item.package)?;
    match item.kind {
        muga::package::PackageItemKind::Record => package
            .records
            .iter()
            .find(|record| record.item == item.id)
            .map(HoverPublicItem::Record),
        muga::package::PackageItemKind::Enum => package
            .enums
            .iter()
            .find(|enumeration| enumeration.item == item.id)
            .map(HoverPublicItem::Enum),
        muga::package::PackageItemKind::OpaqueType => package
            .opaque_types
            .iter()
            .find(|opaque| opaque.item == item.id)
            .map(HoverPublicItem::OpaqueType),
        muga::package::PackageItemKind::Function => package
            .functions
            .iter()
            .find(|function| function.item == item.id)
            .map(HoverPublicItem::Function),
    }
}

fn push_hover_item_json(
    output: &mut String,
    item: &muga::package::PackageItemInfo,
    graph: &muga::package::PackageSymbolGraph,
    public_item: Option<HoverPublicItem<'_>>,
    symbols: &muga::symbol::SymbolTable,
) {
    output.push('{');
    write!(output, "\"item\":{}", item.id.as_u32())
        .expect("writing hover JSON to String should not fail");
    output.push_str(",\"name\":");
    push_json_string(output, &item.name);
    output.push_str(",\"kind\":");
    push_json_string(output, package_item_kind_label(item.kind));
    output.push_str(",\"visibility\":");
    push_json_string(output, visibility_label(item.visibility));
    output.push_str(",\"package\":");
    push_package_ref_json(output, graph.package(item.package));
    output.push_str(",\"module\":");
    push_module_ref_json(output, graph.module(item.module));
    output.push_str(",\"span\":");
    push_span_json(output, item.span);
    output.push_str(",\"signature\":");
    if let Some(public_item) = &public_item {
        push_json_string(output, &render_hover_signature(public_item, symbols));
    } else {
        output.push_str("null");
    }
    output.push_str(",\"docComments\":");
    if let Some(public_item) = &public_item {
        push_string_array_json(output, hover_doc_comments(public_item));
    } else {
        output.push_str("[]");
    }
    output.push_str(",\"metadata\":");
    if let Some(public_item) = &public_item {
        push_hover_metadata_json(output, public_item);
    } else {
        output.push_str("null");
    }
    output.push('}');
}

fn push_hover_metadata_json(output: &mut String, item: &HoverPublicItem<'_>) {
    match item {
        HoverPublicItem::OpaqueType(opaque) => {
            output.push_str("{\"handleFacts\":");
            push_opaque_handle_facts_json(output, &opaque.handle_facts);
            output.push('}');
        }
        HoverPublicItem::Function(function) => {
            output.push_str("{\"paramModes\":[");
            for (index, param) in function.params.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push('{');
                output.push_str("\"name\":");
                push_json_string(output, &param.name);
                output.push_str(",\"mode\":");
                push_json_string(output, param.mode.as_str());
                output.push('}');
            }
            output.push_str("]}");
        }
        HoverPublicItem::Record(_) | HoverPublicItem::Enum(_) => {
            output.push_str("{}");
        }
    }
}

fn render_hover_signature(
    item: &HoverPublicItem<'_>,
    symbols: &muga::symbol::SymbolTable,
) -> String {
    match item {
        HoverPublicItem::Record(record) => render_hover_record_signature(record, symbols),
        HoverPublicItem::Enum(enumeration) => render_hover_enum_signature(enumeration, symbols),
        HoverPublicItem::OpaqueType(opaque) => render_hover_opaque_type_signature(opaque),
        HoverPublicItem::Function(function) => render_hover_function_signature(function, symbols),
    }
}

fn hover_doc_comments<'a>(item: &'a HoverPublicItem<'_>) -> &'a [String] {
    match item {
        HoverPublicItem::Record(record) => &record.doc_comments,
        HoverPublicItem::Enum(enumeration) => &enumeration.doc_comments,
        HoverPublicItem::OpaqueType(opaque) => &opaque.doc_comments,
        HoverPublicItem::Function(function) => &function.doc_comments,
    }
}

fn render_hover_record_signature(
    record: &muga::interface::PackageInterfaceRecord,
    symbols: &muga::symbol::SymbolTable,
) -> String {
    let fields = record
        .fields
        .iter()
        .map(|field| {
            format!(
                "{}: {}",
                field.name,
                muga::doc::render_type_info(&field.ty, symbols)
            )
        })
        .collect::<Vec<_>>();
    let type_params = render_hover_type_params(&record.type_params);
    if fields.is_empty() {
        format!("pub record {}{} {{}}", record.name, type_params)
    } else {
        format!(
            "pub record {}{} {{ {} }}",
            record.name,
            type_params,
            fields.join(", ")
        )
    }
}

fn render_hover_enum_signature(
    enumeration: &muga::interface::PackageInterfaceEnum,
    symbols: &muga::symbol::SymbolTable,
) -> String {
    let variants = enumeration
        .variants
        .iter()
        .map(|variant| {
            if let Some(payload) = &variant.payload {
                format!(
                    "{}({})",
                    variant.name,
                    muga::doc::render_type_info(payload, symbols)
                )
            } else {
                variant.name.clone()
            }
        })
        .collect::<Vec<_>>();
    let type_params = render_hover_type_params(&enumeration.type_params);
    if variants.is_empty() {
        format!("pub enum {}{} {{}}", enumeration.name, type_params)
    } else {
        format!(
            "pub enum {}{} {{ {} }}",
            enumeration.name,
            type_params,
            variants.join(", ")
        )
    }
}

fn render_hover_opaque_type_signature(
    opaque: &muga::interface::PackageInterfaceOpaqueType,
) -> String {
    format!("pub opaque type {}", opaque.name)
}

fn render_hover_function_signature(
    function: &muga::interface::PackageInterfaceFunction,
    symbols: &muga::symbol::SymbolTable,
) -> String {
    let params = function
        .params
        .iter()
        .map(|param| {
            format!(
                "{}: {}",
                param.name,
                muga::doc::render_type_info(&param.ty, symbols)
            )
        })
        .collect::<Vec<_>>();
    format!(
        "pub fn {}{}({}): {}",
        function.name,
        render_hover_type_params(&function.type_params),
        params.join(", "),
        muga::doc::render_type_info(&function.ret, symbols)
    )
}

fn render_hover_type_params(type_params: &[String]) -> String {
    if type_params.is_empty() {
        String::new()
    } else {
        format!("[{}]", type_params.join(", "))
    }
}

fn push_source_position_json(output: &mut String, position: SourcePositionArg) {
    write!(
        output,
        "{{\"line\":{},\"column\":{}}}",
        position.line, position.column
    )
    .expect("writing hover JSON to String should not fail");
}

fn entry_json_object(path: &Path) -> String {
    let mut output = String::new();
    push_path_ref_json(&mut output, path);
    output
}

fn push_path_ref_json(output: &mut String, path: &Path) {
    output.push('{');
    output.push_str("\"path\":");
    push_json_string(output, &path.display().to_string());
    output.push_str(",\"uri\":");
    push_json_string(output, &file_uri_for_path(path));
    output.push('}');
}

fn push_optional_path_ref_json(output: &mut String, path: Option<&Path>) {
    if let Some(path) = path {
        push_path_ref_json(output, path);
    } else {
        output.push_str("null");
    }
}

fn push_optional_json_string(output: &mut String, value: Option<&str>) {
    if let Some(value) = value {
        push_json_string(output, value);
    } else {
        output.push_str("null");
    }
}

fn file_uri_for_path(path: &Path) -> String {
    let path = absolute_path_for_uri(path);
    let path_text = path.to_string_lossy();
    let mut uri = String::from("file://");
    if !path_text.starts_with('/') {
        uri.push('/');
    }
    uri.push_str(&percent_encode_uri_path(&path_text));
    uri
}

fn absolute_path_for_uri(path: &Path) -> PathBuf {
    if let Ok(canonical_path) = path.canonicalize() {
        return canonical_path;
    }
    if path.is_absolute() {
        return path.to_path_buf();
    }
    env::current_dir()
        .map(|cwd| cwd.join(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

fn percent_encode_uri_path(path: &str) -> String {
    let mut output = String::new();
    for byte in path.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' | b':' => {
                output.push(byte as char)
            }
            _ => write!(&mut output, "%{byte:02X}")
                .expect("writing URI escape to String should not fail"),
        }
    }
    output
}

fn push_json_string(output: &mut String, value: &str) {
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{08}' => output.push_str("\\b"),
            '\u{0c}' => output.push_str("\\f"),
            ch if ch.is_control() => {
                write!(output, "\\u{:04x}", ch as u32)
                    .expect("writing JSON escape to String should not fail");
            }
            ch => output.push(ch),
        }
    }
    output.push('"');
}

fn print_run_outcome(outcome: &muga::runtime::RunOutcome) {
    let has_output = !outcome.output_text.is_empty();
    if has_output {
        print!("{}", outcome.output_text);
    }
    if !outcome.stderr_text.is_empty() {
        eprint!("{}", outcome.stderr_text);
    }
    if let Some(value) = &outcome.main_result {
        if has_output && !outcome.output_text.ends_with('\n') {
            println!();
        }
        println!("{value}");
    } else {
        if has_output && !outcome.output_text.ends_with('\n') {
            println!();
        }
        println!("ok");
    }
}

fn print_test_outcome(outcome: &muga::TestRunOutcome) {
    for test in &outcome.tests {
        match test.status {
            muga::TestStatus::Passed => println!("test {} ... ok", test.name),
            muga::TestStatus::Failed => {
                println!("test {} ... FAILED", test.name);
                if let Some(message) = &test.message {
                    println!("  {message}");
                }
                for diagnostic in &test.diagnostics {
                    println!("  {diagnostic}");
                }
                if !test.output_text.is_empty() {
                    println!("  stdout:");
                    for line in test.output_text.lines() {
                        println!("    {line}");
                    }
                }
                if !test.stderr_text.is_empty() {
                    println!("  stderr:");
                    for line in test.stderr_text.lines() {
                        println!("    {line}");
                    }
                }
            }
        }
    }
    println!(
        "test result: {}. {} passed; {} failed",
        if outcome.failed_count() == 0 {
            "ok"
        } else {
            "FAILED"
        },
        outcome.passed_count(),
        outcome.failed_count()
    );
}

fn manifest_dependency_key(package_name: &str) -> String {
    if package_name
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        package_name.to_string()
    } else {
        format!("\"{}\"", escape_manifest_string(package_name))
    }
}

fn escape_manifest_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
