pub mod ast;
pub mod bytecode;
pub mod diagnostic;
pub mod hir;
pub mod identity;
pub mod lexer;
pub mod package;
pub mod parser;
pub mod resolver;
pub mod runtime;
pub mod span;
pub mod symbol;
pub mod token;
pub mod typed_hir;
pub mod typing;

use ast::Program;
use bytecode::Program as BytecodeProgram;
use diagnostic::Diagnostic;
use hir::Program as HirProgram;
use runtime::RunOutcome;
use std::path::Path;
use typed_hir::Program as TypedHirProgram;

pub fn check_source(source: &str) -> Result<Program, Vec<Diagnostic>> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;
    if program.package.is_some() {
        return Err(vec![Diagnostic::new(
            "PK001",
            "package mode requires a file-based entrypoint",
            Default::default(),
        )]);
    }

    let mut diagnostics = resolver::resolve(&program);
    diagnostics.extend(typing::typecheck(&program));

    if diagnostics.is_empty() {
        Ok(program)
    } else {
        Err(diagnostics)
    }
}

pub fn check_path(path: &Path) -> Result<Program, Vec<Diagnostic>> {
    let program = package::load_program_from_entry(path)?;
    let mut diagnostics = resolver::resolve(&program);
    diagnostics.extend(typing::typecheck(&program));

    if diagnostics.is_empty() {
        Ok(program)
    } else {
        Err(diagnostics)
    }
}

pub fn compile_source(source: &str) -> Result<HirProgram, Vec<Diagnostic>> {
    let program = check_source(source)?;
    Ok(hir::lower(&program))
}

pub fn compile_path(path: &Path) -> Result<HirProgram, Vec<Diagnostic>> {
    let program = check_path(path)?;
    Ok(hir::lower(&program))
}

pub fn compile_typed_source(source: &str) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;
    if program.package.is_some() {
        return Err(vec![Diagnostic::new(
            "PK001",
            "package mode requires a file-based entrypoint",
            Default::default(),
        )]);
    }

    let resolve_output = resolver::resolve_program(&program);
    let type_output = typing::typecheck_program(&program);
    let mut diagnostics = resolve_output.diagnostics;
    diagnostics.extend(type_output.diagnostics.clone());
    if diagnostics.is_empty() {
        Ok(typed_hir::lower(
            &program,
            &type_output,
            package::PackageSymbolGraph::default(),
        ))
    } else {
        Err(diagnostics)
    }
}

pub fn compile_typed_path(path: &Path) -> Result<TypedHirProgram, Vec<Diagnostic>> {
    let loaded = package::load_from_entry(path)?;
    let resolve_output = resolver::resolve_program(&loaded.program);
    let type_output = typing::typecheck_program(&loaded.program);
    let mut diagnostics = resolve_output.diagnostics;
    diagnostics.extend(type_output.diagnostics.clone());
    if diagnostics.is_empty() {
        Ok(typed_hir::lower(
            &loaded.program,
            &type_output,
            loaded.package_graph,
        ))
    } else {
        Err(diagnostics)
    }
}

pub fn compile_bytecode_source(source: &str) -> Result<BytecodeProgram, Vec<Diagnostic>> {
    let program = compile_source(source)?;
    Ok(bytecode::compile(program))
}

pub fn compile_bytecode_path(path: &Path) -> Result<BytecodeProgram, Vec<Diagnostic>> {
    let program = compile_path(path)?;
    Ok(bytecode::compile(program))
}

pub fn run_source(source: &str) -> Result<RunOutcome, Vec<Diagnostic>> {
    let program = compile_bytecode_source(source)?;
    runtime::run(&program)
}

pub fn run_path(path: &Path) -> Result<RunOutcome, Vec<Diagnostic>> {
    let program = compile_bytecode_path(path)?;
    runtime::run(&program)
}
