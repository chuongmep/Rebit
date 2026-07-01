//! rendering_engine — viewport rendering pipeline and draw command buffer.
//!
//! # Phase B additions
//! - Collects visible nodes from the scene graph via frustum culling
//! - Generates draw calls with shader/material binding
//! - Wireframe/solid rendering modes per draw call

use core_math::Tolerance;
use geometry_kernel::shapes::BoundingBox;
use scene_graph::SceneGraph;

/// A draw command targeting a subset of the scene.
#[derive(Debug, Clone)]
pub struct DrawCall {
    /// IDs of scene nodes to draw.
    pub node_ids: Vec<u64>,
    /// Shader program identifier.
    pub shader_id: u64,
    /// Whether to render as wireframe.
    pub wireframe: bool,
    /// Opacity (0.0 = transparent, 1.0 = fully opaque).
    pub opacity: f32,
}

/// A render pipeline that collects draw calls from the scene graph.
#[derive(Debug, Clone)]
pub struct RenderPipeline {
    /// Current view frustum (world-space bounding box).
    pub frustum: BoundingBox,
    /// Default shader to use for geometry.
    pub default_shader: u64,
}

impl Default for RenderPipeline {
    fn default() -> Self {
        Self {
            frustum: BoundingBox::from_corners(
                &geometry_kernel::Point3D::new(-100.0, -100.0, -100.0),
                &geometry_kernel::Point3D::new(100.0, 100.0, 100.0),
            ),
            default_shader: 1,
        }
    }
}

impl RenderPipeline {
    /// Create a new render pipeline.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the view frustum.
    pub fn set_frustum(&mut self, frustum: BoundingBox) {
        self.frustum = frustum;
    }

    /// Collect visible nodes from the scene graph and produce draw calls.
    pub fn collect(&self, scene: &SceneGraph, tol: &Tolerance) -> Vec<DrawCall> {
        let visible_ids = scene.collect_visible(&self.frustum, tol);
        if visible_ids.is_empty() {
            return vec![];
        }

        // Batch all visible opaque nodes into one draw call.
        // In Phase C, this will be split by material/shader groups.
        vec![DrawCall {
            node_ids: visible_ids,
            shader_id: self.default_shader,
            wireframe: false,
            opacity: 1.0,
        }]
    }

    /// Produce a wireframe overlay draw call.
    pub fn collect_wireframe(&self, scene: &SceneGraph, tol: &Tolerance) -> Vec<DrawCall> {
        let visible_ids = scene.collect_visible(&self.frustum, tol);
        if visible_ids.is_empty() {
            return vec![];
        }
        vec![DrawCall {
            node_ids: visible_ids,
            shader_id: self.default_shader,
            wireframe: true,
            opacity: 0.5,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geometry_kernel::{Point3D, shapes::BoundingBox};
    use scene_graph::SceneGraph;

    #[test]
    fn pipeline_collects_visible_nodes() {
        let tol = Tolerance::default();
        let mut scene = SceneGraph::new();
        scene.add_node(
            "cube",
            BoundingBox::from_corners(&Point3D::new(0.0, 0.0, 0.0), &Point3D::new(1.0, 1.0, 1.0)),
        );
        let pipeline = RenderPipeline::new();
        let calls = pipeline.collect(&scene, &tol);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].node_ids.len(), 1);
        assert!(!calls[0].wireframe);
    }

    #[test]
    fn pipeline_collects_empty_scene() {
        let scene = SceneGraph::new();
        let pipeline = RenderPipeline::new();
        let tol = Tolerance::default();
        let calls = pipeline.collect(&scene, &tol);
        assert!(calls.is_empty());
    }

    #[test]
    fn pipeline_wireframe_mode() {
        let mut scene = SceneGraph::new();
        scene.add_node(
            "box",
            BoundingBox::from_corners(&Point3D::new(0.0, 0.0, 0.0), &Point3D::new(1.0, 1.0, 1.0)),
        );
        let pipeline = RenderPipeline::new();
        let tol = Tolerance::default();
        let calls = pipeline.collect_wireframe(&scene, &tol);
        assert_eq!(calls.len(), 1);
        assert!(calls[0].wireframe);
    }
}
