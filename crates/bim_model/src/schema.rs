//! BIM schema — versioned metadata, definitions, and migration stubs.
//!
//! Phase A defines the schema envelope with a version identifier and entity
//! category registry.  Full schema validation and migration tooling is
//! deferred to Phase B.

// ---------------------------------------------------------------------------
// Schema version
// ---------------------------------------------------------------------------

/// A schema version following SemVer semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaVersion {
    /// Major version — increments when backwards-incompatible changes occur.
    pub major: u32,
    /// Minor version — increments for backwards-compatible additions.
    pub minor: u32,
    /// Patch version — increments for backwards-compatible bug fixes.
    pub patch: u32,
}

impl SchemaVersion {
    /// Create a new schema version.
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    /// The current schema version for Phase A.
    pub const CURRENT: Self = Self::new(0, 1, 0);
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "v{}.{}.{}", self.major, self.minor, self.patch)
    }
}

// ---------------------------------------------------------------------------
// Schema definition
// ---------------------------------------------------------------------------

/// The BIM schema — metadata about the model format and structure.
#[derive(Debug, Clone)]
pub struct Schema {
    /// Schema version.
    pub version: SchemaVersion,
    /// Human-readable name of this schema.
    pub name: String,
    /// Description of the schema purpose.
    pub description: String,
    /// Categories defined in this schema.
    pub categories: Vec<CategoryDefinition>,
}

impl Schema {
    /// Create a new schema with the current version.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            version: SchemaVersion::CURRENT,
            name: name.into(),
            description: description.into(),
            categories: Vec::new(),
        }
    }

    /// Register an entity category definition.
    pub fn register_category(&mut self, category: CategoryDefinition) {
        self.categories.push(category);
    }
}

impl Default for Schema {
    fn default() -> Self {
        let mut schema = Self::new(
            "Rebit Core Schema",
            "Phase A BIM data model for architectural elements",
        );
        schema.register_category(CategoryDefinition {
            name: "Wall".into(),
            description: "A vertical planar element that defines and separates spaces".into(),
            has_geometry: true,
        });
        schema.register_category(CategoryDefinition {
            name: "Slab".into(),
            description: "A horizontal planar element (floor or roof)".into(),
            has_geometry: true,
        });
        schema.register_category(CategoryDefinition {
            name: "Beam".into(),
            description: "A horizontal or sloped structural member".into(),
            has_geometry: true,
        });
        schema.register_category(CategoryDefinition {
            name: "Column".into(),
            description: "A vertical structural member".into(),
            has_geometry: true,
        });
        schema.register_category(CategoryDefinition {
            name: "Door".into(),
            description: "An opening element for access".into(),
            has_geometry: true,
        });
        schema.register_category(CategoryDefinition {
            name: "Window".into(),
            description: "An opening element for light and ventilation".into(),
            has_geometry: true,
        });
        schema
    }
}

// ---------------------------------------------------------------------------
// Category definition
// ---------------------------------------------------------------------------

/// Metadata for a single entity category in the schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CategoryDefinition {
    /// Category name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Whether entities of this category carry geometry.
    pub has_geometry: bool,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_display() {
        let v = SchemaVersion::new(1, 2, 3);
        assert_eq!(v.to_string(), "v1.2.3");
    }

    #[test]
    fn schema_current_version_is_defined() {
        assert_eq!(SchemaVersion::CURRENT.major, 0);
        assert_eq!(SchemaVersion::CURRENT.minor, 1);
    }

    #[test]
    fn default_schema_has_six_categories() {
        let schema = Schema::default();
        assert_eq!(schema.categories.len(), 6);
        assert_eq!(schema.categories[0].name, "Wall");
        assert_eq!(schema.categories[5].name, "Window");
    }

    #[test]
    fn schema_register_category() {
        let mut schema = Schema::new("Test", "Test schema");
        assert!(schema.categories.is_empty());
        schema.register_category(CategoryDefinition {
            name: "TestCategory".into(),
            description: "A test".into(),
            has_geometry: false,
        });
        assert_eq!(schema.categories.len(), 1);
    }
}
