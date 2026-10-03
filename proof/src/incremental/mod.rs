//! Incremental Compilation & Low-Latency Performance Engine (Stage 9).
//!
//! Provides:
//! - Thread-safe cancellation of in-flight compilation on rapid keystrokes.
//! - Content-addressed item hashing (signature vs body).
//! - Declaration-level dependency graph and selective invalidation.
//! - Memoized theorem elaboration and kernel verification cache.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::ast::{Item, Theorem, ProofStepKind};
use crate::parser::Parser;
use crate::hir::resolve::Resolver;
use crate::elab::{Elaborator, ElabResult};
use crate::id::Revision;

/// Thread-safe cancellation token used by the editor/server to abort in-flight work.
#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub fn check_cancelled(&self) -> Result<(), CancelledError> {
        if self.is_cancelled() {
            Err(CancelledError)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CancelledError;

impl std::fmt::Display for CancelledError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compilation cancelled by newer revision")
    }
}

impl std::error::Error for CancelledError {}

/// Stable 64-bit content hash of an AST item or string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemHash(pub u64);

impl ItemHash {
    /// FNV-1a 64-bit non-cryptographic fast hash.
    pub fn from_str(s: &str) -> Self {
        let mut hash: u64 = 0xcbf29ce484222325;
        for byte in s.as_bytes() {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        Self(hash)
    }
}

/// Compute signature hash (theorem name + statement) and body hash (proof steps).
pub fn hash_theorem(thm: &Theorem) -> (ItemHash, ItemHash) {
    let sig_str = format!("{}:{}", thm.name, thm.statement);
    let mut body_str = String::new();
    for step in &thm.proof {
        body_str.push_str(&format!("{:?}", step.kind));
    }
    (ItemHash::from_str(&sig_str), ItemHash::from_str(&body_str))
}

/// Directed dependency graph between theorems.
#[derive(Debug, Clone, Default)]
pub struct DeclDependencyGraph {
    /// Mapping from theorem name to the set of theorems it directly depends on.
    pub dependencies: HashMap<String, HashSet<String>>,
    /// Reverse mapping: theorem name -> theorems that depend on it.
    pub dependents: HashMap<String, HashSet<String>>,
}

impl DeclDependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.dependencies.clear();
        self.dependents.clear();
    }

    pub fn record_dep(&mut self, from_thm: &str, depends_on: &str) {
        self.dependencies
            .entry(from_thm.to_string())
            .or_default()
            .insert(depends_on.to_string());
        self.dependents
            .entry(depends_on.to_string())
            .or_default()
            .insert(from_thm.to_string());
    }

    /// Extract all downstream dependents that must be invalidated if `thm` changes.
    pub fn transitive_dependents(&self, thm: &str) -> HashSet<String> {
        let mut result = HashSet::new();
        let mut queue = vec![thm.to_string()];

        while let Some(current) = queue.pop() {
            if let Some(deps) = self.dependents.get(&current) {
                for d in deps {
                    if result.insert(d.clone()) {
                        queue.push(d.clone());
                    }
                }
            }
        }
        result
    }
}

/// Cached result for a single verified theorem.
#[derive(Debug, Clone)]
pub struct CachedTheorem {
    pub name: String,
    pub sig_hash: ItemHash,
    pub body_hash: ItemHash,
    pub elab_result: ElabResult,
}

/// Result metrics and data of an incremental compilation cycle.
#[derive(Debug, Clone)]
pub struct IncrementalCompileResult {
    pub revision: Revision,
    pub verified: bool,
    pub theorems: Vec<ElabResult>,
    pub reused_count: usize,
    pub recompiled_count: usize,
    pub elapsed_micros: u128,
}

/// Incremental compilation manager maintaining caches across document revisions.
#[derive(Debug, Default)]
pub struct IncrementalEngine {
    current_revision: u32,
    theorem_cache: HashMap<String, CachedTheorem>,
    dependency_graph: DeclDependencyGraph,
}

impl IncrementalEngine {
    pub fn new() -> Self {
        Self {
            current_revision: 0,
            theorem_cache: HashMap::new(),
            dependency_graph: DeclDependencyGraph::new(),
        }
    }

    pub fn current_revision(&self) -> Revision {
        Revision(self.current_revision)
    }

    /// Compile source code incrementally, reusing unchanged theorem results.
    pub fn compile_source(
        &mut self,
        code: &str,
        cancel_token: Option<&CancellationToken>,
    ) -> Result<IncrementalCompileResult, CancelledError> {
        let start_time = Instant::now();
        self.current_revision += 1;
        let rev = Revision(self.current_revision);

        if let Some(tok) = cancel_token {
            tok.check_cancelled()?;
        }

        let mut parser = Parser::new(code);
        let file_ast = parser.parse_file();

        if let Some(tok) = cancel_token {
            tok.check_cancelled()?;
        }

        // Analyze which theorems are present in this revision
        let mut current_theorems: HashMap<String, &Theorem> = HashMap::new();
        for item in &file_ast.items {
            if let Item::Theorem(thm) = item {
                current_theorems.insert(thm.name.clone(), thm);
            }
        }

        // Rebuild dependency graph for the current AST
        let mut new_deps = DeclDependencyGraph::new();
        for (name, thm) in &current_theorems {
            for step in &thm.proof {
                let from_list = match &step.kind {
                    ProofStepKind::Have { from, .. } => from.clone(),
                    ProofStepKind::Derive { from, .. } => from.clone(),
                    ProofStepKind::Therefore { from, .. } => from.clone(),
                    _ => Vec::new(),
                };
                for ref_name in from_list {
                    if current_theorems.contains_key(&ref_name) {
                        new_deps.record_dep(name, &ref_name);
                    }
                }
            }
        }

        // Invalidation analysis
        let mut invalidated: HashSet<String> = HashSet::new();

        // 1. Invalidate theorems that no longer exist or whose signature changed
        for (cached_name, cached) in &self.theorem_cache {
            if let Some(current_thm) = current_theorems.get(cached_name) {
                let (sig_hash, body_hash) = hash_theorem(current_thm);
                if sig_hash != cached.sig_hash {
                    // Signature changed: invalidate self and all downstream dependents
                    invalidated.insert(cached_name.clone());
                    let deps = self.dependency_graph.transitive_dependents(cached_name);
                    invalidated.extend(deps);
                } else if body_hash != cached.body_hash {
                    // Only body changed: invalidate self
                    invalidated.insert(cached_name.clone());
                }
            } else {
                // Theorem was deleted
                invalidated.insert(cached_name.clone());
                let deps = self.dependency_graph.transitive_dependents(cached_name);
                invalidated.extend(deps);
            }
        }

        // Remove invalidated entries from cache
        for name in &invalidated {
            self.theorem_cache.remove(name);
        }
        self.dependency_graph = new_deps;

        // Perform HIR resolution
        let mut resolver = Resolver::new();
        let pkg = resolver.resolve_file(&file_ast);

        if let Some(tok) = cancel_token {
            tok.check_cancelled()?;
        }

        let mut elaborator = Elaborator::new();
        let elab_results = elaborator.elaborate_package(&pkg);

        if let Some(tok) = cancel_token {
            tok.check_cancelled()?;
        }

        let mut final_results = Vec::new();
        let mut reused_count = 0;
        let mut recompiled_count = 0;
        let mut all_verified = true;

        for res in elab_results {
            let thm_name = &res.name;
            if let Some(cached) = self.theorem_cache.get(thm_name) {
                // Reused from cache
                reused_count += 1;
                if !cached.elab_result.is_verified() {
                    all_verified = false;
                }
                final_results.push(cached.elab_result.clone());
            } else {
                // Newly compiled and verified
                recompiled_count += 1;
                if !res.is_verified() {
                    all_verified = false;
                }
                if let Some(thm) = current_theorems.get(thm_name) {
                    let (sig_hash, body_hash) = hash_theorem(thm);
                    self.theorem_cache.insert(
                        thm_name.clone(),
                        CachedTheorem {
                            name: thm_name.clone(),
                            sig_hash,
                            body_hash,
                            elab_result: res.clone(),
                        },
                    );
                }
                final_results.push(res);
            }
        }

        let elapsed = start_time.elapsed().as_micros();

        Ok(IncrementalCompileResult {
            revision: rev,
            verified: all_verified,
            theorems: final_results,
            reused_count,
            recompiled_count,
            elapsed_micros: elapsed,
        })
    }
}
