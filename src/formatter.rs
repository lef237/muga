use crate::ast::*;
use std::{
    cell::RefCell,
    collections::{BTreeMap, HashSet, VecDeque},
};

const INDENT: &str = "  ";

const PREC_LOWEST: u8 = 0;
const PREC_OR: u8 = 1;
const PREC_AND: u8 = 2;
const PREC_EQUALITY: u8 = 3;
const PREC_COMPARISON: u8 = 4;
const PREC_ADDITIVE: u8 = 5;
const PREC_MULTIPLICATIVE: u8 = 6;
const PREC_UNARY: u8 = 7;
const PREC_POSTFIX: u8 = 8;
const PREC_PRIMARY: u8 = 9;

pub fn format_program(program: &Program) -> String {
    let formatter = Formatter::default();
    format_program_with_formatter(program, &formatter)
}

pub fn format_program_preserving_comments(program: &Program, source: &str) -> String {
    let formatter = Formatter::with_comments(CommentStore::from_source(source));
    format_program_with_formatter(program, &formatter)
}

fn format_program_with_formatter(program: &Program, formatter: &Formatter) -> String {
    let mut out = String::new();

    if let Some(package) = &program.package {
        out.push_str(&formatter.format_own_comments_before(package.span.start.line, 0));
        let line =
            formatter.append_trailing(package.span.end.line, format!("package {}", package.path));
        out.push_str(&line);
        out.push('\n');
        if !program.imports.is_empty() || !program.statements.is_empty() {
            out.push('\n');
        }
    }

    for (index, import) in program.imports.iter().enumerate() {
        out.push_str(&formatter.format_own_comments_before(import.span.start.line, 0));
        out.push_str(&formatter.format_import(import));
        out.push('\n');
        if index + 1 == program.imports.len() && !program.statements.is_empty() {
            out.push('\n');
        }
    }

    for (index, statement) in program.statements.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&formatter.format_own_comments_before(stmt_comment_start_line(statement), 0));
        out.push_str(&formatter.format_stmt(statement, 0));
        out.push('\n');
    }

    out.push_str(&formatter.format_all_remaining_comments(0));

    if out.is_empty() || !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[derive(Default)]
struct Formatter {
    comments: RefCell<CommentStore>,
    suppressed_trailing_lines: RefCell<Vec<usize>>,
}

impl Formatter {
    fn with_comments(comments: CommentStore) -> Self {
        Self {
            comments: RefCell::new(comments),
            suppressed_trailing_lines: RefCell::default(),
        }
    }
}

impl Formatter {
    fn format_import(&self, import: &ImportDecl) -> String {
        let mut out = String::from("import ");
        out.push_str(&import.path);
        let default_alias = import
            .path
            .rsplit("::")
            .next()
            .expect("import path has a segment");
        if import.alias != default_alias {
            out.push_str(" as ");
            out.push_str(&import.alias);
        }
        self.append_trailing(import.span.end.line, out)
    }

    fn format_stmt(&self, stmt: &Stmt, indent: usize) -> String {
        format!(
            "{}{}",
            self.indent(indent),
            self.format_stmt_inner(stmt, indent)
        )
    }

    fn format_stmt_inner(&self, stmt: &Stmt, indent: usize) -> String {
        match stmt {
            Stmt::Assign(stmt) => {
                let mut out = String::new();
                if stmt.mutable {
                    out.push_str("mut ");
                }
                out.push_str(&stmt.name);
                if let Some(type_name) = &stmt.type_name {
                    out.push_str(": ");
                    out.push_str(&self.format_type(type_name));
                }
                out.push_str(" = ");
                out.push_str(&self.format_expr(&stmt.value, PREC_LOWEST, indent));
                self.append_trailing(stmt.span.end.line, out)
            }
            Stmt::RecordDecl(stmt) => self.format_record_decl(stmt, indent),
            Stmt::EnumDecl(stmt) => self.format_enum_decl(stmt, indent),
            Stmt::OpaqueTypeDecl(stmt) => self.format_opaque_type_decl(stmt),
            Stmt::FuncDecl(stmt) => self.format_func_decl(stmt, indent),
            Stmt::If(stmt) => self.format_if_stmt(stmt, indent),
            Stmt::While(stmt) => format!(
                "while {} {}",
                self.format_expr(&stmt.condition, PREC_LOWEST, indent),
                self.format_block(&stmt.body, indent)
            ),
            Stmt::For(stmt) => format!(
                "for {} in {} {}",
                stmt.item,
                self.format_expr(&stmt.iterable, PREC_LOWEST, indent),
                self.format_block(&stmt.body, indent)
            ),
            Stmt::Using(stmt) => format!(
                "using {} = {} {}",
                stmt.name,
                self.format_expr(&stmt.value, PREC_LOWEST, indent),
                self.format_block(&stmt.body, indent)
            ),
            Stmt::Break(stmt) => self.append_trailing(stmt.span.end.line, "break".to_string()),
            Stmt::Continue(stmt) => {
                self.append_trailing(stmt.span.end.line, "continue".to_string())
            }
            Stmt::Return(stmt) => self.append_trailing(
                stmt.span.end.line,
                format!(
                    "return {}",
                    self.format_expr(&stmt.value, PREC_LOWEST, indent)
                ),
            ),
            Stmt::Expr(stmt) => self.append_trailing(
                stmt.span.end.line,
                self.format_expr(&stmt.expr, PREC_LOWEST, indent),
            ),
        }
    }

    fn format_record_decl(&self, stmt: &RecordDecl, indent: usize) -> String {
        let mut out = String::new();
        out.push_str(&self.format_attributes(&stmt.attributes, indent));
        self.push_visibility(&mut out, stmt.visibility);
        out.push_str("record ");
        out.push_str(&stmt.name);
        self.push_type_params(&mut out, &stmt.type_params);
        out.push_str(" {\n");
        out.push_str(&self.format_block_open_trailing_as_own(
            stmt.span.start.line,
            stmt.span.end.line,
            indent + 1,
        ));
        let mut prev_line = stmt.span.start.line;
        for (index, field) in stmt.fields.iter().enumerate() {
            let target_line = field
                .attributes
                .first()
                .map(|attribute| attribute.span.start.line)
                .unwrap_or(field.span.start.line);
            if index > 0 && self.has_gap_before(prev_line, target_line) {
                out.push('\n');
            }
            if field.attributes.is_empty() {
                out.push_str(&self.format_own_comments_before(field.span.start.line, indent + 1));
            } else {
                out.push_str(&self.format_attributes(&field.attributes, indent + 1));
            }
            out.push_str(&self.indent(indent + 1));
            out.push_str(&field.name);
            out.push_str(": ");
            out.push_str(&self.format_type(&field.type_name));
            if field.span.end.line != stmt.span.end.line {
                out = self.append_trailing(field.span.end.line, out);
            }
            out.push('\n');
            prev_line = field.span.end.line;
        }
        out.push_str(&self.format_own_comments_before(stmt.span.end.line, indent + 1));
        out.push_str(&self.indent(indent));
        out.push('}');
        self.append_trailing_to_last_line(stmt.span.end.line, out)
    }

    fn format_enum_decl(&self, stmt: &EnumDecl, indent: usize) -> String {
        let mut out = String::new();
        out.push_str(&self.format_attributes(&stmt.attributes, indent));
        self.push_visibility(&mut out, stmt.visibility);
        out.push_str("enum ");
        out.push_str(&stmt.name);
        self.push_type_params(&mut out, &stmt.type_params);
        out.push_str(" {\n");
        out.push_str(&self.format_block_open_trailing_as_own(
            stmt.span.start.line,
            stmt.span.end.line,
            indent + 1,
        ));
        let mut prev_line = stmt.span.start.line;
        for (index, variant) in stmt.variants.iter().enumerate() {
            let target_line = variant
                .attributes
                .first()
                .map(|attribute| attribute.span.start.line)
                .unwrap_or(variant.span.start.line);
            if index > 0 && self.has_gap_before(prev_line, target_line) {
                out.push('\n');
            }
            if variant.attributes.is_empty() {
                out.push_str(&self.format_own_comments_before(variant.span.start.line, indent + 1));
            } else {
                out.push_str(&self.format_attributes(&variant.attributes, indent + 1));
            }
            out.push_str(&self.indent(indent + 1));
            out.push_str(&variant.name);
            if let Some(payload) = &variant.payload {
                out.push('(');
                out.push_str(&self.format_type(payload));
                out.push(')');
            }
            if variant.span.end.line != stmt.span.end.line {
                out = self.append_trailing(variant.span.end.line, out);
            }
            out.push('\n');
            prev_line = variant.span.end.line;
        }
        out.push_str(&self.format_own_comments_before(stmt.span.end.line, indent + 1));
        out.push_str(&self.indent(indent));
        out.push('}');
        self.append_trailing_to_last_line(stmt.span.end.line, out)
    }

    fn format_opaque_type_decl(&self, stmt: &OpaqueTypeDecl) -> String {
        let mut out = String::new();
        self.push_visibility(&mut out, stmt.visibility);
        out.push_str("opaque type ");
        out.push_str(&stmt.name);
        self.append_trailing(stmt.span.end.line, out)
    }

    fn format_func_decl(&self, stmt: &FuncDecl, indent: usize) -> String {
        let mut out = String::new();
        out.push_str(&self.format_attributes(&stmt.attributes, indent));
        out.push_str(&self.format_own_comments_before(stmt.span.start.line, indent));
        self.push_visibility(&mut out, stmt.visibility);
        out.push_str("fn ");
        out.push_str(&stmt.name);
        self.push_type_params(&mut out, &stmt.type_params);
        out.push('(');
        out.push_str(&self.format_params(&stmt.params));
        out.push(')');
        if let Some(return_type) = &stmt.return_type {
            out.push_str(": ");
            out.push_str(&self.format_type(return_type));
        }
        out.push(' ');
        out.push_str(&self.format_value_block(&stmt.body, indent));
        out
    }

    fn format_attributes(&self, attributes: &[Attribute], indent: usize) -> String {
        let mut out = String::new();
        for attribute in attributes {
            out.push_str(&self.format_own_comments_before(attribute.span.start.line, indent));
            out.push_str(&self.indent(indent));
            out.push('@');
            out.push_str(&attribute.name);
            if !attribute.arguments.is_empty() {
                out.push('(');
                for (index, argument) in attribute.arguments.iter().enumerate() {
                    if index > 0 {
                        out.push_str(", ");
                    }
                    out.push_str(&argument.name);
                    if let Some(value) = &argument.value {
                        match value {
                            crate::ast::AttributeArgumentValue::String(value) => {
                                out.push_str(": \"");
                                out.push_str(&escape_string(value));
                                out.push('"');
                            }
                            crate::ast::AttributeArgumentValue::Int(value) => {
                                out.push_str(": ");
                                out.push_str(&value.to_string());
                            }
                        }
                    }
                }
                out.push(')');
            }
            out = self.append_trailing(attribute.span.end.line, out);
            out.push('\n');
        }
        out
    }

    fn format_if_stmt(&self, stmt: &IfStmt, indent: usize) -> String {
        let mut out = format!(
            "if {} {}",
            self.format_expr(&stmt.condition, PREC_LOWEST, indent),
            self.format_block(&stmt.then_branch, indent)
        );
        if let Some(else_branch) = &stmt.else_branch {
            out.push_str(" else ");
            if let [Stmt::If(nested)] = else_branch.statements.as_slice() {
                out.push_str(&self.format_if_stmt(nested, indent));
            } else {
                out.push_str(&self.format_block(else_branch, indent));
            }
        }
        out
    }

    fn format_block(&self, block: &Block, indent: usize) -> String {
        let mut out = String::from("{\n");
        out.push_str(&self.format_block_open_trailing_as_own(
            block.span.start.line,
            block.span.end.line,
            indent + 1,
        ));
        let mut prev_line = block.span.start.line;
        for (index, statement) in block.statements.iter().enumerate() {
            let target_line = stmt_comment_start_line(statement);
            if index > 0 && self.has_gap_before(prev_line, target_line) {
                out.push('\n');
            }
            out.push_str(&self.format_own_comments_before(target_line, indent + 1));
            out.push_str(&self.format_stmt_in_block(statement, indent + 1, block.span.end.line));
            out.push('\n');
            prev_line = statement.span().end.line;
        }
        out.push_str(&self.format_own_comments_before(block.span.end.line, indent + 1));
        out.push_str(&self.indent(indent));
        out.push('}');
        self.append_trailing_to_last_line(block.span.end.line, out)
    }

    fn format_value_block(&self, block: &ValueBlock, indent: usize) -> String {
        let mut out = String::from("{\n");
        out.push_str(&self.format_block_open_trailing_as_own(
            block.span.start.line,
            block.span.end.line,
            indent + 1,
        ));
        let mut prev_line = block.span.start.line;
        for (index, statement) in block.statements.iter().enumerate() {
            let target_line = stmt_comment_start_line(statement);
            if index > 0 && self.has_gap_before(prev_line, target_line) {
                out.push('\n');
            }
            out.push_str(&self.format_own_comments_before(target_line, indent + 1));
            out.push_str(&self.format_stmt_in_block(statement, indent + 1, block.span.end.line));
            out.push('\n');
            prev_line = statement.span().end.line;
        }
        if !block.terminal_return {
            let expr_target_line = block.expr.span().start.line;
            if !block.statements.is_empty() && self.has_gap_before(prev_line, expr_target_line) {
                out.push('\n');
            }
            out.push_str(&self.format_own_comments_before(expr_target_line, indent + 1));
            out.push_str(&self.indent(indent + 1));
            let expr_line = block.expr.span().end.line;
            let formatted_expr = |formatter: &Self| {
                formatter.append_trailing(
                    expr_line,
                    formatter.format_expr(&block.expr, PREC_LOWEST, indent + 1),
                )
            };
            if expr_line == block.span.end.line {
                out.push_str(&self.with_suppressed_trailing_line(expr_line, formatted_expr));
            } else {
                out.push_str(&formatted_expr(self));
            }
            out.push('\n');
        }
        out.push_str(&self.format_own_comments_before(block.span.end.line, indent + 1));
        out.push_str(&self.indent(indent));
        out.push('}');
        self.append_trailing_to_last_line(block.span.end.line, out)
    }

    fn format_expr(&self, expr: &Expr, parent_prec: u8, indent: usize) -> String {
        let prec = self.expr_prec(expr);
        let text = match expr {
            Expr::Int(expr) => expr.value.to_string(),
            Expr::Bool(expr) => expr.value.to_string(),
            Expr::String(expr) => format!("\"{}\"", escape_string(&expr.value)),
            Expr::Unit(_) => "()".to_string(),
            Expr::Ident(expr) => expr.name.clone(),
            Expr::ListLit(expr) => self.format_list_lit(expr, indent),
            Expr::Index(expr) => format!(
                "{}[{}]",
                self.format_expr(&expr.base, PREC_POSTFIX, indent),
                self.format_expr(&expr.index, PREC_LOWEST, indent)
            ),
            Expr::RecordLit(expr) => self.format_record_lit(expr, indent),
            Expr::Field(expr) => format!(
                "{}.{}",
                self.format_expr(&expr.base, PREC_POSTFIX, indent),
                expr.field
            ),
            Expr::RecordUpdate(expr) => self.format_record_update(expr, indent),
            Expr::Unary(expr) => self.format_unary_expr(expr, indent),
            Expr::Binary(expr) => self.format_binary_expr(expr, indent),
            Expr::Call(expr) => self.format_call_expr(expr, indent),
            Expr::Try(expr) => format!(
                "try {}",
                self.format_expr(&expr.expr, PREC_UNARY + 1, indent)
            ),
            Expr::If(expr) => self.format_if_expr(expr, indent),
            Expr::Match(expr) => self.format_match_expr(expr, indent),
            Expr::Fn(expr) => self.format_fn_expr(expr, indent),
            Expr::Group(expr) => {
                format!("group {}", self.format_value_block(&expr.body, indent))
            }
            Expr::Spawn(expr) => match expr.expr.as_ref() {
                Expr::Group(group_expr) => format!(
                    "spawn group {}",
                    self.format_value_block(&group_expr.body, indent)
                ),
                _ => format!(
                    "spawn {}",
                    self.format_expr(&expr.expr, PREC_UNARY + 1, indent)
                ),
            },
        };

        if prec < parent_prec {
            format!("({text})")
        } else {
            text
        }
    }

    fn expr_prec(&self, expr: &Expr) -> u8 {
        match expr {
            Expr::Binary(expr) => self.binary_prec(expr.op),
            Expr::Unary(_) | Expr::Try(_) | Expr::Spawn(_) => PREC_UNARY,
            Expr::Call(_) | Expr::Field(_) | Expr::RecordUpdate(_) | Expr::Index(_) => PREC_POSTFIX,
            Expr::If(_) | Expr::Match(_) | Expr::Group(_) => PREC_LOWEST,
            Expr::Int(_)
            | Expr::Bool(_)
            | Expr::String(_)
            | Expr::Unit(_)
            | Expr::Ident(_)
            | Expr::ListLit(_)
            | Expr::RecordLit(_)
            | Expr::Fn(_) => PREC_PRIMARY,
        }
    }

    fn format_list_lit(&self, expr: &ListLitExpr, indent: usize) -> String {
        format!(
            "[{}]",
            expr.items
                .iter()
                .map(|item| self.format_expr(item, PREC_LOWEST, indent))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }

    fn format_record_lit(&self, expr: &RecordLitExpr, indent: usize) -> String {
        if expr.fields.is_empty() {
            return format!("{} {{}}", expr.type_name);
        }
        format!(
            "{} {{ {} }}",
            expr.type_name,
            self.format_field_inits(&expr.fields, indent)
        )
    }

    fn format_record_update(&self, expr: &RecordUpdateExpr, indent: usize) -> String {
        format!(
            "{}.with({})",
            self.format_expr(&expr.base, PREC_POSTFIX, indent),
            self.format_field_inits(&expr.fields, indent)
        )
    }

    fn format_field_inits(&self, fields: &[RecordFieldInit], indent: usize) -> String {
        fields
            .iter()
            .map(|field| {
                format!(
                    "{}: {}",
                    field.name,
                    self.format_expr(&field.value, PREC_LOWEST, indent)
                )
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_unary_expr(&self, expr: &UnaryExpr, indent: usize) -> String {
        let op = match expr.op {
            UnaryOp::Neg => "-",
            UnaryOp::Not => "!",
        };
        format!(
            "{op}{}",
            self.format_expr(&expr.expr, PREC_UNARY + 1, indent)
        )
    }

    fn format_binary_expr(&self, expr: &BinaryExpr, indent: usize) -> String {
        let prec = self.binary_prec(expr.op);
        format!(
            "{} {} {}",
            self.format_expr(&expr.left, prec, indent),
            self.binary_op(expr.op),
            self.format_expr(&expr.right, prec + 1, indent)
        )
    }

    fn format_call_expr(&self, expr: &CallExpr, indent: usize) -> String {
        let type_args = self.format_call_type_args(&expr.type_args);
        match expr.origin {
            CallOrigin::Ordinary => format!(
                "{}{}({})",
                self.format_expr(&expr.callee, PREC_POSTFIX, indent),
                type_args,
                self.format_args(&expr.args, indent)
            ),
            CallOrigin::Chained | CallOrigin::QualifiedChained => {
                let Some((receiver, args)) = expr.args.split_first() else {
                    return format!(
                        "{}{}({})",
                        self.format_expr(&expr.callee, PREC_POSTFIX, indent),
                        type_args,
                        self.format_args(&expr.args, indent)
                    );
                };
                let Expr::Ident(callee) = expr.callee.as_ref() else {
                    return format!(
                        "{}{}({})",
                        self.format_expr(&expr.callee, PREC_POSTFIX, indent),
                        type_args,
                        self.format_args(&expr.args, indent)
                    );
                };
                format!(
                    "{}.{}{}({})",
                    self.format_expr(receiver, PREC_POSTFIX, indent),
                    callee.name,
                    type_args,
                    self.format_args(args, indent)
                )
            }
        }
    }

    fn format_call_type_args(&self, type_args: &[TypeExpr]) -> String {
        if type_args.is_empty() {
            String::new()
        } else {
            format!(
                "[{}]",
                type_args
                    .iter()
                    .map(|arg| self.format_type(arg))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }

    fn format_args(&self, args: &[Expr], indent: usize) -> String {
        args.iter()
            .map(|arg| self.format_expr(arg, PREC_LOWEST, indent))
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_if_expr(&self, expr: &IfExpr, indent: usize) -> String {
        let mut out = format!(
            "if {} {} else ",
            self.format_expr(&expr.condition, PREC_LOWEST, indent),
            self.format_value_block(&expr.then_branch, indent)
        );
        if let Some(nested) = self.nested_if_expr(&expr.else_branch) {
            out.push_str(&self.format_if_expr(nested, indent));
        } else {
            out.push_str(&self.format_value_block(&expr.else_branch, indent));
        }
        out
    }

    fn nested_if_expr<'a>(&self, block: &'a ValueBlock) -> Option<&'a IfExpr> {
        if block.terminal_return || !block.statements.is_empty() {
            return None;
        }
        match block.expr.as_ref() {
            Expr::If(expr) => Some(expr),
            _ => None,
        }
    }

    fn format_match_expr(&self, expr: &MatchExpr, indent: usize) -> String {
        let mut out = format!(
            "match {} {{\n",
            self.format_expr(&expr.value, PREC_LOWEST, indent)
        );
        out.push_str(&self.format_block_open_trailing_as_own(
            expr.span.start.line,
            expr.span.end.line,
            indent + 1,
        ));
        let mut prev_line = expr.span.start.line;
        for (index, arm) in expr.arms.iter().enumerate() {
            let target_line = arm.pattern.span().start.line;
            if index > 0 && self.has_gap_before(prev_line, target_line) {
                out.push('\n');
            }
            out.push_str(&self.format_own_comments_before(target_line, indent + 1));
            out.push_str(&self.indent(indent + 1));
            out.push_str(&self.format_match_pattern(&arm.pattern));
            out.push_str(" => ");
            out.push_str(&self.format_expr(&arm.value, PREC_LOWEST, indent + 1));
            if arm.span.end.line != expr.span.end.line {
                out = self.append_trailing(arm.span.end.line, out);
            }
            out.push('\n');
            prev_line = arm.span.end.line;
        }
        out.push_str(&self.format_own_comments_before(expr.span.end.line, indent + 1));
        out.push_str(&self.indent(indent));
        out.push('}');
        self.append_trailing_to_last_line(expr.span.end.line, out)
    }

    fn format_match_pattern(&self, pattern: &MatchPattern) -> String {
        match pattern {
            MatchPattern::Variant(pattern) => {
                let mut out = format!("{}::{}", pattern.enum_name, pattern.variant_name);
                match &pattern.payload {
                    EnumVariantPatternPayload::None => {}
                    EnumVariantPatternPayload::Binding(binding) => {
                        out.push('(');
                        out.push_str(binding);
                        out.push(')');
                    }
                    EnumVariantPatternPayload::Discard => out.push_str("(_)"),
                }
                out
            }
        }
    }

    fn format_fn_expr(&self, expr: &FnExpr, indent: usize) -> String {
        let mut out = String::from("fn(");
        out.push_str(&self.format_params(&expr.params));
        out.push(')');
        if let Some(return_type) = &expr.return_type {
            out.push_str(": ");
            out.push_str(&self.format_type(return_type));
        }
        out.push(' ');
        out.push_str(&self.format_value_block(&expr.body, indent));
        out
    }

    fn format_params(&self, params: &[Param]) -> String {
        params
            .iter()
            .map(|param| {
                let mut out = param.name.clone();
                if let Some(type_name) = &param.type_name {
                    out.push_str(": ");
                    out.push_str(&self.format_type(type_name));
                }
                out
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn format_type(&self, ty: &TypeExpr) -> String {
        match ty {
            TypeExpr::Int => "Int".to_string(),
            TypeExpr::Bool => "Bool".to_string(),
            TypeExpr::String => "String".to_string(),
            TypeExpr::Unit => "Unit".to_string(),
            TypeExpr::Named(name) => name.clone(),
            TypeExpr::Generic(generic) => format!(
                "{}[{}]",
                generic.name,
                generic
                    .args
                    .iter()
                    .map(|arg| self.format_type(arg))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            TypeExpr::Function(function) => {
                let domain = if function.params.len() == 1
                    && !matches!(function.params.first(), Some(TypeExpr::Function(_)))
                {
                    self.format_type(&function.params[0])
                } else {
                    format!(
                        "({})",
                        function
                            .params
                            .iter()
                            .map(|param| self.format_type(param))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                };
                format!("{domain} -> {}", self.format_type(&function.ret))
            }
        }
    }

    fn push_visibility(&self, out: &mut String, visibility: Visibility) {
        match visibility {
            Visibility::Private => {}
            Visibility::Package => out.push_str("pkg "),
            Visibility::Public => out.push_str("pub "),
        }
    }

    fn push_type_params(&self, out: &mut String, type_params: &[String]) {
        if type_params.is_empty() {
            return;
        }
        out.push('[');
        out.push_str(&type_params.join(", "));
        out.push(']');
    }

    fn binary_prec(&self, op: BinaryOp) -> u8 {
        match op {
            BinaryOp::Or => PREC_OR,
            BinaryOp::And => PREC_AND,
            BinaryOp::EqEq | BinaryOp::BangEq => PREC_EQUALITY,
            BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => PREC_COMPARISON,
            BinaryOp::Add | BinaryOp::Sub => PREC_ADDITIVE,
            BinaryOp::Mul | BinaryOp::Div => PREC_MULTIPLICATIVE,
        }
    }

    fn binary_op(&self, op: BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Lt => "<",
            BinaryOp::LtEq => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::GtEq => ">=",
            BinaryOp::EqEq => "==",
            BinaryOp::BangEq => "!=",
            BinaryOp::And => "and",
            BinaryOp::Or => "or",
        }
    }

    fn indent(&self, level: usize) -> String {
        INDENT.repeat(level)
    }

    fn format_own_comments_before(&self, line: usize, indent: usize) -> String {
        self.comments
            .borrow_mut()
            .format_own_comments_before(line, indent)
    }

    fn format_trailing_as_own(&self, line: usize, indent: usize) -> String {
        self.comments
            .borrow_mut()
            .format_trailing_as_own(line, indent)
    }

    fn format_block_open_trailing_as_own(
        &self,
        start_line: usize,
        end_line: usize,
        indent: usize,
    ) -> String {
        if start_line == end_line {
            String::new()
        } else {
            self.format_trailing_as_own(start_line, indent)
        }
    }

    fn append_trailing(&self, line: usize, text: String) -> String {
        if self.is_trailing_suppressed(line) {
            return text;
        }
        self.comments.borrow_mut().append_trailing(line, text)
    }

    fn append_trailing_to_last_line(&self, line: usize, text: String) -> String {
        self.comments
            .borrow_mut()
            .append_trailing_to_last_line(line, text)
    }

    fn format_all_remaining_comments(&self, indent: usize) -> String {
        self.comments.borrow_mut().format_all_remaining(indent)
    }

    fn format_stmt_in_block(&self, stmt: &Stmt, indent: usize, block_end_line: usize) -> String {
        if stmt.span().end.line == block_end_line {
            self.with_suppressed_trailing_line(block_end_line, |formatter| {
                formatter.format_stmt(stmt, indent)
            })
        } else {
            self.format_stmt(stmt, indent)
        }
    }

    fn with_suppressed_trailing_line<T>(&self, line: usize, format: impl FnOnce(&Self) -> T) -> T {
        self.suppressed_trailing_lines.borrow_mut().push(line);
        let result = format(self);
        self.suppressed_trailing_lines.borrow_mut().pop();
        result
    }

    fn is_trailing_suppressed(&self, line: usize) -> bool {
        self.suppressed_trailing_lines.borrow().contains(&line)
    }

    /// Whether a preserved blank line should be emitted between two items of a
    /// vertical list (block statements, record fields, enum variants, match arms).
    /// `prev_line` is the previous item's end line; `next_line` is the next item's
    /// own start line (before accounting for any leading comments).
    fn has_gap_before(&self, prev_line: usize, next_line: usize) -> bool {
        let comments = self.comments.borrow();
        let boundary = comments
            .first_own_comment_line_before(next_line)
            .unwrap_or(next_line);
        comments.has_blank_line_between(prev_line, boundary)
    }
}

#[derive(Clone, Debug)]
struct LineComment {
    line: usize,
    text: String,
}

#[derive(Default)]
struct CommentStore {
    own_line: VecDeque<LineComment>,
    trailing: BTreeMap<usize, Vec<String>>,
    blank_lines: HashSet<usize>,
}

impl CommentStore {
    fn from_source(source: &str) -> Self {
        let mut store = Self::default();
        for (line_index, line) in source.lines().enumerate() {
            let line_number = line_index + 1;
            if line.trim().is_empty() {
                store.blank_lines.insert(line_number);
            }
            let Some(comment_start) = line_comment_start(line) else {
                continue;
            };
            let comment = line[comment_start..].trim_end().to_string();
            if line[..comment_start].trim().is_empty() {
                store.own_line.push_back(LineComment {
                    line: line_number,
                    text: comment,
                });
            } else {
                store.trailing.entry(line_number).or_default().push(comment);
            }
        }
        store
    }

    /// Line of the earliest queued own-line comment that precedes `line`, if any,
    /// without consuming it. Used to find the true start of a "leading comments +
    /// statement" group when checking for a preserved blank line before it.
    fn first_own_comment_line_before(&self, line: usize) -> Option<usize> {
        self.own_line
            .front()
            .filter(|comment| comment.line < line)
            .map(|comment| comment.line)
    }

    /// Whether the source has a blank line strictly between `start_exclusive` and
    /// `end_exclusive`.
    fn has_blank_line_between(&self, start_exclusive: usize, end_exclusive: usize) -> bool {
        (start_exclusive + 1..end_exclusive).any(|line| self.blank_lines.contains(&line))
    }

    fn format_own_comments_before(&mut self, line: usize, indent: usize) -> String {
        let mut out = String::new();
        while self
            .own_line
            .front()
            .is_some_and(|comment| comment.line < line)
        {
            let comment = self.own_line.pop_front().expect("front was present");
            out.push_str(&format_comment_line(indent, &comment.text));
        }
        out
    }

    fn format_trailing_as_own(&mut self, line: usize, indent: usize) -> String {
        self.trailing
            .remove(&line)
            .into_iter()
            .flatten()
            .map(|comment| format_comment_line(indent, &comment))
            .collect()
    }

    fn append_trailing(&mut self, line: usize, mut text: String) -> String {
        if let Some(comments) = self.trailing.remove(&line) {
            text.push(' ');
            text.push_str(&comments.join(" "));
        }
        text
    }

    fn append_trailing_to_last_line(&mut self, line: usize, mut text: String) -> String {
        let Some(comments) = self.trailing.remove(&line) else {
            return text;
        };
        text.push(' ');
        text.push_str(&comments.join(" "));
        text
    }

    fn format_all_remaining(&mut self, indent: usize) -> String {
        let mut comments = Vec::new();
        comments.extend(
            self.own_line
                .drain(..)
                .map(|comment| (comment.line, comment.text)),
        );
        comments.extend(std::mem::take(&mut self.trailing).into_iter().flat_map(
            |(line, comments)| {
                comments
                    .into_iter()
                    .map(move |comment| (line, comment))
                    .collect::<Vec<_>>()
            },
        ));
        comments.sort_by_key(|(line, _)| *line);
        comments
            .into_iter()
            .map(|(_, comment)| format_comment_line(indent, &comment))
            .collect()
    }
}

fn format_comment_line(indent: usize, comment: &str) -> String {
    format!("{}{}\n", INDENT.repeat(indent), comment)
}

fn line_comment_start(line: &str) -> Option<usize> {
    let mut in_string = false;
    let mut escaped = false;
    let mut chars = line.char_indices().peekable();
    while let Some((index, ch)) = chars.next() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        if ch == '"' {
            in_string = true;
        } else if ch == '/' && chars.peek().is_some_and(|(_, next)| *next == '/') {
            return Some(index);
        }
    }
    None
}

fn stmt_comment_start_line(stmt: &Stmt) -> usize {
    match stmt {
        Stmt::FuncDecl(stmt) => stmt
            .attributes
            .first()
            .map(|attribute| attribute.span.start.line)
            .unwrap_or(stmt.span.start.line),
        _ => stmt.span().start.line,
    }
}

fn escape_string(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            other => escaped.push(other),
        }
    }
    escaped
}
