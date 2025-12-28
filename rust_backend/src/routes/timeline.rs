use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use crate::db::models;

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
pub fn get_schedule_timeline_data(schedule_id: crate::api::ScheduleId) -> PyResult<ScheduleTimelineData> {
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

impl From<&models::ScheduleTimelineBlock> for crate::api::ScheduleTimelineBlock {
    fn from(block: &models::ScheduleTimelineBlock) -> Self {
        crate::api::ScheduleTimelineBlock {
            scheduling_block_id: block.scheduling_block_id,
            original_block_id: block.original_block_id.clone(),
            priority: block.priority,
            scheduled_start_mjd: block.scheduled_start_mjd.value(),
            scheduled_stop_mjd: block.scheduled_stop_mjd.value(),
            ra_deg: block.ra_deg.value(),
            dec_deg: block.dec_deg.value(),
            requested_hours: block.requested_hours.value(),
            total_visibility_hours: block.total_visibility_hours.value(),
            num_visibility_periods: block.num_visibility_periods,
        }
    }
}
