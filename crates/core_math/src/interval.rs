//! 1D interval [min, max] used for bounding boxes and range queries.

use crate::{Float, Scalar, Tolerance};

/// A closed 1D interval `[min, max]`.
///
/// Invariant: `min <= max` (enforced at construction).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    /// Lower bound.
    pub min: Scalar,
    /// Upper bound.
    pub max: Scalar,
}

impl Interval {
    /// Create a new interval; panics in debug if `min > max`.
    #[inline]
    pub fn new(min: Scalar, max: Scalar) -> Self {
        debug_assert!(min.value <= max.value, "Interval::new: min > max");
        Self { min, max }
    }

    /// Length of the interval: `max - min`.
    #[inline]
    pub fn length(&self) -> Scalar {
        self.max - self.min
    }

    /// Midpoint of the interval.
    #[inline]
    pub fn midpoint(&self) -> Scalar {
        Scalar::new((self.min.value + self.max.value) * 0.5)
    }

    /// Expand the interval to include `value`.
    #[inline]
    pub fn expand(&mut self, value: Scalar) {
        if value.value < self.min.value {
            self.min = value;
        }
        if value.value > self.max.value {
            self.max = value;
        }
    }

    /// `true` when `value` lies within `[min, max]` (inclusive) under `tol`.
    #[inline]
    pub fn contains(&self, value: Scalar, tol: &Tolerance) -> bool {
        value.value >= self.min.value - tol.absolute
            && value.value <= self.max.value + tol.absolute
    }

    /// `true` when two intervals overlap under `tol`.
    #[inline]
    pub fn overlaps(&self, other: &Self, tol: &Tolerance) -> bool {
        self.max.value + tol.absolute >= other.min.value
            && other.max.value + tol.absolute >= self.min.value
    }
}

impl From<(Float, Float)> for Interval {
    #[inline]
    fn from((min, max): (Float, Float)) -> Self {
        Self::new(Scalar::new(min), Scalar::new(max))
    }
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}, {}]", self.min, self.max)
    }
}