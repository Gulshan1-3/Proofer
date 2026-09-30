//! Proof kernel — the trusted verification core.
//!
//! This module is the trust boundary. Everything in here must be correct.
//! Everything outside (parser, solver, editor, renderer) is untrusted.

pub mod types;
pub mod proof_object;
pub mod checker;

pub use types::{KProp, KTerm, KBinder};
pub use proof_object::{ProofNode, ProofNodeId, ProofObject};
pub use checker::{Checker, CheckError, CheckResult, Context, check_proof};
