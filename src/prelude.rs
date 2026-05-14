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
    CharCount,
    StartsWith,
    EndsWith,
    Replace,
    Split,
    Concat,
    SliceChars,
    ToString,
    ParseInt,
    ParseBool,
    StdFsReadText,
    StdFsWriteText,
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
        id: BuiltinId::CharCount,
        name: "char_count",
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
        id: BuiltinId::Replace,
        name: "replace",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Split,
        name: "split",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::Concat,
        name: "concat",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::SliceChars,
        name: "slice_chars",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::ToString,
        name: "to_string",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::ParseInt,
        name: "parse_int",
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::ParseBool,
        name: "parse_bool",
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

const INTERNAL_BUILTINS: &[Builtin] = &[
    Builtin {
        id: BuiltinId::StdFsReadText,
        name: crate::std_package::FS_READ_TEXT_BUILTIN,
        kind: BuiltinKind::Function,
    },
    Builtin {
        id: BuiltinId::StdFsWriteText,
        name: crate::std_package::FS_WRITE_TEXT_BUILTIN,
        kind: BuiltinKind::Function,
    },
];

pub fn builtins() -> &'static [Builtin] {
    BUILTINS
}

pub(crate) fn internal_builtins() -> &'static [Builtin] {
    INTERNAL_BUILTINS
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
        BuiltinId::CharCount => "Builtin(char_count)",
        BuiltinId::StartsWith => "Builtin(starts_with)",
        BuiltinId::EndsWith => "Builtin(ends_with)",
        BuiltinId::Replace => "Builtin(replace)",
        BuiltinId::Split => "Builtin(split)",
        BuiltinId::Concat => "Builtin(concat)",
        BuiltinId::SliceChars => "Builtin(slice_chars)",
        BuiltinId::ToString => "Builtin(to_string)",
        BuiltinId::ParseInt => "Builtin(parse_int)",
        BuiltinId::ParseBool => "Builtin(parse_bool)",
        BuiltinId::StdFsReadText => "Builtin(__muga_std_fs_read_text)",
        BuiltinId::StdFsWriteText => "Builtin(__muga_std_fs_write_text)",
        BuiltinId::OptionSome => "Builtin(Option::Some)",
        BuiltinId::OptionNone => "Builtin(Option::None)",
        BuiltinId::ResultOk => "Builtin(Result::Ok)",
        BuiltinId::ResultErr => "Builtin(Result::Err)",
    }
}

pub fn builtin_by_id(id: BuiltinId) -> Builtin {
    BUILTINS
        .iter()
        .chain(INTERNAL_BUILTINS.iter())
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

pub(crate) fn builtin_by_any_name(name: &str) -> Option<Builtin> {
    BUILTINS
        .iter()
        .chain(INTERNAL_BUILTINS.iter())
        .copied()
        .find(|builtin| builtin.name == name)
}

pub fn is_builtin_name(name: &str) -> bool {
    builtin_by_name(name).is_some()
}
