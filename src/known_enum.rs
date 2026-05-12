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

pub fn option_enum() -> &'static KnownEnum {
    &OPTION_ENUM
}

pub fn known_enum(name: &str) -> Option<&'static KnownEnum> {
    if name == OPTION_ENUM.name {
        Some(&OPTION_ENUM)
    } else {
        None
    }
}
