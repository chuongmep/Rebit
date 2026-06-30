//! Constraint types — the relations enforced between variables.
//!
//! Each constraint type expresses a mathematical relationship between two
//! or more variables.  The solver evaluates the *error* (residual) of each
//! constraint and pushes deltas to bring the error toward zero.
//!
//! # Phase B additions
//! - `Angle` — angle between three points with target value
//! - `Parallel` — two lines must be parallel
//! - `Perpendicular` — two lines must be perpendicular
//! - `EqualLength` — two segments must have equal length
//! - `FixedPoint` — pin a point at a specific location

use core_math::{Scalar, Tolerance};
use geometry_kernel::{Point3D, Vector3D};

/// A constraint relation between variables or points.
#[allow(missing_docs)] // Fields are self-documenting by variant context.
#[derive(Debug, Clone, PartialEq)]
pub enum ConstraintKind {
    // -- dimensional --------------------------------------------------------
    /// Euclidean distance `d` between two points.
    Distance {
        a: Point3D,
        b: Point3D,
        target: Scalar,
    },
    /// Horizontal distance (delta-X) between two points.
    HorizontalDistance {
        a: Point3D,
        b: Point3D,
        target: Scalar,
    },
    /// Vertical distance (delta-Y) between two points.
    VerticalDistance {
        a: Point3D,
        b: Point3D,
        target: Scalar,
    },
    // -- geometric ----------------------------------------------------------
    /// Points must share the same Y coordinate (horizontal alignment).
    Horizontal { a: Point3D, b: Point3D },
    /// Points must share the same X coordinate (vertical alignment).
    Vertical { a: Point3D, b: Point3D },
    /// Two points must coincide.
    Coincident { a: Point3D, b: Point3D },
    /// Three points must be collinear.
    Collinear { a: Point3D, b: Point3D, c: Point3D },
    /// Angle ∠ABC between three points (B is apex) must equal target (radians).
    Angle {
        a: Point3D,
        b: Point3D,
        c: Point3D,
        target: Scalar,
    },
    /// Two segments must be parallel (AB ∥ CD).
    Parallel {
        a1: Point3D,
        a2: Point3D,
        b1: Point3D,
        b2: Point3D,
    },
    /// Two segments must be perpendicular (AB ⟂ CD).
    Perpendicular {
        a1: Point3D,
        a2: Point3D,
        b1: Point3D,
        b2: Point3D,
    },
    /// Two segments must have equal length (|AB| = |CD|).
    EqualLength {
        a1: Point3D,
        a2: Point3D,
        b1: Point3D,
        b2: Point3D,
    },
    /// Pin a point at a fixed location.
    FixedPoint { point: Point3D, anchor: Point3D },
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
            Self::Angle { a, b, c, target } => {
                let ba = Vector3D::between(b, a);
                let bc = Vector3D::between(b, c);
                let dot = ba.dot(&bc);
                let mags = ba.length().value * bc.length().value;
                if mags < 1e-15 {
                    return Scalar::ZERO;
                }
                let cos_angle = (dot.value / mags).clamp(-1.0, 1.0);
                let actual = cos_angle.acos();
                (actual - target.value).abs().into()
            }
            Self::Parallel { a1, a2, b1, b2 } => {
                let ab = Vector3D::between(a1, a2);
                let cd = Vector3D::between(b1, b2);
                let cross = ab.cross(&cd);
                cross.length()
            }
            Self::Perpendicular { a1, a2, b1, b2 } => {
                let ab = Vector3D::between(a1, a2);
                let cd = Vector3D::between(b1, b2);
                ab.dot(&cd).value.abs().into()
            }
            Self::EqualLength { a1, a2, b1, b2 } => {
                let l1 = a1.distance_to(a2);
                let l2 = b1.distance_to(b2);
                (l1 - l2).value.abs().into()
            }
            Self::FixedPoint { point, anchor } => point.distance_to(anchor),
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
            Self::Angle { .. } => "angle",
            Self::Parallel { .. } => "parallel",
            Self::Perpendicular { .. } => "perpendicular",
            Self::EqualLength { .. } => "equal_length",
            Self::FixedPoint { .. } => "fixed_point",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::scalar;
    use std::f64::consts::FRAC_PI_2;

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    #[test]
    fn distance_satisfied() {
        let c = ConstraintKind::Distance {
            a: Point3D::new(0.0, 0.0, 0.0),
            b: Point3D::new(3.0, 4.0, 0.0),
            target: scalar(5.0),
        };
        assert!(c.is_satisfied(&tol()));
    }
    #[test]
    fn distance_violated() {
        let c = ConstraintKind::Distance {
            a: Point3D::new(0.0, 0.0, 0.0),
            b: Point3D::new(1.0, 0.0, 0.0),
            target: scalar(10.0),
        };
        assert!(!c.is_satisfied(&tol()));
    }
    #[test]
    fn horizontal_satisfied() {
        let c = ConstraintKind::Horizontal {
            a: Point3D::new(1.0, 5.0, 0.0),
            b: Point3D::new(10.0, 5.0, 0.0),
        };
        assert!(c.is_satisfied(&tol()));
    }
    #[test]
    fn angle_right() {
        let c = ConstraintKind::Angle {
            a: Point3D::new(1.0, 0.0, 0.0),
            b: Point3D::new(0.0, 0.0, 0.0),
            c: Point3D::new(0.0, 1.0, 0.0),
            target: scalar(FRAC_PI_2),
        };
        assert!(c.is_satisfied(&tol()));
    }
    #[test]
    fn parallel_satisfied() {
        let c = ConstraintKind::Parallel {
            a1: Point3D::new(0.0, 0.0, 0.0),
            a2: Point3D::new(1.0, 0.0, 0.0),
            b1: Point3D::new(0.0, 1.0, 0.0),
            b2: Point3D::new(2.0, 1.0, 0.0),
        };
        assert!(c.is_satisfied(&tol()));
    }
    #[test]
    fn perpendicular_satisfied() {
        let c = ConstraintKind::Perpendicular {
            a1: Point3D::new(0.0, 0.0, 0.0),
            a2: Point3D::new(1.0, 0.0, 0.0),
            b1: Point3D::new(0.0, 0.0, 0.0),
            b2: Point3D::new(0.0, 1.0, 0.0),
        };
        assert!(c.is_satisfied(&tol()));
    }
    #[test]
    fn equal_length_satisfied() {
        let c = ConstraintKind::EqualLength {
            a1: Point3D::new(0.0, 0.0, 0.0),
            a2: Point3D::new(3.0, 4.0, 0.0),
            b1: Point3D::new(10.0, 0.0, 0.0),
            b2: Point3D::new(10.0, 5.0, 0.0),
        };
        assert!(c.is_satisfied(&tol()));
    }
    #[test]
    fn fixed_point_satisfied() {
        let c = ConstraintKind::FixedPoint {
            point: Point3D::new(1.0, 2.0, 3.0),
            anchor: Point3D::new(1.0, 2.0, 3.0),
        };
        assert!(c.is_satisfied(&tol()));
    }
    #[test]
    fn fixed_point_violated() {
        let c = ConstraintKind::FixedPoint {
            point: Point3D::new(0.0, 0.0, 0.0),
            anchor: Point3D::new(1.0, 0.0, 0.0),
        };
        assert!(!c.is_satisfied(&tol()));
    }
}
