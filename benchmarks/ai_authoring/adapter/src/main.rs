use std::{env, path::Path, process::ExitCode};

use muga::runtime::Value;

fn main() -> ExitCode {
    let mut args = env::args();
    let _executable = args.next();
    let Some(entry) = args.next() else {
        eprintln!("usage: muga-minigit-adapter ENTRY [ARG ...]");
        return ExitCode::FAILURE;
    };
    let program_args: Vec<String> = args.collect();
    match muga::run_path_with_args(Path::new(&entry), &program_args) {
        Ok(outcome) => {
            print!("{}", outcome.output_text);
            eprint!("{}", outcome.stderr_text);
            if !outcome.runtime_diagnostics.is_empty() {
                for diagnostic in outcome.runtime_diagnostics {
                    eprintln!("{diagnostic}");
                }
                return ExitCode::FAILURE;
            }
            match outcome.main_result {
                Some(Value::Int(value)) if (0..=255).contains(&value) => {
                    ExitCode::from(value as u8)
                }
                other => {
                    eprintln!("minigit main must return an Int exit code in 0..255; got {other:?}");
                    ExitCode::FAILURE
                }
            }
        }
        Err(diagnostics) => {
            for diagnostic in diagnostics {
                eprintln!("{diagnostic}");
            }
            ExitCode::FAILURE
        }
    }
}
