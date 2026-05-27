use crate::{identity::PackageItemId, prelude::BuiltinId, symbol::Symbol};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeInfo {
    Int,
    Bool,
    String,
    Unit,
    GenericParam(Symbol),
    Record(Symbol, Vec<TypeInfo>),
    PackageRecord {
        symbol: Symbol,
        item: PackageItemId,
        args: Vec<TypeInfo>,
    },
    Enum {
        symbol: Symbol,
        args: Vec<TypeInfo>,
    },
    PackageEnum {
        symbol: Symbol,
        item: PackageItemId,
        args: Vec<TypeInfo>,
    },
    PackageOpaque {
        symbol: Symbol,
        item: PackageItemId,
    },
    List(Box<TypeInfo>),
    Map(Box<TypeInfo>, Box<TypeInfo>),
    Option(Box<TypeInfo>),
    Result(Box<TypeInfo>, Box<TypeInfo>),
    EnumConstructor {
        enum_symbol: Symbol,
        enum_item: Option<PackageItemId>,
        variant: Symbol,
    },
    Function(FunctionTypeInfo),
    Builtin(BuiltinId),
    Unknown,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FunctionTypeInfo {
    pub params: Vec<TypeInfo>,
    pub ret: Box<TypeInfo>,
}
