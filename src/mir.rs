//! Execution-shaped intermediate representation consumed by backend lowering.
//!
//! This is the current backend boundary between typed HIR and bytecode. The
//! representation is still expression-shaped, but it is owned by the MIR module
//! so future control-flow-oriented MIR work can evolve without keeping bytecode
//! tied to the compatibility `hir` module.

use std::collections::HashMap;

use crate::{
    ast,
    identity::PackageItemId,
    known_enum,
    package::PackageSymbolGraph,
    span::Span,
    symbol::{Symbol, SymbolTable},
    typed_hir,
    types::TypeInfo,
};

pub type FunctionId = usize;

#[derive(Clone, Debug)]
pub struct Program {
    pub statements: Vec<Stmt>,
    pub functions: Vec<Function>,
    pub symbols: SymbolTable,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub id: FunctionId,
    pub name: Option<Symbol>,
    pub params: Vec<Symbol>,
    pub body: ValueBlock,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Assign(AssignStmt),
    Function(FunctionStmt),
    If(IfStmt),
    While(WhileStmt),
    Expr(ExprStmt),
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Self::Assign(stmt) => stmt.span,
            Self::Function(stmt) => stmt.span,
            Self::If(stmt) => stmt.span,
            Self::While(stmt) => stmt.span,
            Self::Expr(stmt) => stmt.span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AssignStmt {
    pub mutable: bool,
    pub name: Symbol,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FunctionStmt {
    pub name: Symbol,
    pub function: FunctionId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Block,
    pub else_branch: Option<Block>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ExprStmt {
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
pub enum Expr {
    Int(IntExpr),
    Bool(BoolExpr),
    String(StringExpr),
    Ident(IdentExpr),
    ListLit(ListLitExpr),
    Index(IndexExpr),
    RecordLit(RecordLitExpr),
    EnumVariant(EnumVariantExpr),
    Field(FieldExpr),
    RecordUpdate(RecordUpdateExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    If(IfExpr),
    Match(MatchExpr),
    Closure(ClosureExpr),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Self::Int(expr) => expr.span,
            Self::Bool(expr) => expr.span,
            Self::String(expr) => expr.span,
            Self::Ident(expr) => expr.span,
            Self::ListLit(expr) => expr.span,
            Self::Index(expr) => expr.span,
            Self::RecordLit(expr) => expr.span,
            Self::EnumVariant(expr) => expr.span,
            Self::Field(expr) => expr.span,
            Self::RecordUpdate(expr) => expr.span,
            Self::Unary(expr) => expr.span,
            Self::Binary(expr) => expr.span,
            Self::Call(expr) => expr.span,
            Self::If(expr) => expr.span,
            Self::Match(expr) => expr.span,
            Self::Closure(expr) => expr.span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct IntExpr {
    pub value: i64,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct BoolExpr {
    pub value: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StringExpr {
    pub value: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct IdentExpr {
    pub name: Symbol,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ListLitExpr {
    pub items: Vec<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct IndexExpr {
    pub base: Box<Expr>,
    pub index: Box<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RecordLitExpr {
    pub type_name: Symbol,
    pub fields: Vec<RecordFieldInit>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RecordFieldInit {
    pub name: Symbol,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumVariantExpr {
    pub enum_name: Symbol,
    pub variant_name: Symbol,
    pub payload: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FieldExpr {
    pub base: Box<Expr>,
    pub field: Symbol,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RecordUpdateExpr {
    pub base: Box<Expr>,
    pub fields: Vec<RecordFieldInit>,
    pub span: Span,
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
    pub span: Span,
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
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct IfExpr {
    pub condition: Box<Expr>,
    pub then_branch: ValueBlock,
    pub else_branch: ValueBlock,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct MatchExpr {
    pub value: Box<Expr>,
    pub arms: Vec<MatchArm>,
    pub span: Span,
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
    pub enum_name: Symbol,
    pub variant_name: Symbol,
    pub binding: Option<Symbol>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ClosureExpr {
    pub function: FunctionId,
    pub span: Span,
}

pub fn lower(program: &ast::Program) -> Program {
    let mut lowerer = Lowerer {
        functions: Vec::new(),
        symbols: SymbolTable::default(),
        enum_variants: HashMap::new(),
    };
    lowerer.collect_enum_variants(program);
    let statements = program
        .statements
        .iter()
        .filter_map(|statement| lowerer.lower_stmt(statement))
        .collect();
    Program {
        statements,
        functions: lowerer.functions,
        symbols: lowerer.symbols,
    }
}

pub fn lower_typed(program: &typed_hir::Program) -> Program {
    let mut lowerer = TypedLowerer {
        functions: Vec::new(),
        symbols: SymbolTable::default(),
        source_symbols: &program.symbols,
        package_graph: &program.package_graph,
    };
    let statements = program
        .statements
        .iter()
        .filter_map(|statement| lowerer.lower_stmt(statement))
        .collect();
    Program {
        statements,
        functions: lowerer.functions,
        symbols: lowerer.symbols,
    }
}

struct TypedLowerer<'a> {
    functions: Vec<Function>,
    symbols: SymbolTable,
    source_symbols: &'a SymbolTable,
    package_graph: &'a PackageSymbolGraph,
}

impl TypedLowerer<'_> {
    fn lower_stmt(&mut self, statement: &typed_hir::Stmt) -> Option<Stmt> {
        Some(match statement {
            typed_hir::Stmt::Assign(stmt) => Stmt::Assign(AssignStmt {
                mutable: stmt.mutable,
                name: self.symbol(&stmt.name),
                value: self.lower_expr(&stmt.value),
                span: stmt.span,
            }),
            typed_hir::Stmt::Record(_) | typed_hir::Stmt::Enum(_) => return None,
            typed_hir::Stmt::Function(stmt) => Stmt::Function(FunctionStmt {
                name: self.function_name(stmt),
                function: self.lower_function(stmt),
                span: stmt.span,
            }),
            typed_hir::Stmt::If(stmt) => Stmt::If(IfStmt {
                condition: self.lower_expr(&stmt.condition),
                then_branch: self.lower_block(&stmt.then_branch),
                else_branch: stmt
                    .else_branch
                    .as_ref()
                    .map(|branch| self.lower_block(branch)),
                span: stmt.span,
            }),
            typed_hir::Stmt::While(stmt) => Stmt::While(WhileStmt {
                condition: self.lower_expr(&stmt.condition),
                body: self.lower_block(&stmt.body),
                span: stmt.span,
            }),
            typed_hir::Stmt::Expr(stmt) => Stmt::Expr(ExprStmt {
                expr: self.lower_expr(&stmt.expr),
                span: stmt.span,
            }),
        })
    }

    fn lower_block(&mut self, block: &typed_hir::Block) -> Block {
        Block {
            statements: block
                .statements
                .iter()
                .filter_map(|statement| self.lower_stmt(statement))
                .collect(),
            span: block.span,
        }
    }

    fn lower_value_block(&mut self, block: &typed_hir::ValueBlock) -> ValueBlock {
        ValueBlock {
            statements: block
                .statements
                .iter()
                .filter_map(|statement| self.lower_stmt(statement))
                .collect(),
            expr: Box::new(self.lower_expr(&block.expr)),
            span: block.span,
        }
    }

    fn lower_expr(&mut self, expr: &typed_hir::Expr) -> Expr {
        match &expr.kind {
            typed_hir::ExprKind::Int(value) => Expr::Int(IntExpr {
                value: *value,
                span: expr.span,
            }),
            typed_hir::ExprKind::Bool(value) => Expr::Bool(BoolExpr {
                value: *value,
                span: expr.span,
            }),
            typed_hir::ExprKind::String(value) => Expr::String(StringExpr {
                value: value.clone(),
                span: expr.span,
            }),
            typed_hir::ExprKind::Ident(ident) => self.lower_ident_expr(ident, expr.span),
            typed_hir::ExprKind::ListLit(list) => Expr::ListLit(ListLitExpr {
                items: list
                    .items
                    .iter()
                    .map(|item| self.lower_expr(item))
                    .collect(),
                span: expr.span,
            }),
            typed_hir::ExprKind::Index(index) => Expr::Index(IndexExpr {
                base: Box::new(self.lower_expr(&index.base)),
                index: Box::new(self.lower_expr(&index.index)),
                span: expr.span,
            }),
            typed_hir::ExprKind::RecordLit(record) => Expr::RecordLit(RecordLitExpr {
                type_name: self.record_type_name(expr, &record.type_name),
                fields: record
                    .fields
                    .iter()
                    .map(|field| RecordFieldInit {
                        name: self.symbol(&field.name),
                        value: self.lower_expr(&field.value),
                        span: field.span,
                    })
                    .collect(),
                span: expr.span,
            }),
            typed_hir::ExprKind::Field(field) => Expr::Field(FieldExpr {
                base: Box::new(self.lower_expr(&field.base)),
                field: self.symbol(&field.field),
                span: expr.span,
            }),
            typed_hir::ExprKind::RecordUpdate(update) => Expr::RecordUpdate(RecordUpdateExpr {
                base: Box::new(self.lower_expr(&update.base)),
                fields: update
                    .fields
                    .iter()
                    .map(|field| RecordFieldInit {
                        name: self.symbol(&field.name),
                        value: self.lower_expr(&field.value),
                        span: field.span,
                    })
                    .collect(),
                span: expr.span,
            }),
            typed_hir::ExprKind::Unary(unary) => Expr::Unary(UnaryExpr {
                op: match unary.op {
                    typed_hir::UnaryOp::Neg => UnaryOp::Neg,
                    typed_hir::UnaryOp::Not => UnaryOp::Not,
                },
                expr: Box::new(self.lower_expr(&unary.expr)),
                span: expr.span,
            }),
            typed_hir::ExprKind::Binary(binary) => Expr::Binary(BinaryExpr {
                op: match binary.op {
                    typed_hir::BinaryOp::Add => BinaryOp::Add,
                    typed_hir::BinaryOp::Sub => BinaryOp::Sub,
                    typed_hir::BinaryOp::Mul => BinaryOp::Mul,
                    typed_hir::BinaryOp::Div => BinaryOp::Div,
                    typed_hir::BinaryOp::Lt => BinaryOp::Lt,
                    typed_hir::BinaryOp::LtEq => BinaryOp::LtEq,
                    typed_hir::BinaryOp::Gt => BinaryOp::Gt,
                    typed_hir::BinaryOp::GtEq => BinaryOp::GtEq,
                    typed_hir::BinaryOp::EqEq => BinaryOp::EqEq,
                    typed_hir::BinaryOp::BangEq => BinaryOp::BangEq,
                },
                left: Box::new(self.lower_expr(&binary.left)),
                right: Box::new(self.lower_expr(&binary.right)),
                span: expr.span,
            }),
            typed_hir::ExprKind::Call(call) => self.lower_call_expr(call, expr.span),
            typed_hir::ExprKind::If(if_expr) => Expr::If(IfExpr {
                condition: Box::new(self.lower_expr(&if_expr.condition)),
                then_branch: self.lower_value_block(&if_expr.then_branch),
                else_branch: self.lower_value_block(&if_expr.else_branch),
                span: expr.span,
            }),
            typed_hir::ExprKind::Match(match_expr) => self.lower_match_expr(match_expr, expr.span),
            typed_hir::ExprKind::Fn(function) => Expr::Closure(ClosureExpr {
                function: self.lower_fn_expr(function, expr.span),
                span: expr.span,
            }),
        }
    }

    fn lower_ident_expr(&mut self, expr: &typed_hir::IdentExpr, span: Span) -> Expr {
        match expr.target {
            typed_hir::IdentTarget::EnumVariant {
                enum_name,
                enum_item,
                variant_name,
                ..
            } => Expr::EnumVariant(EnumVariantExpr {
                enum_name: self.enum_type_name(enum_item, enum_name),
                variant_name: self.source_symbol(variant_name),
                payload: None,
                span,
            }),
            typed_hir::IdentTarget::PackageItem { item, .. } => Expr::Ident(IdentExpr {
                name: self.package_item_symbol(item, &expr.name),
                span,
            }),
            typed_hir::IdentTarget::Binding(_) => Expr::Ident(IdentExpr {
                name: self.symbol(&expr.name),
                span,
            }),
        }
    }

    fn lower_call_expr(&mut self, expr: &typed_hir::CallExpr, span: Span) -> Expr {
        if let crate::typing::TypedCalleeInfo::EnumVariant {
            enum_name,
            enum_item,
            variant_name,
            ..
        } = expr.resolved_callee
        {
            let payload = expr.args.first().map(|arg| Box::new(self.lower_expr(arg)));
            return Expr::EnumVariant(EnumVariantExpr {
                enum_name: self.enum_type_name(enum_item, enum_name),
                variant_name: self.source_symbol(variant_name),
                payload,
                span,
            });
        }
        Expr::Call(CallExpr {
            callee: Box::new(self.lower_expr(&expr.callee)),
            args: expr.args.iter().map(|arg| self.lower_expr(arg)).collect(),
            span,
        })
    }

    fn lower_match_expr(&mut self, expr: &typed_hir::MatchExpr, span: Span) -> Expr {
        let fallback_enum = expr
            .arms
            .first()
            .map(|arm| {
                let typed_hir::MatchPattern::Variant(pattern) = &arm.pattern;
                pattern.enum_name.as_str()
            })
            .unwrap_or("");
        let enum_name = self.match_enum_name(&expr.value, fallback_enum);
        Expr::Match(MatchExpr {
            value: Box::new(self.lower_expr(&expr.value)),
            arms: expr
                .arms
                .iter()
                .map(|arm| self.lower_match_arm(arm, enum_name))
                .collect(),
            span,
        })
    }

    fn lower_match_arm(&mut self, arm: &typed_hir::MatchArm, enum_name: Symbol) -> MatchArm {
        let typed_hir::MatchPattern::Variant(pattern) = &arm.pattern;
        MatchArm {
            pattern: MatchPattern::Variant(EnumVariantPattern {
                enum_name,
                variant_name: self.symbol(&pattern.variant_name),
                binding: pattern
                    .binding_name
                    .as_ref()
                    .map(|binding| self.symbol(binding)),
                span: pattern.span,
            }),
            value: self.lower_expr(&arm.value),
            span: arm.span,
        }
    }

    fn lower_function(&mut self, stmt: &typed_hir::FunctionStmt) -> FunctionId {
        let id = self.functions.len();
        let name = self.function_name(stmt);
        let params = stmt
            .params
            .iter()
            .map(|param| self.symbol(&param.name))
            .collect();
        self.functions.push(Function {
            id,
            name: Some(name),
            params,
            body: placeholder_value_block(stmt.span),
            span: stmt.span,
        });
        let body = self.lower_value_block(&stmt.body);
        self.functions[id].body = body;
        id
    }

    fn lower_fn_expr(&mut self, expr: &typed_hir::FnExpr, span: Span) -> FunctionId {
        let id = self.functions.len();
        let params = expr
            .params
            .iter()
            .map(|param| self.symbol(&param.name))
            .collect();
        self.functions.push(Function {
            id,
            name: None,
            params,
            body: placeholder_value_block(span),
            span,
        });
        let body = self.lower_value_block(&expr.body);
        self.functions[id].body = body;
        id
    }

    fn function_name(&mut self, function: &typed_hir::FunctionStmt) -> Symbol {
        if let Some(item) = function.package_item {
            self.package_item_symbol(item, &function.name)
        } else {
            self.symbol(&function.name)
        }
    }

    fn record_type_name(&mut self, expr: &typed_hir::Expr, fallback: &str) -> Symbol {
        match &expr.ty {
            TypeInfo::PackageRecord { item, .. } => self.package_item_symbol(*item, fallback),
            TypeInfo::Record(symbol) => self.source_symbol(*symbol),
            _ => self.symbol(fallback),
        }
    }

    fn enum_type_name(&mut self, item: Option<PackageItemId>, fallback: Symbol) -> Symbol {
        let fallback_name = self.source_symbols.resolve(fallback).to_string();
        if let Some(item) = item {
            self.package_item_symbol(item, &fallback_name)
        } else {
            self.symbol(&fallback_name)
        }
    }

    fn match_enum_name(&mut self, value: &typed_hir::Expr, fallback: &str) -> Symbol {
        match &value.ty {
            TypeInfo::PackageEnum { item, .. } => self.package_item_symbol(*item, fallback),
            TypeInfo::Enum { symbol, .. } => self.source_symbol(*symbol),
            TypeInfo::Option(_) => self.symbol(known_enum::OPTION_NAME),
            TypeInfo::Result(_, _) => self.symbol(known_enum::RESULT_NAME),
            _ => self.symbol(fallback),
        }
    }

    fn package_item_symbol(&mut self, item: PackageItemId, fallback: &str) -> Symbol {
        let name = self
            .package_graph
            .item(item)
            .map(|item| item.mangled_name.clone())
            .unwrap_or_else(|| fallback.to_string());
        self.symbol(&name)
    }

    fn source_symbol(&mut self, symbol: Symbol) -> Symbol {
        let name = self.source_symbols.resolve(symbol).to_string();
        self.symbol(&name)
    }

    fn symbol(&mut self, name: &str) -> Symbol {
        self.symbols.intern(name)
    }
}

struct Lowerer {
    functions: Vec<Function>,
    symbols: SymbolTable,
    enum_variants: HashMap<String, EnumVariantLowering>,
}

#[derive(Clone, Copy)]
struct EnumVariantLowering {
    has_payload: bool,
}

impl Lowerer {
    fn collect_enum_variants(&mut self, program: &ast::Program) {
        for known in [known_enum::option_enum(), known_enum::result_enum()] {
            for variant in known.variants {
                self.enum_variants.insert(
                    known.qualified_variant(*variant),
                    EnumVariantLowering {
                        has_payload: variant.has_payload,
                    },
                );
            }
        }

        for statement in &program.statements {
            if let ast::Stmt::EnumDecl(enumeration) = statement {
                for variant in &enumeration.variants {
                    self.enum_variants.insert(
                        format!("{}::{}", enumeration.name, variant.name),
                        EnumVariantLowering {
                            has_payload: variant.payload.is_some(),
                        },
                    );
                }
            }
        }
    }

    fn lower_stmt(&mut self, statement: &ast::Stmt) -> Option<Stmt> {
        Some(match statement {
            ast::Stmt::Assign(stmt) => Stmt::Assign(AssignStmt {
                mutable: stmt.mutable,
                name: self.symbol(&stmt.name),
                value: self.lower_expr(&stmt.value),
                span: stmt.span,
            }),
            ast::Stmt::RecordDecl(_) => return None,
            ast::Stmt::EnumDecl(_) => return None,
            ast::Stmt::FuncDecl(stmt) => Stmt::Function(FunctionStmt {
                name: self.symbol(&stmt.name),
                function: self.lower_function_decl(stmt),
                span: stmt.span,
            }),
            ast::Stmt::If(stmt) => Stmt::If(IfStmt {
                condition: self.lower_expr(&stmt.condition),
                then_branch: self.lower_block(&stmt.then_branch),
                else_branch: stmt
                    .else_branch
                    .as_ref()
                    .map(|branch| self.lower_block(branch)),
                span: stmt.span,
            }),
            ast::Stmt::While(stmt) => Stmt::While(WhileStmt {
                condition: self.lower_expr(&stmt.condition),
                body: self.lower_block(&stmt.body),
                span: stmt.span,
            }),
            ast::Stmt::Expr(stmt) => Stmt::Expr(ExprStmt {
                expr: self.lower_expr(&stmt.expr),
                span: stmt.span,
            }),
        })
    }

    fn lower_block(&mut self, block: &ast::Block) -> Block {
        Block {
            statements: block
                .statements
                .iter()
                .filter_map(|statement| self.lower_stmt(statement))
                .collect(),
            span: block.span,
        }
    }

    fn lower_value_block(&mut self, block: &ast::ValueBlock) -> ValueBlock {
        ValueBlock {
            statements: block
                .statements
                .iter()
                .filter_map(|statement| self.lower_stmt(statement))
                .collect(),
            expr: Box::new(self.lower_expr(&block.expr)),
            span: block.span,
        }
    }

    fn lower_expr(&mut self, expr: &ast::Expr) -> Expr {
        match expr {
            ast::Expr::Int(expr) => Expr::Int(IntExpr {
                value: expr.value,
                span: expr.span,
            }),
            ast::Expr::Bool(expr) => Expr::Bool(BoolExpr {
                value: expr.value,
                span: expr.span,
            }),
            ast::Expr::String(expr) => Expr::String(StringExpr {
                value: expr.value.clone(),
                span: expr.span,
            }),
            ast::Expr::Ident(expr) => {
                if self
                    .enum_variants
                    .get(&expr.name)
                    .is_some_and(|variant| !variant.has_payload)
                {
                    let (enum_name, variant_name) =
                        split_variant_name(&expr.name).unwrap_or((&expr.name, ""));
                    Expr::EnumVariant(EnumVariantExpr {
                        enum_name: self.symbol(enum_name),
                        variant_name: self.symbol(variant_name),
                        payload: None,
                        span: expr.span,
                    })
                } else {
                    Expr::Ident(IdentExpr {
                        name: self.symbol(&expr.name),
                        span: expr.span,
                    })
                }
            }
            ast::Expr::ListLit(expr) => Expr::ListLit(ListLitExpr {
                items: expr
                    .items
                    .iter()
                    .map(|item| self.lower_expr(item))
                    .collect(),
                span: expr.span,
            }),
            ast::Expr::Index(expr) => Expr::Index(IndexExpr {
                base: Box::new(self.lower_expr(&expr.base)),
                index: Box::new(self.lower_expr(&expr.index)),
                span: expr.span,
            }),
            ast::Expr::RecordLit(expr) => Expr::RecordLit(RecordLitExpr {
                type_name: self.symbol(&expr.type_name),
                fields: expr
                    .fields
                    .iter()
                    .map(|field| RecordFieldInit {
                        name: self.symbol(&field.name),
                        value: self.lower_expr(&field.value),
                        span: field.span,
                    })
                    .collect(),
                span: expr.span,
            }),
            ast::Expr::Field(expr) => Expr::Field(FieldExpr {
                base: Box::new(self.lower_expr(&expr.base)),
                field: self.symbol(&expr.field),
                span: expr.span,
            }),
            ast::Expr::RecordUpdate(expr) => Expr::RecordUpdate(RecordUpdateExpr {
                base: Box::new(self.lower_expr(&expr.base)),
                fields: expr
                    .fields
                    .iter()
                    .map(|field| RecordFieldInit {
                        name: self.symbol(&field.name),
                        value: self.lower_expr(&field.value),
                        span: field.span,
                    })
                    .collect(),
                span: expr.span,
            }),
            ast::Expr::Unary(expr) => Expr::Unary(UnaryExpr {
                op: match expr.op {
                    ast::UnaryOp::Neg => UnaryOp::Neg,
                    ast::UnaryOp::Not => UnaryOp::Not,
                },
                expr: Box::new(self.lower_expr(&expr.expr)),
                span: expr.span,
            }),
            ast::Expr::Binary(expr) => Expr::Binary(BinaryExpr {
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
                span: expr.span,
            }),
            ast::Expr::Call(expr) => {
                if let ast::Expr::Ident(callee) = expr.callee.as_ref() {
                    if self
                        .enum_variants
                        .get(&callee.name)
                        .is_some_and(|variant| variant.has_payload)
                        && expr.args.len() == 1
                    {
                        let (enum_name, variant_name) =
                            split_variant_name(&callee.name).unwrap_or((&callee.name, ""));
                        return Expr::EnumVariant(EnumVariantExpr {
                            enum_name: self.symbol(enum_name),
                            variant_name: self.symbol(variant_name),
                            payload: Some(Box::new(self.lower_expr(&expr.args[0]))),
                            span: expr.span,
                        });
                    }
                }
                Expr::Call(CallExpr {
                    callee: Box::new(self.lower_expr(&expr.callee)),
                    args: expr.args.iter().map(|arg| self.lower_expr(arg)).collect(),
                    span: expr.span,
                })
            }
            ast::Expr::If(expr) => Expr::If(IfExpr {
                condition: Box::new(self.lower_expr(&expr.condition)),
                then_branch: self.lower_value_block(&expr.then_branch),
                else_branch: self.lower_value_block(&expr.else_branch),
                span: expr.span,
            }),
            ast::Expr::Match(expr) => Expr::Match(MatchExpr {
                value: Box::new(self.lower_expr(&expr.value)),
                arms: expr
                    .arms
                    .iter()
                    .map(|arm| MatchArm {
                        pattern: match &arm.pattern {
                            ast::MatchPattern::Variant(pattern) => {
                                MatchPattern::Variant(EnumVariantPattern {
                                    enum_name: self.symbol(&pattern.enum_name),
                                    variant_name: self.symbol(&pattern.variant_name),
                                    binding: pattern
                                        .binding
                                        .as_ref()
                                        .map(|binding| self.symbol(binding)),
                                    span: pattern.span,
                                })
                            }
                        },
                        value: self.lower_expr(&arm.value),
                        span: arm.span,
                    })
                    .collect(),
                span: expr.span,
            }),
            ast::Expr::Fn(expr) => Expr::Closure(ClosureExpr {
                function: self.lower_fn_expr(expr),
                span: expr.span,
            }),
        }
    }

    fn lower_function_decl(&mut self, stmt: &ast::FuncDecl) -> FunctionId {
        let id = self.functions.len();
        let name = self.symbol(&stmt.name);
        let params = stmt
            .params
            .iter()
            .map(|param| self.symbol(&param.name))
            .collect();
        self.functions.push(Function {
            id,
            name: Some(name),
            params,
            body: placeholder_value_block(stmt.span),
            span: stmt.span,
        });
        let body = self.lower_value_block(&stmt.body);
        self.functions[id].body = body;
        id
    }

    fn lower_fn_expr(&mut self, expr: &ast::FnExpr) -> FunctionId {
        let id = self.functions.len();
        let params = expr
            .params
            .iter()
            .map(|param| self.symbol(&param.name))
            .collect();
        self.functions.push(Function {
            id,
            name: None,
            params,
            body: placeholder_value_block(expr.span),
            span: expr.span,
        });
        let body = self.lower_value_block(&expr.body);
        self.functions[id].body = body;
        id
    }

    fn symbol(&mut self, name: &str) -> Symbol {
        self.symbols.intern(name)
    }
}

fn placeholder_value_block(span: Span) -> ValueBlock {
    ValueBlock {
        statements: Vec::new(),
        expr: Box::new(Expr::Int(IntExpr { value: 0, span })),
        span,
    }
}

fn split_variant_name(name: &str) -> Option<(&str, &str)> {
    name.rsplit_once("::")
}
