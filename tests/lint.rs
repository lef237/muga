use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn muga(args: &[&str], entry: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_muga"))
        .arg("lint")
        .args(args)
        .arg(entry)
        .output()
        .expect("muga command should run")
}

/// Writes a one-module project whose entry imports an unused package and
/// leaves a parameter unused.
fn warning_project(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("muga-lint-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let entry = root.join("src/main/main.muga");
    fs::create_dir_all(entry.parent().unwrap()).unwrap();
    fs::write(
        root.join("muga.toml"),
        "[package]\nname = \"demo\"\nlanguage_revision = 1\nsource = \"src\"\n",
    )
    .unwrap();
    fs::write(
        &entry,
        "import std::env\nimport std::fs\n\nfn label(count: Int, unused: Int): String {\n  count.to_string()\n}\n\nfn main(): Int {\n  _ = env::args()\n  _ = 1.label(2).println()\n  0\n}\n",
    )
    .unwrap();
    entry
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn warnings_are_reported_without_failing_lint() {
    let entry = warning_project("warn");
    let output = muga(&[], &entry);
    assert!(output.status.success(), "{output:#?}");
    assert_eq!(stdout(&output), "ok\n");
    assert_eq!(
        stderr(&output),
        "2:1: W001 warning: unused import `fs`\n  help: 2:1: remove the import\n\
         4:22: W003 warning: unused parameter `unused`\n  help: 4:22: prefix the parameter with `_` if it is intentionally unused; replace with `_unused`\n"
    );
}

#[test]
fn json_output_carries_severity_status_and_module_context() {
    let entry = warning_project("json");
    let output = muga(&["--format", "json"], &entry);
    let text = stdout(&output);
    assert!(output.status.success(), "{output:#?}");
    assert!(
        text.starts_with("{\"schemaVersion\":1,\"command\":\"lint\",\"entry\":"),
        "{text}"
    );
    assert!(text.contains("\"status\":\"ok\""), "{text}");
    assert!(
        text.contains(
            "{\"code\":\"W001\",\"severity\":\"warning\",\"message\":\"unused import `fs`\""
        ),
        "{text}"
    );
    assert!(
        text.contains(
            "\"replacement\":\"\"}],\"context\":[{\"kind\":\"source\",\"role\":\"entry\""
        ),
        "{text}"
    );
    assert!(
        text.contains("{\"kind\":\"source\",\"role\":\"module\",\"path\":\""),
        "{text}"
    );
    assert_eq!(stderr(&output), "");
}

#[test]
fn deny_options_turn_warnings_into_failing_errors() {
    let entry = warning_project("deny");

    let output = muga(&["--deny-warnings", "--format=json"], &entry);
    let text = stdout(&output);
    assert_eq!(output.status.code(), Some(1), "{output:#?}");
    assert!(text.contains("\"status\":\"error\""), "{text}");
    assert!(
        text.contains("{\"code\":\"W003\",\"severity\":\"error\""),
        "{text}"
    );
    assert!(!text.contains("\"severity\":\"warning\""), "{text}");

    let output = muga(&["--deny", "W001", "--allow=W003"], &entry);
    assert_eq!(output.status.code(), Some(1), "{output:#?}");
    assert_eq!(stdout(&output), "");
    assert_eq!(
        stderr(&output),
        "2:1: W001 unused import `fs`\n  help: 2:1: remove the import\n"
    );

    let output = muga(&["--allow", "W001,W003", "--deny-warnings"], &entry);
    assert!(output.status.success(), "{output:#?}");
    assert_eq!(stdout(&output), "ok\n");
    assert_eq!(stderr(&output), "");
}

#[test]
fn compile_errors_fail_lint_before_lints_run() {
    let entry = warning_project("compile-error");
    fs::write(&entry, "fn main(): Int {\n  missing\n}\n").unwrap();
    let output = muga(&["--format", "json"], &entry);
    let text = stdout(&output);
    assert_eq!(output.status.code(), Some(1), "{output:#?}");
    assert!(text.contains("\"status\":\"error\""), "{text}");
    assert!(
        text.contains("{\"code\":\"N001\",\"severity\":\"error\""),
        "{text}"
    );
}

#[test]
fn lint_options_are_validated() {
    let entry = warning_project("options");
    for (args, message) in [
        (
            vec!["--allow", "E001"],
            "unknown lint code `E001`; lint codes are S001, S003, W001, W002, W003, W004, W005",
        ),
        (vec!["--warn="], "missing value for --warn"),
        (
            vec!["--fix", "--deny-warnings"],
            "--fix cannot be combined with --format json or lint level options",
        ),
        (
            vec!["--deny-warnings", "--deny-warnings"],
            "--deny-warnings was provided more than once",
        ),
    ] {
        let output = muga(&args, &entry);
        assert_eq!(output.status.code(), Some(2), "{args:?}: {output:#?}");
        assert!(
            stderr(&output).contains(message),
            "{args:?}: {}",
            stderr(&output)
        );
    }

    let output = Command::new(env!("CARGO_BIN_EXE_muga"))
        .args(["lint", "--deny"])
        .output()
        .expect("muga command should run");
    assert_eq!(output.status.code(), Some(2), "{output:#?}");
    assert!(
        stderr(&output).contains("missing value for --deny"),
        "{}",
        stderr(&output)
    );

    let output = Command::new(env!("CARGO_BIN_EXE_muga"))
        .args(["check", "--allow", "W001"])
        .arg(&entry)
        .output()
        .expect("muga command should run");
    assert_eq!(output.status.code(), Some(2), "{output:#?}");
    assert!(
        stderr(&output).contains(
            "--allow, --warn, --deny, and --deny-warnings are only supported with `lint`"
        ),
        "{}",
        stderr(&output)
    );
}
