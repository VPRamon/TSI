//! Application state for the HTTP server.

use std::sync::Arc;
use crate::db::repository::FullRepository;

/// Shared application state passed to all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Repository instance for database operations
    pub repository: Arc<dyn FullRepository>,
}

impl AppState {
    /// Create a new application state with the given repository.
    pub fn new(repository: Arc<dyn FullRepository>) -> Self {
        Self { repository }
    }
}
