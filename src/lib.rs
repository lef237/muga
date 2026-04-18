pub mod ast;
pub mod bytecode;
pub mod diagnostic;
pub mod hir;
pub mod lexer;
pub mod parser;
pub mod resolver;
pub mod runtime;
pub mod span;
pub mod token;
pub mod typing;

use ast::Program;
use bytecode::Program as BytecodeProgram;
use diagnostic::Diagnostic;
use hir::Program as HirProgram;
use runtime::RunOutcome;

pub fn check_source(source: &str) -> Result<Program, Vec<Diagnostic>> {
    let tokens = lexer::lex(source)?;
    let program = parser::parse(tokens)?;

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

pub fn compile_bytecode_source(source: &str) -> Result<BytecodeProgram, Vec<Diagnostic>> {
    let program = compile_source(source)?;
    Ok(bytecode::compile(&program))
}

pub fn run_source(source: &str) -> Result<RunOutcome, Vec<Diagnostic>> {
    let program = compile_bytecode_source(source)?;
    runtime::run(&program)
}
