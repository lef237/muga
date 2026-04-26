use std::collections::{HashMap, HashSet};

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::identity::{BindingId, BindingKind, ExprId};
use crate::span::Span;
use crate::symbol::{Symbol, SymbolTable};

#[derive(Clone, Debug)]
pub struct TypeCheckOutput {
    pub diagnostics: Vec<Diagnostic>,
    pub bindings: Vec<TypedBindingInfo>,
    pub identifier_refs: Vec<TypedIdentifier>,
    pub expr_types: Vec<ExprTypeInfo>,
    pub symbols: SymbolTable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedBindingInfo {
    pub id: BindingId,
    pub symbol: Symbol,
    pub kind: BindingKind,
    pub ty: TypeInfo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedIdentifier {
    pub expr_id: ExprId,
    pub name: Symbol,
    pub span: Span,
    pub binding: BindingId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExprTypeInfo {
    pub expr_id: ExprId,
    pub span: Span,
    pub ty: TypeInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeInfo {
    Int,
    Bool,
    String,
    Record(Symbol),
    Function(FunctionTypeInfo),
    Builtin(&'static str),
    Unknown,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionTypeInfo {
    pub params: Vec<TypeInfo>,
    pub ret: Box<TypeInfo>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Type {
    Int,
    Bool,
    String,
    Record(Symbol),
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
    Println,
}

#[derive(Clone, Debug)]
struct Binding {
    id: BindingId,
    symbol: Symbol,
    kind: BindingKind,
    ty: Type,
}

#[derive(Clone, Debug)]
struct ExprType {
    expr_id: ExprId,
    span: Span,
    ty: Type,
}

#[derive(Clone, Debug)]
struct RecordDef {
    fields: Vec<RecordField>,
}

#[derive(Clone, Debug)]
struct RecordField {
    name: Symbol,
    type_name: TypeExpr,
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

pub fn typecheck(program: &Program) -> Vec<Diagnostic> {
    typecheck_program(program).diagnostics
}

pub fn typecheck_program(program: &Program) -> TypeCheckOutput {
    let mut checker = TypeChecker::new();
    checker.predeclare_records(&program.statements);
    checker.check_scope_statements(&program.statements);
    checker.into_output()
}

struct TypeChecker {
    scopes: Vec<ScopeFrame>,
    records: HashMap<Symbol, RecordDef>,
    bindings: Vec<Binding>,
    identifier_refs: Vec<TypedIdentifier>,
    expr_types: Vec<ExprType>,
    symbols: SymbolTable,
    diagnostics: Vec<Diagnostic>,
    next_unknown: u32,
    substitutions: HashMap<u32, Type>,
}

impl TypeChecker {
    fn new() -> Self {
        let mut checker = Self {
            scopes: vec![ScopeFrame::new(true)],
            records: HashMap::new(),
            bindings: Vec::new(),
            identifier_refs: Vec::new(),
            expr_types: Vec::new(),
            symbols: SymbolTable::default(),
            diagnostics: Vec::new(),
            next_unknown: 0,
            substitutions: HashMap::new(),
        };
        checker.install_prelude();
        checker
    }

    fn into_output(self) -> TypeCheckOutput {
        let bindings = self
            .bindings
            .iter()
            .map(|binding| TypedBindingInfo {
                id: binding.id,
                symbol: binding.symbol,
                kind: binding.kind,
                ty: self.type_info_for(&binding.ty),
            })
            .collect();
        let expr_types = self
            .expr_types
            .iter()
            .map(|expr_type| ExprTypeInfo {
                expr_id: expr_type.expr_id,
                span: expr_type.span,
                ty: self.type_info_for(&expr_type.ty),
            })
            .collect();
        TypeCheckOutput {
            diagnostics: self.diagnostics,
            bindings,
            identifier_refs: self.identifier_refs,
            expr_types,
            symbols: self.symbols,
        }
    }

    fn install_prelude(&mut self) {
        let print = self.symbol("print");
        self.insert_current(
            print,
            BindingKind::Function,
            Type::Builtin(BuiltinFunction::Print),
        );
        let println = self.symbol("println");
        self.insert_current(
            println,
            BindingKind::Function,
            Type::Builtin(BuiltinFunction::Println),
        );
    }

    fn check_scope_statements(&mut self, statements: &[Stmt]) {
        let functions = self.predeclare_functions(statements);
        self.check_recursive_requirements(statements, &functions);
        for statement in statements {
            match statement {
                Stmt::RecordDecl(record) => self.check_record_decl(record),
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
        self.check_value_block_with_expected(block, None)
    }

    fn check_value_block_with_expected(
        &mut self,
        block: &ValueBlock,
        expected: Option<Type>,
    ) -> Type {
        self.push_scope(false);
        let functions = self.predeclare_functions(&block.statements);
        self.check_recursive_requirements(&block.statements, &functions);
        for statement in &block.statements {
            match statement {
                Stmt::FuncDecl(func) => self.check_func_decl(func, &functions),
                _ => self.check_stmt(statement),
            }
        }
        let ty = self.check_expr_with_expected(&block.expr, expected);
        self.pop_scope();
        ty
    }

    fn check_stmt(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Assign(stmt) => self.check_assign(stmt),
            Stmt::RecordDecl(_) => {}
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
        let name = self.symbol(&stmt.name);
        if stmt.mutable {
            self.insert_current(name, BindingKind::Mutable, value_ty);
            return;
        }

        if let Some(binding) = self.lookup_in_current_function(name).cloned() {
            if binding.kind == BindingKind::Mutable {
                self.require_exact(&binding.ty, &value_ty, stmt.span, "T002");
            }
            return;
        }

        if self.lookup_beyond_current_function(name).is_none() {
            self.insert_current(name, BindingKind::Immutable, value_ty);
        }
    }

    fn check_func_decl(&mut self, func: &FuncDecl, local_functions: &HashMap<Symbol, FunctionSig>) {
        let name = self.symbol(&func.name);
        let Some(sig) = local_functions.get(&name).cloned() else {
            return;
        };

        self.push_scope(true);
        for (param, param_ty) in func.params.iter().zip(sig.params.iter().cloned()) {
            let name = self.symbol(&param.name);
            self.insert_current(name, BindingKind::Parameter, param_ty);
        }
        let nested_functions = self.predeclare_functions(&func.body.statements);
        self.check_recursive_requirements(&func.body.statements, &nested_functions);
        for statement in &func.body.statements {
            match statement {
                Stmt::FuncDecl(nested) => self.check_func_decl(nested, &nested_functions),
                _ => self.check_stmt(statement),
            }
        }
        self.check_expr_with_expected(&func.body.expr, Some((*sig.ret).clone()));
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
        self.check_expr_with_expected(expr, None)
    }

    fn check_expr_with_expected(&mut self, expr: &Expr, expected: Option<Type>) -> Type {
        let span = expr.span();
        let ty = match expr {
            Expr::Int(_) => self.apply_expected(Type::Int, expected, expr.span()),
            Expr::Bool(_) => self.apply_expected(Type::Bool, expected, expr.span()),
            Expr::String(_) => self.apply_expected(Type::String, expected, expr.span()),
            Expr::Ident(expr) => {
                let name = self.symbol(&expr.name);
                if let Some(binding) = self.lookup(name).cloned() {
                    self.identifier_refs.push(TypedIdentifier {
                        expr_id: expr.id,
                        name,
                        span: expr.span,
                        binding: binding.id,
                    });
                    self.apply_expected(binding.ty, expected, expr.span)
                } else {
                    Type::Error
                }
            }
            Expr::RecordLit(expr) => {
                let ty = self.check_record_lit(expr);
                self.apply_expected(ty, expected, expr.span)
            }
            Expr::Field(expr) => {
                let ty = self.check_field_expr(expr);
                self.apply_expected(ty, expected, expr.span)
            }
            Expr::RecordUpdate(expr) => {
                let ty = self.check_record_update(expr);
                self.apply_expected(ty, expected, expr.span)
            }
            Expr::Unary(expr) => {
                let ty = match expr.op {
                    UnaryOp::Neg => self.check_expr_with_expected(&expr.expr, Some(Type::Int)),
                    UnaryOp::Not => self.check_expr_with_expected(&expr.expr, Some(Type::Bool)),
                };
                match expr.op {
                    UnaryOp::Neg => {
                        self.require_exact(&ty, &Type::Int, expr.span, "T001");
                        self.apply_expected(Type::Int, expected, expr.span)
                    }
                    UnaryOp::Not => {
                        self.require_exact(&ty, &Type::Bool, expr.span, "T001");
                        self.apply_expected(Type::Bool, expected, expr.span)
                    }
                }
            }
            Expr::Binary(expr) => match expr.op {
                BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div => {
                    let left = self.check_expr_with_expected(&expr.left, Some(Type::Int));
                    let right = self.check_expr_with_expected(&expr.right, Some(Type::Int));
                    self.require_exact(&left, &Type::Int, expr.left.span(), "T001");
                    self.require_exact(&right, &Type::Int, expr.right.span(), "T001");
                    self.apply_expected(Type::Int, expected, expr.span)
                }
                BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => {
                    let left = self.check_expr_with_expected(&expr.left, Some(Type::Int));
                    let right = self.check_expr_with_expected(&expr.right, Some(Type::Int));
                    self.require_exact(&left, &Type::Int, expr.left.span(), "T001");
                    self.require_exact(&right, &Type::Int, expr.right.span(), "T001");
                    self.apply_expected(Type::Bool, expected, expr.span)
                }
                BinaryOp::EqEq | BinaryOp::BangEq => {
                    let left = self.check_expr(&expr.left);
                    let right = self.check_expr_with_expected(&expr.right, Some(left.clone()));
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
                    self.apply_expected(Type::Bool, expected, expr.span)
                }
            },
            Expr::Call(expr) => {
                let callee_ty = self.check_expr(&expr.callee);
                match self.resolve_type(&callee_ty) {
                    Type::Builtin(BuiltinFunction::Print | BuiltinFunction::Println) => {
                        if expr.args.len() != 1 {
                            self.diagnostics.push(Diagnostic::new(
                                "T004",
                                format!("expected 1 arguments but found {}", expr.args.len()),
                                expr.span,
                            ));
                            Type::Error
                        } else {
                            let arg_ty =
                                self.check_expr_with_expected(&expr.args[0], expected.clone());
                            let arg_ty = self.resolve_type(&arg_ty);
                            match arg_ty {
                                Type::Int | Type::Bool | Type::String => {
                                    self.apply_expected(arg_ty, expected, expr.span)
                                }
                                Type::Unknown(_) => {
                                    self.diagnostics.push(Diagnostic::new(
                                        "E005",
                                        "type annotation required because inference is not unique",
                                        expr.span,
                                    ));
                                    Type::Error
                                }
                                _ => {
                                    let builtin_name = match self.resolve_type(&callee_ty) {
                                        Type::Builtin(BuiltinFunction::Print) => "print",
                                        Type::Builtin(BuiltinFunction::Println) => "println",
                                        _ => unreachable!("matched builtin branch"),
                                    };
                                    self.diagnostics.push(Diagnostic::new(
                                        "T006",
                                        format!(
                                            "`{builtin_name}` accepts only Int, Bool, or String"
                                        ),
                                        expr.span,
                                    ));
                                    Type::Error
                                }
                            }
                        }
                    }
                    Type::Function(sig) => {
                        if sig.params.len() != expr.args.len() {
                            self.diagnostics.push(Diagnostic::new(
                                "T004",
                                format!(
                                    "expected {} arguments but found {}",
                                    sig.params.len(),
                                    expr.args.len()
                                ),
                                expr.span,
                            ));
                            Type::Error
                        } else {
                            for (arg, param_ty) in expr.args.iter().zip(sig.params.iter()) {
                                self.check_expr_with_expected(arg, Some(param_ty.clone()));
                            }
                            self.apply_expected(*sig.ret.clone(), expected, expr.span)
                        }
                    }
                    Type::Unknown(_) => {
                        let arg_tys: Vec<Type> =
                            expr.args.iter().map(|arg| self.check_expr(arg)).collect();
                        let ret_ty =
                            expected.unwrap_or_else(|| Type::Unknown(self.fresh_unknown()));
                        let inferred_sig = Type::Function(FunctionSig {
                            params: arg_tys,
                            ret: Box::new(ret_ty.clone()),
                        });
                        if let Err(message) = self.unify(callee_ty, inferred_sig) {
                            self.diagnostics
                                .push(Diagnostic::new("T005", message, expr.span));
                            Type::Error
                        } else {
                            self.resolve_type(&ret_ty)
                        }
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
                match expected {
                    Some(expected_ty) => {
                        self.check_value_block_with_expected(
                            &expr.then_branch,
                            Some(expected_ty.clone()),
                        );
                        self.check_value_block_with_expected(
                            &expr.else_branch,
                            Some(expected_ty.clone()),
                        );
                        self.resolve_type(&expected_ty)
                    }
                    None => {
                        let then_ty = self.check_value_block(&expr.then_branch);
                        let else_ty = self.check_value_block(&expr.else_branch);
                        self.require_exact(&then_ty, &else_ty, expr.span, "T002");
                        self.resolve_type(&then_ty)
                    }
                }
            }
            Expr::Fn(expr) => {
                let sig = self.signature_from_fn_expr(expr, expected.as_ref());
                self.push_scope(true);
                for (param, param_ty) in expr.params.iter().zip(sig.params.iter().cloned()) {
                    let name = self.symbol(&param.name);
                    self.insert_current(name, BindingKind::Parameter, param_ty);
                }
                let nested_functions = self.predeclare_functions(&expr.body.statements);
                self.check_recursive_requirements(&expr.body.statements, &nested_functions);
                for statement in &expr.body.statements {
                    match statement {
                        Stmt::FuncDecl(nested) => self.check_func_decl(nested, &nested_functions),
                        _ => self.check_stmt(statement),
                    }
                }
                self.check_expr_with_expected(&expr.body.expr, Some((*sig.ret).clone()));
                self.pop_scope();
                self.apply_expected(Type::Function(sig), expected, expr.span)
            }
        };
        let resolved = self.resolve_type(&ty);
        self.expr_types.push(ExprType {
            expr_id: expr.id(),
            span,
            ty: resolved.clone(),
        });
        resolved
    }

    fn predeclare_records(&mut self, statements: &[Stmt]) {
        for statement in statements {
            let Stmt::RecordDecl(record) = statement else {
                continue;
            };
            let name = self.symbol(&record.name);
            if self.records.contains_key(&name) {
                self.diagnostics.push(Diagnostic::new(
                    "E002",
                    format!("duplicate record `{}` in the current scope", record.name),
                    record.span,
                ));
                continue;
            }
            let mut fields = Vec::new();
            for field in &record.fields {
                fields.push(RecordField {
                    name: self.symbol(&field.name),
                    type_name: field.type_name.clone(),
                    span: field.span,
                });
            }
            self.records.insert(name, RecordDef { fields });
        }
    }

    fn check_record_decl(&mut self, record: &RecordDecl) {
        let mut field_names = HashSet::new();
        for field in &record.fields {
            let field_name = self.symbol(&field.name);
            if !field_names.insert(field_name) {
                self.diagnostics.push(Diagnostic::new(
                    "E002",
                    format!(
                        "duplicate field `{}` in record `{}`",
                        field.name, record.name
                    ),
                    field.span,
                ));
            }
            let field_ty = self.type_from_expr(&field.type_name, field.span);
            if matches!(self.resolve_type(&field_ty), Type::Function(_)) {
                self.diagnostics.push(Diagnostic::new(
                    "E011",
                    "record fields may not have function type in v1",
                    field.span,
                ));
            }
        }
    }

    fn check_record_lit(&mut self, expr: &RecordLitExpr) -> Type {
        let type_name = self.symbol(&expr.type_name);
        let Some(record) = self.records.get(&type_name).cloned() else {
            self.diagnostics.push(Diagnostic::new(
                "T007",
                format!("unknown type `{}`", expr.type_name),
                expr.span,
            ));
            for field in &expr.fields {
                self.check_expr(&field.value);
            }
            return Type::Error;
        };

        let mut seen = HashSet::new();
        let mut has_error = false;
        for field in &expr.fields {
            let value_ty = self.check_expr(&field.value);
            let field_name = self.symbol(&field.name);
            if !seen.insert(field_name) {
                self.diagnostics.push(Diagnostic::new(
                    "E009",
                    format!(
                        "invalid record literal for `{}`: duplicate field `{}`",
                        expr.type_name, field.name
                    ),
                    field.span,
                ));
                has_error = true;
                continue;
            }

            let Some(declared) = find_record_field(&record, field_name) else {
                self.diagnostics.push(Diagnostic::new(
                    "E009",
                    format!(
                        "invalid record literal for `{}`: unknown field `{}`",
                        expr.type_name, field.name
                    ),
                    field.span,
                ));
                has_error = true;
                continue;
            };

            let field_ty = self.type_from_expr(&declared.type_name, declared.span);
            if let Err(message) = self.unify(field_ty, value_ty) {
                self.diagnostics
                    .push(Diagnostic::new("E009", message, field.span));
                has_error = true;
            }
        }

        for declared in &record.fields {
            if !seen.contains(&declared.name) {
                self.diagnostics.push(Diagnostic::new(
                    "E009",
                    format!(
                        "invalid record literal for `{}`: missing field `{}`",
                        expr.type_name,
                        self.symbols.resolve(declared.name)
                    ),
                    expr.span,
                ));
                has_error = true;
            }
        }

        if has_error {
            Type::Error
        } else {
            Type::Record(type_name)
        }
    }

    fn check_field_expr(&mut self, expr: &FieldExpr) -> Type {
        let base_ty = self.check_expr(&expr.base);
        let resolved_base = self.resolve_type(&base_ty);
        let Type::Record(record_name) = resolved_base else {
            self.diagnostics.push(Diagnostic::new(
                "T008",
                "field access requires a record value",
                expr.span,
            ));
            return Type::Error;
        };

        let Some(record) = self.records.get(&record_name).cloned() else {
            let record_name = self.symbols.resolve(record_name);
            self.diagnostics.push(Diagnostic::new(
                "T007",
                format!("unknown type `{record_name}`"),
                expr.span,
            ));
            return Type::Error;
        };

        let field_name = self.symbol(&expr.field);
        let Some(field) = find_record_field(&record, field_name) else {
            self.diagnostics.push(Diagnostic::new(
                "E008",
                format!("unknown field `{}`", expr.field),
                expr.span,
            ));
            return Type::Error;
        };

        self.type_from_expr(&field.type_name, field.span)
    }

    fn check_record_update(&mut self, expr: &RecordUpdateExpr) -> Type {
        let base_ty = self.check_expr(&expr.base);
        let resolved_base = self.resolve_type(&base_ty);
        let Type::Record(record_name) = resolved_base else {
            self.diagnostics
                .push(Diagnostic::new("E012", "invalid record update", expr.span));
            for field in &expr.fields {
                self.check_expr(&field.value);
            }
            return Type::Error;
        };

        let Some(record) = self.records.get(&record_name).cloned() else {
            let record_name = self.symbols.resolve(record_name);
            self.diagnostics.push(Diagnostic::new(
                "T007",
                format!("unknown type `{record_name}`"),
                expr.span,
            ));
            return Type::Error;
        };

        let mut seen = HashSet::new();
        let mut has_error = false;
        for field in &expr.fields {
            let value_ty = self.check_expr(&field.value);
            let field_name = self.symbol(&field.name);
            if !seen.insert(field_name) {
                self.diagnostics
                    .push(Diagnostic::new("E012", "invalid record update", field.span));
                has_error = true;
                continue;
            }

            let Some(declared) = find_record_field(&record, field_name) else {
                self.diagnostics
                    .push(Diagnostic::new("E012", "invalid record update", field.span));
                has_error = true;
                continue;
            };

            let field_ty = self.type_from_expr(&declared.type_name, declared.span);
            if let Err(message) = self.unify(field_ty, value_ty) {
                self.diagnostics
                    .push(Diagnostic::new("E012", message, field.span));
                has_error = true;
            }
        }

        if has_error {
            Type::Error
        } else {
            Type::Record(record_name)
        }
    }

    fn signature_from_fn_expr(&mut self, expr: &FnExpr, expected: Option<&Type>) -> FunctionSig {
        let expected_sig = self.expected_function_sig(expected, expr.params.len());
        let params = expr
            .params
            .iter()
            .enumerate()
            .map(|(index, param)| match param.type_name.as_ref() {
                Some(type_name) => self.type_from_expr(type_name, param.span),
                None => expected_sig
                    .as_ref()
                    .and_then(|sig| sig.params.get(index).cloned())
                    .unwrap_or_else(|| Type::Unknown(self.fresh_unknown())),
            })
            .collect();
        let ret = match expr.return_type.as_ref() {
            Some(type_name) => self.type_from_expr(type_name, expr.span),
            None => expected_sig
                .map(|sig| *sig.ret)
                .unwrap_or_else(|| Type::Unknown(self.fresh_unknown())),
        };
        FunctionSig {
            params,
            ret: Box::new(ret),
        }
    }

    fn predeclare_functions(&mut self, statements: &[Stmt]) -> HashMap<Symbol, FunctionSig> {
        let mut functions = HashMap::new();
        for statement in statements {
            if let Stmt::FuncDecl(func) = statement {
                let name = self.symbol(&func.name);
                let params = func
                    .params
                    .iter()
                    .map(|param| match param.type_name.as_ref() {
                        Some(type_name) => self.type_from_expr(type_name, param.span),
                        None => Type::Unknown(self.fresh_unknown()),
                    })
                    .collect::<Vec<_>>();
                let ret = match func.return_type.as_ref() {
                    Some(type_name) => self.type_from_expr(type_name, func.span),
                    None => Type::Unknown(self.fresh_unknown()),
                };
                let sig = FunctionSig {
                    params,
                    ret: Box::new(ret),
                };
                functions.insert(name, sig.clone());
                self.insert_current(name, BindingKind::Function, Type::Function(sig));
            }
        }
        functions
    }

    fn check_recursive_requirements(
        &mut self,
        statements: &[Stmt],
        functions: &HashMap<Symbol, FunctionSig>,
    ) {
        let names: HashSet<Symbol> = functions.keys().copied().collect();
        let decls: Vec<&FuncDecl> = statements
            .iter()
            .filter_map(|stmt| match stmt {
                Stmt::FuncDecl(func) => Some(func),
                _ => None,
            })
            .collect();
        let graph = build_call_graph(&decls, &names, &mut self.symbols);
        let components = strongly_connected_components(&graph);

        for component in components {
            if component.len() > 1 {
                for name in component {
                    if let Some(func) = decls
                        .iter()
                        .find(|func| self.symbols.lookup(&func.name) == Some(name))
                    {
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
            if let Some(func) = decls
                .iter()
                .find(|func| self.symbols.lookup(&func.name) == Some(*name))
            {
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

    fn type_from_expr(&mut self, type_expr: &TypeExpr, span: crate::span::Span) -> Type {
        match type_expr {
            TypeExpr::Int => Type::Int,
            TypeExpr::Bool => Type::Bool,
            TypeExpr::String => Type::String,
            TypeExpr::Named(name) => {
                let symbol = self.symbol(name);
                if self.records.contains_key(&symbol) {
                    Type::Record(symbol)
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "T007",
                        format!("unknown type `{name}`"),
                        span,
                    ));
                    Type::Error
                }
            }
            TypeExpr::Function(function) => Type::Function(FunctionSig {
                params: function
                    .params
                    .iter()
                    .map(|param| self.type_from_expr(param, span))
                    .collect(),
                ret: Box::new(self.type_from_expr(&function.ret, span)),
            }),
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
            (Type::Record(left), Type::Record(right)) if left == right => Ok(Type::Record(left)),
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

    fn type_info_for(&self, ty: &Type) -> TypeInfo {
        match self.resolve_type(ty) {
            Type::Int => TypeInfo::Int,
            Type::Bool => TypeInfo::Bool,
            Type::String => TypeInfo::String,
            Type::Record(symbol) => TypeInfo::Record(symbol),
            Type::Function(sig) => TypeInfo::Function(FunctionTypeInfo {
                params: sig.params.iter().map(|ty| self.type_info_for(ty)).collect(),
                ret: Box::new(self.type_info_for(&sig.ret)),
            }),
            Type::Builtin(BuiltinFunction::Print) => TypeInfo::Builtin("print"),
            Type::Builtin(BuiltinFunction::Println) => TypeInfo::Builtin("println"),
            Type::Unknown(_) => TypeInfo::Unknown,
            Type::Error => TypeInfo::Error,
        }
    }

    fn apply_expected(
        &mut self,
        inferred: Type,
        expected: Option<Type>,
        span: crate::span::Span,
    ) -> Type {
        let inferred = self.resolve_type(&inferred);
        let Some(expected) = expected else {
            return inferred;
        };
        match self.unify(inferred, expected) {
            Ok(ty) => self.resolve_type(&ty),
            Err(message) => {
                self.diagnostics
                    .push(Diagnostic::new("T002", message, span));
                Type::Error
            }
        }
    }

    fn expected_function_sig(&self, expected: Option<&Type>, arity: usize) -> Option<FunctionSig> {
        let expected = expected?;
        match self.resolve_type(expected) {
            Type::Function(sig) if sig.params.len() == arity => Some(sig),
            _ => None,
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

    fn insert_current(&mut self, name: Symbol, kind: BindingKind, ty: Type) -> BindingId {
        let id = BindingId::new(self.bindings.len() as u32);
        self.bindings.push(Binding {
            id,
            symbol: name,
            kind,
            ty,
        });
        if let Some(scope) = self.scopes.last_mut() {
            scope.bindings.insert(name, id);
        }
        id
    }

    fn lookup(&self, name: Symbol) -> Option<&Binding> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.bindings.get(&name).map(|id| self.binding(*id)))
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

    fn binding(&self, id: BindingId) -> &Binding {
        &self.bindings[id.as_u32() as usize]
    }

    fn symbol(&mut self, name: &str) -> Symbol {
        self.symbols.intern(name)
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
            Self::Record(_) => "Record",
            Self::Function(_) => "Function",
            Self::Builtin(BuiltinFunction::Print) => "Builtin(print)",
            Self::Builtin(BuiltinFunction::Println) => "Builtin(println)",
            Self::Unknown(_) => "Unknown",
            Self::Error => "Error",
        }
    }
}

fn find_record_field(record: &RecordDef, name: Symbol) -> Option<&RecordField> {
    record.fields.iter().find(|field| field.name == name)
}

fn build_call_graph(
    decls: &[&FuncDecl],
    local_names: &HashSet<Symbol>,
    symbols: &mut SymbolTable,
) -> HashMap<Symbol, HashSet<Symbol>> {
    let mut graph = HashMap::new();
    for decl in decls {
        let mut calls = HashSet::new();
        collect_calls_in_statements(&decl.body.statements, local_names, &mut calls, symbols);
        collect_calls_in_expr(&decl.body.expr, local_names, &mut calls, symbols);
        graph.insert(symbols.intern(&decl.name), calls);
    }
    graph
}

fn strongly_connected_components(graph: &HashMap<Symbol, HashSet<Symbol>>) -> Vec<Vec<Symbol>> {
    let mut index = 0usize;
    let mut stack = Vec::<Symbol>::new();
    let mut indices = HashMap::<Symbol, usize>::new();
    let mut lowlinks = HashMap::<Symbol, usize>::new();
    let mut on_stack = HashSet::<Symbol>::new();
    let mut components = Vec::new();

    for node in graph.keys() {
        if !indices.contains_key(node) {
            strong_connect(
                *node,
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
    node: Symbol,
    graph: &HashMap<Symbol, HashSet<Symbol>>,
    index: &mut usize,
    stack: &mut Vec<Symbol>,
    indices: &mut HashMap<Symbol, usize>,
    lowlinks: &mut HashMap<Symbol, usize>,
    on_stack: &mut HashSet<Symbol>,
    components: &mut Vec<Vec<Symbol>>,
) {
    indices.insert(node, *index);
    lowlinks.insert(node, *index);
    *index += 1;
    stack.push(node);
    on_stack.insert(node);

    if let Some(neighbors) = graph.get(&node) {
        for neighbor in neighbors {
            if !indices.contains_key(neighbor) {
                strong_connect(
                    *neighbor, graph, index, stack, indices, lowlinks, on_stack, components,
                );
                let neighbor_low = lowlinks[neighbor];
                let node_low = lowlinks[&node];
                lowlinks.insert(node, node_low.min(neighbor_low));
            } else if on_stack.contains(neighbor) {
                let neighbor_index = indices[neighbor];
                let node_low = lowlinks[&node];
                lowlinks.insert(node, node_low.min(neighbor_index));
            }
        }
    }

    if lowlinks[&node] == indices[&node] {
        let mut component = Vec::new();
        while let Some(candidate) = stack.pop() {
            on_stack.remove(&candidate);
            component.push(candidate);
            if candidate == node {
                break;
            }
        }
        components.push(component);
    }
}

fn collect_calls_in_statements(
    statements: &[Stmt],
    local_names: &HashSet<Symbol>,
    calls: &mut HashSet<Symbol>,
    symbols: &mut SymbolTable,
) {
    for statement in statements {
        match statement {
            Stmt::Assign(stmt) => collect_calls_in_expr(&stmt.value, local_names, calls, symbols),
            Stmt::RecordDecl(_) => {}
            Stmt::FuncDecl(_) => {}
            Stmt::If(stmt) => {
                collect_calls_in_expr(&stmt.condition, local_names, calls, symbols);
                collect_calls_in_statements(
                    &stmt.then_branch.statements,
                    local_names,
                    calls,
                    symbols,
                );
                if let Some(else_branch) = &stmt.else_branch {
                    collect_calls_in_statements(
                        &else_branch.statements,
                        local_names,
                        calls,
                        symbols,
                    );
                }
            }
            Stmt::While(stmt) => {
                collect_calls_in_expr(&stmt.condition, local_names, calls, symbols);
                collect_calls_in_statements(&stmt.body.statements, local_names, calls, symbols);
            }
            Stmt::Expr(stmt) => collect_calls_in_expr(&stmt.expr, local_names, calls, symbols),
        }
    }
}

fn collect_calls_in_expr(
    expr: &Expr,
    local_names: &HashSet<Symbol>,
    calls: &mut HashSet<Symbol>,
    symbols: &mut SymbolTable,
) {
    match expr {
        Expr::Int(_) | Expr::Bool(_) | Expr::String(_) | Expr::Ident(_) => {}
        Expr::RecordLit(expr) => {
            for field in &expr.fields {
                collect_calls_in_expr(&field.value, local_names, calls, symbols);
            }
        }
        Expr::Field(expr) => collect_calls_in_expr(&expr.base, local_names, calls, symbols),
        Expr::RecordUpdate(expr) => {
            collect_calls_in_expr(&expr.base, local_names, calls, symbols);
            for field in &expr.fields {
                collect_calls_in_expr(&field.value, local_names, calls, symbols);
            }
        }
        Expr::Unary(expr) => collect_calls_in_expr(&expr.expr, local_names, calls, symbols),
        Expr::Binary(expr) => {
            collect_calls_in_expr(&expr.left, local_names, calls, symbols);
            collect_calls_in_expr(&expr.right, local_names, calls, symbols);
        }
        Expr::Call(expr) => {
            if let Expr::Ident(ident) = expr.callee.as_ref() {
                let name = symbols.intern(&ident.name);
                if local_names.contains(&name) {
                    calls.insert(name);
                }
            }
            collect_calls_in_expr(&expr.callee, local_names, calls, symbols);
            for arg in &expr.args {
                collect_calls_in_expr(arg, local_names, calls, symbols);
            }
        }
        Expr::If(expr) => {
            collect_calls_in_expr(&expr.condition, local_names, calls, symbols);
            collect_calls_in_statements(&expr.then_branch.statements, local_names, calls, symbols);
            collect_calls_in_expr(&expr.then_branch.expr, local_names, calls, symbols);
            collect_calls_in_statements(&expr.else_branch.statements, local_names, calls, symbols);
            collect_calls_in_expr(&expr.else_branch.expr, local_names, calls, symbols);
        }
        Expr::Fn(_) => {}
    }
}
