use std::collections::{HashMap, HashSet};

use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::identity::{BindingId, BindingKind, ExprId, PackageItemId, StmtId};
use crate::known_enum;
use crate::prelude::{self, BuiltinId, BuiltinKind};
use crate::span::Span;
use crate::symbol::{Symbol, SymbolTable};
pub use crate::types::{FunctionTypeInfo, TypeInfo};

#[derive(Clone, Debug)]
pub struct TypeCheckOutput {
    pub diagnostics: Vec<Diagnostic>,
    pub bindings: Vec<TypedBindingInfo>,
    pub assignment_targets: Vec<TypedAssignmentTarget>,
    pub identifier_refs: Vec<TypedIdentifier>,
    pub calls: Vec<TypedCallInfo>,
    pub expr_types: Vec<ExprTypeInfo>,
    pub symbols: SymbolTable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedBindingInfo {
    pub id: BindingId,
    pub symbol: Symbol,
    pub kind: BindingKind,
    pub ty: TypeInfo,
    pub span: Span,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedAssignmentTarget {
    pub stmt_id: StmtId,
    pub name: Symbol,
    pub span: Span,
    pub binding: BindingId,
    pub is_update: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypedIdentifier {
    pub expr_id: ExprId,
    pub name: Symbol,
    pub span: Span,
    pub binding: BindingId,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TypedCallInfo {
    pub expr_id: ExprId,
    pub span: Span,
    pub callee: TypedCalleeInfo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypedCalleeInfo {
    Binding(BindingId),
    PackageItem {
        binding: BindingId,
        item: PackageItemId,
    },
    EnumVariant {
        binding: BindingId,
        enum_name: Symbol,
        enum_item: Option<PackageItemId>,
        variant_name: Symbol,
    },
    Builtin {
        binding: BindingId,
        name: &'static str,
    },
    Value,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExprTypeInfo {
    pub expr_id: ExprId,
    pub span: Span,
    pub ty: TypeInfo,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Type {
    Int,
    Bool,
    String,
    Record(Symbol),
    Enum(Symbol, Vec<Type>),
    GenericParam(Symbol),
    List(Box<Type>),
    Map(Box<Type>, Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
    OptionNone,
    EnumConstructor {
        enum_name: Symbol,
        enum_item: Option<PackageItemId>,
        variant_name: Symbol,
    },
    Function(FunctionSig),
    Builtin(BuiltinId),
    Unknown(u32),
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FunctionSig {
    params: Vec<Type>,
    ret: Box<Type>,
}

#[derive(Clone, Debug)]
struct Binding {
    id: BindingId,
    symbol: Symbol,
    kind: BindingKind,
    ty: Type,
    span: Span,
}

#[derive(Clone, Debug)]
struct ExprType {
    expr_id: ExprId,
    span: Span,
    ty: Type,
}

#[derive(Clone, Debug)]
struct RecordDef {
    span: Span,
    fields: Vec<RecordField>,
}

#[derive(Clone, Debug)]
struct RecordField {
    name: Symbol,
    type_name: TypeExpr,
    span: Span,
}

#[derive(Clone, Debug)]
struct EnumDef {
    span: Span,
    type_params: Vec<Symbol>,
    variants: Vec<EnumVariantDef>,
}

#[derive(Clone, Debug)]
struct EnumVariantDef {
    name: Symbol,
    payload: Option<TypeExpr>,
    span: Span,
}

#[derive(Clone, Debug)]
struct EnumMatchSpec {
    enum_name: Symbol,
    display_name: String,
    variants: Vec<EnumMatchVariant>,
}

impl EnumMatchSpec {
    fn variant(&self, name: Symbol) -> Option<&EnumMatchVariant> {
        self.variants.iter().find(|variant| variant.name == name)
    }
}

#[derive(Clone, Debug)]
struct EnumMatchVariant {
    name: Symbol,
    payload: Option<Type>,
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
    checker.predeclare_enums(&program.statements);
    checker.check_scope_statements(&program.statements);
    checker.into_output()
}

struct TypeChecker {
    scopes: Vec<ScopeFrame>,
    records: HashMap<Symbol, RecordDef>,
    enums: HashMap<Symbol, EnumDef>,
    bindings: Vec<Binding>,
    assignment_targets: Vec<TypedAssignmentTarget>,
    identifier_refs: Vec<TypedIdentifier>,
    calls: Vec<TypedCallInfo>,
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
            enums: HashMap::new(),
            bindings: Vec::new(),
            assignment_targets: Vec::new(),
            identifier_refs: Vec::new(),
            calls: Vec::new(),
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
                span: binding.span,
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
            assignment_targets: self.assignment_targets,
            identifier_refs: self.identifier_refs,
            calls: self.calls,
            expr_types,
            symbols: self.symbols,
        }
    }

    fn install_prelude(&mut self) {
        for builtin in prelude::builtins() {
            let kind = match builtin.kind {
                BuiltinKind::Function => BindingKind::Function,
                BuiltinKind::Value => BindingKind::Immutable,
            };
            let ty = if builtin.id == BuiltinId::OptionNone {
                Type::OptionNone
            } else {
                Type::Builtin(builtin.id)
            };
            let symbol = self.symbol(builtin.name);
            self.insert_current(symbol, kind, ty, Span::default());
        }
    }

    fn check_scope_statements(&mut self, statements: &[Stmt]) {
        let functions = self.predeclare_functions(statements);
        self.check_recursive_requirements(statements, &functions);
        for statement in statements {
            match statement {
                Stmt::RecordDecl(record) => self.check_record_decl(record),
                Stmt::EnumDecl(enumeration) => self.check_enum_decl(enumeration),
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
            Stmt::EnumDecl(_) => {}
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
        let annotation_ty = stmt
            .type_name
            .as_ref()
            .map(|type_name| self.type_from_expr(type_name, stmt.span));
        let value_ty = match annotation_ty.clone() {
            Some(expected) => self.check_expr_with_expected(&stmt.value, Some(expected)),
            None => self.check_expr(&stmt.value),
        };
        let binding_ty = annotation_ty.unwrap_or_else(|| value_ty.clone());
        let name = self.symbol(&stmt.name);
        if stmt.mutable {
            let binding = self.insert_current(name, BindingKind::Mutable, binding_ty, stmt.span);
            self.assignment_targets.push(TypedAssignmentTarget {
                stmt_id: stmt.id,
                name,
                span: stmt.span,
                binding,
                is_update: false,
            });
            return;
        }

        if let Some(binding) = self.lookup_in_current_function(name).cloned() {
            if stmt.type_name.is_some() {
                self.diagnostics.push(Diagnostic::new(
                    "T014",
                    "type annotations are allowed only on new local bindings",
                    stmt.span,
                ));
            }
            if binding.kind == BindingKind::Mutable {
                self.require_exact(&binding.ty, &value_ty, stmt.span, "T002");
            }
            self.assignment_targets.push(TypedAssignmentTarget {
                stmt_id: stmt.id,
                name,
                span: stmt.span,
                binding: binding.id,
                is_update: true,
            });
            return;
        }

        if self.lookup_beyond_current_function(name).is_none() {
            let binding = self.insert_current(name, BindingKind::Immutable, binding_ty, stmt.span);
            self.assignment_targets.push(TypedAssignmentTarget {
                stmt_id: stmt.id,
                name,
                span: stmt.span,
                binding,
                is_update: false,
            });
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
            self.insert_current(name, BindingKind::Parameter, param_ty, param.span);
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

    fn check_call_callee(&mut self, callee: &Expr) -> Type {
        if let Expr::Ident(ident) = callee {
            let name = self.symbol(&ident.name);
            if let Some(binding) = self.lookup(name).cloned()
                && matches!(binding.ty, Type::EnumConstructor { .. })
            {
                self.identifier_refs.push(TypedIdentifier {
                    expr_id: ident.id,
                    name,
                    span: ident.span,
                    binding: binding.id,
                });
                return self.record_expr_type(ident.id, ident.span, binding.ty);
            }
        }
        self.check_expr(callee)
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
                    if matches!(binding.ty, Type::OptionNone) {
                        let ty = self.check_option_none(expected, expr.span);
                        return self.record_expr_type(expr.id, span, ty);
                    }
                    if let Type::EnumConstructor {
                        enum_name,
                        enum_item: _,
                        variant_name,
                    } = binding.ty
                    {
                        let ty = self.check_user_enum_constructor(
                            expr.span,
                            expected,
                            enum_name,
                            variant_name,
                            &[],
                        );
                        return self.record_expr_type(expr.id, span, ty);
                    }
                    self.apply_expected(binding.ty, expected, expr.span)
                } else {
                    if let Some((enum_name, variant_name)) = split_variant_name(&expr.name) {
                        self.diagnose_unknown_enum_variant(enum_name, variant_name, expr.span);
                    }
                    Type::Error
                }
            }
            Expr::ListLit(expr) => self.check_list_lit(expr, expected),
            Expr::Index(expr) => self.check_index_expr(expr, expected),
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
                let callee_ty = self.check_call_callee(&expr.callee);
                let ty = match self.resolve_type(&callee_ty) {
                    Type::Builtin(BuiltinId::Print | BuiltinId::Println) => {
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
                                        Type::Builtin(builtin) => Self::builtin_name(builtin),
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
                    Type::Builtin(BuiltinId::Len | BuiltinId::IsEmpty) => {
                        let builtin = match self.resolve_type(&callee_ty) {
                            Type::Builtin(builtin) => builtin,
                            _ => unreachable!("matched builtin branch"),
                        };
                        if expr.args.len() != 1 {
                            self.diagnostics.push(Diagnostic::new(
                                "T004",
                                format!("expected 1 arguments but found {}", expr.args.len()),
                                expr.span,
                            ));
                            Type::Error
                        } else {
                            let arg_ty = self.check_expr(&expr.args[0]);
                            match self.resolve_type(&arg_ty) {
                                Type::List(_) | Type::Map(_, _) => {
                                    let ret = match builtin {
                                        BuiltinId::Len => Type::Int,
                                        BuiltinId::IsEmpty => Type::Bool,
                                        _ => unreachable!("matched collection query builtin"),
                                    };
                                    self.apply_expected(ret, expected, expr.span)
                                }
                                Type::Unknown(_) => {
                                    self.diagnostics.push(Diagnostic::new(
                                        "E005",
                                        "type annotation required because inference is not unique",
                                        expr.span,
                                    ));
                                    Type::Error
                                }
                                Type::Error => Type::Error,
                                _ => {
                                    self.diagnostics.push(Diagnostic::new(
                                        "T006",
                                        format!(
                                            "`{}` expects List[T] or Map[K, V] as its first argument",
                                            Self::builtin_name(builtin)
                                        ),
                                        expr.span,
                                    ));
                                    Type::Error
                                }
                            }
                        }
                    }
                    Type::Builtin(BuiltinId::Push) => {
                        if expr.args.len() != 2 {
                            self.diagnostics.push(Diagnostic::new(
                                "T004",
                                format!("expected 2 arguments but found {}", expr.args.len()),
                                expr.span,
                            ));
                            Type::Error
                        } else {
                            let item_expected = Type::Unknown(self.fresh_unknown());
                            let list_expected = Type::List(Box::new(item_expected.clone()));
                            let list_ty =
                                self.check_expr_with_expected(&expr.args[0], Some(list_expected));
                            match self.resolve_type(&list_ty) {
                                Type::List(item_ty) => {
                                    self.check_expr_with_expected(
                                        &expr.args[1],
                                        Some((*item_ty).clone()),
                                    );
                                    self.apply_expected(Type::List(item_ty), expected, expr.span)
                                }
                                Type::Unknown(_) => {
                                    self.diagnostics.push(Diagnostic::new(
                                        "E005",
                                        "type annotation required because inference is not unique",
                                        expr.span,
                                    ));
                                    Type::Error
                                }
                                Type::Error => Type::Error,
                                _ => {
                                    self.diagnostics.push(Diagnostic::new(
                                        "T006",
                                        "`push` expects List[T] as its first argument",
                                        expr.span,
                                    ));
                                    Type::Error
                                }
                            }
                        }
                    }
                    Type::Builtin(BuiltinId::Get) => self.check_get_builtin(expr, expected),
                    Type::Builtin(BuiltinId::Set) => {
                        if expr.args.len() != 3 {
                            self.diagnostics.push(Diagnostic::new(
                                "T004",
                                format!("expected 3 arguments but found {}", expr.args.len()),
                                expr.span,
                            ));
                            Type::Error
                        } else {
                            let expected = expected.map(|ty| self.resolve_type(&ty));
                            let expected_item = match expected.as_ref() {
                                Some(Type::List(item)) => Some(*item.clone()),
                                _ => None,
                            };
                            let list_ty = if let Some(expected_item) = expected_item {
                                self.check_expr_with_expected(
                                    &expr.args[0],
                                    Some(Type::List(Box::new(expected_item))),
                                )
                            } else {
                                self.check_expr(&expr.args[0])
                            };
                            self.check_expr_with_expected(&expr.args[1], Some(Type::Int));
                            match self.resolve_type(&list_ty) {
                                Type::List(item_ty) => {
                                    self.check_expr_with_expected(
                                        &expr.args[2],
                                        Some((*item_ty).clone()),
                                    );
                                    let list_ty = Type::List(item_ty);
                                    match expected {
                                        Some(Type::List(_)) | None => list_ty,
                                        Some(expected) => {
                                            self.apply_expected(list_ty, Some(expected), expr.span)
                                        }
                                    }
                                }
                                Type::Unknown(_) => {
                                    self.diagnostics.push(Diagnostic::new(
                                        "E005",
                                        "type annotation required because inference is not unique",
                                        expr.span,
                                    ));
                                    Type::Error
                                }
                                Type::Error => Type::Error,
                                _ => {
                                    self.diagnostics.push(Diagnostic::new(
                                        "T006",
                                        "`set` expects List[T] as its first argument",
                                        expr.span,
                                    ));
                                    Type::Error
                                }
                            }
                        }
                    }
                    Type::Builtin(BuiltinId::MapEmpty) => {
                        self.check_map_empty_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::Contains) => {
                        self.check_contains_builtin(expr, expected)
                    }
                    Type::Builtin(BuiltinId::Insert) => self.check_insert_builtin(expr, expected),
                    Type::Builtin(BuiltinId::Remove) => self.check_remove_builtin(expr, expected),
                    Type::Builtin(BuiltinId::OptionSome) => {
                        if expr.args.len() != 1 {
                            self.diagnostics.push(Diagnostic::new(
                                "T004",
                                format!("expected 1 arguments but found {}", expr.args.len()),
                                expr.span,
                            ));
                            Type::Error
                        } else {
                            let expected = expected.map(|ty| self.resolve_type(&ty));
                            let expected_item = match expected.as_ref() {
                                Some(Type::Option(item)) => Some(*item.clone()),
                                _ => None,
                            };
                            let item_ty = if let Some(expected_item) = expected_item {
                                self.check_expr_with_expected(
                                    &expr.args[0],
                                    Some(expected_item.clone()),
                                );
                                expected_item
                            } else {
                                self.check_expr(&expr.args[0])
                            };
                            let option_ty = Type::Option(Box::new(self.resolve_type(&item_ty)));
                            match expected {
                                Some(Type::Option(_)) | None => option_ty,
                                Some(expected) => {
                                    self.apply_expected(option_ty, Some(expected), expr.span)
                                }
                            }
                        }
                    }
                    Type::Builtin(BuiltinId::ResultOk) => self.check_result_constructor_builtin(
                        expr,
                        expected,
                        known_enum::RESULT_OK_NAME,
                    ),
                    Type::Builtin(BuiltinId::ResultErr) => self.check_result_constructor_builtin(
                        expr,
                        expected,
                        known_enum::RESULT_ERR_NAME,
                    ),
                    Type::EnumConstructor {
                        enum_name,
                        enum_item: _,
                        variant_name,
                    } => self.check_user_enum_constructor(
                        expr.span,
                        expected,
                        enum_name,
                        variant_name,
                        &expr.args,
                    ),
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
                        if let Err(message) = self.unify(callee_ty.clone(), inferred_sig) {
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
                };
                let resolved_callee = self.resolve_type(&callee_ty);
                self.calls.push(TypedCallInfo {
                    expr_id: expr.id,
                    span: expr.span,
                    callee: self.typed_callee_for(&expr.callee, &resolved_callee),
                });
                ty
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
            Expr::Match(expr) => self.check_match_expr(expr, expected),
            Expr::Fn(expr) => {
                let sig = self.signature_from_fn_expr(expr, expected.as_ref());
                self.push_scope(true);
                for (param, param_ty) in expr.params.iter().zip(sig.params.iter().cloned()) {
                    let name = self.symbol(&param.name);
                    self.insert_current(name, BindingKind::Parameter, param_ty, param.span);
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
        self.record_expr_type(expr.id(), span, ty)
    }

    fn record_expr_type(&mut self, expr_id: ExprId, span: Span, ty: Type) -> Type {
        let resolved = self.resolve_type(&ty);
        self.expr_types.push(ExprType {
            expr_id,
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
            if let Some(existing) = self.records.get(&name) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E002",
                        format!("duplicate record `{}` in the current scope", record.name),
                        record.span,
                    )
                    .with_related("previous record declaration is here", existing.span),
                );
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
            self.records.insert(
                name,
                RecordDef {
                    span: record.span,
                    fields,
                },
            );
        }
    }

    fn predeclare_enums(&mut self, statements: &[Stmt]) {
        for statement in statements {
            let Stmt::EnumDecl(enumeration) = statement else {
                continue;
            };
            let name = self.symbol(&enumeration.name);
            if let Some(existing) = self.enums.get(&name) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E002",
                        format!("duplicate enum `{}` in the current scope", enumeration.name),
                        enumeration.span,
                    )
                    .with_related("previous enum declaration is here", existing.span),
                );
                continue;
            }
            if let Some(existing) = self.records.get(&name) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E002",
                        format!("duplicate type `{}` in the current scope", enumeration.name),
                        enumeration.span,
                    )
                    .with_related("previous type declaration is here", existing.span),
                );
                continue;
            }

            let type_params = enumeration
                .type_params
                .iter()
                .map(|param| self.symbol(param))
                .collect::<Vec<_>>();
            let mut variants = Vec::new();
            for variant in &enumeration.variants {
                let variant_name = self.symbol(&variant.name);
                variants.push(EnumVariantDef {
                    name: variant_name,
                    payload: variant.payload.clone(),
                    span: variant.span,
                });
                let qualified = self.symbol(&format!("{}::{}", enumeration.name, variant.name));
                let kind = if variant.payload.is_some() {
                    BindingKind::Function
                } else {
                    BindingKind::Immutable
                };
                self.insert_current(
                    qualified,
                    kind,
                    Type::EnumConstructor {
                        enum_name: name,
                        enum_item: enumeration.package_item,
                        variant_name,
                    },
                    variant.span,
                );
            }
            self.enums.insert(
                name,
                EnumDef {
                    span: enumeration.span,
                    type_params,
                    variants,
                },
            );
        }
    }

    fn check_record_decl(&mut self, record: &RecordDecl) {
        let mut field_names = HashMap::new();
        for field in &record.fields {
            let field_name = self.symbol(&field.name);
            if let Some(previous_span) = field_names.insert(field_name, field.span) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E002",
                        format!(
                            "duplicate field `{}` in record `{}`",
                            field.name, record.name
                        ),
                        field.span,
                    )
                    .with_related("previous field declaration is here", previous_span),
                );
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

    fn check_enum_decl(&mut self, enumeration: &EnumDecl) {
        let mut type_params = HashSet::new();
        for param in &enumeration.type_params {
            let symbol = self.symbol(param);
            if !type_params.insert(symbol) {
                self.diagnostics.push(Diagnostic::new(
                    "E002",
                    format!(
                        "duplicate type parameter `{param}` in enum `{}`",
                        enumeration.name
                    ),
                    enumeration.span,
                ));
            }
            if matches!(param.as_str(), "Int" | "Bool" | "String") {
                self.diagnostics.push(Diagnostic::new(
                    "T022",
                    format!("type parameter `{param}` shadows a built-in type"),
                    enumeration.span,
                ));
            }
        }

        let params = type_params.into_iter().collect::<Vec<_>>();
        let mut variant_names = HashMap::new();
        for variant in &enumeration.variants {
            let variant_name = self.symbol(&variant.name);
            if let Some(previous_span) = variant_names.insert(variant_name, variant.span) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E002",
                        format!(
                            "duplicate variant `{}` in enum `{}`",
                            variant.name, enumeration.name
                        ),
                        variant.span,
                    )
                    .with_related("previous variant declaration is here", previous_span),
                );
            }
            if let Some(payload) = &variant.payload {
                let _ = self.type_from_expr_with_params(payload, variant.span, &params);
            }
        }
    }

    fn check_list_lit(&mut self, expr: &ListLitExpr, expected: Option<Type>) -> Type {
        let expected = expected.map(|ty| self.resolve_type(&ty));
        let expected_item = match expected.as_ref() {
            Some(Type::List(item)) => Some(*item.clone()),
            _ => None,
        };

        if expr.items.is_empty() {
            return match expected {
                Some(Type::List(item)) => Type::List(item),
                Some(Type::Error) => Type::Error,
                Some(_) | None => {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "T015",
                            "empty list literal requires an expected List[T] type",
                            expr.span,
                        )
                        .with_suggestion(
                            "add a local binding annotation such as `items: List[Int] = []`",
                        ),
                    );
                    Type::Error
                }
            };
        }

        let item_ty = if let Some(expected_item) = expected_item {
            for item in &expr.items {
                self.check_expr_with_expected(item, Some(expected_item.clone()));
            }
            expected_item
        } else {
            let first_ty = self.check_expr(&expr.items[0]);
            for item in expr.items.iter().skip(1) {
                self.check_expr_with_expected(item, Some(first_ty.clone()));
            }
            first_ty
        };
        let list_ty = Type::List(Box::new(self.resolve_type(&item_ty)));
        match expected {
            Some(Type::List(_)) | None => list_ty,
            Some(expected) => self.apply_expected(list_ty, Some(expected), expr.span),
        }
    }

    fn check_index_expr(&mut self, expr: &IndexExpr, expected: Option<Type>) -> Type {
        let expected = expected.map(|ty| self.resolve_type(&ty));
        let base_expected = expected
            .as_ref()
            .map(|item| Type::List(Box::new(item.clone())));
        let base_ty = self.check_expr_with_expected(&expr.base, base_expected);
        self.check_expr_with_expected(&expr.index, Some(Type::Int));
        match self.resolve_type(&base_ty) {
            Type::List(item_ty) => self.apply_expected(*item_ty, expected, expr.span),
            Type::Unknown(_) => {
                self.diagnostics.push(Diagnostic::new(
                    "E005",
                    "type annotation required because inference is not unique",
                    expr.span,
                ));
                Type::Error
            }
            Type::Error => Type::Error,
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    "list indexing expects List[T] as its base",
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_map_empty_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if !expr.args.is_empty() {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 0 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let expected = expected.map(|ty| self.resolve_type(&ty));
        match expected {
            Some(Type::Map(key, value)) => Type::Map(key, value),
            Some(Type::Error) => Type::Error,
            Some(_) | None => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "T019",
                        "`Map.empty()` requires an expected Map[K, V] type",
                        expr.span,
                    )
                    .with_suggestion(
                        "add a local binding annotation such as `items: Map[String, Int] = Map.empty()`",
                    ),
                );
                Type::Error
            }
        }
    }

    fn check_get_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let expected = expected.map(|ty| self.resolve_type(&ty));
        let expected_item = match expected.as_ref() {
            Some(Type::Option(item)) => Some(*item.clone()),
            _ => None,
        };

        if Self::is_map_empty_call(&expr.args[0]) {
            let key_ty = self.check_expr(&expr.args[1]);
            let key_ty = self.resolve_type(&key_ty);
            if !self.validate_map_key_type(&key_ty, expr.args[1].span()) {
                return Type::Error;
            }
            let value_ty = expected_item
                .clone()
                .unwrap_or_else(|| Type::Unknown(self.fresh_unknown()));
            let base_expected = Type::Map(Box::new(key_ty), Box::new(value_ty));
            let base_ty = self.check_expr_with_expected(&expr.args[0], Some(base_expected));
            return match self.resolve_type(&base_ty) {
                Type::Map(_, value_ty) => {
                    self.apply_expected_option(Type::Option(value_ty), expected, expr.span)
                }
                Type::Error => Type::Error,
                _ => {
                    self.diagnostics.push(Diagnostic::new(
                        "T006",
                        "`get` expects List[T] or Map[K, V] as its first argument",
                        expr.span,
                    ));
                    Type::Error
                }
            };
        }

        let base_expected = match expected_item {
            Some(item) if Self::is_empty_list_literal(&expr.args[0]) => {
                Some(Type::List(Box::new(item)))
            }
            _ => None,
        };
        let base_ty = self.check_expr_with_expected(&expr.args[0], base_expected);
        match self.resolve_type(&base_ty) {
            Type::List(item_ty) => {
                self.check_expr_with_expected(&expr.args[1], Some(Type::Int));
                self.apply_expected_option(Type::Option(item_ty), expected, expr.span)
            }
            Type::Map(key_ty, value_ty) => {
                if !self.validate_map_key_type(&key_ty, expr.args[0].span()) {
                    return Type::Error;
                }
                self.check_expr_with_expected(&expr.args[1], Some((*key_ty).clone()));
                self.apply_expected_option(Type::Option(value_ty), expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(Diagnostic::new(
                    "E005",
                    "type annotation required because inference is not unique",
                    expr.span,
                ));
                Type::Error
            }
            Type::Error => Type::Error,
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    "`get` expects List[T] or Map[K, V] as its first argument",
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_contains_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        if Self::is_map_empty_call(&expr.args[0]) {
            self.diagnostics.push(
                Diagnostic::new(
                    "T019",
                    "`Map.empty().contains(...)` requires an expected Map[K, V] type",
                    expr.span,
                )
                .with_suggestion(
                    "add a local binding annotation such as `items: Map[String, Int] = Map.empty()`",
                ),
            );
            return Type::Error;
        }

        let base_ty = self.check_expr(&expr.args[0]);
        match self.resolve_type(&base_ty) {
            Type::Map(key_ty, _) => {
                if !self.validate_map_key_type(&key_ty, expr.args[0].span()) {
                    return Type::Error;
                }
                self.check_expr_with_expected(&expr.args[1], Some((*key_ty).clone()));
                self.apply_expected(Type::Bool, expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(Diagnostic::new(
                    "E005",
                    "type annotation required because inference is not unique",
                    expr.span,
                ));
                Type::Error
            }
            Type::Error => Type::Error,
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    "`contains` expects Map[K, V] as its first argument",
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_insert_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 3 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 3 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let expected = expected.map(|ty| self.resolve_type(&ty));
        let expected_map = match expected.as_ref() {
            Some(Type::Map(key, value)) => Some((*key.clone(), *value.clone())),
            _ => None,
        };

        if Self::is_map_empty_call(&expr.args[0]) {
            let key_ty = if let Some((key_ty, _)) = expected_map.as_ref() {
                self.check_expr_with_expected(&expr.args[1], Some(key_ty.clone()))
            } else {
                self.check_expr(&expr.args[1])
            };
            let key_ty = self.resolve_type(&key_ty);
            if !self.validate_map_key_type(&key_ty, expr.args[1].span()) {
                return Type::Error;
            }

            let value_ty = if let Some((_, value_ty)) = expected_map.as_ref() {
                self.check_expr_with_expected(&expr.args[2], Some(value_ty.clone()))
            } else {
                self.check_expr(&expr.args[2])
            };
            let value_ty = self.resolve_type(&value_ty);
            let map_ty = Type::Map(Box::new(key_ty), Box::new(value_ty));
            let base_ty = self.check_expr_with_expected(&expr.args[0], Some(map_ty.clone()));
            if matches!(self.resolve_type(&base_ty), Type::Error) {
                return Type::Error;
            }
            return self.apply_expected_map(map_ty, expected, expr.span);
        }

        let base_expected = match expected.as_ref() {
            Some(Type::Map(_, _)) => expected.clone(),
            _ => None,
        };
        let base_ty = self.check_expr_with_expected(&expr.args[0], base_expected);
        match self.resolve_type(&base_ty) {
            Type::Map(key_ty, value_ty) => {
                if !self.validate_map_key_type(&key_ty, expr.args[0].span()) {
                    return Type::Error;
                }
                self.check_expr_with_expected(&expr.args[1], Some((*key_ty).clone()));
                self.check_expr_with_expected(&expr.args[2], Some((*value_ty).clone()));
                self.apply_expected_map(Type::Map(key_ty, value_ty), expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(Diagnostic::new(
                    "E005",
                    "type annotation required because inference is not unique",
                    expr.span,
                ));
                Type::Error
            }
            Type::Error => Type::Error,
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    "`insert` expects Map[K, V] as its first argument",
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn check_remove_builtin(&mut self, expr: &CallExpr, expected: Option<Type>) -> Type {
        if expr.args.len() != 2 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 2 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let expected = expected.map(|ty| self.resolve_type(&ty));
        if Self::is_map_empty_call(&expr.args[0]) {
            return match expected.clone() {
                Some(Type::Map(key_ty, value_ty)) => {
                    self.check_expr_with_expected(&expr.args[1], Some((*key_ty).clone()));
                    if !self.validate_map_key_type(&key_ty, expr.args[1].span()) {
                        return Type::Error;
                    }
                    let map_ty = Type::Map(key_ty, value_ty);
                    let base_ty =
                        self.check_expr_with_expected(&expr.args[0], Some(map_ty.clone()));
                    if matches!(self.resolve_type(&base_ty), Type::Error) {
                        Type::Error
                    } else {
                        self.apply_expected_map(map_ty, expected, expr.span)
                    }
                }
                Some(Type::Error) => Type::Error,
                Some(_) | None => {
                    self.diagnostics.push(
                        Diagnostic::new(
                            "T019",
                            "`Map.empty().remove(...)` requires an expected Map[K, V] type",
                            expr.span,
                        )
                        .with_suggestion(
                            "add a local binding annotation such as `items: Map[String, Int] = Map.empty().remove(\"missing\")`",
                        ),
                    );
                    Type::Error
                }
            };
        }

        let base_expected = match expected.as_ref() {
            Some(Type::Map(_, _)) => expected.clone(),
            _ => None,
        };
        let base_ty = self.check_expr_with_expected(&expr.args[0], base_expected);
        match self.resolve_type(&base_ty) {
            Type::Map(key_ty, value_ty) => {
                if !self.validate_map_key_type(&key_ty, expr.args[0].span()) {
                    return Type::Error;
                }
                self.check_expr_with_expected(&expr.args[1], Some((*key_ty).clone()));
                self.apply_expected_map(Type::Map(key_ty, value_ty), expected, expr.span)
            }
            Type::Unknown(_) => {
                self.diagnostics.push(Diagnostic::new(
                    "E005",
                    "type annotation required because inference is not unique",
                    expr.span,
                ));
                Type::Error
            }
            Type::Error => Type::Error,
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "T006",
                    "`remove` expects Map[K, V] as its first argument",
                    expr.span,
                ));
                Type::Error
            }
        }
    }

    fn apply_expected_option(
        &mut self,
        option_ty: Type,
        expected: Option<Type>,
        span: Span,
    ) -> Type {
        match expected {
            None => option_ty,
            Some(expected) => self.apply_expected(option_ty, Some(expected), span),
        }
    }

    fn apply_expected_map(&mut self, map_ty: Type, expected: Option<Type>, span: Span) -> Type {
        match expected {
            None => map_ty,
            Some(expected) => self.apply_expected(map_ty, Some(expected), span),
        }
    }

    fn validate_map_key_type(&mut self, key_ty: &Type, span: Span) -> bool {
        match self.resolve_type(key_ty) {
            Type::Int | Type::Bool | Type::String | Type::Unknown(_) | Type::Error => true,
            _ => {
                self.diagnostics.push(
                    Diagnostic::new("T020", "Map key type must be Int, Bool, or String", span)
                        .with_suggestion("use Int, Bool, or String as the Map key type"),
                );
                false
            }
        }
    }

    fn is_map_empty_call(expr: &Expr) -> bool {
        matches!(
            expr,
            Expr::Call(call)
                if matches!(
                    call.callee.as_ref(),
                    Expr::Ident(IdentExpr { name, .. }) if name == "Map.empty"
                )
        )
    }

    fn is_empty_list_literal(expr: &Expr) -> bool {
        matches!(expr, Expr::ListLit(list) if list.items.is_empty())
    }

    fn check_option_none(&mut self, expected: Option<Type>, span: Span) -> Type {
        let expected = expected.map(|ty| self.resolve_type(&ty));
        match expected {
            Some(Type::Option(item)) => Type::Option(item),
            Some(Type::Error) => Type::Error,
            Some(_) | None => {
                self.diagnostics.push(
                    Diagnostic::new(
                        "T017",
                        "`Option::None` requires an expected Option[T] type",
                        span,
                    )
                    .with_suggestion(
                        "add a local binding annotation such as `value: Option[Int] = Option::None`",
                    ),
                );
                Type::Error
            }
        }
    }

    fn check_result_constructor_builtin(
        &mut self,
        expr: &CallExpr,
        expected: Option<Type>,
        variant_name: &'static str,
    ) -> Type {
        if expr.args.len() != 1 {
            self.diagnostics.push(Diagnostic::new(
                "T004",
                format!("expected 1 arguments but found {}", expr.args.len()),
                expr.span,
            ));
            return Type::Error;
        }

        let expected = expected.map(|ty| self.resolve_type(&ty));
        let (ok_ty, err_ty) = match expected {
            Some(Type::Result(ok_ty, err_ty)) => (ok_ty, err_ty),
            Some(Type::Error) => {
                self.check_expr(&expr.args[0]);
                return Type::Error;
            }
            Some(_) | None => {
                self.check_expr(&expr.args[0]);
                self.diagnostics.push(
                    Diagnostic::new(
                        "T021",
                        format!(
                            "`Result::{variant_name}` requires an expected Result[T, E] type"
                        ),
                        expr.span,
                    )
                    .with_suggestion(
                        "add a local binding annotation such as `value: Result[Int, String] = Result::Ok(1)`",
                    ),
                );
                return Type::Error;
            }
        };

        let payload_ty = if variant_name == known_enum::RESULT_OK_NAME {
            (*ok_ty).clone()
        } else {
            (*err_ty).clone()
        };
        self.check_expr_with_expected(&expr.args[0], Some(payload_ty));
        Type::Result(ok_ty, err_ty)
    }

    fn check_user_enum_constructor(
        &mut self,
        span: Span,
        expected: Option<Type>,
        enum_name: Symbol,
        variant_name: Symbol,
        args: &[Expr],
    ) -> Type {
        let Some(enumeration) = self.enums.get(&enum_name).cloned() else {
            for arg in args {
                self.check_expr(arg);
            }
            return Type::Error;
        };
        let Some(variant) = enumeration
            .variants
            .iter()
            .find(|variant| variant.name == variant_name)
            .cloned()
        else {
            for arg in args {
                self.check_expr(arg);
            }
            return Type::Error;
        };

        let expected = expected.map(|ty| self.resolve_type(&ty));
        let enum_args = match expected {
            Some(Type::Enum(expected_enum, args)) if expected_enum == enum_name => args,
            Some(Type::Error) => {
                for arg in args {
                    self.check_expr(arg);
                }
                return Type::Error;
            }
            Some(_) | None if !enumeration.type_params.is_empty() => {
                for arg in args {
                    self.check_expr(arg);
                }
                let enum_text = self.symbols.resolve(enum_name).to_string();
                let variant_text = self.symbols.resolve(variant_name).to_string();
                self.diagnostics.push(
                    Diagnostic::new(
                        "T022",
                        format!(
                            "`{enum_text}::{variant_text}` requires an expected {enum_text}[...] type"
                        ),
                        span,
                    )
                    .with_suggestion(format!(
                        "add a local binding annotation such as `value: {enum_text}[Int] = {enum_text}::{variant_text}`"
                    )),
                );
                return Type::Error;
            }
            Some(_) | None => Vec::new(),
        };

        match (&variant.payload, args) {
            (None, []) => {}
            (None, _) => {
                self.diagnostics.push(Diagnostic::new(
                    "T004",
                    format!("expected 0 arguments but found {}", args.len()),
                    span,
                ));
                for arg in args {
                    self.check_expr(arg);
                }
                return Type::Error;
            }
            (Some(_), []) => {
                self.diagnostics.push(Diagnostic::new(
                    "T004",
                    "expected 1 arguments but found 0",
                    span,
                ));
                return Type::Error;
            }
            (Some(_), [_]) => {}
            (Some(_), _) => {
                self.diagnostics.push(Diagnostic::new(
                    "T004",
                    format!("expected 1 arguments but found {}", args.len()),
                    span,
                ));
                for arg in args {
                    self.check_expr(arg);
                }
                return Type::Error;
            }
        }

        let enum_ty = Type::Enum(enum_name, enum_args.clone());
        if let (Some(payload_expr), Some(payload_type)) = (args.first(), variant.payload.as_ref()) {
            let payload_ty = self.type_from_expr_with_params(
                payload_type,
                variant.span,
                &enumeration.type_params,
            );
            let payload_ty =
                self.substitute_type_params(payload_ty, &enumeration.type_params, &enum_args);
            self.check_expr_with_expected(payload_expr, Some(payload_ty));
        }
        enum_ty
    }

    fn diagnose_unknown_enum_variant(&mut self, enum_name: &str, variant_name: &str, span: Span) {
        let enum_symbol = self.symbol(enum_name);
        let variant_symbol = self.symbol(variant_name);
        if let Some(enumeration) = self.enums.get(&enum_symbol) {
            if enumeration
                .variants
                .iter()
                .any(|variant| variant.name == variant_symbol)
            {
                return;
            }
            self.diagnostics.push(
                Diagnostic::new(
                    "T022",
                    format!("unknown variant `{variant_name}` for enum `{enum_name}`"),
                    span,
                )
                .with_related("enum is declared here", enumeration.span),
            );
            return;
        }

        self.diagnostics.push(Diagnostic::new(
            "T022",
            format!("unknown enum `{enum_name}` in variant constructor"),
            span,
        ));
    }

    fn check_match_expr(&mut self, expr: &MatchExpr, expected: Option<Type>) -> Type {
        let value_ty = self.check_expr(&expr.value);
        let spec = self.enum_match_spec_for_value(&value_ty, expr.value.span());
        let mut seen_variants = HashMap::new();
        let mut result_ty = None;
        let expected = expected.map(|ty| self.resolve_type(&ty));

        for arm in &expr.arms {
            self.push_scope(false);
            let MatchPattern::Variant(pattern) = &arm.pattern;
            let pattern_enum = self.symbol(&pattern.enum_name);
            let pattern_variant = self.symbol(&pattern.variant_name);
            if pattern_enum != spec.enum_name {
                self.diagnostics.push(Diagnostic::new(
                    "T018",
                    format!(
                        "`{}::{}` does not belong to {}",
                        pattern.enum_name, pattern.variant_name, spec.display_name
                    ),
                    pattern.span,
                ));
            } else if let Some(variant) = spec.variant(pattern_variant) {
                if let Some(previous) = seen_variants.insert(variant.name, pattern.span) {
                    let qualified = self.qualified_variant(spec.enum_name, variant.name);
                    self.diagnostics.push(
                        Diagnostic::new(
                            "T018",
                            format!("duplicate `{qualified}` match arm"),
                            pattern.span,
                        )
                        .with_related(format!("previous `{qualified}` arm is here"), previous),
                    );
                }
                match (&variant.payload, &pattern.binding) {
                    (Some(payload_ty), Some(binding)) => {
                        let name = self.symbol(binding);
                        self.insert_current(
                            name,
                            BindingKind::Immutable,
                            payload_ty.clone(),
                            pattern.span,
                        );
                    }
                    (Some(_), None) => {
                        let qualified = self.qualified_variant(spec.enum_name, variant.name);
                        self.diagnostics.push(Diagnostic::new(
                            "T018",
                            format!("`{qualified}` match arm must bind its payload"),
                            pattern.span,
                        ));
                    }
                    (None, Some(_)) => {
                        let qualified = self.qualified_variant(spec.enum_name, variant.name);
                        self.diagnostics.push(Diagnostic::new(
                            "T018",
                            format!("`{qualified}` match arm does not carry a payload"),
                            pattern.span,
                        ));
                    }
                    (None, None) => {}
                }
            } else {
                self.diagnostics.push(Diagnostic::new(
                    "T018",
                    format!(
                        "unknown variant `{}` in match for {}",
                        pattern.variant_name, spec.display_name
                    ),
                    pattern.span,
                ));
            }

            let arm_ty = if let Some(expected_ty) = expected.clone() {
                self.check_expr_with_expected(&arm.value, Some(expected_ty))
            } else if let Some(result_ty) = result_ty.clone() {
                self.check_expr_with_expected(&arm.value, Some(result_ty))
            } else {
                self.check_expr(&arm.value)
            };
            if result_ty.is_none() {
                result_ty = Some(arm_ty);
            }
            self.pop_scope();
        }

        for variant in &spec.variants {
            if !seen_variants.contains_key(&variant.name) {
                let qualified = self.qualified_variant(spec.enum_name, variant.name);
                self.diagnostics.push(Diagnostic::new(
                    "T018",
                    format!(
                        "`match` on {} requires an `{qualified}` arm",
                        spec.display_name
                    ),
                    expr.span,
                ));
            }
        }

        match expected {
            Some(expected) => self.resolve_type(&expected),
            None => result_ty.unwrap_or(Type::Error),
        }
    }

    fn enum_match_spec_for_value(&mut self, value_ty: &Type, value_span: Span) -> EnumMatchSpec {
        match self.resolve_type(value_ty) {
            Type::Option(item) => self.option_match_spec(*item),
            Type::Result(ok, err) => self.result_match_spec(*ok, *err),
            Type::Enum(enum_name, args) => self.user_enum_match_spec(enum_name, args),
            Type::Unknown(_) => {
                let item = Type::Unknown(self.fresh_unknown());
                let option = Type::Option(Box::new(item.clone()));
                if let Err(message) = self.unify(value_ty.clone(), option) {
                    self.diagnostics
                        .push(Diagnostic::new("T018", message, value_span));
                    self.option_match_spec(Type::Error)
                } else {
                    self.option_match_spec(item)
                }
            }
            Type::Error => self.option_match_spec(Type::Error),
            _ => {
                self.diagnostics.push(Diagnostic::new(
                    "T018",
                    "`match` requires an enum value",
                    value_span,
                ));
                self.option_match_spec(Type::Error)
            }
        }
    }

    fn option_match_spec(&mut self, item_ty: Type) -> EnumMatchSpec {
        let known = known_enum::option_enum();
        let enum_name = self.symbol(known.name);
        EnumMatchSpec {
            enum_name,
            display_name: "Option[T]".to_string(),
            variants: known
                .variants
                .iter()
                .copied()
                .map(|variant| EnumMatchVariant {
                    name: self.symbol(variant.name),
                    payload: if variant.has_payload {
                        Some(item_ty.clone())
                    } else {
                        None
                    },
                })
                .collect(),
        }
    }

    fn result_match_spec(&mut self, ok_ty: Type, err_ty: Type) -> EnumMatchSpec {
        let known = known_enum::result_enum();
        let enum_name = self.symbol(known.name);
        EnumMatchSpec {
            enum_name,
            display_name: "Result[T, E]".to_string(),
            variants: known
                .variants
                .iter()
                .copied()
                .map(|variant| {
                    let payload = match variant.name {
                        known_enum::RESULT_OK_NAME => Some(ok_ty.clone()),
                        known_enum::RESULT_ERR_NAME => Some(err_ty.clone()),
                        _ => None,
                    };
                    EnumMatchVariant {
                        name: self.symbol(variant.name),
                        payload,
                    }
                })
                .collect(),
        }
    }

    fn user_enum_match_spec(&mut self, enum_name: Symbol, args: Vec<Type>) -> EnumMatchSpec {
        let Some(enumeration) = self.enums.get(&enum_name).cloned() else {
            return EnumMatchSpec {
                enum_name,
                display_name: self.symbols.resolve(enum_name).to_string(),
                variants: Vec::new(),
            };
        };
        let variants = enumeration
            .variants
            .iter()
            .map(|variant| {
                let payload = variant.payload.as_ref().map(|payload| {
                    let payload_ty = self.type_from_expr_with_params(
                        payload,
                        variant.span,
                        &enumeration.type_params,
                    );
                    self.substitute_type_params(payload_ty, &enumeration.type_params, &args)
                });
                EnumMatchVariant {
                    name: variant.name,
                    payload,
                }
            })
            .collect();
        EnumMatchSpec {
            enum_name,
            display_name: self.symbols.resolve(enum_name).to_string(),
            variants,
        }
    }

    fn qualified_variant(&self, enum_name: Symbol, variant_name: Symbol) -> String {
        format!(
            "{}::{}",
            self.symbols.resolve(enum_name),
            self.symbols.resolve(variant_name)
        )
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
                self.diagnostics.push(
                    Diagnostic::new(
                        "E009",
                        format!(
                            "invalid record literal for `{}`: duplicate field `{}`",
                            expr.type_name, field.name
                        ),
                        field.span,
                    )
                    .with_related("record is declared here", record.span),
                );
                has_error = true;
                continue;
            }

            let Some(declared) = find_record_field(&record, field_name) else {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E009",
                        format!(
                            "invalid record literal for `{}`: unknown field `{}`",
                            expr.type_name, field.name
                        ),
                        field.span,
                    )
                    .with_related("record is declared here", record.span),
                );
                has_error = true;
                continue;
            };

            let field_ty = self.type_from_expr(&declared.type_name, declared.span);
            if let Err(message) = self.unify(field_ty, value_ty) {
                self.diagnostics.push(
                    Diagnostic::new("E009", message, field.span)
                        .with_related("field type is declared here", declared.span),
                );
                has_error = true;
            }
        }

        for declared in &record.fields {
            if !seen.contains(&declared.name) {
                self.diagnostics.push(
                    Diagnostic::new(
                        "E009",
                        format!(
                            "invalid record literal for `{}`: missing field `{}`",
                            expr.type_name,
                            self.symbols.resolve(declared.name)
                        ),
                        expr.span,
                    )
                    .with_related("required field is declared here", declared.span),
                );
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
            self.diagnostics.push(
                Diagnostic::new("E008", format!("unknown field `{}`", expr.field), expr.span)
                    .with_related("record is declared here", record.span),
            );
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
                self.diagnostics.push(
                    Diagnostic::new("E012", "invalid record update", field.span)
                        .with_related("record is declared here", record.span),
                );
                has_error = true;
                continue;
            }

            let Some(declared) = find_record_field(&record, field_name) else {
                self.diagnostics.push(
                    Diagnostic::new("E012", "invalid record update", field.span)
                        .with_related("record is declared here", record.span),
                );
                has_error = true;
                continue;
            };

            let field_ty = self.type_from_expr(&declared.type_name, declared.span);
            if let Err(message) = self.unify(field_ty, value_ty) {
                self.diagnostics.push(
                    Diagnostic::new("E012", message, field.span)
                        .with_related("field type is declared here", declared.span),
                );
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
                self.insert_current(name, BindingKind::Function, Type::Function(sig), func.span);
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
        self.type_from_expr_with_params(type_expr, span, &[])
    }

    fn type_from_expr_with_params(
        &mut self,
        type_expr: &TypeExpr,
        span: crate::span::Span,
        type_params: &[Symbol],
    ) -> Type {
        match type_expr {
            TypeExpr::Int => Type::Int,
            TypeExpr::Bool => Type::Bool,
            TypeExpr::String => Type::String,
            TypeExpr::Named(name) => {
                let symbol = self.symbol(name);
                if type_params.contains(&symbol) {
                    Type::GenericParam(symbol)
                } else if self.records.contains_key(&symbol) {
                    Type::Record(symbol)
                } else if self
                    .enums
                    .get(&symbol)
                    .is_some_and(|enumeration| enumeration.type_params.is_empty())
                {
                    Type::Enum(symbol, Vec::new())
                } else {
                    self.diagnostics.push(Diagnostic::new(
                        "T007",
                        format!("unknown type `{name}`"),
                        span,
                    ));
                    Type::Error
                }
            }
            TypeExpr::Generic(generic) if generic.name == "List" => {
                if generic.args.len() != 1 {
                    self.diagnostics.push(Diagnostic::new(
                        "T016",
                        "List expects exactly 1 type argument",
                        span,
                    ));
                    return Type::Error;
                }
                Type::List(Box::new(self.type_from_expr_with_params(
                    &generic.args[0],
                    span,
                    type_params,
                )))
            }
            TypeExpr::Generic(generic) if generic.name == known_enum::OPTION_NAME => {
                if generic.args.len() != 1 {
                    self.diagnostics.push(Diagnostic::new(
                        "T017",
                        "Option expects exactly 1 type argument",
                        span,
                    ));
                    return Type::Error;
                }
                Type::Option(Box::new(self.type_from_expr_with_params(
                    &generic.args[0],
                    span,
                    type_params,
                )))
            }
            TypeExpr::Generic(generic) if generic.name == known_enum::RESULT_NAME => {
                if generic.args.len() != 2 {
                    self.diagnostics.push(Diagnostic::new(
                        "T021",
                        "Result expects exactly 2 type arguments",
                        span,
                    ));
                    return Type::Error;
                }
                Type::Result(
                    Box::new(self.type_from_expr_with_params(&generic.args[0], span, type_params)),
                    Box::new(self.type_from_expr_with_params(&generic.args[1], span, type_params)),
                )
            }
            TypeExpr::Generic(generic) if generic.name == "Map" => {
                if generic.args.len() != 2 {
                    self.diagnostics.push(Diagnostic::new(
                        "T019",
                        "Map expects exactly 2 type arguments",
                        span,
                    ));
                    return Type::Error;
                }
                let key = self.type_from_expr_with_params(&generic.args[0], span, type_params);
                self.validate_map_key_type(&key, span);
                let value = self.type_from_expr_with_params(&generic.args[1], span, type_params);
                Type::Map(Box::new(key), Box::new(value))
            }
            TypeExpr::Generic(generic) => {
                let symbol = self.symbol(&generic.name);
                if let Some(enumeration) = self.enums.get(&symbol).cloned() {
                    if generic.args.len() != enumeration.type_params.len() {
                        self.diagnostics.push(Diagnostic::new(
                            "T022",
                            format!(
                                "enum `{}` expects exactly {} type arguments",
                                generic.name,
                                enumeration.type_params.len()
                            ),
                            span,
                        ));
                        return Type::Error;
                    }
                    return Type::Enum(
                        symbol,
                        generic
                            .args
                            .iter()
                            .map(|arg| self.type_from_expr_with_params(arg, span, type_params))
                            .collect(),
                    );
                }
                for arg in &generic.args {
                    let _ = self.type_from_expr_with_params(arg, span, type_params);
                }
                self.diagnostics.push(
                    Diagnostic::new(
                        "T013",
                        format!("generic type `{}` is not implemented yet", generic.name),
                        span,
                    )
                    .with_suggestion(
                        "generic type syntax is reserved for upcoming collection types",
                    ),
                );
                Type::Error
            }
            TypeExpr::Function(function) => Type::Function(FunctionSig {
                params: function
                    .params
                    .iter()
                    .map(|param| self.type_from_expr_with_params(param, span, type_params))
                    .collect(),
                ret: Box::new(self.type_from_expr_with_params(&function.ret, span, type_params)),
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
            (Type::Unknown(left), Type::Unknown(right)) if left == right => Ok(Type::Unknown(left)),
            (Type::Unknown(id), ty) | (ty, Type::Unknown(id)) => {
                if self.type_contains_unknown(&ty, id) {
                    return Err("type inference would require an infinite type".to_string());
                }
                self.substitutions.insert(id, ty.clone());
                Ok(ty)
            }
            (Type::Int, Type::Int) => Ok(Type::Int),
            (Type::Bool, Type::Bool) => Ok(Type::Bool),
            (Type::String, Type::String) => Ok(Type::String),
            (Type::Record(left), Type::Record(right)) if left == right => Ok(Type::Record(left)),
            (Type::Enum(left_name, left_args), Type::Enum(right_name, right_args))
                if left_name == right_name && left_args.len() == right_args.len() =>
            {
                let args = left_args
                    .into_iter()
                    .zip(right_args.into_iter())
                    .map(|(left, right)| self.unify(left, right))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Type::Enum(left_name, args))
            }
            (Type::GenericParam(left), Type::GenericParam(right)) if left == right => {
                Ok(Type::GenericParam(left))
            }
            (Type::List(left), Type::List(right)) => {
                let item = self.unify(*left, *right)?;
                Ok(Type::List(Box::new(item)))
            }
            (Type::Map(left_key, left_value), Type::Map(right_key, right_value)) => {
                let key = self.unify(*left_key, *right_key)?;
                let value = self.unify(*left_value, *right_value)?;
                Ok(Type::Map(Box::new(key), Box::new(value)))
            }
            (Type::Option(left), Type::Option(right)) => {
                let item = self.unify(*left, *right)?;
                Ok(Type::Option(Box::new(item)))
            }
            (Type::Result(left_ok, left_err), Type::Result(right_ok, right_err)) => {
                let ok = self.unify(*left_ok, *right_ok)?;
                let err = self.unify(*left_err, *right_err)?;
                Ok(Type::Result(Box::new(ok), Box::new(err)))
            }
            (Type::Function(left), Type::Function(right)) => {
                if left.params.len() != right.params.len() {
                    return Err("function arity mismatch".to_string());
                }
                let mut params = Vec::with_capacity(left.params.len());
                for (left_param, right_param) in left.params.iter().zip(right.params.iter()) {
                    params.push(self.unify(left_param.clone(), right_param.clone())?);
                }
                let ret = self.unify(*left.ret.clone(), *right.ret.clone())?;
                Ok(Type::Function(FunctionSig {
                    params,
                    ret: Box::new(ret),
                }))
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
            Type::Enum(name, args) => Type::Enum(
                *name,
                args.iter().map(|arg| self.resolve_type(arg)).collect(),
            ),
            Type::List(item) => Type::List(Box::new(self.resolve_type(item))),
            Type::Map(key, value) => Type::Map(
                Box::new(self.resolve_type(key)),
                Box::new(self.resolve_type(value)),
            ),
            Type::Option(item) => Type::Option(Box::new(self.resolve_type(item))),
            Type::Result(ok, err) => Type::Result(
                Box::new(self.resolve_type(ok)),
                Box::new(self.resolve_type(err)),
            ),
            Type::Builtin(builtin) => Type::Builtin(*builtin),
            Type::EnumConstructor {
                enum_name,
                enum_item,
                variant_name,
            } => Type::EnumConstructor {
                enum_name: *enum_name,
                enum_item: *enum_item,
                variant_name: *variant_name,
            },
            other => other.clone(),
        }
    }

    fn type_info_for(&self, ty: &Type) -> TypeInfo {
        match self.resolve_type(ty) {
            Type::Int => TypeInfo::Int,
            Type::Bool => TypeInfo::Bool,
            Type::String => TypeInfo::String,
            Type::Record(symbol) => TypeInfo::Record(symbol),
            Type::Enum(symbol, args) => TypeInfo::Enum {
                symbol,
                args: args.iter().map(|arg| self.type_info_for(arg)).collect(),
            },
            Type::GenericParam(symbol) => TypeInfo::GenericParam(symbol),
            Type::List(item) => TypeInfo::List(Box::new(self.type_info_for(&item))),
            Type::Map(key, value) => TypeInfo::Map(
                Box::new(self.type_info_for(&key)),
                Box::new(self.type_info_for(&value)),
            ),
            Type::Option(item) => TypeInfo::Option(Box::new(self.type_info_for(&item))),
            Type::Result(ok, err) => TypeInfo::Result(
                Box::new(self.type_info_for(&ok)),
                Box::new(self.type_info_for(&err)),
            ),
            Type::Function(sig) => TypeInfo::Function(FunctionTypeInfo {
                params: sig.params.iter().map(|ty| self.type_info_for(ty)).collect(),
                ret: Box::new(self.type_info_for(&sig.ret)),
            }),
            Type::Builtin(builtin) => TypeInfo::Builtin(builtin),
            Type::OptionNone => TypeInfo::Builtin(BuiltinId::OptionNone),
            Type::EnumConstructor {
                enum_name,
                enum_item,
                variant_name,
            } => TypeInfo::EnumConstructor {
                enum_symbol: enum_name,
                enum_item,
                variant: variant_name,
            },
            Type::Unknown(_) => TypeInfo::Unknown,
            Type::Error => TypeInfo::Error,
        }
    }

    fn type_contains_unknown(&self, ty: &Type, needle: u32) -> bool {
        match self.resolve_type(ty) {
            Type::Unknown(id) => id == needle,
            Type::Function(sig) => {
                sig.params
                    .iter()
                    .any(|param| self.type_contains_unknown(param, needle))
                    || self.type_contains_unknown(&sig.ret, needle)
            }
            Type::Enum(_, args) => args
                .iter()
                .any(|arg| self.type_contains_unknown(arg, needle)),
            Type::List(item) => self.type_contains_unknown(&item, needle),
            Type::Map(key, value) => {
                self.type_contains_unknown(&key, needle)
                    || self.type_contains_unknown(&value, needle)
            }
            Type::Option(item) => self.type_contains_unknown(&item, needle),
            Type::Result(ok, err) => {
                self.type_contains_unknown(&ok, needle) || self.type_contains_unknown(&err, needle)
            }
            _ => false,
        }
    }

    fn substitute_type_params(&self, ty: Type, params: &[Symbol], args: &[Type]) -> Type {
        match ty {
            Type::GenericParam(param) => params
                .iter()
                .position(|candidate| *candidate == param)
                .and_then(|index| args.get(index).cloned())
                .unwrap_or(Type::GenericParam(param)),
            Type::Function(sig) => Type::Function(FunctionSig {
                params: sig
                    .params
                    .into_iter()
                    .map(|param| self.substitute_type_params(param, params, args))
                    .collect(),
                ret: Box::new(self.substitute_type_params(*sig.ret, params, args)),
            }),
            Type::Enum(name, enum_args) => Type::Enum(
                name,
                enum_args
                    .into_iter()
                    .map(|arg| self.substitute_type_params(arg, params, args))
                    .collect(),
            ),
            Type::List(item) => {
                Type::List(Box::new(self.substitute_type_params(*item, params, args)))
            }
            Type::Map(key, value) => Type::Map(
                Box::new(self.substitute_type_params(*key, params, args)),
                Box::new(self.substitute_type_params(*value, params, args)),
            ),
            Type::Option(item) => {
                Type::Option(Box::new(self.substitute_type_params(*item, params, args)))
            }
            Type::Result(ok, err) => Type::Result(
                Box::new(self.substitute_type_params(*ok, params, args)),
                Box::new(self.substitute_type_params(*err, params, args)),
            ),
            other => other,
        }
    }

    fn typed_callee_for(&self, callee: &Expr, resolved_ty: &Type) -> TypedCalleeInfo {
        match resolved_ty {
            Type::Builtin(builtin) => self
                .binding_for_expr(callee.id())
                .map(|binding| TypedCalleeInfo::Builtin {
                    binding,
                    name: Self::builtin_name(*builtin),
                })
                .unwrap_or(TypedCalleeInfo::Error),
            Type::EnumConstructor {
                enum_name,
                enum_item,
                variant_name,
            } => self
                .binding_for_expr(callee.id())
                .map(|binding| TypedCalleeInfo::EnumVariant {
                    binding,
                    enum_name: *enum_name,
                    enum_item: *enum_item,
                    variant_name: *variant_name,
                })
                .unwrap_or(TypedCalleeInfo::Error),
            Type::Function(_) | Type::Unknown(_) => self
                .binding_for_expr(callee.id())
                .map(TypedCalleeInfo::Binding)
                .unwrap_or(TypedCalleeInfo::Value),
            Type::Error => TypedCalleeInfo::Error,
            _ => TypedCalleeInfo::Error,
        }
    }

    fn binding_for_expr(&self, expr_id: ExprId) -> Option<BindingId> {
        self.identifier_refs
            .iter()
            .rev()
            .find(|identifier| identifier.expr_id == expr_id)
            .map(|identifier| identifier.binding)
    }

    fn builtin_name(builtin: BuiltinId) -> &'static str {
        prelude::builtin_name(builtin)
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

    fn insert_current(
        &mut self,
        name: Symbol,
        kind: BindingKind,
        ty: Type,
        span: Span,
    ) -> BindingId {
        let id = BindingId::new(self.bindings.len() as u32);
        self.bindings.push(Binding {
            id,
            symbol: name,
            kind,
            ty,
            span,
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
            Self::Enum(_, _) => "Enum",
            Self::GenericParam(_) => "Type parameter",
            Self::List(_) => "List",
            Self::Map(_, _) => "Map",
            Self::Option(_) | Self::OptionNone => "Option",
            Self::Result(_, _) => "Result",
            Self::EnumConstructor { .. } => "Enum variant",
            Self::Function(_) => "Function",
            Self::Builtin(builtin) => prelude::builtin_debug_label(*builtin),
            Self::Unknown(_) => "Unknown",
            Self::Error => "Error",
        }
    }
}

fn split_variant_name(name: &str) -> Option<(&str, &str)> {
    let (enum_name, variant_name) = name.rsplit_once("::")?;
    if enum_name.is_empty() || variant_name.is_empty() {
        None
    } else {
        Some((enum_name, variant_name))
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
    let mut state = SccState::new(graph);

    for node in graph.keys() {
        if !state.indices.contains_key(node) {
            state.strong_connect(*node);
        }
    }

    state.components
}

struct SccState<'a> {
    graph: &'a HashMap<Symbol, HashSet<Symbol>>,
    index: usize,
    stack: Vec<Symbol>,
    indices: HashMap<Symbol, usize>,
    lowlinks: HashMap<Symbol, usize>,
    on_stack: HashSet<Symbol>,
    components: Vec<Vec<Symbol>>,
}

impl<'a> SccState<'a> {
    fn new(graph: &'a HashMap<Symbol, HashSet<Symbol>>) -> Self {
        Self {
            graph,
            index: 0,
            stack: Vec::new(),
            indices: HashMap::new(),
            lowlinks: HashMap::new(),
            on_stack: HashSet::new(),
            components: Vec::new(),
        }
    }

    fn strong_connect(&mut self, node: Symbol) {
        self.indices.insert(node, self.index);
        self.lowlinks.insert(node, self.index);
        self.index += 1;
        self.stack.push(node);
        self.on_stack.insert(node);

        if let Some(neighbors) = self.graph.get(&node) {
            for neighbor in neighbors {
                if !self.indices.contains_key(neighbor) {
                    self.strong_connect(*neighbor);
                    let neighbor_low = self.lowlinks[neighbor];
                    let node_low = self.lowlinks[&node];
                    self.lowlinks.insert(node, node_low.min(neighbor_low));
                } else if self.on_stack.contains(neighbor) {
                    let neighbor_index = self.indices[neighbor];
                    let node_low = self.lowlinks[&node];
                    self.lowlinks.insert(node, node_low.min(neighbor_index));
                }
            }
        }

        if self.lowlinks[&node] == self.indices[&node] {
            let mut component = Vec::new();
            while let Some(candidate) = self.stack.pop() {
                self.on_stack.remove(&candidate);
                component.push(candidate);
                if candidate == node {
                    break;
                }
            }
            self.components.push(component);
        }
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
            Stmt::EnumDecl(_) => {}
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
        Expr::ListLit(expr) => {
            for item in &expr.items {
                collect_calls_in_expr(item, local_names, calls, symbols);
            }
        }
        Expr::Index(expr) => {
            collect_calls_in_expr(&expr.base, local_names, calls, symbols);
            collect_calls_in_expr(&expr.index, local_names, calls, symbols);
        }
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
        Expr::Match(expr) => {
            collect_calls_in_expr(&expr.value, local_names, calls, symbols);
            for arm in &expr.arms {
                collect_calls_in_expr(&arm.value, local_names, calls, symbols);
            }
        }
        Expr::Fn(_) => {}
    }
}
