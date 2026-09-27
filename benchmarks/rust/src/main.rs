#[path = "../../alloc.rs"]
mod alloc;

use std::io::{self, BufRead, Write};
use std::path::Path;
use std::time::Instant;

#[derive(Clone, Copy)]
struct Point {
    x: i64,
    y: i64,
}

enum Step {
    Add(i64),
    Sub(i64),
}

fn fib(n: i64) -> i64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

fn count_paths(root: &Path) -> io::Result<i64> {
    let mut count = 0;
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        count += 1;
        if entry.file_type()?.is_dir() {
            count += count_paths(&entry.path())?;
        }
    }
    Ok(count)
}

fn run(case: &str, data: Option<&str>) -> Result<i64, Box<dyn std::error::Error>> {
    Ok(match case {
        "cpu_loop" => {
            let mut sum = 0;
            for i in 0..200_000_i64 {
                sum = (sum * 33 + i * 17 + 3) % 1_000_003;
            }
            sum
        }
        "cpu_recursion" => fib(22),
        "cpu_records" => {
            let mut point = Point { x: 1, y: 2 };
            for i in 0..10_000 {
                let x = (point.x * 33 + i) % 1_000_003;
                let y = (point.y * 17 + x) % 1_000_003;
                point = Point { x, y };
            }
            point.x + point.y
        }
        "cpu_enums" => {
            let mut total = 0;
            let mut state = 1;
            for i in 0..20_000_i64 {
                state = (state * 33 + i) % 1_000_003;
                let step = if i % 2 == 0 {
                    Step::Add(state)
                } else {
                    Step::Sub(state)
                };
                total += match step {
                    Step::Add(value) => value,
                    Step::Sub(value) => -value,
                };
            }
            total
        }
        "string" => {
            let mut text = String::new();
            for _ in 0..300 {
                text.push_str("alpha,beta,gamma|");
            }
            text.split('|').count() as i64 + text.len() as i64
        }
        "list" => {
            let mut values = Vec::new();
            for i in 0..800_i64 {
                values.push(i);
            }
            values.iter().sum()
        }
        "map" => {
            let mut items = std::collections::HashMap::new();
            for i in 0..300_i64 {
                items.insert(i.to_string(), i);
            }
            (0..300_i64)
                .map(|i| items.get(&i.to_string()).copied().unwrap_or(0))
                .sum()
        }
        "json" => {
            let text = std::fs::read_to_string(data.ok_or("missing data path")?)?;
            let records: serde_json::Value = serde_json::from_str(&text)?;
            records
                .as_array()
                .ok_or("expected JSON array")?
                .iter()
                .map(|item| item["score"].as_i64().ok_or("expected integer score"))
                .sum::<Result<i64, _>>()?
        }
        "directory" => count_paths(Path::new(data.ok_or("missing data path")?))?,
        "text" => {
            let text = std::fs::read_to_string(data.ok_or("missing data path")?)?;
            text.split('\n')
                .filter(|line| line.contains("error"))
                .count() as i64
        }
        _ => return Err(format!("unknown case: {case}").into()),
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let case = args.next().ok_or("expected case name")?;
    let data = args.next();
    if args.next().is_some() {
        return Err("expected case name and optional data path".into());
    }
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        if line? != "run" {
            return Err("expected protocol command `run`".into());
        }
        let before = alloc::snapshot();
        let started = Instant::now();
        let checksum = std::hint::black_box(run(&case, data.as_deref())?);
        let elapsed_ns = started.elapsed().as_nanos();
        let after = alloc::snapshot();
        writeln!(
            stdout,
            "{}\t{}\t{}\t{}",
            checksum,
            elapsed_ns,
            after.0 - before.0,
            after.1 - before.1
        )?;
        stdout.flush()?;
    }
    Ok(())
}
