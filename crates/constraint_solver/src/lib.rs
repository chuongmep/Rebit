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
//! | [`constraint`] | Constraint types (dimensional, geometric, coincident) |
//! | [`solver`] | Deterministic iterative solver with convergence checks |
//!
//! # Determinism guarantees
//!
//! 1. All iteration uses fixed numeric seeds and stable sort orders.
//! 2. Constraint evaluation order is deterministic (graph traversal by id).
//! 3. No floating-point comparisons outside of [`core_math::Tolerance`].
//! 4. The solver stops at a fixed iteration cap regardless of convergence.
//!
//! # Phase A scope
//!
//! Phase A delivers:
//! - Variable system with scalar-valued degrees of freedom
//! - Constraint graph with bipartite representation
//! - Dimensional constraints: distance, horizontal distance, vertical distance
//! - Geometric constraints: horizontal, vertical, coincident, collinear
//!
//! Full nonlinear solver and sketch-level integration are Phase B deliverables.

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
pub mod solver;
pub mod variable;

// ---------------------------------------------------------------------------
// Placeholder — removed original stub
// ---------------------------------------------------------------------------
