use crate::known_enum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BuiltinId {
    Print,
    Println,
    Len,
    IsEmpty,
    Push,
    Get,
    Set,
    MapEmpty,
    Contains,
    Insert,
    Remove,
    Trim,
    StartsWith,
    EndsWith,
    OptionSome,
    OptionNone,
    ResultOk,
    ResultErr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinKind {
    Function,
    Value,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Builtin {
    pub id: BuiltinId,
    pub name: &'static str,
    pub kind: BuiltinKind,
}

pub const BUILTINS: &[Builtin] = &[
    Builtin {
        id: BuiltinId::Print,
        name: "print",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Println,
        name: "println",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Len,
        name: "len",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::IsEmpty,
        name: "is_empty",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Push,
        name: "push",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Get,
        name: "get",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Set,
        name: "set",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::MapEmpty,
        name: "Map.empty",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Contains,
        name: "contains",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Insert,
        name: "insert",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Remove,
        name: "remove",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Trim,
        name: "trim",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StartsWith,
        name: "starts_with",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::EndsWith,
        name: "ends_with",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::OptionSome,
        name: known_enum::OPTION_SOME_QUALIFIED,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::OptionNone,
        name: known_enum::OPTION_NONE_QUALIFIED,
        kind: BuiltinKind::Value,
    },
    Builtin {
        id: BuiltinId::ResultOk,
        name: known_enum::RESULT_OK_QUALIFIED,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::ResultErr,
        name: known_enum::RESULT_ERR_QUALIFIED,
        kind: BuiltinKind::Function,
    },
];

pub fn builtins() -> &'static [Builtin] {
    BUILTINS
}

pub fn builtin_name(id: BuiltinId) -> &'static str {
    builtin_by_id(id).name
}

pub fn builtin_debug_label(id: BuiltinId) -> &'static str {
    match id {
        BuiltinId::Print => "Builtin(print)",
        BuiltinId::Println => "Builtin(println)",
        BuiltinId::Len => "Builtin(len)",
        BuiltinId::IsEmpty => "Builtin(is_empty)",
        BuiltinId::Push => "Builtin(push)",
        BuiltinId::Get => "Builtin(get)",
        BuiltinId::Set => "Builtin(set)",
        BuiltinId::MapEmpty => "Builtin(Map.empty)",
        BuiltinId::Contains => "Builtin(contains)",
        BuiltinId::Insert => "Builtin(insert)",
        BuiltinId::Remove => "Builtin(remove)",
        BuiltinId::Trim => "Builtin(trim)",
        BuiltinId::StartsWith => "Builtin(starts_with)",
        BuiltinId::EndsWith => "Builtin(ends_with)",
        BuiltinId::OptionSome => "Builtin(Option::Some)",
        BuiltinId::OptionNone => "Builtin(Option::None)",
        BuiltinId::ResultOk => "Builtin(Result::Ok)",
        BuiltinId::ResultErr => "Builtin(Result::Err)",
    }
}

pub fn builtin_by_id(id: BuiltinId) -> Builtin {
    BUILTINS
        .iter()
        .copied()
        .find(|builtin| builtin.id == id)
        .expect("every builtin id must be present in the catalog")
}

pub fn builtin_by_name(name: &str) -> Option<Builtin> {
    BUILTINS
        .iter()
        .copied()
        .find(|builtin| builtin.name == name)
}

pub fn is_builtin_name(name: &str) -> bool {
    builtin_by_name(name).is_some()
}
