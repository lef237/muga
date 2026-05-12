use std::collections::{HashMap, HashSet};

use crate::{
    ast::Visibility,
    diagnostic::Diagnostic,
    identity::{PackageId, PackageItemId},
    package::{PackageItemInfo, PackageItemKind, PackageSymbolGraph},
    span::Span,
    typed_hir::{
        Block, Expr, ExprKind, FunctionStmt, IdentTarget, Program, RecordStmt, Stmt, ValueBlock,
    },
    typing::TypeInfo,
};

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
                        PackageItemKind::Function => functions.push(export),
                    }
                }
                PackageExports {
                    package: package.id,
                    path: package.path.clone(),
                    records,
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
    pub fn package(&self, id: PackageId) -> Option<&PackageInterface> {
        self.packages
            .iter()
            .find(|interface| interface.package == id)
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackageInterface {
    pub package: PackageId,
    pub path: String,
    pub records: Vec<PackageInterfaceRecord>,
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
                    records,
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
