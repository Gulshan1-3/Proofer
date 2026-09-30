/// Stage 7 — Geometry Solver Tests
///
/// Verifies:
/// 1. Automatic proof discovery (forward chaining, pattern matching)
/// 2. Crucial architectural discipline:
///    The solver NEVER outputs a raw boolean as proof!
///    The solver MUST output a certificate (`ProofObject`).
/// 3. The generated certificate is independently verified by the trusted `Checker`.
/// 4. When a goal is unprovable, the solver returns `None`.

use proof::kernel::{KProp, KTerm, check_proof};
use proof::solver::GeometrySolver;

#[test]
fn test_solver_auto_discovers_isosceles_base_angles() {
    let mut solver = GeometrySolver::new();

    // Given AB = AC
    let given = KProp::Eq(KTerm::Const("AB".into()), KTerm::Const("AC".into()));
    solver.add_given(given);

    // Goal: angle_BAC = angle_CAB (base angles for vertex A)
    let goal = KProp::Eq(
        KTerm::Const("angle_BAC".into()),
        KTerm::Const("angle_CAB".into()),
    );

    // Solve automatically
    let cert_opt = solver.solve(&goal);
    assert!(cert_opt.is_some(), "Solver should discover proof for isosceles base angles");

    let cert = cert_opt.unwrap();
    let ctx = solver.context();

    // INDEPENDENT KERNEL VERIFICATION
    let res = check_proof(&cert, &ctx);
    assert_eq!(res, Ok(goal), "Kernel must independently verify the solver's certificate");
}

#[test]
fn test_solver_chained_discovery_sss_to_angle() {
    let mut solver = GeometrySolver::new();

    // Given: AB = AC, BM = CM, AM = AM
    solver.add_given(KProp::Eq(KTerm::Const("AB".into()), KTerm::Const("AC".into())));
    solver.add_given(KProp::Eq(KTerm::Const("BM".into()), KTerm::Const("CM".into())));
    solver.add_given(KProp::Eq(KTerm::Const("AM".into()), KTerm::Const("AM".into())));

    // Target goal: angle_ABM = angle_ACM
    // Solver must chain:
    // 1. SSS -> congruent(triangle_ABM, triangle_ACM)
    // 2. CongruentTrianglesAngles -> angle_ABM = angle_ACM
    let goal = KProp::Eq(
        KTerm::Const("angle_ABM".into()),
        KTerm::Const("angle_ACM".into()),
    );

    let cert_opt = solver.solve(&goal);
    assert!(cert_opt.is_some(), "Solver should discover chained SSS -> angle equality");

    let cert = cert_opt.unwrap();
    let ctx = solver.context();

    // INDEPENDENT KERNEL VERIFICATION
    let res = check_proof(&cert, &ctx);
    assert_eq!(res, Ok(goal), "Kernel must verify multi-step solver certificate");
}

#[test]
fn test_solver_fails_on_unprovable_goal() {
    let mut solver = GeometrySolver::new();

    // Given: AB = AC
    solver.add_given(KProp::Eq(KTerm::Const("AB".into()), KTerm::Const("AC".into())));

    // Unrelated goal: XY = ZW
    let goal = KProp::Eq(KTerm::Const("XY".into()), KTerm::Const("ZW".into()));

    let cert_opt = solver.solve(&goal);
    assert!(cert_opt.is_none(), "Solver must return None for unprovable goal");
}
