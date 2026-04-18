use std::{fs, path::Path};

fn extract_code(markdown: &str) -> String {
    let start = markdown.find("```txt").expect("missing opening code fence");
    let after = &markdown[start + "```txt".len()..];
    let after = after.strip_prefix('\n').unwrap_or(after);
    let end = after.find("```").expect("missing closing code fence");
    after[..end].trim_end().to_string()
}

fn fixture_paths(dir: &str) -> Vec<std::path::PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    paths.sort();
    paths
}

#[test]
fn valid_examples_pass_frontend() {
    for path in fixture_paths("examples/valid") {
        let markdown = fs::read_to_string(&path).unwrap();
        let source = extract_code(&markdown);
        let result = muga::check_source(&source);
        assert!(
            result.is_ok(),
            "expected valid example to pass: {}\n{:#?}",
            display_path(&path),
            result.err()
        );
    }
}

#[test]
fn invalid_examples_fail_frontend() {
    for path in fixture_paths("examples/invalid") {
        let markdown = fs::read_to_string(&path).unwrap();
        let source = extract_code(&markdown);
        let result = muga::check_source(&source);
        assert!(
            result.is_err(),
            "expected invalid example to fail: {}",
            display_path(&path)
        );
    }
}

#[test]
fn runnable_main_returns_value() {
    let source = fs::read_to_string("samples/sum_to.muga").unwrap();
    let result = muga::run_source(&source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "10");
    assert!(result.output_lines.is_empty());
}

#[test]
fn builtin_print_captures_output_and_returns_argument() {
    let source = fs::read_to_string("samples/print_sum.muga").unwrap();
    let result = muga::run_source(&source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "10");
    assert_eq!(result.output_lines, vec!["10".to_string()]);
}

#[test]
fn compile_source_lowers_functions_into_hir_table() {
    let source = r#"
fn main() -> Int {
  add = fn(x: Int) -> Int {
    x + 1
  }
  add(41)
}
"#;
    let program = muga::compile_source(source).unwrap();
    assert_eq!(program.functions.len(), 2);
    assert_eq!(program.functions[0].name.as_deref(), Some("main"));
    assert_eq!(program.functions[1].name, None);
}

#[test]
fn closures_capture_outer_bindings() {
    let source = r#"
fn main() -> Int {
  base = 41
  add = fn(x: Int) -> Int {
    x + base
  }
  add(1)
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "42");
}

#[test]
fn mutually_recursive_functions_run() {
    let source = r#"
fn even(n: Int) -> Bool {
  if n == 0 {
    true
  } else {
    odd(n - 1)
  }
}

fn odd(n: Int) -> Bool {
  if n == 0 {
    false
  } else {
    even(n - 1)
  }
}

fn main() -> Bool {
  even(10)
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "true");
}

#[test]
fn runtime_reports_division_by_zero() {
    let source = r#"
fn main() -> Int {
  1 / 0
}
"#;
    let diagnostics = muga::run_source(source).expect_err("expected runtime error");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "R013")
    );
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
