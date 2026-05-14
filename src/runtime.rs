use std::{cell::RefCell, fmt, rc::Rc};

use crate::{
    bytecode::*,
    diagnostic::Diagnostic,
    identity::LocalId,
    known_enum::{self, KnownEnum, KnownEnumVariant},
    prelude::{self, BuiltinId},
    span::Span,
    symbol::Symbol,
};

type EnvRef = Rc<RefCell<Env>>;

#[derive(Clone, Debug)]
pub enum Value {
    Int(i64),
    Bool(bool),
    String(String),
    Unit,
    List(Vec<Value>),
    Map(MapValue),
    Enum(EnumValue),
    Record(RecordValue),
    Function(Rc<ClosureValue>),
    Builtin(BuiltinId),
}

#[derive(Clone, Debug)]
pub struct RecordValue {
    type_name: String,
    fields: Vec<RecordFieldValue>,
}

#[derive(Clone, Debug)]
pub struct MapValue {
    entries: Vec<MapEntryValue>,
}

#[derive(Clone, Debug)]
pub struct EnumValue {
    type_name: String,
    variant_name: String,
    payload: Option<Box<Value>>,
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

fn enum_value(known: &KnownEnum, variant: KnownEnumVariant, payload: Option<Value>) -> Value {
    Value::Enum(EnumValue {
        type_name: known.name.to_string(),
        variant_name: variant.name.to_string(),
        payload: payload.map(Box::new),
    })
}

fn option_some(value: Value) -> Value {
    let option = known_enum::option_enum();
    let some = option
        .variant(known_enum::OPTION_SOME_NAME)
        .expect("known Option enum should define Some");
    enum_value(option, some, Some(value))
}

fn option_none() -> Value {
    let option = known_enum::option_enum();
    let none = option
        .variant(known_enum::OPTION_NONE_NAME)
        .expect("known Option enum should define None");
    enum_value(option, none, None)
}

fn result_ok(value: Value) -> Value {
    let result = known_enum::result_enum();
    let ok = result
        .variant(known_enum::RESULT_OK_NAME)
        .expect("known Result enum should define Ok");
    enum_value(result, ok, Some(value))
}

fn result_err(value: Value) -> Value {
    let result = known_enum::result_enum();
    let err = result
        .variant(known_enum::RESULT_ERR_NAME)
        .expect("known Result enum should define Err");
    enum_value(result, err, Some(value))
}

fn make_enum_value(
    program: &Program,
    enum_name: Symbol,
    variant_name: Symbol,
    payload: Option<Value>,
) -> Value {
    Value::Enum(EnumValue {
        type_name: program.symbols.resolve(enum_name).to_string(),
        variant_name: program.symbols.resolve(variant_name).to_string(),
        payload: payload.map(Box::new),
    })
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(value) => write!(f, "{value}"),
            Self::Bool(value) => write!(f, "{value}"),
            Self::String(value) => write!(f, "{value}"),
            Self::Unit => write!(f, "()"),
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
            Self::Enum(value) => {
                write!(f, "{}::{}", value.type_name, value.variant_name)?;
                if let Some(payload) = &value.payload {
                    write!(f, "({payload})")?;
                }
                Ok(())
            }
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
            Self::Builtin(builtin) => write!(f, "<builtin:{}>", prelude::builtin_name(*builtin)),
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
    let root = Rc::new(RefCell::new(Env::new(
        None,
        true,
        output.clone(),
        program.local_count,
    )));
    install_prelude(program, &root);
    let _ = execute_chunk(program, &program.entry, root.clone())?;

    match program.main {
        Some(main) => match lookup_any(&root, main.local) {
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

#[derive(Clone, Debug)]
struct Binding {
    mutable: bool,
    value: Value,
    span: Span,
}

#[derive(Debug)]
struct Env {
    bindings: Vec<Option<Binding>>,
    parent: Option<EnvRef>,
    function_boundary: bool,
    output: Rc<RefCell<String>>,
}

impl Env {
    fn new(
        parent: Option<EnvRef>,
        function_boundary: bool,
        output: Rc<RefCell<String>>,
        local_count: usize,
    ) -> Self {
        Self {
            bindings: vec![None; local_count],
            parent,
            function_boundary,
            output,
        }
    }

    fn binding(&self, local: LocalId) -> Option<&Binding> {
        self.bindings
            .get(local.as_u32() as usize)
            .and_then(Option::as_ref)
    }

    fn binding_mut(&mut self, local: LocalId) -> Option<&mut Binding> {
        self.bindings
            .get_mut(local.as_u32() as usize)
            .and_then(Option::as_mut)
    }

    fn contains(&self, local: LocalId) -> bool {
        self.binding(local).is_some()
    }

    fn insert(&mut self, local: LocalId, binding: Binding) {
        let slot = self
            .bindings
            .get_mut(local.as_u32() as usize)
            .expect("bytecode local should be allocated in the program frame");
        *slot = Some(binding);
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
            Instruction::LoadUnit => stack.push(Value::Unit),
            Instruction::MakeRecord {
                type_name,
                fields,
                span,
            } => {
                let values = pop_args(&mut stack, fields.len(), *span)?;
                stack.push(make_record_value(program, *type_name, fields, values));
            }
            Instruction::MakeEnum {
                enum_name,
                variant_name,
                has_payload,
                span,
            } => {
                let payload = if *has_payload {
                    Some(pop_value(
                        &mut stack,
                        *span,
                        "R015",
                        "missing enum variant payload",
                    )?)
                } else {
                    None
                };
                stack.push(make_enum_value(program, *enum_name, *variant_name, payload));
            }
            Instruction::MakeList { len, span } => {
                let values = pop_args(&mut stack, *len, *span)?;
                stack.push(Value::List(values));
            }
            Instruction::LoadName { target, span } => {
                let Some(binding) = lookup_any(&current_env, target.local) else {
                    return Err(vec![Diagnostic::new(
                        "R008",
                        format!(
                            "unresolved runtime name `{}`",
                            symbol_name(program, target.name)
                        ),
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
                target,
                mutable,
                is_update,
                span,
            } => {
                let value = pop_value(&mut stack, *span, "R015", "missing value for assignment")?;
                execute_assign(
                    program,
                    &current_env,
                    *target,
                    *mutable,
                    *is_update,
                    value,
                    *span,
                )?;
            }
            Instruction::DefineFunction {
                target,
                function,
                span,
            } => {
                current_env.borrow_mut().insert(
                    target.local,
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
            Instruction::JumpIfNotEnumVariant {
                enum_name,
                variant_name,
                target,
                span,
            } => {
                let value = pop_value(&mut stack, *span, "R015", "missing enum value for match")?;
                let enum_name = program.symbols.resolve(*enum_name);
                let variant_name = program.symbols.resolve(*variant_name);
                match value {
                    Value::Enum(value)
                        if value.type_name == enum_name && value.variant_name == variant_name =>
                    {
                        if let Some(payload) = value.payload {
                            stack.push(*payload);
                        }
                    }
                    Value::Enum(value) if value.type_name == enum_name => {
                        pc = *target;
                        continue;
                    }
                    Value::Enum(value) => {
                        return Err(vec![Diagnostic::new(
                            "R019",
                            format!(
                                "`match` expected a {enum_name} value but found `{}::{}`",
                                value.type_name, value.variant_name
                            ),
                            *span,
                        )]);
                    }
                    _ => {
                        return Err(vec![Diagnostic::new(
                            "R019",
                            format!("`match` expected a {enum_name} value"),
                            *span,
                        )]);
                    }
                }
            }
            Instruction::MatchExhausted { enum_name, span } => {
                return Err(vec![Diagnostic::new(
                    "R019",
                    format!(
                        "`match` did not cover a {} variant",
                        program.symbols.resolve(*enum_name)
                    ),
                    *span,
                )]);
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
    target: NameRef,
    mutable: bool,
    is_update: bool,
    value: Value,
    span: Span,
) -> Result<(), Vec<Diagnostic>> {
    if is_update {
        return execute_update(program, env, target, value, span);
    }

    if env.borrow().contains(target.local) {
        return Err(vec![Diagnostic::new(
            "R004",
            format!(
                "duplicate binding `{}` in the current scope",
                symbol_name(program, target.name)
            ),
            span,
        )]);
    }

    env.borrow_mut().insert(
        target.local,
        Binding {
            mutable,
            value,
            span,
        },
    );
    Ok(())
}

fn execute_update(
    program: &Program,
    env: &EnvRef,
    target: NameRef,
    value: Value,
    span: Span,
) -> Result<(), Vec<Diagnostic>> {
    if let Some(target_env) = lookup_in_current_function_env(env, target.local) {
        let mut env = target_env.borrow_mut();
        let binding = env.binding_mut(target.local).expect("binding must exist");
        if binding.mutable {
            binding.value = value;
            binding.span = span;
            return Ok(());
        }
        return Err(vec![Diagnostic::new(
            "R006",
            format!(
                "cannot update immutable binding `{}`",
                symbol_name(program, target.name)
            ),
            span,
        )]);
    }

    if let Some(binding) = lookup_beyond_current_function(env, target.local) {
        let code = if binding.mutable { "R007" } else { "R005" };
        let message = if binding.mutable {
            format!(
                "cannot update outer-scope mutable binding `{}` in v1",
                symbol_name(program, target.name)
            )
        } else {
            format!(
                "shadowing is prohibited for `{}`",
                symbol_name(program, target.name)
            )
        };
        return Err(vec![Diagnostic::new(code, message, span)]);
    }

    Err(vec![Diagnostic::new(
        "R008",
        format!(
            "unresolved runtime name `{}`",
            symbol_name(program, target.name)
        ),
        span,
    )])
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
        return Err(arg_count_error(
            definition.params.len(),
            args.len(),
            definition.span,
        ));
    }

    let env = Rc::new(RefCell::new(Env::new(
        Some(function.env.clone()),
        true,
        function.env.borrow().output.clone(),
        program.local_count,
    )));
    for (param, arg) in definition.params.iter().zip(args) {
        env.borrow_mut().insert(
            param.local,
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

fn arg_count_error(expected: usize, actual: usize, span: Span) -> Vec<Diagnostic> {
    vec![Diagnostic::new(
        "R012",
        format!("expected {expected} arguments but found {actual}"),
        span,
    )]
}

fn expect_no_args(args: Vec<Value>, span: Span) -> Result<(), Vec<Diagnostic>> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(arg_count_error(0, args.len(), span))
    }
}

fn expect_one_arg(args: Vec<Value>, span: Span) -> Result<Value, Vec<Diagnostic>> {
    let actual = args.len();
    let mut args = args.into_iter();
    match (args.next(), args.next()) {
        (Some(value), None) => Ok(value),
        _ => Err(arg_count_error(1, actual, span)),
    }
}

fn expect_two_args(args: Vec<Value>, span: Span) -> Result<(Value, Value), Vec<Diagnostic>> {
    let actual = args.len();
    let mut args = args.into_iter();
    match (args.next(), args.next(), args.next()) {
        (Some(first), Some(second), None) => Ok((first, second)),
        _ => Err(arg_count_error(2, actual, span)),
    }
}

fn expect_three_args(
    args: Vec<Value>,
    span: Span,
) -> Result<(Value, Value, Value), Vec<Diagnostic>> {
    let actual = args.len();
    let mut args = args.into_iter();
    match (args.next(), args.next(), args.next(), args.next()) {
        (Some(first), Some(second), Some(third), None) => Ok((first, second, third)),
        _ => Err(arg_count_error(3, actual, span)),
    }
}

fn call_builtin(
    builtin: BuiltinId,
    args: Vec<Value>,
    env: &EnvRef,
    span: Span,
) -> Result<Value, Vec<Diagnostic>> {
    match builtin {
        BuiltinId::Print => {
            let value = expect_one_arg(args, span)?;
            match &value {
                Value::Int(_) | Value::Bool(_) | Value::String(_) => {
                    env.borrow()
                        .output
                        .borrow_mut()
                        .push_str(&value.to_string());
                    Ok(value)
                }
                Value::Unit
                | Value::List(_)
                | Value::Map(_)
                | Value::Enum(_)
                | Value::Record(_)
                | Value::Function(_)
                | Value::Builtin(_) => Err(vec![Diagnostic::new(
                    "R014",
                    "`print` accepts only Int, Bool, or String",
                    span,
                )]),
            }
        }
        BuiltinId::Println => {
            let value = expect_one_arg(args, span)?;
            match &value {
                Value::Int(_) | Value::Bool(_) | Value::String(_) => {
                    let borrowed_env = env.borrow();
                    let mut output = borrowed_env.output.borrow_mut();
                    output.push_str(&value.to_string());
                    output.push('\n');
                    Ok(value)
                }
                Value::Unit
                | Value::List(_)
                | Value::Map(_)
                | Value::Enum(_)
                | Value::Record(_)
                | Value::Function(_)
                | Value::Builtin(_) => Err(vec![Diagnostic::new(
                    "R014",
                    "`println` accepts only Int, Bool, or String",
                    span,
                )]),
            }
        }
        BuiltinId::Len => {
            let value = expect_one_arg(args, span)?;
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
        BuiltinId::IsEmpty => {
            let value = expect_one_arg(args, span)?;
            match value {
                Value::String(text) => Ok(Value::Bool(text.is_empty())),
                Value::List(items) => Ok(Value::Bool(items.is_empty())),
                Value::Map(map) => Ok(Value::Bool(map.entries.is_empty())),
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`is_empty` expects String, List[T], or Map[K, V] as its first argument",
                    span,
                )]),
            }
        }
        BuiltinId::Push => {
            let (list, value) = expect_two_args(args, span)?;
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
        BuiltinId::Get => {
            let (collection, key_or_index) = expect_two_args(args, span)?;
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
                        return Ok(option_none());
                    }
                    match items.get(index as usize).cloned() {
                        Some(value) => Ok(option_some(value)),
                        None => Ok(option_none()),
                    }
                }
                Value::Map(map) => {
                    let key = map_key(key_or_index, span, "get")?;
                    match map.entries.iter().find(|entry| entry.key == key) {
                        Some(entry) => Ok(option_some(entry.value.clone())),
                        None => Ok(option_none()),
                    }
                }
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`get` expects List[T] or Map[K, V] as its first argument",
                    span,
                )]),
            }
        }
        BuiltinId::Set => {
            let (list, index, value) = expect_three_args(args, span)?;
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
        BuiltinId::MapEmpty => {
            expect_no_args(args, span)?;
            Ok(Value::Map(MapValue {
                entries: Vec::new(),
            }))
        }
        BuiltinId::Contains => {
            let (collection, key_or_needle) = expect_two_args(args, span)?;
            match collection {
                Value::String(text) => {
                    let Value::String(needle) = key_or_needle else {
                        return Err(vec![Diagnostic::new(
                            "R014",
                            "`contains` expects String as its second argument for String",
                            span,
                        )]);
                    };
                    Ok(Value::Bool(text.contains(&needle)))
                }
                Value::Map(map) => {
                    let key = map_key(key_or_needle, span, "contains")?;
                    Ok(Value::Bool(
                        map.entries.iter().any(|entry| entry.key == key),
                    ))
                }
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`contains` expects String or Map[K, V] as its first argument",
                    span,
                )]),
            }
        }
        BuiltinId::Insert => {
            let (map, key, value) = expect_three_args(args, span)?;
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
        BuiltinId::Remove => {
            let (map, key) = expect_two_args(args, span)?;
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
        BuiltinId::Trim => {
            let value = expect_one_arg(args, span)?;
            let Value::String(text) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`trim` expects String as its first argument",
                    span,
                )]);
            };
            Ok(Value::String(text.trim().to_string()))
        }
        BuiltinId::CharCount => {
            let value = expect_one_arg(args, span)?;
            let Value::String(text) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`char_count` expects String as its first argument",
                    span,
                )]);
            };
            Ok(Value::Int(text.chars().count() as i64))
        }
        BuiltinId::StartsWith => {
            let (text, prefix) = expect_two_args(args, span)?;
            let Value::String(text) = text else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`starts_with` expects String as its first argument",
                    span,
                )]);
            };
            let Value::String(prefix) = prefix else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`starts_with` expects String as its second argument",
                    span,
                )]);
            };
            Ok(Value::Bool(text.starts_with(&prefix)))
        }
        BuiltinId::EndsWith => {
            let (text, suffix) = expect_two_args(args, span)?;
            let Value::String(text) = text else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`ends_with` expects String as its first argument",
                    span,
                )]);
            };
            let Value::String(suffix) = suffix else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`ends_with` expects String as its second argument",
                    span,
                )]);
            };
            Ok(Value::Bool(text.ends_with(&suffix)))
        }
        BuiltinId::Replace => {
            let (text, old, new) = expect_three_args(args, span)?;
            let Value::String(text) = text else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`replace` expects String as its first argument",
                    span,
                )]);
            };
            let Value::String(old) = old else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`replace` expects String as its second argument",
                    span,
                )]);
            };
            let Value::String(new) = new else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`replace` expects String as its third argument",
                    span,
                )]);
            };
            if old.is_empty() {
                Ok(Value::String(text))
            } else {
                Ok(Value::String(text.replace(&old, &new)))
            }
        }
        BuiltinId::Split => {
            let (text, separator) = expect_two_args(args, span)?;
            let Value::String(text) = text else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`split` expects String as its first argument",
                    span,
                )]);
            };
            let Value::String(separator) = separator else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`split` expects String as its second argument",
                    span,
                )]);
            };
            let parts = if separator.is_empty() {
                vec![Value::String(text)]
            } else {
                text.split(&separator)
                    .map(|part| Value::String(part.to_string()))
                    .collect()
            };
            Ok(Value::List(parts))
        }
        BuiltinId::Concat => {
            let (left, right) = expect_two_args(args, span)?;
            let Value::String(mut left) = left else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`concat` expects String as its first argument",
                    span,
                )]);
            };
            let Value::String(right) = right else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`concat` expects String as its second argument",
                    span,
                )]);
            };
            left.push_str(&right);
            Ok(Value::String(left))
        }
        BuiltinId::SliceChars => {
            let (text, start, count) = expect_three_args(args, span)?;
            let Value::String(text) = text else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`slice_chars` expects String as its first argument",
                    span,
                )]);
            };
            let Value::Int(start) = start else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`slice_chars` expects Int as its second argument",
                    span,
                )]);
            };
            let Value::Int(count) = count else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`slice_chars` expects Int as its third argument",
                    span,
                )]);
            };
            let Some(end) = start.checked_add(count) else {
                return Ok(result_err(Value::String("invalid slice range".to_string())));
            };
            let char_count = text.chars().count() as i64;
            if start < 0 || count < 0 || end > char_count {
                return Ok(result_err(Value::String("invalid slice range".to_string())));
            }
            let slice = text
                .chars()
                .skip(start as usize)
                .take(count as usize)
                .collect();
            Ok(result_ok(Value::String(slice)))
        }
        BuiltinId::ToString => {
            let value = expect_one_arg(args, span)?;
            match value {
                Value::Int(value) => Ok(Value::String(value.to_string())),
                Value::Bool(value) => Ok(Value::String(value.to_string())),
                Value::String(value) => Ok(Value::String(value)),
                _ => Err(vec![Diagnostic::new(
                    "R014",
                    "`to_string` accepts only Int, Bool, or String",
                    span,
                )]),
            }
        }
        BuiltinId::ParseInt => {
            let value = expect_one_arg(args, span)?;
            let Value::String(text) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`parse_int` expects String as its first argument",
                    span,
                )]);
            };
            match text.parse::<i64>() {
                Ok(value) => Ok(result_ok(Value::Int(value))),
                Err(_) => Ok(result_err(Value::String("invalid Int".to_string()))),
            }
        }
        BuiltinId::ParseBool => {
            let value = expect_one_arg(args, span)?;
            let Value::String(text) = value else {
                return Err(vec![Diagnostic::new(
                    "R014",
                    "`parse_bool` expects String as its first argument",
                    span,
                )]);
            };
            match text.as_str() {
                "true" => Ok(result_ok(Value::Bool(true))),
                "false" => Ok(result_ok(Value::Bool(false))),
                _ => Ok(result_err(Value::String("invalid Bool".to_string()))),
            }
        }
        BuiltinId::OptionSome => {
            let value = expect_one_arg(args, span)?;
            Ok(option_some(value))
        }
        BuiltinId::ResultOk => {
            let value = expect_one_arg(args, span)?;
            Ok(result_ok(value))
        }
        BuiltinId::ResultErr => {
            let value = expect_one_arg(args, span)?;
            Ok(result_err(value))
        }
        BuiltinId::OptionNone => Err(vec![Diagnostic::new(
            "R010",
            "attempted to call a non-function value",
            span,
        )]),
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
    for binding in &program.bindings {
        let name = symbol_name(program, binding.name);
        let Some(builtin) = prelude::builtin_by_name(name) else {
            continue;
        };
        let value = if builtin.id == BuiltinId::OptionNone {
            option_none()
        } else {
            Value::Builtin(builtin.id)
        };
        env.borrow_mut().insert(
            binding.local,
            Binding {
                mutable: false,
                value,
                span: Span::default(),
            },
        );
    }
}

fn child_env(parent: &EnvRef, function_boundary: bool) -> EnvRef {
    let local_count = parent.borrow().bindings.len();
    Rc::new(RefCell::new(Env::new(
        Some(parent.clone()),
        function_boundary,
        parent.borrow().output.clone(),
        local_count,
    )))
}

fn lookup_any(env: &EnvRef, local: LocalId) -> Option<Binding> {
    let mut current = Some(env.clone());
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if let Some(found) = borrowed.binding(local) {
            return Some(found.clone());
        }
        current = borrowed.parent.clone();
    }
    None
}

fn lookup_in_current_function_env(env: &EnvRef, local: LocalId) -> Option<EnvRef> {
    let mut current = Some(env.clone());
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if borrowed.contains(local) {
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

fn lookup_beyond_current_function(env: &EnvRef, local: LocalId) -> Option<Binding> {
    let mut first_boundary_seen = false;
    let mut current = Some(env.clone());
    while let Some(candidate) = current {
        let borrowed = candidate.borrow();
        if first_boundary_seen {
            if let Some(found) = borrowed.binding(local) {
                return Some(found.clone());
            }
        }
        if borrowed.function_boundary {
            first_boundary_seen = true;
        }
        current = borrowed.parent.clone();
    }
    None
}
