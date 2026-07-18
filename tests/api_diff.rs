use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use muga::{
    api_diff::{PackageApiDiffClassification, PackageApiDiffStatus, diff_package_interfaces},
    identity::{PackageId, PackageItemId},
    interface::{
        OpaqueHandleFacts, PackageInterface, PackageInterfaceEnum, PackageInterfaceEnumVariant,
        PackageInterfaceField, PackageInterfaceFunction, PackageInterfaceGraph,
        PackageInterfaceOpaqueType, PackageInterfaceParam, PackageInterfaceParamMode,
        PackageInterfaceRecord,
    },
    span::Span,
    symbol::SymbolTable,
    types::TypeInfo,
};

#[test]
fn package_api_diff_reports_compatible_for_span_only_changes() {
    let symbols = SymbolTable::default();
    let old = graph(vec![function(
        1,
        "parse",
        Vec::new(),
        vec![param(
            "input",
            TypeInfo::String,
            PackageInterfaceParamMode::Borrow,
        )],
        TypeInfo::Int,
    )]);
    let mut new = old.clone();
    new.packages[0].functions[0].span = Span::single(muga::span::Position::new(20, 4));

    let diff = diff_package_interfaces(&old, &new, "app::api", &symbols);

    assert_eq!(diff.status, PackageApiDiffStatus::Compatible);
    assert_eq!(diff.summary.compatible, 1);
    assert!(diff.changes.is_empty(), "{diff:#?}");
}

#[test]
fn package_api_diff_reports_source_compatible_public_additions_and_renames() {
    let mut symbols = SymbolTable::default();
    let t = symbols.intern("T");
    let value = symbols.intern("Value");
    let old = graph(vec![function(
        1,
        "identity",
        vec!["T".to_string()],
        vec![param(
            "input",
            TypeInfo::GenericParam(t),
            PackageInterfaceParamMode::Consume,
        )],
        TypeInfo::GenericParam(t),
    )]);
    let new = graph(vec![
        function(
            1,
            "identity",
            vec!["Value".to_string()],
            vec![param(
                "source",
                TypeInfo::GenericParam(value),
                PackageInterfaceParamMode::Borrow,
            )],
            TypeInfo::GenericParam(value),
        ),
        function(
            2,
            "format",
            Vec::new(),
            vec![param(
                "value",
                TypeInfo::Int,
                PackageInterfaceParamMode::Borrow,
            )],
            TypeInfo::String,
        ),
    ]);

    let diff = diff_package_interfaces(&old, &new, "app::api", &symbols);
    let kinds = change_kinds(&diff);

    assert_eq!(diff.status, PackageApiDiffStatus::SourceCompatible);
    assert_eq!(diff.summary.source_compatible, 4);
    assert!(kinds.contains("function-added"), "{diff:#?}");
    assert!(kinds.contains("function-parameter-renamed"), "{diff:#?}");
    assert!(
        kinds.contains("function-parameter-mode-relaxed"),
        "{diff:#?}"
    );
    assert!(
        kinds.contains("function-type-parameters-renamed"),
        "{diff:#?}"
    );
    assert_eq!(diff.summary.breaking, 0);
    assert_eq!(diff.summary.unknown, 0);
}

#[test]
fn package_api_diff_reports_breaking_public_shape_changes() {
    let old = PackageInterfaceGraph {
        packages: vec![PackageInterface {
            package: PackageId::new(1),
            path: "app::api".to_string(),
            dependencies: Vec::new(),
            records: vec![record(1, "User", vec![field("name", TypeInfo::String)])],
            enums: vec![enumeration(2, "Status", vec![variant("Ready", None)])],
            opaque_types: Vec::new(),
            functions: vec![function(
                3,
                "load",
                Vec::new(),
                vec![param(
                    "id",
                    TypeInfo::Int,
                    PackageInterfaceParamMode::Borrow,
                )],
                TypeInfo::String,
            )],
        }],
    };
    let new = PackageInterfaceGraph {
        packages: vec![PackageInterface {
            package: PackageId::new(1),
            path: "app::api".to_string(),
            dependencies: Vec::new(),
            records: vec![record(
                1,
                "User",
                vec![field("name", TypeInfo::Int), field("age", TypeInfo::Int)],
            )],
            enums: vec![enumeration(
                2,
                "Status",
                vec![variant("Ready", None), variant("Pending", None)],
            )],
            opaque_types: Vec::new(),
            functions: vec![function(
                3,
                "load",
                Vec::new(),
                vec![param(
                    "id",
                    TypeInfo::String,
                    PackageInterfaceParamMode::Borrow,
                )],
                TypeInfo::String,
            )],
        }],
    };
    let symbols = SymbolTable::default();

    let diff = diff_package_interfaces(&old, &new, "app::api", &symbols);
    let kinds = change_kinds(&diff);

    assert_eq!(diff.status, PackageApiDiffStatus::Breaking);
    assert!(kinds.contains("record-field-type-changed"), "{diff:#?}");
    assert!(kinds.contains("record-field-added"), "{diff:#?}");
    assert!(kinds.contains("enum-variant-added"), "{diff:#?}");
    assert!(
        kinds.contains("function-parameter-type-changed"),
        "{diff:#?}"
    );
    assert!(
        diff.changes.iter().any(
            |change| change.kind == "record-field-added" && change.path == "app::api::User.age"
        ),
        "{diff:#?}"
    );
    assert!(
        diff.changes
            .iter()
            .any(|change| change.kind == "enum-variant-added"
                && change.path == "app::api::Status.Pending"),
        "{diff:#?}"
    );
}

#[test]
fn package_api_diff_fails_closed_for_unknown_opaque_handle_facts() {
    let old = graph_with_opaque(OpaqueHandleFacts {
        runtime_backed: false,
        ..OpaqueHandleFacts::default()
    });
    let new = graph_with_opaque(OpaqueHandleFacts {
        runtime_backed: true,
        ..OpaqueHandleFacts::default()
    });
    let symbols = SymbolTable::default();

    let diff = diff_package_interfaces(&old, &new, "app::api", &symbols);

    assert_eq!(diff.status, PackageApiDiffStatus::Unknown);
    assert_eq!(diff.summary.unknown, 1);
    assert_eq!(
        diff.changes[0].classification,
        PackageApiDiffClassification::Unknown
    );
    assert_eq!(diff.changes[0].kind, "opaque-handle-fact-changed");
}

#[test]
fn cli_api_diff_reports_text_and_json_from_artifacts() {
    let workspace = temp_api_diff_root("cli-api-diff");
    let project = workspace.join("project");
    write_package_file(
        &project,
        "muga.toml",
        r#"
[package]
name = "api_app"
language_revision = 1
source = "src"
"#,
    );
    let entry = write_package_file(
        &project,
        "src/main/main.muga",
        r#"
pub fn answer(): Int {
  1
}

fn main(): Int {
  answer()
}
"#,
    );
    let old_artifacts = workspace.join("old-artifacts");
    muga::write_package_artifacts(&entry, &old_artifacts).expect("old artifacts should build");

    write_package_file(
        &project,
        "src/main/main.muga",
        r#"
pub fn answer(): String {
  "one"
}

fn main(): String {
  answer()
}
"#,
    );
    let new_artifacts = workspace.join("new-artifacts");
    muga::write_package_artifacts(&entry, &new_artifacts).expect("new artifacts should build");

    let text = muga_command()
        .arg("api-diff")
        .arg("--old-artifact-root")
        .arg(&old_artifacts)
        .arg("--new-artifact-root")
        .arg(&new_artifacts)
        .arg("--package")
        .arg("api_app::main")
        .output()
        .expect("muga api-diff text should run");
    assert!(text.status.success(), "{text:?}");
    let stdout = String::from_utf8(text.stdout).expect("api-diff text should be UTF-8");
    assert!(stdout.contains("status\tbreaking"), "{stdout}");
    assert!(stdout.contains("function-return-type-changed"), "{stdout}");
    assert!(stdout.contains("api_app::main::answer"), "{stdout}");

    let json = muga_command()
        .arg("api-diff")
        .arg("--format")
        .arg("json")
        .arg("--old-artifact-root")
        .arg(&old_artifacts)
        .arg("--new-artifact-root")
        .arg(&new_artifacts)
        .arg("--package")
        .arg("api_app::main")
        .output()
        .expect("muga api-diff JSON should run");
    assert!(json.status.success(), "{json:?}");
    let stdout = String::from_utf8(json.stdout).expect("api-diff JSON should be UTF-8");
    assert!(stdout.contains("\"command\":\"api-diff\""), "{stdout}");
    assert!(stdout.contains("\"status\":\"breaking\""), "{stdout}");
    assert!(
        stdout.contains("\"kind\":\"function-return-type-changed\""),
        "{stdout}"
    );

    let gated = muga_command()
        .arg("api-diff")
        .arg("--fail-on")
        .arg("breaking")
        .arg("--old-artifact-root")
        .arg(&old_artifacts)
        .arg("--new-artifact-root")
        .arg(&new_artifacts)
        .arg("--package")
        .arg("api_app::main")
        .output()
        .expect("muga api-diff fail-on threshold should run");
    assert!(!gated.status.success(), "{gated:?}");
    let stdout = String::from_utf8(gated.stdout).expect("api-diff text should be UTF-8");
    assert!(stdout.contains("status\tbreaking"), "{stdout}");
}

#[test]
fn cli_api_diff_validates_arguments_and_reports_json_diagnostics() {
    let missing_roots = muga_command()
        .arg("api-diff")
        .output()
        .expect("muga api-diff validation should run");
    assert!(!missing_roots.status.success(), "{missing_roots:?}");
    let stderr = String::from_utf8(missing_roots.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("api-diff requires --old-artifact-root"),
        "{stderr}"
    );

    let with_source = muga_command()
        .arg("api-diff")
        .arg("--old-artifact-root")
        .arg("old")
        .arg("--new-artifact-root")
        .arg("new")
        .arg("--package")
        .arg("api_app::main")
        .arg("main.muga")
        .output()
        .expect("muga api-diff source-file validation should run");
    assert!(!with_source.status.success(), "{with_source:?}");
    let stderr = String::from_utf8(with_source.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("api-diff does not accept a source file"),
        "{stderr}"
    );

    let workspace = temp_api_diff_root("cli-api-diff-json-error");
    let old_artifacts = workspace.join("missing-old");
    let new_artifacts = workspace.join("missing-new");
    let json_error = muga_command()
        .arg("api-diff")
        .arg("--format")
        .arg("json")
        .arg("--old-artifact-root")
        .arg(old_artifacts)
        .arg("--new-artifact-root")
        .arg(new_artifacts)
        .arg("--package")
        .arg("api_app::main")
        .output()
        .expect("muga api-diff JSON diagnostics should run");
    assert!(!json_error.status.success(), "{json_error:?}");
    let stdout = String::from_utf8(json_error.stdout).expect("stdout should be UTF-8");
    assert!(stdout.contains("\"command\":\"api-diff\""), "{stdout}");
    assert!(stdout.contains("\"status\":\"error\""), "{stdout}");
    assert!(stdout.contains("\"package\":\"api_app::main\""), "{stdout}");
    assert!(stdout.contains("\"diagnostics\":["), "{stdout}");

    let invalid_fail_on = muga_command()
        .arg("api-diff")
        .arg("--fail-on")
        .arg("compatible")
        .output()
        .expect("muga api-diff fail-on validation should run");
    assert!(!invalid_fail_on.status.success(), "{invalid_fail_on:?}");
    let stderr = String::from_utf8(invalid_fail_on.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("unknown api-diff --fail-on value `compatible`"),
        "{stderr}"
    );

    let wrong_mode_fail_on = muga_command()
        .arg("check")
        .arg("--fail-on")
        .arg("breaking")
        .arg("samples/println_sum.muga")
        .output()
        .expect("muga non-api-diff fail-on validation should run");
    assert!(
        !wrong_mode_fail_on.status.success(),
        "{wrong_mode_fail_on:?}"
    );
    let stderr = String::from_utf8(wrong_mode_fail_on.stderr).expect("stderr should be UTF-8");
    assert!(
        stderr.contains("--fail-on is only supported with `api-diff`"),
        "{stderr}"
    );
}

#[test]
fn cli_api_diff_reports_compatible_for_persisted_implementation_only_edits() {
    let workspace = temp_api_diff_root("cli-api-diff-implementation-only");
    let project = workspace.join("project");
    write_api_diff_manifest(&project);
    let entry = write_package_file(
        &project,
        "src/main/main.muga",
        r#"
pub record Box[T] {
  value: T
}

pub enum Status[T] {
  Ready(T)
  Waiting
}

pub fn wrap(value: Int): Box[Int] {
  Box {
    value: value
  }
}

pub fn current(value: Int): Status[Int] {
  Status::Ready(value)
}

fn helper(value: Int): Int {
  value + 1
}

fn main(): Int {
  helper(wrap(1).value)
}
"#,
    );
    let old_artifacts = workspace.join("old-artifacts");
    muga::write_package_artifacts(&entry, &old_artifacts).expect("old artifacts should build");

    write_package_file(
        &project,
        "src/main/main.muga",
        r#"
pub record Box[T] {
  value: T
}

pub enum Status[T] {
  Ready(T)
  Waiting
}

pub fn wrap(value: Int): Box[Int] {
  Box {
    value: value
  }
}

pub fn current(value: Int): Status[Int] {
  Status::Ready(value)
}

fn helper(value: Int): Int {
  value + 2
}

fn main(): Int {
  helper(wrap(1).value)
}
"#,
    );
    let new_artifacts = workspace.join("new-artifacts");
    muga::write_package_artifacts(&entry, &new_artifacts).expect("new artifacts should build");

    let json = muga_command()
        .arg("api-diff")
        .arg("--format")
        .arg("json")
        .arg("--fail-on")
        .arg("breaking")
        .arg("--old-artifact-root")
        .arg(&old_artifacts)
        .arg("--new-artifact-root")
        .arg(&new_artifacts)
        .arg("--package")
        .arg("api_app::main")
        .output()
        .expect("muga api-diff JSON should run");
    assert!(json.status.success(), "{json:?}");
    let stdout = String::from_utf8(json.stdout).expect("api-diff JSON should be UTF-8");
    assert!(stdout.contains("\"status\":\"compatible\""), "{stdout}");
    assert!(stdout.contains("\"compatible\":1"), "{stdout}");
    assert!(stdout.contains("\"changes\":[]"), "{stdout}");
}

#[test]
fn cli_api_diff_reports_record_enum_and_generic_changes_from_artifacts() {
    let workspace = temp_api_diff_root("cli-api-diff-public-shapes");
    let project = workspace.join("project");
    write_api_diff_manifest(&project);
    let entry = write_package_file(
        &project,
        "src/main/main.muga",
        r#"
pub record Box[T] {
  value: T
}

pub enum Status[T] {
  Ready(T)
  Waiting
}

pub fn wrap(value: Int): Box[Int] {
  Box {
    value: value
  }
}

pub fn current(value: Int): Status[Int] {
  Status::Ready(value)
}

fn main(): Int {
  wrap(1).value
}
"#,
    );
    let old_artifacts = workspace.join("old-artifacts");
    muga::write_package_artifacts(&entry, &old_artifacts).expect("old artifacts should build");

    write_package_file(
        &project,
        "src/main/main.muga",
        r#"
pub record Box[T] {
  value: String
}

pub enum Status[T] {
  Ready(String)
  Waiting
}

pub fn wrap(value: Int): Box[Int] {
  Box {
    value: value.to_string()
  }
}

pub fn current(value: Int): Status[Int] {
  Status::Ready(value.to_string())
}

fn main(): String {
  wrap(1).value
}
"#,
    );
    let new_artifacts = workspace.join("new-artifacts");
    muga::write_package_artifacts(&entry, &new_artifacts).expect("new artifacts should build");

    let text = muga_command()
        .arg("api-diff")
        .arg("--old-artifact-root")
        .arg(&old_artifacts)
        .arg("--new-artifact-root")
        .arg(&new_artifacts)
        .arg("--package")
        .arg("api_app::main")
        .output()
        .expect("muga api-diff text should run");
    assert!(text.status.success(), "{text:?}");
    let stdout = String::from_utf8(text.stdout).expect("api-diff text should be UTF-8");
    assert!(stdout.contains("status\tbreaking"), "{stdout}");
    assert!(stdout.contains("record-field-type-changed"), "{stdout}");
    assert!(stdout.contains("enum-variant-payload-changed"), "{stdout}");
    assert!(stdout.contains("api_app::main::Box.value"), "{stdout}");
    assert!(stdout.contains("api_app::main::Status.Ready"), "{stdout}");
}

fn graph(functions: Vec<PackageInterfaceFunction>) -> PackageInterfaceGraph {
    PackageInterfaceGraph {
        packages: vec![PackageInterface {
            package: PackageId::new(1),
            path: "app::api".to_string(),
            dependencies: Vec::new(),
            records: Vec::new(),
            enums: Vec::new(),
            opaque_types: Vec::new(),
            functions,
        }],
    }
}

fn graph_with_opaque(handle_facts: OpaqueHandleFacts) -> PackageInterfaceGraph {
    PackageInterfaceGraph {
        packages: vec![PackageInterface {
            package: PackageId::new(1),
            path: "app::api".to_string(),
            dependencies: Vec::new(),
            records: Vec::new(),
            enums: Vec::new(),
            opaque_types: vec![PackageInterfaceOpaqueType {
                item: PackageItemId::new(1),
                name: "File".to_string(),
                doc_comments: Vec::new(),
                handle_facts,
                span: Span::default(),
            }],
            functions: Vec::new(),
        }],
    }
}

fn record(id: u32, name: &str, fields: Vec<PackageInterfaceField>) -> PackageInterfaceRecord {
    PackageInterfaceRecord {
        item: PackageItemId::new(id),
        name: name.to_string(),
        doc_comments: Vec::new(),
        type_params: Vec::new(),
        json_deny_unknown_fields: false,
        cli_about: None,
        fields,
        span: Span::default(),
    }
}

fn field(name: &str, ty: TypeInfo) -> PackageInterfaceField {
    PackageInterfaceField {
        name: name.to_string(),
        json_rename: None,
        json_aliases: Vec::new(),
        json_validation: Vec::new(),
        cli_name: None,
        cli_short: None,
        cli_position: None,
        cli_value_source: None,
        cli_aliases: Vec::new(),
        cli_help: None,
        cli_hidden: false,
        cli_subcommand: false,
        ty,
        span: Span::default(),
    }
}

fn enumeration(
    id: u32,
    name: &str,
    variants: Vec<PackageInterfaceEnumVariant>,
) -> PackageInterfaceEnum {
    PackageInterfaceEnum {
        item: PackageItemId::new(id),
        name: name.to_string(),
        doc_comments: Vec::new(),
        type_params: Vec::new(),
        cli_about: None,
        variants,
        span: Span::default(),
    }
}

fn variant(name: &str, payload: Option<TypeInfo>) -> PackageInterfaceEnumVariant {
    PackageInterfaceEnumVariant {
        name: name.to_string(),
        json_rename: None,
        json_aliases: Vec::new(),
        cli_name: None,
        cli_aliases: Vec::new(),
        cli_about: None,
        cli_hidden: false,
        payload,
        span: Span::default(),
    }
}

fn function(
    id: u32,
    name: &str,
    type_params: Vec<String>,
    params: Vec<PackageInterfaceParam>,
    ret: TypeInfo,
) -> PackageInterfaceFunction {
    PackageInterfaceFunction {
        item: PackageItemId::new(id),
        name: name.to_string(),
        doc_comments: Vec::new(),
        type_params,
        params,
        ret,
        span: Span::default(),
    }
}

fn param(name: &str, ty: TypeInfo, mode: PackageInterfaceParamMode) -> PackageInterfaceParam {
    PackageInterfaceParam {
        name: name.to_string(),
        ty,
        mode,
        span: Span::default(),
    }
}

fn change_kinds(diff: &muga::api_diff::PackageApiDiff) -> BTreeSet<&str> {
    diff.changes
        .iter()
        .map(|change| change.kind.as_str())
        .collect()
}

fn temp_api_diff_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("muga-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("temp api-diff root should be created");
    root
}

fn write_package_file(root: &Path, relative: &str, source: &str) -> PathBuf {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().expect("package file should have parent"))
        .expect("package directory should be created");
    fs::write(&path, source.trim_start()).expect("package file should be written");
    path
}

fn write_api_diff_manifest(project: &Path) {
    write_package_file(
        project,
        "muga.toml",
        r#"
[package]
name = "api_app"
language_revision = 1
source = "src"
"#,
    );
}

fn muga_command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_muga"))
}
