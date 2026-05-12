#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnownEnum {
    pub name: &'static str,
    pub variants: &'static [KnownEnumVariant],
}

impl KnownEnum {
    pub fn variant(self, name: &str) -> Option<KnownEnumVariant> {
        self.variants
            .iter()
            .copied()
            .find(|variant| variant.name == name)
    }

    pub fn qualified_variant(self, variant: KnownEnumVariant) -> String {
        format!("{}::{}", self.name, variant.name)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KnownEnumVariant {
    pub name: &'static str,
    pub has_payload: bool,
}

pub const OPTION_NAME: &str = "Option";
pub const OPTION_SOME_NAME: &str = "Some";
pub const OPTION_NONE_NAME: &str = "None";
pub const OPTION_SOME_QUALIFIED: &str = "Option::Some";
pub const OPTION_NONE_QUALIFIED: &str = "Option::None";
pub const RESULT_NAME: &str = "Result";
pub const RESULT_OK_NAME: &str = "Ok";
pub const RESULT_ERR_NAME: &str = "Err";
pub const RESULT_OK_QUALIFIED: &str = "Result::Ok";
pub const RESULT_ERR_QUALIFIED: &str = "Result::Err";

static OPTION_VARIANTS: [KnownEnumVariant; 2] = [
    KnownEnumVariant {
        name: OPTION_SOME_NAME,
        has_payload: true,
    },
    KnownEnumVariant {
        name: OPTION_NONE_NAME,
        has_payload: false,
    },
];

static OPTION_ENUM: KnownEnum = KnownEnum {
    name: OPTION_NAME,
    variants: &OPTION_VARIANTS,
};

static RESULT_VARIANTS: [KnownEnumVariant; 2] = [
    KnownEnumVariant {
        name: RESULT_OK_NAME,
        has_payload: true,
    },
    KnownEnumVariant {
        name: RESULT_ERR_NAME,
        has_payload: true,
    },
];

static RESULT_ENUM: KnownEnum = KnownEnum {
    name: RESULT_NAME,
    variants: &RESULT_VARIANTS,
};

pub fn option_enum() -> &'static KnownEnum {
    &OPTION_ENUM
}

pub fn result_enum() -> &'static KnownEnum {
    &RESULT_ENUM
}

pub fn known_enum(name: &str) -> Option<&'static KnownEnum> {
    match name {
        OPTION_NAME => Some(&OPTION_ENUM),
        RESULT_NAME => Some(&RESULT_ENUM),
        _ => None,
    }
}

pub fn known_variant_qualified(name: &str) -> Option<(&'static KnownEnum, KnownEnumVariant)> {
    let (enum_name, variant_name) = name.split_once("::")?;
    let known = known_enum(enum_name)?;
    let variant = known.variant(variant_name)?;
    Some((known, variant))
}
