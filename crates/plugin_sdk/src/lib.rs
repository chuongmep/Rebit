//! plugin_sdk — stable API surface for third-party plugin development.
#![forbid(unsafe_code)]
use geometry_kernel::Point3D;

#[derive(Debug, Clone)]
pub struct PluginManifest {
    pub name: String,
    pub version: String,
    pub author: String,
}

#[derive(Debug, Clone, Default)]
pub struct PluginContext {
    pub api_version: u32,
}
impl PluginContext {
    pub fn new() -> Self {
        Self { api_version: 1 }
    }
    pub fn create_point(&self, x: f64, y: f64, z: f64) -> Point3D {
        Point3D::new(x, y, z)
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
}
