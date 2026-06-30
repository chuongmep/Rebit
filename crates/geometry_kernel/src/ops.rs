//! Geometry operations — Phase B placeholder.
//!
//! This module will house boolean operations (union, subtract, intersect),
//! intersection curves between surfaces, and solid-to-solid conversions.
//! In Phase A it only exports scaffolding types to unblock downstream modules
//! that need to reference operation result types.

use crate::shapes::{Line, Plane};
use core_math::{Point3D, Tolerance, Vector3D};

// ---------------------------------------------------------------------------
// Intersection result types
// ---------------------------------------------------------------------------

/// Result of intersecting two geometric entities.
///
/// This enum is intentionally minimal in Phase A — it will be expanded as
/// booleans and surface-curve intersections are implemented in Phase B.
#[derive(Debug, Clone)]
pub enum Intersection {
    /// No intersection.
    None,
    /// Intersection at a single point.
    Point(Point3D),
    /// Intersection along a line (coincident planes, etc.).
    Line(Line),
}

// ---------------------------------------------------------------------------
// Phase A utilities — moved here to keep topology clean
// ---------------------------------------------------------------------------

/// Intersect an infinite line with an infinite plane.
///
/// Returns [`Intersection::None`] when the line is parallel to the plane
/// (including when it lies within the plane — use
/// [`Plane::contains_point`] to check containment separately).
pub fn intersect_line_plane(line: &Line, plane: &Plane, tol: &Tolerance) -> Intersection {
    let denom = plane.normal.dot(&line.direction);
    if tol.nearly_equal(denom.value, 0.0) {
        // Parallel — either disjoint or contained.
        return Intersection::None;
    }
    let numerator = -(plane
        .normal
        .dot(&Vector3D::new(line.origin.x, line.origin.y, line.origin.z))
        .value
        + plane.d.value);
    let t = numerator / denom.value;
    let point = Point3D::new(
        line.origin.x.value + t * line.direction.x.value,
        line.origin.y.value + t * line.direction.y.value,
        line.origin.z.value + t * line.direction.z.value,
    );
    Intersection::Point(point)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::Line;

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    #[test]
    fn line_plane_intersection() {
        // Plane z = 0
        let plane = Plane::from_normal_and_point(Vector3D::Z, &Point3D::ORIGIN);
        // Line from (1,2,5) pointing down
        let line = Line::new(Point3D::new(1.0, 2.0, 5.0), Vector3D::new(0.0, 0.0, -1.0));
        match intersect_line_plane(&line, &plane, &tol()) {
            Intersection::Point(p) => {
                assert!(p.nearly_equal(&Point3D::new(1.0, 2.0, 0.0), &tol()));
            }
            other => panic!("Expected Point, got {other:?}"),
        }
    }

    #[test]
    fn line_plane_parallel() {
        let plane = Plane::from_normal_and_point(Vector3D::Z, &Point3D::ORIGIN);
        let line = Line::new(Point3D::new(1.0, 2.0, 5.0), Vector3D::X);
        match intersect_line_plane(&line, &plane, &tol()) {
            Intersection::None => {} // expected
            other => panic!("Expected None, got {other:?}"),
        }
    }
}
