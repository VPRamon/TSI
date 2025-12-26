//! Visualization repository trait for specialized dashboard queries.
//!
//! This trait defines operations for fetching data optimized for specific
//! visualization components (sky maps, timelines, histograms, comparisons).

use async_trait::async_trait;

use super::error::RepositoryResult;
use crate::db::models::{
    BlockHistogramData, CompareBlock, ScheduleTimelineBlock,
};

/// Repository trait for visualization-specific queries.
///
/// This trait provides optimized data fetching methods for various
/// dashboard visualizations. These methods return data structures
/// that are specifically tailored for rendering charts and graphs.
///
/// # Thread Safety
/// Implementations must be `Send + Sync` to work with async Rust.
#[async_trait]
pub trait VisualizationRepository: Send + Sync {
    /// Fetch blocks for visibility map visualization.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(VisibilityMapData)` - Complete data bundle for visibility map
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_visibility_map_data(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<crate::api::VisibilityMapData>;

    /// Fetch blocks for histogram generation.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    /// * `priority_min` - Optional minimum priority filter
    /// * `priority_max` - Optional maximum priority filter
    /// * `block_ids` - Optional specific block IDs to fetch
    ///
    /// # Returns
    /// * `Ok(Vec<BlockHistogramData>)` - Blocks for histogram
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_blocks_for_histogram(
        &self,
        schedule_id: i64,
        priority_min: Option<i32>,
        priority_max: Option<i32>,
        block_ids: Option<Vec<i64>>,
    ) -> RepositoryResult<Vec<BlockHistogramData>>;

    /// Fetch blocks for timeline visualization.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<ScheduleTimelineBlock>)` - Blocks for timeline
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_schedule_timeline_blocks(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<ScheduleTimelineBlock>>;

    /// Fetch blocks for comparison view.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<CompareBlock>)` - Blocks for comparison
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_compare_blocks(&self, schedule_id: i64) -> RepositoryResult<Vec<CompareBlock>>;
}
