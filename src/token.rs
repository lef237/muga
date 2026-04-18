use crate::span::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Eof,
    Newline,
    Ident(String),
    Int(String),
    String(String),
    Fn,
    Mut,
    If,
    Else,
    While,
    True,
    False,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Eq,
    EqEq,
    Bang,
    BangEq,
    Plus,
    Minus,
    Star,
    Slash,
    Lt,
    LtEq,
    Gt,
    GtEq,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub const fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

impl TokenKind {
    pub fn is_stmt_continuation(&self) -> bool {
        matches!(
            self,
            TokenKind::Eq
                | TokenKind::Comma
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Lt
                | TokenKind::LtEq
                | TokenKind::Gt
                | TokenKind::GtEq
                | TokenKind::EqEq
                | TokenKind::BangEq
        )
    }
}
