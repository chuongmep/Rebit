//! gpu_backend — GPU abstraction layer for Vulkan/Metal compute and draw.
#![forbid(unsafe_code)]
/// GPU buffer handle (opaque in Phase A).
#[derive(Debug, Clone, Copy)]
pub struct GpuBufferId(pub u64);
/// GPU backend initialization stub.
pub fn init() -> Result<(), &'static str> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn init_succeeds() {
        assert!(init().is_ok());
    }
}
