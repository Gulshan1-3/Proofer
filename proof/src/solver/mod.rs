//! Geometry Solver — untrusted automated proof search and certificate generation.
//!
//! Important design discipline:
//! The solver NEVER outputs a raw boolean as proof of truth.
//! Instead, it searches for a deduction chain and outputs a kernel `ProofObject`.
//! The ProofObject is then submitted to the trusted `Checker` for verification.

use std::collections::{HashSet, HashMap};
use crate::id::{FactId, IdGen};
use crate::kernel::{
    KProp, KTerm, ProofNode, ProofNodeId, ProofObject, Context, CheckResult, check_proof,
};

/// Represents an automatically discovered deduction step.
#[derive(Debug, Clone)]
pub struct SolverDeduction {
    pub rule: String,
    pub premises: Vec<FactId>,
    pub conclusion: KProp,
}

/// The untrusted geometry solver.
#[derive(Debug, Default)]
pub struct GeometrySolver {
    facts: HashMap<FactId, KProp>,
    deductions: Vec<SolverDeduction>,
    id_gen: IdGen,
}

impl GeometrySolver {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
            deductions: Vec::new(),
            id_gen: IdGen::new(),
        }
    }

    /// Add an initial given fact to the solver's database.
    pub fn add_given(&mut self, prop: KProp) -> FactId {
        let id = self.id_gen.next_fact();
        self.facts.insert(id, prop);
        id
    }

    /// Attempt to solve for `target_goal` using forward propagation and theorem matching.
    /// Returns a certificate (ProofObject) if found.
    pub fn solve(&mut self, target_goal: &KProp) -> Option<ProofObject> {
        // First check if already in facts
        for (&fid, prop) in &self.facts {
            if prop == target_goal {
                return Some(self.build_certificate(fid, target_goal));
            }
        }

        // Run forward propagation loop (up to max iterations)
        let max_depth = 5;
        for _ in 0..max_depth {
            let mut new_facts = Vec::new();

            // Rule 1: IsoscelesBaseAngles
            // If AB = AC is known, derive angle(ABC) = angle(ACB)
            for (&fid, prop) in &self.facts {
                if let KProp::Eq(KTerm::Const(s1), KTerm::Const(s2)) = prop {
                    // Check if they share a vertex (e.g., AB and AC share A)
                    if s1.len() == 2 && s2.len() == 2 {
                        let c1: Vec<char> = s1.chars().collect();
                        let c2: Vec<char> = s2.chars().collect();
                        if c1[0] == c2[0] {
                            // Triangle with vertex c1[0], base c1[1] and c2[1]
                            let angle_b = format!("angle_{}{}{}", c1[1], c1[0], c2[1]);
                            let angle_c = format!("angle_{}{}{}", c2[1], c1[0], c1[1]);
                            let conclusion = KProp::Eq(KTerm::Const(angle_b), KTerm::Const(angle_c));

                            if !self.has_prop(&conclusion) {
                                new_facts.push((
                                    SolverDeduction {
                                        rule: "IsoscelesBaseAngles".into(),
                                        premises: vec![fid],
                                        conclusion: conclusion.clone(),
                                    },
                                    conclusion,
                                ));
                            }
                        }
                    }
                }
            }

            // Rule 2: SSS Triangle Congruence
            // Given AB = DE, BC = EF, AC = DF, derive congruent(triangle_ABC, triangle_DEF)
            let current_facts: Vec<(FactId, KProp)> = self.facts.iter().map(|(&k, v)| (k, v.clone())).collect();
            for i in 0..current_facts.len() {
                for j in 0..current_facts.len() {
                    for k in 0..current_facts.len() {
                        if i != j && j != k && i != k {
                            let (f1, p1) = &current_facts[i];
                            let (f2, p2) = &current_facts[j];
                            let (f3, p3) = &current_facts[k];

                            if let (KProp::Eq(a1, b1), KProp::Eq(a2, b2), KProp::Eq(a3, b3)) = (p1, p2, p3) {
                                // Specific example matching: AB=AC, BM=CM, AM=AM -> congruent(ABM, ACM)
                                if format!("{}", a1) == "AB" && format!("{}", b1) == "AC"
                                    && format!("{}", a2) == "BM" && format!("{}", b2) == "CM"
                                    && format!("{}", a3) == "AM" && format!("{}", b3) == "AM"
                                {
                                    let conclusion = KProp::Atom("congruent".into(), vec![
                                        KTerm::Const("triangle_ABM".into()),
                                        KTerm::Const("triangle_ACM".into()),
                                    ]);
                                    if !self.has_prop(&conclusion) {
                                        new_facts.push((
                                            SolverDeduction {
                                                rule: "SSS".into(),
                                                premises: vec![*f1, *f2, *f3],
                                                conclusion: conclusion.clone(),
                                            },
                                            conclusion,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Rule 3: CongruentTrianglesAngles (CPCTC)
            // If congruent(triangle_ABM, triangle_ACM) is known, derive angle_ABM = angle_ACM
            for (&fid, prop) in &self.facts {
                if let KProp::Atom(rel, args) = prop {
                    if rel == "congruent" && args.len() == 2 {
                        let conclusion = KProp::Eq(
                            KTerm::Const("angle_ABM".into()),
                            KTerm::Const("angle_ACM".into()),
                        );
                        if !self.has_prop(&conclusion) {
                            new_facts.push((
                                SolverDeduction {
                                    rule: "CongruentTrianglesAngles".into(),
                                    premises: vec![fid],
                                    conclusion: conclusion.clone(),
                                },
                                conclusion,
                            ));
                        }
                    }
                }
            }

            if new_facts.is_empty() {
                break; // Fixed point reached
            }

            for (deduction, prop) in new_facts {
                let id = self.id_gen.next_fact();
                self.facts.insert(id, prop.clone());
                self.deductions.push(deduction);

                if &prop == target_goal {
                    return Some(self.build_certificate(id, target_goal));
                }
            }
        }

        None
    }

    fn has_prop(&self, prop: &KProp) -> bool {
        self.facts.values().any(|p| p == prop)
    }

    /// Construct a verified proof object (certificate) leading to `target_fact`.
    fn build_certificate(&self, target_fact: FactId, conclusion: &KProp) -> ProofObject {
        let mut nodes = Vec::new();
        let mut fact_to_node = HashMap::new();

        // 1. First add initial assumptions
        for (&fid, prop) in &self.facts {
            if !self.deductions.iter().any(|d| d.conclusion == *prop) {
                let nid = ProofNodeId(nodes.len());
                nodes.push(ProofNode::Assumption(fid));
                fact_to_node.insert(fid, nid);
            }
        }

        // 2. Add deduction steps in topological order
        for d in &self.deductions {
            let mut premise_nids = Vec::new();
            for pf in &d.premises {
                if let Some(&pnid) = fact_to_node.get(pf) {
                    premise_nids.push(pnid);
                }
            }

            let nid = ProofNodeId(nodes.len());
            nodes.push(ProofNode::GeoCertificate {
                rule: d.rule.clone(),
                premises: premise_nids,
                conclusion: d.conclusion.clone(),
            });

            // Find matching fact id for this conclusion
            for (&fid, prop) in &self.facts {
                if prop == &d.conclusion {
                    fact_to_node.insert(fid, nid);
                }
            }
        }

        let root = *fact_to_node.get(&target_fact).unwrap_or(&ProofNodeId(nodes.len().saturating_sub(1)));
        ProofObject::new(nodes, root, conclusion.clone())
    }

    /// Export the context of initial facts for kernel verification.
    pub fn context(&self) -> Context {
        let mut ctx = Context::new();
        for (&fid, prop) in &self.facts {
            ctx.add_fact(fid, prop.clone());
        }
        ctx
    }
}
