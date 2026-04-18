use crate::{
    hir,
    span::Span,
    symbol::{Symbol, SymbolTable},
};

pub type FunctionId = usize;

#[derive(Clone, Debug)]
pub struct Program {
    pub entry: Chunk,
    pub functions: Vec<Function>,
    pub symbols: SymbolTable,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub id: FunctionId,
    pub name: Option<Symbol>,
    pub params: Vec<Symbol>,
    pub chunk: Chunk,
    pub span: Span,
}

#[derive(Clone, Debug, Default)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
}

#[derive(Clone, Debug)]
pub enum Instruction {
    LoadInt(i64),
    LoadBool(bool),
    LoadString(String),
    LoadName {
        name: Symbol,
        span: Span,
    },
    Assign {
        name: Symbol,
        mutable: bool,
        span: Span,
    },
    DefineFunction {
        name: Symbol,
        function: FunctionId,
        span: Span,
    },
    MakeClosure {
        function: FunctionId,
    },
    UnaryNeg {
        span: Span,
    },
    UnaryNot {
        span: Span,
    },
    Binary {
        op: BinaryOp,
        span: Span,
    },
    Call {
        argc: usize,
        span: Span,
    },
    JumpIfFalse {
        target: usize,
        span: Span,
    },
    Jump {
        target: usize,
    },
    PushScope,
    PopScope,
    Pop,
    Return,
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

pub fn compile(program: hir::Program) -> Program {
    let hir::Program {
        statements,
        functions,
        symbols,
    } = program;
    let mut compiler = Compiler::new(symbols);
    let entry = compiler.compile_top_level(&statements);
    for function in &functions {
        compiler.compile_function(function);
    }
    Program {
        entry,
        functions: compiler.functions,
        symbols: compiler.symbols,
    }
}

struct Compiler {
    functions: Vec<Function>,
    symbols: SymbolTable,
}

impl Compiler {
    fn new(symbols: SymbolTable) -> Self {
        Self {
            functions: Vec::new(),
            symbols,
        }
    }

    fn compile_top_level(&mut self, statements: &[hir::Stmt]) -> Chunk {
        let mut chunk = Chunk::default();
        self.compile_scope_statements(statements, &mut chunk);
        chunk
    }

    fn compile_function(&mut self, function: &hir::Function) {
        if self.functions.len() <= function.id {
            self.functions
                .resize_with(function.id + 1, placeholder_function);
        }
        let mut chunk = Chunk::default();
        self.compile_scope_statements(&function.body.statements, &mut chunk);
        self.compile_expr(&function.body.expr, &mut chunk);
        chunk.instructions.push(Instruction::Return);
        self.functions[function.id] = Function {
            id: function.id,
            name: function.name.clone(),
            params: function.params.clone(),
            chunk,
            span: function.span,
        };
    }

    fn compile_scope_statements(&mut self, statements: &[hir::Stmt], chunk: &mut Chunk) {
        for statement in statements {
            if let hir::Stmt::Function(func) = statement {
                chunk.instructions.push(Instruction::DefineFunction {
                    name: func.name.clone(),
                    function: func.function,
                    span: func.span,
                });
            }
        }

        for statement in statements {
            self.compile_stmt(statement, chunk);
        }
    }

    fn compile_stmt(&mut self, statement: &hir::Stmt, chunk: &mut Chunk) {
        match statement {
            hir::Stmt::Assign(stmt) => {
                self.compile_expr(&stmt.value, chunk);
                chunk.instructions.push(Instruction::Assign {
                    name: stmt.name.clone(),
                    mutable: stmt.mutable,
                    span: stmt.span,
                });
            }
            hir::Stmt::Function(_) => {}
            hir::Stmt::If(stmt) => self.compile_if_stmt(stmt, chunk),
            hir::Stmt::While(stmt) => self.compile_while_stmt(stmt, chunk),
            hir::Stmt::Expr(stmt) => {
                self.compile_expr(&stmt.expr, chunk);
                chunk.instructions.push(Instruction::Pop);
            }
        }
    }

    fn compile_if_stmt(&mut self, stmt: &hir::IfStmt, chunk: &mut Chunk) {
        self.compile_expr(&stmt.condition, chunk);
        let false_jump = self.emit_jump_if_false(chunk, stmt.condition.span());
        self.compile_block(&stmt.then_branch, chunk);
        let end_jump = stmt.else_branch.as_ref().map(|_| self.emit_jump(chunk));
        let else_target = chunk.instructions.len();
        self.patch_jump_if_false(chunk, false_jump, else_target);
        if let Some(else_branch) = &stmt.else_branch {
            self.compile_block(else_branch, chunk);
            let end_target = chunk.instructions.len();
            if let Some(end_jump) = end_jump {
                self.patch_jump(chunk, end_jump, end_target);
            }
        }
    }

    fn compile_while_stmt(&mut self, stmt: &hir::WhileStmt, chunk: &mut Chunk) {
        let loop_start = chunk.instructions.len();
        self.compile_expr(&stmt.condition, chunk);
        let exit_jump = self.emit_jump_if_false(chunk, stmt.condition.span());
        self.compile_block(&stmt.body, chunk);
        chunk
            .instructions
            .push(Instruction::Jump { target: loop_start });
        let loop_end = chunk.instructions.len();
        self.patch_jump_if_false(chunk, exit_jump, loop_end);
    }

    fn compile_block(&mut self, block: &hir::Block, chunk: &mut Chunk) {
        chunk.instructions.push(Instruction::PushScope);
        self.compile_scope_statements(&block.statements, chunk);
        chunk.instructions.push(Instruction::PopScope);
    }

    fn compile_value_block(&mut self, block: &hir::ValueBlock, chunk: &mut Chunk) {
        chunk.instructions.push(Instruction::PushScope);
        self.compile_scope_statements(&block.statements, chunk);
        self.compile_expr(&block.expr, chunk);
        chunk.instructions.push(Instruction::PopScope);
    }

    fn compile_expr(&mut self, expr: &hir::Expr, chunk: &mut Chunk) {
        match expr {
            hir::Expr::Int(expr) => chunk.instructions.push(Instruction::LoadInt(expr.value)),
            hir::Expr::Bool(expr) => chunk.instructions.push(Instruction::LoadBool(expr.value)),
            hir::Expr::String(expr) => {
                chunk
                    .instructions
                    .push(Instruction::LoadString(expr.value.clone()));
            }
            hir::Expr::Ident(expr) => chunk.instructions.push(Instruction::LoadName {
                name: expr.name.clone(),
                span: expr.span,
            }),
            hir::Expr::Unary(expr) => {
                self.compile_expr(&expr.expr, chunk);
                chunk.instructions.push(match expr.op {
                    hir::UnaryOp::Neg => Instruction::UnaryNeg { span: expr.span },
                    hir::UnaryOp::Not => Instruction::UnaryNot { span: expr.span },
                });
            }
            hir::Expr::Binary(expr) => {
                self.compile_expr(&expr.left, chunk);
                self.compile_expr(&expr.right, chunk);
                chunk.instructions.push(Instruction::Binary {
                    op: match expr.op {
                        hir::BinaryOp::Add => BinaryOp::Add,
                        hir::BinaryOp::Sub => BinaryOp::Sub,
                        hir::BinaryOp::Mul => BinaryOp::Mul,
                        hir::BinaryOp::Div => BinaryOp::Div,
                        hir::BinaryOp::Lt => BinaryOp::Lt,
                        hir::BinaryOp::LtEq => BinaryOp::LtEq,
                        hir::BinaryOp::Gt => BinaryOp::Gt,
                        hir::BinaryOp::GtEq => BinaryOp::GtEq,
                        hir::BinaryOp::EqEq => BinaryOp::EqEq,
                        hir::BinaryOp::BangEq => BinaryOp::BangEq,
                    },
                    span: expr.span,
                });
            }
            hir::Expr::Call(expr) => {
                self.compile_expr(&expr.callee, chunk);
                for arg in &expr.args {
                    self.compile_expr(arg, chunk);
                }
                chunk.instructions.push(Instruction::Call {
                    argc: expr.args.len(),
                    span: expr.span,
                });
            }
            hir::Expr::If(expr) => self.compile_if_expr(expr, chunk),
            hir::Expr::Closure(expr) => chunk.instructions.push(Instruction::MakeClosure {
                function: expr.function,
            }),
        }
    }

    fn compile_if_expr(&mut self, expr: &hir::IfExpr, chunk: &mut Chunk) {
        self.compile_expr(&expr.condition, chunk);
        let false_jump = self.emit_jump_if_false(chunk, expr.condition.span());
        self.compile_value_block(&expr.then_branch, chunk);
        let end_jump = self.emit_jump(chunk);
        let else_target = chunk.instructions.len();
        self.patch_jump_if_false(chunk, false_jump, else_target);
        self.compile_value_block(&expr.else_branch, chunk);
        let end_target = chunk.instructions.len();
        self.patch_jump(chunk, end_jump, end_target);
    }

    fn emit_jump_if_false(&self, chunk: &mut Chunk, span: Span) -> usize {
        let index = chunk.instructions.len();
        chunk
            .instructions
            .push(Instruction::JumpIfFalse { target: 0, span });
        index
    }

    fn emit_jump(&self, chunk: &mut Chunk) -> usize {
        let index = chunk.instructions.len();
        chunk.instructions.push(Instruction::Jump { target: 0 });
        index
    }

    fn patch_jump_if_false(&self, chunk: &mut Chunk, index: usize, target: usize) {
        let Instruction::JumpIfFalse {
            target: patched_target,
            ..
        } = &mut chunk.instructions[index]
        else {
            unreachable!("expected JumpIfFalse at patch site");
        };
        *patched_target = target;
    }

    fn patch_jump(&self, chunk: &mut Chunk, index: usize, target: usize) {
        let Instruction::Jump {
            target: patched_target,
        } = &mut chunk.instructions[index]
        else {
            unreachable!("expected Jump at patch site");
        };
        *patched_target = target;
    }
}

fn placeholder_function() -> Function {
    Function {
        id: 0,
        name: None,
        params: Vec::new(),
        chunk: Chunk::default(),
        span: Span::default(),
    }
}
