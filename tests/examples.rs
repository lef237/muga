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
fn crlf_newlines_are_counted_once() {
    let source = "fn main(): Int {\r\n  value = @\r\n  value\r\n}\r\n";
    let diagnostics = muga::check_source(source).unwrap_err();
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "L001")
        .expect("lexer diagnostic should exist");
    assert_eq!(diagnostic.span.start.line, 2, "{diagnostic:#?}");
}

#[test]
fn diagnostic_display_without_notes_stays_single_line() {
    let diagnostic = muga::diagnostic::Diagnostic::new(
        "X001",
        "example diagnostic",
        muga::span::Span::new(
            muga::span::Position::new(1, 2),
            muga::span::Position::new(1, 9),
        ),
    );
    assert_eq!(diagnostic.to_string(), "1:2: X001 example diagnostic");
}

#[test]
fn diagnostic_display_includes_related_notes_and_suggestions() {
    let diagnostic = muga::diagnostic::Diagnostic::new(
        "X002",
        "primary",
        muga::span::Span::new(
            muga::span::Position::new(1, 1),
            muga::span::Position::new(1, 8),
        ),
    )
    .with_related(
        "related",
        muga::span::Span::new(
            muga::span::Position::new(2, 3),
            muga::span::Position::new(2, 10),
        ),
    )
    .with_suggestion("try this");

    assert_eq!(
        diagnostic.to_string(),
        "1:1: X002 primary\n  note: 2:3: related\n  help: try this"
    );
}

#[test]
fn package_mode_syntax_suggests_package_entry() {
    let source = r#"
pub fn helper(): Int {
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "P014")
        .expect("P014 diagnostic should exist");
    assert!(
        diagnostic
            .suggestions
            .iter()
            .any(|suggestion| suggestion.message.contains("muga.toml")
                && suggestion.message.contains("package")),
        "{diagnostic:#?}"
    );
}

#[test]
fn import_in_script_suggests_package_entry() {
    let source = r#"
import my_service::users

fn main(): Int {
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "P014")
        .expect("P014 diagnostic should exist");
    assert!(
        diagnostic
            .suggestions
            .iter()
            .any(|suggestion| suggestion.message.contains("muga.toml")
                && suggestion.message.contains("package")),
        "{diagnostic:#?}"
    );
}

#[test]
fn resolver_duplicate_binding_diagnostic_points_to_previous_binding() {
    let source = r#"
fn main(): Int {
  mut value = 1
  mut value = 2
  value
}
"#;
    let program = parse_source(source);
    let output = muga::resolver::resolve_program(&program);
    let diagnostic = output
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "E002")
        .expect("duplicate binding diagnostic should exist");
    assert!(
        diagnostic
            .related
            .iter()
            .any(|note| note.message.contains("previous binding")),
        "{diagnostic:#?}"
    );
}

#[test]
fn typechecker_record_literal_mismatch_points_to_field_declaration() {
    let source = r#"
record User {
  age: Int
}

fn main(): User {
  User {
    age: "old"
  }
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "E009")
        .expect("record literal diagnostic should exist");
    assert!(
        diagnostic
            .related
            .iter()
            .any(|note| note.message.contains("field type")),
        "{diagnostic:#?}"
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
fn closure_capture_sample_runs() {
    assert_sample_runs("samples/closure_capture.muga", "42", "");
}

#[test]
fn inferred_types_sample_runs() {
    assert_sample_runs("samples/inferred_types.muga", "10", "10\n");
}

#[test]
fn local_inferred_equality_sample_runs() {
    assert_sample_runs("samples/local_inferred_equality.muga", "true", "");
}

#[test]
fn no_main_sample_runs() {
    assert_sample_without_main_runs("samples/no_main.muga", "7\n");
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
fn package_entry_reads_all_files_in_entry_directory() {
    assert_package_runs("samples/packages/app/split_main/main.muga", "42", "");
}

#[test]
fn package_module_visibility_allows_pkg_item_from_sibling_file() {
    assert_package_runs("samples/packages/app/module_visibility/main.muga", "42", "");
}

#[test]
fn package_module_private_item_from_sibling_file_is_rejected() {
    let diagnostics = muga::check_path(Path::new(
        "samples/packages_invalid/app/private_from_sibling/main.muga",
    ))
    .unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "PK015"),
        "{diagnostics:#?}"
    );
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "PK015")
        .expect("PK015 diagnostic should exist");
    assert!(
        diagnostic
            .related
            .iter()
            .any(|note| note.message.contains("module-private")),
        "{diagnostic:#?}"
    );
    assert!(
        diagnostic
            .suggestions
            .iter()
            .any(|suggestion| suggestion.message.contains("pkg")),
        "{diagnostic:#?}"
    );
}

#[test]
fn manifest_project_infers_package_paths_from_directories() {
    assert_package_runs(
        "samples/projects/my_service/src/main/main.muga",
        "21",
        "Ada\n",
    );
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
fn package_loader_exposes_package_export_graph() {
    let loaded =
        muga::package::load_from_entry(Path::new("samples/packages/app/main/main.muga")).unwrap();
    let graph = loaded.package_graph;
    let exports = loaded.package_exports;
    let numbers = graph
        .package_id("util::numbers")
        .expect("numbers package should exist");
    let users = graph
        .package_id("util::users")
        .expect("users package should exist");

    let inc_twice_item = graph
        .item_id(
            numbers,
            "inc_twice",
            muga::package::PackageItemKind::Function,
        )
        .expect("inc_twice item should exist");
    let inc_twice = exports
        .function_by_name(numbers, "inc_twice")
        .expect("inc_twice should be exported");
    assert_eq!(inc_twice.item, inc_twice_item);
    assert_eq!(
        inc_twice.mangled_name,
        "__muga_pkg__util__numbers__inc_twice"
    );

    let user_item = graph
        .item_id(users, "User", muga::package::PackageItemKind::Record)
        .expect("User item should exist");
    let user = exports
        .record_by_name(users, "User")
        .expect("User should be exported");
    assert_eq!(user.item, user_item);
    assert_eq!(user.mangled_name, "__muga_pkg__util__users__User");
}

#[test]
fn package_symbol_graph_exposes_module_identity() {
    let loaded = muga::package::load_from_entry(Path::new(
        "samples/packages/app/module_visibility/main.muga",
    ))
    .unwrap();
    let graph = &loaded.package_graph;
    let package = graph
        .package_id("app::module_visibility")
        .expect("package should exist");
    let package_info = graph.package(package).expect("package info should exist");

    let module_paths: HashSet<_> = package_info
        .modules
        .iter()
        .map(|module| {
            graph
                .module(*module)
                .expect("module should exist")
                .path
                .as_str()
        })
        .collect();
    assert!(module_paths.contains("main.muga"), "{module_paths:#?}");
    assert!(module_paths.contains("helper.muga"), "{module_paths:#?}");

    let helper_module = graph
        .module_id(package, "helper.muga")
        .expect("helper module should exist");
    let package_value = graph
        .item_id_in_module(
            helper_module,
            "PackageValue",
            muga::package::PackageItemKind::Record,
        )
        .expect("PackageValue should exist");
    let package_value = graph
        .item(package_value)
        .expect("PackageValue info should exist");
    assert_eq!(package_value.visibility, muga::ast::Visibility::Package);
    let helper = graph
        .item_id_in_module(
            helper_module,
            "helper",
            muga::package::PackageItemKind::Function,
        )
        .expect("helper should exist");
    let helper = graph.item(helper).expect("helper info should exist");
    assert_eq!(helper.visibility, muga::ast::Visibility::Package);
    let helper_module = graph
        .module(helper.module)
        .expect("helper module should exist");
    assert_eq!(helper_module.path, "helper.muga");
    assert!(
        loaded
            .package_exports
            .function_by_name(package, "helper")
            .is_none(),
        "pkg helper should not be exported"
    );
}

#[test]
fn typed_hir_generates_package_interface_summaries() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = program.package_interfaces();
    let numbers = program
        .package_graph
        .package_id("util::numbers")
        .expect("numbers package should exist");
    let users = program
        .package_graph
        .package_id("util::users")
        .expect("users package should exist");

    let inc_twice = interfaces
        .function_by_name(numbers, "inc_twice")
        .expect("inc_twice should be exported");
    assert_eq!(inc_twice.params.len(), 1);
    assert_eq!(inc_twice.params[0].ty, muga::typing::TypeInfo::Int);
    assert_eq!(inc_twice.ret, muga::typing::TypeInfo::Int);
    let singleton = interfaces
        .function_by_name(numbers, "singleton")
        .expect("singleton should be exported");
    assert_eq!(
        singleton.ret,
        muga::typing::TypeInfo::List(Box::new(muga::typing::TypeInfo::Int))
    );
    let singleton_len = interfaces
        .function_by_name(numbers, "singleton_len")
        .expect("singleton_len should be exported");
    assert_eq!(singleton_len.ret, muga::typing::TypeInfo::Int);
    let singleton_first = interfaces
        .function_by_name(numbers, "singleton_first")
        .expect("singleton_first should be exported");
    assert_eq!(singleton_first.ret, muga::typing::TypeInfo::Int);
    let singleton_get = interfaces
        .function_by_name(numbers, "singleton_get")
        .expect("singleton_get should be exported");
    assert_eq!(
        singleton_get.ret,
        muga::typing::TypeInfo::Option(Box::new(muga::typing::TypeInfo::Int))
    );
    let replace_singleton = interfaces
        .function_by_name(numbers, "replace_singleton")
        .expect("replace_singleton should be exported");
    assert_eq!(
        replace_singleton.ret,
        muga::typing::TypeInfo::List(Box::new(muga::typing::TypeInfo::Int))
    );
    let maybe_positive = interfaces
        .function_by_name(numbers, "maybe_positive")
        .expect("maybe_positive should be exported");
    assert_eq!(
        maybe_positive.ret,
        muga::typing::TypeInfo::Option(Box::new(muga::typing::TypeInfo::Int))
    );
    let value_or_zero = interfaces
        .function_by_name(numbers, "value_or_zero")
        .expect("value_or_zero should be exported");
    assert_eq!(
        value_or_zero.params[0].ty,
        muga::typing::TypeInfo::Option(Box::new(muga::typing::TypeInfo::Int))
    );
    assert_eq!(value_or_zero.ret, muga::typing::TypeInfo::Int);
    let singleton_map = interfaces
        .function_by_name(numbers, "singleton_map")
        .expect("singleton_map should be exported");
    assert_eq!(singleton_map.params[0].ty, muga::typing::TypeInfo::String);
    assert_eq!(singleton_map.params[1].ty, muga::typing::TypeInfo::Int);
    assert_eq!(
        singleton_map.ret,
        muga::typing::TypeInfo::Map(
            Box::new(muga::typing::TypeInfo::String),
            Box::new(muga::typing::TypeInfo::Int)
        )
    );
    let map_get_or_zero = interfaces
        .function_by_name(numbers, "map_get_or_zero")
        .expect("map_get_or_zero should be exported");
    assert_eq!(
        map_get_or_zero.params[0].ty,
        muga::typing::TypeInfo::Map(
            Box::new(muga::typing::TypeInfo::String),
            Box::new(muga::typing::TypeInfo::Int)
        )
    );
    assert_eq!(map_get_or_zero.params[1].ty, muga::typing::TypeInfo::String);
    assert_eq!(map_get_or_zero.ret, muga::typing::TypeInfo::Int);

    let user_item = program
        .package_graph
        .item_id(users, "User", muga::package::PackageItemKind::Record)
        .expect("User item should exist");
    let user_record = interfaces
        .record_by_name(users, "User")
        .expect("User should be exported");
    assert_eq!(user_record.item, user_item);
    assert!(
        user_record
            .fields
            .iter()
            .any(|field| field.name == "age" && field.ty == muga::typing::TypeInfo::Int),
        "{user_record:#?}"
    );

    let birthday = interfaces
        .function_by_name(users, "birthday")
        .expect("birthday should be exported");
    assert_eq!(birthday.params.len(), 1);
    assert!(
        matches!(
            &birthday.params[0].ty,
            muga::typing::TypeInfo::PackageRecord { item, .. } if *item == user_item
        ),
        "{birthday:#?}"
    );
    assert!(
        matches!(
            &birthday.ret,
            muga::typing::TypeInfo::PackageRecord { item, .. } if *item == user_item
        ),
        "{birthday:#?}"
    );
}

#[test]
fn package_export_graph_can_be_derived_from_typed_interfaces() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = program.package_interfaces();
    let symbol_exports =
        muga::package::PackageExportGraph::from_symbol_graph(&program.package_graph);
    let interface_exports =
        muga::package::PackageExportGraph::from_interfaces(&interfaces, &program.package_graph);

    assert_eq!(interface_exports, symbol_exports);
}

#[test]
fn typed_hir_validates_package_references_against_interfaces() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = program.package_interfaces();
    let diagnostics = program.validate_package_references_against_interfaces(&interfaces);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");

    let numbers = program
        .package_graph
        .package_id("util::numbers")
        .expect("numbers package should exist");
    let mut broken_interfaces = interfaces.clone();
    broken_interfaces
        .packages
        .iter_mut()
        .find(|interface| interface.package == numbers)
        .expect("numbers interface should exist")
        .functions
        .retain(|function| function.name != "inc_twice");

    let diagnostics = program.validate_package_references_against_interfaces(&broken_interfaces);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "PK016")
        .expect("missing interface export diagnostic should exist");
    assert!(diagnostic.message.contains("inc_twice"), "{diagnostic:#?}");
    assert!(
        diagnostic
            .related
            .iter()
            .any(|note| note.message.contains("declared")),
        "{diagnostic:#?}"
    );
}

#[test]
fn typed_hir_validates_package_interfaces_by_export_name() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let numbers = program
        .package_graph
        .package_id("util::numbers")
        .expect("numbers package should exist");
    let users = program
        .package_graph
        .package_id("util::users")
        .expect("users package should exist");
    let mut interfaces = program.package_interfaces();
    interfaces
        .packages
        .iter_mut()
        .find(|interface| interface.package == numbers)
        .expect("numbers interface should exist")
        .functions
        .iter_mut()
        .find(|function| function.name == "inc_twice")
        .expect("inc_twice should be exported")
        .name = "inc_twice_old".to_string();
    interfaces
        .packages
        .iter_mut()
        .find(|interface| interface.package == users)
        .expect("users interface should exist")
        .records
        .iter_mut()
        .find(|record| record.name == "User")
        .expect("User should be exported")
        .name = "Account".to_string();

    let diagnostics = program.validate_package_references_against_interfaces(&interfaces);
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "PK016" && diagnostic.message.contains("function `inc_twice`")
        }),
        "{diagnostics:#?}"
    );
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "PK016" && diagnostic.message.contains("record `User`")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn typed_hir_rejects_stale_package_interface_item_identity() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let numbers = program
        .package_graph
        .package_id("util::numbers")
        .expect("numbers package should exist");
    let inc = program
        .package_graph
        .item_id(numbers, "inc", muga::package::PackageItemKind::Function)
        .expect("inc should exist");
    let mut interfaces = program.package_interfaces();
    interfaces
        .packages
        .iter_mut()
        .find(|interface| interface.package == numbers)
        .expect("numbers interface should exist")
        .functions
        .iter_mut()
        .find(|function| function.name == "inc_twice")
        .expect("inc_twice should be exported")
        .item = inc;

    let diagnostics = program.validate_package_references_against_interfaces(&interfaces);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "PK017")
        .expect("stale interface diagnostic should exist");
    assert!(
        diagnostic.message.contains("function identity"),
        "{diagnostic:#?}"
    );
}

#[test]
fn typed_hir_rejects_stale_package_interface_signatures() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let numbers = program
        .package_graph
        .package_id("util::numbers")
        .expect("numbers package should exist");
    let mut interfaces = program.package_interfaces();
    interfaces
        .packages
        .iter_mut()
        .find(|interface| interface.package == numbers)
        .expect("numbers interface should exist")
        .functions
        .iter_mut()
        .find(|function| function.name == "inc_twice")
        .expect("inc_twice should be exported")
        .ret = muga::typing::TypeInfo::String;

    let diagnostics = program.validate_package_references_against_interfaces(&interfaces);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "PK017")
        .expect("stale interface diagnostic should exist");
    assert!(
        diagnostic.message.contains("function signature"),
        "{diagnostic:#?}"
    );
    assert!(
        diagnostic
            .suggestions
            .iter()
            .any(|suggestion| suggestion.message.contains("regenerate")),
        "{diagnostic:#?}"
    );
}

#[test]
fn typed_hir_rejects_stale_package_interface_record_shapes() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let users = program
        .package_graph
        .package_id("util::users")
        .expect("users package should exist");
    let mut interfaces = program.package_interfaces();
    interfaces
        .packages
        .iter_mut()
        .find(|interface| interface.package == users)
        .expect("users interface should exist")
        .records
        .iter_mut()
        .find(|record| record.name == "User")
        .expect("User should be exported")
        .fields
        .iter_mut()
        .find(|field| field.name == "age")
        .expect("age field should exist")
        .ty = muga::typing::TypeInfo::String;

    let diagnostics = program.validate_package_references_against_interfaces(&interfaces);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "PK017")
        .expect("stale interface diagnostic should exist");
    assert!(
        diagnostic.message.contains("record shape"),
        "{diagnostic:#?}"
    );
}

#[test]
fn package_interface_summaries_exclude_pkg_items() {
    let program = muga::compile_typed_path(Path::new(
        "samples/packages/app/module_visibility/main.muga",
    ))
    .expect("typed package compilation should pass");
    let interfaces = program.package_interfaces();
    let package = program
        .package_graph
        .package_id("app::module_visibility")
        .expect("package should exist");
    let interface = interfaces.package(package).expect("interface should exist");
    assert!(
        !interface
            .functions
            .iter()
            .any(|function| function.name == "helper"),
        "{interface:#?}"
    );
    assert!(
        !interface
            .records
            .iter()
            .any(|record| record.name == "PackageValue"),
        "{interface:#?}"
    );
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
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "PK011")
        .expect("PK011 diagnostic should exist");
    assert!(
        diagnostic
            .suggestions
            .iter()
            .any(|suggestion| suggestion.message.contains("return type")),
        "{diagnostic:#?}"
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
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "PK007")
        .expect("PK007 diagnostic should exist");
    assert!(
        diagnostic
            .related
            .iter()
            .any(|note| note.message.contains("previous import")),
        "{diagnostic:#?}"
    );
    assert!(
        diagnostic
            .suggestions
            .iter()
            .any(|suggestion| suggestion.message.contains("as")),
        "{diagnostic:#?}"
    );
}

#[test]
fn package_imports_are_resolved_through_export_surface() {
    let diagnostics = muga::check_path(Path::new(
        "samples/packages_invalid/app/import_pkg_item/main.muga",
    ))
    .unwrap_err();
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.code == "PK010"
                && diagnostic
                    .message
                    .contains("does not export function `helper`")
        })
        .expect("PK010 diagnostic should exist");
    assert!(
        diagnostic
            .related
            .iter()
            .any(|note| note.message.contains("not public")),
        "{diagnostic:#?}"
    );
    assert!(
        diagnostic
            .suggestions
            .iter()
            .any(|suggestion| suggestion.message.contains("pub")),
        "{diagnostic:#?}"
    );
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.code == "PK010"
                && diagnostic
                    .message
                    .contains("does not export record `PackageValue`")
        })
        .expect("PK010 record diagnostic should exist");
    assert!(
        diagnostic
            .related
            .iter()
            .any(|note| note.message.contains("not public")),
        "{diagnostic:#?}"
    );
    assert!(
        diagnostic
            .suggestions
            .iter()
            .any(|suggestion| suggestion.message.contains("pub")),
        "{diagnostic:#?}"
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
fn parser_accepts_generic_type_expression_in_local_annotation() {
    let source = r#"
fn main(): Int {
  items: List[Int] = []
  1
}
"#;
    let program = parse_source(source);
    let main = match &program.statements[0] {
        muga::ast::Stmt::FuncDecl(function) => function,
        _ => panic!("expected function"),
    };
    let assign = match &main.body.statements[0] {
        muga::ast::Stmt::Assign(assign) => assign,
        _ => panic!("expected assignment"),
    };
    let generic = match assign.type_name.as_ref() {
        Some(muga::ast::TypeExpr::Generic(generic)) => generic,
        other => panic!("expected generic type expression, got {other:#?}"),
    };
    assert_eq!(generic.name, "List");
    assert_eq!(generic.args.len(), 1);
    assert!(matches!(generic.args[0], muga::ast::TypeExpr::Int));
}

#[test]
fn parser_preserves_option_match_patterns_as_enum_variants() {
    let source = r#"
fn main(): Int {
  value: Option[Int] = Option::Some(1)
  match value {
    Option::Some(x) => x
    Option::None => 0
  }
}
"#;
    let program = parse_source(source);
    let main = match &program.statements[0] {
        muga::ast::Stmt::FuncDecl(function) => function,
        _ => panic!("expected function"),
    };
    let match_expr = match main.body.expr.as_ref() {
        muga::ast::Expr::Match(expr) => expr,
        other => panic!("expected match expression, got {other:#?}"),
    };
    let some = match &match_expr.arms[0].pattern {
        muga::ast::MatchPattern::Variant(pattern) => pattern,
    };
    assert_eq!(some.enum_name, "Option");
    assert_eq!(some.variant_name, "Some");
    assert_eq!(some.binding.as_deref(), Some("x"));

    let none = match &match_expr.arms[1].pattern {
        muga::ast::MatchPattern::Variant(pattern) => pattern,
    };
    assert_eq!(none.enum_name, "Option");
    assert_eq!(none.variant_name, "None");
    assert_eq!(none.binding, None);
}

#[test]
fn known_enum_metadata_describes_option_variants() {
    let option = muga::known_enum::option_enum();
    let some = option
        .variant(muga::known_enum::OPTION_SOME_NAME)
        .expect("Option should define Some");
    let none = option
        .variant(muga::known_enum::OPTION_NONE_NAME)
        .expect("Option should define None");

    assert_eq!(option.name, "Option");
    assert!(some.has_payload);
    assert!(!none.has_payload);
    assert_eq!(option.qualified_variant(some), "Option::Some");
    assert_eq!(option.qualified_variant(none), "Option::None");
}

#[test]
fn local_binding_annotation_sets_binding_type() {
    let source = r#"
fn main(): Int {
  value: Int = 1
  value
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let main = match &program.statements[0] {
        muga::typed_hir::Stmt::Function(function) => function,
        _ => panic!("expected typed function"),
    };
    let assign = match &main.body.statements[0] {
        muga::typed_hir::Stmt::Assign(assign) => assign,
        _ => panic!("expected typed assignment"),
    };
    let binding = program
        .bindings
        .iter()
        .find(|binding| binding.id == assign.binding)
        .expect("assignment binding should exist");
    assert_eq!(binding.ty, muga::typing::TypeInfo::Int);
}

#[test]
fn local_binding_annotation_rejects_mismatch() {
    let source = r#"
fn main(): Int {
  value: Int = "one"
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn local_binding_annotation_on_update_is_rejected() {
    let source = r#"
fn main(): Int {
  mut value: Int = 1
  value: Int = 2
  value
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T014"),
        "{diagnostics:#?}"
    );
}

#[test]
fn unsupported_generic_type_expression_is_reserved() {
    let source = r#"
fn main(): Int {
  items: Set[Int] = 1
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T013"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_literal_sample_runs() {
    let source = r#"
fn main(): List[Int] {
  [1, 2, 3]
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "[1, 2, 3]");
}

#[test]
fn empty_list_literal_uses_local_annotation() {
    let source = r#"
fn main(): List[Int] {
  items: List[Int] = []
  items
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "[]");
}

#[test]
fn typed_hir_preserves_list_type_info() {
    let source = r#"
fn main(): List[Int] {
  items: List[Int] = [1]
  items
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let main = match &program.statements[0] {
        muga::typed_hir::Stmt::Function(function) => function,
        _ => panic!("expected typed function"),
    };
    assert_eq!(
        main.return_ty,
        muga::typing::TypeInfo::List(Box::new(muga::typing::TypeInfo::Int))
    );
    let assign = match &main.body.statements[0] {
        muga::typed_hir::Stmt::Assign(assign) => assign,
        _ => panic!("expected typed assignment"),
    };
    let binding = program
        .bindings
        .iter()
        .find(|binding| binding.id == assign.binding)
        .expect("assignment binding should exist");
    assert_eq!(
        binding.ty,
        muga::typing::TypeInfo::List(Box::new(muga::typing::TypeInfo::Int))
    );
}

#[test]
fn empty_list_literal_requires_expected_type() {
    let source = r#"
fn main(): Int {
  items = []
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T015"),
        "{diagnostics:#?}"
    );
}

#[test]
fn empty_list_literal_requires_list_expected_type() {
    let source = r#"
fn main(): Int {
  []
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T015"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_literal_items_must_share_one_type() {
    let source = r#"
fn main(): Int {
  items = [1, "two"]
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_len_sample_runs() {
    let source = r#"
fn main(): Int {
  [1, 2, 3].len()
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "3");
}

#[test]
fn list_is_empty_sample_runs() {
    let source = r#"
fn main(): Bool {
  items: List[Int] = []
  items.is_empty()
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "true");
}

#[test]
fn list_push_sample_runs() {
    let source = r#"
fn main(): List[Int] {
  [1, 2].push(3)
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "[1, 2, 3]");
}

#[test]
fn empty_list_push_infers_element_type() {
    let source = r#"
fn main(): List[Int] {
  [].push(1)
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "[1]");
}

#[test]
fn list_push_checks_value_type() {
    let source = r#"
fn main(): List[Int] {
  [1].push("two")
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_len_requires_list_argument() {
    let source = r#"
fn main(): Int {
  1.len()
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T006"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_get_some_sample_runs() {
    let source = r#"
fn main(): Int {
  match [10, 20].get(1) {
    Option::Some(x) => x
    Option::None => 0
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "20");
}

#[test]
fn list_get_none_sample_runs() {
    let source = r#"
fn main(): Int {
  match [10].get(2) {
    Option::Some(x) => x
    Option::None => 0
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "0");
}

#[test]
fn list_get_negative_index_returns_none() {
    let source = r#"
fn main(): Int {
  match [10].get(-1) {
    Option::Some(x) => x
    Option::None => 0
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "0");
}

#[test]
fn empty_list_get_infers_from_expected_option() {
    let source = r#"
fn main(): Int {
  value: Option[Int] = [].get(0)
  match value {
    Option::Some(x) => x
    Option::None => 0
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "0");
}

#[test]
fn list_get_checks_index_type() {
    let source = r#"
fn main(): Option[Int] {
  [1].get("bad")
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_get_checks_expected_option_type() {
    let source = r#"
fn main(): Option[String] {
  [1].get(0)
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_get_requires_list_argument() {
    let source = r#"
fn main(): Int {
  bad = 1.get(0)
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T006"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_index_sample_runs() {
    let source = r#"
fn main(): Int {
  [10, 20][1]
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "20");
}

#[test]
fn list_index_checks_index_type() {
    let source = r#"
fn main(): Int {
  [1]["bad"]
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_index_requires_list_base() {
    let source = r#"
fn main(): Int {
  bad = 1[0]
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T006"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_index_negative_reports_runtime_error() {
    let source = r#"
fn main(): Int {
  [10][-1]
}
"#;
    let diagnostics = muga::run_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "R020"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_index_out_of_bounds_reports_runtime_error() {
    let source = r#"
fn main(): Int {
  [10][1]
}
"#;
    let diagnostics = muga::run_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "R020"),
        "{diagnostics:#?}"
    );
}

#[test]
fn empty_list_index_infers_from_expected_type() {
    let source = r#"
fn main(): Int {
  [][0]
}
"#;
    let diagnostics = muga::run_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "R020"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_set_sample_runs() {
    let source = r#"
fn main(): List[Int] {
  [1, 2, 3].set(1, 99)
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "[1, 99, 3]");
}

#[test]
fn empty_list_set_infers_element_type() {
    let source = r#"
fn main(): List[Int] {
  [].push(0).set(0, 1)
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "[1]");
}

#[test]
fn list_set_checks_index_type() {
    let source = r#"
fn main(): List[Int] {
  [1].set("bad", 2)
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_set_checks_value_type() {
    let source = r#"
fn main(): List[Int] {
  [1].set(0, "bad")
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_set_requires_list_argument() {
    let source = r#"
fn main(): Int {
  bad = 1.set(0, 2)
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T006"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_set_negative_index_reports_runtime_error() {
    let source = r#"
fn main(): List[Int] {
  [10].set(-1, 20)
}
"#;
    let diagnostics = muga::run_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "R020"),
        "{diagnostics:#?}"
    );
}

#[test]
fn list_set_out_of_bounds_reports_runtime_error() {
    let source = r#"
fn main(): List[Int] {
  [10].set(1, 20)
}
"#;
    let diagnostics = muga::run_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "R020"),
        "{diagnostics:#?}"
    );
}

#[test]
fn option_some_match_sample_runs() {
    let source = r#"
fn main(): Int {
  value: Option[Int] = Option::Some(10)
  match value {
    Option::Some(x) => x
    Option::None => 0
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "10");
}

#[test]
fn option_none_match_sample_runs() {
    let source = r#"
fn main(): Int {
  value: Option[Int] = Option::None
  match value {
    Option::Some(x) => x
    Option::None => 0
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "0");
}

#[test]
fn option_none_requires_expected_type() {
    let source = r#"
fn main(): Int {
  value = Option::None
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T017"),
        "{diagnostics:#?}"
    );
}

#[test]
fn option_some_checks_expected_type() {
    let source = r#"
fn main(): Option[Int] {
  Option::Some("bad")
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn option_match_requires_some_and_none_arms() {
    let source = r#"
fn main(): Int {
  value: Option[Int] = Option::Some(1)
  match value {
    Option::Some(x) => x
  }
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T018"),
        "{diagnostics:#?}"
    );
}

#[test]
fn option_match_arm_types_must_match() {
    let source = r#"
fn main(): Int {
  value: Option[Int] = Option::None
  match value {
    Option::Some(x) => x
    Option::None => "missing"
  }
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn typed_hir_preserves_option_type_info() {
    let source = r#"
fn main(): Option[Int] {
  value: Option[Int] = Option::Some(1)
  value
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let main = match &program.statements[0] {
        muga::typed_hir::Stmt::Function(function) => function,
        _ => panic!("expected typed function"),
    };
    assert_eq!(
        main.return_ty,
        muga::typing::TypeInfo::Option(Box::new(muga::typing::TypeInfo::Int))
    );
    let assign = match &main.body.statements[0] {
        muga::typed_hir::Stmt::Assign(assign) => assign,
        _ => panic!("expected typed assignment"),
    };
    let binding = program
        .bindings
        .iter()
        .find(|binding| binding.id == assign.binding)
        .expect("assignment binding should exist");
    assert_eq!(
        binding.ty,
        muga::typing::TypeInfo::Option(Box::new(muga::typing::TypeInfo::Int))
    );
}

#[test]
fn typed_hir_preserves_option_match_patterns_as_enum_variants() {
    let source = r#"
fn main(): Int {
  value: Option[Int] = Option::Some(1)
  match value {
    Option::Some(x) => x
    Option::None => 0
  }
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let main = match &program.statements[0] {
        muga::typed_hir::Stmt::Function(function) => function,
        _ => panic!("expected typed function"),
    };
    let match_expr = match &main.body.expr.kind {
        muga::typed_hir::ExprKind::Match(expr) => expr,
        other => panic!("expected typed match expression, got {other:#?}"),
    };
    let some = match &match_expr.arms[0].pattern {
        muga::typed_hir::MatchPattern::Variant(pattern) => pattern,
    };
    assert_eq!(some.enum_name, "Option");
    assert_eq!(some.variant_name, "Some");
    assert_eq!(some.binding_name.as_deref(), Some("x"));
    assert!(some.binding.is_some());

    let none = match &match_expr.arms[1].pattern {
        muga::typed_hir::MatchPattern::Variant(pattern) => pattern,
    };
    assert_eq!(none.enum_name, "Option");
    assert_eq!(none.variant_name, "None");
    assert_eq!(none.binding_name, None);
    assert_eq!(none.binding, None);
}

#[test]
fn option_runtime_display_uses_enum_value_shape() {
    let source = r#"
fn main(): Option[Int] {
  Option::Some(1)
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "Option::Some(1)");
}

#[test]
fn empty_map_uses_local_annotation() {
    let source = r#"
fn main(): Map[String, Int] {
  items: Map[String, Int] = Map.empty()
  items
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "Map {}");
}

#[test]
fn map_insert_get_some_sample_runs() {
    let source = r#"
fn main(): Int {
  ages: Map[String, Int] = Map.empty().insert("Ada", 20)
  match ages.get("Ada") {
    Option::Some(age) => age
    Option::None => 0
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "20");
}

#[test]
fn map_get_none_sample_runs() {
    let source = r#"
fn main(): Int {
  ages: Map[String, Int] = Map.empty()
  match ages.get("Grace") {
    Option::Some(age) => age
    Option::None => 0
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "0");
}

#[test]
fn empty_map_get_infers_from_expected_option() {
    let source = r#"
fn main(): Int {
  value: Option[Int] = Map.empty().get("missing")
  match value {
    Option::Some(age) => age
    Option::None => 0
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "0");
}

#[test]
fn empty_map_get_can_be_inferred_from_match_arms() {
    let source = r#"
fn main(): Int {
  match Map.empty().get("missing") {
    Option::Some(age) => age
    Option::None => 0
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "0");
}

#[test]
fn map_len_is_empty_contains_and_remove_sample_runs() {
    let source = r#"
fn main(): Int {
  empty: Map[String, Int] = Map.empty()
  ages = empty.insert("Ada", 20).insert("Grace", 30)
  without_ada = ages.remove("Ada")
  if empty.is_empty() {
    if ages.len() == 2 {
      if ages.contains("Ada") {
        if without_ada.contains("Ada") {
          -1
        } else {
          without_ada.len()
        }
      } else {
        -1
      }
    } else {
      -1
    }
  } else {
    -1
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "1");
}

#[test]
fn map_insert_replaces_existing_key() {
    let source = r#"
fn main(): Int {
  ages: Map[String, Int] = Map.empty().insert("Ada", 20).insert("Ada", 21)
  match ages.get("Ada") {
    Option::Some(age) => age
    Option::None => 0
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "21");
}

#[test]
fn map_empty_requires_expected_type() {
    let source = r#"
fn main(): Int {
  items = Map.empty()
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T019"),
        "{diagnostics:#?}"
    );
}

#[test]
fn map_type_requires_two_arguments() {
    let source = r#"
fn main(): Int {
  items: Map[String] = Map.empty()
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T019"),
        "{diagnostics:#?}"
    );
}

#[test]
fn map_key_type_must_be_scalar() {
    let source = r#"
fn main(): Int {
  items: Map[List[Int], Int] = Map.empty()
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T020"),
        "{diagnostics:#?}"
    );
}

#[test]
fn map_insert_checks_key_type() {
    let source = r#"
fn main(): Map[String, Int] {
  ages: Map[String, Int] = Map.empty()
  ages.insert(1, 20)
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn map_insert_rejects_invalid_inferred_key() {
    let source = r#"
fn main(): Int {
  items = Map.empty().insert([1], 20)
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T020"),
        "{diagnostics:#?}"
    );
}

#[test]
fn map_get_checks_expected_option_type() {
    let source = r#"
fn main(): Option[String] {
  ages: Map[String, Int] = Map.empty().insert("Ada", 20)
  ages.get("Ada")
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn map_empty_remove_requires_expected_type() {
    let source = r#"
fn main(): Int {
  items = Map.empty().remove("missing")
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T019"),
        "{diagnostics:#?}"
    );
}

#[test]
fn map_empty_contains_requires_expected_map_type() {
    let source = r#"
fn main(): Bool {
  Map.empty().contains("missing")
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T019"),
        "{diagnostics:#?}"
    );
}

#[test]
fn typed_hir_preserves_map_type_info() {
    let source = r#"
fn main(): Map[String, Int] {
  items: Map[String, Int] = Map.empty().insert("Ada", 20)
  items
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let main = match &program.statements[0] {
        muga::typed_hir::Stmt::Function(function) => function,
        _ => panic!("expected typed function"),
    };
    let map_ty = muga::typing::TypeInfo::Map(
        Box::new(muga::typing::TypeInfo::String),
        Box::new(muga::typing::TypeInfo::Int),
    );
    assert_eq!(main.return_ty, map_ty);
    let assign = match &main.body.statements[0] {
        muga::typed_hir::Stmt::Assign(assign) => assign,
        _ => panic!("expected typed assignment"),
    };
    let binding = program
        .bindings
        .iter()
        .find(|binding| binding.id == assign.binding)
        .expect("assignment binding should exist");
    assert_eq!(binding.ty, map_ty);
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
fn typechecker_output_resolves_direct_call_callee() {
    let source = r#"
fn inc(x: Int): Int {
  x + 1
}

fn main(): Int {
  inc(1)
}
"#;
    let program = parse_source(source);
    let output = muga::typing::typecheck_program(&program);
    assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);

    let inc = typecheck_binding_id(&output, "inc");
    assert!(
        output
            .calls
            .iter()
            .any(|call| call.callee == muga::typing::TypedCalleeInfo::Binding(inc)),
        "{:#?}",
        output.calls
    );
}

#[test]
fn typed_hir_preserves_direct_call_callee() {
    let source = r#"
fn inc(x: Int): Int {
  x + 1
}

fn main(): Int {
  inc(1)
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let inc = typed_binding_id(&program, "inc");
    let calls = collect_typed_calls(&program);
    assert!(
        calls.iter().any(|call| {
            call.origin == muga::typed_hir::CallOrigin::Ordinary
                && call.resolved_callee == muga::typing::TypedCalleeInfo::Binding(inc)
        }),
        "{calls:#?}"
    );
}

#[test]
fn typed_hir_preserves_local_function_value_call_callee() {
    let source = r#"
fn main(): Int {
  add = fn(x: Int): Int {
    x + 1
  }
  add(41)
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let add = typed_binding_id(&program, "add");
    let calls = collect_typed_calls(&program);
    assert!(
        calls
            .iter()
            .any(|call| call.resolved_callee == muga::typing::TypedCalleeInfo::Binding(add)),
        "{calls:#?}"
    );
}

#[test]
fn typed_hir_preserves_chained_call_origin() {
    let source = r#"
fn inc(x: Int): Int {
  x + 1
}

fn main(): Int {
  1.inc()
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let inc = typed_binding_id(&program, "inc");
    let calls = collect_typed_calls(&program);
    assert!(
        calls.iter().any(|call| {
            call.origin == muga::typed_hir::CallOrigin::Chained
                && call.resolved_callee == muga::typing::TypedCalleeInfo::Binding(inc)
        }),
        "{calls:#?}"
    );
}

#[test]
fn typed_hir_preserves_builtin_call_callee() {
    let source = r#"
fn main(): Int {
  println(1)
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let println = typed_binding_id(&program, "println");
    let calls = collect_typed_calls(&program);
    assert!(
        calls.iter().any(|call| {
            call.resolved_callee
                == muga::typing::TypedCalleeInfo::Builtin {
                    binding: println,
                    name: "println",
                }
        }),
        "{calls:#?}"
    );
}

#[test]
fn typed_hir_preserves_package_qualified_call_callee() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let inc_twice = typed_binding_id(&program, "__muga_pkg__util__numbers__inc_twice");
    let numbers = program
        .package_graph
        .package_id("util::numbers")
        .expect("numbers package should exist");
    let users = program
        .package_graph
        .package_id("util::users")
        .expect("users package should exist");
    let inc_twice_item = program
        .package_graph
        .item_id(
            numbers,
            "inc_twice",
            muga::package::PackageItemKind::Function,
        )
        .expect("inc_twice package item should exist");
    let user_item = program
        .package_graph
        .item_id(users, "User", muga::package::PackageItemKind::Record)
        .expect("User package item should exist");
    let calls = collect_typed_calls(&program);
    assert!(
        calls.iter().any(|call| {
            call.origin == muga::typed_hir::CallOrigin::QualifiedChained
                && call.resolved_callee
                    == muga::typing::TypedCalleeInfo::PackageItem {
                        binding: inc_twice,
                        item: inc_twice_item,
                    }
                && matches!(
                    &call.callee.kind,
                    muga::typed_hir::ExprKind::Ident(muga::typed_hir::IdentExpr {
                        target: muga::typed_hir::IdentTarget::PackageItem {
                            binding,
                            item,
                        },
                        ..
                    }) if *binding == inc_twice && *item == inc_twice_item
                )
        }),
        "{calls:#?}"
    );

    let user_binding = program
        .bindings
        .iter()
        .find(|binding| program.symbols.resolve(binding.symbol) == "user")
        .expect("local user binding should exist");
    assert!(
        matches!(
            &user_binding.ty,
            muga::typing::TypeInfo::PackageRecord { item, .. } if *item == user_item
        ),
        "{user_binding:#?}"
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

#[test]
fn runtime_reports_integer_overflow() {
    let source = r#"
fn main(): Int {
  9223372036854775807 + 1
}
"#;
    let diagnostics = muga::run_source(source).expect_err("expected runtime error");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "R019"),
        "{diagnostics:#?}"
    );
}

#[test]
fn i64_min_literal_is_accepted() {
    let source = r#"
fn main(): Int {
  -9223372036854775808
}
"#;
    let outcome = muga::run_source(source).expect("expected i64::MIN literal to evaluate");
    let value = outcome
        .main_result
        .expect("expected main to return a value");
    let muga::runtime::Value::Int(value) = value else {
        panic!("expected Int, got {value:?}");
    };
    assert_eq!(value, i64::MIN);
}

#[test]
fn negative_literal_keeps_member_access() {
    let source = r#"
record P { x: Int }
fn main(): Int {
  p = P { x: 7 }
  -p.x
}
"#;
    let outcome = muga::run_source(source).expect("expected -p.x to evaluate");
    let value = outcome
        .main_result
        .expect("expected main to return a value");
    let muga::runtime::Value::Int(value) = value else {
        panic!("expected Int, got {value:?}");
    };
    assert_eq!(value, -7);
}

#[test]
fn negative_int_literal_with_dot_is_not_combined() {
    let source = r#"
fn main(): Int {
  -1.x
}
"#;
    let diagnostics = muga::check_source(source).expect_err("expected type error");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T008"),
        "expected `-1.x` to flow through the unary path and report T008: {diagnostics:#?}"
    );
}

#[test]
fn negative_int_literal_with_call_is_not_combined() {
    let source = r#"
fn main(): Int {
  -1(2)
}
"#;
    let diagnostics = muga::check_source(source).expect_err("expected type error");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T005"),
        "expected `-1(2)` to flow through the unary path and report T005: {diagnostics:#?}"
    );
}

#[test]
fn negating_i64_min_overflows_at_runtime() {
    let source = r#"
fn main(): Int {
  --9223372036854775808
}
"#;
    let diagnostics = muga::run_source(source).expect_err("expected runtime overflow");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "R019"),
        "negating i64::MIN should overflow at runtime: {diagnostics:#?}"
    );
}

#[test]
fn ordinary_negative_literal_still_works() {
    let source = r#"
fn main(): Int {
  -123
}
"#;
    let outcome = muga::run_source(source).expect("ordinary negative literal should evaluate");
    let value = outcome
        .main_result
        .expect("expected main to return a value");
    let muga::runtime::Value::Int(value) = value else {
        panic!("expected Int, got {value:?}");
    };
    assert_eq!(value, -123);
}

#[test]
fn if_with_empty_body_is_not_parsed_as_record_literal() {
    let source = r#"
fn main(): Int {
  flag = true
  if flag {}
  0
}
"#;
    muga::check_source(source).expect("`if flag {}` must parse as an `if` statement");
}

#[test]
fn while_with_empty_body_is_not_parsed_as_record_literal() {
    let source = r#"
fn main(): Int {
  mut count = 0
  while count {}
  0
}
"#;
    let diagnostics = muga::check_source(source).expect_err("expected type error");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T001"),
        "expected `while` body to type-check past parse: {diagnostics:#?}"
    );
}

#[test]
fn parenthesized_record_literal_is_allowed_in_if_condition() {
    let source = r#"
record P { x: Int }
fn main(): Int {
  if (P { x: 1 }) { 1 } else { 0 }
}
"#;
    let diagnostics = muga::check_source(source).expect_err("expected type error, not parse error");
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| !diagnostic.code.starts_with("P")),
        "parser must accept parenthesized record literal: {diagnostics:#?}"
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T001"),
        "type checker must reject record value as Bool condition: {diagnostics:#?}"
    );
}

#[test]
fn record_literal_still_allowed_as_call_argument() {
    let source = r#"
record P { x: Int }
fn get_x(p: P): Int { p.x }
fn main(): Int {
  get_x(P { x: 42 })
}
"#;
    let outcome =
        muga::run_source(source).expect("record literal should still parse as a call argument");
    let value = outcome
        .main_result
        .expect("expected main to return a value");
    let muga::runtime::Value::Int(value) = value else {
        panic!("expected Int, got {value:?}");
    };
    assert_eq!(value, 42);
}

#[test]
fn function_equality_reports_only_kind_diagnostic() {
    let source = r#"
fn f(x: Int): Int { x }
fn main(): Bool {
  f == f
}
"#;
    let diagnostics = muga::check_source(source).expect_err("expected type error");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T003"),
        "{diagnostics:#?}"
    );
    assert!(
        !diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "function equality must not report a spurious type-mismatch: {diagnostics:#?}"
    );
}

#[test]
fn recursive_type_inference_is_rejected() {
    let source = r#"
fn main(): Int {
  bad = fn(x) {
    x(x)
  }
  0
}
"#;
    let diagnostics = muga::check_source(source).expect_err("expected type error");
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "T005" && diagnostic.message.contains("infinite type")
        }),
        "{diagnostics:#?}"
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

fn assert_sample_without_main_runs(path: &str, expected_output: &str) {
    let source = fs::read_to_string(path).unwrap();
    let result = muga::run_source(&source).unwrap();
    assert!(result.main_result.is_none(), "sample: {path}");
    assert_eq!(result.output_text, expected_output, "sample: {path}");
}

fn parse_source(source: &str) -> muga::ast::Program {
    let tokens = muga::lexer::lex(source).unwrap();
    muga::parser::parse(tokens).unwrap()
}

fn typecheck_binding_id(
    output: &muga::typing::TypeCheckOutput,
    name: &str,
) -> muga::identity::BindingId {
    output
        .bindings
        .iter()
        .find(|binding| output.symbols.resolve(binding.symbol) == name)
        .map(|binding| binding.id)
        .unwrap_or_else(|| {
            let names: Vec<_> = output
                .bindings
                .iter()
                .map(|binding| output.symbols.resolve(binding.symbol))
                .collect();
            panic!("binding `{name}` should exist; found {names:?}");
        })
}

fn typed_binding_id(program: &muga::typed_hir::Program, name: &str) -> muga::identity::BindingId {
    program
        .bindings
        .iter()
        .find(|binding| program.symbols.resolve(binding.symbol) == name)
        .map(|binding| binding.id)
        .unwrap_or_else(|| {
            let names: Vec<_> = program
                .bindings
                .iter()
                .map(|binding| program.symbols.resolve(binding.symbol))
                .collect();
            panic!("binding `{name}` should exist; found {names:?}");
        })
}

fn collect_typed_calls(program: &muga::typed_hir::Program) -> Vec<&muga::typed_hir::CallExpr> {
    let mut calls = Vec::new();
    for statement in &program.statements {
        collect_typed_calls_in_stmt(statement, &mut calls);
    }
    calls
}

fn collect_typed_calls_in_stmt<'a>(
    statement: &'a muga::typed_hir::Stmt,
    calls: &mut Vec<&'a muga::typed_hir::CallExpr>,
) {
    match statement {
        muga::typed_hir::Stmt::Assign(stmt) => collect_typed_calls_in_expr(&stmt.value, calls),
        muga::typed_hir::Stmt::Record(_) => {}
        muga::typed_hir::Stmt::Function(stmt) => {
            collect_typed_calls_in_value_block(&stmt.body, calls);
        }
        muga::typed_hir::Stmt::If(stmt) => {
            collect_typed_calls_in_expr(&stmt.condition, calls);
            collect_typed_calls_in_block(&stmt.then_branch, calls);
            if let Some(else_branch) = &stmt.else_branch {
                collect_typed_calls_in_block(else_branch, calls);
            }
        }
        muga::typed_hir::Stmt::While(stmt) => {
            collect_typed_calls_in_expr(&stmt.condition, calls);
            collect_typed_calls_in_block(&stmt.body, calls);
        }
        muga::typed_hir::Stmt::Expr(stmt) => collect_typed_calls_in_expr(&stmt.expr, calls),
    }
}

fn collect_typed_calls_in_block<'a>(
    block: &'a muga::typed_hir::Block,
    calls: &mut Vec<&'a muga::typed_hir::CallExpr>,
) {
    for statement in &block.statements {
        collect_typed_calls_in_stmt(statement, calls);
    }
}

fn collect_typed_calls_in_value_block<'a>(
    block: &'a muga::typed_hir::ValueBlock,
    calls: &mut Vec<&'a muga::typed_hir::CallExpr>,
) {
    for statement in &block.statements {
        collect_typed_calls_in_stmt(statement, calls);
    }
    collect_typed_calls_in_expr(&block.expr, calls);
}

fn collect_typed_calls_in_expr<'a>(
    expr: &'a muga::typed_hir::Expr,
    calls: &mut Vec<&'a muga::typed_hir::CallExpr>,
) {
    match &expr.kind {
        muga::typed_hir::ExprKind::Int(_)
        | muga::typed_hir::ExprKind::Bool(_)
        | muga::typed_hir::ExprKind::String(_)
        | muga::typed_hir::ExprKind::Ident(_) => {}
        muga::typed_hir::ExprKind::ListLit(expr) => {
            for item in &expr.items {
                collect_typed_calls_in_expr(item, calls);
            }
        }
        muga::typed_hir::ExprKind::Index(expr) => {
            collect_typed_calls_in_expr(&expr.base, calls);
            collect_typed_calls_in_expr(&expr.index, calls);
        }
        muga::typed_hir::ExprKind::RecordLit(expr) => {
            for field in &expr.fields {
                collect_typed_calls_in_expr(&field.value, calls);
            }
        }
        muga::typed_hir::ExprKind::Field(expr) => collect_typed_calls_in_expr(&expr.base, calls),
        muga::typed_hir::ExprKind::RecordUpdate(expr) => {
            collect_typed_calls_in_expr(&expr.base, calls);
            for field in &expr.fields {
                collect_typed_calls_in_expr(&field.value, calls);
            }
        }
        muga::typed_hir::ExprKind::Unary(expr) => collect_typed_calls_in_expr(&expr.expr, calls),
        muga::typed_hir::ExprKind::Binary(expr) => {
            collect_typed_calls_in_expr(&expr.left, calls);
            collect_typed_calls_in_expr(&expr.right, calls);
        }
        muga::typed_hir::ExprKind::Call(expr) => {
            calls.push(expr);
            collect_typed_calls_in_expr(&expr.callee, calls);
            for arg in &expr.args {
                collect_typed_calls_in_expr(arg, calls);
            }
        }
        muga::typed_hir::ExprKind::If(expr) => {
            collect_typed_calls_in_expr(&expr.condition, calls);
            collect_typed_calls_in_value_block(&expr.then_branch, calls);
            collect_typed_calls_in_value_block(&expr.else_branch, calls);
        }
        muga::typed_hir::ExprKind::Match(expr) => {
            collect_typed_calls_in_expr(&expr.value, calls);
            for arm in &expr.arms {
                collect_typed_calls_in_expr(&arm.value, calls);
            }
        }
        muga::typed_hir::ExprKind::Fn(expr) => {
            collect_typed_calls_in_value_block(&expr.body, calls);
        }
    }
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
