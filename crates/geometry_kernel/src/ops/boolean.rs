//! Solid boolean operations (union, subtract, intersect).

use crate::shapes::Plane;
use crate::topology::{Edge, Face, Loop, Shell, Solid};
use core_math::{Tolerance, scalar};

use super::classification::{FaceSide, classify_face};
use super::split::split_face_by_plane;

/// Result of a boolean operation.
#[derive(Debug, Clone)]
pub struct BooleanResult {
    /// The resulting solid, or None if the operation produced empty geometry.
    pub solid: Option<Solid>,
    /// Number of faces in the result.
    pub face_count: usize,
    /// Whether the operation succeeded.
    pub success: bool,
}

/// Compute the union of two solids (A ∪ B).
pub fn boolean_union(a: &Solid, b: &Solid, tol: &Tolerance) -> BooleanResult {
    let mut id_gen = IdGenerator::new(a, b);
    let mut result_faces: Vec<Face> = Vec::new();
    for face in &a.shell.faces {
        process_boolean_face(face, b, true, &mut result_faces, &mut id_gen, tol);
    }
    for face in &b.shell.faces {
        process_boolean_face(face, a, true, &mut result_faces, &mut id_gen, tol);
    }
    build_boolean_result(result_faces, id_gen)
}

/// Compute the subtract of B from A (A − B).
pub fn boolean_subtract(a: &Solid, b: &Solid, tol: &Tolerance) -> BooleanResult {
    let mut id_gen = IdGenerator::new(a, b);
    let mut result_faces: Vec<Face> = Vec::new();
    for face in &a.shell.faces {
        process_boolean_face(face, b, true, &mut result_faces, &mut id_gen, tol);
    }
    for face in &b.shell.faces {
        let side = classify_face(face, a, tol);
        match side {
            FaceSide::Inside => {
                let inverted = invert_face(face, &mut id_gen);
                result_faces.push(inverted);
            }
            FaceSide::Straddling => {
                for b_face_plane in &a.shell.faces {
                    let split = split_face_by_plane(
                        face,
                        &b_face_plane.plane,
                        &mut id_gen.next_face(),
                        &mut id_gen.next_edge(),
                        &mut id_gen.next_vert(),
                        tol,
                    );
                    if let Some(neg) = split.negative {
                        let inverted = invert_face(&neg, &mut id_gen);
                        result_faces.push(inverted);
                    }
                }
            }
            _ => {}
        }
    }
    build_boolean_result(result_faces, id_gen)
}

/// Compute the intersection of two solids (A ∩ B).
pub fn boolean_intersect(a: &Solid, b: &Solid, tol: &Tolerance) -> BooleanResult {
    let mut id_gen = IdGenerator::new(a, b);
    let mut result_faces: Vec<Face> = Vec::new();
    for face in &a.shell.faces {
        process_boolean_face(face, b, false, &mut result_faces, &mut id_gen, tol);
    }
    for face in &b.shell.faces {
        process_boolean_face(face, a, false, &mut result_faces, &mut id_gen, tol);
    }
    build_boolean_result(result_faces, id_gen)
}

// ---------------------------------------------------------------------------
// Internal
// ---------------------------------------------------------------------------

fn process_boolean_face(
    face: &Face,
    other: &Solid,
    keep_outside: bool,
    result_faces: &mut Vec<Face>,
    id_gen: &mut IdGenerator,
    tol: &Tolerance,
) {
    let side = classify_face(face, other, tol);
    match side {
        FaceSide::Outside if keep_outside => {
            let mut f = face.clone();
            f.id = id_gen.next_face();
            result_faces.push(f);
        }
        FaceSide::Inside if !keep_outside => {
            let mut f = face.clone();
            f.id = id_gen.next_face();
            result_faces.push(f);
        }
        FaceSide::Straddling => {
            let mut current = vec![face.clone()];
            for other_face in &other.shell.faces {
                let mut next: Vec<Face> = Vec::new();
                for f in &current {
                    let split = split_face_by_plane(
                        f,
                        &other_face.plane,
                        &mut id_gen.next_face(),
                        &mut id_gen.next_edge(),
                        &mut id_gen.next_vert(),
                        tol,
                    );
                    if keep_outside {
                        if let Some(pos) = split.positive {
                            next.push(pos);
                        }
                        if let Some(neg) = split.negative {
                            next.push(neg);
                        }
                    } else {
                        if let Some(neg) = split.negative {
                            next.push(neg);
                        }
                        if let Some(pos) = split.positive {
                            next.push(pos);
                        }
                    }
                }
                current = next;
                if current.is_empty() {
                    break;
                }
            }
            for f in &current {
                let side2 = classify_face(f, other, tol);
                let keep = if keep_outside {
                    matches!(side2, FaceSide::Outside | FaceSide::On)
                } else {
                    matches!(side2, FaceSide::Inside | FaceSide::On)
                };
                if keep {
                    let mut kept = f.clone();
                    kept.id = id_gen.next_face();
                    result_faces.push(kept);
                }
            }
        }
        _ => {}
    }
}

fn invert_face(face: &Face, id_gen: &mut IdGenerator) -> Face {
    let rev_edges: Vec<Edge> = face
        .outer_loop
        .edges
        .iter()
        .rev()
        .map(|e| {
            let mut e2 = e.clone();
            std::mem::swap(&mut e2.start, &mut e2.end);
            e2
        })
        .collect();
    Face {
        id: id_gen.next_face(),
        outer_loop: Loop::new(0, rev_edges),
        holes: vec![],
        plane: Plane::from_normal_and_point(
            face.plane.normal.scale(scalar(-1.0)),
            &face.outer_loop.edges[0].start.position,
        ),
    }
}

fn build_boolean_result(faces: Vec<Face>, _id_gen: IdGenerator) -> BooleanResult {
    if faces.is_empty() {
        return BooleanResult {
            solid: None,
            face_count: 0,
            success: true,
        };
    }
    let face_count = faces.len();
    let shell = Shell::new(0, faces);
    let solid = Solid::new(0, shell);
    BooleanResult {
        solid: Some(solid),
        face_count,
        success: true,
    }
}

/// ID generator for keeping IDs unique across split faces.
pub struct IdGenerator {
    next_face: u64,
    next_edge: u64,
    next_vert: u64,
}

impl IdGenerator {
    /// Create a new ID generator from two solids.
    pub fn new(a: &Solid, b: &Solid) -> Self {
        let max_face = a
            .shell
            .faces
            .iter()
            .chain(b.shell.faces.iter())
            .map(|f| f.id)
            .max()
            .unwrap_or(0);
        Self {
            next_face: max_face + 1,
            next_edge: 10000,
            next_vert: 20000,
        }
    }
    /// Allocate next face ID.
    pub fn next_face(&mut self) -> u64 {
        let id = self.next_face;
        self.next_face += 1;
        id
    }
    /// Allocate next edge ID.
    pub fn next_edge(&mut self) -> u64 {
        let id = self.next_edge;
        self.next_edge += 1;
        id
    }
    /// Allocate next vertex ID.
    pub fn next_vert(&mut self) -> u64 {
        let id = self.next_vert;
        self.next_vert += 1;
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::{Edge as TEdge, Loop, Vertex};
    use core_math::{Point3D, Tolerance};

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    fn make_cube(id: u64, cx: f64, cy: f64, cz: f64, size: f64) -> Solid {
        let half = size * 0.5;
        let v = [
            Point3D::new(cx - half, cy - half, cz - half),
            Point3D::new(cx + half, cy - half, cz - half),
            Point3D::new(cx + half, cy + half, cz - half),
            Point3D::new(cx - half, cy + half, cz - half),
            Point3D::new(cx - half, cy - half, cz + half),
            Point3D::new(cx + half, cy - half, cz + half),
            Point3D::new(cx + half, cy + half, cz + half),
            Point3D::new(cx - half, cy + half, cz + half),
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
        Solid::new(id, Shell::new(0, faces))
    }

    #[test]
    fn boolean_union_disjoint_cubes() {
        let a = make_cube(0, 0.0, 0.0, 0.0, 2.0);
        let b = make_cube(1, 5.0, 0.0, 0.0, 2.0);
        let r = boolean_union(&a, &b, &tol());
        assert!(r.success);
        assert!(r.solid.is_some());
        assert!(r.face_count >= 8, "got {}", r.face_count);
    }
    #[test]
    fn boolean_intersect_contained() {
        let a = make_cube(0, 0.0, 0.0, 0.0, 2.0);
        let b = make_cube(1, 0.0, 0.0, 0.0, 5.0);
        let r = boolean_intersect(&a, &b, &tol());
        assert!(r.success);
        assert!(r.face_count >= 4, "got {}", r.face_count);
    }
    #[test]
    fn boolean_subtract_non_overlapping() {
        let a = make_cube(0, 0.0, 0.0, 0.0, 2.0);
        let b = make_cube(1, 5.0, 0.0, 0.0, 2.0);
        let r = boolean_subtract(&a, &b, &tol());
        assert!(r.success);
        assert!(r.solid.is_some());
    }
}
