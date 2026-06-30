//! core_math — foundational linear-algebra, scalar, and 3D types for the Rebit engine stack.
//!
//! This crate provides the lowest-level mathematical primitives used by every
//! downstream crate (geometry kernel, constraint solver, parametric engine,
//! rendering, etc.).  The chosen representations deliberately favour
//! correctness and numerical robustness over raw throughput; see the
//! [`Tolerance`] documentation for the general tolerance philosophy applied
//! across the codebase.
//!
//! # Design invariants
//!
//! - Every floating-point comparison goes through [`Tolerance`] — never
//!   compare `f64` values with `==` or `<`/`>` directly outside this module.
//! - Public types are `Copy` + `Clone` + `PartialEq` where reasonable.
//! - Geometric primitives use `Point3D` / `Vector3D` as their canonical
//!   position and direction types; `Scalar` is the canonical unit-bearing
//!   numerical type.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ---------------------------------------------------------------------------
// Modules
// ---------------------------------------------------------------------------

mod scalar;
pub use scalar::{Scalar, Tolerance, scalar};

mod point;
pub use point::Point3D;

mod vector;
pub use vector::Vector3D;

mod matrix;
pub use matrix::Matrix4x4;

mod interval;
pub use interval::Interval;

// ---------------------------------------------------------------------------
// Re-exports for convenience
// ---------------------------------------------------------------------------

/// The underlying floating-point type used throughout the engine.
pub type Float = f64;
