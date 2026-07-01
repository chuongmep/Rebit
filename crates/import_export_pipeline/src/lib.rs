//! import_export_pipeline — job orchestration and conversion pipeline.
//!
//! # Phase B additions
//! - Job queue with priority levels
//! - Job lifecycle (Pending → Running → Done/Failed)
//! - Batch processing of multiple files

/// Pipeline job status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Pending,
    Running,
    Done,
    Failed,
}

impl JobStatus {
    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

/// Priority level for a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 0,
    Normal = 1,
    High = 2,
}

/// An import/export job.
#[derive(Debug, Clone)]
pub struct ImportJob {
    pub id: u64,
    pub path: String,
    pub status: JobStatus,
    pub priority: Priority,
    pub error: Option<String>,
    pub entity_count: usize,
    pub duration_ms: u64,
}

impl ImportJob {
    pub fn new(id: u64, path: &str) -> Self {
        Self {
            id,
            path: path.into(),
            status: JobStatus::Pending,
            priority: Priority::Normal,
            error: None,
            entity_count: 0,
            duration_ms: 0,
        }
    }

    /// Mark the job as running.
    pub fn start(&mut self) {
        self.status = JobStatus::Running;
    }

    /// Mark the job as done with a result.
    pub fn complete(&mut self, entity_count: usize, duration_ms: u64) {
        self.status = JobStatus::Done;
        self.entity_count = entity_count;
        self.duration_ms = duration_ms;
    }

    /// Mark the job as failed with an error message.
    pub fn fail(&mut self, error: &str) {
        self.status = JobStatus::Failed;
        self.error = Some(error.into());
    }
}

/// A simple job queue for processing import/export tasks.
#[derive(Debug, Default)]
pub struct JobQueue {
    jobs: Vec<ImportJob>,
    next_id: u64,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            jobs: vec![],
            next_id: 1,
        }
    }

    /// Enqueue a new job.
    pub fn enqueue(&mut self, path: &str, priority: Priority) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let mut job = ImportJob::new(id, path);
        job.priority = priority;
        self.jobs.push(job);
        id
    }

    /// Get the next pending job (highest priority first).
    pub fn dequeue(&mut self) -> Option<&mut ImportJob> {
        self.jobs
            .iter_mut()
            .filter(|j| j.status == JobStatus::Pending)
            .max_by_key(|j| j.priority)
    }

    /// Get a job by id.
    pub fn get(&self, id: u64) -> Option<&ImportJob> {
        self.jobs.iter().find(|j| j.id == id)
    }

    /// Get mutable access to a job.
    pub fn get_mut(&mut self, id: u64) -> Option<&mut ImportJob> {
        self.jobs.iter_mut().find(|j| j.id == id)
    }

    /// Count jobs by status.
    pub fn count_by_status(&self, status: JobStatus) -> usize {
        self.jobs.iter().filter(|j| j.status == status).count()
    }

    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_starts_pending() {
        assert_eq!(ImportJob::new(1, "t.ifc").status, JobStatus::Pending);
    }

    #[test]
    fn job_lifecycle() {
        let mut job = ImportJob::new(1, "t.ifc");
        job.start();
        assert_eq!(job.status, JobStatus::Running);
        job.complete(42, 100);
        assert_eq!(job.status, JobStatus::Done);
        assert_eq!(job.entity_count, 42);
    }

    #[test]
    fn job_queue_priority() {
        let mut queue = JobQueue::new();
        queue.enqueue("a.ifc", Priority::Normal);
        queue.enqueue("b.ifc", Priority::High);
        queue.enqueue("c.ifc", Priority::Low);
        let next = queue.dequeue().unwrap();
        assert_eq!(next.path, "b.ifc"); // High priority first
        assert_eq!(next.priority, Priority::High);
    }

    #[test]
    fn job_queue_status_count() {
        let mut queue = JobQueue::new();
        queue.enqueue("a.ifc", Priority::Normal);
        queue.enqueue("b.ifc", Priority::Normal);
        assert_eq!(queue.count_by_status(JobStatus::Pending), 2);
    }
}
