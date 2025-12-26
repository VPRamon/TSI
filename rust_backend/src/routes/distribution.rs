use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

// =========================================================
// Distribution types + route
// =========================================================

/// Block data for distribution analysis.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionBlock {
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub elevation_range_deg: f64,
    pub scheduled: bool,
}

/// Distribution statistics.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

/// Complete distribution dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionData {
    pub blocks: Vec<DistributionBlock>,
    pub priority_stats: DistributionStats,
    pub visibility_stats: DistributionStats,
    pub requested_hours_stats: DistributionStats,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub impossible_count: usize,
}

/// Route function name constant for distribution data
pub const GET_DISTRIBUTION_DATA: &str = "get_distribution_data";

/// Get distribution visualization data (wraps service call)
#[pyfunction]
pub fn get_distribution_data(schedule_id: i64) -> PyResult<crate::api::DistributionData> {
    let data = crate::services::py_get_distribution_data_analytics(schedule_id)?;
    Ok(data)
}

/// Register distribution functions, classes and constants with the Python module.
pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_distribution_data, m)?)?;
    m.add_class::<DistributionBlock>()?;
    m.add_class::<DistributionStats>()?;
    m.add_class::<DistributionData>()?;
    m.add("GET_DISTRIBUTION_DATA", GET_DISTRIBUTION_DATA)?;
    Ok(())
}
