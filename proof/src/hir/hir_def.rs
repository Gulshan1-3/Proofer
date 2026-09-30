use crate::id::{FactId, PointId, SymbolId, TheoremId};
use crate::syntax::span::{Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirType {
    Prop,
    Type,
    Named(String),
    Arrow(Box<HirType>, Box<HirType>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirBinder {
    pub id: SymbolId,
    pub name: String,
    pub typ: HirType,
    pub span: Span,
}

impl Spanned for HirBinder {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirTerm {
    pub kind: HirTermKind,
    pub span: Span,
}

impl Spanned for HirTerm {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirTermKind {
    Var(SymbolId),
    Point(PointId),
    Const(String),
    Number(i64),
    App(Box<HirTerm>, Vec<HirTerm>),
    BinaryOp(String, Box<HirTerm>, Box<HirTerm>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirProposition {
    pub kind: HirPropKind,
    pub span: Span,
}

impl Spanned for HirProposition {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirPropKind {
    Atomic(String, Vec<HirTerm>),
    And(Box<HirProposition>, Box<HirProposition>),
    Or(Box<HirProposition>, Box<HirProposition>),
    Implies(Box<HirProposition>, Box<HirProposition>),
    Iff(Box<HirProposition>, Box<HirProposition>),
    Not(Box<HirProposition>),
    ForAll(Vec<HirBinder>, Box<HirProposition>),
    Exists(Vec<HirBinder>, Box<HirProposition>),
    Equal(Box<HirTerm>, Box<HirTerm>),
    GeometryRel {
        rel: String,
        args: Vec<HirTerm>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirProofStep {
    pub kind: HirProofStepKind,
    pub span: Span,
}

impl Spanned for HirProofStep {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HirProofStepKind {
    Take(Vec<HirBinder>),
    Suppose {
        fact: FactId,
        name: String,
        prop: HirProposition,
    },
    Have {
        fact: FactId,
        name: Option<String>,
        prop: HirProposition,
        from_facts: Vec<FactId>,
    },
    Therefore {
        prop: HirProposition,
        from_facts: Vec<FactId>,
    },
    Derive {
        fact: FactId,
        name: Option<String>,
        prop: HirProposition,
        from_facts: Vec<FactId>,
        using_rule: Option<String>,
    },
    Apply {
        theorem: TheoremId,
        args: Vec<HirTerm>,
    },
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HirTheorem {
    pub id: TheoremId,
    pub name: String,
    pub statement: HirProposition,
    pub proof: Vec<HirProofStep>,
    pub span: Span,
}

impl Spanned for HirTheorem {
    fn span(&self) -> Span {
        self.span
    }
}

#[derive(Debug, Clone, Default)]
pub struct HirPackage {
    pub theorems: Vec<HirTheorem>,
    pub figures: Vec<crate::geometry::GeoFigure>,
}

