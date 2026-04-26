use std::collections::HashMap;

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::identity::{BindingId, BindingKind};
use crate::span::Span;
use crate::symbol::{Symbol, SymbolTable};

#[derive(Clone, Debug)]
pub struct ResolveOutput {
    pub diagnostics: Vec<Diagnostic>,
    pub bindings: Vec<BindingInfo>,
    pub identifier_refs: Vec<ResolvedIdentifier>,
    pub symbols: SymbolTable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindingInfo {
    pub id: BindingId,
    pub symbol: Symbol,
    pub kind: BindingKind,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResolvedIdentifier {
    pub name: Symbol,
    pub span: Span,
    pub binding: BindingId,
}

pub fn resolve(program: &Program) -> Vec<Diagnostic> {
    resolve_program(program).diagnostics
}

pub fn resolve_program(program: &Program) -> ResolveOutput {
    let mut resolver = Resolver::new();
    resolver.install_prelude();
    resolver.predeclare_records(&program.statements);
    resolver.resolve_scope_statements(&program.statements);
    resolver.into_output()
}

#[derive(Clone, Copy, Debug)]
struct Binding {
    id: BindingId,
    symbol: Symbol,
    kind: BindingKind,
    span: Span,
}

struct ScopeFrame {
    bindings: HashMap<Symbol, BindingId>,
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
    records: HashMap<Symbol, Span>,
    bindings: Vec<Binding>,
    identifier_refs: Vec<ResolvedIdentifier>,
    symbols: SymbolTable,
    diagnostics: Vec<Diagnostic>,
}

impl Resolver {
    fn new() -> Self {
        Self {
            scopes: vec![ScopeFrame::new(true)],
            records: HashMap::new(),
            bindings: Vec::new(),
            identifier_refs: Vec::new(),
            symbols: SymbolTable::default(),
            diagnostics: Vec::new(),
        }
    }

    fn into_output(self) -> ResolveOutput {
        let bindings = self
            .bindings
            .iter()
            .map(|binding| BindingInfo {
                id: binding.id,
                symbol: binding.symbol,
                kind: binding.kind,
                span: binding.span,
            })
            .collect();
        ResolveOutput {
            diagnostics: self.diagnostics,
            bindings,
            identifier_refs: self.identifier_refs,
            symbols: self.symbols,
        }
    }

    fn install_prelude(&mut self) {
        let print = self.symbol("print");
        self.insert_current(print, BindingKind::Function, Span::default());
        let println = self.symbol("println");
        self.insert_current(println, BindingKind::Function, Span::default());
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
        let name = self.symbol(&stmt.name);
        if stmt.mutable {
            if self.current_scope_contains(name) {
                self.diagnostics.push(Diagnostic::new(
                    "E002",
                    format!("duplicate binding `{}` in the current scope", stmt.name),
                    stmt.span,
                ));
            } else if self.any_enclosing_scope_lookup(name).is_some() {
                self.diagnostics.push(Diagnostic::new(
                    "E003",
                    format!("shadowing is prohibited for `{}`", stmt.name),
                    stmt.span,
                ));
            } else {
                self.insert_current(name, BindingKind::Mutable, stmt.span);
            }
            return;
        }

        if let Some(binding) = self.lookup_in_current_function(name) {
            match binding.kind {
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

        if let Some(binding) = self.lookup_beyond_current_function(name) {
            match binding.kind {
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

        self.insert_current(name, BindingKind::Immutable, stmt.span);
    }

    fn resolve_func_decl(&mut self, stmt: &FuncDecl) {
        self.push_scope(true);
        for param in &stmt.params {
            let name = self.symbol(&param.name);
            if self.current_scope_contains(name) {
                self.diagnostics.push(Diagnostic::new(
                    "E002",
                    format!("duplicate binding `{}` in the current scope", param.name),
                    param.span,
                ));
                continue;
            }
            if self.any_enclosing_scope_lookup(name).is_some() {
                self.diagnostics.push(Diagnostic::new(
                    "E003",
                    format!("shadowing is prohibited for `{}`", param.name),
                    param.span,
                ));
                continue;
            }
            self.insert_current(name, BindingKind::Parameter, param.span);
        }
        self.resolve_scope_statements(&stmt.body.statements);
        self.resolve_expr(&stmt.body.expr);
        self.pop_scope();
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Int(_) | Expr::Bool(_) | Expr::String(_) => {}
            Expr::Ident(expr) => {
                let name = self.symbol(&expr.name);
                if let Some(binding) = self.any_scope_lookup(name).copied() {
                    self.identifier_refs.push(ResolvedIdentifier {
                        name,
                        span: expr.span,
                        binding: binding.id,
                    });
                } else {
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
                    let name = self.symbol(&param.name);
                    if self.current_scope_contains(name) {
                        self.diagnostics.push(Diagnostic::new(
                            "E002",
                            format!("duplicate binding `{}` in the current scope", param.name),
                            param.span,
                        ));
                        continue;
                    }
                    if self.any_enclosing_scope_lookup(name).is_some() {
                        self.diagnostics.push(Diagnostic::new(
                            "E003",
                            format!("shadowing is prohibited for `{}`", param.name),
                            param.span,
                        ));
                        continue;
                    }
                    self.insert_current(name, BindingKind::Parameter, param.span);
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
                let name = self.symbol(&func.name);
                if self.current_scope_contains(name) {
                    self.diagnostics.push(Diagnostic::new(
                        "E002",
                        format!("duplicate binding `{}` in the current scope", func.name),
                        func.span,
                    ));
                } else if self.any_enclosing_scope_lookup(name).is_some() {
                    self.diagnostics.push(Diagnostic::new(
                        "E003",
                        format!("shadowing is prohibited for `{}`", func.name),
                        func.span,
                    ));
                } else {
                    self.insert_current(name, BindingKind::Function, func.span);
                }
            }
        }
    }

    fn predeclare_records(&mut self, statements: &[Stmt]) {
        for statement in statements {
            if let Stmt::RecordDecl(record) = statement {
                let name = self.symbol(&record.name);
                if self.records.contains_key(&name) {
                    self.diagnostics.push(Diagnostic::new(
                        "E002",
                        format!("duplicate record `{}` in the current scope", record.name),
                        record.span,
                    ));
                } else {
                    self.records.insert(name, record.span);
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

    fn current_scope_contains(&self, name: Symbol) -> bool {
        self.scopes
            .last()
            .map(|scope| scope.bindings.contains_key(&name))
            .unwrap_or(false)
    }

    fn lookup_in_current_function(&self, name: Symbol) -> Option<&Binding> {
        for scope in self.scopes.iter().rev() {
            if let Some(id) = scope.bindings.get(&name) {
                return Some(self.binding(*id));
            }
            if scope.function_boundary {
                break;
            }
        }
        None
    }

    fn lookup_beyond_current_function(&self, name: Symbol) -> Option<&Binding> {
        let boundary_index = self
            .scopes
            .iter()
            .rposition(|scope| scope.function_boundary)
            .unwrap_or(0);
        self.scopes[..boundary_index]
            .iter()
            .rev()
            .find_map(|scope| scope.bindings.get(&name).map(|id| self.binding(*id)))
    }

    fn any_enclosing_scope_lookup(&self, name: Symbol) -> Option<&Binding> {
        self.scopes
            .iter()
            .rev()
            .skip(1)
            .find_map(|scope| scope.bindings.get(&name).map(|id| self.binding(*id)))
    }

    fn any_scope_lookup(&self, name: Symbol) -> Option<&Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.bindings.get(&name).map(|id| self.binding(*id)))
    }

    fn insert_current(&mut self, name: Symbol, kind: BindingKind, span: Span) -> BindingId {
        let id = BindingId::new(self.bindings.len() as u32);
        self.bindings.push(Binding {
            id,
            symbol: name,
            kind,
            span,
        });
        if let Some(scope) = self.scopes.last_mut() {
            scope.bindings.insert(name, id);
        }
        id
    }

    fn binding(&self, id: BindingId) -> &Binding {
        &self.bindings[id.as_u32() as usize]
    }

    fn symbol(&mut self, name: &str) -> Symbol {
        self.symbols.intern(name)
    }
}
