//! Phase C topology validation and diagnostics.

use crate::topology::{Edge, Solid};
use core_math::Tolerance;
use std::collections::HashMap;

/// A directed edge key: (min_id, max_id).
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
struct DirectedEdgeKey {
    a: u64,
    b: u64,
}

/// Result of validating a solid.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the solid is topologically valid.
    pub valid: bool,
    /// Hard errors (watertightness violations, degenerate faces, etc.).
    pub issues: Vec<String>,
    /// Soft warnings (normal inconsistency, non-manifold edges).
    pub warnings: Vec<String>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            valid: true,
            issues: Vec::new(),
            warnings: Vec::new(),
        }
    }
}

impl ValidationResult {
    /// Create a new, initially-valid result.
    pub fn new() -> Self {
        Self::default()
    }
    fn add_issue(&mut self, msg: &str) {
        self.valid = false;
        self.issues.push(msg.into());
    }
    fn add_warning(&mut self, msg: &str) {
        self.warnings.push(msg.into());
    }
}

/// Validate a solid for topological correctness.
pub fn validate_solid(solid: &Solid, tol: &Tolerance) -> ValidationResult {
    let mut result = ValidationResult::new();
    if solid.shell.faces.is_empty() {
        result.add_issue("solid has no faces");
        return result;
    }
    check_face_planes(solid, &mut result, tol);
    check_loop_closedness(solid, &mut result, tol);
    check_edge_pairing(solid, &mut result);
    check_normal_outward_consistency(solid, &mut result);
    result
}

fn check_face_planes(solid: &Solid, result: &mut ValidationResult, tol: &Tolerance) {
    for face in &solid.shell.faces {
        let len = face.normal().length();
        if tol.nearly_equal(len.value, 0.0) {
            result.add_issue(&format!(
                "face {} has degenerate (zero-length) normal",
                face.id
            ));
        }
    }
}

fn check_loop_closedness(solid: &Solid, result: &mut ValidationResult, tol: &Tolerance) {
    for face in &solid.shell.faces {
        if face.outer_loop.edges.len() < 3 {
            result.add_issue(&format!("face {} outer loop has <3 edges", face.id));
        }
        if !face.outer_loop.is_closed(tol) {
            result.add_issue(&format!("face {} outer loop is not closed", face.id));
        }
        for (hi, hole) in face.holes.iter().enumerate() {
            if hole.edges.len() < 3 {
                result.add_issue(&format!("face {} hole {} has <3 edges", face.id, hi));
            }
            if !hole.is_closed(tol) {
                result.add_issue(&format!("face {} hole {} is not closed", face.id, hi));
            }
        }
    }
}

fn check_edge_pairing(solid: &Solid, result: &mut ValidationResult) {
    let mut edge_counts: HashMap<DirectedEdgeKey, usize> = HashMap::new();
    for face in &solid.shell.faces {
        for edge in &face.outer_loop.edges {
            count_edge(edge, &mut edge_counts);
        }
        for hole in &face.holes {
            for edge in &hole.edges {
                count_edge(edge, &mut edge_counts);
            }
        }
    }
    for (key, count) in &edge_counts {
        if *count > 2 {
            result.add_warning(&format!(
                "edge ({},{}) appears {} times (non-manifold)",
                key.a, key.b, count
            ));
        }
    }
    for (key, count) in &edge_counts {
        if *count == 1 {
            result.add_issue(&format!(
                "edge ({},{}) appears only once — shell is not watertight",
                key.a, key.b
            ));
        }
    }
}

fn count_edge(edge: &Edge, counts: &mut HashMap<DirectedEdgeKey, usize>) {
    // Use hashed positions for edge identity, since vertex IDs may differ
    // across faces even when the geometric edge is shared.
    let a = hash_point(&edge.start.position);
    let b = hash_point(&edge.end.position);
    let key = DirectedEdgeKey {
        a: a.min(b),
        b: a.max(b),
    };
    *counts.entry(key).or_insert(0) += 1;
}

/// Hash a point's coordinates into a u64 for fast edge lookup.
fn hash_point(pt: &core_math::Point3D) -> u64 {
    let x = (pt.x.value * 1e6).round() as i64 as u64;
    let y = (pt.y.value * 1e6).round() as i64 as u64;
    let z = (pt.z.value * 1e6).round() as i64 as u64;
    x.wrapping_mul(1_000_003)
        .wrapping_add(y.wrapping_mul(1_000_009))
        .wrapping_add(z.wrapping_mul(1_000_013))
}

fn check_normal_outward_consistency(solid: &Solid, result: &mut ValidationResult) {
    use core_math::Point3D;
    let mut sum_x = 0.0f64;
    let mut sum_y = 0.0f64;
    let mut sum_z = 0.0f64;
    let mut count = 0usize;
    for face in &solid.shell.faces {
        for edge in &face.outer_loop.edges {
            sum_x += edge.start.position.x.value;
            sum_y += edge.start.position.y.value;
            sum_z += edge.start.position.z.value;
            count += 1;
        }
    }
    if count == 0 {
        return;
    }
    let centroid = Point3D::new(
        sum_x / count as f64,
        sum_y / count as f64,
        sum_z / count as f64,
    );
    for face in &solid.shell.faces {
        let sd = face.plane.signed_distance(&centroid).value;
        if sd > 0.0 {
            result.add_warning(&format!(
                "face {} normal appears to point inward (centroid signed distance = {:.3e})",
                face.id, sd
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::{Edge as TEdge, Face, Loop, Shell, Vertex};
    use core_math::Point3D;

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    fn make_valid_cube() -> Solid {
        let v = [
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(0.0, 0.0, 1.0),
            Point3D::new(1.0, 0.0, 1.0),
            Point3D::new(1.0, 1.0, 1.0),
            Point3D::new(0.0, 1.0, 1.0),
        ];
        let verts: Vec<Vertex> = v
            .iter()
            .enumerate()
            .map(|(i, p)| Vertex::new(i as u64, *p))
            .collect();
        fn quad(v0: &Vertex, v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Loop {
            Loop::new(
                0,
                vec![
                    TEdge::new(0, v0.clone(), v1.clone()),
                    TEdge::new(1, v1.clone(), v2.clone()),
                    TEdge::new(2, v2.clone(), v3.clone()),
                    TEdge::new(3, v3.clone(), v0.clone()),
                ],
            )
        }
        let faces = vec![
            Face::new(0, quad(&verts[0], &verts[1], &verts[2], &verts[3]), vec![]),
            Face::new(1, quad(&verts[4], &verts[5], &verts[6], &verts[7]), vec![]),
            Face::new(2, quad(&verts[0], &verts[1], &verts[5], &verts[4]), vec![]),
            Face::new(3, quad(&verts[2], &verts[3], &verts[7], &verts[6]), vec![]),
            Face::new(4, quad(&verts[0], &verts[3], &verts[7], &verts[4]), vec![]),
            Face::new(5, quad(&verts[1], &verts[2], &verts[6], &verts[5]), vec![]),
        ];
        Solid::new(0, Shell::new(0, faces))
    }

    #[test]
    fn valid_cube_passes() {
        let cube = make_valid_cube();
        let r = validate_solid(&cube, &tol());
        assert!(r.valid, "issues: {:?}", r.issues);
    }
    #[test]
    fn empty_solid_fails() {
        let r = validate_solid(&Solid::new(0, Shell::new(0, vec![])), &tol());
        assert!(!r.valid);
    }
    #[test]
    fn open_shell_detected() {
        let v = [
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(1.0, 0.0, 0.0),
            Point3D::new(1.0, 1.0, 0.0),
            Point3D::new(0.0, 1.0, 0.0),
            Point3D::new(0.0, 0.0, 1.0),
            Point3D::new(1.0, 0.0, 1.0),
            Point3D::new(1.0, 1.0, 1.0),
            Point3D::new(0.0, 1.0, 1.0),
        ];
        let verts: Vec<Vertex> = v
            .iter()
            .enumerate()
            .map(|(i, p)| Vertex::new(i as u64, *p))
            .collect();
        fn quad(v0: &Vertex, v1: &Vertex, v2: &Vertex, v3: &Vertex) -> Loop {
            Loop::new(
                0,
                vec![
                    TEdge::new(0, v0.clone(), v1.clone()),
                    TEdge::new(1, v1.clone(), v2.clone()),
                    TEdge::new(2, v2.clone(), v3.clone()),
                    TEdge::new(3, v3.clone(), v0.clone()),
                ],
            )
        }
        let faces = vec![
            Face::new(0, quad(&verts[0], &verts[1], &verts[2], &verts[3]), vec![]),
            Face::new(1, quad(&verts[4], &verts[5], &verts[6], &verts[7]), vec![]),
            Face::new(2, quad(&verts[0], &verts[1], &verts[5], &verts[4]), vec![]),
            Face::new(3, quad(&verts[2], &verts[3], &verts[7], &verts[6]), vec![]),
            Face::new(4, quad(&verts[0], &verts[3], &verts[7], &verts[4]), vec![]),
        ];
        let r = validate_solid(&Solid::new(0, Shell::new(0, faces)), &tol());
        assert!(!r.valid);
        assert!(!r.issues.is_empty());
    }
}
