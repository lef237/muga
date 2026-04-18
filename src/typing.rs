use std::collections::{HashMap, HashSet};

use crate::ast::*;
use crate::diagnostic::Diagnostic;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Type {
    Int,
    Bool,
    String,
    Function(FunctionSig),
    Builtin(BuiltinFunction),
    Unknown(u32),
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FunctionSig {
    params: Vec<Type>,
    ret: Box<Type>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BuiltinFunction {
    Print,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BindingKind {
    Immutable,
    Mutable,
    Function,
    Parameter,
}

#[derive(Clone, Debug)]
struct Binding {
    kind: BindingKind,
    ty: Type,
}

struct ScopeFrame {
    bindings: HashMap<String, Binding>,
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

pub fn typecheck(program: &Program) -> Vec<Diagnostic> {
    let mut checker = TypeChecker::new();
    checker.check_scope_statements(&program.statements);
    checker.diagnostics
}

struct TypeChecker {
    scopes: Vec<ScopeFrame>,
    diagnostics: Vec<Diagnostic>,
    next_unknown: u32,
    substitutions: HashMap<u32, Type>,
}

impl TypeChecker {
    fn new() -> Self {
        let mut checker = Self {
            scopes: vec![ScopeFrame::new(true)],
            diagnostics: Vec::new(),
            next_unknown: 0,
            substitutions: HashMap::new(),
        };
        checker.install_prelude();
        checker
    }

    fn install_prelude(&mut self) {
        self.insert_current(
            "print".to_string(),
            Binding {
                kind: BindingKind::Function,
                ty: Type::Builtin(BuiltinFunction::Print),
            },
        );
    }

    fn check_scope_statements(&mut self, statements: &[Stmt]) {
        let functions = self.predeclare_functions(statements);
        self.check_recursive_requirements(statements, &functions);
        for statement in statements {
            match statement {
                Stmt::FuncDecl(func) => self.check_func_decl(func, &functions),
                _ => self.check_stmt(statement),
            }
        }
    }

    fn check_block(&mut self, block: &Block) {
        self.push_scope(false);
        self.check_scope_statements(&block.statements);
        self.pop_scope();
    }

    fn check_value_block(&mut self, block: &ValueBlock) -> Type {
        self.push_scope(false);
        let functions = self.predeclare_functions(&block.statements);
        self.check_recursive_requirements(&block.statements, &functions);
        for statement in &block.statements {
            match statement {
                Stmt::FuncDecl(func) => self.check_func_decl(func, &functions),
                _ => self.check_stmt(statement),
            }
        }
        let ty = self.check_expr(&block.expr);
        self.pop_scope();
        ty
    }

    fn check_stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Assign(stmt) => self.check_assign(stmt),
            Stmt::FuncDecl(_) => {}
            Stmt::If(stmt) => {
                let condition = self.check_expr(&stmt.condition);
                self.require_exact(&condition, &Type::Bool, stmt.condition.span(), "T001");
                self.check_block(&stmt.then_branch);
                if let Some(else_branch) = &stmt.else_branch {
                    self.check_block(else_branch);
                }
            }
            Stmt::While(stmt) => {
                let condition = self.check_expr(&stmt.condition);
                self.require_exact(&condition, &Type::Bool, stmt.condition.span(), "T001");
                self.check_block(&stmt.body);
            }
            Stmt::Expr(stmt) => {
                self.check_expr(&stmt.expr);
            }
        }
    }

    fn check_assign(&mut self, stmt: &AssignStmt) {
        let value_ty = self.check_expr(&stmt.value);
        if stmt.mutable {
            self.insert_current(
                stmt.name.clone(),
                Binding {
                    kind: BindingKind::Mutable,
                    ty: value_ty,
                },
            );
            return;
        }

        if let Some(binding) = self.lookup_in_current_function(&stmt.name).cloned() {
            if binding.kind == BindingKind::Mutable {
                self.require_exact(&binding.ty, &value_ty, stmt.span, "T002");
            }
            return;
        }

        if self.lookup_beyond_current_function(&stmt.name).is_none() {
            self.insert_current(
                stmt.name.clone(),
                Binding {
                    kind: BindingKind::Immutable,
                    ty: value_ty,
                },
            );
        }
    }

    fn check_func_decl(&mut self, func: &FuncDecl, local_functions: &HashMap<String, FunctionSig>) {
        let Some(sig) = local_functions.get(&func.name).cloned() else {
            return;
        };

        self.push_scope(true);
        for (param, param_ty) in func.params.iter().zip(sig.params.iter().cloned()) {
            self.insert_current(
                param.name.clone(),
                Binding {
                    kind: BindingKind::Parameter,
                    ty: param_ty,
                },
            );
        }
        let nested_functions = self.predeclare_functions(&func.body.statements);
        self.check_recursive_requirements(&func.body.statements, &nested_functions);
        for statement in &func.body.statements {
            match statement {
                Stmt::FuncDecl(nested) => self.check_func_decl(nested, &nested_functions),
                _ => self.check_stmt(statement),
            }
        }
        let body_ty = self.check_expr(&func.body.expr);
        self.require_exact(&body_ty, &sig.ret, func.body.expr.span(), "T002");
        self.pop_scope();

        let resolved_params: Vec<Type> =
            sig.params.iter().map(|ty| self.resolve_type(ty)).collect();
        let resolved_ret = self.resolve_type(&sig.ret);
        if resolved_params.iter().any(Type::is_unknown) || resolved_ret.is_unknown() {
            self.diagnostics.push(Diagnostic::new(
                "E005",
                "type annotation required because inference is not unique",
                func.span,
            ));
        }
    }

    fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Int(_) => Type::Int,
            Expr::Bool(_) => Type::Bool,
            Expr::String(_) => Type::String,
            Expr::Ident(expr) => self
                .lookup(&expr.name)
                .map(|binding| binding.ty.clone())
                .unwrap_or(Type::Error),
            Expr::Unary(expr) => {
                let ty = self.check_expr(&expr.expr);
                match expr.op {
                    UnaryOp::Neg => {
                        self.require_exact(&ty, &Type::Int, expr.span, "T001");
                        Type::Int
                    }
                    UnaryOp::Not => {
                        self.require_exact(&ty, &Type::Bool, expr.span, "T001");
                        Type::Bool
                    }
                }
            }
            Expr::Binary(expr) => {
                let left = self.check_expr(&expr.left);
                let right = self.check_expr(&expr.right);
                match expr.op {
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                        self.require_exact(&left, &Type::Int, expr.left.span(), "T001");
                        self.require_exact(&right, &Type::Int, expr.right.span(), "T001");
                        Type::Int
                    }
                    BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => {
                        self.require_exact(&left, &Type::Int, expr.left.span(), "T001");
                        self.require_exact(&right, &Type::Int, expr.right.span(), "T001");
                        Type::Bool
                    }
                    BinaryOp::EqEq | BinaryOp::BangEq => {
                        self.require_exact(&left, &right, expr.span, "T002");
                        let resolved = self.resolve_type(&left);
                        if !matches!(
                            resolved,
                            Type::Int | Type::Bool | Type::String | Type::Unknown(_)
                        ) {
                            self.diagnostics.push(Diagnostic::new(
                                "T003",
                                "equality is allowed only for Int, Bool, and String",
                                expr.span,
                            ));
                        }
                        Type::Bool
                    }
                }
            }
            Expr::Call(expr) => {
                let callee_ty = self.check_expr(&expr.callee);
                let arg_tys: Vec<Type> = expr.args.iter().map(|arg| self.check_expr(arg)).collect();
                match self.resolve_type(&callee_ty) {
                    Type::Builtin(BuiltinFunction::Print) => {
                        if arg_tys.len() != 1 {
                            self.diagnostics.push(Diagnostic::new(
                                "T004",
                                format!("expected 1 arguments but found {}", arg_tys.len()),
                                expr.span,
                            ));
                            return Type::Error;
                        }
                        let arg_ty = self.resolve_type(&arg_tys[0]);
                        match arg_ty {
                            Type::Int | Type::Bool | Type::String => arg_ty,
                            Type::Unknown(_) => {
                                self.diagnostics.push(Diagnostic::new(
                                    "E005",
                                    "type annotation required because inference is not unique",
                                    expr.span,
                                ));
                                Type::Error
                            }
                            _ => {
                                self.diagnostics.push(Diagnostic::new(
                                    "T006",
                                    "`print` accepts only Int, Bool, or String",
                                    expr.span,
                                ));
                                Type::Error
                            }
                        }
                    }
                    Type::Function(sig) => {
                        if sig.params.len() != arg_tys.len() {
                            self.diagnostics.push(Diagnostic::new(
                                "T004",
                                format!(
                                    "expected {} arguments but found {}",
                                    sig.params.len(),
                                    arg_tys.len()
                                ),
                                expr.span,
                            ));
                            return Type::Error;
                        }
                        for (param_ty, arg_ty) in sig.params.iter().zip(arg_tys.iter()) {
                            let resolved_param = self.resolve_type(param_ty);
                            if !resolved_param.is_unknown() {
                                self.require_exact(&resolved_param, arg_ty, expr.span, "T002");
                            }
                        }
                        self.resolve_type(&sig.ret)
                    }
                    Type::Error => Type::Error,
                    _ => {
                        self.diagnostics.push(Diagnostic::new(
                            "T005",
                            "attempted to call a non-function value",
                            expr.span,
                        ));
                        Type::Error
                    }
                }
            }
            Expr::If(expr) => {
                let condition = self.check_expr(&expr.condition);
                self.require_exact(&condition, &Type::Bool, expr.condition.span(), "T001");
                let then_ty = self.check_value_block(&expr.then_branch);
                let else_ty = self.check_value_block(&expr.else_branch);
                self.require_exact(&then_ty, &else_ty, expr.span, "T002");
                self.resolve_type(&then_ty)
            }
            Expr::Fn(expr) => {
                let sig = self.signature_from_fn_expr(expr);
                self.push_scope(true);
                for (param, param_ty) in expr.params.iter().zip(sig.params.iter().cloned()) {
                    self.insert_current(
                        param.name.clone(),
                        Binding {
                            kind: BindingKind::Parameter,
                            ty: param_ty,
                        },
                    );
                }
                let nested_functions = self.predeclare_functions(&expr.body.statements);
                self.check_recursive_requirements(&expr.body.statements, &nested_functions);
                for statement in &expr.body.statements {
                    match statement {
                        Stmt::FuncDecl(nested) => self.check_func_decl(nested, &nested_functions),
                        _ => self.check_stmt(statement),
                    }
                }
                let body_ty = self.check_expr(&expr.body.expr);
                self.require_exact(&body_ty, &sig.ret, expr.span, "T002");
                self.pop_scope();
                Type::Function(sig)
            }
        }
    }

    fn signature_from_fn_expr(&mut self, expr: &FnExpr) -> FunctionSig {
        let params = expr
            .params
            .iter()
            .map(|param| match param.type_name {
                Some(type_name) => self.type_from_name(type_name),
                None => Type::Unknown(self.fresh_unknown()),
            })
            .collect();
        let ret = match expr.return_type {
            Some(type_name) => self.type_from_name(type_name),
            None => Type::Unknown(self.fresh_unknown()),
        };
        FunctionSig {
            params,
            ret: Box::new(ret),
        }
    }

    fn predeclare_functions(&mut self, statements: &[Stmt]) -> HashMap<String, FunctionSig> {
        let mut functions = HashMap::new();
        for statement in statements {
            if let Stmt::FuncDecl(func) = statement {
                let params = func
                    .params
                    .iter()
                    .map(|param| match param.type_name {
                        Some(type_name) => self.type_from_name(type_name),
                        None => Type::Unknown(self.fresh_unknown()),
                    })
                    .collect::<Vec<_>>();
                let ret = match func.return_type {
                    Some(type_name) => self.type_from_name(type_name),
                    None => Type::Unknown(self.fresh_unknown()),
                };
                let sig = FunctionSig {
                    params,
                    ret: Box::new(ret),
                };
                functions.insert(func.name.clone(), sig.clone());
                self.insert_current(
                    func.name.clone(),
                    Binding {
                        kind: BindingKind::Function,
                        ty: Type::Function(sig),
                    },
                );
            }
        }
        functions
    }

    fn check_recursive_requirements(
        &mut self,
        statements: &[Stmt],
        functions: &HashMap<String, FunctionSig>,
    ) {
        let names: HashSet<String> = functions.keys().cloned().collect();
        let decls: Vec<&FuncDecl> = statements
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::FuncDecl(func) => Some(func),
                _ => None,
            })
            .collect();
        let graph = build_call_graph(&decls, &names);
        let components = strongly_connected_components(&graph);

        for component in components {
            if component.len() > 1 {
                for name in component {
                    if let Some(func) = decls.iter().find(|func| func.name == name) {
                        let has_full_signature =
                            func.params.iter().all(|param| param.type_name.is_some())
                                && func.return_type.is_some();
                        if !has_full_signature {
                            self.diagnostics.push(Diagnostic::new(
                                "E007",
                                "mutually recursive functions require explicit signatures in v1",
                                func.span,
                            ));
                        }
                    }
                }
                continue;
            }

            let name = &component[0];
            let has_self_edge = graph
                .get(name)
                .is_some_and(|targets| targets.contains(name));
            if !has_self_edge {
                continue;
            }
            if let Some(func) = decls.iter().find(|func| &func.name == name) {
                let has_annotation = func.return_type.is_some()
                    || func.params.iter().any(|param| param.type_name.is_some());
                if !has_annotation {
                    self.diagnostics.push(Diagnostic::new(
                        "E006",
                        "recursive function requires at least one parameter or return type annotation",
                        func.span,
                    ));
                }
            }
        }
    }

    fn type_from_name(&self, type_name: TypeName) -> Type {
        match type_name {
            TypeName::Int => Type::Int,
            TypeName::Bool => Type::Bool,
            TypeName::String => Type::String,
        }
    }

    fn require_exact(
        &mut self,
        left: &Type,
        right: &Type,
        span: crate::span::Span,
        code: &'static str,
    ) {
        if let Err(message) = self.unify(left.clone(), right.clone()) {
            self.diagnostics.push(Diagnostic::new(code, message, span));
        }
    }

    fn unify(&mut self, left: Type, right: Type) -> Result<Type, String> {
        let left = self.resolve_type(&left);
        let right = self.resolve_type(&right);
        match (left, right) {
            (Type::Error, _) | (_, Type::Error) => Ok(Type::Error),
            (Type::Unknown(id), ty) | (ty, Type::Unknown(id)) => {
                self.substitutions.insert(id, ty.clone());
                Ok(ty)
            }
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Bool, Type::Bool) => Ok(Type::Bool),
            (Type::String, Type::String) => Ok(Type::String),
            (Type::Function(left), Type::Function(right)) => {
                if left.params.len() != right.params.len() {
                    return Err("function arity mismatch".to_string());
                }
                for (left_param, right_param) in left.params.iter().zip(right.params.iter()) {
                    self.unify(left_param.clone(), right_param.clone())?;
                }
                self.unify(*left.ret.clone(), *right.ret.clone())
            }
            (left, right) => Err(format!(
                "type mismatch: expected {}, found {}",
                left.display(),
                right.display()
            )),
        }
    }

    fn resolve_type(&self, ty: &Type) -> Type {
        match ty {
            Type::Unknown(id) => {
                if let Some(next) = self.substitutions.get(id) {
                    self.resolve_type(next)
                } else {
                    Type::Unknown(*id)
                }
            }
            Type::Function(sig) => Type::Function(FunctionSig {
                params: sig.params.iter().map(|ty| self.resolve_type(ty)).collect(),
                ret: Box::new(self.resolve_type(&sig.ret)),
            }),
            Type::Builtin(builtin) => Type::Builtin(*builtin),
            other => other.clone(),
        }
    }

    fn fresh_unknown(&mut self) -> u32 {
        let id = self.next_unknown;
        self.next_unknown += 1;
        id
    }

    fn push_scope(&mut self, function_boundary: bool) {
        self.scopes.push(ScopeFrame::new(function_boundary));
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn insert_current(&mut self, name: String, binding: Binding) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.bindings.insert(name, binding);
        }
    }

    fn lookup(&self, name: &str) -> Option<&Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.bindings.get(name))
    }

    fn lookup_in_current_function(&self, name: &str) -> Option<&Binding> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.bindings.get(name) {
                return Some(binding);
            }
            if scope.function_boundary {
                break;
            }
        }
        None
    }

    fn lookup_beyond_current_function(&self, name: &str) -> Option<&Binding> {
        let boundary_index = self
            .scopes
            .iter()
            .rposition(|scope| scope.function_boundary)
            .unwrap_or(0);
        self.scopes[..boundary_index]
            .iter()
            .rev()
            .find_map(|scope| scope.bindings.get(name))
    }
}

impl Type {
    fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }

    fn display(&self) -> &'static str {
        match self {
            Self::Int => "Int",
            Self::Bool => "Bool",
            Self::String => "String",
            Self::Function(_) => "Function",
            Self::Builtin(BuiltinFunction::Print) => "Builtin(print)",
            Self::Unknown(_) => "Unknown",
            Self::Error => "Error",
        }
    }
}

fn build_call_graph(
    decls: &[&FuncDecl],
    local_names: &HashSet<String>,
) -> HashMap<String, HashSet<String>> {
    let mut graph = HashMap::new();
    for decl in decls {
        let mut calls = HashSet::new();
        collect_calls_in_statements(&decl.body.statements, local_names, &mut calls);
        collect_calls_in_expr(&decl.body.expr, local_names, &mut calls);
        graph.insert(decl.name.clone(), calls);
    }
    graph
}

fn strongly_connected_components(graph: &HashMap<String, HashSet<String>>) -> Vec<Vec<String>> {
    let mut index = 0usize;
    let mut stack = Vec::<String>::new();
    let mut indices = HashMap::<String, usize>::new();
    let mut lowlinks = HashMap::<String, usize>::new();
    let mut on_stack = HashSet::<String>::new();
    let mut components = Vec::new();

    for node in graph.keys() {
        if !indices.contains_key(node) {
            strong_connect(
                node,
                graph,
                &mut index,
                &mut stack,
                &mut indices,
                &mut lowlinks,
                &mut on_stack,
                &mut components,
            );
        }
    }

    components
}

fn strong_connect(
    node: &str,
    graph: &HashMap<String, HashSet<String>>,
    index: &mut usize,
    stack: &mut Vec<String>,
    indices: &mut HashMap<String, usize>,
    lowlinks: &mut HashMap<String, usize>,
    on_stack: &mut HashSet<String>,
    components: &mut Vec<Vec<String>>,
) {
    indices.insert(node.to_string(), *index);
    lowlinks.insert(node.to_string(), *index);
    *index += 1;
    stack.push(node.to_string());
    on_stack.insert(node.to_string());

    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            if !indices.contains_key(neighbor) {
                strong_connect(
                    neighbor, graph, index, stack, indices, lowlinks, on_stack, components,
                );
                let neighbor_low = lowlinks[neighbor];
                let node_low = lowlinks[node];
                lowlinks.insert(node.to_string(), node_low.min(neighbor_low));
            } else if on_stack.contains(neighbor) {
                let neighbor_index = indices[neighbor];
                let node_low = lowlinks[node];
                lowlinks.insert(node.to_string(), node_low.min(neighbor_index));
            }
        }
    }

    if lowlinks[node] == indices[node] {
        let mut component = Vec::new();
        while let Some(candidate) = stack.pop() {
            on_stack.remove(&candidate);
            component.push(candidate.clone());
            if candidate == node {
                break;
            }
        }
        components.push(component);
    }
}

fn collect_calls_in_statements(
    statements: &[Stmt],
    local_names: &HashSet<String>,
    calls: &mut HashSet<String>,
) {
    for statement in statements {
        match statement {
            Stmt::Assign(stmt) => collect_calls_in_expr(&stmt.value, local_names, calls),
            Stmt::FuncDecl(_) => {}
            Stmt::If(stmt) => {
                collect_calls_in_expr(&stmt.condition, local_names, calls);
                collect_calls_in_statements(&stmt.then_branch.statements, local_names, calls);
                if let Some(else_branch) = &stmt.else_branch {
                    collect_calls_in_statements(&else_branch.statements, local_names, calls);
                }
            }
            Stmt::While(stmt) => {
                collect_calls_in_expr(&stmt.condition, local_names, calls);
                collect_calls_in_statements(&stmt.body.statements, local_names, calls);
            }
            Stmt::Expr(stmt) => collect_calls_in_expr(&stmt.expr, local_names, calls),
        }
    }
}

fn collect_calls_in_expr(expr: &Expr, local_names: &HashSet<String>, calls: &mut HashSet<String>) {
    match expr {
        Expr::Int(_) | Expr::Bool(_) | Expr::String(_) | Expr::Ident(_) => {}
        Expr::Unary(expr) => collect_calls_in_expr(&expr.expr, local_names, calls),
        Expr::Binary(expr) => {
            collect_calls_in_expr(&expr.left, local_names, calls);
            collect_calls_in_expr(&expr.right, local_names, calls);
        }
        Expr::Call(expr) => {
            if let Expr::Ident(ident) = expr.callee.as_ref() {
                if local_names.contains(&ident.name) {
                    calls.insert(ident.name.clone());
                }
            }
            collect_calls_in_expr(&expr.callee, local_names, calls);
            for arg in &expr.args {
                collect_calls_in_expr(arg, local_names, calls);
            }
        }
        Expr::If(expr) => {
            collect_calls_in_expr(&expr.condition, local_names, calls);
            collect_calls_in_statements(&expr.then_branch.statements, local_names, calls);
            collect_calls_in_expr(&expr.then_branch.expr, local_names, calls);
            collect_calls_in_statements(&expr.else_branch.statements, local_names, calls);
            collect_calls_in_expr(&expr.else_branch.expr, local_names, calls);
        }
        Expr::Fn(_) => {}
    }
}
