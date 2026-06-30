//! BIM entity types and the entity graph.
//!
//! This module defines the building-domain primitives: walls, slabs, beams,
//! columns, doors, windows — each carrying geometry and user-defined properties.

use core_math::Scalar;
use geometry_kernel::topology::Solid;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Entity ID
// ---------------------------------------------------------------------------

/// A unique identifier for a BIM entity within a model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub u64);

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "E{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// Entity properties
// ---------------------------------------------------------------------------

/// A typed property value attached to an entity.
#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    /// String value.
    String(String),
    /// Scalar numeric value.
    Number(Scalar),
    /// Boolean flag.
    Bool(bool),
}

/// A key-value property map for an entity.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Properties(HashMap<String, PropertyValue>);

impl Properties {
    /// Create an empty property set.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Insert a property.
    pub fn insert(&mut self, key: impl Into<String>, value: PropertyValue) {
        self.0.insert(key.into(), value);
    }

    /// Get a property by key.
    pub fn get(&self, key: &str) -> Option<&PropertyValue> {
        self.0.get(key)
    }

    /// Number of properties.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// `true` when empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterate over key-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &PropertyValue)> {
        self.0.iter()
    }
}

// ---------------------------------------------------------------------------
// BIM entity category
// ---------------------------------------------------------------------------

/// Top-level category of a BIM entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityCategory {
    /// Wall.
    Wall,
    /// Floor slab.
    Slab,
    /// Beam.
    Beam,
    /// Column.
    Column,
    /// Door.
    Door,
    /// Window.
    Window,
}

// ---------------------------------------------------------------------------
// BIM entity
// ---------------------------------------------------------------------------

/// A single BIM entity — a building element with geometry, properties, and
/// optional relationships to other entities.
#[derive(Debug, Clone, PartialEq)]
pub struct Entity {
    /// Unique identifier.
    pub id: EntityId,
    /// Category.
    pub category: EntityCategory,
    /// Display name (human-readable).
    pub name: String,
    /// Solid geometry defining the entity's volume.
    pub geometry: Solid,
    /// User-defined properties.
    pub properties: Properties,
    /// Parent entity ID (e.g. wall containing a door).
    pub parent: Option<EntityId>,
}

impl Entity {
    /// Create a new entity.
    pub fn new(
        id: EntityId,
        category: EntityCategory,
        name: impl Into<String>,
        geometry: Solid,
    ) -> Self {
        Self {
            id,
            category,
            name: name.into(),
            geometry,
            properties: Properties::new(),
            parent: None,
        }
    }

    /// Compute the axis-aligned bounding box of this entity's geometry.
    pub fn bounding_box(&self) -> geometry_kernel::shapes::BoundingBox {
        self.geometry.bounding_box()
    }
}

// ---------------------------------------------------------------------------
// Entity graph
// ---------------------------------------------------------------------------

/// The entity graph — a collection of all BIM entities in a model, indexed
/// by [`EntityId`].
#[derive(Debug, Clone, Default)]
pub struct EntityGraph {
    entities: HashMap<u64, Entity>,
    next_id: u64,
}

impl EntityGraph {
    /// Create an empty entity graph.
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_id: 1,
        }
    }

    /// Insert an entity (or update if id already exists).
    pub fn insert(&mut self, entity: Entity) {
        let id = entity.id.0;
        if id >= self.next_id {
            self.next_id = id + 1;
        }
        self.entities.insert(id, entity);
    }

    /// Create and insert a new entity with an auto-generated ID.
    pub fn create(
        &mut self,
        category: EntityCategory,
        name: impl Into<String>,
        geometry: Solid,
    ) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id += 1;
        self.insert(Entity::new(id, category, name, geometry));
        id
    }

    /// Get an entity by id.
    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id.0)
    }

    /// Get mutable access by id.
    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id.0)
    }

    /// Remove an entity by id.
    pub fn remove(&mut self, id: EntityId) -> Option<Entity> {
        self.entities.remove(&id.0)
    }

    /// Number of entities.
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// `true` when empty.
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Iterate over all entities.
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    /// Find all entities of a given category.
    pub fn by_category(&self, category: EntityCategory) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|e| e.category == category)
            .collect()
    }

    /// Find children of a parent entity.
    pub fn children(&self, parent_id: EntityId) -> Vec<&Entity> {
        self.entities
            .values()
            .filter(|e| e.parent == Some(parent_id))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use geometry_kernel::Point3D;
    use geometry_kernel::topology::{Edge, Face, Loop, Shell, Solid, Vertex};

    /// Helper: build a simple box solid at origin with given dimensions.
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
    fn entity_graph_insert_and_retrieve() {
        let mut graph = EntityGraph::new();
        let solid = make_box_solid(0, 5.0, 0.3, 3.0);
        let id = graph.create(EntityCategory::Wall, "Wall-01", solid);
        let entity = graph.get(id).unwrap();
        assert_eq!(entity.category, EntityCategory::Wall);
        assert_eq!(entity.name, "Wall-01");
    }

    #[test]
    fn entity_graph_by_category() {
        let mut graph = EntityGraph::new();
        graph.create(EntityCategory::Wall, "W1", make_box_solid(0, 5.0, 0.3, 3.0));
        graph.create(EntityCategory::Wall, "W2", make_box_solid(1, 3.0, 0.3, 3.0));
        graph.create(EntityCategory::Door, "D1", make_box_solid(2, 1.0, 0.1, 2.1));
        let walls = graph.by_category(EntityCategory::Wall);
        assert_eq!(walls.len(), 2);
        let doors = graph.by_category(EntityCategory::Door);
        assert_eq!(doors.len(), 1);
    }

    #[test]
    fn entity_properties() {
        let mut graph = EntityGraph::new();
        let id = graph.create(
            EntityCategory::Slab,
            "Slab-01",
            make_box_solid(0, 10.0, 10.0, 0.2),
        );
        let entity = graph.get_mut(id).unwrap();
        entity
            .properties
            .insert("material", PropertyValue::String("Concrete".into()));
        entity
            .properties
            .insert("thickness", PropertyValue::Number(Scalar::new(0.2)));
        assert_eq!(
            entity.properties.get("material"),
            Some(&PropertyValue::String("Concrete".into()))
        );
    }

    #[test]
    fn entity_children() {
        let mut graph = EntityGraph::new();
        let wall_id = graph.create(
            EntityCategory::Wall,
            "Wall-A",
            make_box_solid(0, 6.0, 0.3, 3.0),
        );
        // Add a window child of the wall
        let mut window = Entity::new(
            EntityId(100),
            EntityCategory::Window,
            "Win-01",
            make_box_solid(100, 1.2, 0.1, 1.5),
        );
        window.parent = Some(wall_id);
        graph.insert(window);
        let children = graph.children(wall_id);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].category, EntityCategory::Window);
    }
}
