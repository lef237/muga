use crate::{identity::PackageItemId, symbol::Symbol};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TypeInfo {
    Int,
    Bool,
    String,
    Record(Symbol),
    PackageRecord { symbol: Symbol, item: PackageItemId },
    List(Box<TypeInfo>),
    Map(Box<TypeInfo>, Box<TypeInfo>),
    Option(Box<TypeInfo>),
    Result(Box<TypeInfo>, Box<TypeInfo>),
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
