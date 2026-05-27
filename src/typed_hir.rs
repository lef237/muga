use std::collections::{HashMap, HashSet};

use crate::{
    ast,
    cli_schema::{CliSchema, CliValueSource},
    identity::{BindingId, BindingKind, ExprId, PackageItemId, StmtId},
    json_decode::{JsonDecodeSchema, JsonDecodeValidationRule},
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
    OpaqueType(OpaqueTypeStmt),
    Function(FunctionStmt),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Using(UsingStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Return(ReturnStmt),
    Expr(ExprStmt),
}

impl Stmt {
    pub fn id(&self) -> StmtId {
        match self {
            Self::Assign(stmt) => stmt.id,
            Self::Record(stmt) => stmt.id,
            Self::Enum(stmt) => stmt.id,
            Self::OpaqueType(stmt) => stmt.id,
            Self::Function(stmt) => stmt.id,
            Self::If(stmt) => stmt.id,
            Self::While(stmt) => stmt.id,
            Self::For(stmt) => stmt.id,
            Self::Using(stmt) => stmt.id,
            Self::Break(stmt) => stmt.id,
            Self::Continue(stmt) => stmt.id,
            Self::Return(stmt) => stmt.id,
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
    pub doc_comments: Vec<String>,
    pub type_params: Vec<String>,
    pub json_deny_unknown_fields: bool,
    pub cli_about: Option<String>,
    pub fields: Vec<RecordField>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RecordField {
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
pub struct EnumStmt {
    pub id: StmtId,
    pub name: String,
    pub package_item: Option<PackageItemId>,
    pub doc_comments: Vec<String>,
    pub type_params: Vec<String>,
    pub cli_about: Option<String>,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct EnumVariant {
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
pub struct OpaqueTypeStmt {
    pub id: StmtId,
    pub name: String,
    pub package_item: Option<PackageItemId>,
    pub doc_comments: Vec<String>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FunctionStmt {
    pub id: StmtId,
    pub name: String,
    pub binding: BindingId,
    pub package_item: Option<PackageItemId>,
    pub doc_comments: Vec<String>,
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
pub struct ForStmt {
    pub id: StmtId,
    pub item: String,
    pub item_binding: BindingId,
    pub iterable: Expr,
    pub body: Block,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct UsingStmt {
    pub id: StmtId,
    pub name: String,
    pub binding: BindingId,
    pub value: Expr,
    pub body: Block,
    pub cleanup: UsingCleanup,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct UsingCleanup {
    pub name: String,
    pub target: IdentTarget,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct BreakStmt {
    pub id: StmtId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ContinueStmt {
    pub id: StmtId,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ReturnStmt {
    pub id: StmtId,
    pub value: Expr,
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
    pub terminal_return: bool,
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
    Unit,
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
    And,
    Or,
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
    pub json_decode_schema: Option<JsonDecodeSchema>,
    pub json_required_decode_schema: Option<JsonDecodeSchema>,
    pub json_to_value_schema: Option<JsonDecodeSchema>,
    pub json_encode_typed_schema: Option<JsonDecodeSchema>,
    pub config_required_load_json_schema: Option<Box<JsonDecodeSchema>>,
    pub config_load_json_schema: Option<JsonDecodeSchema>,
    pub cli_parse_schema: Option<Box<CliSchema>>,
    pub cli_parse_or_schema: Option<Box<CliSchema>>,
    pub cli_parse_request_schema: Option<Box<CliSchema>>,
    pub cli_parse_request_or_schema: Option<Box<CliSchema>>,
    pub cli_usage_for_schema: Option<Box<CliSchema>>,
    pub cli_usage_for_required_schema: Option<Box<CliSchema>>,
    pub cli_help_for_schema: Option<Box<CliSchema>>,
    pub cli_help_for_required_schema: Option<Box<CliSchema>>,
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
    pub payload: EnumVariantPatternPayload,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum EnumVariantPatternPayload {
    None,
    Binding { name: String, binding: BindingId },
    Discard,
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
                doc_comments: stmt.doc_comments.clone(),
                type_params: stmt.type_params.clone(),
                json_deny_unknown_fields: stmt.json_deny_unknown_fields,
                cli_about: stmt.cli_about.clone(),
                fields: stmt
                    .fields
                    .iter()
                    .map(|field| RecordField {
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
                doc_comments: stmt.doc_comments.clone(),
                type_params: stmt.type_params.clone(),
                cli_about: stmt.cli_about.clone(),
                variants: stmt
                    .variants
                    .iter()
                    .map(|variant| EnumVariant {
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
                            .map(|payload| self.type_info(payload)),
                        span: variant.span,
                    })
                    .collect(),
                span: stmt.span,
            }),
            Stmt::OpaqueType(stmt) => Stmt::OpaqueType(OpaqueTypeStmt {
                id: self.stmt_id(stmt.id),
                name: stmt.name.clone(),
                package_item: stmt.package_item,
                doc_comments: stmt.doc_comments.clone(),
                span: stmt.span,
            }),
            Stmt::Function(stmt) => Stmt::Function(FunctionStmt {
                id: self.stmt_id(stmt.id),
                name: stmt.name.clone(),
                binding: self.binding(stmt.binding),
                package_item: stmt.package_item,
                doc_comments: stmt.doc_comments.clone(),
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
            Stmt::For(stmt) => Stmt::For(ForStmt {
                id: self.stmt_id(stmt.id),
                item: stmt.item.clone(),
                item_binding: self.binding(stmt.item_binding),
                iterable: self.expr(&stmt.iterable),
                body: self.block(&stmt.body),
                span: stmt.span,
            }),
            Stmt::Using(stmt) => Stmt::Using(UsingStmt {
                id: self.stmt_id(stmt.id),
                name: stmt.name.clone(),
                binding: self.binding(stmt.binding),
                value: self.expr(&stmt.value),
                body: self.block(&stmt.body),
                cleanup: UsingCleanup {
                    name: stmt.cleanup.name.clone(),
                    target: self.ident_target(stmt.cleanup.target),
                    span: stmt.cleanup.span,
                },
                span: stmt.span,
            }),
            Stmt::Break(stmt) => Stmt::Break(BreakStmt {
                id: self.stmt_id(stmt.id),
                span: stmt.span,
            }),
            Stmt::Continue(stmt) => Stmt::Continue(ContinueStmt {
                id: self.stmt_id(stmt.id),
                span: stmt.span,
            }),
            Stmt::Return(stmt) => Stmt::Return(ReturnStmt {
                id: self.stmt_id(stmt.id),
                value: self.expr(&stmt.value),
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
            terminal_return: block.terminal_return,
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
                ExprKind::Unit => ExprKind::Unit,
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
                    json_decode_schema: expr
                        .json_decode_schema
                        .as_ref()
                        .map(|schema| self.json_decode_schema(schema)),
                    json_required_decode_schema: expr
                        .json_required_decode_schema
                        .as_ref()
                        .map(|schema| self.json_decode_schema(schema)),
                    json_to_value_schema: expr
                        .json_to_value_schema
                        .as_ref()
                        .map(|schema| self.json_decode_schema(schema)),
                    json_encode_typed_schema: expr
                        .json_encode_typed_schema
                        .as_ref()
                        .map(|schema| self.json_decode_schema(schema)),
                    config_required_load_json_schema: expr
                        .config_required_load_json_schema
                        .as_ref()
                        .map(|schema| Box::new(self.json_decode_schema(schema))),
                    config_load_json_schema: expr
                        .config_load_json_schema
                        .as_ref()
                        .map(|schema| self.json_decode_schema(schema)),
                    cli_parse_schema: expr
                        .cli_parse_schema
                        .as_ref()
                        .map(|schema| Box::new(self.cli_schema(schema))),
                    cli_parse_or_schema: expr
                        .cli_parse_or_schema
                        .as_ref()
                        .map(|schema| Box::new(self.cli_schema(schema))),
                    cli_parse_request_schema: expr
                        .cli_parse_request_schema
                        .as_ref()
                        .map(|schema| Box::new(self.cli_schema(schema))),
                    cli_parse_request_or_schema: expr
                        .cli_parse_request_or_schema
                        .as_ref()
                        .map(|schema| Box::new(self.cli_schema(schema))),
                    cli_usage_for_schema: expr
                        .cli_usage_for_schema
                        .as_ref()
                        .map(|schema| Box::new(self.cli_schema(schema))),
                    cli_usage_for_required_schema: expr
                        .cli_usage_for_required_schema
                        .as_ref()
                        .map(|schema| Box::new(self.cli_schema(schema))),
                    cli_help_for_schema: expr
                        .cli_help_for_schema
                        .as_ref()
                        .map(|schema| Box::new(self.cli_schema(schema))),
                    cli_help_for_required_schema: expr
                        .cli_help_for_required_schema
                        .as_ref()
                        .map(|schema| Box::new(self.cli_schema(schema))),
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
                payload: self.match_pattern_payload(&pattern.payload),
                span: pattern.span,
            }),
        }
    }

    fn match_pattern_payload(
        &mut self,
        payload: &EnumVariantPatternPayload,
    ) -> EnumVariantPatternPayload {
        match payload {
            EnumVariantPatternPayload::None => EnumVariantPatternPayload::None,
            EnumVariantPatternPayload::Binding { name, binding } => {
                EnumVariantPatternPayload::Binding {
                    name: name.clone(),
                    binding: self.binding(*binding),
                }
            }
            EnumVariantPatternPayload::Discard => EnumVariantPatternPayload::Discard,
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

    fn json_decode_schema(&mut self, schema: &JsonDecodeSchema) -> JsonDecodeSchema {
        schema.map_symbols(&mut |symbol| self.symbol(symbol))
    }

    fn cli_schema(&mut self, schema: &CliSchema) -> CliSchema {
        schema.map_symbols(&mut |symbol| self.symbol(symbol))
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
            TypeInfo::PackageOpaque { symbol, item } => TypeInfo::PackageOpaque {
                symbol: self.symbol(*symbol),
                item: *item,
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
            | TypeInfo::Unit
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
        Stmt::Record(_) | Stmt::Enum(_) | Stmt::OpaqueType(_) => {}
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
        Stmt::For(stmt) => {
            max = max_opt(max, Some(stmt.item_binding.as_u32()));
            max = max_opt(max, max_binding_id_in_expr(&stmt.iterable));
            max = max_opt(max, max_binding_id_in_block(&stmt.body));
        }
        Stmt::Using(stmt) => {
            max = max_opt(max, Some(stmt.binding.as_u32()));
            max = max_opt(max, max_binding_id_in_expr(&stmt.value));
            max = max_opt(max, max_binding_id_in_block(&stmt.body));
            max = max_opt(max, max_binding_id_in_ident_target(stmt.cleanup.target));
        }
        Stmt::Break(_) | Stmt::Continue(_) => {}
        Stmt::Return(stmt) => {
            max = max_opt(max, max_binding_id_in_expr(&stmt.value));
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
        ExprKind::Int(_) | ExprKind::Bool(_) | ExprKind::String(_) | ExprKind::Unit => {}
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
                if let EnumVariantPatternPayload::Binding { binding, .. } = &pattern.payload {
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
        Stmt::For(stmt) => max = max_opt(max, max_stmt_id_in_block(&stmt.body)),
        Stmt::Using(stmt) => max = max_opt(max, max_stmt_id_in_block(&stmt.body)),
        Stmt::Assign(_)
        | Stmt::Record(_)
        | Stmt::Enum(_)
        | Stmt::OpaqueType(_)
        | Stmt::Break(_)
        | Stmt::Continue(_)
        | Stmt::Return(_)
        | Stmt::Expr(_) => {}
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
        Stmt::Record(_) | Stmt::Enum(_) | Stmt::OpaqueType(_) => None,
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
        Stmt::For(stmt) => max_opt(
            max_expr_id_in_expr(&stmt.iterable),
            max_expr_id_in_block(&stmt.body),
        ),
        Stmt::Using(stmt) => max_opt(
            max_expr_id_in_expr(&stmt.value),
            max_expr_id_in_block(&stmt.body),
        ),
        Stmt::Break(_) | Stmt::Continue(_) => None,
        Stmt::Return(stmt) => max_expr_id_in_expr(&stmt.value),
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
        ExprKind::Int(_)
        | ExprKind::Bool(_)
        | ExprKind::String(_)
        | ExprKind::Unit
        | ExprKind::Ident(_) => {}
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
    json_decode_schemas: HashMap<ExprId, JsonDecodeSchema>,
    json_required_decode_schemas: HashMap<ExprId, JsonDecodeSchema>,
    json_to_value_schemas: HashMap<ExprId, JsonDecodeSchema>,
    json_encode_typed_schemas: HashMap<ExprId, JsonDecodeSchema>,
    config_load_json_schemas: HashMap<ExprId, JsonDecodeSchema>,
    config_required_load_json_schemas: HashMap<ExprId, JsonDecodeSchema>,
    cli_parse_schemas: HashMap<ExprId, CliSchema>,
    cli_parse_or_schemas: HashMap<ExprId, CliSchema>,
    cli_parse_request_schemas: HashMap<ExprId, CliSchema>,
    cli_parse_request_or_schemas: HashMap<ExprId, CliSchema>,
    cli_usage_for_schemas: HashMap<ExprId, CliSchema>,
    cli_usage_for_required_schemas: HashMap<ExprId, CliSchema>,
    cli_help_for_schemas: HashMap<ExprId, CliSchema>,
    cli_help_for_required_schemas: HashMap<ExprId, CliSchema>,
    package_items_by_binding: HashMap<BindingId, PackageItemId>,
    package_items_by_symbol: HashMap<Symbol, PackageItemId>,
    package_opaque_items_by_symbol: HashMap<Symbol, PackageItemId>,
    enum_symbols: HashSet<Symbol>,
    opaque_symbols: HashSet<Symbol>,
    assignment_targets: HashMap<StmtId, TypedAssignmentTarget>,
    using_cleanups: HashMap<StmtId, crate::typing::TypedUsingCleanupInfo>,
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
        let mut package_opaque_items_by_symbol: HashMap<_, _> =
            analysis.package_opaque_types.iter().copied().collect();
        package_opaque_items_by_symbol.extend(program.statements.iter().filter_map(|statement| {
            match statement {
                ast::Stmt::OpaqueTypeDecl(opaque) => {
                    let item = opaque.package_item?;
                    let symbol = analysis.symbols.lookup(&opaque.name)?;
                    Some((symbol, item))
                }
                _ => None,
            }
        }));
        let enum_symbols = program
            .statements
            .iter()
            .filter_map(|statement| match statement {
                ast::Stmt::EnumDecl(enumeration) => analysis.symbols.lookup(&enumeration.name),
                _ => None,
            })
            .collect();
        let mut opaque_symbols: HashSet<_> = analysis
            .package_opaque_types
            .iter()
            .map(|(symbol, _)| *symbol)
            .collect();
        opaque_symbols.extend(
            program
                .statements
                .iter()
                .filter_map(|statement| match statement {
                    ast::Stmt::OpaqueTypeDecl(opaque) => analysis.symbols.lookup(&opaque.name),
                    _ => None,
                }),
        );
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
            json_decode_schemas: analysis
                .json_decode_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            json_required_decode_schemas: analysis
                .json_required_decode_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            json_to_value_schemas: analysis
                .json_to_value_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            json_encode_typed_schemas: analysis
                .json_encode_typed_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            config_load_json_schemas: analysis
                .config_load_json_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            config_required_load_json_schemas: analysis
                .config_required_load_json_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            cli_parse_schemas: analysis
                .cli_parse_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            cli_parse_or_schemas: analysis
                .cli_parse_or_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            cli_parse_request_schemas: analysis
                .cli_parse_request_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            cli_parse_request_or_schemas: analysis
                .cli_parse_request_or_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            cli_usage_for_schemas: analysis
                .cli_usage_for_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            cli_usage_for_required_schemas: analysis
                .cli_usage_for_required_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            cli_help_for_schemas: analysis
                .cli_help_for_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            cli_help_for_required_schemas: analysis
                .cli_help_for_required_schemas
                .iter()
                .map(|schema| (schema.expr_id, schema.schema.clone()))
                .collect(),
            package_items_by_binding,
            package_items_by_symbol,
            package_opaque_items_by_symbol,
            enum_symbols,
            opaque_symbols,
            assignment_targets: analysis
                .assignment_targets
                .iter()
                .map(|target| (target.stmt_id, *target))
                .collect(),
            using_cleanups: analysis
                .using_cleanups
                .iter()
                .map(|cleanup| (cleanup.stmt_id, *cleanup))
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
                doc_comments: stmt.doc_comments.clone(),
                type_params: stmt.type_params.clone(),
                json_deny_unknown_fields: json_deny_unknown_fields_from_attributes(
                    &stmt.attributes,
                ),
                cli_about: cli_about_from_attributes(&stmt.attributes),
                fields: stmt
                    .fields
                    .iter()
                    .map(|field| RecordField {
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
                doc_comments: stmt.doc_comments.clone(),
                type_params: stmt.type_params.clone(),
                cli_about: cli_about_from_attributes(&stmt.attributes),
                variants: stmt
                    .variants
                    .iter()
                    .map(|variant| EnumVariant {
                        name: variant.name.clone(),
                        json_rename: json_rename_from_attributes(&variant.attributes),
                        json_aliases: json_aliases_from_attributes(&variant.attributes),
                        cli_name: cli_name_from_attributes(&variant.attributes),
                        cli_aliases: cli_aliases_from_attributes(&variant.attributes),
                        cli_about: cli_about_from_attributes(&variant.attributes),
                        cli_hidden: cli_hidden_from_attributes(&variant.attributes),
                        payload: variant.payload.as_ref().map(|payload| {
                            self.type_info_from_type_expr_with_params(payload, &stmt.type_params)
                        }),
                        span: variant.span,
                    })
                    .collect(),
                span: stmt.span,
            }),
            ast::Stmt::OpaqueTypeDecl(stmt) => Stmt::OpaqueType(OpaqueTypeStmt {
                id: stmt.id,
                name: stmt.name.clone(),
                package_item: stmt.package_item,
                doc_comments: stmt.doc_comments.clone(),
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
                    doc_comments: stmt.doc_comments.clone(),
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
            ast::Stmt::For(stmt) => Stmt::For(ForStmt {
                id: stmt.id,
                item: stmt.item.clone(),
                item_binding: self.binding_for_decl(
                    &stmt.item,
                    stmt.item_span,
                    BindingKind::Immutable,
                ),
                iterable: self.lower_expr(&stmt.iterable),
                body: self.lower_block(&stmt.body),
                span: stmt.span,
            }),
            ast::Stmt::Using(stmt) => {
                let cleanup = self.using_cleanup(stmt.id);
                Stmt::Using(UsingStmt {
                    id: stmt.id,
                    name: stmt.name.clone(),
                    binding: self.binding_for_decl(
                        &stmt.name,
                        stmt.name_span,
                        BindingKind::Immutable,
                    ),
                    value: self.lower_expr(&stmt.value),
                    body: self.lower_block(&stmt.body),
                    cleanup,
                    span: stmt.span,
                })
            }
            ast::Stmt::Break(stmt) => Stmt::Break(BreakStmt {
                id: stmt.id,
                span: stmt.span,
            }),
            ast::Stmt::Continue(stmt) => Stmt::Continue(ContinueStmt {
                id: stmt.id,
                span: stmt.span,
            }),
            ast::Stmt::Return(stmt) => Stmt::Return(ReturnStmt {
                id: stmt.id,
                value: self.lower_expr(&stmt.value),
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
            terminal_return: block.terminal_return,
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
            ast::Expr::Unit(_) => ExprKind::Unit,
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
                    ast::BinaryOp::And => BinaryOp::And,
                    ast::BinaryOp::Or => BinaryOp::Or,
                },
                left: Box::new(self.lower_expr(&expr.left)),
                right: Box::new(self.lower_expr(&expr.right)),
            }),
            ast::Expr::Call(expr) => ExprKind::Call(CallExpr {
                callee: Box::new(self.lower_expr(&expr.callee)),
                args: expr.args.iter().map(|arg| self.lower_expr(arg)).collect(),
                origin: CallOrigin::from(expr.origin),
                resolved_callee: self.resolved_callee_for_call(expr.id),
                json_decode_schema: self.json_decode_schema_for_call(expr.id),
                json_required_decode_schema: self.json_required_decode_schema_for_call(expr.id),
                json_to_value_schema: self.json_to_value_schema_for_call(expr.id),
                json_encode_typed_schema: self.json_encode_typed_schema_for_call(expr.id),
                config_required_load_json_schema: self
                    .config_required_load_json_schema_for_call(expr.id),
                config_load_json_schema: self.config_load_json_schema_for_call(expr.id),
                cli_parse_schema: self.cli_parse_schema_for_call(expr.id),
                cli_parse_or_schema: self.cli_parse_or_schema_for_call(expr.id),
                cli_parse_request_schema: self.cli_parse_request_schema_for_call(expr.id),
                cli_parse_request_or_schema: self.cli_parse_request_or_schema_for_call(expr.id),
                cli_usage_for_schema: self.cli_usage_for_schema_for_call(expr.id),
                cli_usage_for_required_schema: self.cli_usage_for_required_schema_for_call(expr.id),
                cli_help_for_schema: self.cli_help_for_schema_for_call(expr.id),
                cli_help_for_required_schema: self.cli_help_for_required_schema_for_call(expr.id),
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
                                    payload: self.lower_match_pattern_payload(pattern),
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

    fn lower_match_pattern_payload(
        &self,
        pattern: &ast::EnumVariantPattern,
    ) -> EnumVariantPatternPayload {
        match &pattern.payload {
            ast::EnumVariantPatternPayload::None => EnumVariantPatternPayload::None,
            ast::EnumVariantPatternPayload::Binding(name) => EnumVariantPatternPayload::Binding {
                name: name.clone(),
                binding: self.binding_for_decl(name, pattern.span, BindingKind::Immutable),
            },
            ast::EnumVariantPatternPayload::Discard => EnumVariantPatternPayload::Discard,
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

    fn using_cleanup(&self, id: StmtId) -> UsingCleanup {
        let cleanup = *self
            .using_cleanups
            .get(&id)
            .expect("checked using statement should have cleanup info");
        UsingCleanup {
            name: self.analysis.symbols.resolve(cleanup.name).to_string(),
            target: self.ident_target_for_callee(cleanup.callee),
            span: cleanup.span,
        }
    }

    fn ident_target_for_callee(&self, callee: TypedCalleeInfo) -> IdentTarget {
        match self.package_target_for_callee(callee) {
            TypedCalleeInfo::Binding(binding) => IdentTarget::Binding(binding),
            TypedCalleeInfo::PackageItem { binding, item } => {
                IdentTarget::PackageItem { binding, item }
            }
            _ => unreachable!("using cleanup should target a function binding"),
        }
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

    fn json_decode_schema_for_call(&self, id: ExprId) -> Option<JsonDecodeSchema> {
        self.json_decode_schemas.get(&id).cloned()
    }

    fn json_required_decode_schema_for_call(&self, id: ExprId) -> Option<JsonDecodeSchema> {
        self.json_required_decode_schemas.get(&id).cloned()
    }

    fn json_to_value_schema_for_call(&self, id: ExprId) -> Option<JsonDecodeSchema> {
        self.json_to_value_schemas.get(&id).cloned()
    }

    fn json_encode_typed_schema_for_call(&self, id: ExprId) -> Option<JsonDecodeSchema> {
        self.json_encode_typed_schemas.get(&id).cloned()
    }

    fn config_load_json_schema_for_call(&self, id: ExprId) -> Option<JsonDecodeSchema> {
        self.config_load_json_schemas.get(&id).cloned()
    }

    fn config_required_load_json_schema_for_call(
        &self,
        id: ExprId,
    ) -> Option<Box<JsonDecodeSchema>> {
        self.config_required_load_json_schemas
            .get(&id)
            .cloned()
            .map(Box::new)
    }

    fn cli_parse_schema_for_call(&self, id: ExprId) -> Option<Box<CliSchema>> {
        self.cli_parse_schemas.get(&id).cloned().map(Box::new)
    }

    fn cli_parse_or_schema_for_call(&self, id: ExprId) -> Option<Box<CliSchema>> {
        self.cli_parse_or_schemas.get(&id).cloned().map(Box::new)
    }

    fn cli_parse_request_schema_for_call(&self, id: ExprId) -> Option<Box<CliSchema>> {
        self.cli_parse_request_schemas
            .get(&id)
            .cloned()
            .map(Box::new)
    }

    fn cli_parse_request_or_schema_for_call(&self, id: ExprId) -> Option<Box<CliSchema>> {
        self.cli_parse_request_or_schemas
            .get(&id)
            .cloned()
            .map(Box::new)
    }

    fn cli_usage_for_schema_for_call(&self, id: ExprId) -> Option<Box<CliSchema>> {
        self.cli_usage_for_schemas.get(&id).cloned().map(Box::new)
    }

    fn cli_usage_for_required_schema_for_call(&self, id: ExprId) -> Option<Box<CliSchema>> {
        self.cli_usage_for_required_schemas
            .get(&id)
            .cloned()
            .map(Box::new)
    }

    fn cli_help_for_schema_for_call(&self, id: ExprId) -> Option<Box<CliSchema>> {
        self.cli_help_for_schemas.get(&id).cloned().map(Box::new)
    }

    fn cli_help_for_required_schema_for_call(&self, id: ExprId) -> Option<Box<CliSchema>> {
        self.cli_help_for_required_schemas
            .get(&id)
            .cloned()
            .map(Box::new)
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
            ast::TypeExpr::Unit => TypeInfo::Unit,
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
                    } else if self.opaque_symbols.contains(&symbol) {
                        return self
                            .package_opaque_items_by_symbol
                            .get(&symbol)
                            .copied()
                            .map(|item| TypeInfo::PackageOpaque { symbol, item })
                            .unwrap_or(TypeInfo::Error);
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
                if self
                    .analysis
                    .symbols
                    .lookup(&generic.name)
                    .is_some_and(|symbol| self.opaque_symbols.contains(&symbol))
                {
                    return TypeInfo::Error;
                }
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

fn json_rename_from_attributes(attributes: &[ast::Attribute]) -> Option<String> {
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

fn json_aliases_from_attributes(attributes: &[ast::Attribute]) -> Vec<String> {
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

fn cli_name_from_attributes(attributes: &[ast::Attribute]) -> Option<String> {
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

fn cli_short_from_attributes(attributes: &[ast::Attribute]) -> Option<String> {
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

fn cli_position_from_attributes(attributes: &[ast::Attribute]) -> Option<u32> {
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

fn cli_value_source_from_attributes(attributes: &[ast::Attribute]) -> Option<CliValueSource> {
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

fn cli_aliases_from_attributes(attributes: &[ast::Attribute]) -> Vec<String> {
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

fn cli_help_from_attributes(attributes: &[ast::Attribute]) -> Option<String> {
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

fn cli_about_from_attributes(attributes: &[ast::Attribute]) -> Option<String> {
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

fn cli_hidden_from_attributes(attributes: &[ast::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.name == "cli"
            && attribute
                .arguments
                .iter()
                .any(|argument| argument.name == "hidden" && argument.value.is_none())
    })
}

fn cli_subcommand_from_attributes(attributes: &[ast::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.name == "cli"
            && attribute
                .arguments
                .iter()
                .any(|argument| argument.name == "subcommand" && argument.value.is_none())
    })
}

fn json_validation_from_attributes(attributes: &[ast::Attribute]) -> Vec<JsonDecodeValidationRule> {
    attributes
        .iter()
        .filter(|attribute| attribute.name == "validate")
        .flat_map(|attribute| attribute.arguments.iter())
        .filter_map(json_validation_rule_from_argument)
        .collect()
}

fn json_validation_rule_from_argument(
    argument: &ast::AttributeArgument,
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

fn json_deny_unknown_fields_from_attributes(attributes: &[ast::Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.name == "json"
            && attribute
                .arguments
                .iter()
                .any(|argument| argument.name == "deny_unknown_fields" && argument.value.is_none())
    })
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
