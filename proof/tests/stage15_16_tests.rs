//! Integration tests for Stage 15 (Package Manager & Mathlib) and Stage 16 (Proof Synthesizer).

use proof::package::PackageManager;
use proof::synthesizer::ProofSynthesizer;
use std::path::Path;

#[test]
fn test_stage15_scaffold_and_build_project() {
    let temp_dir = std::env::temp_dir().join("proofer_test_pkg");
    if temp_dir.exists() {
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    // 1. Scaffolding
    PackageManager::create_project(&temp_dir, "test_geometry_pkg").expect("Failed to scaffold package");
    assert!(temp_dir.join("proof.toml").exists());
    assert!(temp_dir.join("src/main.proof").exists());

    // 2. Build and verify scaffolded package
    let summary = PackageManager::build_project(&temp_dir).expect("Failed to build package");
    assert_eq!(summary.package_name, "test_geometry_pkg");
    assert_eq!(summary.version, "0.1.0");
    assert_eq!(summary.theorems_count, 1);
    assert_eq!(summary.verified_count, 1);
    assert!(summary.errors.is_empty(), "Expected no verification errors: {:?}", summary.errors);

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_stage15_mathlib_package_builds_and_verifies() {
    let mathlib_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("mathlib");
    if mathlib_path.exists() {
        let summary = PackageManager::build_project(&mathlib_path).expect("Failed to build mathlib");
        assert_eq!(summary.package_name, "mathlib");
        assert!(summary.files_count >= 4, "Expected at least 4 mathlib modules, got {}", summary.files_count);
        assert!(summary.theorems_count >= 10, "Expected at least 10 formal theorems in mathlib, got {}", summary.theorems_count);
        assert_eq!(summary.theorems_count, summary.verified_count, "All mathlib theorems must verify with kernel! Errors: {:?}", summary.errors);
    }
}

#[test]
fn test_stage16_synthesizer_infills_valid_step() {
    let incomplete_proof = r#"theorem isosceles_test:
    AB = AC -> angle_ABC = angle_ACB
proof
    suppose h1 : AB = AC
end
"#;

    let step_opt = ProofSynthesizer::infill_next_step(incomplete_proof);
    assert!(step_opt.is_some(), "Synthesizer must find next step");
    let step = step_opt.unwrap();
    assert!(step.verified_by_kernel, "Synthesized step must be kernel verified");
    assert!(step.text.contains("derive") || step.text.contains("therefore"));
    assert!(step.text.contains("angle_ABC = angle_ACB"));
}

#[test]
fn test_stage16_synthesizer_discharges_goal_immediately() {
    let ready_to_discharge = r#"theorem direct_discharge:
    AB = AC -> angle_ABC = angle_ACB
proof
    suppose h1 : AB = AC
    derive h2 : angle_ABC = angle_ACB from h1 using IsoscelesBaseAngles
end
"#;

    let step_opt = ProofSynthesizer::infill_next_step(ready_to_discharge);
    assert!(step_opt.is_some(), "Synthesizer must discharge target goal");
    let step = step_opt.unwrap();
    assert!(step.verified_by_kernel, "Step must be kernel verified");
    assert_eq!(step.kind, "therefore");
    assert_eq!(step.conclusion, "angle_ABC = angle_ACB");
}

#[test]
fn test_stage16_zero_hallucination_guardrail() {
    // If no sound deduction exists, synthesizer must not hallucinate a valid step
    let unprovable_context = r#"theorem bogus:
    random_fact_xyz -> target_abc
proof
    suppose h1 : random_fact_xyz
end
"#;

    let step_opt = ProofSynthesizer::infill_next_step(unprovable_context);
    assert!(step_opt.is_none(), "Synthesizer must NEVER hallucinate when kernel cannot verify");
}
