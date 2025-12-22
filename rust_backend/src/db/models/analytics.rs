//! Analytics and visualization domain models.
//!
//! This module contains types for analytics, visualizations, and statistical data:
//! - LightweightBlock: Simplified block for sky map visualization
//! - DistributionBlock: Block data for distribution visualizations
//! - SkyMapData: Complete sky map data with priority bins
//! - DistributionData: Complete distribution data with statistics
//! - PriorityBinInfo: Priority bin metadata for color mapping

use pyo3::prelude::*;

use super::schedule::Period;

/// Lightweight scheduling block for sky map visualization.
/// Contains only the essential fields needed for plotting.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct LightweightBlock {
    #[pyo3(get)]
    pub original_block_id: String, // Original ID from JSON (shown to user)
    #[pyo3(get)]
    pub priority: f64,
    #[pyo3(get)]
    pub priority_bin: String,
    #[pyo3(get)]
    pub requested_duration_seconds: f64,
    #[pyo3(get)]
    pub target_ra_deg: f64,
    #[pyo3(get)]
    pub target_dec_deg: f64,
    #[pyo3(get)]
    pub scheduled_period: Option<Period>,
}

#[pymethods]
impl LightweightBlock {

    fn __repr__(&self) -> String {
        format!(
            "LightweightBlock(id={}, priority={:.2}, ra={:.2}, dec={:.2}, scheduled={})",
            self.original_block_id,
            self.priority,
            self.target_ra_deg,
            self.target_dec_deg,
            self.scheduled_period.is_some()
        )
    }
}

/// Computed priority bin with range information.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct PriorityBinInfo {
    #[pyo3(get)]
    pub label: String,
    #[pyo3(get)]
    pub min_priority: f64,
    #[pyo3(get)]
    pub max_priority: f64,
    #[pyo3(get)]
    pub color: String,
}

#[pymethods]
impl PriorityBinInfo {

    fn __repr__(&self) -> String {
        format!(
            "PriorityBinInfo(label='{}', range=[{:.2}, {:.2}], color='{}')",
            self.label, self.min_priority, self.max_priority, self.color
        )
    }
}

/// Complete sky map data with blocks and computed metadata.
/// This structure contains everything the frontend needs to render the sky map.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct SkyMapData {
    #[pyo3(get)]
    pub blocks: Vec<LightweightBlock>,
    #[pyo3(get)]
    pub priority_bins: Vec<PriorityBinInfo>,
    #[pyo3(get)]
    pub priority_min: f64,
    #[pyo3(get)]
    pub priority_max: f64,
    #[pyo3(get)]
    pub ra_min: f64,
    #[pyo3(get)]
    pub ra_max: f64,
    #[pyo3(get)]
    pub dec_min: f64,
    #[pyo3(get)]
    pub dec_max: f64,
    #[pyo3(get)]
    pub total_count: usize,
    #[pyo3(get)]
    pub scheduled_count: usize,
    #[pyo3(get)]
    pub scheduled_time_min: Option<f64>,
    #[pyo3(get)]
    pub scheduled_time_max: Option<f64>,
}

#[pymethods]
impl SkyMapData {

    fn __repr__(&self) -> String {
        format!(
            "SkyMapData(blocks={}, bins={}, priority=[{:.2}, {:.2}], scheduled={}/{})",
            self.total_count,
            self.priority_bins.len(),
            self.priority_min,
            self.priority_max,
            self.scheduled_count,
            self.total_count
        )
    }
}

/// Lightweight scheduling block for distribution visualizations.
/// Contains only the fields needed for statistical plots and histograms.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct DistributionBlock {
    #[pyo3(get)]
    pub priority: f64,
    #[pyo3(get)]
    pub total_visibility_hours: f64,
    #[pyo3(get)]
    pub requested_hours: f64,
    #[pyo3(get)]
    pub elevation_range_deg: f64,
    #[pyo3(get)]
    pub scheduled: bool,
}

#[pymethods]
impl DistributionBlock {

    fn __repr__(&self) -> String {
        format!(
            "DistributionBlock(priority={:.2}, visibility={:.1}h, requested={:.1}h, elevation={:.1}°, scheduled={})",
            self.priority,
            self.total_visibility_hours,
            self.requested_hours,
            self.elevation_range_deg,
            self.scheduled
        )
    }
}

/// Statistical summary for a group of distribution blocks.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct DistributionStats {
    #[pyo3(get)]
    pub count: usize,
    #[pyo3(get)]
    pub mean: f64,
    #[pyo3(get)]
    pub median: f64,
    #[pyo3(get)]
    pub std_dev: f64,
    #[pyo3(get)]
    pub min: f64,
    #[pyo3(get)]
    pub max: f64,
    #[pyo3(get)]
    pub sum: f64,
}

#[pymethods]
impl DistributionStats {

    fn __repr__(&self) -> String {
        format!(
            "DistributionStats(count={}, mean={:.2}, median={:.2}, std={:.2})",
            self.count, self.mean, self.median, self.std_dev
        )
    }
}

/// Complete distribution data with blocks and computed statistics.
/// This structure contains everything the frontend needs for distribution visualizations.
#[pyclass(module = "tsi_rust")]
#[derive(Debug, Clone)]
pub struct DistributionData {
    #[pyo3(get)]
    pub blocks: Vec<DistributionBlock>,
    #[pyo3(get)]
    pub priority_stats: DistributionStats,
    #[pyo3(get)]
    pub visibility_stats: DistributionStats,
    #[pyo3(get)]
    pub requested_hours_stats: DistributionStats,
    #[pyo3(get)]
    pub total_count: usize,
    #[pyo3(get)]
    pub scheduled_count: usize,
    #[pyo3(get)]
    pub unscheduled_count: usize,
    #[pyo3(get)]
    pub impossible_count: usize,
}

#[pymethods]
impl DistributionData {

    fn __repr__(&self) -> String {
        format!(
            "DistributionData(total={}, scheduled={}, impossible={})",
            self.total_count, self.scheduled_count, self.impossible_count
        )
    }
}
