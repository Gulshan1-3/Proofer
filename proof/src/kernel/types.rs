//! Kernel-level types — the trusted representation of propositions and terms.
//!
//! These types are intentionally independent of the parser, AST, and HIR.
//! The kernel must never depend on untrusted representations.

use std::fmt;

/// A kernel-level term (first-order).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KTerm {
    /// A bound or free variable, identified by name.
    Var(String),
    /// A constant symbol.
    Const(String),
    /// Function application: f(t1, t2, ...).
    App(String, Vec<KTerm>),
}

impl fmt::Display for KTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KTerm::Var(v) => write!(f, "{}", v),
            KTerm::Const(c) => write!(f, "{}", c),
            KTerm::App(fun, args) => {
                let a: Vec<String> = args.iter().map(|t| t.to_string()).collect();
                write!(f, "{}({})", fun, a.join(", "))
            }
        }
    }
}

/// A kernel-level binder (variable with optional type annotation).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KBinder {
    pub name: String,
    pub sort: String, // "Prop", "Type", or a named sort
}

/// A kernel-level proposition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum KProp {
    /// Atomic proposition: P, Q(x), R(x, y)
    Atom(String, Vec<KTerm>),
    /// P ∧ Q
    And(Box<KProp>, Box<KProp>),
    /// P ∨ Q
    Or(Box<KProp>, Box<KProp>),
    /// P → Q
    Implies(Box<KProp>, Box<KProp>),
    /// ¬P
    Not(Box<KProp>),
    /// ∀x. P
    ForAll(KBinder, Box<KProp>),
    /// ∃x. P
    Exists(KBinder, Box<KProp>),
    /// t₁ = t₂
    Eq(KTerm, KTerm),
    /// ⊥ (falsity / contradiction)
    False,
    /// ⊤ (truth)
    True,
}

impl fmt::Display for KProp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KProp::Atom(name, args) if args.is_empty() => write!(f, "{}", name),
            KProp::Atom(name, args) => {
                let a: Vec<String> = args.iter().map(|t| t.to_string()).collect();
                write!(f, "{}({})", name, a.join(", "))
            }
            KProp::And(a, b) => write!(f, "({} ∧ {})", a, b),
            KProp::Or(a, b) => write!(f, "({} ∨ {})", a, b),
            KProp::Implies(a, b) => write!(f, "({} → {})", a, b),
            KProp::Not(p) => write!(f, "¬{}", p),
            KProp::ForAll(b, p) => write!(f, "∀{}:{}.{}", b.name, b.sort, p),
            KProp::Exists(b, p) => write!(f, "∃{}:{}.{}", b.name, b.sort, p),
            KProp::Eq(a, b) => write!(f, "({} = {})", a, b),
            KProp::False => write!(f, "⊥"),
            KProp::True => write!(f, "⊤"),
        }
    }
}

/// Substitution: replace free occurrences of `var` with `replacement` in a proposition.
impl KProp {
    pub fn subst(&self, var: &str, replacement: &KTerm) -> KProp {
        match self {
            KProp::Atom(name, args) => {
                let new_args = args.iter().map(|t| t.subst(var, replacement)).collect();
                KProp::Atom(name.clone(), new_args)
            }
            KProp::And(a, b) => KProp::And(
                Box::new(a.subst(var, replacement)),
                Box::new(b.subst(var, replacement)),
            ),
            KProp::Or(a, b) => KProp::Or(
                Box::new(a.subst(var, replacement)),
                Box::new(b.subst(var, replacement)),
            ),
            KProp::Implies(a, b) => KProp::Implies(
                Box::new(a.subst(var, replacement)),
                Box::new(b.subst(var, replacement)),
            ),
            KProp::Not(p) => KProp::Not(Box::new(p.subst(var, replacement))),
            KProp::ForAll(binder, body) => {
                if binder.name == var {
                    // Variable is shadowed by the binder — don't substitute inside
                    self.clone()
                } else {
                    KProp::ForAll(binder.clone(), Box::new(body.subst(var, replacement)))
                }
            }
            KProp::Exists(binder, body) => {
                if binder.name == var {
                    self.clone()
                } else {
                    KProp::Exists(binder.clone(), Box::new(body.subst(var, replacement)))
                }
            }
            KProp::Eq(a, b) => KProp::Eq(
                a.subst(var, replacement),
                b.subst(var, replacement),
            ),
            KProp::False => KProp::False,
            KProp::True => KProp::True,
        }
    }
}

impl KTerm {
    pub fn subst(&self, var: &str, replacement: &KTerm) -> KTerm {
        match self {
            KTerm::Var(v) if v == var => replacement.clone(),
            KTerm::Var(_) | KTerm::Const(_) => self.clone(),
            KTerm::App(fun, args) => {
                let new_args = args.iter().map(|t| t.subst(var, replacement)).collect();
                KTerm::App(fun.clone(), new_args)
            }
        }
    }
}
