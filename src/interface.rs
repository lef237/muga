use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};

use crate::{
    ast::Visibility,
    diagnostic::Diagnostic,
    identity::{PackageId, PackageItemId},
    package::{PackageItemInfo, PackageItemKind, PackageSymbolGraph},
    prelude, span,
    span::Span,
    symbol::SymbolTable,
    typed_hir::{
        Block, EnumStmt, Expr, ExprKind, FunctionStmt, IdentTarget, Program, RecordStmt, Stmt,
        ValueBlock,
    },
    types::{FunctionTypeInfo, TypeInfo},
};

const PERSISTED_INTERFACE_HEADER: &str = "muga-package-interface-v1";
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
                        PackageItemKind::Function => functions.push(export),
                    }
                }
                PackageExports {
                    package: package.id,
                    path: package.path.clone(),
                    records,
                    enums,
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

                PackageExports {
                    package: interface.package,
                    path: interface.path.clone(),
                    records,
                    enums,
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

impl PackageInterfaceGraph {
    pub fn to_persisted_text(&self, symbols: &SymbolTable) -> String {
        let body = self.persisted_body_text(symbols);
        format!(
            "{PERSISTED_INTERFACE_HEADER}\nhash\t{}\n{body}",
            stable_hash_hex(&body)
        )
    }

    pub fn stable_hash(&self, symbols: &SymbolTable) -> String {
        stable_hash_hex(&self.persisted_body_text(symbols))
    }

    pub fn stable_hash_for_package(
        &self,
        package_path: &str,
        symbols: &SymbolTable,
    ) -> Option<String> {
        let package = self.package_by_path(package_path)?.clone();
        Some(
            Self {
                packages: vec![package],
            }
            .stable_hash(symbols),
        )
    }

    fn persisted_body_text(&self, symbols: &SymbolTable) -> String {
        let mut out = String::new();
        for package in &self.packages {
            push_line(
                &mut out,
                &[
                    "package".to_string(),
                    package.package.as_u32().to_string(),
                    package.path.clone(),
                    package.dependencies.len().to_string(),
                    package.records.len().to_string(),
                    package.enums.len().to_string(),
                    package.functions.len().to_string(),
                ],
            );
            for dependency in &package.dependencies {
                push_line(&mut out, &["dependency".to_string(), dependency.clone()]);
            }
            for record in &package.records {
                push_line(
                    &mut out,
                    &[
                        "record".to_string(),
                        record.item.as_u32().to_string(),
                        record.name.clone(),
                        format_span(record.span),
                        record.fields.len().to_string(),
                    ],
                );
                for field in &record.fields {
                    push_line(
                        &mut out,
                        &[
                            "field".to_string(),
                            field.name.clone(),
                            format_span(field.span),
                            format_type_info(&field.ty, symbols),
                        ],
                    );
                }
            }
            for enumeration in &package.enums {
                let mut parts = vec![
                    "enum".to_string(),
                    enumeration.item.as_u32().to_string(),
                    enumeration.name.clone(),
                    format_span(enumeration.span),
                    enumeration.type_params.len().to_string(),
                ];
                parts.extend(enumeration.type_params.iter().cloned());
                parts.push(enumeration.variants.len().to_string());
                push_line(&mut out, &parts);
                for variant in &enumeration.variants {
                    push_line(
                        &mut out,
                        &[
                            "variant".to_string(),
                            variant.name.clone(),
                            format_span(variant.span),
                            match &variant.payload {
                                Some(payload) => format_type_info(payload, symbols),
                                None => "-".to_string(),
                            },
                        ],
                    );
                }
            }
            for function in &package.functions {
                push_line(
                    &mut out,
                    &[
                        "function".to_string(),
                        function.item.as_u32().to_string(),
                        function.name.clone(),
                        format_span(function.span),
                        function.params.len().to_string(),
                        format_type_info(&function.ret, symbols),
                    ],
                );
                for param in &function.params {
                    push_line(
                        &mut out,
                        &[
                            "param".to_string(),
                            param.name.clone(),
                            format_span(param.span),
                            format_type_info(&param.ty, symbols),
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
        let Some(graph) = self.package_graph_by_path(package_path) else {
            return Err(Diagnostic::new(
                "PK016",
                format!("compiled package interfaces do not contain `{package_path}`"),
                Span::default(),
            )
            .with_suggestion("choose a package that is reachable from the entrypoint"));
        };
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
        graph.write_persisted_file(&path, symbols)?;
        Ok(path)
    }

    pub fn read_persisted_file(
        path: &Path,
        symbols: &mut SymbolTable,
    ) -> Result<Self, Vec<Diagnostic>> {
        let text = fs::read_to_string(path).map_err(|error| {
            vec![Diagnostic::new(
                "PK018",
                format!(
                    "failed to read package interface `{}`: {error}",
                    path.display()
                ),
                Span::default(),
            )]
        })?;
        Self::from_persisted_text(&text, symbols)
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
                    Diagnostic::new(
                        "PK016",
                        format!(
                            "missing package interface artifact `{}` for `{package_path}`",
                            artifact_path.display()
                        ),
                        Span::default(),
                    )
                    .with_suggestion("regenerate the package interface artifact"),
                );
                continue;
            }

            let graph = match Self::read_persisted_file(&artifact_path, symbols) {
                Ok(graph) => graph,
                Err(mut errors) => {
                    diagnostics.append(&mut errors);
                    continue;
                }
            };
            if graph.package_by_path(&package_path).is_none() {
                diagnostics.push(
                    Diagnostic::new(
                        "PK016",
                        format!(
                            "package interface artifact `{}` does not contain `{package_path}`",
                            artifact_path.display()
                        ),
                        Span::default(),
                    )
                    .with_suggestion("regenerate the package interface artifact"),
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
            remap_persisted_artifact_ids(packages, symbols)
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
                            fields: record
                                .fields
                                .iter()
                                .map(|field| PackageInterfaceField {
                                    name: field.name.clone(),
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
                            type_params: enumeration.type_params.clone(),
                            variants: enumeration
                                .variants
                                .iter()
                                .map(|variant| PackageInterfaceEnumVariant {
                                    name: variant.name.clone(),
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
                    functions: package
                        .functions
                        .iter()
                        .map(|function| PackageInterfaceFunction {
                            item: function.item,
                            name: function.name.clone(),
                            params: function
                                .params
                                .iter()
                                .map(|param| PackageInterfaceParam {
                                    name: param.name.clone(),
                                    ty: reintern_type_info(&param.ty, from, to),
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
                    .with_suggestion("regenerate the package interface artifact"),
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
            TypeInfo::PackageRecord { symbol, item } => {
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
            TypeInfo::List(item) | TypeInfo::Option(item) => {
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
            TypeInfo::GenericParam(_)
            | TypeInfo::Record(_)
            | TypeInfo::EnumConstructor {
                enum_item: None, ..
            }
            | TypeInfo::Int
            | TypeInfo::Bool
            | TypeInfo::String
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
                    .with_suggestion("regenerate the package interface artifacts together"),
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
            .with_suggestion("regenerate the package interface artifacts together"),
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

fn package_item_kind_label(kind: PackageItemKind) -> &'static str {
    match kind {
        PackageItemKind::Record => "record",
        PackageItemKind::Enum => "enum",
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
    pub functions: Vec<PackageInterfaceFunction>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterfaceRecord {
    pub item: PackageItemId,
    pub name: String,
    pub fields: Vec<PackageInterfaceField>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterfaceField {
    pub name: String,
    pub ty: TypeInfo,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterfaceEnum {
    pub item: PackageItemId,
    pub name: String,
    pub type_params: Vec<String>,
    pub variants: Vec<PackageInterfaceEnumVariant>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterfaceEnumVariant {
    pub name: String,
    pub payload: Option<TypeInfo>,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterfaceFunction {
    pub item: PackageItemId,
    pub name: String,
    pub params: Vec<PackageInterfaceParam>,
    pub ret: TypeInfo,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterfaceParam {
    pub name: String,
    pub ty: TypeInfo,
    pub span: Span,
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
            Some(parts) if parts == [PERSISTED_INTERFACE_HEADER] => {}
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
        let actual = stable_hash_hex(&body);
        if expected != actual {
            self.diagnostics.push(
                Diagnostic::new(
                    "PK019",
                    format!(
                        "package interface hash mismatch: expected `{expected}` but found `{actual}`"
                    ),
                    Span::default(),
                )
                .with_suggestion("regenerate the package interface"),
            );
        }
    }

    fn parse_package(&mut self, parts: Vec<&str>) -> Option<PackageInterface> {
        if parts.len() != 6 && parts.len() != 7 {
            self.push_error("invalid package line");
            return None;
        }
        let package = PackageId::new(self.parse_u32(parts[1], "package id")?);
        let path = parts[2].to_string();
        let (dependency_count, record_count, enum_count, function_count) = if parts.len() == 7 {
            (
                self.parse_usize(parts[3], "dependency count")?,
                self.parse_usize(parts[4], "record count")?,
                self.parse_usize(parts[5], "enum count")?,
                self.parse_usize(parts[6], "function count")?,
            )
        } else {
            (
                0,
                self.parse_usize(parts[3], "record count")?,
                self.parse_usize(parts[4], "enum count")?,
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
            functions,
        })
    }

    fn parse_record(&mut self) -> Option<PackageInterfaceRecord> {
        let parts = self.expect_line("record")?;
        if parts.len() != 5 {
            self.push_error("invalid record line");
            return None;
        }
        let item = PackageItemId::new(self.parse_u32(parts[1], "record item id")?);
        let name = parts[2].to_string();
        let span = self.parse_span(parts[3])?;
        let field_count = self.parse_usize(parts[4], "field count")?;
        let mut fields = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            let field = self.expect_line("field")?;
            if field.len() != 4 {
                self.push_error("invalid field line");
                return None;
            }
            fields.push(PackageInterfaceField {
                name: field[1].to_string(),
                span: self.parse_span(field[2])?,
                ty: self.parse_type(field[3])?,
            });
        }
        Some(PackageInterfaceRecord {
            item,
            name,
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
        if parts.len() != variant_count_index + 1 {
            self.push_error("invalid enum type parameter list");
            return None;
        }
        let type_params = parts[5..variant_count_index]
            .iter()
            .map(|part| (*part).to_string())
            .collect::<Vec<_>>();
        let variant_count = self.parse_usize(parts[variant_count_index], "enum variant count")?;
        let mut variants = Vec::with_capacity(variant_count);
        for _ in 0..variant_count {
            let variant = self.expect_line("variant")?;
            if variant.len() != 4 {
                self.push_error("invalid enum variant line");
                return None;
            }
            variants.push(PackageInterfaceEnumVariant {
                name: variant[1].to_string(),
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
            type_params,
            variants,
            span,
        })
    }

    fn parse_function(&mut self) -> Option<PackageInterfaceFunction> {
        let parts = self.expect_line("function")?;
        if parts.len() != 6 {
            self.push_error("invalid function line");
            return None;
        }
        let item = PackageItemId::new(self.parse_u32(parts[1], "function item id")?);
        let name = parts[2].to_string();
        let span = self.parse_span(parts[3])?;
        let param_count = self.parse_usize(parts[4], "function parameter count")?;
        let ret = self.parse_type(parts[5])?;
        let mut params = Vec::with_capacity(param_count);
        for _ in 0..param_count {
            let param = self.expect_line("param")?;
            if param.len() != 4 {
                self.push_error("invalid function parameter line");
                return None;
            }
            params.push(PackageInterfaceParam {
                name: param[1].to_string(),
                span: self.parse_span(param[2])?,
                ty: self.parse_type(param[3])?,
            });
        }
        Some(PackageInterfaceFunction {
            item,
            name,
            params,
            ret,
            span,
        })
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
            "GenericParam" => Ok(TypeInfo::GenericParam(self.symbol()?)),
            "Record" => Ok(TypeInfo::Record(self.symbol()?)),
            "PackageRecord" => Ok(TypeInfo::PackageRecord {
                symbol: self.symbol()?,
                item: PackageItemId::new(self.u32("package record item id")?),
            }),
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
                prelude::builtin_by_name(name)
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

fn format_span(span: Span) -> String {
    format!(
        "{}:{}-{}:{}",
        span.start.line, span.start.column, span.end.line, span.end.column
    )
}

fn format_type_info(ty: &TypeInfo, symbols: &SymbolTable) -> String {
    let mut tokens = Vec::new();
    push_type_info_tokens(ty, symbols, &mut tokens);
    tokens.join(" ")
}

fn push_type_info_tokens(ty: &TypeInfo, symbols: &SymbolTable, tokens: &mut Vec<String>) {
    match ty {
        TypeInfo::Int => tokens.push("Int".to_string()),
        TypeInfo::Bool => tokens.push("Bool".to_string()),
        TypeInfo::String => tokens.push("String".to_string()),
        TypeInfo::GenericParam(symbol) => {
            tokens.push("GenericParam".to_string());
            tokens.push(symbols.resolve(*symbol).to_string());
        }
        TypeInfo::Record(symbol) => {
            tokens.push("Record".to_string());
            tokens.push(symbols.resolve(*symbol).to_string());
        }
        TypeInfo::PackageRecord { symbol, item } => {
            tokens.push("PackageRecord".to_string());
            tokens.push(symbols.resolve(*symbol).to_string());
            tokens.push(item.as_u32().to_string());
        }
        TypeInfo::Enum { symbol, args } => {
            tokens.push("Enum".to_string());
            tokens.push(symbols.resolve(*symbol).to_string());
            push_type_args(args, symbols, tokens);
        }
        TypeInfo::PackageEnum { symbol, item, args } => {
            tokens.push("PackageEnum".to_string());
            tokens.push(symbols.resolve(*symbol).to_string());
            tokens.push(item.as_u32().to_string());
            push_type_args(args, symbols, tokens);
        }
        TypeInfo::List(item) => {
            tokens.push("List".to_string());
            push_type_info_tokens(item, symbols, tokens);
        }
        TypeInfo::Map(key, value) => {
            tokens.push("Map".to_string());
            push_type_info_tokens(key, symbols, tokens);
            push_type_info_tokens(value, symbols, tokens);
        }
        TypeInfo::Option(item) => {
            tokens.push("Option".to_string());
            push_type_info_tokens(item, symbols, tokens);
        }
        TypeInfo::Result(ok, err) => {
            tokens.push("Result".to_string());
            push_type_info_tokens(ok, symbols, tokens);
            push_type_info_tokens(err, symbols, tokens);
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
                    .map(|item| item.as_u32().to_string())
                    .unwrap_or_else(|| "-".to_string()),
            );
            tokens.push(symbols.resolve(*variant).to_string());
        }
        TypeInfo::Function(function) => {
            tokens.push("Function".to_string());
            tokens.push(function.params.len().to_string());
            for param in &function.params {
                push_type_info_tokens(param, symbols, tokens);
            }
            push_type_info_tokens(&function.ret, symbols, tokens);
        }
        TypeInfo::Builtin(builtin) => {
            tokens.push("Builtin".to_string());
            tokens.push(prelude::builtin_name(*builtin).to_string());
        }
        TypeInfo::Unknown => tokens.push("Unknown".to_string()),
        TypeInfo::Error => tokens.push("Error".to_string()),
    }
}

fn push_type_args(args: &[TypeInfo], symbols: &SymbolTable, tokens: &mut Vec<String>) {
    tokens.push(args.len().to_string());
    for arg in args {
        push_type_info_tokens(arg, symbols, tokens);
    }
}

fn reintern_type_info(ty: &TypeInfo, from: &SymbolTable, to: &mut SymbolTable) -> TypeInfo {
    match ty {
        TypeInfo::GenericParam(symbol) => {
            TypeInfo::GenericParam(reintern_symbol(*symbol, from, to))
        }
        TypeInfo::Record(symbol) => TypeInfo::Record(reintern_symbol(*symbol, from, to)),
        TypeInfo::PackageRecord { symbol, item } => TypeInfo::PackageRecord {
            symbol: reintern_symbol(*symbol, from, to),
            item: *item,
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
                let mut functions = Vec::new();

                for item in self.package_graph.items.iter().filter(|item| {
                    item.package == package.id && item.visibility == Visibility::Public
                }) {
                    match item.kind {
                        PackageItemKind::Record => {
                            if let Some(record) = records_by_item.get(&item.id) {
                                records.push(PackageInterfaceRecord {
                                    item: item.id,
                                    name: item.name.clone(),
                                    fields: record
                                        .fields
                                        .iter()
                                        .map(|field| PackageInterfaceField {
                                            name: field.name.clone(),
                                            ty: field.ty.clone(),
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
                                    type_params: enumeration.type_params.clone(),
                                    variants: enumeration
                                        .variants
                                        .iter()
                                        .map(|variant| PackageInterfaceEnumVariant {
                                            name: variant.name.clone(),
                                            payload: variant.payload.clone(),
                                            span: variant.span,
                                        })
                                        .collect(),
                                    span: item.span,
                                });
                            }
                        }
                        PackageItemKind::Function => {
                            if let Some(function) = functions_by_item.get(&item.id) {
                                functions.push(PackageInterfaceFunction {
                                    item: item.id,
                                    name: item.name.clone(),
                                    params: function
                                        .params
                                        .iter()
                                        .map(|param| PackageInterfaceParam {
                                            name: param.name.clone(),
                                            ty: param.ty.clone(),
                                            span: param.span,
                                        })
                                        .collect(),
                                    ret: function.return_ty.clone(),
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
            ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::String(_) => {}
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
            && record
                .fields
                .iter()
                .zip(interface.fields.iter())
                .all(|(field, expected)| field.name == expected.name && field.ty == expected.ty);
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
            && enumeration.variants.len() == interface.variants.len()
            && enumeration
                .variants
                .iter()
                .zip(interface.variants.iter())
                .all(|(variant, expected)| {
                    variant.name == expected.name && variant.payload == expected.payload
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
            && function
                .params
                .iter()
                .zip(interface.params.iter())
                .all(|(param, expected)| param.name == expected.name && param.ty == expected.ty)
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
            .with_suggestion("regenerate the package interface"),
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
