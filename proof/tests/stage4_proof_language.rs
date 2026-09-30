/// Stage 4 — Proof Language Elaboration to Kernel tests
///
/// Verifies end-to-end integration:
/// Source -> Parser -> Resolver (HIR) -> Elaborator -> Kernel Proof Object -> Kernel Verification.

use proof::parser::Parser;
use proof::hir::resolve::Resolver;
use proof::elab::Elaborator;
use proof::kernel::KProp;

#[test]
fn test_elaborate_and_verify_identity_theorem() {
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
    assert_eq!(parser.diagnostics().len(), 0, "Parser diagnostics should be empty");

    let mut resolver = Resolver::new();
    let pkg = resolver.resolve_file(&file_ast);
    assert_eq!(resolver.diagnostics().len(), 0, "Resolver diagnostics should be empty");

    let mut elaborator = Elaborator::new();
    let results = elaborator.elaborate_package(&pkg);
    assert_eq!(results.len(), 1);
    let res = &results[0];
    assert_eq!(res.name, "identity");
    assert!(res.errors.is_empty(), "Elaboration should succeed: {:?}", res.errors);
    assert!(res.proven.is_some(), "Kernel should verify identity theorem");

    if let Some(KProp::ForAll(binder, body)) = &res.proven {
        assert_eq!(binder.name, "P");
        assert!(matches!(&**body, KProp::Implies(_, _)));
    } else {
        panic!("Expected quantified implication, got {:?}", res.proven);
    }
}

#[test]
fn test_elaborate_simple_implication() {
    let src = r#"
    theorem imp_self:
        P -> P
    proof
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

    let mut elaborator = Elaborator::new();
    let results = elaborator.elaborate_package(&pkg);
    assert_eq!(results.len(), 1);
    let res = &results[0];
    assert!(res.errors.is_empty(), "Expected success: {:?}", res.errors);
    assert!(res.proven.is_some());
}

#[test]
fn test_elaborate_modus_ponens_pipeline() {
    let src = r#"
    theorem mp_pipeline:
        (P -> Q) -> P -> Q
    proof
        suppose h1 : P -> Q
        suppose h2 : P
        have h3 : Q from h1, h2
        therefore Q from h3
    end
    "#;
    let mut parser = Parser::new(src);
    let file_ast = parser.parse_file();
    assert_eq!(parser.diagnostics().len(), 0, "Parser diagnostics: {:?}", parser.diagnostics());

    let mut resolver = Resolver::new();
    let pkg = resolver.resolve_file(&file_ast);
    assert_eq!(resolver.diagnostics().len(), 0);

    let mut elaborator = Elaborator::new();
    let results = elaborator.elaborate_package(&pkg);
    assert_eq!(results.len(), 1);
    let res = &results[0];
    assert!(res.errors.is_empty(), "Expected success: {:?}", res.errors);
    assert!(res.proven.is_some());
}

#[test]
fn test_kernel_rejects_invalid_conclusion_from_source() {
    // Theorem states P -> Q, but only supposes P and concludes P
    let src = r#"
    theorem invalid_claim:
        P -> Q
    proof
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

    let mut elaborator = Elaborator::new();
    let results = elaborator.elaborate_package(&pkg);
    assert_eq!(results.len(), 1);
    let res = &results[0];
    assert!(res.proven.is_none(), "Kernel MUST reject invalid conclusion");
    assert!(!res.errors.is_empty(), "Should report kernel rejection error");
}
