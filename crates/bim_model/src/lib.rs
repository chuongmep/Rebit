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
//! | [`relationship`] | Phase B: entity dependency/connection graph |
//! | [`transaction`] | Change tracking, undo/redo, atomic commits |
//! | [`schema`] | Schema definition, versioning, migration stubs |
//!
//! # Phase B additions
//!
//! - `RelationshipGraph` with `ConnectedTo`, `Contains`, `Supports`, `Hosts`
//! - Entity neighbor queries, outgoing/incoming edge traversal

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use core_math::{Scalar, Tolerance, scalar};
pub use geometry_kernel::{Point3D, Vector3D, shapes::BoundingBox, topology::Solid};

pub mod entity;
pub mod relationship;
pub mod schema;
pub mod transaction;
