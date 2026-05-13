use crate::{
    mir,
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
    MakeRecord {
        type_name: Symbol,
        fields: Vec<Symbol>,
        span: Span,
    },
    MakeEnum {
        enum_name: Symbol,
        variant_name: Symbol,
        has_payload: bool,
        span: Span,
    },
    MakeList {
        len: usize,
        span: Span,
    },
    LoadName {
        name: Symbol,
        span: Span,
    },
    LoadField {
        field: Symbol,
        span: Span,
    },
    LoadIndex {
        span: Span,
    },
    UpdateRecord {
        fields: Vec<Symbol>,
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
    JumpIfNotEnumVariant {
        enum_name: Symbol,
        variant_name: Symbol,
        target: usize,
        span: Span,
    },
    MatchExhausted {
        enum_name: Symbol,
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

pub fn compile(program: mir::Program) -> Program {
    let mir::Program {
        entry,
        functions,
        symbols,
    } = program;
    let mut compiler = Compiler::new(symbols);
    let entry = compiler.compile_entry_body(&entry);
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
    next_match_temp: usize,
}

impl Compiler {
    fn new(symbols: SymbolTable) -> Self {
        Self {
            functions: Vec::new(),
            symbols,
            next_match_temp: 0,
        }
    }

    fn compile_entry_body(&mut self, body: &mir::Body) -> Chunk {
        let mut chunk = Chunk::default();
        self.compile_function_defs(&body.function_defs, &mut chunk);
        self.compile_scope_statements(&body.statements, &mut chunk);
        if let Some(result) = &body.result {
            self.compile_expr(result, &mut chunk);
            chunk.instructions.push(Instruction::Pop);
        }
        chunk
    }

    fn compile_function(&mut self, function: &mir::Function) {
        if self.functions.len() <= function.id {
            self.functions
                .resize_with(function.id + 1, placeholder_function);
        }
        let mut chunk = Chunk::default();
        self.compile_function_defs(&function.body.function_defs, &mut chunk);
        self.compile_scope_statements(&function.body.statements, &mut chunk);
        let Some(result) = &function.body.result else {
            unreachable!("function MIR body should have a result expression");
        };
        self.compile_expr(result, &mut chunk);
        chunk.instructions.push(Instruction::Return);
        self.functions[function.id] = Function {
            id: function.id,
            name: function.name,
            params: function.params.clone(),
            chunk,
            span: function.span,
        };
    }

    fn compile_function_defs(&mut self, function_defs: &[mir::FunctionDef], chunk: &mut Chunk) {
        for func in function_defs {
            chunk.instructions.push(Instruction::DefineFunction {
                name: func.name,
                function: func.function,
                span: func.span,
            });
        }
    }

    fn compile_scope_statements(&mut self, statements: &[mir::Stmt], chunk: &mut Chunk) {
        for statement in statements {
            self.compile_stmt(statement, chunk);
        }
    }

    fn compile_stmt(&mut self, statement: &mir::Stmt, chunk: &mut Chunk) {
        match statement {
            mir::Stmt::Assign(stmt) => {
                self.compile_expr(&stmt.value, chunk);
                chunk.instructions.push(Instruction::Assign {
                    name: stmt.name,
                    mutable: stmt.mutable,
                    span: stmt.span,
                });
            }
            mir::Stmt::If(stmt) => self.compile_if_stmt(stmt, chunk),
            mir::Stmt::While(stmt) => self.compile_while_stmt(stmt, chunk),
            mir::Stmt::Expr(stmt) => {
                self.compile_expr(&stmt.expr, chunk);
                chunk.instructions.push(Instruction::Pop);
            }
        }
    }

    fn compile_if_stmt(&mut self, stmt: &mir::IfStmt, chunk: &mut Chunk) {
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

    fn compile_while_stmt(&mut self, stmt: &mir::WhileStmt, chunk: &mut Chunk) {
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

    fn compile_block(&mut self, block: &mir::Block, chunk: &mut Chunk) {
        chunk.instructions.push(Instruction::PushScope);
        self.compile_function_defs(&block.function_defs, chunk);
        self.compile_scope_statements(&block.statements, chunk);
        chunk.instructions.push(Instruction::PopScope);
    }

    fn compile_value_block(&mut self, block: &mir::ValueBlock, chunk: &mut Chunk) {
        chunk.instructions.push(Instruction::PushScope);
        self.compile_function_defs(&block.function_defs, chunk);
        self.compile_scope_statements(&block.statements, chunk);
        self.compile_expr(&block.expr, chunk);
        chunk.instructions.push(Instruction::PopScope);
    }

    fn compile_expr(&mut self, expr: &mir::Expr, chunk: &mut Chunk) {
        match expr {
            mir::Expr::Int(expr) => chunk.instructions.push(Instruction::LoadInt(expr.value)),
            mir::Expr::Bool(expr) => chunk.instructions.push(Instruction::LoadBool(expr.value)),
            mir::Expr::String(expr) => {
                chunk
                    .instructions
                    .push(Instruction::LoadString(expr.value.clone()));
            }
            mir::Expr::RecordLit(expr) => {
                for field in &expr.fields {
                    self.compile_expr(&field.value, chunk);
                }
                chunk.instructions.push(Instruction::MakeRecord {
                    type_name: expr.type_name,
                    fields: expr.fields.iter().map(|field| field.name).collect(),
                    span: expr.span,
                });
            }
            mir::Expr::EnumVariant(expr) => {
                if let Some(payload) = &expr.payload {
                    self.compile_expr(payload, chunk);
                }
                chunk.instructions.push(Instruction::MakeEnum {
                    enum_name: expr.enum_name,
                    variant_name: expr.variant_name,
                    has_payload: expr.payload.is_some(),
                    span: expr.span,
                });
            }
            mir::Expr::ListLit(expr) => {
                for item in &expr.items {
                    self.compile_expr(item, chunk);
                }
                chunk.instructions.push(Instruction::MakeList {
                    len: expr.items.len(),
                    span: expr.span,
                });
            }
            mir::Expr::Ident(expr) => chunk.instructions.push(Instruction::LoadName {
                name: expr.name,
                span: expr.span,
            }),
            mir::Expr::Field(expr) => {
                self.compile_expr(&expr.base, chunk);
                chunk.instructions.push(Instruction::LoadField {
                    field: expr.field,
                    span: expr.span,
                });
            }
            mir::Expr::Index(expr) => {
                self.compile_expr(&expr.base, chunk);
                self.compile_expr(&expr.index, chunk);
                chunk
                    .instructions
                    .push(Instruction::LoadIndex { span: expr.span });
            }
            mir::Expr::RecordUpdate(expr) => {
                self.compile_expr(&expr.base, chunk);
                for field in &expr.fields {
                    self.compile_expr(&field.value, chunk);
                }
                chunk.instructions.push(Instruction::UpdateRecord {
                    fields: expr.fields.iter().map(|field| field.name).collect(),
                    span: expr.span,
                });
            }
            mir::Expr::Unary(expr) => {
                self.compile_expr(&expr.expr, chunk);
                chunk.instructions.push(match expr.op {
                    mir::UnaryOp::Neg => Instruction::UnaryNeg { span: expr.span },
                    mir::UnaryOp::Not => Instruction::UnaryNot { span: expr.span },
                });
            }
            mir::Expr::Binary(expr) => {
                self.compile_expr(&expr.left, chunk);
                self.compile_expr(&expr.right, chunk);
                chunk.instructions.push(Instruction::Binary {
                    op: match expr.op {
                        mir::BinaryOp::Add => BinaryOp::Add,
                        mir::BinaryOp::Sub => BinaryOp::Sub,
                        mir::BinaryOp::Mul => BinaryOp::Mul,
                        mir::BinaryOp::Div => BinaryOp::Div,
                        mir::BinaryOp::Lt => BinaryOp::Lt,
                        mir::BinaryOp::LtEq => BinaryOp::LtEq,
                        mir::BinaryOp::Gt => BinaryOp::Gt,
                        mir::BinaryOp::GtEq => BinaryOp::GtEq,
                        mir::BinaryOp::EqEq => BinaryOp::EqEq,
                        mir::BinaryOp::BangEq => BinaryOp::BangEq,
                    },
                    span: expr.span,
                });
            }
            mir::Expr::Call(expr) => {
                self.compile_expr(&expr.callee, chunk);
                for arg in &expr.args {
                    self.compile_expr(arg, chunk);
                }
                chunk.instructions.push(Instruction::Call {
                    argc: expr.args.len(),
                    span: expr.span,
                });
            }
            mir::Expr::If(expr) => self.compile_if_expr(expr, chunk),
            mir::Expr::Match(expr) => self.compile_match_expr(expr, chunk),
            mir::Expr::Closure(expr) => chunk.instructions.push(Instruction::MakeClosure {
                function: expr.function,
            }),
        }
    }

    fn compile_if_expr(&mut self, expr: &mir::IfExpr, chunk: &mut Chunk) {
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

    fn compile_match_expr(&mut self, expr: &mir::MatchExpr, chunk: &mut Chunk) {
        let enum_name = self.pattern_enum_symbol(expr);
        let temp = self.match_temp_symbol();
        chunk.instructions.push(Instruction::PushScope);
        self.compile_expr(&expr.value, chunk);
        chunk.instructions.push(Instruction::Assign {
            name: temp,
            mutable: false,
            span: expr.value.span(),
        });

        let mut end_jumps = Vec::new();
        for arm in &expr.arms {
            let (_, variant_name) = self.pattern_variant_symbols(&arm.pattern);
            chunk.instructions.push(Instruction::LoadName {
                name: temp,
                span: expr.value.span(),
            });
            let next_arm_jump = self.emit_jump_if_not_enum_variant(
                chunk,
                enum_name,
                variant_name,
                expr.value.span(),
            );
            self.compile_match_arm(arm, chunk);
            end_jumps.push(self.emit_jump(chunk));
            let next_target = chunk.instructions.len();
            self.patch_jump_if_not_enum_variant(chunk, next_arm_jump, next_target);
        }

        chunk.instructions.push(Instruction::MatchExhausted {
            enum_name,
            span: expr.span,
        });
        let end_target = chunk.instructions.len();
        for jump in end_jumps {
            self.patch_jump(chunk, jump, end_target);
        }
        chunk.instructions.push(Instruction::PopScope);
    }

    fn compile_match_arm(&mut self, arm: &mir::MatchArm, chunk: &mut Chunk) {
        chunk.instructions.push(Instruction::PushScope);
        if self.pattern_binding(&arm.pattern).is_some() {
            let (binding, span) = self
                .pattern_binding(&arm.pattern)
                .expect("typechecked payload variant arm should bind payload");
            chunk.instructions.push(Instruction::Assign {
                name: binding,
                mutable: false,
                span,
            });
        }
        self.compile_expr(&arm.value, chunk);
        chunk.instructions.push(Instruction::PopScope);
    }

    fn pattern_enum_symbol(&self, expr: &mir::MatchExpr) -> Symbol {
        let first = expr
            .arms
            .first()
            .expect("typechecked match should have at least one arm");
        let mir::MatchPattern::Variant(pattern) = &first.pattern;
        pattern.enum_name
    }

    fn pattern_binding(&self, pattern: &mir::MatchPattern) -> Option<(Symbol, Span)> {
        let mir::MatchPattern::Variant(pattern) = pattern;
        pattern.binding.map(|binding| (binding, pattern.span))
    }

    fn pattern_variant_symbols(&self, pattern: &mir::MatchPattern) -> (Symbol, Symbol) {
        let mir::MatchPattern::Variant(pattern) = pattern;
        (pattern.enum_name, pattern.variant_name)
    }

    fn emit_jump_if_false(&self, chunk: &mut Chunk, span: Span) -> usize {
        let index = chunk.instructions.len();
        chunk
            .instructions
            .push(Instruction::JumpIfFalse { target: 0, span });
        index
    }

    fn emit_jump_if_not_enum_variant(
        &self,
        chunk: &mut Chunk,
        enum_name: Symbol,
        variant_name: Symbol,
        span: Span,
    ) -> usize {
        let index = chunk.instructions.len();
        chunk.instructions.push(Instruction::JumpIfNotEnumVariant {
            enum_name,
            variant_name,
            target: 0,
            span,
        });
        index
    }

    fn match_temp_symbol(&mut self) -> Symbol {
        let name = format!("__muga_match_value_{}", self.next_match_temp);
        self.next_match_temp += 1;
        self.symbols.intern(&name)
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

    fn patch_jump_if_not_enum_variant(&self, chunk: &mut Chunk, index: usize, target: usize) {
        let Instruction::JumpIfNotEnumVariant {
            target: patched_target,
            ..
        } = &mut chunk.instructions[index]
        else {
            unreachable!("expected JumpIfNotEnumVariant at patch site");
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
