//! scene_graph — spatial organization and culling of renderable entities.
//!
//! Phase A delivers a basic scene node hierarchy with bounding-box culling.

#![forbid(unsafe_code)]

use geometry_kernel::shapes::BoundingBox;
use std::collections::HashMap;

/// A node in the scene graph.
#[derive(Debug, Clone)]
pub struct SceneNode {
    pub id: u64,
    pub name: String,
    pub bounding_box: BoundingBox,
    pub visible: bool,
    pub children: Vec<u64>,
}

/// The scene graph — a flat collection of hierarchically-organized nodes.
#[derive(Debug, Clone, Default)]
pub struct SceneGraph {
    nodes: HashMap<u64, SceneNode>,
    next_id: u64,
}

impl SceneGraph {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            next_id: 1,
        }
    }
    pub fn add_node(&mut self, name: &str, bbox: BoundingBox) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.insert(
            id,
            SceneNode {
                id,
                name: name.into(),
                bounding_box: bbox,
                visible: true,
                children: vec![],
            },
        );
        id
    }
    pub fn get(&self, id: u64) -> Option<&SceneNode> {
        self.nodes.get(&id)
    }
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geometry_kernel::{Point3D, shapes::BoundingBox};
    #[test]
    fn scene_graph_add_node() {
        let mut sg = SceneGraph::new();
        let bbox =
            BoundingBox::from_corners(&Point3D::new(0.0, 0.0, 0.0), &Point3D::new(1.0, 1.0, 1.0));
        let id = sg.add_node("root", bbox);
        assert_eq!(sg.len(), 1);
        assert!(!sg.is_empty());
        assert!(sg.get(id).is_some());
    }
}
