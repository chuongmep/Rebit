//! 4×4 homogeneous transformation matrix.

use crate::{Float, Scalar, Point3D, Vector3D};

/// A 4×4 matrix in row-major storage for homogeneous transforms.
///
/// Used for affine transformations (translation, rotation, scale) in the
/// geometry pipeline.  The storage convention is row-major:
/// `row * column` with elements `m[usize]` indexed `r*4 + c`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix4x4 {
    /// Row-major elements: m[0..3] = row 0, m[4..7] = row 1, etc.
    pub m: [Float; 16],
}

impl Matrix4x4 {
    /// Identity matrix.
    pub const IDENTITY: Self = Self {
        m: [
            1.0, 0.0, 0.0, 0.0, //
            0.0, 1.0, 0.0, 0.0, //
            0.0, 0.0, 1.0, 0.0, //
            0.0, 0.0, 0.0, 1.0, //
        ],
    };

    /// Build a translation matrix.
    #[inline]
    pub fn translation(v: &Vector3D) -> Self {
        let mut mat = Self::IDENTITY;
        mat.m[3] = v.x.value;
        mat.m[7] = v.y.value;
        mat.m[11] = v.z.value;
        mat
    }

    /// Multiply this matrix by another: `self * other`.
    #[inline]
    pub fn multiply(&self, other: &Self) -> Self {
        let mut m = [0.0_f64; 16];
        for r in 0..4 {
            for c in 0..4 {
                m[r * 4 + c] = self.m[r * 4] * other.m[c]
                    + self.m[r * 4 + 1] * other.m[4 + c]
                    + self.m[r * 4 + 2] * other.m[8 + c]
                    + self.m[r * 4 + 3] * other.m[12 + c];
            }
        }
        Self { m }
    }

    /// Transform a point (treating the homogeneous w = 1).
    #[inline]
    pub fn transform_point(&self, p: &Point3D) -> Point3D {
        let x = self.m[0] * p.x.value
            + self.m[1] * p.y.value
            + self.m[2] * p.z.value
            + self.m[3];
        let y = self.m[4] * p.x.value
            + self.m[5] * p.y.value
            + self.m[6] * p.z.value
            + self.m[7];
        let z = self.m[8] * p.x.value
            + self.m[9] * p.y.value
            + self.m[10] * p.z.value
            + self.m[11];
        let w = self.m[12] * p.x.value
            + self.m[13] * p.y.value
            + self.m[14] * p.z.value
            + self.m[15];
        debug_assert!(w.abs() > 1e-15, "degenerate homogeneous coordinate w = {w}");
        Point3D::new(Scalar::new(x / w), Scalar::new(y / w), Scalar::new(z / w))
    }

    /// Transform a direction vector (ignoring translation, w = 0).
    #[inline]
    pub fn transform_vector(&self, v: &Vector3D) -> Vector3D {
        let x = self.m[0] * v.x.value + self.m[1] * v.y.value + self.m[2] * v.z.value;
        let y = self.m[4] * v.x.value + self.m[5] * v.y.value + self.m[6] * v.z.value;
        let z = self.m[8] * v.x.value + self.m[9] * v.y.value + self.m[10] * v.z.value;
        Vector3D::new(Scalar::new(x), Scalar::new(y), Scalar::new(z))
    }
}

impl Default for Matrix4x4 {
    fn default() -> Self {
        Self::IDENTITY
    }
}