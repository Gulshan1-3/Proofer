//! Geometry IR — symbolic representation of Euclidean geometry.
//!
//! Provides first-class semantic types for:
//! - Geometry Objects (Point, Line, Segment, Ray, Angle, Circle, Triangle, Quadrilateral)
//! - Geometric Relations (on, between, collinear, parallel, perpendicular, equal,
//!   congruent, similar, tangent, cyclic, midpoint, bisects)
//! - Geometric Constraints and Constructions
//! - Symbolic Geometry Propositions that integrate with the proof system

use crate::id::{PointId, LineId, SegmentId, CircleId, TriangleId, ConstructionId};
use crate::syntax::span::{Span, Spanned};
use std::fmt;

/// First-class geometric entities with unique identity.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeoObject {
    Point { id: PointId, name: String },
    Line { id: LineId, name: String, points: Vec<PointId> },
    Segment { id: SegmentId, p1: PointId, p2: PointId },
    Ray { origin: PointId, through: PointId },
    Angle { a: PointId, vertex: PointId, b: PointId },
    Circle { id: CircleId, center: PointId, radius_pt: Option<PointId> },
    Triangle { id: TriangleId, a: PointId, b: PointId, c: PointId },
    Quadrilateral { a: PointId, b: PointId, c: PointId, d: PointId },
}

/// Geometric relation kinds supported by Proofer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeoRelKind {
    On,             // Point on Line/Circle
    Between,        // Point B between A and C
    Collinear,      // Points are collinear
    Parallel,       // Line || Line
    Perpendicular,  // Line ⊥ Line or Segment ⊥ Segment
    EqualLength,    // Length(AB) = Length(CD)
    EqualAngle,     // Measure(∠ABC) = Measure(∠DEF)
    Congruent,      // △ABC ≅ △DEF
    Similar,        // △ABC ~ △DEF
    Tangent,        // Line tangent to Circle
    Cyclic,         // Points lie on a common circle
    Midpoint,       // M is midpoint of AB
    Bisects,        // Ray/Line bisects Angle/Segment
}

impl fmt::Display for GeoRelKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeoRelKind::On => write!(f, "on"),
            GeoRelKind::Between => write!(f, "between"),
            GeoRelKind::Collinear => write!(f, "collinear"),
            GeoRelKind::Parallel => write!(f, "parallel"),
            GeoRelKind::Perpendicular => write!(f, "perpendicular"),
            GeoRelKind::EqualLength => write!(f, "equal_length"),
            GeoRelKind::EqualAngle => write!(f, "equal_angle"),
            GeoRelKind::Congruent => write!(f, "congruent"),
            GeoRelKind::Similar => write!(f, "similar"),
            GeoRelKind::Tangent => write!(f, "tangent"),
            GeoRelKind::Cyclic => write!(f, "cyclic"),
            GeoRelKind::Midpoint => write!(f, "midpoint_of"),
            GeoRelKind::Bisects => write!(f, "bisects"),
        }
    }
}

/// Symbolic Geometric Proposition.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeoProp {
    pub rel: GeoRelKind,
    pub args: Vec<GeoTerm>,
    pub span: Span,
}

impl Spanned for GeoProp {
    fn span(&self) -> Span {
        self.span
    }
}

impl fmt::Display for GeoProp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let args_str: Vec<String> = self.args.iter().map(|a| a.to_string()).collect();
        write!(f, "{}({})", self.rel, args_str.join(", "))
    }
}

/// Terms appearing as arguments to geometric relations.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GeoTerm {
    Point(PointId),
    Line(LineId),
    Segment(PointId, PointId),
    Angle(PointId, PointId, PointId),
    Circle(CircleId),
    Triangle(TriangleId),
    Named(String),
}

impl fmt::Display for GeoTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeoTerm::Point(p) => write!(f, "pt#{}", p.0),
            GeoTerm::Line(l) => write!(f, "line#{}", l.0),
            GeoTerm::Segment(a, b) => write!(f, "seg(pt#{}, pt#{})", a.0, b.0),
            GeoTerm::Angle(a, v, b) => write!(f, "angle(pt#{}, pt#{}, pt#{})", a.0, v.0, b.0),
            GeoTerm::Circle(c) => write!(f, "circ#{}", c.0),
            GeoTerm::Triangle(t) => write!(f, "tri#{}", t.0),
            GeoTerm::Named(s) => write!(f, "{}", s),
        }
    }
}

/// A geometric construction step (e.g. "construct M as midpoint of BC").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeoConstruction {
    pub id: ConstructionId,
    pub result_name: String,
    pub as_description: String,
    pub inputs: Vec<GeoTerm>,
    pub span: Span,
}

impl Spanned for GeoConstruction {
    fn span(&self) -> Span {
        self.span
    }
}

/// Symbolic Figure Model (mathematical declaration).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeoFigure {
    pub name: String,
    pub objects: Vec<GeoObject>,
    pub givens: Vec<GeoProp>,
    pub constructions: Vec<GeoConstruction>,
    pub span: Span,
}

impl Spanned for GeoFigure {
    fn span(&self) -> Span {
        self.span
    }
}

impl GeoFigure {
    pub fn new(name: impl Into<String>, span: Span) -> Self {
        Self {
            name: name.into(),
            objects: Vec::new(),
            givens: Vec::new(),
            constructions: Vec::new(),
            span,
        }
    }

    pub fn add_object(&mut self, obj: GeoObject) {
        self.objects.push(obj);
    }

    pub fn add_given(&mut self, prop: GeoProp) {
        self.givens.push(prop);
    }

    pub fn add_construction(&mut self, c: GeoConstruction) {
        self.constructions.push(c);
    }

    /// Check consistency: ensure every referenced PointId exists in the figure.
    pub fn check_point_references(&self) -> Result<(), Vec<String>> {
        let mut known_points = std::collections::HashSet::new();
        for obj in &self.objects {
            match obj {
                GeoObject::Point { id, .. } => { known_points.insert(*id); }
                GeoObject::Triangle { a, b, c, .. } => {
                    known_points.insert(*a);
                    known_points.insert(*b);
                    known_points.insert(*c);
                }
                GeoObject::Segment { p1, p2, .. } => {
                    known_points.insert(*p1);
                    known_points.insert(*p2);
                }
                GeoObject::Angle { a, vertex, b } => {
                    known_points.insert(*a);
                    known_points.insert(*vertex);
                    known_points.insert(*b);
                }
                GeoObject::Circle { center, radius_pt, .. } => {
                    known_points.insert(*center);
                    if let Some(r) = radius_pt {
                        known_points.insert(*r);
                    }
                }
                GeoObject::Line { points, .. } => {
                    for p in points {
                        known_points.insert(*p);
                    }
                }
                GeoObject::Quadrilateral { a, b, c, d } => {
                    known_points.insert(*a);
                    known_points.insert(*b);
                    known_points.insert(*c);
                    known_points.insert(*d);
                }
                GeoObject::Ray { origin, through } => {
                    known_points.insert(*origin);
                    known_points.insert(*through);
                }
            }
        }

        let mut errors = Vec::new();
        for g in &self.givens {
            for arg in &g.args {
                match arg {
                    GeoTerm::Point(p) if !known_points.contains(p) => {
                        errors.push(format!("Unknown point pt#{} in relation {}", p.0, g.rel));
                    }
                    GeoTerm::Segment(p1, p2) => {
                        if !known_points.contains(p1) {
                            errors.push(format!("Unknown point pt#{} in segment for {}", p1.0, g.rel));
                        }
                        if !known_points.contains(p2) {
                            errors.push(format!("Unknown point pt#{} in segment for {}", p2.0, g.rel));
                        }
                    }
                    _ => {}
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
