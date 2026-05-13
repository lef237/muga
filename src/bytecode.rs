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
    pub locals: Vec<LocalDef>,
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

#[derive(Clone, Debug)]
pub struct LocalDef {
    pub id: LocalId,
    pub binding: Option<BindingId>,
    pub name: Symbol,
    pub kind: LocalKind,
    pub package_item: Option<PackageItemId>,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalKind {
    Binding(BindingKind),
    Synthetic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NameRef {
    pub binding: Option<BindingId>,
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
    let locals: Vec<_> = bindings.iter().map(LocalDef::from).collect();
    let mut compiler = Compiler::new(symbols, next_synthetic_local(&locals), locals);
    let entry = compiler.compile_entry_body(&entry);
    for function in &functions {
        compiler.compile_function(function);
    }
    let local_count = compiler.next_synthetic_local as usize;
    let locals = compiler.locals;
    Program {
        entry,
        functions: compiler.functions,
        bindings,
        locals,
        main,
        local_count,
        symbols: compiler.symbols,
    }
}

pub fn merge(entry: Program, dependencies: Vec<Program>) -> Program {
    let mut merger = ProgramMerger::default();
    let mut dependency_entries = Vec::new();
    for dependency in dependencies {
        let index = merger.add_program(&dependency);
        dependency_entries.push(merger.import_chunk_at(index, &dependency, &dependency.entry));
        for function in &dependency.functions {
            merger.import_function_at(index, &dependency, function);
        }
    }
    let entry_index = merger.add_program(&entry);
    let entry_chunk = merger.import_chunk_at(entry_index, &entry, &entry.entry);
    for function in &entry.functions {
        merger.import_function_at(entry_index, &entry, function);
    }
    let mut instructions = Vec::new();
    for chunk in dependency_entries {
        append_chunk(&mut instructions, chunk);
    }
    append_chunk(&mut instructions, entry_chunk);
    let main = entry
        .main
        .map(|main| merger.name_ref_at(entry_index, &entry, main));
    let mut program = Program {
        entry: Chunk { instructions },
        functions: merger.functions,
        bindings: merger.bindings,
        locals: merger.locals,
        main,
        local_count: merger.next_local as usize,
        symbols: merger.symbols,
    };
    canonicalize_package_function_refs(&mut program);
    program
}

#[derive(Default)]
struct ProgramMerger {
    symbols: SymbolTable,
    bindings: Vec<BindingDef>,
    locals: Vec<LocalDef>,
    functions: Vec<Function>,
    binding_maps: Vec<BindingMap>,
    local_maps: Vec<LocalMap>,
    function_maps: Vec<FunctionMap>,
    next_binding: u32,
    next_local: u32,
    next_function: usize,
}

type BindingMap = HashMap<BindingId, BindingId>;
type LocalMap = HashMap<LocalId, LocalId>;
type FunctionMap = HashMap<FunctionId, FunctionId>;

impl ProgramMerger {
    fn add_program(&mut self, program: &Program) -> usize {
        let index = self.binding_maps.len();
        let mut binding_map = HashMap::new();
        let mut local_map = HashMap::new();
        let mut function_map = HashMap::new();

        for binding in &program.bindings {
            let mapped = BindingId::new(self.next_binding);
            self.next_binding += 1;
            binding_map.insert(binding.id, mapped);
        }
        for local in &program.locals {
            let mapped = LocalId::new(self.next_local);
            self.next_local += 1;
            local_map.insert(local.id, mapped);
        }
        for function in &program.functions {
            let mapped = self.next_function;
            self.next_function += 1;
            function_map.insert(function.id, mapped);
        }

        for binding in &program.bindings {
            let name = self.symbol(program, binding.name);
            self.bindings.push(BindingDef {
                id: binding_map[&binding.id],
                local: local_map[&binding.local],
                name,
                kind: binding.kind,
                package_item: binding.package_item,
                span: binding.span,
            });
        }
        for local in &program.locals {
            let name = self.symbol(program, local.name);
            self.locals.push(LocalDef {
                id: local_map[&local.id],
                binding: local.binding.map(|binding| binding_map[&binding]),
                name,
                kind: local.kind,
                package_item: local.package_item,
                span: local.span,
            });
        }

        self.binding_maps.push(binding_map);
        self.local_maps.push(local_map);
        self.function_maps.push(function_map);
        index
    }

    fn import_chunk_at(&mut self, index: usize, program: &Program, chunk: &Chunk) -> Chunk {
        Chunk {
            instructions: chunk
                .instructions
                .iter()
                .map(|instruction| self.instruction(index, program, instruction))
                .collect(),
        }
    }

    fn import_function_at(&mut self, index: usize, program: &Program, function: &Function) {
        let id = self.function_maps[index][&function.id];
        if self.functions.len() <= id {
            self.functions.resize_with(id + 1, placeholder_function);
        }
        self.functions[id] = Function {
            id,
            name: function.name.map(|name| self.symbol(program, name)),
            params: function
                .params
                .iter()
                .map(|param| self.name_ref_at(index, program, *param))
                .collect(),
            chunk: self.import_chunk_at(index, program, &function.chunk),
            span: function.span,
        };
    }

    fn instruction(
        &mut self,
        index: usize,
        program: &Program,
        instruction: &Instruction,
    ) -> Instruction {
        match instruction {
            Instruction::LoadInt(value) => Instruction::LoadInt(*value),
            Instruction::LoadBool(value) => Instruction::LoadBool(*value),
            Instruction::LoadString(value) => Instruction::LoadString(value.clone()),
            Instruction::MakeRecord {
                type_name,
                fields,
                span,
            } => Instruction::MakeRecord {
                type_name: self.symbol(program, *type_name),
                fields: fields
                    .iter()
                    .map(|field| self.symbol(program, *field))
                    .collect(),
                span: *span,
            },
            Instruction::MakeEnum {
                enum_name,
                variant_name,
                has_payload,
                span,
            } => Instruction::MakeEnum {
                enum_name: self.symbol(program, *enum_name),
                variant_name: self.symbol(program, *variant_name),
                has_payload: *has_payload,
                span: *span,
            },
            Instruction::MakeList { len, span } => Instruction::MakeList {
                len: *len,
                span: *span,
            },
            Instruction::LoadName { target, span } => Instruction::LoadName {
                target: self.name_ref_at(index, program, *target),
                span: *span,
            },
            Instruction::LoadField { field, span } => Instruction::LoadField {
                field: self.symbol(program, *field),
                span: *span,
            },
            Instruction::LoadIndex { span } => Instruction::LoadIndex { span: *span },
            Instruction::UpdateRecord { fields, span } => Instruction::UpdateRecord {
                fields: fields
                    .iter()
                    .map(|field| self.symbol(program, *field))
                    .collect(),
                span: *span,
            },
            Instruction::Assign {
                target,
                mutable,
                is_update,
                span,
            } => Instruction::Assign {
                target: self.name_ref_at(index, program, *target),
                mutable: *mutable,
                is_update: *is_update,
                span: *span,
            },
            Instruction::DefineFunction {
                target,
                function,
                span,
            } => Instruction::DefineFunction {
                target: self.name_ref_at(index, program, *target),
                function: self.function_maps[index][function],
                span: *span,
            },
            Instruction::MakeClosure { function } => Instruction::MakeClosure {
                function: self.function_maps[index][function],
            },
            Instruction::UnaryNeg { span } => Instruction::UnaryNeg { span: *span },
            Instruction::UnaryNot { span } => Instruction::UnaryNot { span: *span },
            Instruction::Binary { op, span } => Instruction::Binary {
                op: *op,
                span: *span,
            },
            Instruction::Call { argc, span } => Instruction::Call {
                argc: *argc,
                span: *span,
            },
            Instruction::JumpIfFalse { target, span } => Instruction::JumpIfFalse {
                target: *target,
                span: *span,
            },
            Instruction::JumpIfNotEnumVariant {
                enum_name,
                variant_name,
                target,
                span,
            } => Instruction::JumpIfNotEnumVariant {
                enum_name: self.symbol(program, *enum_name),
                variant_name: self.symbol(program, *variant_name),
                target: *target,
                span: *span,
            },
            Instruction::MatchExhausted { enum_name, span } => Instruction::MatchExhausted {
                enum_name: self.symbol(program, *enum_name),
                span: *span,
            },
            Instruction::Jump { target } => Instruction::Jump { target: *target },
            Instruction::PushScope => Instruction::PushScope,
            Instruction::PopScope => Instruction::PopScope,
            Instruction::Pop => Instruction::Pop,
            Instruction::Return => Instruction::Return,
        }
    }

    fn name_ref_at(&mut self, index: usize, program: &Program, name_ref: NameRef) -> NameRef {
        NameRef {
            binding: name_ref
                .binding
                .map(|binding| self.binding_maps[index][&binding]),
            local: self.local_maps[index][&name_ref.local],
            name: self.symbol(program, name_ref.name),
        }
    }

    fn symbol(&mut self, program: &Program, symbol: Symbol) -> Symbol {
        self.symbols.intern(program.symbols.resolve(symbol))
    }
}

fn canonicalize_package_function_refs(program: &mut Program) {
    let local_items = program
        .locals
        .iter()
        .filter_map(|local| local.package_item.map(|item| (local.id, item)))
        .collect::<HashMap<_, _>>();
    let mut canonical = HashMap::new();
    for instruction in &program.entry.instructions {
        if let Instruction::DefineFunction { target, .. } = instruction
            && let Some(item) = local_items.get(&target.local)
        {
            canonical.entry(*item).or_insert(*target);
        }
    }
    for function in &program.functions {
        for instruction in &function.chunk.instructions {
            if let Instruction::DefineFunction { target, .. } = instruction
                && let Some(item) = local_items.get(&target.local)
            {
                canonical.entry(*item).or_insert(*target);
            }
        }
    }
    rewrite_chunk_package_refs(&mut program.entry, &local_items, &canonical);
    for function in &mut program.functions {
        rewrite_chunk_package_refs(&mut function.chunk, &local_items, &canonical);
    }
    if let Some(main) = program.main
        && let Some(item) = local_items.get(&main.local)
        && let Some(target) = canonical.get(item)
    {
        program.main = Some(*target);
    }
}

fn rewrite_chunk_package_refs(
    chunk: &mut Chunk,
    local_items: &HashMap<LocalId, PackageItemId>,
    canonical: &HashMap<PackageItemId, NameRef>,
) {
    for instruction in &mut chunk.instructions {
        if let Instruction::LoadName { target, .. } = instruction {
            if let Some(item) = local_items.get(&target.local)
                && let Some(canonical) = canonical.get(item)
            {
                *target = *canonical;
            }
        }
    }
}

fn append_chunk(instructions: &mut Vec<Instruction>, chunk: Chunk) {
    let offset = instructions.len();
    instructions.extend(
        chunk
            .instructions
            .into_iter()
            .map(|instruction| offset_jump_targets(instruction, offset)),
    );
}

fn offset_jump_targets(instruction: Instruction, offset: usize) -> Instruction {
    match instruction {
        Instruction::JumpIfFalse { target, span } => Instruction::JumpIfFalse {
            target: target + offset,
            span,
        },
        Instruction::JumpIfNotEnumVariant {
            enum_name,
            variant_name,
            target,
            span,
        } => Instruction::JumpIfNotEnumVariant {
            enum_name,
            variant_name,
            target: target + offset,
            span,
        },
        Instruction::Jump { target } => Instruction::Jump {
            target: target + offset,
        },
        instruction => instruction,
    }
}

fn next_synthetic_local(locals: &[LocalDef]) -> u32 {
    locals
        .iter()
        .map(|local| local.id.as_u32())
        .max()
        .map_or(0, |id| id + 1)
}

fn entry_main_ref(entry: &mir::Body, symbols: &SymbolTable) -> Option<NameRef> {
    entry
        .function_defs
        .iter()
        .find_map(|function| {
            (symbols.resolve(function.name) == "main").then_some(NameRef {
                binding: Some(function.binding),
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
                            binding: Some(statement.binding),
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

impl From<&BindingDef> for LocalDef {
    fn from(binding: &BindingDef) -> Self {
        Self {
            id: binding.local,
            binding: Some(binding.id),
            name: binding.name,
            kind: LocalKind::Binding(binding.kind),
            package_item: binding.package_item,
            span: binding.span,
        }
    }
}

struct Compiler {
    functions: Vec<Function>,
    symbols: SymbolTable,
    locals: Vec<LocalDef>,
    package_function_bindings: HashMap<PackageItemId, BindingId>,
    next_match_temp: usize,
    next_synthetic_local: u32,
}

impl Compiler {
    fn new(symbols: SymbolTable, next_synthetic_local: u32, locals: Vec<LocalDef>) -> Self {
        Self {
            functions: Vec::new(),
            symbols,
            locals,
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
            binding: Some(binding),
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
        self.next_synthetic_local += 1;
        self.locals.push(LocalDef {
            id: local,
            binding: None,
            name,
            kind: LocalKind::Synthetic,
            package_item: None,
            span: Span::default(),
        });
        NameRef {
            binding: None,
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
