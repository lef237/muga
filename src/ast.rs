use crate::span::Span;

#[derive(Clone, Debug)]
pub struct Program {
    pub statements: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub enum Stmt {
    Assign(AssignStmt),
    RecordDecl(RecordDecl),
    FuncDecl(FuncDecl),
    If(IfStmt),
    While(WhileStmt),
    Expr(ExprStmt),
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Self::Assign(stmt) => stmt.span,
            Self::RecordDecl(stmt) => stmt.span,
            Self::FuncDecl(stmt) => stmt.span,
            Self::If(stmt) => stmt.span,
            Self::While(stmt) => stmt.span,
            Self::Expr(stmt) => stmt.span,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AssignStmt {
    pub mutable: bool,
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RecordDecl {
    pub name: String,
    pub fields: Vec<RecordFieldDecl>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RecordFieldDecl {
    pub name: String,
    pub type_name: TypeExpr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: ValueBlock,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct Param {
    pub name: String,
    pub type_name: Option<TypeExpr>,
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
    RecordLit(RecordLitExpr),
    Field(FieldExpr),
    RecordUpdate(RecordUpdateExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    If(IfExpr),
    Fn(FnExpr),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Self::Int(expr) => expr.span,
            Self::Bool(expr) => expr.span,
            Self::String(expr) => expr.span,
            Self::Ident(expr) => expr.span,
            Self::RecordLit(expr) => expr.span,
            Self::Field(expr) => expr.span,
            Self::RecordUpdate(expr) => expr.span,
            Self::Unary(expr) => expr.span,
            Self::Binary(expr) => expr.span,
            Self::Call(expr) => expr.span,
            Self::If(expr) => expr.span,
            Self::Fn(expr) => expr.span,
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
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RecordLitExpr {
    pub type_name: String,
    pub fields: Vec<RecordFieldInit>,
    pub span: Span,
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
pub struct FnExpr {
    pub params: Vec<Param>,
    pub return_type: Option<TypeExpr>,
    pub body: ValueBlock,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeExpr {
    Int,
    Bool,
    String,
    Named(String),
    Function(FunctionTypeExpr),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionTypeExpr {
    pub params: Vec<TypeExpr>,
    pub ret: Box<TypeExpr>,
}
