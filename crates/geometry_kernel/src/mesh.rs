//! Mesh extraction — converts B-Rep Solids to triangle meshes for GPU rendering.
//!
//! Phase D: tessellates each face of a Solid into triangles via fan triangulation,
//! producing flat (position, normal, color) vertex arrays and index arrays.

#![allow(missing_docs)]

use crate::tessellation::triangulate_face;
use crate::topology::Solid;
use core_math::{Point3D, Tolerance};

/// A triangle mesh extracted from a solid, ready for GPU upload.
#[derive(Debug, Clone)]
pub struct GpuMesh {
    /// Flat array of vertex attributes: [px, py, pz, nx, ny, nz, r, g, b].
    /// 9 floats per vertex.
    pub vertices: Vec<f32>,
    /// Triangle indices (3 per triangle).
    pub indices: Vec<u32>,
}

impl GpuMesh {
    /// Create an empty mesh.
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }

    /// Vertex stride in bytes (9 floats × 4 bytes = 36).
    pub const STRIDE_BYTES: u32 = 36;
}

impl Default for GpuMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract a coloured triangle mesh from a solid.
///
/// Each face is assigned a pseudo-random colour based on its id for
/// visual differentiation of individual faces.
pub fn extract_mesh(solid: &Solid, tol: &Tolerance) -> GpuMesh {
    let mut mesh = GpuMesh::new();
    let mut vertex_base: u32 = 0;

    for face in &solid.shell.faces {
        // Collect face vertices (loop start positions).
        let face_verts: Vec<Point3D> = face
            .outer_loop
            .edges
            .iter()
            .map(|e| e.start.position)
            .collect();
        if face_verts.len() < 3 {
            continue;
        }

        // Face colour — pseudo-random per face id.
        let hue = (face.id.wrapping_mul(2654435761) % 360) as f32;
        let (r, g, b) = hsl_to_rgb(hue / 360.0, 0.7, 0.6);
        let normal = face.normal();

        // Append vertices.
        for pt in &face_verts {
            mesh.vertices.extend_from_slice(&[
                pt.x.value as f32,
                pt.y.value as f32,
                pt.z.value as f32,
                normal.x.value as f32,
                normal.y.value as f32,
                normal.z.value as f32,
                r,
                g,
                b,
            ]);
        }

        // Triangulate the face and offset indices.
        let tris = triangulate_face(&face_verts, tol);
        for tri in &tris {
            mesh.indices.push(vertex_base + tri[0] as u32);
            mesh.indices.push(vertex_base + tri[1] as u32);
            mesh.indices.push(vertex_base + tri[2] as u32);
        }

        vertex_base += face_verts.len() as u32;
    }

    mesh
}

/// Simple HSL → RGB conversion for face colouring.
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h * 6.0) % 2.0 - 1.0).abs());
    let m = l - c * 0.5;
    let (r, g, b) = if h < 1.0 / 6.0 {
        (c, x, 0.0)
    } else if h < 2.0 / 6.0 {
        (x, c, 0.0)
    } else if h < 3.0 / 6.0 {
        (0.0, c, x)
    } else if h < 4.0 / 6.0 {
        (0.0, x, c)
    } else if h < 5.0 / 6.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (r + m, g + m, b + m)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::topology::{Edge, Face, Loop, Shell, Solid, Vertex};

    fn make_cube_solid(id: u64, cx: f64, cy: f64, cz: f64, size: f64) -> Solid {
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
    fn extract_cube_mesh() {
        let cube = make_cube_solid(0, 0.0, 0.0, 0.0, 2.0);
        let tol = Tolerance::default();
        let mesh = extract_mesh(&cube, &tol);
        // Cube: 6 faces × 4 vertices = 24 vertices, 12 triangles = 36 indices.
        assert_eq!(mesh.vertices.len(), 24 * 9); // 9 floats per vertex
        assert_eq!(mesh.indices.len(), 36);
    }
}
