use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

// =========================================================
// Trends types + route
// =========================================================

/// Block data for trends analysis.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsBlock {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub scheduled: bool,
}

/// Empirical scheduling rate point.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpiricalRatePoint {
    pub bin_label: String,
    pub mid_value: f64,
    pub scheduled_rate: f64,
    pub count: usize,
}

/// Smoothed trend point.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmoothedPoint {
    pub x: f64,
    pub y_smoothed: f64,
    pub n_samples: usize,
}

/// Heatmap bin data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapBin {
    pub visibility_mid: f64,
    pub time_mid: f64,
    pub scheduled_rate: f64,
    pub count: usize,
}

/// Trends metrics summary.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsMetrics {
    pub total_count: usize,
    pub scheduled_count: usize,
    pub scheduling_rate: f64,
    pub zero_visibility_count: usize,
    pub priority_min: f64,
    pub priority_max: f64,
    pub priority_mean: f64,
    pub visibility_min: f64,
    pub visibility_max: f64,
    pub visibility_mean: f64,
    pub time_min: f64,
    pub time_max: f64,
    pub time_mean: f64,
}

/// Complete trends dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsData {
    pub blocks: Vec<TrendsBlock>,
    pub metrics: TrendsMetrics,
    pub by_priority: Vec<EmpiricalRatePoint>,
    pub by_visibility: Vec<EmpiricalRatePoint>,
    pub by_time: Vec<EmpiricalRatePoint>,
    pub smoothed_visibility: Vec<SmoothedPoint>,
    pub smoothed_time: Vec<SmoothedPoint>,
    pub heatmap_bins: Vec<HeatmapBin>,
    pub priority_values: Vec<f64>,
}

/// Route function name constant for trends
pub const GET_TRENDS_DATA: &str = "get_trends_data";

/// Get trends analysis data (wraps service call).
/// Accepts optional parameters from Python and uses sensible defaults.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub fn get_trends_data(
    schedule_id: crate::api::ScheduleId,
    n_bins: Option<i64>,
    bandwidth: Option<f64>,
    n_smooth_points: Option<i64>,
) -> PyResult<crate::api::TrendsData> {
    let n_bins = n_bins.unwrap_or(10) as usize;
    let bandwidth = bandwidth.unwrap_or(0.5);
    let n_smooth_points = n_smooth_points.unwrap_or(12) as usize;

    let data =
        crate::services::py_get_trends_data(schedule_id, n_bins, bandwidth, n_smooth_points)?;
    Ok(data)
}

/// Register trends-related functions, classes, and constants.
pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_trends_data, m)?)?;
    m.add_class::<TrendsBlock>()?;
    m.add_class::<EmpiricalRatePoint>()?;
    m.add_class::<SmoothedPoint>()?;
    m.add_class::<HeatmapBin>()?;
    m.add_class::<TrendsMetrics>()?;
    m.add_class::<TrendsData>()?;
    m.add("GET_TRENDS_DATA", GET_TRENDS_DATA)?;
    Ok(())
}
