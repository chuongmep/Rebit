//! file_ifc — IFC (Industry Foundation Classes) read/write support.
#![forbid(unsafe_code)]
use bim_model::entity::EntityGraph;

#[derive(Debug, Clone)]
pub struct IfcImportResult {
    pub entity_count: usize,
    pub warnings: Vec<String>,
}

pub fn import_ifc(_path: &str) -> Result<(EntityGraph, IfcImportResult), &'static str> {
    Ok((
        EntityGraph::new(),
        IfcImportResult {
            entity_count: 0,
            warnings: vec![],
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn import_stub_returns_empty() {
        assert!(import_ifc("t.ifc").is_ok());
    }
}
