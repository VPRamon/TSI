use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

// =========================================================
// Schedule timeline types + route
// =========================================================

/// Timeline block data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTimelineBlock {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub scheduled_start_mjd: crate::api::ModifiedJulianDate,
    pub scheduled_stop_mjd: crate::api::ModifiedJulianDate,
    pub ra_deg: qtty::Degrees,
    pub dec_deg: qtty::Degrees,
    pub requested_hours: qtty::Hours,
    pub total_visibility_hours: qtty::Hours,
    pub num_visibility_periods: usize,
}

/// Schedule timeline dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTimelineData {
    pub blocks: Vec<ScheduleTimelineBlock>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub unique_months: Vec<String>,
    pub dark_periods: Vec<crate::api::Period>,
}

/// Route function name constant for schedule timeline
pub const GET_SCHEDULE_TIMELINE_DATA: &str = "get_schedule_timeline_data";

/// Get schedule timeline data (wraps service call)
#[pyfunction]
pub fn get_schedule_timeline_data(
    schedule_id: crate::api::ScheduleId,
) -> PyResult<ScheduleTimelineData> {
    let data = crate::services::py_get_schedule_timeline_data(schedule_id)?;
    Ok(data)
}

/// Register timeline route functions, classes, and constants.
pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_schedule_timeline_data, m)?)?;
    m.add_class::<ScheduleTimelineBlock>()?;
    m.add_class::<ScheduleTimelineData>()?;
    m.add("GET_SCHEDULE_TIMELINE_DATA", GET_SCHEDULE_TIMELINE_DATA)?;
    Ok(())
}
