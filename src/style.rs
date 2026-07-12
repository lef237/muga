use std::collections::{HashMap, HashSet};

use crate::ast::{self, CallOrigin, Expr, Program, Stmt};
use crate::diagnostic::Diagnostic;
use crate::identity::{BindingId, BindingKind, ExprId};
use crate::typing::{TypeCheckOutput, TypedCalleeInfo};

/// Reports ordinary calls to named functions that have a first argument.
/// Calls through function values, zero-argument calls, and enum constructors
/// remain ordinary calls.
pub fn lint_call_style(program: &Program, types: &TypeCheckOutput) -> Vec<Diagnostic> {
    lint_call_style_with_source(program, types, "")
}

pub fn lint_call_style_with_source(
    program: &Program,
    types: &TypeCheckOutput,
    source: &str,
) -> Vec<Diagnostic> {
    let calls: HashMap<ExprId, TypedCalleeInfo> = types
        .calls
        .iter()
        .map(|call| (call.expr_id, call.callee))
        .collect();
    let binding_kinds: HashMap<BindingId, BindingKind> = types
        .bindings
        .iter()
        .map(|binding| (binding.id, binding.kind))
        .collect();
    let mut diagnostics = Vec::new();
    let suppressions = lint_suppressions(source);
    visit_statements(
        &program.statements,
        &calls,
        &binding_kinds,
        &suppressions,
        &mut diagnostics,
    );
    diagnostics
}

/// Rewrites lint-eligible ordinary calls to their canonical chained form.
/// Returns the number of rewritten calls.
pub fn fix_call_style(program: &mut Program, types: &TypeCheckOutput) -> usize {
    fix_call_style_with_source(program, types, "")
}

pub fn fix_call_style_with_source(
    program: &mut Program,
    types: &TypeCheckOutput,
    source: &str,
) -> usize {
    let binding_kinds: HashMap<BindingId, BindingKind> = types
        .bindings
        .iter()
        .map(|binding| (binding.id, binding.kind))
        .collect();
    let eligible: HashSet<ExprId> = types
        .calls
        .iter()
        .filter(|call| is_named_function(call.callee, &binding_kinds))
        .map(|call| call.expr_id)
        .collect();
    let enum_constructors: HashSet<ExprId> = types
        .calls
        .iter()
        .filter(|call| is_enum_constructor(call.callee))
        .map(|call| call.expr_id)
        .collect();
    let mut fixed = 0;
    let suppressions = lint_suppressions(source);
    fix_statements(
        &mut program.statements,
        &eligible,
        &enum_constructors,
        &suppressions,
        &mut fixed,
    );
    fixed
}

fn fix_statements(
    statements: &mut [Stmt],
    eligible: &HashSet<ExprId>,
    enum_constructors: &HashSet<ExprId>,
    suppressions: &HashSet<(usize, String)>,
    fixed: &mut usize,
) {
    for statement in statements {
        match statement {
            Stmt::Assign(stmt) => fix_expr(
                &mut stmt.value,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            ),
            Stmt::FuncDecl(stmt) => fix_value_block(
                &mut stmt.body,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            ),
            Stmt::If(stmt) => {
                fix_expr(
                    &mut stmt.condition,
                    eligible,
                    enum_constructors,
                    suppressions,
                    fixed,
                );
                fix_statements(
                    &mut stmt.then_branch.statements,
                    eligible,
                    enum_constructors,
                    suppressions,
                    fixed,
                );
                if let Some(branch) = &mut stmt.else_branch {
                    fix_statements(
                        &mut branch.statements,
                        eligible,
                        enum_constructors,
                        suppressions,
                        fixed,
                    );
                }
            }
            Stmt::While(stmt) => {
                fix_expr(
                    &mut stmt.condition,
                    eligible,
                    enum_constructors,
                    suppressions,
                    fixed,
                );
                fix_statements(
                    &mut stmt.body.statements,
                    eligible,
                    enum_constructors,
                    suppressions,
                    fixed,
                );
            }
            Stmt::For(stmt) => {
                fix_expr(
                    &mut stmt.iterable,
                    eligible,
                    enum_constructors,
                    suppressions,
                    fixed,
                );
                fix_statements(
                    &mut stmt.body.statements,
                    eligible,
                    enum_constructors,
                    suppressions,
                    fixed,
                );
            }
            Stmt::Using(stmt) => {
                fix_expr(
                    &mut stmt.value,
                    eligible,
                    enum_constructors,
                    suppressions,
                    fixed,
                );
                fix_statements(
                    &mut stmt.body.statements,
                    eligible,
                    enum_constructors,
                    suppressions,
                    fixed,
                );
            }
            Stmt::Return(stmt) => fix_expr(
                &mut stmt.value,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            ),
            Stmt::Expr(stmt) => fix_expr(
                &mut stmt.expr,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            ),
            Stmt::RecordDecl(_)
            | Stmt::EnumDecl(_)
            | Stmt::OpaqueTypeDecl(_)
            | Stmt::Break(_)
            | Stmt::Continue(_) => {}
        }
    }
}

fn fix_value_block(
    block: &mut ast::ValueBlock,
    eligible: &HashSet<ExprId>,
    enum_constructors: &HashSet<ExprId>,
    suppressions: &HashSet<(usize, String)>,
    fixed: &mut usize,
) {
    fix_statements(
        &mut block.statements,
        eligible,
        enum_constructors,
        suppressions,
        fixed,
    );
    fix_expr(
        &mut block.expr,
        eligible,
        enum_constructors,
        suppressions,
        fixed,
    );
}

fn fix_expr(
    expr: &mut Expr,
    eligible: &HashSet<ExprId>,
    enum_constructors: &HashSet<ExprId>,
    suppressions: &HashSet<(usize, String)>,
    fixed: &mut usize,
) {
    match expr {
        Expr::ListLit(expr) => fix_exprs(
            &mut expr.items,
            eligible,
            enum_constructors,
            suppressions,
            fixed,
        ),
        Expr::Index(expr) => {
            fix_expr(
                &mut expr.base,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            );
            fix_expr(
                &mut expr.index,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            );
        }
        Expr::RecordLit(expr) => {
            for field in &mut expr.fields {
                fix_expr(
                    &mut field.value,
                    eligible,
                    enum_constructors,
                    suppressions,
                    fixed,
                );
            }
        }
        Expr::Field(expr) => fix_expr(
            &mut expr.base,
            eligible,
            enum_constructors,
            suppressions,
            fixed,
        ),
        Expr::RecordUpdate(expr) => {
            fix_expr(
                &mut expr.base,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            );
            for field in &mut expr.fields {
                fix_expr(
                    &mut field.value,
                    eligible,
                    enum_constructors,
                    suppressions,
                    fixed,
                );
            }
        }
        Expr::Unary(expr) => fix_expr(
            &mut expr.expr,
            eligible,
            enum_constructors,
            suppressions,
            fixed,
        ),
        Expr::Binary(expr) => {
            fix_expr(
                &mut expr.left,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            );
            fix_expr(
                &mut expr.right,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            );
        }
        Expr::Call(expr) => {
            fix_expr(
                &mut expr.callee,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            );
            fix_exprs(
                &mut expr.args,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            );
            if expr.origin == CallOrigin::Ordinary
                && !expr.args.is_empty()
                && expr.type_args.is_empty()
                && eligible.contains(&expr.id)
                && !is_suppressed(suppressions, expr.span.start.line, "L001")
            {
                expr.origin = CallOrigin::Chained;
                *fixed += 1;
            } else if matches!(
                expr.origin,
                CallOrigin::Chained | CallOrigin::QualifiedChained
            ) && enum_constructors.contains(&expr.id)
                && !is_suppressed(suppressions, expr.span.start.line, "L003")
            {
                expr.origin = CallOrigin::Ordinary;
                *fixed += 1;
            }
        }
        Expr::Try(expr) => fix_expr(
            &mut expr.expr,
            eligible,
            enum_constructors,
            suppressions,
            fixed,
        ),
        Expr::If(expr) => {
            fix_expr(
                &mut expr.condition,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            );
            fix_value_block(
                &mut expr.then_branch,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            );
            fix_value_block(
                &mut expr.else_branch,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            );
        }
        Expr::Match(expr) => {
            fix_expr(
                &mut expr.value,
                eligible,
                enum_constructors,
                suppressions,
                fixed,
            );
            for arm in &mut expr.arms {
                fix_expr(
                    &mut arm.value,
                    eligible,
                    enum_constructors,
                    suppressions,
                    fixed,
                );
            }
        }
        Expr::Fn(expr) => fix_value_block(
            &mut expr.body,
            eligible,
            enum_constructors,
            suppressions,
            fixed,
        ),
        Expr::Group(expr) => fix_value_block(
            &mut expr.body,
            eligible,
            enum_constructors,
            suppressions,
            fixed,
        ),
        Expr::Spawn(expr) => fix_expr(
            &mut expr.expr,
            eligible,
            enum_constructors,
            suppressions,
            fixed,
        ),
        Expr::Int(_) | Expr::Bool(_) | Expr::String(_) | Expr::Unit(_) | Expr::Ident(_) => {}
    }
}

fn fix_exprs(
    expressions: &mut [Expr],
    eligible: &HashSet<ExprId>,
    enum_constructors: &HashSet<ExprId>,
    suppressions: &HashSet<(usize, String)>,
    fixed: &mut usize,
) {
    for expression in expressions {
        fix_expr(expression, eligible, enum_constructors, suppressions, fixed);
    }
}

fn visit_statements(
    statements: &[Stmt],
    calls: &HashMap<ExprId, TypedCalleeInfo>,
    binding_kinds: &HashMap<BindingId, BindingKind>,
    suppressions: &HashSet<(usize, String)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for statement in statements {
        match statement {
            Stmt::Assign(stmt) => {
                visit_expr(&stmt.value, calls, binding_kinds, suppressions, diagnostics)
            }
            Stmt::FuncDecl(stmt) => {
                visit_value_block(&stmt.body, calls, binding_kinds, suppressions, diagnostics)
            }
            Stmt::If(stmt) => {
                visit_expr(
                    &stmt.condition,
                    calls,
                    binding_kinds,
                    suppressions,
                    diagnostics,
                );
                visit_statements(
                    &stmt.then_branch.statements,
                    calls,
                    binding_kinds,
                    suppressions,
                    diagnostics,
                );
                if let Some(branch) = &stmt.else_branch {
                    visit_statements(
                        &branch.statements,
                        calls,
                        binding_kinds,
                        suppressions,
                        diagnostics,
                    );
                }
            }
            Stmt::While(stmt) => {
                visit_expr(
                    &stmt.condition,
                    calls,
                    binding_kinds,
                    suppressions,
                    diagnostics,
                );
                visit_statements(
                    &stmt.body.statements,
                    calls,
                    binding_kinds,
                    suppressions,
                    diagnostics,
                );
            }
            Stmt::For(stmt) => {
                visit_expr(
                    &stmt.iterable,
                    calls,
                    binding_kinds,
                    suppressions,
                    diagnostics,
                );
                visit_statements(
                    &stmt.body.statements,
                    calls,
                    binding_kinds,
                    suppressions,
                    diagnostics,
                );
            }
            Stmt::Using(stmt) => {
                visit_expr(&stmt.value, calls, binding_kinds, suppressions, diagnostics);
                visit_statements(
                    &stmt.body.statements,
                    calls,
                    binding_kinds,
                    suppressions,
                    diagnostics,
                );
            }
            Stmt::Return(stmt) => {
                visit_expr(&stmt.value, calls, binding_kinds, suppressions, diagnostics)
            }
            Stmt::Expr(stmt) => {
                visit_expr(&stmt.expr, calls, binding_kinds, suppressions, diagnostics)
            }
            Stmt::RecordDecl(_)
            | Stmt::EnumDecl(_)
            | Stmt::OpaqueTypeDecl(_)
            | Stmt::Break(_)
            | Stmt::Continue(_) => {}
        }
    }
}

fn visit_value_block(
    block: &ast::ValueBlock,
    calls: &HashMap<ExprId, TypedCalleeInfo>,
    binding_kinds: &HashMap<BindingId, BindingKind>,
    suppressions: &HashSet<(usize, String)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    visit_statements(
        &block.statements,
        calls,
        binding_kinds,
        suppressions,
        diagnostics,
    );
    visit_expr(&block.expr, calls, binding_kinds, suppressions, diagnostics);
}

fn visit_expr(
    expr: &Expr,
    calls: &HashMap<ExprId, TypedCalleeInfo>,
    binding_kinds: &HashMap<BindingId, BindingKind>,
    suppressions: &HashSet<(usize, String)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    match expr {
        Expr::ListLit(expr) => {
            visit_exprs(&expr.items, calls, binding_kinds, suppressions, diagnostics)
        }
        Expr::Index(expr) => {
            visit_expr(&expr.base, calls, binding_kinds, suppressions, diagnostics);
            visit_expr(&expr.index, calls, binding_kinds, suppressions, diagnostics);
        }
        Expr::RecordLit(expr) => {
            for field in &expr.fields {
                visit_expr(
                    &field.value,
                    calls,
                    binding_kinds,
                    suppressions,
                    diagnostics,
                );
            }
        }
        Expr::Field(expr) => {
            visit_expr(&expr.base, calls, binding_kinds, suppressions, diagnostics)
        }
        Expr::RecordUpdate(expr) => {
            visit_expr(&expr.base, calls, binding_kinds, suppressions, diagnostics);
            for field in &expr.fields {
                visit_expr(
                    &field.value,
                    calls,
                    binding_kinds,
                    suppressions,
                    diagnostics,
                );
            }
        }
        Expr::Unary(expr) => {
            visit_expr(&expr.expr, calls, binding_kinds, suppressions, diagnostics)
        }
        Expr::Binary(expr) => {
            visit_expr(&expr.left, calls, binding_kinds, suppressions, diagnostics);
            visit_expr(&expr.right, calls, binding_kinds, suppressions, diagnostics);
        }
        Expr::Call(expr) => {
            let resolved_callee = calls.get(&expr.id).copied();
            if expr.origin == CallOrigin::Ordinary
                && !expr.args.is_empty()
                && expr.type_args.is_empty()
                && resolved_callee.is_some_and(|callee| is_named_function(callee, binding_kinds))
                && !is_suppressed(suppressions, expr.span.start.line, "L001")
            {
                diagnostics.push(
                    Diagnostic::new(
                        "L001",
                        "named functions with arguments use chained-call syntax",
                        expr.span,
                    )
                    .with_suggestion(
                        "move the first argument before the function name as the chain receiver",
                    ),
                );
            } else if matches!(
                expr.origin,
                CallOrigin::Chained | CallOrigin::QualifiedChained
            ) && resolved_callee.is_some_and(is_enum_constructor)
                && !is_suppressed(suppressions, expr.span.start.line, "L003")
            {
                diagnostics.push(
                    Diagnostic::new(
                        "L003",
                        "enum constructors use ordinary-call syntax",
                        expr.span,
                    )
                    .with_suggestion(
                        "move the chain receiver into the enum constructor argument list",
                    ),
                );
            }
            visit_expr(
                &expr.callee,
                calls,
                binding_kinds,
                suppressions,
                diagnostics,
            );
            visit_exprs(&expr.args, calls, binding_kinds, suppressions, diagnostics);
        }
        Expr::Try(expr) => visit_expr(&expr.expr, calls, binding_kinds, suppressions, diagnostics),
        Expr::If(expr) => {
            visit_expr(
                &expr.condition,
                calls,
                binding_kinds,
                suppressions,
                diagnostics,
            );
            visit_value_block(
                &expr.then_branch,
                calls,
                binding_kinds,
                suppressions,
                diagnostics,
            );
            visit_value_block(
                &expr.else_branch,
                calls,
                binding_kinds,
                suppressions,
                diagnostics,
            );
        }
        Expr::Match(expr) => {
            visit_expr(&expr.value, calls, binding_kinds, suppressions, diagnostics);
            for arm in &expr.arms {
                visit_expr(&arm.value, calls, binding_kinds, suppressions, diagnostics);
            }
        }
        Expr::Fn(expr) => {
            visit_value_block(&expr.body, calls, binding_kinds, suppressions, diagnostics)
        }
        Expr::Group(expr) => {
            visit_value_block(&expr.body, calls, binding_kinds, suppressions, diagnostics)
        }
        Expr::Spawn(expr) => {
            visit_expr(&expr.expr, calls, binding_kinds, suppressions, diagnostics)
        }
        Expr::Int(_) | Expr::Bool(_) | Expr::String(_) | Expr::Unit(_) | Expr::Ident(_) => {}
    }
}

fn visit_exprs(
    expressions: &[Expr],
    calls: &HashMap<ExprId, TypedCalleeInfo>,
    binding_kinds: &HashMap<BindingId, BindingKind>,
    suppressions: &HashSet<(usize, String)>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for expression in expressions {
        visit_expr(expression, calls, binding_kinds, suppressions, diagnostics);
    }
}

fn is_named_function(
    callee: TypedCalleeInfo,
    binding_kinds: &HashMap<BindingId, BindingKind>,
) -> bool {
    if is_enum_constructor(callee) {
        return false;
    }
    let binding = match callee {
        TypedCalleeInfo::Binding(binding)
        | TypedCalleeInfo::PackageItem { binding, .. }
        | TypedCalleeInfo::Builtin { binding, .. } => binding,
        TypedCalleeInfo::EnumVariant { .. } | TypedCalleeInfo::Value | TypedCalleeInfo::Error => {
            return false;
        }
    };
    binding_kinds.get(&binding) == Some(&BindingKind::Function)
}

fn is_enum_constructor(callee: TypedCalleeInfo) -> bool {
    match callee {
        TypedCalleeInfo::EnumVariant { .. } => true,
        TypedCalleeInfo::Builtin { name, .. } => matches!(
            name,
            "Option::Some" | "Option::None" | "Result::Ok" | "Result::Err"
        ),
        TypedCalleeInfo::Binding(_)
        | TypedCalleeInfo::PackageItem { .. }
        | TypedCalleeInfo::Value
        | TypedCalleeInfo::Error => false,
    }
}

fn lint_suppressions(source: &str) -> HashSet<(usize, String)> {
    let mut suppressions = HashSet::new();
    for (index, line) in source.lines().enumerate() {
        let Some(codes) = line.trim().strip_prefix("// muga-lint: allow-next-line ") else {
            continue;
        };
        let codes = codes.split_once("--").map_or(codes, |(codes, _)| codes);
        for code in codes.split(',').flat_map(str::split_whitespace) {
            if !code.is_empty() {
                suppressions.insert((index + 2, code.to_ascii_uppercase()));
            }
        }
    }
    suppressions
}

fn is_suppressed(suppressions: &HashSet<(usize, String)>, line: usize, code: &str) -> bool {
    suppressions.contains(&(line, code.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{
        fix_call_style, fix_call_style_with_source, lint_call_style, lint_call_style_with_source,
    };
    use crate::{formatter, lexer, parser, typing};

    fn lint(source: &str) -> Vec<crate::diagnostic::Diagnostic> {
        let tokens = lexer::lex(source).expect("source should lex");
        let program = parser::parse(tokens).expect("source should parse");
        let types = typing::typecheck_program(&program);
        assert!(types.diagnostics.is_empty(), "{:#?}", types.diagnostics);
        lint_call_style(&program, &types)
    }

    #[test]
    fn reports_named_ordinary_calls_with_arguments() {
        let diagnostics = lint(
            r#"
fn inc(value: Int): Int { value + 1 }
fn main(): Int { inc(1) }
"#,
        );
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "L001");
    }

    #[test]
    fn accepts_chains_zero_argument_calls_and_function_values() {
        let diagnostics = lint(
            r#"
fn seed(): Int { 1 }
fn inc(value: Int): Int { value + 1 }
fn apply(value: Int, callback: Int -> Int): Int { callback(value) }
fn main(): Int { seed().inc().apply(inc) }
"#,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn accepts_enum_constructors_in_ordinary_form() {
        let diagnostics = lint(
            r#"
enum Value { Number(Int) }
fn main(): Value { Value::Number(1) }
"#,
        );
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn reports_and_fixes_builtin_enum_constructor_chains() {
        let source = "fn main(): Result[Int, String] { 1.Result::Ok() }\n";
        let tokens = lexer::lex(source).expect("source should lex");
        let mut program = parser::parse(tokens).expect("source should parse");
        let types = typing::typecheck_program(&program);
        let diagnostics = lint_call_style(&program, &types);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code, "L003");
        assert_eq!(fix_call_style(&mut program, &types), 1);
        let formatted = formatter::format_program_preserving_comments(&program, source);
        assert!(formatted.contains("Result::Ok(1)"), "{formatted}");
    }

    #[test]
    fn fixes_nested_named_calls_and_preserves_function_value_calls() {
        let source = r#"
fn inc(value: Int): Int { value + 1 }
fn apply(value: Int, callback: Int -> Int): Int { callback(value) }
fn main(): Int { apply(inc(1), inc) }
"#;
        let tokens = lexer::lex(source).expect("source should lex");
        let mut program = parser::parse(tokens).expect("source should parse");
        let types = typing::typecheck_program(&program);
        assert_eq!(fix_call_style(&mut program, &types), 2);
        let formatted = formatter::format_program_preserving_comments(&program, source);
        assert!(formatted.contains("1.inc().apply(inc)"), "{formatted}");
        assert!(formatted.contains("callback(value)"), "{formatted}");
    }

    #[test]
    fn allow_next_line_suppresses_lint_and_fix_only_on_the_next_line() {
        let source = r#"fn inc(value: Int): Int { value + 1 }
fn main(): Int {
  // muga-lint: allow-next-line L001 -- intentionally ordinary
  first = inc(1)
  inc(first)
}
"#;
        let tokens = lexer::lex(source).expect("source should lex");
        let mut program = parser::parse(tokens).expect("source should parse");
        let types = typing::typecheck_program(&program);
        let diagnostics = lint_call_style_with_source(&program, &types, source);
        assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
        assert_eq!(diagnostics[0].span.start.line, 5);

        assert_eq!(fix_call_style_with_source(&mut program, &types, source), 1);
        let formatted = formatter::format_program_preserving_comments(&program, source);
        assert!(formatted.contains("first = inc(1)"), "{formatted}");
        assert!(formatted.contains("first.inc()"), "{formatted}");
        assert!(
            formatted.contains("// muga-lint: allow-next-line L001 -- intentionally ordinary"),
            "{formatted}"
        );
    }
}
