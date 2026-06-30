//! Intersection types and functions (preserved from Phase A).

use crate::shapes::{Line, Plane};
use core_math::{Point3D, Tolerance, Vector3D};

/// Result of intersecting two geometric entities.
#[derive(Debug, Clone)]
pub enum Intersection {
    /// No intersection.
    None,
    /// Intersection at a single point.
    Point(Point3D),
    /// Intersection along a line (coincident planes, etc.).
    Line(Line),
}

/// Intersect an infinite line with an infinite plane.
pub fn intersect_line_plane(line: &Line, plane: &Plane, tol: &Tolerance) -> Intersection {
    let denom = plane.normal.dot(&line.direction);
    if tol.nearly_equal(denom.value, 0.0) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    #[test]
    fn line_plane_intersection() {
        let plane = Plane::from_normal_and_point(Vector3D::Z, &Point3D::ORIGIN);
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
            Intersection::None => {}
            other => panic!("Expected None, got {other:?}"),
        }
    }
}
