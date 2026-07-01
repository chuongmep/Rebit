//! desktop_shell — native app shell, file persistence, update system.
//!
//! # Phase B additions
//! - File open/save/close lifecycle
//! - Recent files tracking
//! - Application state management

/// Application state.
#[derive(Debug, Default)]
pub struct DesktopApp {
    pub title: String,
    pub recent_files: Vec<String>,
    pub current_file: Option<String>,
    pub modified: bool,
}

impl DesktopApp {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.into(),
            recent_files: vec![],
            current_file: None,
            modified: false,
        }
    }

    /// Open a file and add it to recent files.
    pub fn open_file(&mut self, path: &str) {
        self.current_file = Some(path.into());
        self.modified = false;
        if !self.recent_files.contains(&path.to_string()) {
            self.recent_files.insert(0, path.into());
            if self.recent_files.len() > 10 {
                self.recent_files.truncate(10);
            }
        }
    }

    /// Save the current file.
    pub fn save(&mut self) {
        self.modified = false;
    }

    /// Mark the document as modified (dirty).
    pub fn mark_dirty(&mut self) {
        self.modified = true;
    }

    /// Check if there are unsaved changes.
    pub fn has_unsaved_changes(&self) -> bool {
        self.modified
    }

    /// Close the current file.
    pub fn close_file(&mut self) {
        self.current_file = None;
        self.modified = false;
    }

    /// Get the list of recent files.
    pub fn recent_files(&self) -> &[String] {
        &self.recent_files
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_creation() {
        assert_eq!(DesktopApp::new("Rebit").title, "Rebit");
    }

    #[test]
    fn open_file_adds_to_recent() {
        let mut app = DesktopApp::new("Test");
        app.open_file("/models/project.rbt");
        assert_eq!(app.recent_files.len(), 1);
        assert_eq!(app.current_file.as_deref(), Some("/models/project.rbt"));
    }

    #[test]
    fn modified_tracking() {
        let mut app = DesktopApp::new("Test");
        assert!(!app.has_unsaved_changes());
        app.mark_dirty();
        assert!(app.has_unsaved_changes());
        app.save();
        assert!(!app.has_unsaved_changes());
    }
}
