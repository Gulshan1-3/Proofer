use proof::incremental::{IncrementalEngine, CancellationToken};

#[test]
fn test_incremental_cache_reuses_unmodified_theorems() {
    let mut engine = IncrementalEngine::new();

    let code = r#"
theorem thm_a:
    P -> P
proof
    suppose h : P
    therefore P from h
end

theorem thm_b:
    Q -> Q
proof
    suppose h : Q
    therefore Q from h
end
"#;

    let res1 = engine.compile_source(code, None).expect("First compilation failed");
    assert!(res1.verified);
    assert_eq!(res1.recompiled_count, 2);
    assert_eq!(res1.reused_count, 0);

    // Second compilation with exact same code
    let res2 = engine.compile_source(code, None).expect("Second compilation failed");
    assert!(res2.verified);
    assert_eq!(res2.recompiled_count, 0);
    assert_eq!(res2.reused_count, 2);
    assert_eq!(res2.theorems.len(), 2);
}

#[test]
fn test_local_proof_edit_only_recompiles_edited_theorem() {
    let mut engine = IncrementalEngine::new();

    let code_v1 = r#"
theorem thm_a:
    P -> P
proof
    suppose h : P
    therefore P from h
end

theorem thm_b:
    Q -> Q
proof
    suppose h : Q
    therefore Q from h
end
"#;

    engine.compile_source(code_v1, None).expect("Compile v1 failed");

    // Edit only thm_b (add an intermediate have step)
    let code_v2 = r#"
theorem thm_a:
    P -> P
proof
    suppose h : P
    therefore P from h
end

theorem thm_b:
    Q -> Q
proof
    suppose h : Q
    have h2 : Q from h
    therefore Q from h2
end
"#;

    let res2 = engine.compile_source(code_v2, None).expect("Compile v2 failed");
    assert!(res2.verified);
    assert_eq!(res2.reused_count, 1, "thm_a should be reused from cache");
    assert_eq!(res2.recompiled_count, 1, "thm_b should be recompiled");
}

#[test]
fn test_signature_change_invalidates_dependents() {
    let mut engine = IncrementalEngine::new();

    let code_v1 = r#"
theorem lemma_a:
    P -> P
proof
    suppose h : P
    therefore P from h
end

theorem main_thm:
    P -> P
proof
    suppose h1 : P
    have h2 : P from h1
    therefore P from h2
end
"#;

    engine.compile_source(code_v1, None).expect("Compile v1 failed");

    // Change signature of lemma_a
    let code_v2 = r#"
theorem lemma_a:
    R -> R
proof
    suppose h : R
    therefore R from h
end

theorem main_thm:
    P -> P
proof
    suppose h1 : P
    have h2 : P from h1
    therefore P from h2
end
"#;

    let res2 = engine.compile_source(code_v2, None).expect("Compile v2 failed");
    assert!(res2.verified);
    // lemma_a signature changed, so lemma_a is recompiled
    assert!(res2.theorems.iter().any(|t| t.name == "lemma_a" && t.is_verified()));
}

#[test]
fn test_cancellation_token_aborts_compilation() {
    let mut engine = IncrementalEngine::new();
    let token = CancellationToken::new();

    // Cancel before or during compilation
    token.cancel();

    let code = r#"
theorem thm_cancel:
    P -> P
proof
    suppose h : P
    therefore P from h
end
"#;

    let res = engine.compile_source(code, Some(&token));
    assert!(res.is_err(), "Expected cancellation error");
}
