//! Security and Adversarial Penetration Test Suite for Proofer Kernel.
//!
//! Validates:
//! 1. Rejection of unproven `have` axiom injections.
//! 2. Rejection of geometric certificate wildcard bypasses (`Thales`, `ModusPonens`, etc.).
//! 3. Graceful handling of nested recursion DoS attempts without stack overflow.
//! 4. Rejection of module loader path traversal attempts.
//! 5. Preservation of legitimate formal deductions.

use proof::api::process_proof_request;
use proof::module::ModuleLoader;

#[test]
fn test_adversarial_rejection_of_unproven_have_axiom() {
    let malicious_code = r#"
theorem pwn_math : False
proof
    have h : False
    therefore False from h
end
"#;
    let res_json = process_proof_request(malicious_code);
    assert!(res_json.contains("\"verified\": false") || res_json.contains("\"verified\":false"),
        "Kernel must reject unproven 'have' step claiming False: {}", res_json);
}

#[test]
fn test_adversarial_rejection_of_false_via_have_1_equals_2() {
    let malicious_code = r#"
theorem fake : 1 = 2
proof
    have h : 1 = 2
    therefore 1 = 2 from h
end
"#;
    let res_json = process_proof_request(malicious_code);
    assert!(res_json.contains("\"verified\": false") || res_json.contains("\"verified\":false"),
        "Kernel must reject unproven claim 1 = 2: {}", res_json);
}

#[test]
fn test_adversarial_rejection_of_wildcard_thales_with_no_premises() {
    let malicious_code = r#"
theorem fake : 0 = 1
proof
    derive h : 0 = 1 using Thales
    therefore 0 = 1 from h
end
"#;
    let res_json = process_proof_request(malicious_code);
    assert!(res_json.contains("\"verified\": false") || res_json.contains("\"verified\":false"),
        "Kernel must reject 0 = 1 using Thales without premises: {}", res_json);
}

#[test]
fn test_adversarial_rejection_of_wildcard_thales_with_fake_premise() {
    let malicious_code = r#"
theorem fake : 0 = 1
proof
    suppose h1 : diameter(AB)
    derive h : 0 = 1 from h1 using Thales
    therefore 0 = 1 from h
end
"#;
    let res_json = process_proof_request(malicious_code);
    assert!(res_json.contains("\"verified\": false") || res_json.contains("\"verified\":false"),
        "Kernel must reject 0 = 1 even with diameter premise: {}", res_json);
}

#[test]
fn test_adversarial_rejection_of_modus_ponens_with_unmatched_conclusion() {
    let malicious_code = r#"
theorem fake : False
proof
    suppose h1 : P and (P -> Q)
    derive h : False from h1 using ModusPonens
    therefore False from h
end
"#;
    let res_json = process_proof_request(malicious_code);
    assert!(res_json.contains("\"verified\": false") || res_json.contains("\"verified\":false"),
        "Kernel must reject ModusPonens when conclusion does not match: {}", res_json);
}

#[test]
fn test_parser_stack_overflow_depth_limit() {
    // 1,000 nested parentheses should gracefully return diagnostic rather than SIGABRT stack overflow
    let deep_nested = format!("theorem deep : {}P{}\nproof\nend", "(".repeat(1000), ")".repeat(1000));
    let res_json = process_proof_request(&deep_nested);
    assert!(res_json.contains("Maximum recursion depth"),
        "Parser must gracefully report recursion limit exceeded: {}", res_json);
    assert!(res_json.contains("\"verified\": false") || res_json.contains("\"verified\":false"));
}

#[test]
fn test_module_path_traversal_rejection() {
    let loader = ModuleLoader::new();
    let res = loader.load_source("../../../../../etc/passwd");
    assert!(res.is_err(), "Module loader must reject directory traversal sequences");
}

#[test]
fn test_sound_theorems_still_verify() {
    let valid_code = r#"
theorem identity_law : P -> P
proof
    suppose h : P
    therefore P from h
end
"#;
    let res_json = process_proof_request(valid_code);
    assert!(res_json.contains("\"verified\": true") || res_json.contains("\"verified\":true"),
        "Kernel must verify valid identity theorem: {}", res_json);
}

#[test]
fn test_adversarial_rejection_of_transitivity_with_arbitrary_conclusion() {
    let malicious_code = r#"
theorem fake_trans : A = B -> (B = C -> False)
proof
    suppose h1 : A = B
    suppose h2 : B = C
    derive h3 : False from h1, h2 using EqTrans
    therefore False from h3
end
"#;
    let res_json = process_proof_request(malicious_code);
    assert!(res_json.contains("\"verified\": false") || res_json.contains("\"verified\":false"),
        "Kernel must reject EqTrans deriving False: {}", res_json);
}

#[test]
fn test_adversarial_rejection_of_symmetry_with_arbitrary_conclusion() {
    let malicious_code = r#"
theorem fake_symm : A = B -> False
proof
    suppose h1 : A = B
    derive h2 : False from h1 using EqSymm
    therefore False from h2
end
"#;
    let res_json = process_proof_request(malicious_code);
    assert!(res_json.contains("\"verified\": false") || res_json.contains("\"verified\":false"),
        "Kernel must reject EqSymm deriving False: {}", res_json);
}

#[test]
fn test_adversarial_rejection_of_cyclic_quad_proving_numeric_equality() {
    let malicious_code = r#"
theorem fake_cyclic : cyclic(ABCD) -> 0 = 1
proof
    suppose h1 : cyclic(ABCD)
    derive h2 : 0 = 1 from h1 using CyclicQuad
    therefore 0 = 1 from h2
end
"#;
    let res_json = process_proof_request(malicious_code);
    assert!(res_json.contains("\"verified\": false") || res_json.contains("\"verified\":false"),
        "Kernel must reject CyclicQuad deriving 0 = 1: {}", res_json);
}

#[test]
fn test_lsp_content_length_oversized_rejection() {
    let payload = "Content-Length: 50000000\r\n\r\n";
    let mut cursor = std::io::Cursor::new(payload.as_bytes());
    let res = proof::lsp::protocol::read_message(&mut cursor);
    assert!(res.is_err(), "LSP protocol reader must reject frames larger than 16MB");
}

#[test]
fn test_json_str_escapes_all_control_characters() {
    let input = "line1\nline2\rline3\ttab\0null\x1funit";
    let json = proof::api::json_str(input);
    assert!(json.starts_with('"') && json.ends_with('"'));
    assert!(!json.contains('\n'));
    assert!(!json.contains('\r'));
    assert!(!json.contains('\t'));
    assert!(!json.contains('\0'));
    assert!(json.contains("\\u0000"));
    assert!(json.contains("\\u001f"));
}

#[test]
fn test_extract_json_string_field_resists_substring_collision() {
    let raw_json = r#"{"description": "Here is a sample payload with \"code\": \"fake_injected_code\"", "code": "real theorem body"}"#;
    let extracted = proof::api::extract_json_string_field(raw_json, "code");
    assert_eq!(extracted.as_deref(), Some("real theorem body"),
        "JSON field extractor must not match substring keys inside earlier string values");
}

#[test]
fn test_package_create_sanitizes_toml_injection() {
    let tmp_dir = std::env::temp_dir().join(format!("proofer_test_{}", std::process::id()));
    let malicious_name = "test_proj\"\nevil = true\n[injected_section]\nhacked = true\n#";
    let res = proof::package::PackageManager::create_project(&tmp_dir, malicious_name);
    assert!(res.is_ok());

    let manifest_path = tmp_dir.join("proof.toml");
    let content = std::fs::read_to_string(&manifest_path).expect("Read proof.toml");
    assert!(!content.contains("[injected_section]"), "Must not allow arbitrary TOML section injection");
    assert!(!content.contains("hacked = true"), "Must not allow arbitrary TOML key injection");

    let _ = std::fs::remove_dir_all(&tmp_dir);
}


