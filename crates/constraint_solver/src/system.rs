//! Constraint system — Phase C: bridges VariableSet ↔ constraint graph points.
//!
//! Maps variable IDs to geometric point coordinates, enabling the solver
//! to operate on variables rather than graph-embedded point data.

use crate::constraint::ConstraintKind;
use crate::graph::{ConstraintEdge, ConstraintGraph};
use crate::variable::{VariableSet, VariableValue};
use geometry_kernel::Point3D;

/// A mapping from variable IDs to 2D/3D point coordinates.
#[derive(Debug, Clone, Default)]
pub struct PointMapping {
    /// Variable IDs for x-coordinates per point.
    pub x_vars: Vec<(u64, u64)>, // (point_id, var_id)
    /// Variable IDs for y-coordinates per point.
    pub y_vars: Vec<(u64, u64)>,
    /// Variable IDs for z-coordinates per point.
    pub z_vars: Vec<(u64, u64)>,
}

impl PointMapping {
    /// Create an empty mapping.
    pub fn new() -> Self {
        Self::default()
    }

    /// Map a point's coordinates to variable IDs.
    pub fn add_point(&mut self, point_id: u64, x_var: u64, y_var: u64, z_var: u64) {
        self.x_vars.push((point_id, x_var));
        self.y_vars.push((point_id, y_var));
        self.z_vars.push((point_id, z_var));
    }

    /// Build a point from variable values.
    pub fn build_point(&self, point_id: u64, vars: &VariableSet) -> Option<Point3D> {
        let x = self
            .x_vars
            .iter()
            .find(|(pid, _)| *pid == point_id)
            .and_then(|(_, vid)| vars.get(*vid));
        let y = self
            .y_vars
            .iter()
            .find(|(pid, _)| *pid == point_id)
            .and_then(|(_, vid)| vars.get(*vid));
        let z = self
            .z_vars
            .iter()
            .find(|(pid, _)| *pid == point_id)
            .and_then(|(_, vid)| vars.get(*vid));
        match (x, y, z) {
            (Some(xv), Some(yv), Some(zv)) => Some(Point3D::new(xv.value, yv.value, zv.value)),
            _ => None,
        }
    }
}

/// The unified constraint system — variables, point mapping, and constraint graph.
#[derive(Debug, Clone, Default)]
pub struct ConstraintSystem {
    /// Variable store.
    pub variables: VariableSet,
    /// Point ↔ variable mapping.
    pub mapping: PointMapping,
    /// Constraint graph (edges reference variable IDs).
    pub graph: ConstraintGraph,
    /// Next available ID.
    next_id: u64,
}

impl ConstraintSystem {
    /// Create an empty constraint system.
    pub fn new() -> Self {
        Self {
            variables: VariableSet::new(),
            mapping: PointMapping::new(),
            graph: ConstraintGraph::new(),
            next_id: 1,
        }
    }

    /// Add a point with 3 scalar variables (x, y, z).
    pub fn add_point(&mut self, point_id: u64, x: f64, y: f64, z: f64) -> (u64, u64, u64) {
        let xv = self.next_id;
        self.next_id += 1;
        let yv = self.next_id;
        self.next_id += 1;
        let zv = self.next_id;
        self.next_id += 1;
        self.variables
            .insert(VariableValue::new(xv, core_math::scalar(x)));
        self.variables
            .insert(VariableValue::new(yv, core_math::scalar(y)));
        self.variables
            .insert(VariableValue::new(zv, core_math::scalar(z)));
        self.mapping.add_point(point_id, xv, yv, zv);
        (xv, yv, zv)
    }

    /// Add a fixed (anchored) point.
    pub fn add_fixed_point(&mut self, point_id: u64, x: f64, y: f64, z: f64) -> (u64, u64, u64) {
        let (xv, yv, zv) = self.add_point(point_id, x, y, z);
        // Add FixedPoint constraints for all coordinates.
        let anchor = Point3D::new(x, y, z);
        let pt = Point3D::new(x, y, z);
        let cid = self.next_id;
        self.next_id += 1;
        self.graph.insert(ConstraintEdge::new(
            cid,
            ConstraintKind::FixedPoint { point: pt, anchor },
            vec![xv, yv, zv],
        ));
        (xv, yv, zv)
    }

    /// Add a distance constraint between two points.
    pub fn add_distance(&mut self, p1: u64, p2: u64, target: f64) {
        let a = self.mapping.build_point(p1, &self.variables);
        let b = self.mapping.build_point(p2, &self.variables);
        if let (Some(a), Some(b)) = (a, b) {
            let cid = self.next_id;
            self.next_id += 1;
            self.graph.insert(ConstraintEdge::new(
                cid,
                ConstraintKind::Distance {
                    a,
                    b,
                    target: core_math::scalar(target),
                },
                vec![p1, p2],
            ));
        }
    }

    /// Get a point's current position.
    pub fn get_point(&self, point_id: u64) -> Option<Point3D> {
        self.mapping.build_point(point_id, &self.variables)
    }

    /// Number of points registered.
    pub fn point_count(&self) -> usize {
        self.mapping.x_vars.len()
    }

    /// Number of variables.
    pub fn var_count(&self) -> usize {
        self.variables.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constraint_system_point_mapping() {
        let mut sys = ConstraintSystem::new();
        sys.add_point(1, 0.0, 0.0, 0.0);
        sys.add_point(2, 5.0, 0.0, 0.0);
        assert_eq!(sys.point_count(), 2);
        assert_eq!(sys.var_count(), 6);
        let pt1 = sys.get_point(1).unwrap();
        assert!((pt1.x.value - 0.0).abs() < 1e-9);
    }

    #[test]
    fn constraint_system_fixed_point() {
        let mut sys = ConstraintSystem::new();
        sys.add_fixed_point(1, 2.0, 3.0, 0.0);
        assert_eq!(sys.point_count(), 1);
        assert_eq!(sys.graph.len(), 1);
    }
}
