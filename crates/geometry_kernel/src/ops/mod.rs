//! Geometry operations — split into focused sub-modules.
//!
//! | Module | Responsibility |
//! |---|---|
//! | [`classification`] | Face/point classification relative to solids |
//! | [`split`] | Face-plane clipping (Sutherland-Hodgman) |
//! | [`boolean`] | Solid boolean operations (union, subtract, intersect) |
//! | [`intersection`] | Line-plane and general intersection types |
//! | [`validation`] | Phase C topology validation and diagnostics |

pub mod boolean;
pub mod classification;
pub mod intersection;
pub mod split;
pub mod validation;
