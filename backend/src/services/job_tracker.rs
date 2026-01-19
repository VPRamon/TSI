//! Job tracking for async schedule processing.
//!
//! This module provides a simple in-memory job tracker that stores progress logs
//! for background tasks like schedule upload and processing.

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// A single log entry with timestamp and message.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LogEntry {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Job status enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Running,
    Completed,
    Failed,
}

/// Job metadata and logs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Job {
    pub job_id: String,
    pub status: JobStatus,
    pub logs: Vec<LogEntry>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Result of the job (e.g., schedule_id if successful)
    pub result: Option<serde_json::Value>,
}

/// In-memory job tracker.
#[derive(Clone)]
pub struct JobTracker {
    jobs: Arc<RwLock<HashMap<String, Job>>>,
}

impl JobTracker {
    /// Create a new job tracker.
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new job and return its ID.
    pub fn create_job(&self) -> String {
        let job_id = Uuid::new_v4().to_string();
        let job = Job {
            job_id: job_id.clone(),
            status: JobStatus::Running,
            logs: vec![],
            created_at: chrono::Utc::now(),
            completed_at: None,
            result: None,
        };
        self.jobs.write().insert(job_id.clone(), job);
        job_id
    }

    /// Add a log entry to a job.
    pub fn log(&self, job_id: &str, level: LogLevel, message: impl Into<String>) {
        let mut jobs = self.jobs.write();
        if let Some(job) = jobs.get_mut(job_id) {
            job.logs.push(LogEntry {
                timestamp: chrono::Utc::now(),
                level,
                message: message.into(),
            });
        }
    }

    /// Mark a job as completed with optional result.
    pub fn complete_job(&self, job_id: &str, result: Option<serde_json::Value>) {
        let mut jobs = self.jobs.write();
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = JobStatus::Completed;
            job.completed_at = Some(chrono::Utc::now());
            job.result = result;
        }
    }

    /// Mark a job as failed.
    pub fn fail_job(&self, job_id: &str, error_message: impl Into<String>) {
        let mut jobs = self.jobs.write();
        if let Some(job) = jobs.get_mut(job_id) {
            job.status = JobStatus::Failed;
            job.completed_at = Some(chrono::Utc::now());
            job.logs.push(LogEntry {
                timestamp: chrono::Utc::now(),
                level: LogLevel::Error,
                message: error_message.into(),
            });
        }
    }

    /// Get a job by ID.
    pub fn get_job(&self, job_id: &str) -> Option<Job> {
        self.jobs.read().get(job_id).cloned()
    }

    /// Get all logs for a job.
    pub fn get_logs(&self, job_id: &str) -> Vec<LogEntry> {
        self.jobs
            .read()
            .get(job_id)
            .map(|job| job.logs.clone())
            .unwrap_or_default()
    }
}

impl Default for JobTracker {
    fn default() -> Self {
        Self::new()
    }
}
