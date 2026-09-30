/// Stage 3 — Kernel tests
///
/// For every inference rule, we test:
///   1. Valid proof object
///   2. Wrong premise
///   3. Wrong conclusion
///   4. Malformed object (invalid node refs, type mismatches)
///
/// The kernel must be testable WITHOUT invoking parser/editor/geometry code.

use proof::id::FactId;
use proof::kernel::{
    KProp, KTerm, KBinder,
    ProofNode, ProofNodeId, ProofObject,
    CheckError, Context, check_proof,
};

// ═══════════════════════════════════════════════════════════════
//  Helpers
// ═══════════════════════════════════════════════════════════════

fn atom(name: &str) -> KProp {
    KProp::Atom(name.to_string(), vec![])
}

fn var(name: &str) -> KTerm {
    KTerm::Var(name.to_string())
}

fn nid(i: usize) -> ProofNodeId {
    ProofNodeId(i)
}

// ═══════════════════════════════════════════════════════════════
//  Assumption
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_assumption_valid() {
    // Context: h1 = P
    // Proof: assume h1
    // Conclusion: P
    let p = atom("P");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p.clone());

    let proof = ProofObject::new(
        vec![ProofNode::Assumption(FactId(1))],
        nid(0),
        p.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(p));
}

#[test]
fn test_assumption_unknown_fact() {
    let p = atom("P");
    let ctx = Context::new(); // empty context

    let proof = ProofObject::new(
        vec![ProofNode::Assumption(FactId(99))],
        nid(0),
        p,
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::UnknownAssumption(FactId(99)))
    );
}

#[test]
fn test_assumption_wrong_conclusion() {
    // Context: h1 = P, but proof claims Q
    let p = atom("P");
    let q = atom("Q");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p.clone());

    let proof = ProofObject::new(
        vec![ProofNode::Assumption(FactId(1))],
        nid(0),
        q.clone(), // WRONG: claims Q but proves P
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::ConclusionMismatch { claimed: q, proven: p })
    );
}

// ═══════════════════════════════════════════════════════════════
//  Implication Introduction & Elimination (Modus Ponens)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_imp_intro_valid() {
    // Prove: P → P (identity)
    // Node 0: assume h1 (P is in context)
    // Node 1: ImpIntro(premise=P, body=node0)
    let p = atom("P");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p.clone());

    let conclusion = KProp::Implies(Box::new(p.clone()), Box::new(p.clone()));
    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),         // node 0: P
            ProofNode::ImpIntro { premise: p.clone(), body: nid(0) }, // node 1: P → P
        ],
        nid(1),
        conclusion.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(conclusion));
}

#[test]
fn test_modus_ponens_valid() {
    // Context: h1 = P → Q, h2 = P
    // Prove: Q via ImpElim
    let p = atom("P");
    let q = atom("Q");
    let imp_pq = KProp::Implies(Box::new(p.clone()), Box::new(q.clone()));

    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), imp_pq.clone());
    ctx.add_fact(FactId(2), p.clone());

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),       // node 0: P → Q
            ProofNode::Assumption(FactId(2)),       // node 1: P
            ProofNode::ImpElim { imp: nid(0), arg: nid(1) }, // node 2: Q
        ],
        nid(2),
        q.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(q));
}

#[test]
fn test_modus_ponens_wrong_premise() {
    // Context: h1 = P → Q, h2 = R (should be P!)
    let p = atom("P");
    let q = atom("Q");
    let r = atom("R");
    let imp_pq = KProp::Implies(Box::new(p.clone()), Box::new(q.clone()));

    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), imp_pq);
    ctx.add_fact(FactId(2), r.clone());

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::Assumption(FactId(2)),
            ProofNode::ImpElim { imp: nid(0), arg: nid(1) },
        ],
        nid(2),
        q,
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::PremiseMismatch { expected: p, got: r })
    );
}

#[test]
fn test_imp_elim_not_implication() {
    // Try to apply ImpElim to a non-implication
    let p = atom("P");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p.clone()); // P is not an implication!
    ctx.add_fact(FactId(2), p.clone());

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::Assumption(FactId(2)),
            ProofNode::ImpElim { imp: nid(0), arg: nid(1) },
        ],
        nid(2),
        p.clone(),
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::NotAnImplication(p))
    );
}

// ═══════════════════════════════════════════════════════════════
//  Conjunction (And)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_and_intro_valid() {
    let p = atom("P");
    let q = atom("Q");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p.clone());
    ctx.add_fact(FactId(2), q.clone());

    let conclusion = KProp::And(Box::new(p.clone()), Box::new(q.clone()));
    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::Assumption(FactId(2)),
            ProofNode::AndIntro { left: nid(0), right: nid(1) },
        ],
        nid(2),
        conclusion.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(conclusion));
}

#[test]
fn test_and_elim_left_valid() {
    let p = atom("P");
    let q = atom("Q");
    let pq = KProp::And(Box::new(p.clone()), Box::new(q.clone()));
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), pq);

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::AndElimLeft(nid(0)),
        ],
        nid(1),
        p.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(p));
}

#[test]
fn test_and_elim_right_valid() {
    let p = atom("P");
    let q = atom("Q");
    let pq = KProp::And(Box::new(p.clone()), Box::new(q.clone()));
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), pq);

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::AndElimRight(nid(0)),
        ],
        nid(1),
        q.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(q));
}

#[test]
fn test_and_elim_not_conjunction() {
    let p = atom("P");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p.clone()); // not a conjunction!

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::AndElimLeft(nid(0)),
        ],
        nid(1),
        p.clone(),
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::NotAConjunction(p))
    );
}

// ═══════════════════════════════════════════════════════════════
//  Disjunction (Or)
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_or_intro_left_valid() {
    let p = atom("P");
    let q = atom("Q");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p.clone());

    let conclusion = KProp::Or(Box::new(p.clone()), Box::new(q.clone()));
    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::OrIntroLeft { proof: nid(0), right_prop: q.clone() },
        ],
        nid(1),
        conclusion.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(conclusion));
}

#[test]
fn test_or_intro_right_valid() {
    let p = atom("P");
    let q = atom("Q");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), q.clone());

    let conclusion = KProp::Or(Box::new(p.clone()), Box::new(q.clone()));
    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::OrIntroRight { proof: nid(0), left_prop: p.clone() },
        ],
        nid(1),
        conclusion.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(conclusion));
}

#[test]
fn test_or_elim_valid() {
    // From P ∨ Q, P ⊢ R, Q ⊢ R → R
    let p = atom("P");
    let q = atom("Q");
    let r = atom("R");
    let p_or_q = KProp::Or(Box::new(p.clone()), Box::new(q.clone()));

    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p_or_q.clone());
    ctx.add_fact(FactId(2), r.clone()); // proof of R (left case)
    ctx.add_fact(FactId(3), r.clone()); // proof of R (right case)

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),  // P ∨ Q
            ProofNode::Assumption(FactId(2)),  // R (left case)
            ProofNode::Assumption(FactId(3)),  // R (right case)
            ProofNode::OrElim {
                disjunction: nid(0),
                left_case: nid(1),
                right_case: nid(2),
            },
        ],
        nid(3),
        r.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(r));
}

#[test]
fn test_or_elim_case_mismatch() {
    let p = atom("P");
    let q = atom("Q");
    let r = atom("R");
    let s = atom("S");
    let p_or_q = KProp::Or(Box::new(p.clone()), Box::new(q.clone()));

    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p_or_q);
    ctx.add_fact(FactId(2), r.clone()); // left case proves R
    ctx.add_fact(FactId(3), s.clone()); // right case proves S — MISMATCH!

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::Assumption(FactId(2)),
            ProofNode::Assumption(FactId(3)),
            ProofNode::OrElim {
                disjunction: nid(0),
                left_case: nid(1),
                right_case: nid(2),
            },
        ],
        nid(3),
        r.clone(),
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::CaseMismatch { left: r, right: s })
    );
}

#[test]
fn test_or_elim_not_disjunction() {
    let p = atom("P");
    let r = atom("R");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p.clone()); // not a disjunction!
    ctx.add_fact(FactId(2), r.clone());
    ctx.add_fact(FactId(3), r.clone());

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::Assumption(FactId(2)),
            ProofNode::Assumption(FactId(3)),
            ProofNode::OrElim {
                disjunction: nid(0),
                left_case: nid(1),
                right_case: nid(2),
            },
        ],
        nid(3),
        r,
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::NotADisjunction(p))
    );
}

// ═══════════════════════════════════════════════════════════════
//  Universal Quantifier
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_forall_intro_elim_valid() {
    // Prove: ∀x. P(x) → P(a) via ForAllIntro then ForAllElim
    let binder = KBinder { name: "x".into(), sort: "Type".into() };
    let px = KProp::Atom("P".into(), vec![var("x")]);
    let pa = KProp::Atom("P".into(), vec![var("a")]);
    let forall_px = KProp::ForAll(binder.clone(), Box::new(px.clone()));

    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), px.clone()); // P(x)

    // Build: ForAllIntro(x, assumption P(x)) → ∀x. P(x)
    //        ForAllElim(above, a)            → P(a)
    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),       // node 0: P(x)
            ProofNode::ForAllIntro { binder: binder.clone(), body: nid(0) }, // node 1: ∀x. P(x)
            ProofNode::ForAllElim { proof: nid(1), term: var("a") },         // node 2: P(a)
        ],
        nid(2),
        pa.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(pa));
}

#[test]
fn test_forall_elim_not_forall() {
    let p = atom("P");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p.clone()); // not ∀!

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::ForAllElim { proof: nid(0), term: var("a") },
        ],
        nid(1),
        p.clone(),
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::NotAForAll(p))
    );
}

// ═══════════════════════════════════════════════════════════════
//  Equality
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_eq_refl_valid() {
    let t = var("a");
    let conclusion = KProp::Eq(t.clone(), t.clone());
    let ctx = Context::new();

    let proof = ProofObject::new(
        vec![ProofNode::EqRefl(t)],
        nid(0),
        conclusion.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(conclusion));
}

#[test]
fn test_eq_refl_wrong_conclusion() {
    let a = var("a");
    let b = var("b");
    let ctx = Context::new();

    let proof = ProofObject::new(
        vec![ProofNode::EqRefl(a.clone())],
        nid(0),
        KProp::Eq(a.clone(), b.clone()), // claims a=b but EqRefl gives a=a
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::ConclusionMismatch {
            claimed: KProp::Eq(a.clone(), b),
            proven: KProp::Eq(a.clone(), a),
        })
    );
}

#[test]
fn test_eq_subst_valid() {
    // From a = b and P(a), derive P(b).
    let a = var("a");
    let b = var("b");
    let pa = KProp::Atom("P".into(), vec![a.clone()]);
    let pb = KProp::Atom("P".into(), vec![b.clone()]);
    let eq_ab = KProp::Eq(a.clone(), b.clone());
    // Template: P(z) where z is the substitution variable
    let template = KProp::Atom("P".into(), vec![var("z")]);

    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), eq_ab);
    ctx.add_fact(FactId(2), pa);

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),  // a = b
            ProofNode::Assumption(FactId(2)),  // P(a)
            ProofNode::EqSubst {
                equality: nid(0),
                target: nid(1),
                var: "z".into(),
                prop_template: template,
            },
        ],
        nid(2),
        pb.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(pb));
}

#[test]
fn test_eq_subst_mismatch() {
    // From a = b and Q(a) (not P(a)!), try EqSubst with template P(z) — should fail
    let a = var("a");
    let b = var("b");
    let qa = KProp::Atom("Q".into(), vec![a.clone()]);
    let eq_ab = KProp::Eq(a.clone(), b.clone());
    let template = KProp::Atom("P".into(), vec![var("z")]);

    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), eq_ab);
    ctx.add_fact(FactId(2), qa.clone());

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::Assumption(FactId(2)),
            ProofNode::EqSubst {
                equality: nid(0),
                target: nid(1),
                var: "z".into(),
                prop_template: template.clone(),
            },
        ],
        nid(2),
        atom("whatever"),
    );
    let pa = KProp::Atom("P".into(), vec![a]);
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::SubstitutionMismatch { expected: pa, got: qa })
    );
}

#[test]
fn test_eq_subst_not_equality() {
    let p = atom("P");
    let template = KProp::Atom("Q".into(), vec![var("z")]);
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p.clone()); // not an equality!
    ctx.add_fact(FactId(2), atom("Q"));

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::Assumption(FactId(2)),
            ProofNode::EqSubst {
                equality: nid(0),
                target: nid(1),
                var: "z".into(),
                prop_template: template,
            },
        ],
        nid(2),
        atom("Q"),
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::NotAnEquality(p))
    );
}

// ═══════════════════════════════════════════════════════════════
//  Negation & Contradiction
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_not_intro_valid() {
    // Prove ¬P: assume P, derive ⊥
    let p = atom("P");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), KProp::False); // ⊥ is available assuming P

    let conclusion = KProp::Not(Box::new(p.clone()));
    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),  // ⊥
            ProofNode::NotIntro { premise: p.clone(), body: nid(0) },
        ],
        nid(1),
        conclusion.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(conclusion));
}

#[test]
fn test_not_intro_body_not_false() {
    // Try NotIntro but body doesn't prove ⊥
    let p = atom("P");
    let q = atom("Q");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), q.clone()); // body proves Q, not ⊥

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::NotIntro { premise: p, body: nid(0) },
        ],
        nid(1),
        atom("whatever"),
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::NotFalse(q))
    );
}

#[test]
fn test_not_elim_valid() {
    // From P and ¬P, derive ⊥
    let p = atom("P");
    let not_p = KProp::Not(Box::new(p.clone()));
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p.clone());
    ctx.add_fact(FactId(2), not_p);

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),  // P
            ProofNode::Assumption(FactId(2)),  // ¬P
            ProofNode::NotElim { proof: nid(0), negation: nid(1) },
        ],
        nid(2),
        KProp::False,
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(KProp::False));
}

#[test]
fn test_not_elim_not_a_negation() {
    let p = atom("P");
    let q = atom("Q");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), p);
    ctx.add_fact(FactId(2), q.clone()); // Q, not ¬P

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::Assumption(FactId(2)),
            ProofNode::NotElim { proof: nid(0), negation: nid(1) },
        ],
        nid(2),
        KProp::False,
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::NotANegation(q))
    );
}

#[test]
fn test_false_elim_valid() {
    // From ⊥, derive anything
    let p = atom("P");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), KProp::False);

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::FalseElim { proof: nid(0), conclusion: p.clone() },
        ],
        nid(1),
        p.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(p));
}

#[test]
fn test_false_elim_not_false() {
    let p = atom("P");
    let q = atom("Q");
    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), q.clone()); // not ⊥!

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::FalseElim { proof: nid(0), conclusion: p },
        ],
        nid(1),
        atom("whatever"),
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::NotFalse(q))
    );
}

// ═══════════════════════════════════════════════════════════════
//  Invalid node references
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_invalid_node_ref() {
    let p = atom("P");
    let ctx = Context::new();

    let proof = ProofObject::new(
        vec![
            ProofNode::ImpElim { imp: nid(99), arg: nid(98) }, // nonexistent nodes
        ],
        nid(0),
        p,
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::InvalidNodeRef(nid(99)))
    );
}

#[test]
fn test_invalid_root_ref() {
    let p = atom("P");
    let ctx = Context::new();

    let proof = ProofObject::new(
        vec![],  // empty arena
        nid(0),  // root points to nothing
        p,
    );
    assert_eq!(
        check_proof(&proof, &ctx),
        Err(CheckError::InvalidNodeRef(nid(0)))
    );
}

// ═══════════════════════════════════════════════════════════════
//  Compound proof: chaining multiple rules
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_compound_proof_chain() {
    // Prove: from (P → Q) and (Q → R) and P, derive R
    // via two modus ponens steps
    let p = atom("P");
    let q = atom("Q");
    let r = atom("R");
    let imp_pq = KProp::Implies(Box::new(p.clone()), Box::new(q.clone()));
    let imp_qr = KProp::Implies(Box::new(q.clone()), Box::new(r.clone()));

    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), imp_pq);
    ctx.add_fact(FactId(2), imp_qr);
    ctx.add_fact(FactId(3), p.clone());

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),                           // node 0: P → Q
            ProofNode::Assumption(FactId(3)),                           // node 1: P
            ProofNode::ImpElim { imp: nid(0), arg: nid(1) },           // node 2: Q
            ProofNode::Assumption(FactId(2)),                           // node 3: Q → R
            ProofNode::ImpElim { imp: nid(3), arg: nid(2) },           // node 4: R
        ],
        nid(4),
        r.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(r));
}

#[test]
fn test_compound_and_then_imp() {
    // From P ∧ Q, prove P → (P ∧ Q)
    let p = atom("P");
    let q = atom("Q");
    let pq = KProp::And(Box::new(p.clone()), Box::new(q.clone()));

    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), pq.clone());

    let conclusion = KProp::Implies(Box::new(p.clone()), Box::new(pq.clone()));
    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),  // node 0: P ∧ Q
            ProofNode::ImpIntro { premise: p.clone(), body: nid(0) }, // node 1: P → (P ∧ Q)
        ],
        nid(1),
        conclusion.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(conclusion));
}

#[test]
fn test_substitution_with_forall() {
    // Given ∀x. P(x) → Q(x), prove P(a) → Q(a) via ForAllElim
    let binder = KBinder { name: "x".into(), sort: "Type".into() };
    let px = KProp::Atom("P".into(), vec![var("x")]);
    let qx = KProp::Atom("Q".into(), vec![var("x")]);
    let body = KProp::Implies(Box::new(px), Box::new(qx));
    let forall = KProp::ForAll(binder, Box::new(body));

    let pa = KProp::Atom("P".into(), vec![var("a")]);
    let qa = KProp::Atom("Q".into(), vec![var("a")]);
    let expected = KProp::Implies(Box::new(pa), Box::new(qa));

    let mut ctx = Context::new();
    ctx.add_fact(FactId(1), forall);

    let proof = ProofObject::new(
        vec![
            ProofNode::Assumption(FactId(1)),
            ProofNode::ForAllElim { proof: nid(0), term: var("a") },
        ],
        nid(1),
        expected.clone(),
    );
    assert_eq!(check_proof(&proof, &ctx), Ok(expected));
}
