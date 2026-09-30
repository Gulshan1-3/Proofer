use proof::hir::resolve::Resolver;
use proof::hir::scope::ScopeStack;
use proof::parser::Parser;
use proof::hir::hir_def::*;

#[test]
fn test_scope_shadowing() {
    let mut scopes = ScopeStack::new();
    let s1 = proof::id::SymbolId(1);
    let s2 = proof::id::SymbolId(2);

    scopes.insert("x", proof::hir::scope::SymbolKind::Variable(s1), proof::syntax::span::Span::DUMMY);
    assert_eq!(
        scopes.resolve("x").unwrap().kind,
        proof::hir::scope::SymbolKind::Variable(s1)
    );

    scopes.enter();
    scopes.insert("x", proof::hir::scope::SymbolKind::Variable(s2), proof::syntax::span::Span::DUMMY);
    assert_eq!(
        scopes.resolve("x").unwrap().kind,
        proof::hir::scope::SymbolKind::Variable(s2)
    );

    scopes.exit();
    assert_eq!(
        scopes.resolve("x").unwrap().kind,
        proof::hir::scope::SymbolKind::Variable(s1)
    );
}

#[test]
fn test_resolve_theorem_and_proof_flow() {
    let src = r#"
    theorem identity:
        forall P : Prop, P -> P
    proof
        take P : Prop
        suppose h : P
        therefore P from h
    end
    "#;
    let mut parser = Parser::new(src);
    let file_ast = parser.parse_file();
    assert_eq!(parser.diagnostics().len(), 0);

    let mut resolver = Resolver::new();
    let pkg = resolver.resolve_file(&file_ast);
    assert_eq!(resolver.diagnostics().len(), 0);

    assert_eq!(pkg.theorems.len(), 1);
    let thm = &pkg.theorems[0];
    assert_eq!(thm.name, "identity");
    assert_eq!(thm.proof.len(), 3);

    // Suppose defines a fact
    let fact_id = match &thm.proof[1].kind {
        HirProofStepKind::Suppose { fact, name, .. } => {
            assert_eq!(name, "h");
            *fact
        }
        _ => panic!("Expected Suppose step"),
    };

    // Therefore resolves reference 'h' to the exact fact_id
    match &thm.proof[2].kind {
        HirProofStepKind::Therefore { from_facts, .. } => {
            assert_eq!(from_facts.len(), 1);
            assert_eq!(from_facts[0], fact_id);
        }
        _ => panic!("Expected Therefore step"),
    }
}

#[test]
fn test_resolve_unknown_reference() {
    let src = r#"
    theorem broken:
        P -> P
    proof
        therefore P from non_existent_fact
    end
    "#;
    let mut parser = Parser::new(src);
    let file_ast = parser.parse_file();

    let mut resolver = Resolver::new();
    let _pkg = resolver.resolve_file(&file_ast);
    assert!(!resolver.diagnostics().is_empty());
    assert!(resolver.diagnostics()[0].message.contains("Unknown fact reference 'non_existent_fact'"));
}
