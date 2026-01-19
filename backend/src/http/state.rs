//! Application state for the HTTP server.

use std::sync::Arc;
use crate::db::repository::FullRepository;
use crate::services::job_tracker::JobTracker;

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Repository instance for database operations
    pub repository: Arc<dyn FullRepository>,
    /// Job tracker for async schedule processing
    pub job_tracker: JobTracker,
}

impl AppState {
    /// Create a new application state with the given repository.
    pub fn new(repository: Arc<dyn FullRepository>) -> Self {
        Self {
            repository,
            job_tracker: JobTracker::new(),
        }
    }
}
