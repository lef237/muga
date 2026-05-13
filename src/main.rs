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
                muga::compile_typed_path_against_cached_artifact_root(
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
        Mode::Run => match muga::run_path(Path::new(&cli.path)) {
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
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Check,
    Run,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Cli {
    mode: Mode,
    path: String,
    artifact_root: Option<String>,
}

impl Cli {
    fn parse(mut args: Vec<String>) -> Result<Self, String> {
        if args.is_empty() {
            return Err("missing source file".to_string());
        }

        let mode = if matches!(args.first().map(String::as_str), Some("check" | "run")) {
            match args.remove(0).as_str() {
                "check" => Mode::Check,
                "run" => Mode::Run,
                _ => unreachable!("mode was already checked"),
            }
        } else {
            Mode::Run
        };

        let mut artifact_root = None;
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
            } else if arg.starts_with('-') {
                return Err(format!("unknown option `{arg}`"));
            } else if path.replace(arg.clone()).is_some() {
                return Err("too many source files".to_string());
            }
            index += 1;
        }

        if artifact_root.is_some() && mode != Mode::Check {
            return Err("--artifact-root is only supported with `check`".to_string());
        }

        let Some(path) = path else {
            return Err("missing source file".to_string());
        };

        Ok(Self {
            mode,
            path,
            artifact_root,
        })
    }
}

fn usage() -> &'static str {
    "usage: muga [check|run] [--artifact-root <dir>] <source-file>"
}
