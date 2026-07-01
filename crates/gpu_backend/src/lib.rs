//! gpu_backend — GPU abstraction layer for Vulkan/Metal compute and draw.
//!
//! # Phase B additions
//! - Buffer management (create, upload, readback)
//! - Shader compilation stubs (vertex/fragment)
//! - Draw command submission with pipeline state

#![forbid(unsafe_code)]

/// A GPU buffer handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuBufferId(pub u64);

/// A shader program handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuShaderId(pub u64);

/// A vertex buffer with typed data.
#[derive(Debug, Clone)]
pub struct VertexBuffer {
    pub id: GpuBufferId,
    pub vertex_count: usize,
    pub stride_bytes: usize,
}

/// GPU backend with buffer and shader management.
#[derive(Debug, Default)]
pub struct GpuBackend {
    buffers: Vec<GpuBufferId>,
    shaders: Vec<GpuShaderId>,
    next_buffer_id: u64,
    next_shader_id: u64,
}

impl GpuBackend {
    /// Initialize the GPU backend.
    pub fn init() -> Result<Self, &'static str> {
        Ok(Self {
            buffers: vec![],
            shaders: vec![],
            next_buffer_id: 1,
            next_shader_id: 1,
        })
    }

    /// Create a vertex buffer with given vertex count.
    pub fn create_vertex_buffer(&mut self, vertex_count: usize) -> VertexBuffer {
        let id = GpuBufferId(self.next_buffer_id);
        self.next_buffer_id += 1;
        self.buffers.push(id);
        VertexBuffer {
            id,
            vertex_count,
            stride_bytes: 32, // 8 floats × 4 bytes
        }
    }

    /// Compile a vertex/fragment shader pair (stub).
    pub fn compile_shader(&mut self, _source: &str) -> Result<GpuShaderId, &'static str> {
        let id = GpuShaderId(self.next_shader_id);
        self.next_shader_id += 1;
        self.shaders.push(id);
        Ok(id)
    }

    /// Submit a draw command for a buffer with a shader.
    pub fn draw(&self, _buffer: &VertexBuffer, _shader: &GpuShaderId) {
        // Phase B stub — no actual GPU submission.
    }

    /// Number of buffers allocated.
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }

    /// Number of shaders compiled.
    pub fn shader_count(&self) -> usize {
        self.shaders.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_succeeds() {
        assert!(GpuBackend::init().is_ok());
    }

    #[test]
    fn create_vertex_buffer() {
        let mut backend = GpuBackend::init().unwrap();
        let vb = backend.create_vertex_buffer(100);
        assert_eq!(backend.buffer_count(), 1);
        assert_eq!(vb.vertex_count, 100);
    }

    #[test]
    fn compile_shader_allocates_id() {
        let mut backend = GpuBackend::init().unwrap();
        let _sid = backend.compile_shader("void main() {}").unwrap();
        assert_eq!(backend.shader_count(), 1);
    }
}
