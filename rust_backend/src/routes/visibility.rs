use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;
use crate::db::repository::visualization::VisualizationRepository;

// =========================================================
// Visibility Map types + route
// =========================================================

/// Block summary for visibility map.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityBlockSummary {
    pub scheduling_block_id: i64,
    pub original_block_id: String,
    pub priority: f64,
    pub num_visibility_periods: usize,
    pub scheduled: bool,
}

/// Visibility map visualization data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityMapData {
    pub blocks: Vec<VisibilityBlockSummary>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub total_count: usize,
    pub scheduled_count: usize,
}

/// Route function name constant for visibility map
pub const GET_VISIBILITY_MAP_DATA: &str = "get_visibility_map_data";

/// Get visibility map data (wraps repository call)
#[pyfunction]
pub fn get_visibility_map_data(schedule_id: i64) -> PyResult<VisibilityMapData> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    let repo = crate::db::get_repository()
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;

    let data = runtime
        .block_on(repo.fetch_visibility_map_data(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("{}", e)))?;

    Ok(data)
}

/// Register visibility-related functions, classes, and constants.
pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_visibility_map_data, m)?)?;
    m.add_class::<VisibilityBlockSummary>()?;
    m.add_class::<VisibilityMapData>()?;
    m.add("GET_VISIBILITY_MAP_DATA", GET_VISIBILITY_MAP_DATA)?;
    Ok(())
}