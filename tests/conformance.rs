use std::{
    fs, io,
    path::{Path, PathBuf},
};

const CONFORMANCE_ROOT: &str = "conformance/current";

#[test]
fn valid_conformance_programs_run() {
    let fixtures = muga_files(Path::new(CONFORMANCE_ROOT).join("valid"));
    assert!(
        !fixtures.is_empty(),
        "conformance suite should include valid fixtures"
    );

    for fixture in fixtures {
        let source = read(&fixture);
        let expected = required_directive(&source, "expect-main");
        let outcome = muga::run_path(&fixture).unwrap_or_else(|diagnostics| {
            panic!(
                "valid conformance fixture failed: {}\n{}",
                fixture.display(),
                diagnostics_text(&diagnostics)
            )
        });
        let value = outcome.main_result.unwrap_or_else(|| {
            panic!(
                "valid conformance fixture did not return a main value: {}",
                fixture.display()
            )
        });
        assert_eq!(
            value.to_string(),
            expected,
            "fixture: {}",
            fixture.display()
        );
    }
}

#[test]
fn rejecting_conformance_programs_report_expected_codes() {
    let fixtures = muga_files(Path::new(CONFORMANCE_ROOT).join("rejecting"));
    assert!(
        !fixtures.is_empty(),
        "conformance suite should include rejecting fixtures"
    );

    for fixture in fixtures {
        let source = read(&fixture);
        let expected = required_directive(&source, "expect-error");
        let diagnostics = muga::check_path(&fixture).unwrap_err_or_else(|| {
            panic!(
                "rejecting conformance fixture unexpectedly passed: {}",
                fixture.display()
            )
        });
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == expected),
            "fixture: {}\nexpected diagnostic: {expected}\nactual diagnostics:\n{}",
            fixture.display(),
            diagnostics_text(&diagnostics)
        );
    }
}

#[test]
fn package_artifact_conformance_runs_without_dependency_source_fallback() {
    let fixture_root = Path::new(CONFORMANCE_ROOT)
        .join("package-artifacts")
        .join("basic");
    let fixture_entry = fixture_root.join("app/main/main.muga");
    let source = read(&fixture_entry);
    let expected = required_directive(&source, "expect-main");

    let workspace = conformance_temp_root("package-artifacts-basic");
    copy_dir(&fixture_root, &workspace).unwrap_or_else(|error| {
        panic!(
            "failed to copy package conformance fixture to {}: {error}",
            workspace.display()
        )
    });

    let entry = workspace.join("app/main/main.muga");
    let source_outcome = muga::run_path(&entry).unwrap_or_else(|diagnostics| {
        panic!(
            "source package conformance fixture failed before artifact emission: {}\n{}",
            entry.display(),
            diagnostics_text(&diagnostics)
        )
    });
    assert_main_value(&source_outcome, &expected, &entry);

    let artifact_root = workspace.join("artifacts");
    muga::write_package_artifacts(&entry, &artifact_root).unwrap_or_else(|diagnostics| {
        panic!(
            "failed to emit conformance package artifacts: {}\n{}",
            entry.display(),
            diagnostics_text(&diagnostics)
        )
    });

    let hidden_sources = workspace.join("dependency-sources-hidden");
    fs::create_dir_all(&hidden_sources).expect("hidden source directory should be created");
    fs::rename(workspace.join("model"), hidden_sources.join("model"))
        .expect("model dependency source should be hidden");
    fs::rename(workspace.join("api"), hidden_sources.join("api"))
        .expect("api dependency source should be hidden");

    let artifact_outcome = muga::run_path_against_artifact_root(&entry, &artifact_root)
        .unwrap_or_else(|diagnostics| {
            panic!(
                "artifact-backed package conformance fixture failed: {}\n{}",
                entry.display(),
                diagnostics_text(&diagnostics)
            )
        });
    assert_main_value(&artifact_outcome, &expected, &entry);
}

fn muga_files(root: PathBuf) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_muga_files(&root, &mut files);
    files.sort();
    files
}

fn collect_muga_files(root: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", root.display()))
    {
        let path = entry
            .unwrap_or_else(|error| {
                panic!("failed to read entry under {}: {error}", root.display())
            })
            .path();
        if path.is_dir() {
            collect_muga_files(&path, out);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("muga") {
            out.push(path);
        }
    }
}

fn required_directive(source: &str, name: &str) -> String {
    let prefix = format!("// {name}:");
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix(&prefix).map(str::trim))
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| panic!("missing `{prefix}` directive"))
}

trait ExpectErrOrElse<T, E> {
    fn unwrap_err_or_else(self, on_ok: impl FnOnce() -> E) -> E;
}

impl<T, E> ExpectErrOrElse<T, E> for Result<T, E> {
    fn unwrap_err_or_else(self, on_ok: impl FnOnce() -> E) -> E {
        match self {
            Ok(_) => on_ok(),
            Err(error) => error,
        }
    }
}

fn assert_main_value(outcome: &muga::runtime::RunOutcome, expected: &str, entry: &Path) {
    let value = outcome.main_result.as_ref().unwrap_or_else(|| {
        panic!(
            "package conformance fixture did not return a main value: {}",
            entry.display()
        )
    });
    assert_eq!(value.to_string(), expected, "fixture: {}", entry.display());
}

fn conformance_temp_root(name: &str) -> PathBuf {
    let home = std::env::var_os("HOME").expect("HOME should be set for conformance temp files");
    let root = PathBuf::from(home)
        .join("tmp")
        .join(format!("muga-conformance-{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).unwrap_or_else(|error| {
            panic!(
                "failed to clear existing temp root {}: {error}",
                root.display()
            )
        });
    }
    fs::create_dir_all(&root)
        .unwrap_or_else(|error| panic!("failed to create temp root {}: {error}", root.display()));
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

fn read(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn diagnostics_text(diagnostics: &[muga::diagnostic::Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}
