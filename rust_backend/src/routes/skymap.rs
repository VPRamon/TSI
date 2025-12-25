use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use crate::api::types as api;

/// Sky map visualization data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkyMapData {
    pub blocks: Vec<crate::api::LightweightBlock>,
    pub priority_bins: Vec<crate::api::PriorityBinInfo>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub ra_min: f64,
    pub ra_max: f64,
    pub dec_min: f64,
    pub dec_max: f64,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub scheduled_time_min: Option<f64>,
    pub scheduled_time_max: Option<f64>,
}

/// Route function name constant
pub const GET_SKY_MAP_DATA: &str = "get_sky_map_data";

/// Get sky map visualization data (ETL-based).
#[pyfunction]
pub fn get_sky_map_data(schedule_id: i64) -> PyResult<api::SkyMapData> {
    let data = crate::services::py_get_sky_map_data_analytics(schedule_id)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
    Ok(data)
}
