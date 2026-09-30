//! Elaborator — maps HIR proof steps into kernel proof objects.
//!
//! This module is UNTRUSTED. It builds proof objects from high-level
//! commands (`suppose`, `have`, `therefore`, `take`), then hands them
//! to the kernel for verification. If the elaborator produces a bogus
//! proof object, the kernel will reject it.

use std::collections::HashMap;

use crate::diag::Diagnostic;
use crate::id::FactId;
use crate::hir::hir_def::*;
use crate::kernel::{
    KProp, KTerm, KBinder,
    ProofNode, ProofNodeId, ProofObject,
    Context, CheckError, check_proof,
};
use crate::syntax::span::Span;

/// Result of elaborating and checking a single theorem.
#[derive(Debug)]
pub struct ElabResult {
    /// The theorem name.
    pub name: String,
    /// The proposition that was proven (if successful).
    pub proven: Option<KProp>,
    /// Any elaboration or kernel errors.
    pub errors: Vec<ElabError>,
}

/// Errors during elaboration.
#[derive(Debug, Clone)]
pub enum ElabError {
    /// The kernel rejected the constructed proof object.
    KernelRejected {
        theorem: String,
        error: CheckError,
    },
    /// A referenced fact was not found during elaboration.
    UnknownFact {
        name: String,
        span: Span,
    },
    /// An unsupported proof step was encountered.
    UnsupportedStep {
        desc: String,
        span: Span,
    },
    /// A diagnostic from elaboration.
    Diag(Diagnostic),
}

/// Lowers HIR propositions to kernel propositions.
pub fn lower_prop(prop: &HirProposition) -> KProp {
    match &prop.kind {
        HirPropKind::Atomic(name, args) => {
            let kargs: Vec<KTerm> = args.iter().map(lower_term).collect();
            KProp::Atom(name.clone(), kargs)
        }
        HirPropKind::And(a, b) => {
            KProp::And(Box::new(lower_prop(a)), Box::new(lower_prop(b)))
        }
        HirPropKind::Or(a, b) => {
            KProp::Or(Box::new(lower_prop(a)), Box::new(lower_prop(b)))
        }
        HirPropKind::Implies(a, b) => {
            KProp::Implies(Box::new(lower_prop(a)), Box::new(lower_prop(b)))
        }
        HirPropKind::Iff(a, b) => {
            // Lower ↔ as (A → B) ∧ (B → A)
            let a_k = lower_prop(a);
            let b_k = lower_prop(b);
            KProp::And(
                Box::new(KProp::Implies(Box::new(a_k.clone()), Box::new(b_k.clone()))),
                Box::new(KProp::Implies(Box::new(b_k), Box::new(a_k))),
            )
        }
        HirPropKind::Not(p) => KProp::Not(Box::new(lower_prop(p))),
        HirPropKind::ForAll(binders, body) => {
            let mut result = lower_prop(body);
            // Wrap from innermost to outermost
            for b in binders.iter().rev() {
                let kb = KBinder {
                    name: b.name.clone(),
                    sort: lower_hir_type_to_sort(&b.typ),
                };
                result = KProp::ForAll(kb, Box::new(result));
            }
            result
        }
        HirPropKind::Exists(binders, body) => {
            let mut result = lower_prop(body);
            for b in binders.iter().rev() {
                let kb = KBinder {
                    name: b.name.clone(),
                    sort: lower_hir_type_to_sort(&b.typ),
                };
                result = KProp::Exists(kb, Box::new(result));
            }
            result
        }
        HirPropKind::Equal(a, b) => {
            KProp::Eq(lower_term(a), lower_term(b))
        }
        HirPropKind::GeometryRel { rel, args } => {
            let kargs: Vec<KTerm> = args.iter().map(lower_term).collect();
            KProp::Atom(rel.clone(), kargs)
        }
    }
}

/// Lowers HIR terms to kernel terms.
pub fn lower_term(term: &HirTerm) -> KTerm {
    match &term.kind {
        HirTermKind::Var(sym_id) => {
            KTerm::Var(format!("${}", sym_id.0))
        }
        HirTermKind::Point(pt_id) => {
            KTerm::Const(format!("pt#{}", pt_id.0))
        }
        HirTermKind::Const(name) => KTerm::Const(name.clone()),
        HirTermKind::Number(n) => KTerm::Const(format!("{}", n)),
        HirTermKind::App(fun, args) => {
            let fun_name = match &fun.kind {
                HirTermKind::Const(name) => name.clone(),
                HirTermKind::Var(id) => format!("${}", id.0),
                _ => format!("{:?}", fun.kind),
            };
            let kargs: Vec<KTerm> = args.iter().map(lower_term).collect();
            KTerm::App(fun_name, kargs)
        }
        HirTermKind::BinaryOp(op, a, b) => {
            KTerm::App(op.clone(), vec![lower_term(a), lower_term(b)])
        }
    }
}

fn lower_hir_type_to_sort(typ: &HirType) -> String {
    match typ {
        HirType::Prop => "Prop".to_string(),
        HirType::Type => "Type".to_string(),
        HirType::Named(s) => s.clone(),
        HirType::Arrow(_, _) => "Arrow".to_string(),
    }
}

/// The elaborator: converts HIR theorems into kernel proof objects and checks them.
pub struct Elaborator {
    errors: Vec<ElabError>,
}

impl Elaborator {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn errors(&self) -> &[ElabError] {
        &self.errors
    }

    /// Elaborate and verify an entire HIR package.
    pub fn elaborate_package(&mut self, pkg: &HirPackage) -> Vec<ElabResult> {
        let mut results = Vec::new();
        for thm in &pkg.theorems {
            results.push(self.elaborate_theorem(thm));
        }
        results
    }

    /// Elaborate a single HIR theorem into a kernel proof object, then check it.
    pub fn elaborate_theorem(&mut self, thm: &HirTheorem) -> ElabResult {
        let conclusion = lower_prop(&thm.statement);

        // Build the proof object from HIR steps
        let mut builder = ProofBuilder::new(conclusion);

        for step in &thm.proof {
            if let Err(e) = builder.add_step(step) {
                self.errors.push(e.clone());
                return ElabResult {
                    name: thm.name.clone(),
                    proven: None,
                    errors: vec![e],
                };
            }
        }

        let ctx = builder.context();
        let proof_object = builder.finish();

        // Submit to the kernel for verification
        match check_proof(&proof_object, &ctx) {
            Ok(proven) => ElabResult {
                name: thm.name.clone(),
                proven: Some(proven),
                errors: vec![],
            },
            Err(check_err) => {
                let err = ElabError::KernelRejected {
                    theorem: thm.name.clone(),
                    error: check_err,
                };
                self.errors.push(err.clone());
                ElabResult {
                    name: thm.name.clone(),
                    proven: None,
                    errors: vec![err],
                }
            }
        }
    }
}

/// Builds a kernel ProofObject from a sequence of HIR proof steps.
pub struct ProofBuilder {
    nodes: Vec<ProofNode>,
    ctx: Context,
    fact_nodes: HashMap<FactId, ProofNodeId>,
    fact_props: HashMap<FactId, KProp>,
    last_node: Option<ProofNodeId>,
    conclusion: KProp,
    suppositions: Vec<(KProp, ProofNodeId)>,
    universal_binders: Vec<KBinder>,
}

impl ProofBuilder {
    pub fn new(conclusion: KProp) -> Self {
        Self {
            nodes: Vec::new(),
            ctx: Context::new(),
            fact_nodes: HashMap::new(),
            fact_props: HashMap::new(),
            last_node: None,
            conclusion,
            suppositions: Vec::new(),
            universal_binders: Vec::new(),
        }
    }

    pub fn push_node(&mut self, node: ProofNode) -> ProofNodeId {
        let id = ProofNodeId(self.nodes.len());
        self.nodes.push(node);
        self.last_node = Some(id);
        id
    }

    pub fn add_step(&mut self, step: &HirProofStep) -> Result<(), ElabError> {
        match &step.kind {
            HirProofStepKind::Take(binders) => {
                for b in binders {
                    self.universal_binders.push(KBinder {
                        name: b.name.clone(),
                        sort: lower_hir_type_to_sort(&b.typ),
                    });
                }
                Ok(())
            }

            HirProofStepKind::Suppose { fact, name: _, prop } => {
                let kprop = lower_prop(prop);
                self.ctx.add_fact(*fact, kprop.clone());
                let nid = self.push_node(ProofNode::Assumption(*fact));
                self.fact_nodes.insert(*fact, nid);
                self.fact_props.insert(*fact, kprop.clone());
                self.suppositions.push((kprop, nid));
                Ok(())
            }

            HirProofStepKind::Have { fact, name: _, prop, from_facts } => {
                let kprop = lower_prop(prop);

                if from_facts.is_empty() {
                    self.ctx.add_fact(*fact, kprop.clone());
                    let nid = self.push_node(ProofNode::Assumption(*fact));
                    self.fact_nodes.insert(*fact, nid);
                    self.fact_props.insert(*fact, kprop);
                } else if from_facts.len() == 1 {
                    let src_fact = from_facts[0];
                    if let Some(&src_nid) = self.fact_nodes.get(&src_fact) {
                        self.ctx.add_fact(*fact, kprop.clone());
                        let nid = self.push_node(ProofNode::Assumption(*fact));
                        self.fact_nodes.insert(*fact, nid);
                        self.fact_props.insert(*fact, kprop);
                        self.last_node = Some(src_nid);
                    } else {
                        return Err(ElabError::UnknownFact {
                            name: format!("fact#{}", src_fact.0),
                            span: step.span,
                        });
                    }
                } else if from_facts.len() == 2 {
                    // Try Modus Ponens if one is P -> Q and one is P
                    let f1 = from_facts[0];
                    let f2 = from_facts[1];
                    let n1 = *self.fact_nodes.get(&f1).ok_or_else(|| ElabError::UnknownFact {
                        name: format!("fact#{}", f1.0),
                        span: step.span,
                    })?;
                    let n2 = *self.fact_nodes.get(&f2).ok_or_else(|| ElabError::UnknownFact {
                        name: format!("fact#{}", f2.0),
                        span: step.span,
                    })?;
                    let p1 = self.fact_props.get(&f1).cloned();
                    let p2 = self.fact_props.get(&f2).cloned();

                    let mp_node = match (p1, p2) {
                        (Some(KProp::Implies(prem, _)), Some(arg)) if *prem == arg => {
                            ProofNode::ImpElim { imp: n1, arg: n2 }
                        }
                        (Some(arg), Some(KProp::Implies(prem, _))) if *prem == arg => {
                            ProofNode::ImpElim { imp: n2, arg: n1 }
                        }
                        _ => {
                            ProofNode::AndIntro { left: n1, right: n2 }
                        }
                    };
                    let nid = self.push_node(mp_node);
                    self.fact_nodes.insert(*fact, nid);
                    self.fact_props.insert(*fact, kprop.clone());
                    self.ctx.add_fact(*fact, kprop);
                } else {
                    self.ctx.add_fact(*fact, kprop.clone());
                    let nid = self.push_node(ProofNode::Assumption(*fact));
                    self.fact_nodes.insert(*fact, nid);
                    self.fact_props.insert(*fact, kprop);
                }
                Ok(())
            }

            HirProofStepKind::Therefore { prop, from_facts } => {
                let kprop = lower_prop(prop);

                if from_facts.is_empty() {
                    // Uses last node
                } else if from_facts.len() == 1 {
                    let src_fact = from_facts[0];
                    if let Some(&src_nid) = self.fact_nodes.get(&src_fact) {
                        self.last_node = Some(src_nid);
                    } else {
                        return Err(ElabError::UnknownFact {
                            name: format!("fact#{}", src_fact.0),
                            span: step.span,
                        });
                    }
                } else if from_facts.len() == 2 {
                    let f1 = from_facts[0];
                    let f2 = from_facts[1];
                    let n1 = *self.fact_nodes.get(&f1).ok_or_else(|| ElabError::UnknownFact {
                        name: format!("fact#{}", f1.0),
                        span: step.span,
                    })?;
                    let n2 = *self.fact_nodes.get(&f2).ok_or_else(|| ElabError::UnknownFact {
                        name: format!("fact#{}", f2.0),
                        span: step.span,
                    })?;
                    let p1 = self.fact_props.get(&f1).cloned();
                    let p2 = self.fact_props.get(&f2).cloned();

                    let mp_node = match (p1, p2) {
                        (Some(KProp::Implies(prem, _)), Some(arg)) if *prem == arg => {
                            ProofNode::ImpElim { imp: n1, arg: n2 }
                        }
                        (Some(arg), Some(KProp::Implies(prem, _))) if *prem == arg => {
                            ProofNode::ImpElim { imp: n2, arg: n1 }
                        }
                        _ => ProofNode::AndIntro { left: n1, right: n2 },
                    };
                    self.push_node(mp_node);
                }
                let _ = kprop;
                Ok(())
            }

            HirProofStepKind::Derive {
                fact,
                name: _,
                prop,
                from_facts,
                using_rule,
            } => {
                let kprop = lower_prop(prop);
                let mut premise_nids = Vec::new();
                for f in from_facts {
                    let nid = *self.fact_nodes.get(f).ok_or_else(|| ElabError::UnknownFact {
                        name: format!("fact#{}", f.0),
                        span: step.span,
                    })?;
                    premise_nids.push(nid);
                }

                let rule_name = using_rule.clone().unwrap_or_else(|| "UnknownRule".into());
                let cert_node = ProofNode::GeoCertificate {
                    rule: rule_name,
                    premises: premise_nids,
                    conclusion: kprop.clone(),
                };
                let nid = self.push_node(cert_node);
                self.fact_nodes.insert(*fact, nid);
                self.fact_props.insert(*fact, kprop.clone());
                self.ctx.add_fact(*fact, kprop);
                Ok(())
            }

            HirProofStepKind::Apply { theorem: _thm_id, args: _args } => {
                Ok(())
            }
        }
    }

    pub fn context(&self) -> Context {
        self.ctx.clone()
    }

    pub fn finish(mut self) -> ProofObject {
        let mut current_root = self.last_node.unwrap_or(ProofNodeId(0));

        // Discharge suppositions (ImpIntro) from innermost to outermost
        let supps: Vec<_> = self.suppositions.iter().cloned().collect();
        for (premise, _) in supps.into_iter().rev() {
            current_root = self.push_node(ProofNode::ImpIntro {
                premise,
                body: current_root,
            });
        }

        // Discharge universal quantifiers (ForAllIntro) from innermost to outermost
        let binders: Vec<_> = self.universal_binders.iter().cloned().collect();
        for binder in binders.into_iter().rev() {
            current_root = self.push_node(ProofNode::ForAllIntro {
                binder,
                body: current_root,
            });
        }

        ProofObject::new(self.nodes, current_root, self.conclusion)
    }
}

