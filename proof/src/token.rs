
use std::fmt::{Debug, Display};
use std::hash::{Hash, Hasher};
use std::fmt::Formatter;
use std::fmt;

// End of file 
pub const EOF: char = '\0';


pub trait ToToken {
    // returns token
    fn token(&self) -> Token;
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
pub struct Token {
    pub kind: TokenType,
    pub lexeme: Option<String>, // or Option<String> if not always needed
    pub position: usize,
    pub source_id: usize,
}
impl Default for TokenType {
    fn default() -> Self {
        TokenType::Unknown //fallback
    }
}


impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.lexeme == other.lexeme
    }
}

impl Eq for Token {}

impl PartialOrd for Token {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Token {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.kind.cmp(&other.kind).then_with(|| self.lexeme.cmp(&other.lexeme))
    }
}

impl Hash for Token {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.lexeme.hash(state);
    }
}


impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.kind)?;

        if let Some(ref lex) = self.lexeme {
            write!(f, " [{}]", lex)?;
        }

        write!(f, " @pos:{} src:{}", self.position, self.source_id)
    }
}

impl Token {
    pub fn new(kind: TokenType) -> Self {
        Token {
            kind,
            lexeme: None,
            position: 0,
            source_id: 0,
        }
    }

    pub fn with_lexeme<S: Into<String>>(kind: TokenType, lexeme: S) -> Self {
        Self {
            kind,
            lexeme: Some(lexeme.into()),
            position: 0,
            source_id: 0,
        }

    }

    pub fn lexeme(&self) -> String {
        self.lexeme.as_deref().unwrap_or("?").to_string()
    }

}