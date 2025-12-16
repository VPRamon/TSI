//! Analytics repository trait for pre-computed analytics operations.
//!
//! This trait defines operations for managing and querying pre-computed
//! analytics data, including block-level analytics, summary statistics,
//! and visibility time bins.

use async_trait::async_trait;

use crate::db::analytics::{
    HeatmapBinData, PriorityRate, ScheduleSummary, VisibilityBin,
    VisibilityTimeMetadata,
};
use crate::db::models::{DistributionBlock, LightweightBlock};
use super::error::RepositoryResult;

/// Repository trait for analytics operations.
///
/// This trait handles pre-computed analytics data that accelerates
/// dashboard queries and visualizations. It includes three phases:
/// 1. Block-level analytics (denormalized block data)
/// 2. Summary analytics (aggregated statistics)
/// 3. Visibility time bins (histogram data)
///
/// # Thread Safety
/// Implementations must be `Send + Sync` to work with async Rust.
#[async_trait]
pub trait AnalyticsRepository: Send + Sync {
    // ==================== Block-Level Analytics ====================

    /// Populate the analytics table for a schedule.
    ///
    /// This pre-computes denormalized data for faster queries.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule to analyze
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of analytics rows inserted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn populate_schedule_analytics(&self, schedule_id: i64) -> RepositoryResult<usize>;

    /// Delete analytics data for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of rows deleted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn delete_schedule_analytics(&self, schedule_id: i64) -> RepositoryResult<usize>;

    /// Check if analytics data exists for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(bool)` - True if analytics data exists
    /// * `Err(RepositoryError)` - If the operation fails
    async fn has_analytics_data(&self, schedule_id: i64) -> RepositoryResult<bool>;

    /// Fetch analytics blocks for sky map visualization.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<LightweightBlock>)` - Blocks optimized for sky map display
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_analytics_blocks_for_sky_map(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<LightweightBlock>>;

    /// Fetch analytics blocks for distribution charts.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<DistributionBlock>)` - Blocks for distribution analysis
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_analytics_blocks_for_distribution(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<DistributionBlock>>;

    // ==================== Summary Analytics ====================

    /// Populate summary analytics (aggregated statistics) for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    /// * `n_bins` - Number of bins for histogram data
    ///
    /// # Returns
    /// * `Ok(())` - If successful
    /// * `Err(RepositoryError)` - If the operation fails
    async fn populate_summary_analytics(
        &self,
        schedule_id: i64,
        n_bins: usize,
    ) -> RepositoryResult<()>;

    /// Fetch schedule summary statistics.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Some(ScheduleSummary))` - Summary statistics if available
    /// * `Ok(None)` - If no summary exists
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_schedule_summary(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Option<ScheduleSummary>>;

    /// Fetch priority rate distribution.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<PriorityRate>)` - Priority distribution data
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_priority_rates(&self, schedule_id: i64)
        -> RepositoryResult<Vec<PriorityRate>>;

    /// Fetch visibility bins for histogram.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<VisibilityBin>)` - Visibility histogram data
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_visibility_bins(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<VisibilityBin>>;

    /// Fetch heatmap bin data.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<HeatmapBinData>)` - Heatmap data
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_heatmap_bins(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<HeatmapBinData>>;

    /// Check if summary analytics exist for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(bool)` - True if summary analytics exist
    /// * `Err(RepositoryError)` - If the operation fails
    async fn has_summary_analytics(&self, schedule_id: i64) -> RepositoryResult<bool>;

    /// Delete summary analytics for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of rows deleted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn delete_summary_analytics(&self, schedule_id: i64) -> RepositoryResult<usize>;

    // ==================== Visibility Time Bins ====================

    /// Populate visibility time bins for histogram analysis.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    /// * `bin_duration_seconds` - Optional bin duration (default: 900 seconds)
    ///
    /// # Returns
    /// * `Ok((usize, usize))` - Tuple of (metadata_rows, bin_rows) inserted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn populate_visibility_time_bins(
        &self,
        schedule_id: i64,
        bin_duration_seconds: Option<i64>,
    ) -> RepositoryResult<(usize, usize)>;

    /// Fetch visibility histogram data from analytics.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    /// * `start_unix` - Start time in Unix seconds
    /// * `end_unix` - End time in Unix seconds
    /// * `target_bin_duration_seconds` - Target bin duration
    ///
    /// # Returns
    /// * `Ok(Vec<VisibilityBin>)` - Histogram bins
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_visibility_histogram_from_analytics(
        &self,
        schedule_id: i64,
        start_unix: i64,
        end_unix: i64,
        target_bin_duration_seconds: i64,
    ) -> RepositoryResult<Vec<crate::db::models::VisibilityBin>>;

    /// Fetch visibility metadata (range, bins, etc.).
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Some(VisibilityTimeMetadata))` - Metadata if available
    /// * `Ok(None)` - If no metadata exists
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_visibility_metadata(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Option<VisibilityTimeMetadata>>;

    /// Check if visibility time bins exist for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(bool)` - True if bins exist
    /// * `Err(RepositoryError)` - If the operation fails
    async fn has_visibility_time_bins(&self, schedule_id: i64) -> RepositoryResult<bool>;

    /// Delete visibility time bins for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of rows deleted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn delete_visibility_time_bins(&self, schedule_id: i64) -> RepositoryResult<usize>;
}
