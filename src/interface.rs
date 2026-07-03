use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    ast::Visibility,
    cli_schema::CliValueSource,
    diagnostic::{
        Diagnostic, DiagnosticContext, artifact_file_context, artifact_hash_context,
        regeneration_command_context,
    },
    identity::{PackageId, PackageItemId},
    json_decode::JsonDecodeValidationRule,
    package::{PackageItemInfo, PackageItemKind, PackageSymbolGraph},
    prelude, span,
    span::Span,
    symbol::SymbolTable,
    typed_hir::{
        Block, EnumStmt, Expr, ExprKind, FunctionStmt, IdentTarget, OpaqueTypeStmt, Program,
        RecordStmt, Stmt, ValueBlock,
    },
    types::{FunctionTypeInfo, TypeInfo},
};

const PERSISTED_INTERFACE_HEADER: &str = "muga-package-interface-v11";
const LEGACY_PERSISTED_INTERFACE_HEADERS: &[&str] = &[
    "muga-package-interface-v10",
    "muga-package-interface-v9",
    "muga-package-interface-v8",
    "muga-package-interface-v7",
    "muga-package-interface-v6",
    "muga-package-interface-v5",
    "muga-package-interface-v4",
    "muga-package-interface-v3",
    "muga-package-interface-v2",
    "muga-package-interface-v1",
];
const PERSISTED_INTERFACE_HASH_SPAN: &str = "0:0-0:0";
const REGENERATE_INTERFACE_COMMANDS: [(&str, &str); 3] = [
    ("default-build", "muga build <entry>"),
    (
        "artifact-root",
        "muga emit-artifacts --artifact-root <dir> <entry>",
    ),
    (
        "interface",
        "muga emit-interface --artifact-root <dir> <entry>",
    ),
];
const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageExportGraph {
    pub packages: Vec<PackageExports>,
}

impl PackageExportGraph {
    pub fn from_symbol_graph(graph: &PackageSymbolGraph) -> Self {
        let packages = graph
            .packages
            .iter()
            .map(|package| {
                let mut records = Vec::new();
                let mut enums = Vec::new();
                let mut opaque_types = Vec::new();
                let mut functions = Vec::new();
                for item in graph.items.iter().filter(|item| {
                    item.package == package.id && item.visibility == Visibility::Public
                }) {
                    let export = PackageExportItem {
                        item: item.id,
                        name: item.name.clone(),
                        mangled_name: item.mangled_name.clone(),
                        span: item.span,
                    };
                    match item.kind {
                        PackageItemKind::Record => records.push(export),
                        PackageItemKind::Enum => enums.push(export),
                        PackageItemKind::OpaqueType => opaque_types.push(export),
                        PackageItemKind::Function => functions.push(export),
                    }
                }
                PackageExports {
                    package: package.id,
                    path: package.path.clone(),
                    records,
                    enums,
                    opaque_types,
                    functions,
                }
            })
            .collect();

        Self { packages }
    }

    pub fn from_interfaces(interfaces: &PackageInterfaceGraph, graph: &PackageSymbolGraph) -> Self {
        let packages = interfaces
            .packages
            .iter()
            .map(|interface| {
                let records = interface
                    .records
                    .iter()
                    .filter_map(|record| {
                        export_item_from_interface(
                            graph,
                            record.item,
                            &record.name,
                            record.span,
                            PackageItemKind::Record,
                        )
                    })
                    .collect();
                let enums = interface
                    .enums
                    .iter()
                    .filter_map(|enumeration| {
                        export_item_from_interface(
                            graph,
                            enumeration.item,
                            &enumeration.name,
                            enumeration.span,
                            PackageItemKind::Enum,
                        )
                    })
                    .collect();
                let functions = interface
                    .functions
                    .iter()
                    .filter_map(|function| {
                        export_item_from_interface(
                            graph,
                            function.item,
                            &function.name,
                            function.span,
                            PackageItemKind::Function,
                        )
                    })
                    .collect();
                let opaque_types = interface
                    .opaque_types
                    .iter()
                    .filter_map(|opaque| {
                        export_item_from_interface(
                            graph,
                            opaque.item,
                            &opaque.name,
                            opaque.span,
                            PackageItemKind::OpaqueType,
                        )
                    })
                    .collect();

                PackageExports {
                    package: interface.package,
                    path: interface.path.clone(),
                    records,
                    enums,
                    opaque_types,
                    functions,
                }
            })
            .collect();

        Self { packages }
    }

    pub fn package(&self, id: PackageId) -> Option<&PackageExports> {
        self.packages.iter().find(|package| package.package == id)
    }

    pub fn record_by_name(&self, package: PackageId, name: &str) -> Option<&PackageExportItem> {
        self.package(package)?
            .records
            .iter()
            .find(|record| record.name == name)
    }

    pub fn enum_by_name(&self, package: PackageId, name: &str) -> Option<&PackageExportItem> {
        self.package(package)?
            .enums
            .iter()
            .find(|enumeration| enumeration.name == name)
    }

    pub fn opaque_type_by_name(
        &self,
        package: PackageId,
        name: &str,
    ) -> Option<&PackageExportItem> {
        self.package(package)?
            .opaque_types
            .iter()
            .find(|opaque| opaque.name == name)
    }

    pub fn function_by_name(&self, package: PackageId, name: &str) -> Option<&PackageExportItem> {
        self.package(package)?
            .functions
            .iter()
            .find(|function| function.name == name)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageExports {
    pub package: PackageId,
    pub path: String,
    pub records: Vec<PackageExportItem>,
    pub enums: Vec<PackageExportItem>,
    pub opaque_types: Vec<PackageExportItem>,
    pub functions: Vec<PackageExportItem>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageExportItem {
    pub item: PackageItemId,
    pub name: String,
    pub mangled_name: String,
    pub span: Span,
}

fn export_item_from_interface(
    graph: &PackageSymbolGraph,
    item: PackageItemId,
    name: &str,
    span: Span,
    kind: PackageItemKind,
) -> Option<PackageExportItem> {
    let info = graph.item(item)?;
    if info.kind != kind || info.visibility != Visibility::Public {
        return None;
    }
    Some(PackageExportItem {
        item,
        name: name.to_string(),
        mangled_name: info.mangled_name.clone(),
        span,
    })
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PackageInterfaceGraph {
    pub packages: Vec<PackageInterface>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PersistedInterfaceBodyShape {
    Text,
    Hash,
}

impl PackageInterfaceGraph {
    pub fn to_persisted_text(&self, symbols: &SymbolTable) -> String {
        let body = self.persisted_body_text(symbols);
        let hash_body = self.persisted_hash_body_text(symbols);
        format!(
            "{PERSISTED_INTERFACE_HEADER}\nhash\t{}\n{body}",
            stable_hash_hex(&hash_body)
        )
    }

    pub fn stable_hash(&self, symbols: &SymbolTable) -> String {
        stable_hash_hex(&self.persisted_hash_body_text(symbols))
    }

    pub fn stable_hash_for_package(
        &self,
        package_path: &str,
        symbols: &SymbolTable,
    ) -> Option<String> {
        let package = self.package_by_path(package_path)?.clone();
        let context = PersistedInterfaceIdentityContext::from_graph(self);
        Some(stable_hash_hex(
            &Self {
                packages: vec![package],
            }
            .persisted_hash_body_text_with_context(symbols, &context),
        ))
    }

    fn persisted_body_text(&self, symbols: &SymbolTable) -> String {
        let context = PersistedInterfaceIdentityContext::from_graph(self);
        self.persisted_body_text_with_context(symbols, &context, PersistedInterfaceBodyShape::Text)
    }

    fn persisted_hash_body_text(&self, symbols: &SymbolTable) -> String {
        let context = PersistedInterfaceIdentityContext::from_graph(self);
        self.persisted_hash_body_text_with_context(symbols, &context)
    }

    fn persisted_hash_body_text_with_context(
        &self,
        symbols: &SymbolTable,
        context: &PersistedInterfaceIdentityContext,
    ) -> String {
        self.persisted_body_text_with_context(symbols, context, PersistedInterfaceBodyShape::Hash)
    }

    fn persisted_body_text_with_context(
        &self,
        symbols: &SymbolTable,
        context: &PersistedInterfaceIdentityContext,
        shape: PersistedInterfaceBodyShape,
    ) -> String {
        let mut out = String::new();
        for package in &self.packages {
            push_line(
                &mut out,
                &[
                    "package".to_string(),
                    stable_artifact_package_id(&package.path)
                        .as_u32()
                        .to_string(),
                    package.path.clone(),
                    package.dependencies.len().to_string(),
                    package.records.len().to_string(),
                    package.enums.len().to_string(),
                    package.opaque_types.len().to_string(),
                    package.functions.len().to_string(),
                ],
            );
            for dependency in &package.dependencies {
                push_line(&mut out, &["dependency".to_string(), dependency.clone()]);
            }
            for record in &package.records {
                let mut parts = vec![
                    "record".to_string(),
                    context
                        .item_id(PackageItemKind::Record, record.item)
                        .unwrap_or(record.item)
                        .as_u32()
                        .to_string(),
                    record.name.clone(),
                    format_interface_span(record.span, shape),
                    record.type_params.len().to_string(),
                ];
                parts.extend(record.type_params.iter().cloned());
                parts.push(record.fields.len().to_string());
                parts.push(record_json_flags(record).to_string());
                if let Some(about) = &record.cli_about {
                    parts.push("cli".to_string());
                    parts.push("1".to_string());
                    parts.push(about.clone());
                }
                push_line(&mut out, &parts);
                push_doc_comment_lines(&mut out, &record.doc_comments, shape);
                for field in &record.fields {
                    let mut parts = vec![
                        "field".to_string(),
                        field.name.clone(),
                        format_interface_span(field.span, shape),
                        format_type_info(&field.ty, symbols, context),
                    ];
                    let has_cli_metadata = field.cli_name.is_some()
                        || field.cli_short.is_some()
                        || field.cli_position.is_some()
                        || field.cli_value_source.is_some()
                        || !field.cli_aliases.is_empty()
                        || field.cli_help.is_some()
                        || field.cli_hidden
                        || field.cli_subcommand;
                    if field.json_aliases.is_empty()
                        && field.json_validation.is_empty()
                        && !has_cli_metadata
                    {
                        if let Some(rename) = &field.json_rename {
                            parts.push(rename.clone());
                        }
                    } else {
                        parts.push(field.json_rename.clone().unwrap_or_else(|| "-".to_string()));
                        parts.push(field.json_aliases.len().to_string());
                        parts.extend(field.json_aliases.iter().cloned());
                        parts.push(field.json_validation.len().to_string());
                        parts.extend(
                            field
                                .json_validation
                                .iter()
                                .map(JsonDecodeValidationRule::artifact_token),
                        );
                        if has_cli_metadata {
                            parts.push("cli".to_string());
                            parts.push(field.cli_name.clone().unwrap_or_else(|| "-".to_string()));
                            parts.push(field.cli_aliases.len().to_string());
                            parts.extend(field.cli_aliases.iter().cloned());
                            let cli_flags = u32::from(field.cli_hidden)
                                | (u32::from(field.cli_subcommand) << 1);
                            parts.push(cli_flags.to_string());
                            match &field.cli_help {
                                Some(help) => {
                                    parts.push("1".to_string());
                                    parts.push(help.clone());
                                }
                                None => parts.push("0".to_string()),
                            }
                            if let Some(short) = &field.cli_short {
                                parts.push("short".to_string());
                                parts.push(short.clone());
                            }
                            if let Some(position) = field.cli_position {
                                parts.push("position".to_string());
                                parts.push(position.to_string());
                            }
                            if let Some(value_source) = field.cli_value_source {
                                parts.push("value_source".to_string());
                                parts.push(value_source.artifact_token().to_string());
                            }
                        }
                    }
                    push_line(&mut out, &parts);
                }
            }
            for enumeration in &package.enums {
                let mut parts = vec![
                    "enum".to_string(),
                    context
                        .item_id(PackageItemKind::Enum, enumeration.item)
                        .unwrap_or(enumeration.item)
                        .as_u32()
                        .to_string(),
                    enumeration.name.clone(),
                    format_interface_span(enumeration.span, shape),
                    enumeration.type_params.len().to_string(),
                ];
                parts.extend(enumeration.type_params.iter().cloned());
                parts.push(enumeration.variants.len().to_string());
                if let Some(about) = &enumeration.cli_about {
                    parts.push("cli".to_string());
                    parts.push("1".to_string());
                    parts.push(about.clone());
                }
                push_line(&mut out, &parts);
                push_doc_comment_lines(&mut out, &enumeration.doc_comments, shape);
                for variant in &enumeration.variants {
                    let has_cli_metadata = variant.cli_name.is_some()
                        || !variant.cli_aliases.is_empty()
                        || variant.cli_about.is_some()
                        || variant.cli_hidden;
                    let mut parts = vec![
                        "variant".to_string(),
                        variant.name.clone(),
                        format_interface_span(variant.span, shape),
                        match &variant.payload {
                            Some(payload) => format_type_info(payload, symbols, context),
                            None => "-".to_string(),
                        },
                    ];
                    if variant.json_aliases.is_empty() && !has_cli_metadata {
                        if let Some(rename) = &variant.json_rename {
                            parts.push(rename.clone());
                        }
                    } else {
                        parts.push(
                            variant
                                .json_rename
                                .clone()
                                .unwrap_or_else(|| "-".to_string()),
                        );
                        parts.push(variant.json_aliases.len().to_string());
                        parts.extend(variant.json_aliases.iter().cloned());
                        if has_cli_metadata {
                            parts.push("cli".to_string());
                            parts.push(variant.cli_name.clone().unwrap_or_else(|| "-".to_string()));
                            parts.push(variant.cli_aliases.len().to_string());
                            parts.extend(variant.cli_aliases.iter().cloned());
                            parts.push(if variant.cli_hidden { "1" } else { "0" }.to_string());
                            match &variant.cli_about {
                                Some(about) => {
                                    parts.push("1".to_string());
                                    parts.push(about.clone());
                                }
                                None => parts.push("0".to_string()),
                            }
                        }
                    }
                    push_line(&mut out, &parts);
                }
            }
            for opaque in &package.opaque_types {
                push_line(
                    &mut out,
                    &[
                        "opaque-type".to_string(),
                        context
                            .item_id(PackageItemKind::OpaqueType, opaque.item)
                            .unwrap_or(opaque.item)
                            .as_u32()
                            .to_string(),
                        opaque.name.clone(),
                        format_interface_span(opaque.span, shape),
                        format_opaque_handle_facts(&opaque.handle_facts, context),
                    ],
                );
                push_doc_comment_lines(&mut out, &opaque.doc_comments, shape);
            }
            for function in &package.functions {
                let mut parts = vec![
                    "function".to_string(),
                    context
                        .item_id(PackageItemKind::Function, function.item)
                        .unwrap_or(function.item)
                        .as_u32()
                        .to_string(),
                    function.name.clone(),
                    format_interface_span(function.span, shape),
                    function.type_params.len().to_string(),
                ];
                parts.extend(function.type_params.iter().cloned());
                parts.push(function.params.len().to_string());
                parts.push(format_type_info(&function.ret, symbols, context));
                push_line(&mut out, &parts);
                push_doc_comment_lines(&mut out, &function.doc_comments, shape);
                for param in &function.params {
                    push_line(
                        &mut out,
                        &[
                            "param".to_string(),
                            param.name.clone(),
                            format_interface_span(param.span, shape),
                            param.mode.as_str().to_string(),
                            format_type_info(&param.ty, symbols, context),
                        ],
                    );
                }
            }
        }
        out
    }

    pub fn from_persisted_text(
        text: &str,
        symbols: &mut SymbolTable,
    ) -> Result<Self, Vec<Diagnostic>> {
        let graph = Self::parse_persisted_text(text, symbols)?;
        let graph = remap_persisted_artifact_ids(graph.packages, symbols)?;
        validate_persisted_interface_graph(graph)
    }

    fn parse_persisted_text(
        text: &str,
        symbols: &mut SymbolTable,
    ) -> Result<Self, Vec<Diagnostic>> {
        PersistedInterfaceParser::new(text, symbols).parse()
    }

    pub fn write_persisted_file(
        &self,
        path: &Path,
        symbols: &SymbolTable,
    ) -> Result<(), Diagnostic> {
        fs::write(path, self.to_persisted_text(symbols)).map_err(|error| {
            Diagnostic::new(
                "PK018",
                format!(
                    "failed to write package interface `{}`: {error}",
                    path.display()
                ),
                Span::default(),
            )
        })
    }

    pub fn package_graph_by_path(&self, package_path: &str) -> Option<Self> {
        let package = self.package_by_path(package_path)?.clone();
        Some(Self {
            packages: vec![package],
        })
    }

    pub fn write_persisted_artifact(
        &self,
        root: &Path,
        package_path: &str,
        symbols: &SymbolTable,
    ) -> Result<PathBuf, Diagnostic> {
        let text = self.persisted_artifact_text(package_path, symbols)?;
        let path = Self::persisted_file_path(root, package_path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                Diagnostic::new(
                    "PK018",
                    format!(
                        "failed to create package interface artifact directory {}: {error}",
                        parent.display()
                    ),
                    Span::default(),
                )
            })?;
        }
        fs::write(&path, text).map_err(|error| {
            Diagnostic::new(
                "PK018",
                format!(
                    "failed to write package interface `{}`: {error}",
                    path.display()
                ),
                Span::default(),
            )
        })?;
        Ok(path)
    }

    pub fn persisted_artifact_text(
        &self,
        package_path: &str,
        symbols: &SymbolTable,
    ) -> Result<String, Diagnostic> {
        let Some(graph) = self.package_graph_by_path(package_path) else {
            return Err(Diagnostic::new(
                "PK016",
                format!("compiled package interfaces do not contain `{package_path}`"),
                Span::default(),
            )
            .with_suggestion("choose a package that is reachable from the entrypoint"));
        };
        let context = PersistedInterfaceIdentityContext::from_graph(self);
        let body = graph.persisted_body_text_with_context(
            symbols,
            &context,
            PersistedInterfaceBodyShape::Text,
        );
        let hash_body = graph.persisted_hash_body_text_with_context(symbols, &context);
        Ok(format!(
            "{PERSISTED_INTERFACE_HEADER}\nhash\t{}\n{body}",
            stable_hash_hex(&hash_body)
        ))
    }

    pub fn read_persisted_file(
        path: &Path,
        symbols: &mut SymbolTable,
    ) -> Result<Self, Vec<Diagnostic>> {
        let text = fs::read_to_string(path).map_err(|error| {
            vec![
                with_interface_regeneration_context(Diagnostic::new(
                    "PK018",
                    format!(
                        "failed to read package interface `{}`: {error}",
                        path.display()
                    ),
                    Span::default(),
                ))
                .with_context(interface_artifact_file_context(path, "interface")),
            ]
        })?;
        Self::parse_persisted_text(&text, symbols).map_err(|mut diagnostics| {
            add_interface_artifact_file_context(&mut diagnostics, path, "interface");
            diagnostics
        })
    }

    pub fn read_persisted_artifacts(
        root: &Path,
        package_paths: &[String],
        symbols: &mut SymbolTable,
    ) -> Result<Self, Vec<Diagnostic>> {
        let mut packages = Vec::new();
        let mut seen_paths = HashSet::new();
        let mut queued_paths = HashSet::new();
        let mut queue = VecDeque::new();
        let mut diagnostics = Vec::new();

        for package_path in package_paths {
            if queued_paths.insert(package_path.clone()) {
                queue.push_back(package_path.clone());
            }
        }

        while let Some(package_path) = queue.pop_front() {
            let artifact_path = Self::persisted_file_path(root, &package_path);
            if !artifact_path.is_file() {
                diagnostics.push(
                    with_interface_regeneration_context(Diagnostic::new(
                        "PK016",
                        format!(
                            "missing package interface artifact `{}` for `{package_path}`",
                            artifact_path.display()
                        ),
                        Span::default(),
                    ))
                    .with_context(interface_artifact_file_context(
                        &artifact_path,
                        "dependency-interface",
                    ))
                    .with_suggestion(regenerate_interface_artifact_suggestion()),
                );
                continue;
            }

            let graph = match Self::read_persisted_file(&artifact_path, symbols) {
                Ok(graph) => graph,
                Err(mut errors) => {
                    add_interface_artifact_context(&mut errors, &artifact_path, &package_path);
                    diagnostics.append(&mut errors);
                    continue;
                }
            };
            if graph.package_by_path(&package_path).is_none() {
                diagnostics.push(
                    with_interface_regeneration_context(Diagnostic::new(
                        "PK016",
                        format!(
                            "package interface artifact `{}` does not contain `{package_path}`",
                            artifact_path.display()
                        ),
                        Span::default(),
                    ))
                    .with_context(interface_artifact_file_context(
                        &artifact_path,
                        "dependency-interface",
                    ))
                    .with_suggestion(regenerate_interface_artifact_suggestion()),
                );
            }
            for package in graph.packages {
                for dependency in &package.dependencies {
                    if queued_paths.insert(dependency.clone()) {
                        queue.push_back(dependency.clone());
                    }
                }
                if seen_paths.insert(package.path.clone()) {
                    packages.push(package);
                }
            }
        }

        if diagnostics.is_empty() {
            let graph = match remap_persisted_artifact_ids(packages, symbols) {
                Ok(graph) => graph,
                Err(mut errors) => {
                    add_interface_artifact_root_context(&mut errors, root);
                    return Err(errors);
                }
            };
            validate_persisted_interface_graph(graph).map_err(|mut errors| {
                add_interface_artifact_root_context(&mut errors, root);
                errors
            })
        } else {
            Err(diagnostics)
        }
    }

    pub fn persisted_file_path(root: &Path, package_path: &str) -> PathBuf {
        root.join(format!("{}.mgi", package_path.replace("::", "__")))
    }

    pub fn reintern_symbols(&self, from: &SymbolTable, to: &mut SymbolTable) -> Self {
        Self {
            packages: self
                .packages
                .iter()
                .map(|package| PackageInterface {
                    package: package.package,
                    path: package.path.clone(),
                    dependencies: package.dependencies.clone(),
                    records: package
                        .records
                        .iter()
                        .map(|record| PackageInterfaceRecord {
                            item: record.item,
                            name: record.name.clone(),
                            doc_comments: record.doc_comments.clone(),
                            type_params: record.type_params.clone(),
                            json_deny_unknown_fields: record.json_deny_unknown_fields,
                            cli_about: record.cli_about.clone(),
                            fields: record
                                .fields
                                .iter()
                                .map(|field| PackageInterfaceField {
                                    name: field.name.clone(),
                                    json_rename: field.json_rename.clone(),
                                    json_aliases: field.json_aliases.clone(),
                                    json_validation: field.json_validation.clone(),
                                    cli_name: field.cli_name.clone(),
                                    cli_short: field.cli_short.clone(),
                                    cli_position: field.cli_position,
                                    cli_value_source: field.cli_value_source,
                                    cli_aliases: field.cli_aliases.clone(),
                                    cli_help: field.cli_help.clone(),
                                    cli_hidden: field.cli_hidden,
                                    cli_subcommand: field.cli_subcommand,
                                    ty: reintern_type_info(&field.ty, from, to),
                                    span: field.span,
                                })
                                .collect(),
                            span: record.span,
                        })
                        .collect(),
                    enums: package
                        .enums
                        .iter()
                        .map(|enumeration| PackageInterfaceEnum {
                            item: enumeration.item,
                            name: enumeration.name.clone(),
                            doc_comments: enumeration.doc_comments.clone(),
                            type_params: enumeration.type_params.clone(),
                            cli_about: enumeration.cli_about.clone(),
                            variants: enumeration
                                .variants
                                .iter()
                                .map(|variant| PackageInterfaceEnumVariant {
                                    name: variant.name.clone(),
                                    json_rename: variant.json_rename.clone(),
                                    json_aliases: variant.json_aliases.clone(),
                                    cli_name: variant.cli_name.clone(),
                                    cli_aliases: variant.cli_aliases.clone(),
                                    cli_about: variant.cli_about.clone(),
                                    cli_hidden: variant.cli_hidden,
                                    payload: variant
                                        .payload
                                        .as_ref()
                                        .map(|payload| reintern_type_info(payload, from, to)),
                                    span: variant.span,
                                })
                                .collect(),
                            span: enumeration.span,
                        })
                        .collect(),
                    opaque_types: package
                        .opaque_types
                        .iter()
                        .map(|opaque| PackageInterfaceOpaqueType {
                            item: opaque.item,
                            name: opaque.name.clone(),
                            doc_comments: opaque.doc_comments.clone(),
                            handle_facts: opaque.handle_facts.clone(),
                            span: opaque.span,
                        })
                        .collect(),
                    functions: package
                        .functions
                        .iter()
                        .map(|function| PackageInterfaceFunction {
                            item: function.item,
                            name: function.name.clone(),
                            doc_comments: function.doc_comments.clone(),
                            type_params: function.type_params.clone(),
                            params: function
                                .params
                                .iter()
                                .map(|param| PackageInterfaceParam {
                                    name: param.name.clone(),
                                    ty: reintern_type_info(&param.ty, from, to),
                                    mode: param.mode,
                                    span: param.span,
                                })
                                .collect(),
                            ret: reintern_type_info(&function.ret, from, to),
                            span: function.span,
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    pub fn package(&self, id: PackageId) -> Option<&PackageInterface> {
        self.packages
            .iter()
            .find(|interface| interface.package == id)
    }

    pub fn package_by_path(&self, path: &str) -> Option<&PackageInterface> {
        self.packages
            .iter()
            .find(|interface| interface.path == path)
    }

    pub fn record(&self, item: PackageItemId) -> Option<&PackageInterfaceRecord> {
        self.packages
            .iter()
            .flat_map(|package| package.records.iter())
            .find(|record| record.item == item)
    }

    pub fn record_by_name(
        &self,
        package: PackageId,
        name: &str,
    ) -> Option<&PackageInterfaceRecord> {
        self.package(package)?
            .records
            .iter()
            .find(|record| record.name == name)
    }

    pub fn enum_by_name(&self, package: PackageId, name: &str) -> Option<&PackageInterfaceEnum> {
        self.package(package)?
            .enums
            .iter()
            .find(|enumeration| enumeration.name == name)
    }

    pub fn opaque_type(&self, item: PackageItemId) -> Option<&PackageInterfaceOpaqueType> {
        self.packages
            .iter()
            .flat_map(|package| package.opaque_types.iter())
            .find(|opaque| opaque.item == item)
    }

    pub fn opaque_type_by_name(
        &self,
        package: PackageId,
        name: &str,
    ) -> Option<&PackageInterfaceOpaqueType> {
        self.package(package)?
            .opaque_types
            .iter()
            .find(|opaque| opaque.name == name)
    }

    pub fn function(&self, item: PackageItemId) -> Option<&PackageInterfaceFunction> {
        self.packages
            .iter()
            .flat_map(|package| package.functions.iter())
            .find(|function| function.item == item)
    }

    pub fn function_by_name(
        &self,
        package: PackageId,
        name: &str,
    ) -> Option<&PackageInterfaceFunction> {
        self.package(package)?
            .functions
            .iter()
            .find(|function| function.name == name)
    }
}

fn add_interface_artifact_context(
    diagnostics: &mut [Diagnostic],
    artifact_path: &Path,
    package_path: &str,
) {
    for diagnostic in diagnostics {
        diagnostic.add_context(interface_artifact_file_context(
            artifact_path,
            "dependency-interface",
        ));
        add_interface_regeneration_context(diagnostic);
        let message = std::mem::take(&mut diagnostic.message);
        diagnostic.message = format!(
            "package interface artifact `{}` for `{package_path}` failed to load: {message}",
            artifact_path.display()
        );
    }
}

fn interface_artifact_file_context(
    artifact_path: &Path,
    role: &str,
) -> crate::diagnostic::DiagnosticContext {
    artifact_file_context(role, "interface", artifact_path)
}

fn add_interface_artifact_file_context(
    diagnostics: &mut [Diagnostic],
    artifact_path: &Path,
    role: &str,
) {
    for diagnostic in diagnostics {
        diagnostic.add_context(interface_artifact_file_context(artifact_path, role));
        add_interface_regeneration_context(diagnostic);
    }
}

fn add_interface_artifact_root_context(diagnostics: &mut [Diagnostic], root: &Path) {
    for diagnostic in diagnostics {
        let message = std::mem::take(&mut diagnostic.message);
        diagnostic.message = format!(
            "package interface artifacts under `{}` failed to load: {message}",
            root.display()
        );
    }
}

fn regenerate_interface_artifact_suggestion() -> &'static str {
    "regenerate package interfaces with `muga build`, `muga emit-artifacts`, or `muga emit-interface`"
}

fn with_interface_regeneration_context(mut diagnostic: Diagnostic) -> Diagnostic {
    add_interface_regeneration_context(&mut diagnostic);
    diagnostic
}

fn add_interface_regeneration_context(diagnostic: &mut Diagnostic) {
    for (role, command) in REGENERATE_INTERFACE_COMMANDS {
        let already_present = diagnostic.context.iter().any(|context| {
            matches!(
                context,
                DiagnosticContext::RegenerationCommand {
                    command: existing,
                    ..
                } if existing == command
            )
        });
        if !already_present {
            diagnostic.add_context(regeneration_command_context(role, command));
        }
    }
}

#[derive(Clone, Debug)]
struct PersistedItemIdentity {
    stable_item: PackageItemId,
}

#[derive(Clone, Debug, Default)]
struct PersistedInterfaceIdentityContext {
    items: HashMap<(PackageItemKind, PackageItemId), PersistedItemIdentity>,
}

impl PersistedInterfaceIdentityContext {
    fn from_graph(graph: &PackageInterfaceGraph) -> Self {
        let mut items = HashMap::new();
        for package in &graph.packages {
            for record in &package.records {
                items.insert(
                    (PackageItemKind::Record, record.item),
                    PersistedItemIdentity {
                        stable_item: stable_artifact_item_id(
                            &package.path,
                            PackageItemKind::Record,
                            &record.name,
                        ),
                    },
                );
            }
            for enumeration in &package.enums {
                items.insert(
                    (PackageItemKind::Enum, enumeration.item),
                    PersistedItemIdentity {
                        stable_item: stable_artifact_item_id(
                            &package.path,
                            PackageItemKind::Enum,
                            &enumeration.name,
                        ),
                    },
                );
            }
            for opaque in &package.opaque_types {
                items.insert(
                    (PackageItemKind::OpaqueType, opaque.item),
                    PersistedItemIdentity {
                        stable_item: stable_artifact_item_id(
                            &package.path,
                            PackageItemKind::OpaqueType,
                            &opaque.name,
                        ),
                    },
                );
            }
            for function in &package.functions {
                items.insert(
                    (PackageItemKind::Function, function.item),
                    PersistedItemIdentity {
                        stable_item: stable_artifact_item_id(
                            &package.path,
                            PackageItemKind::Function,
                            &function.name,
                        ),
                    },
                );
            }
        }
        Self { items }
    }

    fn item_id(&self, kind: PackageItemKind, item: PackageItemId) -> Option<PackageItemId> {
        self.items
            .get(&(kind, item))
            .map(|identity| identity.stable_item)
    }
}

fn stable_artifact_package_id(package_path: &str) -> PackageId {
    PackageId::new(stable_artifact_key_u32(&["package", package_path]))
}

fn stable_artifact_item_id(package_path: &str, kind: PackageItemKind, name: &str) -> PackageItemId {
    PackageItemId::new(stable_artifact_key_u32(&[
        "item",
        package_path,
        package_item_kind_label(kind),
        name,
    ]))
}

fn stable_artifact_key_u32(parts: &[&str]) -> u32 {
    let mut hash = FNV_OFFSET_BASIS;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    (hash ^ (hash >> 32)) as u32
}

type ArtifactItemNameKey = (PackageItemKind, PackageItemId, String);
type ArtifactItemIdKey = (PackageItemKind, PackageItemId);
type ArtifactDeclKey = (String, PackageItemKind, PackageItemId, String);

#[derive(Clone, Debug)]
struct ArtifactItemCandidate {
    package_path: String,
    item: PackageItemId,
}

fn remap_persisted_artifact_ids(
    mut packages: Vec<PackageInterface>,
    symbols: &SymbolTable,
) -> Result<PackageInterfaceGraph, Vec<Diagnostic>> {
    packages.sort_by(|left, right| left.path.cmp(&right.path));

    let mut package_ids = HashMap::new();
    for (index, package) in packages.iter().enumerate() {
        package_ids.insert(package.path.clone(), PackageId::new(index as u32));
    }

    let mut registry = ArtifactItemRegistry::default();

    for package in &packages {
        for record in &package.records {
            registry.register(package, PackageItemKind::Record, record.item, &record.name);
        }
        for enumeration in &package.enums {
            registry.register(
                package,
                PackageItemKind::Enum,
                enumeration.item,
                &enumeration.name,
            );
        }
        for opaque in &package.opaque_types {
            registry.register(
                package,
                PackageItemKind::OpaqueType,
                opaque.item,
                &opaque.name,
            );
        }
        for function in &package.functions {
            registry.register(
                package,
                PackageItemKind::Function,
                function.item,
                &function.name,
            );
        }
    }

    let mut remapper = PersistedArtifactIdRemapper {
        symbols,
        decl_items: registry.decl_items,
        candidates_by_name: registry.candidates_by_name,
        candidates_by_id: registry.candidates_by_id,
        diagnostics: Vec::new(),
    };

    for package in &mut packages {
        let current_package = package.path.clone();
        let dependencies = package.dependencies.clone();
        if let Some(package_id) = package_ids.get(&package.path).copied() {
            package.package = package_id;
        }

        for record in &mut package.records {
            record.item = remapper.declaration_item(
                &current_package,
                PackageItemKind::Record,
                record.item,
                &record.name,
                record.span,
            );
            for field in &mut record.fields {
                remapper.type_info(&mut field.ty, &current_package, &dependencies, field.span);
            }
        }

        for enumeration in &mut package.enums {
            enumeration.item = remapper.declaration_item(
                &current_package,
                PackageItemKind::Enum,
                enumeration.item,
                &enumeration.name,
                enumeration.span,
            );
            for variant in &mut enumeration.variants {
                if let Some(payload) = &mut variant.payload {
                    remapper.type_info(payload, &current_package, &dependencies, variant.span);
                }
            }
        }

        for opaque in &mut package.opaque_types {
            opaque.item = remapper.declaration_item(
                &current_package,
                PackageItemKind::OpaqueType,
                opaque.item,
                &opaque.name,
                opaque.span,
            );
            if let Some(close_function) = opaque.handle_facts.close_function
                && let Some(remapped) = remapper.reference_item(
                    PackageItemKind::Function,
                    close_function,
                    "<close function>",
                    &current_package,
                    &dependencies,
                    opaque.span,
                )
            {
                opaque.handle_facts.close_function = Some(remapped);
            }
        }

        for function in &mut package.functions {
            function.item = remapper.declaration_item(
                &current_package,
                PackageItemKind::Function,
                function.item,
                &function.name,
                function.span,
            );
            for param in &mut function.params {
                remapper.type_info(&mut param.ty, &current_package, &dependencies, param.span);
            }
            remapper.type_info(
                &mut function.ret,
                &current_package,
                &dependencies,
                function.span,
            );
        }
    }

    if remapper.diagnostics.is_empty() {
        Ok(PackageInterfaceGraph { packages })
    } else {
        Err(remapper.diagnostics)
    }
}

#[derive(Default)]
struct ArtifactItemRegistry {
    next_item: u32,
    decl_items: HashMap<ArtifactDeclKey, PackageItemId>,
    candidates_by_name: HashMap<ArtifactItemNameKey, Vec<ArtifactItemCandidate>>,
    candidates_by_id: HashMap<ArtifactItemIdKey, Vec<ArtifactItemCandidate>>,
}

impl ArtifactItemRegistry {
    fn register(
        &mut self,
        package: &PackageInterface,
        kind: PackageItemKind,
        old_item: PackageItemId,
        name: &str,
    ) {
        let new_item = PackageItemId::new(self.next_item);
        self.next_item += 1;
        let candidate = ArtifactItemCandidate {
            package_path: package.path.clone(),
            item: new_item,
        };
        self.decl_items.insert(
            (package.path.clone(), kind, old_item, name.to_string()),
            new_item,
        );
        self.candidates_by_name
            .entry((kind, old_item, name.to_string()))
            .or_default()
            .push(candidate.clone());
        self.candidates_by_id
            .entry((kind, old_item))
            .or_default()
            .push(candidate);
    }
}

struct PersistedArtifactIdRemapper<'a> {
    symbols: &'a SymbolTable,
    decl_items: HashMap<ArtifactDeclKey, PackageItemId>,
    candidates_by_name: HashMap<ArtifactItemNameKey, Vec<ArtifactItemCandidate>>,
    candidates_by_id: HashMap<ArtifactItemIdKey, Vec<ArtifactItemCandidate>>,
    diagnostics: Vec<Diagnostic>,
}

impl PersistedArtifactIdRemapper<'_> {
    fn declaration_item(
        &mut self,
        package_path: &str,
        kind: PackageItemKind,
        old_item: PackageItemId,
        name: &str,
        span: Span,
    ) -> PackageItemId {
        self.decl_items
            .get(&(package_path.to_string(), kind, old_item, name.to_string()))
            .copied()
            .unwrap_or_else(|| {
                self.diagnostics.push(
                    Diagnostic::new(
                        "PK019",
                        format!(
                            "package interface could not remap {} `{name}` from `{package_path}`",
                            package_item_kind_label(kind)
                        ),
                        span,
                    )
                    .with_suggestion(regenerate_interface_artifact_suggestion()),
                );
                old_item
            })
    }

    fn type_info(
        &mut self,
        ty: &mut TypeInfo,
        current_package: &str,
        dependencies: &[String],
        span: Span,
    ) {
        match ty {
            TypeInfo::PackageRecord { symbol, item, args } => {
                for arg in args {
                    self.type_info(arg, current_package, dependencies, span);
                }
                let name = self.symbols.resolve(*symbol).to_string();
                if let Some(remapped) = self.reference_item(
                    PackageItemKind::Record,
                    *item,
                    &name,
                    current_package,
                    dependencies,
                    span,
                ) {
                    *item = remapped;
                }
            }
            TypeInfo::PackageEnum { symbol, item, args } => {
                let name = self.symbols.resolve(*symbol).to_string();
                if let Some(remapped) = self.reference_item(
                    PackageItemKind::Enum,
                    *item,
                    &name,
                    current_package,
                    dependencies,
                    span,
                ) {
                    *item = remapped;
                }
                for arg in args {
                    self.type_info(arg, current_package, dependencies, span);
                }
            }
            TypeInfo::PackageOpaque { symbol, item } => {
                let name = self.symbols.resolve(*symbol).to_string();
                if let Some(remapped) = self.reference_item(
                    PackageItemKind::OpaqueType,
                    *item,
                    &name,
                    current_package,
                    dependencies,
                    span,
                ) {
                    *item = remapped;
                }
            }
            TypeInfo::EnumConstructor {
                enum_symbol,
                enum_item: Some(item),
                ..
            } => {
                let name = self.symbols.resolve(*enum_symbol).to_string();
                if let Some(remapped) = self.reference_item(
                    PackageItemKind::Enum,
                    *item,
                    &name,
                    current_package,
                    dependencies,
                    span,
                ) {
                    *item = remapped;
                }
            }
            TypeInfo::Enum { args, .. } => {
                for arg in args {
                    self.type_info(arg, current_package, dependencies, span);
                }
            }
            TypeInfo::List(item) | TypeInfo::Option(item) | TypeInfo::Task(item) => {
                self.type_info(item, current_package, dependencies, span);
            }
            TypeInfo::Map(key, value) | TypeInfo::Result(key, value) => {
                self.type_info(key, current_package, dependencies, span);
                self.type_info(value, current_package, dependencies, span);
            }
            TypeInfo::Function(function) => {
                for param in &mut function.params {
                    self.type_info(param, current_package, dependencies, span);
                }
                self.type_info(&mut function.ret, current_package, dependencies, span);
            }
            TypeInfo::Record(_, args) => {
                for arg in args {
                    self.type_info(arg, current_package, dependencies, span);
                }
            }
            TypeInfo::GenericParam(_)
            | TypeInfo::EnumConstructor {
                enum_item: None, ..
            }
            | TypeInfo::Int
            | TypeInfo::Bool
            | TypeInfo::String
            | TypeInfo::Unit
            | TypeInfo::Builtin(_)
            | TypeInfo::Unknown
            | TypeInfo::Error => {}
        }
    }

    fn reference_item(
        &mut self,
        kind: PackageItemKind,
        old_item: PackageItemId,
        name: &str,
        current_package: &str,
        dependencies: &[String],
        span: Span,
    ) -> Option<PackageItemId> {
        let reference = ArtifactReference {
            kind,
            old_item,
            name,
            current_package,
            dependencies,
            span,
        };
        let exact = self
            .candidates_by_name
            .get(&(kind, old_item, name.to_string()))
            .cloned()
            .unwrap_or_default();
        match self.choose_candidate(&exact, &reference) {
            CandidateChoice::Resolved(item) => return Some(item),
            CandidateChoice::Ambiguous => return None,
            CandidateChoice::Missing => {}
        }

        let by_id = self
            .candidates_by_id
            .get(&(kind, old_item))
            .cloned()
            .unwrap_or_default();
        match self.choose_candidate(&by_id, &reference) {
            CandidateChoice::Resolved(item) => Some(item),
            CandidateChoice::Ambiguous => None,
            CandidateChoice::Missing => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "PK019",
                        format!(
                            "package interface references unknown {} `{name}` identity {:?}",
                            package_item_kind_label(reference.kind),
                            reference.old_item
                        ),
                        reference.span,
                    )
                    .with_suggestion(regenerate_interface_artifact_suggestion()),
                );
                None
            }
        }
    }

    fn choose_candidate(
        &mut self,
        candidates: &[ArtifactItemCandidate],
        reference: &ArtifactReference<'_>,
    ) -> CandidateChoice {
        if candidates.is_empty() {
            return CandidateChoice::Missing;
        }
        if candidates.len() == 1 {
            return CandidateChoice::Resolved(candidates[0].item);
        }

        let current = candidates
            .iter()
            .filter(|candidate| candidate.package_path == reference.current_package)
            .collect::<Vec<_>>();
        if current.len() == 1 {
            return CandidateChoice::Resolved(current[0].item);
        }

        let dependency = candidates
            .iter()
            .filter(|candidate| {
                reference
                    .dependencies
                    .iter()
                    .any(|dependency| dependency == &candidate.package_path)
            })
            .collect::<Vec<_>>();
        if dependency.len() == 1 {
            return CandidateChoice::Resolved(dependency[0].item);
        }

        let packages = candidates
            .iter()
            .map(|candidate| candidate.package_path.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        self.diagnostics.push(
            Diagnostic::new(
                "PK019",
                format!(
                    "package interface has ambiguous {} `{name}` identity {:?}: {packages}",
                    package_item_kind_label(reference.kind),
                    reference.old_item,
                    name = reference.name
                ),
                reference.span,
            )
            .with_suggestion(regenerate_interface_artifact_suggestion()),
        );
        CandidateChoice::Ambiguous
    }
}

struct ArtifactReference<'a> {
    kind: PackageItemKind,
    old_item: PackageItemId,
    name: &'a str,
    current_package: &'a str,
    dependencies: &'a [String],
    span: Span,
}

enum CandidateChoice {
    Resolved(PackageItemId),
    Missing,
    Ambiguous,
}

fn validate_persisted_interface_graph(
    graph: PackageInterfaceGraph,
) -> Result<PackageInterfaceGraph, Vec<Diagnostic>> {
    let mut validator = PersistedInterfaceConsistencyValidator {
        graph: &graph,
        diagnostics: Vec::new(),
    };
    validator.validate();
    if validator.diagnostics.is_empty() {
        Ok(graph)
    } else {
        Err(validator.diagnostics)
    }
}

struct PersistedInterfaceConsistencyValidator<'a> {
    graph: &'a PackageInterfaceGraph,
    diagnostics: Vec<Diagnostic>,
}

impl PersistedInterfaceConsistencyValidator<'_> {
    fn validate(&mut self) {
        for package in &self.graph.packages {
            for record in &package.records {
                for field in &record.fields {
                    self.validate_type(&field.ty, field.span);
                }
            }
            for enumeration in &package.enums {
                for variant in &enumeration.variants {
                    if let Some(payload) = &variant.payload {
                        self.validate_type(payload, variant.span);
                    }
                }
            }
            for function in &package.functions {
                for param in &function.params {
                    self.validate_type(&param.ty, param.span);
                }
                self.validate_type(&function.ret, function.span);
            }
        }
    }

    fn validate_type(&mut self, ty: &TypeInfo, span: Span) {
        match ty {
            TypeInfo::PackageRecord { item, args, .. } => {
                self.validate_package_record_reference(*item, args.len(), span);
                for arg in args {
                    self.validate_type(arg, span);
                }
            }
            TypeInfo::PackageEnum { item, args, .. } => {
                self.validate_package_enum_reference(*item, args.len(), span);
                for arg in args {
                    self.validate_type(arg, span);
                }
            }
            TypeInfo::PackageOpaque { item, .. } => {
                if self.package_opaque_type(*item).is_none() {
                    self.push_unknown_item("opaque type", *item, span);
                }
            }
            TypeInfo::EnumConstructor {
                enum_item: Some(item),
                ..
            } => {
                if self.package_enum(*item).is_none() {
                    self.push_unknown_item("enum", *item, span);
                }
            }
            TypeInfo::Record(_, args) | TypeInfo::Enum { args, .. } => {
                for arg in args {
                    self.validate_type(arg, span);
                }
            }
            TypeInfo::List(item) | TypeInfo::Option(item) | TypeInfo::Task(item) => {
                self.validate_type(item, span)
            }
            TypeInfo::Map(key, value) | TypeInfo::Result(key, value) => {
                self.validate_type(key, span);
                self.validate_type(value, span);
            }
            TypeInfo::Function(function) => {
                for param in &function.params {
                    self.validate_type(param, span);
                }
                self.validate_type(&function.ret, span);
            }
            TypeInfo::GenericParam(_)
            | TypeInfo::EnumConstructor {
                enum_item: None, ..
            }
            | TypeInfo::Int
            | TypeInfo::Bool
            | TypeInfo::String
            | TypeInfo::Unit
            | TypeInfo::Builtin(_)
            | TypeInfo::Unknown
            | TypeInfo::Error => {}
        }
    }

    fn validate_package_record_reference(
        &mut self,
        item: PackageItemId,
        actual: usize,
        span: Span,
    ) {
        let Some((package_path, record)) = self.package_record(item) else {
            self.push_unknown_item("record", item, span);
            return;
        };
        let package_path = package_path.to_string();
        let name = record.name.clone();
        let expected = record.type_params.len();
        self.validate_generic_arity("record", &package_path, &name, expected, actual, span);
    }

    fn validate_package_enum_reference(&mut self, item: PackageItemId, actual: usize, span: Span) {
        let Some((package_path, enumeration)) = self.package_enum(item) else {
            self.push_unknown_item("enum", item, span);
            return;
        };
        let package_path = package_path.to_string();
        let name = enumeration.name.clone();
        let expected = enumeration.type_params.len();
        self.validate_generic_arity("enum", &package_path, &name, expected, actual, span);
    }

    fn validate_generic_arity(
        &mut self,
        kind: &str,
        package_path: &str,
        name: &str,
        expected: usize,
        actual: usize,
        span: Span,
    ) {
        if expected == actual {
            return;
        }
        self.diagnostics.push(
            Diagnostic::new(
                "PK019",
                format!(
                    "package interface has stale generic {kind} reference `{package_path}::{name}`: expected {expected} type arguments but found {actual}"
                ),
                span,
            )
            .with_suggestion(regenerate_interface_artifact_suggestion()),
        );
    }

    fn push_unknown_item(&mut self, kind: &str, item: PackageItemId, span: Span) {
        self.diagnostics.push(
            Diagnostic::new(
                "PK019",
                format!("package interface references unknown {kind} identity {item:?}"),
                span,
            )
            .with_suggestion(regenerate_interface_artifact_suggestion()),
        );
    }

    fn package_record(&self, item: PackageItemId) -> Option<(&str, &PackageInterfaceRecord)> {
        self.graph.packages.iter().find_map(|package| {
            package
                .records
                .iter()
                .find(|record| record.item == item)
                .map(|record| (package.path.as_str(), record))
        })
    }

    fn package_enum(&self, item: PackageItemId) -> Option<(&str, &PackageInterfaceEnum)> {
        self.graph.packages.iter().find_map(|package| {
            package
                .enums
                .iter()
                .find(|enumeration| enumeration.item == item)
                .map(|enumeration| (package.path.as_str(), enumeration))
        })
    }

    fn package_opaque_type(
        &self,
        item: PackageItemId,
    ) -> Option<(&str, &PackageInterfaceOpaqueType)> {
        self.graph.packages.iter().find_map(|package| {
            package
                .opaque_types
                .iter()
                .find(|opaque| opaque.item == item)
                .map(|opaque| (package.path.as_str(), opaque))
        })
    }
}

fn package_item_kind_label(kind: PackageItemKind) -> &'static str {
    match kind {
        PackageItemKind::Record => "record",
        PackageItemKind::Enum => "enum",
        PackageItemKind::OpaqueType => "opaque type",
        PackageItemKind::Function => "function",
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterface {
    pub package: PackageId,
    pub path: String,
    pub dependencies: Vec<String>,
    pub records: Vec<PackageInterfaceRecord>,
    pub enums: Vec<PackageInterfaceEnum>,
    pub opaque_types: Vec<PackageInterfaceOpaqueType>,
    pub functions: Vec<PackageInterfaceFunction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterfaceRecord {
    pub item: PackageItemId,
    pub name: String,
    pub doc_comments: Vec<String>,
    pub type_params: Vec<String>,
    pub json_deny_unknown_fields: bool,
    pub cli_about: Option<String>,
    pub fields: Vec<PackageInterfaceField>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterfaceField {
    pub name: String,
    pub json_rename: Option<String>,
    pub json_aliases: Vec<String>,
    pub json_validation: Vec<JsonDecodeValidationRule>,
    pub cli_name: Option<String>,
    pub cli_short: Option<String>,
    pub cli_position: Option<u32>,
    pub cli_value_source: Option<CliValueSource>,
    pub cli_aliases: Vec<String>,
    pub cli_help: Option<String>,
    pub cli_hidden: bool,
    pub cli_subcommand: bool,
    pub ty: TypeInfo,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterfaceEnum {
    pub item: PackageItemId,
    pub name: String,
    pub doc_comments: Vec<String>,
    pub type_params: Vec<String>,
    pub cli_about: Option<String>,
    pub variants: Vec<PackageInterfaceEnumVariant>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterfaceEnumVariant {
    pub name: String,
    pub json_rename: Option<String>,
    pub json_aliases: Vec<String>,
    pub cli_name: Option<String>,
    pub cli_aliases: Vec<String>,
    pub cli_about: Option<String>,
    pub cli_hidden: bool,
    pub payload: Option<TypeInfo>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterfaceOpaqueType {
    pub item: PackageItemId,
    pub name: String,
    pub doc_comments: Vec<String>,
    pub handle_facts: OpaqueHandleFacts,
    pub span: Span,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpaqueHandleFacts {
    pub runtime_backed: bool,
    pub copyable: bool,
    pub cloneable: bool,
    pub sendable: bool,
    pub shareable: bool,
    pub structurally_comparable: bool,
    pub serializable: bool,
    pub closeable: bool,
    pub close_function: Option<PackageItemId>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterfaceFunction {
    pub item: PackageItemId,
    pub name: String,
    pub doc_comments: Vec<String>,
    pub type_params: Vec<String>,
    pub params: Vec<PackageInterfaceParam>,
    pub ret: TypeInfo,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterfaceParam {
    pub name: String,
    pub ty: TypeInfo,
    pub mode: PackageInterfaceParamMode,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PackageInterfaceParamMode {
    #[default]
    Borrow,
    Consume,
}

impl PackageInterfaceParamMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Borrow => "borrow",
            Self::Consume => "consume",
        }
    }
}

struct PersistedInterfaceParser<'a> {
    lines: Vec<&'a str>,
    index: usize,
    symbols: &'a mut SymbolTable,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> PersistedInterfaceParser<'a> {
    fn new(text: &'a str, symbols: &'a mut SymbolTable) -> Self {
        Self {
            lines: text.lines().collect(),
            index: 0,
            symbols,
            diagnostics: Vec::new(),
        }
    }

    fn parse(mut self) -> Result<PackageInterfaceGraph, Vec<Diagnostic>> {
        match self.next_parts() {
            Some(parts)
                if parts.len() == 1
                    && (parts[0] == PERSISTED_INTERFACE_HEADER
                        || LEGACY_PERSISTED_INTERFACE_HEADERS.contains(&parts[0])) => {}
            Some(_) => self.push_error("invalid package interface header"),
            None => self.push_error("empty package interface"),
        }
        self.validate_optional_hash();

        let mut packages = Vec::new();
        while self.index < self.lines.len() {
            let Some(parts) = self.next_parts() else {
                break;
            };
            if parts.first().copied() != Some("package") {
                self.push_error("expected package line");
                break;
            }
            let Some(package) = self.parse_package(parts) else {
                break;
            };
            packages.push(package);
        }

        if self.diagnostics.is_empty() {
            Ok(PackageInterfaceGraph { packages })
        } else {
            Err(self.diagnostics)
        }
    }

    fn validate_optional_hash(&mut self) {
        let Some(line) = self.lines.get(self.index).copied() else {
            return;
        };
        let Some(expected) = line.strip_prefix("hash\t") else {
            return;
        };
        self.index += 1;
        let body = if self.index < self.lines.len() {
            format!("{}\n", self.lines[self.index..].join("\n"))
        } else {
            String::new()
        };
        let actual = stable_hash_hex(&canonicalize_persisted_interface_body_for_hash(&body));
        let legacy_actual = stable_hash_hex(&body);
        if expected != actual && expected != legacy_actual {
            let mut diagnostic = Diagnostic::new(
                "PK019",
                format!(
                    "package interface hash mismatch: expected `{expected}` but found `{actual}`"
                ),
                Span::default(),
            )
            .with_context(artifact_hash_context(
                "expected", "artifact", None, expected,
            ))
            .with_context(artifact_hash_context("actual", "artifact", None, actual))
            .with_suggestion(regenerate_interface_artifact_suggestion());
            add_interface_regeneration_context(&mut diagnostic);
            self.diagnostics.push(diagnostic);
        }
    }

    fn parse_package(&mut self, parts: Vec<&str>) -> Option<PackageInterface> {
        if parts.len() != 6 && parts.len() != 7 && parts.len() != 8 {
            self.push_error("invalid package line");
            return None;
        }
        let package = PackageId::new(self.parse_u32(parts[1], "package id")?);
        let path = parts[2].to_string();
        let (dependency_count, record_count, enum_count, opaque_count, function_count) =
            if parts.len() == 8 {
                (
                    self.parse_usize(parts[3], "dependency count")?,
                    self.parse_usize(parts[4], "record count")?,
                    self.parse_usize(parts[5], "enum count")?,
                    self.parse_usize(parts[6], "opaque type count")?,
                    self.parse_usize(parts[7], "function count")?,
                )
            } else if parts.len() == 7 {
                (
                    self.parse_usize(parts[3], "dependency count")?,
                    self.parse_usize(parts[4], "record count")?,
                    self.parse_usize(parts[5], "enum count")?,
                    0,
                    self.parse_usize(parts[6], "function count")?,
                )
            } else {
                (
                    0,
                    self.parse_usize(parts[3], "record count")?,
                    self.parse_usize(parts[4], "enum count")?,
                    0,
                    self.parse_usize(parts[5], "function count")?,
                )
            };

        let mut dependencies = Vec::with_capacity(dependency_count);
        for _ in 0..dependency_count {
            let dependency = self.expect_line("dependency")?;
            if dependency.len() != 2 {
                self.push_error("invalid package dependency line");
                return None;
            }
            dependencies.push(dependency[1].to_string());
        }
        let mut records = Vec::with_capacity(record_count);
        for _ in 0..record_count {
            records.push(self.parse_record()?);
        }
        let mut enums = Vec::with_capacity(enum_count);
        for _ in 0..enum_count {
            enums.push(self.parse_enum()?);
        }
        let mut opaque_types = Vec::with_capacity(opaque_count);
        for _ in 0..opaque_count {
            opaque_types.push(self.parse_opaque_type()?);
        }
        let mut functions = Vec::with_capacity(function_count);
        for _ in 0..function_count {
            functions.push(self.parse_function()?);
        }

        Some(PackageInterface {
            package,
            path,
            dependencies,
            records,
            enums,
            opaque_types,
            functions,
        })
    }

    fn parse_record(&mut self) -> Option<PackageInterfaceRecord> {
        let parts = self.expect_line("record")?;
        if parts.len() < 5 {
            self.push_error("invalid record line");
            return None;
        }
        let item = PackageItemId::new(self.parse_u32(parts[1], "record item id")?);
        let name = parts[2].to_string();
        let span = self.parse_span(parts[3])?;
        let (type_params, field_count_index) = if parts.len() == 5 {
            (Vec::new(), 4)
        } else {
            let type_param_count = self.parse_usize(parts[4], "record type parameter count")?;
            let field_count_index = 5 + type_param_count;
            if parts.len() < field_count_index + 1 {
                self.push_error("invalid record type parameter list");
                return None;
            }
            let type_params = parts[5..field_count_index]
                .iter()
                .map(|part| (*part).to_string())
                .collect::<Vec<_>>();
            (type_params, field_count_index)
        };
        let field_count = self.parse_usize(parts[field_count_index], "field count")?;
        let mut index = field_count_index + 1;
        let mut json_flags = 0;
        if index < parts.len() && parts[index] != "cli" {
            json_flags = self.parse_u32(parts[index], "record JSON flags")?;
            index += 1;
        }
        if json_flags & !1 != 0 {
            self.push_error("invalid record JSON flags");
            return None;
        }
        let cli_about = if index < parts.len() {
            if parts[index] != "cli" {
                self.push_error("invalid record CLI metadata");
                return None;
            }
            index += 1;
            let Some(about_marker) = parts.get(index).copied() else {
                self.push_error("invalid record CLI about marker");
                return None;
            };
            index += 1;
            let about = match about_marker {
                "0" => None,
                "1" => {
                    let Some(about) = parts.get(index) else {
                        self.push_error("invalid record CLI about");
                        return None;
                    };
                    index += 1;
                    Some((*about).to_string())
                }
                _ => {
                    self.push_error("invalid record CLI about marker");
                    return None;
                }
            };
            if index != parts.len() {
                self.push_error("invalid record CLI metadata");
                return None;
            }
            about
        } else {
            None
        };
        let doc_comments = self.parse_doc_comments()?;
        let mut fields = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            let field = self.expect_line("field")?;
            if field.len() < 4 {
                self.push_error("invalid field line");
                return None;
            }
            let (
                json_rename,
                json_aliases,
                json_validation,
                cli_name,
                cli_short,
                cli_position,
                cli_value_source,
                cli_aliases,
                cli_help,
                cli_hidden,
                cli_subcommand,
            ) = match field.len() {
                4 => (
                    None,
                    Vec::new(),
                    Vec::new(),
                    None,
                    None,
                    None,
                    None,
                    Vec::new(),
                    None,
                    false,
                    false,
                ),
                5 => (
                    Some(field[4].to_string()),
                    Vec::new(),
                    Vec::new(),
                    None,
                    None,
                    None,
                    None,
                    Vec::new(),
                    None,
                    false,
                    false,
                ),
                _ => {
                    let alias_count = self.parse_usize(field[5], "field JSON alias count")?;
                    if field.len() != 6 + alias_count && field.len() < 7 + alias_count {
                        self.push_error("invalid field JSON alias list");
                        return None;
                    }
                    let rename = if field[4] == "-" {
                        None
                    } else {
                        Some(field[4].to_string())
                    };
                    let aliases = field[6..]
                        .iter()
                        .take(alias_count)
                        .map(|alias| (*alias).to_string())
                        .collect();
                    if field.len() == 6 + alias_count {
                        (
                            rename,
                            aliases,
                            Vec::new(),
                            None,
                            None,
                            None,
                            None,
                            Vec::new(),
                            None,
                            false,
                            false,
                        )
                    } else {
                        let validation_count_index = 6 + alias_count;
                        let validation_count = self.parse_usize(
                            field[validation_count_index],
                            "field JSON validation count",
                        )?;
                        let cli_marker_index = validation_count_index + 1 + validation_count;
                        if field.len() != cli_marker_index
                            && field.get(cli_marker_index).copied() != Some("cli")
                        {
                            self.push_error("invalid field JSON validation list");
                            return None;
                        }
                        let mut validation = Vec::with_capacity(validation_count);
                        for token in &field[validation_count_index + 1..cli_marker_index] {
                            match JsonDecodeValidationRule::from_artifact_token(token) {
                                Ok(rule) => validation.push(rule),
                                Err(error) => {
                                    self.push_error(error);
                                    return None;
                                }
                            }
                        }
                        if field.len() == cli_marker_index {
                            (
                                rename,
                                aliases,
                                validation,
                                None,
                                None,
                                None,
                                None,
                                Vec::new(),
                                None,
                                false,
                                false,
                            )
                        } else {
                            let mut index = cli_marker_index + 1;
                            let cli_name = if field.get(index).copied() == Some("-") {
                                index += 1;
                                None
                            } else {
                                let Some(name) = field.get(index) else {
                                    self.push_error("invalid field CLI metadata");
                                    return None;
                                };
                                index += 1;
                                Some((*name).to_string())
                            };
                            let Some(alias_count) = field.get(index).copied() else {
                                self.push_error("invalid field CLI alias count");
                                return None;
                            };
                            let cli_alias_count =
                                self.parse_usize(alias_count, "field CLI alias count")?;
                            index += 1;
                            if field.len() < index + cli_alias_count + 2 {
                                self.push_error("invalid field CLI alias list");
                                return None;
                            }
                            let cli_aliases = field[index..index + cli_alias_count]
                                .iter()
                                .map(|alias| (*alias).to_string())
                                .collect::<Vec<_>>();
                            index += cli_alias_count;
                            let flags = self.parse_u32(field[index], "field CLI flags")?;
                            if flags & !3 != 0 {
                                self.push_error("invalid field CLI flags");
                                return None;
                            }
                            index += 1;
                            let Some(help_marker) = field.get(index).copied() else {
                                self.push_error("invalid field CLI help marker");
                                return None;
                            };
                            index += 1;
                            let cli_help = match help_marker {
                                "0" => None,
                                "1" => {
                                    let Some(help) = field.get(index) else {
                                        self.push_error("invalid field CLI help");
                                        return None;
                                    };
                                    index += 1;
                                    Some((*help).to_string())
                                }
                                _ => {
                                    self.push_error("invalid field CLI help marker");
                                    return None;
                                }
                            };
                            let mut cli_short = None;
                            let mut cli_position = None;
                            let mut cli_value_source = None;
                            while index < field.len() {
                                match field[index] {
                                    "short" => {
                                        if cli_short.is_some() {
                                            self.push_error("duplicate field CLI short");
                                            return None;
                                        }
                                        index += 1;
                                        let Some(short) = field.get(index) else {
                                            self.push_error("invalid field CLI short");
                                            return None;
                                        };
                                        if !is_cli_short_option_token(short) {
                                            self.push_error("invalid field CLI short");
                                            return None;
                                        }
                                        index += 1;
                                        cli_short = Some((*short).to_string());
                                    }
                                    "position" => {
                                        if cli_position.is_some() {
                                            self.push_error("duplicate field CLI position");
                                            return None;
                                        }
                                        index += 1;
                                        let Some(position) = field.get(index) else {
                                            self.push_error("invalid field CLI position");
                                            return None;
                                        };
                                        let position =
                                            self.parse_u32(position, "field CLI position")?;
                                        if position == 0 {
                                            self.push_error("invalid field CLI position");
                                            return None;
                                        }
                                        index += 1;
                                        cli_position = Some(position);
                                    }
                                    "value_source" => {
                                        if cli_value_source.is_some() {
                                            self.push_error("duplicate field CLI value source");
                                            return None;
                                        }
                                        index += 1;
                                        let Some(value_source) = field.get(index) else {
                                            self.push_error("invalid field CLI value source");
                                            return None;
                                        };
                                        let value_source =
                                            match CliValueSource::from_artifact_token(value_source)
                                            {
                                                Ok(value_source) => value_source,
                                                Err(error) => {
                                                    self.push_error(error);
                                                    return None;
                                                }
                                            };
                                        index += 1;
                                        cli_value_source = Some(value_source);
                                    }
                                    _ => {
                                        self.push_error("invalid field CLI metadata");
                                        return None;
                                    }
                                }
                            }
                            (
                                rename,
                                aliases,
                                validation,
                                cli_name,
                                cli_short,
                                cli_position,
                                cli_value_source,
                                cli_aliases,
                                cli_help,
                                flags & 1 != 0,
                                flags & 2 != 0,
                            )
                        }
                    }
                }
            };
            fields.push(PackageInterfaceField {
                name: field[1].to_string(),
                json_rename,
                json_aliases,
                json_validation,
                cli_name,
                cli_short,
                cli_position,
                cli_value_source,
                cli_aliases,
                cli_help,
                cli_hidden,
                cli_subcommand,
                span: self.parse_span(field[2])?,
                ty: self.parse_type(field[3])?,
            });
        }
        Some(PackageInterfaceRecord {
            item,
            name,
            doc_comments,
            type_params,
            json_deny_unknown_fields: json_flags & 1 != 0,
            cli_about,
            fields,
            span,
        })
    }

    fn parse_enum(&mut self) -> Option<PackageInterfaceEnum> {
        let parts = self.expect_line("enum")?;
        if parts.len() < 6 {
            self.push_error("invalid enum line");
            return None;
        }
        let item = PackageItemId::new(self.parse_u32(parts[1], "enum item id")?);
        let name = parts[2].to_string();
        let span = self.parse_span(parts[3])?;
        let type_param_count = self.parse_usize(parts[4], "enum type parameter count")?;
        let variant_count_index = 5 + type_param_count;
        if parts.len() < variant_count_index + 1 {
            self.push_error("invalid enum type parameter list");
            return None;
        }
        let type_params = parts[5..variant_count_index]
            .iter()
            .map(|part| (*part).to_string())
            .collect::<Vec<_>>();
        let variant_count = self.parse_usize(parts[variant_count_index], "enum variant count")?;
        let mut index = variant_count_index + 1;
        let cli_about = if index < parts.len() {
            if parts[index] != "cli" {
                self.push_error("invalid enum CLI metadata");
                return None;
            }
            index += 1;
            let Some(about_marker) = parts.get(index).copied() else {
                self.push_error("invalid enum CLI about marker");
                return None;
            };
            index += 1;
            let about = match about_marker {
                "0" => None,
                "1" => {
                    let Some(about) = parts.get(index) else {
                        self.push_error("invalid enum CLI about");
                        return None;
                    };
                    index += 1;
                    Some((*about).to_string())
                }
                _ => {
                    self.push_error("invalid enum CLI about marker");
                    return None;
                }
            };
            if index != parts.len() {
                self.push_error("invalid enum CLI metadata");
                return None;
            }
            about
        } else {
            None
        };
        let doc_comments = self.parse_doc_comments()?;
        let mut variants = Vec::with_capacity(variant_count);
        for _ in 0..variant_count {
            let variant = self.expect_line("variant")?;
            if variant.len() < 4 {
                self.push_error("invalid enum variant line");
                return None;
            }
            let (json_rename, json_aliases, cli_name, cli_aliases, cli_about, cli_hidden) =
                match variant.len() {
                    4 => (None, Vec::new(), None, Vec::new(), None, false),
                    5 => (
                        Some(variant[4].to_string()),
                        Vec::new(),
                        None,
                        Vec::new(),
                        None,
                        false,
                    ),
                    _ => {
                        let alias_count =
                            self.parse_usize(variant[5], "enum variant JSON alias count")?;
                        if variant.len() < 6 + alias_count {
                            self.push_error("invalid enum variant JSON alias list");
                            return None;
                        }
                        let rename = if variant[4] == "-" {
                            None
                        } else {
                            Some(variant[4].to_string())
                        };
                        let aliases = variant[6..]
                            .iter()
                            .take(alias_count)
                            .map(|alias| (*alias).to_string())
                            .collect::<Vec<_>>();
                        let mut index = 6 + alias_count;
                        let (cli_name, cli_aliases, cli_about, cli_hidden) = if index
                            < variant.len()
                        {
                            if variant[index] != "cli" {
                                self.push_error("invalid enum variant CLI metadata");
                                return None;
                            }
                            index += 1;
                            let Some(cli_name_token) = variant.get(index).copied() else {
                                self.push_error("invalid enum variant CLI name");
                                return None;
                            };
                            index += 1;
                            let cli_name = if cli_name_token == "-" {
                                None
                            } else {
                                if !is_cli_command_token(cli_name_token) {
                                    self.push_error("invalid enum variant CLI name");
                                    return None;
                                }
                                Some(cli_name_token.to_string())
                            };
                            let Some(alias_count) = variant.get(index).copied() else {
                                self.push_error("invalid enum variant CLI alias count");
                                return None;
                            };
                            let cli_alias_count =
                                self.parse_usize(alias_count, "enum variant CLI alias count")?;
                            index += 1;
                            if variant.len() < index + cli_alias_count + 2 {
                                self.push_error("invalid enum variant CLI alias list");
                                return None;
                            }
                            let cli_aliases = variant[index..index + cli_alias_count]
                                .iter()
                                .map(|alias| {
                                    if !is_cli_command_token(alias) {
                                        self.push_error("invalid enum variant CLI alias");
                                        return None;
                                    }
                                    Some((*alias).to_string())
                                })
                                .collect::<Option<Vec<_>>>()?;
                            index += cli_alias_count;
                            let flags = self.parse_u32(variant[index], "enum variant CLI flags")?;
                            if flags & !1 != 0 {
                                self.push_error("invalid enum variant CLI flags");
                                return None;
                            }
                            index += 1;
                            let Some(about_marker) = variant.get(index).copied() else {
                                self.push_error("invalid enum variant CLI about marker");
                                return None;
                            };
                            index += 1;
                            let cli_about = match about_marker {
                                "0" => None,
                                "1" => {
                                    let Some(about) = variant.get(index) else {
                                        self.push_error("invalid enum variant CLI about");
                                        return None;
                                    };
                                    index += 1;
                                    Some((*about).to_string())
                                }
                                _ => {
                                    self.push_error("invalid enum variant CLI about marker");
                                    return None;
                                }
                            };
                            if index != variant.len() {
                                self.push_error("invalid enum variant CLI metadata");
                                return None;
                            }
                            (cli_name, cli_aliases, cli_about, flags & 1 != 0)
                        } else {
                            (None, Vec::new(), None, false)
                        };
                        (
                            rename,
                            aliases,
                            cli_name,
                            cli_aliases,
                            cli_about,
                            cli_hidden,
                        )
                    }
                };
            variants.push(PackageInterfaceEnumVariant {
                name: variant[1].to_string(),
                json_rename,
                json_aliases,
                cli_name,
                cli_aliases,
                cli_about,
                cli_hidden,
                span: self.parse_span(variant[2])?,
                payload: if variant[3] == "-" {
                    None
                } else {
                    Some(self.parse_type(variant[3])?)
                },
            });
        }
        Some(PackageInterfaceEnum {
            item,
            name,
            doc_comments,
            type_params,
            cli_about,
            variants,
            span,
        })
    }

    fn parse_opaque_type(&mut self) -> Option<PackageInterfaceOpaqueType> {
        let parts = self.expect_line("opaque-type")?;
        if parts.len() != 4 && parts.len() != 5 {
            self.push_error("invalid opaque type line");
            return None;
        }
        let item = PackageItemId::new(self.parse_u32(parts[1], "opaque type item id")?);
        let name = parts[2].to_string();
        let span = self.parse_span(parts[3])?;
        let handle_facts = if parts.len() == 5 {
            self.parse_opaque_handle_facts(parts[4])?
        } else {
            OpaqueHandleFacts::default()
        };
        let doc_comments = self.parse_doc_comments()?;
        Some(PackageInterfaceOpaqueType {
            item,
            name,
            doc_comments,
            handle_facts,
            span,
        })
    }

    fn parse_function(&mut self) -> Option<PackageInterfaceFunction> {
        let parts = self.expect_line("function")?;
        if parts.len() < 6 {
            self.push_error("invalid function line");
            return None;
        }
        let item = PackageItemId::new(self.parse_u32(parts[1], "function item id")?);
        let name = parts[2].to_string();
        let span = self.parse_span(parts[3])?;
        let (type_params, param_count_index) = if parts.len() == 6 {
            (Vec::new(), 4)
        } else {
            let type_param_count = self.parse_usize(parts[4], "function type parameter count")?;
            let param_count_index = 5 + type_param_count;
            if parts.len() != param_count_index + 2 {
                self.push_error("invalid function type parameter list");
                return None;
            }
            let type_params = parts[5..param_count_index]
                .iter()
                .map(|part| (*part).to_string())
                .collect::<Vec<_>>();
            (type_params, param_count_index)
        };
        let param_count = self.parse_usize(parts[param_count_index], "function parameter count")?;
        let ret = self.parse_type(parts[param_count_index + 1])?;
        let doc_comments = self.parse_doc_comments()?;
        let mut params = Vec::with_capacity(param_count);
        for _ in 0..param_count {
            let param = self.expect_line("param")?;
            if param.len() != 4 && param.len() != 5 {
                self.push_error("invalid function parameter line");
                return None;
            }
            let (mode, ty_index) = if param.len() == 5 {
                (self.parse_param_mode(param[3])?, 4)
            } else {
                (PackageInterfaceParamMode::Borrow, 3)
            };
            params.push(PackageInterfaceParam {
                name: param[1].to_string(),
                span: self.parse_span(param[2])?,
                ty: self.parse_type(param[ty_index])?,
                mode,
            });
        }
        Some(PackageInterfaceFunction {
            item,
            name,
            doc_comments,
            type_params,
            params,
            ret,
            span,
        })
    }

    fn parse_opaque_handle_facts(&mut self, value: &str) -> Option<OpaqueHandleFacts> {
        let mut facts = OpaqueHandleFacts::default();
        if value == "-" {
            return Some(facts);
        }
        for part in value.split(',') {
            let Some((name, raw_value)) = part.split_once('=') else {
                self.push_error(format!("invalid opaque handle fact `{part}`"));
                return None;
            };
            match name {
                "runtimeBacked" => {
                    facts.runtime_backed = self.parse_bool(raw_value, "runtimeBacked")?
                }
                "copyable" => facts.copyable = self.parse_bool(raw_value, "copyable")?,
                "cloneable" => facts.cloneable = self.parse_bool(raw_value, "cloneable")?,
                "sendable" => facts.sendable = self.parse_bool(raw_value, "sendable")?,
                "shareable" => facts.shareable = self.parse_bool(raw_value, "shareable")?,
                "structurallyComparable" => {
                    facts.structurally_comparable =
                        self.parse_bool(raw_value, "structurallyComparable")?
                }
                "serializable" => {
                    facts.serializable = self.parse_bool(raw_value, "serializable")?
                }
                "closeable" => facts.closeable = self.parse_bool(raw_value, "closeable")?,
                "closeFunction" => {
                    facts.close_function = if raw_value == "-" {
                        None
                    } else {
                        Some(PackageItemId::new(
                            self.parse_u32(raw_value, "closeFunction item id")?,
                        ))
                    };
                }
                unknown => {
                    self.push_error(format!("unknown opaque handle fact `{unknown}`"));
                    return None;
                }
            }
        }
        Some(facts)
    }

    fn parse_bool(&mut self, value: &str, label: &str) -> Option<bool> {
        match value {
            "true" => Some(true),
            "false" => Some(false),
            _ => {
                self.push_error(format!("invalid {label} boolean `{value}`"));
                None
            }
        }
    }

    fn parse_param_mode(&mut self, value: &str) -> Option<PackageInterfaceParamMode> {
        match value {
            "borrow" => Some(PackageInterfaceParamMode::Borrow),
            "consume" => Some(PackageInterfaceParamMode::Consume),
            _ => {
                self.push_error(format!("invalid parameter mode `{value}`"));
                None
            }
        }
    }

    fn parse_doc_comments(&mut self) -> Option<Vec<String>> {
        let mut comments = Vec::new();
        while self
            .lines
            .get(self.index)
            .is_some_and(|line| line.starts_with("doc\t"))
        {
            let parts = self.next_parts()?;
            if parts.len() != 2 {
                self.push_error("invalid doc comment line");
                return None;
            }
            comments.push(parse_doc_comment_text(parts[1]));
        }
        Some(comments)
    }

    fn expect_line(&mut self, tag: &str) -> Option<Vec<&'a str>> {
        let parts = self.next_parts()?;
        if parts.first().copied() != Some(tag) {
            self.push_error(format!("expected {tag} line"));
            return None;
        }
        Some(parts)
    }

    fn next_parts(&mut self) -> Option<Vec<&'a str>> {
        let line = *self.lines.get(self.index)?;
        self.index += 1;
        Some(line.split('\t').collect())
    }

    fn parse_u32(&mut self, value: &str, label: &str) -> Option<u32> {
        value.parse().map_or_else(
            |_| {
                self.push_error(format!("invalid {label} `{value}`"));
                None
            },
            Some,
        )
    }

    fn parse_usize(&mut self, value: &str, label: &str) -> Option<usize> {
        value.parse().map_or_else(
            |_| {
                self.push_error(format!("invalid {label} `{value}`"));
                None
            },
            Some,
        )
    }

    fn parse_span(&mut self, value: &str) -> Option<Span> {
        let Some((start, end)) = value.split_once('-') else {
            self.push_error(format!("invalid span `{value}`"));
            return None;
        };
        let Some((start_line, start_column)) = start.split_once(':') else {
            self.push_error(format!("invalid span `{value}`"));
            return None;
        };
        let Some((end_line, end_column)) = end.split_once(':') else {
            self.push_error(format!("invalid span `{value}`"));
            return None;
        };
        Some(Span::new(
            span::Position::new(
                self.parse_usize(start_line, "span start line")?,
                self.parse_usize(start_column, "span start column")?,
            ),
            span::Position::new(
                self.parse_usize(end_line, "span end line")?,
                self.parse_usize(end_column, "span end column")?,
            ),
        ))
    }

    fn parse_type(&mut self, value: &str) -> Option<TypeInfo> {
        let tokens = value.split_whitespace().collect::<Vec<_>>();
        let mut parser = TypeInfoParser {
            tokens: &tokens,
            index: 0,
            symbols: self.symbols,
        };
        match parser.parse() {
            Ok(ty) if parser.index == tokens.len() => Some(ty),
            Ok(_) => {
                self.push_error(format!("trailing tokens in type `{value}`"));
                None
            }
            Err(message) => {
                self.push_error(message);
                None
            }
        }
    }

    fn push_error(&mut self, message: impl Into<String>) {
        self.diagnostics
            .push(Diagnostic::new("PK018", message.into(), Span::default()));
    }
}

struct TypeInfoParser<'a, 's> {
    tokens: &'a [&'a str],
    index: usize,
    symbols: &'s mut SymbolTable,
}

impl<'a, 's> TypeInfoParser<'a, 's> {
    fn parse(&mut self) -> Result<TypeInfo, String> {
        let token = self.next()?;
        match token {
            "Int" => Ok(TypeInfo::Int),
            "Bool" => Ok(TypeInfo::Bool),
            "String" => Ok(TypeInfo::String),
            "Unit" => Ok(TypeInfo::Unit),
            "GenericParam" => Ok(TypeInfo::GenericParam(self.symbol()?)),
            "Record" => {
                let symbol = self.symbol()?;
                let args = self.type_args_if_present()?;
                Ok(TypeInfo::Record(symbol, args))
            }
            "PackageRecord" => {
                let symbol = self.symbol()?;
                let item = PackageItemId::new(self.u32("package record item id")?);
                let args = self.type_args_if_present()?;
                Ok(TypeInfo::PackageRecord { symbol, item, args })
            }
            "Enum" => {
                let symbol = self.symbol()?;
                let args = self.type_args()?;
                Ok(TypeInfo::Enum { symbol, args })
            }
            "PackageEnum" => {
                let symbol = self.symbol()?;
                let item = PackageItemId::new(self.u32("package enum item id")?);
                let args = self.type_args()?;
                Ok(TypeInfo::PackageEnum { symbol, item, args })
            }
            "PackageOpaque" => {
                let symbol = self.symbol()?;
                let item = PackageItemId::new(self.u32("package opaque type item id")?);
                Ok(TypeInfo::PackageOpaque { symbol, item })
            }
            "List" => Ok(TypeInfo::List(Box::new(self.parse()?))),
            "Map" => Ok(TypeInfo::Map(
                Box::new(self.parse()?),
                Box::new(self.parse()?),
            )),
            "Option" => Ok(TypeInfo::Option(Box::new(self.parse()?))),
            "Result" => Ok(TypeInfo::Result(
                Box::new(self.parse()?),
                Box::new(self.parse()?),
            )),
            "Task" => Ok(TypeInfo::Task(Box::new(self.parse()?))),
            "Function" => {
                let param_count = self.usize("function type parameter count")?;
                let mut params = Vec::with_capacity(param_count);
                for _ in 0..param_count {
                    params.push(self.parse()?);
                }
                let ret = Box::new(self.parse()?);
                Ok(TypeInfo::Function(crate::types::FunctionTypeInfo {
                    params,
                    ret,
                }))
            }
            "EnumConstructor" => {
                let enum_symbol = self.symbol()?;
                let enum_item = match self.next()? {
                    "-" => None,
                    value => Some(PackageItemId::new(parse_u32_token(
                        value,
                        "enum constructor item id",
                    )?)),
                };
                let variant = self.symbol()?;
                Ok(TypeInfo::EnumConstructor {
                    enum_symbol,
                    enum_item,
                    variant,
                })
            }
            "Builtin" => {
                let name = self.next()?;
                prelude::builtin_by_any_name(name)
                    .map(|builtin| TypeInfo::Builtin(builtin.id))
                    .ok_or_else(|| format!("unknown builtin `{name}` in persisted type"))
            }
            "Unknown" => Ok(TypeInfo::Unknown),
            "Error" => Ok(TypeInfo::Error),
            other => Err(format!("unknown type tag `{other}`")),
        }
    }

    fn type_args(&mut self) -> Result<Vec<TypeInfo>, String> {
        let count = self.usize("type argument count")?;
        let mut args = Vec::with_capacity(count);
        for _ in 0..count {
            args.push(self.parse()?);
        }
        Ok(args)
    }

    fn type_args_if_present(&mut self) -> Result<Vec<TypeInfo>, String> {
        let Some(token) = self.tokens.get(self.index) else {
            return Ok(Vec::new());
        };
        if token.parse::<usize>().is_err() {
            return Ok(Vec::new());
        }
        self.type_args()
    }

    fn symbol(&mut self) -> Result<crate::symbol::Symbol, String> {
        let name = self.next()?;
        Ok(self.symbols.intern(name))
    }

    fn usize(&mut self, label: &str) -> Result<usize, String> {
        self.next()?.parse().map_err(|_| format!("invalid {label}"))
    }

    fn u32(&mut self, label: &str) -> Result<u32, String> {
        parse_u32_token(self.next()?, label)
    }

    fn next(&mut self) -> Result<&'a str, String> {
        let Some(token) = self.tokens.get(self.index).copied() else {
            return Err("unexpected end of persisted type".to_string());
        };
        self.index += 1;
        Ok(token)
    }
}

fn parse_u32_token(value: &str, label: &str) -> Result<u32, String> {
    value.parse().map_err(|_| format!("invalid {label}"))
}

fn is_cli_short_option_token(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    chars.next().is_none() && first.is_ascii_alphabetic()
}

fn is_cli_command_token(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

pub(crate) fn stable_hash_hex(text: &str) -> String {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}

fn push_line(out: &mut String, parts: &[String]) {
    out.push_str(&parts.join("\t"));
    out.push('\n');
}

fn record_json_flags(record: &PackageInterfaceRecord) -> u32 {
    u32::from(record.json_deny_unknown_fields)
}

fn push_doc_comment_lines(
    out: &mut String,
    comments: &[String],
    shape: PersistedInterfaceBodyShape,
) {
    if shape == PersistedInterfaceBodyShape::Hash {
        return;
    }
    for comment in comments {
        push_line(out, &["doc".to_string(), format_doc_comment_text(comment)]);
    }
}

fn format_doc_comment_text(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '\t' => escaped.push_str("\\t"),
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn parse_doc_comment_text(text: &str) -> String {
    let mut parsed = String::new();
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            parsed.push(ch);
            continue;
        }
        match chars.next() {
            Some('\\') => parsed.push('\\'),
            Some('t') => parsed.push('\t'),
            Some(other) => {
                parsed.push('\\');
                parsed.push(other);
            }
            None => parsed.push('\\'),
        }
    }
    parsed
}

fn format_opaque_handle_facts(
    facts: &OpaqueHandleFacts,
    context: &PersistedInterfaceIdentityContext,
) -> String {
    let close_function = facts
        .close_function
        .map(|item| {
            context
                .item_id(PackageItemKind::Function, item)
                .unwrap_or(item)
                .as_u32()
                .to_string()
        })
        .unwrap_or_else(|| "-".to_string());
    format!(
        "runtimeBacked={},copyable={},cloneable={},sendable={},shareable={},structurallyComparable={},serializable={},closeable={},closeFunction={}",
        bool_label(facts.runtime_backed),
        bool_label(facts.copyable),
        bool_label(facts.cloneable),
        bool_label(facts.sendable),
        bool_label(facts.shareable),
        bool_label(facts.structurally_comparable),
        bool_label(facts.serializable),
        bool_label(facts.closeable),
        close_function
    )
}

fn bool_label(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn canonicalize_persisted_interface_body_for_hash(body: &str) -> String {
    let mut out = String::new();
    for line in body.lines() {
        let mut parts = line
            .split('\t')
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        match parts.first().map(String::as_str) {
            Some("doc") => continue,
            Some("record" | "enum" | "opaque-type" | "function") if parts.len() >= 4 => {
                parts[3] = PERSISTED_INTERFACE_HASH_SPAN.to_string();
            }
            Some("field" | "variant" | "param") if parts.len() >= 3 => {
                parts[2] = PERSISTED_INTERFACE_HASH_SPAN.to_string();
            }
            _ => {}
        }
        push_line(&mut out, &parts);
    }
    out
}

fn format_interface_span(span: Span, shape: PersistedInterfaceBodyShape) -> String {
    match shape {
        PersistedInterfaceBodyShape::Text => format_span(span),
        PersistedInterfaceBodyShape::Hash => PERSISTED_INTERFACE_HASH_SPAN.to_string(),
    }
}

fn format_span(span: Span) -> String {
    format!(
        "{}:{}-{}:{}",
        span.start.line, span.start.column, span.end.line, span.end.column
    )
}

fn format_type_info(
    ty: &TypeInfo,
    symbols: &SymbolTable,
    context: &PersistedInterfaceIdentityContext,
) -> String {
    let mut tokens = Vec::new();
    push_type_info_tokens(ty, symbols, context, &mut tokens);
    tokens.join(" ")
}

fn push_type_info_tokens(
    ty: &TypeInfo,
    symbols: &SymbolTable,
    context: &PersistedInterfaceIdentityContext,
    tokens: &mut Vec<String>,
) {
    match ty {
        TypeInfo::Int => tokens.push("Int".to_string()),
        TypeInfo::Bool => tokens.push("Bool".to_string()),
        TypeInfo::String => tokens.push("String".to_string()),
        TypeInfo::Unit => tokens.push("Unit".to_string()),
        TypeInfo::GenericParam(symbol) => {
            tokens.push("GenericParam".to_string());
            tokens.push(symbols.resolve(*symbol).to_string());
        }
        TypeInfo::Record(symbol, args) => {
            tokens.push("Record".to_string());
            tokens.push(symbols.resolve(*symbol).to_string());
            push_type_args(args, symbols, context, tokens);
        }
        TypeInfo::PackageRecord { symbol, item, args } => {
            tokens.push("PackageRecord".to_string());
            tokens.push(symbols.resolve(*symbol).to_string());
            tokens.push(
                context
                    .item_id(PackageItemKind::Record, *item)
                    .unwrap_or(*item)
                    .as_u32()
                    .to_string(),
            );
            push_type_args(args, symbols, context, tokens);
        }
        TypeInfo::Enum { symbol, args } => {
            tokens.push("Enum".to_string());
            tokens.push(symbols.resolve(*symbol).to_string());
            push_type_args(args, symbols, context, tokens);
        }
        TypeInfo::PackageEnum { symbol, item, args } => {
            tokens.push("PackageEnum".to_string());
            tokens.push(symbols.resolve(*symbol).to_string());
            tokens.push(
                context
                    .item_id(PackageItemKind::Enum, *item)
                    .unwrap_or(*item)
                    .as_u32()
                    .to_string(),
            );
            push_type_args(args, symbols, context, tokens);
        }
        TypeInfo::PackageOpaque { symbol, item } => {
            tokens.push("PackageOpaque".to_string());
            tokens.push(symbols.resolve(*symbol).to_string());
            tokens.push(
                context
                    .item_id(PackageItemKind::OpaqueType, *item)
                    .unwrap_or(*item)
                    .as_u32()
                    .to_string(),
            );
        }
        TypeInfo::List(item) => {
            tokens.push("List".to_string());
            push_type_info_tokens(item, symbols, context, tokens);
        }
        TypeInfo::Map(key, value) => {
            tokens.push("Map".to_string());
            push_type_info_tokens(key, symbols, context, tokens);
            push_type_info_tokens(value, symbols, context, tokens);
        }
        TypeInfo::Option(item) => {
            tokens.push("Option".to_string());
            push_type_info_tokens(item, symbols, context, tokens);
        }
        TypeInfo::Result(ok, err) => {
            tokens.push("Result".to_string());
            push_type_info_tokens(ok, symbols, context, tokens);
            push_type_info_tokens(err, symbols, context, tokens);
        }
        TypeInfo::Task(item) => {
            tokens.push("Task".to_string());
            push_type_info_tokens(item, symbols, context, tokens);
        }
        TypeInfo::EnumConstructor {
            enum_symbol,
            enum_item,
            variant,
        } => {
            tokens.push("EnumConstructor".to_string());
            tokens.push(symbols.resolve(*enum_symbol).to_string());
            tokens.push(
                enum_item
                    .map(|item| {
                        context
                            .item_id(PackageItemKind::Enum, item)
                            .unwrap_or(item)
                            .as_u32()
                            .to_string()
                    })
                    .unwrap_or_else(|| "-".to_string()),
            );
            tokens.push(symbols.resolve(*variant).to_string());
        }
        TypeInfo::Function(function) => {
            tokens.push("Function".to_string());
            tokens.push(function.params.len().to_string());
            for param in &function.params {
                push_type_info_tokens(param, symbols, context, tokens);
            }
            push_type_info_tokens(&function.ret, symbols, context, tokens);
        }
        TypeInfo::Builtin(builtin) => {
            tokens.push("Builtin".to_string());
            tokens.push(prelude::builtin_name(*builtin).to_string());
        }
        TypeInfo::Unknown => tokens.push("Unknown".to_string()),
        TypeInfo::Error => tokens.push("Error".to_string()),
    }
}

fn push_type_args(
    args: &[TypeInfo],
    symbols: &SymbolTable,
    context: &PersistedInterfaceIdentityContext,
    tokens: &mut Vec<String>,
) {
    tokens.push(args.len().to_string());
    for arg in args {
        push_type_info_tokens(arg, symbols, context, tokens);
    }
}

fn reintern_type_info(ty: &TypeInfo, from: &SymbolTable, to: &mut SymbolTable) -> TypeInfo {
    match ty {
        TypeInfo::GenericParam(symbol) => {
            TypeInfo::GenericParam(reintern_symbol(*symbol, from, to))
        }
        TypeInfo::Record(symbol, args) => TypeInfo::Record(
            reintern_symbol(*symbol, from, to),
            args.iter()
                .map(|arg| reintern_type_info(arg, from, to))
                .collect(),
        ),
        TypeInfo::PackageRecord { symbol, item, args } => TypeInfo::PackageRecord {
            symbol: reintern_symbol(*symbol, from, to),
            item: *item,
            args: args
                .iter()
                .map(|arg| reintern_type_info(arg, from, to))
                .collect(),
        },
        TypeInfo::Enum { symbol, args } => TypeInfo::Enum {
            symbol: reintern_symbol(*symbol, from, to),
            args: args
                .iter()
                .map(|arg| reintern_type_info(arg, from, to))
                .collect(),
        },
        TypeInfo::PackageEnum { symbol, item, args } => TypeInfo::PackageEnum {
            symbol: reintern_symbol(*symbol, from, to),
            item: *item,
            args: args
                .iter()
                .map(|arg| reintern_type_info(arg, from, to))
                .collect(),
        },
        TypeInfo::PackageOpaque { symbol, item } => TypeInfo::PackageOpaque {
            symbol: reintern_symbol(*symbol, from, to),
            item: *item,
        },
        TypeInfo::List(item) => TypeInfo::List(Box::new(reintern_type_info(item, from, to))),
        TypeInfo::Map(key, value) => TypeInfo::Map(
            Box::new(reintern_type_info(key, from, to)),
            Box::new(reintern_type_info(value, from, to)),
        ),
        TypeInfo::Option(item) => TypeInfo::Option(Box::new(reintern_type_info(item, from, to))),
        TypeInfo::Result(ok, err) => TypeInfo::Result(
            Box::new(reintern_type_info(ok, from, to)),
            Box::new(reintern_type_info(err, from, to)),
        ),
        TypeInfo::Task(item) => TypeInfo::Task(Box::new(reintern_type_info(item, from, to))),
        TypeInfo::EnumConstructor {
            enum_symbol,
            enum_item,
            variant,
        } => TypeInfo::EnumConstructor {
            enum_symbol: reintern_symbol(*enum_symbol, from, to),
            enum_item: *enum_item,
            variant: reintern_symbol(*variant, from, to),
        },
        TypeInfo::Function(function) => TypeInfo::Function(FunctionTypeInfo {
            params: function
                .params
                .iter()
                .map(|param| reintern_type_info(param, from, to))
                .collect(),
            ret: Box::new(reintern_type_info(&function.ret, from, to)),
        }),
        TypeInfo::Int
        | TypeInfo::Bool
        | TypeInfo::String
        | TypeInfo::Unit
        | TypeInfo::Builtin(_)
        | TypeInfo::Unknown
        | TypeInfo::Error => ty.clone(),
    }
}

fn reintern_symbol(
    symbol: crate::symbol::Symbol,
    from: &SymbolTable,
    to: &mut SymbolTable,
) -> crate::symbol::Symbol {
    if symbol.as_u32() < from.len() as u32 {
        to.intern(from.resolve(symbol))
    } else {
        symbol
    }
}

impl Program {
    pub fn package_interfaces(&self) -> PackageInterfaceGraph {
        let public_type_items = public_type_items_by_package(&self.package_graph);
        let records_by_item: HashMap<PackageItemId, &RecordStmt> = self
            .statements
            .iter()
            .filter_map(|statement| match statement {
                Stmt::Record(record) => record.package_item.map(|item| (item, record)),
                _ => None,
            })
            .collect();
        let enums_by_item: HashMap<PackageItemId, &EnumStmt> = self
            .statements
            .iter()
            .filter_map(|statement| match statement {
                Stmt::Enum(enumeration) => enumeration.package_item.map(|item| (item, enumeration)),
                _ => None,
            })
            .collect();
        let opaque_types_by_item: HashMap<PackageItemId, &OpaqueTypeStmt> = self
            .statements
            .iter()
            .filter_map(|statement| match statement {
                Stmt::OpaqueType(opaque) => opaque.package_item.map(|item| (item, opaque)),
                _ => None,
            })
            .collect();
        let functions_by_item: HashMap<PackageItemId, &FunctionStmt> = self
            .statements
            .iter()
            .filter_map(|statement| match statement {
                Stmt::Function(function) => function.package_item.map(|item| (item, function)),
                _ => None,
            })
            .collect();

        let packages = self
            .package_graph
            .packages
            .iter()
            .map(|package| {
                let mut records = Vec::new();
                let mut enums = Vec::new();
                let mut opaque_types = Vec::new();
                let mut functions = Vec::new();
                let std_fs_close_item = package_function_item(
                    &self.package_graph,
                    package.id,
                    crate::std_package::FS_PACKAGE,
                    "close",
                );

                for item in self.package_graph.items.iter().filter(|item| {
                    item.package == package.id && item.visibility == Visibility::Public
                }) {
                    match item.kind {
                        PackageItemKind::Record => {
                            if let Some(record) = records_by_item.get(&item.id) {
                                records.push(PackageInterfaceRecord {
                                    item: item.id,
                                    name: item.name.clone(),
                                    doc_comments: record.doc_comments.clone(),
                                    type_params: record.type_params.clone(),
                                    json_deny_unknown_fields: record.json_deny_unknown_fields,
                                    cli_about: record.cli_about.clone(),
                                    fields: record
                                        .fields
                                        .iter()
                                        .map(|field| PackageInterfaceField {
                                            name: field.name.clone(),
                                            json_rename: field.json_rename.clone(),
                                            json_aliases: field.json_aliases.clone(),
                                            json_validation: field.json_validation.clone(),
                                            cli_name: field.cli_name.clone(),
                                            cli_short: field.cli_short.clone(),
                                            cli_position: field.cli_position,
                                            cli_value_source: field.cli_value_source,
                                            cli_aliases: field.cli_aliases.clone(),
                                            cli_help: field.cli_help.clone(),
                                            cli_hidden: field.cli_hidden,
                                            cli_subcommand: field.cli_subcommand,
                                            ty: canonical_public_signature_type(
                                                &field.ty,
                                                package.id,
                                                &self.symbols,
                                                &public_type_items,
                                            ),
                                            span: field.span,
                                        })
                                        .collect(),
                                    span: item.span,
                                });
                            }
                        }
                        PackageItemKind::Enum => {
                            if let Some(enumeration) = enums_by_item.get(&item.id) {
                                enums.push(PackageInterfaceEnum {
                                    item: item.id,
                                    name: item.name.clone(),
                                    doc_comments: enumeration.doc_comments.clone(),
                                    type_params: enumeration.type_params.clone(),
                                    cli_about: enumeration.cli_about.clone(),
                                    variants: enumeration
                                        .variants
                                        .iter()
                                        .map(|variant| PackageInterfaceEnumVariant {
                                            name: variant.name.clone(),
                                            json_rename: variant.json_rename.clone(),
                                            json_aliases: variant.json_aliases.clone(),
                                            cli_name: variant.cli_name.clone(),
                                            cli_aliases: variant.cli_aliases.clone(),
                                            cli_about: variant.cli_about.clone(),
                                            cli_hidden: variant.cli_hidden,
                                            payload: variant.payload.as_ref().map(|payload| {
                                                canonical_public_signature_type(
                                                    payload,
                                                    package.id,
                                                    &self.symbols,
                                                    &public_type_items,
                                                )
                                            }),
                                            span: variant.span,
                                        })
                                        .collect(),
                                    span: item.span,
                                });
                            }
                        }
                        PackageItemKind::OpaqueType => {
                            if let Some(opaque) = opaque_types_by_item.get(&item.id) {
                                opaque_types.push(PackageInterfaceOpaqueType {
                                    item: item.id,
                                    name: item.name.clone(),
                                    doc_comments: opaque.doc_comments.clone(),
                                    handle_facts: package_opaque_handle_facts(
                                        &package.path,
                                        &item.name,
                                        std_fs_close_item,
                                    ),
                                    span: item.span,
                                });
                            }
                        }
                        PackageItemKind::Function => {
                            if let Some(function) = functions_by_item.get(&item.id) {
                                functions.push(PackageInterfaceFunction {
                                    item: item.id,
                                    name: item.name.clone(),
                                    doc_comments: function.doc_comments.clone(),
                                    type_params: function.type_params.clone(),
                                    params: function
                                        .params
                                        .iter()
                                        .map(|param| PackageInterfaceParam {
                                            name: param.name.clone(),
                                            ty: canonical_public_signature_type(
                                                &param.ty,
                                                package.id,
                                                &self.symbols,
                                                &public_type_items,
                                            ),
                                            mode: package_param_mode(
                                                &package.path,
                                                &item.name,
                                                &param.name,
                                            ),
                                            span: param.span,
                                        })
                                        .collect(),
                                    ret: canonical_public_signature_type(
                                        &function.return_ty,
                                        package.id,
                                        &self.symbols,
                                        &public_type_items,
                                    ),
                                    span: item.span,
                                });
                            }
                        }
                    }
                }

                PackageInterface {
                    package: package.id,
                    path: package.path.clone(),
                    dependencies: package_interface_dependencies(package),
                    records,
                    enums,
                    opaque_types,
                    functions,
                }
            })
            .collect();

        PackageInterfaceGraph { packages }
    }

    pub fn validate_package_references_against_interfaces(
        &self,
        interfaces: &PackageInterfaceGraph,
    ) -> Vec<Diagnostic> {
        let mut validator = PackageInterfaceReferenceValidator {
            program: self,
            interfaces,
            checked_items: HashSet::new(),
            diagnostics: Vec::new(),
        };
        validator.validate();
        validator.diagnostics
    }
}

fn package_interface_dependencies(package: &crate::package::PackageInfo) -> Vec<String> {
    let mut dependencies = package
        .imports
        .iter()
        .map(|import| import.path.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    dependencies.sort();
    dependencies
}

fn package_function_item(
    graph: &PackageSymbolGraph,
    package_id: PackageId,
    package_path: &str,
    function_name: &str,
) -> Option<PackageItemId> {
    let package = graph.package(package_id)?;
    if package.path != package_path {
        return None;
    }
    graph
        .items
        .iter()
        .find(|item| {
            item.package == package_id
                && item.kind == PackageItemKind::Function
                && item.name == function_name
        })
        .map(|item| item.id)
}

fn package_opaque_handle_facts(
    package_path: &str,
    opaque_name: &str,
    std_fs_close_item: Option<PackageItemId>,
) -> OpaqueHandleFacts {
    if package_path == crate::std_package::FS_PACKAGE && opaque_name == "File" {
        OpaqueHandleFacts {
            runtime_backed: true,
            copyable: false,
            cloneable: false,
            sendable: false,
            shareable: false,
            structurally_comparable: false,
            serializable: false,
            closeable: true,
            close_function: std_fs_close_item,
        }
    } else {
        OpaqueHandleFacts::default()
    }
}

fn package_param_mode(
    package_path: &str,
    function_name: &str,
    param_name: &str,
) -> PackageInterfaceParamMode {
    if package_path == crate::std_package::FS_PACKAGE
        && function_name == "close"
        && param_name == "file"
    {
        PackageInterfaceParamMode::Consume
    } else {
        PackageInterfaceParamMode::Borrow
    }
}

type PublicTypeItemKey = (PackageId, String, PackageItemKind);

fn public_type_items_by_package(
    graph: &PackageSymbolGraph,
) -> HashMap<PublicTypeItemKey, PackageItemId> {
    let mut items = HashMap::new();
    for item in graph.items.iter().filter(|item| {
        item.visibility == Visibility::Public
            && matches!(
                item.kind,
                PackageItemKind::Record | PackageItemKind::Enum | PackageItemKind::OpaqueType
            )
    }) {
        items.insert((item.package, item.name.clone(), item.kind), item.id);
    }
    for package in &graph.packages {
        for import in &package.imports {
            for item in graph.items.iter().filter(|item| {
                item.package == import.package
                    && item.visibility == Visibility::Public
                    && matches!(
                        item.kind,
                        PackageItemKind::Record
                            | PackageItemKind::Enum
                            | PackageItemKind::OpaqueType
                    )
            }) {
                items.insert(
                    (
                        package.id,
                        format!("{}::{}", import.alias, item.name),
                        item.kind,
                    ),
                    item.id,
                );
            }
        }
    }
    items
}

fn canonical_public_signature_type(
    ty: &TypeInfo,
    package: PackageId,
    symbols: &SymbolTable,
    public_type_items: &HashMap<PublicTypeItemKey, PackageItemId>,
) -> TypeInfo {
    match ty {
        TypeInfo::Record(symbol, args) => {
            let args = args
                .iter()
                .map(|arg| {
                    canonical_public_signature_type(arg, package, symbols, public_type_items)
                })
                .collect();
            if let Some(item) = public_type_items
                .get(&(
                    package,
                    symbols.resolve(*symbol).to_string(),
                    PackageItemKind::Record,
                ))
                .copied()
            {
                TypeInfo::PackageRecord {
                    symbol: *symbol,
                    item,
                    args,
                }
            } else {
                TypeInfo::Record(*symbol, args)
            }
        }
        TypeInfo::PackageRecord { symbol, item, args } => TypeInfo::PackageRecord {
            symbol: *symbol,
            item: *item,
            args: args
                .iter()
                .map(|arg| {
                    canonical_public_signature_type(arg, package, symbols, public_type_items)
                })
                .collect(),
        },
        TypeInfo::Enum { symbol, args } => {
            let args = args
                .iter()
                .map(|arg| {
                    canonical_public_signature_type(arg, package, symbols, public_type_items)
                })
                .collect();
            if let Some(item) = public_type_items
                .get(&(
                    package,
                    symbols.resolve(*symbol).to_string(),
                    PackageItemKind::Enum,
                ))
                .copied()
            {
                TypeInfo::PackageEnum {
                    symbol: *symbol,
                    item,
                    args,
                }
            } else {
                TypeInfo::Enum {
                    symbol: *symbol,
                    args,
                }
            }
        }
        TypeInfo::PackageEnum { symbol, item, args } => TypeInfo::PackageEnum {
            symbol: *symbol,
            item: *item,
            args: args
                .iter()
                .map(|arg| {
                    canonical_public_signature_type(arg, package, symbols, public_type_items)
                })
                .collect(),
        },
        TypeInfo::PackageOpaque { symbol, item } => TypeInfo::PackageOpaque {
            symbol: *symbol,
            item: *item,
        },
        TypeInfo::List(item) => TypeInfo::List(Box::new(canonical_public_signature_type(
            item,
            package,
            symbols,
            public_type_items,
        ))),
        TypeInfo::Map(key, value) => TypeInfo::Map(
            Box::new(canonical_public_signature_type(
                key,
                package,
                symbols,
                public_type_items,
            )),
            Box::new(canonical_public_signature_type(
                value,
                package,
                symbols,
                public_type_items,
            )),
        ),
        TypeInfo::Option(item) => TypeInfo::Option(Box::new(canonical_public_signature_type(
            item,
            package,
            symbols,
            public_type_items,
        ))),
        TypeInfo::Task(item) => TypeInfo::Task(Box::new(canonical_public_signature_type(
            item,
            package,
            symbols,
            public_type_items,
        ))),
        TypeInfo::Result(ok, err) => TypeInfo::Result(
            Box::new(canonical_public_signature_type(
                ok,
                package,
                symbols,
                public_type_items,
            )),
            Box::new(canonical_public_signature_type(
                err,
                package,
                symbols,
                public_type_items,
            )),
        ),
        TypeInfo::Function(function) => TypeInfo::Function(FunctionTypeInfo {
            params: function
                .params
                .iter()
                .map(|param| {
                    canonical_public_signature_type(param, package, symbols, public_type_items)
                })
                .collect(),
            ret: Box::new(canonical_public_signature_type(
                &function.ret,
                package,
                symbols,
                public_type_items,
            )),
        }),
        TypeInfo::GenericParam(_)
        | TypeInfo::EnumConstructor { .. }
        | TypeInfo::Int
        | TypeInfo::Bool
        | TypeInfo::String
        | TypeInfo::Unit
        | TypeInfo::Builtin(_)
        | TypeInfo::Unknown
        | TypeInfo::Error => ty.clone(),
    }
}

struct PackageInterfaceReferenceValidator<'a> {
    program: &'a Program,
    interfaces: &'a PackageInterfaceGraph,
    checked_items: HashSet<PackageItemId>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> PackageInterfaceReferenceValidator<'a> {
    fn validate(&mut self) {
        for binding in &self.program.bindings {
            self.validate_type(&binding.ty, binding.span);
        }
        for statement in &self.program.statements {
            self.validate_stmt(statement);
        }
    }

    fn validate_stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Assign(stmt) => self.validate_expr(&stmt.value),
            Stmt::Record(record) => {
                for field in &record.fields {
                    self.validate_type(&field.ty, field.span);
                }
            }
            Stmt::Enum(enumeration) => {
                for variant in &enumeration.variants {
                    if let Some(payload) = &variant.payload {
                        self.validate_type(payload, variant.span);
                    }
                }
            }
            Stmt::OpaqueType(_) => {}
            Stmt::Function(function) => {
                for param in &function.params {
                    self.validate_type(&param.ty, param.span);
                }
                self.validate_type(&function.return_ty, function.span);
                self.validate_value_block(&function.body);
            }
            Stmt::If(stmt) => {
                self.validate_expr(&stmt.condition);
                self.validate_block(&stmt.then_branch);
                if let Some(else_branch) = &stmt.else_branch {
                    self.validate_block(else_branch);
                }
            }
            Stmt::While(stmt) => {
                self.validate_expr(&stmt.condition);
                self.validate_block(&stmt.body);
            }
            Stmt::For(stmt) => {
                self.validate_expr(&stmt.iterable);
                self.validate_block(&stmt.body);
            }
            Stmt::Using(stmt) => {
                self.validate_expr(&stmt.value);
                self.validate_block(&stmt.body);
            }
            Stmt::Break(_) | Stmt::Continue(_) => {}
            Stmt::Return(stmt) => self.validate_expr(&stmt.value),
            Stmt::Expr(stmt) => self.validate_expr(&stmt.expr),
        }
    }

    fn validate_block(&mut self, block: &Block) {
        for statement in &block.statements {
            self.validate_stmt(statement);
        }
    }

    fn validate_value_block(&mut self, block: &ValueBlock) {
        for statement in &block.statements {
            self.validate_stmt(statement);
        }
        self.validate_expr(&block.expr);
    }

    fn validate_expr(&mut self, expr: &Expr) {
        self.validate_type(&expr.ty, expr.span);
        match &expr.kind {
            ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::String(_) | ExprKind::Unit => {}
            ExprKind::Ident(ident) => {
                if let IdentTarget::PackageItem { item, .. } = ident.target {
                    self.validate_item(item, expr.span);
                }
            }
            ExprKind::ListLit(list) => {
                for item in &list.items {
                    self.validate_expr(item);
                }
            }
            ExprKind::RecordLit(record) => {
                for field in &record.fields {
                    self.validate_expr(&field.value);
                }
            }
            ExprKind::Field(field) => self.validate_expr(&field.base),
            ExprKind::RecordUpdate(update) => {
                self.validate_expr(&update.base);
                for field in &update.fields {
                    self.validate_expr(&field.value);
                }
            }
            ExprKind::Index(index) => {
                self.validate_expr(&index.base);
                self.validate_expr(&index.index);
            }
            ExprKind::Unary(unary) => self.validate_expr(&unary.expr),
            ExprKind::Binary(binary) => {
                self.validate_expr(&binary.left);
                self.validate_expr(&binary.right);
            }
            ExprKind::Call(call) => {
                self.validate_expr(&call.callee);
                for arg in &call.args {
                    self.validate_expr(arg);
                }
            }
            ExprKind::Try(try_expr) => self.validate_expr(&try_expr.expr),
            ExprKind::If(if_expr) => {
                self.validate_expr(&if_expr.condition);
                self.validate_value_block(&if_expr.then_branch);
                self.validate_value_block(&if_expr.else_branch);
            }
            ExprKind::Match(match_expr) => {
                self.validate_expr(&match_expr.value);
                for arm in &match_expr.arms {
                    self.validate_expr(&arm.value);
                }
            }
            ExprKind::Fn(fn_expr) => {
                for param in &fn_expr.params {
                    self.validate_type(&param.ty, param.span);
                }
                self.validate_type(&fn_expr.return_ty, expr.span);
                self.validate_value_block(&fn_expr.body);
            }
            ExprKind::Group(group_expr) => self.validate_value_block(&group_expr.body),
            ExprKind::Spawn(spawn_expr) => self.validate_expr(&spawn_expr.expr),
        }
    }

    fn validate_type(&mut self, ty: &TypeInfo, span: Span) {
        match ty {
            TypeInfo::PackageRecord { item, .. } => self.validate_item(*item, span),
            TypeInfo::PackageEnum { item, args, .. } => {
                self.validate_item(*item, span);
                for arg in args {
                    self.validate_type(arg, span);
                }
            }
            TypeInfo::PackageOpaque { item, .. } => self.validate_item(*item, span),
            TypeInfo::Enum { args, .. } => {
                for arg in args {
                    self.validate_type(arg, span);
                }
            }
            TypeInfo::List(item) => self.validate_type(item, span),
            TypeInfo::Map(key, value) => {
                self.validate_type(key, span);
                self.validate_type(value, span);
            }
            TypeInfo::Option(item) => self.validate_type(item, span),
            TypeInfo::Result(ok, err) => {
                self.validate_type(ok, span);
                self.validate_type(err, span);
            }
            TypeInfo::Task(item) => self.validate_type(item, span),
            TypeInfo::Function(function) => {
                for param in &function.params {
                    self.validate_type(param, span);
                }
                self.validate_type(&function.ret, span);
            }
            _ => {}
        }
    }

    fn validate_item(&mut self, item: PackageItemId, span: Span) {
        if !self.checked_items.insert(item) {
            return;
        }

        let Some(info) = self.program.package_graph.item(item).cloned() else {
            self.diagnostics.push(Diagnostic::new(
                "PK016",
                format!(
                    "package interface reference points at unknown item {:?}",
                    item
                ),
                span,
            ));
            return;
        };
        if info.visibility != Visibility::Public {
            return;
        }

        match info.kind {
            PackageItemKind::Record => {
                let Some(interface) = self.interfaces.record_by_name(info.package, &info.name)
                else {
                    self.push_missing_interface_export(&info, "record", span);
                    return;
                };
                if interface.item != item {
                    self.push_stale_interface_diagnostic(&info, "record identity", span);
                    return;
                }
                self.validate_record_shape(&info, interface, span);
            }
            PackageItemKind::Enum => {
                let Some(interface) = self.interfaces.enum_by_name(info.package, &info.name) else {
                    self.push_missing_interface_export(&info, "enum", span);
                    return;
                };
                if interface.item != item {
                    self.push_stale_interface_diagnostic(&info, "enum identity", span);
                    return;
                }
                self.validate_enum_shape(&info, interface, span);
            }
            PackageItemKind::OpaqueType => {
                let Some(interface) = self
                    .interfaces
                    .opaque_type_by_name(info.package, &info.name)
                else {
                    self.push_missing_interface_export(&info, "opaque type", span);
                    return;
                };
                if interface.item != item {
                    self.push_stale_interface_diagnostic(&info, "opaque type identity", span);
                }
            }
            PackageItemKind::Function => {
                let Some(interface) = self.interfaces.function_by_name(info.package, &info.name)
                else {
                    self.push_missing_interface_export(&info, "function", span);
                    return;
                };
                if interface.item != item {
                    self.push_stale_interface_diagnostic(&info, "function identity", span);
                    return;
                }
                self.validate_function_signature(&info, interface, span);
            }
        }
    }

    fn validate_record_shape(
        &mut self,
        info: &PackageItemInfo,
        interface: &PackageInterfaceRecord,
        span: Span,
    ) {
        let Some(record) = self.record_stmt_for(info) else {
            self.push_stale_interface_diagnostic(info, "record shape", span);
            return;
        };
        let matches = record.fields.len() == interface.fields.len()
            && record.type_params == interface.type_params
            && record.json_deny_unknown_fields == interface.json_deny_unknown_fields
            && record.cli_about == interface.cli_about
            && record
                .fields
                .iter()
                .zip(interface.fields.iter())
                .all(|(field, expected)| {
                    field.name == expected.name
                        && field.json_rename == expected.json_rename
                        && field.json_aliases == expected.json_aliases
                        && field.json_validation == expected.json_validation
                        && field.cli_name == expected.cli_name
                        && field.cli_short == expected.cli_short
                        && field.cli_position == expected.cli_position
                        && field.cli_value_source == expected.cli_value_source
                        && field.cli_aliases == expected.cli_aliases
                        && field.cli_help == expected.cli_help
                        && field.cli_hidden == expected.cli_hidden
                        && field.cli_subcommand == expected.cli_subcommand
                        && field.ty == expected.ty
                });
        if !matches {
            self.push_stale_interface_diagnostic(info, "record shape", span);
        }
    }

    fn validate_enum_shape(
        &mut self,
        info: &PackageItemInfo,
        interface: &PackageInterfaceEnum,
        span: Span,
    ) {
        let Some(enumeration) = self.enum_stmt_for(info) else {
            self.push_stale_interface_diagnostic(info, "enum shape", span);
            return;
        };
        let matches = enumeration.type_params == interface.type_params
            && enumeration.cli_about == interface.cli_about
            && enumeration.variants.len() == interface.variants.len()
            && enumeration
                .variants
                .iter()
                .zip(interface.variants.iter())
                .all(|(variant, expected)| {
                    variant.name == expected.name
                        && variant.json_rename == expected.json_rename
                        && variant.json_aliases == expected.json_aliases
                        && variant.cli_name == expected.cli_name
                        && variant.cli_aliases == expected.cli_aliases
                        && variant.cli_about == expected.cli_about
                        && variant.cli_hidden == expected.cli_hidden
                        && variant.payload == expected.payload
                });
        if !matches {
            self.push_stale_interface_diagnostic(info, "enum shape", span);
        }
    }

    fn validate_function_signature(
        &mut self,
        info: &PackageItemInfo,
        interface: &PackageInterfaceFunction,
        span: Span,
    ) {
        let Some(function) = self.function_stmt_for(info) else {
            self.push_stale_interface_diagnostic(info, "function signature", span);
            return;
        };
        let matches = function.params.len() == interface.params.len()
            && function.type_params == interface.type_params
            && function
                .params
                .iter()
                .zip(interface.params.iter())
                .all(|(param, expected)| {
                    param.name == expected.name
                        && param.ty == expected.ty
                        && expected.mode == PackageInterfaceParamMode::Borrow
                })
            && function.return_ty == interface.ret;
        if !matches {
            self.push_stale_interface_diagnostic(info, "function signature", span);
        }
    }

    fn record_stmt_for(&self, info: &PackageItemInfo) -> Option<&RecordStmt> {
        self.program
            .statements
            .iter()
            .find_map(|statement| match statement {
                Stmt::Record(record) if record.package_item == Some(info.id) => Some(record),
                _ => None,
            })
    }

    fn enum_stmt_for(&self, info: &PackageItemInfo) -> Option<&EnumStmt> {
        self.program
            .statements
            .iter()
            .find_map(|statement| match statement {
                Stmt::Enum(enumeration) if enumeration.package_item == Some(info.id) => {
                    Some(enumeration)
                }
                _ => None,
            })
    }

    fn function_stmt_for(&self, info: &PackageItemInfo) -> Option<&FunctionStmt> {
        self.program
            .statements
            .iter()
            .find_map(|statement| match statement {
                Stmt::Function(function) if function.package_item == Some(info.id) => {
                    Some(function)
                }
                _ => None,
            })
    }

    fn push_missing_interface_export(&mut self, info: &PackageItemInfo, kind: &str, span: Span) {
        self.diagnostics.push(
            Diagnostic::new(
                "PK016",
                format!(
                    "package interface for `{}` does not export {kind} `{}`",
                    self.package_path(info),
                    info.name
                ),
                span,
            )
            .with_related("package item is declared here", info.span),
        );
    }

    fn push_stale_interface_diagnostic(
        &mut self,
        info: &PackageItemInfo,
        interface_part: &str,
        span: Span,
    ) {
        self.diagnostics.push(
            Diagnostic::new(
                "PK017",
                format!(
                    "package interface for `{}` has stale {interface_part} for `{}`",
                    self.package_path(info),
                    info.name
                ),
                span,
            )
            .with_related("package item is declared here", info.span)
            .with_suggestion(regenerate_interface_artifact_suggestion()),
        );
    }

    fn package_path(&self, info: &PackageItemInfo) -> &str {
        self.program
            .package_graph
            .package(info.package)
            .map(|package| package.path.as_str())
            .unwrap_or("<unknown>")
    }
}
