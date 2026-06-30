//! Tolerance-aware scalar arithmetic.
//!
//! Every floating-point comparison in the engine is routed through
//! [`Tolerance`] so that the codebase has a single tuning knob for
//! geometric robustness.  The default tolerance is chosen to be tight
//! enough for architectural-scale modelling (millimetre-level accuracy over
//! hundreds of metres) while avoiding the worst false-positive failures
//! from `f64` round-off.

use crate::Float;

// ---------------------------------------------------------------------------
// Tolerance
// ---------------------------------------------------------------------------

/// Global tolerance context used for all geometric comparisons.
///
/// # Usage
///
/// ```rust,ignore
/// use core_math::{Tolerance, scalar};
/// let tol = Tolerance::default();
/// let a = scalar(1.0);
/// let b = scalar(1.0 + 1e-12);
/// assert!(tol.nearly_equal(a, b));
/// assert!(tol.distinct(a, scalar(1.1)));
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Tolerance {
    /// Absolute tolerance for comparing lengths/distances.
    pub absolute: Float,
    /// Relative tolerance as a fraction, e.g. 1e-9 means 1 part per billion.
    pub relative: Float,
}

impl Default for Tolerance {
    /// Sensible defaults for BIM/CAD modelling.
    ///
    /// - absolute = `1e-6` (micrometre — well below architectural precision)
    /// - relative = `1e-9`
    fn default() -> Self {
        Self {
            absolute: 1e-6,
            relative: 1e-9,
        }
    }
}

impl Tolerance {
    // -- absolute helpers ---------------------------------------------------

    /// `true` when `|a - b| <= self.absolute`.
    #[inline]
    pub fn nearly_equal_abs(&self, a: Float, b: Float) -> bool {
        (a - b).abs() <= self.absolute
    }

    /// `true` when `a < b - self.absolute`.
    #[inline]
    pub fn strictly_less_abs(&self, a: Float, b: Float) -> bool {
        a < b - self.absolute
    }

    /// `true` when `a > b + self.absolute`.
    #[inline]
    pub fn strictly_greater_abs(&self, a: Float, b: Float) -> bool {
        a > b + self.absolute
    }

    // -- relative helpers ---------------------------------------------------

    /// Combined absolute + relative comparison (the default used by geometry).
    ///
    /// Satisfies `|a - b| <= max(self.absolute, self.relative * max(|a|, |b|))`.
    #[inline]
    pub fn nearly_equal(&self, a: Float, b: Float) -> bool {
        let abs_diff = (a - b).abs();
        let threshold = self.absolute.max(self.relative * a.abs().max(b.abs()));
        abs_diff <= threshold
    }

    /// Convenience: `true` when `a` and `b` are **not** nearly-equal.
    #[inline]
    pub fn distinct(&self, a: Float, b: Float) -> bool {
        !self.nearly_equal(a, b)
    }

    /// Return `self.absolute` (maximum meaningful distance to still consider two
    /// values equal).  Most callers should use `nearly_equal` instead of raw
    /// comparison with this value.
    #[inline]
    pub fn absolute_tolerance(&self) -> Float {
        self.absolute
    }

    /// Return `self.relative`.
    #[inline]
    pub fn relative_tolerance(&self) -> Float {
        self.relative
    }

    /// Clamp `value` to `0.0` if it lies within the absolute tolerance of zero.
    #[inline]
    pub fn snap_to_zero(&self, value: Float) -> Float {
        if value.abs() <= self.absolute {
            0.0
        } else {
            value
        }
    }
}

// ---------------------------------------------------------------------------
// Scalar — newtype with unit-bearing metadata (reservoir for future units)
// ---------------------------------------------------------------------------

/// Newtype over [`Float`] that carries optional unit metadata.
///
/// In v1 this is a transparent wrapper; future phases may add compile-time or
/// run-time unit checking to prevent e.g. adding millimetres to metres.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Scalar {
    /// The underlying `f64` value.
    pub value: Float,
}

impl Scalar {
    /// Create a [`Scalar`] from a raw [`Float`].
    #[inline]
    pub fn new(value: Float) -> Self {
        Self { value }
    }

    /// Convenience: `Scalar::new(0.0)`.
    pub const ZERO: Self = Self { value: 0.0 };

    /// Convenience: `Scalar::new(1.0)`.
    pub const ONE: Self = Self { value: 1.0 };
}

impl std::ops::Add for Scalar {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self::new(self.value + rhs.value)
    }
}

impl std::ops::Sub for Scalar {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.value - rhs.value)
    }
}

impl std::ops::Mul for Scalar {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self::new(self.value * rhs.value)
    }
}

impl std::ops::Div for Scalar {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        debug_assert!(!rhs.value.is_nan() && rhs.value != 0.0);
        Self::new(self.value / rhs.value)
    }
}

impl std::ops::Neg for Scalar {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(-self.value)
    }
}

impl std::fmt::Display for Scalar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<Float> for Scalar {
    #[inline]
    fn from(v: Float) -> Self {
        Self::new(v)
    }
}

impl From<Scalar> for Float {
    #[inline]
    fn from(s: Scalar) -> Self {
        s.value
    }
}

// ---------------------------------------------------------------------------
// Convenience constructor
// ---------------------------------------------------------------------------

/// Create a [`Scalar`] from a literal or `f64` value.
///
/// ```rust,ignore
/// let d = scalar(3.1415);
/// ```
#[inline]
pub fn scalar(value: Float) -> Scalar {
    Scalar::new(value)
}