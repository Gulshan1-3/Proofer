mod lexer;
mod token;

use lexer::prelude::*;
use token::prelude::*;

fn main() {
    let input = r#"
        theorem my_theorem
        proof
        ∀x ∃y (x = y + 1)
        42 := 3.14
        "hello"
    "#;

    let mut lexer = Lexer::new(input);
    println!("--- Tokens ---");

    while let Some(token) = lexer.next_token() {
        println!("{}", token);
        if matches!(token.kind, TokenType::Eof) {
            break;
        }
    }

    // Direct TokenType usage tests to prevent warnings
    let _ = TokenType::And;
    let _ = TokenType::Or;
    let _ = TokenType::Implies;
    let _ = TokenType::Equal;
    let _ = TokenType::Plus;
    let _ = TokenType::Minus;
    let _ = TokenType::ColonEqual;
    let _ = TokenType::StringLiteral("example".to_string());
    let _ = TokenType::Ident("x".to_string());
    let _ = TokenType::Number(123);

    // Test Token display and comparison
    let t1 = Token::with_lexeme(TokenType::Ident("foo".into()), "foo");
    let t2 = Token::with_lexeme(TokenType::Ident("bar".into()), "bar");

    println!("\n--- Token Display & Comparison ---");
    println!("t1: {}", t1);
    println!("t2: {}", t2);
    println!("t1 == t2? {}", t1 == t2);
    println!("t1 < t2? {}", t1 < t2);

    // Hashing check (to trigger Hash implementation)
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(t1);
    set.insert(t2);
    println!("\nToken set size: {}", set.len());
}
