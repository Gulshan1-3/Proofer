//! Standard Geometry Theorem Library (Stage 11).
//!
//! Provides the foundational Euclidean geometry theorem library packaged with Proofer:
//! - Vertical Angles Theorem
//! - Triangle Interior Angle Sum (180 degrees)
//! - Isosceles Base Angles Theorem
//! - Thales' Theorem (Inscribed right angle in semicircle)
//! - Triangle Midpoint Theorem

/// Source code of the Proofer Standard Geometry Library.
pub const STANDARD_LIBRARY_SOURCE: &str = r#"
theorem vertical_angles:
    intersect(line_AB, line_CD) -> angle_AEC = angle_BED
proof
    suppose h1 : intersect(line_AB, line_CD)
    derive h2 : angle_AEC = angle_BED from h1 using VerticalAngles
    therefore angle_AEC = angle_BED from h2
end

theorem triangle_angle_sum:
    triangle(ABC) -> angle_sum = 180
proof
    suppose h1 : triangle(ABC)
    derive h2 : angle_sum = 180 from h1 using AngleSum180
    therefore angle_sum = 180 from h2
end

theorem isosceles_base_angles:
    AB = AC -> angle_ABC = angle_ACB
proof
    suppose h1 : AB = AC
    derive h2 : angle_ABC = angle_ACB from h1 using IsoscelesBaseAngles
    therefore angle_ABC = angle_ACB from h2
end

theorem triangle_midpoint_theorem:
    midpoint(M, AB) and midpoint(N, AC) -> parallel(line_MN, line_BC)
proof
    suppose h1 : midpoint(M, AB) and midpoint(N, AC)
    derive h2 : parallel(line_MN, line_BC) from h1 using MidpointBisects
    therefore parallel(line_MN, line_BC) from h2
end

theorem thales_theorem:
    diameter(AB) and on_circle(C) -> angle_ACB = 90
proof
    suppose h1 : diameter(AB) and on_circle(C)
    derive h2 : angle_ACB = 90 from h1 using InscribedAngle
    therefore angle_ACB = 90 from h2
end

theorem cyclic_quad_opposite_angles:
    cyclic(ABCD) -> angle_DAB + angle_BCD = 180
proof
    suppose h1 : cyclic(ABCD)
    derive h2 : angle_DAB + angle_BCD = 180 from h1 using CyclicQuad
    therefore angle_DAB + angle_BCD = 180 from h2
end

theorem parallelogram_opposite_sides:
    parallelogram(ABCD) -> AB = CD and BC = DA
proof
    suppose h1 : parallelogram(ABCD)
    derive h2 : AB = CD and BC = DA from h1 using ParallelogramOppSides
    therefore AB = CD and BC = DA from h2
end
"#;

/// Returns the standard list of included geometry theorem names.
pub fn standard_theorem_names() -> &'static [&'static str] {
    &[
        "vertical_angles",
        "triangle_angle_sum",
        "isosceles_base_angles",
        "triangle_midpoint_theorem",
        "thales_theorem",
        "cyclic_quad_opposite_angles",
        "parallelogram_opposite_sides",
    ]
}
