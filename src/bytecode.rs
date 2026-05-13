use std::collections::HashMap;

use crate::{
    identity::{BindingId, BindingKind, LocalId, PackageItemId},
    mir,
    span::Span,
    symbol::{Symbol, SymbolTable},
};

pub type FunctionId = usize;

#[derive(Clone, Debug)]
pub struct Program {
    pub entry: Chunk,
    pub functions: Vec<Function>,
    pub bindings: Vec<BindingDef>,
    pub main: Option<NameRef>,
    pub local_count: usize,
    pub symbols: SymbolTable,
}

#[derive(Clone, Debug)]
pub struct BindingDef {
    pub id: BindingId,
    pub local: LocalId,
    pub name: Symbol,
    pub kind: BindingKind,
    pub package_item: Option<PackageItemId>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NameRef {
    pub binding: BindingId,
    pub local: LocalId,
    pub name: Symbol,
}

#[derive(Clone, Debug)]
pub struct Function {
    pub id: FunctionId,
    pub name: Option<Symbol>,
    pub params: Vec<NameRef>,
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
        target: NameRef,
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
        target: NameRef,
        mutable: bool,
        is_update: bool,
        span: Span,
    },
    DefineFunction {
        target: NameRef,
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
        bindings,
        symbols,
    } = program;
    let main = entry_main_ref(&entry, &symbols);
    let bindings: Vec<_> = bindings.into_iter().map(BindingDef::from).collect();
    let mut compiler = Compiler::new(symbols, next_synthetic_local(&bindings));
    let entry = compiler.compile_entry_body(&entry);
    for function in &functions {
        compiler.compile_function(function);
    }
    let local_count = compiler.next_synthetic_local as usize;
    Program {
        entry,
        functions: compiler.functions,
        bindings,
        main,
        local_count,
        symbols: compiler.symbols,
    }
}

fn next_synthetic_local(bindings: &[BindingDef]) -> u32 {
    bindings
        .iter()
        .map(|binding| binding.local.as_u32())
        .max()
        .map_or(0, |id| id + 1)
}

fn entry_main_ref(entry: &mir::Body, symbols: &SymbolTable) -> Option<NameRef> {
    entry
        .function_defs
        .iter()
        .find_map(|function| {
            (symbols.resolve(function.name) == "main").then_some(NameRef {
                binding: function.binding,
                local: local_for_binding(function.binding),
                name: function.name,
            })
        })
        .or_else(|| {
            entry
                .statements
                .iter()
                .find_map(|statement| match statement {
                    mir::Stmt::Assign(statement) if symbols.resolve(statement.name) == "main" => {
                        Some(NameRef {
                            binding: statement.binding,
                            local: local_for_binding(statement.binding),
                            name: statement.name,
                        })
                    }
                    _ => None,
                })
        })
}

fn local_for_binding(binding: BindingId) -> LocalId {
    LocalId::new(binding.as_u32())
}

impl From<mir::BindingDef> for BindingDef {
    fn from(binding: mir::BindingDef) -> Self {
        Self {
            id: binding.id,
            local: local_for_binding(binding.id),
            name: binding.name,
            kind: binding.kind,
            package_item: binding.package_item,
            span: binding.span,
        }
    }
}

struct Compiler {
    functions: Vec<Function>,
    symbols: SymbolTable,
    package_function_bindings: HashMap<PackageItemId, BindingId>,
    next_match_temp: usize,
    next_synthetic_local: u32,
}

impl Compiler {
    fn new(symbols: SymbolTable, next_synthetic_local: u32) -> Self {
        Self {
            functions: Vec::new(),
            symbols,
            package_function_bindings: HashMap::new(),
            next_match_temp: 0,
            next_synthetic_local,
        }
    }

    fn compile_entry_body(&mut self, body: &mir::Body) -> Chunk {
        let mut chunk = Chunk::default();
        self.compile_function_defs(&body.function_defs, &mut chunk);
        self.compile_scope_statements(&body.statements, &mut chunk);
        match &body.terminator {
            mir::BodyTerminator::Effect => {}
            mir::BodyTerminator::Return(result) => {
                self.compile_expr(result, &mut chunk);
                chunk.instructions.push(Instruction::Pop);
            }
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
        let mir::BodyTerminator::Return(result) = &function.body.terminator else {
            unreachable!("function MIR body should return a value");
        };
        self.compile_expr(result, &mut chunk);
        chunk.instructions.push(Instruction::Return);
        self.functions[function.id] = Function {
            id: function.id,
            name: function.name,
            params: function
                .params
                .iter()
                .map(|param| self.name_ref(param.binding, param.name))
                .collect(),
            chunk,
            span: function.span,
        };
    }

    fn compile_function_defs(&mut self, function_defs: &[mir::FunctionDef], chunk: &mut Chunk) {
        for func in function_defs {
            if let Some(item) = func.package_item {
                self.package_function_bindings
                    .entry(item)
                    .or_insert(func.binding);
            }
            chunk.instructions.push(Instruction::DefineFunction {
                target: self.name_ref(func.binding, func.name),
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
                    target: self.name_ref(stmt.binding, stmt.name),
                    mutable: stmt.mutable,
                    is_update: stmt.is_update,
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
            mir::Expr::Ident(expr) => {
                chunk.instructions.push(Instruction::LoadName {
                    target: self.name_ref_for_ident_target(expr.target, expr.name),
                    span: expr.span,
                });
            }
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
        let temp_ref = self.synthetic_name_ref(temp);
        chunk.instructions.push(Instruction::PushScope);
        self.compile_expr(&expr.value, chunk);
        chunk.instructions.push(Instruction::Assign {
            target: temp_ref,
            mutable: false,
            is_update: false,
            span: expr.value.span(),
        });

        let mut end_jumps = Vec::new();
        for arm in &expr.arms {
            let (_, variant_name) = self.pattern_variant_symbols(&arm.pattern);
            chunk.instructions.push(Instruction::LoadName {
                target: temp_ref,
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
                target: binding,
                mutable: false,
                is_update: false,
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

    fn pattern_binding(&self, pattern: &mir::MatchPattern) -> Option<(NameRef, Span)> {
        let mir::MatchPattern::Variant(pattern) = pattern;
        pattern
            .binding
            .as_ref()
            .map(|binding| (self.name_ref(binding.binding, binding.name), pattern.span))
    }

    fn pattern_variant_symbols(&self, pattern: &mir::MatchPattern) -> (Symbol, Symbol) {
        let mir::MatchPattern::Variant(pattern) = pattern;
        (pattern.enum_name, pattern.variant_name)
    }

    fn name_ref(&self, binding: BindingId, name: Symbol) -> NameRef {
        NameRef {
            binding,
            local: local_for_binding(binding),
            name,
        }
    }

    fn name_ref_for_ident_target(&self, target: mir::IdentTarget, name: Symbol) -> NameRef {
        let binding = match target {
            mir::IdentTarget::Binding(binding) => binding,
            mir::IdentTarget::PackageItem { binding, item } => self
                .package_function_bindings
                .get(&item)
                .copied()
                .unwrap_or(binding),
        };
        self.name_ref(binding, name)
    }

    fn synthetic_name_ref(&mut self, name: Symbol) -> NameRef {
        let local = LocalId::new(self.next_synthetic_local);
        let binding = BindingId::new(self.next_synthetic_local);
        self.next_synthetic_local += 1;
        NameRef {
            binding,
            local,
            name,
        }
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
