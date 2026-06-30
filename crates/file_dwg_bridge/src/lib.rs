//! file_dwg_bridge — DWG file import/export bridge.
#![forbid(unsafe_code)]
#[derive(Debug, Clone)]
pub struct DwgImportResult {
    pub entity_count: usize,
}
pub fn import_dwg(_path: &str) -> Result<DwgImportResult, &'static str> {
    Ok(DwgImportResult { entity_count: 0 })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dwg_import_stub() {
        assert!(import_dwg("t.dwg").is_ok());
    }
}
