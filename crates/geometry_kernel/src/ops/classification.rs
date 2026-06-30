//! Face and point classification relative to solids.

use crate::topology::{Face, Solid};
use core_math::{Point3D, Tolerance, Vector3D};

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

/// Classify a face relative to a reference solid.
pub fn classify_face(face: &Face, solid: &Solid, tol: &Tolerance) -> FaceSide {
    let mut inside_count = 0usize;
    let mut outside_count = 0usize;
    for edge in &face.outer_loop.edges {
        let pt = &edge.start.position;
        let loc = point_vs_solid(pt, solid, tol);
        match loc {
            PointLocation::Inside => inside_count += 1,
            PointLocation::Outside => outside_count += 1,
            PointLocation::OnBoundary => {}
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

/// Classify a point relative to a solid using ray-casting along +X.
pub fn point_vs_solid(point: &Point3D, solid: &Solid, tol: &Tolerance) -> PointLocation {
    let ray_origin = *point;
    let ray_dir = Vector3D::X;
    let mut intersection_count = 0usize;
    for face in &solid.shell.faces {
        let sd = face.plane.signed_distance(&ray_origin).value;
        let nd = face.normal().dot(&ray_dir).value;
        if nd.abs() < 1e-15 {
            if tol.nearly_equal(sd, 0.0) {
                let proj = face.plane.project_point(&ray_origin);
                if point_in_face_polygon(&proj, face, tol) {
                    return PointLocation::OnBoundary;
                }
            }
            continue;
        }
        let t = -sd / nd;
        if t < -tol.absolute {
            continue;
        }
        let hit = Point3D::new(
            ray_origin.x.value + t * ray_dir.x.value,
            ray_origin.y.value + t * ray_dir.y.value,
            ray_origin.z.value + t * ray_dir.z.value,
        );
        if point_in_face_polygon(&hit, face, tol) {
            if t < tol.absolute {
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

/// Dominant 2D projection plane based on face normal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionPlane {
    /// Project onto XY plane (normal is mostly Z).
    XY,
    /// Project onto XZ plane (normal is mostly Y).
    XZ,
    /// Project onto YZ plane (normal is mostly X).
    YZ,
}

/// Project a point onto the chosen 2D coordinate plane.
pub fn project_onto(plane: ProjectionPlane, pt: &Point3D) -> (f64, f64) {
    match plane {
        ProjectionPlane::XY => (pt.x.value, pt.y.value),
        ProjectionPlane::XZ => (pt.x.value, pt.z.value),
        ProjectionPlane::YZ => (pt.y.value, pt.z.value),
    }
}

/// Point-in-polygon test using ray-casting onto the face's dominant 2D plane.
pub fn point_in_face_polygon(pt: &Point3D, face: &Face, _tol: &Tolerance) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::{Edge, Loop, Shell, Solid, Vertex};
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
    #[test]
    fn point_inside_cube() {
        assert_eq!(
            point_vs_solid(
                &Point3D::new(0.0, 0.0, 0.0),
                &make_cube(0, 0.0, 0.0, 0.0, 2.0),
                &tol()
            ),
            PointLocation::Inside
        );
    }
    #[test]
    fn point_outside_cube() {
        assert_eq!(
            point_vs_solid(
                &Point3D::new(5.0, 0.0, 0.0),
                &make_cube(0, 0.0, 0.0, 0.0, 2.0),
                &tol()
            ),
            PointLocation::Outside
        );
    }
    #[test]
    fn point_on_cube_boundary() {
        assert_eq!(
            point_vs_solid(
                &Point3D::new(1.0, 0.0, 0.0),
                &make_cube(0, 0.0, 0.0, 0.0, 2.0),
                &tol()
            ),
            PointLocation::OnBoundary
        );
    }
    #[test]
    fn classify_face_outside() {
        let a = make_cube(0, 0.0, 0.0, 0.0, 2.0);
        let b = make_cube(1, 5.0, 0.0, 0.0, 2.0);
        assert_eq!(
            classify_face(&a.shell.faces[5], &b, &tol()),
            FaceSide::Outside
        );
    }
    #[test]
    fn classify_face_inside() {
        let a = make_cube(0, 0.0, 0.0, 0.0, 2.0);
        let b = make_cube(1, 0.0, 0.0, 0.0, 5.0);
        assert!(matches!(
            classify_face(&a.shell.faces[0], &b, &tol()),
            FaceSide::Inside | FaceSide::On
        ));
    }
}
