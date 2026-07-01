//! Constraint system — Phase C: bridges VariableSet ↔ constraint graph points.
#![allow(missing_docs)]

use crate::constraint::ConstraintKind;
use crate::graph::{ConstraintEdge, ConstraintGraph};
use crate::variable::{VariableSet, VariableValue};
use core_math::scalar;
use geometry_kernel::Point3D;

#[derive(Debug, Clone, Default)]
pub struct PointMapping {
    pub x_vars: Vec<(u64, u64)>,
    pub y_vars: Vec<(u64, u64)>,
    pub z_vars: Vec<(u64, u64)>,
}

impl PointMapping {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_point(&mut self, point_id: u64, x_var: u64, y_var: u64, z_var: u64) {
        self.x_vars.push((point_id, x_var));
        self.y_vars.push((point_id, y_var));
        self.z_vars.push((point_id, z_var));
    }
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

#[derive(Debug, Clone)]
pub struct ConstraintSystem {
    pub variables: VariableSet,
    pub mapping: PointMapping,
    pub graph: ConstraintGraph,
    next_id: u64,
}

impl Default for ConstraintSystem {
    fn default() -> Self {
        Self {
            variables: VariableSet::new(),
            mapping: PointMapping::new(),
            graph: ConstraintGraph::new(),
            next_id: 1,
        }
    }
}

impl ConstraintSystem {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_point(&mut self, point_id: u64, x: f64, y: f64, z: f64) -> (u64, u64, u64) {
        let xv = self.next_id;
        self.next_id += 1;
        let yv = self.next_id;
        self.next_id += 1;
        let zv = self.next_id;
        self.next_id += 1;
        self.variables.insert(VariableValue::new(xv, scalar(x)));
        self.variables.insert(VariableValue::new(yv, scalar(y)));
        self.variables.insert(VariableValue::new(zv, scalar(z)));
        self.mapping.add_point(point_id, xv, yv, zv);
        (xv, yv, zv)
    }
    pub fn add_fixed_point(&mut self, point_id: u64, x: f64, y: f64, z: f64) -> (u64, u64, u64) {
        let (xv, yv, zv) = self.add_point(point_id, x, y, z);
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
                    target: scalar(target),
                },
                vec![p1, p2],
            ));
        }
    }
    pub fn add_horizontal(&mut self, p1: u64, p2: u64) {
        let a = self.mapping.build_point(p1, &self.variables);
        let b = self.mapping.build_point(p2, &self.variables);
        if let (Some(a), Some(b)) = (a, b) {
            let cid = self.next_id;
            self.next_id += 1;
            self.graph.insert(ConstraintEdge::new(
                cid,
                ConstraintKind::Horizontal { a, b },
                vec![p1, p2],
            ));
        }
    }
    pub fn add_vertical(&mut self, p1: u64, p2: u64) {
        let a = self.mapping.build_point(p1, &self.variables);
        let b = self.mapping.build_point(p2, &self.variables);
        if let (Some(a), Some(b)) = (a, b) {
            let cid = self.next_id;
            self.next_id += 1;
            self.graph.insert(ConstraintEdge::new(
                cid,
                ConstraintKind::Vertical { a, b },
                vec![p1, p2],
            ));
        }
    }
    pub fn add_coincident(&mut self, p1: u64, p2: u64) {
        let a = self.mapping.build_point(p1, &self.variables);
        let b = self.mapping.build_point(p2, &self.variables);
        if let (Some(a), Some(b)) = (a, b) {
            let cid = self.next_id;
            self.next_id += 1;
            self.graph.insert(ConstraintEdge::new(
                cid,
                ConstraintKind::Coincident { a, b },
                vec![p1, p2],
            ));
        }
    }
    pub fn get_point(&self, point_id: u64) -> Option<Point3D> {
        self.mapping.build_point(point_id, &self.variables)
    }

    pub fn refresh_constraints_from_variables(&mut self) {
        let n = self.graph.len();
        let mut updated_kinds: Vec<ConstraintKind> = Vec::with_capacity(n);
        for i in 0..n {
            let edge = &self.graph.edges[i];
            let vid0 = edge.variable_ids.first().copied().unwrap_or(0);
            let vid1 = edge.variable_ids.get(1).copied().unwrap_or(0);
            let vid2 = edge.variable_ids.get(2).copied().unwrap_or(0);
            let kind = &edge.kind;
            let new_kind = match kind {
                ConstraintKind::Distance { target, .. } => {
                    let a = self.get_point(vid0).unwrap_or(Point3D::ORIGIN);
                    let b = self.get_point(vid1).unwrap_or(Point3D::ORIGIN);
                    ConstraintKind::Distance {
                        a,
                        b,
                        target: *target,
                    }
                }
                ConstraintKind::Horizontal { .. } => {
                    let a = self.get_point(vid0).unwrap_or(Point3D::ORIGIN);
                    let b = self.get_point(vid1).unwrap_or(Point3D::ORIGIN);
                    ConstraintKind::Horizontal { a, b }
                }
                ConstraintKind::Vertical { .. } => {
                    let a = self.get_point(vid0).unwrap_or(Point3D::ORIGIN);
                    let b = self.get_point(vid1).unwrap_or(Point3D::ORIGIN);
                    ConstraintKind::Vertical { a, b }
                }
                ConstraintKind::Coincident { .. } => {
                    let a = self.get_point(vid0).unwrap_or(Point3D::ORIGIN);
                    let b = self.get_point(vid1).unwrap_or(Point3D::ORIGIN);
                    ConstraintKind::Coincident { a, b }
                }
                ConstraintKind::Collinear { a, b, .. } => {
                    let c = self.get_point(vid2).unwrap_or(Point3D::ORIGIN);
                    ConstraintKind::Collinear { a: *a, b: *b, c }
                }
                ConstraintKind::HorizontalDistance { target, .. } => {
                    let a = self.get_point(vid0).unwrap_or(Point3D::ORIGIN);
                    let b = self.get_point(vid1).unwrap_or(Point3D::ORIGIN);
                    ConstraintKind::HorizontalDistance {
                        a,
                        b,
                        target: *target,
                    }
                }
                ConstraintKind::VerticalDistance { target, .. } => {
                    let a = self.get_point(vid0).unwrap_or(Point3D::ORIGIN);
                    let b = self.get_point(vid1).unwrap_or(Point3D::ORIGIN);
                    ConstraintKind::VerticalDistance {
                        a,
                        b,
                        target: *target,
                    }
                }
                _ => kind.clone(),
            };
            updated_kinds.push(new_kind);
        }
        for (i, kind) in updated_kinds.into_iter().enumerate() {
            self.graph.edges[i].kind = kind;
        }
    }

    pub fn residual_vector(&self) -> Vec<f64> {
        self.graph.iter().map(|e| e.kind.residual().value).collect()
    }
    pub fn residual_norm(&self) -> f64 {
        self.residual_vector()
            .iter()
            .map(|r| r * r)
            .sum::<f64>()
            .sqrt()
    }
    pub fn point_count(&self) -> usize {
        self.mapping.x_vars.len()
    }
    pub fn var_count(&self) -> usize {
        self.variables.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn point_mapping() {
        let mut s = ConstraintSystem::new();
        s.add_point(1, 0.0, 0.0, 0.0);
        s.add_point(2, 5.0, 0.0, 0.0);
        assert_eq!(s.point_count(), 2);
        assert_eq!(s.var_count(), 6);
        assert!((s.get_point(1).unwrap().x.value - 0.0).abs() < 1e-9);
    }
    #[test]
    fn fixed_point() {
        let mut s = ConstraintSystem::new();
        s.add_fixed_point(1, 2.0, 3.0, 0.0);
        assert_eq!(s.point_count(), 1);
        assert_eq!(s.graph.len(), 1);
    }
    #[test]
    fn refresh_updates() {
        let mut s = ConstraintSystem::new();
        s.add_point(1, 0.0, 0.0, 0.0);
        s.add_point(2, 5.0, 0.0, 0.0);
        s.add_distance(1, 2, 3.0);
        // Variable 4 is pt2.x — set it to 3.0 to satisfy distance=3
        s.variables.get_mut(4).unwrap().value = scalar(3.0);
        s.refresh_constraints_from_variables();
        assert!(s.residual_vector()[0] < 1e-6);
    }
    #[test]
    fn setup() {
        let mut s = ConstraintSystem::new();
        s.add_fixed_point(1, 0.0, 0.0, 0.0);
        s.add_point(2, 5.0, 0.0, 0.0);
        s.add_distance(1, 2, 3.0);
        assert_eq!(s.point_count(), 2);
        assert_eq!(s.graph.len(), 2);
    }
}
