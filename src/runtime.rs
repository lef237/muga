use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{diagnostic::Diagnostic, hir::*, span::Span};

type EnvRef = Rc<RefCell<Env>>;

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
    Function(Rc<ClosureValue>),
    Builtin(BuiltinFunction),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value}"),
            Self::Function(_) => write!(f, "<function>"),
            Self::Builtin(builtin) => write!(f, "<builtin:{}>", builtin.name()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RunOutcome {
    pub main_result: Option<Value>,
    pub output_lines: Vec<String>,
}

pub fn run(program: &Program) -> Result<RunOutcome, Vec<Diagnostic>> {
    let output = Rc::new(RefCell::new(Vec::new()));
    let root = Rc::new(RefCell::new(Env::new(None, true, output.clone())));
    install_prelude(&root);
    execute_scope_statements(program, &program.statements, &root)?;

    match lookup_any(&root, "main") {
        None => Ok(RunOutcome {
            main_result: None,
            output_lines: output.borrow().clone(),
        }),
        Some(Binding {
            value: Value::Function(function),
            ..
        }) => {
            let def = function.definition(program);
            if !def.params.is_empty() {
                return Err(vec![Diagnostic::new(
                    "R001",
                    "`main` must be a zero-argument function to be used as the CLI entrypoint",
                    def.span,
                )]);
            }
            let value = call_function(program, &function, Vec::new())?;
            Ok(RunOutcome {
                main_result: Some(value),
                output_lines: output.borrow().clone(),
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
pub struct ClosureValue {
    function: FunctionId,
    env: EnvRef,
}

impl ClosureValue {
    fn definition<'a>(&self, program: &'a Program) -> &'a Function {
        &program.functions[self.function]
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BuiltinFunction {
    Print,
}

impl BuiltinFunction {
    fn name(self) -> &'static str {
        match self {
            Self::Print => "print",
        }
    }
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
    output: Rc<RefCell<Vec<String>>>,
}

impl Env {
    fn new(
        parent: Option<EnvRef>,
        function_boundary: bool,
        output: Rc<RefCell<Vec<String>>>,
    ) -> Self {
        Self {
            bindings: HashMap::new(),
            parent,
            function_boundary,
            output,
        }
    }
}

fn execute_scope_statements(
    program: &Program,
    statements: &[Stmt],
    env: &EnvRef,
) -> Result<(), Vec<Diagnostic>> {
    predeclare_functions(statements, env);
    for statement in statements {
        match statement {
            Stmt::Function(_) => {}
            _ => execute_stmt(program, statement, env)?,
        }
    }
    Ok(())
}

fn execute_stmt(program: &Program, statement: &Stmt, env: &EnvRef) -> Result<(), Vec<Diagnostic>> {
    match statement {
        Stmt::Assign(stmt) => execute_assign(program, stmt, env)?,
        Stmt::Function(_) => {}
        Stmt::If(stmt) => {
            let condition = eval_expr(program, &stmt.condition, env)?;
            match condition {
                Value::Bool(true) => {
                    let child = child_env(env, false);
                    execute_scope_statements(program, &stmt.then_branch.statements, &child)?;
                }
                Value::Bool(false) => {
                    if let Some(else_branch) = &stmt.else_branch {
                        let child = child_env(env, false);
                        execute_scope_statements(program, &else_branch.statements, &child)?;
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
            let condition = eval_expr(program, &stmt.condition, env)?;
            match condition {
                Value::Bool(true) => {
                    let child = child_env(env, false);
                    execute_scope_statements(program, &stmt.body.statements, &child)?;
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
            let _ = eval_expr(program, &stmt.expr, env)?;
        }
    }
    Ok(())
}

fn execute_assign(
    program: &Program,
    stmt: &AssignStmt,
    env: &EnvRef,
) -> Result<(), Vec<Diagnostic>> {
    let value = eval_expr(program, &stmt.value, env)?;

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

fn eval_expr(program: &Program, expr: &Expr, env: &EnvRef) -> Result<Value, Vec<Diagnostic>> {
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
            let value = eval_expr(program, &expr.expr, env)?;
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
            let left = eval_expr(program, &expr.left, env)?;
            let right = eval_expr(program, &expr.right, env)?;
            eval_binary(expr, left, right)
        }
        Expr::Call(expr) => {
            let callee = eval_expr(program, &expr.callee, env)?;
            let mut args = Vec::with_capacity(expr.args.len());
            for arg in &expr.args {
                args.push(eval_expr(program, arg, env)?);
            }
            match callee {
                Value::Function(function) => call_function(program, &function, args),
                Value::Builtin(builtin) => call_builtin(builtin, args, env, expr.span),
                _ => Err(vec![Diagnostic::new(
                    "R010",
                    "attempted to call a non-function value",
                    expr.span,
                )]),
            }
        }
        Expr::If(expr) => {
            let condition = eval_expr(program, &expr.condition, env)?;
            match condition {
                Value::Bool(true) => eval_value_block(program, &expr.then_branch, env),
                Value::Bool(false) => eval_value_block(program, &expr.else_branch, env),
                _ => Err(vec![Diagnostic::new(
                    "R003",
                    "`if` condition did not evaluate to Bool",
                    expr.condition.span(),
                )]),
            }
        }
        Expr::Closure(expr) => Ok(Value::Function(Rc::new(ClosureValue {
            function: expr.function,
            env: env.clone(),
        }))),
    }
}

fn eval_binary(expr: &BinaryExpr, left: Value, right: Value) -> Result<Value, Vec<Diagnostic>> {
    match (expr.op, left, right) {
        (BinaryOp::Add, Value::Int(left), Value::Int(right)) => Ok(Value::Int(left + right)),
        (BinaryOp::Sub, Value::Int(left), Value::Int(right)) => Ok(Value::Int(left - right)),
        (BinaryOp::Mul, Value::Int(left), Value::Int(right)) => Ok(Value::Int(left * right)),
        (BinaryOp::Div, Value::Int(_), Value::Int(0)) => {
            Err(vec![Diagnostic::new("R013", "division by zero", expr.span)])
        }
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

fn eval_value_block(
    program: &Program,
    block: &ValueBlock,
    env: &EnvRef,
) -> Result<Value, Vec<Diagnostic>> {
    let child = child_env(env, false);
    execute_scope_statements(program, &block.statements, &child)?;
    eval_expr(program, &block.expr, &child)
}

fn call_function(
    program: &Program,
    function: &ClosureValue,
    args: Vec<Value>,
) -> Result<Value, Vec<Diagnostic>> {
    let definition = function.definition(program);
    if definition.params.len() != args.len() {
        return Err(vec![Diagnostic::new(
            "R012",
            format!(
                "expected {} arguments but found {}",
                definition.params.len(),
                args.len()
            ),
            definition.span,
        )]);
    }

    let env = Rc::new(RefCell::new(Env::new(
        Some(function.env.clone()),
        true,
        function.env.borrow().output.clone(),
    )));
    for (param, arg) in definition.params.iter().zip(args) {
        env.borrow_mut().bindings.insert(
            param.clone(),
            Binding {
                mutable: false,
                value: arg,
                span: definition.span,
            },
        );
    }
    execute_scope_statements(program, &definition.body.statements, &env)?;
    eval_expr(program, &definition.body.expr, &env)
}

fn call_builtin(
    builtin: BuiltinFunction,
    args: Vec<Value>,
    env: &EnvRef,
    span: Span,
) -> Result<Value, Vec<Diagnostic>> {
    match builtin {
        BuiltinFunction::Print => {
            if args.len() != 1 {
                return Err(vec![Diagnostic::new(
                    "R012",
                    format!("expected 1 arguments but found {}", args.len()),
                    span,
                )]);
            }
            let value = args.into_iter().next().expect("checked length");
            match &value {
                Value::Int(_) | Value::Bool(_) | Value::String(_) => {
                    env.borrow().output.borrow_mut().push(value.to_string());
                    Ok(value)
                }
                Value::Function(_) | Value::Builtin(_) => Err(vec![Diagnostic::new(
                    "R014",
                    "`print` accepts only Int, Bool, or String",
                    span,
                )]),
            }
        }
    }
}

fn predeclare_functions(statements: &[Stmt], env: &EnvRef) {
    for statement in statements {
        if let Stmt::Function(func) = statement {
            env.borrow_mut().bindings.insert(
                func.name.clone(),
                Binding {
                    mutable: false,
                    value: Value::Function(Rc::new(ClosureValue {
                        function: func.function,
                        env: env.clone(),
                    })),
                    span: func.span,
                },
            );
        }
    }
}

fn install_prelude(env: &EnvRef) {
    env.borrow_mut().bindings.insert(
        "print".to_string(),
        Binding {
            mutable: false,
            value: Value::Builtin(BuiltinFunction::Print),
            span: Span::default(),
        },
    );
}

fn child_env(parent: &EnvRef, function_boundary: bool) -> EnvRef {
    Rc::new(RefCell::new(Env::new(
        Some(parent.clone()),
        function_boundary,
        parent.borrow().output.clone(),
    )))
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
    let mut first_boundary_seen = false;
    let mut current = Some(env.clone());
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if first_boundary_seen {
            if let Some(binding) = borrowed.bindings.get(name) {
                return Some(binding.clone());
            }
        }
        if borrowed.function_boundary {
            first_boundary_seen = true;
        }
        current = borrowed.parent.clone();
    }
    None
}
