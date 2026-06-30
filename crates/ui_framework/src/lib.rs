//! ui_framework — command architecture, dockable panels, tool states.
#![forbid(unsafe_code)]
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Command {
    pub id: String,
    pub label: String,
    pub shortcut: Option<String>,
}
#[derive(Debug, Clone, Default)]
pub struct CommandRegistry {
    commands: HashMap<String, Command>,
}
impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }
    pub fn register(&mut self, cmd: Command) {
        self.commands.insert(cmd.id.clone(), cmd);
    }
    pub fn get(&self, id: &str) -> Option<&Command> {
        self.commands.get(id)
    }
    pub fn len(&self) -> usize {
        self.commands.len()
    }
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registry_register_command() {
        let mut r = CommandRegistry::new();
        r.register(Command {
            id: "undo".into(),
            label: "Undo".into(),
            shortcut: Some("Ctrl+Z".into()),
        });
        assert_eq!(r.len(), 1);
        assert!(!r.is_empty());
        assert!(r.get("undo").is_some());
    }
}
