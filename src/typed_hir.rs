use std::collections::HashMap;

use crate::{
    ast,
    identity::{BindingId, BindingKind, ExprId, StmtId},
    package::PackageSymbolGraph,
    span::Span,
    symbol::SymbolTable,
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
    RecordLit(RecordLitExpr),
    Field(FieldExpr),
    RecordUpdate(RecordUpdateExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    If(IfExpr),
    Fn(FnExpr),
}

#[derive(Clone, Debug)]
pub struct IdentExpr {
    pub name: String,
    pub binding: BindingId,
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
    let lowerer = Lowerer::new(analysis);
    let statements = program
        .statements
        .iter()
        .map(|statement| lowerer.lower_stmt(statement))
        .collect();
    Program {
        statements,
        bindings: analysis.bindings.clone(),
        package_graph,
        symbols: analysis.symbols.clone(),
    }
}

struct Lowerer<'a> {
    analysis: &'a TypeCheckOutput,
    expr_types: HashMap<ExprId, TypeInfo>,
    identifier_refs: HashMap<ExprId, BindingId>,
    calls: HashMap<ExprId, TypedCalleeInfo>,
    assignment_targets: HashMap<StmtId, TypedAssignmentTarget>,
}

impl<'a> Lowerer<'a> {
    fn new(analysis: &'a TypeCheckOutput) -> Self {
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
            assignment_targets: analysis
                .assignment_targets
                .iter()
                .map(|target| (target.stmt_id, *target))
                .collect(),
        }
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
        self.expr_types
            .get(&id)
            .cloned()
            .expect("checked expression should have a type")
    }

    fn resolved_callee_for_call(&self, id: ExprId) -> TypedCalleeInfo {
        *self
            .calls
            .get(&id)
            .expect("checked call should have resolved callee info")
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
        self.analysis
            .bindings
            .iter()
            .find(|binding| binding.id == id)
            .map(|binding| binding.ty.clone())
            .expect("checked binding should have a type")
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
                .unwrap_or(TypeInfo::Error),
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
