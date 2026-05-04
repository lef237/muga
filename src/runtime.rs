use std::{cell::RefCell, collections::HashMap, fmt, rc::Rc};

use crate::{bytecode::*, diagnostic::Diagnostic, span::Span, symbol::Symbol};

type EnvRef = Rc<RefCell<Env>>;

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
    List(Vec<Value>),
    Map(MapValue),
    Option(OptionValue),
    Record(RecordValue),
    Function(Rc<ClosureValue>),
    Builtin(BuiltinFunction),
}

#[derive(Clone, Debug)]
pub struct RecordValue {
    type_name: String,
    fields: Vec<RecordFieldValue>,
}

#[derive(Clone, Debug)]
pub enum OptionValue {
    Some(Box<Value>),
    None,
}

#[derive(Clone, Debug)]
pub struct MapValue {
    entries: Vec<MapEntryValue>,
}

#[derive(Clone, Debug)]
struct MapEntryValue {
    key: MapKey,
    value: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum MapKey {
    Int(i64),
    Bool(bool),
    String(String),
}

#[derive(Clone, Debug)]
struct RecordFieldValue {
    name: String,
    value: Value,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value}"),
            Self::List(items) => {
                write!(f, "[")?;
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{item}")?;
                }
                write!(f, "]")
            }
            Self::Map(map) => {
                write!(f, "Map {{")?;
                for (index, entry) in map.entries.iter().enumerate() {
                    if index == 0 {
                        write!(f, " ")?;
                    } else {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", entry.key, entry.value)?;
                }
                if !map.entries.is_empty() {
                    write!(f, " ")?;
                }
                write!(f, "}}")
            }
            Self::Option(OptionValue::Some(value)) => write!(f, "Option::Some({value})"),
            Self::Option(OptionValue::None) => write!(f, "Option::None"),
            Self::Record(record) => {
                write!(f, "{} {{ ", record.type_name)?;
                for (index, field) in record.fields.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field.name, field.value)?;
                }
                write!(f, " }}")
            }
            Self::Function(_) => write!(f, "<function>"),
            Self::Builtin(builtin) => write!(f, "<builtin:{}>", builtin.name()),
        }
    }
}

impl fmt::Display for MapKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "\"{value}\""),
        }
    }
}

#[derive(Clone, Debug)]
pub struct RunOutcome {
    pub main_result: Option<Value>,
    pub output_text: String,
}

pub fn run(program: &Program) -> Result<RunOutcome, Vec<Diagnostic>> {
    let output = Rc::new(RefCell::new(String::new()));
    let root = Rc::new(RefCell::new(Env::new(None, true, output.clone())));
    install_prelude(program, &root);
    let _ = execute_chunk(program, &program.entry, root.clone())?;

    match main_symbol(program) {
        Some(main_symbol) => match lookup_any(&root, main_symbol) {
            None => Ok(RunOutcome {
                main_result: None,
                output_text: output.borrow().clone(),
            }),
            Some(Binding {
                value: Value::Function(function),
                ..
            }) => {
                let definition = function.definition(program);
                if !definition.params.is_empty() {
                    return Err(vec![Diagnostic::new(
                        "R001",
                        "`main` must be a zero-argument function to be used as the CLI entrypoint",
                        definition.span,
                    )]);
                }
                let value = call_function(program, &function, Vec::new())?;
                Ok(RunOutcome {
                    main_result: Some(value),
                    output_text: output.borrow().clone(),
                })
            }
            Some(binding) => Err(vec![Diagnostic::new(
                "R002",
                "`main` must be a function",
                binding.span,
            )]),
        },
        None => Ok(RunOutcome {
            main_result: None,
            output_text: output.borrow().clone(),
        }),
    }
}

fn main_symbol(program: &Program) -> Option<Symbol> {
    program.symbols.lookup("main")
}

fn symbol_name(program: &Program, symbol: Symbol) -> &str {
    program.symbols.resolve(symbol)
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
    Println,
    Len,
    IsEmpty,
    ListPush,
    Get,
    ListSet,
    MapEmpty,
    Contains,
    Insert,
    Remove,
    OptionSome,
}

impl BuiltinFunction {
    fn name(self) -> &'static str {
        match self {
            Self::Print => "print",
            Self::Println => "println",
            Self::Len => "len",
            Self::IsEmpty => "is_empty",
            Self::ListPush => "push",
            Self::Get => "get",
            Self::ListSet => "set",
            Self::MapEmpty => "Map.empty",
            Self::Contains => "contains",
            Self::Insert => "insert",
            Self::Remove => "remove",
            Self::OptionSome => "Option::Some",
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
    bindings: HashMap<Symbol, Binding>,
    parent: Option<EnvRef>,
    function_boundary: bool,
    output: Rc<RefCell<String>>,
}

impl Env {
    fn new(parent: Option<EnvRef>, function_boundary: bool, output: Rc<RefCell<String>>) -> Self {
        Self {
            bindings: HashMap::new(),
            parent,
            function_boundary,
            output,
        }
    }
}

fn execute_chunk(
    program: &Program,
    chunk: &Chunk,
    env: EnvRef,
) -> Result<Option<Value>, Vec<Diagnostic>> {
    let mut stack = Vec::<Value>::new();
    let mut current_env = env;
    let mut pc = 0usize;

    while let Some(instruction) = chunk.instructions.get(pc) {
        match instruction {
            Instruction::LoadInt(value) => stack.push(Value::Int(*value)),
            Instruction::LoadBool(value) => stack.push(Value::Bool(*value)),
            Instruction::LoadString(value) => stack.push(Value::String(value.clone())),
            Instruction::MakeRecord {
                type_name,
                fields,
                span,
            } => {
                let values = pop_args(&mut stack, fields.len(), *span)?;
                stack.push(make_record_value(program, *type_name, fields, values));
            }
            Instruction::MakeList { len, span } => {
                let values = pop_args(&mut stack, *len, *span)?;
                stack.push(Value::List(values));
            }
            Instruction::LoadName { name, span } => {
                let Some(binding) = lookup_any(&current_env, *name) else {
                    return Err(vec![Diagnostic::new(
                        "R008",
                        format!("unresolved runtime name `{}`", symbol_name(program, *name)),
                        *span,
                    )]);
                };
                stack.push(binding.value);
            }
            Instruction::LoadField { field, span } => {
                let base = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing record value for field access",
                )?;
                let value = load_record_field(program, base, *field, *span)?;
                stack.push(value);
            }
            Instruction::LoadIndex { span } => {
                let index = pop_value(&mut stack, *span, "R015", "missing list index")?;
                let base = pop_value(&mut stack, *span, "R015", "missing list value")?;
                let value = load_list_index(base, index, *span)?;
                stack.push(value);
            }
            Instruction::UpdateRecord { fields, span } => {
                let values = pop_args(&mut stack, fields.len(), *span)?;
                let base = pop_value(&mut stack, *span, "R015", "missing record value for update")?;
                let value = update_record_value(program, base, fields, values, *span)?;
                stack.push(value);
            }
            Instruction::Assign {
                name,
                mutable,
                span,
            } => {
                let value = pop_value(&mut stack, *span, "R015", "missing value for assignment")?;
                execute_assign(program, &current_env, *name, *mutable, value, *span)?;
            }
            Instruction::DefineFunction {
                name,
                function,
                span,
            } => {
                current_env.borrow_mut().bindings.insert(
                    *name,
                    Binding {
                        mutable: false,
                        value: Value::Function(Rc::new(ClosureValue {
                            function: *function,
                            env: current_env.clone(),
                        })),
                        span: *span,
                    },
                );
            }
            Instruction::MakeClosure { function } => {
                stack.push(Value::Function(Rc::new(ClosureValue {
                    function: *function,
                    env: current_env.clone(),
                })));
            }
            Instruction::UnaryNeg { span } => {
                let value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing operand for unary operator",
                )?;
                match value {
                    Value::Int(value) => {
                        let Some(value) = value.checked_neg() else {
                            return Err(integer_overflow(*span));
                        };
                        stack.push(Value::Int(value));
                    }
                    _ => {
                        return Err(vec![Diagnostic::new(
                            "R009",
                            "invalid operand for unary operator",
                            *span,
                        )]);
                    }
                }
            }
            Instruction::UnaryNot { span } => {
                let value = pop_value(
                    &mut stack,
                    *span,
                    "R015",
                    "missing operand for unary operator",
                )?;
                match value {
                    Value::Bool(value) => stack.push(Value::Bool(!value)),
                    _ => {
                        return Err(vec![Diagnostic::new(
                            "R009",
                            "invalid operand for unary operator",
                            *span,
                        )]);
                    }
                }
            }
            Instruction::Binary { op, span } => {
                let right = pop_value(&mut stack, *span, "R015", "missing right operand")?;
                let left = pop_value(&mut stack, *span, "R015", "missing left operand")?;
                let value = eval_binary(*op, left, right, *span)?;
                stack.push(value);
            }
            Instruction::Call { argc, span } => {
                let args = pop_args(&mut stack, *argc, *span)?;
                let callee = pop_value(&mut stack, *span, "R015", "missing callee for call")?;
                let value = call_value(program, callee, args, &current_env, *span)?;
                stack.push(value);
            }
            Instruction::JumpIfFalse { target, span } => {
                let condition = pop_value(&mut stack, *span, "R015", "missing condition for jump")?;
                match condition {
                    Value::Bool(false) => {
                        pc = *target;
                        continue;
                    }
                    Value::Bool(true) => {}
                    _ => {
                        return Err(vec![Diagnostic::new(
                            "R003",
                            "`if`/`while` condition did not evaluate to Bool",
                            *span,
                        )]);
                    }
                }
            }
            Instruction::JumpIfOptionNone { target, span } => {
                let value = pop_value(&mut stack, *span, "R015", "missing Option value for match")?;
                match value {
                    Value::Option(OptionValue::Some(value)) => stack.push(*value),
                    Value::Option(OptionValue::None) => {
                        pc = *target;
                        continue;
                    }
                    _ => {
                        return Err(vec![Diagnostic::new(
                            "R019",
                            "`match` expected an Option value",
                            *span,
                        )]);
                    }
                }
            }
            Instruction::Jump { target } => {
                pc = *target;
                continue;
            }
            Instruction::PushScope => {
                current_env = child_env(&current_env, false);
            }
            Instruction::PopScope => {
                let parent = current_env
                    .borrow()
                    .parent
                    .clone()
                    .expect("scope must have parent");
                current_env = parent;
            }
            Instruction::Pop => {
                let _ = pop_value(
                    &mut stack,
                    Span::default(),
                    "R015",
                    "missing value to discard",
                )?;
            }
            Instruction::Return => {
                let value = pop_value(
                    &mut stack,
                    Span::default(),
                    "R015",
                    "missing return value at end of function",
                )?;
                return Ok(Some(value));
            }
        }
        pc += 1;
    }

    Ok(None)
}

fn execute_assign(
    program: &Program,
    env: &EnvRef,
    name: Symbol,
    mutable: bool,
    value: Value,
    span: Span,
) -> Result<(), Vec<Diagnostic>> {
    if mutable {
        if env.borrow().bindings.contains_key(&name) {
            return Err(vec![Diagnostic::new(
                "R004",
                format!(
                    "duplicate binding `{}` in the current scope",
                    symbol_name(program, name)
                ),
                span,
            )]);
        }
        if lookup_any_enclosing(env, name).is_some() {
            return Err(vec![Diagnostic::new(
                "R005",
                format!(
                    "shadowing is prohibited for `{}`",
                    symbol_name(program, name)
                ),
                span,
            )]);
        }
        env.borrow_mut().bindings.insert(
            name,
            Binding {
                mutable: true,
                value,
                span,
            },
        );
        return Ok(());
    }

    if let Some(target_env) = lookup_in_current_function_env(env, name) {
        let mut target = target_env.borrow_mut();
        let binding = target.bindings.get_mut(&name).expect("binding must exist");
        if binding.mutable {
            binding.value = value;
            binding.span = span;
            return Ok(());
        }
        return Err(vec![Diagnostic::new(
            "R006",
            format!(
                "cannot update immutable binding `{}`",
                symbol_name(program, name)
            ),
            span,
        )]);
    }

    if let Some(binding) = lookup_beyond_current_function(env, name) {
        let code = if binding.mutable { "R007" } else { "R005" };
        let message = if binding.mutable {
            format!(
                "cannot update outer-scope mutable binding `{}` in v1",
                symbol_name(program, name)
            )
        } else {
            format!(
                "shadowing is prohibited for `{}`",
                symbol_name(program, name)
            )
        };
        return Err(vec![Diagnostic::new(code, message, span)]);
    }

    env.borrow_mut().bindings.insert(
        name,
        Binding {
            mutable: false,
            value,
            span,
        },
    );
    Ok(())
}

fn call_value(
    program: &Program,
    callee: Value,
    args: Vec<Value>,
    env: &EnvRef,
    span: Span,
) -> Result<Value, Vec<Diagnostic>> {
    match callee {
        Value::Function(function) => call_function(program, &function, args),
        Value::Builtin(builtin) => call_builtin(builtin, args, env, span),
        _ => Err(vec![Diagnostic::new(
            "R010",
            "attempted to call a non-function value",
            span,
        )]),
    }
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
            *param,
            Binding {
                mutable: false,
                value: arg,
                span: definition.span,
            },
        );
    }

    execute_chunk(program, &definition.chunk, env)?.ok_or_else(|| {
        vec![Diagnostic::new(
            "R015",
            "function did not produce a value",
            definition.span,
        )]
    })
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
                    env.borrow()
                        .output
                        .borrow_mut()
                        .push_str(&value.to_string());
                    Ok(value)
                }
                Value::List(_)
                | Value::Map(_)
                | Value::Option(_)
                | Value::Record(_)
                | Value::Function(_)
                | Value::Builtin(_) => Err(vec![Diagnostic::new(
                    "R014",
                    "`print` accepts only Int, Bool, or String",
                    span,
                )]),
            }
        }
        BuiltinFunction::Println => {
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
                    let borrowed_env = env.borrow();
                    let mut output = borrowed_env.output.borrow_mut();
                    output.push_str(&value.to_string());
                    output.push('\n');
                    Ok(value)
                }
                Value::List(_)
                | Value::Map(_)
                | Value::Option(_)
                | Value::Record(_)
                | Value::Function(_)
                | Value::Builtin(_) => Err(vec![Diagnostic::new(
                    "R014",
                    "`println` accepts only Int, Bool, or String",
                    span,
                )]),
            }
        }
        BuiltinFunction::Len => {
            if args.len() != 1 {
                return Err(vec![Diagnostic::new(
                    "R012",
                    format!("expected 1 arguments but found {}", args.len()),
                    span,
                )]);
            }
            let value = args.into_iter().next().expect("checked length");
            match value {
                Value::List(items) => Ok(Value::Int(items.len() as i64)),
                Value::Map(map) => Ok(Value::Int(map.entries.len() as i64)),
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`len` expects List[T] or Map[K, V] as its first argument",
                    span,
                )]),
            }
        }
        BuiltinFunction::IsEmpty => {
            if args.len() != 1 {
                return Err(vec![Diagnostic::new(
                    "R012",
                    format!("expected 1 arguments but found {}", args.len()),
                    span,
                )]);
            }
            let value = args.into_iter().next().expect("checked length");
            match value {
                Value::List(items) => Ok(Value::Bool(items.is_empty())),
                Value::Map(map) => Ok(Value::Bool(map.entries.is_empty())),
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`is_empty` expects List[T] or Map[K, V] as its first argument",
                    span,
                )]),
            }
        }
        BuiltinFunction::ListPush => {
            if args.len() != 2 {
                return Err(vec![Diagnostic::new(
                    "R012",
                    format!("expected 2 arguments but found {}", args.len()),
                    span,
                )]);
            }
            let mut args = args.into_iter();
            let list = args.next().expect("checked length");
            let value = args.next().expect("checked length");
            match list {
                Value::List(mut items) => {
                    items.push(value);
                    Ok(Value::List(items))
                }
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`push` expects List[T] as its first argument",
                    span,
                )]),
            }
        }
        BuiltinFunction::Get => {
            if args.len() != 2 {
                return Err(vec![Diagnostic::new(
                    "R012",
                    format!("expected 2 arguments but found {}", args.len()),
                    span,
                )]);
            }
            let mut args = args.into_iter();
            let collection = args.next().expect("checked length");
            let key_or_index = args.next().expect("checked length");
            match collection {
                Value::List(items) => {
                    let Value::Int(index) = key_or_index else {
                        return Err(vec![Diagnostic::new(
                            "R014",
                            "`get` expects Int as its second argument for List[T]",
                            span,
                        )]);
                    };
                    if index < 0 {
                        return Ok(Value::Option(OptionValue::None));
                    }
                    match items.get(index as usize).cloned() {
                        Some(value) => Ok(Value::Option(OptionValue::Some(Box::new(value)))),
                        None => Ok(Value::Option(OptionValue::None)),
                    }
                }
                Value::Map(map) => {
                    let key = map_key(key_or_index, span, "get")?;
                    match map.entries.iter().find(|entry| entry.key == key) {
                        Some(entry) => Ok(Value::Option(OptionValue::Some(Box::new(
                            entry.value.clone(),
                        )))),
                        None => Ok(Value::Option(OptionValue::None)),
                    }
                }
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`get` expects List[T] or Map[K, V] as its first argument",
                    span,
                )]),
            }
        }
        BuiltinFunction::ListSet => {
            if args.len() != 3 {
                return Err(vec![Diagnostic::new(
                    "R012",
                    format!("expected 3 arguments but found {}", args.len()),
                    span,
                )]);
            }
            let mut args = args.into_iter();
            let list = args.next().expect("checked length");
            let index = args.next().expect("checked length");
            let value = args.next().expect("checked length");
            let Value::List(mut items) = list else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`set` expects List[T] as its first argument",
                    span,
                )]);
            };
            let index = list_index(index, items.len(), span)?;
            items[index] = value;
            Ok(Value::List(items))
        }
        BuiltinFunction::MapEmpty => {
            if args.len() != 0 {
                return Err(vec![Diagnostic::new(
                    "R012",
                    format!("expected 0 arguments but found {}", args.len()),
                    span,
                )]);
            }
            Ok(Value::Map(MapValue {
                entries: Vec::new(),
            }))
        }
        BuiltinFunction::Contains => {
            if args.len() != 2 {
                return Err(vec![Diagnostic::new(
                    "R012",
                    format!("expected 2 arguments but found {}", args.len()),
                    span,
                )]);
            }
            let mut args = args.into_iter();
            let map = args.next().expect("checked length");
            let key = args.next().expect("checked length");
            let Value::Map(map) = map else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`contains` expects Map[K, V] as its first argument",
                    span,
                )]);
            };
            let key = map_key(key, span, "contains")?;
            Ok(Value::Bool(
                map.entries.iter().any(|entry| entry.key == key),
            ))
        }
        BuiltinFunction::Insert => {
            if args.len() != 3 {
                return Err(vec![Diagnostic::new(
                    "R012",
                    format!("expected 3 arguments but found {}", args.len()),
                    span,
                )]);
            }
            let mut args = args.into_iter();
            let map = args.next().expect("checked length");
            let key = args.next().expect("checked length");
            let value = args.next().expect("checked length");
            let Value::Map(mut map) = map else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`insert` expects Map[K, V] as its first argument",
                    span,
                )]);
            };
            let key = map_key(key, span, "insert")?;
            if let Some(entry) = map.entries.iter_mut().find(|entry| entry.key == key) {
                entry.value = value;
            } else {
                map.entries.push(MapEntryValue { key, value });
            }
            Ok(Value::Map(map))
        }
        BuiltinFunction::Remove => {
            if args.len() != 2 {
                return Err(vec![Diagnostic::new(
                    "R012",
                    format!("expected 2 arguments but found {}", args.len()),
                    span,
                )]);
            }
            let mut args = args.into_iter();
            let map = args.next().expect("checked length");
            let key = args.next().expect("checked length");
            let Value::Map(mut map) = map else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`remove` expects Map[K, V] as its first argument",
                    span,
                )]);
            };
            let key = map_key(key, span, "remove")?;
            map.entries.retain(|entry| entry.key != key);
            Ok(Value::Map(map))
        }
        BuiltinFunction::OptionSome => {
            if args.len() != 1 {
                return Err(vec![Diagnostic::new(
                    "R012",
                    format!("expected 1 arguments but found {}", args.len()),
                    span,
                )]);
            }
            let value = args.into_iter().next().expect("checked length");
            Ok(Value::Option(OptionValue::Some(Box::new(value))))
        }
    }
}

fn map_key(value: Value, span: Span, builtin_name: &str) -> Result<MapKey, Vec<Diagnostic>> {
    match value {
        Value::Int(value) => Ok(MapKey::Int(value)),
        Value::Bool(value) => Ok(MapKey::Bool(value)),
        Value::String(value) => Ok(MapKey::String(value)),
        _ => Err(vec![Diagnostic::new(
            "R014",
            format!("`{builtin_name}` expects an Int, Bool, or String Map key"),
            span,
        )]),
    }
}

fn eval_binary(
    op: BinaryOp,
    left: Value,
    right: Value,
    span: Span,
) -> Result<Value, Vec<Diagnostic>> {
    match (op, left, right) {
        (BinaryOp::Add, Value::Int(left), Value::Int(right)) => {
            checked_int(left.checked_add(right), span)
        }
        (BinaryOp::Sub, Value::Int(left), Value::Int(right)) => {
            checked_int(left.checked_sub(right), span)
        }
        (BinaryOp::Mul, Value::Int(left), Value::Int(right)) => {
            checked_int(left.checked_mul(right), span)
        }
        (BinaryOp::Div, Value::Int(_), Value::Int(0)) => {
            Err(vec![Diagnostic::new("R013", "division by zero", span)])
        }
        (BinaryOp::Div, Value::Int(left), Value::Int(right)) => {
            checked_int(left.checked_div(right), span)
        }
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
            span,
        )]),
    }
}

fn checked_int(value: Option<i64>, span: Span) -> Result<Value, Vec<Diagnostic>> {
    value.map(Value::Int).ok_or_else(|| integer_overflow(span))
}

fn integer_overflow(span: Span) -> Vec<Diagnostic> {
    vec![Diagnostic::new("R019", "integer overflow", span)]
}

fn pop_args(
    stack: &mut Vec<Value>,
    argc: usize,
    span: Span,
) -> Result<Vec<Value>, Vec<Diagnostic>> {
    if stack.len() < argc {
        return Err(vec![Diagnostic::new(
            "R015",
            "missing call arguments on stack",
            span,
        )]);
    }
    let mut args = Vec::with_capacity(argc);
    for _ in 0..argc {
        args.push(stack.pop().expect("checked length"));
    }
    args.reverse();
    Ok(args)
}

fn pop_value(
    stack: &mut Vec<Value>,
    span: Span,
    code: &'static str,
    message: &'static str,
) -> Result<Value, Vec<Diagnostic>> {
    stack
        .pop()
        .ok_or_else(|| vec![Diagnostic::new(code, message, span)])
}

fn make_record_value(
    program: &Program,
    type_name: Symbol,
    fields: &[Symbol],
    values: Vec<Value>,
) -> Value {
    Value::Record(RecordValue {
        type_name: symbol_name(program, type_name).to_string(),
        fields: fields
            .iter()
            .zip(values)
            .map(|(field, value)| RecordFieldValue {
                name: symbol_name(program, *field).to_string(),
                value,
            })
            .collect(),
    })
}

fn load_record_field(
    program: &Program,
    base: Value,
    field: Symbol,
    span: Span,
) -> Result<Value, Vec<Diagnostic>> {
    let field_name = symbol_name(program, field);
    let Value::Record(record) = base else {
        return Err(vec![Diagnostic::new(
            "R016",
            "field access requires a record value",
            span,
        )]);
    };
    let Some(field_value) = record
        .fields
        .iter()
        .find(|candidate| candidate.name == field_name)
    else {
        return Err(vec![Diagnostic::new(
            "R017",
            format!("unknown field `{field_name}`"),
            span,
        )]);
    };
    Ok(field_value.value.clone())
}

fn load_list_index(base: Value, index: Value, span: Span) -> Result<Value, Vec<Diagnostic>> {
    let Value::List(items) = base else {
        return Err(vec![Diagnostic::new(
            "R014",
            "list indexing expects List[T] as its base",
            span,
        )]);
    };
    let index = list_index(index, items.len(), span)?;
    Ok(items[index].clone())
}

fn list_index(index: Value, len: usize, span: Span) -> Result<usize, Vec<Diagnostic>> {
    let Value::Int(index) = index else {
        return Err(vec![Diagnostic::new(
            "R014",
            "list index must be Int",
            span,
        )]);
    };
    if index < 0 {
        return Err(list_index_out_of_bounds(span));
    }
    let index = usize::try_from(index).map_err(|_| list_index_out_of_bounds(span))?;
    if index >= len {
        return Err(list_index_out_of_bounds(span));
    }
    Ok(index)
}

fn list_index_out_of_bounds(span: Span) -> Vec<Diagnostic> {
    vec![Diagnostic::new("R020", "list index out of bounds", span)]
}

fn update_record_value(
    program: &Program,
    base: Value,
    fields: &[Symbol],
    values: Vec<Value>,
    span: Span,
) -> Result<Value, Vec<Diagnostic>> {
    let Value::Record(mut record) = base else {
        return Err(vec![Diagnostic::new("R018", "invalid record update", span)]);
    };

    for (field, value) in fields.iter().zip(values) {
        let field_name = symbol_name(program, *field);
        let Some(existing) = record
            .fields
            .iter_mut()
            .find(|candidate| candidate.name == field_name)
        else {
            return Err(vec![Diagnostic::new("R018", "invalid record update", span)]);
        };
        existing.value = value;
    }

    Ok(Value::Record(record))
}

fn install_prelude(program: &Program, env: &EnvRef) {
    if let Some(print_symbol) = program.symbols.lookup("print") {
        env.borrow_mut().bindings.insert(
            print_symbol,
            Binding {
                mutable: false,
                value: Value::Builtin(BuiltinFunction::Print),
                span: Span::default(),
            },
        );
    }
    if let Some(print_symbol) = program.symbols.lookup("println") {
        env.borrow_mut().bindings.insert(
            print_symbol,
            Binding {
                mutable: false,
                value: Value::Builtin(BuiltinFunction::Println),
                span: Span::default(),
            },
        );
    }
    if let Some(len_symbol) = program.symbols.lookup("len") {
        env.borrow_mut().bindings.insert(
            len_symbol,
            Binding {
                mutable: false,
                value: Value::Builtin(BuiltinFunction::Len),
                span: Span::default(),
            },
        );
    }
    if let Some(is_empty_symbol) = program.symbols.lookup("is_empty") {
        env.borrow_mut().bindings.insert(
            is_empty_symbol,
            Binding {
                mutable: false,
                value: Value::Builtin(BuiltinFunction::IsEmpty),
                span: Span::default(),
            },
        );
    }
    if let Some(push_symbol) = program.symbols.lookup("push") {
        env.borrow_mut().bindings.insert(
            push_symbol,
            Binding {
                mutable: false,
                value: Value::Builtin(BuiltinFunction::ListPush),
                span: Span::default(),
            },
        );
    }
    if let Some(get_symbol) = program.symbols.lookup("get") {
        env.borrow_mut().bindings.insert(
            get_symbol,
            Binding {
                mutable: false,
                value: Value::Builtin(BuiltinFunction::Get),
                span: Span::default(),
            },
        );
    }
    if let Some(set_symbol) = program.symbols.lookup("set") {
        env.borrow_mut().bindings.insert(
            set_symbol,
            Binding {
                mutable: false,
                value: Value::Builtin(BuiltinFunction::ListSet),
                span: Span::default(),
            },
        );
    }
    if let Some(map_empty_symbol) = program.symbols.lookup("Map.empty") {
        env.borrow_mut().bindings.insert(
            map_empty_symbol,
            Binding {
                mutable: false,
                value: Value::Builtin(BuiltinFunction::MapEmpty),
                span: Span::default(),
            },
        );
    }
    if let Some(contains_symbol) = program.symbols.lookup("contains") {
        env.borrow_mut().bindings.insert(
            contains_symbol,
            Binding {
                mutable: false,
                value: Value::Builtin(BuiltinFunction::Contains),
                span: Span::default(),
            },
        );
    }
    if let Some(insert_symbol) = program.symbols.lookup("insert") {
        env.borrow_mut().bindings.insert(
            insert_symbol,
            Binding {
                mutable: false,
                value: Value::Builtin(BuiltinFunction::Insert),
                span: Span::default(),
            },
        );
    }
    if let Some(remove_symbol) = program.symbols.lookup("remove") {
        env.borrow_mut().bindings.insert(
            remove_symbol,
            Binding {
                mutable: false,
                value: Value::Builtin(BuiltinFunction::Remove),
                span: Span::default(),
            },
        );
    }
    if let Some(option_some_symbol) = program.symbols.lookup("Option::Some") {
        env.borrow_mut().bindings.insert(
            option_some_symbol,
            Binding {
                mutable: false,
                value: Value::Builtin(BuiltinFunction::OptionSome),
                span: Span::default(),
            },
        );
    }
    if let Some(option_none_symbol) = program.symbols.lookup("Option::None") {
        env.borrow_mut().bindings.insert(
            option_none_symbol,
            Binding {
                mutable: false,
                value: Value::Option(OptionValue::None),
                span: Span::default(),
            },
        );
    }
}

fn child_env(parent: &EnvRef, function_boundary: bool) -> EnvRef {
    Rc::new(RefCell::new(Env::new(
        Some(parent.clone()),
        function_boundary,
        parent.borrow().output.clone(),
    )))
}

fn lookup_any(env: &EnvRef, name: Symbol) -> Option<Binding> {
    let mut current = Some(env.clone());
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if let Some(binding) = borrowed.bindings.get(&name) {
            return Some(binding.clone());
        }
        current = borrowed.parent.clone();
    }
    None
}

fn lookup_any_enclosing(env: &EnvRef, name: Symbol) -> Option<Binding> {
    let mut current = env.borrow().parent.clone();
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if let Some(binding) = borrowed.bindings.get(&name) {
            return Some(binding.clone());
        }
        current = borrowed.parent.clone();
    }
    None
}

fn lookup_in_current_function_env(env: &EnvRef, name: Symbol) -> Option<EnvRef> {
    let mut current = Some(env.clone());
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if borrowed.bindings.contains_key(&name) {
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

fn lookup_beyond_current_function(env: &EnvRef, name: Symbol) -> Option<Binding> {
    let mut first_boundary_seen = false;
    let mut current = Some(env.clone());
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if first_boundary_seen {
            if let Some(binding) = borrowed.bindings.get(&name) {
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
