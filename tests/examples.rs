use std::{collections::HashSet, fs, path::Path, process::Command};

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
fn prelude_catalog_names_are_unique_and_classified() {
    let mut names = HashSet::new();
    let mut function_count = 0;
    let mut value_count = 0;

    for builtin in muga::prelude::builtins() {
        assert!(names.insert(builtin.name), "duplicate builtin: {builtin:?}");
        match builtin.kind {
            muga::prelude::BuiltinKind::Function => function_count += 1,
            muga::prelude::BuiltinKind::Value => value_count += 1,
        }
        assert!(muga::prelude::is_builtin_name(builtin.name));
        assert!(
            muga::prelude::builtin_debug_label(builtin.id).contains(builtin.name),
            "{builtin:?}"
        );
    }

    assert_eq!(
        muga::prelude::builtin_by_name("Option::None").map(|builtin| builtin.kind),
        Some(muga::prelude::BuiltinKind::Value)
    );
    assert_eq!(value_count, 1);
    assert_eq!(
        function_count + value_count,
        muga::prelude::builtins().len()
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
fn prelude_catalog_sample_runs() {
    assert_sample_runs("samples/prelude_catalog.muga", "22", "");
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
    let program = muga::package::load_flattened_program_from_entry(Path::new(
        "samples/packages/app/main/main.muga",
    ))
    .unwrap();
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
        muga::package::load_flattened_from_entry(Path::new("samples/packages/app/main/main.muga"))
            .unwrap();
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
        muga::package::load_flattened_from_entry(Path::new("samples/packages/app/main/main.muga"))
            .unwrap();
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
fn package_loader_can_return_unflattened_package_graph() {
    let unflattened = muga::package::load_package_graph_from_entry(Path::new(
        "samples/packages/app/main/main.muga",
    ))
    .expect("package graph should load without flattening");
    let flattened =
        muga::package::load_flattened_from_entry(Path::new("samples/packages/app/main/main.muga"))
            .unwrap();

    assert_eq!(
        unflattened.package_graph.packages,
        flattened.package_graph.packages
    );
    assert_eq!(
        unflattened.package_graph.items,
        flattened.package_graph.items
    );
    assert_eq!(unflattened.package_exports, flattened.package_exports);
    assert_eq!(
        unflattened
            .packages
            .iter()
            .map(|package| package.path.as_str())
            .collect::<Vec<_>>(),
        vec!["app::main", "util::numbers", "util::users"]
    );
    let app = unflattened
        .packages
        .iter()
        .find(|package| package.path == "app::main")
        .expect("app package should be loaded");
    assert!(app.files.iter().any(|file| {
        file.module_path == "main.muga"
            && file
                .program
                .package
                .as_ref()
                .is_some_and(|package| package.path == "app::main")
    }));
    assert!(
        flattened.program.package.is_none(),
        "existing load path should still return a flattened program"
    );
}

#[test]
fn package_aware_checking_preserves_public_import_resolution() {
    let result = muga::check_package_aware_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("package-aware checking should pass");

    assert!(
        result
            .packages
            .package_graph
            .package_id("app::main")
            .is_some(),
        "{:#?}",
        result.packages.package_graph.packages
    );
    assert!(
        result
            .packages
            .package_graph
            .package_id("util::numbers")
            .is_some(),
        "{:#?}",
        result.packages.package_graph.packages
    );
    assert_eq!(
        main_return_type(&result.typed_program),
        Some(muga::types::TypeInfo::Int)
    );
    let function_names: Vec<_> = result
        .typed_program
        .statements
        .iter()
        .filter_map(|statement| match statement {
            muga::typed_hir::Stmt::Function(function) => Some(function.name.as_str()),
            _ => None,
        })
        .collect();
    assert!(function_names.contains(&"main"), "{function_names:#?}");
    assert!(function_names.contains(&"inc_twice"), "{function_names:#?}");
    let binding_ids = result
        .typed_program
        .bindings
        .iter()
        .map(|binding| binding.id.as_u32())
        .collect::<Vec<_>>();
    let unique_binding_ids: HashSet<_> = binding_ids.iter().copied().collect();
    assert_eq!(
        unique_binding_ids.len(),
        binding_ids.len(),
        "{binding_ids:?}"
    );
    assert_unique_typed_ids(&result.typed_program);
}

#[test]
fn package_aware_typed_hir_lowers_to_mir_bytecode_runtime() {
    let result = muga::check_package_aware_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("package-aware checking should pass");
    let mir = muga::mir::lower_typed(&result.typed_program);
    let bytecode = muga::bytecode::compile(mir);
    let outcome = muga::runtime::run(&bytecode).expect("package-aware MIR should execute");
    let value = outcome.main_result.expect("main result should exist");

    assert_eq!(value.to_string(), "23");
    assert_eq!(outcome.output_text, "");
}

#[test]
fn package_aware_typed_hir_lowers_package_item_identity_to_mir() {
    let result = muga::check_package_aware_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("package-aware checking should pass");
    let package = result
        .packages
        .package_graph
        .package_id("app::main")
        .expect("app::main package should be loaded");
    let main_item = result
        .packages
        .package_graph
        .item_id(package, "main", muga::package::PackageItemKind::Function)
        .expect("main package item should be known");
    let mir = muga::mir::lower_typed(&result.typed_program);

    assert!(
        mir.entry
            .function_defs
            .iter()
            .any(|function| function.package_item == Some(main_item))
    );
}

#[test]
fn package_aware_typed_hir_lowers_imported_enum_to_mir_runtime() {
    let result =
        muga::check_package_aware_path(Path::new("samples/packages/app/enum_demo/main.muga"))
            .expect("package-aware checking should pass");
    let mir = muga::mir::lower_typed(&result.typed_program);
    let bytecode = muga::bytecode::compile(mir);
    let outcome = muga::runtime::run(&bytecode).expect("package-aware enum MIR should execute");
    let value = outcome.main_result.expect("main result should exist");

    assert_eq!(value.to_string(), "7");
    assert_eq!(outcome.output_text, "");
}

#[test]
fn package_aware_typed_program_preserves_public_interface_items() {
    let root = temp_package_root("package-aware-entry-interface-items");
    let entry = write_package_file(
        &root,
        "app/entry_interface/main.muga",
        r#"
package app::entry_interface

pub record User {
  name: String
}

pub fn make_user(name: String): User {
  User {
    name: name
  }
}

fn main(): Int {
  1
}
"#,
    );
    let result =
        muga::check_package_aware_path(&entry).expect("package-aware checking should pass");
    let package = result
        .packages
        .package_graph
        .package_id("app::entry_interface")
        .expect("entry package should exist");
    let user_item = result
        .packages
        .package_graph
        .item_id(package, "User", muga::package::PackageItemKind::Record)
        .expect("User item should exist");
    let make_user_item = result
        .packages
        .package_graph
        .item_id(
            package,
            "make_user",
            muga::package::PackageItemKind::Function,
        )
        .expect("make_user item should exist");
    let interfaces = result.typed_program.package_interfaces();

    assert!(
        interfaces
            .record_by_name(package, "User")
            .is_some_and(|record| record.item == user_item),
        "{interfaces:#?}"
    );
    assert!(
        interfaces
            .function_by_name(package, "make_user")
            .is_some_and(|function| function.item == make_user_item),
        "{interfaces:#?}"
    );
}

#[test]
fn package_aware_checking_rejects_private_cross_package_references() {
    let diagnostics = muga::check_package_aware_path(Path::new(
        "samples/packages_invalid/app/import_pkg_item/main.muga",
    ))
    .unwrap_err();

    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "PK010"
                && diagnostic
                    .message
                    .contains("does not export function `helper`")
        }),
        "{diagnostics:#?}"
    );
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "PK010"
                && diagnostic
                    .message
                    .contains("does not export record `PackageValue`")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn package_aware_checking_reports_package_qualified_type_errors() {
    let root = temp_package_root("package-aware-type-error");
    let entry = write_package_file(
        &root,
        "app/type_error/main.muga",
        r#"
package app::type_error

import util::numbers

fn main(): Int {
  numbers::inc("not an int")
}
"#,
    );
    write_package_file(
        &root,
        "util/numbers/main.muga",
        r#"
package util::numbers

pub fn inc(value: Int): Int {
  value + 1
}
"#,
    );

    let diagnostics = muga::check_package_aware_path(&entry).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn package_aware_checking_reuses_unflattened_package_graph() {
    let result =
        muga::check_package_aware_path(Path::new("samples/packages/app/split_main/main.muga"))
            .expect("package-aware checking should pass");
    let app = result
        .packages
        .packages
        .iter()
        .find(|package| package.path == "app::split_main")
        .expect("entry package should be loaded");

    assert_eq!(app.files.len(), 2, "{app:#?}");
    assert!(
        app.files.iter().all(|file| file.program.package.is_some()),
        "{app:#?}"
    );
}

#[test]
fn package_signature_environment_preserves_same_package_type_identities() {
    let result = muga::check_package_aware_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("package-aware checking should pass");
    let users = result
        .packages
        .package_graph
        .package_id("util::users")
        .expect("users package should exist");
    let user_item = result
        .packages
        .package_graph
        .item_id(users, "User", muga::package::PackageItemKind::Record)
        .expect("User item should exist");
    let user = result
        .signatures
        .record(user_item)
        .expect("User signature should exist");
    let birthday_item = result
        .packages
        .package_graph
        .item_id(users, "birthday", muga::package::PackageItemKind::Function)
        .expect("birthday item should exist");
    let birthday = result
        .signatures
        .function(birthday_item)
        .expect("birthday signature should exist");

    assert!(
        user.fields
            .iter()
            .any(|field| field.name == "name" && field.ty == muga::types::TypeInfo::String)
    );
    assert!(
        user.fields
            .iter()
            .any(|field| field.name == "age" && field.ty == muga::types::TypeInfo::Int)
    );
    assert_eq!(birthday.params.len(), 1);
    assert!(matches!(
        birthday.params[0].ty.as_ref(),
        Some(muga::types::TypeInfo::PackageRecord { item, .. }) if *item == user_item
    ));
    assert!(matches!(
        birthday.ret.as_ref(),
        Some(muga::types::TypeInfo::PackageRecord { item, .. }) if *item == user_item
    ));
}

#[test]
fn package_signature_environment_resolves_imported_public_types() {
    let root = temp_package_root("package-signature-imported-types");
    let entry = write_transitive_interface_provider(&root);
    let result =
        muga::check_package_aware_path(&entry).expect("package-aware checking should pass");
    let users = result
        .packages
        .package_graph
        .package_id("model::users")
        .expect("users package should exist");
    let user_item = result
        .packages
        .package_graph
        .item_id(users, "User", muga::package::PackageItemKind::Record)
        .expect("User item should exist");
    let facade = result
        .packages
        .package_graph
        .package_id("api::facade")
        .expect("facade package should exist");
    let default_user_item = result
        .packages
        .package_graph
        .item_id(
            facade,
            "default_user",
            muga::package::PackageItemKind::Function,
        )
        .expect("default_user item should exist");
    let default_user = result
        .signatures
        .function(default_user_item)
        .expect("default_user signature should exist");

    assert!(matches!(
        default_user.ret.as_ref(),
        Some(muga::types::TypeInfo::PackageRecord { item, .. }) if *item == user_item
    ));
}

#[test]
fn package_signature_environment_preserves_generic_enum_signatures() {
    let result =
        muga::check_package_aware_path(Path::new("samples/packages/app/enum_demo/main.muga"))
            .expect("package-aware checking should pass");
    let states = result
        .packages
        .package_graph
        .package_id("util::states")
        .expect("states package should exist");
    let status_item = result
        .packages
        .package_graph
        .item_id(states, "Status", muga::package::PackageItemKind::Enum)
        .expect("Status item should exist");
    let ready_item = result
        .packages
        .package_graph
        .item_id(states, "ready", muga::package::PackageItemKind::Function)
        .expect("ready item should exist");
    let status = result
        .signatures
        .enumeration(status_item)
        .expect("Status signature should exist");
    let ready = result
        .signatures
        .function(ready_item)
        .expect("ready signature should exist");

    assert_eq!(status.type_params, vec!["T".to_string()]);
    assert!(status.variants.iter().any(|variant| {
        variant.name == "Ready"
            && matches!(
                variant.payload.as_ref(),
                Some(muga::types::TypeInfo::GenericParam(_))
            )
    }));
    assert!(matches!(
        ready.ret.as_ref(),
        Some(muga::types::TypeInfo::PackageEnum { item, args, .. })
            if *item == status_item && args == &[muga::types::TypeInfo::Int]
    ));
}

#[test]
fn package_signature_environment_rejects_generic_enum_arity_mismatch() {
    let root = temp_package_root("package-signature-enum-arity");
    let entry = write_package_file(
        &root,
        "app/main/main.muga",
        r#"
package app::main

import util::states

pub fn bad(): states::Status {
  states::ready(1)
}

fn main(): Int {
  0
}
"#,
    );
    write_package_file(
        &root,
        "util/states/model.muga",
        r#"
package util::states

pub enum Status[T] {
  Ready(T)
  Waiting
}

pub fn ready(value: Int): Status[Int] {
  Status::Ready(value)
}
"#,
    );

    let diagnostics = muga::check_package_aware_path(&entry).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "T022"
                && diagnostic
                    .message
                    .contains("enum `Status` expects exactly 1 type arguments")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn package_module_signature_environment_tracks_module_visibility() {
    let root = temp_package_root("package-module-signature-visibility");
    let entry = write_package_file(
        &root,
        "app/env/main.muga",
        r#"
package app::env

fn local(): Int {
  1
}

fn main(): Int {
  helper() + local()
}
"#,
    );
    write_package_file(
        &root,
        "app/env/helper.muga",
        r#"
package app::env

record Hidden {
  value: Int
}

pkg record Visible {
  value: Int
}

pkg fn helper(): Int {
  41
}
"#,
    );

    let result =
        muga::check_package_aware_path(&entry).expect("package-aware checking should pass");
    let package = result
        .packages
        .package_graph
        .package_id("app::env")
        .expect("package should exist");
    let main_module = result
        .packages
        .package_graph
        .module_id(package, "main.muga")
        .expect("main module should exist");
    let helper_module = result
        .packages
        .package_graph
        .module_id(package, "helper.muga")
        .expect("helper module should exist");
    let local_item = result
        .packages
        .package_graph
        .item_id_in_module(
            main_module,
            "local",
            muga::package::PackageItemKind::Function,
        )
        .expect("local item should exist");
    let helper_item = result
        .packages
        .package_graph
        .item_id_in_module(
            helper_module,
            "helper",
            muga::package::PackageItemKind::Function,
        )
        .expect("helper item should exist");
    let visible_item = result
        .packages
        .package_graph
        .item_id_in_module(
            helper_module,
            "Visible",
            muga::package::PackageItemKind::Record,
        )
        .expect("Visible item should exist");
    let main_env = result
        .signatures
        .module(main_module)
        .expect("main module signature environment should exist");

    let local = main_env
        .function("local")
        .expect("local function should be visible");
    assert_eq!(local.item, local_item);
    assert_eq!(
        local.source,
        muga::package_signature::PackageSignatureSource::ModuleLocal
    );
    let local_signature = main_env
        .function_signature(&result.signatures, "local")
        .expect("local function signature should be available");
    assert_eq!(local_signature.name, "local");
    assert_eq!(local_signature.ret, Some(muga::types::TypeInfo::Int));

    let helper = main_env
        .function("helper")
        .expect("pkg helper should be visible");
    assert_eq!(helper.item, helper_item);
    assert_eq!(
        helper.source,
        muga::package_signature::PackageSignatureSource::SamePackage
    );
    let helper_signature = main_env
        .function_signature(&result.signatures, "helper")
        .expect("helper function signature should be available");
    assert_eq!(helper_signature.ret, Some(muga::types::TypeInfo::Int));

    let visible = main_env
        .record("Visible")
        .expect("pkg record should be visible");
    assert_eq!(visible.item, visible_item);
    assert_eq!(
        visible.source,
        muga::package_signature::PackageSignatureSource::SamePackage
    );
    let visible_signature = main_env
        .record_signature(&result.signatures, "Visible")
        .expect("Visible record signature should be available");
    assert_eq!(visible_signature.name, "Visible");
    assert!(
        main_env.record("Hidden").is_none(),
        "module-private helper record should not be visible from main"
    );
}

#[test]
fn package_module_signature_environment_tracks_imported_exports() {
    let root = temp_package_root("package-module-signature-imports");
    let entry = write_transitive_interface_provider(&root);
    let result =
        muga::check_package_aware_path(&entry).expect("package-aware checking should pass");
    let users = result
        .packages
        .package_graph
        .package_id("model::users")
        .expect("users package should exist");
    let user_item = result
        .packages
        .package_graph
        .item_id(users, "User", muga::package::PackageItemKind::Record)
        .expect("User item should exist");
    let facade = result
        .packages
        .package_graph
        .package_id("api::facade")
        .expect("facade package should exist");
    let facade_module = result
        .packages
        .package_graph
        .module_id(facade, "main.muga")
        .expect("facade module should exist");
    let facade_env = result
        .signatures
        .module(facade_module)
        .expect("facade module signature environment should exist");
    let user = facade_env
        .record("users::User")
        .expect("imported User should be visible by alias");

    assert_eq!(user.item, user_item);
    assert_eq!(
        user.source,
        muga::package_signature::PackageSignatureSource::Imported {
            alias: "users".to_string(),
            package: users,
        }
    );
    let user_signature = facade_env
        .record_signature(&result.signatures, "users::User")
        .expect("imported User signature should be available");
    assert_eq!(user_signature.name, "User");
    assert!(
        user_signature
            .fields
            .iter()
            .any(|field| field.name == "age" && field.ty == muga::types::TypeInfo::Int)
    );
}

#[test]
fn package_module_typechecking_uses_signature_environment_for_body_errors() {
    let root = temp_package_root("package-module-body-check");
    let entry = write_package_file(
        &root,
        "app/body/main.muga",
        r#"
package app::body

fn main(): Int {
  helper("bad")
}
"#,
    );
    write_package_file(
        &root,
        "app/body/helper.muga",
        r#"
package app::body

pkg fn helper(value: Int): Int {
  value + 1
}
"#,
    );

    let loaded =
        muga::package::load_package_graph_from_entry(&entry).expect("package graph should load");
    let signatures =
        muga::package_signature::PackageSignatureEnvironment::from_loaded_graph(&loaded)
            .expect("signatures should build");
    let package = loaded
        .package_graph
        .package_id("app::body")
        .expect("package should exist");
    let module = loaded
        .package_graph
        .module_id(package, "main.muga")
        .expect("main module should exist");
    let helper_module = loaded
        .package_graph
        .module_id(package, "helper.muga")
        .expect("helper module should exist");
    let helper_item = loaded
        .package_graph
        .item_id_in_module(
            helper_module,
            "helper",
            muga::package::PackageItemKind::Function,
        )
        .expect("helper item should exist");
    let file = loaded
        .packages
        .iter()
        .find(|package| package.path == "app::body")
        .and_then(|package| {
            package
                .files
                .iter()
                .find(|file| file.module_path == "main.muga")
        })
        .expect("main file should exist");
    let output = muga::typing::typecheck_package_module(&file.program, &signatures, module);

    assert!(
        output
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{:#?}",
        output.diagnostics
    );
    assert!(
        output.calls.iter().any(|call| {
            matches!(
                call.callee,
                muga::typing::TypedCalleeInfo::PackageItem { item, .. } if item == helper_item
            )
        }),
        "{:#?}",
        output.calls
    );
}

#[test]
fn package_aware_checking_runs_module_resolver_for_body_errors() {
    let root = temp_package_root("package-module-resolver-check");
    let entry = write_package_file(
        &root,
        "app/body_resolver/main.muga",
        r#"
package app::body_resolver

fn main(): Int {
  value = 1
  value = 2
  value
}
"#,
    );

    let diagnostics = muga::check_package_aware_path(&entry).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "E001"
                && diagnostic
                    .message
                    .contains("cannot update immutable binding `value`")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn package_aware_checking_exposes_module_type_outputs() {
    let result = muga::check_package_aware_path(Path::new(
        "samples/packages/app/module_visibility/main.muga",
    ))
    .expect("package-aware checking should pass");
    let package = result
        .packages
        .package_graph
        .package_id("app::module_visibility")
        .expect("package should exist");
    let helper_module = result
        .packages
        .package_graph
        .module_id(package, "helper.muga")
        .expect("helper module should exist");
    let helper_item = result
        .packages
        .package_graph
        .item_id_in_module(
            helper_module,
            "helper",
            muga::package::PackageItemKind::Function,
        )
        .expect("helper item should exist");
    let package_value_item = result
        .packages
        .package_graph
        .item_id_in_module(
            helper_module,
            "PackageValue",
            muga::package::PackageItemKind::Record,
        )
        .expect("PackageValue item should exist");
    let main_check = result
        .module_checks
        .iter()
        .find(|check| check.module_path == "main.muga")
        .expect("main module check should exist");

    assert_eq!(main_check.package, package);
    assert!(
        main_check
            .resolve_output
            .identifier_refs
            .iter()
            .any(|ident| { main_check.resolve_output.symbols.resolve(ident.name) == "helper" })
    );
    assert!(main_check.type_output.calls.iter().any(|call| {
        matches!(
            call.callee,
            muga::typing::TypedCalleeInfo::PackageItem { item, .. } if item == helper_item
        )
    }));
    assert!(main_check.type_output.bindings.iter().any(|binding| {
        main_check.type_output.symbols.resolve(binding.symbol) == "value"
            && matches!(
                binding.ty,
                muga::types::TypeInfo::PackageRecord { item, .. } if item == package_value_item
            )
    }));
}

#[test]
fn package_module_typed_hir_lowering_preserves_package_binding_identity() {
    let result = muga::check_package_aware_path(Path::new(
        "samples/packages/app/module_visibility/main.muga",
    ))
    .expect("package-aware checking should pass");
    let package = result
        .packages
        .package_graph
        .package_id("app::module_visibility")
        .expect("package should exist");
    let helper_module = result
        .packages
        .package_graph
        .module_id(package, "helper.muga")
        .expect("helper module should exist");
    let helper_item = result
        .packages
        .package_graph
        .item_id_in_module(
            helper_module,
            "helper",
            muga::package::PackageItemKind::Function,
        )
        .expect("helper item should exist");
    let main_check = result
        .module_checks
        .iter()
        .find(|check| check.module_path == "main.muga")
        .expect("main module check should exist");
    let program = &main_check.typed_program;
    let helper_binding = program
        .bindings
        .iter()
        .find(|binding| {
            program.symbols.resolve(binding.symbol) == "helper"
                && binding.package_item == Some(helper_item)
        })
        .map(|binding| binding.id)
        .expect("helper binding should retain package item identity");
    let calls = collect_typed_calls(program);

    assert!(
        calls.iter().any(|call| {
            call.resolved_callee
                == muga::typing::TypedCalleeInfo::PackageItem {
                    binding: helper_binding,
                    item: helper_item,
                }
                && matches!(
                    &call.callee.kind,
                    muga::typed_hir::ExprKind::Ident(muga::typed_hir::IdentExpr {
                        target: muga::typed_hir::IdentTarget::PackageItem {
                            binding,
                            item,
                        },
                        ..
                    }) if *binding == helper_binding && *item == helper_item
                )
        }),
        "{calls:#?}"
    );
}

#[test]
fn package_symbol_graph_exposes_module_identity() {
    let loaded = muga::package::load_flattened_from_entry(Path::new(
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
    assert_eq!(inc_twice.params[0].ty, muga::types::TypeInfo::Int);
    assert_eq!(inc_twice.ret, muga::types::TypeInfo::Int);
    let singleton = interfaces
        .function_by_name(numbers, "singleton")
        .expect("singleton should be exported");
    assert_eq!(
        singleton.ret,
        muga::types::TypeInfo::List(Box::new(muga::types::TypeInfo::Int))
    );
    let singleton_len = interfaces
        .function_by_name(numbers, "singleton_len")
        .expect("singleton_len should be exported");
    assert_eq!(singleton_len.ret, muga::types::TypeInfo::Int);
    let singleton_first = interfaces
        .function_by_name(numbers, "singleton_first")
        .expect("singleton_first should be exported");
    assert_eq!(singleton_first.ret, muga::types::TypeInfo::Int);
    let singleton_get = interfaces
        .function_by_name(numbers, "singleton_get")
        .expect("singleton_get should be exported");
    assert_eq!(
        singleton_get.ret,
        muga::types::TypeInfo::Option(Box::new(muga::types::TypeInfo::Int))
    );
    let replace_singleton = interfaces
        .function_by_name(numbers, "replace_singleton")
        .expect("replace_singleton should be exported");
    assert_eq!(
        replace_singleton.ret,
        muga::types::TypeInfo::List(Box::new(muga::types::TypeInfo::Int))
    );
    let maybe_positive = interfaces
        .function_by_name(numbers, "maybe_positive")
        .expect("maybe_positive should be exported");
    assert_eq!(
        maybe_positive.ret,
        muga::types::TypeInfo::Option(Box::new(muga::types::TypeInfo::Int))
    );
    let value_or_zero = interfaces
        .function_by_name(numbers, "value_or_zero")
        .expect("value_or_zero should be exported");
    assert_eq!(
        value_or_zero.params[0].ty,
        muga::types::TypeInfo::Option(Box::new(muga::types::TypeInfo::Int))
    );
    assert_eq!(value_or_zero.ret, muga::types::TypeInfo::Int);
    let singleton_map = interfaces
        .function_by_name(numbers, "singleton_map")
        .expect("singleton_map should be exported");
    assert_eq!(singleton_map.params[0].ty, muga::types::TypeInfo::String);
    assert_eq!(singleton_map.params[1].ty, muga::types::TypeInfo::Int);
    assert_eq!(
        singleton_map.ret,
        muga::types::TypeInfo::Map(
            Box::new(muga::types::TypeInfo::String),
            Box::new(muga::types::TypeInfo::Int)
        )
    );
    let map_get_or_zero = interfaces
        .function_by_name(numbers, "map_get_or_zero")
        .expect("map_get_or_zero should be exported");
    assert_eq!(
        map_get_or_zero.params[0].ty,
        muga::types::TypeInfo::Map(
            Box::new(muga::types::TypeInfo::String),
            Box::new(muga::types::TypeInfo::Int)
        )
    );
    assert_eq!(map_get_or_zero.params[1].ty, muga::types::TypeInfo::String);
    assert_eq!(map_get_or_zero.ret, muga::types::TypeInfo::Int);
    let positive_result = interfaces
        .function_by_name(numbers, "positive_result")
        .expect("positive_result should be exported");
    assert_eq!(
        positive_result.ret,
        muga::types::TypeInfo::Result(
            Box::new(muga::types::TypeInfo::Int),
            Box::new(muga::types::TypeInfo::String)
        )
    );
    let result_or_zero = interfaces
        .function_by_name(numbers, "result_or_zero")
        .expect("result_or_zero should be exported");
    assert_eq!(
        result_or_zero.params[0].ty,
        muga::types::TypeInfo::Result(
            Box::new(muga::types::TypeInfo::Int),
            Box::new(muga::types::TypeInfo::String)
        )
    );
    assert_eq!(result_or_zero.ret, muga::types::TypeInfo::Int);

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
            .any(|field| field.name == "age" && field.ty == muga::types::TypeInfo::Int),
        "{user_record:#?}"
    );

    let birthday = interfaces
        .function_by_name(users, "birthday")
        .expect("birthday should be exported");
    assert_eq!(birthday.params.len(), 1);
    assert!(
        matches!(
            &birthday.params[0].ty,
            muga::types::TypeInfo::PackageRecord { item, .. } if *item == user_item
        ),
        "{birthday:#?}"
    );
    assert!(
        matches!(
            &birthday.ret,
            muga::types::TypeInfo::PackageRecord { item, .. } if *item == user_item
        ),
        "{birthday:#?}"
    );
}

#[test]
fn typed_hir_package_items_are_attached_to_public_statements() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let users = program
        .package_graph
        .package_id("util::users")
        .expect("users package should exist");
    let user_item = program
        .package_graph
        .item_id(users, "User", muga::package::PackageItemKind::Record)
        .expect("User item should exist");
    let birthday_item = program
        .package_graph
        .item_id(users, "birthday", muga::package::PackageItemKind::Function)
        .expect("birthday item should exist");

    assert!(
        program.statements.iter().any(|statement| {
            matches!(statement, muga::typed_hir::Stmt::Record(record)
                if record.package_item == Some(user_item))
        }),
        "{:#?}",
        program.statements
    );
    assert!(
        program.statements.iter().any(|statement| {
            matches!(statement, muga::typed_hir::Stmt::Function(function)
                if function.package_item == Some(birthday_item))
        }),
        "{:#?}",
        program.statements
    );
}

#[test]
fn package_export_graph_can_be_derived_from_typed_interfaces() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = program.package_interfaces();
    let symbol_exports =
        muga::interface::PackageExportGraph::from_symbol_graph(&program.package_graph);
    let interface_exports =
        muga::interface::PackageExportGraph::from_interfaces(&interfaces, &program.package_graph);

    assert_eq!(interface_exports, symbol_exports);
}

#[test]
fn package_interfaces_round_trip_public_records_functions_and_enums() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/enum_demo/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = program.package_interfaces();
    let text = interfaces.to_persisted_text(&program.symbols);
    assert!(text.starts_with("muga-package-interface-v2\n"), "{text}");
    assert!(text.contains("\nhash\t"), "{text}");

    let mut symbols = program.symbols.clone();
    let loaded = muga::interface::PackageInterfaceGraph::from_persisted_text(&text, &mut symbols)
        .expect("persisted interfaces should parse");
    assert_eq!(
        loaded.stable_hash(&symbols),
        interfaces.stable_hash(&program.symbols)
    );
    assert!(
        loaded.package_by_path("util::states").is_some(),
        "{loaded:#?}"
    );

    let diagnostics = program.validate_package_references_against_interfaces(&interfaces);
    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
}

#[test]
fn package_interface_hash_is_stable_for_same_interface() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = program.package_interfaces();
    assert_eq!(
        interfaces.stable_hash(&program.symbols),
        interfaces.stable_hash(&program.symbols)
    );

    let text = interfaces.to_persisted_text(&program.symbols);
    let mut symbols = program.symbols.clone();
    let loaded = muga::interface::PackageInterfaceGraph::from_persisted_text(&text, &mut symbols)
        .expect("persisted interfaces should parse");
    assert_eq!(
        loaded.stable_hash(&symbols),
        interfaces.stable_hash(&program.symbols)
    );
}

#[test]
fn package_interface_hash_changes_when_public_signature_changes() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = program.package_interfaces();
    let original_hash = interfaces.stable_hash(&program.symbols);
    let numbers = program
        .package_graph
        .package_id("util::numbers")
        .expect("numbers package should exist");
    let mut changed = interfaces.clone();
    changed
        .packages
        .iter_mut()
        .find(|interface| interface.package == numbers)
        .expect("numbers interface should exist")
        .functions
        .iter_mut()
        .find(|function| function.name == "inc_twice")
        .expect("inc_twice should be exported")
        .ret = muga::types::TypeInfo::String;

    assert_ne!(changed.stable_hash(&program.symbols), original_hash);
}

#[test]
fn package_interface_hash_changes_when_public_enum_shape_changes() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/enum_demo/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = program.package_interfaces();
    let original_hash = interfaces.stable_hash(&program.symbols);
    let states = program
        .package_graph
        .package_id("util::states")
        .expect("states package should exist");
    let mut changed = interfaces.clone();
    changed
        .packages
        .iter_mut()
        .find(|interface| interface.package == states)
        .expect("states interface should exist")
        .enums
        .iter_mut()
        .find(|enumeration| enumeration.name == "Status")
        .expect("Status enum should be exported")
        .variants
        .iter_mut()
        .find(|variant| variant.name == "Ready")
        .expect("Ready variant should exist")
        .name = "Done".to_string();

    assert_ne!(changed.stable_hash(&program.symbols), original_hash);
}

#[test]
fn package_interface_artifact_path_is_deterministic() {
    let path = muga::interface::PackageInterfaceGraph::persisted_file_path(
        Path::new(".muga/interfaces"),
        "util::states",
    );
    assert_eq!(path, Path::new(".muga/interfaces/util__states.mgi"));
}

#[test]
fn package_interface_file_round_trip_preserves_type_info_identities() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = program.package_interfaces();
    let path = std::env::temp_dir().join(format!(
        "muga-package-interface-round-trip-{}.mgi",
        std::process::id()
    ));
    interfaces
        .write_persisted_file(&path, &program.symbols)
        .expect("interface file should be written");

    let mut symbols = program.symbols.clone();
    let loaded = muga::interface::PackageInterfaceGraph::read_persisted_file(&path, &mut symbols)
        .expect("interface file should load");
    let _ = fs::remove_file(&path);

    assert_eq!(
        loaded.stable_hash(&symbols),
        interfaces.stable_hash(&program.symbols)
    );
    let users = loaded
        .package_by_path("util::users")
        .expect("users package should exist")
        .package;
    let user_item = loaded
        .record_by_name(users, "User")
        .expect("User record should exist")
        .item;
    let birthday = loaded
        .function_by_name(users, "birthday")
        .expect("birthday should be exported");
    assert!(
        matches!(
            &birthday.ret,
            muga::types::TypeInfo::PackageRecord { item, .. } if *item == user_item
        ),
        "{birthday:#?}"
    );
}

#[test]
fn package_interface_rejects_hash_mismatch() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/enum_demo/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = program.package_interfaces();
    let text = interfaces.to_persisted_text(&program.symbols);
    let tampered = text.replacen("Ready", "Done", 1);
    let mut symbols = program.symbols.clone();
    let diagnostics =
        muga::interface::PackageInterfaceGraph::from_persisted_text(&tampered, &mut symbols)
            .unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "PK019" && diagnostic.message.contains("hash mismatch")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn typed_hir_validates_reloaded_package_interfaces() {
    let path = Path::new("samples/packages/app/enum_demo/main.muga");
    let program = muga::compile_typed_path(path).expect("typed package compilation should pass");
    let text = program
        .package_interfaces()
        .to_persisted_text(&program.symbols);
    let mut symbols = program.symbols.clone();
    let loaded = muga::interface::PackageInterfaceGraph::from_persisted_text(&text, &mut symbols)
        .expect("persisted interfaces should parse");

    muga::compile_typed_path_against_loaded_interfaces(path, &loaded, &symbols)
        .expect("typed package compilation against loaded interfaces should pass");
}

#[test]
fn downstream_package_can_check_against_loaded_interface_summary() {
    let provider = muga::compile_typed_path(Path::new("samples/packages/app/enum_demo/main.muga"))
        .expect("typed package compilation should pass");
    let (interfaces, symbols) = persisted_interfaces_from_program(&provider);
    let root = temp_package_root("loaded-interface-enum");
    let entry = write_package_file(
        &root,
        "app/interface_enum/main.muga",
        r#"
package app::interface_enum

import util::states

fn main(): Int {
  status: states::Status[Int] = states::ready(7)
  match status {
    states::Status::Ready(value) => value
    states::Status::Waiting => 0
    states::Status::Failed(message) => 0
  }
}
"#,
    );

    assert!(!root.join("util/states/model.muga").exists());
    let program = muga::compile_typed_path_against_loaded_interfaces(&entry, &interfaces, &symbols)
        .expect("downstream package should typecheck against loaded interfaces");
    assert!(
        program.package_graph.package_id("util::states").is_some(),
        "{:#?}",
        program.package_graph.packages
    );
}

#[test]
fn downstream_package_does_not_require_dependency_function_body_for_signature_checking() {
    let provider = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let (interfaces, symbols) = persisted_interfaces_from_program(&provider);
    let root = temp_package_root("loaded-interface-signatures");
    let entry = write_package_file(
        &root,
        "app/interface_signatures/main.muga",
        r#"
package app::interface_signatures

import util::numbers

fn main(): Int {
  maybe: Option[Int] = numbers::maybe_positive(3)
  result: Result[Int, String] = numbers::positive_result(4)
  numbers::value_or_zero(maybe) + numbers::result_or_zero(result)
}
"#,
    );

    assert!(!root.join("util/numbers/option.muga").exists());
    muga::compile_typed_path_against_loaded_interfaces(&entry, &interfaces, &symbols)
        .expect("dependency function signatures should be enough for downstream checking");
}

#[test]
fn package_aware_checking_can_use_loaded_interface_signatures_without_dependency_source() {
    let provider = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let (interfaces, symbols) = persisted_interfaces_from_program(&provider);
    let root = temp_package_root("package-aware-loaded-interface-signatures");
    let entry = write_package_file(
        &root,
        "app/package_aware_interface_signatures/main.muga",
        r#"
package app::package_aware_interface_signatures

import util::numbers

fn main(): Int {
  maybe: Option[Int] = numbers::maybe_positive(3)
  result: Result[Int, String] = numbers::positive_result(4)
  numbers::value_or_zero(maybe) + numbers::result_or_zero(result)
}
"#,
    );

    assert!(!root.join("util/numbers/option.muga").exists());
    let result =
        muga::check_package_aware_path_against_loaded_interfaces(&entry, &interfaces, &symbols)
            .expect("package-aware checking should use loaded interface signatures");
    let app = result
        .packages
        .package_graph
        .package_id("app::package_aware_interface_signatures")
        .expect("entry package should exist");
    assert!(
        result
            .packages
            .packages
            .iter()
            .all(|package| package.path != "util::numbers"),
        "{:#?}",
        result.packages.packages
    );
    let numbers = result
        .packages
        .package_graph
        .package_id("util::numbers")
        .expect("interface package should exist");
    let value_or_zero = result
        .packages
        .package_graph
        .item_id(
            numbers,
            "value_or_zero",
            muga::package::PackageItemKind::Function,
        )
        .expect("interface function item should exist");
    assert!(
        result.signatures.function(value_or_zero).is_some(),
        "{:#?}",
        result.signatures.functions
    );
    assert!(
        result
            .module_checks
            .iter()
            .all(|check| check.package == app && check.module_path != "<interface>"),
        "{:#?}",
        result.module_checks
    );
    let main_check = result
        .module_checks
        .iter()
        .find(|check| check.package == app && check.module_path == "main.muga")
        .expect("entry module check should exist");

    assert!(main_check.type_output.calls.iter().any(|call| {
        matches!(
            call.callee,
            muga::typing::TypedCalleeInfo::PackageItem { item, .. } if item == value_or_zero
        )
    }));
    let calls = collect_typed_calls(&main_check.typed_program);
    assert!(calls.iter().any(|call| {
        matches!(
            call.resolved_callee,
            muga::typing::TypedCalleeInfo::PackageItem { item, .. } if item == value_or_zero
        )
    }));
}

#[test]
fn interface_artifact_excludes_private_and_pkg_items() {
    let program = muga::compile_typed_path(Path::new(
        "samples/packages/app/module_visibility/main.muga",
    ))
    .expect("typed package compilation should pass");
    let text = program
        .package_interfaces()
        .to_persisted_text(&program.symbols);

    assert!(!text.contains("helper"), "{text}");
    assert!(!text.contains("PackageValue"), "{text}");
    assert!(!text.contains("PackageState"), "{text}");
}

#[test]
fn downstream_package_reports_missing_interface_export() {
    let provider = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let mut interfaces = provider.package_interfaces();
    let numbers = provider
        .package_graph
        .package_id("util::numbers")
        .expect("numbers package should exist");
    interfaces
        .packages
        .iter_mut()
        .find(|interface| interface.package == numbers)
        .expect("numbers interface should exist")
        .functions
        .retain(|function| function.name != "inc_twice");
    let text = interfaces.to_persisted_text(&provider.symbols);
    let mut symbols = provider.symbols.clone();
    let interfaces =
        muga::interface::PackageInterfaceGraph::from_persisted_text(&text, &mut symbols)
            .expect("persisted interfaces should parse");
    let root = temp_package_root("loaded-interface-missing-export");
    let entry = write_package_file(
        &root,
        "app/missing_export/main.muga",
        r#"
package app::missing_export

import util::numbers

fn main(): Int {
  numbers::inc_twice(10)
}
"#,
    );

    let diagnostics =
        muga::compile_typed_path_against_loaded_interfaces(&entry, &interfaces, &symbols)
            .unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "PK010"
                && diagnostic
                    .message
                    .contains("does not export function `inc_twice`")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn downstream_package_reports_stale_loaded_interface() {
    let provider = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let numbers = provider
        .package_graph
        .package_id("util::numbers")
        .expect("numbers package should exist");
    let mut interfaces = provider.package_interfaces();
    interfaces
        .packages
        .iter_mut()
        .find(|interface| interface.package == numbers)
        .expect("numbers interface should exist")
        .functions
        .iter_mut()
        .find(|function| function.name == "maybe_positive")
        .expect("maybe_positive should be exported")
        .ret = muga::types::TypeInfo::String;
    let text = interfaces.to_persisted_text(&provider.symbols);
    let mut symbols = provider.symbols.clone();
    let interfaces =
        muga::interface::PackageInterfaceGraph::from_persisted_text(&text, &mut symbols)
            .expect("persisted interfaces should parse");
    let root = temp_package_root("loaded-interface-stale");
    let entry = write_package_file(
        &root,
        "app/stale_interface/main.muga",
        r#"
package app::stale_interface

import util::numbers

fn main(): Int {
  maybe: Option[Int] = numbers::maybe_positive(3)
  numbers::value_or_zero(maybe)
}
"#,
    );

    let diagnostics =
        muga::compile_typed_path_against_loaded_interfaces(&entry, &interfaces, &symbols)
            .unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn package_body_checking_and_interface_checking_agree_for_existing_samples() {
    for path in [
        "samples/packages/app/main/main.muga",
        "samples/packages/app/enum_demo/main.muga",
    ] {
        let body_checked = muga::compile_typed_path(Path::new(path))
            .expect("body-based package checking should pass");
        let (interfaces, symbols) = persisted_interfaces_from_program(&body_checked);
        let interface_checked = muga::compile_typed_path_against_loaded_interfaces(
            Path::new(path),
            &interfaces,
            &symbols,
        )
        .expect("interface-based package checking should pass");

        assert_eq!(
            main_return_type(&interface_checked),
            main_return_type(&body_checked),
            "sample: {path}"
        );
    }
}

#[test]
fn package_check_finds_dependency_interface_artifact() {
    let provider = muga::compile_typed_path(Path::new("samples/packages/app/enum_demo/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = provider.package_interfaces();
    let artifact_root = temp_package_root("interface-artifact-found");
    write_interface_artifacts(
        &artifact_root,
        &interfaces,
        &provider.symbols,
        &["util::states"],
    );
    let root = temp_package_root("artifact-downstream-enum");
    let entry = write_package_file(
        &root,
        "app/artifact_enum/main.muga",
        r#"
package app::artifact_enum

import util::states

fn main(): Int {
  status: states::Status[Int] = states::ready(7)
  match status {
    states::Status::Ready(value) => value
    states::Status::Waiting => 0
    states::Status::Failed(message) => 0
  }
}
"#,
    );

    assert!(!root.join("util/states/model.muga").exists());
    muga::compile_typed_path_against_interface_artifacts(&entry, &artifact_root)
        .expect("package should check against discovered interface artifact");
}

#[test]
fn package_aware_checking_can_use_cached_interface_artifacts_without_dependency_source() {
    let provider = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = provider.package_interfaces();
    let artifact_root = temp_package_root("package-aware-interface-artifact");
    write_interface_artifacts(
        &artifact_root,
        &interfaces,
        &provider.symbols,
        &["util::numbers"],
    );
    let root = temp_package_root("package-aware-artifact-downstream");
    let entry = write_package_file(
        &root,
        "app/package_aware_artifact/main.muga",
        r#"
package app::package_aware_artifact

import util::numbers

fn main(): Int {
  maybe: Option[Int] = numbers::maybe_positive(3)
  numbers::value_or_zero(maybe)
}
"#,
    );
    muga::write_package_check_cache_artifact_for_root(&entry, &artifact_root)
        .expect("check cache artifact should be written");

    assert!(!root.join("util/numbers/option.muga").exists());
    let result =
        muga::check_package_aware_path_against_cached_artifact_root(&entry, &artifact_root)
            .expect("package-aware checking should use cached interface artifacts");
    let app = result
        .packages
        .package_graph
        .package_id("app::package_aware_artifact")
        .expect("entry package should exist");
    assert!(
        result
            .packages
            .packages
            .iter()
            .all(|package| package.path != "util::numbers"),
        "{:#?}",
        result.packages.packages
    );
    let numbers = result
        .packages
        .package_graph
        .package_id("util::numbers")
        .expect("interface package should exist");
    let value_or_zero = result
        .packages
        .package_graph
        .item_id(
            numbers,
            "value_or_zero",
            muga::package::PackageItemKind::Function,
        )
        .expect("interface function item should exist");
    assert!(
        result.signatures.function(value_or_zero).is_some(),
        "{:#?}",
        result.signatures.functions
    );
    assert!(
        result
            .module_checks
            .iter()
            .all(|check| check.package == app && check.module_path != "<interface>"),
        "{:#?}",
        result.module_checks
    );

    assert!(result.module_checks.iter().any(|check| {
        check.type_output.calls.iter().any(|call| {
            matches!(
                call.callee,
                muga::typing::TypedCalleeInfo::PackageItem { item, .. } if item == value_or_zero
            )
        })
    }));
}

#[test]
fn package_check_reports_missing_interface_artifact() {
    let artifact_root = temp_package_root("interface-artifact-missing");
    let root = temp_package_root("artifact-missing-downstream");
    let entry = write_package_file(
        &root,
        "app/missing_artifact/main.muga",
        r#"
package app::missing_artifact

import util::numbers

fn main(): Int {
  numbers::inc_twice(10)
}
"#,
    );

    let diagnostics =
        muga::compile_typed_path_against_interface_artifacts(&entry, &artifact_root).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "PK016"
                && diagnostic
                    .message
                    .contains("missing package interface artifact")
                && diagnostic.message.contains("util::numbers")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn artifact_workflow_rejects_missing_artifacts_without_source_fallback() {
    let artifact_root = temp_package_root("artifact-missing-no-source-fallback");
    let root = temp_package_root("artifact-source-present");
    let entry = write_package_file(
        &root,
        "app/source_present/main.muga",
        r#"
package app::source_present

import util::numbers

fn main(): Int {
  numbers::inc(1)
}
"#,
    );
    write_package_file(
        &root,
        "util/numbers/main.muga",
        r#"
package util::numbers

pub fn inc(value: Int): Int {
  value + 1
}
"#,
    );

    let diagnostics =
        muga::compile_typed_path_against_interface_artifacts(&entry, &artifact_root).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "PK016"
                && diagnostic
                    .message
                    .contains("missing package interface artifact")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn package_check_reports_hash_mismatched_interface_artifact() {
    let provider = muga::compile_typed_path(Path::new("samples/packages/app/enum_demo/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = provider.package_interfaces();
    let artifact_root = temp_package_root("interface-artifact-hash");
    let artifact_path =
        muga::interface::PackageInterfaceGraph::persisted_file_path(&artifact_root, "util::states");
    fs::create_dir_all(artifact_path.parent().expect("artifact should have parent"))
        .expect("artifact directory should be created");
    let tampered = interfaces
        .to_persisted_text(&provider.symbols)
        .replacen("Ready", "Done", 1);
    fs::write(&artifact_path, tampered).expect("tampered artifact should be written");
    let root = temp_package_root("artifact-hash-downstream");
    let entry = write_package_file(
        &root,
        "app/hash_mismatch/main.muga",
        r#"
package app::hash_mismatch

import util::states

fn main(): Int {
  states::value_or_zero(states::ready(1))
}
"#,
    );

    let diagnostics =
        muga::compile_typed_path_against_interface_artifacts(&entry, &artifact_root).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "PK019"),
        "{diagnostics:#?}"
    );
}

#[test]
fn package_check_rejects_stale_interface_signature() {
    let provider = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let numbers = provider
        .package_graph
        .package_id("util::numbers")
        .expect("numbers package should exist");
    let mut interfaces = provider.package_interfaces();
    interfaces
        .packages
        .iter_mut()
        .find(|interface| interface.package == numbers)
        .expect("numbers interface should exist")
        .functions
        .iter_mut()
        .find(|function| function.name == "maybe_positive")
        .expect("maybe_positive should be exported")
        .ret = muga::types::TypeInfo::String;
    let artifact_root = temp_package_root("interface-artifact-stale");
    write_interface_artifacts(
        &artifact_root,
        &interfaces,
        &provider.symbols,
        &["util::numbers"],
    );
    let root = temp_package_root("artifact-stale-downstream");
    let entry = write_package_file(
        &root,
        "app/stale_artifact/main.muga",
        r#"
package app::stale_artifact

import util::numbers

fn main(): Int {
  maybe: Option[Int] = numbers::maybe_positive(3)
  numbers::value_or_zero(maybe)
}
"#,
    );

    let diagnostics =
        muga::compile_typed_path_against_interface_artifacts(&entry, &artifact_root).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn artifact_interface_checking_and_loaded_interface_checking_agree() {
    let path = Path::new("samples/packages/app/main/main.muga");
    let provider = muga::compile_typed_path(path).expect("typed package compilation should pass");
    let (interfaces, symbols) = persisted_interfaces_from_program(&provider);
    let artifact_root = temp_package_root("interface-artifact-agree");
    write_interface_artifacts(
        &artifact_root,
        &interfaces,
        &symbols,
        &["util::numbers", "util::users"],
    );

    let loaded = muga::compile_typed_path_against_loaded_interfaces(path, &interfaces, &symbols)
        .expect("loaded-interface checking should pass");
    let artifact = muga::compile_typed_path_against_interface_artifacts(path, &artifact_root)
        .expect("artifact-interface checking should pass");

    assert_eq!(main_return_type(&artifact), main_return_type(&loaded));
}

#[test]
fn interface_artifact_checking_handles_independently_generated_package_ids() {
    let numbers_provider =
        muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
            .expect("numbers provider should typecheck");
    let states_provider =
        muga::compile_typed_path(Path::new("samples/packages/app/enum_demo/main.muga"))
            .expect("states provider should typecheck");
    let artifact_root = temp_package_root("interface-artifact-independent-ids");
    write_interface_artifacts(
        &artifact_root,
        &numbers_provider.package_interfaces(),
        &numbers_provider.symbols,
        &["util::numbers"],
    );
    write_interface_artifacts(
        &artifact_root,
        &states_provider.package_interfaces(),
        &states_provider.symbols,
        &["util::states"],
    );
    let consumer_root = temp_package_root("interface-artifact-independent-consumer");
    let consumer_entry = write_package_file(
        &consumer_root,
        "app/consumer/main.muga",
        r#"
package app::consumer

import util::numbers
import util::states

fn main(): Int {
  value = numbers::inc_twice(10)
  status: states::Status[Int] = states::ready(value)
  match status {
    states::Status::Ready(x) => x
    states::Status::Waiting => 0
    states::Status::Failed(message) => 0
  }
}
"#,
    );

    assert!(!consumer_root.join("util/numbers/main.muga").exists());
    assert!(!consumer_root.join("util/states/model.muga").exists());
    muga::check_package_aware_path_against_interface_artifacts(&consumer_entry, &artifact_root)
        .expect("independently generated package interfaces should check together");
}

#[test]
fn interface_artifact_checking_loads_transitive_public_type_interfaces() {
    let provider_root = temp_package_root("interface-artifact-transitive-provider");
    let provider_entry = write_transitive_interface_provider(&provider_root);
    let provider =
        muga::compile_typed_path(&provider_entry).expect("provider package graph should typecheck");
    let interfaces = provider.package_interfaces();
    let artifact_root = temp_package_root("interface-artifact-transitive");
    write_interface_artifacts(
        &artifact_root,
        &interfaces,
        &provider.symbols,
        &["api::facade", "model::users"],
    );
    let facade_artifact =
        muga::interface::PackageInterfaceGraph::persisted_file_path(&artifact_root, "api::facade");
    let facade_text = fs::read_to_string(&facade_artifact).expect("facade artifact should exist");
    assert!(facade_text.contains("\ndependency\tmodel::users\n"));
    let mut single_symbols = muga::symbol::SymbolTable::default();
    let single_artifact = muga::interface::PackageInterfaceGraph::read_persisted_file(
        &facade_artifact,
        &mut single_symbols,
    )
    .expect("single-package interface artifact should parse before dependency remapping");
    assert!(
        single_artifact.package_by_path("api::facade").is_some(),
        "{single_artifact:#?}"
    );
    let consumer_root = temp_package_root("interface-artifact-transitive-consumer");
    let consumer_entry = write_package_file(
        &consumer_root,
        "app/consumer/main.muga",
        r#"
package app::consumer

import api::facade

fn main(): Int {
  facade::age(facade::default_user())
}
"#,
    );

    assert!(!consumer_root.join("api/facade/main.muga").exists());
    assert!(!consumer_root.join("model/users/main.muga").exists());
    muga::compile_typed_path_against_interface_artifacts(&consumer_entry, &artifact_root)
        .expect("transitive public type interface should be loaded from artifacts");
}

#[test]
fn package_cache_key_includes_transitive_public_type_interface_hashes() {
    let provider_root = temp_package_root("cache-transitive-provider");
    let provider_entry = write_transitive_interface_provider(&provider_root);
    let provider =
        muga::compile_typed_path(&provider_entry).expect("provider package graph should typecheck");
    let mut interfaces = provider.package_interfaces();
    let artifact_root = temp_package_root("cache-transitive-artifacts");
    write_interface_artifacts(
        &artifact_root,
        &interfaces,
        &provider.symbols,
        &["api::facade", "model::users"],
    );
    let consumer_root = temp_package_root("cache-transitive-consumer");
    let consumer_entry = write_package_file(
        &consumer_root,
        "app/consumer/main.muga",
        r#"
package app::consumer

import api::facade

fn main(): Int {
  facade::age(facade::default_user())
}
"#,
    );
    let before = muga::package_check_cache_key(&consumer_entry, &artifact_root)
        .expect("cache key should include transitive interface");

    let users = provider
        .package_graph
        .package_id("model::users")
        .expect("users package should exist");
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
        .push(muga::interface::PackageInterfaceField {
            name: "active".to_string(),
            ty: muga::types::TypeInfo::Bool,
            span: Default::default(),
        });
    write_interface_artifacts(
        &artifact_root,
        &interfaces,
        &provider.symbols,
        &["model::users"],
    );
    let after = muga::package_check_cache_key(&consumer_entry, &artifact_root)
        .expect("cache key should be recomputed");

    assert_ne!(before.stable_hash(), after.stable_hash());
    assert!(
        before
            .dependency_interfaces
            .iter()
            .any(|dependency| dependency.package_path == "api::facade")
    );
    assert!(
        before
            .dependency_interfaces
            .iter()
            .any(|dependency| dependency.package_path == "model::users")
    );
}

#[test]
fn package_cache_key_changes_when_source_changes() {
    let root = temp_package_root("cache-source");
    let entry = write_package_file(
        &root,
        "app/cache_source/main.muga",
        r#"
package app::cache_source

fn main(): Int {
  1
}
"#,
    );
    let artifact_root = temp_package_root("cache-source-artifacts");
    let before = muga::package_check_cache_key(&entry, &artifact_root)
        .expect("cache key should be computed");

    fs::write(
        &entry,
        r#"
package app::cache_source

fn main(): Int {
  2
}
"#
        .trim_start(),
    )
    .expect("package file should be rewritten");
    let after = muga::package_check_cache_key(&entry, &artifact_root)
        .expect("cache key should be recomputed");

    assert_ne!(before.stable_hash(), after.stable_hash());
}

#[test]
fn package_cache_key_changes_when_dependency_interface_hash_changes() {
    let provider = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let numbers = provider
        .package_graph
        .package_id("util::numbers")
        .expect("numbers package should exist");
    let mut interfaces = provider.package_interfaces();
    let artifact_root = temp_package_root("cache-dependency-artifacts");
    write_interface_artifacts(
        &artifact_root,
        &interfaces,
        &provider.symbols,
        &["util::numbers"],
    );
    let root = temp_package_root("cache-dependency-downstream");
    let entry = write_package_file(
        &root,
        "app/cache_dependency/main.muga",
        r#"
package app::cache_dependency

import util::numbers

fn main(): Int {
  numbers::inc_twice(1)
}
"#,
    );
    let before = muga::package_check_cache_key(&entry, &artifact_root)
        .expect("cache key should be computed");

    interfaces
        .packages
        .iter_mut()
        .find(|interface| interface.package == numbers)
        .expect("numbers interface should exist")
        .functions
        .iter_mut()
        .find(|function| function.name == "inc_twice")
        .expect("inc_twice should be exported")
        .ret = muga::types::TypeInfo::String;
    write_interface_artifacts(
        &artifact_root,
        &interfaces,
        &provider.symbols,
        &["util::numbers"],
    );
    let after = muga::package_check_cache_key(&entry, &artifact_root)
        .expect("cache key should be recomputed");

    assert_ne!(before.stable_hash(), after.stable_hash());
}

#[test]
fn package_cache_rejects_missing_checked_artifact() {
    let root = temp_package_root("cache-missing-artifact");
    let entry = write_package_file(
        &root,
        "app/cache_missing/main.muga",
        r#"
package app::cache_missing

fn main(): Int {
  1
}
"#,
    );
    let artifact_root = temp_package_root("cache-missing-artifacts");
    let diagnostics = muga::compile_typed_path_against_cached_interface_artifacts(
        &entry,
        &artifact_root,
        &artifact_root.join("missing.mgc"),
    )
    .unwrap_err();

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "PK020"),
        "{diagnostics:#?}"
    );
}

#[test]
fn package_cache_rejects_stale_checked_artifact() {
    let root = temp_package_root("cache-stale-artifact");
    let entry = write_package_file(
        &root,
        "app/cache_stale/main.muga",
        r#"
package app::cache_stale

fn main(): Int {
  1
}
"#,
    );
    let artifact_root = temp_package_root("cache-stale-artifacts");
    let artifact_path = artifact_root.join("app__cache_stale.mgc");
    let key = muga::package_check_cache_key(&entry, &artifact_root)
        .expect("cache key should be computed");
    muga::write_package_check_cache_artifact(&artifact_path, &key)
        .expect("check cache artifact should be written");

    fs::write(
        &entry,
        r#"
package app::cache_stale

fn main(): Int {
  2
}
"#
        .trim_start(),
    )
    .expect("package file should be rewritten");
    let diagnostics = muga::compile_typed_path_against_cached_interface_artifacts(
        &entry,
        &artifact_root,
        &artifact_path,
    )
    .unwrap_err();

    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "PK021"),
        "{diagnostics:#?}"
    );
}

#[test]
fn package_check_cache_artifact_writer_requires_successful_body_check() {
    let root = temp_package_root("cache-artifact-invalid-body");
    let entry = write_package_file(
        &root,
        "app/cache_invalid/main.muga",
        r#"
package app::cache_invalid

fn main(): Int {
  "bad"
}
"#,
    );
    let artifact_root = temp_package_root("cache-artifact-invalid-body-artifacts");

    let diagnostics = muga::write_package_check_cache_artifact_for_root(&entry, &artifact_root)
        .expect_err("invalid package body should not write a check cache artifact");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T002"),
        "{diagnostics:#?}"
    );
    let artifact_path = muga::package_check_cache_artifact_path(&artifact_root, &entry)
        .expect("check cache path should be derived");
    assert!(!artifact_path.exists());
}

#[test]
fn cache_backed_checking_and_body_checking_agree_for_existing_samples() {
    let path = Path::new("samples/packages/app/main/main.muga");
    let provider = muga::compile_typed_path(path).expect("typed package compilation should pass");
    let (interfaces, symbols) = persisted_interfaces_from_program(&provider);
    let artifact_root = temp_package_root("cache-artifact-agree");
    write_interface_artifacts(
        &artifact_root,
        &interfaces,
        &symbols,
        &["util::numbers", "util::users"],
    );
    let artifact_path = artifact_root.join("app__main.mgc");
    let key =
        muga::package_check_cache_key(path, &artifact_root).expect("cache key should be computed");
    muga::write_package_check_cache_artifact(&artifact_path, &key)
        .expect("check cache artifact should be written");

    let cached = muga::compile_typed_path_against_cached_interface_artifacts(
        path,
        &artifact_root,
        &artifact_path,
    )
    .expect("cache-backed checking should pass");
    let body = muga::compile_typed_path(path).expect("body-based package checking should pass");

    assert_eq!(main_return_type(&cached), main_return_type(&body));
}

#[test]
fn cli_check_uses_artifact_root_without_dependency_source() {
    let provider = muga::compile_typed_path(Path::new("samples/packages/app/enum_demo/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = provider.package_interfaces();
    let artifact_root = temp_package_root("cli-artifact-root");
    write_interface_artifacts(
        &artifact_root,
        &interfaces,
        &provider.symbols,
        &["util::states"],
    );
    let root = temp_package_root("cli-artifact-downstream");
    let entry = write_package_file(
        &root,
        "app/cli_artifact/main.muga",
        r#"
package app::cli_artifact

import util::states

fn main(): Int {
  states::value_or_zero(states::ready(9))
}
"#,
    );
    let key = muga::package_check_cache_key(&entry, &artifact_root)
        .expect("cache key should be computed");
    let artifact_path = muga::package_check_cache_artifact_path(&artifact_root, &entry)
        .expect("check cache path should be derived");
    muga::write_package_check_cache_artifact(&artifact_path, &key)
        .expect("check cache artifact should be written");

    assert!(!root.join("util/states/model.muga").exists());
    let output = muga_command()
        .arg("check")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");

    assert!(output.status.success(), "{output:#?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn cli_check_reports_missing_interface_artifact() {
    let artifact_root = temp_package_root("cli-missing-interface-artifact");
    let root = temp_package_root("cli-missing-interface-downstream");
    let entry = write_package_file(
        &root,
        "app/cli_missing_interface/main.muga",
        r#"
package app::cli_missing_interface

import util::numbers

fn main(): Int {
  numbers::inc_twice(1)
}
"#,
    );
    let output = muga_command()
        .arg("check")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "{output:#?}");
    assert!(stderr.contains("PK016"), "{stderr}");
    assert!(
        stderr.contains("missing package interface artifact"),
        "{stderr}"
    );
    assert!(stderr.contains("util::numbers"), "{stderr}");
}

#[test]
fn cli_check_reports_stale_package_check_artifact() {
    let root = temp_package_root("cli-stale-check-artifact");
    let entry = write_package_file(
        &root,
        "app/cli_stale/main.muga",
        r#"
package app::cli_stale

fn main(): Int {
  1
}
"#,
    );
    let artifact_root = temp_package_root("cli-stale-artifacts");
    let key = muga::package_check_cache_key(&entry, &artifact_root)
        .expect("cache key should be computed");
    let artifact_path = muga::package_check_cache_artifact_path(&artifact_root, &entry)
        .expect("check cache path should be derived");
    muga::write_package_check_cache_artifact(&artifact_path, &key)
        .expect("check cache artifact should be written");
    fs::write(
        &entry,
        r#"
package app::cli_stale

fn main(): Int {
  2
}
"#
        .trim_start(),
    )
    .expect("package file should be rewritten");

    let output = muga_command()
        .arg("check")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "{output:#?}");
    assert!(stderr.contains("PK021"), "{stderr}");
    assert!(
        stderr.contains("stale package check cache artifact"),
        "{stderr}"
    );
}

#[test]
fn default_cli_check_accepts_package_entry() {
    let output = muga_command()
        .arg("check")
        .arg("samples/packages/app/main/main.muga")
        .output()
        .expect("muga command should run");

    assert!(output.status.success(), "{output:#?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn cli_emit_interface_writes_requested_package_artifacts() {
    let artifact_root = temp_package_root("cli-emit-interface");
    let output = muga_command()
        .arg("emit-interface")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg("--package")
        .arg("util::states")
        .arg("samples/packages/app/enum_demo/main.muga")
        .output()
        .expect("muga command should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "{output:#?}");
    assert!(stdout.contains("util__states.mgi"), "{stdout}");
    assert_eq!(stderr, "");

    let artifact_path =
        muga::interface::PackageInterfaceGraph::persisted_file_path(&artifact_root, "util::states");
    assert!(artifact_path.is_file());
    let mut symbols = muga::symbol::SymbolTable::default();
    let graph =
        muga::interface::PackageInterfaceGraph::read_persisted_file(&artifact_path, &mut symbols)
            .expect("emitted interface should parse");
    assert!(graph.package_by_path("util::states").is_some());
    assert!(graph.package_by_path("app::enum_demo").is_none());
}

#[test]
fn cli_emit_interface_without_package_writes_reachable_package_artifacts() {
    let artifact_root = temp_package_root("cli-emit-interface-all");
    let output = muga_command()
        .arg("emit-interface")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg("samples/packages/app/enum_demo/main.muga")
        .output()
        .expect("muga command should run");
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "{output:#?}");
    assert!(stdout.contains("app__enum_demo.mgi"), "{stdout}");
    assert!(stdout.contains("util__states.mgi"), "{stdout}");
    assert!(
        muga::interface::PackageInterfaceGraph::persisted_file_path(
            &artifact_root,
            "app::enum_demo"
        )
        .is_file()
    );
    assert!(
        muga::interface::PackageInterfaceGraph::persisted_file_path(&artifact_root, "util::states")
            .is_file()
    );
}

#[test]
fn cli_emit_check_cache_writes_entry_package_mgc() {
    let artifact_root = temp_package_root("cli-emit-check-cache");
    emit_states_interface(&artifact_root);
    let root = temp_package_root("cli-emit-check-cache-downstream");
    let entry = write_package_file(
        &root,
        "app/emit_cache/main.muga",
        r#"
package app::emit_cache

import util::states

fn main(): Int {
  states::value_or_zero(states::ready(3))
}
"#,
    );

    let output = muga_command()
        .arg("emit-check-cache")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "{output:#?}");
    assert!(stdout.contains("app__emit_cache.mgc"), "{stdout}");
    assert_eq!(stderr, "");
    let artifact_path = muga::package_check_cache_artifact_path(&artifact_root, &entry)
        .expect("check cache path should be derived");
    assert!(artifact_path.is_file());
}

#[test]
fn generated_artifacts_can_drive_cli_artifact_check() {
    let artifact_root = temp_package_root("cli-generated-artifacts");
    emit_states_interface(&artifact_root);
    let root = temp_package_root("cli-generated-artifacts-downstream");
    let entry = write_package_file(
        &root,
        "app/generated_artifacts/main.muga",
        r#"
package app::generated_artifacts

import util::states

fn main(): Int {
  states::value_or_zero(states::ready(5))
}
"#,
    );
    let cache = muga_command()
        .arg("emit-check-cache")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");
    assert!(cache.status.success(), "{cache:#?}");

    let output = muga_command()
        .arg("check")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");

    assert!(output.status.success(), "{output:#?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn cli_emit_artifacts_writes_interfaces_and_check_cache() {
    let artifact_root = temp_package_root("cli-emit-artifacts");
    let output = muga_command()
        .arg("emit-artifacts")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg("samples/packages/app/enum_demo/main.muga")
        .output()
        .expect("muga command should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "{output:#?}");
    assert!(stdout.contains("app__enum_demo.mgi"), "{stdout}");
    assert!(stdout.contains("util__states.mgi"), "{stdout}");
    assert!(stdout.contains("app__enum_demo.mgb"), "{stdout}");
    assert!(stdout.contains("util__states.mgb"), "{stdout}");
    assert!(stdout.contains("app__enum_demo.mgc"), "{stdout}");
    assert_eq!(stderr, "");
    assert!(
        muga::interface::PackageInterfaceGraph::persisted_file_path(
            &artifact_root,
            "app::enum_demo"
        )
        .is_file()
    );
    assert!(
        muga::interface::PackageInterfaceGraph::persisted_file_path(&artifact_root, "util::states")
            .is_file()
    );
    assert!(
        muga::implementation_artifact::persisted_file_path(&artifact_root, "app::enum_demo")
            .is_file()
    );
    assert!(
        muga::implementation_artifact::persisted_file_path(&artifact_root, "util::states")
            .is_file()
    );
    assert!(artifact_root.join("app__enum_demo.mgc").is_file());
}

#[test]
fn cli_emit_artifacts_can_drive_cli_artifact_check() {
    let artifact_root = temp_package_root("cli-emit-artifacts-check");
    let emitted = muga_command()
        .arg("emit-artifacts")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg("samples/packages/app/enum_demo/main.muga")
        .output()
        .expect("muga command should run");
    assert!(emitted.status.success(), "{emitted:#?}");

    let output = muga_command()
        .arg("check")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg("samples/packages/app/enum_demo/main.muga")
        .output()
        .expect("muga command should run");

    assert!(output.status.success(), "{output:#?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "ok\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn cli_run_uses_artifact_root_without_dependency_source() {
    let artifact_root = temp_package_root("cli-run-artifact-root");
    let emitted = muga_command()
        .arg("emit-artifacts")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg("samples/packages/app/enum_demo/main.muga")
        .output()
        .expect("muga command should run");
    assert!(emitted.status.success(), "{emitted:#?}");

    let root = temp_package_root("cli-run-artifact-downstream");
    let entry = write_package_file(
        &root,
        "app/artifact_run/main.muga",
        r#"
package app::artifact_run

import util::states

fn main(): Int {
  states::value_or_zero(states::ready(9))
}
"#,
    );
    assert!(!root.join("util/states/model.muga").exists());

    let cache = muga_command()
        .arg("emit-check-cache")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");
    assert!(cache.status.success(), "{cache:#?}");

    let output = muga_command()
        .arg("run")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");

    assert!(output.status.success(), "{output:#?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "9\n");
    assert_eq!(String::from_utf8_lossy(&output.stderr), "");
}

#[test]
fn cli_run_reports_missing_dependency_implementation_artifact() {
    let artifact_root = temp_package_root("cli-run-missing-implementation");
    let emitted = muga_command()
        .arg("emit-artifacts")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg("samples/packages/app/enum_demo/main.muga")
        .output()
        .expect("muga command should run");
    assert!(emitted.status.success(), "{emitted:#?}");
    fs::remove_file(muga::implementation_artifact::persisted_file_path(
        &artifact_root,
        "util::states",
    ))
    .expect("implementation artifact should be removable");

    let root = temp_package_root("cli-run-missing-implementation-downstream");
    let entry = write_package_file(
        &root,
        "app/missing_impl/main.muga",
        r#"
package app::missing_impl

import util::states

fn main(): Int {
  states::value_or_zero(states::ready(4))
}
"#,
    );
    let cache = muga_command()
        .arg("emit-check-cache")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");
    assert!(cache.status.success(), "{cache:#?}");

    let output = muga_command()
        .arg("run")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "{output:#?}");
    assert!(stderr.contains("PK022"), "{stderr}");
    assert!(
        stderr.contains("missing package implementation artifact"),
        "{stderr}"
    );
    assert!(stderr.contains("util::states"), "{stderr}");
}

#[test]
fn cli_run_reports_stale_dependency_implementation_artifact() {
    let artifact_root = temp_package_root("cli-run-stale-implementation");
    let emitted = muga_command()
        .arg("emit-artifacts")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg("samples/packages/app/enum_demo/main.muga")
        .output()
        .expect("muga command should run");
    assert!(emitted.status.success(), "{emitted:#?}");

    let artifact_path =
        muga::implementation_artifact::persisted_file_path(&artifact_root, "util::states");
    let mut artifact = muga::implementation_artifact::read_persisted_file(&artifact_path)
        .expect("implementation artifact should parse");
    artifact.interface_hash = "stale-interface-hash".to_string();
    artifact
        .write_persisted_artifact(&artifact_root)
        .expect("stale implementation artifact should be rewritten");

    let root = temp_package_root("cli-run-stale-implementation-downstream");
    let entry = write_package_file(
        &root,
        "app/stale_impl/main.muga",
        r#"
package app::stale_impl

import util::states

fn main(): Int {
  states::value_or_zero(states::ready(6))
}
"#,
    );
    let cache = muga_command()
        .arg("emit-check-cache")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");
    assert!(cache.status.success(), "{cache:#?}");

    let output = muga_command()
        .arg("run")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "{output:#?}");
    assert!(stderr.contains("PK023"), "{stderr}");
    assert!(
        stderr.contains("stale package implementation artifact"),
        "{stderr}"
    );
    assert!(stderr.contains("util::states"), "{stderr}");
}

#[test]
fn cli_run_reports_dependency_implementation_interface_mismatch() {
    let artifact_root = temp_package_root("cli-run-implementation-interface-mismatch");
    let emitted = muga_command()
        .arg("emit-artifacts")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg("samples/packages/app/enum_demo/main.muga")
        .output()
        .expect("muga command should run");
    assert!(emitted.status.success(), "{emitted:#?}");

    let artifact_path =
        muga::implementation_artifact::persisted_file_path(&artifact_root, "util::states");
    let mut artifact = muga::implementation_artifact::read_persisted_file(&artifact_path)
        .expect("implementation artifact should parse");
    artifact.files[0].source.push_str(
        r#"

pub fn extra_public_value(): Int {
  1
}
"#,
    );
    artifact
        .write_persisted_artifact(&artifact_root)
        .expect("mismatched implementation artifact should be rewritten");

    let root = temp_package_root("cli-run-implementation-interface-mismatch-downstream");
    let entry = write_package_file(
        &root,
        "app/impl_mismatch/main.muga",
        r#"
package app::impl_mismatch

import util::states

fn main(): Int {
  states::value_or_zero(states::ready(8))
}
"#,
    );
    let cache = muga_command()
        .arg("emit-check-cache")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");
    assert!(cache.status.success(), "{cache:#?}");

    let output = muga_command()
        .arg("run")
        .arg("--artifact-root")
        .arg(&artifact_root)
        .arg(&entry)
        .output()
        .expect("muga command should run");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(!output.status.success(), "{output:#?}");
    assert!(stderr.contains("PK023"), "{stderr}");
    assert!(stderr.contains("interface hash"), "{stderr}");
    assert!(stderr.contains("util::states"), "{stderr}");
}

#[test]
fn typed_hir_rejects_reloaded_stale_enum_interface_shape() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/enum_demo/main.muga"))
        .expect("typed package compilation should pass");
    let states = program
        .package_graph
        .package_id("util::states")
        .expect("states package should exist");
    let mut interfaces = program.package_interfaces();
    interfaces
        .packages
        .iter_mut()
        .find(|interface| interface.package == states)
        .expect("states interface should exist")
        .enums
        .iter_mut()
        .find(|enumeration| enumeration.name == "Status")
        .expect("Status should be exported")
        .variants
        .iter_mut()
        .find(|variant| variant.name == "Ready")
        .expect("Ready variant should exist")
        .payload = Some(muga::types::TypeInfo::String);

    let text = interfaces.to_persisted_text(&program.symbols);
    let mut symbols = program.symbols.clone();
    let _loaded = muga::interface::PackageInterfaceGraph::from_persisted_text(&text, &mut symbols)
        .expect("persisted interfaces should parse");
    let diagnostics = program.validate_package_references_against_interfaces(&interfaces);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "PK017")
        .expect("stale interface diagnostic should exist");
    assert!(diagnostic.message.contains("enum shape"), "{diagnostic:#?}");
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
        .ret = muga::types::TypeInfo::String;

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
        .ty = muga::types::TypeInfo::String;

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
    assert!(
        !interface
            .enums
            .iter()
            .any(|enumeration| enumeration.name == "PackageState"),
        "{interface:#?}"
    );
}

#[test]
fn package_alias_demo_runs() {
    assert_package_runs("samples/packages/app/alias_demo/main.muga", "112", "");
}

#[test]
fn package_public_enum_is_exported_and_used() {
    assert_package_runs("samples/packages/app/enum_demo/main.muga", "7", "");

    let program = muga::compile_typed_path(Path::new("samples/packages/app/enum_demo/main.muga"))
        .expect("typed package compilation should pass");
    let interfaces = program.package_interfaces();
    let states = program
        .package_graph
        .package_id("util::states")
        .expect("states package should exist");
    let status_item = program
        .package_graph
        .item_id(states, "Status", muga::package::PackageItemKind::Enum)
        .expect("Status enum item should exist");
    let status = interfaces
        .enum_by_name(states, "Status")
        .expect("Status should be exported");
    assert_eq!(status.item, status_item);
    assert_eq!(status.type_params, vec!["T"]);
    assert_eq!(status.variants.len(), 3);

    let ready = interfaces
        .function_by_name(states, "ready")
        .expect("ready should be exported");
    assert!(
        matches!(
            &ready.ret,
            muga::types::TypeInfo::PackageEnum { item, args, .. }
                if *item == status_item && args == &vec![muga::types::TypeInfo::Int]
        ),
        "{ready:#?}"
    );
}

#[test]
fn package_imported_enum_constructor_and_pattern_are_rewritten() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/enum_demo/main.muga"))
        .expect("typed package compilation should pass");
    let states = program
        .package_graph
        .package_id("util::states")
        .expect("states package should exist");
    let status_item = program
        .package_graph
        .item_id(states, "Status", muga::package::PackageItemKind::Enum)
        .expect("Status enum item should exist");
    let calls = collect_typed_calls(&program);
    let ready_constructor = calls.iter().find(|call| {
        matches!(
            call.resolved_callee,
            muga::typing::TypedCalleeInfo::EnumVariant {
                enum_name,
                enum_item: Some(enum_item),
                variant_name,
                ..
            }
                if program.symbols.resolve(enum_name) == "states::Status"
                    && enum_item == status_item
                    && program.symbols.resolve(variant_name) == "Ready"
        )
    });
    assert!(
        ready_constructor.is_some(),
        "imported enum constructor should resolve as enum variant: {calls:#?}"
    );

    let main = program
        .statements
        .iter()
        .find_map(|statement| match statement {
            muga::typed_hir::Stmt::Function(function) if function.name == "main" => Some(function),
            _ => None,
        })
        .expect("main should exist");
    let match_expr = match &main.body.expr.kind {
        muga::typed_hir::ExprKind::Match(expr) => expr,
        other => panic!("expected match expression, got {other:#?}"),
    };
    assert!(
        match_expr.arms.iter().any(|arm| {
            matches!(
                &arm.pattern,
                muga::typed_hir::MatchPattern::Variant(pattern)
                    if pattern.enum_name == "states::Status"
                        && pattern.variant_name == "Ready"
            )
        }),
        "{match_expr:#?}"
    );
}

#[test]
fn package_private_enum_from_sibling_file_is_rejected() {
    let diagnostics = muga::check_path(Path::new(
        "samples/packages/app/enum_private_visibility/main.muga",
    ))
    .unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "PK015" && diagnostic.message.contains("enum `HiddenState`")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn package_private_enum_from_import_is_rejected() {
    let diagnostics = muga::check_path(Path::new(
        "samples/packages/app/enum_private_import/main.muga",
    ))
    .unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "PK010" && diagnostic.message.contains("enum `Secret`")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn typed_hir_rejects_stale_package_interface_enum_shapes() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/enum_demo/main.muga"))
        .expect("typed package compilation should pass");
    let states = program
        .package_graph
        .package_id("util::states")
        .expect("states package should exist");
    let mut interfaces = program.package_interfaces();
    let status = interfaces
        .packages
        .iter_mut()
        .find(|interface| interface.package == states)
        .expect("states interface should exist")
        .enums
        .iter_mut()
        .find(|enumeration| enumeration.name == "Status")
        .expect("Status should be exported");
    status.type_params.push("E".to_string());
    status
        .variants
        .iter_mut()
        .find(|variant| variant.name == "Ready")
        .expect("Ready variant should exist")
        .name = "Done".to_string();
    status
        .variants
        .iter_mut()
        .find(|variant| variant.name == "Done")
        .expect("renamed variant should exist")
        .payload = Some(muga::types::TypeInfo::String);

    let diagnostics = program.validate_package_references_against_interfaces(&interfaces);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "PK017")
        .expect("stale interface diagnostic should exist");
    assert!(diagnostic.message.contains("enum shape"), "{diagnostic:#?}");
}

#[test]
fn typed_hir_rejects_stale_package_interface_enum_identity() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/enum_demo/main.muga"))
        .expect("typed package compilation should pass");
    let states = program
        .package_graph
        .package_id("util::states")
        .expect("states package should exist");
    let ready = program
        .package_graph
        .item_id(states, "ready", muga::package::PackageItemKind::Function)
        .expect("ready function item should exist");
    let mut interfaces = program.package_interfaces();
    interfaces
        .packages
        .iter_mut()
        .find(|interface| interface.package == states)
        .expect("states interface should exist")
        .enums
        .iter_mut()
        .find(|enumeration| enumeration.name == "Status")
        .expect("Status should be exported")
        .item = ready;

    let diagnostics = program.validate_package_references_against_interfaces(&interfaces);
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "PK017")
        .expect("stale interface diagnostic should exist");
    assert!(
        diagnostic.message.contains("enum identity"),
        "{diagnostic:#?}"
    );
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
fn compile_source_lowers_functions_into_mir_table() {
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
    assert_eq!(program.functions[1].params.len(), 1);
    assert_eq!(
        program.symbols.resolve(program.functions[1].params[0].name),
        "x"
    );
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
        Some(Instruction::DefineFunction { target, .. })
            if program.symbols.resolve(target.name) == "helper"
    ));
    assert!(matches!(
        program.entry.instructions.get(1),
        Some(Instruction::DefineFunction { target, .. })
            if program.symbols.resolve(target.name) == "main"
    ));
    let main_target = match program.entry.instructions.get(1) {
        Some(Instruction::DefineFunction { target, .. }) => *target,
        _ => panic!("expected main function definition"),
    };
    assert_eq!(program.main, Some(main_target));
}

#[test]
fn compile_mir_source_hoists_function_definitions_into_bodies() {
    let source = r#"
fn main(): Int {
  value = 1
  fn local(): Int {
    2
  }
  local() + value
}
"#;
    let program = muga::compile_mir_source(source).unwrap();
    assert_eq!(program.entry.statements.len(), 0);
    assert_eq!(program.entry.function_defs.len(), 1);
    assert!(matches!(
        program.entry.terminator,
        muga::mir::BodyTerminator::Effect
    ));
    let main_def = &program.entry.function_defs[0];
    assert_eq!(program.symbols.resolve(main_def.name), "main");

    let main = &program.functions[main_def.function];
    assert_eq!(main.body.function_defs.len(), 1);
    assert_eq!(main.body.statements.len(), 1);
    assert_eq!(
        program.symbols.resolve(main.body.function_defs[0].name),
        "local"
    );
    assert!(matches!(
        main.body.statements[0],
        muga::mir::Stmt::Assign(_)
    ));
    assert!(matches!(
        &main.body.terminator,
        muga::mir::BodyTerminator::Return(result)
            if matches!(result.as_ref(), muga::mir::Expr::Binary(_))
    ));
}

#[test]
fn compile_bytecode_source_emits_function_definitions_in_function_chunks() {
    let source = r#"
fn main(): Int {
  value = 1
  fn local(): Int {
    2
  }
  local() + value
}
"#;
    let program = muga::compile_bytecode_source(source).unwrap();
    let main = &program.functions[0];

    assert!(matches!(
        main.chunk.instructions.first(),
        Some(Instruction::DefineFunction { target, .. })
            if program.symbols.resolve(target.name) == "local"
    ));
    assert!(matches!(
        main.chunk.instructions.get(1),
        Some(Instruction::LoadInt(1))
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
    let (value_symbol, value_binding) = match &function.body.statements[0] {
        muga::mir::Stmt::Assign(stmt) => (stmt.name, stmt.binding),
        _ => panic!("expected assign statement"),
    };
    let (final_symbol, final_binding) = match &function.body.terminator {
        muga::mir::BodyTerminator::Return(result) => match result.as_ref() {
            muga::mir::Expr::Ident(expr) => match expr.target {
                muga::mir::IdentTarget::Binding(binding) => (expr.name, binding),
                muga::mir::IdentTarget::PackageItem { .. } => {
                    panic!("expected local binding target")
                }
            },
            _ => panic!("expected final identifier"),
        },
        muga::mir::BodyTerminator::Effect => panic!("function body should return a value"),
    };
    assert_eq!(value_symbol, final_symbol);
    assert_eq!(value_binding, final_binding);
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
    assert_eq!(main.return_ty, muga::types::TypeInfo::Int);

    let assign = match &main.body.statements[0] {
        muga::typed_hir::Stmt::Assign(assign) => assign,
        _ => panic!("expected typed assignment"),
    };
    assert!(!assign.is_update);
    assert_eq!(assign.value.ty, muga::types::TypeInfo::Int);

    let final_ident = match &main.body.expr.kind {
        muga::typed_hir::ExprKind::Ident(ident) => ident,
        _ => panic!("expected typed identifier"),
    };
    assert_eq!(final_ident.binding, assign.binding);
    assert_eq!(main.body.expr.ty, muga::types::TypeInfo::Int);
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
    let muga::ast::MatchPattern::Variant(some) = &match_expr.arms[0].pattern;
    assert_eq!(some.enum_name, "Option");
    assert_eq!(some.variant_name, "Some");
    assert_eq!(some.binding.as_deref(), Some("x"));

    let muga::ast::MatchPattern::Variant(none) = &match_expr.arms[1].pattern;
    assert_eq!(none.enum_name, "Option");
    assert_eq!(none.variant_name, "None");
    assert_eq!(none.binding, None);
}

#[test]
fn parser_preserves_result_match_patterns_as_enum_variants() {
    let source = r#"
fn main(): Int {
  value: Result[Int, String] = Result::Ok(1)
  match value {
    Result::Ok(x) => x
    Result::Err(message) => 0
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
    let muga::ast::MatchPattern::Variant(ok) = &match_expr.arms[0].pattern;
    assert_eq!(ok.enum_name, "Result");
    assert_eq!(ok.variant_name, "Ok");
    assert_eq!(ok.binding.as_deref(), Some("x"));

    let muga::ast::MatchPattern::Variant(err) = &match_expr.arms[1].pattern;
    assert_eq!(err.enum_name, "Result");
    assert_eq!(err.variant_name, "Err");
    assert_eq!(err.binding.as_deref(), Some("message"));
}

#[test]
fn parser_preserves_user_enum_declarations() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
  Error(String)
}

fn main(): Int {
  1
}
"#;
    let program = parse_source(source);
    let enumeration = match &program.statements[0] {
        muga::ast::Stmt::EnumDecl(enumeration) => enumeration,
        other => panic!("expected enum declaration, got {other:#?}"),
    };
    assert_eq!(enumeration.name, "Choice");
    assert_eq!(enumeration.type_params, vec!["T"]);
    assert_eq!(enumeration.variants.len(), 3);
    assert_eq!(enumeration.variants[0].name, "First");
    assert!(enumeration.variants[0].payload.is_some());
    assert_eq!(enumeration.variants[1].name, "Second");
    assert!(enumeration.variants[1].payload.is_none());
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
fn known_enum_metadata_describes_result_variants() {
    let result = muga::known_enum::result_enum();
    let ok = result
        .variant(muga::known_enum::RESULT_OK_NAME)
        .expect("Result should define Ok");
    let err = result
        .variant(muga::known_enum::RESULT_ERR_NAME)
        .expect("Result should define Err");

    assert_eq!(result.name, "Result");
    assert!(ok.has_payload);
    assert!(err.has_payload);
    assert_eq!(result.qualified_variant(ok), "Result::Ok");
    assert_eq!(result.qualified_variant(err), "Result::Err");
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
    assert_eq!(binding.ty, muga::types::TypeInfo::Int);
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
        muga::types::TypeInfo::List(Box::new(muga::types::TypeInfo::Int))
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
        muga::types::TypeInfo::List(Box::new(muga::types::TypeInfo::Int))
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
fn result_ok_match_sample_runs() {
    let source = r#"
fn main(): Int {
  value: Result[Int, String] = Result::Ok(10)
  match value {
    Result::Ok(x) => x
    Result::Err(message) => 0
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "10");
}

#[test]
fn result_err_match_sample_runs() {
    let source = r#"
fn main(): Int {
  value: Result[Int, String] = Result::Err("missing")
  match value {
    Result::Ok(x) => x
    Result::Err(message) => 0
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "0");
}

#[test]
fn result_runtime_display_uses_enum_value_shape() {
    let source = r#"
fn main(): Result[Int, String] {
  Result::Ok(1)
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "Result::Ok(1)");
}

#[test]
fn result_constructor_requires_expected_type() {
    let source = r#"
fn main(): Int {
  value = Result::Ok(1)
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T021"),
        "{diagnostics:#?}"
    );
}

#[test]
fn result_constructor_checks_expected_type() {
    let source = r#"
fn main(): Result[Int, String] {
  Result::Ok("bad")
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
fn result_match_requires_ok_and_err_arms() {
    let source = r#"
fn main(): Int {
  value: Result[Int, String] = Result::Ok(1)
  match value {
    Result::Ok(x) => x
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
fn result_match_arm_types_must_match() {
    let source = r#"
fn main(): Int {
  value: Result[Int, String] = Result::Err("missing")
  match value {
    Result::Ok(x) => x
    Result::Err(message) => message
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
fn user_enum_payload_match_sample_runs() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
  Error(String)
}

fn main(): Int {
  value: Choice[Int] = Choice::First(42)
  match value {
    Choice::First(x) => x
    Choice::Second => 0
    Choice::Error(message) => 1
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "42");
}

#[test]
fn user_enum_zero_payload_match_sample_runs() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
  Error(String)
}

fn main(): Int {
  value: Choice[Int] = Choice::Second
  match value {
    Choice::First(x) => x
    Choice::Second => 7
    Choice::Error(message) => 1
  }
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "7");
}

#[test]
fn user_enum_constructor_requires_expected_type() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
}

fn main(): Int {
  value = Choice::First(1)
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "T022"),
        "{diagnostics:#?}"
    );
}

#[test]
fn user_enum_constructor_payload_type_must_match() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
}

fn main(): Choice[Int] {
  Choice::First("bad")
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
fn user_enum_match_requires_all_variants() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
  Error(String)
}

fn main(): Int {
  value: Choice[Int] = Choice::First(1)
  match value {
    Choice::First(x) => x
    Choice::Second => 0
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
fn user_enum_zero_payload_constructor_requires_expected_type() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
}

fn main(): Int {
  value = Choice::Second
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "T022"
                && diagnostic
                    .message
                    .contains("requires an expected Choice[...] type")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn user_enum_unknown_variant_has_targeted_diagnostic() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
}

fn main(): Choice[Int] {
  Choice::Third(1)
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "T022"
                && diagnostic.message.contains("unknown variant `Third`")
                && diagnostic.message.contains("Choice")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn user_enum_unknown_enum_has_targeted_diagnostic() {
    let source = r#"
fn main(): Int {
  value = Missing::Ready
  1
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "T022"
                && diagnostic
                    .message
                    .contains("unknown enum `Missing` in variant constructor")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn user_enum_constructor_arity_has_targeted_diagnostic() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
}

fn main(): Choice[Int] {
  Choice::Second(1)
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "T004"
                && diagnostic
                    .message
                    .contains("expected 0 arguments but found 1")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn user_enum_match_rejects_duplicate_variant_arm() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
}

fn main(): Int {
  value: Choice[Int] = Choice::First(1)
  match value {
    Choice::First(x) => x
    Choice::First(y) => y
    Choice::Second => 0
  }
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic.code == "T018"
                && diagnostic
                    .message
                    .contains("duplicate `Choice::First` match arm")
        })
        .expect("duplicate match arm diagnostic should exist");
    assert!(
        diagnostic
            .related
            .iter()
            .any(|note| note.message.contains("previous `Choice::First` arm")),
        "{diagnostic:#?}"
    );
}

#[test]
fn user_enum_payload_binding_is_arm_local() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
}

fn main(): Int {
  value: Choice[Int] = Choice::Second
  match value {
    Choice::First(x) => x
    Choice::Second => x
  }
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "N001" && diagnostic.message.contains("unresolved name `x`")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn user_enum_match_rejects_foreign_variant() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
}

enum Other {
  First
}

fn main(): Int {
  value: Choice[Int] = Choice::First(1)
  match value {
    Choice::First(x) => x
    Other::First => 0
    Choice::Second => 0
  }
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "T018"
                && diagnostic.message.contains("Other::First")
                && diagnostic.message.contains("does not belong to Choice")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn user_enum_generic_arity_mismatch_has_targeted_diagnostic() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
}

fn main(): Choice[Int, String] {
  Choice::Second
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "T022"
                && diagnostic
                    .message
                    .contains("enum `Choice` expects exactly 1 type arguments")
        }),
        "{diagnostics:#?}"
    );
}

#[test]
fn user_enum_runtime_display_uses_enum_value_shape() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
}

fn main(): Choice[Int] {
  Choice::First(3)
}
"#;
    let result = muga::run_source(source).unwrap();
    let value = result.main_result.expect("main result should exist");
    assert_eq!(value.to_string(), "Choice::First(3)");
}

#[test]
fn parser_rejects_multi_payload_enum_variants() {
    let source = r#"
enum Bad {
  Pair(Int, String)
}
"#;
    let tokens = muga::lexer::lex(source).unwrap();
    let diagnostics = muga::parser::parse(tokens).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| diagnostic
            .message
            .contains("support at most one payload type")),
        "{diagnostics:#?}"
    );
}

#[test]
fn parser_rejects_duplicate_enum_variant_names() {
    let source = r#"
enum Bad {
  Same
  Same
}
"#;
    let diagnostics = muga::check_source(source).unwrap_err();
    assert!(
        diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "E002"
                && diagnostic
                    .message
                    .contains("duplicate variant `Same` in enum `Bad`")
        }),
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
        muga::types::TypeInfo::Option(Box::new(muga::types::TypeInfo::Int))
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
        muga::types::TypeInfo::Option(Box::new(muga::types::TypeInfo::Int))
    );
}

#[test]
fn typed_hir_preserves_result_type_info() {
    let source = r#"
fn main(): Result[Int, String] {
  value: Result[Int, String] = Result::Ok(1)
  value
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let main = match &program.statements[0] {
        muga::typed_hir::Stmt::Function(function) => function,
        _ => panic!("expected typed function"),
    };
    let result_ty = muga::types::TypeInfo::Result(
        Box::new(muga::types::TypeInfo::Int),
        Box::new(muga::types::TypeInfo::String),
    );
    assert_eq!(main.return_ty, result_ty);
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
        muga::types::TypeInfo::Result(
            Box::new(muga::types::TypeInfo::Int),
            Box::new(muga::types::TypeInfo::String)
        )
    );
}

#[test]
fn typed_hir_preserves_user_enum_type_info() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
}

fn main(): Choice[Int] {
  value: Choice[Int] = Choice::First(1)
  value
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let choice = program
        .symbols
        .lookup("Choice")
        .expect("Choice symbol should exist");
    let enum_ty = muga::types::TypeInfo::Enum {
        symbol: choice,
        args: vec![muga::types::TypeInfo::Int],
    };
    let main = match &program.statements[1] {
        muga::typed_hir::Stmt::Function(function) => function,
        _ => panic!("expected typed function"),
    };
    assert_eq!(main.return_ty, enum_ty);
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
        muga::types::TypeInfo::Enum {
            symbol: choice,
            args: vec![muga::types::TypeInfo::Int]
        }
    );
}

#[test]
fn typed_hir_preserves_user_enum_variant_call_callee() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
}

fn main(): Choice[Int] {
  Choice::First(1)
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let choice = program
        .symbols
        .lookup("Choice")
        .expect("Choice symbol should exist");
    let first = program
        .symbols
        .lookup("First")
        .expect("First symbol should exist");
    let calls = collect_typed_calls(&program);
    assert!(
        calls.iter().any(|call| {
            matches!(
                call.resolved_callee,
                muga::typing::TypedCalleeInfo::EnumVariant {
                    enum_name,
                    variant_name,
                    ..
                } if enum_name == choice && variant_name == first
            )
        }),
        "{calls:#?}"
    );
}

#[test]
fn typed_hir_preserves_user_enum_match_patterns() {
    let source = r#"
enum Choice[T] {
  First(T)
  Second
}

fn main(): Int {
  value: Choice[Int] = Choice::First(1)
  match value {
    Choice::First(x) => x
    Choice::Second => 0
  }
}
"#;
    let program = muga::compile_typed_source(source).unwrap();
    let main = match &program.statements[1] {
        muga::typed_hir::Stmt::Function(function) => function,
        _ => panic!("expected typed function"),
    };
    let match_expr = match &main.body.expr.kind {
        muga::typed_hir::ExprKind::Match(expr) => expr,
        other => panic!("expected match expression, got {other:#?}"),
    };
    assert_eq!(match_expr.arms.len(), 2);
    assert!(
        match_expr.arms.iter().any(|arm| {
            matches!(
                &arm.pattern,
                muga::typed_hir::MatchPattern::Variant(pattern)
                    if pattern.enum_name == "Choice" && pattern.variant_name == "First"
            )
        }),
        "{match_expr:#?}"
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
    let muga::typed_hir::MatchPattern::Variant(some) = &match_expr.arms[0].pattern;
    assert_eq!(some.enum_name, "Option");
    assert_eq!(some.variant_name, "Some");
    assert_eq!(some.binding_name.as_deref(), Some("x"));
    assert!(some.binding.is_some());

    let muga::typed_hir::MatchPattern::Variant(none) = &match_expr.arms[1].pattern;
    assert_eq!(none.enum_name, "Option");
    assert_eq!(none.variant_name, "None");
    assert_eq!(none.binding_name, None);
    assert_eq!(none.binding, None);
}

#[test]
fn typed_hir_preserves_result_match_patterns_as_enum_variants() {
    let source = r#"
fn main(): Int {
  value: Result[Int, String] = Result::Ok(1)
  match value {
    Result::Ok(x) => x
    Result::Err(message) => 0
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
    let muga::typed_hir::MatchPattern::Variant(ok) = &match_expr.arms[0].pattern;
    assert_eq!(ok.enum_name, "Result");
    assert_eq!(ok.variant_name, "Ok");
    assert_eq!(ok.binding_name.as_deref(), Some("x"));
    assert!(ok.binding.is_some());

    let muga::typed_hir::MatchPattern::Variant(err) = &match_expr.arms[1].pattern;
    assert_eq!(err.enum_name, "Result");
    assert_eq!(err.variant_name, "Err");
    assert_eq!(err.binding_name.as_deref(), Some("message"));
    assert!(err.binding.is_some());
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
    let map_ty = muga::types::TypeInfo::Map(
        Box::new(muga::types::TypeInfo::String),
        Box::new(muga::types::TypeInfo::Int),
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
fn compile_mir_source_marks_mutable_updates() {
    let source = r#"
fn main(): Int {
  mut value = 1
  value = 2
  value
}
"#;
    let program = muga::compile_mir_source(source).unwrap();
    let main = &program.functions[0];
    let first = match &main.body.statements[0] {
        muga::mir::Stmt::Assign(assign) => assign,
        _ => panic!("expected first assignment"),
    };
    let second = match &main.body.statements[1] {
        muga::mir::Stmt::Assign(assign) => assign,
        _ => panic!("expected second assignment"),
    };

    assert!(!first.is_update);
    assert!(second.is_update);
    assert_eq!(first.binding, second.binding);
}

#[test]
fn compile_bytecode_source_marks_mutable_updates() {
    let source = r#"
fn main(): Int {
  mut value = 1
  value = 2
  value
}
"#;
    let program = muga::compile_bytecode_source(source).unwrap();
    let main = &program.functions[0];
    let update_modes = main
        .chunk
        .instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::Assign { is_update, .. } => Some(*is_update),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(update_modes, vec![false, true]);
}

#[test]
fn compile_bytecode_source_uses_binding_refs_for_runtime_names() {
    let source = r#"
fn main(): Int {
  mut value = 1
  value = 2
  value
}
"#;
    let program = muga::compile_bytecode_source(source).unwrap();
    let main = &program.functions[0];
    let assign_targets = main
        .chunk
        .instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::Assign { target, .. } => Some(*target),
            _ => None,
        })
        .collect::<Vec<_>>();
    let load_targets = main
        .chunk
        .instructions
        .iter()
        .filter_map(|instruction| match instruction {
            Instruction::LoadName { target, .. } => Some(*target),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(assign_targets.len(), 2);
    assert_eq!(load_targets.len(), 1);
    assert_eq!(assign_targets[0], assign_targets[1]);
    assert_eq!(assign_targets[0], load_targets[0]);
    assert_eq!(program.symbols.resolve(assign_targets[0].name), "value");
    let target_binding = assign_targets[0]
        .binding
        .expect("value name ref should preserve semantic binding");
    let binding = program
        .bindings
        .iter()
        .find(|binding| binding.id == target_binding)
        .expect("value binding should be preserved");
    assert_eq!(binding.local, assign_targets[0].local);
    assert!(program.locals.iter().any(|local| {
        local.id == assign_targets[0].local && local.binding == Some(target_binding)
    }));
    assert!(program.local_count > assign_targets[0].local.as_u32() as usize);
}

#[test]
fn compile_bytecode_source_records_synthetic_match_locals() {
    let source = r#"
fn main(): Int {
  value: Option[Int] = Option::Some(1)
  match value {
    Option::Some(x) => x
    Option::None => 0
  }
}
"#;
    let program = muga::compile_bytecode_source(source).unwrap();
    let local = program
        .locals
        .iter()
        .find(|local| matches!(local.kind, muga::bytecode::LocalKind::Synthetic))
        .expect("match temporary should be recorded as a synthetic local");

    assert!(
        program
            .symbols
            .resolve(local.name)
            .starts_with("__muga_match_value_")
    );
    assert_eq!(local.binding, None);
    assert!(program.local_count > local.id.as_u32() as usize);
}

#[test]
fn runtime_reports_non_function_top_level_main_from_bytecode_entrypoint() {
    let diagnostics = muga::run_source("main = 1").expect_err("main value should not be callable");
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "R002"),
        "{diagnostics:#?}"
    );
}

#[test]
fn compile_bytecode_path_uses_package_definition_binding_for_runtime_names() {
    let program =
        muga::compile_bytecode_path(Path::new("samples/packages/app/main/main.muga")).unwrap();
    let package_name = "__muga_pkg__util__numbers__inc_twice";
    let mut definition_targets = Vec::new();
    let mut load_targets = Vec::new();

    for instruction in &program.entry.instructions {
        match instruction {
            Instruction::DefineFunction { target, .. }
                if program.symbols.resolve(target.name) == package_name =>
            {
                definition_targets.push(*target);
            }
            Instruction::LoadName { target, .. }
                if program.symbols.resolve(target.name) == package_name =>
            {
                load_targets.push(*target);
            }
            _ => {}
        }
    }
    for function in &program.functions {
        for instruction in &function.chunk.instructions {
            match instruction {
                Instruction::DefineFunction { target, .. }
                    if program.symbols.resolve(target.name) == package_name =>
                {
                    definition_targets.push(*target);
                }
                Instruction::LoadName { target, .. }
                    if program.symbols.resolve(target.name) == package_name =>
                {
                    load_targets.push(*target);
                }
                _ => {}
            }
        }
    }
    let import = program
        .bindings
        .iter()
        .find(|binding| program.symbols.resolve(binding.name) == "numbers::inc_twice")
        .expect("import binding should be preserved");

    assert_eq!(definition_targets.len(), 1, "{definition_targets:#?}");
    assert!(!load_targets.is_empty(), "{load_targets:#?}");
    assert!(
        load_targets
            .iter()
            .all(|target| target.binding == definition_targets[0].binding
                && target.local == definition_targets[0].local),
        "{load_targets:#?}"
    );
    assert!(
        definition_targets[0].binding.is_some(),
        "{definition_targets:#?}"
    );
    assert_ne!(definition_targets[0].binding, Some(import.id));
    assert_ne!(definition_targets[0].local, import.local);
    assert!(program.locals.iter().any(|local| {
        local.id == definition_targets[0].local && local.binding == definition_targets[0].binding
    }));
    assert!(program.local_count > definition_targets[0].local.as_u32() as usize);
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
    assert_eq!(value_binding.ty, muga::types::TypeInfo::Int);

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
    assert_eq!(value_expr_type.ty, muga::types::TypeInfo::Int);
}

#[test]
fn typechecker_builtin_type_info_uses_builtin_ids() {
    let program = parse_source(
        r#"
fn main(): Option[Int] {
  Option::None
}
"#,
    );
    let output = muga::typing::typecheck_program(&program);
    assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);

    let none = output
        .bindings
        .iter()
        .find(|binding| output.symbols.resolve(binding.symbol) == "Option::None")
        .expect("Option::None prelude binding should be exposed");
    assert_eq!(
        none.ty,
        muga::types::TypeInfo::Builtin(muga::prelude::BuiltinId::OptionNone)
    );

    let some = output
        .bindings
        .iter()
        .find(|binding| output.symbols.resolve(binding.symbol) == "Option::Some")
        .expect("Option::Some prelude binding should be exposed");
    assert_eq!(
        some.ty,
        muga::types::TypeInfo::Builtin(muga::prelude::BuiltinId::OptionSome)
    );
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
        muga::types::TypeInfo::Function(muga::types::FunctionTypeInfo {
            params: vec![muga::types::TypeInfo::Int],
            ret: Box::new(muga::types::TypeInfo::Int),
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
                    name: muga::prelude::builtin_name(muga::prelude::BuiltinId::Println),
                }
        }),
        "{calls:#?}"
    );
}

#[test]
fn typed_hir_preserves_package_qualified_call_callee() {
    let program = muga::compile_typed_path(Path::new("samples/packages/app/main/main.muga"))
        .expect("typed package compilation should pass");
    let inc_twice = typed_binding_id(&program, "numbers::inc_twice");
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
            muga::types::TypeInfo::PackageRecord { item, .. } if *item == user_item
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

fn persisted_interfaces_from_program(
    program: &muga::typed_hir::Program,
) -> (
    muga::interface::PackageInterfaceGraph,
    muga::symbol::SymbolTable,
) {
    let text = program
        .package_interfaces()
        .to_persisted_text(&program.symbols);
    let mut symbols = program.symbols.clone();
    let interfaces =
        muga::interface::PackageInterfaceGraph::from_persisted_text(&text, &mut symbols)
            .expect("persisted interfaces should parse");
    (interfaces, symbols)
}

fn temp_package_root(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("muga-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("temp package root should be created");
    root
}

fn write_package_file(root: &Path, relative: &str, source: &str) -> std::path::PathBuf {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("package file should have parent"))
        .expect("package directory should be created");
    fs::write(&path, source.trim_start()).expect("package file should be written");
    path
}

fn write_interface_artifacts(
    root: &Path,
    interfaces: &muga::interface::PackageInterfaceGraph,
    symbols: &muga::symbol::SymbolTable,
    package_paths: &[&str],
) {
    fs::create_dir_all(root).expect("interface artifact root should be created");
    for package_path in package_paths {
        interfaces
            .write_persisted_artifact(root, package_path, symbols)
            .expect("interface artifact should be written");
    }
}

fn write_transitive_interface_provider(root: &Path) -> std::path::PathBuf {
    write_package_file(
        root,
        "model/users/main.muga",
        r#"
package model::users

pub record User {
  name: String,
  age: Int
}
"#,
    );
    write_package_file(
        root,
        "api/facade/main.muga",
        r#"
package api::facade

import model::users

pub fn default_user(): users::User {
  users::User { name: "Ada", age: 21 }
}

pub fn age(user: users::User): Int {
  user.age
}
"#,
    );
    write_package_file(
        root,
        "app/provider/main.muga",
        r#"
package app::provider

import api::facade

fn main(): Int {
  facade::age(facade::default_user())
}
"#,
    )
}

fn muga_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_muga"))
}

fn emit_states_interface(artifact_root: &Path) {
    let output = muga_command()
        .arg("emit-interface")
        .arg("--artifact-root")
        .arg(artifact_root)
        .arg("--package")
        .arg("util::states")
        .arg("samples/packages/app/enum_demo/main.muga")
        .output()
        .expect("muga command should run");
    assert!(output.status.success(), "{output:#?}");
}

fn main_return_type(program: &muga::typed_hir::Program) -> Option<muga::types::TypeInfo> {
    program
        .statements
        .iter()
        .find_map(|statement| match statement {
            muga::typed_hir::Stmt::Function(function) if function.name == "main" => {
                Some(function.return_ty.clone())
            }
            _ => None,
        })
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
        muga::typed_hir::Stmt::Enum(_) => {}
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

fn assert_unique_typed_ids(program: &muga::typed_hir::Program) {
    let mut statement_ids = HashSet::new();
    let mut expr_ids = HashSet::new();
    for statement in &program.statements {
        collect_typed_ids_in_stmt(statement, &mut statement_ids, &mut expr_ids);
    }
}

fn collect_typed_ids_in_stmt(
    statement: &muga::typed_hir::Stmt,
    statement_ids: &mut HashSet<u32>,
    expr_ids: &mut HashSet<u32>,
) {
    assert!(
        statement_ids.insert(statement.id().as_u32()),
        "duplicate typed statement id: {}",
        statement.id().as_u32()
    );
    match statement {
        muga::typed_hir::Stmt::Assign(stmt) => {
            collect_typed_ids_in_expr(&stmt.value, statement_ids, expr_ids);
        }
        muga::typed_hir::Stmt::Record(_) | muga::typed_hir::Stmt::Enum(_) => {}
        muga::typed_hir::Stmt::Function(stmt) => {
            collect_typed_ids_in_value_block(&stmt.body, statement_ids, expr_ids);
        }
        muga::typed_hir::Stmt::If(stmt) => {
            collect_typed_ids_in_expr(&stmt.condition, statement_ids, expr_ids);
            collect_typed_ids_in_block(&stmt.then_branch, statement_ids, expr_ids);
            if let Some(else_branch) = &stmt.else_branch {
                collect_typed_ids_in_block(else_branch, statement_ids, expr_ids);
            }
        }
        muga::typed_hir::Stmt::While(stmt) => {
            collect_typed_ids_in_expr(&stmt.condition, statement_ids, expr_ids);
            collect_typed_ids_in_block(&stmt.body, statement_ids, expr_ids);
        }
        muga::typed_hir::Stmt::Expr(stmt) => {
            collect_typed_ids_in_expr(&stmt.expr, statement_ids, expr_ids);
        }
    }
}

fn collect_typed_ids_in_block(
    block: &muga::typed_hir::Block,
    statement_ids: &mut HashSet<u32>,
    expr_ids: &mut HashSet<u32>,
) {
    for statement in &block.statements {
        collect_typed_ids_in_stmt(statement, statement_ids, expr_ids);
    }
}

fn collect_typed_ids_in_value_block(
    block: &muga::typed_hir::ValueBlock,
    statement_ids: &mut HashSet<u32>,
    expr_ids: &mut HashSet<u32>,
) {
    for statement in &block.statements {
        collect_typed_ids_in_stmt(statement, statement_ids, expr_ids);
    }
    collect_typed_ids_in_expr(&block.expr, statement_ids, expr_ids);
}

fn collect_typed_ids_in_expr(
    expr: &muga::typed_hir::Expr,
    statement_ids: &mut HashSet<u32>,
    expr_ids: &mut HashSet<u32>,
) {
    assert!(
        expr_ids.insert(expr.id.as_u32()),
        "duplicate typed expression id: {}",
        expr.id.as_u32()
    );
    match &expr.kind {
        muga::typed_hir::ExprKind::Int(_)
        | muga::typed_hir::ExprKind::Bool(_)
        | muga::typed_hir::ExprKind::String(_)
        | muga::typed_hir::ExprKind::Ident(_) => {}
        muga::typed_hir::ExprKind::ListLit(expr) => {
            for item in &expr.items {
                collect_typed_ids_in_expr(item, statement_ids, expr_ids);
            }
        }
        muga::typed_hir::ExprKind::Index(expr) => {
            collect_typed_ids_in_expr(&expr.base, statement_ids, expr_ids);
            collect_typed_ids_in_expr(&expr.index, statement_ids, expr_ids);
        }
        muga::typed_hir::ExprKind::RecordLit(expr) => {
            for field in &expr.fields {
                collect_typed_ids_in_expr(&field.value, statement_ids, expr_ids);
            }
        }
        muga::typed_hir::ExprKind::Field(expr) => {
            collect_typed_ids_in_expr(&expr.base, statement_ids, expr_ids);
        }
        muga::typed_hir::ExprKind::RecordUpdate(expr) => {
            collect_typed_ids_in_expr(&expr.base, statement_ids, expr_ids);
            for field in &expr.fields {
                collect_typed_ids_in_expr(&field.value, statement_ids, expr_ids);
            }
        }
        muga::typed_hir::ExprKind::Unary(expr) => {
            collect_typed_ids_in_expr(&expr.expr, statement_ids, expr_ids);
        }
        muga::typed_hir::ExprKind::Binary(expr) => {
            collect_typed_ids_in_expr(&expr.left, statement_ids, expr_ids);
            collect_typed_ids_in_expr(&expr.right, statement_ids, expr_ids);
        }
        muga::typed_hir::ExprKind::Call(expr) => {
            collect_typed_ids_in_expr(&expr.callee, statement_ids, expr_ids);
            for arg in &expr.args {
                collect_typed_ids_in_expr(arg, statement_ids, expr_ids);
            }
        }
        muga::typed_hir::ExprKind::If(expr) => {
            collect_typed_ids_in_expr(&expr.condition, statement_ids, expr_ids);
            collect_typed_ids_in_value_block(&expr.then_branch, statement_ids, expr_ids);
            collect_typed_ids_in_value_block(&expr.else_branch, statement_ids, expr_ids);
        }
        muga::typed_hir::ExprKind::Match(expr) => {
            collect_typed_ids_in_expr(&expr.value, statement_ids, expr_ids);
            for arm in &expr.arms {
                collect_typed_ids_in_expr(&arm.value, statement_ids, expr_ids);
            }
        }
        muga::typed_hir::ExprKind::Fn(expr) => {
            collect_typed_ids_in_value_block(&expr.body, statement_ids, expr_ids);
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
