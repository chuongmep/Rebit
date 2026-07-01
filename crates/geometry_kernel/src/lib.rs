//! geometry_kernel — the foundational geometry engine for the Rebit BIM/CAD platform.
//!
//! # Architecture
//!
//! The kernel is organised in four layers:
//!
//! | Layer | Module | Responsibility |
//! |---|---|---|
//! | 0 — math primitives | `core_math` | Points, vectors, matrices, tolerances |
//! | 1 — shapes | [`shapes`] | Analytic geometry: lines, planes, bboxes, csg primitives |
//! | 2 — topology | [`topology`] | Boundary representation: vertex → edge → face → shell → solid |
//! | 3 — operations | [`ops`] | Booleans, intersections, conversions (Phase B+) |
//!
//! ## Tolerance strategy
//!
//! Every geometric comparison uses [`core_math::Tolerance`].  The kernel
//! **never** compares `f64` values directly.  The default tolerance
//! (`absolute = 1e-6`, `relative = 1e-9`) is tuned for architectural-scale
//! modelling with millimetre-accuracy over hundreds of metres.
//!
//! ## Invariants (enforced by construction / debug assertions)
//!
//! 1. No raw `f64` equality or ordering outside of tolerance-aware paths.
//! 2. Topological entities are immutable once constructed (value semantics).
//! 3. Faces are planar.  Curved surfaces are deferred to Phase B.
//! 4. Bounding boxes are always valid: `min[i] <= max[i]` for all axes.
//! 5. No unsafe code.
//!
//! ## Design Decision: Phase A scope
//!
//! Phase A delivers the architecture scaffolding and basic primitives —
//! enough to unblock the BIM Model and Parametric Engine teams.  Full
//! boolean operations and B-Rep validation are Phase B deliverables.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ---------------------------------------------------------------------------
// Re-export core_math for downstream convenience
// ---------------------------------------------------------------------------

pub use core_math::{Float, Matrix4x4, Point3D, Scalar, Tolerance, Vector3D, scalar};

// ---------------------------------------------------------------------------
// Modules
// ---------------------------------------------------------------------------

pub mod curves;
pub mod mesh;
pub mod ops;
pub mod shapes;
pub mod surfaces;
pub mod tessellation;
pub mod topology;

// ---------------------------------------------------------------------------
// Kernel-wide configuration
// ---------------------------------------------------------------------------

/// Build the default tolerance instance used by the kernel.
///
/// This is a function (not a constant) so that future work can introduce
/// per-context tolerance overrides without changing callers.
#[inline]
pub fn default_tolerance() -> Tolerance {
    Tolerance::default()
}

// ---------------------------------------------------------------------------
// Placeholder — removed original stub
// ---------------------------------------------------------------------------
