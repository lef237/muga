use std::collections::HashMap;

use crate::{
    ast::{EnumDecl, FuncDecl, ImportDecl, OpaqueTypeDecl, RecordDecl, TypeExpr, Visibility},
    cli_schema::CliValueSource,
    diagnostic::Diagnostic,
    identity::{ModuleId, PackageId, PackageItemId},
    interface::{OpaqueHandleFacts, PackageInterfaceGraph, PackageInterfaceParamMode},
    json_decode::JsonDecodeValidationRule,
    known_enum,
    package::{LoadedPackageGraph, PackageItemInfo, PackageItemKind},
    span::Span,
    symbol::{Symbol, SymbolTable},
    types::{FunctionTypeInfo, TypeInfo},
};

#[derive(Clone, Debug)]
pub struct PackageSignatureEnvironment {
    pub symbols: SymbolTable,
    pub package_paths: HashMap<PackageId, String>,
    pub records: Vec<PackageRecordSignature>,
    pub enums: Vec<PackageEnumSignature>,
    pub opaque_types: Vec<PackageOpaqueTypeSignature>,
    pub functions: Vec<PackageFunctionSignature>,
    pub modules: Vec<PackageModuleSignatureEnvironment>,
}

impl PackageSignatureEnvironment {
    pub fn from_loaded_graph(loaded: &LoadedPackageGraph) -> Result<Self, Vec<Diagnostic>> {
        let mut collector = SignatureCollector::new(loaded);
        collector.collect();
        collector.finish()
    }

    pub fn from_interfaces(interfaces: &PackageInterfaceGraph, symbols: SymbolTable) -> Self {
        let mut package_paths = HashMap::new();
        let mut records = Vec::new();
        let mut enums = Vec::new();
        let mut opaque_types = Vec::new();
        let mut functions = Vec::new();

        for package in &interfaces.packages {
            package_paths.insert(package.package, package.path.clone());
            for record in &package.records {
                records.push(PackageRecordSignature {
                    item: record.item,
                    package: package.package,
                    module: ModuleId::new(0),
                    name: record.name.clone(),
                    visibility: Visibility::Public,
                    type_params: record.type_params.clone(),
                    json_deny_unknown_fields: record.json_deny_unknown_fields,
                    cli_about: record.cli_about.clone(),
                    fields: record
                        .fields
                        .iter()
                        .map(|field| PackageFieldSignature {
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
                            ty: field.ty.clone(),
                            span: field.span,
                        })
                        .collect(),
                    span: record.span,
                });
            }
            for enumeration in &package.enums {
                enums.push(PackageEnumSignature {
                    item: enumeration.item,
                    package: package.package,
                    module: ModuleId::new(0),
                    name: enumeration.name.clone(),
                    visibility: Visibility::Public,
                    type_params: enumeration.type_params.clone(),
                    cli_about: enumeration.cli_about.clone(),
                    variants: enumeration
                        .variants
                        .iter()
                        .map(|variant| PackageEnumVariantSignature {
                            name: variant.name.clone(),
                            json_rename: variant.json_rename.clone(),
                            json_aliases: variant.json_aliases.clone(),
                            cli_name: variant.cli_name.clone(),
                            cli_aliases: variant.cli_aliases.clone(),
                            cli_about: variant.cli_about.clone(),
                            cli_hidden: variant.cli_hidden,
                            payload: variant.payload.clone(),
                            span: variant.span,
                        })
                        .collect(),
                    span: enumeration.span,
                });
            }
            for opaque in &package.opaque_types {
                opaque_types.push(PackageOpaqueTypeSignature {
                    item: opaque.item,
                    package: package.package,
                    module: ModuleId::new(0),
                    name: opaque.name.clone(),
                    visibility: Visibility::Public,
                    handle_facts: opaque.handle_facts.clone(),
                    span: opaque.span,
                });
            }
            for function in &package.functions {
                functions.push(PackageFunctionSignature {
                    item: function.item,
                    package: package.package,
                    module: ModuleId::new(0),
                    name: function.name.clone(),
                    visibility: Visibility::Public,
                    type_params: function.type_params.clone(),
                    params: function
                        .params
                        .iter()
                        .map(|param| PackageParamSignature {
                            name: param.name.clone(),
                            ty: Some(param.ty.clone()),
                            mode: param.mode,
                            span: param.span,
                        })
                        .collect(),
                    ret: Some(function.ret.clone()),
                    span: function.span,
                });
            }
        }

        Self {
            symbols,
            package_paths,
            records,
            enums,
            opaque_types,
            functions,
            modules: Vec::new(),
        }
    }

    pub fn record(&self, item: PackageItemId) -> Option<&PackageRecordSignature> {
        self.records.iter().find(|record| record.item == item)
    }

    pub fn enumeration(&self, item: PackageItemId) -> Option<&PackageEnumSignature> {
        self.enums
            .iter()
            .find(|enumeration| enumeration.item == item)
    }

    pub fn opaque_type(&self, item: PackageItemId) -> Option<&PackageOpaqueTypeSignature> {
        self.opaque_types.iter().find(|opaque| opaque.item == item)
    }

    pub fn function(&self, item: PackageItemId) -> Option<&PackageFunctionSignature> {
        self.functions.iter().find(|function| function.item == item)
    }

    pub fn module(&self, module: ModuleId) -> Option<&PackageModuleSignatureEnvironment> {
        self.modules
            .iter()
            .find(|environment| environment.module == module)
    }

    pub fn package_path(&self, package: PackageId) -> Option<&str> {
        self.package_paths
            .get(&package)
            .map(std::string::String::as_str)
    }
}

#[derive(Clone, Debug)]
pub struct PackageRecordSignature {
    pub item: PackageItemId,
    pub package: PackageId,
    pub module: ModuleId,
    pub name: String,
    pub visibility: Visibility,
    pub type_params: Vec<String>,
    pub json_deny_unknown_fields: bool,
    pub cli_about: Option<String>,
    pub fields: Vec<PackageFieldSignature>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct PackageFieldSignature {
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

#[derive(Clone, Debug)]
pub struct PackageEnumSignature {
    pub item: PackageItemId,
    pub package: PackageId,
    pub module: ModuleId,
    pub name: String,
    pub visibility: Visibility,
    pub type_params: Vec<String>,
    pub cli_about: Option<String>,
    pub variants: Vec<PackageEnumVariantSignature>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct PackageEnumVariantSignature {
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

#[derive(Clone, Debug)]
pub struct PackageOpaqueTypeSignature {
    pub item: PackageItemId,
    pub package: PackageId,
    pub module: ModuleId,
    pub name: String,
    pub visibility: Visibility,
    pub handle_facts: OpaqueHandleFacts,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct PackageFunctionSignature {
    pub item: PackageItemId,
    pub package: PackageId,
    pub module: ModuleId,
    pub name: String,
    pub visibility: Visibility,
    pub type_params: Vec<String>,
    pub params: Vec<PackageParamSignature>,
    pub ret: Option<TypeInfo>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct PackageParamSignature {
    pub name: String,
    pub ty: Option<TypeInfo>,
    pub mode: PackageInterfaceParamMode,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct PackageModuleSignatureEnvironment {
    pub package: PackageId,
    pub module: ModuleId,
    pub module_path: String,
    pub records: Vec<PackageVisibleSignature>,
    pub enums: Vec<PackageVisibleSignature>,
    pub opaque_types: Vec<PackageVisibleSignature>,
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

    pub fn opaque_type(&self, name: &str) -> Option<&PackageVisibleSignature> {
        self.opaque_types.iter().find(|opaque| opaque.name == name)
    }

    pub fn opaque_type_signature<'a>(
        &self,
        signatures: &'a PackageSignatureEnvironment,
        name: &str,
    ) -> Option<&'a PackageOpaqueTypeSignature> {
        signatures.opaque_type(self.opaque_type(name)?.item)
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
    record_type_params: HashMap<PackageItemId, Vec<String>>,
    enum_type_params: HashMap<PackageItemId, Vec<String>>,
    imports: HashMap<String, String>,
    current_package: PackageId,
    current_module: ModuleId,
    records: Vec<PackageRecordSignature>,
    enums: Vec<PackageEnumSignature>,
    opaque_types: Vec<PackageOpaqueTypeSignature>,
    functions: Vec<PackageFunctionSignature>,
    modules: Vec<PackageModuleSignatureEnvironment>,
}

impl<'a> SignatureCollector<'a> {
    fn new(loaded: &'a LoadedPackageGraph) -> Self {
        Self {
            loaded,
            symbols: SymbolTable::default(),
            diagnostics: Vec::new(),
            record_type_params: HashMap::new(),
            enum_type_params: HashMap::new(),
            imports: HashMap::new(),
            current_package: PackageId::new(0),
            current_module: ModuleId::new(0),
            records: Vec::new(),
            enums: Vec::new(),
            opaque_types: Vec::new(),
            functions: Vec::new(),
            modules: Vec::new(),
        }
    }

    fn collect(&mut self) {
        self.collect_interface_signatures();
        self.collect_type_headers();
        for package in &self.loaded.packages {
            if self.loaded.is_loaded_interface_package_path(&package.path) {
                continue;
            }
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
                        crate::ast::Stmt::OpaqueTypeDecl(opaque) => {
                            self.collect_opaque_type(opaque)
                        }
                        crate::ast::Stmt::FuncDecl(function) => self.collect_function(function),
                        _ => {}
                    }
                }
            }
        }
    }

    fn collect_interface_signatures(&mut self) {
        let Some(interfaces) = self.loaded.interfaces.clone() else {
            return;
        };
        for interface in &interfaces.graph.packages {
            if self
                .loaded
                .package_graph
                .package(self.loaded.entry_package)
                .is_some_and(|package| package.path == interface.path)
            {
                continue;
            }
            for record in &interface.records {
                let Some(item) = self.loaded.package_graph.item(record.item) else {
                    continue;
                };
                let fields = record
                    .fields
                    .iter()
                    .map(|field| PackageFieldSignature {
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
                        ty: self.interface_type_info(&field.ty, &interfaces.symbols),
                        span: field.span,
                    })
                    .collect();
                self.records.push(PackageRecordSignature {
                    item: record.item,
                    package: item.package,
                    module: item.module,
                    name: record.name.clone(),
                    visibility: Visibility::Public,
                    type_params: record.type_params.clone(),
                    json_deny_unknown_fields: record.json_deny_unknown_fields,
                    cli_about: record.cli_about.clone(),
                    fields,
                    span: record.span,
                });
                self.record_type_params
                    .insert(record.item, record.type_params.clone());
            }
            for enumeration in &interface.enums {
                let Some(item) = self.loaded.package_graph.item(enumeration.item) else {
                    continue;
                };
                self.enum_type_params
                    .insert(enumeration.item, enumeration.type_params.clone());
                let variants = enumeration
                    .variants
                    .iter()
                    .map(|variant| PackageEnumVariantSignature {
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
                            .map(|payload| self.interface_type_info(payload, &interfaces.symbols)),
                        span: variant.span,
                    })
                    .collect();
                self.enums.push(PackageEnumSignature {
                    item: enumeration.item,
                    package: item.package,
                    module: item.module,
                    name: enumeration.name.clone(),
                    visibility: Visibility::Public,
                    type_params: enumeration.type_params.clone(),
                    cli_about: enumeration.cli_about.clone(),
                    variants,
                    span: enumeration.span,
                });
            }
            for opaque in &interface.opaque_types {
                let Some(item) = self.loaded.package_graph.item(opaque.item) else {
                    continue;
                };
                self.opaque_types.push(PackageOpaqueTypeSignature {
                    item: opaque.item,
                    package: item.package,
                    module: item.module,
                    name: opaque.name.clone(),
                    visibility: Visibility::Public,
                    handle_facts: opaque.handle_facts.clone(),
                    span: opaque.span,
                });
            }
            for function in &interface.functions {
                let Some(item) = self.loaded.package_graph.item(function.item) else {
                    continue;
                };
                let params = function
                    .params
                    .iter()
                    .map(|param| PackageParamSignature {
                        name: param.name.clone(),
                        ty: Some(self.interface_type_info(&param.ty, &interfaces.symbols)),
                        mode: param.mode,
                        span: param.span,
                    })
                    .collect();
                let ret = self.interface_type_info(&function.ret, &interfaces.symbols);
                self.functions.push(PackageFunctionSignature {
                    item: function.item,
                    package: item.package,
                    module: item.module,
                    name: function.name.clone(),
                    visibility: Visibility::Public,
                    type_params: function.type_params.clone(),
                    params,
                    ret: Some(ret),
                    span: function.span,
                });
            }
        }
    }

    fn finish(self) -> Result<PackageSignatureEnvironment, Vec<Diagnostic>> {
        if self.diagnostics.is_empty() {
            let package_paths = self
                .loaded
                .package_graph
                .packages
                .iter()
                .map(|package| (package.id, package.path.clone()))
                .collect();
            Ok(PackageSignatureEnvironment {
                symbols: self.symbols,
                package_paths,
                records: self.records,
                enums: self.enums,
                opaque_types: self.opaque_types,
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
            opaque_types: Vec::new(),
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
                PackageItemKind::OpaqueType => environment.opaque_types.push(visible),
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
            for opaque in &exports.opaque_types {
                environment.opaque_types.push(PackageVisibleSignature {
                    name: format!("{}::{}", import.alias, opaque.name),
                    item: opaque.item,
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

    fn collect_type_headers(&mut self) {
        let mut record_headers = Vec::new();
        let mut enum_headers = Vec::new();
        for package in &self.loaded.packages {
            if self.loaded.is_loaded_interface_package_path(&package.path) {
                continue;
            }
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
                    match statement {
                        crate::ast::Stmt::RecordDecl(record) => {
                            if let Some(item) =
                                self.item_id(module_id, &record.name, PackageItemKind::Record)
                            {
                                record_headers.push((item, record.type_params.clone()));
                            }
                        }
                        crate::ast::Stmt::EnumDecl(enumeration) => {
                            if let Some(item) =
                                self.item_id(module_id, &enumeration.name, PackageItemKind::Enum)
                            {
                                enum_headers.push((item, enumeration.type_params.clone()));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        self.record_type_params.extend(record_headers);
        self.enum_type_params.extend(enum_headers);
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
                json_rename: json_rename_from_attributes(&field.attributes),
                json_aliases: json_aliases_from_attributes(&field.attributes),
                json_validation: json_validation_from_attributes(&field.attributes),
                cli_name: cli_name_from_attributes(&field.attributes),
                cli_short: cli_short_from_attributes(&field.attributes),
                cli_position: cli_position_from_attributes(&field.attributes),
                cli_value_source: cli_value_source_from_attributes(&field.attributes),
                cli_aliases: cli_aliases_from_attributes(&field.attributes),
                cli_help: cli_help_from_attributes(&field.attributes),
                cli_hidden: cli_hidden_from_attributes(&field.attributes),
                cli_subcommand: cli_subcommand_from_attributes(&field.attributes),
                ty: self.type_info_from_type_expr(
                    &field.type_name,
                    field.span,
                    &record.type_params,
                ),
                span: field.span,
            })
            .collect();
        self.records.push(PackageRecordSignature {
            item,
            package: self.current_package,
            module: self.current_module,
            name: record.name.clone(),
            visibility: record.visibility,
            type_params: record.type_params.clone(),
            json_deny_unknown_fields: json_deny_unknown_fields_from_attributes(&record.attributes),
            cli_about: cli_about_from_attributes(&record.attributes),
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
                json_rename: json_rename_from_attributes(&variant.attributes),
                json_aliases: json_aliases_from_attributes(&variant.attributes),
                cli_name: cli_name_from_attributes(&variant.attributes),
                cli_aliases: cli_aliases_from_attributes(&variant.attributes),
                cli_about: cli_about_from_attributes(&variant.attributes),
                cli_hidden: cli_hidden_from_attributes(&variant.attributes),
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
            cli_about: cli_about_from_attributes(&enumeration.attributes),
            variants,
            span: enumeration.span,
        });
    }

    fn collect_opaque_type(&mut self, opaque: &OpaqueTypeDecl) {
        let Some(item) = self.item_id(
            self.current_module,
            &opaque.name,
            PackageItemKind::OpaqueType,
        ) else {
            return;
        };
        self.opaque_types.push(PackageOpaqueTypeSignature {
            item,
            package: self.current_package,
            module: self.current_module,
            name: opaque.name.clone(),
            visibility: opaque.visibility,
            handle_facts: self.package_opaque_handle_facts(&opaque.name),
            span: opaque.span,
        });
    }

    fn package_opaque_handle_facts(&self, opaque_name: &str) -> OpaqueHandleFacts {
        if self.current_package_path() == Some(crate::std_package::FS_PACKAGE)
            && opaque_name == "File"
        {
            OpaqueHandleFacts {
                runtime_backed: true,
                copyable: false,
                cloneable: false,
                sendable: false,
                shareable: false,
                structurally_comparable: false,
                serializable: false,
                closeable: true,
                close_function: self.item_id(
                    self.current_module,
                    "close",
                    PackageItemKind::Function,
                ),
            }
        } else {
            OpaqueHandleFacts::default()
        }
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
                ty: param.type_name.as_ref().map(|type_name| {
                    self.type_info_from_type_expr(type_name, param.span, &function.type_params)
                }),
                mode: self.package_param_mode(&function.name, &param.name),
                span: param.span,
            })
            .collect();
        let ret = function.return_type.as_ref().map(|type_name| {
            self.type_info_from_type_expr(type_name, function.span, &function.type_params)
        });
        self.functions.push(PackageFunctionSignature {
            item,
            package: self.current_package,
            module: self.current_module,
            name: function.name.clone(),
            visibility: function.visibility,
            type_params: function.type_params.clone(),
            params,
            ret,
            span: function.span,
        });
    }

    fn package_param_mode(
        &self,
        function_name: &str,
        param_name: &str,
    ) -> PackageInterfaceParamMode {
        if self.current_package_path() == Some(crate::std_package::FS_PACKAGE)
            && function_name == "close"
            && param_name == "file"
        {
            PackageInterfaceParamMode::Consume
        } else {
            PackageInterfaceParamMode::Borrow
        }
    }

    fn current_package_path(&self) -> Option<&str> {
        self.loaded
            .package_graph
            .package(self.current_package)
            .map(|package| package.path.as_str())
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
            TypeExpr::Unit => TypeInfo::Unit,
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
            TypeExpr::Generic(generic)
                if generic.name == "Task"
                    && generic.args.len() == 1
                    && self.current_package_path() == Some(crate::std_package::TASK_PACKAGE) =>
            {
                TypeInfo::Task(Box::new(self.type_info_from_type_expr(
                    &generic.args[0],
                    span,
                    type_params,
                )))
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
            if !self.validate_record_arg_count(item, name, args.len(), span) {
                return TypeInfo::Error;
            }
            let symbol = self.symbol(name);
            return TypeInfo::PackageRecord { symbol, item, args };
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
        if let Some(item) = self
            .visible_same_package_item(name, PackageItemKind::OpaqueType)
            .map(|item| item.id)
        {
            if !self.validate_opaque_arg_count(name, args.len(), span) {
                return TypeInfo::Error;
            }
            let symbol = self.symbol(name);
            return TypeInfo::PackageOpaque { symbol, item };
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
            self.diagnostics
                .push(unknown_import_alias_diagnostic(alias, span));
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
            if !self.validate_record_arg_count(export.item, name, args.len(), span) {
                return TypeInfo::Error;
            }
            let symbol = self.symbol(name);
            return TypeInfo::PackageRecord {
                symbol,
                item: export.item,
                args,
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
        if let Some(export) = self
            .loaded
            .package_exports
            .opaque_type_by_name(package_id, name)
        {
            if !self.validate_opaque_arg_count(name, args.len(), span) {
                return TypeInfo::Error;
            }
            let symbol = self.symbol(name);
            return TypeInfo::PackageOpaque {
                symbol,
                item: export.item,
            };
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

    fn validate_record_arg_count(
        &mut self,
        item: PackageItemId,
        name: &str,
        actual: usize,
        span: Span,
    ) -> bool {
        let expected = self.record_type_params.get(&item).map_or(0, Vec::len);
        if actual == expected {
            return true;
        }
        self.diagnostics.push(Diagnostic::new(
            "T022",
            format!("record `{name}` expects exactly {expected} type arguments but found {actual}"),
            span,
        ));
        false
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
            format!("enum `{name}` expects exactly {expected} type arguments but found {actual}"),
            span,
        ));
        false
    }

    fn validate_opaque_arg_count(&mut self, name: &str, actual: usize, span: Span) -> bool {
        if actual == 0 {
            return true;
        }
        self.diagnostics.push(Diagnostic::new(
            "T022",
            format!("opaque type `{name}` expects exactly 0 type arguments but found {actual}"),
            span,
        ));
        false
    }

    fn interface_type_info(&mut self, ty: &TypeInfo, symbols: &SymbolTable) -> TypeInfo {
        match ty {
            TypeInfo::GenericParam(symbol) => {
                TypeInfo::GenericParam(self.interface_symbol(*symbol, symbols))
            }
            TypeInfo::Record(symbol, args) => TypeInfo::Record(
                self.interface_symbol(*symbol, symbols),
                args.iter()
                    .map(|arg| self.interface_type_info(arg, symbols))
                    .collect(),
            ),
            TypeInfo::PackageRecord { symbol, item, args } => TypeInfo::PackageRecord {
                symbol: self.interface_symbol(*symbol, symbols),
                item: *item,
                args: args
                    .iter()
                    .map(|arg| self.interface_type_info(arg, symbols))
                    .collect(),
            },
            TypeInfo::Enum { symbol, args } => TypeInfo::Enum {
                symbol: self.interface_symbol(*symbol, symbols),
                args: args
                    .iter()
                    .map(|arg| self.interface_type_info(arg, symbols))
                    .collect(),
            },
            TypeInfo::PackageEnum { symbol, item, args } => TypeInfo::PackageEnum {
                symbol: self.interface_symbol(*symbol, symbols),
                item: *item,
                args: args
                    .iter()
                    .map(|arg| self.interface_type_info(arg, symbols))
                    .collect(),
            },
            TypeInfo::PackageOpaque { symbol, item } => TypeInfo::PackageOpaque {
                symbol: self.interface_symbol(*symbol, symbols),
                item: *item,
            },
            TypeInfo::List(item) => {
                TypeInfo::List(Box::new(self.interface_type_info(item, symbols)))
            }
            TypeInfo::Map(key, value) => TypeInfo::Map(
                Box::new(self.interface_type_info(key, symbols)),
                Box::new(self.interface_type_info(value, symbols)),
            ),
            TypeInfo::Option(item) => {
                TypeInfo::Option(Box::new(self.interface_type_info(item, symbols)))
            }
            TypeInfo::Result(ok, err) => TypeInfo::Result(
                Box::new(self.interface_type_info(ok, symbols)),
                Box::new(self.interface_type_info(err, symbols)),
            ),
            TypeInfo::Task(item) => {
                TypeInfo::Task(Box::new(self.interface_type_info(item, symbols)))
            }
            TypeInfo::EnumConstructor {
                enum_symbol,
                enum_item,
                variant,
            } => TypeInfo::EnumConstructor {
                enum_symbol: self.interface_symbol(*enum_symbol, symbols),
                enum_item: *enum_item,
                variant: self.interface_symbol(*variant, symbols),
            },
            TypeInfo::Function(function) => TypeInfo::Function(FunctionTypeInfo {
                params: function
                    .params
                    .iter()
                    .map(|param| self.interface_type_info(param, symbols))
                    .collect(),
                ret: Box::new(self.interface_type_info(&function.ret, symbols)),
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

    fn interface_symbol(&mut self, symbol: Symbol, symbols: &SymbolTable) -> Symbol {
        self.symbols.intern(symbols.resolve(symbol))
    }
}

fn unknown_import_alias_diagnostic(alias: &str, span: Span) -> Diagnostic {
    let diagnostic = Diagnostic::new("PK009", format!("unknown import alias `{alias}`"), span);
    match alias {
        "cli" => diagnostic.with_suggestion("add `import std::cli` before using `cli::...`"),
        "env" => diagnostic.with_suggestion("add `import std::env` before using `env::...`"),
        "fmt" => diagnostic.with_suggestion("add `import std::fmt` before using `fmt::...`"),
        "fs" => diagnostic.with_suggestion("add `import std::fs` before using `fs::...`"),
        "io" => diagnostic.with_suggestion("add `import std::io` before using `io::...`"),
        "json" => diagnostic.with_suggestion("add `import std::json` before using `json::...`"),
        "path" => diagnostic.with_suggestion("add `import std::path` before using `path::...`"),
        "string" => {
            diagnostic.with_suggestion("add `import std::string` before using `string::...`")
        }
        "time" => diagnostic.with_suggestion("add `import std::time` before using `time::...`"),
        _ => {
            diagnostic.with_suggestion("add an import declaration or use an existing import alias")
        }
    }
}

fn json_rename_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "json" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "rename")
                .and_then(|argument| argument.string_value().map(ToOwned::to_owned))
        } else {
            None
        }
    })
}

fn json_aliases_from_attributes(attributes: &[crate::ast::Attribute]) -> Vec<String> {
    attributes
        .iter()
        .filter(|attribute| attribute.name == "json")
        .flat_map(|attribute| {
            attribute
                .arguments
                .iter()
                .filter(|argument| argument.name == "alias")
                .filter_map(|argument| argument.string_value().map(ToOwned::to_owned))
        })
        .collect()
}

fn cli_name_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "cli" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "name")
                .and_then(|argument| argument.string_value().map(ToOwned::to_owned))
        } else {
            None
        }
    })
}

fn cli_short_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "cli" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "short")
                .and_then(|argument| argument.string_value().map(ToOwned::to_owned))
        } else {
            None
        }
    })
}

fn cli_position_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<u32> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "cli" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "positional")
                .and_then(|argument| {
                    argument
                        .int_value()
                        .and_then(|value| u32::try_from(value).ok())
                        .filter(|value| *value > 0)
                })
        } else {
            None
        }
    })
}

fn cli_value_source_from_attributes(
    attributes: &[crate::ast::Attribute],
) -> Option<CliValueSource> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "cli" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "value_source")
                .and_then(|argument| argument.string_value())
                .and_then(|value| CliValueSource::from_artifact_token(value).ok())
        } else {
            None
        }
    })
}

fn cli_aliases_from_attributes(attributes: &[crate::ast::Attribute]) -> Vec<String> {
    attributes
        .iter()
        .filter(|attribute| attribute.name == "cli")
        .flat_map(|attribute| {
            attribute
                .arguments
                .iter()
                .filter(|argument| argument.name == "alias")
                .filter_map(|argument| argument.string_value().map(ToOwned::to_owned))
        })
        .collect()
}

fn cli_help_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "cli" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "help")
                .and_then(|argument| argument.string_value().map(ToOwned::to_owned))
        } else {
            None
        }
    })
}

fn cli_about_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<String> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "cli" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "about")
                .and_then(|argument| argument.string_value().map(ToOwned::to_owned))
        } else {
            None
        }
    })
}

fn cli_hidden_from_attributes(attributes: &[crate::ast::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.name == "cli"
            && attribute
                .arguments
                .iter()
                .any(|argument| argument.name == "hidden" && argument.value.is_none())
    })
}

fn cli_subcommand_from_attributes(attributes: &[crate::ast::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.name == "cli"
            && attribute
                .arguments
                .iter()
                .any(|argument| argument.name == "subcommand" && argument.value.is_none())
    })
}

fn json_validation_from_attributes(
    attributes: &[crate::ast::Attribute],
) -> Vec<JsonDecodeValidationRule> {
    attributes
        .iter()
        .filter(|attribute| attribute.name == "validate")
        .flat_map(|attribute| attribute.arguments.iter())
        .filter_map(json_validation_rule_from_argument)
        .collect()
}

fn json_validation_rule_from_argument(
    argument: &crate::ast::AttributeArgument,
) -> Option<JsonDecodeValidationRule> {
    match argument.name.as_str() {
        "non_empty" => Some(JsonDecodeValidationRule::NonEmpty),
        "min" => argument.int_value().map(JsonDecodeValidationRule::Min),
        "max" => argument.int_value().map(JsonDecodeValidationRule::Max),
        "min_len" => argument.int_value().map(JsonDecodeValidationRule::MinLen),
        "max_len" => argument.int_value().map(JsonDecodeValidationRule::MaxLen),
        _ => None,
    }
}

fn json_deny_unknown_fields_from_attributes(attributes: &[crate::ast::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.name == "json"
            && attribute
                .arguments
                .iter()
                .any(|argument| argument.name == "deny_unknown_fields" && argument.value.is_none())
    })
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
