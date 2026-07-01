//! scene_graph — spatial organization and culling of renderable entities.
//!
//! # Phase B additions
//! - Parent-child hierarchy with relative transforms
//! - Frustum culling via bounding-box intersection tests
//! - Visible-node collection for draw-command generation

use geometry_kernel::shapes::BoundingBox;
use std::collections::HashMap;

/// A node in the scene graph.
#[derive(Debug, Clone)]
pub struct SceneNode {
    pub id: u64,
    pub name: String,
    pub bounding_box: BoundingBox,
    pub visible: bool,
    pub parent: Option<u64>,
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
                parent: None,
                children: vec![],
            },
        );
        id
    }

    /// Add a child node under a parent.
    pub fn add_child(&mut self, name: &str, bbox: BoundingBox, parent_id: u64) -> Option<u64> {
        if !self.nodes.contains_key(&parent_id) {
            return None;
        }
        let id = self.add_node(name, bbox);
        if let Some(node) = self.nodes.get_mut(&id) {
            node.parent = Some(parent_id);
        }
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.children.push(id);
        }
        Some(id)
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

    /// Collect all visible leaf nodes that intersect the given bounding box.
    pub fn collect_visible(&self, frustum: &BoundingBox, _tol: &core_math::Tolerance) -> Vec<u64> {
        let mut result = Vec::new();
        for (&id, node) in &self.nodes {
            if node.visible && node.bounding_box.overlaps(frustum, _tol) {
                // Only collect if it's a leaf (no children) or if children are
                // not visible.
                if node.children.is_empty()
                    || node
                        .children
                        .iter()
                        .all(|c| !self.nodes.get(c).is_some_and(|n| n.visible))
                {
                    result.push(id);
                }
            }
        }
        result.sort();
        result
    }

    /// Get all children of a node.
    pub fn children(&self, parent_id: u64) -> Vec<&SceneNode> {
        self.nodes
            .get(&parent_id)
            .map(|n| {
                n.children
                    .iter()
                    .filter_map(|c| self.nodes.get(c))
                    .collect()
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core_math::Tolerance;
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

    #[test]
    fn scene_graph_hierarchy() {
        let mut sg = SceneGraph::new();
        let bbox = BoundingBox::from_corners(
            &Point3D::new(0.0, 0.0, 0.0),
            &Point3D::new(10.0, 10.0, 10.0),
        );
        let root = sg.add_node("root", bbox);
        let child = sg
            .add_child(
                "child",
                BoundingBox::from_corners(
                    &Point3D::new(0.0, 0.0, 0.0),
                    &Point3D::new(1.0, 1.0, 1.0),
                ),
                root,
            )
            .unwrap();
        let children = sg.children(root);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, child);
    }

    #[test]
    fn scene_graph_frustum_culling() {
        let mut sg = SceneGraph::new();
        let tol = Tolerance::default();
        // Node inside frustum.
        sg.add_node(
            "inside",
            BoundingBox::from_corners(&Point3D::new(0.0, 0.0, 0.0), &Point3D::new(1.0, 1.0, 1.0)),
        );
        // Node outside frustum.
        sg.add_node(
            "outside",
            BoundingBox::from_corners(
                &Point3D::new(10.0, 10.0, 10.0),
                &Point3D::new(11.0, 11.0, 11.0),
            ),
        );
        let frustum = BoundingBox::from_corners(
            &Point3D::new(-5.0, -5.0, -5.0),
            &Point3D::new(5.0, 5.0, 5.0),
        );
        let visible = sg.collect_visible(&frustum, &tol);
        assert_eq!(visible.len(), 1);
    }
}
