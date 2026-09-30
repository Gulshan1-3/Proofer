use crate::syntax::span::{Span, Spanned};
use std::fmt;

// Types in the proof language
#[derive(Debug, Clone, PartialEq)]
pub struct AstType {
    pub kind: AstTypeKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AstTypeKind {
    Prop,                               // Prop
    Type,                               // Type
    Named(String),                      // Point, Line, etc.
    Arrow(Box<AstType>, Box<AstType>),  // P -> Q
    All(String, Box<AstType>),          // ∀x. P
    Exists(String, Box<AstType>),       // ∃x. P
}

impl Spanned for AstType {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binder {
    pub name: String,
    pub typ: Option<AstType>,
    pub span: Span,
}

impl Spanned for Binder {
    fn span(&self) -> Span {
        self.span
    }
}

// Terms in the language
#[derive(Debug, Clone, PartialEq)]
pub struct Term {
    pub kind: TermKind,
    pub span: Span,
}

impl Spanned for Term {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TermKind {
    Var(String),
    Const(String),
    Number(i64),
    StringLit(String),
    App(Box<Term>, Vec<Term>),
    BinaryOp(String, Box<Term>, Box<Term>),
}

// Logical propositions
#[derive(Debug, Clone, PartialEq)]
pub struct Proposition {
    pub kind: PropositionKind,
    pub span: Span,
}

impl Spanned for Proposition {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropositionKind {
    Atomic(String, Vec<Term>),
    And(Box<Proposition>, Box<Proposition>),
    Or(Box<Proposition>, Box<Proposition>),
    Implies(Box<Proposition>, Box<Proposition>),
    Iff(Box<Proposition>, Box<Proposition>),
    Not(Box<Proposition>),
    ForAll(Vec<Binder>, Box<Proposition>),
    Exists(Vec<Binder>, Box<Proposition>),
    Equal(Box<Term>, Box<Term>),
    GeometryRel {
        rel: String,
        args: Vec<Term>,
    },
}

// Proof steps / statements
#[derive(Debug, Clone, PartialEq)]
pub struct ProofStep {
    pub kind: ProofStepKind,
    pub span: Span,
}

impl Spanned for ProofStep {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProofStepKind {
    // take x : T
    Take(Vec<Binder>),
    // suppose h : P
    Suppose {
        name: String,
        prop: Proposition,
    },
    // let x = t / let x : T
    Let {
        name: String,
        val: Option<Term>,
        typ: Option<AstType>,
    },
    // have h : P [from ...]
    Have {
        name: Option<String>,
        prop: Proposition,
        from: Vec<String>,
        by: Option<String>,
    },
    // derive h : P from ... using ...
    Derive {
        name: Option<String>,
        prop: Proposition,
        from: Vec<String>,
        using_rule: Option<String>,
    },
    // construct M as midpoint of BC
    Construct {
        name: String,
        as_desc: String,
        args: Vec<Term>,
    },
    // show P
    Show(Proposition),
    // therefore P [from ...]
    Therefore {
        prop: Proposition,
        from: Vec<String>,
    },
    // use t
    Use(Vec<Term>),
    // cases h
    Cases {
        target: String,
    },
    // contradict
    Contradict {
        target: Option<String>,
    },
    // Raw step / legacy tactic: assume, intro, apply, etc.
    Assume(Proposition),
    Intro(Vec<String>),
    Apply {
        theorem: String,
        args: Vec<Term>,
    },
}

// Complete Theorem declaration
#[derive(Debug, Clone, PartialEq)]
pub struct Theorem {
    pub name: String,
    pub statement: Proposition,
    pub proof: Vec<ProofStep>,
    pub span: Span,
}

impl Spanned for Theorem {
    fn span(&self) -> Span {
        self.span
    }
}

// Figure declaration: figure name ... end
#[derive(Debug, Clone, PartialEq)]
pub struct FigureDecl {
    pub name: String,
    pub items: Vec<FigureItem>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FigureItem {
    Object {
        kind: String, // triangle, point, line, etc.
        name: String,
        span: Span,
    },
    Given {
        prop: Proposition,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Theorem(Theorem),
    Figure(FigureDecl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileAst {
    pub items: Vec<Item>,
    pub span: Span,
}

// Display implementations for clean debugging
impl fmt::Display for AstType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            AstTypeKind::Prop => write!(f, "Prop"),
            AstTypeKind::Type => write!(f, "Type"),
            AstTypeKind::Named(s) => write!(f, "{}", s),
            AstTypeKind::Arrow(a, b) => write!(f, "({} → {})", a, b),
            AstTypeKind::All(v, t) => write!(f, "(∀{}: {})", v, t),
            AstTypeKind::Exists(v, t) => write!(f, "(∃{}: {})", v, t),
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            TermKind::Var(v) => write!(f, "{}", v),
            TermKind::Const(c) => write!(f, "{}", c),
            TermKind::Number(n) => write!(f, "{}", n),
            TermKind::StringLit(s) => write!(f, "\"{}\"", s),
            TermKind::App(fun, args) => {
                let args_str: Vec<String> = args.iter().map(|t| t.to_string()).collect();
                write!(f, "{}({})", fun, args_str.join(", "))
            }
            TermKind::BinaryOp(op, a, b) => write!(f, "({} {} {})", a, op, b),
        }
    }
}

impl fmt::Display for Proposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            PropositionKind::Atomic(name, args) => {
                if args.is_empty() {
                    write!(f, "{}", name)
                } else {
                    let args_str: Vec<String> = args.iter().map(|t| t.to_string()).collect();
                    write!(f, "{}({})", name, args_str.join(", "))
                }
            }
            PropositionKind::And(a, b) => write!(f, "({} ∧ {})", a, b),
            PropositionKind::Or(a, b) => write!(f, "({} ∨ {})", a, b),
            PropositionKind::Implies(a, b) => write!(f, "({} → {})", a, b),
            PropositionKind::Iff(a, b) => write!(f, "({} ↔ {})", a, b),
            PropositionKind::Not(p) => write!(f, "¬{}", p),
            PropositionKind::ForAll(binders, p) => {
                let b_str: Vec<String> = binders.iter().map(|b| {
                    if let Some(t) = &b.typ {
                        format!("{} : {}", b.name, t)
                    } else {
                        b.name.clone()
                    }
                }).collect();
                write!(f, "∀{}. {}", b_str.join(" "), p)
            }
            PropositionKind::Exists(binders, p) => {
                let b_str: Vec<String> = binders.iter().map(|b| {
                    if let Some(t) = &b.typ {
                        format!("{} : {}", b.name, t)
                    } else {
                        b.name.clone()
                    }
                }).collect();
                write!(f, "∃{}. {}", b_str.join(" "), p)
            }
            PropositionKind::Equal(a, b) => write!(f, "({} = {})", a, b),
            PropositionKind::GeometryRel { rel, args } => {
                let args_str: Vec<String> = args.iter().map(|t| t.to_string()).collect();
                write!(f, "{} {}", rel, args_str.join(" "))
            }
        }
    }
}
