//! 3D point type.

use crate::{Float, Scalar, Tolerance};

/// A point in 3D space.
///
/// Represented as three [`Scalar`] components `(x, y, z)`.  All operations
/// are delegated through [`Tolerance`] for comparisons.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point3D {
    /// X coordinate.
    pub x: Scalar,
    /// Y coordinate.
    pub y: Scalar,
    /// Z coordinate.
    pub z: Scalar,
}

impl Point3D {
    /// Origin `(0, 0, 0)`.
    pub const ORIGIN: Self = Self {
        x: Scalar::ZERO,
        y: Scalar::ZERO,
        z: Scalar::ZERO,
    };

    /// Create a new point.
    #[inline]
    pub fn new(x: impl Into<Scalar>, y: impl Into<Scalar>, z: impl Into<Scalar>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }

    /// Distance (L2 norm) from this point to another.
    #[inline]
    pub fn distance_to(&self, other: &Self) -> Scalar {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        Scalar::new((dx.value.powi(2) + dy.value.powi(2) + dz.value.powi(2)).sqrt())
    }

    /// Squared distance from this point to another (avoids a sqrt).
    #[inline]
    pub fn distance_squared_to(&self, other: &Self) -> Scalar {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        Scalar::new(dx.value.powi(2) + dy.value.powi(2) + dz.value.powi(2))
    }

    /// `true` when this point coincides with `other` under `tol`.
    #[inline]
    pub fn nearly_equal(&self, other: &Self, tol: &Tolerance) -> bool {
        tol.nearly_equal(self.x.value, other.x.value)
            && tol.nearly_equal(self.y.value, other.y.value)
            && tol.nearly_equal(self.z.value, other.z.value)
    }

    /// Midpoint between two points.
    #[inline]
    pub fn midpoint(&self, other: &Self) -> Self {
        Self::new(
            (self.x + other.x) * Scalar::new(0.5),
            (self.y + other.y) * Scalar::new(0.5),
            (self.z + other.z) * Scalar::new(0.5),
        )
    }

    /// Access components as a `[Float; 3]` array (for interop).
    #[inline]
    pub fn to_array(&self) -> [Float; 3] {
        [self.x.value, self.y.value, self.z.value]
    }

    /// Build from a `[Float; 3]` array.
    #[inline]
    pub fn from_array(arr: [Float; 3]) -> Self {
        Self::new(Scalar::new(arr[0]), Scalar::new(arr[1]), Scalar::new(arr[2]))
    }
}

impl std::fmt::Display for Point3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}