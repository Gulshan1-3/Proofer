use crate::{
    ast::*,
    diag::Diagnostic,
    lexer::Lexer,
    syntax::span::Span,
    token::{Token, TokenType::{self, *}},
};
use crate::ast::Theorem;

pub const MAX_RECURSION_DEPTH: usize = 128;

pub struct Parser<'a> {
    tokens: Vec<Token<'a>>,
    cursor: usize,
    diagnostics: Vec<Diagnostic>,
    recursion_depth: usize,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self {
        Self::new_with_file(source, crate::syntax::span::FileId(0))
    }

    pub fn new_with_file(source: &'a str, file_id: crate::syntax::span::FileId) -> Self {
        let mut lexer = Lexer::new_with_file(source, file_id);
        let mut tokens = Vec::new();
        while let Some(tok) = lexer.next_token() {
            let is_eof = tok.kind == TokenType::Eof;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Self {
            tokens,
            cursor: 0,
            diagnostics: Vec::new(),
            recursion_depth: 0,
        }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    fn current(&self) -> &Token<'a> {
        if self.cursor < self.tokens.len() {
            &self.tokens[self.cursor]
        } else {
            self.tokens.last().expect("token stream must end with EOF")
        }
    }

    fn peek(&self) -> &TokenType {
        &self.current().kind
    }

    fn peek_span(&self) -> Span {
        self.current().span
    }

    fn advance(&mut self) -> Token<'a> {
        let tok = self.current().clone();
        if self.cursor < self.tokens.len() - 1 {
            self.cursor += 1;
        }
        tok
    }

    fn check(&self, kind: &TokenType) -> bool {
        self.peek() == kind
    }

    fn match_token(&mut self, kind: &TokenType) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenType) -> Result<Token<'a>, Diagnostic> {
        if self.peek() == &kind {
            Ok(self.advance())
        } else {
            let found = self.current().clone();
            let diag = Diagnostic::error(
                format!("Expected {:?}, but found {:?}", kind, found.kind),
                found.span,
            );
            self.diagnostics.push(diag.clone());
            Err(diag)
        }
    }

    pub fn parse_file(&mut self) -> FileAst {
        let start_span = self.peek_span();
        let mut items = Vec::new();

        while !self.check(&Eof) {
            let prev_cursor = self.cursor;
            match self.peek() {
                Theorem => match self.parse_theorem() {
                    Ok(thm) => items.push(Item::Theorem(thm)),
                    Err(_) => self.synchronize_item(),
                },
                Figure => match self.parse_figure() {
                    Ok(fig) => items.push(Item::Figure(fig)),
                    Err(_) => self.synchronize_item(),
                },
                _ => {
                    let tok = self.advance();
                    self.diagnostics.push(Diagnostic::error(
                        format!("Unexpected top-level token {:?}", tok.kind),
                        tok.span,
                    ));
                    self.synchronize_item();
                }
            }
            // Safety: if no progress was made, break to avoid infinite loop
            if self.cursor == prev_cursor {
                break;
            }
        }

        let end_span = self.peek_span();
        FileAst {
            items,
            span: start_span.merge(end_span),
        }
    }

    fn synchronize_item(&mut self) {
        if self.cursor < self.tokens.len() - 1 {
            self.cursor += 1;
        }
        while !self.check(&Eof) {
            if matches!(self.peek(), Theorem | Figure) {
                return;
            }
            if self.cursor < self.tokens.len() - 1 {
                self.cursor += 1;
            } else {
                break;
            }
        }
    }

    pub fn parse_theorem(&mut self) -> Result<Theorem, Diagnostic> {
        let thm_tok = self.expect(Theorem)?;
        let name_tok = match self.peek() {
            Ident(_) => self.advance(),
            _ => {
                let diag = Diagnostic::error("Expected theorem name", self.peek_span());
                self.diagnostics.push(diag.clone());
                return Err(diag);
            }
        };
        let name = match name_tok.kind {
            Ident(s) => s,
            _ => unreachable!(),
        };

        let _ = self.match_token(&Colon);

        let statement = self.parse_proposition()?;

        let mut proof_steps = Vec::new();
        if self.match_token(&Proof) {
            while !self.check(&End) && !self.check(&Eof) {
                match self.parse_proof_step() {
                    Ok(step) => proof_steps.push(step),
                    Err(_) => {
                        // recover to next step keyword or end
                        self.synchronize_step();
                    }
                }
            }
            let _ = self.expect(End);
        }

        let end_span = self.tokens[self.cursor.saturating_sub(1)].span;
        Ok(Theorem {
            name,
            statement,
            proof: proof_steps,
            span: thm_tok.span.merge(end_span),
        })
    }

    fn synchronize_step(&mut self) {
        while !self.check(&Eof) && !self.check(&End) {
            match self.peek() {
                Take | Suppose | Let | Have | Construct | Show | Derive | Use | Therefore
                | Cases | Contradict => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    pub fn parse_figure(&mut self) -> Result<FigureDecl, Diagnostic> {
        let fig_tok = self.expect(Figure)?;
        let name_tok = match self.peek() {
            Ident(_) => self.advance(),
            _ => {
                let diag = Diagnostic::error("Expected figure name", self.peek_span());
                self.diagnostics.push(diag.clone());
                return Err(diag);
            }
        };
        let name = match name_tok.kind {
            Ident(s) => s,
            _ => unreachable!(),
        };

        let mut items = Vec::new();
        while !self.check(&End) && !self.check(&Eof) {
            if self.match_token(&Given) {
                let prop = self.parse_proposition()?;
                let span = prop.span;
                items.push(FigureItem::Given { prop, span });
            } else if let Ident(kind_name) = self.peek().clone() {
                let start_tok = self.advance();
                if let Ident(obj_name) = self.peek().clone() {
                    let end_tok = self.advance();
                    items.push(FigureItem::Object {
                        kind: kind_name,
                        name: obj_name,
                        span: start_tok.span.merge(end_tok.span),
                    });
                } else {
                    let diag = Diagnostic::error("Expected object name after kind in figure", self.peek_span());
                    self.diagnostics.push(diag);
                }
            } else {
                let tok = self.advance();
                let diag = Diagnostic::error(format!("Unexpected token in figure: {:?}", tok.kind), tok.span);
                self.diagnostics.push(diag);
            }
        }
        let end_tok = self.expect(End)?;
        Ok(FigureDecl {
            name,
            items,
            span: fig_tok.span.merge(end_tok.span),
        })
    }

    pub fn parse_proof_step(&mut self) -> Result<ProofStep, Diagnostic> {
        let start_span = self.peek_span();
        match self.peek() {
            Take => {
                self.advance();
                let mut binders = Vec::new();
                while !self.is_step_terminator() {
                    binders.push(self.parse_binder()?);
                    if !self.match_token(&Comma) && !matches!(self.peek(), Ident(_) | LParen) {
                        break;
                    }
                }
                let end_span = binders.last().map(|b| b.span).unwrap_or(start_span);
                Ok(ProofStep {
                    kind: ProofStepKind::Take(binders),
                    span: start_span.merge(end_span),
                })
            }
            Suppose => {
                self.advance();
                let name = if let Ident(s) = self.peek() {
                    let s = s.clone();
                    self.advance();
                    self.expect(Colon)?;
                    s
                } else {
                    "h".to_string()
                };
                let prop = self.parse_proposition()?;
                let span = start_span.merge(prop.span);
                Ok(ProofStep {
                    kind: ProofStepKind::Suppose { name, prop },
                    span,
                })
            }
            Let => {
                self.advance();
                let name = match self.advance().kind {
                    Ident(s) => s,
                    other => {
                        return Err(Diagnostic::error(format!("Expected let variable name, found {:?}", other), start_span));
                    }
                };
                let typ = if self.match_token(&Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                let val = if self.match_token(&Equal) || self.match_token(&ColonEqual) {
                    Some(self.parse_term()?)
                } else {
                    None
                };
                let end_span = val.as_ref().map(|v| v.span).or_else(|| typ.as_ref().map(|t| t.span)).unwrap_or(start_span);
                Ok(ProofStep {
                    kind: ProofStepKind::Let { name, val, typ },
                    span: start_span.merge(end_span),
                })
            }
            Have => {
                self.advance();
                let mut name = None;
                if let Ident(s) = self.peek().clone() {
                    // Check if followed by colon
                    if self.cursor + 1 < self.tokens.len() && self.tokens[self.cursor + 1].kind == Colon {
                        self.advance();
                        self.advance(); // consume colon
                        name = Some(s);
                    }
                }
                let prop = self.parse_proposition()?;
                let mut from = Vec::new();
                if self.match_token(&From) {
                    while let Ident(s) = self.peek().clone() {
                        self.advance();
                        from.push(s);
                        if !self.match_token(&Comma) {
                            break;
                        }
                    }
                }
                let end_span = self.tokens[self.cursor.saturating_sub(1)].span;
                Ok(ProofStep {
                    kind: ProofStepKind::Have {
                        name,
                        prop,
                        from,
                        by: None,
                    },
                    span: start_span.merge(end_span),
                })
            }
            Derive => {
                self.advance();
                let mut name = None;
                if let Ident(s) = self.peek().clone() {
                    if self.cursor + 1 < self.tokens.len() && self.tokens[self.cursor + 1].kind == Colon {
                        self.advance();
                        self.advance();
                        name = Some(s);
                    }
                }
                let prop = self.parse_proposition()?;
                let mut from = Vec::new();
                if self.match_token(&From) {
                    while let Ident(s) = self.peek().clone() {
                        self.advance();
                        from.push(s);
                        if !self.match_token(&Comma) {
                            break;
                        }
                    }
                }
                let mut using_rule = None;
                if self.match_token(&Using) || self.match_token(&Use) {
                    if let Ident(s) = self.peek().clone() {
                        self.advance();
                        using_rule = Some(s);
                    }
                }
                let end_span = self.tokens[self.cursor.saturating_sub(1)].span;
                Ok(ProofStep {
                    kind: ProofStepKind::Derive {
                        name,
                        prop,
                        from,
                        using_rule,
                    },
                    span: start_span.merge(end_span),
                })
            }
            Construct => {
                self.advance();
                let name = match self.advance().kind {
                    Ident(s) => s,
                    other => return Err(Diagnostic::error(format!("Expected construction target name, found {:?}", other), start_span)),
                };
                let _ = self.match_token(&As);
                let as_desc = match self.advance().kind {
                    Ident(s) => s,
                    other => return Err(Diagnostic::error(format!("Expected construction description, found {:?}", other), start_span)),
                };
                let _ = self.match_token(&Ident("of".into()));
                let mut args = Vec::new();
                while !self.is_step_terminator() {
                    args.push(self.parse_term()?);
                    if !self.match_token(&Comma) && !matches!(self.peek(), Ident(_)) {
                        break;
                    }
                }
                let end_span = self.tokens[self.cursor.saturating_sub(1)].span;
                Ok(ProofStep {
                    kind: ProofStepKind::Construct { name, as_desc, args },
                    span: start_span.merge(end_span),
                })
            }
            Show => {
                self.advance();
                let prop = self.parse_proposition()?;
                let span = start_span.merge(prop.span);
                Ok(ProofStep {
                    kind: ProofStepKind::Show(prop),
                    span,
                })
            }
            Therefore => {
                self.advance();
                let prop = self.parse_proposition()?;
                let mut from = Vec::new();
                if self.match_token(&From) {
                    while let Ident(s) = self.peek().clone() {
                        self.advance();
                        from.push(s);
                        if !self.match_token(&Comma) {
                            break;
                        }
                    }
                }
                let end_span = self.tokens[self.cursor.saturating_sub(1)].span;
                Ok(ProofStep {
                    kind: ProofStepKind::Therefore { prop, from },
                    span: start_span.merge(end_span),
                })
            }
            Use => {
                self.advance();
                let mut terms = Vec::new();
                while !self.is_step_terminator() {
                    terms.push(self.parse_term()?);
                    if !self.match_token(&Comma) {
                        break;
                    }
                }
                let end_span = self.tokens[self.cursor.saturating_sub(1)].span;
                Ok(ProofStep {
                    kind: ProofStepKind::Use(terms),
                    span: start_span.merge(end_span),
                })
            }
            Cases => {
                self.advance();
                let target = match self.advance().kind {
                    Ident(s) => s,
                    other => return Err(Diagnostic::error(format!("Expected cases identifier, found {:?}", other), start_span)),
                };
                let end_span = self.tokens[self.cursor.saturating_sub(1)].span;
                Ok(ProofStep {
                    kind: ProofStepKind::Cases { target },
                    span: start_span.merge(end_span),
                })
            }
            Contradict => {
                self.advance();
                let target = if let Ident(s) = self.peek().clone() {
                    self.advance();
                    Some(s)
                } else {
                    None
                };
                let end_span = self.tokens[self.cursor.saturating_sub(1)].span;
                Ok(ProofStep {
                    kind: ProofStepKind::Contradict { target },
                    span: start_span.merge(end_span),
                })
            }
            Ident(name) if name == "assume" => {
                self.advance();
                let prop = self.parse_proposition()?;
                let span = start_span.merge(prop.span);
                Ok(ProofStep {
                    kind: ProofStepKind::Assume(prop),
                    span,
                })
            }
            Ident(name) if name == "intro" => {
                self.advance();
                let mut vars = Vec::new();
                while let Ident(v) = self.peek().clone() {
                    self.advance();
                    vars.push(v);
                }
                let end_span = self.tokens[self.cursor.saturating_sub(1)].span;
                Ok(ProofStep {
                    kind: ProofStepKind::Intro(vars),
                    span: start_span.merge(end_span),
                })
            }
            Ident(name) if name == "apply" => {
                self.advance();
                let thm = match self.advance().kind {
                    Ident(s) => s,
                    other => return Err(Diagnostic::error(format!("Expected theorem name after apply, found {:?}", other), start_span)),
                };
                let mut args = Vec::new();
                if self.peek() == &Ident("to".into()) {
                    self.advance();
                }
                while !self.is_step_terminator() {
                    args.push(self.parse_term()?);
                    if !self.match_token(&Comma) && self.peek() != &Ident("and".into()) {
                        break;
                    }
                    if self.peek() == &Ident("and".into()) {
                        self.advance();
                    }
                }
                let end_span = self.tokens[self.cursor.saturating_sub(1)].span;
                Ok(ProofStep {
                    kind: ProofStepKind::Apply { theorem: thm, args },
                    span: start_span.merge(end_span),
                })
            }
            _ => {
                let tok = self.advance();
                let diag = Diagnostic::error(format!("Unknown proof step keyword {:?}", tok.kind), tok.span);
                self.diagnostics.push(diag.clone());
                Err(diag)
            }
        }
    }

    fn is_step_terminator(&self) -> bool {
        matches!(
            self.peek(),
            Eof | End | Take | Suppose | Let | Have | Construct | Show | Derive | Use | Therefore | Cases | Contradict | From
        )
    }

    pub fn parse_proposition(&mut self) -> Result<Proposition, Diagnostic> {
        if self.recursion_depth >= MAX_RECURSION_DEPTH {
            let diag = Diagnostic::error(
                format!("Maximum recursion depth ({}) exceeded: expression is nested too deeply", MAX_RECURSION_DEPTH),
                self.peek_span(),
            );
            self.diagnostics.push(diag.clone());
            return Err(diag);
        }
        self.recursion_depth += 1;
        let res = self.parse_prop_quantifier();
        self.recursion_depth -= 1;
        res
    }

    // 1. Quantifiers: ∀x : T, P / ∃x : T, P
    fn parse_prop_quantifier(&mut self) -> Result<Proposition, Diagnostic> {
        let start_span = self.peek_span();
        if self.check(&ForAll) || self.check(&Exists) {
            let is_forall = self.advance().kind == ForAll;
            let mut binders = Vec::new();
            while !self.check(&Dot) && !self.check(&Comma) && !self.check(&Eof) {
                binders.push(self.parse_binder()?);
                if self.check(&Dot) || self.check(&Comma) {
                    break;
                }
            }
            if self.check(&Dot) || self.check(&Comma) {
                self.advance();
            }
            let body = self.parse_proposition()?;
            let span = start_span.merge(body.span);
            let kind = if is_forall {
                PropositionKind::ForAll(binders, Box::new(body))
            } else {
                PropositionKind::Exists(binders, Box::new(body))
            };
            return Ok(Proposition { kind, span });
        }

        self.parse_prop_iff()
    }

    // 2. Iff (<->, ↔)
    fn parse_prop_iff(&mut self) -> Result<Proposition, Diagnostic> {
        let mut left = self.parse_prop_implies()?;
        while self.match_token(&Iff) {
            let right = self.parse_prop_implies()?;
            let span = left.span.merge(right.span);
            left = Proposition {
                kind: PropositionKind::Iff(Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    // 3. Implies (->, →) - right associative
    fn parse_prop_implies(&mut self) -> Result<Proposition, Diagnostic> {
        let left = self.parse_prop_or()?;
        if self.match_token(&Implies) {
            let right = self.parse_prop_implies()?;
            let span = left.span.merge(right.span);
            return Ok(Proposition {
                kind: PropositionKind::Implies(Box::new(left), Box::new(right)),
                span,
            });
        }
        Ok(left)
    }

    // 4. Or (∨, or)
    fn parse_prop_or(&mut self) -> Result<Proposition, Diagnostic> {
        let mut left = self.parse_prop_and()?;
        while self.match_token(&Or) {
            let right = self.parse_prop_and()?;
            let span = left.span.merge(right.span);
            left = Proposition {
                kind: PropositionKind::Or(Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    // 5. And (∧, and)
    fn parse_prop_and(&mut self) -> Result<Proposition, Diagnostic> {
        let mut left = self.parse_prop_not()?;
        while self.match_token(&And) {
            let right = self.parse_prop_not()?;
            let span = left.span.merge(right.span);
            left = Proposition {
                kind: PropositionKind::And(Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    // 6. Not (¬, !, not)
    fn parse_prop_not(&mut self) -> Result<Proposition, Diagnostic> {
        let start_span = self.peek_span();
        if self.match_token(&Not) {
            let sub = self.parse_prop_not()?;
            let span = start_span.merge(sub.span);
            return Ok(Proposition {
                kind: PropositionKind::Not(Box::new(sub)),
                span,
            });
        }
        self.parse_prop_rel_or_atom()
    }

    // 7. Relations (Equal, Geometry relations) or Atomic
    fn parse_prop_rel_or_atom(&mut self) -> Result<Proposition, Diagnostic> {
        let start_span = self.peek_span();

        // Check grouped proposition: ( P )
        if self.check(&LParen) {
            // Distinguish whether it's a grouped proposition or a term
            // Try parsing as prop; if followed by =, it was a term
            let checkpoint = self.cursor;
            let diag_checkpoint = self.diagnostics.len();
            self.advance(); // consume (
            if let Ok(inner) = self.parse_proposition() {
                if self.match_token(&RParen) {
                    // Check if followed by = (i.e. (a + b) = c)
                    if self.match_token(&Equal) {
                        let right = self.parse_term()?;
                        let span = start_span.merge(right.span);
                        // Re-parse left as term
                        self.cursor = checkpoint;
                        self.diagnostics.truncate(diag_checkpoint);
                        let left_term = self.parse_term()?;
                        self.expect(Equal)?;
                        let r_term = self.parse_term()?;
                        return Ok(Proposition {
                            kind: PropositionKind::Equal(Box::new(left_term), Box::new(r_term)),
                            span,
                        });
                    }
                    return Ok(Proposition {
                        kind: inner.kind,
                        span: start_span.merge(self.tokens[self.cursor.saturating_sub(1)].span),
                    });
                }
            }
            self.cursor = checkpoint;
            self.diagnostics.truncate(diag_checkpoint);
        }

        // Try term first to see if it's an equality or geometry relation
        let term = self.parse_term()?;
        if self.match_token(&Equal) {
            let right = self.parse_term()?;
            let span = term.span.merge(right.span);
            return Ok(Proposition {
                kind: PropositionKind::Equal(Box::new(term), Box::new(right)),
                span,
            });
        }

        // Geometry relation check (e.g. `triangle ABC congruent triangle DEF`)
        if let Ident(rel_name) = self.peek().clone() {
            if matches!(rel_name.as_str(), "congruent" | "similar" | "perpendicular" | "parallel" | "on" | "midpoint_of" | "bisects") {
                self.advance();
                let right = self.parse_term()?;
                let span = term.span.merge(right.span);
                return Ok(Proposition {
                    kind: PropositionKind::GeometryRel {
                        rel: rel_name,
                        args: vec![term, right],
                    },
                    span,
                });
            }
        }

        // Atomic proposition from term: if term was App or Var
        match term.kind {
            TermKind::Var(name) => Ok(Proposition {
                kind: PropositionKind::Atomic(name, vec![]),
                span: term.span,
            }),
            TermKind::App(fun, args) => {
                if let TermKind::Var(name) = fun.kind {
                    Ok(Proposition {
                        kind: PropositionKind::Atomic(name, args),
                        span: term.span,
                    })
                } else {
                    Ok(Proposition {
                        kind: PropositionKind::Atomic(fun.to_string(), args),
                        span: term.span,
                    })
                }
            }
            _ => Ok(Proposition {
                kind: PropositionKind::Atomic(term.to_string(), vec![]),
                span: term.span,
            }),
        }
    }

    pub fn parse_term(&mut self) -> Result<Term, Diagnostic> {
        if self.recursion_depth >= MAX_RECURSION_DEPTH {
            let diag = Diagnostic::error(
                format!("Maximum recursion depth ({}) exceeded: term is nested too deeply", MAX_RECURSION_DEPTH),
                self.peek_span(),
            );
            self.diagnostics.push(diag.clone());
            return Err(diag);
        }
        self.recursion_depth += 1;
        let res = self.parse_term_additive();
        self.recursion_depth -= 1;
        res
    }

    fn parse_term_additive(&mut self) -> Result<Term, Diagnostic> {
        let mut left = self.parse_term_multiplicative()?;
        while self.check(&Plus) || self.check(&Minus) {
            let op_tok = self.advance();
            let op_str = if op_tok.kind == Plus { "+" } else { "-" };
            let right = self.parse_term_multiplicative()?;
            let span = left.span.merge(right.span);
            left = Term {
                kind: TermKind::BinaryOp(op_str.to_string(), Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_term_multiplicative(&mut self) -> Result<Term, Diagnostic> {
        let mut left = self.parse_term_app()?;
        while self.match_token(&Star) {
            let right = self.parse_term_app()?;
            let span = left.span.merge(right.span);
            left = Term {
                kind: TermKind::BinaryOp("*".to_string(), Box::new(left), Box::new(right)),
                span,
            };
        }
        Ok(left)
    }

    fn parse_term_app(&mut self) -> Result<Term, Diagnostic> {
        let mut base = self.parse_term_primary()?;

        // If followed by '(' -> function application: f(x, y)
        if self.match_token(&LParen) {
            let mut args = Vec::new();
            if !self.check(&RParen) {
                loop {
                    args.push(self.parse_term()?);
                    if !self.match_token(&Comma) {
                        break;
                    }
                }
            }
            let rparen = self.expect(RParen)?;
            let span = base.span.merge(rparen.span);
            base = Term {
                kind: TermKind::App(Box::new(base), args),
                span,
            };
        }

        // Geometry compound identifier: `triangle ABC` or `angle ABC`
        if let TermKind::Var(ref v) = base.kind {
            if matches!(v.as_str(), "triangle" | "angle" | "line" | "segment" | "circle") {
                if let Ident(_) = self.peek() {
                    let next_term = self.parse_term_primary()?;
                    let span = base.span.merge(next_term.span);
                    base = Term {
                        kind: TermKind::App(Box::new(base), vec![next_term]),
                        span,
                    };
                }
            }
        }

        Ok(base)
    }

    fn parse_term_primary(&mut self) -> Result<Term, Diagnostic> {
        let tok = self.advance();
        match tok.kind {
            Ident(name) => Ok(Term {
                kind: TermKind::Var(name),
                span: tok.span,
            }),
            Number(n) => Ok(Term {
                kind: TermKind::Number(n),
                span: tok.span,
            }),
            StringLiteral(s) => Ok(Term {
                kind: TermKind::StringLit(s),
                span: tok.span,
            }),
            LParen => {
                let inner = self.parse_term()?;
                let rparen = self.expect(RParen)?;
                Ok(Term {
                    kind: inner.kind,
                    span: tok.span.merge(rparen.span),
                })
            }
            other => {
                let diag = Diagnostic::error(format!("Expected term, but found {:?}", other), tok.span);
                self.diagnostics.push(diag.clone());
                Err(diag)
            }
        }
    }

    pub fn parse_binder(&mut self) -> Result<Binder, Diagnostic> {
        let start_span = self.peek_span();
        if self.match_token(&LParen) {
            let name = match self.advance().kind {
                Ident(s) => s,
                other => return Err(Diagnostic::error(format!("Expected binder name, found {:?}", other), start_span)),
            };
            self.expect(Colon)?;
            let typ = self.parse_type()?;
            let rparen = self.expect(RParen)?;
            return Ok(Binder {
                name,
                typ: Some(typ),
                span: start_span.merge(rparen.span),
            });
        }

        let name_tok = self.advance();
        let name = match name_tok.kind {
            Ident(s) => s,
            other => return Err(Diagnostic::error(format!("Expected binder identifier, found {:?}", other), name_tok.span)),
        };

        let typ = if self.match_token(&Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let end_span = typ.as_ref().map(|t| t.span).unwrap_or(name_tok.span);
        Ok(Binder {
            name,
            typ,
            span: name_tok.span.merge(end_span),
        })
    }

    pub fn parse_type(&mut self) -> Result<AstType, Diagnostic> {
        let start_span = self.peek_span();
        let base = match self.advance().kind {
            Ident(s) if s == "Prop" => AstType {
                kind: AstTypeKind::Prop,
                span: start_span,
            },
            Ident(s) if s == "Type" => AstType {
                kind: AstTypeKind::Type,
                span: start_span,
            },
            Ident(s) => AstType {
                kind: AstTypeKind::Named(s),
                span: start_span,
            },
            LParen => {
                let inner = self.parse_type()?;
                let rparen = self.expect(RParen)?;
                AstType {
                    kind: inner.kind,
                    span: start_span.merge(rparen.span),
                }
            }
            other => return Err(Diagnostic::error(format!("Expected type, found {:?}", other), start_span)),
        };

        if self.match_token(&Implies) {
            let target = self.parse_type()?;
            let span = base.span.merge(target.span);
            return Ok(AstType {
                kind: AstTypeKind::Arrow(Box::new(base), Box::new(target)),
                span,
            });
        }

        Ok(base)
    }
}
