use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

// =========================================================
// Schedule timeline types + route
// =========================================================

/// Timeline block data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTimelineBlock {
    pub scheduling_block_id: i64,
    pub original_block_id: String,
    pub priority: f64,
    pub scheduled_start_mjd: f64,
    pub scheduled_stop_mjd: f64,
    pub ra_deg: f64,
    pub dec_deg: f64,
    pub requested_hours: f64,
    pub total_visibility_hours: f64,
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
pub fn get_schedule_timeline_data(schedule_id: i64) -> PyResult<ScheduleTimelineData> {
    let data = crate::services::py_get_schedule_timeline_data(schedule_id)?;
    Ok(data)
}
