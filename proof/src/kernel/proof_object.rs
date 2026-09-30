//! Proof objects — the DAG representation of proof certificates.
//!
//! Each `ProofNode` represents a single inference step. The kernel checker
//! verifies these nodes against the logical rules to confirm validity.

use crate::id::FactId;
use super::types::{KProp, KTerm, KBinder};

/// An index into the proof node arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProofNodeId(pub usize);

/// A single inference step in a proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofNode {
    /// Assume a proposition as a hypothesis. The checker must confirm
    /// this fact is in the active context.
    Assumption(FactId),

    // ── Implication ──────────────────────────────────────────────

    /// → Introduction: assume P, derive Q ⊢ P → Q.
    /// `premise` is the assumed proposition, `body` proves Q under that assumption.
    ImpIntro {
        premise: KProp,
        body: ProofNodeId,
    },

    /// → Elimination (Modus Ponens): from (P → Q) and P, derive Q.
    ImpElim {
        imp: ProofNodeId,   // proof of P → Q
        arg: ProofNodeId,   // proof of P
    },

    // ── Conjunction ──────────────────────────────────────────────

    /// ∧ Introduction: from P and Q, derive P ∧ Q.
    AndIntro {
        left: ProofNodeId,
        right: ProofNodeId,
    },

    /// ∧ Elimination Left: from P ∧ Q, derive P.
    AndElimLeft(ProofNodeId),

    /// ∧ Elimination Right: from P ∧ Q, derive Q.
    AndElimRight(ProofNodeId),

    // ── Disjunction ──────────────────────────────────────────────

    /// ∨ Introduction Left: from P, derive P ∨ Q.
    OrIntroLeft {
        proof: ProofNodeId,
        right_prop: KProp,   // the Q in P ∨ Q
    },

    /// ∨ Introduction Right: from Q, derive P ∨ Q.
    OrIntroRight {
        proof: ProofNodeId,
        left_prop: KProp,    // the P in P ∨ Q
    },

    /// ∨ Elimination (case analysis): from P ∨ Q, (P ⊢ R), (Q ⊢ R), derive R.
    OrElim {
        disjunction: ProofNodeId,  // proof of P ∨ Q
        left_case: ProofNodeId,    // proof of R assuming P
        right_case: ProofNodeId,   // proof of R assuming Q
    },

    // ── Universal quantifier ─────────────────────────────────────

    /// ∀ Introduction: prove P(x) for arbitrary x ⊢ ∀x. P(x).
    ForAllIntro {
        binder: KBinder,
        body: ProofNodeId,
    },

    /// ∀ Elimination: from ∀x. P(x) and term t, derive P(t).
    ForAllElim {
        proof: ProofNodeId,
        term: KTerm,
    },

    // ── Existential quantifier ───────────────────────────────────

    /// ∃ Introduction: from P(t), derive ∃x. P(x).
    ExistsIntro {
        witness: KTerm,
        body: ProofNodeId,
    },

    /// ∃ Elimination: from ∃x. P(x) and (∀x. P(x) → R), derive R.
    ExistsElim {
        exists_proof: ProofNodeId,
        binder: KBinder,
        body: ProofNodeId,       // proof of R assuming P(x)
    },

    // ── Equality ─────────────────────────────────────────────────

    /// Reflexivity: t = t.
    EqRefl(KTerm),

    /// Substitution: from (a = b) and P(a), derive P(b).
    EqSubst {
        equality: ProofNodeId,  // proof of a = b
        target: ProofNodeId,    // proof of P(a)
        var: String,            // the variable to substitute in
        prop_template: KProp,   // P(var) — with `var` free
    },

    // ── Negation / Contradiction ─────────────────────────────────

    /// ¬ Introduction: assume P, derive ⊥ ⊢ ¬P.
    NotIntro {
        premise: KProp,
        body: ProofNodeId,   // proof of ⊥ assuming P
    },

    /// ¬ Elimination: from P and ¬P, derive ⊥.
    NotElim {
        proof: ProofNodeId,     // proof of P
        negation: ProofNodeId,  // proof of ¬P
    },

    /// Ex falso quodlibet: from ⊥, derive any proposition.
    FalseElim {
        proof: ProofNodeId,
        conclusion: KProp,
    },

    // ── Synthetic Geometry Certificates ──────────────────────────

    /// A verified geometric inference certificate (SSS, SAS, IsoscelesBaseAngles, etc.)
    GeoCertificate {
        rule: String,
        premises: Vec<ProofNodeId>,
        conclusion: KProp,
    },
}


/// A proof object is an arena of proof nodes plus an entry-point.
#[derive(Debug, Clone)]
pub struct ProofObject {
    pub nodes: Vec<ProofNode>,
    pub root: ProofNodeId,
    /// The proposition this proof claims to prove.
    pub conclusion: KProp,
}

impl ProofObject {
    pub fn new(nodes: Vec<ProofNode>, root: ProofNodeId, conclusion: KProp) -> Self {
        Self { nodes, root, conclusion }
    }

    pub fn get(&self, id: ProofNodeId) -> Option<&ProofNode> {
        self.nodes.get(id.0)
    }
}
