use std::collections::{HashMap, HashSet};

use crate::{
    ast,
    diagnostic::Diagnostic,
    identity::{BindingId, BindingKind, ExprId, PackageItemId, StmtId},
    package::{
        PackageInterface, PackageInterfaceField, PackageInterfaceFunction, PackageInterfaceGraph,
        PackageInterfaceParam, PackageInterfaceRecord, PackageItemInfo, PackageItemKind,
        PackageSymbolGraph,
    },
    span::Span,
    symbol::{Symbol, SymbolTable},
    typing::{
        FunctionTypeInfo, TypeCheckOutput, TypeInfo, TypedAssignmentTarget, TypedBindingInfo,
        TypedCalleeInfo,
    },
};

#[derive(Clone, Debug)]
pub struct Program {
    pub statements: Vec<Stmt>,
    pub bindings: Vec<TypedBindingInfo>,
    pub package_graph: PackageSymbolGraph,
    pub symbols: SymbolTable,
}

impl Program {
    pub fn package_interfaces(&self) -> PackageInterfaceGraph {
        let records_by_mangled_name: HashMap<&str, &RecordStmt> = self
            .statements
            .iter()
            .filter_map(|statement| match statement {
                Stmt::Record(record) => Some((record.name.as_str(), record)),
                _ => None,
            })
            .collect();
        let functions_by_mangled_name: HashMap<&str, &FunctionStmt> = self
            .statements
            .iter()
            .filter_map(|statement| match statement {
                Stmt::Function(function) => Some((function.name.as_str(), function)),
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
                    item.package == package.id && item.visibility == ast::Visibility::Public
                }) {
                    match item.kind {
                        PackageItemKind::Record => {
                            if let Some(record) =
                                records_by_mangled_name.get(item.mangled_name.as_str())
                            {
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
                            if let Some(function) =
                                functions_by_mangled_name.get(item.mangled_name.as_str())
                            {
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
            TypeInfo::Option(item) => self.validate_type(item, span),
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
        if info.visibility != ast::Visibility::Public {
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
                Stmt::Record(record) if record.name == info.mangled_name => Some(record),
                _ => None,
            })
    }

    fn function_stmt_for(&self, info: &PackageItemInfo) -> Option<&FunctionStmt> {
        self.program
            .statements
            .iter()
            .find_map(|statement| match statement {
                Stmt::Function(function) if function.name == info.mangled_name => Some(function),
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

#[derive(Clone, Debug)]
pub enum Stmt {
    Assign(AssignStmt),
    Record(RecordStmt),
    Function(FunctionStmt),
    If(IfStmt),
    While(WhileStmt),
    Expr(ExprStmt),
}

impl Stmt {
    pub fn id(&self) -> StmtId {
        match self {
            Self::Assign(stmt) => stmt.id,
            Self::Record(stmt) => stmt.id,
            Self::Function(stmt) => stmt.id,
            Self::If(stmt) => stmt.id,
            Self::While(stmt) => stmt.id,
            Self::Expr(stmt) => stmt.id,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AssignStmt {
    pub id: StmtId,
    pub mutable: bool,
    pub is_update: bool,
    pub name: String,
    pub binding: BindingId,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RecordStmt {
    pub id: StmtId,
    pub name: String,
    pub fields: Vec<RecordField>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RecordField {
    pub name: String,
    pub ty: TypeInfo,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FunctionStmt {
    pub id: StmtId,
    pub name: String,
    pub binding: BindingId,
    pub params: Vec<Param>,
    pub return_ty: TypeInfo,
    pub body: ValueBlock,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub binding: BindingId,
    pub ty: TypeInfo,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct IfStmt {
    pub id: StmtId,
    pub condition: Expr,
    pub then_branch: Block,
    pub else_branch: Option<Block>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct WhileStmt {
    pub id: StmtId,
    pub condition: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ExprStmt {
    pub id: StmtId,
    pub expr: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub statements: Vec<Stmt>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ValueBlock {
    pub statements: Vec<Stmt>,
    pub expr: Box<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Expr {
    pub id: ExprId,
    pub ty: TypeInfo,
    pub kind: ExprKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum ExprKind {
    Int(i64),
    Bool(bool),
    String(String),
    Ident(IdentExpr),
    ListLit(ListLitExpr),
    Index(IndexExpr),
    RecordLit(RecordLitExpr),
    Field(FieldExpr),
    RecordUpdate(RecordUpdateExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    If(IfExpr),
    Match(MatchExpr),
    Fn(FnExpr),
}

#[derive(Clone, Debug)]
pub struct IdentExpr {
    pub name: String,
    pub binding: BindingId,
    pub target: IdentTarget,
}

#[derive(Clone, Debug)]
pub struct ListLitExpr {
    pub items: Vec<Expr>,
}

#[derive(Clone, Debug)]
pub struct IndexExpr {
    pub base: Box<Expr>,
    pub index: Box<Expr>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdentTarget {
    Binding(BindingId),
    PackageItem {
        binding: BindingId,
        item: PackageItemId,
    },
}

#[derive(Clone, Debug)]
pub struct RecordLitExpr {
    pub type_name: String,
    pub fields: Vec<RecordFieldInit>,
}

#[derive(Clone, Debug)]
pub struct RecordFieldInit {
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FieldExpr {
    pub base: Box<Expr>,
    pub field: String,
}

#[derive(Clone, Debug)]
pub struct RecordUpdateExpr {
    pub base: Box<Expr>,
    pub fields: Vec<RecordFieldInit>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Clone, Debug)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub expr: Box<Expr>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Lt,
    LtEq,
    Gt,
    GtEq,
    EqEq,
    BangEq,
}

#[derive(Clone, Debug)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
}

#[derive(Clone, Debug)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub origin: CallOrigin,
    pub resolved_callee: TypedCalleeInfo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallOrigin {
    Ordinary,
    Chained,
    QualifiedChained,
}

#[derive(Clone, Debug)]
pub struct IfExpr {
    pub condition: Box<Expr>,
    pub then_branch: ValueBlock,
    pub else_branch: ValueBlock,
}

#[derive(Clone, Debug)]
pub struct MatchExpr {
    pub value: Box<Expr>,
    pub arms: Vec<MatchArm>,
}

#[derive(Clone, Debug)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum MatchPattern {
    OptionSome {
        binding_name: String,
        binding: BindingId,
        span: Span,
    },
    OptionNone {
        span: Span,
    },
}

#[derive(Clone, Debug)]
pub struct FnExpr {
    pub params: Vec<Param>,
    pub return_ty: TypeInfo,
    pub body: ValueBlock,
}

pub fn lower(
    program: &ast::Program,
    analysis: &TypeCheckOutput,
    package_graph: PackageSymbolGraph,
) -> Program {
    let lowerer = Lowerer::new(analysis, &package_graph);
    let bindings = lowerer.lower_bindings();
    let statements = program
        .statements
        .iter()
        .map(|statement| lowerer.lower_stmt(statement))
        .collect();
    Program {
        statements,
        bindings,
        package_graph,
        symbols: analysis.symbols.clone(),
    }
}

struct Lowerer<'a> {
    analysis: &'a TypeCheckOutput,
    expr_types: HashMap<ExprId, TypeInfo>,
    identifier_refs: HashMap<ExprId, BindingId>,
    calls: HashMap<ExprId, TypedCalleeInfo>,
    package_items_by_binding: HashMap<BindingId, PackageItemId>,
    package_items_by_symbol: HashMap<Symbol, PackageItemId>,
    assignment_targets: HashMap<StmtId, TypedAssignmentTarget>,
}

impl<'a> Lowerer<'a> {
    fn new(analysis: &'a TypeCheckOutput, package_graph: &PackageSymbolGraph) -> Self {
        let package_items_by_mangled_name: HashMap<&str, PackageItemId> = package_graph
            .items
            .iter()
            .map(|item| (item.mangled_name.as_str(), item.id))
            .collect();
        let package_items_by_binding = analysis
            .bindings
            .iter()
            .filter_map(|binding| {
                package_items_by_mangled_name
                    .get(analysis.symbols.resolve(binding.symbol))
                    .copied()
                    .map(|item| (binding.id, item))
            })
            .collect();
        let package_items_by_symbol = package_graph
            .items
            .iter()
            .filter_map(|item| {
                analysis
                    .symbols
                    .lookup(&item.mangled_name)
                    .map(|symbol| (symbol, item.id))
            })
            .collect();
        Self {
            analysis,
            expr_types: analysis
                .expr_types
                .iter()
                .map(|expr| (expr.expr_id, expr.ty.clone()))
                .collect(),
            identifier_refs: analysis
                .identifier_refs
                .iter()
                .map(|identifier| (identifier.expr_id, identifier.binding))
                .collect(),
            calls: analysis
                .calls
                .iter()
                .map(|call| (call.expr_id, call.callee))
                .collect(),
            package_items_by_binding,
            package_items_by_symbol,
            assignment_targets: analysis
                .assignment_targets
                .iter()
                .map(|target| (target.stmt_id, *target))
                .collect(),
        }
    }

    fn lower_bindings(&self) -> Vec<TypedBindingInfo> {
        self.analysis
            .bindings
            .iter()
            .map(|binding| TypedBindingInfo {
                id: binding.id,
                symbol: binding.symbol,
                kind: binding.kind,
                ty: self.package_target_for_type(binding.ty.clone()),
                span: binding.span,
            })
            .collect()
    }

    fn lower_stmt(&self, statement: &ast::Stmt) -> Stmt {
        match statement {
            ast::Stmt::Assign(stmt) => {
                let target = self.assignment_target(stmt.id);
                Stmt::Assign(AssignStmt {
                    id: stmt.id,
                    mutable: stmt.mutable,
                    is_update: target.is_update,
                    name: stmt.name.clone(),
                    binding: target.binding,
                    value: self.lower_expr(&stmt.value),
                    span: stmt.span,
                })
            }
            ast::Stmt::RecordDecl(stmt) => Stmt::Record(RecordStmt {
                id: stmt.id,
                name: stmt.name.clone(),
                fields: stmt
                    .fields
                    .iter()
                    .map(|field| RecordField {
                        name: field.name.clone(),
                        ty: self.type_info_from_type_expr(&field.type_name),
                        span: field.span,
                    })
                    .collect(),
                span: stmt.span,
            }),
            ast::Stmt::FuncDecl(stmt) => {
                let binding = self.binding_for_decl(&stmt.name, stmt.span, BindingKind::Function);
                let return_ty = self.function_return_type(binding);
                Stmt::Function(FunctionStmt {
                    id: stmt.id,
                    name: stmt.name.clone(),
                    binding,
                    params: stmt
                        .params
                        .iter()
                        .map(|param| self.lower_param(param))
                        .collect(),
                    return_ty,
                    body: self.lower_value_block(&stmt.body),
                    span: stmt.span,
                })
            }
            ast::Stmt::If(stmt) => Stmt::If(IfStmt {
                id: stmt.id,
                condition: self.lower_expr(&stmt.condition),
                then_branch: self.lower_block(&stmt.then_branch),
                else_branch: stmt
                    .else_branch
                    .as_ref()
                    .map(|branch| self.lower_block(branch)),
                span: stmt.span,
            }),
            ast::Stmt::While(stmt) => Stmt::While(WhileStmt {
                id: stmt.id,
                condition: self.lower_expr(&stmt.condition),
                body: self.lower_block(&stmt.body),
                span: stmt.span,
            }),
            ast::Stmt::Expr(stmt) => Stmt::Expr(ExprStmt {
                id: stmt.id,
                expr: self.lower_expr(&stmt.expr),
                span: stmt.span,
            }),
        }
    }

    fn lower_block(&self, block: &ast::Block) -> Block {
        Block {
            statements: block
                .statements
                .iter()
                .map(|statement| self.lower_stmt(statement))
                .collect(),
            span: block.span,
        }
    }

    fn lower_value_block(&self, block: &ast::ValueBlock) -> ValueBlock {
        ValueBlock {
            statements: block
                .statements
                .iter()
                .map(|statement| self.lower_stmt(statement))
                .collect(),
            expr: Box::new(self.lower_expr(&block.expr)),
            span: block.span,
        }
    }

    fn lower_expr(&self, expr: &ast::Expr) -> Expr {
        let id = expr.id();
        let ty = self.type_for_expr(id);
        let kind = match expr {
            ast::Expr::Int(expr) => ExprKind::Int(expr.value),
            ast::Expr::Bool(expr) => ExprKind::Bool(expr.value),
            ast::Expr::String(expr) => ExprKind::String(expr.value.clone()),
            ast::Expr::Ident(expr) => ExprKind::Ident(IdentExpr {
                name: expr.name.clone(),
                binding: self.binding_for_expr(expr.id),
                target: self.target_for_expr(expr.id),
            }),
            ast::Expr::ListLit(expr) => ExprKind::ListLit(ListLitExpr {
                items: expr
                    .items
                    .iter()
                    .map(|item| self.lower_expr(item))
                    .collect(),
            }),
            ast::Expr::Index(expr) => ExprKind::Index(IndexExpr {
                base: Box::new(self.lower_expr(&expr.base)),
                index: Box::new(self.lower_expr(&expr.index)),
            }),
            ast::Expr::RecordLit(expr) => ExprKind::RecordLit(RecordLitExpr {
                type_name: expr.type_name.clone(),
                fields: expr
                    .fields
                    .iter()
                    .map(|field| RecordFieldInit {
                        name: field.name.clone(),
                        value: self.lower_expr(&field.value),
                        span: field.span,
                    })
                    .collect(),
            }),
            ast::Expr::Field(expr) => ExprKind::Field(FieldExpr {
                base: Box::new(self.lower_expr(&expr.base)),
                field: expr.field.clone(),
            }),
            ast::Expr::RecordUpdate(expr) => ExprKind::RecordUpdate(RecordUpdateExpr {
                base: Box::new(self.lower_expr(&expr.base)),
                fields: expr
                    .fields
                    .iter()
                    .map(|field| RecordFieldInit {
                        name: field.name.clone(),
                        value: self.lower_expr(&field.value),
                        span: field.span,
                    })
                    .collect(),
            }),
            ast::Expr::Unary(expr) => ExprKind::Unary(UnaryExpr {
                op: match expr.op {
                    ast::UnaryOp::Neg => UnaryOp::Neg,
                    ast::UnaryOp::Not => UnaryOp::Not,
                },
                expr: Box::new(self.lower_expr(&expr.expr)),
            }),
            ast::Expr::Binary(expr) => ExprKind::Binary(BinaryExpr {
                op: match expr.op {
                    ast::BinaryOp::Add => BinaryOp::Add,
                    ast::BinaryOp::Sub => BinaryOp::Sub,
                    ast::BinaryOp::Mul => BinaryOp::Mul,
                    ast::BinaryOp::Div => BinaryOp::Div,
                    ast::BinaryOp::Lt => BinaryOp::Lt,
                    ast::BinaryOp::LtEq => BinaryOp::LtEq,
                    ast::BinaryOp::Gt => BinaryOp::Gt,
                    ast::BinaryOp::GtEq => BinaryOp::GtEq,
                    ast::BinaryOp::EqEq => BinaryOp::EqEq,
                    ast::BinaryOp::BangEq => BinaryOp::BangEq,
                },
                left: Box::new(self.lower_expr(&expr.left)),
                right: Box::new(self.lower_expr(&expr.right)),
            }),
            ast::Expr::Call(expr) => ExprKind::Call(CallExpr {
                callee: Box::new(self.lower_expr(&expr.callee)),
                args: expr.args.iter().map(|arg| self.lower_expr(arg)).collect(),
                origin: CallOrigin::from(expr.origin),
                resolved_callee: self.resolved_callee_for_call(expr.id),
            }),
            ast::Expr::If(expr) => ExprKind::If(IfExpr {
                condition: Box::new(self.lower_expr(&expr.condition)),
                then_branch: self.lower_value_block(&expr.then_branch),
                else_branch: self.lower_value_block(&expr.else_branch),
            }),
            ast::Expr::Match(expr) => ExprKind::Match(MatchExpr {
                value: Box::new(self.lower_expr(&expr.value)),
                arms: expr
                    .arms
                    .iter()
                    .map(|arm| MatchArm {
                        pattern: match &arm.pattern {
                            ast::MatchPattern::OptionSome { binding, span } => {
                                MatchPattern::OptionSome {
                                    binding_name: binding.clone(),
                                    binding: self.binding_for_decl(
                                        binding,
                                        *span,
                                        BindingKind::Immutable,
                                    ),
                                    span: *span,
                                }
                            }
                            ast::MatchPattern::OptionNone { span } => {
                                MatchPattern::OptionNone { span: *span }
                            }
                        },
                        value: self.lower_expr(&arm.value),
                        span: arm.span,
                    })
                    .collect(),
            }),
            ast::Expr::Fn(expr) => {
                let return_ty = match ty.clone() {
                    TypeInfo::Function(FunctionTypeInfo { ret, .. }) => *ret,
                    _ => TypeInfo::Error,
                };
                ExprKind::Fn(FnExpr {
                    params: expr
                        .params
                        .iter()
                        .map(|param| self.lower_param(param))
                        .collect(),
                    return_ty,
                    body: self.lower_value_block(&expr.body),
                })
            }
        };
        Expr {
            id,
            ty,
            kind,
            span: expr.span(),
        }
    }

    fn lower_param(&self, param: &ast::Param) -> Param {
        let binding = self.binding_for_decl(&param.name, param.span, BindingKind::Parameter);
        Param {
            name: param.name.clone(),
            binding,
            ty: self.type_for_binding(binding),
            span: param.span,
        }
    }

    fn assignment_target(&self, id: StmtId) -> TypedAssignmentTarget {
        *self
            .assignment_targets
            .get(&id)
            .expect("checked assignment should have a target binding")
    }

    fn binding_for_expr(&self, id: ExprId) -> BindingId {
        *self
            .identifier_refs
            .get(&id)
            .expect("checked identifier should have a target binding")
    }

    fn type_for_expr(&self, id: ExprId) -> TypeInfo {
        let ty = self
            .expr_types
            .get(&id)
            .cloned()
            .expect("checked expression should have a type");
        self.package_target_for_type(ty)
    }

    fn resolved_callee_for_call(&self, id: ExprId) -> TypedCalleeInfo {
        let callee = *self
            .calls
            .get(&id)
            .expect("checked call should have resolved callee info");
        self.package_target_for_callee(callee)
    }

    fn target_for_expr(&self, id: ExprId) -> IdentTarget {
        let binding = self.binding_for_expr(id);
        self.package_items_by_binding
            .get(&binding)
            .copied()
            .map(|item| IdentTarget::PackageItem { binding, item })
            .unwrap_or(IdentTarget::Binding(binding))
    }

    fn package_target_for_callee(&self, callee: TypedCalleeInfo) -> TypedCalleeInfo {
        match callee {
            TypedCalleeInfo::Binding(binding) => self
                .package_items_by_binding
                .get(&binding)
                .copied()
                .map(|item| TypedCalleeInfo::PackageItem { binding, item })
                .unwrap_or(TypedCalleeInfo::Binding(binding)),
            other => other,
        }
    }

    fn binding_for_decl(&self, name: &str, span: Span, kind: BindingKind) -> BindingId {
        self.analysis
            .bindings
            .iter()
            .find(|binding| {
                binding.kind == kind
                    && binding.span == span
                    && self.analysis.symbols.resolve(binding.symbol) == name
            })
            .map(|binding| binding.id)
            .expect("checked declaration should have a binding")
    }

    fn type_for_binding(&self, id: BindingId) -> TypeInfo {
        let ty = self
            .analysis
            .bindings
            .iter()
            .find(|binding| binding.id == id)
            .map(|binding| binding.ty.clone())
            .expect("checked binding should have a type");
        self.package_target_for_type(ty)
    }

    fn function_return_type(&self, id: BindingId) -> TypeInfo {
        match self.type_for_binding(id) {
            TypeInfo::Function(sig) => *sig.ret,
            _ => TypeInfo::Error,
        }
    }

    fn type_info_from_type_expr(&self, type_expr: &ast::TypeExpr) -> TypeInfo {
        match type_expr {
            ast::TypeExpr::Int => TypeInfo::Int,
            ast::TypeExpr::Bool => TypeInfo::Bool,
            ast::TypeExpr::String => TypeInfo::String,
            ast::TypeExpr::Named(name) => self
                .analysis
                .symbols
                .lookup(name)
                .map(TypeInfo::Record)
                .map(|ty| self.package_target_for_type(ty))
                .unwrap_or(TypeInfo::Error),
            ast::TypeExpr::Generic(generic)
                if generic.name == "List" && generic.args.len() == 1 =>
            {
                TypeInfo::List(Box::new(self.type_info_from_type_expr(&generic.args[0])))
            }
            ast::TypeExpr::Generic(generic)
                if generic.name == "Option" && generic.args.len() == 1 =>
            {
                TypeInfo::Option(Box::new(self.type_info_from_type_expr(&generic.args[0])))
            }
            ast::TypeExpr::Generic(_) => TypeInfo::Error,
            ast::TypeExpr::Function(function) => TypeInfo::Function(FunctionTypeInfo {
                params: function
                    .params
                    .iter()
                    .map(|param| self.type_info_from_type_expr(param))
                    .collect(),
                ret: Box::new(self.type_info_from_type_expr(&function.ret)),
            }),
        }
    }

    fn package_target_for_type(&self, ty: TypeInfo) -> TypeInfo {
        match ty {
            TypeInfo::Record(symbol) => self
                .package_items_by_symbol
                .get(&symbol)
                .copied()
                .map(|item| TypeInfo::PackageRecord { symbol, item })
                .unwrap_or(TypeInfo::Record(symbol)),
            TypeInfo::Function(function) => TypeInfo::Function(FunctionTypeInfo {
                params: function
                    .params
                    .into_iter()
                    .map(|param| self.package_target_for_type(param))
                    .collect(),
                ret: Box::new(self.package_target_for_type(*function.ret)),
            }),
            TypeInfo::List(item) => TypeInfo::List(Box::new(self.package_target_for_type(*item))),
            TypeInfo::Option(item) => {
                TypeInfo::Option(Box::new(self.package_target_for_type(*item)))
            }
            other => other,
        }
    }
}

impl From<ast::CallOrigin> for CallOrigin {
    fn from(origin: ast::CallOrigin) -> Self {
        match origin {
            ast::CallOrigin::Ordinary => Self::Ordinary,
            ast::CallOrigin::Chained => Self::Chained,
            ast::CallOrigin::QualifiedChained => Self::QualifiedChained,
        }
    }
}
