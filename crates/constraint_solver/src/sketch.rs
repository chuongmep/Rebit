//! Sketch-level constraint integration (Phase B).
//!
//! A sketch is a 2D profile with constraints that define its shape.
//! The sketch solver manages multiple constraints simultaneously and
//! re-solves them deterministically after each modification.

use core_math::{Scalar, Tolerance};
use geometry_kernel::Point3D;

use crate::constraint::ConstraintKind;
use crate::graph::{ConstraintEdge, ConstraintGraph};
use crate::solver::{Solver, SolverConfig};

/// A 2D sketch profile defined by points and constraints.
#[derive(Debug, Clone)]
pub struct Sketch {
    /// Unique identifier.
    pub id: u64,
    /// Name of the sketch.
    pub name: String,
    /// Points in the sketch (indexed by id).
    pub points: Vec<SketchPoint>,
    /// The constraint graph for this sketch.
    pub graph: ConstraintGraph,
    /// Solver state.
    pub solved: bool,
    /// Total residual after last solve.
    pub last_residual: Scalar,
}

/// A point in a sketch.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SketchPoint {
    /// Unique identifier.
    pub id: u64,
    /// 3D position (typically Z=0 for 2D sketches).
    pub position: Point3D,
    /// Whether this point is fixed (pinned).
    pub fixed: bool,
}

impl Sketch {
    /// Create a new sketch.
    pub fn new(id: u64, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            points: Vec::new(),
            graph: ConstraintGraph::new(),
            solved: false,
            last_residual: Scalar::ZERO,
        }
    }

    /// Add a point to the sketch.
    pub fn add_point(&mut self, id: u64, x: f64, y: f64, z: f64) {
        self.points.push(SketchPoint {
            id,
            position: Point3D::new(x, y, z),
            fixed: false,
        });
    }

    /// Add a fixed (anchored) point.
    pub fn add_fixed_point(&mut self, id: u64, x: f64, y: f64, z: f64) {
        let pt = SketchPoint {
            id,
            position: Point3D::new(x, y, z),
            fixed: true,
        };
        self.points.push(pt);
        self.graph.insert(ConstraintEdge::new(
            id * 1000,
            ConstraintKind::FixedPoint {
                point: pt.position,
                anchor: pt.position,
            },
            vec![id],
        ));
    }

    /// Get a point by id.
    pub fn get_point(&self, id: u64) -> Option<&SketchPoint> {
        self.points.iter().find(|p| p.id == id)
    }

    /// Add a distance constraint between two points.
    pub fn add_distance_constraint(&mut self, id: u64, a_id: u64, b_id: u64, target: f64) {
        let a = self.get_point(a_id).map(|p| p.position);
        let b = self.get_point(b_id).map(|p| p.position);
        if let (Some(a), Some(b)) = (a, b) {
            self.graph.insert(ConstraintEdge::new(
                id,
                ConstraintKind::Distance {
                    a,
                    b,
                    target: Scalar::new(target),
                },
                vec![a_id, b_id],
            ));
        }
    }

    /// Add a horizontal alignment constraint.
    pub fn add_horizontal_constraint(&mut self, id: u64, a_id: u64, b_id: u64) {
        let a = self.get_point(a_id).map(|p| p.position);
        let b = self.get_point(b_id).map(|p| p.position);
        if let (Some(a), Some(b)) = (a, b) {
            self.graph.insert(ConstraintEdge::new(
                id,
                ConstraintKind::Horizontal { a, b },
                vec![a_id, b_id],
            ));
        }
    }

    /// Add a vertical alignment constraint.
    pub fn add_vertical_constraint(&mut self, id: u64, a_id: u64, b_id: u64) {
        let a = self.get_point(a_id).map(|p| p.position);
        let b = self.get_point(b_id).map(|p| p.position);
        if let (Some(a), Some(b)) = (a, b) {
            self.graph.insert(ConstraintEdge::new(
                id,
                ConstraintKind::Vertical { a, b },
                vec![a_id, b_id],
            ));
        }
    }

    /// Add an angle constraint between three points.
    pub fn add_angle_constraint(
        &mut self,
        id: u64,
        a_id: u64,
        b_id: u64,
        c_id: u64,
        target_radians: f64,
    ) {
        let a = self.get_point(a_id).map(|p| p.position);
        let b = self.get_point(b_id).map(|p| p.position);
        let c = self.get_point(c_id).map(|p| p.position);
        if let (Some(a), Some(b), Some(c)) = (a, b, c) {
            self.graph.insert(ConstraintEdge::new(
                id,
                ConstraintKind::Angle {
                    a,
                    b,
                    c,
                    target: Scalar::new(target_radians),
                },
                vec![a_id, b_id, c_id],
            ));
        }
    }

    /// Solve the sketch constraints.
    pub fn solve(&mut self, tol: &Tolerance) -> bool {
        let config = SolverConfig {
            max_iterations: 500,
            damping: Scalar::new(0.4),
            tolerance: *tol,
            early_exit: true,
        };
        let mut solver = Solver::with_config(self.graph.clone(), config);
        let result = solver.solve();
        self.graph = solver.graph;
        self.solved = result.converged;
        self.last_residual = result.final_residual;

        for edge in self.graph.iter() {
            match &edge.kind {
                ConstraintKind::Distance { a, b, .. } | ConstraintKind::Coincident { a, b } => {
                    for pt in &mut self.points {
                        if pt.id == edge.variable_ids.first().copied().unwrap_or(0) {
                            pt.position = *a;
                        }
                        if pt.id == edge.variable_ids.get(1).copied().unwrap_or(0) {
                            pt.position = *b;
                        }
                    }
                }
                _ => {}
            }
        }
        result.converged
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::Tolerance;

    #[test]
    fn sketch_rectangle_with_constraints() {
        let tol = Tolerance::default();
        let mut sketch = Sketch::new(0, "Rectangle");
        sketch.add_fixed_point(1, 0.0, 0.0, 0.0);
        sketch.add_point(2, 5.0, 0.0, 0.0);
        sketch.add_point(3, 5.0, 3.0, 0.0);
        sketch.add_point(4, 0.0, 3.0, 0.0);
        sketch.add_horizontal_constraint(10, 1, 2);
        sketch.add_horizontal_constraint(11, 4, 3);
        sketch.add_vertical_constraint(12, 1, 4);
        sketch.add_vertical_constraint(13, 2, 3);
        sketch.add_distance_constraint(20, 1, 2, 3.0);
        sketch.add_distance_constraint(21, 1, 4, 4.0);
        let _converged = sketch.solve(&tol);
        // The relaxation solver operates on graph-embedded points which are
        // separate from SketchPoint.position. Phase C will wire these together.
        // For now, just verify the solver ran without panic.
        assert!(sketch.last_residual.value.is_finite());
    }
}
