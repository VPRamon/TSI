//! Azure SQL Server repository implementation.
//!
//! This module implements the `ScheduleRepository` trait using Azure SQL Server
//! as the backend. It wraps the existing database operations from `operations.rs`,
//! `analytics.rs`, and `validation.rs`.

use async_trait::async_trait;

use super::{analytics, operations, validation};
use crate::db::{
    repository::*,
    models::{Period, Schedule, ScheduleInfo, ScheduleMetadata},
};
use crate::services::validation::ValidationResult;

/// Azure SQL Server repository implementation.
///
/// This implementation uses Azure SQL Server with connection pooling via bb8.
/// The database pool must be initialized before creating this repository.
///
/// # Example
/// ```no_run
/// use tsi_rust::db::{DbConfig, pool, repositories::AzureRepository, ScheduleRepository};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = DbConfig::from_env()?;
///     pool::init_pool(&config).await?;
///     
///     let repo = AzureRepository::new();
///     let schedules = repo.list_schedules().await?;
///     println!("Found {} schedules", schedules.len());
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct AzureRepository;

impl AzureRepository {
    /// Create a new Azure repository instance.
    ///
    /// The global database pool must be initialized via `pool::init_pool()`
    /// before using this repository.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AzureRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ScheduleRepository for AzureRepository {
    async fn health_check(&self) -> RepositoryResult<bool> {
        operations::health_check()
            .await
            .map_err(RepositoryError::from)
    }

    async fn store_schedule(&self, schedule: &Schedule) -> RepositoryResult<ScheduleMetadata> {
        operations::store_schedule(schedule)
            .await
            .map_err(RepositoryError::from)
    }

    async fn get_schedule(&self, schedule_id: i64) -> RepositoryResult<Schedule> {
        operations::get_schedule(Some(schedule_id), None)
            .await
            .map_err(|e| {
                if e.contains("not found") || e.contains("does not exist") {
                    RepositoryError::NotFound(e)
                } else {
                    RepositoryError::from(e)
                }
            })
    }

    async fn list_schedules(&self) -> RepositoryResult<Vec<ScheduleInfo>> {
        operations::list_schedules()
            .await
            .map_err(RepositoryError::from)
    }

    async fn get_schedule_time_range(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Option<Period>> {
        operations::get_schedule_time_range(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn get_scheduling_block(
        &self,
        scheduling_block_id: i64,
    ) -> RepositoryResult<crate::db::models::SchedulingBlock> {
        operations::get_scheduling_block(scheduling_block_id)
            .await
            .map_err(|e| {
                if e.contains("not found") || e.contains("does not exist") {
                    RepositoryError::NotFound(e)
                } else {
                    RepositoryError::from(e)
                }
            })
    }

    async fn get_blocks_for_schedule(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::SchedulingBlock>> {
        operations::get_blocks_for_schedule(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_dark_periods(&self, schedule_id: i64) -> RepositoryResult<Vec<Period>> {
        operations::fetch_dark_periods_public(Some(schedule_id))
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_possible_periods(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<Period>> {
        operations::fetch_possible_periods(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn populate_schedule_analytics(&self, schedule_id: i64) -> RepositoryResult<usize> {
        analytics::populate_schedule_analytics(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn delete_schedule_analytics(&self, schedule_id: i64) -> RepositoryResult<usize> {
        analytics::delete_schedule_analytics(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn has_analytics_data(&self, schedule_id: i64) -> RepositoryResult<bool> {
        analytics::has_analytics_data(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_analytics_blocks_for_sky_map(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::LightweightBlock>> {
        analytics::fetch_analytics_blocks_for_sky_map(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_analytics_blocks_for_distribution(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::DistributionBlock>> {
        analytics::fetch_analytics_blocks_for_distribution(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn populate_summary_analytics(
        &self,
        schedule_id: i64,
        n_bins: usize,
    ) -> RepositoryResult<()> {
        analytics::populate_summary_analytics(schedule_id, n_bins)
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_schedule_summary(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Option<analytics::ScheduleSummary>> {
        analytics::fetch_schedule_summary(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_priority_rates(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<analytics::PriorityRate>> {
        analytics::fetch_priority_rates(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_visibility_bins(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<analytics::VisibilityBin>> {
        analytics::fetch_visibility_bins(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_heatmap_bins(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<analytics::HeatmapBinData>> {
        analytics::fetch_heatmap_bins(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn has_summary_analytics(&self, schedule_id: i64) -> RepositoryResult<bool> {
        analytics::has_summary_analytics(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn delete_summary_analytics(&self, schedule_id: i64) -> RepositoryResult<usize> {
        analytics::delete_summary_analytics(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn populate_visibility_time_bins(
        &self,
        schedule_id: i64,
        bin_duration_seconds: Option<i64>,
    ) -> RepositoryResult<(usize, usize)> {
        analytics::populate_visibility_time_bins(schedule_id, bin_duration_seconds)
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_visibility_histogram_from_analytics(
        &self,
        schedule_id: i64,
        start_unix: i64,
        end_unix: i64,
        target_bin_duration_seconds: i64,
    ) -> RepositoryResult<Vec<crate::db::models::VisibilityBin>> {
        analytics::fetch_visibility_histogram_from_analytics(
            schedule_id,
            start_unix,
            end_unix,
            target_bin_duration_seconds,
        )
        .await
        .map_err(RepositoryError::from)
    }

    async fn fetch_visibility_metadata(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Option<analytics::VisibilityTimeMetadata>> {
        analytics::fetch_visibility_metadata(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn has_visibility_time_bins(&self, schedule_id: i64) -> RepositoryResult<bool> {
        analytics::has_visibility_time_bins(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn delete_visibility_time_bins(&self, schedule_id: i64) -> RepositoryResult<usize> {
        analytics::delete_visibility_time_bins(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn insert_validation_results(
        &self,
        results: &[ValidationResult],
    ) -> RepositoryResult<usize> {
        validation::insert_validation_results(results)
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_validation_results(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<validation::ValidationReportData> {
        validation::fetch_validation_results(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn has_validation_results(&self, schedule_id: i64) -> RepositoryResult<bool> {
        validation::has_validation_results(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn delete_validation_results(&self, schedule_id: i64) -> RepositoryResult<u64> {
        validation::delete_validation_results(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_visibility_map_data(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<crate::db::models::VisibilityMapData> {
        operations::fetch_visibility_map_data(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_blocks_for_histogram(
        &self,
        schedule_id: i64,
        priority_min: Option<i32>,
        priority_max: Option<i32>,
        block_ids: Option<Vec<i64>>,
    ) -> RepositoryResult<Vec<crate::db::models::BlockHistogramData>> {
        operations::fetch_blocks_for_histogram(
            schedule_id,
            priority_min,
            priority_max,
            block_ids.as_deref(),
        )
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_schedule_timeline_blocks(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::ScheduleTimelineBlock>> {
        operations::fetch_schedule_timeline_blocks(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }

    async fn fetch_compare_blocks(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::CompareBlock>> {
        operations::fetch_compare_blocks(schedule_id)
            .await
            .map_err(RepositoryError::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_azure_repository_creation() {
        let _repo = AzureRepository::new();
        let _repo2 = AzureRepository::default();
        // Just verify we can create instances
    }
}
