//! desktop_shell — native app shell, file persistence, update system.
#![forbid(unsafe_code)]
#[derive(Debug, Default)]
pub struct DesktopApp {
    pub title: String,
}
impl DesktopApp {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn app_creation() {
        assert_eq!(DesktopApp::new("Rebit").title, "Rebit");
    }
}
