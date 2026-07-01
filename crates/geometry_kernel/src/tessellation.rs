//! Tessellation — converts curved topology to planar face approximations.
//!
//! Phase C: enables curved-edge faces to be split into planar triangles/quads
//! that downstream boolean and rendering operations can consume.
#![allow(missing_docs)]

use crate::curves::{BezierCubic, tessellate_bezier};
use core_math::{Point3D, Tolerance};

/// Tessellation quality parameters.
#[derive(Debug, Clone, Copy)]
pub struct TessellationConfig {
    /// Maximum chordal deviation from the true curve (mm).
    pub max_error: f64,
    /// Maximum angle between consecutive tessellation segments (degrees).
    pub max_angle: f64,
}

impl Default for TessellationConfig {
    fn default() -> Self {
        Self {
            max_error: 0.1,
            max_angle: 15.0,
        }
    }
}

/// Tessellate a curved edge into straight line segments.
pub fn tessellate_edge(curve: &BezierCubic, config: &TessellationConfig) -> Vec<Point3D> {
    tessellate_bezier(curve, config.max_error)
}

/// Build a planar fan-triangulation of a polygon (convex only in Phase C).
pub fn triangulate_face(vertices: &[Point3D], _tol: &Tolerance) -> Vec<[usize; 3]> {
    // Simple fan triangulation from vertex 0.
    let n = vertices.len();
    if n < 3 {
        return vec![];
    }
    let mut tris = Vec::with_capacity(n - 2);
    for i in 1..(n - 1) {
        tris.push([0, i, i + 1]);
    }
    tris
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn triangulate_quad() {
        let pts = vec![
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
        ];
        let tris = triangulate_face(&pts, &Tolerance::default());
        assert_eq!(tris.len(), 2);
    }
    #[test]
    fn tessellate_edge_straight() {
        let c = BezierCubic {
            p0: Point3D::new(0.0, 0.0, 0.0),
            p1: Point3D::new(1.0, 0.0, 0.0),
            p2: Point3D::new(2.0, 0.0, 0.0),
            p3: Point3D::new(3.0, 0.0, 0.0),
        };
        let pts = tessellate_edge(&c, &TessellationConfig::default());
        assert!(pts.len() >= 2);
    }
}
