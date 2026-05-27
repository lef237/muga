use std::collections::HashMap;

use crate::{
    cli_schema::CliSchema,
    identity::{BindingId, BindingKind, LocalId, PackageItemId},
    json_decode::JsonDecodeSchema,
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
    LoadUnit,
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
    ListLen {
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
    DecodeJson {
        schema: JsonDecodeSchema,
        span: Span,
    },
    DecodeJsonRequired {
        schema: JsonDecodeSchema,
        span: Span,
    },
    JsonToValue {
        schema: JsonDecodeSchema,
        span: Span,
    },
    JsonEncodeTyped {
        schema: JsonDecodeSchema,
        span: Span,
    },
    LoadJsonConfigRequired {
        schema: JsonDecodeSchema,
        span: Span,
    },
    LoadJsonConfig {
        schema: JsonDecodeSchema,
        span: Span,
    },
    CliParse {
        schema: CliSchema,
        span: Span,
    },
    CliParseOr {
        schema: CliSchema,
        span: Span,
    },
    CliParseRequest {
        schema: CliSchema,
        span: Span,
    },
    CliParseRequestOr {
        schema: CliSchema,
        span: Span,
    },
    CliUsageFor {
        schema: CliSchema,
        span: Span,
    },
    CliUsageForRequired {
        schema: CliSchema,
        span: Span,
    },
    CliHelpFor {
        schema: CliSchema,
        span: Span,
    },
    CliHelpForRequired {
        schema: CliSchema,
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
            Instruction::LoadUnit => Instruction::LoadUnit,
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
            Instruction::ListLen { span } => Instruction::ListLen { span: *span },
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
            Instruction::DecodeJson { schema, span } => Instruction::DecodeJson {
                schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
                span: *span,
            },
            Instruction::DecodeJsonRequired { schema, span } => Instruction::DecodeJsonRequired {
                schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
                span: *span,
            },
            Instruction::JsonToValue { schema, span } => Instruction::JsonToValue {
                schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
                span: *span,
            },
            Instruction::JsonEncodeTyped { schema, span } => Instruction::JsonEncodeTyped {
                schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
                span: *span,
            },
            Instruction::LoadJsonConfigRequired { schema, span } => {
                Instruction::LoadJsonConfigRequired {
                    schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
                    span: *span,
                }
            }
            Instruction::LoadJsonConfig { schema, span } => Instruction::LoadJsonConfig {
                schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
                span: *span,
            },
            Instruction::CliParse { schema, span } => Instruction::CliParse {
                schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
                span: *span,
            },
            Instruction::CliParseOr { schema, span } => Instruction::CliParseOr {
                schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
                span: *span,
            },
            Instruction::CliParseRequest { schema, span } => Instruction::CliParseRequest {
                schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
                span: *span,
            },
            Instruction::CliParseRequestOr { schema, span } => Instruction::CliParseRequestOr {
                schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
                span: *span,
            },
            Instruction::CliUsageFor { schema, span } => Instruction::CliUsageFor {
                schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
                span: *span,
            },
            Instruction::CliUsageForRequired { schema, span } => Instruction::CliUsageForRequired {
                schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
                span: *span,
            },
            Instruction::CliHelpFor { schema, span } => Instruction::CliHelpFor {
                schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
                span: *span,
            },
            Instruction::CliHelpForRequired { schema, span } => Instruction::CliHelpForRequired {
                schema: schema.map_symbols(&mut |symbol| self.symbol(program, symbol)),
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
        if let Instruction::LoadName { target, .. } = instruction
            && let Some(item) = local_items.get(&target.local)
            && let Some(canonical) = canonical.get(item)
        {
            *target = *canonical;
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
    scope_depth: usize,
    loop_stack: Vec<LoopContext>,
    cleanup_stack: Vec<CleanupContext>,
}

#[derive(Clone, Debug)]
struct LoopContext {
    continue_target: Option<usize>,
    scope_depth: usize,
    break_jumps: Vec<usize>,
    continue_jumps: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
struct CleanupContext {
    scope_depth: usize,
    handle: NameRef,
    close: NameRef,
    result_enum: Symbol,
    ok_variant: Symbol,
    err_variant: Symbol,
    span: Span,
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
            scope_depth: 0,
            loop_stack: Vec::new(),
            cleanup_stack: Vec::new(),
        }
    }

    fn compile_entry_body(&mut self, body: &mir::Body) -> Chunk {
        debug_assert_eq!(self.scope_depth, 0);
        debug_assert!(self.loop_stack.is_empty());
        debug_assert!(self.cleanup_stack.is_empty());
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
        debug_assert_eq!(self.scope_depth, 0);
        debug_assert!(self.loop_stack.is_empty());
        debug_assert!(self.cleanup_stack.is_empty());
        chunk
    }

    fn compile_function(&mut self, function: &mir::Function) {
        debug_assert_eq!(self.scope_depth, 0);
        debug_assert!(self.loop_stack.is_empty());
        debug_assert!(self.cleanup_stack.is_empty());
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
        debug_assert_eq!(self.scope_depth, 0);
        debug_assert!(self.loop_stack.is_empty());
        debug_assert!(self.cleanup_stack.is_empty());
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
            mir::Stmt::For(stmt) => self.compile_for_stmt(stmt, chunk),
            mir::Stmt::Using(stmt) => self.compile_using_stmt(stmt, chunk),
            mir::Stmt::Break(_) => self.compile_break_stmt(chunk),
            mir::Stmt::Continue(_) => self.compile_continue_stmt(chunk),
            mir::Stmt::Return(stmt) => {
                self.compile_expr(&stmt.value, chunk);
                self.emit_scope_unwind_to(chunk, 0);
                chunk.instructions.push(Instruction::Return);
            }
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
        let loop_scope_depth = self.scope_depth;
        self.loop_stack.push(LoopContext {
            continue_target: Some(loop_start),
            scope_depth: loop_scope_depth,
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
        });
        self.compile_block(&stmt.body, chunk);
        chunk
            .instructions
            .push(Instruction::Jump { target: loop_start });
        let loop_end = chunk.instructions.len();
        let loop_context = self
            .loop_stack
            .pop()
            .expect("while compilation should have pushed a loop context");
        for break_jump in loop_context.break_jumps {
            self.patch_jump(chunk, break_jump, loop_end);
        }
        for continue_jump in loop_context.continue_jumps {
            self.patch_jump(chunk, continue_jump, loop_start);
        }
        self.patch_jump_if_false(chunk, exit_jump, loop_end);
    }

    fn compile_for_stmt(&mut self, stmt: &mir::ForStmt, chunk: &mut Chunk) {
        let list_symbol = self.temp_symbol("for_list");
        let index_symbol = self.temp_symbol("for_index");
        let list_ref = self.synthetic_name_ref(list_symbol);
        let index_ref = self.synthetic_name_ref(index_symbol);

        self.push_scope(chunk);
        self.compile_expr(&stmt.iterable, chunk);
        chunk.instructions.push(Instruction::Assign {
            target: list_ref,
            mutable: false,
            is_update: false,
            span: stmt.iterable.span(),
        });
        chunk.instructions.push(Instruction::LoadInt(0));
        chunk.instructions.push(Instruction::Assign {
            target: index_ref,
            mutable: true,
            is_update: false,
            span: stmt.span,
        });

        let loop_start = chunk.instructions.len();
        chunk.instructions.push(Instruction::LoadName {
            target: index_ref,
            span: stmt.span,
        });
        chunk.instructions.push(Instruction::LoadName {
            target: list_ref,
            span: stmt.iterable.span(),
        });
        chunk.instructions.push(Instruction::ListLen {
            span: stmt.iterable.span(),
        });
        chunk.instructions.push(Instruction::Binary {
            op: BinaryOp::Lt,
            span: stmt.span,
        });
        let exit_jump = self.emit_jump_if_false(chunk, stmt.span);

        let loop_scope_depth = self.scope_depth;
        self.loop_stack.push(LoopContext {
            continue_target: None,
            scope_depth: loop_scope_depth,
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
        });

        self.push_scope(chunk);
        self.compile_function_defs(&stmt.body.function_defs, chunk);
        chunk.instructions.push(Instruction::LoadName {
            target: list_ref,
            span: stmt.iterable.span(),
        });
        chunk.instructions.push(Instruction::LoadName {
            target: index_ref,
            span: stmt.span,
        });
        chunk.instructions.push(Instruction::LoadIndex {
            span: stmt.iterable.span(),
        });
        chunk.instructions.push(Instruction::Assign {
            target: self.name_ref(stmt.item_binding, stmt.item),
            mutable: false,
            is_update: false,
            span: stmt.span,
        });
        self.compile_scope_statements(&stmt.body.statements, chunk);
        self.pop_scope(chunk);

        let continue_target = chunk.instructions.len();
        let continue_jumps = {
            let loop_context = self
                .loop_stack
                .last_mut()
                .expect("for compilation should have pushed a loop context");
            loop_context.continue_target = Some(continue_target);
            std::mem::take(&mut loop_context.continue_jumps)
        };
        for continue_jump in continue_jumps {
            self.patch_jump(chunk, continue_jump, continue_target);
        }

        chunk.instructions.push(Instruction::LoadName {
            target: index_ref,
            span: stmt.span,
        });
        chunk.instructions.push(Instruction::LoadInt(1));
        chunk.instructions.push(Instruction::Binary {
            op: BinaryOp::Add,
            span: stmt.span,
        });
        chunk.instructions.push(Instruction::Assign {
            target: index_ref,
            mutable: true,
            is_update: true,
            span: stmt.span,
        });
        chunk
            .instructions
            .push(Instruction::Jump { target: loop_start });

        let loop_end = chunk.instructions.len();
        let loop_context = self
            .loop_stack
            .pop()
            .expect("for compilation should have pushed a loop context");
        for break_jump in loop_context.break_jumps {
            self.patch_jump(chunk, break_jump, loop_end);
        }
        for continue_jump in loop_context.continue_jumps {
            self.patch_jump(chunk, continue_jump, continue_target);
        }
        self.patch_jump_if_false(chunk, exit_jump, loop_end);
        self.pop_scope(chunk);
    }

    fn compile_using_stmt(&mut self, stmt: &mir::UsingStmt, chunk: &mut Chunk) {
        self.push_scope(chunk);
        self.compile_expr(&stmt.value, chunk);
        let handle = self.name_ref(stmt.binding, stmt.name);
        chunk.instructions.push(Instruction::Assign {
            target: handle,
            mutable: false,
            is_update: false,
            span: stmt.span,
        });
        let cleanup = CleanupContext {
            scope_depth: self.scope_depth,
            handle,
            close: self.name_ref_for_ident_target(stmt.cleanup.target, stmt.cleanup.name),
            result_enum: stmt.result_enum,
            ok_variant: stmt.ok_variant,
            err_variant: stmt.err_variant,
            span: stmt.cleanup.span,
        };
        self.cleanup_stack.push(cleanup);
        self.compile_block(&stmt.body, chunk);
        let popped = self
            .cleanup_stack
            .pop()
            .expect("using compilation should have pushed cleanup context");
        debug_assert_eq!(popped.scope_depth, cleanup.scope_depth);
        let remaining_cleanups = self.cleanup_stack.iter().rev().copied().collect::<Vec<_>>();
        self.emit_cleanup_call(chunk, cleanup, &remaining_cleanups);
        self.pop_scope(chunk);
    }

    fn compile_break_stmt(&mut self, chunk: &mut Chunk) {
        let Some(loop_index) = self.loop_stack.len().checked_sub(1) else {
            debug_assert!(false, "typechecker should reject `break` outside loops");
            return;
        };
        let target_scope_depth = self.loop_stack[loop_index].scope_depth;
        self.emit_scope_unwind_to(chunk, target_scope_depth);
        let jump = self.emit_jump(chunk);
        self.loop_stack[loop_index].break_jumps.push(jump);
    }

    fn compile_continue_stmt(&mut self, chunk: &mut Chunk) {
        let Some(loop_context) = self.loop_stack.last() else {
            debug_assert!(false, "typechecker should reject `continue` outside loops");
            return;
        };
        let target_scope_depth = loop_context.scope_depth;
        let continue_target = loop_context.continue_target;
        self.emit_scope_unwind_to(chunk, target_scope_depth);
        if let Some(continue_target) = continue_target {
            chunk.instructions.push(Instruction::Jump {
                target: continue_target,
            });
        } else {
            let jump = self.emit_jump(chunk);
            let loop_context = self
                .loop_stack
                .last_mut()
                .expect("typechecker should reject `continue` outside loops");
            loop_context.continue_jumps.push(jump);
        }
    }

    fn compile_block(&mut self, block: &mir::Block, chunk: &mut Chunk) {
        self.push_scope(chunk);
        self.compile_function_defs(&block.function_defs, chunk);
        self.compile_scope_statements(&block.statements, chunk);
        self.pop_scope(chunk);
    }

    fn compile_value_block(&mut self, block: &mir::ValueBlock, chunk: &mut Chunk) {
        self.push_scope(chunk);
        self.compile_function_defs(&block.function_defs, chunk);
        self.compile_scope_statements(&block.statements, chunk);
        self.compile_expr(&block.expr, chunk);
        self.pop_scope(chunk);
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
            mir::Expr::Unit(_) => chunk.instructions.push(Instruction::LoadUnit),
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
            mir::Expr::JsonDecodeOr(expr) => {
                self.compile_expr(&expr.value, chunk);
                self.compile_expr(&expr.fallback, chunk);
                chunk.instructions.push(Instruction::DecodeJson {
                    schema: expr.schema.clone(),
                    span: expr.span,
                });
            }
            mir::Expr::JsonDecode(expr) => {
                self.compile_expr(&expr.value, chunk);
                chunk.instructions.push(Instruction::DecodeJsonRequired {
                    schema: expr.schema.clone(),
                    span: expr.span,
                });
            }
            mir::Expr::JsonToValue(expr) => {
                self.compile_expr(&expr.value, chunk);
                chunk.instructions.push(Instruction::JsonToValue {
                    schema: expr.schema.clone(),
                    span: expr.span,
                });
            }
            mir::Expr::JsonEncodeTyped(expr) => {
                self.compile_expr(&expr.value, chunk);
                chunk.instructions.push(Instruction::JsonEncodeTyped {
                    schema: expr.schema.clone(),
                    span: expr.span,
                });
            }
            mir::Expr::ConfigLoadJson(expr) => {
                self.compile_expr(&expr.path, chunk);
                chunk
                    .instructions
                    .push(Instruction::LoadJsonConfigRequired {
                        schema: expr.schema.clone(),
                        span: expr.span,
                    });
            }
            mir::Expr::ConfigLoadJsonOr(expr) => {
                self.compile_expr(&expr.path, chunk);
                self.compile_expr(&expr.fallback, chunk);
                chunk.instructions.push(Instruction::LoadJsonConfig {
                    schema: expr.schema.clone(),
                    span: expr.span,
                });
            }
            mir::Expr::CliParseOr(expr) => {
                self.compile_expr(&expr.args, chunk);
                self.compile_expr(&expr.defaults, chunk);
                chunk.instructions.push(Instruction::CliParseOr {
                    schema: expr.schema.clone(),
                    span: expr.span,
                });
            }
            mir::Expr::CliParse(expr) => {
                self.compile_expr(&expr.args, chunk);
                chunk.instructions.push(Instruction::CliParse {
                    schema: expr.schema.clone(),
                    span: expr.span,
                });
            }
            mir::Expr::CliParseRequest(expr) => {
                self.compile_expr(&expr.args, chunk);
                self.compile_expr(&expr.program, chunk);
                chunk.instructions.push(Instruction::CliParseRequest {
                    schema: expr.schema.clone(),
                    span: expr.span,
                });
            }
            mir::Expr::CliParseRequestOr(expr) => {
                self.compile_expr(&expr.args, chunk);
                self.compile_expr(&expr.program, chunk);
                self.compile_expr(&expr.defaults, chunk);
                chunk.instructions.push(Instruction::CliParseRequestOr {
                    schema: expr.schema.clone(),
                    span: expr.span,
                });
            }
            mir::Expr::CliUsageFor(expr) => {
                self.compile_expr(&expr.program, chunk);
                self.compile_expr(&expr.defaults, chunk);
                chunk.instructions.push(Instruction::CliUsageFor {
                    schema: expr.schema.clone(),
                    span: expr.span,
                });
            }
            mir::Expr::CliUsageForRequired(expr) => {
                self.compile_expr(&expr.program, chunk);
                chunk.instructions.push(Instruction::CliUsageForRequired {
                    schema: expr.schema.clone(),
                    span: expr.span,
                });
            }
            mir::Expr::CliHelpFor(expr) => {
                self.compile_expr(&expr.program, chunk);
                self.compile_expr(&expr.defaults, chunk);
                chunk.instructions.push(Instruction::CliHelpFor {
                    schema: expr.schema.clone(),
                    span: expr.span,
                });
            }
            mir::Expr::CliHelpForRequired(expr) => {
                self.compile_expr(&expr.program, chunk);
                chunk.instructions.push(Instruction::CliHelpForRequired {
                    schema: expr.schema.clone(),
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
                match expr.op {
                    mir::BinaryOp::And => {
                        self.compile_and_expr(expr, chunk);
                        return;
                    }
                    mir::BinaryOp::Or => {
                        self.compile_or_expr(expr, chunk);
                        return;
                    }
                    _ => {}
                }
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
                        mir::BinaryOp::And | mir::BinaryOp::Or => {
                            unreachable!("short-circuit boolean operators are lowered with jumps")
                        }
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
            mir::Expr::Try(expr) => self.compile_try_expr(expr, chunk),
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

    fn compile_and_expr(&mut self, expr: &mir::BinaryExpr, chunk: &mut Chunk) {
        self.compile_expr(&expr.left, chunk);
        let false_jump = self.emit_jump_if_false(chunk, expr.left.span());
        self.compile_expr(&expr.right, chunk);
        let end_jump = self.emit_jump(chunk);
        let false_target = chunk.instructions.len();
        self.patch_jump_if_false(chunk, false_jump, false_target);
        chunk.instructions.push(Instruction::LoadBool(false));
        let end_target = chunk.instructions.len();
        self.patch_jump(chunk, end_jump, end_target);
    }

    fn compile_or_expr(&mut self, expr: &mir::BinaryExpr, chunk: &mut Chunk) {
        self.compile_expr(&expr.left, chunk);
        let right_jump = self.emit_jump_if_false(chunk, expr.left.span());
        chunk.instructions.push(Instruction::LoadBool(true));
        let end_jump = self.emit_jump(chunk);
        let right_target = chunk.instructions.len();
        self.patch_jump_if_false(chunk, right_jump, right_target);
        self.compile_expr(&expr.right, chunk);
        let end_target = chunk.instructions.len();
        self.patch_jump(chunk, end_jump, end_target);
    }

    fn compile_try_expr(&mut self, expr: &mir::TryExpr, chunk: &mut Chunk) {
        let temp = self.match_temp_symbol();
        let temp_ref = self.synthetic_name_ref(temp);
        self.push_scope(chunk);
        self.compile_expr(&expr.expr, chunk);
        chunk.instructions.push(Instruction::Assign {
            target: temp_ref,
            mutable: false,
            is_update: false,
            span: expr.expr.span(),
        });

        chunk.instructions.push(Instruction::LoadName {
            target: temp_ref,
            span: expr.expr.span(),
        });
        let err_jump = self.emit_jump_if_not_enum_variant(
            chunk,
            expr.result_enum,
            expr.ok_variant,
            expr.expr.span(),
        );
        let end_jump = self.emit_jump(chunk);

        let err_target = chunk.instructions.len();
        self.patch_jump_if_not_enum_variant(chunk, err_jump, err_target);
        chunk.instructions.push(Instruction::LoadName {
            target: temp_ref,
            span: expr.expr.span(),
        });
        let exhausted_jump = self.emit_jump_if_not_enum_variant(
            chunk,
            expr.result_enum,
            expr.err_variant,
            expr.expr.span(),
        );
        self.emit_scope_unwind_to(chunk, 0);
        chunk.instructions.push(Instruction::MakeEnum {
            enum_name: expr.result_enum,
            variant_name: expr.err_variant,
            has_payload: true,
            span: expr.span,
        });
        chunk.instructions.push(Instruction::Return);

        let exhausted_target = chunk.instructions.len();
        self.patch_jump_if_not_enum_variant(chunk, exhausted_jump, exhausted_target);
        chunk.instructions.push(Instruction::MatchExhausted {
            enum_name: expr.result_enum,
            span: expr.span,
        });

        let end_target = chunk.instructions.len();
        self.patch_jump(chunk, end_jump, end_target);
        self.pop_scope(chunk);
    }

    fn compile_match_expr(&mut self, expr: &mir::MatchExpr, chunk: &mut Chunk) {
        let enum_name = self.pattern_enum_symbol(expr);
        let temp = self.match_temp_symbol();
        let temp_ref = self.synthetic_name_ref(temp);
        self.push_scope(chunk);
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
        self.pop_scope(chunk);
    }

    fn compile_match_arm(&mut self, arm: &mir::MatchArm, chunk: &mut Chunk) {
        self.push_scope(chunk);
        match self.pattern_payload(&arm.pattern) {
            mir::EnumVariantPatternPayload::Binding(binding) => {
                chunk.instructions.push(Instruction::Assign {
                    target: self.name_ref(binding.binding, binding.name),
                    mutable: false,
                    is_update: false,
                    span: self.pattern_span(&arm.pattern),
                });
            }
            mir::EnumVariantPatternPayload::Discard => {
                chunk.instructions.push(Instruction::Pop);
            }
            mir::EnumVariantPatternPayload::None => {}
        }
        self.compile_expr(&arm.value, chunk);
        self.pop_scope(chunk);
    }

    fn pattern_enum_symbol(&self, expr: &mir::MatchExpr) -> Symbol {
        let first = expr
            .arms
            .first()
            .expect("typechecked match should have at least one arm");
        let mir::MatchPattern::Variant(pattern) = &first.pattern;
        pattern.enum_name
    }

    fn pattern_payload<'a>(
        &self,
        pattern: &'a mir::MatchPattern,
    ) -> &'a mir::EnumVariantPatternPayload {
        let mir::MatchPattern::Variant(pattern) = pattern;
        &pattern.payload
    }

    fn pattern_span(&self, pattern: &mir::MatchPattern) -> Span {
        let mir::MatchPattern::Variant(pattern) = pattern;
        pattern.span
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

    fn push_scope(&mut self, chunk: &mut Chunk) {
        chunk.instructions.push(Instruction::PushScope);
        self.scope_depth += 1;
    }

    fn pop_scope(&mut self, chunk: &mut Chunk) {
        debug_assert!(self.scope_depth > 0);
        self.scope_depth -= 1;
        chunk.instructions.push(Instruction::PopScope);
    }

    fn emit_scope_unwind_to(&mut self, chunk: &mut Chunk, target_depth: usize) {
        debug_assert!(target_depth <= self.scope_depth);
        let cleanups = self
            .cleanup_stack
            .iter()
            .rev()
            .copied()
            .filter(|cleanup| cleanup.scope_depth > target_depth)
            .collect::<Vec<_>>();
        self.emit_cleanup_unwind_sequence(chunk, &cleanups);
        for _ in target_depth..self.scope_depth {
            chunk.instructions.push(Instruction::PopScope);
        }
    }

    fn emit_cleanup_unwind_sequence(&mut self, chunk: &mut Chunk, cleanups: &[CleanupContext]) {
        let Some(first_cleanup) = cleanups.first().copied() else {
            return;
        };
        let failed_symbol = self.temp_symbol("using_cleanup_failed");
        let error_symbol = self.temp_symbol("using_cleanup_error");
        let failed = self.synthetic_name_ref(failed_symbol);
        let error = self.synthetic_name_ref(error_symbol);
        chunk.instructions.push(Instruction::LoadBool(false));
        chunk.instructions.push(Instruction::Assign {
            target: failed,
            mutable: true,
            is_update: false,
            span: first_cleanup.span,
        });

        for cleanup in cleanups {
            self.emit_cleanup_call_recording_first_error(chunk, *cleanup, failed, error);
        }

        chunk.instructions.push(Instruction::LoadName {
            target: failed,
            span: first_cleanup.span,
        });
        let no_error_jump = self.emit_jump_if_false(chunk, first_cleanup.span);
        chunk.instructions.push(Instruction::LoadName {
            target: error,
            span: first_cleanup.span,
        });
        chunk.instructions.push(Instruction::MakeEnum {
            enum_name: first_cleanup.result_enum,
            variant_name: first_cleanup.err_variant,
            has_payload: true,
            span: first_cleanup.span,
        });
        chunk.instructions.push(Instruction::Return);
        let no_error_target = chunk.instructions.len();
        self.patch_jump_if_false(chunk, no_error_jump, no_error_target);
    }

    fn emit_cleanup_call(
        &mut self,
        chunk: &mut Chunk,
        cleanup: CleanupContext,
        remaining_cleanups: &[CleanupContext],
    ) {
        let temp = self.temp_symbol("using_close");
        let temp_ref = self.synthetic_name_ref(temp);
        chunk.instructions.push(Instruction::LoadName {
            target: cleanup.close,
            span: cleanup.span,
        });
        chunk.instructions.push(Instruction::LoadName {
            target: cleanup.handle,
            span: cleanup.span,
        });
        chunk.instructions.push(Instruction::Call {
            argc: 1,
            span: cleanup.span,
        });
        chunk.instructions.push(Instruction::Assign {
            target: temp_ref,
            mutable: false,
            is_update: false,
            span: cleanup.span,
        });
        chunk.instructions.push(Instruction::LoadName {
            target: temp_ref,
            span: cleanup.span,
        });
        let err_jump = self.emit_jump_if_not_enum_variant(
            chunk,
            cleanup.result_enum,
            cleanup.ok_variant,
            cleanup.span,
        );
        chunk.instructions.push(Instruction::Pop);
        let end_jump = self.emit_jump(chunk);

        let err_target = chunk.instructions.len();
        self.patch_jump_if_not_enum_variant(chunk, err_jump, err_target);
        chunk.instructions.push(Instruction::LoadName {
            target: temp_ref,
            span: cleanup.span,
        });
        let exhausted_jump = self.emit_jump_if_not_enum_variant(
            chunk,
            cleanup.result_enum,
            cleanup.err_variant,
            cleanup.span,
        );
        self.emit_cleanup_ignore_errors_sequence(chunk, remaining_cleanups);
        chunk.instructions.push(Instruction::MakeEnum {
            enum_name: cleanup.result_enum,
            variant_name: cleanup.err_variant,
            has_payload: true,
            span: cleanup.span,
        });
        chunk.instructions.push(Instruction::Return);

        let exhausted_target = chunk.instructions.len();
        self.patch_jump_if_not_enum_variant(chunk, exhausted_jump, exhausted_target);
        chunk.instructions.push(Instruction::MatchExhausted {
            enum_name: cleanup.result_enum,
            span: cleanup.span,
        });

        let end_target = chunk.instructions.len();
        self.patch_jump(chunk, end_jump, end_target);
    }

    fn emit_cleanup_call_recording_first_error(
        &mut self,
        chunk: &mut Chunk,
        cleanup: CleanupContext,
        failed: NameRef,
        error: NameRef,
    ) {
        let temp = self.temp_symbol("using_close");
        let temp_ref = self.synthetic_name_ref(temp);
        chunk.instructions.push(Instruction::LoadName {
            target: cleanup.close,
            span: cleanup.span,
        });
        chunk.instructions.push(Instruction::LoadName {
            target: cleanup.handle,
            span: cleanup.span,
        });
        chunk.instructions.push(Instruction::Call {
            argc: 1,
            span: cleanup.span,
        });
        chunk.instructions.push(Instruction::Assign {
            target: temp_ref,
            mutable: false,
            is_update: false,
            span: cleanup.span,
        });
        chunk.instructions.push(Instruction::LoadName {
            target: temp_ref,
            span: cleanup.span,
        });
        let err_jump = self.emit_jump_if_not_enum_variant(
            chunk,
            cleanup.result_enum,
            cleanup.ok_variant,
            cleanup.span,
        );
        chunk.instructions.push(Instruction::Pop);
        let end_jump = self.emit_jump(chunk);

        let err_target = chunk.instructions.len();
        self.patch_jump_if_not_enum_variant(chunk, err_jump, err_target);
        chunk.instructions.push(Instruction::LoadName {
            target: temp_ref,
            span: cleanup.span,
        });
        let exhausted_jump = self.emit_jump_if_not_enum_variant(
            chunk,
            cleanup.result_enum,
            cleanup.err_variant,
            cleanup.span,
        );
        self.emit_record_first_cleanup_error(chunk, cleanup.span, failed, error);
        let err_end_jump = self.emit_jump(chunk);

        let exhausted_target = chunk.instructions.len();
        self.patch_jump_if_not_enum_variant(chunk, exhausted_jump, exhausted_target);
        chunk.instructions.push(Instruction::MatchExhausted {
            enum_name: cleanup.result_enum,
            span: cleanup.span,
        });

        let end_target = chunk.instructions.len();
        self.patch_jump(chunk, end_jump, end_target);
        self.patch_jump(chunk, err_end_jump, end_target);
    }

    fn emit_record_first_cleanup_error(
        &mut self,
        chunk: &mut Chunk,
        span: Span,
        failed: NameRef,
        error: NameRef,
    ) {
        chunk.instructions.push(Instruction::LoadName {
            target: failed,
            span,
        });
        let set_error_jump = self.emit_jump_if_false(chunk, span);
        chunk.instructions.push(Instruction::Pop);
        let end_jump = self.emit_jump(chunk);

        let set_error_target = chunk.instructions.len();
        self.patch_jump_if_false(chunk, set_error_jump, set_error_target);
        chunk.instructions.push(Instruction::Assign {
            target: error,
            mutable: false,
            is_update: false,
            span,
        });
        chunk.instructions.push(Instruction::LoadBool(true));
        chunk.instructions.push(Instruction::Assign {
            target: failed,
            mutable: true,
            is_update: true,
            span,
        });

        let end_target = chunk.instructions.len();
        self.patch_jump(chunk, end_jump, end_target);
    }

    fn emit_cleanup_ignore_errors_sequence(
        &mut self,
        chunk: &mut Chunk,
        cleanups: &[CleanupContext],
    ) {
        for cleanup in cleanups {
            self.emit_cleanup_call_ignoring_error(chunk, *cleanup);
        }
    }

    fn emit_cleanup_call_ignoring_error(&mut self, chunk: &mut Chunk, cleanup: CleanupContext) {
        let temp = self.temp_symbol("using_close");
        let temp_ref = self.synthetic_name_ref(temp);
        chunk.instructions.push(Instruction::LoadName {
            target: cleanup.close,
            span: cleanup.span,
        });
        chunk.instructions.push(Instruction::LoadName {
            target: cleanup.handle,
            span: cleanup.span,
        });
        chunk.instructions.push(Instruction::Call {
            argc: 1,
            span: cleanup.span,
        });
        chunk.instructions.push(Instruction::Assign {
            target: temp_ref,
            mutable: false,
            is_update: false,
            span: cleanup.span,
        });
        chunk.instructions.push(Instruction::LoadName {
            target: temp_ref,
            span: cleanup.span,
        });
        let err_jump = self.emit_jump_if_not_enum_variant(
            chunk,
            cleanup.result_enum,
            cleanup.ok_variant,
            cleanup.span,
        );
        chunk.instructions.push(Instruction::Pop);
        let end_jump = self.emit_jump(chunk);

        let err_target = chunk.instructions.len();
        self.patch_jump_if_not_enum_variant(chunk, err_jump, err_target);
        chunk.instructions.push(Instruction::LoadName {
            target: temp_ref,
            span: cleanup.span,
        });
        let exhausted_jump = self.emit_jump_if_not_enum_variant(
            chunk,
            cleanup.result_enum,
            cleanup.err_variant,
            cleanup.span,
        );
        chunk.instructions.push(Instruction::Pop);
        let err_end_jump = self.emit_jump(chunk);

        let exhausted_target = chunk.instructions.len();
        self.patch_jump_if_not_enum_variant(chunk, exhausted_jump, exhausted_target);
        chunk.instructions.push(Instruction::MatchExhausted {
            enum_name: cleanup.result_enum,
            span: cleanup.span,
        });

        let end_target = chunk.instructions.len();
        self.patch_jump(chunk, end_jump, end_target);
        self.patch_jump(chunk, err_end_jump, end_target);
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

    fn temp_symbol(&mut self, prefix: &str) -> Symbol {
        let name = format!("__muga_{prefix}_{}", self.next_match_temp);
        self.next_match_temp += 1;
        self.symbols.intern(&name)
    }

    fn match_temp_symbol(&mut self) -> Symbol {
        self.temp_symbol("match_value")
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
