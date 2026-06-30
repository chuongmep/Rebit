//! bim_model — BIM entity graph, transaction model, and schema for the Rebit platform.
//!
//! # Architecture
//!
//! The BIM data model sits above the geometry kernel and provides building-
//! domain semantics: walls, slabs, beams, columns, doors, windows, and the
//! relationships between them.
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`entity`] | Core BIM entity types, property system, entity graph |
//! | [`transaction`] | Change tracking, undo/redo, atomic commits |
//! | [`schema`] | Schema definition, versioning, migration stubs |
//!
//! # Design principles
//!
//! 1. **Entities are value types with unique IDs.** They carry geometry
//!    (via [`geometry_kernel::Solid`]) and properties.
//! 2. **Transactions are atomic.** Every mutation goes through a
//!    transaction context that can be committed or rolled back.
//! 3. **Schema is versioned.** Future-proof metadata enables model
//!    migration between versions.
//!
//! # Phase A scope
//!
//! - Core entity types: Wall, Slab, Beam, Column, Door, Window
//! - Property system: key-value metadata on every entity
//! - Transaction model: commit/rollback with change snapshots
//! - Schema definitions with version identifiers

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ---------------------------------------------------------------------------
// Re-exports
// ---------------------------------------------------------------------------

pub use core_math::{Scalar, Tolerance, scalar};
pub use geometry_kernel::{Point3D, Vector3D, shapes::BoundingBox, topology::Solid};

// ---------------------------------------------------------------------------
// Modules
// ---------------------------------------------------------------------------

pub mod entity;
pub mod schema;
pub mod transaction;
