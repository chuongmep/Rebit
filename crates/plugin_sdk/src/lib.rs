//! plugin_sdk — stable API surface for third-party plugin development.
//!
//! # Phase B additions
//! - Plugin lifecycle (load, initialize, run, unload)
//! - Plugin manifest validation
//! - SDK version compatibility check

use geometry_kernel::Point3D;

/// Plugin manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
    pub min_sdk_version: u32,
    pub max_sdk_version: u32,
}

impl PluginManifest {
    /// Check if this manifest is compatible with a given SDK version.
    pub fn is_compatible(&self, sdk_version: u32) -> bool {
        sdk_version >= self.min_sdk_version && sdk_version <= self.max_sdk_version
    }
}

/// Plugin lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    Unloaded,
    Loaded,
    Initialized,
    Running,
    Error,
}

/// Plugin context provided to plugins at runtime.
#[derive(Debug, Clone)]
pub struct PluginContext {
    pub api_version: u32,
}

impl Default for PluginContext {
    fn default() -> Self {
        Self { api_version: 1 }
    }
}

impl PluginContext {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new 3D point (plugin-safe geometry creation).
    pub fn create_point(&self, x: f64, y: f64, z: f64) -> Point3D {
        Point3D::new(x, y, z)
    }
}

/// A plugin instance.
#[derive(Debug, Clone)]
pub struct Plugin {
    pub manifest: PluginManifest,
    pub state: PluginState,
}

impl Plugin {
    /// Create a new plugin from a manifest.
    pub fn new(manifest: PluginManifest) -> Self {
        Self {
            manifest,
            state: PluginState::Unloaded,
        }
    }

    /// Load the plugin.
    pub fn load(&mut self) {
        self.state = PluginState::Loaded;
    }

    /// Initialize the plugin with a context.
    pub fn initialize(&mut self, _ctx: &PluginContext) {
        self.state = PluginState::Initialized;
    }

    /// Run the plugin.
    pub fn run(&mut self) {
        self.state = PluginState::Running;
    }

    /// Unload the plugin.
    pub fn unload(&mut self) {
        self.state = PluginState::Unloaded;
    }

    /// Check if the plugin is in a runnable state.
    pub fn is_runnable(&self) -> bool {
        matches!(self.state, PluginState::Initialized | PluginState::Running)
    }
}

/// Plugin registry managing installed plugins.
#[derive(Debug, Default)]
pub struct PluginRegistry {
    plugins: Vec<Plugin>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self { plugins: vec![] }
    }

    /// Register a new plugin.
    pub fn register(&mut self, plugin: Plugin) {
        self.plugins.push(plugin);
    }

    /// Get a plugin by name.
    pub fn get(&self, name: &str) -> Option<&Plugin> {
        self.plugins.iter().find(|p| p.manifest.name == name)
    }

    /// Load all compatible plugins.
    pub fn load_all(&mut self, sdk_version: u32) {
        for plugin in &mut self.plugins {
            if plugin.manifest.is_compatible(sdk_version) {
                plugin.load();
            }
        }
    }

    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_creates_point() {
        let ctx = PluginContext::new();
        let pt = ctx.create_point(1.0, 2.0, 3.0);
        assert!((pt.x.value - 1.0).abs() < 1e-9);
    }

    #[test]
    fn plugin_lifecycle() {
        let manifest = PluginManifest {
            name: "TestPlugin".into(),
            version: "1.0".into(),
            author: "dev".into(),
            min_sdk_version: 1,
            max_sdk_version: 3,
        };
        let mut plugin = Plugin::new(manifest);
        assert_eq!(plugin.state, PluginState::Unloaded);
        plugin.load();
        assert_eq!(plugin.state, PluginState::Loaded);
        plugin.initialize(&PluginContext::new());
        assert!(plugin.is_runnable());
    }

    #[test]
    fn manifest_compatibility() {
        let manifest = PluginManifest {
            name: "P".into(),
            version: "1.0".into(),
            author: "a".into(),
            min_sdk_version: 2,
            max_sdk_version: 5,
        };
        assert!(!manifest.is_compatible(1));
        assert!(manifest.is_compatible(2));
        assert!(manifest.is_compatible(5));
        assert!(!manifest.is_compatible(6));
    }
}
