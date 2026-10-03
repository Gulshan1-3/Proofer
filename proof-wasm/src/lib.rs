use wasm_bindgen::prelude::*;
use proof::api::process_proof_request;

/// High-performance in-browser WebAssembly entry point.
/// Evaluates and verifies the proof code directly in client memory with 0ms network latency.
#[wasm_bindgen]
pub fn verify_proof_wasm(code: &str) -> String {
    process_proof_request(code)
}

#[wasm_bindgen]
pub fn proofer_version() -> String {
    "v0.4.2-wasm".to_string()
}
