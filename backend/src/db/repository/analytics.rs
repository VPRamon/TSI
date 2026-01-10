//! Analytics repository trait for pre-computed analytics operations.
//!
//! This trait defines operations for managing and querying pre-computed
//! analytics data, including block-level analytics, summary statistics,
//! and visibility time bins.

use async_trait::async_trait;

use super::error::RepositoryResult;
use crate::api::InsightsBlock;
use crate::api::{DistributionBlock, LightweightBlock};

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
    async fn populate_schedule_analytics(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<usize>;

    /// Delete analytics data for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of rows deleted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn delete_schedule_analytics(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<usize>;

    /// Check if analytics data exists for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(bool)` - True if analytics data exists
    /// * `Err(RepositoryError)` - If the operation fails
    async fn has_analytics_data(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<bool>;

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
        schedule_id: crate::api::ScheduleId,
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
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<DistributionBlock>>;

    /// Fetch analytics blocks for insights computations.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<InsightsBlock>)` - Blocks for insights
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_analytics_blocks_for_insights(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<InsightsBlock>>;
}
