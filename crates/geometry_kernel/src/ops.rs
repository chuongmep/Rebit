//! Geometry operations — Phase B: boolean operations (union, subtract, intersect).
//!
//! # Architecture
//!
//! Boolean operations follow the standard B-Rep approach:
//!
//! 1. **Classify** each face of solid A against solid B (inside/outside/on).
//! 2. **Split** faces that straddle the boundary using face-plane intersections.
//! 3. **Select** faces based on the operation type.
//! 4. **Stitch** selected faces into a new closed shell/solid.
//!
//! # Phase B limitations
//!
//! - Operates on convex solids with planar faces.
//! - Face splitting is simplified: splits a face by a plane, returning
//!   two faces (inside-half and outside-half).
//! - Non-manifold and coplanar-face cases are deferred to Phase C.
//! - No curved-surface intersection.

use core_math::{Point3D, Tolerance, Vector3D, scalar};

use crate::shapes::{Line, Plane};
use crate::topology::{Edge, Face, Loop, Shell, Solid, Vertex};

// ---------------------------------------------------------------------------
// Face classification
// ---------------------------------------------------------------------------

/// Classification of a face relative to a solid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceSide {
    /// All vertices lie strictly inside the solid.
    Inside,
    /// All vertices lie strictly outside the solid.
    Outside,
    /// Face straddles the boundary (some vertices inside, some outside).
    Straddling,
    /// All vertices lie on the boundary (coplanar — deferred).
    On,
}

/// Classify a face relative to a reference solid.
///
/// Evaluates every face of the solid as an implicit half-space (pointing
/// inward via the face normal), and checks vertex distances.
pub fn classify_face(face: &Face, solid: &Solid, tol: &Tolerance) -> FaceSide {
    let mut inside_count = 0usize;
    let mut outside_count = 0usize;

    for edge in &face.outer_loop.edges {
        let pt = &edge.start.position;
        let loc = point_vs_solid(pt, solid, tol);
        match loc {
            PointLocation::Inside => inside_count += 1,
            PointLocation::Outside => outside_count += 1,
            PointLocation::OnBoundary => { /* count as neither for Phase B */ }
        }
    }

    if inside_count > 0 && outside_count > 0 {
        FaceSide::Straddling
    } else if inside_count > 0 {
        FaceSide::Inside
    } else if outside_count > 0 {
        FaceSide::Outside
    } else {
        FaceSide::On
    }
}

/// Location of a point relative to a solid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointLocation {
    /// Point is strictly inside the solid.
    Inside,
    /// Point is strictly outside the solid.
    Outside,
    /// Point lies on the boundary (on a face).
    OnBoundary,
}

/// Classify a point relative to a solid using ray-casting.
///
/// Shoots a ray along +X and counts face intersections.  Odd count ⇒ inside.
/// This is robust regardless of face normal orientation.
fn point_vs_solid(point: &Point3D, solid: &Solid, tol: &Tolerance) -> PointLocation {
    let ray_origin = *point;
    let ray_dir = Vector3D::X;

    let mut intersection_count = 0usize;
    for face in &solid.shell.faces {
        let sd = face.plane.signed_distance(&ray_origin).value;
        let nd = face.normal().dot(&ray_dir).value;

        if nd.abs() < 1e-15 {
            // Ray is parallel to face.
            if tol.nearly_equal(sd, 0.0) {
                // Point lies on face plane — check if within polygon.
                let proj = face.plane.project_point(&ray_origin);
                if point_in_face_polygon(&proj, face, tol) {
                    return PointLocation::OnBoundary;
                }
            }
            continue;
        }

        let t = -sd / nd;
        if t < -tol.absolute {
            continue; // intersection behind ray origin.
        }

        let hit = Point3D::new(
            ray_origin.x.value + t * ray_dir.x.value,
            ray_origin.y.value + t * ray_dir.y.value,
            ray_origin.z.value + t * ray_dir.z.value,
        );

        // Check if hit is within the face polygon (project onto face plane).
        if point_in_face_polygon(&hit, face, tol) {
            if t < tol.absolute {
                // Hit within tolerance of origin — on boundary.
                return PointLocation::OnBoundary;
            }
            intersection_count += 1;
        }
    }

    if intersection_count % 2 == 1 {
        PointLocation::Inside
    } else {
        PointLocation::Outside
    }
}

/// Select the dominant 2D projection plane based on face normal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectionPlane {
    XY,
    XZ,
    YZ,
}

/// Project a point onto the chosen 2D coordinate plane.
fn project_onto(plane: ProjectionPlane, pt: &Point3D) -> (f64, f64) {
    match plane {
        ProjectionPlane::XY => (pt.x.value, pt.y.value),
        ProjectionPlane::XZ => (pt.x.value, pt.z.value),
        ProjectionPlane::YZ => (pt.y.value, pt.z.value),
    }
}

/// Point-in-polygon test using ray-casting onto the face's dominant 2D plane.
///
/// Projects vertices and test point onto the two axes orthogonal to the face
/// normal, then performs a standard 2D ray-cast (even-odd rule).
fn point_in_face_polygon(pt: &Point3D, face: &Face, _tol: &Tolerance) -> bool {
    let edges = &face.outer_loop.edges;
    if edges.len() < 3 {
        return false;
    }

    let normal = face.normal();
    let (nx, ny, nz) = (
        normal.x.value.abs(),
        normal.y.value.abs(),
        normal.z.value.abs(),
    );

    let proj = if nz >= nx && nz >= ny {
        ProjectionPlane::XY
    } else if ny >= nx && ny >= nz {
        ProjectionPlane::XZ
    } else {
        ProjectionPlane::YZ
    };

    let n = edges.len();
    let mut inside = false;
    for i in 0..n {
        let v0 = &edges[i].start.position;
        let v1 = &edges[(i + 1) % n].start.position;
        let (u0, v0_val) = project_onto(proj, v0);
        let (u1, v1_val) = project_onto(proj, v1);
        let (pu, pv) = project_onto(proj, pt);
        if (v0_val > pv) != (v1_val > pv) {
            let u_intersect = u0 + (pv - v0_val) * (u1 - u0) / (v1_val - v0_val);
            if pu < u_intersect {
                inside = !inside;
            }
        }
    }
    inside
}

// ---------------------------------------------------------------------------
// Face-plane splitting
// ---------------------------------------------------------------------------

/// Result of splitting a face by a plane.
#[derive(Debug, Clone)]
pub struct SplitFaceResult {
    /// Portion of the face on the negative side of the plane (inside).
    pub negative: Option<Face>,
    /// Portion of the face on the positive side of the plane (outside).
    pub positive: Option<Face>,
    /// Whether the split produced valid geometry.
    pub split_occurred: bool,
}

/// Split a face by a plane, returning the inside-half and outside-half.
///
/// Uses the Sutherland-Hodgman polygon clipping algorithm.
/// The plane normal points outward from the reference solid.
pub fn split_face_by_plane(
    face: &Face,
    plane: &Plane,
    next_face_id: &mut u64,
    next_edge_id: &mut u64,
    next_vert_id: &mut u64,
    tol: &Tolerance,
) -> SplitFaceResult {
    let vertices: Vec<Point3D> = face
        .outer_loop
        .edges
        .iter()
        .map(|e| e.start.position)
        .collect();

    if vertices.len() < 3 {
        return SplitFaceResult {
            negative: None,
            positive: None,
            split_occurred: false,
        };
    }

    // Compute signed distances.
    let dists: Vec<f64> = vertices
        .iter()
        .map(|v| plane.signed_distance(v).value)
        .collect();

    let all_neg = dists.iter().all(|&d| d <= tol.absolute);
    let all_pos = dists.iter().all(|&d| d >= -tol.absolute);

    if all_neg {
        return SplitFaceResult {
            negative: Some(face.clone()),
            positive: None,
            split_occurred: false,
        };
    }
    if all_pos {
        return SplitFaceResult {
            negative: None,
            positive: Some(face.clone()),
            split_occurred: false,
        };
    }

    // Clip on the negative side (keep points where signed_distance ≤ 0).
    let neg_poly = clip_polygon_negative(&vertices, &dists, plane, tol);
    let pos_poly = clip_polygon_positive(&vertices, &dists, plane, tol);

    let mut result = SplitFaceResult {
        negative: None,
        positive: None,
        split_occurred: true,
    };

    if neg_poly.len() >= 3 {
        let f = build_face_from_polygon(&neg_poly, *next_face_id, *next_edge_id, *next_vert_id);
        *next_face_id += 1;
        *next_edge_id += neg_poly.len() as u64;
        *next_vert_id += neg_poly.len() as u64;
        result.negative = Some(f);
    }
    if pos_poly.len() >= 3 {
        let f = build_face_from_polygon(&pos_poly, *next_face_id, *next_edge_id, *next_vert_id);
        *next_face_id += 1;
        *next_edge_id += pos_poly.len() as u64;
        *next_vert_id += pos_poly.len() as u64;
        result.positive = Some(f);
    }

    result
}

/// Clip a polygon to the half-space where signed_distance ≤ 0.
fn clip_polygon_negative(
    vertices: &[Point3D],
    dists: &[f64],
    plane: &Plane,
    _tol: &Tolerance,
) -> Vec<Point3D> {
    clip_sutherland_hodgman(vertices, dists, plane, false)
}

fn clip_polygon_positive(
    vertices: &[Point3D],
    dists: &[f64],
    plane: &Plane,
    _tol: &Tolerance,
) -> Vec<Point3D> {
    clip_sutherland_hodgman(vertices, dists, plane, true)
}

/// Sutherland-Hodgman polygon clipping against a plane.
fn clip_sutherland_hodgman(
    vertices: &[Point3D],
    dists: &[f64],
    _plane: &Plane,
    keep_positive: bool,
) -> Vec<Point3D> {
    let n = vertices.len();
    let mut output: Vec<Point3D> = Vec::new();

    for i in 0..n {
        let curr = &vertices[i];
        let next = &vertices[(i + 1) % n];
        let dc = dists[i];
        let dn = dists[(i + 1) % n];

        let curr_inside = if keep_positive { dc >= 0.0 } else { dc <= 0.0 };
        let next_inside = if keep_positive { dn >= 0.0 } else { dn <= 0.0 };

        if curr_inside {
            output.push(*curr);
        }
        if curr_inside != next_inside {
            // Edge crosses the plane — compute intersection.
            let t = dc / (dc - dn);
            let ix = curr.x.value + t * (next.x.value - curr.x.value);
            let iy = curr.y.value + t * (next.y.value - curr.y.value);
            let iz = curr.z.value + t * (next.z.value - curr.z.value);
            output.push(Point3D::new(ix, iy, iz));
        }
    }

    // Remove near-duplicate vertices.
    dedup_polygon(&output, 1e-9)
}

/// Remove consecutive near-duplicate vertices from a polygon.
fn dedup_polygon(pts: &[Point3D], eps: f64) -> Vec<Point3D> {
    if pts.len() < 2 {
        return pts.to_vec();
    }
    let mut result: Vec<Point3D> = Vec::new();
    for pt in pts {
        if let Some(last) = result.last()
            && last.distance_to(pt).value < eps
        {
            continue;
        }
        result.push(*pt);
    }
    // Check wrap-around.
    if result.len() >= 2 {
        let first = result[0];
        let last = result[result.len() - 1];
        if first.distance_to(&last).value < eps {
            result.pop();
        }
    }
    result
}

/// Build a face from a polygon (assumes CCW order and planar).
fn build_face_from_polygon(
    pts: &[Point3D],
    face_id: u64,
    start_edge_id: u64,
    start_vert_id: u64,
) -> Face {
    let n = pts.len();
    let vertices: Vec<Vertex> = pts
        .iter()
        .enumerate()
        .map(|(i, p)| Vertex::new(start_vert_id + i as u64, *p))
        .collect();

    let edges: Vec<Edge> = (0..n)
        .map(|i| {
            Edge::new(
                start_edge_id + i as u64,
                vertices[i].clone(),
                vertices[(i + 1) % n].clone(),
            )
        })
        .collect();

    let outer_loop = Loop::new(0, edges);
    Face::new(face_id, outer_loop, vec![])
}

// ---------------------------------------------------------------------------
// Solid boolean operations
// ---------------------------------------------------------------------------

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
///
/// Returns faces from A that are outside B + faces from B that are outside A.
pub fn boolean_union(a: &Solid, b: &Solid, tol: &Tolerance) -> BooleanResult {
    let mut id_gen = IdGenerator::new(a, b);
    let mut result_faces: Vec<Face> = Vec::new();

    // Faces from A that are outside B, straddling faces split.
    for face in &a.shell.faces {
        process_boolean_face(
            face,
            b,
            true, // keep_outside = true (union: keep A-faces outside B)
            &mut result_faces,
            &mut id_gen,
            tol,
        );
    }

    // Faces from B that are outside A.
    for face in &b.shell.faces {
        process_boolean_face(
            face,
            a,
            true, // keep_outside = true
            &mut result_faces,
            &mut id_gen,
            tol,
        );
    }

    build_boolean_result(result_faces, id_gen)
}

/// Compute the subtract of B from A (A − B).
///
/// Returns faces from A that are outside B + inverted faces from B that are inside A.
pub fn boolean_subtract(a: &Solid, b: &Solid, tol: &Tolerance) -> BooleanResult {
    let mut id_gen = IdGenerator::new(a, b);
    let mut result_faces: Vec<Face> = Vec::new();

    // Faces from A that are outside B.
    for face in &a.shell.faces {
        process_boolean_face(
            face,
            b,
            true, // keep outside
            &mut result_faces,
            &mut id_gen,
            tol,
        );
    }

    // Inverted faces from B that are inside A.
    for face in &b.shell.faces {
        let side = classify_face(face, a, tol);
        match side {
            FaceSide::Inside => {
                // Keep inverted (flip normal by reversing edge order).
                let mut inverted = face.clone();
                inverted.plane = Plane::from_normal_and_point(
                    face.plane.normal.scale(scalar(-1.0)),
                    &face.outer_loop.edges[0].start.position,
                );
                // Reverse the loop.
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
                inverted.outer_loop = Loop::new(0, rev_edges);
                // Update IDs.
                inverted.id = id_gen.next_face();
                inverted.outer_loop = Loop::new(0, inverted.outer_loop.edges);
                result_faces.push(inverted);
            }
            FaceSide::Straddling => {
                // Split and keep the inside portion, inverted.
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
            _ => { /* discard */ }
        }
    }

    build_boolean_result(result_faces, id_gen)
}

/// Compute the intersection of two solids (A ∩ B).
///
/// Returns faces from A that are inside B + faces from B that are inside A.
pub fn boolean_intersect(a: &Solid, b: &Solid, tol: &Tolerance) -> BooleanResult {
    let mut id_gen = IdGenerator::new(a, b);
    let mut result_faces: Vec<Face> = Vec::new();

    // Faces from A that are inside B.
    for face in &a.shell.faces {
        process_boolean_face(
            face,
            b,
            false, // keep_inside = true (intersection: keep A-faces inside B)
            &mut result_faces,
            &mut id_gen,
            tol,
        );
    }

    // Faces from B that are inside A.
    for face in &b.shell.faces {
        process_boolean_face(
            face,
            a,
            false, // keep_inside = true
            &mut result_faces,
            &mut id_gen,
            tol,
        );
    }

    build_boolean_result(result_faces, id_gen)
}

/// Process a single face for boolean operations: classify, optionally split,
/// and keep the appropriate side.
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
            // Split against each face-plane of the other solid.
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
                            // Negative side relative to this plane may still be
                            // outside the solid overall — add for re-evaluation.
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
            // Keep appropriate pieces.
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
        _ => { /* discard */ }
    }
}

/// Invert a face (flip normal, reverse edges).
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

/// Build the final boolean result from collected faces.
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

// ---------------------------------------------------------------------------
// ID generator (keeps IDs unique across split faces)
// ---------------------------------------------------------------------------

struct IdGenerator {
    next_face: u64,
    next_edge: u64,
    next_vert: u64,
}

impl IdGenerator {
    fn new(a: &Solid, b: &Solid) -> Self {
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
    fn next_face(&mut self) -> u64 {
        let id = self.next_face;
        self.next_face += 1;
        id
    }
    fn next_edge(&mut self) -> u64 {
        let id = self.next_edge;
        self.next_edge += 1;
        id
    }
    fn next_vert(&mut self) -> u64 {
        let id = self.next_vert;
        self.next_vert += 1;
        id
    }
}

// ---------------------------------------------------------------------------
// Original intersection types (preserved from Phase A)
// ---------------------------------------------------------------------------

/// Result of intersecting two geometric entities.
#[derive(Debug, Clone)]
pub enum Intersection {
    /// No intersection.
    None,
    /// Intersection at a single point.
    Point(Point3D),
    /// Intersection along a line (coincident planes, etc.).
    Line(Line),
}

/// Intersect an infinite line with an infinite plane.
pub fn intersect_line_plane(line: &Line, plane: &Plane, tol: &Tolerance) -> Intersection {
    let denom = plane.normal.dot(&line.direction);
    if tol.nearly_equal(denom.value, 0.0) {
        return Intersection::None;
    }
    let numerator = -(plane
        .normal
        .dot(&Vector3D::new(line.origin.x, line.origin.y, line.origin.z))
        .value
        + plane.d.value);
    let t = numerator / denom.value;
    let point = Point3D::new(
        line.origin.x.value + t * line.direction.x.value,
        line.origin.y.value + t * line.direction.y.value,
        line.origin.z.value + t * line.direction.z.value,
    );
    Intersection::Point(point)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::Line;
    use crate::topology::{Edge, Loop, Vertex};

    fn tol() -> Tolerance {
        Tolerance::default()
    }

    /// Build a unit cube solid at origin.
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
                    Edge::new(0, v0.clone(), v1.clone()),
                    Edge::new(1, v1.clone(), v2.clone()),
                    Edge::new(2, v2.clone(), v3.clone()),
                    Edge::new(3, v3.clone(), v0.clone()),
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

    // -- Point classification tests --

    #[test]
    fn point_inside_cube() {
        let cube = make_cube(0, 0.0, 0.0, 0.0, 2.0);
        let pt = Point3D::new(0.0, 0.0, 0.0);
        assert_eq!(point_vs_solid(&pt, &cube, &tol()), PointLocation::Inside);
    }

    #[test]
    fn point_outside_cube() {
        let cube = make_cube(0, 0.0, 0.0, 0.0, 2.0);
        let pt = Point3D::new(5.0, 0.0, 0.0);
        assert_eq!(point_vs_solid(&pt, &cube, &tol()), PointLocation::Outside);
    }

    #[test]
    fn point_on_cube_boundary() {
        let cube = make_cube(0, 0.0, 0.0, 0.0, 2.0);
        let pt = Point3D::new(1.0, 0.0, 0.0); // on the face x=1
        assert_eq!(
            point_vs_solid(&pt, &cube, &tol()),
            PointLocation::OnBoundary
        );
    }

    // -- Face classification tests --

    #[test]
    fn classify_face_outside() {
        let a = make_cube(0, 0.0, 0.0, 0.0, 2.0);
        let b = make_cube(1, 5.0, 0.0, 0.0, 2.0);
        // A's face on x=+1 is completely outside B.
        let face = &a.shell.faces[5]; // right face of A (x=+1)
        assert_eq!(classify_face(face, &b, &tol()), FaceSide::Outside);
    }

    #[test]
    fn classify_face_inside() {
        let a = make_cube(0, 0.0, 0.0, 0.0, 2.0); // [-1,1]³
        let b = make_cube(1, 0.0, 0.0, 0.0, 5.0); // [-2.5,2.5]³
        // Small cube A is completely inside large cube B.
        let face = &a.shell.faces[0]; // bottom
        assert!(matches!(
            classify_face(face, &b, &tol()),
            FaceSide::Inside | FaceSide::On
        ));
    }

    // -- Boolean tests --

    #[test]
    fn boolean_union_disjoint_cubes() {
        let a = make_cube(0, 0.0, 0.0, 0.0, 2.0);
        let b = make_cube(1, 5.0, 0.0, 0.0, 2.0);
        let result = boolean_union(&a, &b, &tol());
        assert!(result.success);
        assert!(result.solid.is_some());
        // Should have faces from both cubes (6 + 6 = 12, no overlap).
        assert!(result.face_count >= 8, "got {} faces", result.face_count);
    }

    #[test]
    fn boolean_intersect_contained() {
        // Small cube A completely inside large cube B → intersection = A.
        let a = make_cube(0, 0.0, 0.0, 0.0, 2.0); // [-1,1]³
        let b = make_cube(1, 0.0, 0.0, 0.0, 5.0); // [-2.5,2.5]³
        let result = boolean_intersect(&a, &b, &tol());
        assert!(result.success);
        // All faces of A should be inside B, producing the intersection.
        assert!(result.face_count >= 4, "got {} faces", result.face_count);
    }

    #[test]
    fn boolean_subtract_non_overlapping() {
        let a = make_cube(0, 0.0, 0.0, 0.0, 2.0);
        let b = make_cube(1, 5.0, 0.0, 0.0, 2.0);
        let result = boolean_subtract(&a, &b, &tol());
        assert!(result.success);
        // A minus disjoint B = A unchanged.
        assert!(result.solid.is_some());
    }

    // -- Face splitting tests --

    #[test]
    fn split_face_by_plane_clean() {
        let face = {
            let verts: Vec<Vertex> = [
                Point3D::new(-1.0, -1.0, 0.0),
                Point3D::new(1.0, -1.0, 0.0),
                Point3D::new(1.0, 1.0, 0.0),
                Point3D::new(-1.0, 1.0, 0.0),
            ]
            .iter()
            .enumerate()
            .map(|(i, p)| Vertex::new(i as u64, *p))
            .collect();
            let outer = Loop::new(
                0,
                vec![
                    Edge::new(0, verts[0].clone(), verts[1].clone()),
                    Edge::new(1, verts[1].clone(), verts[2].clone()),
                    Edge::new(2, verts[2].clone(), verts[3].clone()),
                    Edge::new(3, verts[3].clone(), verts[0].clone()),
                ],
            );
            Face::new(0, outer, vec![])
        };
        // Split by plane x=0.
        let plane = Plane::from_normal_and_point(Vector3D::X, &Point3D::ORIGIN);
        let mut nf = 100;
        let mut ne = 1000;
        let mut nv = 2000;
        let result = split_face_by_plane(&face, &plane, &mut nf, &mut ne, &mut nv, &tol());
        assert!(result.split_occurred);
        assert!(result.negative.is_some());
        assert!(result.positive.is_some());
        // Negative half (x ≤ 0) should have all vertices with x ≤ 0.
        if let Some(neg) = &result.negative {
            for e in &neg.outer_loop.edges {
                assert!(e.start.position.x.value <= tol().absolute);
            }
        }
        if let Some(pos) = &result.positive {
            for e in &pos.outer_loop.edges {
                assert!(e.start.position.x.value >= -tol().absolute);
            }
        }
    }

    // -- Preserved Phase A tests --

    #[test]
    fn line_plane_intersection() {
        let plane = Plane::from_normal_and_point(Vector3D::Z, &Point3D::ORIGIN);
        let line = Line::new(Point3D::new(1.0, 2.0, 5.0), Vector3D::new(0.0, 0.0, -1.0));
        match intersect_line_plane(&line, &plane, &tol()) {
            Intersection::Point(p) => {
                assert!(p.nearly_equal(&Point3D::new(1.0, 2.0, 0.0), &tol()));
            }
            other => panic!("Expected Point, got {other:?}"),
        }
    }

    #[test]
    fn line_plane_parallel() {
        let plane = Plane::from_normal_and_point(Vector3D::Z, &Point3D::ORIGIN);
        let line = Line::new(Point3D::new(1.0, 2.0, 5.0), Vector3D::X);
        match intersect_line_plane(&line, &plane, &tol()) {
            Intersection::None => {}
            other => panic!("Expected None, got {other:?}"),
        }
    }
}
