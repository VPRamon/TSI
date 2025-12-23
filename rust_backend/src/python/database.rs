//! Python bindings for database operations.

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::sync::OnceLock;
use tokio::runtime::Runtime;

use crate::db::{
    models::{Schedule, SchedulingBlock},
    repositories::LocalRepository,
    repository::{analytics::AnalyticsRepository, visualization::VisualizationRepository},
    services,
};

// Global repository instance initialized once
// Using LocalRepository for in-memory storage by default
static REPOSITORY: OnceLock<std::sync::Arc<LocalRepository>> = OnceLock::new();

/// Get a reference to the global repository instance.
///
/// This function is used internally by database operations and validation reporting.
/// It returns an error if the repository hasn't been initialized via `py_init_database()`.
pub(crate) fn get_repository() -> PyResult<&'static std::sync::Arc<LocalRepository>> {
    REPOSITORY.get().ok_or_else(|| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
            "Database not initialized. Call py_init_database() first.",
        )
    })
}

/// Initialize database repository.
///
/// By default, this creates an in-memory local repository (suitable for local development
/// and testing). No database configuration is required.
///
/// This function is idempotent - calling it multiple times is safe and will
/// simply return success if already initialized.
// #[pyfunction] - removed, function now internal only
pub fn py_init_database() -> PyResult<()> {
    // Check if already initialized
    if REPOSITORY.get().is_some() {
        // Already initialized, this is fine - just return success
        return Ok(());
    }

    // Create local in-memory repository (no database required)
    let repo = std::sync::Arc::new(LocalRepository::new());

    // Try to set - if it fails (race condition), that's okay
    let _ = REPOSITORY.set(repo);

    Ok(())
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Parse schedule from JSON strings.
///
/// This helper function reads dark periods and parses the schedule JSON,
/// setting the schedule name.
pub(crate) fn parse_schedule_from_json(
    schedule_name: &str,
    schedule_json: &str,
    visibility_json: Option<&str>,
) -> PyResult<Schedule> {
    let dark_periods = std::fs::read_to_string("data/dark_periods.json").map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to read dark_periods.json: {}",
            e
        ))
    })?;

    let mut schedule: Schedule = crate::db::models::schedule::parse_schedule_json_str(
        schedule_json,
        visibility_json,
        dark_periods.as_str(),
    )
    .map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to parse schedule: {}",
            e
        ))
    })?;
    schedule.name = schedule_name.to_string();

    Ok(schedule)
}

/// Store schedule in database with options.
///
/// This helper function handles the async runtime creation and repository
/// interaction for storing a schedule.
pub(crate) fn store_schedule_in_db(
    schedule: &Schedule,
    populate_analytics: bool,
    skip_time_bins: bool,
) -> PyResult<crate::db::models::ScheduleMetadata> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let repo = get_repository()?;
    runtime
        .block_on(services::store_schedule_with_options(
            repo.as_ref(),
            schedule,
            populate_analytics,
            skip_time_bins,
        ))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

// Python bindings removed: all `py_*` functions were deleted per request.
// Helper functions and repository (`get_repository`, `parse_schedule_from_json`,
// `store_schedule_in_db`, etc.) remain for internal Rust usage.

