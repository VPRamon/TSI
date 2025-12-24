//! Analytics and visualization domain models.
//!
//! This module contains types for analytics, visualizations, and statistical data:
//! - LightweightBlock: Simplified block for sky map visualization
//! - DistributionBlock: Block data for distribution visualizations
//! - SkyMapData: Complete sky map data with priority bins
//! - DistributionData: Complete distribution data with statistics
//! - PriorityBinInfo: Priority bin metadata for color mapping

use super::schedule::Period;

/// Lightweight scheduling block for sky map visualization.
/// Contains only the essential fields needed for plotting.
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

/// Computed priority bin with range information.
type PriorityBinInfo = crate::api::PriorityBinInfo;

/// Complete sky map data with blocks and computed metadata.
/// This structure contains everything the frontend needs to render the sky map.
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

/// Lightweight scheduling block for distribution visualizations.
/// Contains only the fields needed for statistical plots and histograms.
#[derive(Debug, Clone)]
pub struct DistributionBlock {
    pub priority: f64,
    pub total_visibility_hours: f64,
    pub requested_hours: f64,
    pub elevation_range_deg: f64,
    pub scheduled: bool,
}

/// Statistical summary for a group of distribution blocks.
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

/// Complete distribution data with blocks and computed statistics.
/// This structure contains everything the frontend needs for distribution visualizations.
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
