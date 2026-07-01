//! ui_framework — command architecture, tool palette, viewport controls.
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

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Tool {
    Select,
    DrawWall,
    DrawSlab,
    DrawBeam,
    DrawColumn,
    PlaceDoor,
    PlaceWindow,
    Measure,
    #[default]
    None,
}

#[derive(Debug, Default)]
pub struct UndoStack {
    history: Vec<String>,
    position: usize,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            history: vec![],
            position: 0,
        }
    }
    pub fn push(&mut self, d: &str) {
        if self.position < self.history.len() {
            self.history.truncate(self.position);
        }
        self.history.push(d.into());
        self.position = self.history.len();
    }
    pub fn undo(&mut self) -> Option<&str> {
        if self.position > 0 {
            self.position -= 1;
            Some(&self.history[self.position])
        } else {
            None
        }
    }
    pub fn redo(&mut self) -> Option<&str> {
        if self.position < self.history.len() {
            let d = &self.history[self.position];
            self.position += 1;
            Some(d)
        } else {
            None
        }
    }
    pub fn can_undo(&self) -> bool {
        self.position > 0
    }
    pub fn can_redo(&self) -> bool {
        self.position < self.history.len()
    }
    pub fn len(&self) -> usize {
        self.history.len()
    }
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ViewportState {
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
    pub orbit_angle_x: f64,
    pub orbit_angle_y: f64,
}

impl Default for ViewportState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan_x: 0.0,
            pan_y: 0.0,
            orbit_angle_x: 0.0,
            orbit_angle_y: 0.0,
        }
    }
}

impl ViewportState {
    pub fn zoom_in(&mut self, f: f64) {
        self.zoom *= f;
    }
    pub fn zoom_out(&mut self, f: f64) {
        self.zoom /= f;
    }
    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.pan_x += dx;
        self.pan_y += dy;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn undo_redo() {
        let mut s = UndoStack::new();
        s.push("a");
        s.push("b");
        assert_eq!(s.undo(), Some("b"));
        assert_eq!(s.redo(), Some("b"));
    }
    #[test]
    fn viewport_zoom() {
        let mut vp = ViewportState::default();
        vp.zoom_in(2.0);
        assert!((vp.zoom - 2.0).abs() < 1e-9);
        vp.zoom_out(2.0);
        assert!((vp.zoom - 1.0).abs() < 1e-9);
    }
}
