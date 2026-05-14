use std::collections::{HashMap, HashSet};

use crate::{
    ast,
    identity::{BindingId, BindingKind, ExprId, PackageItemId, StmtId},
    known_enum,
    package::PackageSymbolGraph,
    span::Span,
    symbol::{Symbol, SymbolTable},
    types::{FunctionTypeInfo, TypeInfo},
    typing::{TypeCheckOutput, TypedAssignmentTarget, TypedBindingInfo, TypedCalleeInfo},
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
    Enum(EnumStmt),
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
            Self::Enum(stmt) => stmt.id,
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
    pub package_item: Option<PackageItemId>,
    pub type_params: Vec<String>,
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
pub struct EnumStmt {
    pub id: StmtId,
    pub name: String,
    pub package_item: Option<PackageItemId>,
    pub type_params: Vec<String>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumVariant {
    pub name: String,
    pub payload: Option<TypeInfo>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FunctionStmt {
    pub id: StmtId,
    pub name: String,
    pub binding: BindingId,
    pub package_item: Option<PackageItemId>,
    pub type_params: Vec<String>,
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
    Try(TryExpr),
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
    EnumVariant {
        binding: BindingId,
        enum_name: Symbol,
        enum_item: Option<PackageItemId>,
        variant_name: Symbol,
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

#[derive(Clone, Debug)]
pub struct TryExpr {
    pub expr: Box<Expr>,
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
    Variant(EnumVariantPattern),
}

#[derive(Clone, Debug)]
pub struct EnumVariantPattern {
    pub enum_name: String,
    pub variant_name: String,
    pub binding_name: Option<String>,
    pub binding: Option<BindingId>,
    pub span: Span,
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
    let lowerer = Lowerer::new(program, analysis);
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

pub fn merge_modules(modules: &[Program], package_graph: PackageSymbolGraph) -> Program {
    let mut symbols = SymbolTable::default();
    let mut statements = Vec::new();
    let mut bindings = Vec::new();
    let mut next_binding = 0;
    let mut next_stmt = 0;
    let mut next_expr = 0;

    for module in modules {
        let mut remapper = ModuleRemapper {
            from_symbols: &module.symbols,
            to_symbols: &mut symbols,
            binding_offset: next_binding,
            stmt_offset: next_stmt,
            expr_offset: next_expr,
        };
        bindings.extend(
            module
                .bindings
                .iter()
                .map(|binding| remapper.binding_info(binding)),
        );
        statements.extend(
            module
                .statements
                .iter()
                .map(|statement| remapper.stmt(statement)),
        );
        next_binding += max_binding_id_in_program(module).map_or(0, |id| id + 1);
        next_stmt += max_stmt_id_in_program(module).map_or(0, |id| id + 1);
        next_expr += max_expr_id_in_program(module).map_or(0, |id| id + 1);
    }

    Program {
        statements,
        bindings,
        package_graph,
        symbols,
    }
}

struct ModuleRemapper<'a, 's> {
    from_symbols: &'a SymbolTable,
    to_symbols: &'s mut SymbolTable,
    binding_offset: u32,
    stmt_offset: u32,
    expr_offset: u32,
}

impl ModuleRemapper<'_, '_> {
    fn binding_info(&mut self, binding: &TypedBindingInfo) -> TypedBindingInfo {
        TypedBindingInfo {
            id: self.binding(binding.id),
            symbol: self.symbol(binding.symbol),
            kind: binding.kind,
            ty: self.type_info(&binding.ty),
            package_item: binding.package_item,
            span: binding.span,
        }
    }

    fn stmt(&mut self, statement: &Stmt) -> Stmt {
        match statement {
            Stmt::Assign(stmt) => Stmt::Assign(AssignStmt {
                id: self.stmt_id(stmt.id),
                mutable: stmt.mutable,
                is_update: stmt.is_update,
                name: stmt.name.clone(),
                binding: self.binding(stmt.binding),
                value: self.expr(&stmt.value),
                span: stmt.span,
            }),
            Stmt::Record(stmt) => Stmt::Record(RecordStmt {
                id: self.stmt_id(stmt.id),
                name: stmt.name.clone(),
                package_item: stmt.package_item,
                type_params: stmt.type_params.clone(),
                fields: stmt
                    .fields
                    .iter()
                    .map(|field| RecordField {
                        name: field.name.clone(),
                        ty: self.type_info(&field.ty),
                        span: field.span,
                    })
                    .collect(),
                span: stmt.span,
            }),
            Stmt::Enum(stmt) => Stmt::Enum(EnumStmt {
                id: self.stmt_id(stmt.id),
                name: stmt.name.clone(),
                package_item: stmt.package_item,
                type_params: stmt.type_params.clone(),
                variants: stmt
                    .variants
                    .iter()
                    .map(|variant| EnumVariant {
                        name: variant.name.clone(),
                        payload: variant
                            .payload
                            .as_ref()
                            .map(|payload| self.type_info(payload)),
                        span: variant.span,
                    })
                    .collect(),
                span: stmt.span,
            }),
            Stmt::Function(stmt) => Stmt::Function(FunctionStmt {
                id: self.stmt_id(stmt.id),
                name: stmt.name.clone(),
                binding: self.binding(stmt.binding),
                package_item: stmt.package_item,
                type_params: stmt.type_params.clone(),
                params: stmt.params.iter().map(|param| self.param(param)).collect(),
                return_ty: self.type_info(&stmt.return_ty),
                body: self.value_block(&stmt.body),
                span: stmt.span,
            }),
            Stmt::If(stmt) => Stmt::If(IfStmt {
                id: self.stmt_id(stmt.id),
                condition: self.expr(&stmt.condition),
                then_branch: self.block(&stmt.then_branch),
                else_branch: stmt.else_branch.as_ref().map(|branch| self.block(branch)),
                span: stmt.span,
            }),
            Stmt::While(stmt) => Stmt::While(WhileStmt {
                id: self.stmt_id(stmt.id),
                condition: self.expr(&stmt.condition),
                body: self.block(&stmt.body),
                span: stmt.span,
            }),
            Stmt::Expr(stmt) => Stmt::Expr(ExprStmt {
                id: self.stmt_id(stmt.id),
                expr: self.expr(&stmt.expr),
                span: stmt.span,
            }),
        }
    }

    fn block(&mut self, block: &Block) -> Block {
        Block {
            statements: block
                .statements
                .iter()
                .map(|statement| self.stmt(statement))
                .collect(),
            span: block.span,
        }
    }

    fn value_block(&mut self, block: &ValueBlock) -> ValueBlock {
        ValueBlock {
            statements: block
                .statements
                .iter()
                .map(|statement| self.stmt(statement))
                .collect(),
            expr: Box::new(self.expr(&block.expr)),
            span: block.span,
        }
    }

    fn expr(&mut self, expr: &Expr) -> Expr {
        Expr {
            id: self.expr_id(expr.id),
            ty: self.type_info(&expr.ty),
            kind: match &expr.kind {
                ExprKind::Int(value) => ExprKind::Int(*value),
                ExprKind::Bool(value) => ExprKind::Bool(*value),
                ExprKind::String(value) => ExprKind::String(value.clone()),
                ExprKind::Ident(expr) => ExprKind::Ident(IdentExpr {
                    name: expr.name.clone(),
                    binding: self.binding(expr.binding),
                    target: self.ident_target(expr.target),
                }),
                ExprKind::ListLit(expr) => ExprKind::ListLit(ListLitExpr {
                    items: expr.items.iter().map(|item| self.expr(item)).collect(),
                }),
                ExprKind::Index(expr) => ExprKind::Index(IndexExpr {
                    base: Box::new(self.expr(&expr.base)),
                    index: Box::new(self.expr(&expr.index)),
                }),
                ExprKind::RecordLit(expr) => ExprKind::RecordLit(RecordLitExpr {
                    type_name: expr.type_name.clone(),
                    fields: expr
                        .fields
                        .iter()
                        .map(|field| RecordFieldInit {
                            name: field.name.clone(),
                            value: self.expr(&field.value),
                            span: field.span,
                        })
                        .collect(),
                }),
                ExprKind::Field(expr) => ExprKind::Field(FieldExpr {
                    base: Box::new(self.expr(&expr.base)),
                    field: expr.field.clone(),
                }),
                ExprKind::RecordUpdate(expr) => ExprKind::RecordUpdate(RecordUpdateExpr {
                    base: Box::new(self.expr(&expr.base)),
                    fields: expr
                        .fields
                        .iter()
                        .map(|field| RecordFieldInit {
                            name: field.name.clone(),
                            value: self.expr(&field.value),
                            span: field.span,
                        })
                        .collect(),
                }),
                ExprKind::Unary(expr) => ExprKind::Unary(UnaryExpr {
                    op: expr.op,
                    expr: Box::new(self.expr(&expr.expr)),
                }),
                ExprKind::Binary(expr) => ExprKind::Binary(BinaryExpr {
                    op: expr.op,
                    left: Box::new(self.expr(&expr.left)),
                    right: Box::new(self.expr(&expr.right)),
                }),
                ExprKind::Call(expr) => ExprKind::Call(CallExpr {
                    callee: Box::new(self.expr(&expr.callee)),
                    args: expr.args.iter().map(|arg| self.expr(arg)).collect(),
                    origin: expr.origin,
                    resolved_callee: self.callee(expr.resolved_callee),
                }),
                ExprKind::Try(expr) => ExprKind::Try(TryExpr {
                    expr: Box::new(self.expr(&expr.expr)),
                }),
                ExprKind::If(expr) => ExprKind::If(IfExpr {
                    condition: Box::new(self.expr(&expr.condition)),
                    then_branch: self.value_block(&expr.then_branch),
                    else_branch: self.value_block(&expr.else_branch),
                }),
                ExprKind::Match(expr) => ExprKind::Match(MatchExpr {
                    value: Box::new(self.expr(&expr.value)),
                    arms: expr
                        .arms
                        .iter()
                        .map(|arm| MatchArm {
                            pattern: self.match_pattern(&arm.pattern),
                            value: self.expr(&arm.value),
                            span: arm.span,
                        })
                        .collect(),
                }),
                ExprKind::Fn(expr) => ExprKind::Fn(FnExpr {
                    params: expr.params.iter().map(|param| self.param(param)).collect(),
                    return_ty: self.type_info(&expr.return_ty),
                    body: self.value_block(&expr.body),
                }),
            },
            span: expr.span,
        }
    }

    fn param(&mut self, param: &Param) -> Param {
        Param {
            name: param.name.clone(),
            binding: self.binding(param.binding),
            ty: self.type_info(&param.ty),
            span: param.span,
        }
    }

    fn match_pattern(&mut self, pattern: &MatchPattern) -> MatchPattern {
        match pattern {
            MatchPattern::Variant(pattern) => MatchPattern::Variant(EnumVariantPattern {
                enum_name: pattern.enum_name.clone(),
                variant_name: pattern.variant_name.clone(),
                binding_name: pattern.binding_name.clone(),
                binding: pattern.binding.map(|binding| self.binding(binding)),
                span: pattern.span,
            }),
        }
    }

    fn ident_target(&mut self, target: IdentTarget) -> IdentTarget {
        match target {
            IdentTarget::Binding(binding) => IdentTarget::Binding(self.binding(binding)),
            IdentTarget::PackageItem { binding, item } => IdentTarget::PackageItem {
                binding: self.binding(binding),
                item,
            },
            IdentTarget::EnumVariant {
                binding,
                enum_name,
                enum_item,
                variant_name,
            } => IdentTarget::EnumVariant {
                binding: self.binding(binding),
                enum_name: self.symbol(enum_name),
                enum_item,
                variant_name: self.symbol(variant_name),
            },
        }
    }

    fn callee(&mut self, callee: TypedCalleeInfo) -> TypedCalleeInfo {
        match callee {
            TypedCalleeInfo::Binding(binding) => TypedCalleeInfo::Binding(self.binding(binding)),
            TypedCalleeInfo::PackageItem { binding, item } => TypedCalleeInfo::PackageItem {
                binding: self.binding(binding),
                item,
            },
            TypedCalleeInfo::EnumVariant {
                binding,
                enum_name,
                enum_item,
                variant_name,
            } => TypedCalleeInfo::EnumVariant {
                binding: self.binding(binding),
                enum_name: self.symbol(enum_name),
                enum_item,
                variant_name: self.symbol(variant_name),
            },
            TypedCalleeInfo::Builtin { binding, name } => TypedCalleeInfo::Builtin {
                binding: self.binding(binding),
                name,
            },
            TypedCalleeInfo::Value => TypedCalleeInfo::Value,
            TypedCalleeInfo::Error => TypedCalleeInfo::Error,
        }
    }

    fn type_info(&mut self, ty: &TypeInfo) -> TypeInfo {
        match ty {
            TypeInfo::GenericParam(symbol) => TypeInfo::GenericParam(self.symbol(*symbol)),
            TypeInfo::Record(symbol, args) => TypeInfo::Record(
                self.symbol(*symbol),
                args.iter().map(|arg| self.type_info(arg)).collect(),
            ),
            TypeInfo::PackageRecord { symbol, item, args } => TypeInfo::PackageRecord {
                symbol: self.symbol(*symbol),
                item: *item,
                args: args.iter().map(|arg| self.type_info(arg)).collect(),
            },
            TypeInfo::Enum { symbol, args } => TypeInfo::Enum {
                symbol: self.symbol(*symbol),
                args: args.iter().map(|arg| self.type_info(arg)).collect(),
            },
            TypeInfo::PackageEnum { symbol, item, args } => TypeInfo::PackageEnum {
                symbol: self.symbol(*symbol),
                item: *item,
                args: args.iter().map(|arg| self.type_info(arg)).collect(),
            },
            TypeInfo::List(item) => TypeInfo::List(Box::new(self.type_info(item))),
            TypeInfo::Map(key, value) => TypeInfo::Map(
                Box::new(self.type_info(key)),
                Box::new(self.type_info(value)),
            ),
            TypeInfo::Option(item) => TypeInfo::Option(Box::new(self.type_info(item))),
            TypeInfo::Result(ok, err) => {
                TypeInfo::Result(Box::new(self.type_info(ok)), Box::new(self.type_info(err)))
            }
            TypeInfo::EnumConstructor {
                enum_symbol,
                enum_item,
                variant,
            } => TypeInfo::EnumConstructor {
                enum_symbol: self.symbol(*enum_symbol),
                enum_item: *enum_item,
                variant: self.symbol(*variant),
            },
            TypeInfo::Function(function) => TypeInfo::Function(FunctionTypeInfo {
                params: function
                    .params
                    .iter()
                    .map(|param| self.type_info(param))
                    .collect(),
                ret: Box::new(self.type_info(&function.ret)),
            }),
            TypeInfo::Int
            | TypeInfo::Bool
            | TypeInfo::String
            | TypeInfo::Builtin(_)
            | TypeInfo::Unknown
            | TypeInfo::Error => ty.clone(),
        }
    }

    fn symbol(&mut self, symbol: Symbol) -> Symbol {
        self.to_symbols.intern(self.from_symbols.resolve(symbol))
    }

    fn binding(&self, binding: BindingId) -> BindingId {
        BindingId::new(binding.as_u32() + self.binding_offset)
    }

    fn stmt_id(&self, id: StmtId) -> StmtId {
        StmtId::new(id.as_u32() + self.stmt_offset)
    }

    fn expr_id(&self, id: ExprId) -> ExprId {
        ExprId::new(id.as_u32() + self.expr_offset)
    }
}

fn max_binding_id_in_program(program: &Program) -> Option<u32> {
    let mut max = program
        .bindings
        .iter()
        .map(|binding| binding.id.as_u32())
        .max();
    for statement in &program.statements {
        max = max_opt(max, max_binding_id_in_stmt(statement));
    }
    max
}

fn max_binding_id_in_stmt(statement: &Stmt) -> Option<u32> {
    let mut max = None;
    match statement {
        Stmt::Assign(stmt) => {
            max = max_opt(max, Some(stmt.binding.as_u32()));
            max = max_opt(max, max_binding_id_in_expr(&stmt.value));
        }
        Stmt::Record(_) | Stmt::Enum(_) => {}
        Stmt::Function(stmt) => {
            max = max_opt(max, Some(stmt.binding.as_u32()));
            for param in &stmt.params {
                max = max_opt(max, Some(param.binding.as_u32()));
            }
            max = max_opt(max, max_binding_id_in_value_block(&stmt.body));
        }
        Stmt::If(stmt) => {
            max = max_opt(max, max_binding_id_in_expr(&stmt.condition));
            max = max_opt(max, max_binding_id_in_block(&stmt.then_branch));
            if let Some(else_branch) = &stmt.else_branch {
                max = max_opt(max, max_binding_id_in_block(else_branch));
            }
        }
        Stmt::While(stmt) => {
            max = max_opt(max, max_binding_id_in_expr(&stmt.condition));
            max = max_opt(max, max_binding_id_in_block(&stmt.body));
        }
        Stmt::Expr(stmt) => {
            max = max_opt(max, max_binding_id_in_expr(&stmt.expr));
        }
    }
    max
}

fn max_binding_id_in_block(block: &Block) -> Option<u32> {
    max_binding_id_in_statements(&block.statements)
}

fn max_binding_id_in_value_block(block: &ValueBlock) -> Option<u32> {
    let mut max = max_binding_id_in_statements(&block.statements);
    max = max_opt(max, max_binding_id_in_expr(&block.expr));
    max
}

fn max_binding_id_in_statements(statements: &[Stmt]) -> Option<u32> {
    statements.iter().filter_map(max_binding_id_in_stmt).max()
}

fn max_binding_id_in_expr(expr: &Expr) -> Option<u32> {
    let mut max = None;
    match &expr.kind {
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::String(_) => {}
        ExprKind::Ident(expr) => {
            max = max_opt(max, Some(expr.binding.as_u32()));
            max = max_opt(max, max_binding_id_in_ident_target(expr.target));
        }
        ExprKind::ListLit(expr) => {
            for item in &expr.items {
                max = max_opt(max, max_binding_id_in_expr(item));
            }
        }
        ExprKind::Index(expr) => {
            max = max_opt(max, max_binding_id_in_expr(&expr.base));
            max = max_opt(max, max_binding_id_in_expr(&expr.index));
        }
        ExprKind::RecordLit(expr) => {
            for field in &expr.fields {
                max = max_opt(max, max_binding_id_in_expr(&field.value));
            }
        }
        ExprKind::Field(expr) => max = max_opt(max, max_binding_id_in_expr(&expr.base)),
        ExprKind::RecordUpdate(expr) => {
            max = max_opt(max, max_binding_id_in_expr(&expr.base));
            for field in &expr.fields {
                max = max_opt(max, max_binding_id_in_expr(&field.value));
            }
        }
        ExprKind::Unary(expr) => max = max_opt(max, max_binding_id_in_expr(&expr.expr)),
        ExprKind::Binary(expr) => {
            max = max_opt(max, max_binding_id_in_expr(&expr.left));
            max = max_opt(max, max_binding_id_in_expr(&expr.right));
        }
        ExprKind::Call(expr) => {
            max = max_opt(max, max_binding_id_in_expr(&expr.callee));
            max = max_opt(max, max_binding_id_in_callee(expr.resolved_callee));
            for arg in &expr.args {
                max = max_opt(max, max_binding_id_in_expr(arg));
            }
        }
        ExprKind::Try(expr) => max = max_opt(max, max_binding_id_in_expr(&expr.expr)),
        ExprKind::If(expr) => {
            max = max_opt(max, max_binding_id_in_expr(&expr.condition));
            max = max_opt(max, max_binding_id_in_value_block(&expr.then_branch));
            max = max_opt(max, max_binding_id_in_value_block(&expr.else_branch));
        }
        ExprKind::Match(expr) => {
            max = max_opt(max, max_binding_id_in_expr(&expr.value));
            for arm in &expr.arms {
                let MatchPattern::Variant(pattern) = &arm.pattern;
                if let Some(binding) = pattern.binding {
                    max = max_opt(max, Some(binding.as_u32()));
                }
                max = max_opt(max, max_binding_id_in_expr(&arm.value));
            }
        }
        ExprKind::Fn(expr) => {
            for param in &expr.params {
                max = max_opt(max, Some(param.binding.as_u32()));
            }
            max = max_opt(max, max_binding_id_in_value_block(&expr.body));
        }
    }
    max
}

fn max_binding_id_in_ident_target(target: IdentTarget) -> Option<u32> {
    match target {
        IdentTarget::Binding(binding)
        | IdentTarget::PackageItem { binding, .. }
        | IdentTarget::EnumVariant { binding, .. } => Some(binding.as_u32()),
    }
}

fn max_binding_id_in_callee(callee: TypedCalleeInfo) -> Option<u32> {
    match callee {
        TypedCalleeInfo::Binding(binding)
        | TypedCalleeInfo::PackageItem { binding, .. }
        | TypedCalleeInfo::EnumVariant { binding, .. }
        | TypedCalleeInfo::Builtin { binding, .. } => Some(binding.as_u32()),
        TypedCalleeInfo::Value | TypedCalleeInfo::Error => None,
    }
}

fn max_stmt_id_in_program(program: &Program) -> Option<u32> {
    program
        .statements
        .iter()
        .filter_map(max_stmt_id_in_stmt)
        .max()
}

fn max_stmt_id_in_stmt(statement: &Stmt) -> Option<u32> {
    let mut max = Some(statement.id().as_u32());
    match statement {
        Stmt::Function(stmt) => max = max_opt(max, max_stmt_id_in_value_block(&stmt.body)),
        Stmt::If(stmt) => {
            max = max_opt(max, max_stmt_id_in_block(&stmt.then_branch));
            if let Some(else_branch) = &stmt.else_branch {
                max = max_opt(max, max_stmt_id_in_block(else_branch));
            }
        }
        Stmt::While(stmt) => max = max_opt(max, max_stmt_id_in_block(&stmt.body)),
        Stmt::Assign(_) | Stmt::Record(_) | Stmt::Enum(_) | Stmt::Expr(_) => {}
    }
    max
}

fn max_stmt_id_in_block(block: &Block) -> Option<u32> {
    max_stmt_id_in_statements(&block.statements)
}

fn max_stmt_id_in_value_block(block: &ValueBlock) -> Option<u32> {
    max_stmt_id_in_statements(&block.statements)
}

fn max_stmt_id_in_statements(statements: &[Stmt]) -> Option<u32> {
    statements.iter().filter_map(max_stmt_id_in_stmt).max()
}

fn max_expr_id_in_program(program: &Program) -> Option<u32> {
    program
        .statements
        .iter()
        .filter_map(max_expr_id_in_stmt)
        .max()
}

fn max_expr_id_in_stmt(statement: &Stmt) -> Option<u32> {
    match statement {
        Stmt::Assign(stmt) => max_expr_id_in_expr(&stmt.value),
        Stmt::Record(_) | Stmt::Enum(_) => None,
        Stmt::Function(stmt) => max_expr_id_in_value_block(&stmt.body),
        Stmt::If(stmt) => {
            let mut max = max_expr_id_in_expr(&stmt.condition);
            max = max_opt(max, max_expr_id_in_block(&stmt.then_branch));
            if let Some(else_branch) = &stmt.else_branch {
                max = max_opt(max, max_expr_id_in_block(else_branch));
            }
            max
        }
        Stmt::While(stmt) => max_opt(
            max_expr_id_in_expr(&stmt.condition),
            max_expr_id_in_block(&stmt.body),
        ),
        Stmt::Expr(stmt) => max_expr_id_in_expr(&stmt.expr),
    }
}

fn max_expr_id_in_block(block: &Block) -> Option<u32> {
    max_expr_id_in_statements(&block.statements)
}

fn max_expr_id_in_value_block(block: &ValueBlock) -> Option<u32> {
    max_opt(
        max_expr_id_in_statements(&block.statements),
        max_expr_id_in_expr(&block.expr),
    )
}

fn max_expr_id_in_statements(statements: &[Stmt]) -> Option<u32> {
    statements.iter().filter_map(max_expr_id_in_stmt).max()
}

fn max_expr_id_in_expr(expr: &Expr) -> Option<u32> {
    let mut max = Some(expr.id.as_u32());
    match &expr.kind {
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::String(_) | ExprKind::Ident(_) => {}
        ExprKind::ListLit(expr) => {
            for item in &expr.items {
                max = max_opt(max, max_expr_id_in_expr(item));
            }
        }
        ExprKind::Index(expr) => {
            max = max_opt(max, max_expr_id_in_expr(&expr.base));
            max = max_opt(max, max_expr_id_in_expr(&expr.index));
        }
        ExprKind::RecordLit(expr) => {
            for field in &expr.fields {
                max = max_opt(max, max_expr_id_in_expr(&field.value));
            }
        }
        ExprKind::Field(expr) => max = max_opt(max, max_expr_id_in_expr(&expr.base)),
        ExprKind::RecordUpdate(expr) => {
            max = max_opt(max, max_expr_id_in_expr(&expr.base));
            for field in &expr.fields {
                max = max_opt(max, max_expr_id_in_expr(&field.value));
            }
        }
        ExprKind::Unary(expr) => max = max_opt(max, max_expr_id_in_expr(&expr.expr)),
        ExprKind::Binary(expr) => {
            max = max_opt(max, max_expr_id_in_expr(&expr.left));
            max = max_opt(max, max_expr_id_in_expr(&expr.right));
        }
        ExprKind::Call(expr) => {
            max = max_opt(max, max_expr_id_in_expr(&expr.callee));
            for arg in &expr.args {
                max = max_opt(max, max_expr_id_in_expr(arg));
            }
        }
        ExprKind::Try(expr) => max = max_opt(max, max_expr_id_in_expr(&expr.expr)),
        ExprKind::If(expr) => {
            max = max_opt(max, max_expr_id_in_expr(&expr.condition));
            max = max_opt(max, max_expr_id_in_value_block(&expr.then_branch));
            max = max_opt(max, max_expr_id_in_value_block(&expr.else_branch));
        }
        ExprKind::Match(expr) => {
            max = max_opt(max, max_expr_id_in_expr(&expr.value));
            for arm in &expr.arms {
                max = max_opt(max, max_expr_id_in_expr(&arm.value));
            }
        }
        ExprKind::Fn(expr) => max = max_opt(max, max_expr_id_in_value_block(&expr.body)),
    }
    max
}

fn max_opt(left: Option<u32>, right: Option<u32>) -> Option<u32> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left.max(right)),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

struct Lowerer<'a> {
    analysis: &'a TypeCheckOutput,
    expr_types: HashMap<ExprId, TypeInfo>,
    identifier_refs: HashMap<ExprId, BindingId>,
    calls: HashMap<ExprId, TypedCalleeInfo>,
    package_items_by_binding: HashMap<BindingId, PackageItemId>,
    package_items_by_symbol: HashMap<Symbol, PackageItemId>,
    enum_symbols: HashSet<Symbol>,
    assignment_targets: HashMap<StmtId, TypedAssignmentTarget>,
}

impl<'a> Lowerer<'a> {
    fn new(program: &ast::Program, analysis: &'a TypeCheckOutput) -> Self {
        let mut package_items_by_binding: HashMap<_, _> = analysis
            .bindings
            .iter()
            .filter_map(|binding| Some((binding.id, binding.package_item?)))
            .collect();
        package_items_by_binding.extend(program.statements.iter().filter_map(|statement| {
            match statement {
                ast::Stmt::FuncDecl(func) => {
                    let item = func.package_item?;
                    let binding = Self::binding_for_decl_in_analysis(
                        analysis,
                        &func.name,
                        func.span,
                        BindingKind::Function,
                    )?;
                    Some((binding, item))
                }
                _ => None,
            }
        }));
        let package_items_by_symbol = program
            .statements
            .iter()
            .filter_map(|statement| match statement {
                ast::Stmt::RecordDecl(record) => {
                    let item = record.package_item?;
                    let symbol = analysis.symbols.lookup(&record.name)?;
                    Some((symbol, item))
                }
                ast::Stmt::EnumDecl(enumeration) => {
                    let item = enumeration.package_item?;
                    let symbol = analysis.symbols.lookup(&enumeration.name)?;
                    Some((symbol, item))
                }
                _ => None,
            })
            .collect();
        let enum_symbols = program
            .statements
            .iter()
            .filter_map(|statement| match statement {
                ast::Stmt::EnumDecl(enumeration) => analysis.symbols.lookup(&enumeration.name),
                _ => None,
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
            enum_symbols,
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
                package_item: binding
                    .package_item
                    .or_else(|| self.package_items_by_binding.get(&binding.id).copied()),
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
                package_item: stmt.package_item,
                type_params: stmt.type_params.clone(),
                fields: stmt
                    .fields
                    .iter()
                    .map(|field| RecordField {
                        name: field.name.clone(),
                        ty: self.type_info_from_type_expr_with_params(
                            &field.type_name,
                            &stmt.type_params,
                        ),
                        span: field.span,
                    })
                    .collect(),
                span: stmt.span,
            }),
            ast::Stmt::EnumDecl(stmt) => Stmt::Enum(EnumStmt {
                id: stmt.id,
                name: stmt.name.clone(),
                package_item: stmt.package_item,
                type_params: stmt.type_params.clone(),
                variants: stmt
                    .variants
                    .iter()
                    .map(|variant| EnumVariant {
                        name: variant.name.clone(),
                        payload: variant.payload.as_ref().map(|payload| {
                            self.type_info_from_type_expr_with_params(payload, &stmt.type_params)
                        }),
                        span: variant.span,
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
                    package_item: self.package_items_by_binding.get(&binding).copied(),
                    type_params: stmt.type_params.clone(),
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
            ast::Expr::Try(expr) => ExprKind::Try(TryExpr {
                expr: Box::new(self.lower_expr(&expr.expr)),
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
                            ast::MatchPattern::Variant(pattern) => {
                                MatchPattern::Variant(EnumVariantPattern {
                                    enum_name: pattern.enum_name.clone(),
                                    variant_name: pattern.variant_name.clone(),
                                    binding_name: pattern.binding.clone(),
                                    binding: pattern.binding.as_ref().map(|binding| {
                                        self.binding_for_decl(
                                            binding,
                                            pattern.span,
                                            BindingKind::Immutable,
                                        )
                                    }),
                                    span: pattern.span,
                                })
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
        if let Some(TypeInfo::EnumConstructor {
            enum_symbol,
            enum_item,
            variant,
        }) = self
            .analysis
            .bindings
            .iter()
            .find(|candidate| candidate.id == binding)
            .map(|candidate| candidate.ty.clone())
        {
            return IdentTarget::EnumVariant {
                binding,
                enum_name: enum_symbol,
                enum_item,
                variant_name: variant,
            };
        }
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
        Self::binding_for_decl_in_analysis(self.analysis, name, span, kind)
            .expect("checked declaration should have a binding")
    }

    fn binding_for_decl_in_analysis(
        analysis: &TypeCheckOutput,
        name: &str,
        span: Span,
        kind: BindingKind,
    ) -> Option<BindingId> {
        analysis
            .bindings
            .iter()
            .find(|binding| {
                binding.kind == kind
                    && binding.span == span
                    && analysis.symbols.resolve(binding.symbol) == name
            })
            .map(|binding| binding.id)
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

    fn type_info_from_type_expr_with_params(
        &self,
        type_expr: &ast::TypeExpr,
        type_params: &[String],
    ) -> TypeInfo {
        match type_expr {
            ast::TypeExpr::Int => TypeInfo::Int,
            ast::TypeExpr::Bool => TypeInfo::Bool,
            ast::TypeExpr::String => TypeInfo::String,
            ast::TypeExpr::Named(name) if type_params.iter().any(|param| param == name) => self
                .analysis
                .symbols
                .lookup(name)
                .map(TypeInfo::GenericParam)
                .unwrap_or(TypeInfo::Error),
            ast::TypeExpr::Named(name) => self
                .analysis
                .symbols
                .lookup(name)
                .map(|symbol| {
                    let ty = if self.enum_symbols.contains(&symbol) {
                        TypeInfo::Enum {
                            symbol,
                            args: Vec::new(),
                        }
                    } else {
                        TypeInfo::Record(symbol, Vec::new())
                    };
                    self.package_target_for_type(ty)
                })
                .unwrap_or(TypeInfo::Error),
            ast::TypeExpr::Generic(generic)
                if generic.name == "List" && generic.args.len() == 1 =>
            {
                TypeInfo::List(Box::new(
                    self.type_info_from_type_expr_with_params(&generic.args[0], type_params),
                ))
            }
            ast::TypeExpr::Generic(generic)
                if generic.name == known_enum::OPTION_NAME && generic.args.len() == 1 =>
            {
                TypeInfo::Option(Box::new(
                    self.type_info_from_type_expr_with_params(&generic.args[0], type_params),
                ))
            }
            ast::TypeExpr::Generic(generic)
                if generic.name == known_enum::RESULT_NAME && generic.args.len() == 2 =>
            {
                TypeInfo::Result(
                    Box::new(
                        self.type_info_from_type_expr_with_params(&generic.args[0], type_params),
                    ),
                    Box::new(
                        self.type_info_from_type_expr_with_params(&generic.args[1], type_params),
                    ),
                )
            }
            ast::TypeExpr::Generic(generic) if generic.name == "Map" && generic.args.len() == 2 => {
                TypeInfo::Map(
                    Box::new(
                        self.type_info_from_type_expr_with_params(&generic.args[0], type_params),
                    ),
                    Box::new(
                        self.type_info_from_type_expr_with_params(&generic.args[1], type_params),
                    ),
                )
            }
            ast::TypeExpr::Generic(generic)
                if self
                    .analysis
                    .symbols
                    .lookup(&generic.name)
                    .is_some_and(|symbol| self.enum_symbols.contains(&symbol)) =>
            {
                self.analysis
                    .symbols
                    .lookup(&generic.name)
                    .map(|symbol| {
                        self.package_target_for_type(TypeInfo::Enum {
                            symbol,
                            args: generic
                                .args
                                .iter()
                                .map(|arg| {
                                    self.type_info_from_type_expr_with_params(arg, type_params)
                                })
                                .collect(),
                        })
                    })
                    .unwrap_or(TypeInfo::Error)
            }
            ast::TypeExpr::Generic(generic)
                if self.analysis.symbols.lookup(&generic.name).is_some() =>
            {
                self.analysis
                    .symbols
                    .lookup(&generic.name)
                    .map(|symbol| {
                        self.package_target_for_type(TypeInfo::Record(
                            symbol,
                            generic
                                .args
                                .iter()
                                .map(|arg| {
                                    self.type_info_from_type_expr_with_params(arg, type_params)
                                })
                                .collect(),
                        ))
                    })
                    .unwrap_or(TypeInfo::Error)
            }
            ast::TypeExpr::Generic(_) => TypeInfo::Error,
            ast::TypeExpr::Function(function) => TypeInfo::Function(FunctionTypeInfo {
                params: function
                    .params
                    .iter()
                    .map(|param| self.type_info_from_type_expr_with_params(param, type_params))
                    .collect(),
                ret: Box::new(
                    self.type_info_from_type_expr_with_params(&function.ret, type_params),
                ),
            }),
        }
    }

    fn package_target_for_type(&self, ty: TypeInfo) -> TypeInfo {
        match ty {
            TypeInfo::Record(symbol, args) => {
                if let Some(item) = self.package_items_by_symbol.get(&symbol).copied() {
                    TypeInfo::PackageRecord { symbol, item, args }
                } else {
                    TypeInfo::Record(symbol, args)
                }
            }
            TypeInfo::Enum { symbol, args } => {
                let args = args
                    .into_iter()
                    .map(|arg| self.package_target_for_type(arg))
                    .collect();
                if let Some(item) = self.package_items_by_symbol.get(&symbol).copied() {
                    TypeInfo::PackageEnum { symbol, item, args }
                } else {
                    TypeInfo::Enum { symbol, args }
                }
            }
            TypeInfo::PackageEnum { symbol, item, args } => TypeInfo::PackageEnum {
                symbol,
                item,
                args: args
                    .into_iter()
                    .map(|arg| self.package_target_for_type(arg))
                    .collect(),
            },
            TypeInfo::Function(function) => TypeInfo::Function(FunctionTypeInfo {
                params: function
                    .params
                    .into_iter()
                    .map(|param| self.package_target_for_type(param))
                    .collect(),
                ret: Box::new(self.package_target_for_type(*function.ret)),
            }),
            TypeInfo::List(item) => TypeInfo::List(Box::new(self.package_target_for_type(*item))),
            TypeInfo::Map(key, value) => TypeInfo::Map(
                Box::new(self.package_target_for_type(*key)),
                Box::new(self.package_target_for_type(*value)),
            ),
            TypeInfo::Option(item) => {
                TypeInfo::Option(Box::new(self.package_target_for_type(*item)))
            }
            TypeInfo::Result(ok, err) => TypeInfo::Result(
                Box::new(self.package_target_for_type(*ok)),
                Box::new(self.package_target_for_type(*err)),
            ),
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
