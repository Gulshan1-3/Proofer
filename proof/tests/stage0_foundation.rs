use proof::syntax::span::{FileId, Span};
use proof::syntax::source::SourceMap;
use proof::diag::Diagnostic;
use proof::id::{IdGen, FactId, TheoremId};
use proof::lexer::Lexer;
use proof::token::TokenType;

#[test]
fn test_span_boundaries_and_merge() {
    let s1 = Span::new(FileId(1), 10, 20);
    let s2 = Span::new(FileId(1), 25, 30);
    assert_eq!(s1.len(), 10);
    assert_eq!(s2.len(), 5);

    let merged = s1.merge(s2);
    assert_eq!(merged.start, 10);
    assert_eq!(merged.end, 30);
    assert_eq!(merged.file, FileId(1));
    assert!(merged.contains_pos(15));
    assert!(merged.contains_pos(25));
    assert!(!merged.contains_pos(35));
}

#[test]
fn test_unicode_source_spans() {
    let mut sources = SourceMap::new();
    let text = "theorem ∀x ∃y. P(x) → Q(y)";
    let file = sources.add_file("logic.proof", text);

    let mut lexer = Lexer::new_with_file(text, file);
    let mut tokens = Vec::new();
    while let Some(tok) = lexer.next_token() {
        if tok.kind == TokenType::Eof {
            break;
        }
        tokens.push(tok);
    }

    assert_eq!(tokens[0].kind, TokenType::Theorem);
    let file_obj = sources.get(file).unwrap();
    assert_eq!(file_obj.slice(tokens[0].span), "theorem");

    // Check ∀
    let forall_tok = &tokens[1];
    assert_eq!(forall_tok.kind, TokenType::ForAll);
    assert_eq!(file_obj.slice(forall_tok.span), "∀");
    assert_eq!(forall_tok.span.len(), 3); // '∀' is 3 bytes in UTF-8
}

#[test]
fn test_diagnostic_rendering() {
    let mut sources = SourceMap::new();
    let src = "line 1\nline 2 with error\nline 3";
    let file = sources.add_file("test.proof", src);

    // Span for "error" in line 2
    // "line 1\n" is 7 bytes
    // "line 2 with " is 12 bytes
    // "error" starts at 19, ends at 24
    let span = Span::new(file, 19, 24);
    let diag = Diagnostic::error("unknown identifier", span)
        .with_note("identifiers must be declared");

    let rendered = diag.render(&sources);
    assert!(rendered.contains("test.proof:2:13: error: unknown identifier"));
    assert!(rendered.contains("line 2 with error"));
    assert!(rendered.contains("^^^^^"));
    assert!(rendered.contains("= note: identifiers must be declared"));
}

#[test]
fn test_id_stability() {
    let mut generator = IdGen::new();
    let thm1 = generator.next_theorem();
    let thm2 = generator.next_theorem();
    let fact1 = generator.next_fact();

    assert_eq!(thm1, TheoremId(1));
    assert_eq!(thm2, TheoremId(2));
    assert_eq!(fact1, FactId(3));

    assert_eq!(thm1.to_string(), "thm#1");
    assert_eq!(fact1.to_string(), "fact#3");
}
