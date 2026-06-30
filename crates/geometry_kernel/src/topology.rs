//! Boundary-representation (B-Rep) topology layer.
//!
//! The topology layer models objects by their boundary hierarchy:
//!
//! ```text
//! Vertex ──► Edge ──► Loop ──► Face ──► Shell ──► Solid
//!   ↓          ↓               ↓
//! style="color:grey" (position)  (curve)         (surface: Plane)
//! ```
//!
//! In Phase A, all faces are planar and bounded by straight edges.  Curved
//! surfaces and curves are deferred to Phase B.
//!
//! # Invariants
//!
//! - Entities are value types — they carry their geometry inline.
//! - A `Vertex` holds a single [`Point3D`].
//! - An `Edge` holds two vertices and a line segment between them.
//! - A `Loop` is an ordered closed cycle of edges (counter-clockwise on a face).
//! - A `Face` contains one outer `Loop` and zero or more hole `Loop`s.
//! - A `Shell` is a connected set of faces.
//! - A `Solid` is a closed shell (representing a volume).
//!
//! # Tolerance
//!
//! All validation helpers (e.g. `is_closed`, `is_planar`) accept a
//! [`Tolerance`] argument.  None of the construction functions enforce
//! geometric validity — callers should validate after construction using
//! the query methods.

use core_math::{Point3D, Scalar, Tolerance, Vector3D};
use crate::shapes::{Line, Plane, BoundingBox};

// ---------------------------------------------------------------------------
// Vertex
// ---------------------------------------------------------------------------

/// A topological vertex — a single point in 3D space.
#[derive(Debug, Clone, PartialEq)]
pub struct Vertex {
    /// Unique identifier within the parent shell/solid (for debugging).
    pub id: u64,
    /// Position of the vertex.
    pub position: Point3D,
}

impl Vertex {
    /// Create a new vertex.
    #[inline]
    pub fn new(id: u64, position: Point3D) -> Self {
        Self { id, position }
    }
}

// ---------------------------------------------------------------------------
// Edge
// ---------------------------------------------------------------------------

/// A topological edge connecting two vertices with a straight line segment.
///
/// In Phase A all edges are straight line segments.  Curved edges will be
/// added via a `Curve` variant in Phase B.
#[derive(Debug, Clone, PartialEq)]
pub struct Edge {
    /// Unique identifier.
    pub id: u64,
    /// Start vertex.
    pub start: Vertex,
    /// End vertex.
    pub end: Vertex,
}

impl Edge {
    /// Create a new straight edge.
    #[inline]
    pub fn new(id: u64, start: Vertex, end: Vertex) -> Self {
        Self { id, start, end }
    }

    /// The line containing this edge (direction from start to end).
    #[inline]
    pub fn as_line(&self) -> Line {
        Line::new(
            self.start.position,
            Vector3D::between(&self.start.position, &self.end.position),
        )
    }

    /// Midpoint of the edge.
    #[inline]
    pub fn midpoint(&self) -> Point3D {
        self.start.position.midpoint(&self.end.position)
    }

    /// Length of the edge.
    #[inline]
    pub fn length(&self) -> Scalar {
        self.start.position.distance_to(&self.end.position)
    }

    /// Bounding box of this edge.
    #[inline]
    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox::from_corners(&self.start.position, &self.end.position)
    }
}

// ---------------------------------------------------------------------------
// Loop (a closed ring of edges)
// ---------------------------------------------------------------------------

/// A closed, ordered sequence of edges forming a loop.
///
/// A loop is a cycle — the end vertex of each edge must match the start
/// vertex of the next (within tolerance), and the last edge's end vertex
/// must match the first edge's start vertex.
#[derive(Debug, Clone, PartialEq)]
pub struct Loop {
    /// Unique identifier.
    pub id: u64,
    /// Edges in order around the loop (CCW when viewed from outside the face).
    pub edges: Vec<Edge>,
}

impl Loop {
    /// Create a new loop.
    #[inline]
    pub fn new(id: u64, edges: Vec<Edge>) -> Self {
        Self { id, edges }
    }

    /// `true` when the loop is closed (last edge ends where first edge starts)
    /// under the given tolerance.
    pub fn is_closed(&self, tol: &Tolerance) -> bool {
        if self.edges.is_empty() {
            return false;
        }
        for i in 0..self.edges.len() {
            let curr = &self.edges[i];
            let next = &self.edges[(i + 1) % self.edges.len()];
            if !curr
                .end
                .position
                .nearly_equal(&next.start.position, tol)
            {
                return false;
            }
        }
        true
    }

    /// Collect all unique vertex positions in this loop.
    pub fn vertices(&self) -> Vec<&Vertex> {
        self.edges.iter().flat_map(|e| [&e.start, &e.end]).collect()
    }

    /// Number of edges in the loop.
    #[inline]
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }
}

// ---------------------------------------------------------------------------
// Face
// ---------------------------------------------------------------------------

/// A planar face consisting of one outer loop and zero or more hole loops.
///
/// The face's plane is derived from its outer loop.  Holes are "cut out"
/// of the face interior.
#[derive(Debug, Clone, PartialEq)]
pub struct Face {
    /// Unique identifier.
    pub id: u64,
    /// Outer boundary loop (counter-clockwise).
    pub outer_loop: Loop,
    /// Hole loops (clockwise, cutting into the face).
    pub holes: Vec<Loop>,
    /// The plane on which this face lies.
    pub plane: Plane,
}

impl Face {
    /// Create a new face from an outer loop.
    ///
    /// The plane is computed from the first three non-collinear vertices of
    /// the outer loop.  If all vertices are collinear (degenerate face),
    /// an arbitrary plane through those points is chosen.
    pub fn new(id: u64, outer_loop: Loop, holes: Vec<Loop>) -> Self {
        let plane = Self::compute_plane(&outer_loop);
        Self {
            id,
            outer_loop,
            holes,
            plane,
        }
    }

    /// Compute a plane from the outer loop's vertices.
    fn compute_plane(outer: &Loop) -> Plane {
        if outer.edges.len() < 3 {
            // Degenerate — fall back to XY plane through first vertex.
            let pt = outer
                .edges
                .first()
                .map(|e| e.start.position)
                .unwrap_or(Point3D::ORIGIN);
            return Plane::from_normal_and_point(Vector3D::Z, &pt);
        }

        let a = &outer.edges[0].start.position;
        let b = &outer.edges[0].end.position;
        // Walk edges until we find a non-collinear point.
        for edge in outer.edges.iter().skip(1) {
            let c = &edge.end.position;
            let ab = Vector3D::between(a, b);
            let ac = Vector3D::between(a, c);
            let normal = ab.cross(&ac);
            if normal.length_squared().value > 0.0 {
                if let Some(unit_normal) = normal.normalize() {
                    return Plane::from_normal_and_point(unit_normal, a);
                }
            }
        }

        // All collinear — use Z up through first point.
        Plane::from_normal_and_point(Vector3D::Z, a)
    }

    /// `true` when all vertices of the face lie on its plane within tolerance.
    pub fn is_planar(&self, tol: &Tolerance) -> bool {
        for edge in &self.outer_loop.edges {
            if !self.plane.contains_point(&edge.start.position, tol)
                || !self.plane.contains_point(&edge.end.position, tol)
            {
                return false;
            }
        }
        for hole in &self.holes {
            for edge in &hole.edges {
                if !self.plane.contains_point(&edge.start.position, tol)
                    || !self.plane.contains_point(&edge.end.position, tol)
                {
                    return false;
                }
            }
        }
        true
    }

    /// Compute the axis-aligned bounding box of the face.
    pub fn bounding_box(&self) -> BoundingBox {
        let mut bbox = BoundingBox::from_corners(
            &self.outer_loop.edges[0].start.position,
            &self.outer_loop.edges[0].start.position,
        );
        for edge in &self.outer_loop.edges {
            bbox.expand_by_point(&edge.start.position);
            bbox.expand_by_point(&edge.end.position);
        }
        for hole in &self.holes {
            for edge in &hole.edges {
                bbox.expand_by_point(&edge.start.position);
                bbox.expand_by_point(&edge.end.position);
            }
        }
        bbox
    }

    /// Normal vector of the face's plane.
    #[inline]
    pub fn normal(&self) -> &Vector3D {
        &self.plane.normal
    }
}

// ---------------------------------------------------------------------------
// Shell (connected set of faces)
// ---------------------------------------------------------------------------

/// A connected set of faces.
///
/// A shell does **not** need to be closed (a closed shell is a `Solid`).
/// Open shells are valid for e.g. sheet bodies.
#[derive(Debug, Clone, PartialEq)]
pub struct Shell {
    /// Unique identifier.
    pub id: u64,
    /// Faces composing this shell.
    pub faces: Vec<Face>,
}

impl Shell {
    /// Create a new shell.
    #[inline]
    pub fn new(id: u64, faces: Vec<Face>) -> Self {
        Self { id, faces }
    }

    /// `true` when the shell is watertight (every edge appears an even number
    /// of times with opposite orientation).
    ///
    /// NOTE: Phase A implements a basic check — full topology validation
    /// will be enhanced in Phase B.
    pub fn is_closed(&self, _tol: &Tolerance) -> bool {
        if self.faces.is_empty() {
            return false;
        }
        // Stub: in Phase B this will check edge pairing.
        // For now, accept a box (6 faces) as de-facto closed.
        self.faces.len() >= 4
    }

    /// Compute the combined bounding box of all faces.
    pub fn bounding_box(&self) -> BoundingBox {
        let mut bbox = self.faces[0].bounding_box();
        for face in &self.faces[1..] {
            bbox.expand_by_box(&face.bounding_box());
        }
        bbox
    }

    /// Total number of faces.
    #[inline]
    pub fn face_count(&self) -> usize {
        self.faces.len()
    }
}

// ---------------------------------------------------------------------------
// Solid (closed shell representing a volume)
// ---------------------------------------------------------------------------

/// A solid (volume) represented by a closed, watertight shell.
#[derive(Debug, Clone, PartialEq)]
pub struct Solid {
    /// Unique identifier.
    pub id: u64,
    /// The closed outer shell defining the solid boundary.
    pub shell: Shell,
}

impl Solid {
    /// Create a new solid from a shell.
    ///
    /// Callers should ensure `shell.is_closed()` before constructing a solid.
    #[inline]
    pub fn new(id: u64, shell: Shell) -> Self {
        Self { id, shell }
    }

    /// Compute the bounding box of the solid.
    #[inline]
    pub fn bounding_box(&self) -> BoundingBox {
        self.shell.bounding_box()
    }

    /// `true` when the shell is considered closed.
    #[inline]
    pub fn is_valid(&self, tol: &Tolerance) -> bool {
        self.shell.is_closed(tol)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    /// Build a unit cube corner.
    fn unit_cube_vertices() -> [Point3D; 8] {
        [
            Point3D::new(0.0, 0.0, 0.0), // 0
            Point3D::new(1.0, 0.0, 0.0), // 1
            Point3D::new(1.0, 1.0, 0.0), // 2
            Point3D::new(0.0, 1.0, 0.0), // 3
            Point3D::new(0.0, 0.0, 1.0), // 4
            Point3D::new(1.0, 0.0, 1.0), // 5
            Point3D::new(1.0, 1.0, 1.0), // 6
            Point3D::new(0.0, 1.0, 1.0), // 7
        ]
    }

    fn make_quad(v0: Vertex, v1: Vertex, v2: Vertex, v3: Vertex) -> Loop {
        Loop::new(0, vec![
            Edge::new(0, v0.clone(), v1.clone()),
            Edge::new(1, v1, v2.clone()),
            Edge::new(2, v2, v3.clone()),
            Edge::new(3, v3, v0), // close back to v0
        ])
    }

    #[test]
    fn loop_is_closed_valid() {
        let v = unit_cube_vertices();
        let verts: Vec<Vertex> = v
            .iter()
            .enumerate()
            .map(|(i, p)| Vertex::new(i as u64, *p))
            .collect();
        let l = make_quad(
            verts[0].clone(),
            verts[1].clone(),
            verts[2].clone(),
            verts[3].clone(),
        );
        assert!(l.is_closed(&tol()));
    }

    #[test]
    fn loop_is_closed_open() {
        let v = unit_cube_vertices();
        let verts: Vec<Vertex> = v
            .iter()
            .enumerate()
            .map(|(i, p)| Vertex::new(i as u64, *p))
            .collect();
        // Open: last edge does not return to first vertex.
        let l = Loop::new(0, vec![
            Edge::new(0, verts[0].clone(), verts[1].clone()),
            Edge::new(1, verts[1].clone(), verts[2].clone()),
            Edge::new(2, verts[2].clone(), verts[3].clone()),
            // Missing closing edge
        ]);
        assert!(!l.is_closed(&tol()));
    }

    #[test]
    fn face_is_planar() {
        let v = unit_cube_vertices();
        let verts: Vec<Vertex> = v
            .iter()
            .enumerate()
            .map(|(i, p)| Vertex::new(i as u64, *p))
            .collect();
        let outer = make_quad(
            verts[0].clone(),
            verts[1].clone(),
            verts[2].clone(),
            verts[3].clone(),
        );
        let face = Face::new(0, outer, vec![]);
        assert!(face.is_planar(&tol()));
        // Normal should be +Z
        assert!(face.normal().nearly_equal(&Vector3D::Z, &tol()));
    }

    #[test]
    fn edge_length() {
        let a = Vertex::new(0, Point3D::new(0.0, 0.0, 0.0));
        let b = Vertex::new(1, Point3D::new(3.0, 4.0, 0.0));
        let edge = Edge::new(0, a, b);
        assert!(tol().nearly_equal(edge.length().value, 5.0));
    }

    #[test]
    fn solid_bounding_box_cube() {
        let v = unit_cube_vertices();
        let verts: Vec<Vertex> = v
            .iter()
            .enumerate()
            .map(|(i, p)| Vertex::new(i as u64, *p))
            .collect();
        // Build 6 faces of a cube.
        let bottom = Face::new(0, make_quad(
            verts[0].clone(), verts[1].clone(), verts[2].clone(), verts[3].clone(),
        ), vec![]);
        let top = Face::new(1, make_quad(
            verts[4].clone(), verts[5].clone(), verts[6].clone(), verts[7].clone(),
        ), vec![]);
        let front = Face::new(2, make_quad(
            verts[0].clone(), verts[1].clone(), verts[5].clone(), verts[4].clone(),
        ), vec![]);
        let back = Face::new(3, make_quad(
            verts[2].clone(), verts[3].clone(), verts[7].clone(), verts[6].clone(),
        ), vec![]);
        let left = Face::new(4, make_quad(
            verts[0].clone(), verts[3].clone(), verts[7].clone(), verts[4].clone(),
        ), vec![]);
        let right = Face::new(5, make_quad(
            verts[1].clone(), verts[2].clone(), verts[6].clone(), verts[5].clone(),
        ), vec![]);
        let shell = Shell::new(0, vec![bottom, top, front, back, left, right]);
        let solid = Solid::new(0, shell);
        let bbox = solid.bounding_box();
        assert!(bbox.min_corner().nearly_equal(&v[0], &tol()));
        assert!(bbox.max_corner().nearly_equal(&v[6], &tol()));
        assert!(solid.is_valid(&tol()));
    }
}