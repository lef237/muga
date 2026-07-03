use crate::span::Span;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Eof,
    Newline,
    Ident(String),
    Int(String),
    String(String),
    Package,
    Import,
    Pub,
    Pkg,
    As,
    Fn,
    Record,
    Enum,
    Opaque,
    Type,
    Match,
    Mut,
    If,
    Else,
    While,
    For,
    In,
    Break,
    Continue,
    Return,
    Try,
    Using,
    Group,
    Spawn,
    And,
    Or,
    True,
    False,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    At,
    Dot,
    DoubleColon,
    Comma,
    Colon,
    Arrow,
    FatArrow,
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
                | TokenKind::Arrow
                | TokenKind::FatArrow
                | TokenKind::Try
                | TokenKind::Spawn
                | TokenKind::LBracket
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
                | TokenKind::And
                | TokenKind::Or
        )
    }
}
