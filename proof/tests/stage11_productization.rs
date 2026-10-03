use proof::module::{ModuleLoader, ModuleError};
use proof::layout::SceneLayout;
use proof::library::{STANDARD_LIBRARY_SOURCE, standard_theorem_names};
use proof::incremental::IncrementalEngine;
use proof::id::PointId;

#[test]
fn test_module_system_topological_resolution() {
    let mut loader = ModuleLoader::new();
    loader.add_virtual_file(
        "geometry.core",
        r#"
theorem core_identity:
    P -> P
proof
    suppose h : P
    therefore P from h
end
"#,
    );

    loader.add_virtual_file(
        "geometry.triangles",
        r#"
import geometry.core;

theorem triangle_prop:
    Q -> Q
proof
    suppose h : Q
    therefore Q from h
end
"#,
    );

    let resolved = loader.resolve_project("geometry.triangles").expect("Resolution failed");
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[0].id.0, "geometry.core");
    assert_eq!(resolved[1].id.0, "geometry.triangles");
}

#[test]
fn test_module_system_detects_cyclic_imports() {
    let mut loader = ModuleLoader::new();
    loader.add_virtual_file(
        "mod_a",
        r#"
import mod_b;
theorem a_thm: P -> P proof suppose h : P therefore P from h end
"#,
    );
    loader.add_virtual_file(
        "mod_b",
        r#"
import mod_a;
theorem b_thm: P -> P proof suppose h : P therefore P from h end
"#,
    );

    let res = loader.resolve_project("mod_a");
    assert!(res.is_err(), "Expected cyclic dependency error");
    match res {
        Err(ModuleError::CyclicDependency(_)) => {}
        other => panic!("Expected CyclicDependency, got {:?}", other),
    }
}

#[test]
fn test_persistent_layout_serialization_roundtrip() {
    let mut layout = SceneLayout::new("isosceles_test");
    layout.zoom = 1.25;
    layout.pan_x = 45.0;
    layout.pan_y = -30.0;
    layout.add_point(PointId(1), "A", 300.0, 120.0, true);
    layout.add_point(PointId(2), "B", 120.0, 380.0, false);
    layout.add_point(PointId(3), "C", 480.0, 380.0, false);

    let json_str = layout.to_json();
    assert!(json_str.contains("\"name\": \"isosceles_test\""));
    assert!(json_str.contains("\"zoom\": 1.25"));

    let parsed = SceneLayout::from_json(&json_str).expect("Failed to parse JSON");
    assert_eq!(parsed.name, "isosceles_test");
    assert_eq!(parsed.points.len(), 3);
    assert_eq!(parsed.points[0].name, "A");
    assert_eq!(parsed.points[0].x, 300.0);
    assert_eq!(parsed.points[0].y, 120.0);
    assert!(parsed.points[0].fixed);
}

#[test]
fn test_standard_geometry_theorem_library_verifies() {
    let mut engine = IncrementalEngine::new();
    let res = engine.compile_source(STANDARD_LIBRARY_SOURCE, None).expect("Standard library compilation failed");
    assert!(res.verified, "Standard geometry library theorems should all verify");

    let expected_names = standard_theorem_names();
    for expected in expected_names {
        let thm = res.theorems.iter().find(|t| t.name == *expected);
        assert!(thm.is_some(), "Expected standard theorem '{}' to be present", expected);
        assert!(thm.unwrap().is_verified(), "Theorem '{}' should be verified", expected);
    }
}
