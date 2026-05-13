use std::collections::HashMap;

use crate::{
    ast::{EnumDecl, FuncDecl, ImportDecl, RecordDecl, TypeExpr, Visibility},
    diagnostic::Diagnostic,
    identity::{ModuleId, PackageId, PackageItemId},
    known_enum,
    package::{LoadedPackageGraph, PackageItemInfo, PackageItemKind},
    span::Span,
    symbol::{Symbol, SymbolTable},
    types::{FunctionTypeInfo, TypeInfo},
};

#[derive(Clone, Debug)]
pub struct PackageSignatureEnvironment {
    pub symbols: SymbolTable,
    pub records: Vec<PackageRecordSignature>,
    pub enums: Vec<PackageEnumSignature>,
    pub functions: Vec<PackageFunctionSignature>,
    pub modules: Vec<PackageModuleSignatureEnvironment>,
}

impl PackageSignatureEnvironment {
    pub fn from_loaded_graph(loaded: &LoadedPackageGraph) -> Result<Self, Vec<Diagnostic>> {
        let mut collector = SignatureCollector::new(loaded);
        collector.collect();
        collector.finish()
    }

    pub fn record(&self, item: PackageItemId) -> Option<&PackageRecordSignature> {
        self.records.iter().find(|record| record.item == item)
    }

    pub fn enumeration(&self, item: PackageItemId) -> Option<&PackageEnumSignature> {
        self.enums
            .iter()
            .find(|enumeration| enumeration.item == item)
    }

    pub fn function(&self, item: PackageItemId) -> Option<&PackageFunctionSignature> {
        self.functions.iter().find(|function| function.item == item)
    }

    pub fn module(&self, module: ModuleId) -> Option<&PackageModuleSignatureEnvironment> {
        self.modules
            .iter()
            .find(|environment| environment.module == module)
    }
}

#[derive(Clone, Debug)]
pub struct PackageRecordSignature {
    pub item: PackageItemId,
    pub package: PackageId,
    pub module: ModuleId,
    pub name: String,
    pub visibility: Visibility,
    pub fields: Vec<PackageFieldSignature>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct PackageFieldSignature {
    pub name: String,
    pub ty: TypeInfo,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct PackageEnumSignature {
    pub item: PackageItemId,
    pub package: PackageId,
    pub module: ModuleId,
    pub name: String,
    pub visibility: Visibility,
    pub type_params: Vec<String>,
    pub variants: Vec<PackageEnumVariantSignature>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct PackageEnumVariantSignature {
    pub name: String,
    pub payload: Option<TypeInfo>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct PackageFunctionSignature {
    pub item: PackageItemId,
    pub package: PackageId,
    pub module: ModuleId,
    pub name: String,
    pub visibility: Visibility,
    pub params: Vec<PackageParamSignature>,
    pub ret: Option<TypeInfo>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct PackageParamSignature {
    pub name: String,
    pub ty: Option<TypeInfo>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct PackageModuleSignatureEnvironment {
    pub package: PackageId,
    pub module: ModuleId,
    pub module_path: String,
    pub records: Vec<PackageVisibleSignature>,
    pub enums: Vec<PackageVisibleSignature>,
    pub functions: Vec<PackageVisibleSignature>,
}

impl PackageModuleSignatureEnvironment {
    pub fn record(&self, name: &str) -> Option<&PackageVisibleSignature> {
        self.records.iter().find(|record| record.name == name)
    }

    pub fn record_signature<'a>(
        &self,
        signatures: &'a PackageSignatureEnvironment,
        name: &str,
    ) -> Option<&'a PackageRecordSignature> {
        signatures.record(self.record(name)?.item)
    }

    pub fn enumeration(&self, name: &str) -> Option<&PackageVisibleSignature> {
        self.enums
            .iter()
            .find(|enumeration| enumeration.name == name)
    }

    pub fn enum_signature<'a>(
        &self,
        signatures: &'a PackageSignatureEnvironment,
        name: &str,
    ) -> Option<&'a PackageEnumSignature> {
        signatures.enumeration(self.enumeration(name)?.item)
    }

    pub fn function(&self, name: &str) -> Option<&PackageVisibleSignature> {
        self.functions.iter().find(|function| function.name == name)
    }

    pub fn function_signature<'a>(
        &self,
        signatures: &'a PackageSignatureEnvironment,
        name: &str,
    ) -> Option<&'a PackageFunctionSignature> {
        signatures.function(self.function(name)?.item)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageVisibleSignature {
    pub name: String,
    pub item: PackageItemId,
    pub source: PackageSignatureSource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PackageSignatureSource {
    ModuleLocal,
    SamePackage,
    Imported { alias: String, package: PackageId },
}

struct SignatureCollector<'a> {
    loaded: &'a LoadedPackageGraph,
    symbols: SymbolTable,
    diagnostics: Vec<Diagnostic>,
    enum_type_params: HashMap<PackageItemId, Vec<String>>,
    imports: HashMap<String, String>,
    current_package: PackageId,
    current_module: ModuleId,
    records: Vec<PackageRecordSignature>,
    enums: Vec<PackageEnumSignature>,
    functions: Vec<PackageFunctionSignature>,
    modules: Vec<PackageModuleSignatureEnvironment>,
}

impl<'a> SignatureCollector<'a> {
    fn new(loaded: &'a LoadedPackageGraph) -> Self {
        Self {
            loaded,
            symbols: SymbolTable::default(),
            diagnostics: Vec::new(),
            enum_type_params: HashMap::new(),
            imports: HashMap::new(),
            current_package: PackageId::new(0),
            current_module: ModuleId::new(0),
            records: Vec::new(),
            enums: Vec::new(),
            functions: Vec::new(),
            modules: Vec::new(),
        }
    }

    fn collect(&mut self) {
        self.collect_enum_headers();
        for package in &self.loaded.packages {
            let Some(package_id) = self.loaded.package_graph.package_id(&package.path) else {
                continue;
            };
            for file in &package.files {
                let Some(module_id) = self
                    .loaded
                    .package_graph
                    .module_id(package_id, &file.module_path)
                else {
                    continue;
                };
                self.current_package = package_id;
                self.current_module = module_id;
                self.imports = file
                    .program
                    .imports
                    .iter()
                    .map(|import| (import.alias.clone(), import.path.clone()))
                    .collect();
                self.collect_module_environment(&file.module_path, &file.program.imports);

                for statement in &file.program.statements {
                    match statement {
                        crate::ast::Stmt::RecordDecl(record) => self.collect_record(record),
                        crate::ast::Stmt::EnumDecl(enumeration) => self.collect_enum(enumeration),
                        crate::ast::Stmt::FuncDecl(function) => self.collect_function(function),
                        _ => {}
                    }
                }
            }
        }
    }

    fn finish(self) -> Result<PackageSignatureEnvironment, Vec<Diagnostic>> {
        if self.diagnostics.is_empty() {
            Ok(PackageSignatureEnvironment {
                symbols: self.symbols,
                records: self.records,
                enums: self.enums,
                functions: self.functions,
                modules: self.modules,
            })
        } else {
            Err(self.diagnostics)
        }
    }

    fn collect_module_environment(&mut self, module_path: &str, imports: &[ImportDecl]) {
        let mut environment = PackageModuleSignatureEnvironment {
            package: self.current_package,
            module: self.current_module,
            module_path: module_path.to_string(),
            records: Vec::new(),
            enums: Vec::new(),
            functions: Vec::new(),
        };

        for item in &self.loaded.package_graph.items {
            let Some(source) = self.same_package_source(item) else {
                continue;
            };
            let visible = PackageVisibleSignature {
                name: item.name.clone(),
                item: item.id,
                source,
            };
            match item.kind {
                PackageItemKind::Record => environment.records.push(visible),
                PackageItemKind::Enum => environment.enums.push(visible),
                PackageItemKind::Function => environment.functions.push(visible),
            }
        }

        for import in imports {
            let Some(package_id) = self.loaded.package_graph.package_id(&import.path) else {
                continue;
            };
            let Some(exports) = self.loaded.package_exports.package(package_id) else {
                continue;
            };
            for record in &exports.records {
                environment.records.push(PackageVisibleSignature {
                    name: format!("{}::{}", import.alias, record.name),
                    item: record.item,
                    source: PackageSignatureSource::Imported {
                        alias: import.alias.clone(),
                        package: package_id,
                    },
                });
            }
            for enumeration in &exports.enums {
                environment.enums.push(PackageVisibleSignature {
                    name: format!("{}::{}", import.alias, enumeration.name),
                    item: enumeration.item,
                    source: PackageSignatureSource::Imported {
                        alias: import.alias.clone(),
                        package: package_id,
                    },
                });
            }
            for function in &exports.functions {
                environment.functions.push(PackageVisibleSignature {
                    name: format!("{}::{}", import.alias, function.name),
                    item: function.item,
                    source: PackageSignatureSource::Imported {
                        alias: import.alias.clone(),
                        package: package_id,
                    },
                });
            }
        }

        self.modules.push(environment);
    }

    fn same_package_source(&self, item: &PackageItemInfo) -> Option<PackageSignatureSource> {
        if item.package != self.current_package {
            return None;
        }
        if item.module == self.current_module {
            return Some(PackageSignatureSource::ModuleLocal);
        }
        if matches!(item.visibility, Visibility::Package | Visibility::Public) {
            return Some(PackageSignatureSource::SamePackage);
        }
        None
    }

    fn collect_enum_headers(&mut self) {
        let mut headers = Vec::new();
        for package in &self.loaded.packages {
            let Some(package_id) = self.loaded.package_graph.package_id(&package.path) else {
                continue;
            };
            for file in &package.files {
                let Some(module_id) = self
                    .loaded
                    .package_graph
                    .module_id(package_id, &file.module_path)
                else {
                    continue;
                };
                for statement in &file.program.statements {
                    if let crate::ast::Stmt::EnumDecl(enumeration) = statement
                        && let Some(item) =
                            self.item_id(module_id, &enumeration.name, PackageItemKind::Enum)
                    {
                        headers.push((item, enumeration.type_params.clone()));
                    }
                }
            }
        }
        self.enum_type_params.extend(headers);
    }

    fn collect_record(&mut self, record: &RecordDecl) {
        let Some(item) = self.item_id(self.current_module, &record.name, PackageItemKind::Record)
        else {
            return;
        };
        let fields = record
            .fields
            .iter()
            .map(|field| PackageFieldSignature {
                name: field.name.clone(),
                ty: self.type_info_from_type_expr(&field.type_name, field.span, &[]),
                span: field.span,
            })
            .collect();
        self.records.push(PackageRecordSignature {
            item,
            package: self.current_package,
            module: self.current_module,
            name: record.name.clone(),
            visibility: record.visibility,
            fields,
            span: record.span,
        });
    }

    fn collect_enum(&mut self, enumeration: &EnumDecl) {
        let Some(item) = self.item_id(
            self.current_module,
            &enumeration.name,
            PackageItemKind::Enum,
        ) else {
            return;
        };
        let variants = enumeration
            .variants
            .iter()
            .map(|variant| PackageEnumVariantSignature {
                name: variant.name.clone(),
                payload: variant.payload.as_ref().map(|payload| {
                    self.type_info_from_type_expr(payload, variant.span, &enumeration.type_params)
                }),
                span: variant.span,
            })
            .collect();
        self.enums.push(PackageEnumSignature {
            item,
            package: self.current_package,
            module: self.current_module,
            name: enumeration.name.clone(),
            visibility: enumeration.visibility,
            type_params: enumeration.type_params.clone(),
            variants,
            span: enumeration.span,
        });
    }

    fn collect_function(&mut self, function: &FuncDecl) {
        let Some(item) = self.item_id(
            self.current_module,
            &function.name,
            PackageItemKind::Function,
        ) else {
            return;
        };
        let params = function
            .params
            .iter()
            .map(|param| PackageParamSignature {
                name: param.name.clone(),
                ty: param
                    .type_name
                    .as_ref()
                    .map(|type_name| self.type_info_from_type_expr(type_name, param.span, &[])),
                span: param.span,
            })
            .collect();
        let ret = function
            .return_type
            .as_ref()
            .map(|type_name| self.type_info_from_type_expr(type_name, function.span, &[]));
        self.functions.push(PackageFunctionSignature {
            item,
            package: self.current_package,
            module: self.current_module,
            name: function.name.clone(),
            visibility: function.visibility,
            params,
            ret,
            span: function.span,
        });
    }

    fn type_info_from_type_expr(
        &mut self,
        type_expr: &TypeExpr,
        span: Span,
        type_params: &[String],
    ) -> TypeInfo {
        match type_expr {
            TypeExpr::Int => TypeInfo::Int,
            TypeExpr::Bool => TypeInfo::Bool,
            TypeExpr::String => TypeInfo::String,
            TypeExpr::Named(name) if type_params.iter().any(|param| param == name) => {
                let symbol = self.symbol(name);
                TypeInfo::GenericParam(symbol)
            }
            TypeExpr::Named(name) => self.named_type_info(name, Vec::new(), span),
            TypeExpr::Generic(generic) if generic.name == "List" && generic.args.len() == 1 => {
                TypeInfo::List(Box::new(self.type_info_from_type_expr(
                    &generic.args[0],
                    span,
                    type_params,
                )))
            }
            TypeExpr::Generic(generic)
                if generic.name == known_enum::OPTION_NAME && generic.args.len() == 1 =>
            {
                TypeInfo::Option(Box::new(self.type_info_from_type_expr(
                    &generic.args[0],
                    span,
                    type_params,
                )))
            }
            TypeExpr::Generic(generic)
                if generic.name == known_enum::RESULT_NAME && generic.args.len() == 2 =>
            {
                TypeInfo::Result(
                    Box::new(self.type_info_from_type_expr(&generic.args[0], span, type_params)),
                    Box::new(self.type_info_from_type_expr(&generic.args[1], span, type_params)),
                )
            }
            TypeExpr::Generic(generic) if generic.name == "Map" && generic.args.len() == 2 => {
                TypeInfo::Map(
                    Box::new(self.type_info_from_type_expr(&generic.args[0], span, type_params)),
                    Box::new(self.type_info_from_type_expr(&generic.args[1], span, type_params)),
                )
            }
            TypeExpr::Generic(generic) => {
                let args = generic
                    .args
                    .iter()
                    .map(|arg| self.type_info_from_type_expr(arg, span, type_params))
                    .collect();
                self.named_type_info(&generic.name, args, span)
            }
            TypeExpr::Function(function) => TypeInfo::Function(FunctionTypeInfo {
                params: function
                    .params
                    .iter()
                    .map(|param| self.type_info_from_type_expr(param, span, type_params))
                    .collect(),
                ret: Box::new(self.type_info_from_type_expr(&function.ret, span, type_params)),
            }),
        }
    }

    fn named_type_info(&mut self, name: &str, args: Vec<TypeInfo>, span: Span) -> TypeInfo {
        if let Some((alias, item_name)) = split_qualified_name(name) {
            return self.imported_type_info(alias, item_name, args, span);
        }
        if let Some(item) = self
            .visible_same_package_item(name, PackageItemKind::Record)
            .map(|item| item.id)
        {
            if !args.is_empty() {
                self.push_unsupported_generic_record(name, span);
                return TypeInfo::Error;
            }
            let symbol = self.symbol(name);
            return TypeInfo::PackageRecord { symbol, item };
        }
        if let Some(item) = self
            .visible_same_package_item(name, PackageItemKind::Enum)
            .map(|item| item.id)
        {
            if !self.validate_enum_arg_count(item, name, args.len(), span) {
                return TypeInfo::Error;
            }
            let symbol = self.symbol(name);
            return TypeInfo::PackageEnum { symbol, item, args };
        }
        self.diagnostics.push(Diagnostic::new(
            "T007",
            format!("unknown type `{name}`"),
            span,
        ));
        TypeInfo::Error
    }

    fn imported_type_info(
        &mut self,
        alias: &str,
        name: &str,
        args: Vec<TypeInfo>,
        span: Span,
    ) -> TypeInfo {
        let Some(package_path) = self.imports.get(alias) else {
            self.diagnostics.push(Diagnostic::new(
                "PK009",
                format!("unknown import alias `{alias}`"),
                span,
            ));
            return TypeInfo::Error;
        };
        let Some(package_id) = self.loaded.package_graph.package_id(package_path) else {
            self.diagnostics.push(Diagnostic::new(
                "PK010",
                format!("unknown imported package `{package_path}`"),
                span,
            ));
            return TypeInfo::Error;
        };
        if let Some(export) = self.loaded.package_exports.record_by_name(package_id, name) {
            if !args.is_empty() {
                self.push_unsupported_generic_record(name, span);
                return TypeInfo::Error;
            }
            let symbol = self.symbol(name);
            return TypeInfo::PackageRecord {
                symbol,
                item: export.item,
            };
        }
        if let Some(export) = self.loaded.package_exports.enum_by_name(package_id, name) {
            let item = export.item;
            if !self.validate_enum_arg_count(item, name, args.len(), span) {
                return TypeInfo::Error;
            }
            let symbol = self.symbol(name);
            return TypeInfo::PackageEnum { symbol, item, args };
        }
        self.diagnostics.push(Diagnostic::new(
            "PK010",
            format!("package `{package_path}` does not export type `{name}`"),
            span,
        ));
        TypeInfo::Error
    }

    fn visible_same_package_item(
        &self,
        name: &str,
        kind: PackageItemKind,
    ) -> Option<&PackageItemInfo> {
        self.loaded
            .package_graph
            .items
            .iter()
            .find(|item| {
                item.package == self.current_package
                    && item.module == self.current_module
                    && item.name == name
                    && item.kind == kind
            })
            .or_else(|| {
                self.loaded.package_graph.items.iter().find(|item| {
                    item.package == self.current_package
                        && item.name == name
                        && item.kind == kind
                        && matches!(item.visibility, Visibility::Package | Visibility::Public)
                })
            })
    }

    fn item_id(
        &self,
        module: ModuleId,
        name: &str,
        kind: PackageItemKind,
    ) -> Option<PackageItemId> {
        self.loaded
            .package_graph
            .item_id_in_module(module, name, kind)
    }

    fn symbol(&mut self, name: &str) -> Symbol {
        self.symbols.intern(name)
    }

    fn push_unsupported_generic_record(&mut self, name: &str, span: Span) {
        self.diagnostics.push(
            Diagnostic::new(
                "T013",
                format!("generic type `{name}` is not implemented yet"),
                span,
            )
            .with_suggestion("generic records are deferred"),
        );
    }

    fn validate_enum_arg_count(
        &mut self,
        item: PackageItemId,
        name: &str,
        actual: usize,
        span: Span,
    ) -> bool {
        let expected = self.enum_type_params.get(&item).map_or(0, Vec::len);
        if actual == expected {
            return true;
        }
        self.diagnostics.push(Diagnostic::new(
            "T022",
            format!("enum `{name}` expects exactly {expected} type arguments"),
            span,
        ));
        false
    }
}

fn split_qualified_name(name: &str) -> Option<(&str, &str)> {
    let mut parts = name.split("::");
    let first = parts.next()?;
    let second = parts.next()?;
    if parts.next().is_some() {
        None
    } else {
        Some((first, second))
    }
}
