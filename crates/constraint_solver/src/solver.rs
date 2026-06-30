//! Deterministic iterative constraint solver.
//!
//! The solver uses a simple relaxation approach: for each constraint, compute
//! the violation and push the involved point coordinates toward satisfaction.
//! This is a Gauss-Seidel-style iteration that is deterministic (by id
//! ordering) and bounded by a fixed iteration cap.
//!
//! # Phase A limitations
//!
//! - No Jacobian or Newton steps — pure relaxation.
//! - Constraints embedded in the graph own their point data directly.
//! - Phase B will add variable-ID-based dynamic evaluation and a sparse
//!   Newton solver for faster convergence on larger systems.

use core_math::{Scalar, Tolerance, scalar};
use geometry_kernel::{Point3D, Vector3D};

use crate::constraint::ConstraintKind;
use crate::graph::ConstraintGraph;

// ---------------------------------------------------------------------------
// Solver configuration
// ---------------------------------------------------------------------------

/// Solver configuration — controls convergence criteria and iteration budget.
#[derive(Debug, Clone)]
pub struct SolverConfig {
    /// Maximum number of relaxation passes.
    pub max_iterations: usize,
    /// Learning rate (fraction of error to correct per iteration).
    /// 0.3 to 0.5 works well for most architectural sketches.
    pub damping: Scalar,
    /// Tolerance for declaring convergence.
    pub tolerance: Tolerance,
    /// Stop early when all constraints satisfy tolerance.
    pub early_exit: bool,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            max_iterations: 500,
            damping: scalar(0.4),
            tolerance: Tolerance::default(),
            early_exit: true,
        }
    }
}

// ---------------------------------------------------------------------------
// Solver result
// ---------------------------------------------------------------------------

/// Outcome of a solve pass.
#[derive(Debug, Clone)]
pub struct SolverResult {
    /// Number of iterations actually performed.
    pub iterations: usize,
    /// Total residual after the final iteration.
    pub final_residual: Scalar,
    /// Number of constraints satisfied.
    pub satisfied: usize,
    /// Total number of constraints.
    pub total: usize,
    /// Whether the solver converged (all constraints satisfied within tolerance).
    pub converged: bool,
}

impl SolverResult {
    /// `true` when all constraints are satisfied.
    #[inline]
    pub fn is_fully_satisfied(&self) -> bool {
        self.converged
    }
}

// ---------------------------------------------------------------------------
// Solver
// ---------------------------------------------------------------------------

/// The constraint solver.
///
/// Owns the [`ConstraintGraph`] and operates directly on the constraint
/// data (the points embedded in each [`ConstraintKind`]).
#[derive(Debug)]
pub struct Solver {
    /// The constraint graph.
    pub graph: ConstraintGraph,
    /// Solver configuration.
    pub config: SolverConfig,
}

impl Solver {
    /// Create a new solver with a given graph and default config.
    pub fn new(graph: ConstraintGraph) -> Self {
        Self {
            graph,
            config: SolverConfig::default(),
        }
    }

    /// Create a new solver with explicit config.
    pub fn with_config(graph: ConstraintGraph, config: SolverConfig) -> Self {
        Self { graph, config }
    }

    /// Run the solver to convergence (or iteration cap).
    ///
    /// Depending on [`SolverConfig::early_exit`], returns early when all
    /// constraints are satisfied.
    pub fn solve(&mut self) -> SolverResult {
        let total = self.graph.len();

        for iteration in 0..self.config.max_iterations {
            if self.config.early_exit && self.graph.all_satisfied(&self.config.tolerance) {
                return SolverResult {
                    iterations: iteration,
                    final_residual: self.graph.total_residual(),
                    satisfied: self.graph.satisfied_count(&self.config.tolerance),
                    total,
                    converged: true,
                };
            }

            // One relaxation pass — update edges in id order.
            self.relax_pass();
        }

        SolverResult {
            iterations: self.config.max_iterations,
            final_residual: self.graph.total_residual(),
            satisfied: self.graph.satisfied_count(&self.config.tolerance),
            total,
            converged: self.graph.all_satisfied(&self.config.tolerance),
        }
    }

    /// Perform one relaxation pass over all constraints.
    ///
    /// For each constraint, compute the violation delta and push the
    /// affected points toward satisfaction weighted by [`SolverConfig::damping`].
    fn relax_pass(&mut self) {
        let n = self.graph.len();
        for i in 0..n {
            let edge = &mut self.graph.edges[i];
            let kind = &edge.kind;
            let updated_kind = relax_constraint(kind, self.config.damping);
            edge.kind = updated_kind;
        }
    }
}

// ---------------------------------------------------------------------------
// Per-constraint relaxation
// ---------------------------------------------------------------------------

/// Apply one relaxation step to a single constraint.
///
/// Returns an updated `ConstraintKind` with points nudged toward
/// satisfaction.
fn relax_constraint(kind: &ConstraintKind, damping: Scalar) -> ConstraintKind {
    match kind {
        ConstraintKind::Distance { a, b, target } => {
            let actual = a.distance_to(b);
            let error = actual.value - target.value;
            if error.abs() < 1e-15 {
                return kind.clone();
            }
            let dir = Vector3D::between(b, a);
            let len = dir.length();
            if len.value < 1e-15 {
                return kind.clone();
            }
            let half_correction = scalar(error * 0.5 * damping.value);
            let unit = dir.scale(Scalar::new(1.0 / len.value));
            let delta = unit.scale(half_correction);
            ConstraintKind::Distance {
                a: Point3D::new(a.x - delta.x, a.y - delta.y, a.z - delta.z),
                b: Point3D::new(b.x + delta.x, b.y + delta.y, b.z + delta.z),
                target: *target,
            }
        }
        ConstraintKind::HorizontalDistance { a, b, target } => {
            let dx = a.x.value - b.x.value;
            let error = dx - target.value;
            if error.abs() < 1e-15 {
                return kind.clone();
            }
            let half = scalar(error * 0.5 * damping.value);
            ConstraintKind::HorizontalDistance {
                a: Point3D::new(a.x.value - half.value, a.y.value, a.z.value),
                b: Point3D::new(b.x.value + half.value, b.y.value, b.z.value),
                target: *target,
            }
        }
        ConstraintKind::VerticalDistance { a, b, target } => {
            let dy = a.y.value - b.y.value;
            let error = dy - target.value;
            if error.abs() < 1e-15 {
                return kind.clone();
            }
            let half = scalar(error * 0.5 * damping.value);
            ConstraintKind::VerticalDistance {
                a: Point3D::new(a.x.value, a.y.value - half.value, a.z.value),
                b: Point3D::new(b.x.value, b.y.value + half.value, b.z.value),
                target: *target,
            }
        }
        ConstraintKind::Horizontal { a, b } => {
            let dy = a.y.value - b.y.value;
            if dy.abs() < 1e-15 {
                return kind.clone();
            }
            let half = scalar(dy * 0.5 * damping.value);
            ConstraintKind::Horizontal {
                a: Point3D::new(a.x.value, a.y.value - half.value, a.z.value),
                b: Point3D::new(b.x.value, b.y.value + half.value, b.z.value),
            }
        }
        ConstraintKind::Vertical { a, b } => {
            let dx = a.x.value - b.x.value;
            if dx.abs() < 1e-15 {
                return kind.clone();
            }
            let half = scalar(dx * 0.5 * damping.value);
            ConstraintKind::Vertical {
                a: Point3D::new(a.x.value - half.value, a.y.value, a.z.value),
                b: Point3D::new(b.x.value + half.value, b.y.value, b.z.value),
            }
        }
        ConstraintKind::Coincident { a, b } => {
            let dist = a.distance_to(b);
            if dist.value < 1e-15 {
                return kind.clone();
            }
            let dir = Vector3D::between(b, a);
            let len = dir.length();
            if len.value < 1e-15 {
                return kind.clone();
            }
            let half_correction = scalar(dist.value * 0.5 * damping.value);
            let unit = dir.scale(Scalar::new(1.0 / len.value));
            let delta = unit.scale(half_correction);
            ConstraintKind::Coincident {
                a: Point3D::new(a.x - delta.x, a.y - delta.y, a.z - delta.z),
                b: Point3D::new(b.x + delta.x, b.y + delta.y, b.z + delta.z),
            }
        }
        ConstraintKind::Collinear { a, b, c } => {
            let ab = Vector3D::between(a, b);
            let ac = Vector3D::between(a, c);
            let len_sq = ab.length_squared();
            if len_sq.value < 1e-15 {
                return kind.clone();
            }
            let t = ac.dot(&ab);
            let t_clamped = (t.value / len_sq.value).clamp(0.0, 1.0);
            let projected = Point3D::new(
                a.x.value + t_clamped * ab.x.value,
                a.y.value + t_clamped * ab.y.value,
                a.z.value + t_clamped * ab.z.value,
            );
            let to_proj = Vector3D::between(c, &projected);
            let delta = to_proj.scale(damping);
            ConstraintKind::Collinear {
                a: *a,
                b: *b,
                c: Point3D::new(
                    c.x.value + delta.x.value,
                    c.y.value + delta.y.value,
                    c.z.value + delta.z.value,
                ),
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraint::ConstraintKind;
    use crate::graph::{ConstraintEdge, ConstraintGraph};
    use core_math::scalar;
    use geometry_kernel::Point3D;

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    #[test]
    fn solve_single_distance_constraint() {
        let mut graph = ConstraintGraph::new();
        graph.insert(ConstraintEdge::new(
            0,
            ConstraintKind::Distance {
                a: Point3D::new(0.0, 0.0, 0.0),
                b: Point3D::new(1.0, 0.0, 0.0),
                target: scalar(5.0),
            },
            vec![],
        ));
        let mut solver = Solver::new(graph);
        let result = solver.solve();
        assert!(result.converged);
        let edge = solver.graph.get(0).unwrap();
        let actual_dist = match &edge.kind {
            ConstraintKind::Distance { a, b, .. } => a.distance_to(b),
            _ => panic!("wrong kind"),
        };
        assert!(tol().nearly_equal(actual_dist.value, 5.0));
    }

    #[test]
    fn solve_coincident() {
        let mut graph = ConstraintGraph::new();
        graph.insert(ConstraintEdge::new(
            0,
            ConstraintKind::Coincident {
                a: Point3D::new(10.0, 0.0, 0.0),
                b: Point3D::new(0.0, 10.0, 0.0),
            },
            vec![],
        ));
        let mut solver = Solver::new(graph);
        let result = solver.solve();
        assert!(result.converged);
        let edge = solver.graph.get(0).unwrap();
        match &edge.kind {
            ConstraintKind::Coincident { a, b } => {
                assert!(a.nearly_equal(b, &tol()));
            }
            _ => panic!("wrong kind"),
        }
    }

    #[test]
    fn solve_horizontal_and_vertical_alignment() {
        let mut graph = ConstraintGraph::new();
        graph.insert(ConstraintEdge::new(
            0,
            ConstraintKind::Horizontal {
                a: Point3D::new(0.0, 5.0, 0.0),
                b: Point3D::new(10.0, 0.0, 0.0),
            },
            vec![],
        ));
        graph.insert(ConstraintEdge::new(
            1,
            ConstraintKind::Vertical {
                a: Point3D::new(0.0, 5.0, 0.0),
                b: Point3D::new(10.0, 0.0, 0.0),
            },
            vec![],
        ));
        let mut solver = Solver::new(graph);
        let result = solver.solve();
        assert!(result.converged, "residual = {:?}", result.final_residual);
        let e0 = solver.graph.get(0).unwrap();
        let e1 = solver.graph.get(1).unwrap();
        assert!(e0.kind.is_satisfied(&tol()));
        assert!(e1.kind.is_satisfied(&tol()));
    }

    #[test]
    fn solve_collinear() {
        let mut graph = ConstraintGraph::new();
        graph.insert(ConstraintEdge::new(
            0,
            ConstraintKind::Collinear {
                a: Point3D::new(0.0, 0.0, 0.0),
                b: Point3D::new(1.0, 1.0, 0.0),
                c: Point3D::new(0.5, 0.0, 0.0),
            },
            vec![],
        ));
        let mut solver = Solver::new(graph);
        let result = solver.solve();
        assert!(result.converged);
        let edge = solver.graph.get(0).unwrap();
        assert!(edge.kind.is_satisfied(&tol()));
    }

    #[test]
    fn solve_small_sketch() {
        let mut graph = ConstraintGraph::new();
        let a = Point3D::new(0.0, 0.0, 0.0);
        let b = Point3D::new(10.0, 0.0, 0.0);
        let c = Point3D::new(0.0, 10.0, 0.0);

        graph.insert(ConstraintEdge::new(
            0,
            ConstraintKind::Horizontal {
                a: Point3D::ORIGIN,
                b,
            },
            vec![],
        ));
        graph.insert(ConstraintEdge::new(
            1,
            ConstraintKind::Vertical {
                a: Point3D::ORIGIN,
                b: c,
            },
            vec![],
        ));
        graph.insert(ConstraintEdge::new(
            2,
            ConstraintKind::Distance {
                a,
                b,
                target: scalar(3.0),
            },
            vec![],
        ));
        graph.insert(ConstraintEdge::new(
            3,
            ConstraintKind::Distance {
                a,
                b: c,
                target: scalar(4.0),
            },
            vec![],
        ));
        graph.insert(ConstraintEdge::new(
            4,
            ConstraintKind::Distance {
                a: b,
                b: c,
                target: scalar(5.0),
            },
            vec![],
        ));

        let mut solver = Solver::new(graph);
        let result = solver.solve();
        assert!(result.converged, "residual = {:?}", result.final_residual);
        assert_eq!(result.satisfied, 5);
    }
}
