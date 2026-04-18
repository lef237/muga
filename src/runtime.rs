use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{ast::*, diagnostic::Diagnostic, span::Span};

type EnvRef = Rc<RefCell<Env>>;

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
    Function(Rc<FunctionValue>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value}"),
            Self::Function(_) => write!(f, "<function>"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RunOutcome {
    pub main_result: Option<Value>,
}

pub fn run(program: &Program) -> Result<RunOutcome, Vec<Diagnostic>> {
    let root = Rc::new(RefCell::new(Env::new(None, true)));
    execute_scope_statements(&program.statements, &root)?;

    match lookup_any(&root, "main") {
        None => Ok(RunOutcome { main_result: None }),
        Some(Binding {
            value: Value::Function(function),
            ..
        }) => {
            if !function.params.is_empty() {
                return Err(vec![Diagnostic::new(
                    "R001",
                    "`main` must be a zero-argument function to be used as the CLI entrypoint",
                    function.span,
                )]);
            }
            let value = call_function(&function, Vec::new())?;
            Ok(RunOutcome {
                main_result: Some(value),
            })
        }
        Some(binding) => Err(vec![Diagnostic::new(
            "R002",
            "`main` must be a function",
            binding.span,
        )]),
    }
}

#[derive(Clone, Debug)]
pub struct FunctionValue {
    params: Vec<String>,
    body: ValueBlock,
    env: EnvRef,
    span: Span,
}

#[derive(Clone, Debug)]
struct Binding {
    mutable: bool,
    value: Value,
    span: Span,
}

#[derive(Debug)]
struct Env {
    bindings: HashMap<String, Binding>,
    parent: Option<EnvRef>,
    function_boundary: bool,
}

impl Env {
    fn new(parent: Option<EnvRef>, function_boundary: bool) -> Self {
        Self {
            bindings: HashMap::new(),
            parent,
            function_boundary,
        }
    }
}

fn execute_scope_statements(statements: &[Stmt], env: &EnvRef) -> Result<(), Vec<Diagnostic>> {
    predeclare_functions(statements, env)?;
    for statement in statements {
        match statement {
            Stmt::FuncDecl(_) => {}
            _ => execute_stmt(statement, env)?,
        }
    }
    Ok(())
}

fn execute_stmt(statement: &Stmt, env: &EnvRef) -> Result<(), Vec<Diagnostic>> {
    match statement {
        Stmt::Assign(stmt) => {
            execute_assign(stmt, env)?;
        }
        Stmt::FuncDecl(_) => {}
        Stmt::If(stmt) => {
            let condition = eval_expr(&stmt.condition, env)?;
            match condition {
                Value::Bool(true) => {
                    let child = Rc::new(RefCell::new(Env::new(Some(env.clone()), false)));
                    execute_scope_statements(&stmt.then_branch.statements, &child)?;
                }
                Value::Bool(false) => {
                    if let Some(else_branch) = &stmt.else_branch {
                        let child = Rc::new(RefCell::new(Env::new(Some(env.clone()), false)));
                        execute_scope_statements(&else_branch.statements, &child)?;
                    }
                }
                _ => {
                    return Err(vec![Diagnostic::new(
                        "R003",
                        "`if` condition did not evaluate to Bool",
                        stmt.condition.span(),
                    )]);
                }
            }
        }
        Stmt::While(stmt) => loop {
            let condition = eval_expr(&stmt.condition, env)?;
            match condition {
                Value::Bool(true) => {
                    let child = Rc::new(RefCell::new(Env::new(Some(env.clone()), false)));
                    execute_scope_statements(&stmt.body.statements, &child)?;
                }
                Value::Bool(false) => break,
                _ => {
                    return Err(vec![Diagnostic::new(
                        "R003",
                        "`while` condition did not evaluate to Bool",
                        stmt.condition.span(),
                    )]);
                }
            }
        },
        Stmt::Expr(stmt) => {
            let _ = eval_expr(&stmt.expr, env)?;
        }
    }

    Ok(())
}

fn execute_assign(stmt: &AssignStmt, env: &EnvRef) -> Result<(), Vec<Diagnostic>> {
    let value = eval_expr(&stmt.value, env)?;

    if stmt.mutable {
        if env.borrow().bindings.contains_key(&stmt.name) {
            return Err(vec![Diagnostic::new(
                "R004",
                format!("duplicate binding `{}` in the current scope", stmt.name),
                stmt.span,
            )]);
        }
        if lookup_any_enclosing(env, &stmt.name).is_some() {
            return Err(vec![Diagnostic::new(
                "R005",
                format!("shadowing is prohibited for `{}`", stmt.name),
                stmt.span,
            )]);
        }
        env.borrow_mut().bindings.insert(
            stmt.name.clone(),
            Binding {
                mutable: true,
                value,
                span: stmt.span,
            },
        );
        return Ok(());
    }

    if let Some(target_env) = lookup_in_current_function_env(env, &stmt.name) {
        let mut target = target_env.borrow_mut();
        let binding = target
            .bindings
            .get_mut(&stmt.name)
            .expect("binding must exist");
        if binding.mutable {
            binding.value = value;
            binding.span = stmt.span;
            return Ok(());
        }
        return Err(vec![Diagnostic::new(
            "R006",
            format!("cannot update immutable binding `{}`", stmt.name),
            stmt.span,
        )]);
    }

    if let Some(binding) = lookup_beyond_current_function(env, &stmt.name) {
        let code = if binding.mutable { "R007" } else { "R005" };
        let message = if binding.mutable {
            format!(
                "cannot update outer-scope mutable binding `{}` in v1",
                stmt.name
            )
        } else {
            format!("shadowing is prohibited for `{}`", stmt.name)
        };
        return Err(vec![Diagnostic::new(code, message, stmt.span)]);
    }

    env.borrow_mut().bindings.insert(
        stmt.name.clone(),
        Binding {
            mutable: false,
            value,
            span: stmt.span,
        },
    );
    Ok(())
}

fn eval_expr(expr: &Expr, env: &EnvRef) -> Result<Value, Vec<Diagnostic>> {
    match expr {
        Expr::Int(expr) => Ok(Value::Int(expr.value)),
        Expr::Bool(expr) => Ok(Value::Bool(expr.value)),
        Expr::String(expr) => Ok(Value::String(expr.value.clone())),
        Expr::Ident(expr) => lookup_any(env, &expr.name)
            .map(|binding| binding.value)
            .ok_or_else(|| {
                vec![Diagnostic::new(
                    "R008",
                    format!("unresolved runtime name `{}`", expr.name),
                    expr.span,
                )]
            }),
        Expr::Unary(expr) => {
            let value = eval_expr(&expr.expr, env)?;
            match (expr.op, value) {
                (UnaryOp::Neg, Value::Int(value)) => Ok(Value::Int(-value)),
                (UnaryOp::Not, Value::Bool(value)) => Ok(Value::Bool(!value)),
                _ => Err(vec![Diagnostic::new(
                    "R009",
                    "invalid operand for unary operator",
                    expr.span,
                )]),
            }
        }
        Expr::Binary(expr) => {
            let left = eval_expr(&expr.left, env)?;
            let right = eval_expr(&expr.right, env)?;
            eval_binary(expr, left, right)
        }
        Expr::Call(expr) => {
            let callee = eval_expr(&expr.callee, env)?;
            let mut args = Vec::with_capacity(expr.args.len());
            for arg in &expr.args {
                args.push(eval_expr(arg, env)?);
            }
            match callee {
                Value::Function(function) => call_function(&function, args),
                _ => Err(vec![Diagnostic::new(
                    "R010",
                    "attempted to call a non-function value",
                    expr.span,
                )]),
            }
        }
        Expr::If(expr) => {
            let condition = eval_expr(&expr.condition, env)?;
            match condition {
                Value::Bool(true) => eval_value_block(&expr.then_branch, env),
                Value::Bool(false) => eval_value_block(&expr.else_branch, env),
                _ => Err(vec![Diagnostic::new(
                    "R003",
                    "`if` condition did not evaluate to Bool",
                    expr.condition.span(),
                )]),
            }
        }
        Expr::Fn(expr) => Ok(Value::Function(Rc::new(FunctionValue {
            params: expr.params.iter().map(|param| param.name.clone()).collect(),
            body: expr.body.clone(),
            env: env.clone(),
            span: expr.span,
        }))),
    }
}

fn eval_binary(expr: &BinaryExpr, left: Value, right: Value) -> Result<Value, Vec<Diagnostic>> {
    match (expr.op, left, right) {
        (BinaryOp::Add, Value::Int(left), Value::Int(right)) => Ok(Value::Int(left + right)),
        (BinaryOp::Sub, Value::Int(left), Value::Int(right)) => Ok(Value::Int(left - right)),
        (BinaryOp::Mul, Value::Int(left), Value::Int(right)) => Ok(Value::Int(left * right)),
        (BinaryOp::Div, Value::Int(left), Value::Int(right)) => Ok(Value::Int(left / right)),
        (BinaryOp::Lt, Value::Int(left), Value::Int(right)) => Ok(Value::Bool(left < right)),
        (BinaryOp::LtEq, Value::Int(left), Value::Int(right)) => Ok(Value::Bool(left <= right)),
        (BinaryOp::Gt, Value::Int(left), Value::Int(right)) => Ok(Value::Bool(left > right)),
        (BinaryOp::GtEq, Value::Int(left), Value::Int(right)) => Ok(Value::Bool(left >= right)),
        (BinaryOp::EqEq, Value::Int(left), Value::Int(right)) => Ok(Value::Bool(left == right)),
        (BinaryOp::EqEq, Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(left == right)),
        (BinaryOp::EqEq, Value::String(left), Value::String(right)) => {
            Ok(Value::Bool(left == right))
        }
        (BinaryOp::BangEq, Value::Int(left), Value::Int(right)) => Ok(Value::Bool(left != right)),
        (BinaryOp::BangEq, Value::Bool(left), Value::Bool(right)) => Ok(Value::Bool(left != right)),
        (BinaryOp::BangEq, Value::String(left), Value::String(right)) => {
            Ok(Value::Bool(left != right))
        }
        _ => Err(vec![Diagnostic::new(
            "R011",
            "invalid operands for binary operator",
            expr.span,
        )]),
    }
}

fn eval_value_block(block: &ValueBlock, env: &EnvRef) -> Result<Value, Vec<Diagnostic>> {
    let child = Rc::new(RefCell::new(Env::new(Some(env.clone()), false)));
    execute_scope_statements(&block.statements, &child)?;
    eval_expr(&block.expr, &child)
}

fn call_function(function: &FunctionValue, args: Vec<Value>) -> Result<Value, Vec<Diagnostic>> {
    if function.params.len() != args.len() {
        return Err(vec![Diagnostic::new(
            "R012",
            format!(
                "expected {} arguments but found {}",
                function.params.len(),
                args.len()
            ),
            function.span,
        )]);
    }

    let env = Rc::new(RefCell::new(Env::new(Some(function.env.clone()), true)));
    for (param, arg) in function.params.iter().zip(args) {
        env.borrow_mut().bindings.insert(
            param.clone(),
            Binding {
                mutable: false,
                value: arg,
                span: function.span,
            },
        );
    }
    execute_scope_statements(&function.body.statements, &env)?;
    eval_expr(&function.body.expr, &env)
}

fn predeclare_functions(statements: &[Stmt], env: &EnvRef) -> Result<(), Vec<Diagnostic>> {
    for statement in statements {
        if let Stmt::FuncDecl(func) = statement {
            if env.borrow().bindings.contains_key(&func.name) {
                return Err(vec![Diagnostic::new(
                    "R004",
                    format!("duplicate binding `{}` in the current scope", func.name),
                    func.span,
                )]);
            }

            let function = Value::Function(Rc::new(FunctionValue {
                params: func.params.iter().map(|param| param.name.clone()).collect(),
                body: func.body.clone(),
                env: env.clone(),
                span: func.span,
            }));
            env.borrow_mut().bindings.insert(
                func.name.clone(),
                Binding {
                    mutable: false,
                    value: function,
                    span: func.span,
                },
            );
        }
    }
    Ok(())
}

fn lookup_any(env: &EnvRef, name: &str) -> Option<Binding> {
    let mut current = Some(env.clone());
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if let Some(binding) = borrowed.bindings.get(name) {
            return Some(binding.clone());
        }
        current = borrowed.parent.clone();
    }
    None
}

fn lookup_any_enclosing(env: &EnvRef, name: &str) -> Option<Binding> {
    let mut current = env.borrow().parent.clone();
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if let Some(binding) = borrowed.bindings.get(name) {
            return Some(binding.clone());
        }
        current = borrowed.parent.clone();
    }
    None
}

fn lookup_in_current_function_env(env: &EnvRef, name: &str) -> Option<EnvRef> {
    let mut current = Some(env.clone());
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if borrowed.bindings.contains_key(name) {
            return Some(candidate.clone());
        }
        let stop = borrowed.function_boundary;
        let parent = borrowed.parent.clone();
        drop(borrowed);
        if stop {
            break;
        }
        current = parent;
    }
    None
}

fn lookup_beyond_current_function(env: &EnvRef, name: &str) -> Option<Binding> {
    let mut current = Some(env.clone());
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        let stop = borrowed.function_boundary;
        let parent = borrowed.parent.clone();
        drop(borrowed);
        if stop {
            current = parent;
            break;
        }
        current = parent;
    }

    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if let Some(binding) = borrowed.bindings.get(name) {
            return Some(binding.clone());
        }
        current = borrowed.parent.clone();
    }
    None
}
