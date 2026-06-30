//! 3D vector (direction / displacement) type.

use crate::{Float, Scalar, Tolerance, Point3D};

/// A vector in 3D space representing a direction or displacement.
///
/// Vectors are distinct from points: adding a point + vector yields a point,
/// adding two points is not meaningful.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3D {
    /// X component.
    pub x: Scalar,
    /// Y component.
    pub y: Scalar,
    /// Z component.
    pub z: Scalar,
}

impl Vector3D {
    /// Zero vector.
    pub const ZERO: Self = Self {
        x: Scalar::ZERO,
        y: Scalar::ZERO,
        z: Scalar::ZERO,
    };

    /// Unit vector along +X.
    pub const X: Self = Self {
        x: Scalar::ONE,
        y: Scalar::ZERO,
        z: Scalar::ZERO,
    };

    /// Unit vector along +Y.
    pub const Y: Self = Self {
        x: Scalar::ZERO,
        y: Scalar::ONE,
        z: Scalar::ZERO,
    };

    /// Unit vector along +Z.
    pub const Z: Self = Self {
        x: Scalar::ZERO,
        y: Scalar::ZERO,
        z: Scalar::ONE,
    };

    /// Create a new vector.
    #[inline]
    pub fn new(x: impl Into<Scalar>, y: impl Into<Scalar>, z: impl Into<Scalar>) -> Self {
        Self {
            x: x.into(),
            y: y.into(),
            z: z.into(),
        }
    }

    /// Vector from `from` to `to`: `to - from`.
    #[inline]
    pub fn between(from: &Point3D, to: &Point3D) -> Self {
        Self {
            x: to.x - from.x,
            y: to.y - from.y,
            z: to.z - from.z,
        }
    }

    /// Magnitude (L2 norm).
    #[inline]
    pub fn length(&self) -> Scalar {
        Scalar::new((self.x.value.powi(2) + self.y.value.powi(2) + self.z.value.powi(2)).sqrt())
    }

    /// Squared magnitude (avoids sqrt).
    #[inline]
    pub fn length_squared(&self) -> Scalar {
        Scalar::new(self.x.value.powi(2) + self.y.value.powi(2) + self.z.value.powi(2))
    }

    /// Return a unit vector in the same direction, or [`None`] if zero-length.
    #[inline]
    pub fn normalize(&self) -> Option<Self> {
        let len = self.length();
        if len.value == 0.0 {
            None
        } else {
            Some(Self {
                x: self.x / len,
                y: self.y / len,
                z: self.z / len,
            })
        }
    }

    /// Dot (scalar) product.
    #[inline]
    pub fn dot(&self, other: &Self) -> Scalar {
        Scalar::new(
            self.x.value * other.x.value
                + self.y.value * other.y.value
                + self.z.value * other.z.value,
        )
    }

    /// Cross (vector) product.
    #[inline]
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: Scalar::new(self.y.value * other.z.value - self.z.value * other.y.value),
            y: Scalar::new(self.z.value * other.x.value - self.x.value * other.z.value),
            z: Scalar::new(self.x.value * other.y.value - self.y.value * other.x.value),
        }
    }

    /// Scale the vector by a scalar.
    #[inline]
    pub fn scale(&self, factor: Scalar) -> Self {
        Self {
            x: self.x * factor,
            y: self.y * factor,
            z: self.z * factor,
        }
    }

    /// `true` when this vector is approx zero under `tol`.
    #[inline]
    pub fn is_zero(&self, tol: &Tolerance) -> bool {
        tol.nearly_equal(self.x.value, 0.0)
            && tol.nearly_equal(self.y.value, 0.0)
            && tol.nearly_equal(self.z.value, 0.0)
    }

    /// `true` when this vector is approx equal to `other` under `tol`.
    #[inline]
    pub fn nearly_equal(&self, other: &Self, tol: &Tolerance) -> bool {
        tol.nearly_equal(self.x.value, other.x.value)
            && tol.nearly_equal(self.y.value, other.y.value)
            && tol.nearly_equal(self.z.value, other.z.value)
    }

    /// Access components as a `[Float; 3]` array.
    #[inline]
    pub fn to_array(&self) -> [Float; 3] {
        [self.x.value, self.y.value, self.z.value]
    }
}

impl std::ops::Add for Vector3D {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub for Vector3D {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::ops::Neg for Vector3D {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

impl std::ops::Add<Vector3D> for Point3D {
    type Output = Point3D;
    #[inline]
    fn add(self, rhs: Vector3D) -> Point3D {
        Point3D::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl std::ops::Sub<Vector3D> for Point3D {
    type Output = Point3D;
    #[inline]
    fn sub(self, rhs: Vector3D) -> Point3D {
        Point3D::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl std::fmt::Display for Vector3D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}