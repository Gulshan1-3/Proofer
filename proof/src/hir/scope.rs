use crate::id::{FactId, PointId, SymbolId, TheoremId};
use crate::syntax::span::Span;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolKind {
    Variable(SymbolId),
    Theorem(TheoremId),
    Fact(FactId),
    Point(PointId),
    Type(String),
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub def_span: Span,
}

#[derive(Debug, Default, Clone)]
pub struct Scope {
    symbols: HashMap<String, Symbol>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, kind: SymbolKind, span: Span) -> Option<Symbol> {
        let name = name.into();
        self.symbols.insert(
            name.clone(),
            Symbol {
                name,
                kind,
                def_span: span,
            },
        )
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
}

impl ScopeStack {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()], // Root / global scope
        }
    }

    pub fn enter(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn exit(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn insert(&mut self, name: impl Into<String>, kind: SymbolKind, span: Span) -> Option<Symbol> {
        self.scopes
            .last_mut()
            .expect("ScopeStack must have at least one scope")
            .insert(name, kind, span)
    }

    /// Lookup respects lexical shadowing: checks from innermost scope to outermost
    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym);
            }
        }
        None
    }
}
