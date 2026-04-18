use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: muga <source-file>");
        return ExitCode::from(2);
    };

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("failed to read {path}: {error}");
            return ExitCode::from(2);
        }
    };

    match muga::check_source(&source) {
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
