//! rendering_engine — viewport rendering pipeline and draw command buffer.
//!
//! Phase A delivers a render command abstraction and scene traversal.

#![forbid(unsafe_code)]

use scene_graph::SceneGraph;

/// A draw command targeting a subset of the scene.
#[derive(Debug, Clone)]
pub struct DrawCall {
    pub node_ids: Vec<u64>,
    pub shader_id: u64,
    pub wireframe: bool,
}

/// Collects draw calls from the scene graph.
#[derive(Debug, Default)]
pub struct RenderPipeline;

impl RenderPipeline {
    pub fn new() -> Self {
        Self
    }
    /// Collect visible nodes as draw calls (simplified in Phase A).
    pub fn collect(&self, _scene: &SceneGraph) -> Vec<DrawCall> {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pipeline_collects_empty() {
        let scene = SceneGraph::new();
        let pipeline = RenderPipeline::new();
        let calls = pipeline.collect(&scene);
        assert!(calls.is_empty());
    }
}
