use proof::ast::*;
use proof::parser::Parser;
use proof::syntax::span::FileId;

#[test]
fn test_parse_simple_proposition() {
    let src = "P";
    let mut parser = Parser::new(src);
    let prop = parser.parse_proposition().expect("Should parse");
    assert!(matches!(prop.kind, PropositionKind::Atomic(name, args) if name == "P" && args.is_empty()));
    assert_eq!(parser.diagnostics().len(), 0);
}

#[test]
fn test_parse_nested_implication() {
    let src = "P -> Q -> R";
    let mut parser = Parser::new(src);
    let prop = parser.parse_proposition().expect("Should parse");
    // Right associative: P -> (Q -> R)
    match prop.kind {
        PropositionKind::Implies(p, right) => {
            assert!(matches!(p.kind, PropositionKind::Atomic(n, _) if n == "P"));
            match right.kind {
                PropositionKind::Implies(q, r) => {
                    assert!(matches!(q.kind, PropositionKind::Atomic(n, _) if n == "Q"));
                    assert!(matches!(r.kind, PropositionKind::Atomic(n, _) if n == "R"));
                }
                _ => panic!("Expected nested implication on right"),
            }
        }
        _ => panic!("Expected implication"),
    }
}

#[test]
fn test_parse_conjunction_and_disjunction() {
    // ∧ binds tighter than ∨, which binds tighter than →
    let src = "P and Q or R -> S";
    let mut parser = Parser::new(src);
    let prop = parser.parse_proposition().expect("Should parse");

    match prop.kind {
        PropositionKind::Implies(left, s) => {
            assert!(matches!(s.kind, PropositionKind::Atomic(n, _) if n == "S"));
            match left.kind {
                PropositionKind::Or(p_and_q, r) => {
                    assert!(matches!(r.kind, PropositionKind::Atomic(n, _) if n == "R"));
                    assert!(matches!(p_and_q.kind, PropositionKind::And(_, _)));
                }
                _ => panic!("Expected Or on left"),
            }
        }
        _ => panic!("Expected Implies at top level"),
    }
}

#[test]
fn test_parse_quantifiers() {
    let src = "forall x : Type, exists y : Prop, x = y";
    let mut parser = Parser::new(src);
    let prop = parser.parse_proposition().expect("Should parse");

    match prop.kind {
        PropositionKind::ForAll(binders, inner) => {
            assert_eq!(binders.len(), 1);
            assert_eq!(binders[0].name, "x");
            match inner.kind {
                PropositionKind::Exists(y_binders, eq) => {
                    assert_eq!(y_binders.len(), 1);
                    assert_eq!(y_binders[0].name, "y");
                    assert!(matches!(eq.kind, PropositionKind::Equal(_, _)));
                }
                _ => panic!("Expected Exists inside ForAll"),
            }
        }
        _ => panic!("Expected ForAll"),
    }
}

#[test]
fn test_parse_geometry_declaration() {
    let src = r#"
    figure triangle_example
        triangle ABC
        given AB = AC
    end
    "#;
    let mut parser = Parser::new(src);
    let fig = parser.parse_figure().expect("Should parse figure");
    assert_eq!(fig.name, "triangle_example");
    assert_eq!(fig.items.len(), 2);
    match &fig.items[0] {
        FigureItem::Object { kind, name, .. } => {
            assert_eq!(kind, "triangle");
            assert_eq!(name, "ABC");
        }
        _ => panic!("Expected object item"),
    }
    match &fig.items[1] {
        FigureItem::Given { prop, .. } => {
            assert!(matches!(prop.kind, PropositionKind::Equal(_, _)));
        }
        _ => panic!("Expected given item"),
    }
}

#[test]
fn test_parse_proof_blocks() {
    let src = r#"
    theorem identity:
        forall P : Prop, P -> P
    proof
        take P : Prop
        suppose h : P
        therefore P from h
    end
    "#;
    let mut parser = Parser::new_with_file(src, FileId(42));
    let thm = parser.parse_theorem().expect("Should parse theorem");
    assert_eq!(thm.name, "identity");
    assert_eq!(thm.proof.len(), 3);
    assert!(matches!(thm.proof[0].kind, ProofStepKind::Take(_)));
    assert!(matches!(thm.proof[1].kind, ProofStepKind::Suppose { .. }));
    assert!(matches!(thm.proof[2].kind, ProofStepKind::Therefore { .. }));
}

#[test]
fn test_parse_malformed_input_recovery() {
    let src = r#"
    theorem bad1
        ??? 
    proof
        have h : P
    end

    theorem good
        P -> P
    proof
        therefore P
    end
    "#;
    let mut parser = Parser::new(src);
    let file_ast = parser.parse_file();
    // Diagnostics should report error
    assert!(!parser.diagnostics().is_empty());
    // But recovery should allow 'good' theorem to be parsed
    let good_thm = file_ast.items.iter().find_map(|item| match item {
        Item::Theorem(t) if t.name == "good" => Some(t),
        _ => None,
    });
    assert!(good_thm.is_some(), "Parser should have recovered and parsed the second theorem");
}
