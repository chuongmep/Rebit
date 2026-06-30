//! Face-plane splitting (Sutherland-Hodgman polygon clipping).

use crate::shapes::Plane;
use crate::topology::{Edge, Face, Loop, Vertex};
use core_math::{Point3D, Tolerance};

// ---------------------------------------------------------------------------
// Public API
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

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

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
            let t = dc / (dc - dn);
            let ix = curr.x.value + t * (next.x.value - curr.x.value);
            let iy = curr.y.value + t * (next.y.value - curr.y.value);
            let iz = curr.z.value + t * (next.z.value - curr.z.value);
            output.push(Point3D::new(ix, iy, iz));
        }
    }

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
pub fn build_face_from_polygon(
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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shapes::Plane;
    use core_math::Vector3D;

    fn tol() -> Tolerance {
        Tolerance::default()
    }

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
        let plane = Plane::from_normal_and_point(Vector3D::X, &Point3D::ORIGIN);
        let mut nf = 100;
        let mut ne = 1000;
        let mut nv = 2000;
        let result = split_face_by_plane(&face, &plane, &mut nf, &mut ne, &mut nv, &tol());
        assert!(result.split_occurred);
        assert!(result.negative.is_some());
        assert!(result.positive.is_some());
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
}
