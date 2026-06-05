use std::collections::{HashMap, HashSet};

use crate::ast::*;
use crate::cli_schema::{
    CliCommandVariantSchema, CliEnumVariantSchema, CliFieldSchema, CliSchema, CliSubcommandSchema,
    CliValueSchema, CliValueSource,
};
use crate::diagnostic::Diagnostic;
use crate::identity::{BindingId, BindingKind, ExprId, ModuleId, PackageItemId, StmtId};
use crate::interface::{OpaqueHandleFacts, PackageInterfaceParamMode};
use crate::json_decode::{
    JsonDecodeFieldSchema, JsonDecodeSchema, JsonDecodeValidationRule, JsonDecodeVariantSchema,
};
use crate::known_enum;
use crate::package_signature::{
    PackageModuleSignatureEnvironment, PackageSignatureEnvironment, PackageSignatureSource,
};
use crate::prelude::{self, BuiltinId, BuiltinKind};
use crate::span::Span;
use crate::symbol::{Symbol, SymbolTable};
pub use crate::types::{FunctionTypeInfo, TypeInfo};

#[derive(Clone, Debug)]
pub struct TypeCheckOutput {
    pub diagnostics: Vec<Diagnostic>,
    pub bindings: Vec<TypedBindingInfo>,
    pub assignment_targets: Vec<TypedAssignmentTarget>,
    pub using_cleanups: Vec<TypedUsingCleanupInfo>,
    pub identifier_refs: Vec<TypedIdentifier>,
    pub calls: Vec<TypedCallInfo>,
    pub json_decode_schemas: Vec<TypedJsonDecodeSchemaInfo>,
    pub json_required_decode_schemas: Vec<TypedJsonDecodeSchemaInfo>,
    pub json_to_value_schemas: Vec<TypedJsonDecodeSchemaInfo>,
    pub json_encode_typed_schemas: Vec<TypedJsonDecodeSchemaInfo>,
    pub config_required_load_json_schemas: Vec<TypedJsonDecodeSchemaInfo>,
    pub config_load_json_schemas: Vec<TypedJsonDecodeSchemaInfo>,
    pub cli_parse_schemas: Vec<TypedCliSchemaInfo>,
    pub cli_parse_or_schemas: Vec<TypedCliSchemaInfo>,
    pub cli_parse_request_schemas: Vec<TypedCliSchemaInfo>,
    pub cli_parse_request_or_schemas: Vec<TypedCliSchemaInfo>,
    pub cli_usage_for_schemas: Vec<TypedCliSchemaInfo>,
    pub cli_usage_for_required_schemas: Vec<TypedCliSchemaInfo>,
    pub cli_help_for_schemas: Vec<TypedCliSchemaInfo>,
    pub cli_help_for_required_schemas: Vec<TypedCliSchemaInfo>,
    pub expr_types: Vec<ExprTypeInfo>,
    pub symbols: SymbolTable,
    pub package_opaque_types: Vec<(Symbol, PackageItemId)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedBindingInfo {
    pub id: BindingId,
    pub symbol: Symbol,
    pub kind: BindingKind,
    pub ty: TypeInfo,
    pub package_item: Option<PackageItemId>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedAssignmentTarget {
    pub stmt_id: StmtId,
    pub name: Symbol,
    pub span: Span,
    pub binding: BindingId,
    pub is_update: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedUsingCleanupInfo {
    pub stmt_id: StmtId,
    pub name: Symbol,
    pub callee: TypedCalleeInfo,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedIdentifier {
    pub expr_id: ExprId,
    pub name: Symbol,
    pub span: Span,
    pub binding: BindingId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedCallInfo {
    pub expr_id: ExprId,
    pub span: Span,
    pub callee: TypedCalleeInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedJsonDecodeSchemaInfo {
    pub expr_id: ExprId,
    pub schema: JsonDecodeSchema,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedCliSchemaInfo {
    pub expr_id: ExprId,
    pub schema: CliSchema,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypedCalleeInfo {
    Binding(BindingId),
    PackageItem {
        binding: BindingId,
        item: PackageItemId,
    },
    EnumVariant {
        binding: BindingId,
        enum_name: Symbol,
        enum_item: Option<PackageItemId>,
        variant_name: Symbol,
    },
    Builtin {
        binding: BindingId,
        name: &'static str,
    },
    Value,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExprTypeInfo {
    pub expr_id: ExprId,
    pub span: Span,
    pub ty: TypeInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Type {
    Int,
    Bool,
    String,
    Unit,
    Record(Symbol, Vec<Type>),
    Enum(Symbol, Vec<Type>),
    Opaque(Symbol),
    GenericParam(Symbol),
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    OptionNone,
    EnumConstructor {
        enum_name: Symbol,
        enum_item: Option<PackageItemId>,
        variant_name: Symbol,
    },
    Function(FunctionSig),
    Builtin(BuiltinId),
    Never,
    Unknown(u32),
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FunctionSig {
    type_params: Vec<Symbol>,
    params: Vec<Type>,
    ret: Box<Type>,
}

#[derive(Clone, Debug)]
struct Binding {
    id: BindingId,
    symbol: Symbol,
    kind: BindingKind,
    ty: Type,
    span: Span,
}

#[derive(Clone, Debug)]
struct ExprType {
    expr_id: ExprId,
    span: Span,
    ty: Type,
}

#[derive(Clone, Debug)]
struct RecordDef {
    span: Span,
    type_params: Vec<Symbol>,
    json_deny_unknown_fields: bool,
    cli_about: Option<Symbol>,
    fields: Vec<RecordField>,
}

#[derive(Clone, Debug)]
struct RecordField {
    name: Symbol,
    json_rename: Option<Symbol>,
    json_aliases: Vec<Symbol>,
    json_validation: Vec<JsonDecodeValidationRule>,
    cli_name: Option<Symbol>,
    cli_short: Option<Symbol>,
    cli_position: Option<u32>,
    cli_value_source: Option<CliValueSource>,
    cli_aliases: Vec<Symbol>,
    cli_help: Option<Symbol>,
    cli_hidden: bool,
    cli_subcommand: bool,
    type_name: Option<TypeExpr>,
    ty: Option<Type>,
    span: Span,
}

#[derive(Clone, Debug)]
struct EnumDef {
    span: Span,
    type_params: Vec<Symbol>,
    cli_about: Option<Symbol>,
    variants: Vec<EnumVariantDef>,
}

#[derive(Clone, Debug)]
struct EnumVariantDef {
    name: Symbol,
    json_rename: Option<Symbol>,
    json_aliases: Vec<Symbol>,
    cli_name: Option<Symbol>,
    cli_aliases: Vec<Symbol>,
    cli_about: Option<Symbol>,
    cli_hidden: bool,
    payload: Option<TypeExpr>,
    payload_ty: Option<Type>,
    span: Span,
}

#[derive(Clone, Debug)]
struct EnumMatchSpec {
    enum_name: Symbol,
    display_name: String,
    variants: Vec<EnumMatchVariant>,
}

impl EnumMatchSpec {
    fn variant(&self, name: Symbol) -> Option<&EnumMatchVariant> {
        self.variants.iter().find(|variant| variant.name == name)
    }
}

#[derive(Clone, Debug)]
struct EnumMatchVariant {
    name: Symbol,
    payload: Option<Type>,
}

struct ScopeFrame {
    bindings: HashMap<Symbol, BindingId>,
    consumed_bindings: HashMap<BindingId, Span>,
    function_boundary: bool,
}

impl ScopeFrame {
    fn new(function_boundary: bool) -> Self {
        Self {
            bindings: HashMap::new(),
            consumed_bindings: HashMap::new(),
            function_boundary,
        }
    }
}

pub fn typecheck(program: &Program) -> Vec<Diagnostic> {
    typecheck_program(program).diagnostics
}

pub fn typecheck_program(program: &Program) -> TypeCheckOutput {
    let mut checker = TypeChecker::new();
    checker.predeclare_records(&program.statements);
    checker.predeclare_enums(&program.statements);
    checker.predeclare_opaque_types(&program.statements);
    checker.check_scope_statements(&program.statements);
    checker.into_output()
}

pub fn typecheck_package_module(
    program: &Program,
    signatures: &PackageSignatureEnvironment,
    module: ModuleId,
) -> TypeCheckOutput {
    let mut checker = TypeChecker::new();
    if program
        .package
        .as_ref()
        .is_some_and(|package| crate::std_package::allows_internal_builtins(&package.path))
    {
        checker.install_internal_builtins();
    }
    if let Some(environment) = signatures.module(module) {
        checker.install_package_module_signatures(signatures, environment);
    }
    checker.predeclare_records(&program.statements);
    checker.predeclare_enums(&program.statements);
    checker.predeclare_opaque_types(&program.statements);
    checker.check_scope_statements(&program.statements);
    checker.into_output()
}

struct TypeChecker {
    scopes: Vec<ScopeFrame>,
    records: HashMap<Symbol, RecordDef>,
    enums: HashMap<Symbol, EnumDef>,
    bindings: Vec<Binding>,
    assignment_targets: Vec<TypedAssignmentTarget>,
    using_cleanups: Vec<TypedUsingCleanupInfo>,
    identifier_refs: Vec<TypedIdentifier>,
    calls: Vec<TypedCallInfo>,
    json_decode_schemas: Vec<TypedJsonDecodeSchemaInfo>,
    json_required_decode_schemas: Vec<TypedJsonDecodeSchemaInfo>,
    json_to_value_schemas: Vec<TypedJsonDecodeSchemaInfo>,
    json_encode_typed_schemas: Vec<TypedJsonDecodeSchemaInfo>,
    config_required_load_json_schemas: Vec<TypedJsonDecodeSchemaInfo>,
    config_load_json_schemas: Vec<TypedJsonDecodeSchemaInfo>,
    cli_parse_schemas: Vec<TypedCliSchemaInfo>,
    cli_parse_or_schemas: Vec<TypedCliSchemaInfo>,
    cli_parse_request_schemas: Vec<TypedCliSchemaInfo>,
    cli_parse_request_or_schemas: Vec<TypedCliSchemaInfo>,
    cli_usage_for_schemas: Vec<TypedCliSchemaInfo>,
    cli_usage_for_required_schemas: Vec<TypedCliSchemaInfo>,
    cli_help_for_schemas: Vec<TypedCliSchemaInfo>,
    cli_help_for_required_schemas: Vec<TypedCliSchemaInfo>,
    expr_types: Vec<ExprType>,
    symbols: SymbolTable,
    diagnostics: Vec<Diagnostic>,
    next_unknown: u32,
    substitutions: HashMap<u32, Type>,
    package_records: HashMap<PackageItemId, Symbol>,
    package_enums: HashMap<PackageItemId, Symbol>,
    package_opaques: HashMap<PackageItemId, Symbol>,
    package_record_items: HashMap<Symbol, PackageItemId>,
    package_enum_items: HashMap<Symbol, PackageItemId>,
    package_opaque_items: HashMap<Symbol, PackageItemId>,
    package_function_items: HashMap<Symbol, PackageItemId>,
    package_function_bindings_by_item: HashMap<PackageItemId, BindingId>,
    package_items_by_binding: HashMap<BindingId, PackageItemId>,
    package_function_param_modes: HashMap<BindingId, Vec<PackageInterfaceParamMode>>,
    package_opaque_handle_facts: HashMap<PackageItemId, OpaqueHandleFacts>,
    std_json_decode_bindings: HashSet<BindingId>,
    std_json_decode_or_bindings: HashSet<BindingId>,
    std_json_to_value_bindings: HashSet<BindingId>,
    std_json_encode_typed_bindings: HashSet<BindingId>,
    std_config_load_json_bindings: HashSet<BindingId>,
    std_config_load_json_or_bindings: HashSet<BindingId>,
    std_cli_parse_bindings: HashSet<BindingId>,
    std_cli_parse_or_bindings: HashSet<BindingId>,
    std_cli_parse_request_bindings: HashSet<BindingId>,
    std_cli_parse_request_or_bindings: HashSet<BindingId>,
    std_cli_usage_for_bindings: HashSet<BindingId>,
    std_cli_usage_for_required_bindings: HashSet<BindingId>,
    std_cli_help_for_bindings: HashSet<BindingId>,
    std_cli_help_for_required_bindings: HashSet<BindingId>,
    std_json_value_symbols: HashSet<Symbol>,
    using_bindings: HashMap<BindingId, Span>,
    current_function_returns: Vec<Type>,
    current_type_params: Vec<Vec<Symbol>>,
    loop_depth: usize,
}

impl TypeChecker {
    fn new() -> Self {
        let mut checker = Self {
            scopes: vec![ScopeFrame::new(true)],
            records: HashMap::new(),
            enums: HashMap::new(),
            bindings: Vec::new(),
            assignment_targets: Vec::new(),
            using_cleanups: Vec::new(),
            identifier_refs: Vec::new(),
            calls: Vec::new(),
            json_decode_schemas: Vec::new(),
            json_required_decode_schemas: Vec::new(),
            json_to_value_schemas: Vec::new(),
            json_encode_typed_schemas: Vec::new(),
            config_required_load_json_schemas: Vec::new(),
            config_load_json_schemas: Vec::new(),
            cli_parse_schemas: Vec::new(),
            cli_parse_or_schemas: Vec::new(),
            cli_parse_request_schemas: Vec::new(),
            cli_parse_request_or_schemas: Vec::new(),
            cli_usage_for_schemas: Vec::new(),
            cli_usage_for_required_schemas: Vec::new(),
            cli_help_for_schemas: Vec::new(),
            cli_help_for_required_schemas: Vec::new(),
            expr_types: Vec::new(),
            symbols: SymbolTable::default(),
            diagnostics: Vec::new(),
            next_unknown: 0,
            substitutions: HashMap::new(),
            package_records: HashMap::new(),
            package_enums: HashMap::new(),
            package_opaques: HashMap::new(),
            package_record_items: HashMap::new(),
            package_enum_items: HashMap::new(),
            package_opaque_items: HashMap::new(),
            package_function_items: HashMap::new(),
            package_function_bindings_by_item: HashMap::new(),
            package_items_by_binding: HashMap::new(),
            package_function_param_modes: HashMap::new(),
            package_opaque_handle_facts: HashMap::new(),
            std_json_decode_bindings: HashSet::new(),
            std_json_decode_or_bindings: HashSet::new(),
            std_json_to_value_bindings: HashSet::new(),
            std_json_encode_typed_bindings: HashSet::new(),
            std_config_load_json_bindings: HashSet::new(),
            std_config_load_json_or_bindings: HashSet::new(),
            std_cli_parse_bindings: HashSet::new(),
            std_cli_parse_or_bindings: HashSet::new(),
            std_cli_parse_request_bindings: HashSet::new(),
            std_cli_parse_request_or_bindings: HashSet::new(),
            std_cli_usage_for_bindings: HashSet::new(),
            std_cli_usage_for_required_bindings: HashSet::new(),
            std_cli_help_for_bindings: HashSet::new(),
            std_cli_help_for_required_bindings: HashSet::new(),
            std_json_value_symbols: HashSet::new(),
            using_bindings: HashMap::new(),
            current_function_returns: Vec::new(),
            current_type_params: Vec::new(),
            loop_depth: 0,
        };
        checker.install_prelude();
        checker
    }

    fn into_output(self) -> TypeCheckOutput {
        let bindings = self
            .bindings
            .iter()
            .map(|binding| TypedBindingInfo {
                id: binding.id,
                symbol: binding.symbol,
                kind: binding.kind,
                ty: self.type_info_for(&binding.ty),
                package_item: self.package_items_by_binding.get(&binding.id).copied(),
                span: binding.span,
            })
            .collect();
        let expr_types = self
            .expr_types
            .iter()
            .map(|expr_type| ExprTypeInfo {
                expr_id: expr_type.expr_id,
                span: expr_type.span,
                ty: self.type_info_for(&expr_type.ty),
            })
            .collect();
        TypeCheckOutput {
            diagnostics: self.diagnostics,
            bindings,
            assignment_targets: self.assignment_targets,
            using_cleanups: self.using_cleanups,
            identifier_refs: self.identifier_refs,
            calls: self.calls,
            json_decode_schemas: self.json_decode_schemas,
            json_required_decode_schemas: self.json_required_decode_schemas,
            json_to_value_schemas: self.json_to_value_schemas,
            json_encode_typed_schemas: self.json_encode_typed_schemas,
            config_required_load_json_schemas: self.config_required_load_json_schemas,
            config_load_json_schemas: self.config_load_json_schemas,
            cli_parse_schemas: self.cli_parse_schemas,
            cli_parse_or_schemas: self.cli_parse_or_schemas,
            cli_parse_request_schemas: self.cli_parse_request_schemas,
            cli_parse_request_or_schemas: self.cli_parse_request_or_schemas,
            cli_usage_for_schemas: self.cli_usage_for_schemas,
            cli_usage_for_required_schemas: self.cli_usage_for_required_schemas,
            cli_help_for_schemas: self.cli_help_for_schemas,
            cli_help_for_required_schemas: self.cli_help_for_required_schemas,
            expr_types,
            symbols: self.symbols,
            package_opaque_types: self.package_opaque_items.into_iter().collect(),
        }
    }

    fn install_prelude(&mut self) {
        for builtin in prelude::builtins() {
            self.install_builtin(*builtin);
        }
    }

    fn install_internal_builtins(&mut self) {
        for builtin in prelude::internal_builtins() {
            self.install_builtin(*builtin);
        }
    }

    fn install_builtin(&mut self, builtin: prelude::Builtin) {
        let kind = match builtin.kind {
            BuiltinKind::Function => BindingKind::Function,
            BuiltinKind::Value => BindingKind::Immutable,
        };
        let ty = if builtin.id == BuiltinId::OptionNone {
            Type::OptionNone
        } else {
            Type::Builtin(builtin.id)
        };
        let symbol = self.symbol(builtin.name);
        self.insert_current(symbol, kind, ty, Span::default());
    }

    fn install_package_module_signatures(
        &mut self,
        signatures: &PackageSignatureEnvironment,
        environment: &PackageModuleSignatureEnvironment,
    ) {
        for record in &environment.records {
            let symbol = self.symbol(&record.name);
            self.package_records.insert(record.item, symbol);
            self.package_record_items.insert(symbol, record.item);
        }
        for enumeration in &environment.enums {
            let symbol = self.symbol(&enumeration.name);
            self.package_enums.insert(enumeration.item, symbol);
            self.package_enum_items.insert(symbol, enumeration.item);
            if signatures
                .enumeration(enumeration.item)
                .is_some_and(|signature| {
                    signature.name == "Value"
                        && signatures
                            .package_path(signature.package)
                            .is_some_and(|path| path == crate::std_package::JSON_PACKAGE)
                })
            {
                self.std_json_value_symbols.insert(symbol);
            }
        }
        for opaque in &environment.opaque_types {
            let symbol = self.symbol(&opaque.name);
            self.package_opaques.insert(opaque.item, symbol);
            self.package_opaque_items.insert(symbol, opaque.item);
            if let Some(signature) = signatures.opaque_type(opaque.item) {
                self.package_opaque_handle_facts
                    .insert(opaque.item, signature.handle_facts.clone());
            }
        }
        for function in &environment.functions {
            let symbol = self.symbol(&function.name);
            self.package_function_items.insert(symbol, function.item);
        }

        for visible in &environment.records {
            if visible.source == PackageSignatureSource::ModuleLocal {
                continue;
            }
            let Some(record) = signatures.record(visible.item) else {
                continue;
            };
            let symbol = self.symbol(&visible.name);
            let type_params = record
                .type_params
                .iter()
                .map(|param| self.symbol(param))
                .collect::<Vec<_>>();
            let fields = record
                .fields
                .iter()
                .map(|field| RecordField {
                    name: self.symbol(&field.name),
                    json_rename: field.json_rename.as_ref().map(|rename| self.symbol(rename)),
                    json_aliases: field
                        .json_aliases
                        .iter()
                        .map(|alias| self.symbol(alias))
                        .collect(),
                    json_validation: field.json_validation.clone(),
                    cli_name: field.cli_name.as_ref().map(|name| self.symbol(name)),
                    cli_short: field.cli_short.as_ref().map(|short| self.symbol(short)),
                    cli_position: field.cli_position,
                    cli_value_source: field.cli_value_source,
                    cli_aliases: field
                        .cli_aliases
                        .iter()
                        .map(|alias| self.symbol(alias))
                        .collect(),
                    cli_help: field.cli_help.as_ref().map(|help| self.symbol(help)),
                    cli_hidden: field.cli_hidden,
                    cli_subcommand: field.cli_subcommand,
                    type_name: None,
                    ty: Some(self.type_from_signature_info(&field.ty, signatures)),
                    span: field.span,
                })
                .collect();
            let cli_about = record.cli_about.as_ref().map(|about| self.symbol(about));
            self.records.insert(
                symbol,
                RecordDef {
                    span: record.span,
                    type_params,
                    json_deny_unknown_fields: record.json_deny_unknown_fields,
                    cli_about,
                    fields,
                },
            );
        }

        for visible in &environment.enums {
            if visible.source == PackageSignatureSource::ModuleLocal {
                continue;
            }
            let Some(enumeration) = signatures.enumeration(visible.item) else {
                continue;
            };
            let enum_symbol = self.symbol(&visible.name);
            let type_params = enumeration
                .type_params
                .iter()
                .map(|param| self.symbol(param))
                .collect::<Vec<_>>();
            let variants = enumeration
                .variants
                .iter()
                .map(|variant| {
                    let variant_symbol = self.symbol(&variant.name);
                    let qualified = self.symbol(&format!("{}::{}", visible.name, variant.name));
                    let kind = if variant.payload.is_some() {
                        BindingKind::Function
                    } else {
                        BindingKind::Immutable
                    };
                    self.insert_current(
                        qualified,
                        kind,
                        Type::EnumConstructor {
                            enum_name: enum_symbol,
                            enum_item: Some(enumeration.item),
                            variant_name: variant_symbol,
                        },
                        variant.span,
                    );
                    EnumVariantDef {
                        name: variant_symbol,
                        json_rename: variant
                            .json_rename
                            .as_ref()
                            .map(|rename| self.symbol(rename)),
                        json_aliases: variant
                            .json_aliases
                            .iter()
                            .map(|alias| self.symbol(alias))
                            .collect(),
                        cli_name: variant.cli_name.as_ref().map(|name| self.symbol(name)),
                        cli_aliases: variant
                            .cli_aliases
                            .iter()
                            .map(|alias| self.symbol(alias))
                            .collect(),
                        cli_about: variant.cli_about.as_ref().map(|about| self.symbol(about)),
                        cli_hidden: variant.cli_hidden,
                        payload: None,
                        payload_ty: variant
                            .payload
                            .as_ref()
                            .map(|payload| self.type_from_signature_info(payload, signatures)),
                        span: variant.span,
                    }
                })
                .collect();
            let cli_about = enumeration
                .cli_about
                .as_ref()
                .map(|about| self.symbol(about));
            self.enums.insert(
                enum_symbol,
                EnumDef {
                    span: enumeration.span,
                    type_params,
                    cli_about,
                    variants,
                },
            );
        }

        for visible in &environment.functions {
            if visible.source == PackageSignatureSource::ModuleLocal {
                continue;
            }
            let Some(function) = signatures.function(visible.item) else {
                continue;
            };
            let symbol = self.symbol(&visible.name);
            let sig = FunctionSig {
                type_params: function
                    .type_params
                    .iter()
                    .map(|param| self.symbol(param))
                    .collect(),
                params: function
                    .params
                    .iter()
                    .map(|param| {
                        param
                            .ty
                            .as_ref()
                            .map(|ty| self.type_from_signature_info(ty, signatures))
                            .unwrap_or(Type::Unknown(self.fresh_unknown()))
                    })
                    .collect(),
                ret: Box::new(
                    function
                        .ret
                        .as_ref()
                        .map(|ty| self.type_from_signature_info(ty, signatures))
                        .unwrap_or(Type::Unknown(self.fresh_unknown())),
                ),
            };
            let binding = self.insert_current(
                symbol,
                BindingKind::Function,
                Type::Function(sig),
                function.span,
            );
            if function.name == "decode_or"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::JSON_PACKAGE)
            {
                self.std_json_decode_or_bindings.insert(binding);
            }
            if function.name == "decode"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::JSON_PACKAGE)
            {
                self.std_json_decode_bindings.insert(binding);
            }
            if function.name == "to_value"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::JSON_PACKAGE)
            {
                self.std_json_to_value_bindings.insert(binding);
            }
            if function.name == "encode_typed"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::JSON_PACKAGE)
            {
                self.std_json_encode_typed_bindings.insert(binding);
            }
            if function.name == "load_json_or"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::CONFIG_PACKAGE)
            {
                self.std_config_load_json_or_bindings.insert(binding);
            }
            if function.name == "load_json"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::CONFIG_PACKAGE)
            {
                self.std_config_load_json_bindings.insert(binding);
            }
            if function.name == "parse"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::CLI_PACKAGE)
            {
                self.std_cli_parse_bindings.insert(binding);
            }
            if function.name == "parse_or"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::CLI_PACKAGE)
            {
                self.std_cli_parse_or_bindings.insert(binding);
            }
            if function.name == "parse_request"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::CLI_PACKAGE)
            {
                self.std_cli_parse_request_bindings.insert(binding);
            }
            if function.name == "parse_request_or"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::CLI_PACKAGE)
            {
                self.std_cli_parse_request_or_bindings.insert(binding);
            }
            if function.name == "usage_for"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::CLI_PACKAGE)
            {
                self.std_cli_usage_for_bindings.insert(binding);
            }
            if function.name == "usage_for_required"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::CLI_PACKAGE)
            {
                self.std_cli_usage_for_required_bindings.insert(binding);
            }
            if function.name == "help_for"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::CLI_PACKAGE)
            {
                self.std_cli_help_for_bindings.insert(binding);
            }
            if function.name == "help_for_required"
                && signatures
                    .package_path(function.package)
                    .is_some_and(|path| path == crate::std_package::CLI_PACKAGE)
            {
                self.std_cli_help_for_required_bindings.insert(binding);
            }
            self.package_items_by_binding.insert(binding, visible.item);
            self.package_function_bindings_by_item
                .insert(visible.item, binding);
            self.package_function_param_modes.insert(
                binding,
                function.params.iter().map(|param| param.mode).collect(),
            );
        }
    }

    fn type_from_signature_info(
        &mut self,
        ty: &TypeInfo,
        signatures: &PackageSignatureEnvironment,
    ) -> Type {
        match ty {
            TypeInfo::Int => Type::Int,
            TypeInfo::Bool => Type::Bool,
            TypeInfo::String => Type::String,
            TypeInfo::Unit => Type::Unit,
            TypeInfo::GenericParam(symbol) => {
                Type::GenericParam(self.symbol(signatures.symbols.resolve(*symbol)))
            }
            TypeInfo::Record(symbol, args) => Type::Record(
                self.symbol(signatures.symbols.resolve(*symbol)),
                args.iter()
                    .map(|arg| self.type_from_signature_info(arg, signatures))
                    .collect(),
            ),
            TypeInfo::PackageRecord { symbol, item, args } => {
                let symbol = self.package_records.get(item).copied().unwrap_or_else(|| {
                    self.install_transitive_package_record(*item, signatures);
                    self.package_records
                        .get(item)
                        .copied()
                        .unwrap_or_else(|| self.symbol(signatures.symbols.resolve(*symbol)))
                });
                Type::Record(
                    symbol,
                    args.iter()
                        .map(|arg| self.type_from_signature_info(arg, signatures))
                        .collect(),
                )
            }
            TypeInfo::Enum { symbol, args } => Type::Enum(
                self.symbol(signatures.symbols.resolve(*symbol)),
                args.iter()
                    .map(|arg| self.type_from_signature_info(arg, signatures))
                    .collect(),
            ),
            TypeInfo::PackageEnum { symbol, item, args } => {
                let symbol = self
                    .package_enums
                    .get(item)
                    .copied()
                    .unwrap_or_else(|| self.symbol(signatures.symbols.resolve(*symbol)));
                Type::Enum(
                    symbol,
                    args.iter()
                        .map(|arg| self.type_from_signature_info(arg, signatures))
                        .collect(),
                )
            }
            TypeInfo::PackageOpaque { symbol, item } => {
                let symbol = self
                    .package_opaques
                    .get(item)
                    .copied()
                    .unwrap_or_else(|| self.symbol(signatures.symbols.resolve(*symbol)));
                Type::Opaque(symbol)
            }
            TypeInfo::List(item) => {
                Type::List(Box::new(self.type_from_signature_info(item, signatures)))
            }
            TypeInfo::Map(key, value) => Type::Map(
                Box::new(self.type_from_signature_info(key, signatures)),
                Box::new(self.type_from_signature_info(value, signatures)),
            ),
            TypeInfo::Option(item) => {
                Type::Option(Box::new(self.type_from_signature_info(item, signatures)))
            }
            TypeInfo::Result(ok, err) => Type::Result(
                Box::new(self.type_from_signature_info(ok, signatures)),
                Box::new(self.type_from_signature_info(err, signatures)),
            ),
            TypeInfo::EnumConstructor {
                enum_symbol,
                enum_item,
                variant,
            } => Type::EnumConstructor {
                enum_name: self.symbol(signatures.symbols.resolve(*enum_symbol)),
                enum_item: *enum_item,
                variant_name: self.symbol(signatures.symbols.resolve(*variant)),
            },
            TypeInfo::Function(function) => Type::Function(FunctionSig {
                type_params: Vec::new(),
                params: function
                    .params
                    .iter()
                    .map(|param| self.type_from_signature_info(param, signatures))
                    .collect(),
                ret: Box::new(self.type_from_signature_info(&function.ret, signatures)),
            }),
            TypeInfo::Builtin(builtin) => Type::Builtin(*builtin),
            TypeInfo::Unknown => Type::Unknown(self.fresh_unknown()),
            TypeInfo::Error => Type::Error,
        }
    }

    fn install_transitive_package_record(
        &mut self,
        item: crate::identity::PackageItemId,
        signatures: &PackageSignatureEnvironment,
    ) {
        if self.package_records.contains_key(&item) {
            return;
        }
        let Some(record) = signatures.record(item).cloned() else {
            return;
        };
        let display_name = signatures
            .package_path(record.package)
            .map(|path| format!("{path}::{}", record.name))
            .unwrap_or_else(|| record.name.clone());
        let symbol = self.symbol(&display_name);
        self.package_records.insert(item, symbol);
        self.package_record_items.insert(symbol, item);

        let type_params = record
            .type_params
            .iter()
            .map(|param| self.symbol(param))
            .collect::<Vec<_>>();
        let cli_about = record.cli_about.as_ref().map(|about| self.symbol(about));
        self.records.insert(
            symbol,
            RecordDef {
                span: record.span,
                type_params: type_params.clone(),
                json_deny_unknown_fields: record.json_deny_unknown_fields,
                cli_about,
                fields: Vec::new(),
            },
        );

        let fields = record
            .fields
            .iter()
            .map(|field| RecordField {
                name: self.symbol(&field.name),
                json_rename: field.json_rename.as_ref().map(|rename| self.symbol(rename)),
                json_aliases: field
                    .json_aliases
                    .iter()
                    .map(|alias| self.symbol(alias))
                    .collect(),
                json_validation: field.json_validation.clone(),
                cli_name: field.cli_name.as_ref().map(|name| self.symbol(name)),
                cli_short: field.cli_short.as_ref().map(|short| self.symbol(short)),
                cli_position: field.cli_position,
                cli_value_source: field.cli_value_source,
                cli_aliases: field
                    .cli_aliases
                    .iter()
                    .map(|alias| self.symbol(alias))
                    .collect(),
                cli_help: field.cli_help.as_ref().map(|help| self.symbol(help)),
                cli_hidden: field.cli_hidden,
                cli_subcommand: field.cli_subcommand,
                type_name: None,
                ty: Some(self.type_from_signature_info(&field.ty, signatures)),
                span: field.span,
            })
            .collect();
        if let Some(record) = self.records.get_mut(&symbol) {
            record.fields = fields;
        }
    }

    fn check_scope_statements(&mut self, statements: &[Stmt]) {
        let functions = self.predeclare_functions(statements);
        self.check_recursive_requirements(statements, &functions);
        for statement in statements {
            match statement {
                Stmt::RecordDecl(record) => self.check_record_decl(record),
                Stmt::EnumDecl(enumeration) => self.check_enum_decl(enumeration),
                Stmt::OpaqueTypeDecl(_) => {}
                Stmt::FuncDecl(func) => self.check_func_decl(func, &functions),
                _ => self.check_stmt(statement),
            }
        }
    }

    fn check_block(&mut self, block: &Block) {
        self.push_scope(false);
        self.check_scope_statements(&block.statements);
        self.pop_scope();
    }

    fn check_value_block(&mut self, block: &ValueBlock) -> Type {
        self.check_value_block_with_expected(block, None)
    }

    fn check_value_block_with_expected(
        &mut self,
        block: &ValueBlock,
        expected: Option<Type>,
    ) -> Type {
        self.push_scope(false);
        let ty = self.check_value_block_contents(block, expected);
        self.pop_scope();
        ty
    }

    fn check_value_block_contents(&mut self, block: &ValueBlock, expected: Option<Type>) -> Type {
        let functions = self.predeclare_functions(&block.statements);
        self.check_recursive_requirements(&block.statements, &functions);
        for statement in &block.statements {
            match statement {
                Stmt::FuncDecl(func) => self.check_func_decl(func, &functions),
                _ => self.check_stmt(statement),
            }
        }
        if block.terminal_return {
            self.record_expr_type(block.expr.id(), block.expr.span(), Type::Unit);
            return Type::Never;
        }
        self.check_expr_with_expected(&block.expr, expected)
    }

    fn check_stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Assign(stmt) => self.check_assign(stmt),
            Stmt::RecordDecl(_) => {}
            Stmt::EnumDecl(_) => {}
            Stmt::OpaqueTypeDecl(_) => {}
            Stmt::FuncDecl(_) => {}
            Stmt::If(stmt) => {
                let condition = self.check_expr(&stmt.condition);
                self.require_exact(&condition, &Type::Bool, stmt.condition.span(), "T001");
                self.check_block(&stmt.then_branch);
                if let Some(else_branch) = &stmt.else_branch {
                    self.check_block(else_branch);
                }
            }
            Stmt::While(stmt) => {
                let condition = self.check_expr(&stmt.condition);
                self.require_exact(&condition, &Type::Bool, stmt.condition.span(), "T001");
                self.loop_depth += 1;
                self.check_block(&stmt.body);
                self.loop_depth -= 1;
            }
            Stmt::For(stmt) => self.check_for_stmt(stmt),
            Stmt::Using(stmt) => self.check_using_stmt(stmt),
            Stmt::Break(stmt) => self.check_loop_control("break", stmt.span),
            Stmt::Continue(stmt) => self.check_loop_control("continue", stmt.span),
            Stmt::Return(stmt) => self.check_return(stmt),
            Stmt::Expr(stmt) => {
                self.check_expr(&stmt.expr);
            }
        }
    }

    fn check_for_stmt(&mut self, stmt: &ForStmt) {
        let iterable_ty = self.check_expr(&stmt.iterable);
        let item_ty = match self.resolve_type(&iterable_ty) {
            Type::List(item_ty) => *item_ty,
            Type::Unknown(_) => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        stmt.iterable.span(),
                    )
                    .with_suggestion(
                        "annotate the iterable as List[T] before using it in a `for` loop",
                    ),
                );
                Type::Error
            }
            Type::Error => Type::Error,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`for` expects List[T] after `in` but found {}",
                        self.type_label(&other)
                    ),
                    stmt.iterable.span(),
                ));
                Type::Error
            }
        };

        self.push_scope(false);
        let item_name = self.symbol(&stmt.item);
        self.insert_current(item_name, BindingKind::Immutable, item_ty, stmt.item_span);
        self.loop_depth += 1;
        self.check_scope_statements(&stmt.body.statements);
        self.loop_depth -= 1;
        self.pop_scope();
    }

    fn check_using_stmt(&mut self, stmt: &UsingStmt) {
        let return_err = self
            .current_using_result_return(stmt.span)
            .map(|(_, err)| err);
        let value_ty = self.check_expr(&stmt.value);
        let cleanup = return_err
            .as_ref()
            .and_then(|err_ty| self.resolve_using_cleanup(&value_ty, err_ty, stmt));

        self.push_scope(false);
        let name = self.symbol(&stmt.name);
        let binding = self.insert_current(name, BindingKind::Immutable, value_ty, stmt.name_span);
        if let Some(cleanup) = cleanup {
            self.using_cleanups.push(TypedUsingCleanupInfo {
                stmt_id: stmt.id,
                name: cleanup.name,
                callee: cleanup.callee,
                span: cleanup.span,
            });
        }
        self.using_bindings.insert(binding, stmt.name_span);
        self.check_scope_statements(&stmt.body.statements);
        self.using_bindings.remove(&binding);
        self.pop_scope();
    }

    fn resolve_using_cleanup(
        &mut self,
        value_ty: &Type,
        return_err: &Type,
        stmt: &UsingStmt,
    ) -> Option<TypedUsingCleanupInfo> {
        let resolved = self.resolve_type(value_ty);
        let Type::Opaque(symbol) = resolved else {
            self.diagnostics.push(Diagnostic::new(
                "T027",
                format!(
                    "`using` expects a runtime-backed closeable opaque handle but found {}",
                    self.type_label(&resolved)
                ),
                stmt.value.span(),
            ));
            return None;
        };
        let Some(item) = self.package_opaque_items.get(&symbol).copied() else {
            self.diagnostics.push(Diagnostic::new(
                "T027",
                "`using` expects a package opaque handle with cleanup metadata",
                stmt.value.span(),
            ));
            return None;
        };
        let facts = self
            .package_opaque_handle_facts
            .get(&item)
            .cloned()
            .unwrap_or_default();
        if !facts.runtime_backed || !facts.closeable {
            self.diagnostics.push(Diagnostic::new(
                "T027",
                "`using` expects a runtime-backed closeable opaque handle",
                stmt.value.span(),
            ));
            return None;
        }
        let Some(close_item) = facts.close_function else {
            self.diagnostics.push(Diagnostic::new(
                "T027",
                "`using` handle metadata does not name a close function",
                stmt.value.span(),
            ));
            return None;
        };
        let Some(close_binding_id) = self
            .package_function_bindings_by_item
            .get(&close_item)
            .copied()
        else {
            self.diagnostics.push(
                Diagnostic::new(
                    "T027",
                    "`using` close function is not visible in this module",
                    stmt.value.span(),
                )
                .with_suggestion(
                    "import the package that defines the handle and its close function",
                ),
            );
            return None;
        };
        let close_binding = self.binding_by_id(close_binding_id).cloned()?;
        let Type::Function(sig) = self.resolve_type(&close_binding.ty) else {
            self.diagnostics.push(Diagnostic::new(
                "T027",
                "`using` close metadata does not point to a function",
                close_binding.span,
            ));
            return None;
        };
        if sig.params.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T027",
                "`using` close function must accept exactly one handle argument",
                close_binding.span,
            ));
            return None;
        }
        let mode = self
            .package_function_param_modes
            .get(&close_binding_id)
            .and_then(|modes| modes.first())
            .copied()
            .unwrap_or(PackageInterfaceParamMode::Borrow);
        if mode != PackageInterfaceParamMode::Consume {
            self.diagnostics.push(Diagnostic::new(
                "T027",
                "`using` close function must consume the handle argument",
                close_binding.span,
            ));
            return None;
        }
        let handle_ty = Type::Opaque(symbol);
        if let Err(message) = self.unify(sig.params[0].clone(), handle_ty) {
            self.diagnostics
                .push(Diagnostic::new("T027", message, close_binding.span));
            return None;
        }
        let expected_ret = Type::Result(Box::new(Type::Unit), Box::new(return_err.clone()));
        if let Err(message) = self.unify((*sig.ret).clone(), expected_ret) {
            self.diagnostics
                .push(Diagnostic::new("T027", message, close_binding.span));
            return None;
        }
        let callee = self
            .package_items_by_binding
            .get(&close_binding_id)
            .copied()
            .map(|item| TypedCalleeInfo::PackageItem {
                binding: close_binding_id,
                item,
            })
            .unwrap_or(TypedCalleeInfo::Binding(close_binding_id));
        Some(TypedUsingCleanupInfo {
            stmt_id: stmt.id,
            name: close_binding.symbol,
            callee,
            span: close_binding.span,
        })
    }

    fn check_loop_control(&mut self, keyword: &str, span: Span) {
        if self.loop_depth == 0 {
            self.diagnostics.push(Diagnostic::new(
                "T025",
                format!("`{keyword}` is allowed only inside a loop"),
                span,
            ));
        }
    }

    fn check_return(&mut self, stmt: &ReturnStmt) {
        let Some(return_ty) = self.current_function_returns.last().cloned() else {
            self.check_expr(&stmt.value);
            self.diagnostics.push(Diagnostic::new(
                "T024",
                "`return` is allowed only inside a function",
                stmt.span,
            ));
            return;
        };
        self.check_expr_with_expected(&stmt.value, Some(return_ty));
    }

    fn check_assign(&mut self, stmt: &AssignStmt) {
        let annotation_ty = stmt
            .type_name
            .as_ref()
            .map(|type_name| self.type_from_expr(type_name, stmt.span));
        let value_ty = match annotation_ty.clone() {
            Some(expected) => self.check_expr_with_expected(&stmt.value, Some(expected)),
            None => self.check_expr(&stmt.value),
        };
        let binding_ty = annotation_ty.unwrap_or_else(|| value_ty.clone());
        let name = self.symbol(&stmt.name);
        if stmt.mutable {
            let binding = self.insert_current(name, BindingKind::Mutable, binding_ty, stmt.span);
            self.assignment_targets.push(TypedAssignmentTarget {
                stmt_id: stmt.id,
                name,
                span: stmt.span,
                binding,
                is_update: false,
            });
            return;
        }

        if let Some(binding) = self.lookup_in_current_function(name).cloned() {
            if stmt.type_name.is_some() {
                self.diagnostics.push(Diagnostic::new(
                    "T014",
                    "type annotations are allowed only on new local bindings",
                    stmt.span,
                ));
            }
            if binding.kind == BindingKind::Mutable {
                self.require_exact(&binding.ty, &value_ty, stmt.span, "T002");
            }
            self.assignment_targets.push(TypedAssignmentTarget {
                stmt_id: stmt.id,
                name,
                span: stmt.span,
                binding: binding.id,
                is_update: true,
            });
            if binding.kind == BindingKind::Mutable {
                self.clear_consumed_binding(binding.id);
            }
            return;
        }

        if self.lookup_beyond_current_function(name).is_none() {
            let binding = self.insert_current(name, BindingKind::Immutable, binding_ty, stmt.span);
            self.assignment_targets.push(TypedAssignmentTarget {
                stmt_id: stmt.id,
                name,
                span: stmt.span,
                binding,
                is_update: false,
            });
        }
    }

    fn check_func_decl(&mut self, func: &FuncDecl, local_functions: &HashMap<Symbol, FunctionSig>) {
        let name = self.symbol(&func.name);
        let Some(sig) = local_functions.get(&name).cloned() else {
            return;
        };

        self.push_scope(true);
        self.current_function_returns.push((*sig.ret).clone());
        self.current_type_params.push(sig.type_params.clone());
        let outer_loop_depth = std::mem::replace(&mut self.loop_depth, 0);
        for (param, param_ty) in func.params.iter().zip(sig.params.iter().cloned()) {
            let name = self.symbol(&param.name);
            self.insert_current(name, BindingKind::Parameter, param_ty, param.span);
        }
        self.check_value_block_contents(&func.body, Some((*sig.ret).clone()));
        self.loop_depth = outer_loop_depth;
        self.current_type_params.pop();
        self.current_function_returns.pop();
        self.pop_scope();

        let resolved_params: Vec<Type> =
            sig.params.iter().map(|ty| self.resolve_type(ty)).collect();
        let resolved_ret = self.resolve_type(&sig.ret);
        if resolved_params.iter().any(Type::is_unknown) || resolved_ret.is_unknown() {
            self.diagnostics.push(
                Diagnostic::new(
                    "E005",
                    "type annotation required because inference is not unique",
                    func.span,
                )
                .with_suggestion(
                    "add parameter or return type annotations until the function signature is unique",
                ),
            );
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        self.check_expr_with_expected(expr, None)
    }

    fn check_call_callee(&mut self, callee: &Expr) -> Type {
        if let Expr::Ident(ident) = callee {
            let name = self.symbol(&ident.name);
            if let Some(binding) = self.lookup(name).cloned()
                && matches!(binding.ty, Type::EnumConstructor { .. })
            {
                self.identifier_refs.push(TypedIdentifier {
                    expr_id: ident.id,
                    name,
                    span: ident.span,
                    binding: binding.id,
                });
                return self.record_expr_type(ident.id, ident.span, binding.ty);
            }
        }
        self.check_expr(callee)
    }

    fn check_expr_with_expected(&mut self, expr: &Expr, expected: Option<Type>) -> Type {
        let span = expr.span();
        let ty = match expr {
            Expr::Int(_) => self.apply_expected(Type::Int, expected, expr.span()),
            Expr::Bool(_) => self.apply_expected(Type::Bool, expected, expr.span()),
            Expr::String(_) => self.apply_expected(Type::String, expected, expr.span()),
            Expr::Unit(_) => self.apply_expected(Type::Unit, expected, expr.span()),
            Expr::Ident(expr) => {
                let name = self.symbol(&expr.name);
                if let Some(binding) = self.lookup(name).cloned() {
                    self.identifier_refs.push(TypedIdentifier {
                        expr_id: expr.id,
                        name,
                        span: expr.span,
                        binding: binding.id,
                    });
                    self.check_consumed_binding_use(&binding, expr.span);
                    if matches!(binding.ty, Type::OptionNone) {
                        let ty = self.check_option_none(expected, expr.span);
                        return self.record_expr_type(expr.id, span, ty);
                    }
                    if let Type::EnumConstructor {
                        enum_name,
                        enum_item: _,
                        variant_name,
                    } = binding.ty
                    {
                        let ty = self.check_user_enum_constructor(
                            expr.span,
                            expected,
                            enum_name,
                            variant_name,
                            &[],
                        );
                        return self.record_expr_type(expr.id, span, ty);
                    }
                    self.apply_expected(binding.ty, expected, expr.span)
                } else {
                    if let Some((enum_name, variant_name)) = split_variant_name(&expr.name) {
                        self.diagnose_unknown_enum_variant(enum_name, variant_name, expr.span);
                    }
                    Type::Error
                }
            }
            Expr::ListLit(expr) => self.check_list_lit(expr, expected),
            Expr::Index(expr) => self.check_index_expr(expr, expected),
            Expr::RecordLit(expr) => {
                let ty = self.check_record_lit(expr, expected.as_ref());
                self.apply_expected(ty, expected, expr.span)
            }
            Expr::Field(expr) => {
                let ty = self.check_field_expr(expr);
                self.apply_expected(ty, expected, expr.span)
            }
            Expr::RecordUpdate(expr) => {
                let ty = self.check_record_update(expr);
                self.apply_expected(ty, expected, expr.span)
            }
            Expr::Unary(expr) => {
                let ty = match expr.op {
                    UnaryOp::Neg => self.check_expr_with_expected(&expr.expr, Some(Type::Int)),
                    UnaryOp::Not => self.check_expr_with_expected(&expr.expr, Some(Type::Bool)),
                };
                match expr.op {
                    UnaryOp::Neg => {
                        self.require_exact(&ty, &Type::Int, expr.span, "T001");
                        self.apply_expected(Type::Int, expected, expr.span)
                    }
                    UnaryOp::Not => {
                        self.require_exact(&ty, &Type::Bool, expr.span, "T001");
                        self.apply_expected(Type::Bool, expected, expr.span)
                    }
                }
            }
            Expr::Binary(expr) => match expr.op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                    let left = self.check_expr_with_expected(&expr.left, Some(Type::Int));
                    let right = self.check_expr_with_expected(&expr.right, Some(Type::Int));
                    self.require_exact(&left, &Type::Int, expr.left.span(), "T001");
                    self.require_exact(&right, &Type::Int, expr.right.span(), "T001");
                    self.apply_expected(Type::Int, expected, expr.span)
                }
                BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => {
                    let left = self.check_expr_with_expected(&expr.left, Some(Type::Int));
                    let right = self.check_expr_with_expected(&expr.right, Some(Type::Int));
                    self.require_exact(&left, &Type::Int, expr.left.span(), "T001");
                    self.require_exact(&right, &Type::Int, expr.right.span(), "T001");
                    self.apply_expected(Type::Bool, expected, expr.span)
                }
                BinaryOp::EqEq | BinaryOp::BangEq => {
                    let left = self.check_expr(&expr.left);
                    let right = self.check_expr_with_expected(&expr.right, Some(left.clone()));
                    self.require_exact(&left, &right, expr.span, "T002");
                    let resolved = self.resolve_type(&left);
                    if !matches!(
                        resolved,
                        Type::Int | Type::Bool | Type::String | Type::Unknown(_)
                    ) {
                        self.diagnostics.push(Diagnostic::new(
                            "T003",
                            "equality is allowed only for Int, Bool, and String",
                            expr.span,
                        ));
                    }
                    self.apply_expected(Type::Bool, expected, expr.span)
                }
                BinaryOp::And | BinaryOp::Or => {
                    let left = self.check_expr_with_expected(&expr.left, Some(Type::Bool));
                    let right = self.check_expr_with_expected(&expr.right, Some(Type::Bool));
                    self.require_exact(&left, &Type::Bool, expr.left.span(), "T001");
                    self.require_exact(&right, &Type::Bool, expr.right.span(), "T001");
                    self.apply_expected(Type::Bool, expected, expr.span)
                }
            },
            Expr::Call(expr) => {
                let callee_ty = self.check_call_callee(&expr.callee);
                let ty = match self.resolve_type(&callee_ty) {
                    Type::Builtin(
                        BuiltinId::Print
                        | BuiltinId::Println
                        | BuiltinId::Eprint
                        | BuiltinId::Eprintln,
                    ) => {
                        if expr.args.len() != 1 {
                            self.diagnostics.push(Diagnostic::new(
                                "T004",
                                format!("expected 1 arguments but found {}", expr.args.len()),
                                expr.span,
                            ));
                            Type::Error
                        } else {
                            let arg_ty =
                                self.check_expr_with_expected(&expr.args[0], expected.clone());
                            let arg_ty = self.resolve_type(&arg_ty);
                            match arg_ty {
                                Type::Int | Type::Bool | Type::String => {
                                    self.apply_expected(arg_ty, expected, expr.span)
                                }
                                Type::Unknown(_) => {
                                    self.diagnostics.push(
                                        Diagnostic::new(
                                            "E005",
                                            "type annotation required because inference is not unique",
                                            expr.span,
                                        )
                                        .with_suggestion(
                                            "annotate the argument as Int, Bool, or String before calling `print`, `println`, `eprint`, or `eprintln`",
                                        ),
                                    );
                                    Type::Error
                                }
                                Type::Error => Type::Error,
                                other => {
                                    let builtin_name = match self.resolve_type(&callee_ty) {
                                        Type::Builtin(builtin) => Self::builtin_name(builtin),
                                        _ => unreachable!("matched builtin branch"),
                                    };
                                    self.diagnostics.push(Diagnostic::new(
                                        "T006",
                                        format!(
                                            "`{builtin_name}` accepts only Int, Bool, or String but found {}",
                                            self.type_label(&other)
                                        ),
                                        expr.span,
                                    ));
                                    Type::Error
                                }
                            }
                        }
                    }
                    Type::Builtin(BuiltinId::Len | BuiltinId::IsEmpty) => {
                        let builtin = match self.resolve_type(&callee_ty) {
                            Type::Builtin(builtin) => builtin,
                            _ => unreachable!("matched builtin branch"),
                        };
                        if expr.args.len() != 1 {
                            self.diagnostics.push(Diagnostic::new(
                                "T004",
                                format!("expected 1 arguments but found {}", expr.args.len()),
                                expr.span,
                            ));
                            Type::Error
                        } else {
                            let arg_ty = self.check_expr(&expr.args[0]);
                            match self.resolve_type(&arg_ty) {
                                Type::List(_) | Type::Map(_, _)
                                    if matches!(builtin, BuiltinId::Len | BuiltinId::IsEmpty) =>
                                {
                                    let ret = match builtin {
                                        BuiltinId::Len => Type::Int,
                                        BuiltinId::IsEmpty => Type::Bool,
                                        _ => unreachable!("matched collection query builtin"),
                                    };
                                    self.apply_expected(ret, expected, expr.span)
                                }
                                Type::String if builtin == BuiltinId::IsEmpty => {
                                    self.apply_expected(Type::Bool, expected, expr.span)
                                }
                                Type::Unknown(_) => {
                                    let suggestion = match builtin {
                                        BuiltinId::Len => {
                                            "annotate the argument as List[T] or Map[K, V] before calling `len`"
                                        }
                                        BuiltinId::IsEmpty => {
                                            "annotate the argument as String, List[T], or Map[K, V] before calling `is_empty`"
                                        }
                                        _ => unreachable!("matched collection query builtin"),
                                    };
                                    self.diagnostics.push(
                                        Diagnostic::new(
                                            "E005",
                                            "type annotation required because inference is not unique",
                                            expr.span,
                                        )
                                        .with_suggestion(suggestion),
                                    );
                                    Type::Error
                                }
                                Type::Error => Type::Error,
                                other => {
                                    let expected_message = match builtin {
                                        BuiltinId::Len => "List[T] or Map[K, V]",
                                        BuiltinId::IsEmpty => "String, List[T], or Map[K, V]",
                                        _ => unreachable!("matched collection query builtin"),
                                    };
                                    self.diagnostics.push(Diagnostic::new(
                                        "T006",
                                        format!(
                                            "`{}` expects {expected_message} as its first argument but found {}",
                                            Self::builtin_name(builtin),
                                            self.type_label(&other),
                                        ),
                                        expr.span,
                                    ));
                                    Type::Error
                                }
                            }
                        }
                    }
                    Type::Builtin(BuiltinId::Push) => {
                        if expr.args.len() != 2 {
                            self.diagnostics.push(Diagnostic::new(
                                "T004",
                                format!("expected 2 arguments but found {}", expr.args.len()),
                                expr.span,
                            ));
                            Type::Error
                        } else {
                            let item_expected = Type::Unknown(self.fresh_unknown());
                            let list_ty =
                                self.check_list_receiver_type(&expr.args[0], Some(item_expected));
                            match self.resolve_type(&list_ty) {
                                Type::List(item_ty) => {
                                    self.check_expr_with_expected(
                                        &expr.args[1],
                                        Some((*item_ty).clone()),
                                    );
                                    self.apply_expected(Type::List(item_ty), expected, expr.span)
                                }
                                Type::Error => Type::Error,
                                other => {
                                    self.diagnostics.push(Diagnostic::new(
                                        "T006",
                                        format!(
                                            "`push` expects List[T] as its first argument but found {}",
                                            self.type_label(&other)
                                        ),
                                        expr.span,
                                    ));
                                    Type::Error
                                }
                            }
                        }
                    }
                    Type::Builtin(BuiltinId::Get) => self.check_get_builtin(expr, expected),
                    Type::Builtin(BuiltinId::Set) => {
                        if expr.args.len() != 3 {
                            self.diagnostics.push(Diagnostic::new(
                                "T004",
                                format!("expected 3 arguments but found {}", expr.args.len()),
                                expr.span,
                            ));
                            Type::Error
                        } else {
                            let expected = expected.map(|ty| self.resolve_type(&ty));
                            let item_expected = match expected.as_ref() {
                                Some(Type::List(item)) => *item.clone(),
                                _ => Type::Unknown(self.fresh_unknown()),
                            };
                            let list_ty =
                                self.check_list_receiver_type(&expr.args[0], Some(item_expected));
                            self.check_expr_with_expected(&expr.args[1], Some(Type::Int));
                            match self.resolve_type(&list_ty) {
                                Type::List(item_ty) => {
                                    self.check_expr_with_expected(
                                        &expr.args[2],
                                        Some((*item_ty).clone()),
                                    );
                                    let list_ty = Type::List(item_ty);
                                    match expected {
                                        Some(Type::List(_)) | None => list_ty,
                                        Some(expected) => {
                                            self.apply_expected(list_ty, Some(expected), expr.span)
                                        }
                                    }
                                }
                                Type::Error => Type::Error,
                                other => {
                                    self.diagnostics.push(Diagnostic::new(
                                        "T006",
                                        format!(
                                            "`set` expects List[T] as its first argument but found {}",
                                            self.type_label(&other)
                                        ),
                                        expr.span,
                                    ));
                                    Type::Error
                                }
                            }
                        }
                    }
                    Type::Builtin(BuiltinId::MapEmpty) => {
                        self.check_map_empty_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::Contains) => {
                        self.check_contains_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::Insert) => self.check_insert_builtin(expr, expected),
                    Type::Builtin(BuiltinId::Remove) => self.check_remove_builtin(expr, expected),
                    Type::Builtin(BuiltinId::Trim) => self.check_string_unary_builtin(
                        expr,
                        expected,
                        BuiltinId::Trim,
                        Type::String,
                    ),
                    Type::Builtin(BuiltinId::CharCount | BuiltinId::ByteLen) => {
                        let builtin = match self.resolve_type(&callee_ty) {
                            Type::Builtin(builtin) => builtin,
                            _ => unreachable!("matched builtin branch"),
                        };
                        self.check_string_unary_builtin(expr, expected, builtin, Type::Int)
                    }
                    Type::Builtin(BuiltinId::StartsWith | BuiltinId::EndsWith) => {
                        let builtin = match self.resolve_type(&callee_ty) {
                            Type::Builtin(builtin) => builtin,
                            _ => unreachable!("matched builtin branch"),
                        };
                        self.check_string_predicate_builtin(expr, expected, builtin)
                    }
                    Type::Builtin(BuiltinId::Replace) => self.check_string_binary_builtin(
                        expr,
                        expected,
                        BuiltinId::Replace,
                        Type::String,
                    ),
                    Type::Builtin(BuiltinId::Split) => self.check_string_pair_builtin(
                        expr,
                        expected,
                        BuiltinId::Split,
                        Type::List(Box::new(Type::String)),
                    ),
                    Type::Builtin(BuiltinId::Concat) => self.check_string_pair_builtin(
                        expr,
                        expected,
                        BuiltinId::Concat,
                        Type::String,
                    ),
                    Type::Builtin(BuiltinId::SliceChars) => {
                        self.check_slice_chars_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::ToString) => {
                        self.check_to_string_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::ParseInt) => {
                        self.check_parse_int_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::ParseBool) => {
                        self.check_parse_bool_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdPathJoin) => {
                        self.check_std_path_join_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdPathNormalize) => {
                        self.check_std_path_normalize_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdPathFileName) => {
                        self.check_std_path_file_name_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdPathWithFileName) => {
                        self.check_std_path_with_file_name_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdPathParent) => {
                        self.check_std_path_parent_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdPathStripPrefix) => {
                        self.check_std_path_strip_prefix_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdPathExtension) => {
                        self.check_std_path_extension_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdPathFileStem) => {
                        self.check_std_path_file_stem_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdPathWithExtension) => {
                        self.check_std_path_with_extension_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdPathIsAbsolute) => {
                        self.check_std_path_is_absolute_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdBytesSize) => {
                        self.check_std_bytes_unary_builtin(expr, expected, BuiltinId::StdBytesSize)
                    }
                    Type::Builtin(BuiltinId::StdBytesIsEmpty) => self
                        .check_std_bytes_unary_builtin(expr, expected, BuiltinId::StdBytesIsEmpty),
                    Type::Builtin(BuiltinId::StdBytesAt) => {
                        self.check_std_bytes_at_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdFsReadText) => {
                        self.check_std_fs_read_text_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdFsReadBytes) => {
                        self.check_std_fs_read_bytes_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdFsReadResourceText) => {
                        self.check_std_fs_read_resource_text_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdFsReadResourceBytes) => {
                        self.check_std_fs_read_resource_bytes_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdFsWriteText) => {
                        self.check_std_fs_write_text_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdFsWriteBytes) => {
                        self.check_std_fs_write_bytes_builtin(expr, expected)
                    }
                    Type::Builtin(
                        BuiltinId::StdFsOpenText
                        | BuiltinId::StdFsCreateText
                        | BuiltinId::StdFsAppendText,
                    ) => self.check_std_fs_open_text_handle_builtin(expr, expected),
                    Type::Builtin(BuiltinId::StdFsReadTextFrom) => {
                        self.check_std_fs_read_text_from_handle_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdFsWriteTextTo) => {
                        self.check_std_fs_write_text_to_handle_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdFsFlush | BuiltinId::StdFsClose) => {
                        self.check_std_fs_close_handle_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdFsReadDir | BuiltinId::StdFsReadDirRecursive) => {
                        self.check_std_fs_read_dir_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdFsDirectorySizeMetadata) => {
                        self.check_std_fs_directory_size_metadata_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdFsCanonicalize) => {
                        self.check_std_fs_canonicalize_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdFsFileSize) => {
                        self.check_std_fs_file_size_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdFsModifiedUnixMillis) => {
                        self.check_std_fs_modified_unix_millis_builtin(expr, expected)
                    }
                    Type::Builtin(
                        BuiltinId::StdFsCreateDir
                        | BuiltinId::StdFsCreateDirAll
                        | BuiltinId::StdFsRemoveFile
                        | BuiltinId::StdFsRemoveDir
                        | BuiltinId::StdFsRemoveDirAll,
                    ) => self.check_std_fs_unit_path_builtin(expr, expected),
                    Type::Builtin(
                        BuiltinId::StdFsCopyFile
                        | BuiltinId::StdFsCopyDirAll
                        | BuiltinId::StdFsMoveDirAll
                        | BuiltinId::StdFsRename,
                    ) => self.check_std_fs_copy_file_builtin(expr, expected),
                    Type::Builtin(
                        BuiltinId::StdFsExists | BuiltinId::StdFsIsFile | BuiltinId::StdFsIsDir,
                    ) => self.check_std_fs_metadata_bool_builtin(expr, expected),
                    Type::Builtin(BuiltinId::StdEnvGetVar) => {
                        self.check_std_env_get_var_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdEnvArgs) => {
                        self.check_std_env_args_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdEnvCurrentDir) => {
                        self.check_std_env_current_dir_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdEnvTempDir) => {
                        self.check_std_env_temp_dir_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdProcessRun) => {
                        self.check_std_process_run_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdTimeNowUnixMillis) => {
                        self.check_std_time_now_unix_millis_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdHashSha256Hex) => {
                        self.check_std_hash_sha256_hex_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdTestAssertTrue) => {
                        self.check_std_test_assert_true_builtin(expr, expected)
                    }
                    Type::Builtin(
                        BuiltinId::StdTestAssertEqInt
                        | BuiltinId::StdTestAssertEqBool
                        | BuiltinId::StdTestAssertEqString,
                    ) => {
                        let builtin = match self.resolve_type(&callee_ty) {
                            Type::Builtin(builtin) => builtin,
                            _ => unreachable!("matched builtin branch"),
                        };
                        self.check_std_test_assert_eq_builtin(expr, expected, builtin)
                    }
                    Type::Builtin(BuiltinId::StdMapKeys) => {
                        self.check_std_map_keys_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdMapValues) => {
                        self.check_std_map_values_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdJsonParse) => {
                        self.check_std_json_parse_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::StdJsonEncode) => {
                        self.check_std_json_single_value_builtin(expr, expected, "encode")
                    }
                    Type::Builtin(BuiltinId::StdJsonNumberAsInt) => {
                        self.check_std_json_single_value_builtin(expr, expected, "number_as_int")
                    }
                    Type::Builtin(BuiltinId::OptionSome) => {
                        if expr.args.len() != 1 {
                            self.diagnostics.push(Diagnostic::new(
                                "T004",
                                format!("expected 1 arguments but found {}", expr.args.len()),
                                expr.span,
                            ));
                            Type::Error
                        } else {
                            let expected = expected.map(|ty| self.resolve_type(&ty));
                            let expected_item = match expected.as_ref() {
                                Some(Type::Option(item)) => Some(*item.clone()),
                                _ => None,
                            };
                            let item_ty = if let Some(expected_item) = expected_item {
                                self.check_expr_with_expected(
                                    &expr.args[0],
                                    Some(expected_item.clone()),
                                );
                                expected_item
                            } else {
                                self.check_expr(&expr.args[0])
                            };
                            let option_ty = Type::Option(Box::new(self.resolve_type(&item_ty)));
                            match expected {
                                Some(Type::Option(_)) | None => option_ty,
                                Some(expected) => {
                                    self.apply_expected(option_ty, Some(expected), expr.span)
                                }
                            }
                        }
                    }
                    Type::Builtin(BuiltinId::ResultOk) => self.check_result_constructor_builtin(
                        expr,
                        expected,
                        known_enum::RESULT_OK_NAME,
                    ),
                    Type::Builtin(BuiltinId::ResultErr) => self.check_result_constructor_builtin(
                        expr,
                        expected,
                        known_enum::RESULT_ERR_NAME,
                    ),
                    Type::EnumConstructor {
                        enum_name,
                        enum_item: _,
                        variant_name,
                    } => self.check_user_enum_constructor(
                        expr.span,
                        expected,
                        enum_name,
                        variant_name,
                        &expr.args,
                    ),
                    Type::Function(sig) => {
                        if !expr.type_args.is_empty()
                            && !self.is_std_cli_usage_for_required_call(&expr.callee)
                            && !self.is_std_cli_help_for_required_call(&expr.callee)
                            && !self.is_std_cli_parse_request_call(&expr.callee)
                        {
                            self.diagnostics.push(
                                Diagnostic::new(
                                    "T004",
                                    "explicit call type arguments are currently supported only for `cli::usage_for_required`, `cli::help_for_required`, and `cli::parse_request`",
                                    expr.span,
                                )
                                .with_suggestion("remove the call type arguments or use `cli::usage_for_required[Record](program)`, `cli::help_for_required[Record](program)`, or `cli::parse_request[Record](args, program)`"),
                            );
                            for arg in &expr.args {
                                self.check_expr(arg);
                            }
                            Type::Error
                        } else if self.is_std_json_decode_call(&expr.callee) {
                            self.check_std_json_decode_call(expr, expected, sig)
                        } else if self.is_std_json_decode_or_call(&expr.callee) {
                            self.check_std_json_decode_or_call(expr, expected, sig)
                        } else if self.is_std_json_to_value_call(&expr.callee) {
                            self.check_std_json_to_value_call(expr, expected, sig)
                        } else if self.is_std_json_encode_typed_call(&expr.callee) {
                            self.check_std_json_encode_typed_call(expr, expected, sig)
                        } else if self.is_std_config_load_json_call(&expr.callee) {
                            self.check_std_config_load_json_call(expr, expected, sig)
                        } else if self.is_std_config_load_json_or_call(&expr.callee) {
                            self.check_std_config_load_json_or_call(expr, expected, sig)
                        } else if self.is_std_cli_parse_call(&expr.callee) {
                            self.check_std_cli_parse_call(expr, expected, sig)
                        } else if self.is_std_cli_parse_or_call(&expr.callee) {
                            self.check_std_cli_parse_or_call(expr, expected, sig)
                        } else if self.is_std_cli_parse_request_call(&expr.callee) {
                            self.check_std_cli_parse_request_call(expr, expected, sig)
                        } else if self.is_std_cli_parse_request_or_call(&expr.callee) {
                            self.check_std_cli_parse_request_or_call(expr, expected, sig)
                        } else if self.is_std_cli_usage_for_call(&expr.callee) {
                            self.check_std_cli_usage_for_call(expr, expected, sig)
                        } else if self.is_std_cli_usage_for_required_call(&expr.callee) {
                            self.check_std_cli_usage_for_required_call(expr, expected, sig)
                        } else if self.is_std_cli_help_for_call(&expr.callee) {
                            self.check_std_cli_help_for_call(expr, expected, sig)
                        } else if self.is_std_cli_help_for_required_call(&expr.callee) {
                            self.check_std_cli_help_for_required_call(expr, expected, sig)
                        } else {
                            let sig = self.instantiate_function_sig(sig);
                            if sig.params.len() != expr.args.len() {
                                self.diagnostics.push(Diagnostic::new(
                                    "T004",
                                    format!(
                                        "expected {} arguments but found {}",
                                        sig.params.len(),
                                        expr.args.len()
                                    ),
                                    expr.span,
                                ));
                                Type::Error
                            } else {
                                let param_modes = self.param_modes_for_callee(&expr.callee);
                                for (index, (arg, param_ty)) in
                                    expr.args.iter().zip(sig.params.iter()).enumerate()
                                {
                                    let arg_ty =
                                        self.check_expr_with_expected(arg, Some(param_ty.clone()));
                                    if param_modes
                                        .as_ref()
                                        .and_then(|modes| modes.get(index))
                                        .copied()
                                        == Some(PackageInterfaceParamMode::Consume)
                                        && !matches!(self.resolve_type(&arg_ty), Type::Error)
                                    {
                                        self.mark_consumed_argument(arg, expr.span);
                                    }
                                }
                                self.apply_expected(*sig.ret.clone(), expected, expr.span)
                            }
                        }
                    }
                    Type::Unknown(_) => {
                        let arg_tys: Vec<Type> =
                            expr.args.iter().map(|arg| self.check_expr(arg)).collect();
                        let ret_ty =
                            expected.unwrap_or_else(|| Type::Unknown(self.fresh_unknown()));
                        let inferred_sig = Type::Function(FunctionSig {
                            type_params: Vec::new(),
                            params: arg_tys,
                            ret: Box::new(ret_ty.clone()),
                        });
                        if let Err(message) = self.unify(callee_ty.clone(), inferred_sig) {
                            self.diagnostics
                                .push(Diagnostic::new("T005", message, expr.span));
                            Type::Error
                        } else {
                            self.resolve_type(&ret_ty)
                        }
                    }
                    Type::Error => Type::Error,
                    _ => {
                        self.diagnostics.push(Diagnostic::new(
                            "T005",
                            "attempted to call a non-function value",
                            expr.span,
                        ));
                        Type::Error
                    }
                };
                let resolved_callee = self.resolve_type(&callee_ty);
                self.calls.push(TypedCallInfo {
                    expr_id: expr.id,
                    span: expr.span,
                    callee: self.typed_callee_for(&expr.callee, &resolved_callee),
                });
                ty
            }
            Expr::If(expr) => {
                let condition = self.check_expr(&expr.condition);
                self.require_exact(&condition, &Type::Bool, expr.condition.span(), "T001");
                match expected {
                    Some(expected_ty) => {
                        self.check_value_block_with_expected(
                            &expr.then_branch,
                            Some(expected_ty.clone()),
                        );
                        self.check_value_block_with_expected(
                            &expr.else_branch,
                            Some(expected_ty.clone()),
                        );
                        self.resolve_type(&expected_ty)
                    }
                    None => {
                        let then_ty = self.check_value_block(&expr.then_branch);
                        let else_ty = self.check_value_block(&expr.else_branch);
                        match (self.resolve_type(&then_ty), self.resolve_type(&else_ty)) {
                            (Type::Never, Type::Never) => Type::Never,
                            (Type::Never, other) | (other, Type::Never) => other,
                            (then_ty, else_ty) => {
                                self.require_exact(&then_ty, &else_ty, expr.span, "T002");
                                self.resolve_type(&then_ty)
                            }
                        }
                    }
                }
            }
            Expr::Match(expr) => self.check_match_expr(expr, expected),
            Expr::Try(expr) => self.check_try_expr(expr, expected),
            Expr::Fn(expr) => {
                let sig = self.signature_from_fn_expr(expr, expected.as_ref());
                self.push_scope(true);
                self.current_function_returns.push((*sig.ret).clone());
                let outer_loop_depth = std::mem::replace(&mut self.loop_depth, 0);
                for (param, param_ty) in expr.params.iter().zip(sig.params.iter().cloned()) {
                    let name = self.symbol(&param.name);
                    self.insert_current(name, BindingKind::Parameter, param_ty, param.span);
                }
                self.check_value_block_contents(&expr.body, Some((*sig.ret).clone()));
                self.loop_depth = outer_loop_depth;
                self.current_function_returns.pop();
                self.pop_scope();
                self.apply_expected(Type::Function(sig), expected, expr.span)
            }
        };
        self.record_expr_type(expr.id(), span, ty)
    }

    fn record_expr_type(&mut self, expr_id: ExprId, span: Span, ty: Type) -> Type {
        let resolved = self.resolve_type(&ty);
        self.expr_types.push(ExprType {
            expr_id,
            span,
            ty: resolved.clone(),
        });
        resolved
    }

    fn predeclare_records(&mut self, statements: &[Stmt]) {
        for statement in statements {
            let Stmt::RecordDecl(record) = statement else {
                continue;
            };
            let name = self.symbol(&record.name);
            if let Some(existing) = self.records.get(&name) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E002",
                        format!("duplicate record `{}` in the current scope", record.name),
                        record.span,
                    )
                    .with_related("previous record declaration is here", existing.span),
                );
                continue;
            }
            let type_params = record
                .type_params
                .iter()
                .map(|param| self.symbol(param))
                .collect::<Vec<_>>();
            let mut fields = Vec::new();
            for field in &record.fields {
                fields.push(RecordField {
                    name: self.symbol(&field.name),
                    json_rename: json_rename_from_attributes(&field.attributes)
                        .map(|rename| self.symbol(rename)),
                    json_aliases: json_aliases_from_attributes(&field.attributes)
                        .into_iter()
                        .map(|(alias, _)| self.symbol(alias))
                        .collect(),
                    json_validation: json_validation_from_attributes(&field.attributes),
                    cli_name: cli_name_from_attributes(&field.attributes)
                        .map(|name| self.symbol(name)),
                    cli_short: cli_short_from_attributes(&field.attributes)
                        .map(|short| self.symbol(short)),
                    cli_position: cli_position_from_attributes(&field.attributes),
                    cli_value_source: cli_value_source_from_attributes(&field.attributes),
                    cli_aliases: cli_aliases_from_attributes(&field.attributes)
                        .into_iter()
                        .map(|(alias, _)| self.symbol(alias))
                        .collect(),
                    cli_help: cli_help_from_attributes(&field.attributes)
                        .map(|help| self.symbol(help)),
                    cli_hidden: cli_hidden_from_attributes(&field.attributes),
                    cli_subcommand: cli_subcommand_from_attributes(&field.attributes),
                    type_name: Some(field.type_name.clone()),
                    ty: None,
                    span: field.span,
                });
            }
            let cli_about =
                cli_about_from_attributes(&record.attributes).map(|about| self.symbol(about));
            self.records.insert(
                name,
                RecordDef {
                    span: record.span,
                    type_params,
                    json_deny_unknown_fields: json_deny_unknown_fields_from_attributes(
                        &record.attributes,
                    ),
                    cli_about,
                    fields,
                },
            );
        }
    }

    fn predeclare_enums(&mut self, statements: &[Stmt]) {
        for statement in statements {
            let Stmt::EnumDecl(enumeration) = statement else {
                continue;
            };
            let name = self.symbol(&enumeration.name);
            if let Some(existing) = self.enums.get(&name) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E002",
                        format!("duplicate enum `{}` in the current scope", enumeration.name),
                        enumeration.span,
                    )
                    .with_related("previous enum declaration is here", existing.span),
                );
                continue;
            }
            if let Some(existing) = self.records.get(&name) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E002",
                        format!("duplicate type `{}` in the current scope", enumeration.name),
                        enumeration.span,
                    )
                    .with_related("previous type declaration is here", existing.span),
                );
                continue;
            }

            let type_params = enumeration
                .type_params
                .iter()
                .map(|param| self.symbol(param))
                .collect::<Vec<_>>();
            let mut variants = Vec::new();
            for variant in &enumeration.variants {
                let variant_name = self.symbol(&variant.name);
                variants.push(EnumVariantDef {
                    name: variant_name,
                    json_rename: json_rename_from_attributes(&variant.attributes)
                        .map(|rename| self.symbol(rename)),
                    json_aliases: json_aliases_from_attributes(&variant.attributes)
                        .into_iter()
                        .map(|(alias, _)| self.symbol(alias))
                        .collect(),
                    cli_name: cli_name_from_attributes(&variant.attributes)
                        .map(|name| self.symbol(name)),
                    cli_aliases: cli_aliases_from_attributes(&variant.attributes)
                        .into_iter()
                        .map(|(alias, _)| self.symbol(alias))
                        .collect(),
                    cli_about: cli_about_from_attributes(&variant.attributes)
                        .map(|about| self.symbol(about)),
                    cli_hidden: cli_hidden_from_attributes(&variant.attributes),
                    payload: variant.payload.clone(),
                    payload_ty: None,
                    span: variant.span,
                });
                let qualified = self.symbol(&format!("{}::{}", enumeration.name, variant.name));
                let kind = if variant.payload.is_some() {
                    BindingKind::Function
                } else {
                    BindingKind::Immutable
                };
                let enum_item = enumeration
                    .package_item
                    .or_else(|| self.package_enum_items.get(&name).copied());
                self.insert_current(
                    qualified,
                    kind,
                    Type::EnumConstructor {
                        enum_name: name,
                        enum_item,
                        variant_name,
                    },
                    variant.span,
                );
            }
            let cli_about =
                cli_about_from_attributes(&enumeration.attributes).map(|about| self.symbol(about));
            self.enums.insert(
                name,
                EnumDef {
                    span: enumeration.span,
                    type_params,
                    cli_about,
                    variants,
                },
            );
        }
    }

    fn predeclare_opaque_types(&mut self, statements: &[Stmt]) {
        for statement in statements {
            let Stmt::OpaqueTypeDecl(opaque) = statement else {
                continue;
            };
            let name = self.symbol(&opaque.name);
            if let Some(item) = opaque.package_item {
                self.package_opaques.insert(item, name);
                self.package_opaque_items.insert(name, item);
                self.package_opaque_handle_facts.entry(item).or_default();
            }
        }
    }

    fn check_record_decl(&mut self, record: &RecordDecl) {
        let type_params =
            self.type_param_symbols(&record.type_params, "record", &record.name, record.span);
        let mut field_names = HashMap::new();
        let mut json_wire_names: HashMap<Symbol, Span> = HashMap::new();
        let mut cli_option_names: HashMap<Symbol, Span> = HashMap::new();
        let mut cli_short_names: HashMap<Symbol, Span> = HashMap::new();
        let mut cli_positions: HashMap<u32, Span> = HashMap::new();
        let mut cli_position_entries: Vec<(u32, Span, bool, String)> = Vec::new();
        let mut cli_subcommand_fields: Vec<(String, Span)> = Vec::new();
        for field in &record.fields {
            let field_name = self.symbol(&field.name);
            if let Some(previous_span) = field_names.insert(field_name, field.span) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E002",
                        format!(
                            "duplicate field `{}` in record `{}`",
                            field.name, record.name
                        ),
                        field.span,
                    )
                    .with_related("previous field declaration is here", previous_span),
                );
            }
            let mut field_json_names = HashMap::new();
            let primary_json_wire_name = json_rename_from_attributes(&field.attributes)
                .map(|rename| self.symbol(rename))
                .unwrap_or(field_name);
            let primary_span =
                json_rename_span_from_attributes(&field.attributes).unwrap_or(field.span);
            let mut accepted_json_names = vec![(primary_json_wire_name, primary_span)];
            for (alias, span) in json_aliases_from_attributes(&field.attributes) {
                let alias = self.symbol(alias);
                accepted_json_names.push((alias, span));
            }
            for (json_wire_name, json_wire_span) in accepted_json_names {
                let mut duplicate_for_field = false;
                if let Some(previous_span) = field_json_names.insert(json_wire_name, json_wire_span)
                {
                    duplicate_for_field = true;
                    self.diagnostics.push(
                        Diagnostic::new(
                            "E002",
                            format!(
                                "duplicate JSON field wire name `{}` in record `{}`",
                                self.symbols.resolve(json_wire_name),
                                record.name
                            ),
                            json_wire_span,
                        )
                        .with_related("previous JSON field wire name is here", previous_span),
                    );
                }
                if !duplicate_for_field
                    && let Some(previous_span) =
                        json_wire_names.insert(json_wire_name, json_wire_span)
                {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "E002",
                            format!(
                                "duplicate JSON field wire name `{}` in record `{}`",
                                self.symbols.resolve(json_wire_name),
                                record.name
                            ),
                            json_wire_span,
                        )
                        .with_related("previous JSON field wire name is here", previous_span),
                    );
                }
            }
            let field_ty =
                self.type_from_expr_with_params(&field.type_name, field.span, &type_params);
            if matches!(self.resolve_type(&field_ty), Type::Function(_)) {
                self.diagnostics.push(Diagnostic::new(
                    "E011",
                    "record fields may not have function type in v1",
                    field.span,
                ));
            }
            if let Some(cli_subcommand_span) =
                cli_subcommand_argument_from_attributes(&field.attributes)
            {
                cli_subcommand_fields.push((field.name.clone(), cli_subcommand_span));
                if cli_name_from_attributes(&field.attributes).is_some()
                    || cli_short_argument_from_attributes(&field.attributes).is_some()
                    || cli_position_argument_from_attributes(&field.attributes).is_some()
                    || cli_value_source_from_attributes(&field.attributes).is_some()
                    || !cli_aliases_from_attributes(&field.attributes).is_empty()
                    || cli_help_from_attributes(&field.attributes).is_some()
                    || cli_hidden_from_attributes(&field.attributes)
                {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "T006",
                            format!(
                                "CLI subcommand field `{}::{}` may not combine `subcommand` with `name`, `short`, `positional`, `value_source`, `alias`, `help`, or `hidden` metadata",
                                record.name, field.name
                            ),
                            cli_subcommand_span,
                        )
                        .with_suggestion(
                            "keep `@cli(subcommand)` as the field's only `@cli` metadata",
                        ),
                    );
                }
                match self.resolve_type(&field_ty) {
                    Type::Enum(enum_name, args) if args.is_empty() => {
                        let mut visiting = HashSet::new();
                        let _ = self.cli_command_schema_for_enum(
                            enum_name,
                            args,
                            field.span,
                            "@cli(subcommand)",
                            &mut visiting,
                        );
                    }
                    other => {
                        self.diagnostics.push(
                            Diagnostic::new(
                                "T006",
                                format!(
                                    "CLI subcommand field `{}::{}` must have a concrete command enum type; found {}",
                                    record.name,
                                    field.name,
                                    self.type_label(&other)
                                ),
                                field.span,
                            )
                            .with_suggestion(
                                "use a non-generic enum whose variants carry command record payloads",
                            ),
                        );
                    }
                }
                self.check_json_validation_attributes(
                    &record.name,
                    &field.name,
                    &field_ty,
                    &field.attributes,
                );
                continue;
            }
            let cli_metadata_present = cli_metadata_present_from_attributes(&field.attributes);
            let cli_schema = self.cli_field_schema_for_type(&field_ty);
            let cli_supported = cli_schema.is_some();
            if cli_metadata_present && !cli_supported {
                let (message, suggestion) = if cli_position_argument_from_attributes(
                    &field.attributes,
                )
                .is_some()
                {
                    (
                        format!(
                            "field `{}` has `@cli(positional: ...)` metadata but its type is not supported by CLI positional parsing",
                            field.name
                        ),
                        "use a String, Int, Bool, Option[String|Int|Bool], final List[String|Int|Bool], or zero-payload enum positional field",
                    )
                } else {
                    (
                        format!(
                            "field `{}` has `@cli(...)` metadata but its type is not supported by `cli::parse_or`",
                            field.name
                        ),
                        "use a String, Int, Bool, Option[String|Int|Bool], List[String|Int|Bool], or zero-payload enum field",
                    )
                };
                self.diagnostics
                    .push(Diagnostic::new("T006", message, field.span).with_suggestion(suggestion));
            }
            if cli_supported {
                if let Some((_, value_source_span)) =
                    cli_value_source_argument_from_attributes(&field.attributes)
                    && let Some(schema) = cli_schema.as_ref()
                    && !Self::cli_value_source_allowed_for_schema(schema)
                {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "T006",
                            format!(
                                "CLI value source metadata on field `{}` in record `{}` requires a String value type",
                                field.name, record.name
                            ),
                            value_source_span,
                        )
                        .with_suggestion(
                            "use String, Option[String], or List[String], or remove `value_source`",
                        ),
                    );
                }
                if let Some((cli_position, cli_position_span)) =
                    cli_position_argument_from_attributes(&field.attributes)
                {
                    if cli_name_from_attributes(&field.attributes).is_some()
                        || cli_short_argument_from_attributes(&field.attributes).is_some()
                        || !cli_aliases_from_attributes(&field.attributes).is_empty()
                        || cli_hidden_from_attributes(&field.attributes)
                    {
                        self.diagnostics.push(
                            Diagnostic::new(
                                "T006",
                                format!(
                                    "CLI positional field `{}` may not combine `positional` with `name`, `short`, `alias`, or `hidden` metadata in record `{}`",
                                    field.name, record.name
                                ),
                                cli_position_span,
                            )
                            .with_suggestion(
                                "keep positional fields unnamed, or remove `positional` and use option metadata",
                            ),
                        );
                    }
                    if let Some(previous_span) =
                        cli_positions.insert(cli_position, cli_position_span)
                    {
                        self.diagnostics.push(
                            Diagnostic::new(
                                "E002",
                                format!(
                                    "duplicate CLI positional index `{cli_position}` in record `{}`",
                                    record.name
                                ),
                                cli_position_span,
                            )
                            .with_related("previous CLI positional index is here", previous_span),
                        );
                    } else if let Some(schema) = cli_schema.as_ref() {
                        cli_position_entries.push((
                            cli_position,
                            cli_position_span,
                            Self::cli_schema_is_list(schema),
                            field.name.clone(),
                        ));
                    }
                } else {
                    let primary_cli_option_name = cli_name_from_attributes(&field.attributes)
                        .or_else(|| json_rename_from_attributes(&field.attributes))
                        .map(|name| self.symbol(name))
                        .unwrap_or(field_name);
                    let primary_span = cli_name_span_from_attributes(&field.attributes)
                        .or_else(|| json_rename_span_from_attributes(&field.attributes))
                        .unwrap_or(field.span);
                    let mut accepted_cli_names = vec![(primary_cli_option_name, primary_span)];
                    for (alias, span) in cli_aliases_from_attributes(&field.attributes) {
                        accepted_cli_names.push((self.symbol(alias), span));
                    }
                    let mut field_cli_names = HashMap::new();
                    for (cli_option_name, cli_option_span) in accepted_cli_names {
                        let mut duplicate_for_field = false;
                        if let Some(previous_span) =
                            field_cli_names.insert(cli_option_name, cli_option_span)
                        {
                            duplicate_for_field = true;
                            self.diagnostics.push(
                                Diagnostic::new(
                                    "E002",
                                    format!(
                                        "duplicate CLI option name `{}` in record `{}`",
                                        self.symbols.resolve(cli_option_name),
                                        record.name
                                    ),
                                    cli_option_span,
                                )
                                .with_related("previous CLI option name is here", previous_span),
                            );
                        }
                        if !duplicate_for_field
                            && let Some(previous_span) =
                                cli_option_names.insert(cli_option_name, cli_option_span)
                        {
                            self.diagnostics.push(
                                Diagnostic::new(
                                    "E002",
                                    format!(
                                        "duplicate CLI option name `{}` in record `{}`",
                                        self.symbols.resolve(cli_option_name),
                                        record.name
                                    ),
                                    cli_option_span,
                                )
                                .with_related("previous CLI option name is here", previous_span),
                            );
                        }
                    }
                    if let Some((cli_short_name, cli_short_span)) =
                        cli_short_argument_from_attributes(&field.attributes)
                    {
                        let cli_short_name = self.symbol(cli_short_name);
                        if let Some(previous_span) =
                            cli_short_names.insert(cli_short_name, cli_short_span)
                        {
                            self.diagnostics.push(
                                Diagnostic::new(
                                    "E002",
                                    format!(
                                        "duplicate CLI short option name `{}` in record `{}`",
                                        self.symbols.resolve(cli_short_name),
                                        record.name
                                    ),
                                    cli_short_span,
                                )
                                .with_related(
                                    "previous CLI short option name is here",
                                    previous_span,
                                ),
                            );
                        }
                    }
                }
            }
            self.check_json_validation_attributes(
                &record.name,
                &field.name,
                &field_ty,
                &field.attributes,
            );
        }
        if let Some((_, first_span)) = cli_subcommand_fields.first() {
            for (field_name, span) in cli_subcommand_fields.iter().skip(1) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "T006",
                        format!(
                            "CLI wrapper record `{}` may contain exactly one `@cli(subcommand)` field; duplicate field `{field_name}`",
                            record.name
                        ),
                        *span,
                    )
                    .with_related("first `@cli(subcommand)` field is here", *first_span),
                );
            }
        }
        if !cli_position_entries.is_empty() {
            cli_position_entries.sort_by_key(|(position, _, _, _)| *position);
            for (expected, (position, span, _, _)) in (1u32..).zip(cli_position_entries.iter()) {
                if *position != expected {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "E002",
                            format!(
                                "CLI positional indexes in record `{}` must be contiguous starting at 1",
                                record.name
                            ),
                            *span,
                        )
                        .with_suggestion("renumber positional fields as 1, 2, 3 without gaps"),
                    );
                    break;
                }
            }
            let max_position = cli_position_entries
                .last()
                .map(|(position, _, _, _)| *position)
                .unwrap_or(0);
            for (position, span, is_list, field_name) in &cli_position_entries {
                if *is_list && *position != max_position {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "T006",
                            format!(
                                "CLI positional List field `{field_name}` in record `{}` must be the final positional field",
                                record.name
                            ),
                            *span,
                        )
                        .with_suggestion(
                            "move the List positional field to the highest positional index",
                        ),
                    );
                }
            }
        }
    }

    fn check_json_validation_attributes(
        &mut self,
        record_name: &str,
        field_name: &str,
        field_ty: &Type,
        attributes: &[Attribute],
    ) {
        let rules = json_validation_rules_with_spans(attributes);
        if rules.is_empty() {
            return;
        }

        let mut seen: HashMap<&'static str, (JsonDecodeValidationRule, Span)> = HashMap::new();
        let mut min_value = None;
        let mut max_value = None;
        let mut min_len_value = None;
        let mut max_len_value = None;
        for (rule, span) in &rules {
            let key = json_validation_rule_key(rule);
            if let Some((previous, previous_span)) = seen.get(key) {
                if previous != rule {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "T028",
                            format!(
                                "conflicting JSON validation `{key}` on field `{field_name}` in record `{record_name}`"
                            ),
                            *span,
                        )
                        .with_related("previous validation is here", *previous_span),
                    );
                }
                continue;
            }
            seen.insert(key, (rule.clone(), *span));

            match rule {
                JsonDecodeValidationRule::Min(value) => min_value = Some((*value, *span)),
                JsonDecodeValidationRule::Max(value) => max_value = Some((*value, *span)),
                JsonDecodeValidationRule::MinLen(value) => {
                    min_len_value = Some((*value, *span));
                    if *value < 0 {
                        self.diagnostics.push(Diagnostic::new(
                            "T028",
                            format!(
                                "JSON validation `min_len` on field `{field_name}` may not be negative"
                            ),
                            *span,
                        ));
                    }
                }
                JsonDecodeValidationRule::MaxLen(value) => {
                    max_len_value = Some((*value, *span));
                    if *value < 0 {
                        self.diagnostics.push(Diagnostic::new(
                            "T028",
                            format!(
                                "JSON validation `max_len` on field `{field_name}` may not be negative"
                            ),
                            *span,
                        ));
                    }
                }
                JsonDecodeValidationRule::NonEmpty => {}
            }
        }
        if let (Some((min, min_span)), Some((max, max_span))) = (min_value, max_value)
            && min > max
        {
            self.diagnostics.push(
                Diagnostic::new(
                    "T028",
                    format!(
                        "JSON validation `min` may not be greater than `max` on field `{field_name}`"
                    ),
                    max_span,
                )
                .with_related("minimum is declared here", min_span),
            );
        }
        if let (Some((min_len, min_span)), Some((max_len, max_span))) =
            (min_len_value, max_len_value)
            && min_len > max_len
        {
            self.diagnostics.push(
                Diagnostic::new(
                    "T028",
                    format!(
                        "JSON validation `min_len` may not be greater than `max_len` on field `{field_name}`"
                    ),
                    max_span,
                )
                .with_related("minimum length is declared here", min_span),
            );
        }

        let effective_ty = match self.resolve_type(field_ty) {
            Type::Option(item) => self.resolve_type(&item),
            other => other,
        };
        for (rule, span) in rules {
            let supported = matches!(
                (&rule, &effective_ty),
                (
                    JsonDecodeValidationRule::NonEmpty
                        | JsonDecodeValidationRule::MinLen(_)
                        | JsonDecodeValidationRule::MaxLen(_),
                    Type::String,
                ) | (
                    JsonDecodeValidationRule::Min(_) | JsonDecodeValidationRule::Max(_),
                    Type::Int,
                )
            );
            if !supported {
                self.diagnostics.push(
                    Diagnostic::new(
                        "T028",
                        format!(
                            "JSON validation `{}` is not supported for field `{field_name}` of type `{}`",
                            json_validation_rule_key(&rule),
                            self.type_label(field_ty)
                        ),
                        span,
                    )
                    .with_suggestion(
                        "use string validators on String or Option[String], and numeric bounds on Int or Option[Int]",
                    ),
                );
            }
        }
    }

    fn check_enum_decl(&mut self, enumeration: &EnumDecl) {
        let mut type_params = HashSet::new();
        for param in &enumeration.type_params {
            let symbol = self.symbol(param);
            if !type_params.insert(symbol) {
                self.diagnostics.push(Diagnostic::new(
                    "E002",
                    format!(
                        "duplicate type parameter `{param}` in enum `{}`",
                        enumeration.name
                    ),
                    enumeration.span,
                ));
            }
            if matches!(param.as_str(), "Int" | "Bool" | "String") {
                self.diagnostics.push(Diagnostic::new(
                    "T022",
                    format!("type parameter `{param}` shadows a built-in type"),
                    enumeration.span,
                ));
            }
        }

        let params = type_params.into_iter().collect::<Vec<_>>();
        let mut variant_names = HashMap::new();
        let mut json_wire_names: HashMap<Symbol, Span> = HashMap::new();
        let mut cli_command_names: HashMap<Symbol, Span> = HashMap::new();
        for variant in &enumeration.variants {
            let variant_name = self.symbol(&variant.name);
            if let Some(previous_span) = variant_names.insert(variant_name, variant.span) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E002",
                        format!(
                            "duplicate variant `{}` in enum `{}`",
                            variant.name, enumeration.name
                        ),
                        variant.span,
                    )
                    .with_related("previous variant declaration is here", previous_span),
                );
            }
            let mut variant_json_names = HashMap::new();
            let primary_json_wire_name = json_rename_from_attributes(&variant.attributes)
                .map(|rename| self.symbol(rename))
                .unwrap_or(variant_name);
            let primary_span =
                json_rename_span_from_attributes(&variant.attributes).unwrap_or(variant.span);
            let mut accepted_json_names = vec![(primary_json_wire_name, primary_span)];
            for (alias, span) in json_aliases_from_attributes(&variant.attributes) {
                let alias = self.symbol(alias);
                accepted_json_names.push((alias, span));
            }
            for (json_wire_name, json_wire_span) in accepted_json_names {
                let mut duplicate_for_variant = false;
                if let Some(previous_span) =
                    variant_json_names.insert(json_wire_name, json_wire_span)
                {
                    duplicate_for_variant = true;
                    self.diagnostics.push(
                        Diagnostic::new(
                            "E002",
                            format!(
                                "duplicate JSON enum variant wire name `{}` in enum `{}`",
                                self.symbols.resolve(json_wire_name),
                                enumeration.name
                            ),
                            json_wire_span,
                        )
                        .with_related(
                            "previous JSON enum variant wire name is here",
                            previous_span,
                        ),
                    );
                }
                if !duplicate_for_variant
                    && let Some(previous_span) =
                        json_wire_names.insert(json_wire_name, json_wire_span)
                {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "E002",
                            format!(
                                "duplicate JSON enum variant wire name `{}` in enum `{}`",
                                self.symbols.resolve(json_wire_name),
                                enumeration.name
                            ),
                            json_wire_span,
                        )
                        .with_related(
                            "previous JSON enum variant wire name is here",
                            previous_span,
                        ),
                    );
                }
            }
            if cli_metadata_present_from_attributes(&variant.attributes) {
                let Some((primary_cli_name, primary_span)) =
                    cli_name_argument_from_attributes(&variant.attributes)
                else {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "T006",
                            format!(
                                "CLI command variant `{}` in enum `{}` requires `@cli(name: \"...\")`",
                                variant.name, enumeration.name
                            ),
                            variant.span,
                        )
                        .with_suggestion(
                            "write `@cli(name: \"command-name\")` on the enum variant",
                        ),
                    );
                    if let Some(payload) = &variant.payload {
                        let _ = self.type_from_expr_with_params(payload, variant.span, &params);
                    }
                    continue;
                };
                let mut accepted_cli_names = vec![(self.symbol(primary_cli_name), primary_span)];
                for (alias, span) in cli_aliases_from_attributes(&variant.attributes) {
                    accepted_cli_names.push((self.symbol(alias), span));
                }
                let mut variant_cli_names = HashMap::new();
                for (cli_name, cli_span) in accepted_cli_names {
                    let mut duplicate_for_variant = false;
                    if let Some(previous_span) = variant_cli_names.insert(cli_name, cli_span) {
                        duplicate_for_variant = true;
                        self.diagnostics.push(
                            Diagnostic::new(
                                "E002",
                                format!(
                                    "duplicate CLI command name `{}` in enum `{}`",
                                    self.symbols.resolve(cli_name),
                                    enumeration.name
                                ),
                                cli_span,
                            )
                            .with_related("previous CLI command name is here", previous_span),
                        );
                    }
                    if !duplicate_for_variant
                        && let Some(previous_span) = cli_command_names.insert(cli_name, cli_span)
                    {
                        self.diagnostics.push(
                            Diagnostic::new(
                                "E002",
                                format!(
                                    "duplicate CLI command name `{}` in enum `{}`",
                                    self.symbols.resolve(cli_name),
                                    enumeration.name
                                ),
                                cli_span,
                            )
                            .with_related("previous CLI command name is here", previous_span),
                        );
                    }
                }
            }
            if let Some(payload) = &variant.payload {
                let _ = self.type_from_expr_with_params(payload, variant.span, &params);
            }
        }
    }

    fn type_param_symbols(
        &mut self,
        params: &[String],
        owner_kind: &str,
        owner_name: &str,
        span: Span,
    ) -> Vec<Symbol> {
        let mut seen = HashSet::new();
        let mut symbols = Vec::with_capacity(params.len());
        for param in params {
            let symbol = self.symbol(param);
            if !seen.insert(symbol) {
                self.diagnostics.push(Diagnostic::new(
                    "E002",
                    format!("duplicate type parameter `{param}` in {owner_kind} `{owner_name}`"),
                    span,
                ));
                continue;
            }
            if matches!(param.as_str(), "Int" | "Bool" | "String") {
                self.diagnostics.push(Diagnostic::new(
                    "T022",
                    format!("type parameter `{param}` shadows a built-in type"),
                    span,
                ));
            }
            symbols.push(symbol);
        }
        symbols
    }

    fn check_list_lit(&mut self, expr: &ListLitExpr, expected: Option<Type>) -> Type {
        let expected = expected.map(|ty| self.resolve_type(&ty));
        let expected_item = match expected.as_ref() {
            Some(Type::List(item)) => Some(*item.clone()),
            _ => None,
        };

        if expr.items.is_empty() {
            return match expected {
                Some(Type::List(item)) => Type::List(item),
                Some(Type::Error) => Type::Error,
                Some(_) | None => {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "T015",
                            "empty list literal requires an expected List[T] type",
                            expr.span,
                        )
                        .with_suggestion(
                            "add a local binding annotation such as `items: List[Int] = []`",
                        ),
                    );
                    Type::Error
                }
            };
        }

        let item_ty = if let Some(expected_item) = expected_item {
            for item in &expr.items {
                self.check_expr_with_expected(item, Some(expected_item.clone()));
            }
            expected_item
        } else {
            let first_ty = self.check_expr(&expr.items[0]);
            for item in expr.items.iter().skip(1) {
                self.check_expr_with_expected(item, Some(first_ty.clone()));
            }
            first_ty
        };
        let list_ty = Type::List(Box::new(self.resolve_type(&item_ty)));
        match expected {
            Some(Type::List(_)) | None => list_ty,
            Some(expected) => self.apply_expected(list_ty, Some(expected), expr.span),
        }
    }

    fn check_index_expr(&mut self, expr: &IndexExpr, expected: Option<Type>) -> Type {
        let expected = expected.map(|ty| self.resolve_type(&ty));
        let base_ty = self.check_list_receiver_type(&expr.base, expected.clone());
        self.check_expr_with_expected(&expr.index, Some(Type::Int));
        match self.resolve_type(&base_ty) {
            Type::List(item_ty) => self.apply_expected(*item_ty, expected, expr.span),
            Type::Unknown(_) => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        expr.span,
                    )
                    .with_suggestion(
                        "annotate the indexed value as List[T] before using list indexing",
                    ),
                );
                Type::Error
            }
            Type::Error => Type::Error,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "list indexing expects List[T] as its base but found {}",
                        self.type_label(&other)
                    ),
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_list_receiver_type(&mut self, receiver: &Expr, expected_item: Option<Type>) -> Type {
        let list_expected = expected_item.map(|item| Type::List(Box::new(item)));
        let receiver_ty = if Self::is_empty_list_literal(receiver) {
            self.check_expr_with_expected(receiver, list_expected.clone())
        } else {
            self.check_expr(receiver)
        };
        match self.resolve_type(&receiver_ty) {
            Type::Unknown(_) => {
                let Some(list_expected) = list_expected else {
                    return receiver_ty;
                };
                match self.unify(receiver_ty, list_expected) {
                    Ok(ty) => self.resolve_type(&ty),
                    Err(message) => {
                        self.diagnostics
                            .push(Diagnostic::new("T002", message, receiver.span()));
                        Type::Error
                    }
                }
            }
            ty => ty,
        }
    }

    fn check_map_empty_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if !expr.args.is_empty() {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 0 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let expected = expected.map(|ty| self.resolve_type(&ty));
        match expected {
            Some(Type::Map(key, value)) => Type::Map(key, value),
            Some(Type::Error) => Type::Error,
            Some(_) | None => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "T019",
                        "`Map.empty()` requires an expected Map[K, V] type",
                        expr.span,
                    )
                    .with_suggestion(
                        "add a local binding annotation such as `items: Map[String, Int] = Map.empty()`",
                    ),
                );
                Type::Error
            }
        }
    }

    fn check_get_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let expected = expected.map(|ty| self.resolve_type(&ty));
        let expected_item = match expected.as_ref() {
            Some(Type::Option(item)) => Some(*item.clone()),
            _ => None,
        };

        if Self::is_map_empty_call(&expr.args[0]) {
            let key_ty = self.check_expr(&expr.args[1]);
            let key_ty = self.resolve_type(&key_ty);
            if !self.validate_map_key_type(&key_ty, expr.args[1].span()) {
                return Type::Error;
            }
            let value_ty = expected_item
                .clone()
                .unwrap_or_else(|| Type::Unknown(self.fresh_unknown()));
            let base_expected = Type::Map(Box::new(key_ty), Box::new(value_ty));
            let base_ty = self.check_expr_with_expected(&expr.args[0], Some(base_expected));
            return match self.resolve_type(&base_ty) {
                Type::Map(_, value_ty) => {
                    self.apply_expected_option(Type::Option(value_ty), expected, expr.span)
                }
                Type::Error => Type::Error,
                other => {
                    self.diagnostics.push(Diagnostic::new(
                        "T006",
                        format!(
                            "`get` expects List[T] or Map[K, V] as its first argument but found {}",
                            self.type_label(&other)
                        ),
                        expr.span,
                    ));
                    Type::Error
                }
            };
        }

        let base_expected = match expected_item {
            Some(item) if Self::is_empty_list_literal(&expr.args[0]) => {
                Some(Type::List(Box::new(item)))
            }
            _ => None,
        };
        let base_ty = self.check_expr_with_expected(&expr.args[0], base_expected);
        match self.resolve_type(&base_ty) {
            Type::List(item_ty) => {
                self.check_expr_with_expected(&expr.args[1], Some(Type::Int));
                self.apply_expected_option(Type::Option(item_ty), expected, expr.span)
            }
            Type::Map(key_ty, value_ty) => {
                if !self.validate_map_key_type(&key_ty, expr.args[0].span()) {
                    return Type::Error;
                }
                self.check_expr_with_expected(&expr.args[1], Some((*key_ty).clone()));
                self.apply_expected_option(Type::Option(value_ty), expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        expr.span,
                    )
                    .with_suggestion(
                        "annotate the receiver as List[T] or Map[K, V] before calling `get`",
                    ),
                );
                Type::Error
            }
            Type::Error => Type::Error,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`get` expects List[T] or Map[K, V] as its first argument but found {}",
                        self.type_label(&other)
                    ),
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_contains_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        if Self::is_map_empty_call(&expr.args[0]) {
            self.diagnostics.push(
                Diagnostic::new(
                    "T019",
                    "`Map.empty().contains(...)` requires an expected Map[K, V] type",
                    expr.span,
                )
                .with_suggestion(
                    "add a local binding annotation such as `items: Map[String, Int] = Map.empty()`",
                ),
            );
            return Type::Error;
        }

        let base_ty = self.check_expr(&expr.args[0]);
        match self.resolve_type(&base_ty) {
            Type::String => {
                self.check_expr_with_expected(&expr.args[1], Some(Type::String));
                self.apply_expected(Type::Bool, expected, expr.span)
            }
            Type::Map(key_ty, _) => {
                if !self.validate_map_key_type(&key_ty, expr.args[0].span()) {
                    return Type::Error;
                }
                self.check_expr_with_expected(&expr.args[1], Some((*key_ty).clone()));
                self.apply_expected(Type::Bool, expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        expr.span,
                    )
                    .with_suggestion(
                        "annotate the receiver as String or Map[K, V] before calling `contains`",
                    ),
                );
                Type::Error
            }
            Type::Error => Type::Error,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`contains` expects String or Map[K, V] as its first argument but found {}",
                        self.type_label(&other)
                    ),
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_string_unary_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        builtin: BuiltinId,
        return_ty: Type,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        if self.check_string_receiver(&expr.args[0], builtin, expr.span) {
            self.apply_expected(return_ty, expected, expr.span)
        } else {
            Type::Error
        }
    }

    fn check_string_receiver(
        &mut self,
        receiver: &Expr,
        builtin: BuiltinId,
        call_span: Span,
    ) -> bool {
        let receiver_ty = self.check_expr(receiver);
        match self.resolve_type(&receiver_ty) {
            Type::String => true,
            Type::Unknown(_) => match self.unify(receiver_ty, Type::String) {
                Ok(_) => true,
                Err(message) => {
                    self.diagnostics
                        .push(Diagnostic::new("T002", message, receiver.span()));
                    false
                }
            },
            Type::Error => false,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`{}` expects String as its first argument but found {}",
                        Self::builtin_name(builtin),
                        self.type_label(&other)
                    ),
                    call_span,
                ));
                false
            }
        }
    }

    fn check_string_predicate_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        builtin: BuiltinId,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let receiver_ok = self.check_string_receiver(&expr.args[0], builtin, expr.span);
        self.check_expr_with_expected(&expr.args[1], Some(Type::String));
        if receiver_ok {
            self.apply_expected(Type::Bool, expected, expr.span)
        } else {
            Type::Error
        }
    }

    fn check_string_pair_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        builtin: BuiltinId,
        ret: Type,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let receiver_ok = self.check_string_receiver(&expr.args[0], builtin, expr.span);
        self.check_expr_with_expected(&expr.args[1], Some(Type::String));
        if receiver_ok {
            self.apply_expected(ret, expected, expr.span)
        } else {
            Type::Error
        }
    }

    fn check_string_binary_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        builtin: BuiltinId,
        ret: Type,
    ) -> Type {
        if expr.args.len() != 3 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 3 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let receiver_ok = self.check_string_receiver(&expr.args[0], builtin, expr.span);
        self.check_expr_with_expected(&expr.args[1], Some(Type::String));
        self.check_expr_with_expected(&expr.args[2], Some(Type::String));
        if receiver_ok {
            self.apply_expected(ret, expected, expr.span)
        } else {
            Type::Error
        }
    }

    fn check_slice_chars_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 3 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 3 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let receiver_ok =
            self.check_string_receiver(&expr.args[0], BuiltinId::SliceChars, expr.span);
        self.check_expr_with_expected(&expr.args[1], Some(Type::Int));
        self.check_expr_with_expected(&expr.args[2], Some(Type::Int));
        if receiver_ok {
            self.apply_expected(
                Type::Result(Box::new(Type::String), Box::new(Type::String)),
                expected,
                expr.span,
            )
        } else {
            Type::Error
        }
    }

    fn check_to_string_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let arg_ty = self.check_expr(&expr.args[0]);
        match self.resolve_type(&arg_ty) {
            Type::Int | Type::Bool | Type::String => {
                self.apply_expected(Type::String, expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        expr.span,
                    )
                    .with_suggestion(
                        "annotate the receiver as Int, Bool, or String before calling `to_string`",
                    ),
                );
                Type::Error
            }
            Type::Error => Type::Error,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`to_string` accepts only Int, Bool, or String but found {}",
                        self.type_label(&other)
                    ),
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_parse_int_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        if self.check_string_receiver(&expr.args[0], BuiltinId::ParseInt, expr.span) {
            self.apply_expected(
                Type::Result(Box::new(Type::Int), Box::new(Type::String)),
                expected,
                expr.span,
            )
        } else {
            Type::Error
        }
    }

    fn check_parse_bool_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        if self.check_string_receiver(&expr.args[0], BuiltinId::ParseBool, expr.span) {
            self.apply_expected(
                Type::Result(Box::new(Type::Bool), Box::new(Type::String)),
                expected,
                expr.span,
            )
        } else {
            Type::Error
        }
    }

    fn check_std_path_join_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.check_expr_with_expected(&expr.args[1], Some(Type::String));
        self.apply_expected(Type::String, expected, expr.span)
    }

    fn check_std_path_normalize_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.apply_expected(Type::String, expected, expr.span)
    }

    fn check_std_path_file_name_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.apply_expected(Type::Option(Box::new(Type::String)), expected, expr.span)
    }

    fn check_std_path_with_file_name_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.check_expr_with_expected(&expr.args[1], Some(Type::String));
        self.apply_expected(Type::String, expected, expr.span)
    }

    fn check_std_path_parent_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.apply_expected(Type::Option(Box::new(Type::String)), expected, expr.span)
    }

    fn check_std_path_strip_prefix_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.check_expr_with_expected(&expr.args[1], Some(Type::String));
        self.apply_expected(Type::Option(Box::new(Type::String)), expected, expr.span)
    }

    fn check_std_path_extension_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.apply_expected(Type::Option(Box::new(Type::String)), expected, expr.span)
    }

    fn check_std_path_file_stem_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.apply_expected(Type::Option(Box::new(Type::String)), expected, expr.span)
    }

    fn check_std_path_with_extension_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.check_expr_with_expected(&expr.args[1], Some(Type::String));
        self.apply_expected(Type::String, expected, expr.span)
    }

    fn check_std_path_is_absolute_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.apply_expected(Type::Bool, expected, expr.span)
    }

    fn check_std_fs_read_text_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::String), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_read_bytes_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        let bytes_ty = self.std_bytes_type_in_fs();
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(bytes_ty), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_read_resource_text_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.check_expr_with_expected(&expr.args[1], Some(Type::String));
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::String), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_read_resource_bytes_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.check_expr_with_expected(&expr.args[1], Some(Type::String));
        let bytes_ty = self.std_bytes_type_in_fs();
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(bytes_ty), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_bytes_unary_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        builtin: BuiltinId,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let arg_ty = self.check_expr(&expr.args[0]);
        match self.resolve_type(&arg_ty) {
            ty if self.is_std_bytes_type(&ty) => {
                let return_ty = match builtin {
                    BuiltinId::StdBytesSize => Type::Int,
                    BuiltinId::StdBytesIsEmpty => Type::Bool,
                    _ => unreachable!("std bytes unary checker only handles bytes builtins"),
                };
                self.apply_expected(return_ty, expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        expr.span,
                    )
                    .with_suggestion(
                        "annotate the receiver as bytes::Bytes before calling `bytes::size` or `bytes::empty`",
                    ),
                );
                Type::Error
            }
            Type::Error => Type::Error,
            other => {
                let name = match builtin {
                    BuiltinId::StdBytesSize => "bytes::size",
                    BuiltinId::StdBytesIsEmpty => "bytes::empty",
                    _ => unreachable!("std bytes unary checker only handles bytes builtins"),
                };
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`{name}` expects bytes::Bytes as its first argument but found {}",
                        self.type_label(&other)
                    ),
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_std_bytes_at_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let arg_ty = self.check_expr(&expr.args[0]);
        self.check_expr_with_expected(&expr.args[1], Some(Type::Int));
        match self.resolve_type(&arg_ty) {
            ty if self.is_std_bytes_type(&ty) => {
                self.apply_expected(Type::Option(Box::new(Type::Int)), expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        expr.span,
                    )
                    .with_suggestion(
                        "annotate the receiver as bytes::Bytes before calling `bytes::at`",
                    ),
                );
                Type::Error
            }
            Type::Error => Type::Error,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`bytes::at` expects bytes::Bytes as its first argument but found {}",
                        self.type_label(&other)
                    ),
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_std_hash_sha256_hex_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let arg_ty = self.check_expr(&expr.args[0]);
        match self.resolve_type(&arg_ty) {
            ty if self.is_std_bytes_type(&ty) => {
                self.apply_expected(Type::String, expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        expr.span,
                    )
                    .with_suggestion(
                        "annotate the receiver as bytes::Bytes before calling `hash::sha256_hex`",
                    ),
                );
                Type::Error
            }
            Type::Error => Type::Error,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`hash::sha256_hex` expects bytes::Bytes as its first argument but found {}",
                        self.type_label(&other)
                    ),
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_std_fs_write_text_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.check_expr_with_expected(&expr.args[1], Some(Type::String));
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::Unit), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_write_bytes_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        let bytes_ty = self.std_bytes_type_in_fs();
        self.check_expr_with_expected(&expr.args[1], Some(bytes_ty));
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::Unit), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_open_text_handle_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        let file_ty = self.std_fs_file_type();
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(file_ty), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_read_text_from_handle_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let file_ty = self.std_fs_file_type();
        self.check_expr_with_expected(&expr.args[0], Some(file_ty));
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::String), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_write_text_to_handle_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let file_ty = self.std_fs_file_type();
        self.check_expr_with_expected(&expr.args[0], Some(file_ty));
        self.check_expr_with_expected(&expr.args[1], Some(Type::String));
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::Unit), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_close_handle_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let file_ty = self.std_fs_file_type();
        self.check_expr_with_expected(&expr.args[0], Some(file_ty));
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::Unit), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_read_dir_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        let path_ty = self.std_path_type();
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::List(Box::new(path_ty))), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_canonicalize_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::String), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_directory_size_metadata_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        let metadata_ty = self.std_fs_directory_size_metadata_type();
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(metadata_ty), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_file_size_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::Int), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_modified_unix_millis_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::Int), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_unit_path_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::Unit), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_copy_file_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.check_expr_with_expected(&expr.args[1], Some(Type::String));
        let error_ty = self.std_io_path_pair_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::Unit), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_fs_metadata_bool_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.apply_expected(Type::Bool, expected, expr.span)
    }

    fn check_std_env_get_var_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.apply_expected(Type::Option(Box::new(Type::String)), expected, expr.span)
    }

    fn check_std_env_args_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if !expr.args.is_empty() {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 0 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.apply_expected(Type::List(Box::new(Type::String)), expected, expr.span)
    }

    fn check_std_env_current_dir_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if !expr.args.is_empty() {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 0 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::String), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_env_temp_dir_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if !expr.args.is_empty() {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 0 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let error_ty = self.std_io_error_type();
        self.apply_expected(
            Type::Result(Box::new(Type::String), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_process_run_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 3 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 3 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.check_expr_with_expected(&expr.args[1], Some(Type::List(Box::new(Type::String))));
        let options_ty = self.std_process_options_type();
        self.check_expr_with_expected(&expr.args[2], Some(options_ty));

        let output_ty = self.std_process_output_type();
        let error_ty = self.std_process_error_type();
        self.apply_expected(
            Type::Result(Box::new(output_ty), Box::new(error_ty)),
            expected,
            expr.span,
        )
    }

    fn check_std_time_now_unix_millis_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if !expr.args.is_empty() {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 0 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.apply_expected(Type::Int, expected, expr.span)
    }

    fn check_std_test_assert_true_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::Bool));
        let result_ty = self.test_assertion_result_type();
        self.apply_expected(result_ty, expected, expr.span)
    }

    fn check_std_test_assert_eq_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        builtin: BuiltinId,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let scalar_ty = match builtin {
            BuiltinId::StdTestAssertEqInt => Type::Int,
            BuiltinId::StdTestAssertEqBool => Type::Bool,
            BuiltinId::StdTestAssertEqString => Type::String,
            _ => unreachable!("matched std test equality builtin"),
        };
        self.check_expr_with_expected(&expr.args[0], Some(scalar_ty.clone()));
        self.check_expr_with_expected(&expr.args[1], Some(scalar_ty));
        let result_ty = self.test_assertion_result_type();
        self.apply_expected(result_ty, expected, expr.span)
    }

    fn test_assertion_result_type(&mut self) -> Type {
        Type::Result(Box::new(Type::Unit), Box::new(Type::String))
    }

    fn std_io_error_type(&mut self) -> Type {
        Type::Record(
            self.symbol(crate::std_package::IO_ERROR_VISIBLE_NAME_IN_FS),
            Vec::new(),
        )
    }

    fn std_io_path_pair_error_type(&mut self) -> Type {
        Type::Record(
            self.symbol(crate::std_package::PATH_PAIR_ERROR_VISIBLE_NAME_IN_FS),
            Vec::new(),
        )
    }

    fn std_path_type(&mut self) -> Type {
        Type::Record(
            self.symbol(crate::std_package::PATH_VISIBLE_NAME_IN_FS),
            Vec::new(),
        )
    }

    fn std_bytes_type_in_fs(&mut self) -> Type {
        Type::Opaque(self.symbol(crate::std_package::BYTES_VISIBLE_NAME_IN_FS))
    }

    fn std_process_options_type(&mut self) -> Type {
        Type::Record(
            self.symbol(crate::std_package::PROCESS_OPTIONS_VISIBLE_NAME),
            Vec::new(),
        )
    }

    fn std_process_output_type(&mut self) -> Type {
        Type::Record(
            self.symbol(crate::std_package::PROCESS_OUTPUT_VISIBLE_NAME),
            Vec::new(),
        )
    }

    fn std_process_error_type(&mut self) -> Type {
        Type::Record(
            self.symbol(crate::std_package::PROCESS_ERROR_VISIBLE_NAME),
            Vec::new(),
        )
    }

    fn is_std_bytes_type(&self, ty: &Type) -> bool {
        let Type::Opaque(symbol) = ty else {
            return false;
        };
        let label = self.symbols.resolve(*symbol);
        label == "Bytes" || label == crate::std_package::BYTES_VISIBLE_NAME_IN_FS
    }

    fn std_fs_file_type(&mut self) -> Type {
        Type::Opaque(self.symbol("File"))
    }

    fn std_fs_directory_size_metadata_type(&mut self) -> Type {
        Type::Record(
            self.symbol(crate::std_package::FS_DIRECTORY_SIZE_METADATA_VISIBLE_NAME_IN_FS),
            Vec::new(),
        )
    }

    fn check_std_map_keys_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let map_ty = self.check_expr(&expr.args[0]);
        match self.resolve_type(&map_ty) {
            Type::Map(key_ty, _) => {
                if !self.validate_map_key_type(&key_ty, expr.args[0].span()) {
                    return Type::Error;
                }
                self.apply_expected(Type::List(key_ty), expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        expr.span,
                    )
                    .with_suggestion(
                        "annotate the receiver as Map[K, V] before calling `map::keys`",
                    ),
                );
                Type::Error
            }
            Type::Error => Type::Error,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`map::keys` expects Map[K, V] as its first argument but found {}",
                        self.type_label(&other)
                    ),
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_std_map_values_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let map_ty = self.check_expr(&expr.args[0]);
        match self.resolve_type(&map_ty) {
            Type::Map(key_ty, value_ty) => {
                if !self.validate_map_key_type(&key_ty, expr.args[0].span()) {
                    return Type::Error;
                }
                self.apply_expected(Type::List(value_ty), expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        expr.span,
                    )
                    .with_suggestion(
                        "annotate the receiver as Map[K, V] before calling `map::values`",
                    ),
                );
                Type::Error
            }
            Type::Error => Type::Error,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`map::values` expects Map[K, V] as its first argument but found {}",
                        self.type_label(&other)
                    ),
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_std_json_parse_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr_with_expected(&expr.args[0], Some(Type::String));
        self.std_json_expected_return(expr, expected, "parse")
    }

    fn check_std_json_single_value_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        public_name: &str,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        self.check_expr(&expr.args[0]);
        self.std_json_expected_return(expr, expected, public_name)
    }

    fn std_json_expected_return(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        public_name: &str,
    ) -> Type {
        match expected {
            Some(expected) => expected,
            None => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        expr.span,
                    )
                    .with_suggestion(format!(
                        "call `json::{public_name}` through its public std::json wrapper"
                    )),
                );
                Type::Error
            }
        }
    }

    fn is_std_json_decode_or_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_json_decode_or_bindings.contains(&binding))
    }

    fn is_std_json_decode_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_json_decode_bindings.contains(&binding))
    }

    fn is_std_json_to_value_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_json_to_value_bindings.contains(&binding))
    }

    fn is_std_json_encode_typed_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_json_encode_typed_bindings.contains(&binding))
    }

    fn is_std_config_load_json_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_config_load_json_bindings.contains(&binding))
    }

    fn is_std_config_load_json_or_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_config_load_json_or_bindings.contains(&binding))
    }

    fn is_std_cli_parse_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_cli_parse_bindings.contains(&binding))
    }

    fn is_std_cli_parse_or_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_cli_parse_or_bindings.contains(&binding))
    }

    fn is_std_cli_parse_request_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_cli_parse_request_bindings.contains(&binding))
    }

    fn is_std_cli_parse_request_or_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_cli_parse_request_or_bindings.contains(&binding))
    }

    fn is_std_cli_usage_for_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_cli_usage_for_bindings.contains(&binding))
    }

    fn is_std_cli_usage_for_required_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_cli_usage_for_required_bindings.contains(&binding))
    }

    fn is_std_cli_help_for_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_cli_help_for_bindings.contains(&binding))
    }

    fn is_std_cli_help_for_required_call(&self, callee: &Expr) -> bool {
        self.binding_for_expr(callee.id())
            .is_some_and(|binding| self.std_cli_help_for_required_bindings.contains(&binding))
    }

    fn check_std_json_decode_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let value_ty = sig.params.first().cloned().unwrap_or(Type::Error);
        let error_ty = match self.resolve_type(&sig.ret) {
            Type::Result(_, err_ty) => *err_ty,
            _ => Type::Error,
        };
        self.check_expr_with_expected(&expr.args[0], Some(value_ty));

        let expected = expected.map(|ty| self.resolve_type(&ty));
        let Some((target_ty, expected_error_ty)) = expected.as_ref().and_then(|expected| {
            if let Type::Result(ok_ty, err_ty) = self.resolve_type(expected) {
                Some((*ok_ty, *err_ty))
            } else {
                None
            }
        }) else {
            self.diagnostics.push(
                Diagnostic::new(
                    "E005",
                    "type annotation required because `json::decode` has no fallback value",
                    expr.span,
                )
                .with_suggestion(
                    "annotate the result as Result[T, json::Error] or use `try json::decode(...)` in an annotated Result-returning context",
                ),
            );
            return Type::Error;
        };

        if let Err(message) = self.unify(expected_error_ty, error_ty.clone()) {
            self.diagnostics.push(
                Diagnostic::new("T002", message, expr.span).with_suggestion(
                    "`json::decode` returns Result[T, json::Error]; map the error or use json::Error at this boundary",
                ),
            );
            return Type::Error;
        }

        let target_ty = self.resolve_type(&target_ty);
        let mut visiting = HashSet::new();
        let Some(schema) =
            self.json_decode_schema_for_type(&target_ty, expr.span, &mut visiting, "json::decode")
        else {
            return Type::Error;
        };
        self.json_required_decode_schemas
            .push(TypedJsonDecodeSchemaInfo {
                expr_id: expr.id,
                schema,
            });

        let result_ty = Type::Result(Box::new(target_ty), Box::new(error_ty));
        self.apply_expected(result_ty, expected, expr.span)
    }

    fn check_std_json_to_value_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let (value_ty, error_ty) = match self.resolve_type(&sig.ret) {
            Type::Result(value_ty, error_ty) => (*value_ty, *error_ty),
            _ => (Type::Error, Type::Error),
        };
        let checked_ty = self.check_expr(&expr.args[0]);
        let target_ty = self.resolve_type(&checked_ty);
        let mut visiting = HashSet::new();
        let Some(schema) = self.json_encode_schema_for_type(
            &target_ty,
            expr.args[0].span(),
            &mut visiting,
            "json::to_value",
        ) else {
            return Type::Error;
        };
        self.json_to_value_schemas.push(TypedJsonDecodeSchemaInfo {
            expr_id: expr.id,
            schema,
        });

        let result_ty = Type::Result(Box::new(value_ty), Box::new(error_ty));
        self.apply_expected(result_ty, expected, expr.span)
    }

    fn check_std_json_encode_typed_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let result_ty = self.resolve_type(&sig.ret);
        let checked_ty = self.check_expr(&expr.args[0]);
        let target_ty = self.resolve_type(&checked_ty);
        let mut visiting = HashSet::new();
        let Some(schema) = self.json_encode_schema_for_type(
            &target_ty,
            expr.args[0].span(),
            &mut visiting,
            "json::encode_typed",
        ) else {
            return Type::Error;
        };
        self.json_encode_typed_schemas
            .push(TypedJsonDecodeSchemaInfo {
                expr_id: expr.id,
                schema,
            });

        self.apply_expected(result_ty, expected, expr.span)
    }

    fn check_std_json_decode_or_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let value_ty = sig.params.first().cloned().unwrap_or(Type::Error);
        let error_ty = match self.resolve_type(&sig.ret) {
            Type::Result(_, err_ty) => *err_ty,
            _ => Type::Error,
        };
        self.check_expr_with_expected(&expr.args[0], Some(value_ty));

        let expected_target =
            expected
                .as_ref()
                .and_then(|expected| match self.resolve_type(expected) {
                    Type::Result(ok_ty, _) => Some(*ok_ty),
                    _ => None,
                });
        let fallback_ty = self.check_expr_with_expected(&expr.args[1], expected_target);
        let target_ty = self.resolve_type(&fallback_ty);
        let mut visiting = HashSet::new();
        let Some(schema) = self.json_decode_schema_for_type(
            &target_ty,
            expr.args[1].span(),
            &mut visiting,
            "json::decode_or",
        ) else {
            return Type::Error;
        };
        self.json_decode_schemas.push(TypedJsonDecodeSchemaInfo {
            expr_id: expr.id,
            schema,
        });

        let result_ty = Type::Result(Box::new(target_ty), Box::new(error_ty));
        self.apply_expected(result_ty, expected, expr.span)
    }

    fn check_std_config_load_json_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let path_ty = sig.params.first().cloned().unwrap_or(Type::Error);
        let error_ty = match self.resolve_type(&sig.ret) {
            Type::Result(_, err_ty) => *err_ty,
            _ => Type::Error,
        };
        self.check_expr_with_expected(&expr.args[0], Some(path_ty));

        let expected = expected.map(|ty| self.resolve_type(&ty));
        let Some((target_ty, expected_error_ty)) = expected.as_ref().and_then(|expected| {
            if let Type::Result(ok_ty, err_ty) = self.resolve_type(expected) {
                Some((*ok_ty, *err_ty))
            } else {
                None
            }
        }) else {
            self.diagnostics.push(
                Diagnostic::new(
                    "E005",
                    "type annotation required because `config::load_json` has no fallback value",
                    expr.span,
                )
                .with_suggestion(
                    "annotate the result as Result[T, config::Error] or use `try config::load_json(...)` in an annotated Result-returning context",
                ),
            );
            return Type::Error;
        };

        if let Err(message) = self.unify(expected_error_ty, error_ty.clone()) {
            self.diagnostics.push(
                Diagnostic::new("T002", message, expr.span).with_suggestion(
                    "`config::load_json` returns Result[T, config::Error]; map the error or use config::Error at this boundary",
                ),
            );
            return Type::Error;
        }

        let target_ty = self.resolve_type(&target_ty);
        let mut visiting = HashSet::new();
        let Some(schema) = self.json_decode_schema_for_type(
            &target_ty,
            expr.span,
            &mut visiting,
            "config::load_json",
        ) else {
            return Type::Error;
        };
        self.config_required_load_json_schemas
            .push(TypedJsonDecodeSchemaInfo {
                expr_id: expr.id,
                schema,
            });

        let result_ty = Type::Result(Box::new(target_ty), Box::new(error_ty));
        self.apply_expected(result_ty, expected, expr.span)
    }

    fn check_std_config_load_json_or_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let path_ty = sig.params.first().cloned().unwrap_or(Type::Error);
        let error_ty = match self.resolve_type(&sig.ret) {
            Type::Result(_, err_ty) => *err_ty,
            _ => Type::Error,
        };
        self.check_expr_with_expected(&expr.args[0], Some(path_ty));

        let expected_target =
            expected
                .as_ref()
                .and_then(|expected| match self.resolve_type(expected) {
                    Type::Result(ok_ty, _) => Some(*ok_ty),
                    _ => None,
                });
        let fallback_ty = self.check_expr_with_expected(&expr.args[1], expected_target);
        let target_ty = self.resolve_type(&fallback_ty);
        let mut visiting = HashSet::new();
        let Some(schema) = self.json_decode_schema_for_type(
            &target_ty,
            expr.args[1].span(),
            &mut visiting,
            "config::load_json_or",
        ) else {
            return Type::Error;
        };
        self.config_load_json_schemas
            .push(TypedJsonDecodeSchemaInfo {
                expr_id: expr.id,
                schema,
            });

        let result_ty = Type::Result(Box::new(target_ty), Box::new(error_ty));
        self.apply_expected(result_ty, expected, expr.span)
    }

    fn check_std_cli_parse_or_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let args_ty = sig.params.first().cloned().unwrap_or(Type::Error);
        let error_ty = match self.resolve_type(&sig.ret) {
            Type::Result(_, err_ty) => *err_ty,
            _ => Type::Error,
        };
        self.check_expr_with_expected(&expr.args[0], Some(args_ty));

        let expected_target =
            expected
                .as_ref()
                .and_then(|expected| match self.resolve_type(expected) {
                    Type::Result(ok_ty, _) => Some(*ok_ty),
                    _ => None,
                });
        let defaults_ty = self.check_expr_with_expected(&expr.args[1], expected_target);
        let target_ty = self.resolve_type(&defaults_ty);
        let Some(schema) =
            self.cli_schema_for_type(&target_ty, expr.args[1].span(), "cli::parse_or", false)
        else {
            return Type::Error;
        };
        self.cli_parse_or_schemas.push(TypedCliSchemaInfo {
            expr_id: expr.id,
            schema,
        });

        let result_ty = Type::Result(Box::new(target_ty), Box::new(error_ty));
        self.apply_expected(result_ty, expected, expr.span)
    }

    fn check_std_cli_parse_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let args_ty = sig.params.first().cloned().unwrap_or(Type::Error);
        let error_ty = match self.resolve_type(&sig.ret) {
            Type::Result(_, err_ty) => *err_ty,
            _ => Type::Error,
        };
        self.check_expr_with_expected(&expr.args[0], Some(args_ty));

        let expected = expected.map(|ty| self.resolve_type(&ty));
        let Some((target_ty, expected_error_ty)) = expected.as_ref().and_then(|expected| {
            if let Type::Result(ok_ty, err_ty) = self.resolve_type(expected) {
                Some((*ok_ty, *err_ty))
            } else {
                None
            }
        }) else {
            self.diagnostics.push(
                Diagnostic::new(
                    "E005",
                    "type annotation required because `cli::parse` has no default value",
                    expr.span,
                )
                .with_suggestion(
                    "annotate the result as Result[T, cli::Error] or use `try cli::parse(...)` in an annotated Result-returning context",
                ),
            );
            return Type::Error;
        };

        if let Err(message) = self.unify(expected_error_ty, error_ty.clone()) {
            self.diagnostics.push(
                Diagnostic::new("T002", message, expr.span).with_suggestion(
                    "`cli::parse` returns Result[T, cli::Error]; map the error or use cli::Error at this boundary",
                ),
            );
            return Type::Error;
        }

        let target_ty = self.resolve_type(&target_ty);
        let Some(schema) = self.cli_schema_for_type(&target_ty, expr.span, "cli::parse", true)
        else {
            return Type::Error;
        };
        self.cli_parse_schemas.push(TypedCliSchemaInfo {
            expr_id: expr.id,
            schema,
        });

        let result_ty = Type::Result(Box::new(target_ty), Box::new(error_ty));
        self.apply_expected(result_ty, expected, expr.span)
    }

    fn check_std_cli_parse_request_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.type_args.len() != 1 {
            self.diagnostics.push(
                Diagnostic::new(
                    "T004",
                    format!(
                        "`cli::parse_request` requires exactly 1 explicit record type argument but found {}",
                        expr.type_args.len()
                    ),
                    expr.span,
                )
                .with_suggestion(
                    "call it as `cli::parse_request[Command](args, \"cli-tool\")`",
                ),
            );
            for arg in &expr.args {
                self.check_expr(arg);
            }
            return Type::Error;
        }
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let args_ty = sig.params.first().cloned().unwrap_or(Type::Error);
        let program_ty = sig.params.get(1).cloned().unwrap_or(Type::Error);
        self.check_expr_with_expected(&expr.args[0], Some(args_ty));
        self.check_expr_with_expected(&expr.args[1], Some(program_ty));
        let target_ty = self.type_from_expr(&expr.type_args[0], expr.span);
        let target_ty = self.resolve_type(&target_ty);
        let Some(schema) =
            self.cli_schema_for_type(&target_ty, expr.span, "cli::parse_request", true)
        else {
            return Type::Error;
        };
        if !self.validate_cli_help_schema(&schema, expr.span, "cli::parse_request") {
            return Type::Error;
        }
        self.cli_parse_request_schemas.push(TypedCliSchemaInfo {
            expr_id: expr.id,
            schema,
        });

        let result_ty = self.instantiate_cli_request_return_type(&sig, target_ty);
        self.apply_expected(result_ty, expected, expr.span)
    }

    fn check_std_cli_parse_request_or_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.args.len() != 3 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 3 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let args_ty = sig.params.first().cloned().unwrap_or(Type::Error);
        let program_ty = sig.params.get(1).cloned().unwrap_or(Type::Error);
        self.check_expr_with_expected(&expr.args[0], Some(args_ty));
        self.check_expr_with_expected(&expr.args[1], Some(program_ty));

        let expected_target = self.cli_request_target_from_expected(expected.as_ref());
        let defaults_ty = self.check_expr_with_expected(&expr.args[2], expected_target);
        let target_ty = self.resolve_type(&defaults_ty);
        let Some(schema) = self.cli_schema_for_type(
            &target_ty,
            expr.args[2].span(),
            "cli::parse_request_or",
            false,
        ) else {
            return Type::Error;
        };
        if !self.validate_cli_help_schema(&schema, expr.span, "cli::parse_request_or") {
            return Type::Error;
        }
        self.cli_parse_request_or_schemas.push(TypedCliSchemaInfo {
            expr_id: expr.id,
            schema,
        });

        let result_ty = self.instantiate_cli_request_return_type(&sig, target_ty);
        self.apply_expected(result_ty, expected, expr.span)
    }

    fn instantiate_cli_request_return_type(&self, sig: &FunctionSig, target_ty: Type) -> Type {
        self.resolve_type(&self.substitute_type_params(
            *sig.ret.clone(),
            &sig.type_params,
            &[target_ty],
        ))
    }

    fn cli_request_target_from_expected(&self, expected: Option<&Type>) -> Option<Type> {
        let expected = expected?;
        let Type::Result(ok_ty, _) = self.resolve_type(expected) else {
            return None;
        };
        let Type::Enum(_, args) = self.resolve_type(&ok_ty) else {
            return None;
        };
        if args.len() == 1 {
            Some(args[0].clone())
        } else {
            None
        }
    }

    fn check_std_cli_usage_for_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let program_ty = sig.params.first().cloned().unwrap_or(Type::Error);
        self.check_expr_with_expected(&expr.args[0], Some(program_ty));
        let defaults_ty = self.check_expr(&expr.args[1]);
        let target_ty = self.resolve_type(&defaults_ty);
        let Some(schema) =
            self.cli_schema_for_type(&target_ty, expr.args[1].span(), "cli::usage_for", false)
        else {
            return Type::Error;
        };
        self.cli_usage_for_schemas.push(TypedCliSchemaInfo {
            expr_id: expr.id,
            schema,
        });

        self.apply_expected(self.resolve_type(&sig.ret), expected, expr.span)
    }

    fn check_std_cli_usage_for_required_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.type_args.len() != 1 {
            self.diagnostics.push(
                Diagnostic::new(
                    "T004",
                    format!(
                        "`cli::usage_for_required` requires exactly 1 explicit record type argument but found {}",
                        expr.type_args.len()
                    ),
                    expr.span,
                )
                .with_suggestion(
                    "call it as `cli::usage_for_required[Command](\"cli-tool\")`",
                ),
            );
            for arg in &expr.args {
                self.check_expr(arg);
            }
            return Type::Error;
        }
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let program_ty = sig.params.first().cloned().unwrap_or(Type::Error);
        self.check_expr_with_expected(&expr.args[0], Some(program_ty));
        let target_ty = self.type_from_expr(&expr.type_args[0], expr.span);
        let target_ty = self.resolve_type(&target_ty);
        let Some(schema) =
            self.cli_schema_for_type(&target_ty, expr.span, "cli::usage_for_required", true)
        else {
            return Type::Error;
        };
        self.cli_usage_for_required_schemas
            .push(TypedCliSchemaInfo {
                expr_id: expr.id,
                schema,
            });

        self.apply_expected(self.resolve_type(&sig.ret), expected, expr.span)
    }

    fn check_std_cli_help_for_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let program_ty = sig.params.first().cloned().unwrap_or(Type::Error);
        self.check_expr_with_expected(&expr.args[0], Some(program_ty));
        let defaults_ty = self.check_expr(&expr.args[1]);
        let target_ty = self.resolve_type(&defaults_ty);
        let Some(schema) =
            self.cli_schema_for_type(&target_ty, expr.args[1].span(), "cli::help_for", false)
        else {
            return Type::Error;
        };
        if !self.validate_cli_help_schema(&schema, expr.span, "cli::help_for") {
            return Type::Error;
        }
        self.cli_help_for_schemas.push(TypedCliSchemaInfo {
            expr_id: expr.id,
            schema,
        });

        self.apply_expected(self.resolve_type(&sig.ret), expected, expr.span)
    }

    fn check_std_cli_help_for_required_call(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        sig: FunctionSig,
    ) -> Type {
        if expr.type_args.len() != 1 {
            self.diagnostics.push(
                Diagnostic::new(
                    "T004",
                    format!(
                        "`cli::help_for_required` requires exactly 1 explicit record type argument but found {}",
                        expr.type_args.len()
                    ),
                    expr.span,
                )
                .with_suggestion("call it as `cli::help_for_required[Command](\"cli-tool\")`"),
            );
            for arg in &expr.args {
                self.check_expr(arg);
            }
            return Type::Error;
        }
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let program_ty = sig.params.first().cloned().unwrap_or(Type::Error);
        self.check_expr_with_expected(&expr.args[0], Some(program_ty));
        let target_ty = self.type_from_expr(&expr.type_args[0], expr.span);
        let target_ty = self.resolve_type(&target_ty);
        let Some(schema) =
            self.cli_schema_for_type(&target_ty, expr.span, "cli::help_for_required", true)
        else {
            return Type::Error;
        };
        if !self.validate_cli_help_schema(&schema, expr.span, "cli::help_for_required") {
            return Type::Error;
        }
        self.cli_help_for_required_schemas.push(TypedCliSchemaInfo {
            expr_id: expr.id,
            schema,
        });

        self.apply_expected(self.resolve_type(&sig.ret), expected, expr.span)
    }

    fn validate_cli_help_schema(
        &mut self,
        schema: &CliSchema,
        span: Span,
        callee_label: &str,
    ) -> bool {
        let mut valid = true;
        if let Some(subcommand) = &schema.subcommand
            && !self.validate_cli_help_schema(&subcommand.schema, span, callee_label)
        {
            valid = false;
        }
        for command in &schema.commands {
            if !self.validate_cli_help_schema(&command.payload, span, callee_label) {
                valid = false;
            }
        }
        for field in &schema.fields {
            let field_name = self.symbols.resolve(field.name).to_string();
            if self.symbols.resolve(field.option_name) == "help" {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`{callee_label}` reserves `--help` and `-h`; field `{field_name}` uses CLI option name `help`"
                    ),
                    span,
                ));
                valid = false;
            }
            for alias in &field.aliases {
                if self.symbols.resolve(*alias) == "help" {
                    self.diagnostics.push(Diagnostic::new(
                        "T006",
                        format!(
                            "`{callee_label}` reserves `--help` and `-h`; field `{field_name}` uses CLI alias `help`"
                        ),
                        span,
                    ));
                    valid = false;
                }
            }
            if field
                .short
                .is_some_and(|short| self.symbols.resolve(short) == "h")
            {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`{callee_label}` reserves `--help` and `-h`; field `{field_name}` uses `@cli(short: \"h\")`"
                    ),
                    span,
                ));
                valid = false;
            }
        }
        valid
    }

    fn cli_schema_for_type(
        &mut self,
        ty: &Type,
        span: Span,
        callee_label: &str,
        strict: bool,
    ) -> Option<CliSchema> {
        let mut visiting = HashSet::new();
        self.cli_schema_for_type_inner(ty, span, callee_label, strict, &mut visiting)
    }

    fn cli_schema_for_type_inner(
        &mut self,
        ty: &Type,
        span: Span,
        callee_label: &str,
        strict: bool,
        visiting: &mut HashSet<Symbol>,
    ) -> Option<CliSchema> {
        match self.resolve_type(ty) {
            Type::Record(record_name, args) => self.cli_record_schema_for_type(
                record_name,
                args,
                span,
                callee_label,
                strict,
                visiting,
            ),
            Type::Enum(enum_name, args) if strict => {
                self.cli_command_schema_for_enum(enum_name, args, span, callee_label, visiting)
            }
            Type::Enum(enum_name, args) => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`{callee_label}` supports only record targets because defaults/overlays cannot represent command enum dispatch; found {}",
                        self.type_label(&Type::Enum(enum_name, args))
                    ),
                    span,
                )
                .with_suggestion("use `cli::parse`, `cli::parse_request`, `cli::usage_for_required`, or `cli::help_for_required` for command enum targets"));
                None
            }
            other => {
                self.diagnose_cli_schema_unsupported(&other, span, callee_label);
                None
            }
        }
    }

    fn cli_record_schema_for_type(
        &mut self,
        record_name: Symbol,
        args: Vec<Type>,
        span: Span,
        callee_label: &str,
        strict: bool,
        visiting: &mut HashSet<Symbol>,
    ) -> Option<CliSchema> {
        if !args.is_empty() {
            self.diagnose_cli_schema_unsupported(
                &Type::Record(record_name, args),
                span,
                callee_label,
            );
            return None;
        }
        let Some(record) = self.records.get(&record_name).cloned() else {
            self.diagnostics.push(Diagnostic::new(
                "T006",
                format!(
                    "`{callee_label}` cannot inspect record {}",
                    self.type_label(&Type::Record(record_name, Vec::new()))
                ),
                span,
            ));
            return None;
        };
        if !record.type_params.is_empty() {
            self.diagnose_cli_schema_unsupported(
                &Type::Record(record_name, Vec::new()),
                span,
                callee_label,
            );
            return None;
        }
        let wrapper_subcommand_field = record.fields.iter().find(|field| field.cli_subcommand);
        if wrapper_subcommand_field.is_some() && !strict {
            self.diagnostics.push(
                Diagnostic::new(
                    "T006",
                    format!(
                        "`{callee_label}` does not support wrapper record `{}` because defaults/overlays cannot represent every command payload",
                        self.symbols.resolve(record_name)
                    ),
                    span,
                )
                .with_suggestion(
                    "use `cli::parse`, `cli::parse_request`, `cli::usage_for_required`, or `cli::help_for_required` for wrapper records",
                ),
            );
            return None;
        }

        let mut fields = Vec::new();
        let mut subcommand = None;
        let mut valid = true;
        for field in &record.fields {
            let field_ty = self.record_field_type(field, &record.type_params, &[]);
            if field.cli_subcommand {
                let resolved_field_ty = self.resolve_type(&field_ty);
                let Type::Enum(enum_name, args) = resolved_field_ty else {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "T006",
                            format!(
                                "`{callee_label}` wrapper field `{}::{}` must have a concrete command enum type",
                                self.symbols.resolve(record_name),
                                self.symbols.resolve(field.name)
                            ),
                            field.span,
                        )
                        .with_suggestion(
                            "use a non-generic enum whose variants carry command record payloads",
                        ),
                    );
                    valid = false;
                    continue;
                };
                let Some(schema) = self.cli_command_schema_for_enum(
                    enum_name,
                    args,
                    field.span,
                    callee_label,
                    visiting,
                ) else {
                    valid = false;
                    continue;
                };
                subcommand = Some(CliSubcommandSchema {
                    field_name: field.name,
                    schema: Box::new(schema),
                });
                continue;
            }
            if wrapper_subcommand_field.is_some() && field.cli_position.is_some() {
                self.diagnostics.push(
                    Diagnostic::new(
                        "T006",
                        format!(
                            "`{callee_label}` wrapper record `{}` does not support root positional field `{}` before a subcommand",
                            self.symbols.resolve(record_name),
                            self.symbols.resolve(field.name)
                        ),
                        field.span,
                    )
                    .with_suggestion(
                        "use a root option field, or move the positional argument into a command payload record",
                    ),
                );
                valid = false;
                continue;
            }
            let Some(schema) = self.cli_field_schema_for_type(&field_ty) else {
                if strict {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "T006",
                            format!(
                                "`{callee_label}` does not support field `{}` because strict parsing cannot preserve unsupported fields without defaults",
                                self.symbols.resolve(field.name)
                            ),
                            field.span,
                        )
                        .with_suggestion("use a String, Int, Bool, Option[String|Int|Bool], List[String|Int|Bool], or zero-payload enum field"),
                    );
                    continue;
                }
                if field.cli_name.is_some()
                    || field.cli_short.is_some()
                    || field.cli_position.is_some()
                    || field.cli_value_source.is_some()
                    || !field.cli_aliases.is_empty()
                    || field.cli_help.is_some()
                    || field.cli_hidden
                    || field.cli_subcommand
                {
                    self.diagnostics.push(Diagnostic::new(
                        "T006",
                        format!(
                            "`{callee_label}` does not support field `{}` with explicit `@cli(...)` metadata",
                            self.symbols.resolve(field.name)
                        ),
                        field.span,
                    )
                    .with_suggestion("use a String, Int, Bool, Option[String|Int|Bool], List[String|Int|Bool], or zero-payload enum field"));
                }
                continue;
            };
            if strict && field.cli_hidden && !Self::cli_schema_can_synthesize_absent_value(&schema)
            {
                self.diagnostics.push(
                    Diagnostic::new(
                        "T006",
                        format!(
                            "`{callee_label}` hidden field `{}` must be optional, Bool, or a supported List because strict parsing cannot require hidden options",
                            self.symbols.resolve(field.name)
                        ),
                        field.span,
                    )
                    .with_suggestion("make the hidden field Option[T], Bool, or List[T], or remove @cli(hidden)"),
                );
                continue;
            }
            let option_name = field.cli_name.or(field.json_rename).unwrap_or(field.name);
            fields.push(CliFieldSchema {
                name: field.name,
                option_name,
                short: field.cli_short,
                position: field.cli_position,
                value_source: field.cli_value_source,
                aliases: field.cli_aliases.clone(),
                help: field.cli_help,
                hidden: field.cli_hidden,
                validation: field.json_validation.clone(),
                value: schema,
            });
        }
        if !valid {
            return None;
        }

        Some(CliSchema {
            type_name: record_name,
            package_item: self.package_record_items.get(&record_name).copied(),
            about: record.cli_about,
            fields,
            commands: Vec::new(),
            subcommand,
        })
    }

    fn cli_command_schema_for_enum(
        &mut self,
        enum_name: Symbol,
        args: Vec<Type>,
        span: Span,
        callee_label: &str,
        visiting: &mut HashSet<Symbol>,
    ) -> Option<CliSchema> {
        if !args.is_empty() {
            self.diagnose_cli_schema_unsupported(&Type::Enum(enum_name, args), span, callee_label);
            return None;
        }
        let Some(enumeration) = self.enums.get(&enum_name).cloned() else {
            self.diagnostics.push(Diagnostic::new(
                "T006",
                format!(
                    "`{callee_label}` cannot inspect enum {}",
                    self.type_label(&Type::Enum(enum_name, Vec::new()))
                ),
                span,
            ));
            return None;
        };
        if !enumeration.type_params.is_empty() {
            self.diagnose_cli_schema_unsupported(
                &Type::Enum(enum_name, Vec::new()),
                span,
                callee_label,
            );
            return None;
        }
        if !visiting.insert(enum_name) {
            self.diagnostics.push(Diagnostic::new(
                "T006",
                format!(
                    "`{callee_label}` cannot derive a CLI command schema for recursive enum {}",
                    self.type_label(&Type::Enum(enum_name, Vec::new()))
                ),
                span,
            ));
            return None;
        }

        let mut valid = true;
        if enumeration.variants.is_empty() {
            self.diagnostics.push(Diagnostic::new(
                "T006",
                format!(
                    "`{callee_label}` command enum `{}` must have at least one variant",
                    self.symbols.resolve(enum_name)
                ),
                span,
            ));
            valid = false;
        }

        let mut command_names = HashMap::new();
        let mut commands = Vec::with_capacity(enumeration.variants.len());
        for variant in &enumeration.variants {
            let qualified = self.qualified_variant(enum_name, variant.name);
            let Some(command_name) = variant.cli_name else {
                self.diagnostics.push(
                    Diagnostic::new(
                        "T006",
                        format!(
                            "`{callee_label}` command variant `{qualified}` requires `@cli(name: \"...\")`"
                        ),
                        variant.span,
                    )
                    .with_suggestion("write `@cli(name: \"command-name\")` on every command enum variant"),
                );
                valid = false;
                continue;
            };

            let mut names_for_variant = HashSet::new();
            for name in std::iter::once(command_name).chain(variant.cli_aliases.iter().copied()) {
                if !names_for_variant.insert(name) {
                    self.diagnostics.push(Diagnostic::new(
                        "E002",
                        format!(
                            "duplicate CLI command name `{}` in command variant `{qualified}`",
                            self.symbols.resolve(name)
                        ),
                        variant.span,
                    ));
                    valid = false;
                    continue;
                }
                if let Some(previous_span) = command_names.insert(name, variant.span) {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "E002",
                            format!(
                                "duplicate CLI command name `{}` in enum `{}`",
                                self.symbols.resolve(name),
                                self.symbols.resolve(enum_name)
                            ),
                            variant.span,
                        )
                        .with_related("previous CLI command name is here", previous_span),
                    );
                    valid = false;
                }
            }

            let Some(payload_ty) =
                self.enum_variant_payload_type(variant, &enumeration.type_params, &[])
            else {
                self.diagnostics.push(
                    Diagnostic::new(
                        "T006",
                        format!(
                            "`{callee_label}` command variant `{qualified}` must carry a record or command enum payload"
                        ),
                        variant.span,
                    )
                    .with_suggestion("add a payload record type, even when the command has no options"),
                );
                valid = false;
                continue;
            };
            let payload_ty = self.resolve_type(&payload_ty);
            if !matches!(payload_ty, Type::Record(_, _) | Type::Enum(_, _)) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "T006",
                        format!(
                            "`{callee_label}` command variant `{qualified}` has unsupported payload type {}; use a concrete record or nested command enum",
                            self.type_label(&payload_ty)
                        ),
                        variant.span,
                    )
                    .with_suggestion("use a non-generic record payload, or another command enum for nested subcommands"),
                );
                valid = false;
                continue;
            }
            let Some(payload) = self.cli_schema_for_type_inner(
                &payload_ty,
                variant.span,
                callee_label,
                true,
                visiting,
            ) else {
                valid = false;
                continue;
            };
            commands.push(CliCommandVariantSchema {
                variant_name: variant.name,
                command_name,
                aliases: variant.cli_aliases.clone(),
                about: variant.cli_about,
                hidden: variant.cli_hidden,
                payload: Box::new(payload),
            });
        }
        visiting.remove(&enum_name);

        if !valid {
            return None;
        }
        Some(CliSchema {
            type_name: enum_name,
            package_item: self.package_enum_items.get(&enum_name).copied(),
            about: enumeration.cli_about,
            fields: Vec::new(),
            commands,
            subcommand: None,
        })
    }

    fn cli_field_schema_for_type(&mut self, ty: &Type) -> Option<CliValueSchema> {
        match self.resolve_type(ty) {
            Type::String => Some(CliValueSchema::String),
            Type::Int => Some(CliValueSchema::Int),
            Type::Bool => Some(CliValueSchema::Bool),
            Type::Option(item) => match self.resolve_type(&item) {
                Type::String => Some(CliValueSchema::Option(Box::new(CliValueSchema::String))),
                Type::Int => Some(CliValueSchema::Option(Box::new(CliValueSchema::Int))),
                Type::Bool => Some(CliValueSchema::Option(Box::new(CliValueSchema::Bool))),
                Type::Enum(enum_name, args)
                    if args.is_empty() && !self.std_json_value_symbols.contains(&enum_name) =>
                {
                    self.cli_enum_schema_for_type(enum_name)
                        .map(|schema| CliValueSchema::Option(Box::new(schema)))
                }
                _ => None,
            },
            Type::List(item) => match self.resolve_type(&item) {
                Type::String => Some(CliValueSchema::StringList),
                Type::Int => Some(CliValueSchema::IntList),
                Type::Bool => Some(CliValueSchema::BoolList),
                Type::Enum(enum_name, args)
                    if args.is_empty() && !self.std_json_value_symbols.contains(&enum_name) =>
                {
                    self.cli_enum_schema_for_type(enum_name).and_then(|schema| {
                        if let CliValueSchema::Enum {
                            type_name,
                            package_item,
                            variants,
                        } = schema
                        {
                            Some(CliValueSchema::EnumList {
                                type_name,
                                package_item,
                                variants,
                            })
                        } else {
                            None
                        }
                    })
                }
                _ => None,
            },
            Type::Enum(enum_name, args)
                if args.is_empty() && !self.std_json_value_symbols.contains(&enum_name) =>
            {
                self.cli_enum_schema_for_type(enum_name)
            }
            _ => None,
        }
    }

    fn cli_enum_schema_for_type(&mut self, enum_name: Symbol) -> Option<CliValueSchema> {
        let enumeration = self.enums.get(&enum_name).cloned()?;
        if !enumeration.type_params.is_empty() {
            return None;
        }
        let mut variants = Vec::with_capacity(enumeration.variants.len());
        for variant in &enumeration.variants {
            if variant.payload.is_some() {
                return None;
            }
            variants.push(CliEnumVariantSchema {
                name: variant.name,
                tag: variant.json_rename.unwrap_or(variant.name),
            });
        }
        Some(CliValueSchema::Enum {
            type_name: enum_name,
            package_item: self.package_enum_items.get(&enum_name).copied(),
            variants,
        })
    }

    fn cli_schema_can_synthesize_absent_value(schema: &CliValueSchema) -> bool {
        matches!(
            schema,
            CliValueSchema::Bool
                | CliValueSchema::Option(_)
                | CliValueSchema::StringList
                | CliValueSchema::IntList
                | CliValueSchema::BoolList
                | CliValueSchema::EnumList { .. }
        )
    }

    fn cli_schema_is_list(schema: &CliValueSchema) -> bool {
        matches!(
            schema,
            CliValueSchema::StringList
                | CliValueSchema::IntList
                | CliValueSchema::BoolList
                | CliValueSchema::EnumList { .. }
        )
    }

    fn cli_value_source_allowed_for_schema(schema: &CliValueSchema) -> bool {
        match schema {
            CliValueSchema::String | CliValueSchema::StringList => true,
            CliValueSchema::Option(item) => matches!(item.as_ref(), CliValueSchema::String),
            _ => false,
        }
    }

    fn diagnose_cli_schema_unsupported(&mut self, ty: &Type, span: Span, callee_label: &str) {
        self.diagnostics.push(Diagnostic::new(
            "T006",
            format!(
                "`{callee_label}` supports only concrete non-generic record targets in this slice; found {}",
                self.type_label(ty)
            ),
            span,
        ));
    }

    fn json_decode_schema_for_type(
        &mut self,
        ty: &Type,
        span: Span,
        visiting: &mut HashSet<Symbol>,
        callee_label: &str,
    ) -> Option<JsonDecodeSchema> {
        self.json_schema_for_type(ty, span, visiting, callee_label, false)
    }

    fn json_encode_schema_for_type(
        &mut self,
        ty: &Type,
        span: Span,
        visiting: &mut HashSet<Symbol>,
        callee_label: &str,
    ) -> Option<JsonDecodeSchema> {
        self.json_schema_for_type(ty, span, visiting, callee_label, true)
    }

    fn json_schema_for_type(
        &mut self,
        ty: &Type,
        span: Span,
        visiting: &mut HashSet<Symbol>,
        callee_label: &str,
        allow_json_value: bool,
    ) -> Option<JsonDecodeSchema> {
        let ty = self.resolve_type(ty);
        if allow_json_value && self.is_std_json_value_type(&ty) {
            return Some(JsonDecodeSchema::JsonValue);
        }
        match ty {
            Type::String => Some(JsonDecodeSchema::String),
            Type::Int => Some(JsonDecodeSchema::Int),
            Type::Bool => Some(JsonDecodeSchema::Bool),
            Type::List(item) => match self.resolve_type(&item) {
                Type::String => Some(JsonDecodeSchema::StringList),
                Type::Int => Some(JsonDecodeSchema::IntList),
                Type::Bool => Some(JsonDecodeSchema::BoolList),
                other => self
                    .json_schema_for_type(&other, span, visiting, callee_label, allow_json_value)
                    .map(|schema| JsonDecodeSchema::List(Box::new(schema))),
            },
            Type::Option(item) => {
                let item = self.resolve_type(&item);
                if matches!(item, Type::Option(_)) {
                    self.diagnose_json_schema_unsupported(
                        &Type::Option(Box::new(item)),
                        span,
                        callee_label,
                        allow_json_value,
                    );
                    return None;
                }
                self.json_schema_for_type(&item, span, visiting, callee_label, allow_json_value)
                    .map(|schema| JsonDecodeSchema::Option(Box::new(schema)))
            }
            Type::Map(key, value)
                if matches!(self.resolve_type(&key), Type::String)
                    && self.is_std_json_value_type(&value) =>
            {
                Some(JsonDecodeSchema::JsonObjectMap)
            }
            Type::Map(key, value) if matches!(self.resolve_type(&key), Type::String) => self
                .json_schema_for_type(&value, span, visiting, callee_label, allow_json_value)
                .map(|schema| JsonDecodeSchema::TypedStringMap(Box::new(schema))),
            Type::Map(key, value) => {
                self.diagnose_json_schema_unsupported(
                    &Type::Map(
                        Box::new(self.resolve_type(&key)),
                        Box::new(self.resolve_type(&value)),
                    ),
                    span,
                    callee_label,
                    allow_json_value,
                );
                None
            }
            Type::Record(record_name, args) => {
                if !args.is_empty() {
                    self.diagnose_json_schema_unsupported(
                        &Type::Record(record_name, args.clone()),
                        span,
                        callee_label,
                        allow_json_value,
                    );
                    return None;
                }
                let Some(record) = self.records.get(&record_name).cloned() else {
                    self.diagnostics.push(Diagnostic::new(
                        "T006",
                        format!(
                            "`{callee_label}` cannot inspect record {}",
                            self.type_label(&Type::Record(record_name, Vec::new()))
                        ),
                        span,
                    ));
                    return None;
                };
                if !record.type_params.is_empty() {
                    self.diagnose_json_schema_unsupported(
                        &Type::Record(record_name, Vec::new()),
                        span,
                        callee_label,
                        allow_json_value,
                    );
                    return None;
                }
                if !visiting.insert(record_name) {
                    self.diagnostics.push(Diagnostic::new(
                        "T006",
                        format!(
                            "`{callee_label}` does not support recursive record schema {}",
                            self.type_label(&Type::Record(record_name, Vec::new()))
                        ),
                        span,
                    ));
                    return None;
                }

                let mut fields = Vec::with_capacity(record.fields.len());
                for field in &record.fields {
                    let field_ty = self.record_field_type(field, &record.type_params, &[]);
                    let Some(schema) = self.json_schema_for_type(
                        &field_ty,
                        field.span,
                        visiting,
                        callee_label,
                        allow_json_value,
                    ) else {
                        visiting.remove(&record_name);
                        return None;
                    };
                    fields.push(JsonDecodeFieldSchema {
                        name: field.name,
                        wire_name: field.json_rename,
                        aliases: field.json_aliases.clone(),
                        validation: field.json_validation.clone(),
                        schema,
                    });
                }
                visiting.remove(&record_name);
                Some(JsonDecodeSchema::Record {
                    type_name: record_name,
                    package_item: self.package_record_items.get(&record_name).copied(),
                    deny_unknown_fields: record.json_deny_unknown_fields,
                    fields,
                })
            }
            Type::Enum(enum_name, args) => {
                if !args.is_empty() || self.std_json_value_symbols.contains(&enum_name) {
                    self.diagnose_json_schema_unsupported(
                        &Type::Enum(enum_name, args.clone()),
                        span,
                        callee_label,
                        allow_json_value,
                    );
                    return None;
                }
                let Some(enumeration) = self.enums.get(&enum_name).cloned() else {
                    self.diagnostics.push(Diagnostic::new(
                        "T006",
                        format!(
                            "`{callee_label}` cannot inspect enum {}",
                            self.type_label(&Type::Enum(enum_name, Vec::new()))
                        ),
                        span,
                    ));
                    return None;
                };
                if !enumeration.type_params.is_empty() {
                    self.diagnose_json_schema_unsupported(
                        &Type::Enum(enum_name, Vec::new()),
                        span,
                        callee_label,
                        allow_json_value,
                    );
                    return None;
                }
                if !visiting.insert(enum_name) {
                    self.diagnostics.push(Diagnostic::new(
                        "T006",
                        format!(
                            "`{callee_label}` does not support recursive enum schema {}",
                            self.type_label(&Type::Enum(enum_name, Vec::new()))
                        ),
                        span,
                    ));
                    return None;
                }

                let mut variants = Vec::with_capacity(enumeration.variants.len());
                for variant in &enumeration.variants {
                    let payload = if let Some(payload_ty) =
                        self.enum_variant_payload_type(variant, &enumeration.type_params, &[])
                    {
                        let Some(schema) = self.json_schema_for_type(
                            &payload_ty,
                            variant.span,
                            visiting,
                            callee_label,
                            allow_json_value,
                        ) else {
                            visiting.remove(&enum_name);
                            return None;
                        };
                        Some(schema)
                    } else {
                        None
                    };
                    variants.push(JsonDecodeVariantSchema {
                        name: variant.name,
                        wire_name: variant.json_rename,
                        aliases: variant.json_aliases.clone(),
                        payload,
                    });
                }
                visiting.remove(&enum_name);
                Some(JsonDecodeSchema::Enum {
                    type_name: enum_name,
                    package_item: self.package_enum_items.get(&enum_name).copied(),
                    variants,
                })
            }
            Type::Unknown(_) => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        span,
                    )
                    .with_suggestion(format!(
                        "pass a concrete fallback record or scalar to `{callee_label}`"
                    )),
                );
                None
            }
            Type::Error => None,
            other => {
                self.diagnose_json_schema_unsupported(&other, span, callee_label, allow_json_value);
                None
            }
        }
    }

    fn is_std_json_value_type(&mut self, ty: &Type) -> bool {
        match self.resolve_type(ty) {
            Type::Enum(symbol, args) => {
                args.is_empty() && self.std_json_value_symbols.contains(&symbol)
            }
            _ => false,
        }
    }

    fn diagnose_json_schema_unsupported(
        &mut self,
        ty: &Type,
        span: Span,
        callee_label: &str,
        allow_json_value: bool,
    ) {
        let value_support = if allow_json_value {
            "std::json::Value, "
        } else {
            ""
        };
        self.diagnostics.push(Diagnostic::new(
            "T006",
            format!(
                "`{callee_label}` supports only {value_support}String, Int, Bool, Option[T], List[T], Map[String, T], Map[String, json::Value], concrete non-generic records, and concrete non-generic enums over supported payloads in this slice; found {}",
                self.type_label(ty)
            ),
            span,
        ));
    }

    fn check_insert_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 3 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 3 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let expected = expected.map(|ty| self.resolve_type(&ty));
        let expected_map = match expected.as_ref() {
            Some(Type::Map(key, value)) => Some((*key.clone(), *value.clone())),
            _ => None,
        };

        if Self::is_map_empty_call(&expr.args[0]) {
            let key_ty = if let Some((key_ty, _)) = expected_map.as_ref() {
                self.check_expr_with_expected(&expr.args[1], Some(key_ty.clone()))
            } else {
                self.check_expr(&expr.args[1])
            };
            let key_ty = self.resolve_type(&key_ty);
            if !self.validate_map_key_type(&key_ty, expr.args[1].span()) {
                return Type::Error;
            }

            let value_ty = if let Some((_, value_ty)) = expected_map.as_ref() {
                self.check_expr_with_expected(&expr.args[2], Some(value_ty.clone()))
            } else {
                self.check_expr(&expr.args[2])
            };
            let value_ty = self.resolve_type(&value_ty);
            let map_ty = Type::Map(Box::new(key_ty), Box::new(value_ty));
            let base_ty = self.check_expr_with_expected(&expr.args[0], Some(map_ty.clone()));
            if matches!(self.resolve_type(&base_ty), Type::Error) {
                return Type::Error;
            }
            return self.apply_expected_map(map_ty, expected, expr.span);
        }

        let base_expected = match expected.as_ref() {
            Some(Type::Map(_, _)) => expected.clone(),
            _ => None,
        };
        let base_ty = self.check_expr_with_expected(&expr.args[0], base_expected);
        match self.resolve_type(&base_ty) {
            Type::Map(key_ty, value_ty) => {
                if !self.validate_map_key_type(&key_ty, expr.args[0].span()) {
                    return Type::Error;
                }
                self.check_expr_with_expected(&expr.args[1], Some((*key_ty).clone()));
                self.check_expr_with_expected(&expr.args[2], Some((*value_ty).clone()));
                self.apply_expected_map(Type::Map(key_ty, value_ty), expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        expr.span,
                    )
                    .with_suggestion("annotate the receiver as Map[K, V] before calling `insert`"),
                );
                Type::Error
            }
            Type::Error => Type::Error,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`insert` expects Map[K, V] as its first argument but found {}",
                        self.type_label(&other)
                    ),
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_remove_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let expected = expected.map(|ty| self.resolve_type(&ty));
        if Self::is_map_empty_call(&expr.args[0]) {
            return match expected.clone() {
                Some(Type::Map(key_ty, value_ty)) => {
                    self.check_expr_with_expected(&expr.args[1], Some((*key_ty).clone()));
                    if !self.validate_map_key_type(&key_ty, expr.args[1].span()) {
                        return Type::Error;
                    }
                    let map_ty = Type::Map(key_ty, value_ty);
                    let base_ty =
                        self.check_expr_with_expected(&expr.args[0], Some(map_ty.clone()));
                    if matches!(self.resolve_type(&base_ty), Type::Error) {
                        Type::Error
                    } else {
                        self.apply_expected_map(map_ty, expected, expr.span)
                    }
                }
                Some(Type::Error) => Type::Error,
                Some(_) | None => {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "T019",
                            "`Map.empty().remove(...)` requires an expected Map[K, V] type",
                            expr.span,
                        )
                        .with_suggestion(
                            "add a local binding annotation such as `items: Map[String, Int] = Map.empty().remove(\"missing\")`",
                        ),
                    );
                    Type::Error
                }
            };
        }

        let base_expected = match expected.as_ref() {
            Some(Type::Map(_, _)) => expected.clone(),
            _ => None,
        };
        let base_ty = self.check_expr_with_expected(&expr.args[0], base_expected);
        match self.resolve_type(&base_ty) {
            Type::Map(key_ty, value_ty) => {
                if !self.validate_map_key_type(&key_ty, expr.args[0].span()) {
                    return Type::Error;
                }
                self.check_expr_with_expected(&expr.args[1], Some((*key_ty).clone()));
                self.apply_expected_map(Type::Map(key_ty, value_ty), expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        "type annotation required because inference is not unique",
                        expr.span,
                    )
                    .with_suggestion("annotate the receiver as Map[K, V] before calling `remove`"),
                );
                Type::Error
            }
            Type::Error => Type::Error,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    format!(
                        "`remove` expects Map[K, V] as its first argument but found {}",
                        self.type_label(&other)
                    ),
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn apply_expected_option(
        &mut self,
        option_ty: Type,
        expected: Option<Type>,
        span: Span,
    ) -> Type {
        match expected {
            None => option_ty,
            Some(expected) => self.apply_expected(option_ty, Some(expected), span),
        }
    }

    fn apply_expected_map(&mut self, map_ty: Type, expected: Option<Type>, span: Span) -> Type {
        match expected {
            None => map_ty,
            Some(expected) => self.apply_expected(map_ty, Some(expected), span),
        }
    }

    fn validate_map_key_type(&mut self, key_ty: &Type, span: Span) -> bool {
        match self.resolve_type(key_ty) {
            Type::Int
            | Type::Bool
            | Type::String
            | Type::GenericParam(_)
            | Type::Unknown(_)
            | Type::Error => true,
            _ => {
                self.diagnostics.push(
                    Diagnostic::new("T020", "Map key type must be Int, Bool, or String", span)
                        .with_suggestion("use Int, Bool, or String as the Map key type"),
                );
                false
            }
        }
    }

    fn is_map_empty_call(expr: &Expr) -> bool {
        matches!(
            expr,
            Expr::Call(call)
                if matches!(
                    call.callee.as_ref(),
                    Expr::Ident(IdentExpr { name, .. }) if name == "Map.empty"
                )
        )
    }

    fn is_empty_list_literal(expr: &Expr) -> bool {
        matches!(expr, Expr::ListLit(list) if list.items.is_empty())
    }

    fn is_option_none_literal(expr: &Expr) -> bool {
        matches!(expr, Expr::Ident(IdentExpr { name, .. }) if name == "Option::None")
    }

    fn record_field_value_needs_expected_type(expr: &Expr) -> bool {
        Self::is_empty_list_literal(expr)
            || Self::is_map_empty_call(expr)
            || Self::is_option_none_literal(expr)
    }

    fn check_option_none(&mut self, expected: Option<Type>, span: Span) -> Type {
        let expected = expected.map(|ty| self.resolve_type(&ty));
        match expected {
            Some(Type::Option(item)) => Type::Option(item),
            Some(Type::Error) => Type::Error,
            Some(_) | None => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "T017",
                        "`Option::None` requires an expected Option[T] type",
                        span,
                    )
                    .with_suggestion(
                        "add a local binding annotation such as `value: Option[Int] = Option::None`",
                    ),
                );
                Type::Error
            }
        }
    }

    fn check_result_constructor_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        variant_name: &'static str,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let expected = expected.map(|ty| self.resolve_type(&ty));
        let (ok_ty, err_ty) = match expected {
            Some(Type::Result(ok_ty, err_ty)) => (ok_ty, err_ty),
            Some(Type::Error) => {
                self.check_expr(&expr.args[0]);
                return Type::Error;
            }
            Some(_) | None => {
                self.check_expr(&expr.args[0]);
                self.diagnostics.push(
                    Diagnostic::new(
                        "T021",
                        format!(
                            "`Result::{variant_name}` requires an expected Result[T, E] type"
                        ),
                        expr.span,
                    )
                    .with_suggestion(
                        "add a local binding annotation such as `value: Result[Int, String] = Result::Ok(1)`",
                    ),
                );
                return Type::Error;
            }
        };

        let payload_ty = if variant_name == known_enum::RESULT_OK_NAME {
            (*ok_ty).clone()
        } else {
            (*err_ty).clone()
        };
        self.check_expr_with_expected(&expr.args[0], Some(payload_ty));
        Type::Result(ok_ty, err_ty)
    }

    fn check_try_expr(&mut self, expr: &TryExpr, expected: Option<Type>) -> Type {
        let Some((_return_ok, return_err)) = self.current_result_return(expr.span) else {
            self.check_invalid_return_try_operand(&expr.expr);
            return Type::Error;
        };

        if let Some(non_result_ty) = self.obvious_non_result_try_type(&expr.expr) {
            self.check_expr(&expr.expr);
            self.diagnostics.push(Diagnostic::new(
                "T023",
                format!(
                    "`try` expects a Result[T, E] expression but found {}",
                    self.type_label(&non_result_ty)
                ),
                expr.span,
            ));
            return Type::Error;
        }

        let value_expected = expected
            .clone()
            .unwrap_or_else(|| Type::Unknown(self.fresh_unknown()));
        let result_expected = Type::Result(Box::new(value_expected), Box::new(return_err));
        let result_ty = self.check_expr_with_expected(&expr.expr, Some(result_expected));
        match self.resolve_type(&result_ty) {
            Type::Result(ok_ty, _) => self.apply_expected(*ok_ty, expected, expr.span),
            Type::Error => Type::Error,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T023",
                    format!(
                        "`try` expects a Result[T, E] expression but found {}",
                        self.type_label(&other)
                    ),
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_invalid_return_try_operand(&mut self, expr: &Expr) {
        if Self::is_result_constructor_call(expr) {
            let ok_ty = Type::Unknown(self.fresh_unknown());
            let err_ty = Type::Unknown(self.fresh_unknown());
            self.check_expr_with_expected(
                expr,
                Some(Type::Result(Box::new(ok_ty), Box::new(err_ty))),
            );
        } else {
            self.check_expr(expr);
        }
    }

    fn is_result_constructor_call(expr: &Expr) -> bool {
        let Expr::Call(call) = expr else {
            return false;
        };
        let Expr::Ident(callee) = call.callee.as_ref() else {
            return false;
        };
        callee.name == known_enum::RESULT_OK_QUALIFIED
            || callee.name == known_enum::RESULT_ERR_QUALIFIED
    }

    fn obvious_non_result_try_type(&self, expr: &Expr) -> Option<Type> {
        match expr {
            Expr::Int(_) => Some(Type::Int),
            Expr::Bool(_) => Some(Type::Bool),
            Expr::String(_) => Some(Type::String),
            Expr::Unit(_) => Some(Type::Unit),
            Expr::Ident(expr) => self
                .symbols
                .lookup(&expr.name)
                .and_then(|name| self.lookup(name))
                .and_then(|binding| self.non_result_try_type(&binding.ty)),
            Expr::Call(expr) => self
                .direct_call_return_type(&expr.callee)
                .and_then(|ty| self.non_result_try_type(&ty)),
            _ => None,
        }
    }

    fn direct_call_return_type(&self, callee: &Expr) -> Option<Type> {
        let Expr::Ident(expr) = callee else {
            return None;
        };
        let name = self.symbols.lookup(&expr.name)?;
        let binding = self.lookup(name)?;
        match self.resolve_type(&binding.ty) {
            Type::Function(sig) => Some(*sig.ret),
            _ => None,
        }
    }

    fn non_result_try_type(&self, ty: &Type) -> Option<Type> {
        match self.resolve_type(ty) {
            Type::Result(_, _) | Type::Unknown(_) | Type::Error => None,
            other => Some(other),
        }
    }

    fn current_result_return(&mut self, span: Span) -> Option<(Type, Type)> {
        let Some(return_ty) = self.current_function_returns.last().cloned() else {
            self.diagnostics.push(Diagnostic::new(
                "T023",
                "`try` is allowed only inside a function returning Result[T, E]",
                span,
            ));
            return None;
        };

        match self.resolve_type(&return_ty) {
            Type::Result(ok_ty, err_ty) => Some((*ok_ty, *err_ty)),
            Type::Unknown(_) => {
                let ok_ty = Type::Unknown(self.fresh_unknown());
                let err_ty = Type::Unknown(self.fresh_unknown());
                let result_ty = Type::Result(Box::new(ok_ty.clone()), Box::new(err_ty.clone()));
                if let Err(message) = self.unify(return_ty, result_ty) {
                    self.diagnostics
                        .push(Diagnostic::new("T023", message, span));
                    None
                } else {
                    Some((ok_ty, err_ty))
                }
            }
            Type::Error => None,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T023",
                    format!(
                        "`try` requires the enclosing function to return Result[T, E], found {}",
                        self.type_label(&other)
                    ),
                    span,
                ));
                None
            }
        }
    }

    fn current_using_result_return(&mut self, span: Span) -> Option<(Type, Type)> {
        let Some(return_ty) = self.current_function_returns.last().cloned() else {
            self.diagnostics.push(Diagnostic::new(
                "T027",
                "`using` is allowed only inside a function returning Result[T, E]",
                span,
            ));
            return None;
        };

        match self.resolve_type(&return_ty) {
            Type::Result(ok_ty, err_ty) => Some((*ok_ty, *err_ty)),
            Type::Unknown(_) => {
                let ok_ty = Type::Unknown(self.fresh_unknown());
                let err_ty = Type::Unknown(self.fresh_unknown());
                let result_ty = Type::Result(Box::new(ok_ty.clone()), Box::new(err_ty.clone()));
                if let Err(message) = self.unify(return_ty, result_ty) {
                    self.diagnostics
                        .push(Diagnostic::new("T027", message, span));
                    None
                } else {
                    Some((ok_ty, err_ty))
                }
            }
            Type::Error => None,
            other => {
                self.diagnostics.push(Diagnostic::new(
                    "T027",
                    format!(
                        "`using` requires the enclosing function to return Result[T, E], found {}",
                        self.type_label(&other)
                    ),
                    span,
                ));
                None
            }
        }
    }

    fn check_user_enum_constructor(
        &mut self,
        span: Span,
        expected: Option<Type>,
        enum_name: Symbol,
        variant_name: Symbol,
        args: &[Expr],
    ) -> Type {
        let Some(enumeration) = self.enums.get(&enum_name).cloned() else {
            for arg in args {
                self.check_expr(arg);
            }
            return Type::Error;
        };
        let Some(variant) = enumeration
            .variants
            .iter()
            .find(|variant| variant.name == variant_name)
            .cloned()
        else {
            for arg in args {
                self.check_expr(arg);
            }
            return Type::Error;
        };

        let expected = expected.map(|ty| self.resolve_type(&ty));
        let enum_args = match expected {
            Some(Type::Enum(expected_enum, args)) if expected_enum == enum_name => args,
            Some(Type::Error) => {
                for arg in args {
                    self.check_expr(arg);
                }
                return Type::Error;
            }
            Some(_) | None if !enumeration.type_params.is_empty() => {
                for arg in args {
                    self.check_expr(arg);
                }
                let enum_text = self.symbols.resolve(enum_name).to_string();
                let variant_text = self.symbols.resolve(variant_name).to_string();
                self.diagnostics.push(
                    Diagnostic::new(
                        "T022",
                        format!(
                            "`{enum_text}::{variant_text}` requires an expected {enum_text}[...] type"
                        ),
                        span,
                    )
                    .with_suggestion(format!(
                        "add a local binding annotation such as `value: {enum_text}[Int] = {enum_text}::{variant_text}`"
                    )),
                );
                return Type::Error;
            }
            Some(_) | None => Vec::new(),
        };

        let variant_payload =
            self.enum_variant_payload_type(&variant, &enumeration.type_params, &enum_args);
        match (variant_payload.as_ref(), args) {
            (None, []) => {}
            (None, _) => {
                self.diagnostics.push(Diagnostic::new(
                    "T004",
                    format!("expected 0 arguments but found {}", args.len()),
                    span,
                ));
                for arg in args {
                    self.check_expr(arg);
                }
                return Type::Error;
            }
            (Some(_), []) => {
                self.diagnostics.push(Diagnostic::new(
                    "T004",
                    "expected 1 arguments but found 0",
                    span,
                ));
                return Type::Error;
            }
            (Some(_), [_]) => {}
            (Some(_), _) => {
                self.diagnostics.push(Diagnostic::new(
                    "T004",
                    format!("expected 1 arguments but found {}", args.len()),
                    span,
                ));
                for arg in args {
                    self.check_expr(arg);
                }
                return Type::Error;
            }
        }

        let enum_ty = Type::Enum(enum_name, enum_args.clone());
        if let (Some(payload_expr), Some(payload_ty)) = (args.first(), variant_payload) {
            self.check_expr_with_expected(payload_expr, Some(payload_ty));
        }
        enum_ty
    }

    fn diagnose_unknown_enum_variant(&mut self, enum_name: &str, variant_name: &str, span: Span) {
        let enum_symbol = self.symbol(enum_name);
        let variant_symbol = self.symbol(variant_name);
        if let Some(enumeration) = self.enums.get(&enum_symbol) {
            if enumeration
                .variants
                .iter()
                .any(|variant| variant.name == variant_symbol)
            {
                return;
            }
            self.diagnostics.push(
                Diagnostic::new(
                    "T022",
                    format!("unknown variant `{variant_name}` for enum `{enum_name}`"),
                    span,
                )
                .with_related("enum is declared here", enumeration.span),
            );
            return;
        }

        self.diagnostics.push(Diagnostic::new(
            "T022",
            format!("unknown enum `{enum_name}` in variant constructor"),
            span,
        ));
    }

    fn check_match_expr(&mut self, expr: &MatchExpr, expected: Option<Type>) -> Type {
        let value_ty = self.check_expr(&expr.value);
        let spec = self.enum_match_spec_for_value(&value_ty, expr.value.span());
        let mut seen_variants = HashMap::new();
        let mut result_ty = None;
        let expected = expected.map(|ty| self.resolve_type(&ty));

        for arm in &expr.arms {
            self.push_scope(false);
            let MatchPattern::Variant(pattern) = &arm.pattern;
            let pattern_enum = self.symbol(&pattern.enum_name);
            let pattern_variant = self.symbol(&pattern.variant_name);
            if pattern_enum != spec.enum_name {
                self.diagnostics.push(Diagnostic::new(
                    "T018",
                    format!(
                        "`{}::{}` does not belong to {}",
                        pattern.enum_name, pattern.variant_name, spec.display_name
                    ),
                    pattern.span,
                ));
            } else if let Some(variant) = spec.variant(pattern_variant) {
                if let Some(previous) = seen_variants.insert(variant.name, pattern.span) {
                    let qualified = self.qualified_variant(spec.enum_name, variant.name);
                    self.diagnostics.push(
                        Diagnostic::new(
                            "T018",
                            format!("duplicate `{qualified}` match arm"),
                            pattern.span,
                        )
                        .with_related(format!("previous `{qualified}` arm is here"), previous),
                    );
                }
                match (&variant.payload, &pattern.payload) {
                    (Some(payload_ty), EnumVariantPatternPayload::Binding(binding)) => {
                        let name = self.symbol(binding);
                        self.insert_current(
                            name,
                            BindingKind::Immutable,
                            payload_ty.clone(),
                            pattern.span,
                        );
                    }
                    (Some(_), EnumVariantPatternPayload::Discard) => {}
                    (Some(_), EnumVariantPatternPayload::None) => {
                        let qualified = self.qualified_variant(spec.enum_name, variant.name);
                        self.diagnostics.push(Diagnostic::new(
                            "T018",
                            format!("`{qualified}` match arm must bind or discard its payload"),
                            pattern.span,
                        ));
                    }
                    (
                        None,
                        EnumVariantPatternPayload::Binding(_) | EnumVariantPatternPayload::Discard,
                    ) => {
                        let qualified = self.qualified_variant(spec.enum_name, variant.name);
                        self.diagnostics.push(Diagnostic::new(
                            "T018",
                            format!("`{qualified}` match arm does not carry a payload"),
                            pattern.span,
                        ));
                    }
                    (None, EnumVariantPatternPayload::None) => {}
                }
            } else {
                self.diagnostics.push(Diagnostic::new(
                    "T018",
                    format!(
                        "unknown variant `{}` in match for {}",
                        pattern.variant_name, spec.display_name
                    ),
                    pattern.span,
                ));
            }

            let arm_ty = if let Some(expected_ty) = expected.clone() {
                self.check_expr_with_expected(&arm.value, Some(expected_ty))
            } else if let Some(result_ty) = result_ty.clone() {
                self.check_expr_with_expected(&arm.value, Some(result_ty))
            } else {
                self.check_expr(&arm.value)
            };
            if result_ty.is_none() {
                result_ty = Some(arm_ty);
            }
            self.pop_scope();
        }

        for variant in &spec.variants {
            if !seen_variants.contains_key(&variant.name) {
                let qualified = self.qualified_variant(spec.enum_name, variant.name);
                self.diagnostics.push(Diagnostic::new(
                    "T018",
                    format!(
                        "`match` on {} requires an `{qualified}` arm",
                        spec.display_name
                    ),
                    expr.span,
                ));
            }
        }

        match expected {
            Some(expected) => self.resolve_type(&expected),
            None => result_ty.unwrap_or(Type::Error),
        }
    }

    fn enum_match_spec_for_value(&mut self, value_ty: &Type, value_span: Span) -> EnumMatchSpec {
        match self.resolve_type(value_ty) {
            Type::Option(item) => self.option_match_spec(*item),
            Type::Result(ok, err) => self.result_match_spec(*ok, *err),
            Type::Enum(enum_name, args) => self.user_enum_match_spec(enum_name, args),
            Type::Unknown(_) => {
                let item = Type::Unknown(self.fresh_unknown());
                let option = Type::Option(Box::new(item.clone()));
                if let Err(message) = self.unify(value_ty.clone(), option) {
                    self.diagnostics
                        .push(Diagnostic::new("T018", message, value_span));
                    self.option_match_spec(Type::Error)
                } else {
                    self.option_match_spec(item)
                }
            }
            Type::Error => self.option_match_spec(Type::Error),
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "T018",
                    "`match` requires an enum value",
                    value_span,
                ));
                self.option_match_spec(Type::Error)
            }
        }
    }

    fn option_match_spec(&mut self, item_ty: Type) -> EnumMatchSpec {
        let known = known_enum::option_enum();
        let enum_name = self.symbol(known.name);
        EnumMatchSpec {
            enum_name,
            display_name: "Option[T]".to_string(),
            variants: known
                .variants
                .iter()
                .copied()
                .map(|variant| EnumMatchVariant {
                    name: self.symbol(variant.name),
                    payload: if variant.has_payload {
                        Some(item_ty.clone())
                    } else {
                        None
                    },
                })
                .collect(),
        }
    }

    fn result_match_spec(&mut self, ok_ty: Type, err_ty: Type) -> EnumMatchSpec {
        let known = known_enum::result_enum();
        let enum_name = self.symbol(known.name);
        EnumMatchSpec {
            enum_name,
            display_name: "Result[T, E]".to_string(),
            variants: known
                .variants
                .iter()
                .copied()
                .map(|variant| {
                    let payload = match variant.name {
                        known_enum::RESULT_OK_NAME => Some(ok_ty.clone()),
                        known_enum::RESULT_ERR_NAME => Some(err_ty.clone()),
                        _ => None,
                    };
                    EnumMatchVariant {
                        name: self.symbol(variant.name),
                        payload,
                    }
                })
                .collect(),
        }
    }

    fn user_enum_match_spec(&mut self, enum_name: Symbol, args: Vec<Type>) -> EnumMatchSpec {
        let Some(enumeration) = self.enums.get(&enum_name).cloned() else {
            return EnumMatchSpec {
                enum_name,
                display_name: self.symbols.resolve(enum_name).to_string(),
                variants: Vec::new(),
            };
        };
        let mut variants = Vec::new();
        for variant in &enumeration.variants {
            let payload = self.enum_variant_payload_type(variant, &enumeration.type_params, &args);
            variants.push(EnumMatchVariant {
                name: variant.name,
                payload,
            });
        }
        EnumMatchSpec {
            enum_name,
            display_name: self.symbols.resolve(enum_name).to_string(),
            variants,
        }
    }

    fn enum_variant_payload_type(
        &mut self,
        variant: &EnumVariantDef,
        type_params: &[Symbol],
        type_args: &[Type],
    ) -> Option<Type> {
        if let Some(payload_ty) = &variant.payload_ty {
            return Some(self.substitute_type_params(payload_ty.clone(), type_params, type_args));
        }
        variant.payload.as_ref().map(|payload| {
            let payload_ty = self.type_from_expr_with_params(payload, variant.span, type_params);
            self.substitute_type_params(payload_ty, type_params, type_args)
        })
    }

    fn qualified_variant(&self, enum_name: Symbol, variant_name: Symbol) -> String {
        format!(
            "{}::{}",
            self.symbols.resolve(enum_name),
            self.symbols.resolve(variant_name)
        )
    }

    fn check_record_lit(&mut self, expr: &RecordLitExpr, expected: Option<&Type>) -> Type {
        let type_name = self.symbol(&expr.type_name);
        let Some(record) = self.records.get(&type_name).cloned() else {
            if self.package_opaque_items.contains_key(&type_name) {
                self.diagnostics.push(Diagnostic::new(
                    "T007",
                    format!(
                        "opaque type `{}` cannot be constructed with a record literal",
                        expr.type_name
                    ),
                    expr.span,
                ));
                for field in &expr.fields {
                    self.check_expr(&field.value);
                }
                return Type::Error;
            }
            self.diagnostics.push(Diagnostic::new(
                "T007",
                format!("unknown type `{}`", expr.type_name),
                expr.span,
            ));
            for field in &expr.fields {
                self.check_expr(&field.value);
            }
            return Type::Error;
        };
        let record_args = self.record_literal_type_args(type_name, &record, expected);

        let mut seen = HashSet::new();
        let mut has_error = false;
        for field in &expr.fields {
            let field_name = self.symbol(&field.name);
            if !seen.insert(field_name) {
                self.check_expr(&field.value);
                self.diagnostics.push(
                    Diagnostic::new(
                        "E009",
                        format!(
                            "invalid record literal for `{}`: duplicate field `{}`",
                            expr.type_name, field.name
                        ),
                        field.span,
                    )
                    .with_related("record is declared here", record.span),
                );
                has_error = true;
                continue;
            }

            let Some(declared) = find_record_field(&record, field_name) else {
                self.check_expr(&field.value);
                self.diagnostics.push(
                    Diagnostic::new(
                        "E009",
                        format!(
                            "invalid record literal for `{}`: unknown field `{}`",
                            expr.type_name, field.name
                        ),
                        field.span,
                    )
                    .with_related("record is declared here", record.span),
                );
                has_error = true;
                continue;
            };

            let field_ty = self.record_field_type(declared, &record.type_params, &record_args);
            let value_ty = if Self::record_field_value_needs_expected_type(&field.value) {
                self.check_expr_with_expected(&field.value, Some(field_ty.clone()))
            } else {
                self.check_expr(&field.value)
            };
            if let Err(message) = self.unify(field_ty, value_ty) {
                self.diagnostics.push(
                    Diagnostic::new("E009", message, field.span)
                        .with_related("field type is declared here", declared.span),
                );
                has_error = true;
            }
        }

        for declared in &record.fields {
            if !seen.contains(&declared.name) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E009",
                        format!(
                            "invalid record literal for `{}`: missing field `{}`",
                            expr.type_name,
                            self.symbols.resolve(declared.name)
                        ),
                        expr.span,
                    )
                    .with_related("required field is declared here", declared.span),
                );
                has_error = true;
            }
        }

        if has_error {
            Type::Error
        } else {
            let args = record_args
                .into_iter()
                .map(|arg| self.resolve_type(&arg))
                .collect::<Vec<_>>();
            if args.iter().any(Type::is_unknown) {
                let unresolved = record
                    .type_params
                    .iter()
                    .zip(args.iter())
                    .filter(|(_, arg)| arg.is_unknown())
                    .map(|(param, _)| format!("`{}`", self.symbols.resolve(*param)))
                    .collect::<Vec<_>>()
                    .join(", ");
                self.diagnostics.push(
                    Diagnostic::new(
                        "E005",
                        format!(
                            "type annotation required because record literal for `{}` cannot infer type argument {unresolved}",
                            expr.type_name
                        ),
                        expr.span,
                    )
                    .with_suggestion(format!(
                        "add a local binding, parameter, or return annotation with explicit `{}[...]` type arguments",
                        expr.type_name
                    )),
                );
                Type::Error
            } else {
                Type::Record(type_name, args)
            }
        }
    }

    fn record_literal_type_args(
        &mut self,
        type_name: Symbol,
        record: &RecordDef,
        expected: Option<&Type>,
    ) -> Vec<Type> {
        let expected_args = expected.and_then(|ty| match self.resolve_type(ty) {
            Type::Record(expected_name, args) if expected_name == type_name => Some(args),
            _ => None,
        });
        if let Some(args) = expected_args
            && args.len() == record.type_params.len()
        {
            return args;
        }
        record
            .type_params
            .iter()
            .map(|_| Type::Unknown(self.fresh_unknown()))
            .collect()
    }

    fn check_field_expr(&mut self, expr: &FieldExpr) -> Type {
        let base_ty = self.check_expr(&expr.base);
        let resolved_base = self.resolve_type(&base_ty);
        let Type::Record(record_name, record_args) = resolved_base else {
            self.diagnostics.push(Diagnostic::new(
                "T008",
                "field access requires a record value",
                expr.span,
            ));
            return Type::Error;
        };

        let Some(record) = self.records.get(&record_name).cloned() else {
            let record_name = self.symbols.resolve(record_name);
            self.diagnostics.push(Diagnostic::new(
                "T007",
                format!("unknown type `{record_name}`"),
                expr.span,
            ));
            return Type::Error;
        };

        let field_name = self.symbol(&expr.field);
        let Some(field) = find_record_field(&record, field_name) else {
            self.diagnostics.push(
                Diagnostic::new("E008", format!("unknown field `{}`", expr.field), expr.span)
                    .with_related("record is declared here", record.span),
            );
            return Type::Error;
        };

        self.record_field_type(field, &record.type_params, &record_args)
    }

    fn check_record_update(&mut self, expr: &RecordUpdateExpr) -> Type {
        let base_ty = self.check_expr(&expr.base);
        let resolved_base = self.resolve_type(&base_ty);
        let Type::Record(record_name, record_args) = resolved_base else {
            self.diagnostics
                .push(Diagnostic::new("E012", "invalid record update", expr.span));
            for field in &expr.fields {
                self.check_expr(&field.value);
            }
            return Type::Error;
        };

        let Some(record) = self.records.get(&record_name).cloned() else {
            let record_name = self.symbols.resolve(record_name);
            self.diagnostics.push(Diagnostic::new(
                "T007",
                format!("unknown type `{record_name}`"),
                expr.span,
            ));
            return Type::Error;
        };

        let mut seen = HashSet::new();
        let mut has_error = false;
        for field in &expr.fields {
            let value_ty = self.check_expr(&field.value);
            let field_name = self.symbol(&field.name);
            if !seen.insert(field_name) {
                self.diagnostics.push(
                    Diagnostic::new("E012", "invalid record update", field.span)
                        .with_related("record is declared here", record.span),
                );
                has_error = true;
                continue;
            }

            let Some(declared) = find_record_field(&record, field_name) else {
                self.diagnostics.push(
                    Diagnostic::new("E012", "invalid record update", field.span)
                        .with_related("record is declared here", record.span),
                );
                has_error = true;
                continue;
            };

            let field_ty = self.record_field_type(declared, &record.type_params, &record_args);
            if let Err(message) = self.unify(field_ty, value_ty) {
                self.diagnostics.push(
                    Diagnostic::new("E012", message, field.span)
                        .with_related("field type is declared here", declared.span),
                );
                has_error = true;
            }
        }

        if has_error {
            Type::Error
        } else {
            Type::Record(record_name, record_args)
        }
    }

    fn record_field_type(
        &mut self,
        field: &RecordField,
        type_params: &[Symbol],
        type_args: &[Type],
    ) -> Type {
        if let Some(ty) = &field.ty {
            return self.substitute_type_params(ty.clone(), type_params, type_args);
        }
        let Some(type_name) = &field.type_name else {
            return Type::Error;
        };
        let ty = self.type_from_expr_with_params(type_name, field.span, type_params);
        self.substitute_type_params(ty, type_params, type_args)
    }

    fn signature_from_fn_expr(&mut self, expr: &FnExpr, expected: Option<&Type>) -> FunctionSig {
        let expected_sig = self.expected_function_sig(expected, expr.params.len());
        let params = expr
            .params
            .iter()
            .enumerate()
            .map(|(index, param)| match param.type_name.as_ref() {
                Some(type_name) => self.type_from_expr(type_name, param.span),
                None => expected_sig
                    .as_ref()
                    .and_then(|sig| sig.params.get(index).cloned())
                    .unwrap_or_else(|| Type::Unknown(self.fresh_unknown())),
            })
            .collect();
        let ret = match expr.return_type.as_ref() {
            Some(type_name) => self.type_from_expr(type_name, expr.span),
            None => expected_sig
                .map(|sig| *sig.ret)
                .unwrap_or_else(|| Type::Unknown(self.fresh_unknown())),
        };
        FunctionSig {
            type_params: Vec::new(),
            params,
            ret: Box::new(ret),
        }
    }

    fn predeclare_functions(&mut self, statements: &[Stmt]) -> HashMap<Symbol, FunctionSig> {
        let mut functions = HashMap::new();
        for statement in statements {
            if let Stmt::FuncDecl(func) = statement {
                let name = self.symbol(&func.name);
                let type_params =
                    self.type_param_symbols(&func.type_params, "function", &func.name, func.span);
                let params = func
                    .params
                    .iter()
                    .map(|param| match param.type_name.as_ref() {
                        Some(type_name) => {
                            self.type_from_expr_with_params(type_name, param.span, &type_params)
                        }
                        None => Type::Unknown(self.fresh_unknown()),
                    })
                    .collect::<Vec<_>>();
                let ret = match func.return_type.as_ref() {
                    Some(type_name) => {
                        self.type_from_expr_with_params(type_name, func.span, &type_params)
                    }
                    None => Type::Unknown(self.fresh_unknown()),
                };
                let sig = FunctionSig {
                    type_params,
                    params,
                    ret: Box::new(ret),
                };
                functions.insert(name, sig.clone());
                let binding = self.insert_current(
                    name,
                    BindingKind::Function,
                    Type::Function(sig),
                    func.span,
                );
                if let Some(item) = func.package_item {
                    self.package_items_by_binding.insert(binding, item);
                    self.package_function_bindings_by_item.insert(item, binding);
                } else if let Some(item) = self.package_function_items.get(&name).copied() {
                    self.package_items_by_binding.insert(binding, item);
                    self.package_function_bindings_by_item.insert(item, binding);
                }
            }
        }
        functions
    }

    fn check_recursive_requirements(
        &mut self,
        statements: &[Stmt],
        functions: &HashMap<Symbol, FunctionSig>,
    ) {
        let names: HashSet<Symbol> = functions.keys().copied().collect();
        let decls: Vec<&FuncDecl> = statements
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::FuncDecl(func) => Some(func),
                _ => None,
            })
            .collect();
        let graph = build_call_graph(&decls, &names, &mut self.symbols);
        let components = strongly_connected_components(&graph);

        for component in components {
            if component.len() > 1 {
                for name in component {
                    if let Some(func) = decls
                        .iter()
                        .find(|func| self.symbols.lookup(&func.name) == Some(name))
                    {
                        let has_full_signature =
                            func.params.iter().all(|param| param.type_name.is_some())
                                && func.return_type.is_some();
                        if !has_full_signature {
                            self.diagnostics.push(
                                Diagnostic::new(
                                    "E007",
                                    "mutually recursive functions require explicit signatures in v1",
                                    func.span,
                                )
                                .with_suggestion(
                                    "add parameter type annotations and an explicit return type to each function in the mutually recursive group",
                                ),
                            );
                        }
                    }
                }
                continue;
            }

            let name = &component[0];
            let has_self_edge = graph
                .get(name)
                .is_some_and(|targets| targets.contains(name));
            if !has_self_edge {
                continue;
            }
            if let Some(func) = decls
                .iter()
                .find(|func| self.symbols.lookup(&func.name) == Some(*name))
            {
                let has_annotation = func.return_type.is_some()
                    || func.params.iter().any(|param| param.type_name.is_some());
                if !has_annotation {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "E006",
                            "recursive function requires at least one parameter or return type annotation",
                            func.span,
                        )
                        .with_suggestion(
                            "add a parameter type annotation or an explicit return type to the recursive function",
                        ),
                    );
                }
            }
        }
    }

    fn type_from_expr(&mut self, type_expr: &TypeExpr, span: crate::span::Span) -> Type {
        let type_params = self.current_type_params.last().cloned().unwrap_or_default();
        self.type_from_expr_with_params(type_expr, span, &type_params)
    }

    fn type_from_expr_with_params(
        &mut self,
        type_expr: &TypeExpr,
        span: crate::span::Span,
        type_params: &[Symbol],
    ) -> Type {
        match type_expr {
            TypeExpr::Int => Type::Int,
            TypeExpr::Bool => Type::Bool,
            TypeExpr::String => Type::String,
            TypeExpr::Unit => Type::Unit,
            TypeExpr::Named(name) => {
                let symbol = self.symbol(name);
                if type_params.contains(&symbol) {
                    Type::GenericParam(symbol)
                } else if let Some(record) = self.records.get(&symbol) {
                    if !record.type_params.is_empty() {
                        self.diagnostics.push(Diagnostic::new(
                            "T022",
                            format!(
                                "record `{name}` expects exactly {} type arguments but found 0",
                                record.type_params.len(),
                            ),
                            span,
                        ));
                        Type::Error
                    } else {
                        Type::Record(symbol, Vec::new())
                    }
                } else if self
                    .enums
                    .get(&symbol)
                    .is_some_and(|enumeration| enumeration.type_params.is_empty())
                {
                    Type::Enum(symbol, Vec::new())
                } else if self.package_opaque_items.contains_key(&symbol) {
                    Type::Opaque(symbol)
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "T007",
                        format!("unknown type `{name}`"),
                        span,
                    ));
                    Type::Error
                }
            }
            TypeExpr::Generic(generic) if generic.name == "List" => {
                if generic.args.len() != 1 {
                    self.diagnostics.push(Diagnostic::new(
                        "T016",
                        format!(
                            "List expects exactly 1 type argument but found {}",
                            generic.args.len()
                        ),
                        span,
                    ));
                    return Type::Error;
                }
                Type::List(Box::new(self.type_from_expr_with_params(
                    &generic.args[0],
                    span,
                    type_params,
                )))
            }
            TypeExpr::Generic(generic) if generic.name == known_enum::OPTION_NAME => {
                if generic.args.len() != 1 {
                    self.diagnostics.push(Diagnostic::new(
                        "T017",
                        format!(
                            "Option expects exactly 1 type argument but found {}",
                            generic.args.len()
                        ),
                        span,
                    ));
                    return Type::Error;
                }
                Type::Option(Box::new(self.type_from_expr_with_params(
                    &generic.args[0],
                    span,
                    type_params,
                )))
            }
            TypeExpr::Generic(generic) if generic.name == known_enum::RESULT_NAME => {
                if generic.args.len() != 2 {
                    self.diagnostics.push(Diagnostic::new(
                        "T021",
                        format!(
                            "Result expects exactly 2 type arguments but found {}",
                            generic.args.len()
                        ),
                        span,
                    ));
                    return Type::Error;
                }
                Type::Result(
                    Box::new(self.type_from_expr_with_params(&generic.args[0], span, type_params)),
                    Box::new(self.type_from_expr_with_params(&generic.args[1], span, type_params)),
                )
            }
            TypeExpr::Generic(generic) if generic.name == "Map" => {
                if generic.args.len() != 2 {
                    self.diagnostics.push(Diagnostic::new(
                        "T019",
                        format!(
                            "Map expects exactly 2 type arguments but found {}",
                            generic.args.len()
                        ),
                        span,
                    ));
                    return Type::Error;
                }
                let key = self.type_from_expr_with_params(&generic.args[0], span, type_params);
                self.validate_map_key_type(&key, span);
                let value = self.type_from_expr_with_params(&generic.args[1], span, type_params);
                Type::Map(Box::new(key), Box::new(value))
            }
            TypeExpr::Generic(generic) => {
                let symbol = self.symbol(&generic.name);
                if let Some(record) = self.records.get(&symbol).cloned() {
                    if generic.args.len() != record.type_params.len() {
                        self.diagnostics.push(Diagnostic::new(
                            "T022",
                            format!(
                                "record `{}` expects exactly {} type arguments but found {}",
                                generic.name,
                                record.type_params.len(),
                                generic.args.len()
                            ),
                            span,
                        ));
                        return Type::Error;
                    }
                    return Type::Record(
                        symbol,
                        generic
                            .args
                            .iter()
                            .map(|arg| self.type_from_expr_with_params(arg, span, type_params))
                            .collect(),
                    );
                }
                if let Some(enumeration) = self.enums.get(&symbol).cloned() {
                    if generic.args.len() != enumeration.type_params.len() {
                        self.diagnostics.push(Diagnostic::new(
                            "T022",
                            format!(
                                "enum `{}` expects exactly {} type arguments but found {}",
                                generic.name,
                                enumeration.type_params.len(),
                                generic.args.len()
                            ),
                            span,
                        ));
                        return Type::Error;
                    }
                    return Type::Enum(
                        symbol,
                        generic
                            .args
                            .iter()
                            .map(|arg| self.type_from_expr_with_params(arg, span, type_params))
                            .collect(),
                    );
                }
                if self.package_opaque_items.contains_key(&symbol) {
                    for arg in &generic.args {
                        let _ = self.type_from_expr_with_params(arg, span, type_params);
                    }
                    self.diagnostics.push(Diagnostic::new(
                        "T022",
                        format!(
                            "opaque type `{}` expects exactly 0 type arguments but found {}",
                            generic.name,
                            generic.args.len()
                        ),
                        span,
                    ));
                    return Type::Error;
                }
                for arg in &generic.args {
                    let _ = self.type_from_expr_with_params(arg, span, type_params);
                }
                self.diagnostics.push(
                    Diagnostic::new(
                        "T013",
                        format!("unknown generic type `{}`", generic.name),
                        span,
                    )
                    .with_suggestion(
                        "define the generic type or import the package that exposes it",
                    ),
                );
                Type::Error
            }
            TypeExpr::Function(function) => Type::Function(FunctionSig {
                type_params: Vec::new(),
                params: function
                    .params
                    .iter()
                    .map(|param| self.type_from_expr_with_params(param, span, type_params))
                    .collect(),
                ret: Box::new(self.type_from_expr_with_params(&function.ret, span, type_params)),
            }),
        }
    }

    fn require_exact(
        &mut self,
        left: &Type,
        right: &Type,
        span: crate::span::Span,
        code: &'static str,
    ) {
        if let Err(message) = self.unify(left.clone(), right.clone()) {
            self.diagnostics.push(Diagnostic::new(code, message, span));
        }
    }

    fn unify(&mut self, left: Type, right: Type) -> Result<Type, String> {
        let left = self.resolve_type(&left);
        let right = self.resolve_type(&right);
        match (left, right) {
            (Type::Error, _) | (_, Type::Error) => Ok(Type::Error),
            (Type::Never, ty) | (ty, Type::Never) => Ok(ty),
            (Type::Unknown(left), Type::Unknown(right)) if left == right => Ok(Type::Unknown(left)),
            (Type::Unknown(id), ty) | (ty, Type::Unknown(id)) => {
                if self.type_contains_unknown(&ty, id) {
                    return Err("type inference would require an infinite type".to_string());
                }
                self.substitutions.insert(id, ty.clone());
                Ok(ty)
            }
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Bool, Type::Bool) => Ok(Type::Bool),
            (Type::String, Type::String) => Ok(Type::String),
            (Type::Unit, Type::Unit) => Ok(Type::Unit),
            (Type::Record(left_name, left_args), Type::Record(right_name, right_args))
                if left_name == right_name && left_args.len() == right_args.len() =>
            {
                let args = left_args
                    .into_iter()
                    .zip(right_args)
                    .map(|(left, right)| self.unify(left, right))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Type::Record(left_name, args))
            }
            (Type::Enum(left_name, left_args), Type::Enum(right_name, right_args))
                if left_name == right_name && left_args.len() == right_args.len() =>
            {
                let args = left_args
                    .into_iter()
                    .zip(right_args)
                    .map(|(left, right)| self.unify(left, right))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Type::Enum(left_name, args))
            }
            (Type::Opaque(left_name), Type::Opaque(right_name)) if left_name == right_name => {
                Ok(Type::Opaque(left_name))
            }
            (Type::GenericParam(left), Type::GenericParam(right)) if left == right => {
                Ok(Type::GenericParam(left))
            }
            (Type::List(left), Type::List(right)) => {
                let item = self.unify(*left, *right)?;
                Ok(Type::List(Box::new(item)))
            }
            (Type::Map(left_key, left_value), Type::Map(right_key, right_value)) => {
                let key = self.unify(*left_key, *right_key)?;
                let value = self.unify(*left_value, *right_value)?;
                Ok(Type::Map(Box::new(key), Box::new(value)))
            }
            (Type::Option(left), Type::Option(right)) => {
                let item = self.unify(*left, *right)?;
                Ok(Type::Option(Box::new(item)))
            }
            (Type::Result(left_ok, left_err), Type::Result(right_ok, right_err)) => {
                let ok = self.unify(*left_ok, *right_ok)?;
                let err = self.unify(*left_err, *right_err)?;
                Ok(Type::Result(Box::new(ok), Box::new(err)))
            }
            (Type::Function(left), Type::Function(right)) => {
                if left.params.len() != right.params.len() {
                    return Err("function arity mismatch".to_string());
                }
                let mut params = Vec::with_capacity(left.params.len());
                for (left_param, right_param) in left.params.iter().zip(right.params.iter()) {
                    params.push(self.unify(left_param.clone(), right_param.clone())?);
                }
                let ret = self.unify(*left.ret.clone(), *right.ret.clone())?;
                Ok(Type::Function(FunctionSig {
                    type_params: Vec::new(),
                    params,
                    ret: Box::new(ret),
                }))
            }
            (left, right) => Err(format!(
                "type mismatch: expected {}, found {}",
                self.type_label(&left),
                self.type_label(&right)
            )),
        }
    }

    fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Unknown(id) => {
                if let Some(next) = self.substitutions.get(id) {
                    self.resolve_type(next)
                } else {
                    Type::Unknown(*id)
                }
            }
            Type::Function(sig) => Type::Function(FunctionSig {
                type_params: sig.type_params.clone(),
                params: sig.params.iter().map(|ty| self.resolve_type(ty)).collect(),
                ret: Box::new(self.resolve_type(&sig.ret)),
            }),
            Type::Enum(name, args) => Type::Enum(
                *name,
                args.iter().map(|arg| self.resolve_type(arg)).collect(),
            ),
            Type::Record(name, args) => Type::Record(
                *name,
                args.iter().map(|arg| self.resolve_type(arg)).collect(),
            ),
            Type::Opaque(name) => Type::Opaque(*name),
            Type::List(item) => Type::List(Box::new(self.resolve_type(item))),
            Type::Map(key, value) => Type::Map(
                Box::new(self.resolve_type(key)),
                Box::new(self.resolve_type(value)),
            ),
            Type::Option(item) => Type::Option(Box::new(self.resolve_type(item))),
            Type::Result(ok, err) => Type::Result(
                Box::new(self.resolve_type(ok)),
                Box::new(self.resolve_type(err)),
            ),
            Type::Builtin(builtin) => Type::Builtin(*builtin),
            Type::EnumConstructor {
                enum_name,
                enum_item,
                variant_name,
            } => Type::EnumConstructor {
                enum_name: *enum_name,
                enum_item: *enum_item,
                variant_name: *variant_name,
            },
            other => other.clone(),
        }
    }

    fn type_label(&self, ty: &Type) -> String {
        match self.resolve_type(ty) {
            Type::Int => "Int".to_string(),
            Type::Bool => "Bool".to_string(),
            Type::String => "String".to_string(),
            Type::Unit => "Unit".to_string(),
            Type::Record(symbol, args) => self.named_type_label(symbol, &args),
            Type::Enum(symbol, args) => self.named_type_label(symbol, &args),
            Type::Opaque(symbol) => self.symbols.resolve(symbol).to_string(),
            Type::GenericParam(symbol) => self.symbols.resolve(symbol).to_string(),
            Type::List(item) => format!("List[{}]", self.type_label(&item)),
            Type::Map(key, value) => {
                format!(
                    "Map[{}, {}]",
                    self.type_label(&key),
                    self.type_label(&value)
                )
            }
            Type::Option(item) => format!("Option[{}]", self.type_label(&item)),
            Type::Result(ok, err) => {
                format!(
                    "Result[{}, {}]",
                    self.type_label(&ok),
                    self.type_label(&err)
                )
            }
            Type::OptionNone => "Option::None".to_string(),
            Type::EnumConstructor {
                enum_name,
                variant_name,
                ..
            } => format!(
                "{}::{}",
                self.symbols.resolve(enum_name),
                self.symbols.resolve(variant_name)
            ),
            Type::Function(function) => {
                let params = function
                    .params
                    .iter()
                    .map(|param| self.type_label(param))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({params}) -> {}", self.type_label(&function.ret))
            }
            Type::Builtin(builtin) => prelude::builtin_debug_label(builtin).to_string(),
            Type::Never => "Never".to_string(),
            Type::Unknown(_) => "Unknown".to_string(),
            Type::Error => "Error".to_string(),
        }
    }

    fn named_type_label(&self, symbol: Symbol, args: &[Type]) -> String {
        let name = self.symbols.resolve(symbol);
        if args.is_empty() {
            name.to_string()
        } else {
            let args = args
                .iter()
                .map(|arg| self.type_label(arg))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{name}[{args}]")
        }
    }

    fn type_info_for(&self, ty: &Type) -> TypeInfo {
        match self.resolve_type(ty) {
            Type::Int => TypeInfo::Int,
            Type::Bool => TypeInfo::Bool,
            Type::String => TypeInfo::String,
            Type::Unit => TypeInfo::Unit,
            Type::Record(symbol, args) => {
                let args = args.iter().map(|arg| self.type_info_for(arg)).collect();
                if let Some(item) = self.package_record_items.get(&symbol).copied() {
                    TypeInfo::PackageRecord { symbol, item, args }
                } else {
                    TypeInfo::Record(symbol, args)
                }
            }
            Type::Enum(symbol, args) => {
                let args = args.iter().map(|arg| self.type_info_for(arg)).collect();
                if let Some(item) = self.package_enum_items.get(&symbol).copied() {
                    TypeInfo::PackageEnum { symbol, item, args }
                } else {
                    TypeInfo::Enum { symbol, args }
                }
            }
            Type::Opaque(symbol) => {
                if let Some(item) = self.package_opaque_items.get(&symbol).copied() {
                    TypeInfo::PackageOpaque { symbol, item }
                } else {
                    TypeInfo::Error
                }
            }
            Type::GenericParam(symbol) => TypeInfo::GenericParam(symbol),
            Type::List(item) => TypeInfo::List(Box::new(self.type_info_for(&item))),
            Type::Map(key, value) => TypeInfo::Map(
                Box::new(self.type_info_for(&key)),
                Box::new(self.type_info_for(&value)),
            ),
            Type::Option(item) => TypeInfo::Option(Box::new(self.type_info_for(&item))),
            Type::Result(ok, err) => TypeInfo::Result(
                Box::new(self.type_info_for(&ok)),
                Box::new(self.type_info_for(&err)),
            ),
            Type::Function(sig) => TypeInfo::Function(FunctionTypeInfo {
                params: sig.params.iter().map(|ty| self.type_info_for(ty)).collect(),
                ret: Box::new(self.type_info_for(&sig.ret)),
            }),
            Type::Builtin(builtin) => TypeInfo::Builtin(builtin),
            Type::Never => TypeInfo::Unknown,
            Type::OptionNone => TypeInfo::Builtin(BuiltinId::OptionNone),
            Type::EnumConstructor {
                enum_name,
                enum_item,
                variant_name,
            } => TypeInfo::EnumConstructor {
                enum_symbol: enum_name,
                enum_item,
                variant: variant_name,
            },
            Type::Unknown(_) => TypeInfo::Unknown,
            Type::Error => TypeInfo::Error,
        }
    }

    fn type_contains_unknown(&self, ty: &Type, needle: u32) -> bool {
        match self.resolve_type(ty) {
            Type::Unknown(id) => id == needle,
            Type::Function(sig) => {
                sig.params
                    .iter()
                    .any(|param| self.type_contains_unknown(param, needle))
                    || self.type_contains_unknown(&sig.ret, needle)
            }
            Type::Enum(_, args) => args
                .iter()
                .any(|arg| self.type_contains_unknown(arg, needle)),
            Type::Record(_, args) => args
                .iter()
                .any(|arg| self.type_contains_unknown(arg, needle)),
            Type::Opaque(_) => false,
            Type::List(item) => self.type_contains_unknown(&item, needle),
            Type::Map(key, value) => {
                self.type_contains_unknown(&key, needle)
                    || self.type_contains_unknown(&value, needle)
            }
            Type::Option(item) => self.type_contains_unknown(&item, needle),
            Type::Result(ok, err) => {
                self.type_contains_unknown(&ok, needle) || self.type_contains_unknown(&err, needle)
            }
            _ => false,
        }
    }

    fn substitute_type_params(&self, ty: Type, params: &[Symbol], args: &[Type]) -> Type {
        match ty {
            Type::GenericParam(param) => params
                .iter()
                .position(|candidate| *candidate == param)
                .and_then(|index| args.get(index).cloned())
                .unwrap_or(Type::GenericParam(param)),
            Type::Function(sig) => Type::Function(FunctionSig {
                type_params: sig.type_params,
                params: sig
                    .params
                    .into_iter()
                    .map(|param| self.substitute_type_params(param, params, args))
                    .collect(),
                ret: Box::new(self.substitute_type_params(*sig.ret, params, args)),
            }),
            Type::Enum(name, enum_args) => Type::Enum(
                name,
                enum_args
                    .into_iter()
                    .map(|arg| self.substitute_type_params(arg, params, args))
                    .collect(),
            ),
            Type::Record(name, record_args) => Type::Record(
                name,
                record_args
                    .into_iter()
                    .map(|arg| self.substitute_type_params(arg, params, args))
                    .collect(),
            ),
            Type::List(item) => {
                Type::List(Box::new(self.substitute_type_params(*item, params, args)))
            }
            Type::Map(key, value) => Type::Map(
                Box::new(self.substitute_type_params(*key, params, args)),
                Box::new(self.substitute_type_params(*value, params, args)),
            ),
            Type::Option(item) => {
                Type::Option(Box::new(self.substitute_type_params(*item, params, args)))
            }
            Type::Result(ok, err) => Type::Result(
                Box::new(self.substitute_type_params(*ok, params, args)),
                Box::new(self.substitute_type_params(*err, params, args)),
            ),
            other => other,
        }
    }

    fn instantiate_function_sig(&mut self, sig: FunctionSig) -> FunctionSig {
        if sig.type_params.is_empty() {
            return sig;
        }
        let args = sig
            .type_params
            .iter()
            .map(|_| Type::Unknown(self.fresh_unknown()))
            .collect::<Vec<_>>();
        FunctionSig {
            type_params: Vec::new(),
            params: sig
                .params
                .into_iter()
                .map(|param| self.substitute_type_params(param, &sig.type_params, &args))
                .collect(),
            ret: Box::new(self.substitute_type_params(*sig.ret, &sig.type_params, &args)),
        }
    }

    fn typed_callee_for(&self, callee: &Expr, resolved_ty: &Type) -> TypedCalleeInfo {
        match resolved_ty {
            Type::Builtin(builtin) => self
                .binding_for_expr(callee.id())
                .map(|binding| TypedCalleeInfo::Builtin {
                    binding,
                    name: Self::builtin_name(*builtin),
                })
                .unwrap_or(TypedCalleeInfo::Error),
            Type::EnumConstructor {
                enum_name,
                enum_item,
                variant_name,
            } => self
                .binding_for_expr(callee.id())
                .map(|binding| TypedCalleeInfo::EnumVariant {
                    binding,
                    enum_name: *enum_name,
                    enum_item: *enum_item,
                    variant_name: *variant_name,
                })
                .unwrap_or(TypedCalleeInfo::Error),
            Type::Function(_) | Type::Unknown(_) => self
                .binding_for_expr(callee.id())
                .map(|binding| {
                    self.package_items_by_binding
                        .get(&binding)
                        .copied()
                        .map(|item| TypedCalleeInfo::PackageItem { binding, item })
                        .unwrap_or(TypedCalleeInfo::Binding(binding))
                })
                .unwrap_or(TypedCalleeInfo::Value),
            Type::Error => TypedCalleeInfo::Error,
            _ => TypedCalleeInfo::Error,
        }
    }

    fn binding_for_expr(&self, expr_id: ExprId) -> Option<BindingId> {
        self.identifier_refs
            .iter()
            .rev()
            .find(|identifier| identifier.expr_id == expr_id)
            .map(|identifier| identifier.binding)
    }

    fn builtin_name(builtin: BuiltinId) -> &'static str {
        prelude::builtin_name(builtin)
    }

    fn apply_expected(
        &mut self,
        inferred: Type,
        expected: Option<Type>,
        span: crate::span::Span,
    ) -> Type {
        let inferred = self.resolve_type(&inferred);
        let Some(expected) = expected else {
            return inferred;
        };
        match self.unify(expected, inferred) {
            Ok(ty) => self.resolve_type(&ty),
            Err(message) => {
                self.diagnostics
                    .push(Diagnostic::new("T002", message, span));
                Type::Error
            }
        }
    }

    fn expected_function_sig(&self, expected: Option<&Type>, arity: usize) -> Option<FunctionSig> {
        let expected = expected?;
        match self.resolve_type(expected) {
            Type::Function(sig) if sig.params.len() == arity => Some(sig),
            _ => None,
        }
    }

    fn param_modes_for_callee(&self, callee: &Expr) -> Option<Vec<PackageInterfaceParamMode>> {
        let Expr::Ident(ident) = callee else {
            return None;
        };
        let name = self.symbols.lookup(&ident.name)?;
        let binding = self.lookup(name)?;
        self.package_function_param_modes.get(&binding.id).cloned()
    }

    fn check_consumed_binding_use(&mut self, binding: &Binding, span: Span) {
        let Some(consumed_at) = self.consumed_binding_span(binding.id) else {
            return;
        };
        let name = self.symbols.resolve(binding.symbol).to_string();
        self.diagnostics.push(
            Diagnostic::new(
                "T026",
                format!("binding `{name}` was consumed and cannot be used again"),
                span,
            )
            .with_related("binding was consumed here", consumed_at)
            .with_suggestion("do not use a binding after passing it to a consuming parameter"),
        );
    }

    fn mark_consumed_argument(&mut self, arg: &Expr, consume_span: Span) {
        let Some(binding) = self.binding_for_direct_identifier(arg).cloned() else {
            return;
        };
        if !matches!(
            binding.kind,
            BindingKind::Immutable | BindingKind::Mutable | BindingKind::Parameter
        ) {
            return;
        }
        if let Some(using_span) = self.using_bindings.get(&binding.id).copied() {
            let name = self.symbols.resolve(binding.symbol).to_string();
            self.diagnostics.push(
                Diagnostic::new(
                    "T027",
                    format!(
                        "binding `{name}` is managed by `using` and cannot be consumed explicitly"
                    ),
                    consume_span,
                )
                .with_related("`using` binding is declared here", using_span)
                .with_suggestion(
                    "let `using` close the handle automatically at the end of the block",
                ),
            );
            return;
        }
        if let Some(scope) = self.scopes.last_mut() {
            scope
                .consumed_bindings
                .entry(binding.id)
                .or_insert(consume_span);
        }
    }

    fn clear_consumed_binding(&mut self, binding: BindingId) {
        for scope in self.scopes.iter_mut().rev() {
            scope.consumed_bindings.remove(&binding);
            if scope.function_boundary {
                break;
            }
        }
    }

    fn consumed_binding_span(&self, binding: BindingId) -> Option<Span> {
        for scope in self.scopes.iter().rev() {
            if let Some(span) = scope.consumed_bindings.get(&binding) {
                return Some(*span);
            }
            if scope.function_boundary {
                break;
            }
        }
        None
    }

    fn binding_for_direct_identifier(&self, expr: &Expr) -> Option<&Binding> {
        let Expr::Ident(ident) = expr else {
            return None;
        };
        let name = self.symbols.lookup(&ident.name)?;
        self.lookup(name)
    }

    fn binding_by_id(&self, binding: BindingId) -> Option<&Binding> {
        self.bindings
            .iter()
            .find(|candidate| candidate.id == binding)
    }

    fn fresh_unknown(&mut self) -> u32 {
        let id = self.next_unknown;
        self.next_unknown += 1;
        id
    }

    fn push_scope(&mut self, function_boundary: bool) {
        self.scopes.push(ScopeFrame::new(function_boundary));
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert_current(
        &mut self,
        name: Symbol,
        kind: BindingKind,
        ty: Type,
        span: Span,
    ) -> BindingId {
        let id = BindingId::new(self.bindings.len() as u32);
        self.bindings.push(Binding {
            id,
            symbol: name,
            kind,
            ty,
            span,
        });
        if let Some(scope) = self.scopes.last_mut() {
            scope.bindings.insert(name, id);
        }
        id
    }

    fn lookup(&self, name: Symbol) -> Option<&Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.bindings.get(&name).map(|id| self.binding(*id)))
    }

    fn lookup_in_current_function(&self, name: Symbol) -> Option<&Binding> {
        for scope in self.scopes.iter().rev() {
            if let Some(id) = scope.bindings.get(&name) {
                return Some(self.binding(*id));
            }
            if scope.function_boundary {
                break;
            }
        }
        None
    }

    fn lookup_beyond_current_function(&self, name: Symbol) -> Option<&Binding> {
        let boundary_index = self
            .scopes
            .iter()
            .rposition(|scope| scope.function_boundary)
            .unwrap_or(0);
        self.scopes[..boundary_index]
            .iter()
            .rev()
            .find_map(|scope| scope.bindings.get(&name).map(|id| self.binding(*id)))
    }

    fn binding(&self, id: BindingId) -> &Binding {
        &self.bindings[id.as_u32() as usize]
    }

    fn symbol(&mut self, name: &str) -> Symbol {
        self.symbols.intern(name)
    }
}

impl Type {
    fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
}

fn split_variant_name(name: &str) -> Option<(&str, &str)> {
    let (enum_name, variant_name) = name.rsplit_once("::")?;
    if enum_name.is_empty() || variant_name.is_empty() {
        None
    } else {
        Some((enum_name, variant_name))
    }
}

fn find_record_field(record: &RecordDef, name: Symbol) -> Option<&RecordField> {
    record.fields.iter().find(|field| field.name == name)
}

fn json_rename_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<&str> {
    json_rename_argument_from_attributes(attributes).map(|(rename, _)| rename)
}

fn json_rename_span_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<Span> {
    json_rename_argument_from_attributes(attributes).map(|(_, span)| span)
}

fn json_rename_argument_from_attributes(
    attributes: &[crate::ast::Attribute],
) -> Option<(&str, Span)> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "json" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "rename")
                .and_then(|argument| argument.string_value().map(|value| (value, argument.span)))
        } else {
            None
        }
    })
}

fn json_aliases_from_attributes(attributes: &[crate::ast::Attribute]) -> Vec<(&str, Span)> {
    attributes
        .iter()
        .filter(|attribute| attribute.name == "json")
        .flat_map(|attribute| {
            attribute
                .arguments
                .iter()
                .filter(|argument| argument.name == "alias")
                .filter_map(|argument| argument.string_value().map(|value| (value, argument.span)))
        })
        .collect()
}

fn cli_metadata_present_from_attributes(attributes: &[crate::ast::Attribute]) -> bool {
    attributes.iter().any(|attribute| attribute.name == "cli")
}

fn cli_name_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<&str> {
    cli_name_argument_from_attributes(attributes).map(|(name, _)| name)
}

fn cli_name_span_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<Span> {
    cli_name_argument_from_attributes(attributes).map(|(_, span)| span)
}

fn cli_name_argument_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<(&str, Span)> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "cli" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "name")
                .and_then(|argument| argument.string_value().map(|value| (value, argument.span)))
        } else {
            None
        }
    })
}

fn cli_short_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<&str> {
    cli_short_argument_from_attributes(attributes).map(|(short, _)| short)
}

fn cli_short_argument_from_attributes(
    attributes: &[crate::ast::Attribute],
) -> Option<(&str, Span)> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "cli" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "short")
                .and_then(|argument| argument.string_value().map(|value| (value, argument.span)))
        } else {
            None
        }
    })
}

fn cli_position_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<u32> {
    cli_position_argument_from_attributes(attributes).map(|(position, _)| position)
}

fn cli_position_argument_from_attributes(
    attributes: &[crate::ast::Attribute],
) -> Option<(u32, Span)> {
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
                        .map(|value| (value, argument.span))
                })
        } else {
            None
        }
    })
}

fn cli_value_source_from_attributes(
    attributes: &[crate::ast::Attribute],
) -> Option<CliValueSource> {
    cli_value_source_argument_from_attributes(attributes).map(|(value_source, _)| value_source)
}

fn cli_value_source_argument_from_attributes(
    attributes: &[crate::ast::Attribute],
) -> Option<(CliValueSource, Span)> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "cli" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "value_source")
                .and_then(|argument| {
                    argument
                        .string_value()
                        .and_then(|value| CliValueSource::from_artifact_token(value).ok())
                        .map(|value_source| (value_source, argument.span))
                })
        } else {
            None
        }
    })
}

fn cli_aliases_from_attributes(attributes: &[crate::ast::Attribute]) -> Vec<(&str, Span)> {
    attributes
        .iter()
        .filter(|attribute| attribute.name == "cli")
        .flat_map(|attribute| {
            attribute
                .arguments
                .iter()
                .filter(|argument| argument.name == "alias")
                .filter_map(|argument| argument.string_value().map(|value| (value, argument.span)))
        })
        .collect()
}

fn cli_help_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<&str> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "cli" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "help")
                .and_then(|argument| argument.string_value())
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
    cli_subcommand_argument_from_attributes(attributes).is_some()
}

fn cli_subcommand_argument_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<Span> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "cli" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "subcommand" && argument.value.is_none())
                .map(|argument| argument.span)
        } else {
            None
        }
    })
}

fn cli_about_from_attributes(attributes: &[crate::ast::Attribute]) -> Option<&str> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == "cli" {
            attribute
                .arguments
                .iter()
                .find(|argument| argument.name == "about")
                .and_then(|argument| argument.string_value())
        } else {
            None
        }
    })
}

fn json_validation_from_attributes(
    attributes: &[crate::ast::Attribute],
) -> Vec<JsonDecodeValidationRule> {
    let mut rules = Vec::new();
    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.name == "validate")
    {
        for argument in &attribute.arguments {
            let Some(rule) = json_validation_rule_from_argument(argument) else {
                continue;
            };
            if !rules.contains(&rule) {
                rules.push(rule);
            }
        }
    }
    rules
}

fn json_validation_rules_with_spans(
    attributes: &[crate::ast::Attribute],
) -> Vec<(JsonDecodeValidationRule, Span)> {
    attributes
        .iter()
        .filter(|attribute| attribute.name == "validate")
        .flat_map(|attribute| {
            attribute.arguments.iter().filter_map(|argument| {
                json_validation_rule_from_argument(argument).map(|rule| (rule, argument.span))
            })
        })
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

fn json_validation_rule_key(rule: &JsonDecodeValidationRule) -> &'static str {
    match rule {
        JsonDecodeValidationRule::NonEmpty => "non_empty",
        JsonDecodeValidationRule::Min(_) => "min",
        JsonDecodeValidationRule::Max(_) => "max",
        JsonDecodeValidationRule::MinLen(_) => "min_len",
        JsonDecodeValidationRule::MaxLen(_) => "max_len",
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

fn build_call_graph(
    decls: &[&FuncDecl],
    local_names: &HashSet<Symbol>,
    symbols: &mut SymbolTable,
) -> HashMap<Symbol, HashSet<Symbol>> {
    let mut graph = HashMap::new();
    for decl in decls {
        let mut calls = HashSet::new();
        collect_calls_in_statements(&decl.body.statements, local_names, &mut calls, symbols);
        collect_calls_in_expr(&decl.body.expr, local_names, &mut calls, symbols);
        graph.insert(symbols.intern(&decl.name), calls);
    }
    graph
}

fn strongly_connected_components(graph: &HashMap<Symbol, HashSet<Symbol>>) -> Vec<Vec<Symbol>> {
    let mut state = SccState::new(graph);

    for node in graph.keys() {
        if !state.indices.contains_key(node) {
            state.strong_connect(*node);
        }
    }

    state.components
}

struct SccState<'a> {
    graph: &'a HashMap<Symbol, HashSet<Symbol>>,
    index: usize,
    stack: Vec<Symbol>,
    indices: HashMap<Symbol, usize>,
    lowlinks: HashMap<Symbol, usize>,
    on_stack: HashSet<Symbol>,
    components: Vec<Vec<Symbol>>,
}

impl<'a> SccState<'a> {
    fn new(graph: &'a HashMap<Symbol, HashSet<Symbol>>) -> Self {
        Self {
            graph,
            index: 0,
            stack: Vec::new(),
            indices: HashMap::new(),
            lowlinks: HashMap::new(),
            on_stack: HashSet::new(),
            components: Vec::new(),
        }
    }

    fn strong_connect(&mut self, node: Symbol) {
        self.indices.insert(node, self.index);
        self.lowlinks.insert(node, self.index);
        self.index += 1;
        self.stack.push(node);
        self.on_stack.insert(node);

        if let Some(neighbors) = self.graph.get(&node) {
            for neighbor in neighbors {
                if !self.indices.contains_key(neighbor) {
                    self.strong_connect(*neighbor);
                    let neighbor_low = self.lowlinks[neighbor];
                    let node_low = self.lowlinks[&node];
                    self.lowlinks.insert(node, node_low.min(neighbor_low));
                } else if self.on_stack.contains(neighbor) {
                    let neighbor_index = self.indices[neighbor];
                    let node_low = self.lowlinks[&node];
                    self.lowlinks.insert(node, node_low.min(neighbor_index));
                }
            }
        }

        if self.lowlinks[&node] == self.indices[&node] {
            let mut component = Vec::new();
            while let Some(candidate) = self.stack.pop() {
                self.on_stack.remove(&candidate);
                component.push(candidate);
                if candidate == node {
                    break;
                }
            }
            self.components.push(component);
        }
    }
}

fn collect_calls_in_statements(
    statements: &[Stmt],
    local_names: &HashSet<Symbol>,
    calls: &mut HashSet<Symbol>,
    symbols: &mut SymbolTable,
) {
    for statement in statements {
        match statement {
            Stmt::Assign(stmt) => collect_calls_in_expr(&stmt.value, local_names, calls, symbols),
            Stmt::RecordDecl(_) => {}
            Stmt::EnumDecl(_) => {}
            Stmt::OpaqueTypeDecl(_) => {}
            Stmt::FuncDecl(_) => {}
            Stmt::If(stmt) => {
                collect_calls_in_expr(&stmt.condition, local_names, calls, symbols);
                collect_calls_in_statements(
                    &stmt.then_branch.statements,
                    local_names,
                    calls,
                    symbols,
                );
                if let Some(else_branch) = &stmt.else_branch {
                    collect_calls_in_statements(
                        &else_branch.statements,
                        local_names,
                        calls,
                        symbols,
                    );
                }
            }
            Stmt::While(stmt) => {
                collect_calls_in_expr(&stmt.condition, local_names, calls, symbols);
                collect_calls_in_statements(&stmt.body.statements, local_names, calls, symbols);
            }
            Stmt::For(stmt) => {
                collect_calls_in_expr(&stmt.iterable, local_names, calls, symbols);
                collect_calls_in_statements(&stmt.body.statements, local_names, calls, symbols);
            }
            Stmt::Using(stmt) => {
                collect_calls_in_expr(&stmt.value, local_names, calls, symbols);
                collect_calls_in_statements(&stmt.body.statements, local_names, calls, symbols);
            }
            Stmt::Break(_) | Stmt::Continue(_) => {}
            Stmt::Return(stmt) => collect_calls_in_expr(&stmt.value, local_names, calls, symbols),
            Stmt::Expr(stmt) => collect_calls_in_expr(&stmt.expr, local_names, calls, symbols),
        }
    }
}

fn collect_calls_in_expr(
    expr: &Expr,
    local_names: &HashSet<Symbol>,
    calls: &mut HashSet<Symbol>,
    symbols: &mut SymbolTable,
) {
    match expr {
        Expr::Int(_) | Expr::Bool(_) | Expr::String(_) | Expr::Unit(_) | Expr::Ident(_) => {}
        Expr::ListLit(expr) => {
            for item in &expr.items {
                collect_calls_in_expr(item, local_names, calls, symbols);
            }
        }
        Expr::Index(expr) => {
            collect_calls_in_expr(&expr.base, local_names, calls, symbols);
            collect_calls_in_expr(&expr.index, local_names, calls, symbols);
        }
        Expr::RecordLit(expr) => {
            for field in &expr.fields {
                collect_calls_in_expr(&field.value, local_names, calls, symbols);
            }
        }
        Expr::Field(expr) => collect_calls_in_expr(&expr.base, local_names, calls, symbols),
        Expr::RecordUpdate(expr) => {
            collect_calls_in_expr(&expr.base, local_names, calls, symbols);
            for field in &expr.fields {
                collect_calls_in_expr(&field.value, local_names, calls, symbols);
            }
        }
        Expr::Unary(expr) => collect_calls_in_expr(&expr.expr, local_names, calls, symbols),
        Expr::Binary(expr) => {
            collect_calls_in_expr(&expr.left, local_names, calls, symbols);
            collect_calls_in_expr(&expr.right, local_names, calls, symbols);
        }
        Expr::Call(expr) => {
            if let Expr::Ident(ident) = expr.callee.as_ref() {
                let name = symbols.intern(&ident.name);
                if local_names.contains(&name) {
                    calls.insert(name);
                }
            }
            collect_calls_in_expr(&expr.callee, local_names, calls, symbols);
            for arg in &expr.args {
                collect_calls_in_expr(arg, local_names, calls, symbols);
            }
        }
        Expr::Try(expr) => collect_calls_in_expr(&expr.expr, local_names, calls, symbols),
        Expr::If(expr) => {
            collect_calls_in_expr(&expr.condition, local_names, calls, symbols);
            collect_calls_in_statements(&expr.then_branch.statements, local_names, calls, symbols);
            collect_calls_in_expr(&expr.then_branch.expr, local_names, calls, symbols);
            collect_calls_in_statements(&expr.else_branch.statements, local_names, calls, symbols);
            collect_calls_in_expr(&expr.else_branch.expr, local_names, calls, symbols);
        }
        Expr::Match(expr) => {
            collect_calls_in_expr(&expr.value, local_names, calls, symbols);
            for arm in &expr.arms {
                collect_calls_in_expr(&arm.value, local_names, calls, symbols);
            }
        }
        Expr::Fn(_) => {}
    }
}
