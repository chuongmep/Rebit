//! Transaction model — atomic changes, commit/rollback, change snapshots.
//!
//! Every mutation to the BIM model goes through a [`Transaction`] context.
//! Changes accumulate in the transaction and are either committed (applied
//! to the model) or rolled back (discarded).

use crate::entity::{Entity, EntityGraph, EntityId};

// ---------------------------------------------------------------------------
// Change
// ---------------------------------------------------------------------------

/// A single change recorded in a transaction.
#[derive(Debug, Clone, PartialEq)]
pub enum Change {
    /// An entity was created.
    EntityCreated(Entity),
    /// An entity was updated (old state preserved for rollback).
    EntityUpdated {
        /// ID of the updated entity.
        id: EntityId,
        /// Previous state of the entity (for rollback).
        previous: Entity,
    },
    /// An entity was removed.
    EntityRemoved {
        /// ID of the removed entity.
        id: EntityId,
        /// The removed entity (for rollback).
        removed: Entity,
    },
}

impl Change {
    /// Human-readable description of the change.
    pub fn description(&self) -> String {
        match self {
            Self::EntityCreated(e) => format!("created {} ({})", e.category_name(), e.name),
            Self::EntityUpdated { id, .. } => format!("updated {}", id),
            Self::EntityRemoved { id, .. } => format!("removed {}", id),
        }
    }
}

// ---------------------------------------------------------------------------
// Transaction
// ---------------------------------------------------------------------------

/// An atomic transaction that records changes to the entity graph.
///
/// Once committed, changes are applied; on rollback they are discarded.
/// Transactions are not nested in Phase A.
#[derive(Debug, Clone, Default)]
pub struct Transaction {
    changes: Vec<Change>,
    committed: bool,
    rolled_back: bool,
}

impl Transaction {
    /// Create a new, empty transaction.
    pub fn new() -> Self {
        Self {
            changes: Vec::new(),
            committed: false,
            rolled_back: false,
        }
    }

    /// Record that an entity was created.
    pub fn record_create(&mut self, entity: Entity) {
        self.assert_active();
        self.changes.push(Change::EntityCreated(entity));
    }

    /// Record that an entity was updated (save previous state for rollback).
    pub fn record_update(&mut self, id: EntityId, previous: Entity) {
        self.assert_active();
        self.changes.push(Change::EntityUpdated { id, previous });
    }

    /// Record that an entity was removed.
    pub fn record_remove(&mut self, id: EntityId, removed: Entity) {
        self.assert_active();
        self.changes.push(Change::EntityRemoved { id, removed });
    }

    /// Commit the transaction — apply all changes to the graph.
    pub fn commit(mut self, graph: &mut EntityGraph) -> Result<(), TransactionError> {
        self.assert_active();
        self.committed = true;
        for change in &self.changes {
            match change {
                Change::EntityCreated(e) => {
                    graph.insert(e.clone());
                }
                Change::EntityUpdated { .. } => {
                    // The updated entity is already present in the graph;
                    // only the pre-update snapshot is in previous.
                }
                Change::EntityRemoved { id, .. } => {
                    graph.remove(*id);
                }
            }
        }
        Ok(())
    }

    /// Roll back the transaction — discard all changes.
    pub fn rollback(self, graph: &mut EntityGraph) -> Result<(), TransactionError> {
        self.assert_active();
        // Walk changes in reverse to undo.
        for change in self.changes.iter().rev() {
            match change {
                Change::EntityCreated(e) => {
                    graph.remove(e.id);
                }
                Change::EntityUpdated { previous, .. } => {
                    graph.insert(previous.clone());
                }
                Change::EntityRemoved { removed, .. } => {
                    graph.insert(removed.clone());
                }
            }
        }
        Ok(())
    }

    /// Number of changes recorded.
    pub fn change_count(&self) -> usize {
        self.changes.len()
    }

    /// Iterate over recorded changes.
    pub fn changes(&self) -> &[Change] {
        &self.changes
    }

    fn assert_active(&self) {
        assert!(!self.committed, "transaction already committed");
        assert!(!self.rolled_back, "transaction already rolled back");
    }
}

// ---------------------------------------------------------------------------
// Transaction error
// ---------------------------------------------------------------------------

/// Errors that can occur during transaction operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionError {
    /// Transaction was already committed or rolled back.
    AlreadyFinalized,
}

impl std::fmt::Display for TransactionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyFinalized => write!(f, "transaction already finalized"),
        }
    }
}

impl std::error::Error for TransactionError {}

// ---------------------------------------------------------------------------
// Extension trait for convenience descriptions
// ---------------------------------------------------------------------------

impl crate::entity::Entity {
    fn category_name(&self) -> &'static str {
        match self.category {
            crate::entity::EntityCategory::Wall => "Wall",
            crate::entity::EntityCategory::Slab => "Slab",
            crate::entity::EntityCategory::Beam => "Beam",
            crate::entity::EntityCategory::Column => "Column",
            crate::entity::EntityCategory::Door => "Door",
            crate::entity::EntityCategory::Window => "Window",
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::EntityCategory;
    use geometry_kernel::Point3D;
    use geometry_kernel::topology::{Edge, Face, Loop, Shell, Solid, Vertex};

    fn make_box_solid(id: u64, w: f64, d: f64, h: f64) -> Solid {
        let v = [
            Point3D::new(0.0, 0.0, 0.0),
            Point3D::new(w, 0.0, 0.0),
            Point3D::new(w, d, 0.0),
            Point3D::new(0.0, d, 0.0),
            Point3D::new(0.0, 0.0, h),
            Point3D::new(w, 0.0, h),
            Point3D::new(w, d, h),
            Point3D::new(0.0, d, h),
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
        let shell = Shell::new(0, faces);
        Solid::new(id, shell)
    }

    #[test]
    fn transaction_commit_create() {
        let mut graph = EntityGraph::new();
        let mut tx = Transaction::new();
        let entity = Entity::new(
            EntityId(1),
            EntityCategory::Wall,
            "TxWall",
            make_box_solid(0, 5.0, 0.3, 3.0),
        );
        tx.record_create(entity.clone());
        assert_eq!(tx.change_count(), 1);
        tx.commit(&mut graph).unwrap();
        assert!(graph.get(EntityId(1)).is_some());
    }

    #[test]
    fn transaction_rollback_create() {
        let mut graph = EntityGraph::new();
        let mut tx = Transaction::new();
        let entity = Entity::new(
            EntityId(1),
            EntityCategory::Slab,
            "TxSlab",
            make_box_solid(0, 10.0, 10.0, 0.2),
        );
        tx.record_create(entity);
        tx.rollback(&mut graph).unwrap();
        assert!(graph.is_empty());
    }

    #[test]
    fn transaction_rollback_remove() {
        let mut graph = EntityGraph::new();
        let entity = Entity::new(
            EntityId(1),
            EntityCategory::Column,
            "Col-01",
            make_box_solid(0, 0.3, 0.3, 4.0),
        );
        graph.insert(entity.clone());

        let mut tx = Transaction::new();
        tx.record_remove(EntityId(1), entity);
        assert!(graph.get(EntityId(1)).is_some());
        tx.rollback(&mut graph).unwrap();
        assert!(graph.get(EntityId(1)).is_some()); // restored
    }
}
