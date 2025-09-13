#[allow(dead_code)]
#[allow(unused_variables)]
pub mod prelude {
    pub use super::Lexer;
}

use crate::{
    token::{Token, TokenType, EOF},
};
use TokenType::*;
use std::{iter::Peekable, str::Chars};


#[derive(Debug, Clone)]
#[allow(dead_code)]
#[allow(unused_variables)]
pub struct SourceFile {
    pub id: usize,
    pub filename: String,
    pub content: String,
}
#[allow(dead_code)]
#[allow(unused_variables)]
impl SourceFile {
    pub fn new<S: Into<String>>(id: usize, filename: S, content: S) -> Self {
        Self {
            id,
            filename: filename.into(),
            content: content.into(),
        }
    }

    
    pub fn len(&self) -> usize {
        self.content.len()
    }

   
    pub fn char_at(&self, pos: usize) -> char {
        self.content.chars().nth(pos).unwrap_or('\0')
    }

    /// Slice a range from the source
    pub fn slice(&self, start: usize, end: usize) -> &str {
        &self.content[start..end]
    }

    pub fn position_to_line_col(&self, pos: usize) -> (usize, usize) {
        let mut line = 1;
        let mut col = 1;
        let mut current_pos = 0;

        for ch in self.content.chars() {
            if current_pos == pos {
                break;
            }

            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }

         
            current_pos += ch.len_utf8();
        }

        (line, col)
    }
}

#[allow(dead_code)]
#[allow(unused_variables)]

pub struct Lexer<'a> {
    pub chars: Peekable<Chars<'a>>,
    text: &'a [u8],
    pos: usize,
    peeked: Option<Token<'a>>,
    current: Option<Token<'a>>,
    current_len: u32,
}

#[allow(dead_code)]
#[allow(unused_variables)]
impl<'a> Lexer<'a> {
    pub fn new(s: &str) -> Lexer<'_> {
        Lexer {
            chars: s.chars().peekable(),
            text: s.as_bytes(),
            pos: 0,
            peeked: None,
            current: None,
            current_len: 0,
        }
    }

    pub fn current_token(&self) -> Option<&Token> {
        self.current.as_ref()
    }

    pub fn slice(&mut self) -> &'a [u8] {
        &self.text[(self.pos - self.current_len as usize)..(self.pos)]
    }
    pub fn next_token(&mut self) -> Option<Token<'a>> {
       

        let start_pos = self.pos;


        let ch = self.advance()?;

        let token_type = match ch {
            c if c.is_whitespace() => {
                self.consume_whitespace();
                return self.next_token(); // skip and continue
            }
        
            c if Self::is_ident_part(c) => {
                let kind = self.consume_identifier_or_keyword(c);
                return Some(Token {
                    kind,
                    lexeme: None, // optionally Some(...) if you're tracking slices
                    position: self.pos,
                    source_id: 0,
                });
            }
        
            c @ '0'..='9' => {
                let kind = self.consume_number(c);
                return Some(Token {
                    kind,
                    lexeme: None,
                    position: self.pos,
                    source_id: 0,
                });
            }

            '"' => {
                let mut string = String::new();
                while let Some(nc) = self.advance() {
                    if nc == '"' {
                        break;
                    }
                    string.push(nc);
                }
                StringLiteral(string)
            }

            ':' => {
                if self.peek_char() == Some(&'=') {
                    self.advance();
                    ColonEqual
                } else {
                    Colon
                }
            }

            '=' => Equal,
            '+' => Plus,
            '-' => Minus,
            '*' => Star,
            '.' => Dot,
            ',' => Comma,
            '(' => LParen,
            ')' => RParen,
            '¬' | '!' => Not,
            '∧' => And,
            '∨' => Or,
            '→' => Implies,
            '∀' => ForAll,
            '∃' => Exists,
            EOF => Eof,
            _ => Unknown,
        };

        let end_pos = self.pos;
        self.current_len = (end_pos - start_pos) as u32;
        let lexeme_bytes = &self.text[start_pos..end_pos];
        let lexeme = std::str::from_utf8(lexeme_bytes).unwrap();


        let tok = Token {
            kind: token_type,
            lexeme:Some(lexeme),
            position: start_pos,
            source_id:0,
        };

        self.current = Some(tok.clone());
        Some(tok)
    }

    pub fn peek_token(&mut self) -> Option<&Token> {
        if self.peeked.is_none() {
            self.peeked = self.next_token();
        }
        self.peeked.as_ref()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += c.len_utf8();  // Correctly update byte position
        Some(c)
    }

    #[inline]
    fn peek_char(&mut self) -> Option<&char> {
        self.chars.peek()
    }
    #[inline]
    fn issymbol(c: char) -> bool {
        "+-*/=<>^&|".contains(c)
    }
    #[inline]
    fn consume_whitespace(&mut self) {
        while self.peek_char().map_or(false, |c| c.is_whitespace()) {
            self.advance();
        }
    }
#[inline]
    fn consume_identifier_or_keyword(&mut self, first_char: char) -> TokenType {
        let mut ident = String::new();
        ident.push(first_char);
    
        while let Some(nc) = self.peek_char() {
            if Self::is_ident_part(*nc) {
                ident.push(self.advance().unwrap());
            } else {
                break;
            }
        }
    
        let kind = match ident.as_str() {
            "theorem" => TokenType::Theorem,
            "proof"   => TokenType::Proof,
            _         => TokenType::Ident(ident),
        };
        kind
    }
    #[inline]
    fn consume_number(&mut self, first_char: char) -> TokenType {
        let mut number = first_char.to_string();
    
        while let Some(nc) = self.peek_char() {
            if nc.is_ascii_digit() {
                number.push(self.advance().unwrap());
            } else {
                break;
            }
        }
    
        TokenType::Number(number.parse().unwrap_or(0))
    }
    #[inline]
    fn is_ident_part(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }
    
}