// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tauri::command]
fn verify_proof(code: String) -> String {
    proof::api::process_proof_request(&code)
}

#[tauri::command]
fn synthesize_step(code: String) -> String {
    let step = proof::synthesizer::ProofSynthesizer::infill_next_step(&code);
    let candidates = proof::synthesizer::ProofSynthesizer::synthesize_candidates(&code);
    let step_str = step.map(|s| s.to_json()).unwrap_or_else(|| "null".to_string());
    let candidates_str: Vec<String> = candidates.iter().map(|c| c.to_json()).collect();
    format!(
        "{{\"step\": {}, \"candidates\": [{}]}}",
        step_str,
        candidates_str.join(", ")
    )
}

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![verify_proof, synthesize_step])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
