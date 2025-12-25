use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use crate::api::types as api;

// =========================================================
// Compare types + route
// =========================================================

#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareBlock {
    pub scheduling_block_id: String,
    pub priority: f64,
    pub scheduled: bool,
    pub requested_hours: f64,
}

#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareStats {
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub total_priority: f64,
    pub mean_priority: f64,
    pub median_priority: f64,
    pub total_hours: f64,
}

#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingChange {
    pub scheduling_block_id: String,
    pub priority: f64,
    pub change_type: String,
}

#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareData {
    pub current_blocks: Vec<CompareBlock>,
    pub comparison_blocks: Vec<CompareBlock>,
    pub current_stats: CompareStats,
    pub comparison_stats: CompareStats,
    pub common_ids: Vec<String>,
    pub only_in_current: Vec<String>,
    pub only_in_comparison: Vec<String>,
    pub scheduling_changes: Vec<SchedulingChange>,
    pub current_name: String,
    pub comparison_name: String,
}

pub const GET_COMPARE_DATA: &str = "get_compare_data";

#[pyfunction]
pub fn get_compare_data(
    current_schedule_id: i64,
    comparison_schedule_id: i64,
    current_name: Option<String>,
    comparison_name: Option<String>,
) -> PyResult<api::CompareData> {
    let current_name = current_name.unwrap_or_else(|| "Schedule A".to_string());
    let comparison_name = comparison_name.unwrap_or_else(|| "Schedule B".to_string());

    let data = crate::services::py_get_compare_data(
        current_schedule_id,
        comparison_schedule_id,
        current_name,
        comparison_name,
    )?;
    Ok((&data).into())
}

/// Register compare route functions, classes and constants.
pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_compare_data, m)?)?;
    m.add_class::<CompareBlock>()?;
    m.add_class::<CompareStats>()?;
    m.add_class::<SchedulingChange>()?;
    m.add_class::<CompareData>()?;
    m.add("GET_COMPARE_DATA", GET_COMPARE_DATA)?;
    Ok(())
}
