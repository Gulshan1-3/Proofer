use crate::ast::*;
use crate::diag::Diagnostic;
use crate::hir::hir_def::*;
use crate::hir::scope::{ScopeStack, SymbolKind};
use crate::id::IdGen;

pub struct Resolver {
    id_gen: IdGen,
    scopes: ScopeStack,
    diagnostics: Vec<Diagnostic>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            id_gen: IdGen::new(),
            scopes: ScopeStack::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn resolve_file(&mut self, file_ast: &FileAst) -> HirPackage {
        let mut pkg = HirPackage::default();

        // 1st pass: register top-level declarations (theorems)
        for item in &file_ast.items {
            if let Item::Theorem(thm) = item {
                let thm_id = self.id_gen.next_theorem();
                self.scopes.insert(
                    thm.name.clone(),
                    SymbolKind::Theorem(thm_id),
                    thm.span,
                );
            }
        }

        // 2nd pass: resolve bodies (theorems and figures)
        for item in &file_ast.items {
            match item {
                Item::Theorem(thm) => {
                    if let Some(resolved) = self.resolve_theorem(thm) {
                        pkg.theorems.push(resolved);
                    }
                }
                Item::Figure(fig) => {
                    if let Some(resolved_fig) = self.resolve_figure(fig) {
                        pkg.figures.push(resolved_fig);
                    }
                }
            }
        }

        pkg
    }

    pub fn resolve_theorem(&mut self, thm: &Theorem) -> Option<HirTheorem> {
        let thm_id = match self.scopes.resolve(&thm.name) {
            Some(sym) => match sym.kind {
                SymbolKind::Theorem(id) => id,
                _ => self.id_gen.next_theorem(),
            },
            None => self.id_gen.next_theorem(),
        };

        self.scopes.enter();

        let statement = self.resolve_proposition(&thm.statement);

        let mut proof_steps = Vec::new();
        for step in &thm.proof {
            if let Some(resolved_step) = self.resolve_proof_step(step) {
                proof_steps.push(resolved_step);
            }
        }

        self.scopes.exit();

        Some(HirTheorem {
            id: thm_id,
            name: thm.name.clone(),
            statement,
            proof: proof_steps,
            span: thm.span,
        })
    }

    pub fn resolve_proof_step(&mut self, step: &ProofStep) -> Option<HirProofStep> {
        let kind = match &step.kind {
            ProofStepKind::Take(binders) => {
                let mut hir_binders = Vec::new();
                for b in binders {
                    let sym_id = self.id_gen.next_symbol();
                    let typ = b
                        .typ
                        .as_ref()
                        .map(|t| self.resolve_type(t))
                        .unwrap_or(HirType::Type);
                    self.scopes.insert(
                        b.name.clone(),
                        SymbolKind::Variable(sym_id),
                        b.span,
                    );
                    hir_binders.push(HirBinder {
                        id: sym_id,
                        name: b.name.clone(),
                        typ,
                        span: b.span,
                    });
                }
                HirProofStepKind::Take(hir_binders)
            }
            ProofStepKind::Suppose { name, prop } => {
                let fact_id = self.id_gen.next_fact();
                let prop_hir = self.resolve_proposition(prop);
                self.scopes.insert(
                    name.clone(),
                    SymbolKind::Fact(fact_id),
                    step.span,
                );
                HirProofStepKind::Suppose {
                    fact: fact_id,
                    name: name.clone(),
                    prop: prop_hir,
                }
            }
            ProofStepKind::Have {
                name,
                prop,
                from,
                by: _,
            } => {
                let fact_id = self.id_gen.next_fact();
                let prop_hir = self.resolve_proposition(prop);
                let mut from_facts = Vec::new();
                for ref_name in from {
                    match self.scopes.resolve(ref_name) {
                        Some(sym) => match sym.kind {
                            SymbolKind::Fact(id) => from_facts.push(id),
                            _ => {
                                self.diagnostics.push(Diagnostic::error(
                                    format!("Reference '{}' is not a fact", ref_name),
                                    step.span,
                                ));
                            }
                        },
                        None => {
                            self.diagnostics.push(Diagnostic::error(
                                format!("Unknown fact reference '{}'", ref_name),
                                step.span,
                            ));
                        }
                    }
                }
                if let Some(n) = name {
                    self.scopes.insert(
                        n.clone(),
                        SymbolKind::Fact(fact_id),
                        step.span,
                    );
                }
                HirProofStepKind::Have {
                    fact: fact_id,
                    name: name.clone(),
                    prop: prop_hir,
                    from_facts,
                }
            }
            ProofStepKind::Derive {
                name,
                prop,
                from,
                using_rule,
            } => {
                let fact_id = self.id_gen.next_fact();
                let prop_hir = self.resolve_proposition(prop);
                let mut from_facts = Vec::new();
                for ref_name in from {
                    match self.scopes.resolve(ref_name) {
                        Some(sym) => match sym.kind {
                            SymbolKind::Fact(id) => from_facts.push(id),
                            _ => {
                                self.diagnostics.push(Diagnostic::error(
                                    format!("Reference '{}' is not a fact", ref_name),
                                    step.span,
                                ));
                            }
                        },
                        None => {
                            self.diagnostics.push(Diagnostic::error(
                                format!("Unknown fact reference '{}'", ref_name),
                                step.span,
                            ));
                        }
                    }
                }
                if let Some(n) = name {
                    self.scopes.insert(
                        n.clone(),
                        SymbolKind::Fact(fact_id),
                        step.span,
                    );
                }
                HirProofStepKind::Derive {
                    fact: fact_id,
                    name: name.clone(),
                    prop: prop_hir,
                    from_facts,
                    using_rule: using_rule.clone(),
                }
            }
            ProofStepKind::Therefore { prop, from } => {
                let prop_hir = self.resolve_proposition(prop);
                let mut from_facts = Vec::new();
                for ref_name in from {
                    match self.scopes.resolve(ref_name) {
                        Some(sym) => match sym.kind {
                            SymbolKind::Fact(id) => from_facts.push(id),
                            _ => {
                                self.diagnostics.push(Diagnostic::error(
                                    format!("Reference '{}' is not a fact", ref_name),
                                    step.span,
                                ));
                            }
                        },
                        None => {
                            self.diagnostics.push(Diagnostic::error(
                                format!("Unknown fact reference '{}'", ref_name),
                                step.span,
                            ));
                        }
                    }
                }
                HirProofStepKind::Therefore {
                    prop: prop_hir,
                    from_facts,
                }
            }
            ProofStepKind::Apply { theorem, args } => {
                let thm_id = match self.scopes.resolve(theorem) {
                    Some(sym) => match sym.kind {
                        SymbolKind::Theorem(id) => id,
                        _ => {
                            self.diagnostics.push(Diagnostic::error(
                                format!("'{}' is not a theorem", theorem),
                                step.span,
                            ));
                            self.id_gen.next_theorem()
                        }
                    },
                    None => {
                        self.diagnostics.push(Diagnostic::error(
                            format!("Unknown theorem '{}'", theorem),
                            step.span,
                        ));
                        self.id_gen.next_theorem()
                    }
                };
                let hir_args = args.iter().map(|a| self.resolve_term(a)).collect();
                HirProofStepKind::Apply {
                    theorem: thm_id,
                    args: hir_args,
                }
            }
            _ => {
                // Unsupported step in HIR yet
                return None;
            }
        };

        Some(HirProofStep {
            kind,
            span: step.span,
        })
    }

    pub fn resolve_proposition(&mut self, prop: &Proposition) -> HirProposition {
        let kind = match &prop.kind {
            PropositionKind::Atomic(name, args) => {
                let hir_args = args.iter().map(|a| self.resolve_term(a)).collect();
                HirPropKind::Atomic(name.clone(), hir_args)
            }
            PropositionKind::And(a, b) => HirPropKind::And(
                Box::new(self.resolve_proposition(a)),
                Box::new(self.resolve_proposition(b)),
            ),
            PropositionKind::Or(a, b) => HirPropKind::Or(
                Box::new(self.resolve_proposition(a)),
                Box::new(self.resolve_proposition(b)),
            ),
            PropositionKind::Implies(a, b) => HirPropKind::Implies(
                Box::new(self.resolve_proposition(a)),
                Box::new(self.resolve_proposition(b)),
            ),
            PropositionKind::Iff(a, b) => HirPropKind::Iff(
                Box::new(self.resolve_proposition(a)),
                Box::new(self.resolve_proposition(b)),
            ),
            PropositionKind::Not(p) => {
                HirPropKind::Not(Box::new(self.resolve_proposition(p)))
            }
            PropositionKind::Equal(a, b) => HirPropKind::Equal(
                Box::new(self.resolve_term(a)),
                Box::new(self.resolve_term(b)),
            ),
            PropositionKind::GeometryRel { rel, args } => {
                let hir_args = args.iter().map(|a| self.resolve_term(a)).collect();
                HirPropKind::GeometryRel {
                    rel: rel.clone(),
                    args: hir_args,
                }
            }
            PropositionKind::ForAll(binders, inner) => {
                self.scopes.enter();
                let mut hir_binders = Vec::new();
                for b in binders {
                    let sym_id = self.id_gen.next_symbol();
                    let typ = b
                        .typ
                        .as_ref()
                        .map(|t| self.resolve_type(t))
                        .unwrap_or(HirType::Type);
                    self.scopes.insert(
                        b.name.clone(),
                        SymbolKind::Variable(sym_id),
                        b.span,
                    );
                    hir_binders.push(HirBinder {
                        id: sym_id,
                        name: b.name.clone(),
                        typ,
                        span: b.span,
                    });
                }
                let resolved_inner = self.resolve_proposition(inner);
                self.scopes.exit();
                HirPropKind::ForAll(hir_binders, Box::new(resolved_inner))
            }
            PropositionKind::Exists(binders, inner) => {
                self.scopes.enter();
                let mut hir_binders = Vec::new();
                for b in binders {
                    let sym_id = self.id_gen.next_symbol();
                    let typ = b
                        .typ
                        .as_ref()
                        .map(|t| self.resolve_type(t))
                        .unwrap_or(HirType::Type);
                    self.scopes.insert(
                        b.name.clone(),
                        SymbolKind::Variable(sym_id),
                        b.span,
                    );
                    hir_binders.push(HirBinder {
                        id: sym_id,
                        name: b.name.clone(),
                        typ,
                        span: b.span,
                    });
                }
                let resolved_inner = self.resolve_proposition(inner);
                self.scopes.exit();
                HirPropKind::Exists(hir_binders, Box::new(resolved_inner))
            }
        };

        HirProposition {
            kind,
            span: prop.span,
        }
    }

    pub fn resolve_term(&mut self, term: &Term) -> HirTerm {
        let kind = match &term.kind {
            TermKind::Var(name) => match self.scopes.resolve(name) {
                Some(sym) => match sym.kind {
                    SymbolKind::Variable(id) => HirTermKind::Var(id),
                    SymbolKind::Point(id) => HirTermKind::Point(id),
                    _ => HirTermKind::Const(name.clone()),
                },
                None => HirTermKind::Const(name.clone()),
            },
            TermKind::Const(c) => HirTermKind::Const(c.clone()),
            TermKind::Number(n) => HirTermKind::Number(*n),
            TermKind::StringLit(s) => HirTermKind::Const(s.clone()),
            TermKind::BinaryOp(op, a, b) => HirTermKind::BinaryOp(
                op.clone(),
                Box::new(self.resolve_term(a)),
                Box::new(self.resolve_term(b)),
            ),
            TermKind::App(fun, args) => {
                let resolved_fun = Box::new(self.resolve_term(fun));
                let resolved_args = args.iter().map(|a| self.resolve_term(a)).collect();
                HirTermKind::App(resolved_fun, resolved_args)
            }
        };

        HirTerm {
            kind,
            span: term.span,
        }
    }

    pub fn resolve_type(&mut self, typ: &AstType) -> HirType {
        match &typ.kind {
            AstTypeKind::Prop => HirType::Prop,
            AstTypeKind::Type => HirType::Type,
            AstTypeKind::Named(s) => HirType::Named(s.clone()),
            AstTypeKind::Arrow(a, b) => HirType::Arrow(
                Box::new(self.resolve_type(a)),
                Box::new(self.resolve_type(b)),
            ),
            _ => HirType::Type,
        }
    }

    pub fn resolve_figure(&mut self, fig: &FigureDecl) -> Option<crate::geometry::GeoFigure> {
        let mut geo_fig = crate::geometry::GeoFigure::new(fig.name.clone(), fig.span);
        self.scopes.enter();

        let mut point_map = std::collections::HashMap::new();

        for item in &fig.items {
            match item {
                FigureItem::Object { kind, name, span } => {
                    match kind.as_str() {
                        "point" => {
                            let pt_id = self.id_gen.next_point();
                            point_map.insert(name.clone(), pt_id);
                            self.scopes.insert(name.clone(), SymbolKind::Point(pt_id), *span);
                            geo_fig.add_object(crate::geometry::GeoObject::Point {
                                id: pt_id,
                                name: name.clone(),
                            });
                        }
                        "triangle" => {
                            let chars: Vec<char> = name.chars().collect();
                            if chars.len() == 3 {
                                let mut pts = Vec::new();
                                for c in chars {
                                    let p_name = c.to_string();
                                    let pt_id = *point_map.entry(p_name.clone()).or_insert_with(|| {
                                        let pid = self.id_gen.next_point();
                                        self.scopes.insert(p_name.clone(), SymbolKind::Point(pid), *span);
                                        geo_fig.add_object(crate::geometry::GeoObject::Point {
                                            id: pid,
                                            name: p_name,
                                        });
                                        pid
                                    });
                                    pts.push(pt_id);
                                }
                                let tri_id = crate::id::TriangleId(self.id_gen.next_u32());
                                geo_fig.add_object(crate::geometry::GeoObject::Triangle {
                                    id: tri_id,
                                    a: pts[0],
                                    b: pts[1],
                                    c: pts[2],
                                });
                            }
                        }
                        _ => {}
                    }
                }
                FigureItem::Given { prop, span } => {
                    let hir_prop = self.resolve_proposition(prop);
                    match &hir_prop.kind {
                        HirPropKind::Equal(left, right) => {
                            let mut extract_segment = |term: &HirTerm| -> Option<crate::geometry::GeoTerm> {
                                match &term.kind {
                                    HirTermKind::Const(name) if name.len() == 2 => {
                                        let chars: Vec<char> = name.chars().collect();
                                        let p1 = point_map.get(&chars[0].to_string()).copied();
                                        let p2 = point_map.get(&chars[1].to_string()).copied();
                                        match (p1, p2) {
                                            (Some(a), Some(b)) => Some(crate::geometry::GeoTerm::Segment(a, b)),
                                            _ => None,
                                        }
                                    }
                                    _ => None,
                                }
                            };
                            let t1 = extract_segment(left).unwrap_or(crate::geometry::GeoTerm::Named(format!("{:?}", left.kind)));
                            let t2 = extract_segment(right).unwrap_or(crate::geometry::GeoTerm::Named(format!("{:?}", right.kind)));
                            geo_fig.add_given(crate::geometry::GeoProp {
                                rel: crate::geometry::GeoRelKind::EqualLength,
                                args: vec![t1, t2],
                                span: *span,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        self.scopes.exit();
        Some(geo_fig)
    }
}

