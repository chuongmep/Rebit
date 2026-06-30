//! constraint_solver — deterministic 2D/3D constraint solving for the Rebit platform.
//!
//! # Architecture
//!
//! The solver is organised as a bipartite graph of variables and constraints,
//! solved iteratively with deterministic, tolerance-aware convergence:
//!
//! ```text
//! Variable ─── Constraint ─── Variable
//!   (value)       (relation)      (value)
//! ```
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`variable`] | Degrees of freedom — numeric values with optional bounds |
//! | [`graph`] | Bipartite constraint graph — variable & constraint nodes |
//! | [`constraint`] | Constraint types (dimensional, geometric, angle, fixed) |
//! | [`solver`] | Phase A: Gauss-Seidel relaxation solver |
//! | [`newton`] | Phase B: Newton-Raphson solver for faster convergence |
//! | [`sketch`] | Phase B: Sketch-level integration with point management |
//!
//! # Determinism guarantees
//!
//! 1. All iteration uses fixed numeric seeds and stable sort orders.
//! 2. Constraint evaluation order is deterministic (graph traversal by id).
//! 3. No floating-point comparisons outside of [`core_math::Tolerance`].
//! 4. The solver stops at a fixed iteration cap regardless of convergence.
//!
//! # Phase B additions
//!
//! - Newton-Raphson solver with finite-difference Jacobian approximation
//! - Angle, Parallel, Perpendicular, EqualLength, FixedPoint constraints
//! - Sketch-level integration with point management and re-solving

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ---------------------------------------------------------------------------
// Re-exports for convenience
// ---------------------------------------------------------------------------

pub use core_math::{Scalar, Tolerance, scalar};
pub use geometry_kernel::Point3D;

// ---------------------------------------------------------------------------
// Modules
// ---------------------------------------------------------------------------

pub mod constraint;
pub mod graph;
pub mod newton;
pub mod sketch;
pub mod solver;
pub mod variable;
