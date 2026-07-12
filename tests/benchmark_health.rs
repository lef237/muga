use std::{
    fs, io,
    path::{Path, PathBuf},
    time::Instant,
};

#[test]
#[ignore = "manual benchmark health check; run scripts/benchmark-health-check.sh"]
fn compiler_stage_health_check_reports_elapsed_times() {
    let source_path = Path::new("samples/string_helpers.muga");
    let source = fs::read_to_string(source_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", source_path.display()));

    measure_result("compiler.lex", || muga::lexer::lex(&source).map(|_| ()));
    measure_result("compiler.parse", || {
        let tokens = muga::lexer::lex(&source)?;
        muga::parser::parse(tokens).map(|_| ())
    });
    measure_result("compiler.check", || {
        muga::check_path(source_path).map(|_| ())
    });
    measure_result("compiler.typed-hir", || {
        muga::compile_typed_path(source_path).map(|_| ())
    });
    measure_result("compiler.mir", || {
        muga::compile_mir_path(source_path).map(|_| ())
    });
    measure_result("compiler.bytecode", || {
        muga::compile_bytecode_path(source_path).map(|_| ())
    });
}

#[test]
#[ignore = "manual benchmark health check; run scripts/benchmark-health-check.sh"]
fn package_artifact_reuse_health_check_reports_elapsed_times() {
    let workspace = benchmark_temp_root("package-artifact-reuse");
    copy_dir(
        Path::new("conformance/current/package-artifacts/basic"),
        &workspace,
    )
    .unwrap_or_else(|error| {
        panic!(
            "failed to copy package artifact fixture to {}: {error}",
            workspace.display()
        )
    });
    let entry = workspace.join("app/main/main.muga");

    let first = measure_result("package.build.initial", || {
        muga::build_package_artifacts(&entry)
    });
    assert!(
        !first.artifacts.is_empty(),
        "initial package build should produce artifacts"
    );
    assert!(
        !first.written_artifacts.is_empty(),
        "initial package build should write fresh artifacts"
    );

    let second = measure_result("package.build.reuse", || {
        muga::build_package_artifacts(&entry)
    });
    assert_eq!(
        second.reused_artifacts.len(),
        second.artifacts.len(),
        "second package build should reuse every artifact"
    );

    let outcome = measure_result("package.run.built", || {
        muga::run_path_against_default_build_artifacts(&entry)
    });
    assert_main_value(&outcome, "42", &entry);
}

#[test]
#[ignore = "manual benchmark health check; run scripts/benchmark-health-check.sh"]
fn representative_runtime_health_check_reports_elapsed_times() {
    for (label, path, expected) in [
        ("runtime.string", "samples/string_helpers.muga", "Ada Muga"),
        (
            "runtime.std-list",
            "samples/packages/app/std_list/main.muga",
            "14",
        ),
        (
            "runtime.std-map",
            "samples/packages/app/std_map/main.muga",
            "a:2",
        ),
        (
            "runtime.std-task",
            "samples/packages/app/std_task/main.muga",
            "item-7/70",
        ),
    ] {
        let entry = Path::new(path);
        let outcome = measure_result(label, || muga::run_path(entry));
        assert_main_value(&outcome, expected, entry);
    }
}

fn measure<T>(label: &str, run: impl FnOnce() -> T) -> T {
    let started = Instant::now();
    let outcome = run();
    let elapsed = started.elapsed();
    println!("benchmark-health\t{label}\t{}ms", elapsed.as_millis());
    outcome
}

fn measure_result<T>(
    label: &str,
    run: impl FnOnce() -> Result<T, Vec<muga::diagnostic::Diagnostic>>,
) -> T {
    measure(label, run).unwrap_or_else(|diagnostics| {
        panic!(
            "{label} failed with diagnostics:\n{}",
            diagnostics_text(&diagnostics)
        )
    })
}

fn assert_main_value(outcome: &muga::runtime::RunOutcome, expected: &str, entry: &Path) {
    let value = outcome.main_result.as_ref().unwrap_or_else(|| {
        panic!(
            "benchmark health entry did not return a main value: {}",
            entry.display()
        )
    });
    assert_eq!(value.to_string(), expected, "entry: {}", entry.display());
}

fn benchmark_temp_root(name: &str) -> PathBuf {
    let home = std::env::var_os("HOME").expect("HOME should be set for benchmark health checks");
    let root = PathBuf::from(home).join("tmp").join(format!(
        "muga-benchmark-health-{name}-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap_or_else(|error| {
            panic!(
                "failed to clear existing benchmark health temp root {}: {error}",
                root.display()
            )
        });
    }
    fs::create_dir_all(&root).unwrap_or_else(|error| {
        panic!(
            "failed to create benchmark health temp root {}: {error}",
            root.display()
        )
    });
    root
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let from_path = entry.path();
        let to_path = to.join(entry.file_name());
        if from_path.is_dir() {
            copy_dir(&from_path, &to_path)?;
        } else {
            fs::copy(&from_path, &to_path)?;
        }
    }
    Ok(())
}

fn diagnostics_text(diagnostics: &[muga::diagnostic::Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}
