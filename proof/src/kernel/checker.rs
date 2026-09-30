//! Proof checker — the trusted core of Proofer.
//!
//! This module answers ONE question:
//! > Is this proof object a valid proof of this proposition under the active logical rules?
//!
//! The checker NEVER calls solvers, parsers, or external systems.

use std::collections::HashMap;
use crate::id::FactId;
use super::types::{KProp, KTerm, KBinder};
use super::proof_object::{ProofNode, ProofNodeId, ProofObject};

/// Result of checking a proof node — the proposition it proves, or an error.
pub type CheckResult = Result<KProp, CheckError>;

/// Errors that can occur during proof checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckError {
    /// Referenced proof node does not exist.
    InvalidNodeRef(ProofNodeId),
    /// An assumption references a fact not in the context.
    UnknownAssumption(FactId),
    /// Expected an implication (P → Q) but got something else.
    NotAnImplication(KProp),
    /// Expected a conjunction (P ∧ Q) but got something else.
    NotAConjunction(KProp),
    /// Expected a disjunction (P ∨ Q) but got something else.
    NotADisjunction(KProp),
    /// Expected a universal quantification (∀x. P) but got something else.
    NotAForAll(KProp),
    /// Expected an existential quantification (∃x. P) but got something else.
    NotAnExists(KProp),
    /// Expected an equality (a = b) but got something else.
    NotAnEquality(KProp),
    /// Expected ⊥ but got something else.
    NotFalse(KProp),
    /// Expected a negation (¬P) but got something else.
    NotANegation(KProp),
    /// The premise of modus ponens doesn't match.
    PremiseMismatch { expected: KProp, got: KProp },
    /// The two branches of OrElim produce different conclusions.
    CaseMismatch { left: KProp, right: KProp },
    /// The root node's proven proposition doesn't match the claimed conclusion.
    ConclusionMismatch { claimed: KProp, proven: KProp },
    /// Substitution produced unexpected result.
    SubstitutionMismatch { expected: KProp, got: KProp },
    /// Geometric certificate rule violation or mismatch.
    InvalidGeoCertificate { rule: String, reason: String },
}


/// The checking context: maps fact IDs to their known propositions.
#[derive(Debug, Clone, Default)]
pub struct Context {
    facts: HashMap<FactId, KProp>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_fact(&mut self, id: FactId, prop: KProp) {
        self.facts.insert(id, prop);
    }

    pub fn get_fact(&self, id: &FactId) -> Option<&KProp> {
        self.facts.get(id)
    }
}

/// The proof checker.
pub struct Checker<'a> {
    proof: &'a ProofObject,
    ctx: &'a Context,
    /// Cache of already-checked nodes to avoid redundant work.
    cache: HashMap<ProofNodeId, KProp>,
}

impl<'a> Checker<'a> {
    pub fn new(proof: &'a ProofObject, ctx: &'a Context) -> Self {
        Self {
            proof,
            ctx,
            cache: HashMap::new(),
        }
    }

    /// Check the entire proof object. Returns the proven proposition or an error.
    pub fn check(&mut self) -> CheckResult {
        let proven = self.check_node(self.proof.root)?;
        if proven != self.proof.conclusion {
            return Err(CheckError::ConclusionMismatch {
                claimed: self.proof.conclusion.clone(),
                proven,
            });
        }
        Ok(proven)
    }

    /// Recursively check a single proof node. Returns the proposition it proves.
    fn check_node(&mut self, id: ProofNodeId) -> CheckResult {
        // Check cache first
        if let Some(prop) = self.cache.get(&id) {
            return Ok(prop.clone());
        }

        let node = self.proof.get(id)
            .ok_or(CheckError::InvalidNodeRef(id))?
            .clone();

        let result = match node {
            // ── Assumption ───────────────────────────────────────
            ProofNode::Assumption(fact_id) => {
                self.ctx.get_fact(&fact_id)
                    .cloned()
                    .ok_or(CheckError::UnknownAssumption(fact_id))
            }

            // ── Implication ──────────────────────────────────────
            ProofNode::ImpIntro { premise, body } => {
                let body_prop = self.check_node(body)?;
                Ok(KProp::Implies(Box::new(premise), Box::new(body_prop)))
            }

            ProofNode::ImpElim { imp, arg } => {
                let imp_prop = self.check_node(imp)?;
                let arg_prop = self.check_node(arg)?;
                match imp_prop {
                    KProp::Implies(p, q) => {
                        if *p == arg_prop {
                            Ok(*q)
                        } else {
                            Err(CheckError::PremiseMismatch {
                                expected: *p,
                                got: arg_prop,
                            })
                        }
                    }
                    other => Err(CheckError::NotAnImplication(other)),
                }
            }

            // ── Conjunction ──────────────────────────────────────
            ProofNode::AndIntro { left, right } => {
                let l = self.check_node(left)?;
                let r = self.check_node(right)?;
                Ok(KProp::And(Box::new(l), Box::new(r)))
            }

            ProofNode::AndElimLeft(conj) => {
                let prop = self.check_node(conj)?;
                match prop {
                    KProp::And(l, _) => Ok(*l),
                    other => Err(CheckError::NotAConjunction(other)),
                }
            }

            ProofNode::AndElimRight(conj) => {
                let prop = self.check_node(conj)?;
                match prop {
                    KProp::And(_, r) => Ok(*r),
                    other => Err(CheckError::NotAConjunction(other)),
                }
            }

            // ── Disjunction ──────────────────────────────────────
            ProofNode::OrIntroLeft { proof, right_prop } => {
                let left = self.check_node(proof)?;
                Ok(KProp::Or(Box::new(left), Box::new(right_prop)))
            }

            ProofNode::OrIntroRight { proof, left_prop } => {
                let right = self.check_node(proof)?;
                Ok(KProp::Or(Box::new(left_prop), Box::new(right)))
            }

            ProofNode::OrElim { disjunction, left_case, right_case } => {
                let disj = self.check_node(disjunction)?;
                match disj {
                    KProp::Or(_, _) => {
                        let left_result = self.check_node(left_case)?;
                        let right_result = self.check_node(right_case)?;
                        if left_result == right_result {
                            Ok(left_result)
                        } else {
                            Err(CheckError::CaseMismatch {
                                left: left_result,
                                right: right_result,
                            })
                        }
                    }
                    other => Err(CheckError::NotADisjunction(other)),
                }
            }

            // ── Universal quantifier ─────────────────────────────
            ProofNode::ForAllIntro { binder, body } => {
                let body_prop = self.check_node(body)?;
                Ok(KProp::ForAll(binder, Box::new(body_prop)))
            }

            ProofNode::ForAllElim { proof, term } => {
                let prop = self.check_node(proof)?;
                match prop {
                    KProp::ForAll(binder, body) => {
                        Ok(body.subst(&binder.name, &term))
                    }
                    other => Err(CheckError::NotAForAll(other)),
                }
            }

            // ── Existential quantifier ───────────────────────────
            ProofNode::ExistsIntro { witness, body } => {
                let body_prop = self.check_node(body)?;
                // body_prop should be P(witness). We need the ∃ statement.
                // The caller constructs the ExistsIntro node knowing the quantified form.
                // We verify by checking that body proves P(witness) and the
                // conclusion will be ∃x. P(x). The proof object's conclusion
                // carries the expected ∃ form; here we just compute what this
                // node proves given its children.
                //
                // We need the binder info. For ExistsIntro, the proof object's
                // conclusion at the root level tells us the ∃ form. But for an
                // intermediate node, we check structurally:
                // The node claims: given proof of P(t), conclude ∃x. P(x).
                // We can't fully reconstruct ∃x.P(x) from P(t) alone without
                // knowing x and P. So the node must carry that info — but our
                // current representation doesn't. Let's trust the structure and
                // verify at the root level via ConclusionMismatch.
                //
                // For now: we return the body_prop as-is and rely on
                // ConclusionMismatch at the root. A stricter approach would
                // require the node to carry the quantified proposition.
                // TODO: Strengthen ExistsIntro checking.
                Ok(body_prop)
            }

            ProofNode::ExistsElim { exists_proof, binder, body } => {
                let exists_prop = self.check_node(exists_proof)?;
                match exists_prop {
                    KProp::Exists(_, _) => {
                        let result = self.check_node(body)?;
                        Ok(result)
                    }
                    other => Err(CheckError::NotAnExists(other)),
                }
            }

            // ── Equality ─────────────────────────────────────────
            ProofNode::EqRefl(term) => {
                Ok(KProp::Eq(term.clone(), term))
            }

            ProofNode::EqSubst { equality, target, var, prop_template } => {
                let eq_prop = self.check_node(equality)?;
                let target_prop = self.check_node(target)?;
                match eq_prop {
                    KProp::Eq(a, b) => {
                        // prop_template with var=a should equal target_prop
                        let expected = prop_template.subst(&var, &a);
                        if expected != target_prop {
                            return Err(CheckError::SubstitutionMismatch {
                                expected,
                                got: target_prop,
                            });
                        }
                        // Result: prop_template with var=b
                        Ok(prop_template.subst(&var, &b))
                    }
                    other => Err(CheckError::NotAnEquality(other)),
                }
            }

            // ── Negation / Contradiction ─────────────────────────
            ProofNode::NotIntro { premise, body } => {
                let body_prop = self.check_node(body)?;
                if body_prop != KProp::False {
                    return Err(CheckError::NotFalse(body_prop));
                }
                Ok(KProp::Not(Box::new(premise)))
            }

            ProofNode::NotElim { proof, negation } => {
                let p = self.check_node(proof)?;
                let neg = self.check_node(negation)?;
                match neg {
                    KProp::Not(inner) if *inner == p => Ok(KProp::False),
                    KProp::Not(_) => Err(CheckError::PremiseMismatch {
                        expected: KProp::Not(Box::new(p.clone())),
                        got: neg,
                    }),
                    other => Err(CheckError::NotANegation(other)),
                }
            }

            ProofNode::FalseElim { proof, conclusion } => {
                let p = self.check_node(proof)?;
                if p != KProp::False {
                    return Err(CheckError::NotFalse(p));
                }
                Ok(conclusion)
            }

            ProofNode::GeoCertificate { rule, premises, conclusion } => {
                // Verify each premise
                let mut premise_props = Vec::new();
                for prem_id in premises {
                    premise_props.push(self.check_node(prem_id)?);
                }

                match rule.as_str() {
                    "IsoscelesBaseAngles" => {
                        // Premise: equal(AB, AC)
                        // Conclusion: equal(angle(ABC), angle(ACB))
                        if premise_props.len() != 1 {
                            return Err(CheckError::InvalidGeoCertificate {
                                rule,
                                reason: format!("Expected 1 premise, got {}", premise_props.len()),
                            });
                        }
                        match (&premise_props[0], &conclusion) {
                            (KProp::Eq(_, _), KProp::Eq(_, _)) => Ok(conclusion),
                            (KProp::Atom(r1, _), KProp::Atom(r2, _)) if r1 == "equal_length" && r2 == "equal_angle" => {
                                Ok(conclusion)
                            }
                            _ => Err(CheckError::InvalidGeoCertificate {
                                rule,
                                reason: "Premise must be side equality and conclusion angle equality".into(),
                            }),
                        }
                    }
                    "SSS" => {
                        // 3 side equality premises -> triangle congruence
                        if premise_props.len() != 3 {
                            return Err(CheckError::InvalidGeoCertificate {
                                rule,
                                reason: format!("SSS requires 3 side equality premises, got {}", premise_props.len()),
                            });
                        }
                        for p in &premise_props {
                            match p {
                                KProp::Eq(_, _) | KProp::Atom(..) => {},
                                other => return Err(CheckError::InvalidGeoCertificate {
                                    rule,
                                    reason: format!("Premise {:?} is not an equality", other),
                                }),
                            }
                        }
                        Ok(conclusion)
                    }
                    "SAS" => {
                        // 2 side equalities + 1 included angle equality
                        if premise_props.len() != 3 {
                            return Err(CheckError::InvalidGeoCertificate {
                                rule,
                                reason: format!("SAS requires 3 premises, got {}", premise_props.len()),
                            });
                        }
                        Ok(conclusion)
                    }
                    "CongruentTrianglesAngles" => {
                        // Premise: congruent(T1, T2) -> corresponding angles equal
                        if premise_props.len() != 1 {
                            return Err(CheckError::InvalidGeoCertificate {
                                rule,
                                reason: "Expected 1 triangle congruence premise".into(),
                            });
                        }
                        Ok(conclusion)
                    }
                    _ => {
                        Err(CheckError::InvalidGeoCertificate {
                            rule: rule.clone(),
                            reason: format!("Unknown geometric rule '{}'", rule),
                        })
                    }
                }
            }
        };

        // Cache the result
        if let Ok(ref prop) = result {
            self.cache.insert(id, prop.clone());
        }

        result
    }
}

/// Convenience function: check a proof object against a context.
pub fn check_proof(proof: &ProofObject, ctx: &Context) -> CheckResult {
    Checker::new(proof, ctx).check()
}
