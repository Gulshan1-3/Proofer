
#[allow(unused_variables)]
pub mod prelude {
    pub use super::{Token, TokenType,};
}
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::fmt::Formatter;
use std::fmt;

// End of file 
pub const EOF: char = '\0';
#[allow(dead_code)]
#[allow(unused_variables)]

trait ToToken<'a> {
    fn token(&'a self) -> &'a Token<'a>;
}






#[derive(Debug, Clone, PartialEq, Eq,Ord,PartialOrd,Hash)]
pub enum TokenType {
    Eof,
   
    Unknown,
    ForAll, // ∀
    Exists, // ∃
    Implies,// →
    And,    // ∧
    Or,     // ∨
    Plus,
    Minus,
    Star,
    Not,    // ¬ or !
    Equal,  // =
    Colon,  // :
    ColonEqual, // :=
    Dot, // .
    Comma, //,
    LParen, // (
    RParen, // )
   
    
    Theorem,
    Proof,
   

    Ident(String),
    Number(i64),
    StringLiteral(String),
}
#[derive(Debug, Clone,Default)]
pub struct Token<'a>{
    pub kind: TokenType,
    pub lexeme: Option<&'a str>,// or Option<String> if not always needed
    pub position: usize,
    pub source_id: usize,
}
impl Default for TokenType {
    fn default() -> Self {
        TokenType::Unknown //fallback
    }
}


impl <'a> PartialEq for Token<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.lexeme == other.lexeme
    }
}

impl <'a> Eq for Token <'a>{}

impl <'a> PartialOrd for Token<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl <'a> Ord for Token<'a> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.kind.cmp(&other.kind).then_with(|| self.lexeme.cmp(&other.lexeme))
    }
}

impl <'a> Hash for Token <'a>{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.lexeme.hash(state);
    }
}


impl<'a>Display for Token <'a>{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)?;

        if let Some(ref lex) = self.lexeme {
            write!(f, " [{}]", lex)?;
        }

        write!(f, " @pos:{} src:{}", self.position, self.source_id)
    }
}

#[allow(dead_code)]
#[allow(unused_variables)]

impl  <'a> Token <'a>{
    pub fn new(kind: TokenType) -> Self {
        Token {
            kind,
            lexeme: None,
            position: 0,
            source_id: 0,
        }
    }
    #[inline]
    pub fn with_lexeme(kind: TokenType, lexeme: &'a str) -> Self {
        Self {
            kind,
            lexeme: Some(lexeme),
            position: 0,
            source_id: 0,
        }
    }
    #[inline]
    pub fn lexeme(&self) -> &str {
        self.lexeme.unwrap_or("?")
    }

}

