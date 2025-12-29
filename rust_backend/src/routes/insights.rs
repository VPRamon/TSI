use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

// =========================================================
// Insights types + route
// =========================================================

/// Block data for insights analysis.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsBlock {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub total_visibility_hours: qtty::Hours,
    pub requested_hours: qtty::Hours,
    pub elevation_range_deg: qtty::Degrees,
    pub scheduled: bool,
    pub scheduled_start_mjd: Option<crate::api::ModifiedJulianDate>,
    pub scheduled_stop_mjd: Option<crate::api::ModifiedJulianDate>,
}

/// Analytics metrics.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsMetrics {
    pub total_observations: usize,
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub scheduling_rate: f64,
    pub mean_priority: f64,
    pub median_priority: f64,
    pub mean_priority_scheduled: f64,
    pub mean_priority_unscheduled: f64,
    pub total_visibility_hours: qtty::Hours,
    pub mean_requested_hours: qtty::Hours,
}

/// Correlation entry.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationEntry {
    pub variable1: String,
    pub variable2: String,
    pub correlation: f64,
}

/// Conflict record.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub block_id_1: String, // Original ID from JSON
    pub block_id_2: String, // Original ID from JSON
    pub start_time_1: crate::api::ModifiedJulianDate,
    pub stop_time_1: crate::api::ModifiedJulianDate,
    pub start_time_2: crate::api::ModifiedJulianDate,
    pub stop_time_2: crate::api::ModifiedJulianDate,
    pub overlap_hours: qtty::Hours,
}

/// Top observation entry.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopObservation {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub total_visibility_hours: qtty::Hours,
    pub requested_hours: qtty::Hours,
    pub scheduled: bool,
}

/// Complete insights dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsData {
    pub blocks: Vec<InsightsBlock>,
    pub metrics: AnalyticsMetrics,
    pub correlations: Vec<CorrelationEntry>,
    pub top_priority: Vec<TopObservation>,
    pub top_visibility: Vec<TopObservation>,
    pub conflicts: Vec<ConflictRecord>,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub impossible_count: usize,
}

/// Route function name constant for insights
pub const GET_INSIGHTS_DATA: &str = "get_insights_data";

/// Get insights analysis data (wraps service call)
#[pyfunction]
pub fn get_insights_data(schedule_id: i64) -> PyResult<crate::api::InsightsData> {
    let data = crate::services::py_get_insights_data(schedule_id)?;
    Ok(data)
}

/// Register insights functions, classes and constants.
pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_insights_data, m)?)?;
    m.add_class::<InsightsBlock>()?;
    m.add_class::<AnalyticsMetrics>()?;
    m.add_class::<CorrelationEntry>()?;
    m.add_class::<ConflictRecord>()?;
    m.add_class::<TopObservation>()?;
    m.add_class::<InsightsData>()?;
    m.add("GET_INSIGHTS_DATA", GET_INSIGHTS_DATA)?;
    Ok(())
}
