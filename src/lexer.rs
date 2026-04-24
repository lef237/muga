use crate::diagnostic::Diagnostic;
use crate::span::{Position, Span};
use crate::token::{Token, TokenKind};

pub fn lex(source: &str) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let mut lexer = Lexer::new(source);
    lexer.lex();
    if lexer.diagnostics.is_empty() {
        Ok(lexer.tokens)
    } else {
        Err(lexer.diagnostics)
    }
}

struct Lexer {
    chars: Vec<char>,
    index: usize,
    line: usize,
    column: usize,
    paren_depth: usize,
    last_significant: Option<TokenKind>,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl Lexer {
    fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            index: 0,
            line: 1,
            column: 1,
            paren_depth: 0,
            last_significant: None,
            tokens: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn lex(&mut self) {
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' => {
                    self.advance();
                }
                '\r' => {
                    self.advance();
                    if self.peek() == Some('\n') {
                        self.advance();
                    }
                    self.handle_newline();
                }
                '\n' => {
                    self.advance();
                    self.handle_newline();
                }
                '(' => self.emit_simple(TokenKind::LParen),
                ')' => self.emit_simple(TokenKind::RParen),
                '{' => self.emit_simple(TokenKind::LBrace),
                '}' => self.emit_simple(TokenKind::RBrace),
                '.' => self.emit_simple(TokenKind::Dot),
                ',' => self.emit_simple(TokenKind::Comma),
                ':' => {
                    if self.peek_next() == Some(':') {
                        let start = self.position();
                        self.advance();
                        self.advance();
                        self.push(TokenKind::DoubleColon, Span::new(start, self.position()));
                    } else {
                        self.emit_simple(TokenKind::Colon)
                    }
                }
                '+' => self.emit_simple(TokenKind::Plus),
                '*' => self.emit_simple(TokenKind::Star),
                '/' => {
                    if self.peek_next() == Some('/') {
                        self.lex_comment();
                    } else {
                        self.emit_simple(TokenKind::Slash)
                    }
                }
                '-' => {
                    if self.peek_next() == Some('>') {
                        let start = self.position();
                        self.advance();
                        self.advance();
                        self.push(TokenKind::Arrow, Span::new(start, self.position()));
                    } else {
                        self.emit_simple(TokenKind::Minus)
                    }
                }
                '=' => {
                    if self.peek_next() == Some('=') {
                        let start = self.position();
                        self.advance();
                        self.advance();
                        self.push(TokenKind::EqEq, Span::new(start, self.position()));
                    } else {
                        self.emit_simple(TokenKind::Eq);
                    }
                }
                '!' => {
                    if self.peek_next() == Some('=') {
                        let start = self.position();
                        self.advance();
                        self.advance();
                        self.push(TokenKind::BangEq, Span::new(start, self.position()));
                    } else {
                        self.emit_simple(TokenKind::Bang);
                    }
                }
                '<' => {
                    if self.peek_next() == Some('=') {
                        let start = self.position();
                        self.advance();
                        self.advance();
                        self.push(TokenKind::LtEq, Span::new(start, self.position()));
                    } else {
                        self.emit_simple(TokenKind::Lt);
                    }
                }
                '>' => {
                    if self.peek_next() == Some('=') {
                        let start = self.position();
                        self.advance();
                        self.advance();
                        self.push(TokenKind::GtEq, Span::new(start, self.position()));
                    } else {
                        self.emit_simple(TokenKind::Gt);
                    }
                }
                '"' => self.lex_string(),
                '0'..='9' => self.lex_number(),
                'A'..='Z' | 'a'..='z' | '_' => self.lex_ident_or_keyword(),
                _ => {
                    let span = Span::single(self.position());
                    self.diagnostics.push(Diagnostic::new(
                        "L001",
                        format!("unexpected character `{ch}`"),
                        span,
                    ));
                    self.advance();
                }
            }
        }

        let pos = self.position();
        self.tokens
            .push(Token::new(TokenKind::Eof, Span::single(pos)));
    }

    fn lex_comment(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            self.advance();
        }
    }

    fn lex_number(&mut self) {
        let start = self.position();
        let mut text = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                text.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        self.push(TokenKind::Int(text), Span::new(start, self.position()));
    }

    fn lex_ident_or_keyword(&mut self) {
        let start = self.position();
        let mut text = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                text.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        let kind = match text.as_str() {
            "package" => TokenKind::Package,
            "import" => TokenKind::Import,
            "pub" => TokenKind::Pub,
            "as" => TokenKind::As,
            "fn" => TokenKind::Fn,
            "record" => TokenKind::Record,
            "mut" => TokenKind::Mut,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "while" => TokenKind::While,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            _ => TokenKind::Ident(text),
        };
        self.push(kind, Span::new(start, self.position()));
    }

    fn lex_string(&mut self) {
        let start = self.position();
        self.advance();
        let mut value = String::new();

        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    self.advance();
                    self.push(TokenKind::String(value), Span::new(start, self.position()));
                    return;
                }
                '\\' => {
                    self.advance();
                    match self.peek() {
                        Some('\\') => {
                            value.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            value.push('"');
                            self.advance();
                        }
                        Some('n') => {
                            value.push('\n');
                            self.advance();
                        }
                        Some('t') => {
                            value.push('\t');
                            self.advance();
                        }
                        Some(other) => {
                            let span = Span::single(self.position());
                            self.diagnostics.push(Diagnostic::new(
                                "L002",
                                format!("unsupported string escape `\\{other}`"),
                                span,
                            ));
                            self.advance();
                        }
                        None => break,
                    }
                }
                '\n' | '\r' => {
                    self.diagnostics.push(Diagnostic::new(
                        "L003",
                        "unterminated string literal",
                        Span::new(start, self.position()),
                    ));
                    return;
                }
                other => {
                    value.push(other);
                    self.advance();
                }
            }
        }

        self.diagnostics.push(Diagnostic::new(
            "L003",
            "unterminated string literal",
            Span::new(start, self.position()),
        ));
    }

    fn handle_newline(&mut self) {
        if self.paren_depth > 0 {
            return;
        }
        if self
            .last_significant
            .as_ref()
            .is_some_and(TokenKind::is_stmt_continuation)
        {
            return;
        }
        if matches!(
            self.tokens.last().map(|token| &token.kind),
            Some(TokenKind::Newline)
        ) {
            return;
        }
        let pos = self.position();
        self.tokens
            .push(Token::new(TokenKind::Newline, Span::single(pos)));
    }

    fn emit_simple(&mut self, kind: TokenKind) {
        let start = self.position();
        let ch = self.advance();
        let span = Span::new(start, self.position());
        match ch {
            Some('(') => self.paren_depth += 1,
            Some(')') => self.paren_depth = self.paren_depth.saturating_sub(1),
            _ => {}
        }
        self.push(kind, span);
    }

    fn push(&mut self, kind: TokenKind, span: Span) {
        if !matches!(kind, TokenKind::Newline | TokenKind::Eof) {
            self.last_significant = Some(kind.clone());
        }
        self.tokens.push(Token::new(kind, span));
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.index).copied()
    }

    fn peek_next(&self) -> Option<char> {
        self.chars.get(self.index + 1).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.index += 1;
        if ch == '\n' || ch == '\r' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(ch)
    }

    fn position(&self) -> Position {
        Position::new(self.line, self.column)
    }
}
