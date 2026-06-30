//! Bipartite constraint graph — connects variables to constraints.
//!
//! The graph is the central data structure: it owns all [`ConstraintEdge`]
//! objects and provides deterministic, id-ordered traversal for the solver.

use crate::constraint::ConstraintKind;
use crate::variable::VariableSet;
use core_math::{Scalar, Tolerance};

// ---------------------------------------------------------------------------
// ConstraintEdge
// ---------------------------------------------------------------------------

/// An edge in the constraint graph connecting a set of variables with a
/// constraint relation.
#[derive(Debug, Clone, PartialEq)]
pub struct ConstraintEdge {
    /// Unique identifier (determines evaluation order).
    pub id: u64,
    /// The constraint relation.
    pub kind: ConstraintKind,
    /// IDs of variables this constraint involves.
    pub variable_ids: Vec<u64>,
}

impl ConstraintEdge {
    /// Create a new constraint edge.
    #[inline]
    pub fn new(id: u64, kind: ConstraintKind, variable_ids: Vec<u64>) -> Self {
        Self {
            id,
            kind,
            variable_ids,
        }
    }

    /// `true` when all referenced variable IDs exist in the given set.
    pub fn variables_exist(&self, vars: &VariableSet) -> bool {
        self.variable_ids.iter().all(|id| vars.get(*id).is_some())
    }
}

// ---------------------------------------------------------------------------
// ConstraintGraph
// ---------------------------------------------------------------------------

/// The bipartite constraint graph.
///
/// Edges are stored sorted by `id` for deterministic iteration.
/// Variables are owned separately in a [`VariableSet`].
#[derive(Debug, Clone)]
pub struct ConstraintGraph {
    /// Constraint edges sorted by id (accessible to solver for relaxation).
    pub(crate) edges: Vec<ConstraintEdge>,
}

impl ConstraintGraph {
    /// Create an empty constraint graph.
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    /// Insert a constraint edge (id must not already exist).
    pub fn insert(&mut self, edge: ConstraintEdge) {
        let pos = self
            .edges
            .binary_search_by(|e| e.id.cmp(&edge.id))
            .expect_err("duplicate constraint edge id");
        self.edges.insert(pos, edge);
    }

    /// Number of constraints.
    #[inline]
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// `true` when empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Iterate over edges in id order (deterministic).
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &ConstraintEdge> {
        self.edges.iter()
    }

    /// Get a constraint by id.
    #[inline]
    pub fn get(&self, id: u64) -> Option<&ConstraintEdge> {
        self.edges
            .binary_search_by(|e| e.id.cmp(&id))
            .ok()
            .map(|i| &self.edges[i])
    }

    /// Total residual across all constraints (sum of individual residuals).
    pub fn total_residual(&self) -> Scalar {
        let mut sum = Scalar::ZERO;
        for edge in &self.edges {
            sum = sum + edge.kind.residual();
        }
        sum
    }

    /// `true` when every constraint's residual is within tolerance.
    pub fn all_satisfied(&self, tol: &Tolerance) -> bool {
        self.edges.iter().all(|e| e.kind.is_satisfied(tol))
    }

    /// Count how many constraints are satisfied.
    pub fn satisfied_count(&self, tol: &Tolerance) -> usize {
        self.edges
            .iter()
            .filter(|e| e.kind.is_satisfied(tol))
            .count()
    }
}

impl Default for ConstraintGraph {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraint::ConstraintKind;
    use geometry_kernel::Point3D;

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    #[test]
    fn graph_sorted_insertion() {
        let mut g = ConstraintGraph::new();
        g.insert(ConstraintEdge::new(
            100,
            ConstraintKind::Horizontal {
                a: Point3D::ORIGIN,
                b: Point3D::ORIGIN,
            },
            vec![0, 1],
        ));
        g.insert(ConstraintEdge::new(
            5,
            ConstraintKind::Vertical {
                a: Point3D::ORIGIN,
                b: Point3D::ORIGIN,
            },
            vec![2, 3],
        ));
        let ids: Vec<u64> = g.iter().map(|e| e.id).collect();
        assert_eq!(ids, vec![5, 100]);
    }

    #[test]
    fn graph_total_residual() {
        let mut g = ConstraintGraph::new();
        g.insert(ConstraintEdge::new(
            0,
            ConstraintKind::Horizontal {
                a: Point3D::new(0.0, 0.0, 0.0),
                b: Point3D::new(1.0, 0.0, 0.0),
            },
            vec![0, 1],
        ));
        g.insert(ConstraintEdge::new(
            1,
            ConstraintKind::Vertical {
                a: Point3D::new(0.0, 0.0, 0.0),
                b: Point3D::new(0.0, 5.0, 0.0),
            },
            vec![0, 2],
        ));
        assert!(g.all_satisfied(&tol()));
    }

    #[test]
    fn graph_violated_constraint() {
        let mut g = ConstraintGraph::new();
        g.insert(ConstraintEdge::new(
            0,
            ConstraintKind::Coincident {
                a: Point3D::new(0.0, 0.0, 0.0),
                b: Point3D::new(1.0, 0.0, 0.0),
            },
            vec![0, 1],
        ));
        assert!(!g.all_satisfied(&tol()));
        assert_eq!(g.satisfied_count(&tol()), 0);
    }
}
