use std::{collections::HashSet, fs, path::Path};

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
fn slash_slash_comments_are_accepted() {
    let source = r#"
fn main(): Int {
  value = 1 // trailing comment
  // full-line comment
  value
}
"#;
    let result = muga::check_source(source);
    assert!(result.is_ok(), "{:#?}", result.err());
}

#[test]
fn hash_comments_are_rejected() {
    let source = r#"
fn main(): Int {
  value = 1 # old comment syntax
  value
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "L001"),
        "{diagnostics:#?}"
    );
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
fn package_loader_renumbers_statement_ids_after_flattening() {
    let program = muga::check_path(Path::new("samples/packages/app/main/main.muga")).unwrap();
    let mut ids = HashSet::new();
    collect_stmt_ids(&program.statements, &mut ids);
    assert!(
        ids.len() > 1,
        "package sample should contain multiple statements"
    );
}

#[test]
fn package_loader_exposes_package_symbol_graph() {
    let loaded =
        muga::package::load_from_entry(Path::new("samples/packages/app/main/main.muga")).unwrap();
    let graph = loaded.package_graph;

    let app = graph
        .package_id("app::main")
        .expect("app package should exist");
    let numbers = graph
        .package_id("util::numbers")
        .expect("numbers package should exist");
    let users = graph
        .package_id("util::users")
        .expect("users package should exist");

    let app_info = graph.package(app).expect("app package info should exist");
    assert!(
        app_info
            .imports
            .iter()
            .any(|import| import.alias == "numbers" && import.package == numbers)
    );
    assert!(
        app_info
            .imports
            .iter()
            .any(|import| import.alias == "users" && import.package == users)
    );

    let inc_twice = graph
        .item_id(
            numbers,
            "inc_twice",
            muga::package::PackageItemKind::Function,
        )
        .expect("inc_twice should exist");
    let inc_twice = graph.item(inc_twice).expect("inc_twice info should exist");
    assert_eq!(inc_twice.visibility, muga::ast::Visibility::Public);
    assert_eq!(
        inc_twice.mangled_name,
        "__muga_pkg__util__numbers__inc_twice"
    );

    let user = graph
        .item_id(users, "User", muga::package::PackageItemKind::Record)
        .expect("User record should exist");
    let user = graph.item(user).expect("User info should exist");
    assert_eq!(user.visibility, muga::ast::Visibility::Public);
    assert_eq!(user.mangled_name, "__muga_pkg__util__users__User");
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
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "PK007"),
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
fn compile_typed_source_exposes_resolved_bindings_and_types() {
    let source = r#"
fn main(): Int {
  value = 1
  value
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let main = match &program.statements[0] {
        muga::typed_hir::Stmt::Function(function) => function,
        _ => panic!("expected typed function"),
    };
    assert_eq!(main.return_ty, muga::typing::TypeInfo::Int);

    let assign = match &main.body.statements[0] {
        muga::typed_hir::Stmt::Assign(assign) => assign,
        _ => panic!("expected typed assignment"),
    };
    assert!(!assign.is_update);
    assert_eq!(assign.value.ty, muga::typing::TypeInfo::Int);

    let final_ident = match &main.body.expr.kind {
        muga::typed_hir::ExprKind::Ident(ident) => ident,
        _ => panic!("expected typed identifier"),
    };
    assert_eq!(final_ident.binding, assign.binding);
    assert_eq!(main.body.expr.ty, muga::typing::TypeInfo::Int);
}

#[test]
fn compile_typed_source_marks_mutable_updates() {
    let source = r#"
fn main(): Int {
  mut value = 1
  value = 2
  value
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let main = match &program.statements[0] {
        muga::typed_hir::Stmt::Function(function) => function,
        _ => panic!("expected typed function"),
    };
    let first = match &main.body.statements[0] {
        muga::typed_hir::Stmt::Assign(assign) => assign,
        _ => panic!("expected first assignment"),
    };
    let second = match &main.body.statements[1] {
        muga::typed_hir::Stmt::Assign(assign) => assign,
        _ => panic!("expected second assignment"),
    };
    assert!(!first.is_update);
    assert!(second.is_update);
    assert_eq!(first.binding, second.binding);
}

#[test]
fn compile_typed_path_preserves_package_symbol_graph() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    assert!(program.package_graph.package_id("app::main").is_some());
    assert!(program.package_graph.package_id("util::numbers").is_some());
    assert!(program.package_graph.package_id("util::users").is_some());
}

#[test]
fn resolver_exposes_identifier_binding_identity() {
    let source = r#"
fn main(): Int {
  value = 1
  value
}
"#;
    let program = parse_source(source);
    let output = muga::resolver::resolve_program(&program);
    assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);

    let value_binding = output
        .bindings
        .iter()
        .find(|binding| output.symbols.resolve(binding.symbol) == "value")
        .expect("value binding should be exposed");
    assert_eq!(value_binding.kind, muga::identity::BindingKind::Immutable);

    let value_ref = output
        .identifier_refs
        .iter()
        .find(|identifier| output.symbols.resolve(identifier.name) == "value")
        .expect("value identifier use should be exposed");
    assert_eq!(value_ref.binding, value_binding.id);
    assert_eq!(value_ref.expr_id.as_u32(), 1);
}

#[test]
fn typechecker_exposes_identifier_and_expression_types() {
    let source = r#"
fn main(): Int {
  value = 1
  value
}
"#;
    let program = parse_source(source);
    let output = muga::typing::typecheck_program(&program);
    assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);

    let value_binding = output
        .bindings
        .iter()
        .find(|binding| output.symbols.resolve(binding.symbol) == "value")
        .expect("value binding should be exposed");
    assert_eq!(value_binding.ty, muga::typing::TypeInfo::Int);

    let value_ref = output
        .identifier_refs
        .iter()
        .find(|identifier| output.symbols.resolve(identifier.name) == "value")
        .expect("value identifier use should be exposed");
    assert_eq!(value_ref.binding, value_binding.id);

    let value_expr_type = output
        .expr_types
        .iter()
        .find(|expr_type| expr_type.expr_id == value_ref.expr_id)
        .expect("value expression type should be exposed");
    assert_eq!(value_expr_type.ty, muga::typing::TypeInfo::Int);
}

#[test]
fn parser_assigns_stable_expression_and_statement_ids() {
    let source = r#"
fn main(): Int {
  value = 1
  value + 2
}
"#;
    let program = parse_source(source);
    let main = match &program.statements[0] {
        muga::ast::Stmt::FuncDecl(func) => func,
        _ => panic!("expected function declaration"),
    };
    let assign = match &main.body.statements[0] {
        muga::ast::Stmt::Assign(assign) => assign,
        _ => panic!("expected assignment"),
    };
    let final_expr = main.body.expr.as_ref();

    assert_eq!(main.id.as_u32(), 2);
    assert_eq!(assign.id.as_u32(), 0);
    assert_eq!(assign.value.id().as_u32(), 0);
    assert_eq!(final_expr.id().as_u32(), 3);
}

#[test]
fn typechecker_output_resolves_late_inferred_function_types() {
    let source = r#"
fn apply(x: Int, f): Int {
  f(x)
}

fn inc(x: Int): Int {
  x + 1
}

fn main(): Int {
  apply(10, inc)
}
"#;
    let program = parse_source(source);
    let output = muga::typing::typecheck_program(&program);
    assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);

    let f_binding = output
        .bindings
        .iter()
        .find(|binding| output.symbols.resolve(binding.symbol) == "f")
        .expect("f binding should be exposed");

    assert_eq!(
        f_binding.ty,
        muga::typing::TypeInfo::Function(muga::typing::FunctionTypeInfo {
            params: vec![muga::typing::TypeInfo::Int],
            ret: Box::new(muga::typing::TypeInfo::Int),
        })
    );
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

fn parse_source(source: &str) -> muga::ast::Program {
    let tokens = muga::lexer::lex(source).unwrap();
    muga::parser::parse(tokens).unwrap()
}

fn collect_stmt_ids(statements: &[muga::ast::Stmt], ids: &mut HashSet<u32>) {
    for statement in statements {
        assert!(
            ids.insert(statement.id().as_u32()),
            "duplicate statement id: {}",
            statement.id().as_u32()
        );
        match statement {
            muga::ast::Stmt::FuncDecl(func) => {
                collect_stmt_ids(&func.body.statements, ids);
            }
            muga::ast::Stmt::If(stmt) => {
                collect_stmt_ids(&stmt.then_branch.statements, ids);
                if let Some(else_branch) = &stmt.else_branch {
                    collect_stmt_ids(&else_branch.statements, ids);
                }
            }
            muga::ast::Stmt::While(stmt) => {
                collect_stmt_ids(&stmt.body.statements, ids);
            }
            _ => {}
        }
    }
}
