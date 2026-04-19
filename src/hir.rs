use crate::{
    ast,
    span::Span,
    symbol::{Symbol, SymbolTable},
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
    RecordLit(RecordLitExpr),
    Field(FieldExpr),
    RecordUpdate(RecordUpdateExpr),
    Unary(UnaryExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    If(IfExpr),
    Closure(ClosureExpr),
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
pub struct ClosureExpr {
    pub function: FunctionId,
    pub span: Span,
}

pub fn lower(program: &ast::Program) -> Program {
    let mut lowerer = Lowerer {
        functions: Vec::new(),
        symbols: SymbolTable::default(),
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

struct Lowerer {
    functions: Vec<Function>,
    symbols: SymbolTable,
}

impl Lowerer {
    fn lower_stmt(&mut self, statement: &ast::Stmt) -> Option<Stmt> {
        Some(match statement {
            ast::Stmt::Assign(stmt) => Stmt::Assign(AssignStmt {
                mutable: stmt.mutable,
                name: self.symbol(&stmt.name),
                value: self.lower_expr(&stmt.value),
                span: stmt.span,
            }),
            ast::Stmt::RecordDecl(_) => return None,
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
            ast::Expr::Ident(expr) => Expr::Ident(IdentExpr {
                name: self.symbol(&expr.name),
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
            ast::Expr::Call(expr) => Expr::Call(CallExpr {
                callee: Box::new(self.lower_expr(&expr.callee)),
                args: expr.args.iter().map(|arg| self.lower_expr(arg)).collect(),
                span: expr.span,
            }),
            ast::Expr::If(expr) => Expr::If(IfExpr {
                condition: Box::new(self.lower_expr(&expr.condition)),
                then_branch: self.lower_value_block(&expr.then_branch),
                else_branch: self.lower_value_block(&expr.else_branch),
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
