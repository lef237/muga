use std::{fs, process::Command};

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_muga"))
}

#[test]
fn discard_evaluates_each_expression_without_creating_a_binding() {
    let source = "fn main(): Int {\n  _ = 1.println()\n  _ = \"two\".println()\n  3\n}\n";
    let outcome = muga::run_source(source).unwrap();
    assert_eq!(outcome.output_text, "1\ntwo\n");
    assert_eq!(outcome.main_result.unwrap().to_string(), "3");
    let formatted = muga::format_source(source).unwrap();
    assert_eq!(muga::format_source(&formatted).unwrap(), formatted);
    assert_eq!(
        muga::run_source(&formatted).unwrap().output_text,
        "1\ntwo\n"
    );
    assert!(muga::check_source("fn main(): Int {\n  _ = 1\n  _\n}\n").is_err());
    assert!(muga::check_source("fn main(): Int {\n  _ = missing\n  1\n}\n").is_err());
    assert!(muga::check_source("fn main(): Int {\n  _: Int = true\n  1\n}\n").is_err());
}

#[test]
fn discard_try_preserves_error_propagation() {
    let outcome = muga::run_source(
        "fn fail(): Result[Int, String] {\n  Result::Err(\"stop\")\n}\nfn main(): Result[Int, String] {\n  _ = try fail()\n  _ = \"unreachable\".println()\n  Result::Ok(1)\n}\n",
    ).unwrap();
    assert!(outcome.output_text.is_empty());
    assert!(outcome.main_result.unwrap().to_string().contains("stop"));
}

#[test]
fn non_final_expression_explains_discard_syntax() {
    let diagnostics = muga::check_source("fn main(): Int {\n  1.println()\n  2\n}\n").unwrap_err();
    let diagnostic = diagnostics.iter().find(|d| d.code == "P009").unwrap();
    assert!(diagnostic.to_string().contains("_ = expr"));
    let output = command().args(["explain", "P009"]).output().unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("_ = expr")
    );
}

#[test]
fn text_run_preserves_program_streams_exactly() {
    let root = std::env::temp_dir().join(format!("muga-audit-streams-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    for (index, source, expected_out, expected_err) in [
        (
            0,
            "fn main(): Int {\n  _ = \"out\".print()\n  _ = \"err\".eprint()\n  42\n}\n",
            "out",
            "err",
        ),
        (1, "fn main(): Int {\n  42\n}\n", "", ""),
        (2, "_ = \"script\".print()\n", "script", ""),
    ] {
        let path = root.join(format!("case{index}.muga"));
        fs::write(&path, source).unwrap();
        let output = command().arg("run").arg(path).output().unwrap();
        assert!(output.status.success(), "{output:?}");
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected_out);
        assert_eq!(String::from_utf8(output.stderr).unwrap(), expected_err);
    }
}

#[test]
fn package_discard_survives_artifact_execution() {
    let root = std::env::temp_dir().join(format!(
        "muga-audit-discard-artifacts-{}",
        std::process::id()
    ));
    let entry = root.join("app/main.muga");
    fs::create_dir_all(entry.parent().unwrap()).unwrap();
    fs::write(
        &entry,
        "package app\nfn main(): Int {\n  _ = 1.println()\n  _ = 2.println()\n  3\n}\n",
    )
    .unwrap();
    muga::build_package_artifacts(&entry).unwrap();
    let output = command()
        .args(["run", "--built", "--format=json"])
        .arg(entry)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"stdout\":\"1\\n2\\n\""), "{stdout}");
    assert!(stdout.contains("\"mainResult\":\"3\""), "{stdout}");
}
