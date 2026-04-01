//! Application state for the HTTP server.

use crate::db::repository::FullRepository;
use crate::services::job_tracker::JobTracker;
use crate::services::{default_schedule_import_adapter, ScheduleImportAdapter};
use std::sync::Arc;

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Repository instance for database operations
    pub repository: Arc<dyn FullRepository>,
    /// Active import adapter for uploaded payloads
    pub import_adapter: Arc<dyn ScheduleImportAdapter>,
    /// Job tracker for async schedule processing
    pub job_tracker: JobTracker,
}

impl AppState {
    /// Create a new application state with the given repository.
    pub fn new(repository: Arc<dyn FullRepository>) -> Self {
        Self::with_import_adapter(repository, default_schedule_import_adapter())
    }

    /// Create application state with an explicitly provided import adapter.
    pub fn with_import_adapter(
        repository: Arc<dyn FullRepository>,
        import_adapter: Arc<dyn ScheduleImportAdapter>,
    ) -> Self {
        Self {
            repository,
            import_adapter,
            job_tracker: JobTracker::new(),
        }
    }
}
