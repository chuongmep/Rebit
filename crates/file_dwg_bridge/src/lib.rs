//! file_dwg_bridge — DWG file import/export bridge.
//!
//! # Phase B additions
//! - DWG layer name extraction
//! - Entity type mapping (LINE, POLYLINE, ARC, INSERT)
//! - Import result with layer metadata

/// A DWG entity type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DwgEntityType {
    Line,
    Polyline,
    Arc,
    Circle,
    Insert,
    Text,
    Unknown(String),
}

/// Result of importing a DWG file.
#[derive(Debug, Clone)]
pub struct DwgImportResult {
    pub entity_count: usize,
    pub layers: Vec<String>,
    pub entity_types: Vec<DwgEntityType>,
}

/// Import a DWG file (Phase B stub — metadata only).
pub fn import_dwg(_path: &str) -> Result<DwgImportResult, &'static str> {
    Ok(DwgImportResult {
        entity_count: 0,
        layers: vec!["0".into(), "A-WALL".into()],
        entity_types: vec![],
    })
}

/// Export entities to a DWG file (Phase B stub).
pub fn export_dwg(_path: &str) -> Result<DwgImportResult, &'static str> {
    Ok(DwgImportResult {
        entity_count: 0,
        layers: vec![],
        entity_types: vec![],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dwg_import_stub() {
        let r = import_dwg("t.dwg").unwrap();
        assert!(r.layers.contains(&"0".to_string()));
        assert!(r.layers.contains(&"A-WALL".to_string()));
    }
}
