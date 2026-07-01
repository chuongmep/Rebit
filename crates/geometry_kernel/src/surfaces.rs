//! Parametric surface types — Phase C (revolution, sweep, loft).
#![allow(missing_docs)]

use crate::curves::BezierCubic;
use core_math::{Point3D, Vector3D};

/// A surface of revolution around the Z axis.
#[derive(Debug, Clone, PartialEq)]
pub struct RevolutionSurface {
    pub profile: Vec<Point3D>,
    pub axis_origin: Point3D,
    pub axis_direction: Vector3D,
}

impl RevolutionSurface {
    pub fn evaluate(&self, u: f64, v: f64) -> Point3D {
        let u = u.clamp(0.0, 1.0);
        let _v = v.clamp(0.0, 1.0);
        if self.profile.len() < 2 {
            return self.axis_origin;
        }
        let idx = (u * (self.profile.len() - 1) as f64) as usize;
        let idx = idx.min(self.profile.len() - 2);
        let frac = u * (self.profile.len() - 1) as f64 - idx as f64;
        let p0 = self.profile[idx];
        let p1 = self.profile[idx + 1];
        let h = p0.z.value + frac * (p1.z.value - p0.z.value);
        let r = p0.x.value + frac * (p1.x.value - p0.x.value);
        let angle = _v * std::f64::consts::TAU;
        Point3D::new(r * angle.cos(), r * angle.sin(), h)
    }
}

/// A swept surface along a path curve.
#[derive(Debug, Clone, PartialEq)]
pub struct SweepSurface {
    pub profile: Vec<Point3D>,
    pub path: BezierCubic,
}

impl SweepSurface {
    pub fn evaluate(&self, u: f64, v: f64) -> Point3D {
        let path_pt = self.path.evaluate(v);
        let tangent = self.path.derivative(v);
        if self.profile.len() < 2 {
            return path_pt;
        }
        let idx = (u * (self.profile.len() - 1) as f64) as usize;
        let idx = idx.min(self.profile.len() - 2);
        let frac = u * (self.profile.len() - 1) as f64 - idx as f64;
        let p0 = self.profile[idx];
        let p1 = self.profile[idx + 1];
        let x = p0.x.value + frac * (p1.x.value - p0.x.value);
        let y = p0.y.value + frac * (p1.y.value - p0.y.value);
        let normal = tangent.cross(&Vector3D::Z);
        let binormal = tangent.cross(&normal);
        Point3D::new(
            path_pt.x.value + x * binormal.x.value + y * normal.x.value,
            path_pt.y.value + x * binormal.y.value + y * normal.y.value,
            path_pt.z.value + x * binormal.z.value + y * normal.z.value,
        )
    }
}

/// A lofted surface between two profile curves.
#[derive(Debug, Clone, PartialEq)]
pub struct LoftSurface {
    pub profiles: Vec<Vec<Point3D>>,
}

impl LoftSurface {
    pub fn evaluate(&self, u: f64, v: f64) -> Point3D {
        if self.profiles.len() < 2 {
            return Point3D::ORIGIN;
        }
        let up = v.clamp(0.0, 1.0);
        let idx = (up * (self.profiles.len() - 1) as f64) as usize;
        let idx = idx.min(self.profiles.len() - 2);
        let frac = up * (self.profiles.len() - 1) as f64 - idx as f64;
        let p0 = &self.profiles[idx];
        let p1 = &self.profiles[idx + 1];
        if p0.len() < 2 || p1.len() < 2 {
            return Point3D::ORIGIN;
        }
        let u_idx = (u * (p0.len() - 1) as f64) as usize;
        let u_idx = u_idx.min(p0.len() - 2);
        let u_frac = u * (p0.len() - 1) as f64 - u_idx as f64;
        let a = Point3D::new(
            p0[u_idx].x.value + u_frac * (p0[u_idx + 1].x.value - p0[u_idx].x.value),
            p0[u_idx].y.value + u_frac * (p0[u_idx + 1].y.value - p0[u_idx].y.value),
            p0[u_idx].z.value + u_frac * (p0[u_idx + 1].z.value - p0[u_idx].z.value),
        );
        let b = Point3D::new(
            p1[u_idx].x.value + u_frac * (p1[u_idx + 1].x.value - p1[u_idx].x.value),
            p1[u_idx].y.value + u_frac * (p1[u_idx + 1].y.value - p1[u_idx].y.value),
            p1[u_idx].z.value + u_frac * (p1[u_idx + 1].z.value - p1[u_idx].z.value),
        );
        Point3D::new(
            a.x.value + frac * (b.x.value - a.x.value),
            a.y.value + frac * (b.y.value - a.y.value),
            a.z.value + frac * (b.z.value - a.z.value),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn revolution_cylinder() {
        let s = RevolutionSurface {
            profile: vec![Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 2.0)],
            axis_origin: Point3D::ORIGIN,
            axis_direction: Vector3D::Z,
        };
        let p = s.evaluate(0.5, 0.0);
        assert!((p.x.value - 1.0).abs() < 1e-9, "x={}", p.x.value);
        assert!(p.y.value.abs() < 1e-9, "y={}", p.y.value);
    }
    #[test]
    fn revolution_quarter() {
        let s = RevolutionSurface {
            profile: vec![Point3D::new(1.0, 0.0, 0.0), Point3D::new(1.0, 0.0, 2.0)],
            axis_origin: Point3D::ORIGIN,
            axis_direction: Vector3D::Z,
        };
        let p = s.evaluate(0.5, 0.25);
        assert!((p.y.value - 1.0).abs() < 1e-9, "y={}", p.y.value);
        assert!(p.x.value.abs() < 1e-9, "x={}", p.x.value);
    }
    #[test]
    fn sweep_linear() {
        let c = BezierCubic {
            p0: Point3D::new(0.0, 0.0, 0.0),
            p1: Point3D::new(1.0, 0.0, 0.0),
            p2: Point3D::new(2.0, 0.0, 0.0),
            p3: Point3D::new(3.0, 0.0, 0.0),
        };
        let s = SweepSurface {
            profile: vec![Point3D::new(0.0, 1.0, 0.0), Point3D::new(0.0, 0.0, 0.0)],
            path: c,
        };
        let p = s.evaluate(0.0, 0.5);
        assert!((p.x.value - 1.5).abs() < 1e-9, "x={}", p.x.value);
    }
}
