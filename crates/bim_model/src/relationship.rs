//! Entity relationship types and dependency graph (Phase B).
//!
//! Tracks connections, containment, and structural dependencies between
//! BIM entities — enabling query operations like "find all doors on this wall"
//! and "find all beams supported by this column."

use std::collections::{HashMap, HashSet};

use crate::entity::EntityId;

// ---------------------------------------------------------------------------
// Relationship type
// ---------------------------------------------------------------------------

/// A directed relationship between two entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelationshipType {
    /// Source is connected to target (e.g., wall meets wall).
    ConnectedTo,
    /// Source contains target (e.g., wall contains door).
    Contains,
    /// Source supports target (e.g., column supports beam).
    Supports,
    /// Source hosts target (e.g., slab hosts furniture).
    Hosts,
}

impl RelationshipType {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::ConnectedTo => "connected_to",
            Self::Contains => "contains",
            Self::Supports => "supports",
            Self::Hosts => "hosts",
        }
    }
}

// ---------------------------------------------------------------------------
// Relationship edge
// ---------------------------------------------------------------------------

/// A single relationship edge in the entity graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Relationship {
    /// Unique identifier.
    pub id: u64,
    /// Source entity.
    pub source: EntityId,
    /// Target entity.
    pub target: EntityId,
    /// Type of relationship.
    pub rel_type: RelationshipType,
}

// ---------------------------------------------------------------------------
// Relationship graph
// ---------------------------------------------------------------------------

/// A directed graph of entity relationships.
#[derive(Debug, Clone, Default)]
pub struct RelationshipGraph {
    /// All relationships indexed by id.
    edges: HashMap<u64, Relationship>,
    /// Outgoing edges per entity (source → targets).
    outgoing: HashMap<EntityId, Vec<u64>>,
    /// Incoming edges per entity (target → sources).
    incoming: HashMap<EntityId, Vec<u64>>,
    /// Next relationship ID.
    next_id: u64,
}

impl RelationshipGraph {
    /// Create an empty relationship graph.
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            outgoing: HashMap::new(),
            incoming: HashMap::new(),
            next_id: 1,
        }
    }

    /// Add a relationship between two entities.
    pub fn add(&mut self, source: EntityId, target: EntityId, rel_type: RelationshipType) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.edges.insert(
            id,
            Relationship {
                id,
                source,
                target,
                rel_type,
            },
        );
        self.outgoing.entry(source).or_default().push(id);
        self.incoming.entry(target).or_default().push(id);
        id
    }

    /// Get a relationship by id.
    pub fn get(&self, id: u64) -> Option<&Relationship> {
        self.edges.get(&id)
    }

    /// Find all relationships originating from an entity.
    pub fn outgoing_edges(&self, source: EntityId) -> Vec<&Relationship> {
        self.outgoing
            .get(&source)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find all relationships targeting an entity.
    pub fn incoming_edges(&self, target: EntityId) -> Vec<&Relationship> {
        self.incoming
            .get(&target)
            .map(|ids| ids.iter().filter_map(|id| self.edges.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find all entities connected to a given entity (any relationship type).
    pub fn neighbors(&self, entity: EntityId) -> Vec<EntityId> {
        let mut result = HashSet::new();
        for r in self.outgoing_edges(entity) {
            result.insert(r.target);
        }
        for r in self.incoming_edges(entity) {
            result.insert(r.source);
        }
        result.into_iter().collect()
    }

    /// Number of relationships.
    pub fn len(&self) -> usize {
        self.edges.len()
    }

    /// Whether the graph is empty.
    pub fn is_empty(&self) -> bool {
        self.edges.is_empty()
    }

    /// Iterate over all relationships.
    pub fn iter(&self) -> impl Iterator<Item = &Relationship> {
        self.edges.values()
    }

    /// Remove a relationship by id.
    pub fn remove(&mut self, id: u64) -> Option<Relationship> {
        let rel = self.edges.remove(&id)?;
        if let Some(ids) = self.outgoing.get_mut(&rel.source) {
            ids.retain(|&i| i != id);
        }
        if let Some(ids) = self.incoming.get_mut(&rel.target) {
            ids.retain(|&i| i != id);
        }
        Some(rel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relationship_graph_neighbors() {
        let mut g = RelationshipGraph::new();
        let a = EntityId(1);
        let b = EntityId(2);
        let c = EntityId(3);
        g.add(a, b, RelationshipType::Contains);
        g.add(a, c, RelationshipType::ConnectedTo);
        assert_eq!(g.len(), 2);
        let neighbors = g.neighbors(a);
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&b));
        assert!(neighbors.contains(&c));
    }

    #[test]
    fn relationship_graph_outgoing() {
        let mut g = RelationshipGraph::new();
        let wall = EntityId(10);
        let door = EntityId(11);
        g.add(wall, door, RelationshipType::Contains);
        let outgoing = g.outgoing_edges(wall);
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].rel_type, RelationshipType::Contains);
    }

    #[test]
    fn relationship_graph_remove() {
        let mut g = RelationshipGraph::new();
        let id = g.add(EntityId(1), EntityId(2), RelationshipType::Supports);
        assert_eq!(g.len(), 1);
        g.remove(id);
        assert_eq!(g.len(), 0);
    }
}
