/// Stage 8 — Editor & Geometry Canvas Tests
///
/// Verifies:
/// 1. DocumentModel compiles source text and populates the visual geometry scene.
/// 2. Bidirectional sync: Canvas gesture (construct midpoint) generates a source patch,
///    increments document revision, and triggers incremental recompilation.
/// 3. Visual geometry elements (points, segments, triangles, angles) reflect mathematical state.
/// 4. Selection mapping and interactive state.

use proof::editor::{DocumentModel, CanvasAction, CanvasElement};

#[test]
fn test_document_model_compiles_source_to_canvas_scene() {
    let src = r#"
    figure triangle_sample
        triangle ABC
        given AB = AC
    end
    "#;

    let doc = DocumentModel::new(src.into());
    assert_eq!(doc.revision.0, 1);
    assert!(doc.diagnostics.is_empty(), "Diagnostics should be empty: {:?}", doc.diagnostics);

    // Verify canvas scene contains points, segments, triangle, and angles
    assert_eq!(doc.geometry_scene.point_positions.len(), 3); // A, B, C

    let has_triangle = doc.geometry_scene.elements.iter().any(|e| matches!(e, CanvasElement::Triangle { .. }));
    let has_angles = doc.geometry_scene.elements.iter().any(|e| matches!(e, CanvasElement::Angle { .. }));
    assert!(has_triangle, "Scene must contain visual triangle");
    assert!(has_angles, "Scene must contain visual angles");
}

#[test]
fn test_bidirectional_sync_canvas_gesture_emits_source_patch() {
    let initial_src = r#"
    figure triangle_sample
        triangle ABC
        given AB = AC
    end
    "#;

    let mut doc = DocumentModel::new(initial_src.into());
    assert_eq!(doc.revision.0, 1);

    // Canvas gesture: user constructs midpoint M on segment BC
    doc.apply_canvas_action(CanvasAction::ConstructMidpoint {
        p1_name: "B".into(),
        p2_name: "C".into(),
        mid_name: "M".into(),
    });

    // Revision should increment
    assert_eq!(doc.revision.0, 2);

    // Source code must now contain the synthesized construction patch!
    assert!(
        doc.source.contains("construct M as midpoint of BC"),
        "Source patch was not inserted correctly: {}", doc.source
    );
}

#[test]
fn test_selection_action_updates_editor_state() {
    let src = "figure demo triangle ABC end";
    let mut doc = DocumentModel::new(src.into());

    doc.apply_canvas_action(CanvasAction::SelectElement("Point A".into()));
    assert_eq!(doc.selected_item.as_deref(), Some("Point A"));
}
