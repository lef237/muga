use std::collections::HashMap;

use crate::ast::*;
use crate::diagnostic::Diagnostic;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BindingKind {
    Immutable,
    Mutable,
    Function,
    Parameter,
}

pub fn resolve(program: &Program) -> Vec<Diagnostic> {
    let mut resolver = Resolver {
        scopes: vec![ScopeFrame::new(true)],
        records: HashMap::new(),
        diagnostics: Vec::new(),
    };
    resolver.install_prelude();
    resolver.predeclare_records(&program.statements);
    resolver.resolve_scope_statements(&program.statements);
    resolver.diagnostics
}

struct ScopeFrame {
    bindings: HashMap<String, BindingKind>,
    function_boundary: bool,
}

impl ScopeFrame {
    fn new(function_boundary: bool) -> Self {
        Self {
            bindings: HashMap::new(),
            function_boundary,
        }
    }
}

struct Resolver {
    scopes: Vec<ScopeFrame>,
    records: HashMap<String, crate::span::Span>,
    diagnostics: Vec<Diagnostic>,
}

impl Resolver {
    fn install_prelude(&mut self) {
        self.insert_current("println".to_string(), BindingKind::Function);
    }

    fn resolve_scope_statements(&mut self, statements: &[Stmt]) {
        self.predeclare_functions(statements);
        for statement in statements {
            self.resolve_stmt(statement);
        }
    }

    fn resolve_block(&mut self, block: &Block) {
        self.push_scope(false);
        self.resolve_scope_statements(&block.statements);
        self.pop_scope();
    }

    fn resolve_value_block(&mut self, block: &ValueBlock) {
        self.push_scope(false);
        self.predeclare_functions(&block.statements);
        for statement in &block.statements {
            self.resolve_stmt(statement);
        }
        self.resolve_expr(&block.expr);
        self.pop_scope();
    }

    fn resolve_stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Assign(stmt) => self.resolve_assign(stmt),
            Stmt::RecordDecl(_) => {}
            Stmt::FuncDecl(stmt) => self.resolve_func_decl(stmt),
            Stmt::If(stmt) => {
                self.resolve_expr(&stmt.condition);
                self.resolve_block(&stmt.then_branch);
                if let Some(else_branch) = &stmt.else_branch {
                    self.resolve_block(else_branch);
                }
            }
            Stmt::While(stmt) => {
                self.resolve_expr(&stmt.condition);
                self.resolve_block(&stmt.body);
            }
            Stmt::Expr(stmt) => self.resolve_expr(&stmt.expr),
        }
    }

    fn resolve_assign(&mut self, stmt: &AssignStmt) {
        self.resolve_expr(&stmt.value);
        if stmt.mutable {
            if self.current_scope_contains(&stmt.name) {
                self.diagnostics.push(Diagnostic::new(
                    "E002",
                    format!("duplicate binding `{}` in the current scope", stmt.name),
                    stmt.span,
                ));
            } else if self.any_enclosing_scope_lookup(&stmt.name).is_some() {
                self.diagnostics.push(Diagnostic::new(
                    "E003",
                    format!("shadowing is prohibited for `{}`", stmt.name),
                    stmt.span,
                ));
            } else {
                self.insert_current(stmt.name.clone(), BindingKind::Mutable);
            }
            return;
        }

        if let Some(kind) = self.lookup_in_current_function(&stmt.name) {
            match kind {
                BindingKind::Mutable => {}
                BindingKind::Immutable | BindingKind::Function | BindingKind::Parameter => {
                    self.diagnostics.push(Diagnostic::new(
                        "E001",
                        format!("cannot update immutable binding `{}`", stmt.name),
                        stmt.span,
                    ));
                }
            }
            return;
        }

        if let Some(kind) = self.lookup_beyond_current_function(&stmt.name) {
            match kind {
                BindingKind::Mutable => {
                    self.diagnostics.push(Diagnostic::new(
                        "E004",
                        format!(
                            "cannot update outer-scope mutable binding `{}` in v1",
                            stmt.name
                        ),
                        stmt.span,
                    ));
                }
                BindingKind::Immutable | BindingKind::Function | BindingKind::Parameter => {
                    self.diagnostics.push(Diagnostic::new(
                        "E003",
                        format!("shadowing is prohibited for `{}`", stmt.name),
                        stmt.span,
                    ));
                }
            }
            return;
        }

        self.insert_current(stmt.name.clone(), BindingKind::Immutable);
    }

    fn resolve_func_decl(&mut self, stmt: &FuncDecl) {
        self.push_scope(true);
        for param in &stmt.params {
            if self.current_scope_contains(&param.name) {
                self.diagnostics.push(Diagnostic::new(
                    "E002",
                    format!("duplicate binding `{}` in the current scope", param.name),
                    param.span,
                ));
                continue;
            }
            if self.any_enclosing_scope_lookup(&param.name).is_some() {
                self.diagnostics.push(Diagnostic::new(
                    "E003",
                    format!("shadowing is prohibited for `{}`", param.name),
                    param.span,
                ));
                continue;
            }
            self.insert_current(param.name.clone(), BindingKind::Parameter);
        }
        self.resolve_scope_statements(&stmt.body.statements);
        self.resolve_expr(&stmt.body.expr);
        self.pop_scope();
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int(_) | Expr::Bool(_) | Expr::String(_) => {}
            Expr::Ident(expr) => {
                if self.any_scope_lookup(&expr.name).is_none() {
                    self.diagnostics.push(Diagnostic::new(
                        "N001",
                        format!("unresolved name `{}`", expr.name),
                        expr.span,
                    ));
                }
            }
            Expr::RecordLit(expr) => {
                for field in &expr.fields {
                    self.resolve_expr(&field.value);
                }
            }
            Expr::Field(expr) => self.resolve_expr(&expr.base),
            Expr::RecordUpdate(expr) => {
                self.resolve_expr(&expr.base);
                for field in &expr.fields {
                    self.resolve_expr(&field.value);
                }
            }
            Expr::Unary(expr) => self.resolve_expr(&expr.expr),
            Expr::Binary(expr) => {
                self.resolve_expr(&expr.left);
                self.resolve_expr(&expr.right);
            }
            Expr::Call(expr) => {
                self.resolve_expr(&expr.callee);
                for arg in &expr.args {
                    self.resolve_expr(arg);
                }
            }
            Expr::If(expr) => {
                self.resolve_expr(&expr.condition);
                self.resolve_value_block(&expr.then_branch);
                self.resolve_value_block(&expr.else_branch);
            }
            Expr::Fn(expr) => {
                self.push_scope(true);
                for param in &expr.params {
                    if self.current_scope_contains(&param.name) {
                        self.diagnostics.push(Diagnostic::new(
                            "E002",
                            format!("duplicate binding `{}` in the current scope", param.name),
                            param.span,
                        ));
                        continue;
                    }
                    if self.any_enclosing_scope_lookup(&param.name).is_some() {
                        self.diagnostics.push(Diagnostic::new(
                            "E003",
                            format!("shadowing is prohibited for `{}`", param.name),
                            param.span,
                        ));
                        continue;
                    }
                    self.insert_current(param.name.clone(), BindingKind::Parameter);
                }
                self.resolve_scope_statements(&expr.body.statements);
                self.resolve_expr(&expr.body.expr);
                self.pop_scope();
            }
        }
    }

    fn predeclare_functions(&mut self, statements: &[Stmt]) {
        for statement in statements {
            if let Stmt::FuncDecl(func) = statement {
                if self.current_scope_contains(&func.name) {
                    self.diagnostics.push(Diagnostic::new(
                        "E002",
                        format!("duplicate binding `{}` in the current scope", func.name),
                        func.span,
                    ));
                } else if self.any_enclosing_scope_lookup(&func.name).is_some() {
                    self.diagnostics.push(Diagnostic::new(
                        "E003",
                        format!("shadowing is prohibited for `{}`", func.name),
                        func.span,
                    ));
                } else {
                    self.insert_current(func.name.clone(), BindingKind::Function);
                }
            }
        }
    }

    fn predeclare_records(&mut self, statements: &[Stmt]) {
        for statement in statements {
            if let Stmt::RecordDecl(record) = statement {
                if self.records.contains_key(&record.name) {
                    self.diagnostics.push(Diagnostic::new(
                        "E002",
                        format!("duplicate record `{}` in the current scope", record.name),
                        record.span,
                    ));
                } else {
                    self.records.insert(record.name.clone(), record.span);
                }
            }
        }
    }

    fn push_scope(&mut self, function_boundary: bool) {
        self.scopes.push(ScopeFrame::new(function_boundary));
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_scope_contains(&self, name: &str) -> bool {
        self.scopes
            .last()
            .map(|scope| scope.bindings.contains_key(name))
            .unwrap_or(false)
    }

    fn lookup_in_current_function(&self, name: &str) -> Option<BindingKind> {
        for scope in self.scopes.iter().rev() {
            if let Some(kind) = scope.bindings.get(name) {
                return Some(*kind);
            }
            if scope.function_boundary {
                break;
            }
        }
        None
    }

    fn lookup_beyond_current_function(&self, name: &str) -> Option<BindingKind> {
        let boundary_index = self
            .scopes
            .iter()
            .rposition(|scope| scope.function_boundary)
            .unwrap_or(0);
        self.scopes[..boundary_index]
            .iter()
            .rev()
            .find_map(|scope| scope.bindings.get(name).copied())
    }

    fn any_enclosing_scope_lookup(&self, name: &str) -> Option<BindingKind> {
        self.scopes
            .iter()
            .rev()
            .skip(1)
            .find_map(|scope| scope.bindings.get(name).copied())
    }

    fn any_scope_lookup(&self, name: &str) -> Option<BindingKind> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.bindings.get(name).copied())
    }

    fn insert_current(&mut self, name: String, kind: BindingKind) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.bindings.insert(name, kind);
        }
    }
}
