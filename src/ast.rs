use crate::identity::{ExprId, StmtId};
use crate::span::Span;

#[derive(Clone, Debug)]
pub struct Program {
    pub package: Option<PackageDecl>,
    pub imports: Vec<ImportDecl>,
    pub statements: Vec<Stmt>,
}

pub fn renumber_node_ids(program: &mut Program) {
    let mut assigner = NodeIdAssigner {
        next_expr_id: 0,
        next_stmt_id: 0,
    };
    for statement in &mut program.statements {
        assigner.assign_stmt(statement);
    }
}

#[derive(Clone, Debug)]
pub struct PackageDecl {
    pub path: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct ImportDecl {
    pub path: String,
    pub alias: String,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
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
    pub fn id(&self) -> StmtId {
        match self {
            Self::Assign(stmt) => stmt.id,
            Self::RecordDecl(stmt) => stmt.id,
            Self::FuncDecl(stmt) => stmt.id,
            Self::If(stmt) => stmt.id,
            Self::While(stmt) => stmt.id,
            Self::Expr(stmt) => stmt.id,
        }
    }

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
    pub id: StmtId,
    pub mutable: bool,
    pub name: String,
    pub value: Expr,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RecordDecl {
    pub id: StmtId,
    pub name: String,
    pub visibility: Visibility,
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
    pub id: StmtId,
    pub name: String,
    pub visibility: Visibility,
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
    pub fn id(&self) -> ExprId {
        match self {
            Self::Int(expr) => expr.id,
            Self::Bool(expr) => expr.id,
            Self::String(expr) => expr.id,
            Self::Ident(expr) => expr.id,
            Self::RecordLit(expr) => expr.id,
            Self::Field(expr) => expr.id,
            Self::RecordUpdate(expr) => expr.id,
            Self::Unary(expr) => expr.id,
            Self::Binary(expr) => expr.id,
            Self::Call(expr) => expr.id,
            Self::If(expr) => expr.id,
            Self::Fn(expr) => expr.id,
        }
    }

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
    pub id: ExprId,
    pub value: i64,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct BoolExpr {
    pub id: ExprId,
    pub value: bool,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct StringExpr {
    pub id: ExprId,
    pub value: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct IdentExpr {
    pub id: ExprId,
    pub name: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RecordLitExpr {
    pub id: ExprId,
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
    pub id: ExprId,
    pub base: Box<Expr>,
    pub field: String,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct RecordUpdateExpr {
    pub id: ExprId,
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
    pub id: ExprId,
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
    pub id: ExprId,
    pub op: BinaryOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct CallExpr {
    pub id: ExprId,
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub origin: CallOrigin,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallOrigin {
    Ordinary,
    Chained,
    QualifiedChained,
}

#[derive(Clone, Debug)]
pub struct IfExpr {
    pub id: ExprId,
    pub condition: Box<Expr>,
    pub then_branch: ValueBlock,
    pub else_branch: ValueBlock,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub struct FnExpr {
    pub id: ExprId,
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

struct NodeIdAssigner {
    next_expr_id: u32,
    next_stmt_id: u32,
}

impl NodeIdAssigner {
    fn assign_stmt(&mut self, statement: &mut Stmt) {
        match statement {
            Stmt::Assign(stmt) => {
                stmt.id = self.stmt_id();
                self.assign_expr(&mut stmt.value);
            }
            Stmt::RecordDecl(stmt) => {
                stmt.id = self.stmt_id();
            }
            Stmt::FuncDecl(stmt) => {
                stmt.id = self.stmt_id();
                self.assign_value_block(&mut stmt.body);
            }
            Stmt::If(stmt) => {
                stmt.id = self.stmt_id();
                self.assign_expr(&mut stmt.condition);
                self.assign_block(&mut stmt.then_branch);
                if let Some(else_branch) = &mut stmt.else_branch {
                    self.assign_block(else_branch);
                }
            }
            Stmt::While(stmt) => {
                stmt.id = self.stmt_id();
                self.assign_expr(&mut stmt.condition);
                self.assign_block(&mut stmt.body);
            }
            Stmt::Expr(stmt) => {
                stmt.id = self.stmt_id();
                self.assign_expr(&mut stmt.expr);
            }
        }
    }

    fn assign_block(&mut self, block: &mut Block) {
        for statement in &mut block.statements {
            self.assign_stmt(statement);
        }
    }

    fn assign_value_block(&mut self, block: &mut ValueBlock) {
        for statement in &mut block.statements {
            self.assign_stmt(statement);
        }
        self.assign_expr(&mut block.expr);
    }

    fn assign_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Int(expr) => expr.id = self.expr_id(),
            Expr::Bool(expr) => expr.id = self.expr_id(),
            Expr::String(expr) => expr.id = self.expr_id(),
            Expr::Ident(expr) => expr.id = self.expr_id(),
            Expr::RecordLit(expr) => {
                expr.id = self.expr_id();
                for field in &mut expr.fields {
                    self.assign_expr(&mut field.value);
                }
            }
            Expr::Field(expr) => {
                expr.id = self.expr_id();
                self.assign_expr(&mut expr.base);
            }
            Expr::RecordUpdate(expr) => {
                expr.id = self.expr_id();
                self.assign_expr(&mut expr.base);
                for field in &mut expr.fields {
                    self.assign_expr(&mut field.value);
                }
            }
            Expr::Unary(expr) => {
                expr.id = self.expr_id();
                self.assign_expr(&mut expr.expr);
            }
            Expr::Binary(expr) => {
                expr.id = self.expr_id();
                self.assign_expr(&mut expr.left);
                self.assign_expr(&mut expr.right);
            }
            Expr::Call(expr) => {
                expr.id = self.expr_id();
                self.assign_expr(&mut expr.callee);
                for arg in &mut expr.args {
                    self.assign_expr(arg);
                }
            }
            Expr::If(expr) => {
                expr.id = self.expr_id();
                self.assign_expr(&mut expr.condition);
                self.assign_value_block(&mut expr.then_branch);
                self.assign_value_block(&mut expr.else_branch);
            }
            Expr::Fn(expr) => {
                expr.id = self.expr_id();
                self.assign_value_block(&mut expr.body);
            }
        }
    }

    fn expr_id(&mut self) -> ExprId {
        let id = ExprId::new(self.next_expr_id);
        self.next_expr_id += 1;
        id
    }

    fn stmt_id(&mut self) -> StmtId {
        let id = StmtId::new(self.next_stmt_id);
        self.next_stmt_id += 1;
        id
    }
}
