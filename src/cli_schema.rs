use crate::{identity::PackageItemId, json_decode::JsonDecodeValidationRule, symbol::Symbol};

const CLI_FIELD_HIDDEN_FLAG: u32 = 1;
const CLI_COMMAND_HIDDEN_FLAG: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliSchema {
    pub type_name: Symbol,
    pub package_item: Option<PackageItemId>,
    pub about: Option<Symbol>,
    pub fields: Vec<CliFieldSchema>,
    pub commands: Vec<CliCommandVariantSchema>,
    pub subcommand: Option<CliSubcommandSchema>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliFieldSchema {
    pub name: Symbol,
    pub option_name: Symbol,
    pub short: Option<Symbol>,
    pub position: Option<u32>,
    pub value_source: Option<CliValueSource>,
    pub aliases: Vec<Symbol>,
    pub help: Option<Symbol>,
    pub hidden: bool,
    pub validation: Vec<JsonDecodeValidationRule>,
    pub value: CliValueSchema,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CliValueSource {
    File,
    Directory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliValueSchema {
    String,
    Int,
    Bool,
    Option(Box<CliValueSchema>),
    StringList,
    IntList,
    BoolList,
    EnumList {
        type_name: Symbol,
        package_item: Option<PackageItemId>,
        variants: Vec<CliEnumVariantSchema>,
    },
    Enum {
        type_name: Symbol,
        package_item: Option<PackageItemId>,
        variants: Vec<CliEnumVariantSchema>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliEnumVariantSchema {
    pub name: Symbol,
    pub tag: Symbol,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliCommandVariantSchema {
    pub variant_name: Symbol,
    pub command_name: Symbol,
    pub aliases: Vec<Symbol>,
    pub about: Option<Symbol>,
    pub hidden: bool,
    pub payload: Box<CliSchema>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CliSubcommandSchema {
    pub field_name: Symbol,
    pub schema: Box<CliSchema>,
}

impl CliSchema {
    pub fn map_symbols<F>(&self, map: &mut F) -> Self
    where
        F: FnMut(Symbol) -> Symbol,
    {
        Self {
            type_name: map(self.type_name),
            package_item: self.package_item,
            about: self.about.map(&mut *map),
            fields: self
                .fields
                .iter()
                .map(|field| CliFieldSchema {
                    name: map(field.name),
                    option_name: map(field.option_name),
                    short: field.short.map(&mut *map),
                    position: field.position,
                    value_source: field.value_source,
                    aliases: field.aliases.iter().map(|alias| map(*alias)).collect(),
                    help: field.help.map(&mut *map),
                    hidden: field.hidden,
                    validation: field.validation.clone(),
                    value: field.value.map_symbols(map),
                })
                .collect(),
            commands: self
                .commands
                .iter()
                .map(|command| CliCommandVariantSchema {
                    variant_name: map(command.variant_name),
                    command_name: map(command.command_name),
                    aliases: command.aliases.iter().map(|alias| map(*alias)).collect(),
                    about: command.about.map(&mut *map),
                    hidden: command.hidden,
                    payload: Box::new(command.payload.map_symbols(map)),
                })
                .collect(),
            subcommand: self
                .subcommand
                .as_ref()
                .map(|subcommand| CliSubcommandSchema {
                    field_name: map(subcommand.field_name),
                    schema: Box::new(subcommand.schema.map_symbols(map)),
                }),
        }
    }

    pub fn artifact_text(&self) -> String {
        self.artifact_tokens().join(" ")
    }

    fn artifact_tokens(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(subcommand) = &self.subcommand {
            out.push("CW".to_string());
            out.push(self.type_name.as_u32().to_string());
            self.push_record_field_artifact_tokens(&mut out);
            out.push(subcommand.field_name.as_u32().to_string());
            let payload = subcommand.schema.artifact_tokens();
            out.push(payload.len().to_string());
            out.extend(payload);
            self.push_record_trailer_artifact_tokens(&mut out);
        } else if self.commands.is_empty() {
            out.push("CR".to_string());
            out.push(self.type_name.as_u32().to_string());
            self.push_record_field_artifact_tokens(&mut out);
            self.push_record_trailer_artifact_tokens(&mut out);
        } else {
            out.push("CC".to_string());
            out.push(self.type_name.as_u32().to_string());
            out.push(self.commands.len().to_string());
            for command in &self.commands {
                out.push(command.variant_name.as_u32().to_string());
                out.push(command.command_name.as_u32().to_string());
                out.push(command.aliases.len().to_string());
                for alias in &command.aliases {
                    out.push(alias.as_u32().to_string());
                }
                let flags = if command.hidden {
                    CLI_COMMAND_HIDDEN_FLAG
                } else {
                    0
                };
                out.push(flags.to_string());
                match command.about {
                    Some(about) => {
                        out.push("1".to_string());
                        out.push(about.as_u32().to_string());
                    }
                    None => out.push("0".to_string()),
                }
                let payload = command.payload.artifact_tokens();
                out.push(payload.len().to_string());
                out.extend(payload);
            }
            if let Some(about) = self.about {
                out.push("CA".to_string());
                out.push("1".to_string());
                out.push(about.as_u32().to_string());
            }
        }
        out
    }

    fn push_record_field_artifact_tokens(&self, out: &mut Vec<String>) {
        out.push(self.fields.len().to_string());
        for field in &self.fields {
            out.push(field.name.as_u32().to_string());
            out.push(field.option_name.as_u32().to_string());
            out.push(field.aliases.len().to_string());
            for alias in &field.aliases {
                out.push(alias.as_u32().to_string());
            }
            let flags = if field.hidden {
                CLI_FIELD_HIDDEN_FLAG
            } else {
                0
            };
            out.push(flags.to_string());
            match field.help {
                Some(help) => {
                    out.push("1".to_string());
                    out.push(help.as_u32().to_string());
                }
                None => out.push("0".to_string()),
            }
            out.push(field.validation.len().to_string());
            for rule in &field.validation {
                out.push(rule.artifact_token());
            }
            field.value.push_artifact_tokens(out);
        }
    }

    fn push_record_trailer_artifact_tokens(&self, out: &mut Vec<String>) {
        if let Some(about) = self.about {
            out.push("CA".to_string());
            out.push("1".to_string());
            out.push(about.as_u32().to_string());
        }
        let short_fields = self
            .fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| field.short.map(|short| (index, short)))
            .collect::<Vec<_>>();
        if !short_fields.is_empty() {
            out.push("CS".to_string());
            out.push(short_fields.len().to_string());
            for (field_index, short) in short_fields {
                out.push(field_index.to_string());
                out.push(short.as_u32().to_string());
            }
        }
        let positional_fields = self
            .fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| field.position.map(|position| (index, position)))
            .collect::<Vec<_>>();
        if !positional_fields.is_empty() {
            out.push("CP".to_string());
            out.push(positional_fields.len().to_string());
            for (field_index, position) in positional_fields {
                out.push(field_index.to_string());
                out.push(position.to_string());
            }
        }
        let value_source_fields = self
            .fields
            .iter()
            .enumerate()
            .filter_map(|(index, field)| {
                field.value_source.map(|value_source| (index, value_source))
            })
            .collect::<Vec<_>>();
        if !value_source_fields.is_empty() {
            out.push("CV".to_string());
            out.push(value_source_fields.len().to_string());
            for (field_index, value_source) in value_source_fields {
                out.push(field_index.to_string());
                out.push(value_source.artifact_token().to_string());
            }
        }
    }

    pub fn from_artifact_text(text: &str) -> Result<Self, String> {
        let tokens = text.split_whitespace().collect::<Vec<_>>();
        let mut index = 0;
        let schema = Self::parse_artifact_tokens(&tokens, &mut index)?;
        if index != tokens.len() {
            return Err("trailing CLI schema tokens".to_string());
        }
        Ok(schema)
    }

    fn parse_artifact_tokens(tokens: &[&str], index: &mut usize) -> Result<Self, String> {
        let token = next_token(tokens, index, "CLI schema token")?;
        match token {
            "CR" => Self::parse_record_artifact_tokens(tokens, index),
            "CC" => Self::parse_command_artifact_tokens(tokens, index),
            "CW" => Self::parse_wrapper_artifact_tokens(tokens, index),
            _ => Err(format!("unknown CLI schema token `{token}`")),
        }
    }

    fn parse_record_artifact_tokens(tokens: &[&str], index: &mut usize) -> Result<Self, String> {
        let type_name = parse_symbol_token(tokens, index, "CLI record type symbol")?;
        let mut fields = Self::parse_record_field_artifact_tokens(tokens, index)?;
        let about = Self::parse_record_trailer_artifact_tokens(tokens, index, &mut fields)?;
        Ok(Self {
            type_name,
            package_item: None,
            about,
            fields,
            commands: Vec::new(),
            subcommand: None,
        })
    }

    fn parse_wrapper_artifact_tokens(tokens: &[&str], index: &mut usize) -> Result<Self, String> {
        let type_name = parse_symbol_token(tokens, index, "CLI wrapper type symbol")?;
        let mut fields = Self::parse_record_field_artifact_tokens(tokens, index)?;
        let field_name = parse_symbol_token(tokens, index, "CLI wrapper command field symbol")?;
        let payload_len = parse_usize_token(tokens, index, "CLI wrapper command payload length")?;
        if tokens.len() < *index + payload_len {
            return Err(format!(
                "invalid CLI wrapper command payload length `{payload_len}`"
            ));
        }
        let payload_tokens = &tokens[*index..*index + payload_len];
        let mut payload_index = 0;
        let payload = Self::parse_artifact_tokens(payload_tokens, &mut payload_index)?;
        if payload_index != payload_tokens.len() {
            return Err("trailing CLI wrapper command payload tokens".to_string());
        }
        if payload.commands.is_empty() || !payload.fields.is_empty() || payload.subcommand.is_some()
        {
            return Err("CLI wrapper command payload must be a command schema".to_string());
        }
        *index += payload_len;
        let about = Self::parse_record_trailer_artifact_tokens(tokens, index, &mut fields)?;
        Ok(Self {
            type_name,
            package_item: None,
            about,
            fields,
            commands: Vec::new(),
            subcommand: Some(CliSubcommandSchema {
                field_name,
                schema: Box::new(payload),
            }),
        })
    }

    fn parse_record_field_artifact_tokens(
        tokens: &[&str],
        index: &mut usize,
    ) -> Result<Vec<CliFieldSchema>, String> {
        let field_count = parse_usize_token(tokens, index, "CLI field count")?;
        let mut fields = Vec::with_capacity(field_count);
        for field_index in 0..field_count {
            let name =
                parse_symbol_token(tokens, index, &format!("CLI field symbol {field_index}"))?;
            let option_name = parse_symbol_token(
                tokens,
                index,
                &format!("CLI field option symbol {field_index}"),
            )?;
            let alias_count = parse_usize_token(
                tokens,
                index,
                &format!("CLI field alias count {field_index}"),
            )?;
            let mut aliases = Vec::with_capacity(alias_count);
            for alias_index in 0..alias_count {
                aliases.push(parse_symbol_token(
                    tokens,
                    index,
                    &format!("CLI field alias symbol {field_index}.{alias_index}"),
                )?);
            }
            let flags = parse_u32_token(tokens, index, &format!("CLI field flags {field_index}"))?;
            if flags & !CLI_FIELD_HIDDEN_FLAG != 0 {
                return Err(format!("invalid CLI field flags `{flags}`"));
            }
            let help = match next_token(
                tokens,
                index,
                &format!("CLI field help marker {field_index}"),
            )? {
                "0" => None,
                "1" => Some(parse_symbol_token(
                    tokens,
                    index,
                    &format!("CLI field help symbol {field_index}"),
                )?),
                other => return Err(format!("invalid CLI field help marker `{other}`")),
            };
            let validation_count = parse_usize_token(
                tokens,
                index,
                &format!("CLI field validation count {field_index}"),
            )?;
            let mut validation = Vec::with_capacity(validation_count);
            for validation_index in 0..validation_count {
                let token = next_token(
                    tokens,
                    index,
                    &format!("CLI field validation token {field_index}.{validation_index}"),
                )?;
                validation.push(JsonDecodeValidationRule::from_artifact_token(token)?);
            }
            let value = CliValueSchema::parse_artifact_tokens(tokens, index)?;
            fields.push(CliFieldSchema {
                name,
                option_name,
                short: None,
                position: None,
                value_source: None,
                aliases,
                help,
                hidden: flags & CLI_FIELD_HIDDEN_FLAG != 0,
                validation,
                value,
            });
        }
        Ok(fields)
    }

    fn parse_record_trailer_artifact_tokens(
        tokens: &[&str],
        index: &mut usize,
        fields: &mut [CliFieldSchema],
    ) -> Result<Option<Symbol>, String> {
        let mut about = None;
        let mut saw_about = false;
        let mut saw_short_trailer = false;
        let mut saw_position_trailer = false;
        let mut saw_value_source_trailer = false;
        while *index < tokens.len() {
            let token = next_token(tokens, index, "CLI schema trailer")?;
            match token {
                "CA" => {
                    if saw_about {
                        return Err("duplicate CLI about trailer".to_string());
                    }
                    saw_about = true;
                    about = match next_token(tokens, index, "CLI about marker")? {
                        "0" => None,
                        "1" => Some(parse_symbol_token(tokens, index, "CLI about symbol")?),
                        other => return Err(format!("invalid CLI about marker `{other}`")),
                    };
                }
                "CS" => {
                    if saw_short_trailer {
                        return Err("duplicate CLI short trailer".to_string());
                    }
                    saw_short_trailer = true;
                    let short_count = parse_usize_token(tokens, index, "CLI short field count")?;
                    for short_index in 0..short_count {
                        let field_index = parse_usize_token(
                            tokens,
                            index,
                            &format!("CLI short field index {short_index}"),
                        )?;
                        let Some(field) = fields.get_mut(field_index) else {
                            return Err(format!("invalid CLI short field index `{field_index}`"));
                        };
                        if field.short.is_some() {
                            return Err(format!(
                                "duplicate CLI short metadata for field {field_index}"
                            ));
                        }
                        field.short = Some(parse_symbol_token(
                            tokens,
                            index,
                            &format!("CLI short symbol {short_index}"),
                        )?);
                    }
                }
                "CP" => {
                    if saw_position_trailer {
                        return Err("duplicate CLI positional trailer".to_string());
                    }
                    saw_position_trailer = true;
                    let position_count =
                        parse_usize_token(tokens, index, "CLI positional field count")?;
                    for position_index in 0..position_count {
                        let field_index = parse_usize_token(
                            tokens,
                            index,
                            &format!("CLI positional field index {position_index}"),
                        )?;
                        let Some(field) = fields.get_mut(field_index) else {
                            return Err(format!(
                                "invalid CLI positional field index `{field_index}`"
                            ));
                        };
                        if field.position.is_some() {
                            return Err(format!(
                                "duplicate CLI positional metadata for field {field_index}"
                            ));
                        }
                        let position = parse_u32_token(
                            tokens,
                            index,
                            &format!("CLI positional index {position_index}"),
                        )?;
                        if position == 0 {
                            return Err("CLI positional indexes must be positive".to_string());
                        }
                        field.position = Some(position);
                    }
                }
                "CV" => {
                    if saw_value_source_trailer {
                        return Err("duplicate CLI value source trailer".to_string());
                    }
                    saw_value_source_trailer = true;
                    let value_source_count =
                        parse_usize_token(tokens, index, "CLI value source field count")?;
                    for value_source_index in 0..value_source_count {
                        let field_index = parse_usize_token(
                            tokens,
                            index,
                            &format!("CLI value source field index {value_source_index}"),
                        )?;
                        let Some(field) = fields.get_mut(field_index) else {
                            return Err(format!(
                                "invalid CLI value source field index `{field_index}`"
                            ));
                        };
                        if field.value_source.is_some() {
                            return Err(format!(
                                "duplicate CLI value source metadata for field {field_index}"
                            ));
                        }
                        let token = next_token(
                            tokens,
                            index,
                            &format!("CLI value source token {value_source_index}"),
                        )?;
                        field.value_source = Some(CliValueSource::from_artifact_token(token)?);
                    }
                }
                other => return Err(format!("unknown CLI schema trailer `{other}`")),
            }
        }
        Ok(about)
    }

    fn parse_command_artifact_tokens(tokens: &[&str], index: &mut usize) -> Result<Self, String> {
        let type_name = parse_symbol_token(tokens, index, "CLI command type symbol")?;
        let command_count = parse_usize_token(tokens, index, "CLI command count")?;
        if command_count == 0 {
            return Err("CLI command schemas must contain at least one command".to_string());
        }
        let mut commands = Vec::with_capacity(command_count);
        for command_index in 0..command_count {
            let variant_name = parse_symbol_token(
                tokens,
                index,
                &format!("CLI command variant symbol {command_index}"),
            )?;
            let command_name = parse_symbol_token(
                tokens,
                index,
                &format!("CLI command name symbol {command_index}"),
            )?;
            let alias_count = parse_usize_token(
                tokens,
                index,
                &format!("CLI command alias count {command_index}"),
            )?;
            let mut aliases = Vec::with_capacity(alias_count);
            for alias_index in 0..alias_count {
                aliases.push(parse_symbol_token(
                    tokens,
                    index,
                    &format!("CLI command alias symbol {command_index}.{alias_index}"),
                )?);
            }
            let flags =
                parse_u32_token(tokens, index, &format!("CLI command flags {command_index}"))?;
            if flags & !CLI_COMMAND_HIDDEN_FLAG != 0 {
                return Err(format!("invalid CLI command flags `{flags}`"));
            }
            let about = match next_token(
                tokens,
                index,
                &format!("CLI command about marker {command_index}"),
            )? {
                "0" => None,
                "1" => Some(parse_symbol_token(
                    tokens,
                    index,
                    &format!("CLI command about symbol {command_index}"),
                )?),
                other => return Err(format!("invalid CLI command about marker `{other}`")),
            };
            let payload_len = parse_usize_token(
                tokens,
                index,
                &format!("CLI command payload length {command_index}"),
            )?;
            if tokens.len() < *index + payload_len {
                return Err(format!(
                    "invalid CLI command payload length `{payload_len}`"
                ));
            }
            let payload_tokens = &tokens[*index..*index + payload_len];
            let mut payload_index = 0;
            let payload = Self::parse_artifact_tokens(payload_tokens, &mut payload_index)?;
            if payload_index != payload_tokens.len() {
                return Err("trailing CLI command payload tokens".to_string());
            }
            *index += payload_len;
            commands.push(CliCommandVariantSchema {
                variant_name,
                command_name,
                aliases,
                about,
                hidden: flags & CLI_COMMAND_HIDDEN_FLAG != 0,
                payload: Box::new(payload),
            });
        }
        let mut about = None;
        let mut saw_about = false;
        while *index < tokens.len() {
            let token = next_token(tokens, index, "CLI command schema trailer")?;
            match token {
                "CA" => {
                    if saw_about {
                        return Err("duplicate CLI about trailer".to_string());
                    }
                    saw_about = true;
                    about = match next_token(tokens, index, "CLI about marker")? {
                        "0" => None,
                        "1" => Some(parse_symbol_token(tokens, index, "CLI about symbol")?),
                        other => return Err(format!("invalid CLI about marker `{other}`")),
                    };
                }
                other => return Err(format!("unknown CLI command schema trailer `{other}`")),
            }
        }
        Ok(Self {
            type_name,
            package_item: None,
            about,
            fields: Vec::new(),
            commands,
            subcommand: None,
        })
    }

    pub fn validate_symbols(
        &self,
        symbol_count: usize,
        context: &str,
        diagnostics: &mut Vec<String>,
    ) {
        let schema_kind = if self.subcommand.is_some() {
            "wrapper"
        } else if self.commands.is_empty() {
            "record"
        } else {
            "command"
        };
        validate_symbol(
            self.type_name,
            symbol_count,
            &format!("{context} {schema_kind}"),
            diagnostics,
        );
        if let Some(about) = self.about {
            validate_symbol(
                about,
                symbol_count,
                &format!("{context} about"),
                diagnostics,
            );
        }
        for (index, command) in self.commands.iter().enumerate() {
            validate_symbol(
                command.variant_name,
                symbol_count,
                &format!("{context} command {index} variant"),
                diagnostics,
            );
            validate_symbol(
                command.command_name,
                symbol_count,
                &format!("{context} command {index} name"),
                diagnostics,
            );
            for (alias_index, alias) in command.aliases.iter().enumerate() {
                validate_symbol(
                    *alias,
                    symbol_count,
                    &format!("{context} command {index} alias {alias_index}"),
                    diagnostics,
                );
            }
            if let Some(about) = command.about {
                validate_symbol(
                    about,
                    symbol_count,
                    &format!("{context} command {index} about"),
                    diagnostics,
                );
            }
            command.payload.validate_symbols(
                symbol_count,
                &format!("{context} command {index} payload"),
                diagnostics,
            );
        }
        for (index, field) in self.fields.iter().enumerate() {
            validate_symbol(
                field.name,
                symbol_count,
                &format!("{context} field {index}"),
                diagnostics,
            );
            validate_symbol(
                field.option_name,
                symbol_count,
                &format!("{context} field {index} option"),
                diagnostics,
            );
            if let Some(short) = field.short {
                validate_symbol(
                    short,
                    symbol_count,
                    &format!("{context} field {index} short"),
                    diagnostics,
                );
            }
            for (alias_index, alias) in field.aliases.iter().enumerate() {
                validate_symbol(
                    *alias,
                    symbol_count,
                    &format!("{context} field {index} alias {alias_index}"),
                    diagnostics,
                );
            }
            if let Some(help) = field.help {
                validate_symbol(
                    help,
                    symbol_count,
                    &format!("{context} field {index} help"),
                    diagnostics,
                );
            }
            field.value.validate_symbols(
                symbol_count,
                &format!("{context} field {index} value"),
                diagnostics,
            );
        }
        if let Some(subcommand) = &self.subcommand {
            validate_symbol(
                subcommand.field_name,
                symbol_count,
                &format!("{context} wrapper subcommand field"),
                diagnostics,
            );
            subcommand.schema.validate_symbols(
                symbol_count,
                &format!("{context} wrapper subcommand"),
                diagnostics,
            );
        }
    }
}

impl CliValueSource {
    pub fn artifact_token(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Directory => "directory",
        }
    }

    pub fn from_artifact_token(token: &str) -> Result<Self, String> {
        match token {
            "file" => Ok(Self::File),
            "directory" => Ok(Self::Directory),
            _ => Err(format!("invalid CLI value source `{token}`")),
        }
    }
}

impl CliValueSchema {
    pub fn map_symbols<F>(&self, map: &mut F) -> Self
    where
        F: FnMut(Symbol) -> Symbol,
    {
        match self {
            Self::String => Self::String,
            Self::Int => Self::Int,
            Self::Bool => Self::Bool,
            Self::Option(item) => Self::Option(Box::new(item.map_symbols(map))),
            Self::StringList => Self::StringList,
            Self::IntList => Self::IntList,
            Self::BoolList => Self::BoolList,
            Self::EnumList {
                type_name,
                package_item,
                variants,
            } => Self::EnumList {
                type_name: map(*type_name),
                package_item: *package_item,
                variants: variants
                    .iter()
                    .map(|variant| CliEnumVariantSchema {
                        name: map(variant.name),
                        tag: map(variant.tag),
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
                    .map(|variant| CliEnumVariantSchema {
                        name: map(variant.name),
                        tag: map(variant.tag),
                    })
                    .collect(),
            },
        }
    }

    fn push_artifact_tokens(&self, out: &mut Vec<String>) {
        match self {
            Self::String => out.push("S".to_string()),
            Self::Int => out.push("I".to_string()),
            Self::Bool => out.push("B".to_string()),
            Self::Option(item) => {
                out.push("O".to_string());
                item.push_artifact_tokens(out);
            }
            Self::StringList => out.push("LS".to_string()),
            Self::IntList => out.push("LI".to_string()),
            Self::BoolList => out.push("LB".to_string()),
            Self::EnumList {
                type_name,
                variants,
                ..
            } => {
                out.push("LE".to_string());
                out.push(type_name.as_u32().to_string());
                out.push(variants.len().to_string());
                for variant in variants {
                    out.push(variant.name.as_u32().to_string());
                    out.push(variant.tag.as_u32().to_string());
                }
            }
            Self::Enum {
                type_name,
                variants,
                ..
            } => {
                out.push("CE".to_string());
                out.push(type_name.as_u32().to_string());
                out.push(variants.len().to_string());
                for variant in variants {
                    out.push(variant.name.as_u32().to_string());
                    out.push(variant.tag.as_u32().to_string());
                }
            }
        }
    }

    fn parse_artifact_tokens(tokens: &[&str], index: &mut usize) -> Result<Self, String> {
        let token = next_token(tokens, index, "CLI value schema token")?;
        match token {
            "S" => Ok(Self::String),
            "I" => Ok(Self::Int),
            "B" => Ok(Self::Bool),
            "O" => {
                let item = Self::parse_artifact_tokens(tokens, index)?;
                Ok(Self::Option(Box::new(item)))
            }
            "LS" => Ok(Self::StringList),
            "LI" => Ok(Self::IntList),
            "LB" => Ok(Self::BoolList),
            "LE" => {
                let type_name = parse_symbol_token(tokens, index, "CLI enum list type symbol")?;
                let variant_count =
                    parse_usize_token(tokens, index, "CLI enum list variant count")?;
                let mut variants = Vec::with_capacity(variant_count);
                for variant_index in 0..variant_count {
                    let name = parse_symbol_token(
                        tokens,
                        index,
                        &format!("CLI enum list variant symbol {variant_index}"),
                    )?;
                    let tag = parse_symbol_token(
                        tokens,
                        index,
                        &format!("CLI enum list variant tag symbol {variant_index}"),
                    )?;
                    variants.push(CliEnumVariantSchema { name, tag });
                }
                Ok(Self::EnumList {
                    type_name,
                    package_item: None,
                    variants,
                })
            }
            "CE" => {
                let type_name = parse_symbol_token(tokens, index, "CLI enum type symbol")?;
                let variant_count = parse_usize_token(tokens, index, "CLI enum variant count")?;
                let mut variants = Vec::with_capacity(variant_count);
                for variant_index in 0..variant_count {
                    let name = parse_symbol_token(
                        tokens,
                        index,
                        &format!("CLI enum variant symbol {variant_index}"),
                    )?;
                    let tag = parse_symbol_token(
                        tokens,
                        index,
                        &format!("CLI enum variant tag symbol {variant_index}"),
                    )?;
                    variants.push(CliEnumVariantSchema { name, tag });
                }
                Ok(Self::Enum {
                    type_name,
                    package_item: None,
                    variants,
                })
            }
            other => Err(format!("unknown CLI value schema token `{other}`")),
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
            | Self::StringList
            | Self::IntList
            | Self::BoolList => {}
            Self::Option(item) => {
                item.validate_symbols(symbol_count, &format!("{context} option item"), diagnostics);
            }
            Self::Enum {
                type_name,
                variants,
                ..
            }
            | Self::EnumList {
                type_name,
                variants,
                ..
            } => {
                validate_symbol(
                    *type_name,
                    symbol_count,
                    &format!("{context} enum"),
                    diagnostics,
                );
                for (index, variant) in variants.iter().enumerate() {
                    validate_symbol(
                        variant.name,
                        symbol_count,
                        &format!("{context} enum variant {index}"),
                        diagnostics,
                    );
                    validate_symbol(
                        variant.tag,
                        symbol_count,
                        &format!("{context} enum variant {index} tag"),
                        diagnostics,
                    );
                }
            }
        }
    }
}

fn next_token<'a>(tokens: &'a [&str], index: &mut usize, label: &str) -> Result<&'a str, String> {
    let Some(token) = tokens.get(*index).copied() else {
        return Err(format!("missing {label}"));
    };
    *index += 1;
    Ok(token)
}

fn parse_symbol_token(tokens: &[&str], index: &mut usize, label: &str) -> Result<Symbol, String> {
    Ok(Symbol::new(parse_u32_token(tokens, index, label)?))
}

fn parse_u32_token(tokens: &[&str], index: &mut usize, label: &str) -> Result<u32, String> {
    next_token(tokens, index, label)?
        .parse::<u32>()
        .map_err(|_| format!("invalid {label}"))
}

fn parse_usize_token(tokens: &[&str], index: &mut usize, label: &str) -> Result<usize, String> {
    next_token(tokens, index, label)?
        .parse::<usize>()
        .map_err(|_| format!("invalid {label}"))
}

fn validate_symbol(
    symbol: Symbol,
    symbol_count: usize,
    context: &str,
    diagnostics: &mut Vec<String>,
) {
    if symbol.as_u32() as usize >= symbol_count {
        diagnostics.push(format!(
            "{context} references symbol {} but program has {symbol_count} symbols",
            symbol.as_u32()
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CliCommandVariantSchema, CliEnumVariantSchema, CliFieldSchema, CliSchema,
        CliSubcommandSchema, CliValueSchema, CliValueSource,
    };
    use crate::symbol::Symbol;

    #[test]
    fn cli_schema_artifact_round_trips() {
        let schema = CliSchema {
            type_name: Symbol::new(1),
            package_item: None,
            about: Some(Symbol::new(9)),
            fields: vec![CliFieldSchema {
                name: Symbol::new(2),
                option_name: Symbol::new(3),
                short: Some(Symbol::new(10)),
                position: Some(1),
                value_source: Some(CliValueSource::File),
                aliases: vec![Symbol::new(4)],
                help: Some(Symbol::new(5)),
                hidden: true,
                validation: Vec::new(),
                value: CliValueSchema::Enum {
                    type_name: Symbol::new(6),
                    package_item: None,
                    variants: vec![CliEnumVariantSchema {
                        name: Symbol::new(7),
                        tag: Symbol::new(8),
                    }],
                },
            }],
            commands: Vec::new(),
            subcommand: None,
        };
        let text = schema.artifact_text();
        assert_eq!(CliSchema::from_artifact_text(&text).unwrap(), schema);
    }

    #[test]
    fn cli_command_schema_artifact_round_trips() {
        let schema = CliSchema {
            type_name: Symbol::new(1),
            package_item: None,
            about: Some(Symbol::new(2)),
            fields: Vec::new(),
            commands: vec![CliCommandVariantSchema {
                variant_name: Symbol::new(3),
                command_name: Symbol::new(4),
                aliases: vec![Symbol::new(5)],
                about: Some(Symbol::new(6)),
                hidden: true,
                payload: Box::new(CliSchema {
                    type_name: Symbol::new(7),
                    package_item: None,
                    about: None,
                    fields: vec![CliFieldSchema {
                        name: Symbol::new(8),
                        option_name: Symbol::new(9),
                        short: None,
                        position: Some(1),
                        value_source: None,
                        aliases: Vec::new(),
                        help: None,
                        hidden: false,
                        validation: Vec::new(),
                        value: CliValueSchema::String,
                    }],
                    commands: Vec::new(),
                    subcommand: None,
                }),
            }],
            subcommand: None,
        };
        let text = schema.artifact_text();
        assert_eq!(CliSchema::from_artifact_text(&text).unwrap(), schema);
    }

    #[test]
    fn cli_wrapper_schema_artifact_round_trips() {
        let schema = CliSchema {
            type_name: Symbol::new(1),
            package_item: None,
            about: Some(Symbol::new(2)),
            fields: vec![CliFieldSchema {
                name: Symbol::new(3),
                option_name: Symbol::new(4),
                short: Some(Symbol::new(5)),
                position: None,
                value_source: Some(CliValueSource::Directory),
                aliases: Vec::new(),
                help: Some(Symbol::new(6)),
                hidden: false,
                validation: Vec::new(),
                value: CliValueSchema::Bool,
            }],
            commands: Vec::new(),
            subcommand: Some(CliSubcommandSchema {
                field_name: Symbol::new(7),
                schema: Box::new(CliSchema {
                    type_name: Symbol::new(8),
                    package_item: None,
                    about: Some(Symbol::new(9)),
                    fields: Vec::new(),
                    commands: vec![CliCommandVariantSchema {
                        variant_name: Symbol::new(10),
                        command_name: Symbol::new(11),
                        aliases: Vec::new(),
                        about: None,
                        hidden: false,
                        payload: Box::new(CliSchema {
                            type_name: Symbol::new(12),
                            package_item: None,
                            about: None,
                            fields: Vec::new(),
                            commands: Vec::new(),
                            subcommand: None,
                        }),
                    }],
                    subcommand: None,
                }),
            }),
        };
        let text = schema.artifact_text();
        assert!(text.starts_with("CW "), "{text}");
        assert_eq!(CliSchema::from_artifact_text(&text).unwrap(), schema);
    }

    #[test]
    fn cli_schema_artifacts_read_old_payloads_without_short_metadata() {
        let schema = CliSchema::from_artifact_text("CR 1 1 2 3 0 0 0 0 S").unwrap();
        assert_eq!(schema.fields[0].short, None);
        assert_eq!(schema.fields[0].position, None);
        assert_eq!(schema.fields[0].value_source, None);
    }

    #[test]
    fn cli_schema_artifacts_reject_unknown_flag_bits() {
        let error = CliSchema::from_artifact_text("CR 1 1 2 3 0 8 0 0 S")
            .expect_err("invalid hidden flag bits should be rejected");
        assert!(error.contains("invalid CLI field flags"));
    }

    #[test]
    fn cli_command_schema_artifacts_reject_unknown_flag_bits() {
        let error = CliSchema::from_artifact_text("CC 1 1 2 3 0 8 0 2 CR 4 0")
            .expect_err("invalid hidden flag bits should be rejected");
        assert!(error.contains("invalid CLI command flags"));
    }

    #[test]
    fn cli_command_schema_artifacts_reject_empty_command_sets() {
        let error = CliSchema::from_artifact_text("CC 1 0")
            .expect_err("empty command schemas should be rejected");
        assert!(error.contains("at least one command"));
    }
}
