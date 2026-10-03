use std::fmt::{self, Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use crate::syntax::span::{Span, Spanned};

pub mod prelude {
    pub use super::{Token, TokenType};
}

pub const EOF: char = '\0';

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum TokenType {
    Eof,
    Unknown,

    // Logical Operators
    ForAll,  // ∀ or forall
    Exists,  // ∃ or exists
    Implies, // → or ->
    Iff,     // ↔ or <->
    And,     // ∧ or and
    Or,      // ∨ or or
    Not,     // ¬ or ! or not

    // Arithmetic & Relations
    Plus,
    Minus,
    Star,
    Equal,      // =
    Colon,      // :
    ColonEqual, // :=
    Dot,        // .
    Comma,      // ,
    LParen,     // (
    RParen,     // )

    // Language Keywords (from LANGUAGE_SPEC.md & ROADMAP.md)
    Theorem,
    Proof,
    End,
    Take,
    Suppose,
    Let,
    Have,
    Construct,
    Show,
    Derive,
    Use,
    Therefore,
    Choose,
    Cases,
    Contradict,
    Figure,
    Given,
    From,
    As,
    Using,

    // Literals & Identifiers
    Ident(String),
    Number(i64),
    StringLiteral(String),
}

impl Default for TokenType {
    fn default() -> Self {
        TokenType::Unknown
    }
}

#[derive(Debug, Clone, Default)]
pub struct Token<'a> {
    pub kind: TokenType,
    pub lexeme: Option<&'a str>,
    pub position: usize,
    pub source_id: usize,
    pub span: Span,
}

impl<'a> Spanned for Token<'a> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<'a> PartialEq for Token<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.lexeme == other.lexeme
    }
}

impl<'a> Eq for Token<'a> {}

impl<'a> PartialOrd for Token<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Token<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.kind.cmp(&other.kind).then_with(|| self.lexeme.cmp(&other.lexeme))
    }
}

impl<'a> Hash for Token<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.lexeme.hash(state);
    }
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)?;

        if let Some(ref lex) = self.lexeme {
            write!(f, " [{}]", lex)?;
        }

        write!(f, " @pos:{} (span {})", self.position, self.span)
    }
}

impl<'a> Token<'a> {
    pub fn new(kind: TokenType) -> Self {
        Token {
            kind,
            lexeme: None,
            position: 0,
            source_id: 0,
            span: Span::DUMMY,
        }
    }

    #[inline]
    pub fn with_lexeme(kind: TokenType, lexeme: &'a str) -> Self {
        Self {
            kind,
            lexeme: Some(lexeme),
            position: 0,
            source_id: 0,
            span: Span::DUMMY,
        }
    }

    #[inline]
    pub fn with_span(kind: TokenType, lexeme: &'a str, span: Span) -> Self {
        Self {
            kind,
            lexeme: Some(lexeme),
            position: span.start as usize,
            source_id: span.file.0 as usize,
            span,
        }
    }

    #[inline]
    pub fn lexeme(&self) -> &str {
        self.lexeme.unwrap_or("?")
    }
}
