use std::{env, path::Path, process::ExitCode};

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
        Mode::Check => {
            let result = if let Some(artifact_root) = &cli.artifact_root {
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
                    println!("ok");
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
        Mode::Run => match if let Some(artifact_root) = &cli.artifact_root {
            muga::run_path_against_artifact_root(Path::new(&cli.path), Path::new(artifact_root))
        } else {
            muga::run_path(Path::new(&cli.path))
        } {
            Ok(outcome) => {
                let has_output = !outcome.output_text.is_empty();
                if has_output {
                    print!("{}", outcome.output_text);
                }
                if let Some(value) = outcome.main_result {
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
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                for diagnostic in diagnostics {
                    eprintln!("{diagnostic}");
                }
                ExitCode::from(1)
            }
        },
        Mode::EmitInterface => {
            let artifact_root = cli.artifact_root.expect("validated artifact root");
            match muga::write_package_interface_artifacts(
                Path::new(&cli.path),
                Path::new(&artifact_root),
                &cli.packages,
            ) {
                Ok(paths) => {
                    for path in paths {
                        println!("{}", path.display());
                    }
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
        Mode::EmitCheckCache => {
            let artifact_root = cli.artifact_root.expect("validated artifact root");
            match muga::write_package_check_cache_artifact_for_root(
                Path::new(&cli.path),
                Path::new(&artifact_root),
            ) {
                Ok(path) => {
                    println!("{}", path.display());
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
        Mode::EmitArtifacts => {
            let artifact_root = cli.artifact_root.expect("validated artifact root");
            match muga::write_package_artifacts(Path::new(&cli.path), Path::new(&artifact_root)) {
                Ok(paths) => {
                    for path in paths {
                        println!("{}", path.display());
                    }
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Check,
    Run,
    EmitInterface,
    EmitCheckCache,
    EmitArtifacts,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Cli {
    mode: Mode,
    path: String,
    artifact_root: Option<String>,
    packages: Vec<String>,
}

impl Cli {
    fn parse(mut args: Vec<String>) -> Result<Self, String> {
        if args.is_empty() {
            return Err("missing source file".to_string());
        }

        let mode = if matches!(
            args.first().map(String::as_str),
            Some("check" | "run" | "emit-interface" | "emit-check-cache" | "emit-artifacts")
        ) {
            match args.remove(0).as_str() {
                "check" => Mode::Check,
                "run" => Mode::Run,
                "emit-interface" => Mode::EmitInterface,
                "emit-check-cache" => Mode::EmitCheckCache,
                "emit-artifacts" => Mode::EmitArtifacts,
                _ => unreachable!("mode was already checked"),
            }
        } else {
            Mode::Run
        };

        let mut artifact_root = None;
        let mut packages = Vec::new();
        let mut path = None;
        let mut index = 0;
        while index < args.len() {
            let arg = &args[index];
            if arg == "--artifact-root" {
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
            } else if arg.starts_with('-') {
                return Err(format!("unknown option `{arg}`"));
            } else if path.replace(arg.clone()).is_some() {
                return Err("too many source files".to_string());
            }
            index += 1;
        }

        if artifact_root.is_some()
            && !matches!(
                mode,
                Mode::Check
                    | Mode::Run
                    | Mode::EmitInterface
                    | Mode::EmitCheckCache
                    | Mode::EmitArtifacts
            )
        {
            return Err(
                "--artifact-root is only supported with `check`, `run`, `emit-interface`, `emit-check-cache`, or `emit-artifacts`"
                    .to_string(),
            );
        }
        if matches!(
            mode,
            Mode::EmitInterface | Mode::EmitCheckCache | Mode::EmitArtifacts
        ) && artifact_root.is_none()
        {
            return Err(format!("{} requires --artifact-root", mode.name()));
        }
        if !packages.is_empty() && mode != Mode::EmitInterface {
            return Err("--package is only supported with `emit-interface`".to_string());
        }

        let Some(path) = path else {
            return Err("missing source file".to_string());
        };

        Ok(Self {
            mode,
            path,
            artifact_root,
            packages,
        })
    }
}

impl Mode {
    fn name(self) -> &'static str {
        match self {
            Mode::Check => "check",
            Mode::Run => "run",
            Mode::EmitInterface => "emit-interface",
            Mode::EmitCheckCache => "emit-check-cache",
            Mode::EmitArtifacts => "emit-artifacts",
        }
    }
}

fn usage() -> &'static str {
    "usage: muga [check|run|emit-interface|emit-check-cache|emit-artifacts] [--artifact-root <dir>] [--package <package>] <source-file>"
}
