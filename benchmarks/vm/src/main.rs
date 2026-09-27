//! Persistent VM process for the Phase 0 runtime benchmark protocol.
#[path = "../../alloc.rs"]
mod alloc;

use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let source = args.next().ok_or("expected a Muga source path")?;
    let data = args.next();
    if args.next().is_some() {
        return Err("expected a source path and optional data path".into());
    }
    let program = muga::compile_bytecode_path(Path::new(&source)).map_err(|errors| {
        errors
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let program_args: Vec<String> = data.into_iter().collect();
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        if line? != "run" {
            return Err("expected protocol command `run`".into());
        }
        let before = alloc::snapshot();
        let started = Instant::now();
        let outcome = muga::runtime::run_with_args(&program, &program_args).map_err(|errors| {
            errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        })?;
        let elapsed_ns = started.elapsed().as_nanos();
        let after = alloc::snapshot();
        if !outcome.runtime_diagnostics.is_empty()
            || !outcome.output_text.is_empty()
            || !outcome.stderr_text.is_empty()
        {
            return Err("benchmark source produced diagnostics or output".into());
        }
        let value = outcome
            .main_result
            .ok_or("benchmark main returned no value")?;
        writeln!(
            stdout,
            "{}\t{}\t{}\t{}",
            value,
            elapsed_ns,
            after.0 - before.0,
            after.1 - before.1
        )?;
        stdout.flush()?;
    }
    Ok(())
}
