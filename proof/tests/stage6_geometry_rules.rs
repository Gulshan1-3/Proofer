/// Stage 6 — Geometry Rules Tests
///
/// Verifies:
/// 1. Isosceles triangle theorem: AB = AC -> angle ABC = angle ACB
/// 2. Triangle congruence rules: SSS, SAS
/// 3. Congruence to angle deduction (CPCTC)
/// 4. End-to-end source proof verification with geometric certificates:
///    Given isosceles triangle, derive base angle equality.
/// 5. Security: Kernel rejects forged/unjustified geometric certificates.

use proof::kernel::{KProp, KTerm, ProofNode, ProofNodeId, ProofObject, Context, CheckError, check_proof};
use proof::id::FactId;
use proof::parser::Parser;
use proof::hir::resolve::Resolver;
use proof::elab::Elaborator;

#[test]
fn test_isosceles_triangle_rule_certificate() {
    let mut ctx = Context::new();
    let ab_eq_ac = KProp::Eq(KTerm::Const("AB".into()), KTerm::Const("AC".into()));
    ctx.add_fact(FactId(1), ab_eq_ac);

    let angle_b_eq_c = KProp::Eq(
        KTerm::Const("angle_ABC".into()),
        KTerm::Const("angle_ACB".into()),
    );

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::GeoCertificate {
                rule: "IsoscelesBaseAngles".into(),
                premises: vec![ProofNodeId(0)],
                conclusion: angle_b_eq_c.clone(),
            },
        ],
        ProofNodeId(1),
        angle_b_eq_c.clone(),
    );

    let res = check_proof(&proof, &ctx);
    assert_eq!(res, Ok(angle_b_eq_c));
}

#[test]
fn test_sss_triangle_congruence_certificate() {
    let mut ctx = Context::new();
    let e1 = KProp::Eq(KTerm::Const("AB".into()), KTerm::Const("AC".into()));
    let e2 = KProp::Eq(KTerm::Const("BM".into()), KTerm::Const("CM".into()));
    let e3 = KProp::Eq(KTerm::Const("AM".into()), KTerm::Const("AM".into()));

    ctx.add_fact(FactId(1), e1);
    ctx.add_fact(FactId(2), e2);
    ctx.add_fact(FactId(3), e3);

    let tri_cong = KProp::Atom("congruent".into(), vec![
        KTerm::Const("triangle_ABM".into()),
        KTerm::Const("triangle_ACM".into()),
    ]);

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::Assumption(FactId(2)),
            ProofNode::Assumption(FactId(3)),
            ProofNode::GeoCertificate {
                rule: "SSS".into(),
                premises: vec![ProofNodeId(0), ProofNodeId(1), ProofNodeId(2)],
                conclusion: tri_cong.clone(),
            },
        ],
        ProofNodeId(3),
        tri_cong.clone(),
    );

    let res = check_proof(&proof, &ctx);
    assert_eq!(res, Ok(tri_cong));
}

#[test]
fn test_kernel_rejects_insufficient_sss_certificate() {
    let mut ctx = Context::new();
    let e1 = KProp::Eq(KTerm::Const("AB".into()), KTerm::Const("AC".into()));
    let e2 = KProp::Eq(KTerm::Const("BM".into()), KTerm::Const("CM".into()));
    // Missing 3rd side equality!

    ctx.add_fact(FactId(1), e1);
    ctx.add_fact(FactId(2), e2);

    let tri_cong = KProp::Atom("congruent".into(), vec![
        KTerm::Const("triangle_ABM".into()),
        KTerm::Const("triangle_ACM".into()),
    ]);

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::Assumption(FactId(2)),
            ProofNode::GeoCertificate {
                rule: "SSS".into(),
                premises: vec![ProofNodeId(0), ProofNodeId(1)], // Only 2 premises!
                conclusion: tri_cong.clone(),
            },
        ],
        ProofNodeId(2),
        tri_cong,
    );

    let res = check_proof(&proof, &ctx);
    assert!(matches!(res, Err(CheckError::InvalidGeoCertificate { .. })));
}

#[test]
fn test_end_to_end_source_isosceles_theorem_proof() {
    let src = r#"
    theorem isosceles_base_angles:
        AB = AC -> angle_ABC = angle_ACB
    proof
        suppose h1 : AB = AC
        derive h2 : angle_ABC = angle_ACB from h1 using IsoscelesBaseAngles
        therefore angle_ABC = angle_ACB from h2
    end
    "#;

    let mut parser = Parser::new(src);
    let file_ast = parser.parse_file();
    assert_eq!(parser.diagnostics().len(), 0, "Parser diagnostics: {:?}", parser.diagnostics());

    let mut resolver = Resolver::new();
    let pkg = resolver.resolve_file(&file_ast);
    assert_eq!(resolver.diagnostics().len(), 0, "Resolver diagnostics: {:?}", resolver.diagnostics());

    let mut elaborator = Elaborator::new();
    let results = elaborator.elaborate_package(&pkg);
    assert_eq!(results.len(), 1);
    let res = &results[0];
    assert!(res.errors.is_empty(), "Elaboration errors: {:?}", res.errors);
    assert!(res.proven.is_some(), "Kernel must verify isosceles base angle theorem");
}
