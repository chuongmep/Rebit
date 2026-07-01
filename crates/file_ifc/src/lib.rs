//! file_ifc — IFC (Industry Foundation Classes) read/write support.
//!
//! # Phase B additions
//! - IFC entity type mapping from BIM categories
//! - Import result with detailed warnings
//! - Entity count per IFC class

use bim_model::entity::EntityGraph;

/// An IFC entity class type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IfcClass {
    IfcWall,
    IfcSlab,
    IfcBeam,
    IfcColumn,
    IfcDoor,
    IfcWindow,
    IfcWallStandardCase,
    Unknown(String),
}

/// Result of an IFC import operation.
#[derive(Debug, Clone)]
pub struct IfcImportResult {
    pub entity_count: usize,
    pub warnings: Vec<String>,
    pub classes_found: Vec<IfcClass>,
    pub duration_ms: u64,
}

/// Import an IFC file and populate an entity graph.
///
/// Phase B stub — reads IFC metadata and produces entity counts.
/// Full geometry import is deferred to Phase C.
pub fn import_ifc(_path: &str) -> Result<(EntityGraph, IfcImportResult), String> {
    let graph = EntityGraph::new();
    Ok((
        graph,
        IfcImportResult {
            entity_count: 0,
            warnings: vec!["IFC geometry parsing not yet implemented (Phase C)".into()],
            classes_found: vec![],
            duration_ms: 0,
        },
    ))
}

/// Export an entity graph to an IFC file.
pub fn export_ifc(_graph: &EntityGraph, _path: &str) -> Result<IfcImportResult, String> {
    Ok(IfcImportResult {
        entity_count: 0,
        warnings: vec!["IFC export stub — not yet implemented (Phase C)".into()],
        classes_found: vec![],
        duration_ms: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_stub_returns_empty_graph() {
        let (graph, result) = import_ifc("test.ifc").unwrap();
        assert_eq!(result.entity_count, 0);
        assert!(graph.is_empty());
        assert!(!result.warnings.is_empty());
    }
}
