use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let mut args = env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() {
        eprintln!("usage: muga [check|run] <source-file>");
        return ExitCode::from(2);
    }

    let (mode, path) = if args.len() >= 2 && matches!(args[0].as_str(), "check" | "run") {
        (args.remove(0), args.remove(0))
    } else {
        ("run".to_string(), args.remove(0))
    };

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read {path}: {error}");
            return ExitCode::from(2);
        }
    };

    match mode.as_str() {
        "check" => match muga::check_source(&source) {
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
        },
        "run" => match muga::run_source(&source) {
            Ok(outcome) => {
                for line in outcome.output_lines {
                    println!("{line}");
                }
                if let Some(value) = outcome.main_result {
                    println!("{value}");
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
        _ => {
            eprintln!("usage: muga [check|run] <source-file>");
            ExitCode::from(2)
        }
    }
}
