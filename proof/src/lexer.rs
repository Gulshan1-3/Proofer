#[allow(dead_code)]
#[allow(unused_variables)]
pub mod prelude {
    pub use super::Lexer;
}

use crate::{
    syntax::span::{FileId, Span},
    token::{Token, TokenType, EOF},
};
use std::{iter::Peekable, str::Chars};
use TokenType::*;

pub struct Lexer<'a> {
    pub chars: Peekable<Chars<'a>>,
    text: &'a [u8],
    pos: usize,
    peeked: Option<Token<'a>>,
    current: Option<Token<'a>>,
    file_id: FileId,
}

impl<'a> Lexer<'a> {
    pub fn new(s: &'a str) -> Lexer<'a> {
        Self::new_with_file(s, FileId(0))
    }

    pub fn new_with_file(s: &'a str, file_id: FileId) -> Lexer<'a> {
        Lexer {
            chars: s.chars().peekable(),
            text: s.as_bytes(),
            pos: 0,
            peeked: None,
            current: None,
            file_id,
        }
    }

    pub fn current_token(&self) -> Option<&Token<'_>> {
        self.current.as_ref()
    }

    pub fn next_token(&mut self) -> Option<Token<'a>> {
        if let Some(peeked) = self.peeked.take() {
            self.current = Some(peeked.clone());
            return Some(peeked);
        }

        // Skip whitespace and comments before starting token
        loop {
            while let Some(c) = self.chars.peek() {
                if c.is_whitespace() {
                    self.advance();
                } else {
                    break;
                }
            }

            // Check for line comments: //
            if let Some(&'/') = self.chars.peek() {
                let mut clone = self.chars.clone();
                clone.next();
                if let Some(&'/') = clone.peek() {
                    // Skip until newline
                    self.advance(); // consume first '/'
                    self.advance(); // consume second '/'
                    while let Some(c) = self.advance() {
                        if c == '\n' {
                            break;
                        }
                    }
                    continue;
                }
            }
            break;
        }

        let start_pos = self.pos;
        let ch = match self.advance() {
            Some(c) => c,
            None => {
                // End of input — produce the Eof token
                let span = Span::new(self.file_id, start_pos as u32, start_pos as u32);
                let tok = Token {
                    kind: Eof,
                    lexeme: None,
                    position: start_pos,
                    source_id: self.file_id.0 as usize,
                    span,
                };
                self.current = Some(tok.clone());
                return Some(tok);
            }
        };

        let (token_type, end_pos, lexeme) = match ch {
            c if Self::is_ident_start(c) => {
                let kind = self.consume_identifier_or_keyword(c);
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (kind, end, Some(lexeme_str))
            }

            c @ '0'..='9' => {
                let kind = self.consume_number(c);
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (kind, end, Some(lexeme_str))
            }

            '"' => {
                let mut string = String::new();
                while let Some(nc) = self.advance() {
                    if nc == '"' {
                        break;
                    }
                    string.push(nc);
                }
                let end = self.pos;
                (StringLiteral(string), end, None)
            }

            ':' => {
                let kind = if self.peek_char() == Some(&'=') {
                    self.advance();
                    ColonEqual
                } else {
                    Colon
                };
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (kind, end, Some(lexeme_str))
            }

            '=' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (Equal, end, Some(lexeme_str))
            }

            '+' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (Plus, end, Some(lexeme_str))
            }

            '-' => {
                let kind = if self.peek_char() == Some(&'>') {
                    self.advance();
                    Implies
                } else {
                    Minus
                };
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (kind, end, Some(lexeme_str))
            }

            '<' => {
                let kind = if self.peek_char() == Some(&'-') {
                    let mut clone = self.chars.clone();
                    clone.next();
                    if clone.peek() == Some(&'>') {
                        self.advance(); // consume '-'
                        self.advance(); // consume '>'
                        Iff
                    } else {
                        Unknown
                    }
                } else {
                    Unknown
                };
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (kind, end, Some(lexeme_str))
            }

            '*' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (Star, end, Some(lexeme_str))
            }

            '.' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (Dot, end, Some(lexeme_str))
            }

            ',' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (Comma, end, Some(lexeme_str))
            }

            '(' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (LParen, end, Some(lexeme_str))
            }

            ')' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (RParen, end, Some(lexeme_str))
            }

            '¬' | '!' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (Not, end, Some(lexeme_str))
            }

            '∧' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (And, end, Some(lexeme_str))
            }

            '∨' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (Or, end, Some(lexeme_str))
            }

            '→' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (Implies, end, Some(lexeme_str))
            }

            '↔' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (Iff, end, Some(lexeme_str))
            }

            '∀' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (ForAll, end, Some(lexeme_str))
            }

            '∃' => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (Exists, end, Some(lexeme_str))
            }

            EOF => (Eof, self.pos, None),

            _ => {
                let end = self.pos;
                let lexeme_str = std::str::from_utf8(&self.text[start_pos..end]).unwrap();
                (Unknown, end, Some(lexeme_str))
            }
        };

        let span = Span::new(self.file_id, start_pos as u32, end_pos as u32);
        let tok = Token {
            kind: token_type,
            lexeme,
            position: start_pos,
            source_id: self.file_id.0 as usize,
            span,
        };

        self.current = Some(tok.clone());
        Some(tok)
    }

    pub fn peek_token(&mut self) -> Option<&Token<'a>> {
        if self.peeked.is_none() {
            self.peeked = self.next_token();
        }
        self.peeked.as_ref()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    #[inline]
    fn peek_char(&mut self) -> Option<&char> {
        self.chars.peek()
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

        match ident.as_str() {
            "theorem" => TokenType::Theorem,
            "proof" => TokenType::Proof,
            "end" => TokenType::End,
            "take" => TokenType::Take,
            "suppose" => TokenType::Suppose,
            "let" => TokenType::Let,
            "have" => TokenType::Have,
            "construct" => TokenType::Construct,
            "show" => TokenType::Show,
            "derive" => TokenType::Derive,
            "use" => TokenType::Use,
            "therefore" => TokenType::Therefore,
            "choose" => TokenType::Choose,
            "cases" => TokenType::Cases,
            "contradict" => TokenType::Contradict,
            "figure" => TokenType::Figure,
            "given" => TokenType::Given,
            "from" => TokenType::From,
            "as" => TokenType::As,
            "using" => TokenType::Using,
            "forall" => TokenType::ForAll,
            "exists" => TokenType::Exists,
            "and" => TokenType::And,
            "or" => TokenType::Or,
            "not" => TokenType::Not,
            _ => TokenType::Ident(ident),
        }
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
    fn is_ident_start(c: char) -> bool {
        c.is_alphabetic() || c == '_'
    }

    #[inline]
    fn is_ident_part(c: char) -> bool {
        c.is_alphanumeric() || c == '_'
    }
}

