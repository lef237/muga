use std::{fs, path::Path};

use muga::bytecode::Instruction;

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
    assert_sample_runs("samples/sum_to.muga", "10", "");
}

#[test]
fn builtin_println_captures_output_and_returns_argument() {
    assert_sample_runs("samples/println_sum.muga", "10", "10\n");
}

#[test]
fn record_update_sample_runs() {
    assert_sample_runs("samples/record_with_update.muga", "21", "");
}

#[test]
fn record_field_access_sample_runs() {
    assert_sample_runs("samples/record_field_access.muga", "8080", "");
}

#[test]
fn record_counter_loop_sample_runs() {
    assert_sample_runs("samples/record_counter_loop.muga", "5", "");
}

#[test]
fn nested_record_access_sample_runs() {
    assert_sample_runs("samples/nested_record_access.muga", "101", "");
}

#[test]
fn method_chain_user_sample_runs() {
    assert_sample_runs("samples/method_chain_user.muga", "24", "");
}

#[test]
fn record_user_sample_runs() {
    assert_sample_runs("samples/record_user.muga", "Ada", "");
}

#[test]
fn number_chain_sample_runs() {
    assert_sample_runs("samples/number_chain.muga", "4", "");
}

#[test]
fn println_chain_sample_runs() {
    assert_sample_runs("samples/println_chain.muga", "10", "5\n");
}

#[test]
fn mixed_chain_pipeline_sample_runs() {
    assert_sample_runs("samples/mixed_chain_pipeline.muga", "24", "");
}

#[test]
fn higher_order_functions_sample_runs() {
    assert_sample_runs("samples/higher_order_functions.muga", "22", "");
}

#[test]
fn higher_order_local_inference_sample_runs() {
    assert_sample_runs("samples/higher_order_local_inference.muga", "35", "");
}

#[test]
fn higher_order_explicit_arrow_sample_runs() {
    assert_sample_runs("samples/higher_order_explicit_arrow.muga", "big", "big\n");
}

#[test]
fn print_and_println_can_be_mixed() {
    assert_sample_runs("samples/print_then_println.muga", "10", "value = 10 done\n");
}

#[test]
fn package_entry_passes_frontend() {
    let result = muga::check_path(Path::new("samples/packages/app/main/main.muga"));
    assert!(result.is_ok(), "{:#?}", result.err());
}

#[test]
fn package_entry_runs() {
    assert_package_runs("samples/packages/app/main/main.muga", "23", "");
}

#[test]
fn package_alias_demo_runs() {
    assert_package_runs("samples/packages/app/alias_demo/main.muga", "112", "");
}

#[test]
fn package_public_function_requires_explicit_signature() {
    let diagnostics =
        muga::check_path(Path::new("samples/packages_invalid/app/bad/main.muga")).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "PK011"),
        "{diagnostics:#?}"
    );
}

#[test]
fn package_import_alias_conflict_is_rejected() {
    let diagnostics = muga::check_path(Path::new(
        "samples/packages_invalid/app/import_alias_conflict/main.muga",
    ))
    .unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic.code == "PK007"),
        "{diagnostics:#?}"
    );
}

#[test]
fn compile_source_lowers_functions_into_hir_table() {
    let source = r#"
fn main(): Int {
  add = fn(x: Int): Int {
    x + 1
  }
  add(41)
}
"#;
    let program = muga::compile_source(source).unwrap();
    assert_eq!(program.functions.len(), 2);
    assert_eq!(
        program.functions[0]
            .name
            .map(|symbol| program.symbols.resolve(symbol)),
        Some("main")
    );
    assert_eq!(program.functions[1].name, None);
}

#[test]
fn compile_bytecode_source_emits_function_definitions_in_entry_chunk() {
    let source = r#"
fn helper(): Int {
  1
}

fn main(): Int {
  helper()
}
"#;
    let program = muga::compile_bytecode_source(source).unwrap();
    assert_eq!(program.functions.len(), 2);
    assert!(matches!(
        program.entry.instructions.first(),
        Some(Instruction::DefineFunction { name, .. })
            if program.symbols.resolve(*name) == "helper"
    ));
    assert!(matches!(
        program.entry.instructions.get(1),
        Some(Instruction::DefineFunction { name, .. })
            if program.symbols.resolve(*name) == "main"
    ));
}

#[test]
fn compile_source_reuses_one_symbol_for_repeated_name() {
    let source = r#"
fn main(): Int {
  value = 1
  value
}
"#;
    let program = muga::compile_source(source).unwrap();
    let function = &program.functions[0];
    let value_symbol = match &function.body.statements[0] {
        muga::hir::Stmt::Assign(stmt) => stmt.name,
        _ => panic!("expected assign statement"),
    };
    let final_symbol = match function.body.expr.as_ref() {
        muga::hir::Expr::Ident(expr) => expr.name,
        _ => panic!("expected final identifier"),
    };
    assert_eq!(value_symbol, final_symbol);
    assert_eq!(program.symbols.resolve(value_symbol), "value");
}

#[test]
fn closures_capture_outer_bindings() {
    let source = r#"
fn main(): Int {
  base = 41
  add = fn(x: Int): Int {
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
fn even(n: Int): Bool {
  if n == 0 {
    true
  } else {
    odd(n - 1)
  }
}

fn odd(n: Int): Bool {
  if n == 0 {
    false
  } else {
    even(n - 1)
  }
}

fn main(): Bool {
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
fn main(): Int {
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

fn assert_sample_runs(path: &str, expected_main: &str, expected_output: &str) {
    let source = fs::read_to_string(path).unwrap();
    let result = muga::run_source(&source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), expected_main, "sample: {path}");
    assert_eq!(result.output_text, expected_output, "sample: {path}");
}

fn assert_package_runs(path: &str, expected_main: &str, expected_output: &str) {
    let result = muga::run_path(Path::new(path)).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), expected_main, "package sample: {path}");
    assert_eq!(
        result.output_text, expected_output,
        "package sample: {path}"
    );
}
