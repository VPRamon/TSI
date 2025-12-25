use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

use crate::api::types as api;
use crate::db::models;

// =========================================================
// Trends types + route
// =========================================================

/// Block data for trends analysis.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsBlock {
    pub scheduling_block_id: i64,
    pub original_block_id: String,
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
    schedule_id: i64,
    n_bins: Option<i64>,
    bandwidth: Option<f64>,
    n_smooth_points: Option<i64>,
) -> PyResult<api::TrendsData> {
    let n_bins = n_bins.unwrap_or(10) as usize;
    let bandwidth = bandwidth.unwrap_or(0.5);
    let n_smooth_points = n_smooth_points.unwrap_or(12) as usize;

    let data = crate::services::py_get_trends_data(schedule_id, n_bins, bandwidth, n_smooth_points)?;
    Ok((&data).into())
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

impl From<&models::TrendsBlock> for api::TrendsBlock {
    fn from(block: &models::TrendsBlock) -> Self {
        api::TrendsBlock {
            scheduling_block_id: block.scheduling_block_id,
            original_block_id: block.original_block_id.clone(),
            priority: block.priority,
            total_visibility_hours: block.total_visibility_hours.value(),
            requested_hours: block.requested_hours.value(),
            scheduled: block.scheduled,
        }
    }
}

impl From<&models::EmpiricalRatePoint> for api::EmpiricalRatePoint {
    fn from(point: &models::EmpiricalRatePoint) -> Self {
        api::EmpiricalRatePoint {
            bin_label: point.bin_label.clone(),
            mid_value: point.mid_value,
            scheduled_rate: point.scheduled_rate,
            count: point.count,
        }
    }
}

impl From<&models::SmoothedPoint> for api::SmoothedPoint {
    fn from(point: &models::SmoothedPoint) -> Self {
        api::SmoothedPoint {
            x: point.x,
            y_smoothed: point.y_smoothed,
            n_samples: point.n_samples,
        }
    }
}

impl From<&models::HeatmapBin> for api::HeatmapBin {
    fn from(bin: &models::HeatmapBin) -> Self {
        api::HeatmapBin {
            visibility_mid: bin.visibility_mid.value(),
            time_mid: bin.time_mid.value(),
            scheduled_rate: bin.scheduled_rate,
            count: bin.count,
        }
    }
}

impl From<&models::TrendsMetrics> for api::TrendsMetrics {
    fn from(metrics: &models::TrendsMetrics) -> Self {
        api::TrendsMetrics {
            total_count: metrics.total_count,
            scheduled_count: metrics.scheduled_count,
            scheduling_rate: metrics.scheduling_rate,
            zero_visibility_count: metrics.zero_visibility_count,
            priority_min: metrics.priority_min,
            priority_max: metrics.priority_max,
            priority_mean: metrics.priority_mean,
            visibility_min: metrics.visibility_min.value(),
            visibility_max: metrics.visibility_max.value(),
            visibility_mean: metrics.visibility_mean.value(),
            time_min: metrics.time_min.value(),
            time_max: metrics.time_max.value(),
            time_mean: metrics.time_mean.value(),
        }
    }
}

impl From<&models::TrendsData> for api::TrendsData {
    fn from(data: &models::TrendsData) -> Self {
        api::TrendsData {
            blocks: data.blocks.iter().map(|b| b.into()).collect(),
            metrics: (&data.metrics).into(),
            by_priority: data.by_priority.iter().map(|r| r.into()).collect(),
            by_visibility: data.by_visibility.iter().map(|r| r.into()).collect(),
            by_time: data.by_time.iter().map(|r| r.into()).collect(),
            smoothed_visibility: data.smoothed_visibility.iter().map(|s| s.into()).collect(),
            smoothed_time: data.smoothed_time.iter().map(|s| s.into()).collect(),
            heatmap_bins: data.heatmap_bins.iter().map(|h| h.into()).collect(),
            priority_values: data.priority_values.clone(),
        }
    }
}
