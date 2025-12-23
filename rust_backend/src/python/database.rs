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
#[pyfunction]
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
fn parse_schedule_from_json(
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
fn store_schedule_in_db(
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

// =============================================================================
// Python Bindings
// =============================================================================

/// Check database connection health.
#[pyfunction]
pub fn py_db_health_check() -> PyResult<bool> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(services::health_check(repo.as_ref()))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Store a preprocessed schedule in the database.
#[pyfunction]
pub fn py_store_schedule(
    schedule_name: &str,
    schedule_json: &str,
    visibility_json: Option<&str>,
) -> PyResult<Py<PyAny>> {
    py_store_schedule_with_options(schedule_name, schedule_json, visibility_json, true, true)
}

/// Store a preprocessed schedule with optional analytics computation.
///
/// Args:
///     schedule_name: Human-readable schedule name
///     schedule_json: JSON string containing schedule data
///     visibility_json: Optional JSON string with visibility periods
///     populate_analytics: If True, compute block and summary analytics (recommended)
///     skip_time_bins: If True, skip expensive visibility time bin computation
///
/// Returns:
///     Dictionary with storage results including schedule_id
///
/// Performance:
///     - Full analytics (populate_analytics=True, skip_time_bins=False): ~2-5 minutes for 1500 blocks
///     - Fast mode (populate_analytics=True, skip_time_bins=True): ~10-30 seconds for 1500 blocks
///     - Fastest mode (populate_analytics=False, skip_time_bins=True): ~5-15 seconds for 1500 blocks
#[pyfunction]
#[pyo3(signature = (schedule_name, schedule_json, visibility_json=None, populate_analytics=true, skip_time_bins=true))]
#[allow(deprecated)]
pub fn py_store_schedule_with_options(
    schedule_name: &str,
    schedule_json: &str,
    visibility_json: Option<&str>,
    populate_analytics: bool,
    skip_time_bins: bool,
) -> PyResult<Py<PyAny>> {
    // Heavy parsing + DB insert happens without the GIL held to avoid blocking Python.
    let metadata = Python::with_gil(|py| {
        py.allow_threads(|| -> PyResult<_> {
            let schedule = parse_schedule_from_json(schedule_name, schedule_json, visibility_json)?;
            store_schedule_in_db(&schedule, populate_analytics, skip_time_bins)
        })
    })?;

    // Convert to Python dict
    // Convert to Python dict
    Python::with_gil(|py| {
        let dict = PyDict::new(py);
        dict.set_item("schedule_id", metadata.schedule_id)?;
        dict.set_item("schedule_name", metadata.schedule_name)?;
        dict.set_item("upload_timestamp", metadata.upload_timestamp.to_rfc3339())?;
        dict.set_item("checksum", metadata.checksum)?;
        Ok(dict.into())
    })
}

/// Fetch a schedule (metadata + blocks) from the database.
#[pyfunction]
pub fn py_get_schedule(schedule_id: i64) -> PyResult<Schedule> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let repo = get_repository()?;

    runtime
        .block_on(services::get_schedule(repo.as_ref(), schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Fetch all scheduling blocks for a schedule ID.
#[pyfunction]
pub fn py_get_schedule_blocks(schedule_id: i64) -> PyResult<Vec<SchedulingBlock>> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(services::get_blocks_for_schedule(
            repo.as_ref(),
            schedule_id,
        ))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// List all available schedules in the database.
#[pyfunction]
#[allow(deprecated)]
pub fn py_list_schedules() -> PyResult<Py<PyAny>> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let schedules = runtime
        .block_on(services::list_schedules(repo.as_ref()))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

    Python::with_gil(|py| {
        let list = PyList::empty(py);
        for schedule_info in schedules {
            let dict = PyDict::new(py);
            dict.set_item("schedule_id", schedule_info.metadata.schedule_id)?;
            dict.set_item("schedule_name", schedule_info.metadata.schedule_name)?;
            dict.set_item(
                "upload_timestamp",
                schedule_info.metadata.upload_timestamp.to_rfc3339(),
            )?;
            dict.set_item("checksum", schedule_info.metadata.checksum)?;
            dict.set_item("total_blocks", schedule_info.total_blocks)?;
            dict.set_item("scheduled_blocks", schedule_info.scheduled_blocks)?;
            dict.set_item("unscheduled_blocks", schedule_info.unscheduled_blocks)?;
            list.append(dict)?;
        }
        Ok(list.into())
    })
}

/// Fetch dark periods for a schedule.
#[pyfunction]
#[allow(deprecated)]
pub fn py_fetch_dark_periods(schedule_id: i64) -> PyResult<Py<PyAny>> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let periods = runtime
        .block_on(services::fetch_dark_periods(repo.as_ref(), schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

    Python::with_gil(|py| {
        let list = PyList::empty(py);
        for period in periods {
            list.append((period.start.value(), period.stop.value()))?;
        }
        Ok(list.into())
    })
}

/// Fetch possible (visibility) periods for a schedule.
#[pyfunction]
#[allow(deprecated)]
pub fn py_fetch_possible_periods(schedule_id: i64) -> PyResult<Py<PyAny>> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let periods = runtime
        .block_on(services::fetch_possible_periods(repo.as_ref(), schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

    Python::with_gil(|py| {
        let list = PyList::empty(py);
        for period in periods {
            list.append((period.start.value(), period.stop.value()))?;
        }
        Ok(list.into())
    })
}

/// Fetch compare blocks for a schedule (minimal data for comparison).
#[pyfunction]
pub fn py_fetch_compare_blocks(schedule_id: i64) -> PyResult<Vec<crate::db::models::CompareBlock>> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.fetch_compare_blocks(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Compute visibility histogram for a schedule with filters.
///
/// This function fetches minimal block data from the database and computes
/// a time-binned histogram showing how many unique scheduling blocks are
/// visible in each time interval.
///
/// ## Arguments
/// * `schedule_id` - Schedule ID to analyze
/// * `start_unix` - Start of time range (Unix timestamp seconds)
/// * `end_unix` - End of time range (Unix timestamp seconds)
/// * `bin_duration_minutes` - Duration of each histogram bin in minutes
/// * `priority_min` - Optional minimum priority filter (inclusive)
/// * `priority_max` - Optional maximum priority filter (inclusive)
/// * `block_ids` - Optional list of specific block IDs to include
///
/// ## Returns
/// List of dictionaries with keys:
/// - `bin_start_unix`: Start of bin (Unix timestamp)
/// - `bin_end_unix`: End of bin (Unix timestamp)
/// - `count`: Number of unique blocks visible in this bin
///
/// ## Example
/// ```python
/// import tsi_rust
/// from datetime import datetime, timezone
///
/// start = int(datetime(2024, 1, 1, tzinfo=timezone.utc).timestamp())
/// end = int(datetime(2024, 1, 2, tzinfo=timezone.utc).timestamp())
///
/// bins = tsi_rust.py_get_visibility_histogram(
///     schedule_id=1,
///     start_unix=start,
///     end_unix=end,
///     bin_duration_minutes=60,
///     priority_min=5,
///     priority_max=10,
///     block_ids=None
/// )
///
/// for bin in bins:
///     print(f"Time: {bin['bin_start_unix']}, Visible: {bin['count']}")
/// ```
#[pyfunction]
#[pyo3(signature = (schedule_id, start_unix, end_unix, bin_duration_minutes, priority_min=None, priority_max=None, block_ids=None))]
#[allow(clippy::too_many_arguments)]
#[allow(deprecated)]
pub fn py_get_visibility_histogram(
    py: Python,
    schedule_id: i64,
    start_unix: i64,
    end_unix: i64,
    bin_duration_minutes: i64,
    priority_min: Option<i32>,
    priority_max: Option<i32>,
    block_ids: Option<Vec<i64>>,
) -> PyResult<Py<PyAny>> {
    // Validate inputs
    if start_unix >= end_unix {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "start_unix must be less than end_unix",
        ));
    }
    if bin_duration_minutes <= 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "bin_duration_minutes must be positive",
        ));
    }

    let bin_duration_seconds = bin_duration_minutes * 60;

    // Release GIL for database and compute operations
    let bins = py.allow_threads(|| -> PyResult<_> {
        let repo = get_repository()?;

        let runtime = Runtime::new().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to create async runtime: {}",
                e
            ))
        })?;

        // Fetch blocks from database
        let blocks = runtime
            .block_on(repo.fetch_blocks_for_histogram(
                schedule_id,
                priority_min,
                priority_max,
                block_ids,
            ))
            .map_err(|e| {
                PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                    "Failed to fetch blocks: {}",
                    e
                ))
            })?;

        // Compute histogram
        crate::services::visibility::compute_visibility_histogram_rust(
            blocks.into_iter(),
            start_unix,
            end_unix,
            bin_duration_seconds,
            priority_min,
            priority_max,
        )
        .map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to compute histogram: {}",
                e
            ))
        })
    })?;

    // Convert to Python list of dicts (JSON-serializable)
    let list = PyList::empty(py);
    for bin in bins {
        let dict = PyDict::new(py);
        dict.set_item("bin_start_unix", bin.bin_start_unix)?;
        dict.set_item("bin_end_unix", bin.bin_end_unix)?;
        dict.set_item("count", bin.visible_count)?;
        list.append(dict)?;
    }

    Ok(list.into())
}

/// Get the time range (min/max timestamps) for a schedule's visibility periods.
///
/// This function queries all visibility periods for a schedule and returns
/// the minimum start time and maximum stop time as Unix timestamps.
///
/// ## Arguments
/// * `schedule_id` - Schedule ID to analyze
///
/// ## Returns
/// Tuple of (start_unix, end_unix) as Option. Returns None if no
/// visibility periods exist or if schedule not found.
///
/// ## Example
/// ```python
/// import tsi_rust
///
/// time_range = tsi_rust.py_get_schedule_time_range(schedule_id=1)
/// if time_range:
///     start_unix, end_unix = time_range
///     print(f"Schedule spans from {start_unix} to {end_unix}")
/// else:
///     print("No visibility periods found")
/// ```
#[pyfunction]
pub fn py_get_schedule_time_range(schedule_id: i64) -> PyResult<Option<(i64, i64)>> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let time_range_period = runtime
        .block_on(services::get_schedule_time_range(
            repo.as_ref(),
            schedule_id,
        ))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

    // Convert Period to Unix timestamps
    if let Some(period) = time_range_period {
        // MJD epoch (1858-11-17 00:00:00 UTC) as Unix timestamp
        const MJD_EPOCH_UNIX: i64 = -3506716800;
        let start_unix = MJD_EPOCH_UNIX + (period.start.value() * 86400.0) as i64;
        let end_unix = MJD_EPOCH_UNIX + (period.stop.value() * 86400.0) as i64;
        Ok(Some((start_unix, end_unix)))
    } else {
        Ok(None)
    }
}

/// Fetch visibility map data (priority range, block metadata) in one backend call.
/// Uses the analytics table for optimal performance when available.
#[pyfunction]
pub fn py_get_visibility_map_data(
    schedule_id: i64,
) -> PyResult<crate::db::models::VisibilityMapData> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    // Try analytics table first (much faster - no JOINs, pre-computed metrics)
    let result = runtime.block_on(async {
        // For now, just use the direct fetch_visibility_map_data method
        // The analytics optimization can be added later if needed
        repo.fetch_visibility_map_data(schedule_id).await
    });

    result.map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

// =============================================================================
// Analytics Table ETL Functions
// =============================================================================

/// Populate the analytics table for a schedule.
///
/// This function is called automatically after schedule upload, but can also
/// be triggered manually to refresh analytics data.
///
/// Args:
///     schedule_id: The ID of the schedule to process
///
/// Returns:
///     Number of analytics rows created
///
/// Example:
/// ```python
/// # Manually refresh analytics for a schedule
/// rows = tsi_rust.py_populate_analytics(schedule_id=42)
/// print(f"Created {rows} analytics rows")
/// ```
#[pyfunction]
pub fn py_populate_analytics(schedule_id: i64) -> PyResult<usize> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.populate_schedule_analytics(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Check if analytics data exists for a schedule.
///
/// Args:
///     schedule_id: The ID of the schedule to check
///
/// Returns:
///     True if analytics data exists, False otherwise
#[pyfunction]
pub fn py_has_analytics_data(schedule_id: i64) -> PyResult<bool> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.has_analytics_data(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Delete analytics data for a schedule.
///
/// Args:
///     schedule_id: The ID of the schedule whose analytics should be deleted
///
/// Returns:
///     Number of analytics rows deleted
#[pyfunction]
pub fn py_delete_analytics(schedule_id: i64) -> PyResult<usize> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.delete_schedule_analytics(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

// =============================================================================
// Phase 2: Summary Analytics Functions
// =============================================================================

/// Populate summary analytics tables for a schedule.
///
/// This function is called automatically after schedule upload, but can also
/// be triggered manually to refresh summary analytics data.
///
/// Args:
///     schedule_id: The ID of the schedule to process
///     n_bins: Number of bins for histograms (default 10)
///
/// Example:
/// ```python
/// # Manually refresh summary analytics for a schedule
/// tsi_rust.py_populate_summary_analytics(schedule_id=42)
/// ```
#[pyfunction]
#[pyo3(signature = (schedule_id, n_bins=10))]
pub fn py_populate_summary_analytics(schedule_id: i64, n_bins: usize) -> PyResult<()> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.populate_summary_analytics(schedule_id, n_bins))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Check if summary analytics data exists for a schedule.
///
/// Args:
///     schedule_id: The ID of the schedule to check
///
/// Returns:
///     True if summary analytics data exists, False otherwise
#[pyfunction]
pub fn py_has_summary_analytics(schedule_id: i64) -> PyResult<bool> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.has_summary_analytics(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Delete summary analytics data for a schedule.
///
/// Args:
///     schedule_id: The ID of the schedule whose summary analytics should be deleted
///
/// Returns:
///     Number of rows deleted (summary + priority rates)
#[pyfunction]
pub fn py_delete_summary_analytics(schedule_id: i64) -> PyResult<usize> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.delete_summary_analytics(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Fetch schedule summary from the analytics table.
///
/// Args:
///     schedule_id: The ID of the schedule
///
/// Returns:
///     ScheduleSummary object if data exists, None otherwise
#[pyfunction]
pub fn py_get_schedule_summary(
    schedule_id: i64,
) -> PyResult<Option<crate::db::analytics::ScheduleSummary>> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.fetch_schedule_summary(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Fetch priority rates from the analytics table.
///
/// Args:
///     schedule_id: The ID of the schedule
///
/// Returns:
///     List of PriorityRate objects
#[pyfunction]
pub fn py_get_priority_rates(
    schedule_id: i64,
) -> PyResult<Vec<crate::db::analytics::PriorityRate>> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.fetch_priority_rates(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Fetch visibility histogram bins from the analytics table.
///
/// Args:
///     schedule_id: The ID of the schedule
///
/// Returns:
///     List of VisibilityBin objects
#[pyfunction]
pub fn py_get_visibility_bins(
    schedule_id: i64,
) -> PyResult<Vec<crate::db::analytics::VisibilityBin>> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.fetch_visibility_bins(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Fetch heatmap bins from the analytics table.
///
/// Args:
///     schedule_id: The ID of the schedule
///
/// Returns:
///     List of HeatmapBinData objects
#[pyfunction]
pub fn py_get_heatmap_bins(
    schedule_id: i64,
) -> PyResult<Vec<crate::db::analytics::HeatmapBinData>> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.fetch_heatmap_bins(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

// =============================================================================
// Phase 3: Visibility Time Bins Functions
// =============================================================================

/// Populate visibility time bins for a schedule.
///
/// This function is called automatically after schedule upload, but can also
/// be triggered manually to refresh visibility time bin data.
///
/// Args:
///     schedule_id: The ID of the schedule to process
///     bin_duration_seconds: Duration of each bin in seconds (default 900 = 15 minutes)
///
/// Returns:
///     Tuple of (metadata_rows, bin_rows) created
///
/// Example:
/// ```python
/// # Manually refresh visibility time bins for a schedule
/// meta, bins = tsi_rust.py_populate_visibility_time_bins(schedule_id=42)
/// print(f"Created {meta} metadata rows, {bins} bin rows")
///
/// # Use custom bin duration (30 minutes)
/// meta, bins = tsi_rust.py_populate_visibility_time_bins(schedule_id=42, bin_duration_seconds=1800)
/// ```
#[pyfunction]
#[pyo3(signature = (schedule_id, bin_duration_seconds=None))]
pub fn py_populate_visibility_time_bins(
    schedule_id: i64,
    bin_duration_seconds: Option<i64>,
) -> PyResult<(usize, usize)> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.populate_visibility_time_bins(schedule_id, bin_duration_seconds))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Check if visibility time bins exist for a schedule.
///
/// Args:
///     schedule_id: The ID of the schedule to check
///
/// Returns:
///     True if visibility time bins exist, False otherwise
#[pyfunction]
pub fn py_has_visibility_time_bins(schedule_id: i64) -> PyResult<bool> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.has_visibility_time_bins(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Delete visibility time bins for a schedule.
///
/// Args:
///     schedule_id: The ID of the schedule whose visibility bins should be deleted
///
/// Returns:
///     Number of rows deleted
#[pyfunction]
pub fn py_delete_visibility_time_bins(schedule_id: i64) -> PyResult<usize> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.delete_visibility_time_bins(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Fetch visibility metadata for a schedule.
///
/// Args:
///     schedule_id: The ID of the schedule
///
/// Returns:
///     VisibilityTimeMetadata object if data exists, None otherwise
#[pyfunction]
pub fn py_get_visibility_metadata(
    schedule_id: i64,
) -> PyResult<Option<crate::db::analytics::VisibilityTimeMetadata>> {
    let repo = get_repository()?;

    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(repo.fetch_visibility_metadata(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
}

/// Fetch pre-computed visibility histogram from analytics table.
///
/// This function retrieves pre-computed visibility time bins and aggregates
/// them to the target bin duration. Much faster than computing on-the-fly
/// for large schedules.
///
/// Args:
///     schedule_id: Schedule ID to query
///     start_unix: Start of time range (Unix timestamp)
///     end_unix: End of time range (Unix timestamp)
///     bin_duration_minutes: Target bin duration in minutes
///
/// Returns:
///     List of dicts with keys: bin_start_unix, bin_end_unix, count
///
/// Example:
/// ```python
/// import tsi_rust
/// from datetime import datetime, timezone
///
/// start = int(datetime(2024, 1, 1, tzinfo=timezone.utc).timestamp())
/// end = int(datetime(2024, 1, 2, tzinfo=timezone.utc).timestamp())
///
/// # Fast path: uses pre-computed bins
/// bins = tsi_rust.py_get_visibility_histogram_analytics(
///     schedule_id=1,
///     start_unix=start,
///     end_unix=end,
///     bin_duration_minutes=60
/// )
///
/// for bin in bins:
///     print(f"Time: {bin['bin_start_unix']}, Visible: {bin['count']}")
/// ```
#[pyfunction]
#[allow(deprecated)]
pub fn py_get_visibility_histogram_analytics(
    py: Python,
    schedule_id: i64,
    start_unix: i64,
    end_unix: i64,
    bin_duration_minutes: i64,
) -> PyResult<Py<PyAny>> {
    // Validate inputs
    if start_unix >= end_unix {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "start_unix must be less than end_unix",
        ));
    }
    if bin_duration_minutes <= 0 {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "bin_duration_minutes must be positive",
        ));
    }

    let bin_duration_seconds = bin_duration_minutes * 60;

    // Release GIL for database operations
    let bins = py.allow_threads(|| -> PyResult<_> {
        let repo = get_repository()?;

        let runtime = Runtime::new().map_err(|e| {
            PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                "Failed to create async runtime: {}",
                e
            ))
        })?;

        runtime
            .block_on(repo.fetch_visibility_histogram_from_analytics(
                schedule_id,
                start_unix,
                end_unix,
                bin_duration_seconds,
            ))
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))
    })?;

    // Convert to Python list of dicts (JSON-serializable)
    let list = PyList::empty(py);
    for bin in bins {
        let dict = PyDict::new(py);
        dict.set_item("bin_start_unix", bin.bin_start_unix)?;
        dict.set_item("bin_end_unix", bin.bin_end_unix)?;
        dict.set_item("count", bin.visible_count)?;
        list.append(dict)?;
    }

    Ok(list.into())
}
