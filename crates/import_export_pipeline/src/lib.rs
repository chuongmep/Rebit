//! import_export_pipeline — job orchestration and conversion pipeline.
#![forbid(unsafe_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Done,
    Failed,
}
#[derive(Debug, Clone)]
pub struct ImportJob {
    pub id: u64,
    pub path: String,
    pub status: JobStatus,
}
impl ImportJob {
    pub fn new(id: u64, path: &str) -> Self {
        Self {
            id,
            path: path.into(),
            status: JobStatus::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn job_starts_pending() {
        assert_eq!(ImportJob::new(1, "t.ifc").status, JobStatus::Pending);
    }
}
