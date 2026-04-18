pub mod ast;
pub mod diagnostic;
pub mod lexer;
pub mod parser;
pub mod resolver;
pub mod span;
pub mod token;
pub mod typing;

use ast::Program;
use diagnostic::Diagnostic;

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
