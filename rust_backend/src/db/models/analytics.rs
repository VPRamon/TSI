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
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub priority_bin: String,
    pub requested_duration_seconds: f64,
    pub target_ra_deg: f64,
    pub target_dec_deg: f64,
    pub scheduled_period: Option<Period>,
}

#[pymethods]
impl LightweightBlock {
    #[getter]
    pub fn original_block_id(&self) -> String {
        self.original_block_id.clone()
    }

    #[getter]
    pub fn priority(&self) -> f64 {
        self.priority
    }

    #[getter]
    pub fn priority_bin(&self) -> String {
        self.priority_bin.clone()
    }

    #[getter]
    pub fn requested_duration_seconds(&self) -> f64 {
        self.requested_duration_seconds
    }

    #[getter]
    pub fn target_ra_deg(&self) -> f64 {
        self.target_ra_deg
    }

    #[getter]
    pub fn target_dec_deg(&self) -> f64 {
        self.target_dec_deg
    }

    #[getter]
    pub fn scheduled_period(&self) -> Option<Period> {
        self.scheduled_period
    }

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
    pub label: String,
    pub min_priority: f64,
    pub max_priority: f64,
    pub color: String,
}

#[pymethods]
impl PriorityBinInfo {
    #[getter]
    pub fn label(&self) -> String {
        self.label.clone()
    }

    #[getter]
    pub fn min_priority(&self) -> f64 {
        self.min_priority
    }

    #[getter]
    pub fn max_priority(&self) -> f64 {
        self.max_priority
    }

    #[getter]
    pub fn color(&self) -> String {
        self.color.clone()
    }

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
    pub blocks: Vec<LightweightBlock>,
    pub priority_bins: Vec<PriorityBinInfo>,
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

#[pymethods]
impl SkyMapData {
    #[getter]
    pub fn blocks(&self) -> Vec<LightweightBlock> {
        self.blocks.clone()
    }

    #[getter]
    pub fn priority_bins(&self) -> Vec<PriorityBinInfo> {
        self.priority_bins.clone()
    }

    #[getter]
    pub fn priority_min(&self) -> f64 {
        self.priority_min
    }

    #[getter]
    pub fn priority_max(&self) -> f64 {
        self.priority_max
    }

    #[getter]
    pub fn ra_min(&self) -> f64 {
        self.ra_min
    }

    #[getter]
    pub fn ra_max(&self) -> f64 {
        self.ra_max
    }

    #[getter]
    pub fn dec_min(&self) -> f64 {
        self.dec_min
    }

    #[getter]
    pub fn dec_max(&self) -> f64 {
        self.dec_max
    }

    #[getter]
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    #[getter]
    pub fn scheduled_count(&self) -> usize {
        self.scheduled_count
    }

    #[getter]
    pub fn scheduled_time_min(&self) -> Option<f64> {
        self.scheduled_time_min
    }

    #[getter]
    pub fn scheduled_time_max(&self) -> Option<f64> {
        self.scheduled_time_max
    }

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
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub elevation_range_deg: f64,
    pub scheduled: bool,
}

#[pymethods]
impl DistributionBlock {
    #[getter]
    pub fn priority(&self) -> f64 {
        self.priority
    }

    #[getter]
    pub fn total_visibility_hours(&self) -> f64 {
        self.total_visibility_hours
    }

    #[getter]
    pub fn requested_hours(&self) -> f64 {
        self.requested_hours
    }

    #[getter]
    pub fn elevation_range_deg(&self) -> f64 {
        self.elevation_range_deg
    }

    #[getter]
    pub fn scheduled(&self) -> bool {
        self.scheduled
    }

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
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

#[pymethods]
impl DistributionStats {
    #[getter]
    pub fn count(&self) -> usize {
        self.count
    }

    #[getter]
    pub fn mean(&self) -> f64 {
        self.mean
    }

    #[getter]
    pub fn median(&self) -> f64 {
        self.median
    }

    #[getter]
    pub fn std_dev(&self) -> f64 {
        self.std_dev
    }

    #[getter]
    pub fn min(&self) -> f64 {
        self.min
    }

    #[getter]
    pub fn max(&self) -> f64 {
        self.max
    }

    #[getter]
    pub fn sum(&self) -> f64 {
        self.sum
    }

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
    pub blocks: Vec<DistributionBlock>,
    pub priority_stats: DistributionStats,
    pub visibility_stats: DistributionStats,
    pub requested_hours_stats: DistributionStats,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub impossible_count: usize,
}

#[pymethods]
impl DistributionData {
    #[getter]
    pub fn blocks(&self) -> Vec<DistributionBlock> {
        self.blocks.clone()
    }

    #[getter]
    pub fn priority_stats(&self) -> DistributionStats {
        self.priority_stats.clone()
    }

    #[getter]
    pub fn visibility_stats(&self) -> DistributionStats {
        self.visibility_stats.clone()
    }

    #[getter]
    pub fn requested_hours_stats(&self) -> DistributionStats {
        self.requested_hours_stats.clone()
    }

    #[getter]
    pub fn total_count(&self) -> usize {
        self.total_count
    }

    #[getter]
    pub fn scheduled_count(&self) -> usize {
        self.scheduled_count
    }

    #[getter]
    pub fn unscheduled_count(&self) -> usize {
        self.unscheduled_count
    }

    #[getter]
    pub fn impossible_count(&self) -> usize {
        self.impossible_count
    }

    fn __repr__(&self) -> String {
        format!(
            "DistributionData(total={}, scheduled={}, impossible={})",
            self.total_count, self.scheduled_count, self.impossible_count
        )
    }
}
