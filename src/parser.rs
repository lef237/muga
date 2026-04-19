use crate::ast::*;
use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::token::{Token, TokenKind};

pub fn parse(tokens: Vec<Token>) -> Result<Program, Vec<Diagnostic>> {
    let mut parser = Parser::new(tokens);
    parser
        .parse_program()
        .map_err(|diagnostic| vec![diagnostic])
}

struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    fn parse_program(&mut self) -> Result<Program, Diagnostic> {
        let mut statements = Vec::new();
        self.skip_newlines();

        while !self.is_eof() {
            statements.push(self.parse_top_stmt()?);
            self.consume_statement_boundary()?;
            self.skip_newlines();
        }

        Ok(Program { statements })
    }

    fn parse_top_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        match self.peek_kind() {
            TokenKind::Record => self.parse_record_decl().map(Stmt::RecordDecl),
            _ => self.parse_stmt(),
        }
    }

    fn parse_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        match self.peek_kind() {
            TokenKind::Mut => self.parse_assign_stmt(true).map(Stmt::Assign),
            TokenKind::Record => Err(Diagnostic::new(
                "P010",
                "record declarations are top-level only",
                self.current_span(),
            )),
            TokenKind::Fn if matches!(self.peek_kind_n(1), TokenKind::Ident(_)) => {
                self.parse_func_decl().map(Stmt::FuncDecl)
            }
            TokenKind::If => self.parse_if_stmt_or_expr_stmt(),
            TokenKind::While => self.parse_while_stmt().map(Stmt::While),
            TokenKind::Ident(_) if matches!(self.peek_kind_n(1), TokenKind::Eq) => {
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
        self.expect_simple(TokenKind::Eq, "expected `=` after binding name")?;
        let value = self.parse_expr()?;
        Ok(AssignStmt {
            mutable,
            name,
            value,
            span: start.merge(name_span).merge(self.previous_span()),
        })
    }

    fn parse_record_decl(&mut self) -> Result<RecordDecl, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::Record, "expected `record`")?;
        let (name, _) = self.expect_ident()?;
        self.expect_simple(TokenKind::LBrace, "expected `{` after record name")?;
        self.skip_newlines();
        let mut fields = Vec::new();
        while !matches!(self.peek_kind(), TokenKind::RBrace | TokenKind::Eof) {
            let field_start = self.current_span();
            let (field_name, _) = self.expect_ident()?;
            self.expect_simple(TokenKind::Colon, "expected `:` after field name")?;
            let (type_name, type_span) = self.parse_type_name()?;
            fields.push(RecordFieldDecl {
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
            name,
            fields,
            span: start.merge(end),
        })
    }

    fn parse_func_decl(&mut self) -> Result<FuncDecl, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::Fn, "expected `fn`")?;
        let (name, _) = self.expect_ident()?;
        self.expect_simple(TokenKind::LParen, "expected `(` after function name")?;
        let params = self.parse_params()?;
        self.expect_simple(TokenKind::RParen, "expected `)` after parameters")?;
        let return_type = self.parse_return_type_annotation()?;
        let body = self.parse_value_block()?;
        let span = start.merge(body.span);
        Ok(FuncDecl {
            name,
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
                let (type_name, type_span) = self.parse_type_name()?;
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

    fn parse_type_name(&mut self) -> Result<(TypeName, Span), Diagnostic> {
        let token = self.advance();
        match &token.kind {
            TokenKind::Ident(name) if name == "Int" => Ok((TypeName::Int, token.span)),
            TokenKind::Ident(name) if name == "Bool" => Ok((TypeName::Bool, token.span)),
            TokenKind::Ident(name) if name == "String" => Ok((TypeName::String, token.span)),
            TokenKind::Ident(name) => Ok((TypeName::Named(name.clone()), token.span)),
            _ => Err(Diagnostic::new(
                "P001",
                "expected a type name",
                token.span,
            )),
        }
    }

    fn parse_if_stmt_or_expr_stmt(&mut self) -> Result<Stmt, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::If, "expected `if`")?;
        let condition = self.parse_expr()?;
        let then_block = self.parse_block()?;
        if self.matches_simple(&TokenKind::Else) {
            let else_block = self.parse_block()?;
            let then_branch = block_to_value_block(then_block)?;
            let else_branch = block_to_value_block(else_block)?;
            let expr = Expr::If(IfExpr {
                condition: Box::new(condition),
                span: start.merge(else_branch.span),
                then_branch,
                else_branch,
            });
            let span = expr.span();
            Ok(Stmt::Expr(ExprStmt { expr, span }))
        } else {
            let span = start.merge(then_block.span);
            Ok(Stmt::If(IfStmt {
                condition,
                then_branch: then_block,
                else_branch: None,
                span,
            }))
        }
    }

    fn parse_while_stmt(&mut self) -> Result<WhileStmt, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::While, "expected `while`")?;
        let condition = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(WhileStmt {
            condition,
            span: start.merge(body.span),
            body,
        })
    }

    fn parse_expr_stmt(&mut self) -> Result<ExprStmt, Diagnostic> {
        let expr = self.parse_expr()?;
        let span = expr.span();
        Ok(ExprStmt { expr, span })
    }

    fn parse_block(&mut self) -> Result<Block, Diagnostic> {
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
        block_to_value_block(block)
    }

    fn parse_expr(&mut self) -> Result<Expr, Diagnostic> {
        if matches!(self.peek_kind(), TokenKind::If) {
            return self.parse_if_expr();
        }
        self.parse_equality()
    }

    fn parse_if_expr(&mut self) -> Result<Expr, Diagnostic> {
        let start = self.current_span();
        self.expect_simple(TokenKind::If, "expected `if`")?;
        let condition = self.parse_expr()?;
        let then_branch = self.parse_value_block()?;
        self.expect_simple(TokenKind::Else, "expected `else` in `if` expression")?;
        let else_branch = self.parse_value_block()?;
        Ok(Expr::If(IfExpr {
            condition: Box::new(condition),
            span: start.merge(else_branch.span),
            then_branch,
            else_branch,
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
                let expr = self.parse_unary()?;
                Ok(Expr::Unary(UnaryExpr {
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
                    op: UnaryOp::Not,
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
            if self.matches_simple(&TokenKind::LParen) {
                let mut args = Vec::new();
                if !matches!(self.peek_kind(), TokenKind::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if !self.matches_simple(&TokenKind::Comma) {
                            break;
                        }
                    }
                }
                let end =
                    self.expect_simple(TokenKind::RParen, "expected `)` after call arguments")?;
                let span = expr.span().merge(end);
                expr = Expr::Call(CallExpr {
                    callee: Box::new(expr),
                    args,
                    span,
                });
                continue;
            }

            if self.matches_simple(&TokenKind::Dot) {
                let (name, name_span) = self.expect_ident()?;
                if name == "with" && matches!(self.peek_kind(), TokenKind::LParen) {
                    let fields = self.parse_record_update_fields()?;
                    let span = expr
                        .span()
                        .merge(fields.last().map(|field| field.span).unwrap_or(name_span));
                    expr = Expr::RecordUpdate(RecordUpdateExpr {
                        base: Box::new(expr),
                        fields,
                        span,
                    });
                    continue;
                }

                if matches!(self.peek_kind(), TokenKind::LParen) {
                    return Err(Diagnostic::new(
                        "P011",
                        "chained dot calls are not implemented yet",
                        name_span,
                    ));
                }

                let span = expr.span().merge(name_span);
                expr = Expr::Field(FieldExpr {
                    base: Box::new(expr),
                    field: name,
                    span,
                });
                continue;
            }

            break;
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.advance();
        match token.kind {
            TokenKind::Int(text) => {
                let value = text
                    .parse::<i64>()
                    .map_err(|_| Diagnostic::new("P002", "invalid integer literal", token.span))?;
                Ok(Expr::Int(IntExpr {
                    value,
                    span: token.span,
                }))
            }
            TokenKind::String(value) => Ok(Expr::String(StringExpr {
                value,
                span: token.span,
            })),
            TokenKind::True => Ok(Expr::Bool(BoolExpr {
                value: true,
                span: token.span,
            })),
            TokenKind::False => Ok(Expr::Bool(BoolExpr {
                value: false,
                span: token.span,
            })),
            TokenKind::Ident(name) if self.looks_like_record_lit() => {
                self.parse_record_lit(name, token.span)
            }
            TokenKind::Ident(name) => Ok(Expr::Ident(IdentExpr {
                name,
                span: token.span,
            })),
            TokenKind::LParen => {
                let expr = self.parse_expr()?;
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

    fn parse_fn_expr(&mut self, start: Span) -> Result<Expr, Diagnostic> {
        self.expect_simple(TokenKind::LParen, "expected `(` after `fn`")?;
        let params = self.parse_params()?;
        self.expect_simple(TokenKind::RParen, "expected `)` after parameters")?;
        let return_type = self.parse_return_type_annotation()?;
        let body = self.parse_value_block()?;
        let span = start.merge(body.span);
        Ok(Expr::Fn(FnExpr {
            params,
            return_type,
            body,
            span,
        }))
    }

    fn parse_return_type_annotation(&mut self) -> Result<Option<TypeName>, Diagnostic> {
        if self.matches_simple(&TokenKind::Colon) {
            return Ok(Some(self.parse_type_name()?.0));
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
            type_name,
            fields,
            span: start.merge(end),
        }))
    }

    fn parse_record_field_init(&mut self) -> Result<RecordFieldInit, Diagnostic> {
        let start = self.current_span();
        let (name, _) = self.expect_ident()?;
        self.expect_simple(TokenKind::Colon, "expected `:` after field name")?;
        let value = self.parse_expr()?;
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

    fn looks_like_record_lit(&self) -> bool {
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
        match (
            self.tokens.get(index).map(|token| &token.kind),
            self.tokens.get(index + 1).map(|token| &token.kind),
        ) {
            (Some(TokenKind::RBrace), _) => true,
            (Some(TokenKind::Ident(_)), Some(TokenKind::Colon)) => true,
            _ => false,
        }
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
}

fn block_to_value_block(block: Block) -> Result<ValueBlock, Diagnostic> {
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
            if let Stmt::Expr(expr_stmt) = stmt {
                return Ok(ValueBlock {
                    statements: prefix,
                    expr: Box::new(expr_stmt.expr),
                    span: block.span,
                });
            }
            return Err(Diagnostic::new(
                "P008",
                "value block must end with an expression",
                stmt.span(),
            ));
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
