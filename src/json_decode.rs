use crate::{identity::PackageItemId, symbol::Symbol};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonDecodeFieldSchema {
    pub name: Symbol,
    pub wire_name: Option<Symbol>,
    pub aliases: Vec<Symbol>,
    pub validation: Vec<JsonDecodeValidationRule>,
    pub schema: JsonDecodeSchema,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonDecodeValidationRule {
    NonEmpty,
    Min(i64),
    Max(i64),
    MinLen(i64),
    MaxLen(i64),
}

impl JsonDecodeValidationRule {
    pub fn artifact_token(&self) -> String {
        match self {
            Self::NonEmpty => "non_empty".to_string(),
            Self::Min(value) => format!("min={value}"),
            Self::Max(value) => format!("max={value}"),
            Self::MinLen(value) => format!("min_len={value}"),
            Self::MaxLen(value) => format!("max_len={value}"),
        }
    }

    pub fn from_artifact_token(token: &str) -> Result<Self, String> {
        if token == "non_empty" {
            return Ok(Self::NonEmpty);
        }
        let Some((name, value)) = token.split_once('=') else {
            return Err(format!("invalid JSON validation token `{token}`"));
        };
        let value = value
            .parse::<i64>()
            .map_err(|_| format!("invalid JSON validation token `{token}`"))?;
        match name {
            "min" => Ok(Self::Min(value)),
            "max" => Ok(Self::Max(value)),
            "min_len" => Ok(Self::MinLen(value)),
            "max_len" => Ok(Self::MaxLen(value)),
            _ => Err(format!("invalid JSON validation token `{token}`")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonDecodeVariantSchema {
    pub name: Symbol,
    pub wire_name: Option<Symbol>,
    pub aliases: Vec<Symbol>,
    pub payload: Option<JsonDecodeSchema>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonDecodeSchema {
    String,
    Int,
    Bool,
    JsonValue,
    StringList,
    IntList,
    BoolList,
    JsonObjectMap,
    Option(Box<JsonDecodeSchema>),
    List(Box<JsonDecodeSchema>),
    TypedStringMap(Box<JsonDecodeSchema>),
    Record {
        type_name: Symbol,
        package_item: Option<PackageItemId>,
        deny_unknown_fields: bool,
        fields: Vec<JsonDecodeFieldSchema>,
    },
    Enum {
        type_name: Symbol,
        package_item: Option<PackageItemId>,
        variants: Vec<JsonDecodeVariantSchema>,
    },
}

impl JsonDecodeSchema {
    pub fn map_symbols<F>(&self, map: &mut F) -> Self
    where
        F: FnMut(Symbol) -> Symbol,
    {
        match self {
            Self::String => Self::String,
            Self::Int => Self::Int,
            Self::Bool => Self::Bool,
            Self::JsonValue => Self::JsonValue,
            Self::StringList => Self::StringList,
            Self::IntList => Self::IntList,
            Self::BoolList => Self::BoolList,
            Self::JsonObjectMap => Self::JsonObjectMap,
            Self::Option(item) => Self::Option(Box::new(item.map_symbols(map))),
            Self::List(item) => Self::List(Box::new(item.map_symbols(map))),
            Self::TypedStringMap(item) => Self::TypedStringMap(Box::new(item.map_symbols(map))),
            Self::Record {
                type_name,
                package_item,
                deny_unknown_fields,
                fields,
            } => Self::Record {
                type_name: map(*type_name),
                package_item: *package_item,
                deny_unknown_fields: *deny_unknown_fields,
                fields: fields
                    .iter()
                    .map(|field| JsonDecodeFieldSchema {
                        name: map(field.name),
                        wire_name: field.wire_name.map(&mut *map),
                        aliases: field.aliases.iter().map(|alias| map(*alias)).collect(),
                        validation: field.validation.clone(),
                        schema: field.schema.map_symbols(map),
                    })
                    .collect(),
            },
            Self::Enum {
                type_name,
                package_item,
                variants,
            } => Self::Enum {
                type_name: map(*type_name),
                package_item: *package_item,
                variants: variants
                    .iter()
                    .map(|variant| JsonDecodeVariantSchema {
                        name: map(variant.name),
                        wire_name: variant.wire_name.map(&mut *map),
                        aliases: variant.aliases.iter().map(|alias| map(*alias)).collect(),
                        payload: variant
                            .payload
                            .as_ref()
                            .map(|schema| schema.map_symbols(map)),
                    })
                    .collect(),
            },
        }
    }

    pub fn artifact_text(&self) -> String {
        let mut out = Vec::new();
        self.push_artifact_tokens(&mut out);
        out.join(" ")
    }

    fn push_artifact_tokens(&self, out: &mut Vec<String>) {
        match self {
            Self::String => out.push("S".to_string()),
            Self::Int => out.push("I".to_string()),
            Self::Bool => out.push("B".to_string()),
            Self::JsonValue => out.push("V".to_string()),
            Self::StringList => out.push("LS".to_string()),
            Self::IntList => out.push("LI".to_string()),
            Self::BoolList => out.push("LB".to_string()),
            Self::JsonObjectMap => out.push("M".to_string()),
            Self::Option(item) => {
                out.push("O".to_string());
                item.push_artifact_tokens(out);
            }
            Self::List(item) => {
                out.push("L".to_string());
                item.push_artifact_tokens(out);
            }
            Self::TypedStringMap(item) => {
                out.push("MT".to_string());
                item.push_artifact_tokens(out);
            }
            Self::Record {
                type_name,
                deny_unknown_fields,
                fields,
                ..
            } => {
                let has_wire_names = fields.iter().any(|field| field.wire_name.is_some());
                let has_aliases = fields.iter().any(|field| !field.aliases.is_empty());
                let has_validation = fields.iter().any(|field| !field.validation.is_empty());
                if has_validation {
                    out.push("RV".to_string());
                } else if has_aliases {
                    out.push("RG".to_string());
                } else if *deny_unknown_fields {
                    out.push("RF".to_string());
                } else {
                    out.push(if has_wire_names { "RA" } else { "R" }.to_string());
                }
                out.push(type_name.as_u32().to_string());
                if *deny_unknown_fields || has_aliases || has_validation {
                    out.push(if *deny_unknown_fields { "1" } else { "0" }.to_string());
                }
                out.push(fields.len().to_string());
                for field in fields {
                    out.push(field.name.as_u32().to_string());
                    if *deny_unknown_fields || has_wire_names || has_aliases || has_validation {
                        out.push(field.wire_name.unwrap_or(field.name).as_u32().to_string());
                    }
                    if has_aliases || has_validation {
                        out.push(field.aliases.len().to_string());
                        for alias in &field.aliases {
                            out.push(alias.as_u32().to_string());
                        }
                    }
                    if has_validation {
                        out.push(field.validation.len().to_string());
                        for rule in &field.validation {
                            out.push(rule.artifact_token());
                        }
                    }
                    field.schema.push_artifact_tokens(out);
                }
            }
            Self::Enum {
                type_name,
                variants,
                ..
            } => {
                let has_wire_names = variants.iter().any(|variant| variant.wire_name.is_some());
                let has_aliases = variants.iter().any(|variant| !variant.aliases.is_empty());
                out.push(
                    if has_aliases {
                        "EG"
                    } else if has_wire_names {
                        "EA"
                    } else {
                        "E"
                    }
                    .to_string(),
                );
                out.push(type_name.as_u32().to_string());
                out.push(variants.len().to_string());
                for variant in variants {
                    out.push(variant.name.as_u32().to_string());
                    if has_wire_names || has_aliases {
                        out.push(
                            variant
                                .wire_name
                                .unwrap_or(variant.name)
                                .as_u32()
                                .to_string(),
                        );
                    }
                    if has_aliases {
                        out.push(variant.aliases.len().to_string());
                        for alias in &variant.aliases {
                            out.push(alias.as_u32().to_string());
                        }
                    }
                    match &variant.payload {
                        Some(payload) => {
                            out.push("1".to_string());
                            payload.push_artifact_tokens(out);
                        }
                        None => out.push("0".to_string()),
                    }
                }
            }
        }
    }

    pub fn from_artifact_text(text: &str) -> Result<Self, String> {
        let tokens = text.split_whitespace().collect::<Vec<_>>();
        let mut index = 0;
        let schema = Self::parse_artifact_tokens(&tokens, &mut index)?;
        if index != tokens.len() {
            return Err("trailing JSON decoder schema tokens".to_string());
        }
        Ok(schema)
    }

    fn parse_artifact_tokens(tokens: &[&str], index: &mut usize) -> Result<Self, String> {
        let Some(token) = tokens.get(*index).copied() else {
            return Err("missing JSON decoder schema token".to_string());
        };
        *index += 1;
        match token {
            "S" => Ok(Self::String),
            "I" => Ok(Self::Int),
            "B" => Ok(Self::Bool),
            "V" => Ok(Self::JsonValue),
            "LS" => Ok(Self::StringList),
            "LI" => Ok(Self::IntList),
            "LB" => Ok(Self::BoolList),
            "M" => Ok(Self::JsonObjectMap),
            "O" => {
                let item = Self::parse_artifact_tokens(tokens, index)?;
                Ok(Self::Option(Box::new(item)))
            }
            "L" => {
                let item = Self::parse_artifact_tokens(tokens, index)?;
                Ok(Self::List(Box::new(item)))
            }
            "MT" => {
                let item = Self::parse_artifact_tokens(tokens, index)?;
                Ok(Self::TypedStringMap(Box::new(item)))
            }
            "R" | "RA" | "RF" | "RG" | "RV" => {
                let has_wire_names = token == "RA";
                let has_flags = token == "RF";
                let has_aliases = token == "RG";
                let has_validation = token == "RV";
                let type_name = parse_symbol_token(tokens, index, "record type symbol")?;
                let flags = if has_flags || has_aliases || has_validation {
                    parse_u32_token(tokens, index, "record JSON flags")?
                } else {
                    0
                };
                if flags & !1 != 0 {
                    return Err(format!("invalid record JSON flags `{flags}`"));
                }
                let field_count = parse_usize_token(tokens, index, "record field count")?;
                let mut fields = Vec::with_capacity(field_count);
                for field_index in 0..field_count {
                    let name = parse_symbol_token(
                        tokens,
                        index,
                        &format!("record field symbol {field_index}"),
                    )?;
                    let wire_name = if has_wire_names || has_flags || has_aliases || has_validation
                    {
                        Some(parse_symbol_token(
                            tokens,
                            index,
                            &format!("record field wire symbol {field_index}"),
                        )?)
                    } else {
                        None
                    };
                    let alias_count = if has_aliases || has_validation {
                        parse_usize_token(
                            tokens,
                            index,
                            &format!("record field alias count {field_index}"),
                        )?
                    } else {
                        0
                    };
                    let mut aliases = Vec::with_capacity(alias_count);
                    for alias_index in 0..alias_count {
                        aliases.push(parse_symbol_token(
                            tokens,
                            index,
                            &format!("record field alias symbol {field_index}.{alias_index}"),
                        )?);
                    }
                    let validation_count = if has_validation {
                        parse_usize_token(
                            tokens,
                            index,
                            &format!("record field validation count {field_index}"),
                        )?
                    } else {
                        0
                    };
                    let mut validation = Vec::with_capacity(validation_count);
                    for validation_index in 0..validation_count {
                        let token = tokens.get(*index).ok_or_else(|| {
                            format!(
                                "invalid record field validation token {field_index}.{validation_index}"
                            )
                        })?;
                        *index += 1;
                        validation.push(JsonDecodeValidationRule::from_artifact_token(token)?);
                    }
                    let schema = Self::parse_artifact_tokens(tokens, index)?;
                    fields.push(JsonDecodeFieldSchema {
                        name,
                        wire_name,
                        aliases,
                        validation,
                        schema,
                    });
                }
                Ok(Self::Record {
                    type_name,
                    package_item: None,
                    deny_unknown_fields: flags & 1 != 0,
                    fields,
                })
            }
            "E" | "EA" | "EG" => {
                let has_wire_names = token == "EA";
                let has_aliases = token == "EG";
                let type_name = parse_symbol_token(tokens, index, "enum type symbol")?;
                let variant_count = parse_usize_token(tokens, index, "enum variant count")?;
                let mut variants = Vec::with_capacity(variant_count);
                for variant_index in 0..variant_count {
                    let name = parse_symbol_token(
                        tokens,
                        index,
                        &format!("enum variant symbol {variant_index}"),
                    )?;
                    let wire_name = if has_wire_names || has_aliases {
                        Some(parse_symbol_token(
                            tokens,
                            index,
                            &format!("enum variant wire symbol {variant_index}"),
                        )?)
                    } else {
                        None
                    };
                    let alias_count = if has_aliases {
                        parse_usize_token(
                            tokens,
                            index,
                            &format!("enum variant alias count {variant_index}"),
                        )?
                    } else {
                        0
                    };
                    let mut aliases = Vec::with_capacity(alias_count);
                    for alias_index in 0..alias_count {
                        aliases.push(parse_symbol_token(
                            tokens,
                            index,
                            &format!("enum variant alias symbol {variant_index}.{alias_index}"),
                        )?);
                    }
                    let has_payload = parse_bool_token(
                        tokens,
                        index,
                        &format!("enum variant {variant_index} payload flag"),
                    )?;
                    let payload = if has_payload {
                        Some(Self::parse_artifact_tokens(tokens, index)?)
                    } else {
                        None
                    };
                    variants.push(JsonDecodeVariantSchema {
                        name,
                        wire_name,
                        aliases,
                        payload,
                    });
                }
                Ok(Self::Enum {
                    type_name,
                    package_item: None,
                    variants,
                })
            }
            other => Err(format!("invalid JSON decoder schema token `{other}`")),
        }
    }

    pub fn validate_symbols(
        &self,
        symbol_count: usize,
        context: &str,
        diagnostics: &mut Vec<String>,
    ) {
        match self {
            Self::String
            | Self::Int
            | Self::Bool
            | Self::JsonValue
            | Self::StringList
            | Self::IntList
            | Self::BoolList
            | Self::JsonObjectMap => {}
            Self::Option(item) => {
                item.validate_symbols(symbol_count, &format!("{context} option item"), diagnostics);
            }
            Self::List(item) => {
                item.validate_symbols(symbol_count, &format!("{context} list item"), diagnostics);
            }
            Self::TypedStringMap(item) => {
                item.validate_symbols(symbol_count, &format!("{context} map value"), diagnostics);
            }
            Self::Record {
                type_name, fields, ..
            } => {
                if type_name.as_u32() as usize >= symbol_count {
                    diagnostics.push(format!("{context} has invalid decoder record type symbol"));
                }
                for (index, field) in fields.iter().enumerate() {
                    if field.name.as_u32() as usize >= symbol_count {
                        diagnostics.push(format!(
                            "{context} has invalid decoder field symbol {index}"
                        ));
                    }
                    if field
                        .wire_name
                        .is_some_and(|wire_name| wire_name.as_u32() as usize >= symbol_count)
                    {
                        diagnostics.push(format!(
                            "{context} has invalid decoder field wire symbol {index}"
                        ));
                    }
                    for (alias_index, alias) in field.aliases.iter().enumerate() {
                        if alias.as_u32() as usize >= symbol_count {
                            diagnostics.push(format!(
                                "{context} has invalid decoder field alias symbol {index}.{alias_index}"
                            ));
                        }
                    }
                    field.schema.validate_symbols(
                        symbol_count,
                        &format!("{context} decoder field {index}"),
                        diagnostics,
                    );
                }
            }
            Self::Enum {
                type_name,
                variants,
                ..
            } => {
                if type_name.as_u32() as usize >= symbol_count {
                    diagnostics.push(format!("{context} has invalid decoder enum type symbol"));
                }
                for (index, variant) in variants.iter().enumerate() {
                    if variant.name.as_u32() as usize >= symbol_count {
                        diagnostics.push(format!(
                            "{context} has invalid decoder variant symbol {index}"
                        ));
                    }
                    if variant
                        .wire_name
                        .is_some_and(|wire_name| wire_name.as_u32() as usize >= symbol_count)
                    {
                        diagnostics.push(format!(
                            "{context} has invalid decoder variant wire symbol {index}"
                        ));
                    }
                    for (alias_index, alias) in variant.aliases.iter().enumerate() {
                        if alias.as_u32() as usize >= symbol_count {
                            diagnostics.push(format!(
                                "{context} has invalid decoder variant alias symbol {index}.{alias_index}"
                            ));
                        }
                    }
                    if let Some(payload) = &variant.payload {
                        payload.validate_symbols(
                            symbol_count,
                            &format!("{context} decoder variant {index} payload"),
                            diagnostics,
                        );
                    }
                }
            }
        }
    }
}

fn parse_bool_token(tokens: &[&str], index: &mut usize, label: &str) -> Result<bool, String> {
    let Some(token) = tokens.get(*index).copied() else {
        return Err(format!("missing {label}"));
    };
    *index += 1;
    match token {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(format!("invalid {label} `{token}`")),
    }
}

fn parse_symbol_token(tokens: &[&str], index: &mut usize, label: &str) -> Result<Symbol, String> {
    let value = parse_u32_token(tokens, index, label)?;
    Ok(Symbol::new(value))
}

fn parse_u32_token(tokens: &[&str], index: &mut usize, label: &str) -> Result<u32, String> {
    let Some(token) = tokens.get(*index).copied() else {
        return Err(format!("missing {label}"));
    };
    *index += 1;
    token
        .parse::<u32>()
        .map_err(|_| format!("invalid {label} `{token}`"))
}

fn parse_usize_token(tokens: &[&str], index: &mut usize, label: &str) -> Result<usize, String> {
    let Some(token) = tokens.get(*index).copied() else {
        return Err(format!("missing {label}"));
    };
    *index += 1;
    token
        .parse::<usize>()
        .map_err(|_| format!("invalid {label} `{token}`"))
}

#[cfg(test)]
mod tests {
    use super::{
        JsonDecodeFieldSchema, JsonDecodeSchema, JsonDecodeValidationRule, JsonDecodeVariantSchema,
    };
    use crate::symbol::Symbol;

    #[test]
    fn record_artifact_rejects_unknown_json_flag_bits() {
        let error = JsonDecodeSchema::from_artifact_text("RF 1 3 0")
            .expect_err("unknown record JSON flags should be rejected");

        assert_eq!(error, "invalid record JSON flags `3`");
    }

    #[test]
    fn record_alias_artifact_round_trips() {
        let schema = JsonDecodeSchema::Record {
            type_name: Symbol::new(1),
            package_item: None,
            deny_unknown_fields: true,
            fields: vec![JsonDecodeFieldSchema {
                name: Symbol::new(2),
                wire_name: Some(Symbol::new(3)),
                aliases: vec![Symbol::new(4), Symbol::new(5)],
                validation: Vec::new(),
                schema: JsonDecodeSchema::String,
            }],
        };

        let text = schema.artifact_text();
        assert_eq!(text, "RG 1 1 1 2 3 2 4 5 S");
        assert_eq!(JsonDecodeSchema::from_artifact_text(&text).unwrap(), schema);
    }

    #[test]
    fn record_validation_artifact_round_trips() {
        let schema = JsonDecodeSchema::Record {
            type_name: Symbol::new(1),
            package_item: None,
            deny_unknown_fields: true,
            fields: vec![JsonDecodeFieldSchema {
                name: Symbol::new(2),
                wire_name: Some(Symbol::new(3)),
                aliases: vec![Symbol::new(4)],
                validation: vec![
                    JsonDecodeValidationRule::NonEmpty,
                    JsonDecodeValidationRule::MaxLen(8),
                ],
                schema: JsonDecodeSchema::String,
            }],
        };

        let text = schema.artifact_text();
        assert_eq!(text, "RV 1 1 1 2 3 1 4 2 non_empty max_len=8 S");
        assert_eq!(JsonDecodeSchema::from_artifact_text(&text).unwrap(), schema);
    }

    #[test]
    fn enum_alias_artifact_round_trips() {
        let schema = JsonDecodeSchema::Enum {
            type_name: Symbol::new(1),
            package_item: None,
            variants: vec![JsonDecodeVariantSchema {
                name: Symbol::new(2),
                wire_name: Some(Symbol::new(3)),
                aliases: vec![Symbol::new(4)],
                payload: None,
            }],
        };

        let text = schema.artifact_text();
        assert_eq!(text, "EG 1 1 2 3 1 4 0");
        assert_eq!(JsonDecodeSchema::from_artifact_text(&text).unwrap(), schema);
    }

    #[test]
    fn alias_artifacts_reject_malformed_payloads() {
        let record_error = JsonDecodeSchema::from_artifact_text("RG 1 0 1 2 3 S")
            .expect_err("record alias count should be required");
        assert_eq!(record_error, "invalid record field alias count 0 `S`");

        let enum_error = JsonDecodeSchema::from_artifact_text("EG 1 1 2 3 S")
            .expect_err("enum alias count should be required");
        assert_eq!(enum_error, "invalid enum variant alias count 0 `S`");
    }

    #[test]
    fn validation_artifacts_reject_malformed_payloads() {
        let missing_count = JsonDecodeSchema::from_artifact_text("RV 1 0 1 2 2 0 S")
            .expect_err("record validation count should be required");
        assert_eq!(missing_count, "invalid record field validation count 0 `S`");

        let bad_token = JsonDecodeSchema::from_artifact_text("RV 1 0 1 2 2 0 1 min_len=x S")
            .expect_err("record validation token should be valid");
        assert_eq!(bad_token, "invalid JSON validation token `min_len=x`");
    }
}
