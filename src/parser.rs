use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::identity::{ExprId, StmtId};
use crate::span::Span;
use crate::token::{Token, TokenKind};

pub fn parse(tokens: Vec<Token>) -> Result<Program, Vec<Diagnostic>> {
    let mut parser = Parser::new(tokens);
    parser
        .parse_program()
        .map_err(|diagnostic| vec![diagnostic])
}

pub fn parse_inferred_package(
    tokens: Vec<Token>,
    package_path: String,
) -> Result<Program, Vec<Diagnostic>> {
    let mut parser = Parser::new(tokens);
    parser
        .parse_inferred_package_program(package_path)
        .map_err(|diagnostic| vec![diagnostic])
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
    next_expr_id: u32,
    next_stmt_id: u32,
    allow_struct_literal: bool,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            next_expr_id: 0,
            next_stmt_id: 0,
            allow_struct_literal: true,
        }
    }

    fn parse_expr_without_struct_literal(&mut self) -> Result<Expr, Diagnostic> {
        let saved = self.allow_struct_literal;
        self.allow_struct_literal = false;
        let result = self.parse_expr();
        self.allow_struct_literal = saved;
        result
    }

    fn parse_expr_allowing_struct_literal(&mut self) -> Result<Expr, Diagnostic> {
        let saved = self.allow_struct_literal;
        self.allow_struct_literal = true;
        let result = self.parse_expr();
        self.allow_struct_literal = saved;
        result
    }

    fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        let mut package = None;
        let mut imports = Vec::new();
        let mut statements = Vec::new();
        self.skip_newlines();

        if matches!(self.peek_kind(), TokenKind::Package) {
            package = Some(self.parse_package_decl()?);
            self.consume_package_boundary()?;
            self.skip_newlines();

            while matches!(self.peek_kind(), TokenKind::Import) {
                imports.push(self.parse_import_decl()?);
                self.consume_package_boundary()?;
                self.skip_newlines();
            }

            while !self.is_eof() {
                let attributes = self.parse_attributes()?;
                statements.push(self.parse_package_item(attributes)?);
                self.consume_package_boundary()?;
                self.skip_newlines();
            }

            return Ok(Program {
                package,
                imports,
                statements,
            });
        }

        while !self.is_eof() {
            let attributes = self.parse_attributes()?;
            statements.push(self.parse_top_stmt(attributes)?);
            self.consume_statement_boundary()?;
            self.skip_newlines();
        }

        Ok(Program {
            package,
            imports,
            statements,
        })
    }

    fn parse_inferred_package_program(
        &mut self,
        inferred_package_path: String,
    ) -> Result<Program, Diagnostic> {
        let package;
        let mut imports = Vec::new();
        let mut statements = Vec::new();
        self.skip_newlines();

        if matches!(self.peek_kind(), TokenKind::Package) {
            package = self.parse_package_decl()?;
            self.consume_package_boundary()?;
            self.skip_newlines();
        } else {
            package = PackageDecl {
                path: inferred_package_path,
                span: Span::default(),
            };
        }

        while matches!(self.peek_kind(), TokenKind::Import) {
            imports.push(self.parse_import_decl()?);
            self.consume_package_boundary()?;
            self.skip_newlines();
        }

        while !self.is_eof() {
            let attributes = self.parse_attributes()?;
            statements.push(self.parse_package_item(attributes)?);
            self.consume_package_boundary()?;
            self.skip_newlines();
        }

        Ok(Program {
            package: Some(package),
            imports,
            statements,
        })
    }

    fn parse_top_stmt(&mut self, attributes: Vec<Attribute>) -> Result<Stmt, Diagnostic> {
        match self.peek_kind() {
            TokenKind::Fn if matches!(self.peek_kind_n(1), TokenKind::Ident(_)) => self
                .parse_func_decl_with_visibility(Visibility::Private, attributes)
                .map(Stmt::FuncDecl),
            TokenKind::Pub | TokenKind::Pkg => Err(self.package_mode_required_diagnostic(
                "`pub` and `pkg` are only allowed in package mode",
            )),
            TokenKind::Import => {
                Err(self
                    .package_mode_required_diagnostic("`import` is only allowed in package mode"))
            }
            TokenKind::Package => Err(Diagnostic::new(
                "P014",
                "`package` must appear at the start of the file",
                self.current_span(),
            )
            .with_suggestion("move the `package` declaration before imports and declarations")),
            TokenKind::Record => self
                .parse_record_decl_with_visibility(Visibility::Private, attributes)
                .map(Stmt::RecordDecl),
            TokenKind::Enum => self
                .parse_enum_decl_with_visibility(Visibility::Private, attributes)
                .map(Stmt::EnumDecl),
            _ if !attributes.is_empty() => Err(self.attribute_target_diagnostic(&attributes[0])),
            TokenKind::Opaque => Err(self.package_mode_required_diagnostic(
                "`pub opaque type` is only allowed in package mode",
            )),
            _ => self.parse_stmt(),
        }
    }

    fn package_mode_required_diagnostic(&self, message: &'static str) -> Diagnostic {
        Diagnostic::new("P014", message, self.current_span()).with_suggestion(
            "add a `package path::to::name` declaration at the top of the file, or place the file under a project with `muga.toml` so the package path can be inferred",
        )
    }

    fn parse_package_decl(&mut self) -> Result<PackageDecl, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::Package, "expected `package`")?;
        let (path, end) = self.parse_package_path()?;
        Ok(PackageDecl {
            path,
            span: start.merge(end),
        })
    }

    fn parse_import_decl(&mut self) -> Result<ImportDecl, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::Import, "expected `import`")?;
        let (path, end) = self.parse_package_path()?;
        let (alias, end) = if self.matches_simple(&TokenKind::As) {
            let (alias, alias_span) = self.expect_ident()?;
            (alias, end.merge(alias_span))
        } else {
            (
                path.rsplit("::")
                    .next()
                    .expect("package path has segment")
                    .to_string(),
                end,
            )
        };
        Ok(ImportDecl {
            path,
            alias,
            span: start.merge(end),
        })
    }

    fn parse_package_item(&mut self, attributes: Vec<Attribute>) -> Result<Stmt, Diagnostic> {
        let visibility = match self.peek_kind() {
            TokenKind::Pub => {
                self.advance();
                Visibility::Public
            }
            TokenKind::Pkg => {
                self.advance();
                Visibility::Package
            }
            _ => Visibility::Private,
        };
        match self.peek_kind() {
            TokenKind::Fn if matches!(self.peek_kind_n(1), TokenKind::Ident(_)) => self
                .parse_func_decl_with_visibility(visibility, attributes)
                .map(Stmt::FuncDecl),
            TokenKind::Record => self
                .parse_record_decl_with_visibility(visibility, attributes)
                .map(Stmt::RecordDecl),
            TokenKind::Enum => self
                .parse_enum_decl_with_visibility(visibility, attributes)
                .map(Stmt::EnumDecl),
            _ if !attributes.is_empty() => Err(self.attribute_target_diagnostic(&attributes[0])),
            TokenKind::Opaque if visibility == Visibility::Public => self
                .parse_opaque_type_decl_with_visibility(visibility)
                .map(Stmt::OpaqueTypeDecl),
            TokenKind::Opaque => Err(Diagnostic::new(
                "P014",
                "opaque type declarations must be public in this slice",
                self.current_span(),
            )
            .with_suggestion("write `pub opaque type Name`")),
            _ => Err(Diagnostic::new(
                "P014",
                "package mode allows only top-level `record`, `enum`, `fn`, and `pub opaque type` declarations",
                self.current_span(),
            )),
        }
    }

    fn parse_attributes(&mut self) -> Result<Vec<Attribute>, Diagnostic> {
        let mut attributes = Vec::new();
        while matches!(self.peek_kind(), TokenKind::At) {
            let start = self.current_span();
            self.advance();
            let (name, name_span) = self.expect_ident()?;
            let mut span = start.merge(name_span);
            let mut arguments = Vec::new();
            if self.matches_simple(&TokenKind::LParen) {
                if !matches!(self.peek_kind(), TokenKind::RParen) {
                    loop {
                        let arg_start = self.current_span();
                        let (arg_name, arg_name_span) = self.expect_ident()?;
                        let (value, span) = if self.matches_simple(&TokenKind::Colon) {
                            let (value, value_span) = self.parse_attribute_argument_value()?;
                            (
                                Some(value),
                                arg_start.merge(arg_name_span).merge(value_span),
                            )
                        } else {
                            (None, arg_start.merge(arg_name_span))
                        };
                        arguments.push(AttributeArgument {
                            name: arg_name,
                            value,
                            span,
                        });
                        if !self.matches_simple(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                let end = self.expect_simple(TokenKind::RParen, "expected `)` after attribute")?;
                span = span.merge(end);
            }
            match name.as_str() {
                "test" if arguments.is_empty() => {}
                "test" => {
                    return Err(Diagnostic::new(
                        "P014",
                        "attribute `@test` does not take arguments",
                        span,
                    )
                    .with_suggestion("write `@test` directly before a function declaration"));
                }
                "json" => self.validate_json_attribute_arguments(&arguments, span)?,
                "cli" => self.validate_cli_attribute_arguments(&arguments, span)?,
                "validate" => self.validate_validate_attribute_arguments(&arguments, span)?,
                _ => {
                    return Err(Diagnostic::new(
                        "P014",
                        format!("unknown attribute `@{name}`"),
                        span,
                    )
                    .with_suggestion("use a supported attribute or remove it"));
                }
            }
            attributes.push(Attribute {
                name,
                arguments,
                span,
            });
            self.skip_newlines();
        }
        Ok(attributes)
    }

    fn parse_attribute_argument_value(
        &mut self,
    ) -> Result<(AttributeArgumentValue, Span), Diagnostic> {
        let minus_span = if matches!(self.peek_kind(), TokenKind::Minus) {
            Some(self.advance().span)
        } else {
            None
        };
        let value_token = self.advance();
        match value_token.kind {
            TokenKind::String(value) if minus_span.is_none() => {
                Ok((AttributeArgumentValue::String(value), value_token.span))
            }
            TokenKind::Int(text) => {
                let text = if minus_span.is_some() {
                    format!("-{text}")
                } else {
                    text
                };
                let value = text.parse::<i64>().map_err(|_| {
                    Diagnostic::new(
                        "P014",
                        "attribute integer value is outside Int range",
                        minus_span
                            .map(|span| span.merge(value_token.span))
                            .unwrap_or(value_token.span),
                    )
                })?;
                Ok((
                    AttributeArgumentValue::Int(value),
                    minus_span
                        .map(|span| span.merge(value_token.span))
                        .unwrap_or(value_token.span),
                ))
            }
            _ => Err(Diagnostic::new(
                "P014",
                "attribute argument values require string or integer literals",
                minus_span
                    .map(|span| span.merge(value_token.span))
                    .unwrap_or(value_token.span),
            )),
        }
    }

    fn validate_json_attribute_arguments(
        &self,
        arguments: &[AttributeArgument],
        span: Span,
    ) -> Result<(), Diagnostic> {
        if arguments.is_empty() {
            return Err(
                Diagnostic::new("P014", "attribute `@json` requires a supported option", span)
                    .with_suggestion(
                        "write `@json(rename: \"wire_name\")` or `@json(alias: \"legacy_name\")` on a field or variant, or `@json(deny_unknown_fields)` on a record",
                    ),
            );
        }
        if arguments
            .iter()
            .any(|argument| argument.name == "deny_unknown_fields")
        {
            if arguments.len() == 1
                && arguments[0].name == "deny_unknown_fields"
                && arguments[0].value.is_none()
            {
                return Ok(());
            }
            return Err(Diagnostic::new(
                "P014",
                "JSON deny_unknown_fields is a record-level flag",
                arguments
                    .iter()
                    .find(|argument| argument.name == "deny_unknown_fields")
                    .map(|argument| argument.span)
                    .unwrap_or(span),
            )
            .with_suggestion("write `@json(deny_unknown_fields)` by itself on a record"));
        }

        let mut rename_count = 0usize;
        let mut alias_count = 0usize;
        let mut rename_value = None;
        for argument in arguments {
            match argument.name.as_str() {
                "rename" => {
                    rename_count += 1;
                    rename_value = argument
                        .value
                        .as_ref()
                        .and_then(AttributeArgumentValue::as_string);
                }
                "alias" => {
                    alias_count += 1;
                }
                _ => {
                    return Err(
                        Diagnostic::new("P014", "attribute `@json` requires a supported option", argument.span)
                            .with_suggestion(
                                "write `rename: \"wire_name\"` or repeated `alias: \"legacy_name\"` arguments",
                            ),
                    );
                }
            }
            let Some(value) = argument
                .value
                .as_ref()
                .and_then(AttributeArgumentValue::as_string)
            else {
                return Err(Diagnostic::new(
                    "P014",
                    "JSON rename and alias values require string literals",
                    argument.span,
                )
                .with_suggestion("write `@json(rename: \"wire_name\", alias: \"legacy_name\")`"));
            };
            if value.is_empty() || value.chars().any(|ch| matches!(ch, '\n' | '\r' | '\t')) {
                return Err(Diagnostic::new(
                    "P014",
                    "JSON rename and alias values must be non-empty and may not contain tabs or newlines",
                    argument.span,
                ));
            }
        }
        if rename_count > 1 {
            return Err(Diagnostic::new(
                "P014",
                "JSON rename may be specified only once",
                arguments
                    .iter()
                    .find(|argument| argument.name == "rename")
                    .map(|argument| argument.span)
                    .unwrap_or(span),
            ));
        }
        if alias_count > 0 && rename_value == Some("-") {
            return Err(Diagnostic::new(
                "P014",
                "JSON rename `-` may not be combined with aliases",
                arguments
                    .iter()
                    .find(|argument| argument.name == "rename")
                    .map(|argument| argument.span)
                    .unwrap_or(span),
            ));
        }
        Ok(())
    }

    fn validate_cli_attribute_arguments(
        &self,
        arguments: &[AttributeArgument],
        span: Span,
    ) -> Result<(), Diagnostic> {
        if arguments.is_empty() {
            return Err(
                Diagnostic::new("P014", "attribute `@cli` requires a supported option", span)
                    .with_suggestion(
                        "write `@cli(about: \"Command summary\")` on a record, `@cli(name: \"long-option\", short: \"x\", positional: 1, value_source: \"file\", alias: \"old-name\", help: \"Description\", hidden)` on a record field, or `@cli(subcommand)` on a wrapper record field",
                    ),
            );
        }

        let mut name_count = 0usize;
        let mut short_count = 0usize;
        let mut positional_count = 0usize;
        let mut value_source_count = 0usize;
        let mut help_count = 0usize;
        let mut hidden_count = 0usize;
        let mut about_count = 0usize;
        let mut subcommand_count = 0usize;
        for argument in arguments {
            match argument.name.as_str() {
                "name" | "alias" => {
                    if argument.name == "name" {
                        name_count += 1;
                    }
                    let Some(value) = argument
                        .value
                        .as_ref()
                        .and_then(AttributeArgumentValue::as_string)
                    else {
                        return Err(Diagnostic::new(
                            "P014",
                            "CLI option names and aliases require string literals",
                            argument.span,
                        )
                        .with_suggestion(
                            "write `@cli(name: \"long-option\", alias: \"old-name\")`",
                        ));
                    };
                    if !is_cli_long_option_token(value) {
                        return Err(Diagnostic::new(
                            "P014",
                            "CLI option names and aliases must be long-option tokens without leading dashes",
                            argument.span,
                        )
                        .with_suggestion("use letters, digits, `_`, or `-`, starting with a letter"));
                    }
                }
                "short" => {
                    short_count += 1;
                    let Some(value) = argument
                        .value
                        .as_ref()
                        .and_then(AttributeArgumentValue::as_string)
                    else {
                        return Err(Diagnostic::new(
                            "P014",
                            "CLI short option names require string literals",
                            argument.span,
                        )
                        .with_suggestion("write `@cli(short: \"x\")`"));
                    };
                    if !is_cli_short_option_token(value) {
                        return Err(Diagnostic::new(
                            "P014",
                            "CLI short option names must be one ASCII letter without a leading dash",
                            argument.span,
                        )
                        .with_suggestion("write `@cli(short: \"x\")`"));
                    }
                }
                "positional" => {
                    positional_count += 1;
                    let Some(value) = argument
                        .value
                        .as_ref()
                        .and_then(AttributeArgumentValue::as_int)
                    else {
                        return Err(Diagnostic::new(
                            "P014",
                            "CLI positional indexes require integer literals",
                            argument.span,
                        )
                        .with_suggestion("write `@cli(positional: 1)`"));
                    };
                    if value <= 0 || value > i64::from(u32::MAX) {
                        return Err(Diagnostic::new(
                            "P014",
                            "CLI positional indexes must be positive 32-bit integers",
                            argument.span,
                        )
                        .with_suggestion("write `@cli(positional: 1)`"));
                    }
                }
                "value_source" => {
                    value_source_count += 1;
                    let Some(value) = argument
                        .value
                        .as_ref()
                        .and_then(AttributeArgumentValue::as_string)
                    else {
                        return Err(Diagnostic::new(
                            "P014",
                            "CLI value sources require string literals",
                            argument.span,
                        )
                        .with_suggestion(
                            "write `@cli(value_source: \"file\")` or `@cli(value_source: \"directory\")`",
                        ));
                    };
                    if !matches!(value, "file" | "directory") {
                        return Err(Diagnostic::new(
                            "P014",
                            "CLI value sources support only `file` or `directory`",
                            argument.span,
                        )
                        .with_suggestion(
                            "write `@cli(value_source: \"file\")` or `@cli(value_source: \"directory\")`",
                        ));
                    }
                }
                "help" | "about" => {
                    if argument.name == "help" {
                        help_count += 1;
                    } else {
                        about_count += 1;
                    }
                    let Some(value) = argument
                        .value
                        .as_ref()
                        .and_then(AttributeArgumentValue::as_string)
                    else {
                        return Err(Diagnostic::new(
                            "P014",
                            "CLI help and about metadata require string literals",
                            argument.span,
                        )
                        .with_suggestion(
                            "write `@cli(help: \"Description\")` on a field or `@cli(about: \"Command summary\")` on a record",
                        ));
                    };
                    if value.is_empty() || value.chars().any(|ch| matches!(ch, '\n' | '\r' | '\t'))
                    {
                        return Err(Diagnostic::new(
                            "P014",
                            "CLI help and about metadata must be non-empty and may not contain tabs or newlines",
                            argument.span,
                        ));
                    }
                }
                "hidden" => {
                    hidden_count += 1;
                    if argument.value.is_some() {
                        return Err(Diagnostic::new(
                            "P014",
                            "CLI hidden is a flag and does not take a value",
                            argument.span,
                        )
                        .with_suggestion("write `@cli(hidden)`"));
                    }
                }
                "subcommand" => {
                    subcommand_count += 1;
                    if argument.value.is_some() {
                        return Err(Diagnostic::new(
                            "P014",
                            "CLI subcommand is a flag and does not take a value",
                            argument.span,
                        )
                        .with_suggestion("write `@cli(subcommand)`"));
                    }
                }
                _ => {
                    return Err(Diagnostic::new(
                        "P014",
                        "attribute `@cli` requires a supported option",
                        argument.span,
                    )
                    .with_suggestion("write record-level `about`, field-level `name`, `short`, `positional`, `value_source`, repeated `alias`, `help`, `hidden`, or field-level `subcommand`"));
                }
            }
        }
        if name_count > 1 {
            return Err(Diagnostic::new(
                "P014",
                "CLI name may be specified only once",
                arguments
                    .iter()
                    .find(|argument| argument.name == "name")
                    .map(|argument| argument.span)
                    .unwrap_or(span),
            ));
        }
        if short_count > 1 {
            return Err(Diagnostic::new(
                "P014",
                "CLI short option may be specified only once",
                arguments
                    .iter()
                    .find(|argument| argument.name == "short")
                    .map(|argument| argument.span)
                    .unwrap_or(span),
            ));
        }
        if positional_count > 1 {
            return Err(Diagnostic::new(
                "P014",
                "CLI positional may be specified only once",
                arguments
                    .iter()
                    .find(|argument| argument.name == "positional")
                    .map(|argument| argument.span)
                    .unwrap_or(span),
            ));
        }
        if value_source_count > 1 {
            return Err(Diagnostic::new(
                "P014",
                "CLI value source may be specified only once",
                arguments
                    .iter()
                    .find(|argument| argument.name == "value_source")
                    .map(|argument| argument.span)
                    .unwrap_or(span),
            ));
        }
        if help_count > 1 {
            return Err(Diagnostic::new(
                "P014",
                "CLI help may be specified only once",
                arguments
                    .iter()
                    .find(|argument| argument.name == "help")
                    .map(|argument| argument.span)
                    .unwrap_or(span),
            ));
        }
        if hidden_count > 1 {
            return Err(Diagnostic::new(
                "P014",
                "CLI hidden may be specified only once",
                arguments
                    .iter()
                    .find(|argument| argument.name == "hidden")
                    .map(|argument| argument.span)
                    .unwrap_or(span),
            ));
        }
        if about_count > 1 {
            return Err(Diagnostic::new(
                "P014",
                "CLI about may be specified only once",
                arguments
                    .iter()
                    .find(|argument| argument.name == "about")
                    .map(|argument| argument.span)
                    .unwrap_or(span),
            ));
        }
        if subcommand_count > 1 {
            return Err(Diagnostic::new(
                "P014",
                "CLI subcommand may be specified only once",
                arguments
                    .iter()
                    .find(|argument| argument.name == "subcommand")
                    .map(|argument| argument.span)
                    .unwrap_or(span),
            ));
        }
        Ok(())
    }

    fn validate_validate_attribute_arguments(
        &self,
        arguments: &[AttributeArgument],
        span: Span,
    ) -> Result<(), Diagnostic> {
        if arguments.is_empty() {
            return Err(
                Diagnostic::new("P014", "attribute `@validate` requires a supported option", span)
                    .with_suggestion(
                        "write `@validate(non_empty)`, `@validate(min: 1)`, or `@validate(max_len: 255)` on a record field",
                    ),
            );
        }
        for argument in arguments {
            match argument.name.as_str() {
                "non_empty" => {
                    if argument.value.is_some() {
                        return Err(Diagnostic::new(
                            "P014",
                            "validation flag `non_empty` does not take a value",
                            argument.span,
                        )
                        .with_suggestion("write `@validate(non_empty)`"));
                    }
                }
                "min" | "max" | "min_len" | "max_len" => {
                    if argument
                        .value
                        .as_ref()
                        .is_none_or(|value| value.as_int().is_none())
                    {
                        return Err(Diagnostic::new(
                            "P014",
                            "validation bounds require integer literals",
                            argument.span,
                        )
                        .with_suggestion("write `@validate(min: 1, max_len: 255)`"));
                    }
                }
                _ => {
                    return Err(Diagnostic::new(
                        "P014",
                        "attribute `@validate` requires a supported option",
                        argument.span,
                    )
                    .with_suggestion("write `non_empty`, `min`, `max`, `min_len`, or `max_len`"));
                }
            }
        }
        Ok(())
    }

    fn attribute_target_diagnostic(&self, attribute: &Attribute) -> Diagnostic {
        match attribute.name.as_str() {
            "test" => Diagnostic::new(
                "P014",
                "attribute `@test` is allowed only on function declarations",
                attribute.span,
            )
            .with_suggestion("move the attribute directly before a `fn` declaration"),
            "json" => Diagnostic::new(
                "P014",
                "attribute `@json` is allowed only on record declarations, record fields, and enum variants",
                attribute.span,
            )
            .with_suggestion("move the attribute directly before a record declaration, record field, or enum variant"),
            "cli" => Diagnostic::new(
                "P014",
                "attribute `@cli` is allowed only on record declarations, enum declarations, record fields, and enum variants",
                attribute.span,
            )
            .with_suggestion("move the attribute directly before a record declaration, enum declaration, record field, or enum variant"),
            "validate" => Diagnostic::new(
                "P014",
                "attribute `@validate` is allowed only on record fields",
                attribute.span,
            )
            .with_suggestion("move the attribute directly before a record field"),
            _ => Diagnostic::new(
                "P014",
                format!(
                    "attribute `@{}` is not supported on this declaration",
                    attribute.name
                ),
                attribute.span,
            ),
        }
    }

    fn validate_function_attributes(&self, attributes: &[Attribute]) -> Result<(), Diagnostic> {
        for attribute in attributes {
            if attribute.name != "test" {
                return Err(self.attribute_target_diagnostic(attribute));
            }
        }
        Ok(())
    }

    fn validate_record_attributes(&self, attributes: &[Attribute]) -> Result<(), Diagnostic> {
        let mut json_attribute = None;
        let mut cli_attribute = None;
        for attribute in attributes {
            if !matches!(attribute.name.as_str(), "json" | "cli") {
                return Err(self.attribute_target_diagnostic(attribute));
            }
            if attribute.name == "json" {
                if let Some(previous) = json_attribute.replace(attribute.span) {
                    return Err(Diagnostic::new(
                        "P014",
                        "duplicate `@json` attribute",
                        attribute.span,
                    )
                    .with_related("previous `@json` attribute is here", previous));
                }
                if !json_attribute_is_deny_unknown_fields(attribute) {
                    return Err(Diagnostic::new(
                        "P014",
                        "record declarations support only `@json(deny_unknown_fields)`",
                        attribute.span,
                    )
                    .with_suggestion(
                        "move `@json(rename: \"...\", alias: \"...\")` directly before a record field",
                    ));
                }
            } else if attribute.name == "cli" {
                if let Some(previous) = cli_attribute.replace(attribute.span) {
                    return Err(Diagnostic::new(
                        "P014",
                        "duplicate `@cli` attribute",
                        attribute.span,
                    )
                    .with_related("previous `@cli` attribute is here", previous));
                }
                if !cli_attribute_is_record_metadata(attribute) {
                    return Err(Diagnostic::new(
                        "P014",
                        "record declarations support only `@cli(about: \"...\")`",
                        attribute.span,
                    )
                    .with_suggestion(
                        "move `@cli(name: \"...\", short: \"x\", positional: 1, value_source: \"file\", alias: \"...\", help: \"...\", hidden)` directly before a record field",
                    ));
                }
            }
        }
        Ok(())
    }

    fn validate_enum_attributes(&self, attributes: &[Attribute]) -> Result<(), Diagnostic> {
        let mut cli_attribute = None;
        for attribute in attributes {
            if attribute.name != "cli" {
                return Err(self.attribute_target_diagnostic(attribute));
            }
            if let Some(previous) = cli_attribute.replace(attribute.span) {
                return Err(
                    Diagnostic::new("P014", "duplicate `@cli` attribute", attribute.span)
                        .with_related("previous `@cli` attribute is here", previous),
                );
            }
            if !cli_attribute_is_enum_metadata(attribute) {
                return Err(Diagnostic::new(
                    "P014",
                    "enum declarations support only `@cli(about: \"...\")`",
                    attribute.span,
                )
                .with_suggestion(
                    "move `@cli(name: \"...\", alias: \"...\", about: \"...\", hidden)` directly before an enum variant",
                ));
            }
        }
        Ok(())
    }

    fn validate_record_field_attributes(&self, attributes: &[Attribute]) -> Result<(), Diagnostic> {
        let mut json_attribute = None;
        let mut cli_attribute = None;
        let mut validate_attribute = None;
        for attribute in attributes {
            if !matches!(attribute.name.as_str(), "json" | "cli" | "validate") {
                return Err(self.attribute_target_diagnostic(attribute));
            }
            if attribute.name == "json" {
                if let Some(previous) = json_attribute.replace(attribute.span) {
                    return Err(Diagnostic::new(
                        "P014",
                        "duplicate `@json` attribute",
                        attribute.span,
                    )
                    .with_related("previous `@json` attribute is here", previous));
                }
                if !json_attribute_is_json_schema_metadata(attribute) {
                    return Err(Diagnostic::new(
                        "P014",
                        "record fields support only `@json(rename: \"...\", alias: \"...\")`",
                        attribute.span,
                    )
                    .with_suggestion(
                        "move `@json(deny_unknown_fields)` directly before a record declaration",
                    ));
                }
            } else if attribute.name == "cli" {
                if let Some(previous) = cli_attribute.replace(attribute.span) {
                    return Err(Diagnostic::new(
                        "P014",
                        "duplicate `@cli` attribute",
                        attribute.span,
                    )
                    .with_related("previous `@cli` attribute is here", previous));
                }
                if !cli_attribute_is_field_metadata(attribute) {
                    return Err(Diagnostic::new(
                        "P014",
                        "record fields support only `@cli(name: \"...\", short: \"x\", positional: 1, value_source: \"file\", alias: \"...\", help: \"...\", hidden)` or `@cli(subcommand)`",
                        attribute.span,
                    )
                    .with_suggestion(
                        "move `@cli(about: \"...\")` directly before a record declaration",
                    ));
                }
            } else if let Some(previous) = validate_attribute.replace(attribute.span) {
                return Err(Diagnostic::new(
                    "P014",
                    "duplicate `@validate` attribute",
                    attribute.span,
                )
                .with_related("previous `@validate` attribute is here", previous));
            }
        }
        Ok(())
    }

    fn validate_enum_variant_attributes(&self, attributes: &[Attribute]) -> Result<(), Diagnostic> {
        let mut json_attribute = None;
        let mut cli_attribute = None;
        for attribute in attributes {
            if !matches!(attribute.name.as_str(), "json" | "cli") {
                return Err(self.attribute_target_diagnostic(attribute));
            }
            if attribute.name == "json" {
                if let Some(previous) = json_attribute.replace(attribute.span) {
                    return Err(Diagnostic::new(
                        "P014",
                        "duplicate `@json` attribute",
                        attribute.span,
                    )
                    .with_related("previous `@json` attribute is here", previous));
                }
                if !json_attribute_is_json_schema_metadata(attribute) {
                    return Err(Diagnostic::new(
                        "P014",
                        "enum variants support only `@json(rename: \"...\", alias: \"...\")`",
                        attribute.span,
                    )
                    .with_suggestion(
                        "move `@json(deny_unknown_fields)` directly before a record declaration",
                    ));
                }
            } else {
                if let Some(previous) = cli_attribute.replace(attribute.span) {
                    return Err(Diagnostic::new(
                        "P014",
                        "duplicate `@cli` attribute",
                        attribute.span,
                    )
                    .with_related("previous `@cli` attribute is here", previous));
                }
                if !cli_attribute_is_command_variant_metadata(attribute) {
                    return Err(Diagnostic::new(
                        "P014",
                        "enum variants support only `@cli(name: \"...\", alias: \"...\", about: \"...\", hidden)`",
                        attribute.span,
                    )
                    .with_suggestion(
                        "move `@cli(about: \"...\")` directly before an enum declaration",
                    ));
                }
            }
        }
        Ok(())
    }

    fn parse_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        match self.peek_kind() {
            TokenKind::Mut => self.parse_assign_stmt(true).map(Stmt::Assign),
            TokenKind::Record => Err(Diagnostic::new(
                "P010",
                "record declarations are top-level only",
                self.current_span(),
            )),
            TokenKind::Enum => Err(Diagnostic::new(
                "P010",
                "enum declarations are top-level only",
                self.current_span(),
            )),
            TokenKind::Pub | TokenKind::Pkg => Err(Diagnostic::new(
                "P014",
                "`pub` and `pkg` are only allowed for top-level declarations in package mode",
                self.current_span(),
            )
            .with_suggestion(
                "move the `pub` or `pkg` declaration to the top level of a package file",
            )),
            TokenKind::Fn if matches!(self.peek_kind_n(1), TokenKind::Ident(_)) => self
                .parse_func_decl_with_visibility(Visibility::Private, Vec::new())
                .map(Stmt::FuncDecl),
            TokenKind::If => self.parse_if_stmt_or_expr_stmt(),
            TokenKind::While => self.parse_while_stmt().map(Stmt::While),
            TokenKind::For => self.parse_for_stmt().map(Stmt::For),
            TokenKind::Using => self.parse_using_stmt().map(Stmt::Using),
            TokenKind::Break => self.parse_break_stmt().map(Stmt::Break),
            TokenKind::Continue => self.parse_continue_stmt().map(Stmt::Continue),
            TokenKind::Return => self.parse_return_stmt().map(Stmt::Return),
            TokenKind::Ident(_)
                if matches!(self.peek_kind_n(1), TokenKind::Eq | TokenKind::Colon) =>
            {
                self.parse_assign_stmt(false).map(Stmt::Assign)
            }
            _ => self.parse_expr_stmt().map(Stmt::Expr),
        }
    }

    fn parse_assign_stmt(&mut self, mutable: bool) -> Result<AssignStmt, Diagnostic> {
        let start = self.current_span();
        if mutable {
            self.advance();
        }
        let (name, name_span) = self.expect_ident()?;
        let type_name = if self.matches_simple(&TokenKind::Colon) {
            Some(self.parse_type_expr()?.0)
        } else {
            None
        };
        self.expect_simple(
            TokenKind::Eq,
            "expected `=` after binding name or type annotation",
        )?;
        let value = self.parse_expr()?;
        Ok(AssignStmt {
            id: self.stmt_id(),
            mutable,
            name,
            type_name,
            value,
            span: start.merge(name_span).merge(self.previous_span()),
        })
    }

    fn parse_record_decl_with_visibility(
        &mut self,
        visibility: Visibility,
        attributes: Vec<Attribute>,
    ) -> Result<RecordDecl, Diagnostic> {
        let start = self.current_span();
        self.validate_record_attributes(&attributes)?;
        self.expect_simple(TokenKind::Record, "expected `record`")?;
        let (name, _) = self.expect_ident()?;
        let type_params = if self.matches_simple(&TokenKind::LBracket) {
            self.parse_type_param_names()?
        } else {
            Vec::new()
        };
        self.expect_simple(TokenKind::LBrace, "expected `{` after record name")?;
        self.skip_newlines();
        let mut fields = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
            let attributes = self.parse_attributes()?;
            self.validate_record_field_attributes(&attributes)?;
            let field_start = attributes
                .first()
                .map(|attribute| attribute.span)
                .unwrap_or_else(|| self.current_span());
            let (field_name, _) = self.expect_ident()?;
            self.expect_simple(TokenKind::Colon, "expected `:` after field name")?;
            let (type_name, type_span) = self.parse_type_expr()?;
            fields.push(RecordFieldDecl {
                attributes,
                name: field_name,
                type_name,
                span: field_start.merge(type_span),
            });
            if matches!(self.peek_kind(), TokenKind::RBrace) {
                break;
            }
            self.consume_record_boundary()?;
            self.skip_newlines();
        }
        let end = self.expect_simple(TokenKind::RBrace, "expected `}` after record declaration")?;
        Ok(RecordDecl {
            id: self.stmt_id(),
            name,
            package_item: None,
            visibility,
            attributes,
            doc_comments: Vec::new(),
            type_params,
            fields,
            span: start.merge(end),
        })
    }

    fn parse_enum_decl_with_visibility(
        &mut self,
        visibility: Visibility,
        attributes: Vec<Attribute>,
    ) -> Result<EnumDecl, Diagnostic> {
        let start = self.current_span();
        self.validate_enum_attributes(&attributes)?;
        self.expect_simple(TokenKind::Enum, "expected `enum`")?;
        let (name, _) = self.expect_ident()?;
        let type_params = if self.matches_simple(&TokenKind::LBracket) {
            self.parse_type_param_names()?
        } else {
            Vec::new()
        };
        self.expect_simple(TokenKind::LBrace, "expected `{` after enum name")?;
        self.skip_newlines();
        let mut variants = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
            variants.push(self.parse_enum_variant_decl()?);
            if matches!(self.peek_kind(), TokenKind::RBrace) {
                break;
            }
            self.consume_enum_boundary()?;
            self.skip_newlines();
        }
        let end = self.expect_simple(TokenKind::RBrace, "expected `}` after enum declaration")?;
        Ok(EnumDecl {
            id: self.stmt_id(),
            name,
            package_item: None,
            visibility,
            attributes,
            doc_comments: Vec::new(),
            type_params,
            variants,
            span: start.merge(end),
        })
    }

    fn parse_type_param_names(&mut self) -> Result<Vec<String>, Diagnostic> {
        let mut params = Vec::new();
        if matches!(self.peek_kind(), TokenKind::RBracket) {
            return Err(Diagnostic::new(
                "P018",
                "generic declaration requires at least one type parameter",
                self.current_span(),
            ));
        }
        loop {
            let (param, _) = self.expect_ident()?;
            params.push(param);
            if !self.matches_simple(&TokenKind::Comma) {
                break;
            }
        }
        self.expect_simple(TokenKind::RBracket, "expected `]` after type parameters")?;
        Ok(params)
    }

    fn parse_enum_variant_decl(&mut self) -> Result<EnumVariantDecl, Diagnostic> {
        let attributes = self.parse_attributes()?;
        self.validate_enum_variant_attributes(&attributes)?;
        let start = attributes
            .first()
            .map(|attribute| attribute.span)
            .unwrap_or_else(|| self.current_span());
        let (name, name_span) = self.expect_ident()?;
        let mut span = start.merge(name_span);
        let payload = if self.matches_simple(&TokenKind::LParen) {
            let (payload, payload_span) = self.parse_type_expr()?;
            if matches!(self.peek_kind(), TokenKind::Comma) {
                return Err(Diagnostic::new(
                    "P018",
                    "enum variants support at most one payload type in v1",
                    self.current_span(),
                ));
            }
            let end = self.expect_simple(
                TokenKind::RParen,
                "expected `)` after enum variant payload type",
            )?;
            span = span.merge(payload_span).merge(end);
            Some(payload)
        } else {
            None
        };
        Ok(EnumVariantDecl {
            attributes,
            name,
            payload,
            span,
        })
    }

    fn parse_opaque_type_decl_with_visibility(
        &mut self,
        visibility: Visibility,
    ) -> Result<OpaqueTypeDecl, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::Opaque, "expected `opaque`")?;
        self.expect_simple(TokenKind::Type, "expected `type` after `opaque`")?;
        let (name, name_span) = self.expect_ident()?;
        Ok(OpaqueTypeDecl {
            id: self.stmt_id(),
            name,
            package_item: None,
            visibility,
            doc_comments: Vec::new(),
            span: start.merge(name_span),
        })
    }

    fn parse_func_decl_with_visibility(
        &mut self,
        visibility: Visibility,
        attributes: Vec<Attribute>,
    ) -> Result<FuncDecl, Diagnostic> {
        self.validate_function_attributes(&attributes)?;
        let start = self.current_span();
        self.expect_simple(TokenKind::Fn, "expected `fn`")?;
        let (name, _) = self.expect_ident()?;
        let type_params = if self.matches_simple(&TokenKind::LBracket) {
            self.parse_type_param_names()?
        } else {
            Vec::new()
        };
        self.expect_simple(TokenKind::LParen, "expected `(` after function name")?;
        let params = self.parse_params()?;
        self.expect_simple(TokenKind::RParen, "expected `)` after parameters")?;
        let return_type = self.parse_return_type_annotation()?;
        let body = self.parse_value_block()?;
        let span = start.merge(body.span);
        Ok(FuncDecl {
            id: self.stmt_id(),
            name,
            package_item: None,
            visibility,
            attributes,
            doc_comments: Vec::new(),
            type_params,
            params,
            return_type,
            body,
            span,
        })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>, Diagnostic> {
        let mut params = Vec::new();
        if matches!(self.peek_kind(), TokenKind::RParen) {
            return Ok(params);
        }
        loop {
            let start = self.current_span();
            let (name, name_span) = self.expect_ident()?;
            let (type_name, type_span) = if self.matches_simple(&TokenKind::Colon) {
                let (type_name, type_span) = self.parse_type_expr()?;
                (Some(type_name), Some(type_span))
            } else {
                (None, None)
            };
            let end = type_span.unwrap_or(name_span);
            params.push(Param {
                name,
                type_name,
                span: start.merge(end),
            });
            if !self.matches_simple(&TokenKind::Comma) {
                break;
            }
        }
        Ok(params)
    }

    fn parse_type_expr(&mut self) -> Result<(TypeExpr, Span), Diagnostic> {
        let (domain, span) = self.parse_type_domain()?;
        if self.matches_simple(&TokenKind::Arrow) {
            let (ret, ret_span) = self.parse_type_expr()?;
            return Ok((
                TypeExpr::Function(FunctionTypeExpr {
                    params: domain,
                    ret: Box::new(ret),
                }),
                span.merge(ret_span),
            ));
        }

        if domain.len() == 1 {
            return Ok((domain.into_iter().next().expect("checked length"), span));
        }

        if domain.is_empty() {
            return Err(Diagnostic::new(
                "P001",
                "`()` may only appear as the parameter list of a function type",
                span,
            ));
        }

        Err(Diagnostic::new(
            "P001",
            "multiple types in parentheses require `->` to form a function type",
            span,
        ))
    }

    fn parse_type_domain(&mut self) -> Result<(Vec<TypeExpr>, Span), Diagnostic> {
        if self.matches_simple(&TokenKind::LParen) {
            let start = self.previous_span();
            let mut types = Vec::new();
            if !matches!(self.peek_kind(), TokenKind::RParen) {
                loop {
                    let (ty, _) = self.parse_type_expr()?;
                    types.push(ty);
                    if !self.matches_simple(&TokenKind::Comma) {
                        break;
                    }
                }
            }
            let end =
                self.expect_simple(TokenKind::RParen, "expected `)` after type expression")?;
            return Ok((types, start.merge(end)));
        }

        let (ty, span) = self.parse_type_atom()?;
        Ok((vec![ty], span))
    }

    fn parse_type_atom(&mut self) -> Result<(TypeExpr, Span), Diagnostic> {
        let token = self.advance();
        match token.kind {
            TokenKind::Ident(name) => {
                let (name, span) = self.parse_type_name_after_first(name, token.span)?;
                if self.matches_simple(&TokenKind::LBracket) {
                    let (args, end) = self.parse_type_args()?;
                    if matches!(name.as_str(), "Int" | "Bool" | "String" | "Unit") {
                        return Err(Diagnostic::new(
                            "P001",
                            format!("primitive type `{name}` may not have type arguments"),
                            span.merge(end),
                        ));
                    }
                    return Ok((
                        TypeExpr::Generic(GenericTypeExpr { name, args }),
                        span.merge(end),
                    ));
                }
                match name.as_str() {
                    "Int" => Ok((TypeExpr::Int, span)),
                    "Bool" => Ok((TypeExpr::Bool, span)),
                    "String" => Ok((TypeExpr::String, span)),
                    "Unit" => Ok((TypeExpr::Unit, span)),
                    _ => Ok((TypeExpr::Named(name), span)),
                }
            }
            _ => Err(Diagnostic::new(
                "P001",
                "expected a type expression",
                token.span,
            )),
        }
    }

    fn parse_type_args(&mut self) -> Result<(Vec<TypeExpr>, Span), Diagnostic> {
        let start = self.previous_span();
        let mut args = Vec::new();
        if matches!(self.peek_kind(), TokenKind::RBracket) {
            return Err(Diagnostic::new(
                "P001",
                "generic type requires at least one type argument",
                start,
            ));
        }
        loop {
            let (arg, _) = self.parse_type_expr()?;
            args.push(arg);
            if !self.matches_simple(&TokenKind::Comma) {
                break;
            }
        }
        let end = self.expect_simple(TokenKind::RBracket, "expected `]` after type arguments")?;
        Ok((args, start.merge(end)))
    }

    fn parse_if_stmt_or_expr_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::If, "expected `if`")?;
        let condition = self.parse_expr_without_struct_literal()?;
        let then_block = self.parse_block()?;
        if self.matches_simple(&TokenKind::Else) {
            let else_branch = self.parse_else_branch_for_stmt()?;
            let span = start.merge(else_branch.span());
            let then_value_block = self.block_to_value_block(then_block.clone());
            let else_value_block = self.else_branch_into_value_block(else_branch.clone());
            match (then_value_block, else_value_block) {
                (Ok(then_branch), Ok(else_branch)) => {
                    let expr = Expr::If(IfExpr {
                        id: self.expr_id(),
                        condition: Box::new(condition),
                        span,
                        then_branch,
                        else_branch,
                    });
                    Ok(Stmt::Expr(ExprStmt {
                        id: self.stmt_id(),
                        expr,
                        span,
                    }))
                }
                _ => Ok(Stmt::If(IfStmt {
                    id: self.stmt_id(),
                    condition,
                    then_branch: then_block,
                    else_branch: Some(else_branch.into_block()),
                    span,
                })),
            }
        } else {
            let span = start.merge(then_block.span);
            Ok(Stmt::If(IfStmt {
                id: self.stmt_id(),
                condition,
                then_branch: then_block,
                else_branch: None,
                span,
            }))
        }
    }

    fn parse_else_branch_for_stmt(&mut self) -> Result<ElseBranch, Diagnostic> {
        if matches!(self.peek_kind(), TokenKind::If) {
            self.parse_if_stmt_or_expr_stmt()
                .map(|stmt| ElseBranch::If(Box::new(stmt)))
        } else {
            self.parse_block().map(ElseBranch::Block)
        }
    }

    fn parse_while_stmt(&mut self) -> Result<WhileStmt, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::While, "expected `while`")?;
        let condition = self.parse_expr_without_struct_literal()?;
        let body = self.parse_block()?;
        Ok(WhileStmt {
            id: self.stmt_id(),
            condition,
            span: start.merge(body.span),
            body,
        })
    }

    fn parse_for_stmt(&mut self) -> Result<ForStmt, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::For, "expected `for`")?;
        let (item, item_span) = self.expect_ident()?;
        self.expect_simple(TokenKind::In, "expected `in` after loop item")?;
        let iterable = self.parse_expr_without_struct_literal()?;
        let body = self.parse_block()?;
        let span = start.merge(body.span);
        Ok(ForStmt {
            id: self.stmt_id(),
            item,
            item_span,
            iterable,
            body,
            span,
        })
    }

    fn parse_using_stmt(&mut self) -> Result<UsingStmt, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::Using, "expected `using`")?;
        let (name, name_span) = self.expect_ident()?;
        self.expect_simple(TokenKind::Eq, "expected `=` after `using` binding name")?;
        let value = self.parse_expr_without_struct_literal()?;
        let body = self.parse_block()?;
        let span = start.merge(body.span);
        Ok(UsingStmt {
            id: self.stmt_id(),
            name,
            name_span,
            value,
            body,
            span,
        })
    }

    fn parse_break_stmt(&mut self) -> Result<BreakStmt, Diagnostic> {
        let span = self.current_span();
        self.expect_simple(TokenKind::Break, "expected `break`")?;
        Ok(BreakStmt {
            id: self.stmt_id(),
            span,
        })
    }

    fn parse_continue_stmt(&mut self) -> Result<ContinueStmt, Diagnostic> {
        let span = self.current_span();
        self.expect_simple(TokenKind::Continue, "expected `continue`")?;
        Ok(ContinueStmt {
            id: self.stmt_id(),
            span,
        })
    }

    fn parse_return_stmt(&mut self) -> Result<ReturnStmt, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::Return, "expected `return`")?;
        let value = self.parse_expr()?;
        let span = start.merge(value.span());
        Ok(ReturnStmt {
            id: self.stmt_id(),
            value,
            span,
        })
    }

    fn parse_expr_stmt(&mut self) -> Result<ExprStmt, Diagnostic> {
        let expr = self.parse_expr()?;
        let span = expr.span();
        Ok(ExprStmt {
            id: self.stmt_id(),
            expr,
            span,
        })
    }

    fn parse_block(&mut self) -> Result<Block, Diagnostic> {
        let saved = self.allow_struct_literal;
        self.allow_struct_literal = true;
        let result = self.parse_block_inner();
        self.allow_struct_literal = saved;
        result
    }

    fn parse_block_inner(&mut self) -> Result<Block, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::LBrace, "expected `{`")?;
        self.skip_newlines();
        let mut statements = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
            statements.push(self.parse_stmt()?);
            if matches!(self.peek_kind(), TokenKind::RBrace) {
                break;
            }
            self.consume_statement_boundary()?;
            self.skip_newlines();
        }
        let end = self.expect_simple(TokenKind::RBrace, "expected `}` to close block")?;
        Ok(Block {
            statements,
            span: start.merge(end),
        })
    }

    fn parse_value_block(&mut self) -> Result<ValueBlock, Diagnostic> {
        let block = self.parse_block()?;
        self.block_to_value_block(block)
    }

    fn parse_expr(&mut self) -> Result<Expr, Diagnostic> {
        if matches!(self.peek_kind(), TokenKind::If) {
            return self.parse_if_expr();
        }
        if matches!(self.peek_kind(), TokenKind::Match) {
            return self.parse_match_expr();
        }
        self.parse_or()
    }

    fn parse_or(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_and()?;
        while self.matches_simple(&TokenKind::Or) {
            let right = self.parse_and()?;
            let span = expr.span().merge(right.span());
            expr = Expr::Binary(BinaryExpr {
                id: self.expr_id(),
                op: BinaryOp::Or,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            });
        }
        Ok(expr)
    }

    fn parse_and(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_equality()?;
        while self.matches_simple(&TokenKind::And) {
            let right = self.parse_equality()?;
            let span = expr.span().merge(right.span());
            expr = Expr::Binary(BinaryExpr {
                id: self.expr_id(),
                op: BinaryOp::And,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            });
        }
        Ok(expr)
    }

    fn parse_if_expr(&mut self) -> Result<Expr, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::If, "expected `if`")?;
        let condition = self.parse_expr_without_struct_literal()?;
        let then_branch = self.parse_value_block()?;
        self.expect_simple(TokenKind::Else, "expected `else` in `if` expression")?;
        let else_branch = self.parse_else_branch_for_expr()?;
        Ok(Expr::If(IfExpr {
            id: self.expr_id(),
            condition: Box::new(condition),
            span: start.merge(else_branch.span),
            then_branch,
            else_branch,
        }))
    }

    fn parse_else_branch_for_expr(&mut self) -> Result<ValueBlock, Diagnostic> {
        if matches!(self.peek_kind(), TokenKind::If) {
            let expr = self.parse_if_expr()?;
            let span = expr.span();
            Ok(ValueBlock {
                statements: Vec::new(),
                expr: Box::new(expr),
                terminal_return: false,
                span,
            })
        } else {
            self.parse_value_block()
        }
    }

    fn block_to_value_block(&mut self, block: Block) -> Result<ValueBlock, Diagnostic> {
        if block.statements.is_empty() {
            return Err(Diagnostic::new(
                "P007",
                "value block requires a final expression",
                block.span,
            ));
        }

        let mut prefix = Vec::new();
        let mut iter = block.statements.into_iter().peekable();
        while let Some(stmt) = iter.next() {
            if iter.peek().is_none() {
                match stmt {
                    Stmt::Expr(expr_stmt) => {
                        return Ok(ValueBlock {
                            statements: prefix,
                            expr: Box::new(expr_stmt.expr),
                            terminal_return: false,
                            span: block.span,
                        });
                    }
                    Stmt::Return(return_stmt) => {
                        let span = return_stmt.span;
                        prefix.push(Stmt::Return(return_stmt));
                        return Ok(ValueBlock {
                            statements: prefix,
                            expr: Box::new(Expr::Unit(UnitExpr {
                                id: self.expr_id(),
                                span,
                            })),
                            terminal_return: true,
                            span: block.span,
                        });
                    }
                    other => {
                        return Err(Diagnostic::new(
                            "P008",
                            "value block must end with an expression",
                            other.span(),
                        ));
                    }
                }
            }
            if matches!(stmt, Stmt::Expr(_)) {
                return Err(Diagnostic::new(
                    "P009",
                    "only the final item in a value block may be an expression",
                    stmt.span(),
                ));
            }
            prefix.push(stmt);
        }

        Err(Diagnostic::new(
            "P007",
            "value block requires a final expression",
            block.span,
        ))
    }

    fn else_branch_into_value_block(
        &mut self,
        branch: ElseBranch,
    ) -> Result<ValueBlock, Diagnostic> {
        match branch {
            ElseBranch::Block(block) => self.block_to_value_block(block),
            ElseBranch::If(stmt) => match *stmt {
                Stmt::Expr(expr_stmt) => Ok(ValueBlock {
                    statements: Vec::new(),
                    expr: Box::new(expr_stmt.expr),
                    terminal_return: false,
                    span: expr_stmt.span,
                }),
                other => Err(Diagnostic::new(
                    "P008",
                    "value block must end with an expression",
                    other.span(),
                )),
            },
        }
    }

    fn parse_match_expr(&mut self) -> Result<Expr, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::Match, "expected `match`")?;
        let value = self.parse_expr_without_struct_literal()?;
        self.expect_simple(TokenKind::LBrace, "expected `{` after match value")?;
        self.skip_newlines();
        let mut arms = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
            let arm_start = self.current_span();
            let pattern = self.parse_match_pattern()?;
            self.expect_simple(TokenKind::FatArrow, "expected `=>` after match pattern")?;
            let arm_value = self.parse_expr_allowing_struct_literal()?;
            let arm_span = arm_start.merge(arm_value.span());
            arms.push(MatchArm {
                pattern,
                value: arm_value,
                span: arm_span,
            });
            if matches!(self.peek_kind(), TokenKind::RBrace) {
                break;
            }
            self.consume_match_arm_boundary()?;
            self.skip_newlines();
        }
        let end = self.expect_simple(TokenKind::RBrace, "expected `}` after match arms")?;
        Ok(Expr::Match(MatchExpr {
            id: self.expr_id(),
            value: Box::new(value),
            arms,
            span: start.merge(end),
        }))
    }

    fn parse_match_pattern(&mut self) -> Result<MatchPattern, Diagnostic> {
        let (first, first_span) = self.expect_ident()?;
        let (name, name_span) = self.parse_value_name_after_first(first, first_span)?;
        let Some((enum_name, variant_name)) = split_variant_name(&name) else {
            return Err(Diagnostic::new(
                "P016",
                "expected an enum variant pattern such as `Option::Some(name)`, `Option::None`, or `Result::Ok(value)`",
                name_span,
            ));
        };

        let (payload, span) = if self.matches_simple(&TokenKind::LParen) {
            let (binding, binding_span) = self.expect_ident()?;
            let end = self.expect_simple(TokenKind::RParen, "expected `)` after match binding")?;
            let payload = if binding == "_" {
                EnumVariantPatternPayload::Discard
            } else {
                EnumVariantPatternPayload::Binding(binding)
            };
            (payload, name_span.merge(binding_span).merge(end))
        } else {
            (EnumVariantPatternPayload::None, name_span)
        };

        Ok(MatchPattern::Variant(EnumVariantPattern {
            enum_name,
            variant_name,
            payload,
            span,
        }))
    }

    fn parse_equality(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_comparison()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::EqEq => BinaryOp::EqEq,
                TokenKind::BangEq => BinaryOp::BangEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_comparison()?;
            let span = expr.span().merge(right.span());
            expr = Expr::Binary(BinaryExpr {
                id: self.expr_id(),
                op,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            });
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_additive()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Lt => BinaryOp::Lt,
                TokenKind::LtEq => BinaryOp::LtEq,
                TokenKind::Gt => BinaryOp::Gt,
                TokenKind::GtEq => BinaryOp::GtEq,
                _ => break,
            };
            self.advance();
            let right = self.parse_additive()?;
            let span = expr.span().merge(right.span());
            expr = Expr::Binary(BinaryExpr {
                id: self.expr_id(),
                op,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            });
        }
        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_multiplicative()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Plus => BinaryOp::Add,
                TokenKind::Minus => BinaryOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_multiplicative()?;
            let span = expr.span().merge(right.span());
            expr = Expr::Binary(BinaryExpr {
                id: self.expr_id(),
                op,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            });
        }
        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_unary()?;
        loop {
            let op = match self.peek_kind() {
                TokenKind::Star => BinaryOp::Mul,
                TokenKind::Slash => BinaryOp::Div,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            let span = expr.span().merge(right.span());
            expr = Expr::Binary(BinaryExpr {
                id: self.expr_id(),
                op,
                left: Box::new(expr),
                right: Box::new(right),
                span,
            });
        }
        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr, Diagnostic> {
        match self.peek_kind() {
            TokenKind::Minus => {
                let start = self.current_span();
                self.advance();
                if matches!(self.peek_kind(), TokenKind::Int(_))
                    && !matches!(self.peek_kind_n(1), TokenKind::Dot | TokenKind::LParen)
                {
                    let token = self.advance();
                    let TokenKind::Int(text) = token.kind else {
                        unreachable!("matched Int kind above");
                    };
                    let span = start.merge(token.span);
                    let value = format!("-{text}")
                        .parse::<i64>()
                        .map_err(|_| Diagnostic::new("P002", "invalid integer literal", span))?;
                    return Ok(Expr::Int(IntExpr {
                        id: self.expr_id(),
                        value,
                        span,
                    }));
                }
                let expr = self.parse_unary()?;
                Ok(Expr::Unary(UnaryExpr {
                    id: self.expr_id(),
                    op: UnaryOp::Neg,
                    span: start.merge(expr.span()),
                    expr: Box::new(expr),
                }))
            }
            TokenKind::Bang => {
                let start = self.current_span();
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Unary(UnaryExpr {
                    id: self.expr_id(),
                    op: UnaryOp::Not,
                    span: start.merge(expr.span()),
                    expr: Box::new(expr),
                }))
            }
            TokenKind::Try => {
                let start = self.current_span();
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expr::Try(TryExpr {
                    id: self.expr_id(),
                    span: start.merge(expr.span()),
                    expr: Box::new(expr),
                }))
            }
            _ => self.parse_call(),
        }
    }

    fn parse_call(&mut self) -> Result<Expr, Diagnostic> {
        let mut expr = self.parse_primary()?;
        loop {
            if matches!(self.peek_kind(), TokenKind::LBracket)
                && self.call_type_args_are_followed_by_lparen()
            {
                self.expect_simple(TokenKind::LBracket, "expected `[` before type arguments")?;
                let (type_args, _) = self.parse_type_args()?;
                let mut args = Vec::new();
                self.expect_simple(TokenKind::LParen, "expected `(` after call type arguments")?;
                if !matches!(self.peek_kind(), TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr_allowing_struct_literal()?);
                        if !self.matches_simple(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                let end =
                    self.expect_simple(TokenKind::RParen, "expected `)` after call arguments")?;
                let span = expr.span().merge(end);
                expr = Expr::Call(CallExpr {
                    id: self.expr_id(),
                    callee: Box::new(expr),
                    type_args,
                    args,
                    origin: CallOrigin::Ordinary,
                    span,
                });
                continue;
            }

            if self.matches_simple(&TokenKind::LParen) {
                let mut args = Vec::new();
                if !matches!(self.peek_kind(), TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr_allowing_struct_literal()?);
                        if !self.matches_simple(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                let end =
                    self.expect_simple(TokenKind::RParen, "expected `)` after call arguments")?;
                let span = expr.span().merge(end);
                expr = Expr::Call(CallExpr {
                    id: self.expr_id(),
                    callee: Box::new(expr),
                    type_args: Vec::new(),
                    args,
                    origin: CallOrigin::Ordinary,
                    span,
                });
                continue;
            }

            if self.matches_simple(&TokenKind::Dot) {
                let (first_name, first_span) = self.expect_ident()?;
                if first_name == "with" && matches!(self.peek_kind(), TokenKind::LParen) {
                    let fields = self.parse_record_update_fields()?;
                    let span = expr
                        .span()
                        .merge(fields.last().map(|field| field.span).unwrap_or(first_span));
                    expr = Expr::RecordUpdate(RecordUpdateExpr {
                        id: self.expr_id(),
                        base: Box::new(expr),
                        fields,
                        span,
                    });
                    continue;
                }

                let (callee_name, callee_span, qualified) =
                    if self.matches_simple(&TokenKind::DoubleColon) {
                        let (second_name, second_span) = self.expect_ident()?;
                        (
                            format!("{first_name}::{second_name}"),
                            first_span.merge(second_span),
                            true,
                        )
                    } else {
                        (first_name, first_span, false)
                    };

                if matches!(self.peek_kind(), TokenKind::LParen) {
                    let start =
                        self.expect_simple(TokenKind::LParen, "expected `(` after method name")?;
                    let is_map_empty_call = !qualified
                        && callee_name == "empty"
                        && matches!(&expr, Expr::Ident(IdentExpr { name, .. }) if name == "Map");
                    let mut args = Vec::new();
                    if !matches!(self.peek_kind(), TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expr_allowing_struct_literal()?);
                            if !self.matches_simple(&TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    let end =
                        self.expect_simple(TokenKind::RParen, "expected `)` after call arguments")?;
                    let base_span = expr.span();
                    let (callee_name, callee_span, call_args, origin) = if is_map_empty_call {
                        (
                            "Map.empty".to_string(),
                            base_span.merge(callee_span),
                            args,
                            CallOrigin::Ordinary,
                        )
                    } else {
                        let base = expr;
                        let mut call_args = Vec::with_capacity(args.len() + 1);
                        call_args.push(base);
                        call_args.extend(args);
                        (
                            callee_name,
                            callee_span,
                            call_args,
                            if qualified {
                                CallOrigin::QualifiedChained
                            } else {
                                CallOrigin::Chained
                            },
                        )
                    };
                    let callee = Expr::Ident(IdentExpr {
                        id: self.expr_id(),
                        name: callee_name,
                        span: callee_span,
                    });
                    expr = Expr::Call(CallExpr {
                        id: self.expr_id(),
                        callee: Box::new(callee),
                        type_args: Vec::new(),
                        args: call_args,
                        origin,
                        span: base_span.merge(start).merge(end),
                    });
                    continue;
                }

                if qualified {
                    return Err(Diagnostic::new(
                        "P015",
                        "qualified chained calls must use `expr.alias::name(...)`",
                        callee_span,
                    ));
                }

                let span = expr.span().merge(callee_span);
                expr = Expr::Field(FieldExpr {
                    id: self.expr_id(),
                    base: Box::new(expr),
                    field: callee_name,
                    span,
                });
                continue;
            }

            if self.matches_simple(&TokenKind::LBracket) {
                self.skip_newlines();
                let index = self.parse_expr_allowing_struct_literal()?;
                self.skip_newlines();
                let end = self.expect_simple(TokenKind::RBracket, "expected `]` after index")?;
                let span = expr.span().merge(end);
                expr = Expr::Index(IndexExpr {
                    id: self.expr_id(),
                    base: Box::new(expr),
                    index: Box::new(index),
                    span,
                });
                continue;
            }

            break;
        }
        Ok(expr)
    }

    fn call_type_args_are_followed_by_lparen(&self) -> bool {
        if !matches!(self.peek_kind(), TokenKind::LBracket)
            || !matches!(self.peek_kind_n(1), TokenKind::Ident(_))
        {
            return false;
        }

        let mut index = self.current + 1;
        let mut depth = 1usize;
        while let Some(token) = self.tokens.get(index) {
            match token.kind {
                TokenKind::LBracket => depth += 1,
                TokenKind::RBracket => {
                    depth -= 1;
                    if depth == 0 {
                        return matches!(
                            self.tokens.get(index + 1).map(|token| &token.kind),
                            Some(TokenKind::LParen)
                        );
                    }
                }
                TokenKind::Eof | TokenKind::Newline => return false,
                _ => {}
            }
            index += 1;
        }
        false
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.advance();
        match token.kind {
            TokenKind::Int(text) => {
                let value = text
                    .parse::<i64>()
                    .map_err(|_| Diagnostic::new("P002", "invalid integer literal", token.span))?;
                Ok(Expr::Int(IntExpr {
                    id: self.expr_id(),
                    value,
                    span: token.span,
                }))
            }
            TokenKind::String(value) => Ok(Expr::String(StringExpr {
                id: self.expr_id(),
                value,
                span: token.span,
            })),
            TokenKind::True => Ok(Expr::Bool(BoolExpr {
                id: self.expr_id(),
                value: true,
                span: token.span,
            })),
            TokenKind::False => Ok(Expr::Bool(BoolExpr {
                id: self.expr_id(),
                value: false,
                span: token.span,
            })),
            TokenKind::Ident(name) => {
                let (name, span) = self.parse_value_name_after_first(name, token.span)?;
                if self.looks_like_record_lit() {
                    self.parse_record_lit(name, span)
                } else {
                    Ok(Expr::Ident(IdentExpr {
                        id: self.expr_id(),
                        name,
                        span,
                    }))
                }
            }
            TokenKind::LBracket => self.parse_list_lit(token.span),
            TokenKind::LParen => {
                if matches!(self.peek_kind(), TokenKind::RParen) {
                    let end = self.advance().span;
                    return Ok(Expr::Unit(UnitExpr {
                        id: self.expr_id(),
                        span: token.span.merge(end),
                    }));
                }
                let expr = self.parse_expr_allowing_struct_literal()?;
                self.expect_simple(TokenKind::RParen, "expected `)` after expression")?;
                Ok(expr)
            }
            TokenKind::Fn => self.parse_fn_expr(token.span),
            other => Err(Diagnostic::new(
                "P003",
                format!("unexpected token in expression: {:?}", other),
                token.span,
            )),
        }
    }

    fn parse_list_lit(&mut self, start: Span) -> Result<Expr, Diagnostic> {
        self.skip_newlines();
        let mut items = Vec::new();
        if !matches!(self.peek_kind(), TokenKind::RBracket) {
            loop {
                items.push(self.parse_expr_allowing_struct_literal()?);
                self.skip_newlines();
                if !self.matches_simple(&TokenKind::Comma) {
                    break;
                }
                self.skip_newlines();
            }
        }
        let end = self.expect_simple(TokenKind::RBracket, "expected `]` after list literal")?;
        Ok(Expr::ListLit(ListLitExpr {
            id: self.expr_id(),
            items,
            span: start.merge(end),
        }))
    }

    fn parse_fn_expr(&mut self, start: Span) -> Result<Expr, Diagnostic> {
        self.expect_simple(TokenKind::LParen, "expected `(` after `fn`")?;
        let params = self.parse_params()?;
        self.expect_simple(TokenKind::RParen, "expected `)` after parameters")?;
        let return_type = self.parse_return_type_annotation()?;
        let body = self.parse_value_block()?;
        let span = start.merge(body.span);
        Ok(Expr::Fn(FnExpr {
            id: self.expr_id(),
            params,
            return_type,
            body,
            span,
        }))
    }

    fn parse_return_type_annotation(&mut self) -> Result<Option<TypeExpr>, Diagnostic> {
        if self.matches_simple(&TokenKind::Colon) {
            return Ok(Some(self.parse_type_expr()?.0));
        }

        Ok(None)
    }

    fn parse_record_lit(&mut self, type_name: String, start: Span) -> Result<Expr, Diagnostic> {
        self.expect_simple(TokenKind::LBrace, "expected `{` after record type name")?;
        self.skip_newlines();
        let mut fields = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
            fields.push(self.parse_record_field_init()?);
            if matches!(self.peek_kind(), TokenKind::RBrace) {
                break;
            }
            self.consume_record_boundary()?;
            self.skip_newlines();
        }
        let end = self.expect_simple(TokenKind::RBrace, "expected `}` after record literal")?;
        Ok(Expr::RecordLit(RecordLitExpr {
            id: self.expr_id(),
            type_name,
            fields,
            span: start.merge(end),
        }))
    }

    fn parse_record_field_init(&mut self) -> Result<RecordFieldInit, Diagnostic> {
        let start = self.current_span();
        let (name, _) = self.expect_ident()?;
        self.expect_simple(TokenKind::Colon, "expected `:` after field name")?;
        let value = self.parse_expr_allowing_struct_literal()?;
        Ok(RecordFieldInit {
            name,
            span: start.merge(value.span()),
            value,
        })
    }

    fn parse_record_update_fields(&mut self) -> Result<Vec<RecordFieldInit>, Diagnostic> {
        self.expect_simple(TokenKind::LParen, "expected `(` after `.with`")?;
        let mut fields = Vec::new();
        if matches!(self.peek_kind(), TokenKind::RParen) {
            return Err(Diagnostic::new(
                "P012",
                "record update requires at least one field",
                self.current_span(),
            ));
        }
        loop {
            fields.push(self.parse_record_field_init()?);
            if !self.matches_simple(&TokenKind::Comma) {
                break;
            }
        }
        self.expect_simple(TokenKind::RParen, "expected `)` after record update")?;
        Ok(fields)
    }

    fn parse_package_path(&mut self) -> Result<(String, Span), Diagnostic> {
        let (first, first_span) = self.expect_ident()?;
        let mut parts = vec![first];
        let mut span = first_span;
        while self.matches_simple(&TokenKind::DoubleColon) {
            let (segment, segment_span) = self.expect_ident()?;
            parts.push(segment);
            span = span.merge(segment_span);
        }
        Ok((parts.join("::"), span))
    }

    fn parse_type_name_after_first(
        &mut self,
        first: String,
        first_span: Span,
    ) -> Result<(String, Span), Diagnostic> {
        let mut parts = vec![first];
        let mut span = first_span;
        while self.matches_simple(&TokenKind::DoubleColon) {
            let (segment, segment_span) = self.expect_ident()?;
            parts.push(segment);
            span = span.merge(segment_span);
        }
        Ok((parts.join("::"), span))
    }

    fn parse_value_name_after_first(
        &mut self,
        first: String,
        first_span: Span,
    ) -> Result<(String, Span), Diagnostic> {
        self.parse_type_name_after_first(first, first_span)
    }

    fn looks_like_record_lit(&self) -> bool {
        if !self.allow_struct_literal {
            return false;
        }
        if !matches!(self.peek_kind(), TokenKind::LBrace) {
            return false;
        }
        let mut index = self.current + 1;
        while matches!(
            self.tokens.get(index).map(|token| &token.kind),
            Some(TokenKind::Newline)
        ) {
            index += 1;
        }
        matches!(
            (
                self.tokens.get(index).map(|token| &token.kind),
                self.tokens.get(index + 1).map(|token| &token.kind),
            ),
            (Some(TokenKind::RBrace), _) | (Some(TokenKind::Ident(_)), Some(TokenKind::Colon))
        )
    }

    fn consume_statement_boundary(&mut self) -> Result<(), Diagnostic> {
        if matches!(self.peek_kind(), TokenKind::Newline) {
            self.skip_newlines();
            return Ok(());
        }
        if matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
            return Ok(());
        }
        Err(Diagnostic::new(
            "P004",
            "expected newline between statements",
            self.current_span(),
        ))
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek_kind(), TokenKind::Newline) {
            self.advance();
        }
    }

    fn consume_record_boundary(&mut self) -> Result<(), Diagnostic> {
        if self.matches_simple(&TokenKind::Comma) {
            return Ok(());
        }
        if matches!(self.peek_kind(), TokenKind::Newline) {
            self.skip_newlines();
            return Ok(());
        }
        if matches!(self.peek_kind(), TokenKind::RBrace) {
            return Ok(());
        }
        Err(Diagnostic::new(
            "P013",
            "expected newline or `,` between record fields",
            self.current_span(),
        ))
    }

    fn consume_enum_boundary(&mut self) -> Result<(), Diagnostic> {
        if self.matches_simple(&TokenKind::Comma) {
            return Ok(());
        }
        if matches!(self.peek_kind(), TokenKind::Newline) {
            self.skip_newlines();
            return Ok(());
        }
        if matches!(self.peek_kind(), TokenKind::RBrace) {
            return Ok(());
        }
        Err(Diagnostic::new(
            "P018",
            "expected newline or `,` between enum variants",
            self.current_span(),
        ))
    }

    fn consume_match_arm_boundary(&mut self) -> Result<(), Diagnostic> {
        if self.matches_simple(&TokenKind::Comma) {
            return Ok(());
        }
        if matches!(self.peek_kind(), TokenKind::Newline) {
            self.skip_newlines();
            return Ok(());
        }
        if matches!(self.peek_kind(), TokenKind::RBrace) {
            return Ok(());
        }
        Err(Diagnostic::new(
            "P017",
            "expected newline or `,` between match arms",
            self.current_span(),
        ))
    }

    fn consume_package_boundary(&mut self) -> Result<(), Diagnostic> {
        if matches!(self.peek_kind(), TokenKind::Newline) {
            self.skip_newlines();
            return Ok(());
        }
        if matches!(self.peek_kind(), TokenKind::Eof) {
            return Ok(());
        }
        Err(Diagnostic::new(
            "P004",
            "expected newline between package declarations",
            self.current_span(),
        ))
    }

    fn expect_ident(&mut self) -> Result<(String, Span), Diagnostic> {
        let token = self.advance();
        match token.kind {
            TokenKind::Ident(name) => Ok((name, token.span)),
            _ => Err(Diagnostic::new("P005", "expected identifier", token.span)),
        }
    }

    fn expect_simple(&mut self, expected: TokenKind, message: &str) -> Result<Span, Diagnostic> {
        let token = self.advance();
        if std::mem::discriminant(&token.kind) == std::mem::discriminant(&expected) {
            Ok(token.span)
        } else {
            Err(Diagnostic::new("P006", message, token.span))
        }
    }

    fn matches_simple(&mut self, expected: &TokenKind) -> bool {
        if std::mem::discriminant(self.peek_kind()) == std::mem::discriminant(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn current_span(&self) -> Span {
        self.tokens
            .get(self.current)
            .map(|token| token.span)
            .unwrap_or_default()
    }

    fn previous_span(&self) -> Span {
        self.tokens
            .get(self.current.saturating_sub(1))
            .map(|token| token.span)
            .unwrap_or_default()
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.tokens[self.current].kind
    }

    fn peek_kind_n(&self, n: usize) -> &TokenKind {
        self.tokens
            .get(self.current + n)
            .map(|token| &token.kind)
            .unwrap_or(&TokenKind::Eof)
    }

    fn is_eof(&self) -> bool {
        matches!(self.peek_kind(), TokenKind::Eof)
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.current].clone();
        if !matches!(token.kind, TokenKind::Eof) {
            self.current += 1;
        }
        token
    }

    fn expr_id(&mut self) -> ExprId {
        let id = ExprId::new(self.next_expr_id);
        self.next_expr_id += 1;
        id
    }

    fn stmt_id(&mut self) -> StmtId {
        let id = StmtId::new(self.next_stmt_id);
        self.next_stmt_id += 1;
        id
    }
}

#[derive(Clone, Debug)]
enum ElseBranch {
    Block(Block),
    If(Box<Stmt>),
}

impl ElseBranch {
    fn span(&self) -> Span {
        match self {
            Self::Block(block) => block.span,
            Self::If(stmt) => stmt.span(),
        }
    }

    fn into_block(self) -> Block {
        match self {
            Self::Block(block) => block,
            Self::If(stmt) => {
                let span = stmt.span();
                Block {
                    statements: vec![*stmt],
                    span,
                }
            }
        }
    }
}

fn json_attribute_is_json_schema_metadata(attribute: &Attribute) -> bool {
    attribute.name == "json"
        && !attribute.arguments.is_empty()
        && attribute.arguments.iter().all(|argument| {
            matches!(argument.name.as_str(), "rename" | "alias")
                && argument
                    .value
                    .as_ref()
                    .is_some_and(|value| value.as_string().is_some())
        })
}

fn json_attribute_is_deny_unknown_fields(attribute: &Attribute) -> bool {
    attribute.name == "json"
        && attribute.arguments.len() == 1
        && attribute.arguments[0].name == "deny_unknown_fields"
        && attribute.arguments[0].value.is_none()
}

fn cli_attribute_is_record_metadata(attribute: &Attribute) -> bool {
    attribute.name == "cli"
        && attribute.arguments.len() == 1
        && attribute.arguments[0].name == "about"
        && attribute.arguments[0]
            .value
            .as_ref()
            .is_some_and(|value| value.as_string().is_some())
}

fn cli_attribute_is_enum_metadata(attribute: &Attribute) -> bool {
    cli_attribute_is_record_metadata(attribute)
}

fn cli_attribute_is_field_metadata(attribute: &Attribute) -> bool {
    attribute.name == "cli"
        && !attribute.arguments.is_empty()
        && attribute.arguments.iter().all(|argument| {
            matches!(
                argument.name.as_str(),
                "name"
                    | "short"
                    | "positional"
                    | "value_source"
                    | "alias"
                    | "help"
                    | "hidden"
                    | "subcommand"
            )
        })
}

fn cli_attribute_is_command_variant_metadata(attribute: &Attribute) -> bool {
    attribute.name == "cli"
        && !attribute.arguments.is_empty()
        && attribute.arguments.iter().all(|argument| {
            matches!(
                argument.name.as_str(),
                "name" | "alias" | "about" | "hidden"
            )
        })
}

fn is_cli_long_option_token(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

fn is_cli_short_option_token(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    chars.next().is_none() && first.is_ascii_alphabetic()
}

fn split_variant_name(name: &str) -> Option<(String, String)> {
    let (enum_name, variant_name) = name.rsplit_once("::")?;
    if enum_name.is_empty() || variant_name.is_empty() {
        None
    } else {
        Some((enum_name.to_string(), variant_name.to_string()))
    }
}
