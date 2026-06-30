//! Constraint types — the relations enforced between variables.
//!
//! Each constraint type expresses a mathematical relationship between two
//! or more variables.  The solver evaluates the *error* (residual) of each
//! constraint and pushes deltas to bring the error toward zero.

use core_math::{Scalar, Tolerance};
use geometry_kernel::{Point3D, Vector3D};

// ---------------------------------------------------------------------------
// ConstraintKind — the registry of solved relations
// ---------------------------------------------------------------------------

/// A constraint relation between variables or points.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintKind {
    // -- dimensional --------------------------------------------------------
    /// Euclidean distance `d` between two points.
    Distance {
        /// First point.
        a: Point3D,
        /// Second point.
        b: Point3D,
        /// Target distance value.
        target: Scalar,
    },

    /// Horizontal distance (delta-X) between two points.
    HorizontalDistance {
        /// First point.
        a: Point3D,
        /// Second point.
        b: Point3D,
        /// Target horizontal offset.
        target: Scalar,
    },

    /// Vertical distance (delta-Y) between two points.
    VerticalDistance {
        /// First point.
        a: Point3D,
        /// Second point.
        b: Point3D,
        /// Target vertical offset.
        target: Scalar,
    },

    // -- geometric ----------------------------------------------------------
    /// Points must share the same Y coordinate (horizontal alignment).
    Horizontal {
        /// First point.
        a: Point3D,
        /// Second point.
        b: Point3D,
    },

    /// Points must share the same X coordinate (vertical alignment).
    Vertical {
        /// First point.
        a: Point3D,
        /// Second point.
        b: Point3D,
    },

    /// Two points must coincide.
    Coincident {
        /// First point.
        a: Point3D,
        /// Second point.
        b: Point3D,
    },

    /// Three points must be collinear.
    Collinear {
        /// First point.
        a: Point3D,
        /// Second point.
        b: Point3D,
        /// Third point.
        c: Point3D,
    },
}

impl ConstraintKind {
    /// Compute the scalar residual (error) for this constraint.
    ///
    /// The residual is zero when the constraint is exactly satisfied.
    pub fn residual(&self) -> Scalar {
        match self {
            Self::Distance { a, b, target } => {
                let actual = a.distance_to(b);
                (actual - *target).value.abs().into()
            }
            Self::HorizontalDistance { a, b, target } => {
                let dx = a.x - b.x;
                (dx.value - target.value).abs().into()
            }
            Self::VerticalDistance { a, b, target } => {
                let dy = a.y - b.y;
                (dy.value - target.value).abs().into()
            }
            Self::Horizontal { a, b } => {
                let dy = a.y - b.y;
                dy.value.abs().into()
            }
            Self::Vertical { a, b } => {
                let dx = a.x - b.x;
                dx.value.abs().into()
            }
            Self::Coincident { a, b } => a.distance_to(b),
            Self::Collinear { a, b, c } => {
                let ab = Vector3D::between(a, b);
                let ac = Vector3D::between(a, c);
                let cross = ab.cross(&ac);
                cross.length()
            }
        }
    }

    /// `true` when the residual is within tolerance.
    #[inline]
    pub fn is_satisfied(&self, tol: &Tolerance) -> bool {
        tol.nearly_equal(self.residual().value, 0.0)
    }

    /// Human-readable label for debugging.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Distance { .. } => "distance",
            Self::HorizontalDistance { .. } => "horizontal_distance",
            Self::VerticalDistance { .. } => "vertical_distance",
            Self::Horizontal { .. } => "horizontal",
            Self::Vertical { .. } => "vertical",
            Self::Coincident { .. } => "coincident",
            Self::Collinear { .. } => "collinear",
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::scalar;

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    #[test]
    fn distance_constraint_satisfied() {
        let c = ConstraintKind::Distance {
            a: Point3D::new(0.0, 0.0, 0.0),
            b: Point3D::new(3.0, 4.0, 0.0),
            target: scalar(5.0),
        };
        assert!(c.is_satisfied(&tol()));
    }

    #[test]
    fn distance_constraint_violated() {
        let c = ConstraintKind::Distance {
            a: Point3D::new(0.0, 0.0, 0.0),
            b: Point3D::new(1.0, 0.0, 0.0),
            target: scalar(10.0),
        };
        assert!(!c.is_satisfied(&tol()));
    }

    #[test]
    fn horizontal_constraint() {
        let c = ConstraintKind::Horizontal {
            a: Point3D::new(1.0, 5.0, 0.0),
            b: Point3D::new(10.0, 5.0, 0.0),
        };
        assert!(c.is_satisfied(&tol()));
    }

    #[test]
    fn vertical_constraint_violated() {
        let c = ConstraintKind::Vertical {
            a: Point3D::new(1.0, 0.0, 0.0),
            b: Point3D::new(2.0, 1.0, 0.0),
        };
        assert!(!c.is_satisfied(&tol()));
    }

    #[test]
    fn coincident_constraint() {
        let c = ConstraintKind::Coincident {
            a: Point3D::new(1.0, 2.0, 3.0),
            b: Point3D::new(1.0, 2.0, 3.0),
        };
        assert!(c.is_satisfied(&tol()));
    }

    #[test]
    fn collinear_points() {
        let c = ConstraintKind::Collinear {
            a: Point3D::new(0.0, 0.0, 0.0),
            b: Point3D::new(1.0, 1.0, 0.0),
            c: Point3D::new(2.0, 2.0, 0.0),
        };
        assert!(c.is_satisfied(&tol()));
    }

    #[test]
    fn collinear_points_violated() {
        let c = ConstraintKind::Collinear {
            a: Point3D::new(0.0, 0.0, 0.0),
            b: Point3D::new(1.0, 0.0, 0.0),
            c: Point3D::new(0.0, 1.0, 0.0),
        };
        assert!(!c.is_satisfied(&tol()));
    }
}
