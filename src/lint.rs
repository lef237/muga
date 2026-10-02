//! Lint registry, level configuration, and the warning rules run by
//! `muga lint`.
//!
//! Lints never change whether a program is accepted by `check`, `run`, or
//! `test`. `muga lint` reports each enabled lint as a warning or, at the
//! `deny` level, as an error that fails the command.

use std::collections::{HashMap, HashSet};

use crate::ast::{self, Block, EnumVariantPatternPayload, Expr, MatchPattern, Program, Stmt};
use crate::diagnostic::{Diagnostic, Severity};
use crate::identity::{BindingId, BindingKind, ExprId};
use crate::lexer;
use crate::span::{Position, Span};
use crate::style;
use crate::token::TokenKind;
use crate::types::TypeInfo;
use crate::typing::TypeCheckOutput;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LintLevel {
    Allow,
    Warn,
    Deny,
}

impl LintLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Allow => "allow",
            Self::Warn => "warn",
            Self::Deny => "deny",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LintInfo {
    pub code: &'static str,
    pub name: &'static str,
    pub default_level: LintLevel,
}

/// Every lint `muga lint` can report. Codes are the stable identifiers used by
/// level options and `// muga-lint: allow-next-line` suppressions.
pub const LINTS: &[LintInfo] = &[
    LintInfo {
        code: "S001",
        name: "chained-call-style",
        default_level: LintLevel::Deny,
    },
    LintInfo {
        code: "S003",
        name: "enum-constructor-call-style",
        default_level: LintLevel::Deny,
    },
    LintInfo {
        code: "W001",
        name: "unused-import",
        default_level: LintLevel::Warn,
    },
    LintInfo {
        code: "W002",
        name: "unused-binding",
        default_level: LintLevel::Warn,
    },
    LintInfo {
        code: "W003",
        name: "unused-parameter",
        default_level: LintLevel::Warn,
    },
    LintInfo {
        code: "W004",
        name: "unreachable-code",
        default_level: LintLevel::Warn,
    },
    LintInfo {
        code: "W005",
        name: "discarded-result",
        default_level: LintLevel::Warn,
    },
];

pub fn lint_info(code: &str) -> Option<&'static LintInfo> {
    LINTS.iter().find(|lint| lint.code == code)
}

/// Lint levels chosen on the command line. Later level settings for the same
/// code replace earlier ones; `deny_warnings` then turns every lint still at
/// the `warn` level into an error.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LintConfig {
    levels: HashMap<&'static str, LintLevel>,
    pub deny_warnings: bool,
}

impl LintConfig {
    pub fn set_level(&mut self, code: &str, level: LintLevel) -> Result<(), String> {
        let Some(lint) = lint_info(&code.to_ascii_uppercase()) else {
            let known: Vec<&str> = LINTS.iter().map(|lint| lint.code).collect();
            return Err(format!(
                "unknown lint code `{code}`; lint codes are {}",
                known.join(", ")
            ));
        };
        self.levels.insert(lint.code, level);
        Ok(())
    }

    pub fn level(&self, code: &str) -> LintLevel {
        let Some(lint) = lint_info(code) else {
            return LintLevel::Deny;
        };
        let level = self
            .levels
            .get(lint.code)
            .copied()
            .unwrap_or(lint.default_level);
        if level == LintLevel::Warn && self.deny_warnings {
            LintLevel::Deny
        } else {
            level
        }
    }

    /// Drops allowed lints and sets each remaining lint's severity from its
    /// configured level.
    pub fn apply(&self, diagnostics: Vec<Diagnostic>) -> Vec<Diagnostic> {
        diagnostics
            .into_iter()
            .filter_map(|diagnostic| match self.level(&diagnostic.code) {
                LintLevel::Allow => None,
                LintLevel::Warn => Some(diagnostic.with_severity(Severity::Warning)),
                LintLevel::Deny => Some(diagnostic.with_severity(Severity::Error)),
            })
            .collect()
    }
}

/// Runs every lint over one typechecked source module. The result is ordered
/// by source position, honors `allow-next-line` suppressions, and still needs
/// `LintConfig::apply` to receive its configured severities.
pub fn lint_module(program: &Program, types: &TypeCheckOutput, source: &str) -> Vec<Diagnostic> {
    let mut diagnostics = style::lint_call_style_with_source(program, types, source);
    diagnostics.extend(unused_imports(program, source));

    let mut walker = Walker {
        source,
        expr_types: types
            .expr_types
            .iter()
            .map(|info| (info.expr_id, &info.ty))
            .collect(),
        declarations: Vec::new(),
        diagnostics: Vec::new(),
    };
    walker.visit_statements(&program.statements, program.package.is_none());
    diagnostics.extend(walker.diagnostics);
    diagnostics.extend(unused_bindings(&walker.declarations, types, source));

    let suppressions = style::lint_suppressions(source);
    diagnostics.retain(|diagnostic| {
        !suppressions.contains(&(
            diagnostic.span.start.line,
            diagnostic.code.as_str().to_string(),
        ))
    });
    diagnostics
        .sort_by_key(|diagnostic| (diagnostic.span.start.line, diagnostic.span.start.column));
    diagnostics
}

fn unused_imports(program: &Program, source: &str) -> Vec<Diagnostic> {
    if program.imports.is_empty() {
        return Vec::new();
    }
    let Ok(tokens) = lexer::lex(source) else {
        return Vec::new();
    };
    let mut qualifiers = HashSet::new();
    let mut in_import = false;
    for (index, token) in tokens.iter().enumerate() {
        match &token.kind {
            TokenKind::Import => in_import = true,
            TokenKind::Newline => in_import = false,
            TokenKind::Ident(name)
                if !in_import
                    && tokens
                        .get(index + 1)
                        .is_some_and(|next| next.kind == TokenKind::DoubleColon) =>
            {
                qualifiers.insert(name.as_str());
            }
            _ => {}
        }
    }
    program
        .imports
        .iter()
        .filter(|import| !qualifiers.contains(import.alias.as_str()))
        .map(|import| {
            Diagnostic::new(
                "W001",
                format!("unused import `{}`", import.alias),
                import.span,
            )
            .with_replacement("remove the import", import.span, "")
        })
        .collect()
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum DeclarationKind {
    Binding,
    MutableBinding,
    Parameter,
}

/// A source-level name introduction that may be reported as unused.
struct Declaration<'a> {
    name: &'a str,
    span: Span,
    kind: DeclarationKind,
    /// Where the name itself starts, when it can be rewritten to `_`.
    name_start: Option<Position>,
}

fn unused_bindings(
    declarations: &[Declaration<'_>],
    types: &TypeCheckOutput,
    source: &str,
) -> Vec<Diagnostic> {
    let bindings: HashMap<(Span, &str), (BindingId, BindingKind)> = types
        .bindings
        .iter()
        .map(|binding| {
            (
                (binding.span, types.symbols.resolve(binding.symbol)),
                (binding.id, binding.kind),
            )
        })
        .collect();
    let used: HashSet<BindingId> = types
        .identifier_refs
        .iter()
        .map(|reference| reference.binding)
        .collect();

    let mut diagnostics = Vec::new();
    for declaration in declarations {
        if declaration.name.starts_with('_') {
            continue;
        }
        let Some((binding, kind)) = bindings.get(&(declaration.span, declaration.name)) else {
            continue;
        };
        let expected_kind = match declaration.kind {
            DeclarationKind::Binding => BindingKind::Immutable,
            DeclarationKind::MutableBinding => BindingKind::Mutable,
            DeclarationKind::Parameter => BindingKind::Parameter,
        };
        if *kind != expected_kind || used.contains(binding) {
            continue;
        }
        let name_span = declaration
            .name_start
            .and_then(|start| name_span_at(source, start, declaration.name));
        let diagnostic = match declaration.kind {
            DeclarationKind::Parameter => {
                let diagnostic = Diagnostic::new(
                    "W003",
                    format!("unused parameter `{}`", declaration.name),
                    declaration.span,
                );
                match name_span {
                    Some(span) => diagnostic.with_replacement(
                        "prefix the parameter with `_` if it is intentionally unused",
                        span,
                        format!("_{}", declaration.name),
                    ),
                    None => diagnostic.with_suggestion(
                        "prefix the parameter with `_` if it is intentionally unused",
                    ),
                }
            }
            DeclarationKind::Binding | DeclarationKind::MutableBinding => {
                let diagnostic = Diagnostic::new(
                    "W002",
                    format!("unused binding `{}`", declaration.name),
                    declaration.span,
                );
                match name_span {
                    Some(span) => diagnostic.with_replacement(
                        "use `_` if the value is intentionally unused",
                        span,
                        "_",
                    ),
                    None => diagnostic.with_suggestion(
                        "remove the binding, or prefix it with `_` if it is intentionally unused",
                    ),
                }
            }
        };
        diagnostics.push(diagnostic);
    }
    diagnostics
}

/// Returns the span of `name` at `start` when the source really spells it
/// there, so replacements never rewrite the wrong text.
fn name_span_at(source: &str, start: Position, name: &str) -> Option<Span> {
    let line = source.lines().nth(start.line.checked_sub(1)?)?;
    let rest: String = line.chars().skip(start.column.checked_sub(1)?).collect();
    let after = rest.strip_prefix(name)?;
    if after
        .chars()
        .next()
        .is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
    {
        return None;
    }
    let end = Position::new(start.line, start.column + name.chars().count());
    Some(Span::new(start, end))
}

/// Finds the column of `name` inside a single-line `span`, searching from the
/// end so a variant pattern's payload binding is found after its qualifier.
fn last_name_start(source: &str, span: Span, name: &str) -> Option<Position> {
    if span.start.line != span.end.line {
        return None;
    }
    let line = source.lines().nth(span.start.line.checked_sub(1)?)?;
    let chars: Vec<char> = line.chars().collect();
    let start = span.start.column.checked_sub(1)?;
    let end = span.end.column.checked_sub(1)?.min(chars.len());
    let text: String = chars.get(start..end)?.iter().collect();
    let byte_index = text.rfind(&format!("({name}"))? + 1;
    let column = text[..byte_index].chars().count() + span.start.column;
    Some(Position::new(span.start.line, column))
}

struct Walker<'a> {
    source: &'a str,
    expr_types: HashMap<ExprId, &'a TypeInfo>,
    declarations: Vec<Declaration<'a>>,
    diagnostics: Vec<Diagnostic>,
}

impl<'a> Walker<'a> {
    /// `track_bindings` is true inside function bodies and at script top
    /// level, where every binding is local to the file being linted.
    fn visit_statements(&mut self, statements: &'a [Stmt], track_bindings: bool) {
        self.report_unreachable(statements, None);
        for statement in statements {
            self.visit_statement(statement, track_bindings);
        }
    }

    fn visit_block(&mut self, block: &'a Block, track_bindings: bool) {
        self.visit_statements(&block.statements, track_bindings);
    }

    fn visit_value_block(&mut self, block: &'a ast::ValueBlock, track_bindings: bool) {
        let final_expr = (!block.terminal_return).then_some(block.expr.as_ref());
        self.report_unreachable(&block.statements, final_expr);
        for statement in &block.statements {
            self.visit_statement(statement, track_bindings);
        }
        self.visit_expr(&block.expr, track_bindings);
    }

    fn visit_statement(&mut self, statement: &'a Stmt, track_bindings: bool) {
        match statement {
            Stmt::Assign(stmt) => {
                if track_bindings && stmt.name != "_" {
                    self.declarations.push(Declaration {
                        name: &stmt.name,
                        span: stmt.span,
                        kind: if stmt.mutable {
                            DeclarationKind::MutableBinding
                        } else {
                            DeclarationKind::Binding
                        },
                        name_start: (!stmt.mutable).then_some(stmt.span.start),
                    });
                }
                self.visit_expr(&stmt.value, track_bindings);
            }
            Stmt::FuncDecl(stmt) => {
                self.visit_params(&stmt.params);
                self.visit_value_block(&stmt.body, true);
            }
            Stmt::If(stmt) => {
                self.visit_expr(&stmt.condition, track_bindings);
                self.visit_block(&stmt.then_branch, track_bindings);
                if let Some(branch) = &stmt.else_branch {
                    self.visit_block(branch, track_bindings);
                }
            }
            Stmt::While(stmt) => {
                self.visit_expr(&stmt.condition, track_bindings);
                self.visit_block(&stmt.body, track_bindings);
            }
            Stmt::For(stmt) => {
                if track_bindings {
                    self.declarations.push(Declaration {
                        name: &stmt.item,
                        span: stmt.item_span,
                        kind: DeclarationKind::Binding,
                        name_start: Some(stmt.item_span.start),
                    });
                }
                self.visit_expr(&stmt.iterable, track_bindings);
                self.visit_block(&stmt.body, track_bindings);
            }
            Stmt::Using(stmt) => {
                // A `using` binding is consumed by its cleanup call even when
                // the body never reads it.
                self.visit_expr(&stmt.value, track_bindings);
                self.visit_block(&stmt.body, track_bindings);
            }
            Stmt::Return(stmt) => self.visit_expr(&stmt.value, track_bindings),
            Stmt::Expr(stmt) => {
                if let Some(TypeInfo::Result(_, _)) = self.expr_types.get(&stmt.expr.id()) {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "W005",
                            "this `Result` is discarded without handling its error",
                            stmt.expr.span(),
                        )
                        .with_suggestion(
                            "handle it with `match`, propagate the error with `_ = try expr`, \
                             or discard it explicitly with `_ = expr`",
                        ),
                    );
                }
                self.visit_expr(&stmt.expr, track_bindings);
            }
            Stmt::RecordDecl(_)
            | Stmt::EnumDecl(_)
            | Stmt::OpaqueTypeDecl(_)
            | Stmt::Break(_)
            | Stmt::Continue(_) => {}
        }
    }

    fn visit_params(&mut self, params: &'a [ast::Param]) {
        for param in params {
            self.declarations.push(Declaration {
                name: &param.name,
                span: param.span,
                kind: DeclarationKind::Parameter,
                name_start: Some(param.span.start),
            });
        }
    }

    fn visit_exprs(&mut self, exprs: &'a [Expr], track_bindings: bool) {
        for expr in exprs {
            self.visit_expr(expr, track_bindings);
        }
    }

    fn visit_expr(&mut self, expr: &'a Expr, track_bindings: bool) {
        match expr {
            Expr::Int(_) | Expr::Bool(_) | Expr::String(_) | Expr::Unit(_) | Expr::Ident(_) => {}
            Expr::ListLit(expr) => self.visit_exprs(&expr.items, track_bindings),
            Expr::Index(expr) => {
                self.visit_expr(&expr.base, track_bindings);
                self.visit_expr(&expr.index, track_bindings);
            }
            Expr::RecordLit(expr) => {
                for field in &expr.fields {
                    self.visit_expr(&field.value, track_bindings);
                }
            }
            Expr::Field(expr) => self.visit_expr(&expr.base, track_bindings),
            Expr::RecordUpdate(expr) => {
                self.visit_expr(&expr.base, track_bindings);
                for field in &expr.fields {
                    self.visit_expr(&field.value, track_bindings);
                }
            }
            Expr::Unary(expr) => self.visit_expr(&expr.expr, track_bindings),
            Expr::Binary(expr) => {
                self.visit_expr(&expr.left, track_bindings);
                self.visit_expr(&expr.right, track_bindings);
            }
            Expr::Call(expr) => {
                self.visit_expr(&expr.callee, track_bindings);
                self.visit_exprs(&expr.args, track_bindings);
            }
            Expr::Try(expr) => self.visit_expr(&expr.expr, track_bindings),
            Expr::If(expr) => {
                self.visit_expr(&expr.condition, track_bindings);
                self.visit_value_block(&expr.then_branch, track_bindings);
                self.visit_value_block(&expr.else_branch, track_bindings);
            }
            Expr::Match(expr) => {
                self.visit_expr(&expr.value, track_bindings);
                for arm in &expr.arms {
                    let MatchPattern::Variant(pattern) = &arm.pattern;
                    if track_bindings
                        && let EnumVariantPatternPayload::Binding(name) = &pattern.payload
                    {
                        self.declarations.push(Declaration {
                            name,
                            span: pattern.span,
                            kind: DeclarationKind::Binding,
                            name_start: last_name_start(self.source, pattern.span, name),
                        });
                    }
                    self.visit_expr(&arm.value, track_bindings);
                }
            }
            Expr::Fn(expr) => {
                self.visit_params(&expr.params);
                self.visit_value_block(&expr.body, true);
            }
            Expr::Group(expr) => self.visit_value_block(&expr.body, track_bindings),
            Expr::Spawn(expr) => self.visit_expr(&expr.expr, track_bindings),
        }
    }

    /// Reports the code after the first `return`, `break`, or `continue` in a
    /// statement list, once per block.
    fn report_unreachable(&mut self, statements: &'a [Stmt], final_expr: Option<&'a Expr>) {
        let Some(exit) = statements.iter().position(|statement| {
            matches!(
                statement,
                Stmt::Return(_) | Stmt::Break(_) | Stmt::Continue(_)
            )
        }) else {
            return;
        };
        let rest = &statements[exit + 1..];
        let first = rest.first().map(Stmt::span).or(final_expr.map(Expr::span));
        let last = final_expr.map(Expr::span).or(rest.last().map(Stmt::span));
        let (Some(first), Some(last)) = (first, last) else {
            return;
        };
        let keyword = match &statements[exit] {
            Stmt::Return(_) => "return",
            Stmt::Break(_) => "break",
            _ => "continue",
        };
        self.diagnostics.push(
            Diagnostic::new("W004", "unreachable code", first.merge(last))
                .with_related(
                    format!("control flow leaves this block at `{keyword}`"),
                    statements[exit].span(),
                )
                .with_suggestion("remove the unreachable code or move it before the exit"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{LintConfig, LintLevel, lint_module};
    use crate::diagnostic::{Diagnostic, Severity};
    use crate::{lexer, parser, typing};

    fn lint(source: &str) -> Vec<Diagnostic> {
        let tokens = lexer::lex(source).expect("source should lex");
        let program = parser::parse(tokens).expect("source should parse");
        let types = typing::typecheck_program(&program);
        assert!(types.diagnostics.is_empty(), "{:#?}", types.diagnostics);
        LintConfig::default().apply(lint_module(&program, &types, source))
    }

    fn codes(diagnostics: &[Diagnostic]) -> Vec<(&str, usize)> {
        diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.span.start.line))
            .collect()
    }

    #[test]
    fn reports_unused_bindings_and_parameters_with_underscore_fixes() {
        let diagnostics = lint(
            r#"fn add(left: Int, right: Int, _unused: Int): Int {
  mut count = 0
  coutn = count + left
  total = left + right
  total
}
fn main(): Int { 1.add(2, 3) }
"#,
        );
        assert_eq!(codes(&diagnostics), vec![("W002", 3)], "{diagnostics:#?}");
        assert_eq!(diagnostics[0].severity, Severity::Warning);
        let fix = &diagnostics[0].suggestions[0];
        assert_eq!(fix.replacement.as_deref(), Some("_"));
        let span = fix.span.expect("binding fix should have a span");
        assert_eq!((span.start.column, span.end.column), (3, 8));
    }

    #[test]
    fn reports_unused_parameters_loop_items_and_match_payloads() {
        let diagnostics = lint(
            r#"fn pick(value: Result[Int, String], unused: Int): Int {
  for item in [1, 2] {
    _ = 0
  }
  match value {
    Result::Ok(number) => 1
    Result::Err(_) => 0
  }
}
fn main(): Int {
  callback = fn(first: Int, _second: Int): Int { 0 }
  _ = callback(1, 2)
  Result::Ok(1).pick(0)
}
"#,
        );
        assert_eq!(
            codes(&diagnostics),
            vec![("W003", 1), ("W002", 2), ("W002", 6), ("W003", 11),],
            "{diagnostics:#?}"
        );
        assert_eq!(
            diagnostics[0].suggestions[0].replacement.as_deref(),
            Some("_unused")
        );
        let payload_fix = diagnostics[2].suggestions[0]
            .span
            .expect("payload fix should have a span");
        assert_eq!((payload_fix.start.column, payload_fix.end.column), (16, 22));
    }

    #[test]
    fn script_top_level_bindings_are_tracked() {
        let diagnostics = lint("seed = 1\nlimit = 2\nlimit.println()\n");
        assert_eq!(codes(&diagnostics), vec![("W002", 1)], "{diagnostics:#?}");
    }

    #[test]
    fn reports_code_after_return_break_and_continue_once_per_block() {
        let diagnostics = lint(
            r#"fn main(): Int {
  for item in [1, 2] {
    if item > 1 {
      continue
      _ = item
    }
    break
    _ = 1
    _ = 2
  }
  return 0
  1
}
"#,
        );
        assert_eq!(
            codes(&diagnostics),
            vec![("W004", 5), ("W004", 8), ("W004", 12)],
            "{diagnostics:#?}"
        );
        assert_eq!(diagnostics[1].span.end.line, 9);
        assert_eq!(diagnostics[1].related[0].span.start.line, 7);
    }

    #[test]
    fn terminal_return_is_not_unreachable() {
        let diagnostics = lint("fn main(): Int {\n  if true {\n    return 1\n  }\n  return 2\n}\n");
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn reports_results_discarded_by_expression_statements_only() {
        let diagnostics = lint(
            r#"fn check(value: Int): Result[Int, String] { Result::Ok(value) }
fn run(): Result[Int, String] {
  if true {
    1.check()
  }
  _ = 2.check()
  _ = try 3.check()
  4.check()
}
fn main(): Int { 0 }
"#,
        );
        assert_eq!(codes(&diagnostics), vec![("W005", 4)], "{diagnostics:#?}");
    }

    #[test]
    fn allow_next_line_suppresses_warning_lints() {
        let diagnostics = lint(
            "fn main(): Int {\n  // muga-lint: allow-next-line W002 -- kept for debugging\n  unused = 1\n  0\n}\n",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    }

    #[test]
    fn levels_allow_warn_deny_and_deny_warnings() {
        let source =
            "fn inc(value: Int): Int { value + 1 }\nfn main(): Int {\n  unused = 1\n  inc(1)\n}\n";
        let tokens = lexer::lex(source).expect("source should lex");
        let program = parser::parse(tokens).expect("source should parse");
        let types = typing::typecheck_program(&program);
        let raw = lint_module(&program, &types, source);

        let defaults = LintConfig::default().apply(raw.clone());
        let severities: Vec<_> = defaults
            .iter()
            .map(|diagnostic| (diagnostic.code.as_str(), diagnostic.severity))
            .collect();
        assert_eq!(
            severities,
            vec![("W002", Severity::Warning), ("S001", Severity::Error)]
        );

        let mut config = LintConfig::default();
        config
            .set_level("s001", LintLevel::Warn)
            .expect("S001 is a lint");
        config
            .set_level("W002", LintLevel::Allow)
            .expect("W002 is a lint");
        let configured = config.apply(raw.clone());
        assert_eq!(codes(&configured), vec![("S001", 4)]);
        assert_eq!(configured[0].severity, Severity::Warning);

        config.deny_warnings = true;
        assert_eq!(config.apply(raw)[0].severity, Severity::Error);

        let error = LintConfig::default()
            .set_level("E001", LintLevel::Allow)
            .expect_err("compiler errors are not lints");
        assert!(error.contains("unknown lint code `E001`"), "{error}");
    }
}
