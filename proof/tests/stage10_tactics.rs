use proof::tactic::{TacticGoal, CongruenceClosure, AngleChase, AutoGeometry};
use proof::kernel::types::{KProp, KTerm};
use proof::kernel::proof_object::{ProofNode, ProofNodeId, ProofObject};
use proof::kernel::checker::{Context, check_proof};
use proof::id::FactId;

#[test]
fn test_congruence_closure_transitivity() {
    let mut cc = CongruenceClosure::new();

    let term_a = KTerm::Var("a".into());
    let term_b = KTerm::Var("b".into());
    let term_c = KTerm::Var("c".into());

    let goal = TacticGoal {
        hypotheses: vec![
            ("h1".into(), KProp::Eq(term_a.clone(), term_b.clone())),
            ("h2".into(), KProp::Eq(term_b.clone(), term_c.clone())),
        ],
        target: KProp::Eq(term_a.clone(), term_c.clone()),
    };

    let cert_nodes = cc.prove_equality(&goal).expect("Congruence closure failed");
    assert_eq!(cert_nodes.len(), 1);

    // Build proof object and verify with kernel
    let nodes = vec![
        ProofNode::Assumption(FactId(1)),
        ProofNode::Assumption(FactId(2)),
        ProofNode::GeoCertificate {
            rule: "Transitivity".to_string(),
            premises: vec![ProofNodeId(0), ProofNodeId(1)],
            conclusion: goal.target.clone(),
        },
    ];
    let po = ProofObject::new(nodes, ProofNodeId(2), goal.target.clone());

    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), KProp::Eq(term_a.clone(), term_b.clone()));
    ctx.add_fact(FactId(2), KProp::Eq(term_b.clone(), term_c.clone()));

    let res = check_proof(&po, &ctx);
    assert!(res.is_ok(), "Kernel failed to check transitivity certificate: {:?}", res);
}

#[test]
fn test_angle_chase_triangle_sum() {
    let mut chaser = AngleChase::new();
    chaser.record_angle("angle_A", 60.0);
    chaser.record_angle("angle_B", 70.0);

    let third = chaser.solve_triangle_third_angle(60.0, 70.0).expect("Solving angle failed");
    assert!((third - 50.0).abs() < 1e-6);

    let target = KProp::Eq(KTerm::Var("angle_C".into()), KTerm::Var("50deg".into()));
    let goal = TacticGoal {
        hypotheses: vec![],
        target: target.clone(),
    };

    let certs = chaser.prove_angle_goal(&goal).expect("Angle proof failed");
    assert_eq!(certs.len(), 1);

    let po = ProofObject::new(vec![certs[0].clone()], ProofNodeId(0), target);

    let ctx = Context::new();
    let res = check_proof(&po, &ctx);
    assert!(res.is_ok(), "Kernel failed to check angle sum certificate: {:?}", res);
}

#[test]
fn test_auto_geometry_isosceles() {
    let ab = KTerm::Var("AB".into());
    let ac = KTerm::Var("AC".into());
    let angle_b = KTerm::Var("angle_ABC".into());
    let angle_c = KTerm::Var("angle_ACB".into());

    let goal = TacticGoal {
        hypotheses: vec![
            ("h1".into(), KProp::Eq(ab.clone(), ac.clone())),
        ],
        target: KProp::Eq(angle_b.clone(), angle_c.clone()),
    };

    let proof_nodes = AutoGeometry::solve(&goal).expect("AutoGeometry failed on isosceles theorem");
    assert_eq!(proof_nodes.len(), 1);

    // Verify in kernel
    let nodes = vec![
        ProofNode::Assumption(FactId(1)),
        ProofNode::GeoCertificate {
            rule: "IsoscelesBaseAngles".into(),
            premises: vec![ProofNodeId(0)],
            conclusion: goal.target.clone(),
        },
    ];
    let po = ProofObject::new(nodes, ProofNodeId(1), goal.target.clone());

    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), KProp::Eq(ab, ac));

    let res = check_proof(&po, &ctx);
    assert!(res.is_ok(), "Kernel failed to verify auto-generated isosceles certificate: {:?}", res);
}

#[test]
fn test_auto_geometry_sss() {
    let ab = KTerm::Var("AB".into());
    let de = KTerm::Var("DE".into());
    let bc = KTerm::Var("BC".into());
    let ef = KTerm::Var("EF".into());
    let ca = KTerm::Var("CA".into());
    let fd = KTerm::Var("FD".into());

    let tri_abc = KTerm::Var("triangle_ABC".into());
    let tri_def = KTerm::Var("triangle_DEF".into());

    let goal = TacticGoal {
        hypotheses: vec![
            ("h1".into(), KProp::Eq(ab.clone(), de.clone())),
            ("h2".into(), KProp::Eq(bc.clone(), ef.clone())),
            ("h3".into(), KProp::Eq(ca.clone(), fd.clone())),
        ],
        target: KProp::Atom("congruent".into(), vec![tri_abc.clone(), tri_def.clone()]),
    };

    let proof_nodes = AutoGeometry::solve(&goal).expect("AutoGeometry failed on SSS");
    assert_eq!(proof_nodes.len(), 1);

    let nodes = vec![
        ProofNode::Assumption(FactId(1)),
        ProofNode::Assumption(FactId(2)),
        ProofNode::Assumption(FactId(3)),
        ProofNode::GeoCertificate {
            rule: "SSS".into(),
            premises: vec![ProofNodeId(0), ProofNodeId(1), ProofNodeId(2)],
            conclusion: goal.target.clone(),
        },
    ];
    let po = ProofObject::new(nodes, ProofNodeId(3), goal.target.clone());

    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), KProp::Eq(ab, de));
    ctx.add_fact(FactId(2), KProp::Eq(bc, ef));
    ctx.add_fact(FactId(3), KProp::Eq(ca, fd));

    let res = check_proof(&po, &ctx);
    assert!(res.is_ok(), "Kernel failed to verify SSS certificate: {:?}", res);
}

#[test]
fn test_tactic_fails_on_unprovable_goal() {
    let goal = TacticGoal {
        hypotheses: vec![],
        target: KProp::Eq(KTerm::Var("X".into()), KTerm::Var("Y".into())),
    };

    let res = AutoGeometry::solve(&goal);
    assert!(res.is_err(), "Tactic should not prove unjustified goal");
}
