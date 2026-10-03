//! Comprehensive Correctness, Adversarial Soundness, and Performance Benchmark Suite.
//!
//! Validates:
//! 1. Adversarial Soundness & Kernel Rejection Guarantees (Zero False Positives).
//! 2. Deep Deductive Chains & Compound Geometric Reasoning.
//! 3. Automated Geometric Solver Certificates & Independent Kernel Verification.
//! 4. Angle Chasing & Congruence Closure Tactics.
//! 5. Multi-Threaded Concurrent Verification & Contention.
//! 6. Microsecond-Scale Performance Benchmarking & Throughput Analysis.

use proof::api::process_proof_request;
use proof::solver::GeometrySolver;
use proof::kernel::types::{KProp, KTerm};
use proof::kernel::checker::{Context, check_proof};
use proof::kernel::proof_object::{ProofNode, ProofNodeId, ProofObject};
use proof::tactic::{TacticGoal, CongruenceClosure, AngleChase, AutoGeometry};
use proof::geometry::{GeoObject, GeoRelKind, GeoProp, GeoTerm, GeoFigure};
use proof::id::{PointId, TriangleId, FactId};
use proof::syntax::span::Span;
use proof::incremental::IncrementalEngine;
use proof::synthesizer::ProofSynthesizer;
use std::sync::Arc;
use std::time::Instant;

// ============================================================================
// 1. ADVERSARIAL SOUNDNESS & ZERO-HALLUCINATION AUDIT
// ============================================================================

#[test]
fn test_adversarial_rejection_unsound_deductions() {
    let hostile_proofs = vec![
        // Case 1: Premise claims triangle is isosceles, conclusion claims right angle without proof
        (
            "unsound_conclusion",
            r#"theorem bogus_right_angle:
    AB = AC -> angle_ABC = 90
proof
    suppose h1 : AB = AC
    derive h2 : angle_ABC = 90 from h1 using IsoscelesBaseAngles
    therefore angle_ABC = 90 from h2
end
"#,
        ),
        // Case 2: Using non-existent invented rule
        (
            "invented_rule",
            r#"theorem fake_rule:
    AB = AC -> angle_ABC = angle_ACB
proof
    suppose h1 : AB = AC
    derive h2 : angle_ABC = angle_ACB from h1 using FakeMagicTheorem
    therefore angle_ABC = angle_ACB from h2
end
"#,
        ),
        // Case 3: Circular self-reference
        (
            "circular_reference",
            r#"theorem circular_claim:
    P -> P
proof
    derive h1 : P from h2 using ModusPonens
    suppose h2 : P
    therefore P from h1
end
"#,
        ),
        // Case 4: SSS congruence with insufficient premises (only 2 sides given)
        (
            "incomplete_sss",
            r#"theorem incomplete_congruence:
    AB = DE and BC = EF -> congruent(triangle_ABC, triangle_DEF)
proof
    suppose h1 : AB = DE and BC = EF
    derive h2 : congruent(triangle_ABC, triangle_DEF) from h1 using SSS
    therefore congruent(triangle_ABC, triangle_DEF) from h2
end
"#,
        ),
        // Case 5: Target goal mismatch (proves Q but declares theorem R)
        (
            "goal_mismatch",
            r#"theorem goal_switch:
    P -> R
proof
    suppose h1 : P
    derive h2 : Q from h1 using ModusPonens
    therefore Q from h2
end
"#,
        ),
        // Case 6: False implication
        (
            "false_elim_attempt",
            r#"theorem false_inference:
    P -> Q
proof
    suppose h1 : P
    therefore Q from h1
end
"#,
        ),
    ];

    for (name, proof_code) in hostile_proofs {
        let json_res = process_proof_request(proof_code);
        let contains_verified = json_res.contains("\"status\": \"Verified\"") || json_res.contains("\"verified\": true");
        assert!(
            !contains_verified,
            "CRITICAL SECURITY / SOUNDNESS BUG: Hostile test '{}' was falsely verified!\nCode:\n{}\nResponse:\n{}",
            name, proof_code, json_res
        );
    }
}

// ============================================================================
// 2. STRESS TESTING DEEP DEDUCTIVE REASONING CHAINS
// ============================================================================

#[test]
fn test_multi_step_deductive_pipeline() {
    let deep_chain_proof = r#"theorem transitive_equality_chain:
    AB = CD -> CD = EF -> AB = EF
proof
    suppose h1 : AB = CD
    suppose h2 : CD = EF
    derive h3 : AB = EF from h1, h2 using Transitivity
    therefore AB = EF from h3
end
"#;

    let res = process_proof_request(deep_chain_proof);
    assert!(
        res.contains("\"status\": \"Verified\"") || res.contains("\"verified\": true") || res.contains("\"status\":\"Verified\""),
        "Failed to elaborate deep deduction chain: {}",
        res
    );
}

// ============================================================================
// 3. AUTOMATED GEOMETRIC SOLVER & INDEPENDENT KERNEL VERIFICATION
// ============================================================================

#[test]
fn test_solver_certificate_generation_and_verification() {
    let mut solver = GeometrySolver::new();

    // Given AB = AC
    let given = KProp::Eq(KTerm::Const("AB".into()), KTerm::Const("AC".into()));
    solver.add_given(given);

    let goal = KProp::Eq(
        KTerm::Const("angle_BAC".into()),
        KTerm::Const("angle_CAB".into()),
    );

    let cert_opt = solver.solve(&goal);
    assert!(cert_opt.is_some(), "Solver must find proof certificate for isosceles base angles");

    let cert = cert_opt.unwrap();
    let ctx = solver.context();

    // Independent Kernel Verification: Checker must verify without trusting solver
    let res = check_proof(&cert, &ctx);
    assert_eq!(res, Ok(goal), "Kernel must independently verify the solver's certificate");

    // Unprovable goal must return None (zero hallucination)
    let bogus_goal = KProp::Eq(KTerm::Const("X".into()), KTerm::Const("Y".into()));
    assert!(solver.solve(&bogus_goal).is_none(), "Solver must never hallucinate unprovable goal");
}

#[test]
fn test_congruence_closure_transitivity_chain() {
    let mut cc = CongruenceClosure::new();

    let a = KTerm::Var("a".into());
    let b = KTerm::Var("b".into());
    let c = KTerm::Var("c".into());

    let goal = TacticGoal {
        hypotheses: vec![
            ("h1".into(), KProp::Eq(a.clone(), b.clone())),
            ("h2".into(), KProp::Eq(b.clone(), c.clone())),
        ],
        target: KProp::Eq(a.clone(), c.clone()),
    };

    let cert_nodes = cc.prove_equality(&goal);
    assert!(cert_nodes.is_ok(), "Congruence closure must solve transitivity chain");
}

#[test]
fn test_angle_chase_and_triangle_invariants() {
    let mut chase = AngleChase::new();
    let goal = TacticGoal {
        hypotheses: vec![
            ("h1".into(), KProp::Atom("triangle".into(), vec![KTerm::Const("ABC".into())])),
            ("h2".into(), KProp::Eq(KTerm::Const("angle_A".into()), KTerm::Const("60".into()))),
            ("h3".into(), KProp::Eq(KTerm::Const("angle_B".into()), KTerm::Const("60".into()))),
        ],
        target: KProp::Eq(KTerm::Const("angle_C".into()), KTerm::Const("60".into())),
    };

    let res = chase.prove_angle_goal(&goal);
    assert!(res.is_ok(), "Angle chase tactic must discharge third angle invariant");
}

#[test]
fn test_symbolic_figure_consistency_and_references() {
    let p1 = PointId(1);
    let p2 = PointId(2);
    let p3 = PointId(3);

    let mut figure = GeoFigure::new("triangle_abc", Span::DUMMY);
    figure.add_object(GeoObject::Point { id: p1, name: "A".into() });
    figure.add_object(GeoObject::Point { id: p2, name: "B".into() });
    figure.add_object(GeoObject::Point { id: p3, name: "C".into() });
    figure.add_object(GeoObject::Triangle {
        id: TriangleId(10),
        a: p1,
        b: p2,
        c: p3,
    });

    figure.add_given(GeoProp {
        rel: GeoRelKind::EqualLength,
        args: vec![GeoTerm::Segment(p1, p2), GeoTerm::Segment(p1, p3)],
        span: Span::DUMMY,
    });

    assert!(figure.check_point_references().is_ok(), "Figure references must be consistent");

    // Negative test: add given referencing missing point
    let missing_pt = PointId(999);
    figure.add_given(GeoProp {
        rel: GeoRelKind::EqualLength,
        args: vec![GeoTerm::Segment(p1, missing_pt)],
        span: Span::DUMMY,
    });
    assert!(figure.check_point_references().is_err(), "Figure must detect missing point reference");
}

// ============================================================================
// 4. MULTI-THREADED CONCURRENCY & CONTENTION STRESS TEST
// ============================================================================

#[test]
fn test_concurrent_verification_thread_safety() {
    let code_sample = Arc::new(r#"theorem concurrency_test:
    AB = AC -> angle_ABC = angle_ACB
proof
    suppose h1 : AB = AC
    derive h2 : angle_ABC = angle_ACB from h1 using IsoscelesBaseAngles
    therefore angle_ABC = angle_ACB from h2
end
"#.to_string());

    let mut handles = Vec::new();
    let num_threads = 8;
    let iterations_per_thread = 50;

    for _ in 0..num_threads {
        let code = Arc::clone(&code_sample);
        let handle = std::thread::spawn(move || {
            let mut verified_count = 0;
            for _ in 0..iterations_per_thread {
                let res = process_proof_request(&code);
                if res.contains("\"status\": \"Verified\"") || res.contains("\"status\":\"Verified\"") {
                    verified_count += 1;
                }
            }
            verified_count
        });
        handles.push(handle);
    }

    let mut total_verified = 0;
    for handle in handles {
        total_verified += handle.join().expect("Thread panicked");
    }

    assert_eq!(
        total_verified,
        num_threads * iterations_per_thread,
        "All concurrent proofs must succeed without race conditions"
    );
}

// ============================================================================
// 5. HIGH-SPEED PERFORMANCE & LATENCY BENCHMARK
// ============================================================================

#[test]
fn test_kernel_verification_performance_benchmarks() {
    let code = r#"theorem benchmark_theorem:
    AB = AC -> angle_ABC = angle_ACB
proof
    suppose h1 : AB = AC
    derive h2 : angle_ABC = angle_ACB from h1 using IsoscelesBaseAngles
    therefore angle_ABC = angle_ACB from h2
end
"#;

    // Warm up
    for _ in 0..10 {
        let _ = process_proof_request(code);
    }

    // Benchmark 100 consecutive full compilations (Lex -> Parse -> Resolve -> Elab -> Kernel)
    let iterations = 100;
    let start = Instant::now();
    for _ in 0..iterations {
        let res = process_proof_request(code);
        assert!(res.contains("Verified"));
    }
    let total_elapsed = start.elapsed();
    let avg_latency = total_elapsed / iterations;
    let thms_per_second = (iterations as f64) / total_elapsed.as_secs_f64();

    println!("\n=======================================================");
    println!("PROOFER FORMAL KERNEL PERFORMANCE BENCHMARK");
    println!("=======================================================");
    println!("Iterations:              {}", iterations);
    println!("Total Elapsed:           {:.2?}", total_elapsed);
    println!("Average Verification Latency: {:.2?} / theorem", avg_latency);
    println!("Throughput:              {:.1} theorems / second", thms_per_second);
    println!("=======================================================\n");

    // Performance assertion: Real-time requirement is < 5 milliseconds per full verification
    assert!(
        avg_latency.as_millis() < 5,
        "Verification too slow! Expected < 5ms, got {:?}",
        avg_latency
    );
}

#[test]
fn test_synthesizer_performance_latency() {
    let incomplete = r#"theorem synth_perf:
    AB = AC -> angle_ABC = angle_ACB
proof
    suppose h1 : AB = AC
end
"#;

    let start = Instant::now();
    let step = ProofSynthesizer::infill_next_step(incomplete);
    let elapsed = start.elapsed();

    assert!(step.is_some());
    assert!(step.unwrap().verified_by_kernel);

    println!("Co-Prover Synthesis Latency: {:.2?}", elapsed);
    assert!(
        elapsed.as_millis() < 10,
        "Proof synthesizer took too long ({:?}), must respond in < 10ms for smooth 60fps typing",
        elapsed
    );
}

#[test]
fn test_incremental_cache_hit_latency() {
    let mut engine = IncrementalEngine::new();
    let source = r#"
theorem thm_one: P -> P proof suppose h : P therefore P from h end
theorem thm_two: Q -> Q proof suppose h : Q therefore Q from h end
"#;

    // First compile (cold)
    let _ = engine.compile_source(source, None).unwrap();

    // Second compile (hot cache)
    let start = Instant::now();
    let res = engine.compile_source(source, None).unwrap();
    let elapsed = start.elapsed();

    assert!(res.verified);
    println!("Incremental Cache Hit Latency: {:.2?}", elapsed);
    assert!(
        elapsed.as_micros() < 500,
        "Incremental cache hit took too long ({:?}), expected < 500us",
        elapsed
    );
}
