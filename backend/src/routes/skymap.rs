use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// Priority bin information for sky map.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityBinInfo {
    pub label: String,
    pub min_priority: f64,
    pub max_priority: f64,
    pub color: String,
}

/// Minimal block data for visualization queries.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightweightBlock {
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub priority_bin: String,
    pub requested_duration_seconds: qtty::Seconds,
    pub target_ra_deg: qtty::Degrees,
    pub target_dec_deg: qtty::Degrees,
    pub scheduled_period: Option<crate::api::Period>,
}

/// Sky map visualization data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkyMapData {
    pub blocks: Vec<LightweightBlock>,
    pub priority_bins: Vec<crate::api::PriorityBinInfo>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub ra_min: qtty::Degrees,
    pub ra_max: qtty::Degrees,
    pub dec_min: qtty::Degrees,
    pub dec_max: qtty::Degrees,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub scheduled_time_min: Option<f64>,
    pub scheduled_time_max: Option<f64>,
}

/// Route function name constant
pub const GET_SKY_MAP_DATA: &str = "get_sky_map_data";

/// Get sky map visualization data (ETL-based).
#[pyfunction]
pub fn get_sky_map_data(schedule_id: crate::api::ScheduleId) -> PyResult<crate::api::SkyMapData> {
    let data = crate::services::py_get_sky_map_data_analytics(schedule_id)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(data)
}

/// Register skymap functions, classes and constants with Python module.
pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_sky_map_data, m)?)?;
    m.add_class::<PriorityBinInfo>()?;
    m.add_class::<LightweightBlock>()?;
    m.add_class::<SkyMapData>()?;
    m.add("GET_SKY_MAP_DATA", GET_SKY_MAP_DATA)?;
    Ok(())
}
