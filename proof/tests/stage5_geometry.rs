/// Stage 5 — Geometry IR Tests
///
/// Verifies:
/// 1. Symbolic geometry objects, relations, terms, and constraints
/// 2. Parsing and resolving `figure ... end` blocks into `GeoFigure`
/// 3. Symbolic consistency checking without rendering
/// 4. Construction modeling (e.g., midpoint)

use proof::geometry::{GeoObject, GeoRelKind, GeoProp, GeoTerm, GeoConstruction, GeoFigure};
use proof::id::{PointId, LineId, TriangleId, ConstructionId};
use proof::syntax::span::Span;
use proof::parser::Parser;
use proof::hir::resolve::Resolver;

#[test]
fn test_geometry_objects_and_relations_creation() {
    let p1 = PointId(1);
    let p2 = PointId(2);
    let p3 = PointId(3);

    let tri = GeoObject::Triangle {
        id: TriangleId(10),
        a: p1,
        b: p2,
        c: p3,
    };

    let mut figure = GeoFigure::new("triangle_abc", Span::DUMMY);
    figure.add_object(GeoObject::Point { id: p1, name: "A".into() });
    figure.add_object(GeoObject::Point { id: p2, name: "B".into() });
    figure.add_object(GeoObject::Point { id: p3, name: "C".into() });
    figure.add_object(tri);

    // Given AB = AC
    figure.add_given(GeoProp {
        rel: GeoRelKind::EqualLength,
        args: vec![GeoTerm::Segment(p1, p2), GeoTerm::Segment(p1, p3)],
        span: Span::DUMMY,
    });

    // Check consistency: all points exist
    assert!(figure.check_point_references().is_ok());
}

#[test]
fn test_geometry_consistency_catches_unknown_points() {
    let p1 = PointId(1);
    let p2 = PointId(2);
    let unknown_p = PointId(999);

    let mut figure = GeoFigure::new("broken_figure", Span::DUMMY);
    figure.add_object(GeoObject::Point { id: p1, name: "A".into() });
    figure.add_object(GeoObject::Point { id: p2, name: "B".into() });

    // Given AB = AC where C does not exist
    figure.add_given(GeoProp {
        rel: GeoRelKind::EqualLength,
        args: vec![GeoTerm::Segment(p1, p2), GeoTerm::Segment(p1, unknown_p)],
        span: Span::DUMMY,
    });

    let check = figure.check_point_references();
    assert!(check.is_err(), "Should detect unknown point 999");
    let errors = check.unwrap_err();
    assert!(errors[0].contains("pt#999"));
}

#[test]
fn test_parse_and_resolve_figure_block() {
    let src = r#"
    figure triangle_example
        triangle ABC
        given AB = AC
    end
    "#;

    let mut parser = Parser::new(src);
    let file_ast = parser.parse_file();
    assert_eq!(parser.diagnostics().len(), 0);

    let mut resolver = Resolver::new();
    let pkg = resolver.resolve_file(&file_ast);
    assert_eq!(resolver.diagnostics().len(), 0);

    assert_eq!(pkg.figures.len(), 1);
    let fig = &pkg.figures[0];
    assert_eq!(fig.name, "triangle_example");

    // Triangle ABC declares points A, B, C and 1 Triangle
    assert_eq!(fig.objects.len(), 4); // 3 points + 1 triangle
    assert_eq!(fig.givens.len(), 1);

    // Verify given relation is EqualLength between AB and AC
    let given = &fig.givens[0];
    assert_eq!(given.rel, GeoRelKind::EqualLength);
    assert_eq!(given.args.len(), 2);

    // Consistency check passes
    assert!(fig.check_point_references().is_ok());
}

#[test]
fn test_geometry_constructions_representation() {
    let mut figure = GeoFigure::new("midpoint_construction", Span::DUMMY);
    let b = PointId(1);
    let c = PointId(2);
    let m = PointId(3);

    figure.add_object(GeoObject::Point { id: b, name: "B".into() });
    figure.add_object(GeoObject::Point { id: c, name: "C".into() });
    figure.add_object(GeoObject::Point { id: m, name: "M".into() });

    // construct M as midpoint of BC
    figure.add_construction(GeoConstruction {
        id: ConstructionId(1),
        result_name: "M".into(),
        as_description: "midpoint".into(),
        inputs: vec![GeoTerm::Segment(b, c)],
        span: Span::DUMMY,
    });

    assert_eq!(figure.constructions.len(), 1);
    let cst = &figure.constructions[0];
    assert_eq!(cst.result_name, "M");
    assert_eq!(cst.as_description, "midpoint");
    assert_eq!(cst.inputs[0], GeoTerm::Segment(b, c));
}
