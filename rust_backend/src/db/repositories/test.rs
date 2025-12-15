//! In-memory test repository implementation.
//!
//! This module provides a mock implementation of the `ScheduleRepository` trait
//! suitable for unit testing. All data is stored in memory using HashMap and Vec
//! structures, providing fast, deterministic, and isolated test execution.

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::db::{
    analytics,
    models::{Schedule, ScheduleInfo, ScheduleMetadata, SchedulingBlock},
    repository::*,
    validation,
};
use crate::services::validation::ValidationResult;

/// In-memory repository for testing.
///
/// This implementation stores all data in memory using HashMaps and Vecs,
/// making it ideal for unit tests that need isolation and speed.
///
/// # Example
/// ```
/// use tsi_rust::db::repositories::TestRepository;
///
/// #[tokio::test]
/// async fn test_schedule_storage() {
///     let repo = TestRepository::new();
///     
///     // Pre-populate with test data
///     repo.store_schedule_impl(/* ... */);
///     
///     let schedules = repo.list_schedules().await.unwrap();
///     assert_eq!(schedules.len(), 1);
/// }
/// ```
#[derive(Clone)]
pub struct TestRepository {
    data: Arc<RwLock<TestData>>,
}

#[derive(Default)]
struct TestData {
    schedules: HashMap<i64, Schedule>,
    schedule_metadata: HashMap<i64, ScheduleMetadata>,
    blocks: HashMap<i64, SchedulingBlock>,
    dark_periods: HashMap<i64, Vec<(i64, f64, f64)>>,
    possible_periods: HashMap<i64, Vec<(i64, f64, f64)>>,
    
    // Analytics data
    analytics_exists: HashMap<i64, bool>,
    summary_analytics: HashMap<i64, analytics::ScheduleSummary>,
    priority_rates: HashMap<i64, Vec<analytics::PriorityRate>>,
    visibility_bins: HashMap<i64, Vec<analytics::VisibilityBin>>,
    heatmap_bins: HashMap<i64, Vec<analytics::HeatmapBinData>>,
    visibility_time_bins: HashMap<i64, Vec<analytics::VisibilityTimeBin>>,
    visibility_metadata: HashMap<i64, analytics::VisibilityTimeMetadata>,
    
    // Validation data
    validation_results: HashMap<i64, validation::ValidationReportData>,
    
    // ID counters
    next_schedule_id: i64,
    next_block_id: i64,
    next_period_id: i64,
    
    // Connection health
    is_healthy: bool,
}

impl TestRepository {
    /// Create a new empty test repository.
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(TestData {
                is_healthy: true,
                next_schedule_id: 1,
                next_block_id: 1,
                next_period_id: 1,
                ..Default::default()
            })),
        }
    }

    /// Add a test schedule to the repository.
    ///
    /// This is a helper method for setting up test data. The schedule will be
    /// assigned an ID automatically.
    ///
    /// # Arguments
    /// * `schedule` - Schedule to add (id will be overwritten)
    ///
    /// # Returns
    /// The ID assigned to the schedule
    pub fn store_schedule_impl(&self, mut schedule: Schedule) -> i64 {
        let mut data = self.data.write().unwrap();
        let schedule_id = data.next_schedule_id;
        data.next_schedule_id += 1;

        // Assign IDs to blocks
        for block in &mut schedule.blocks {
            let block_id = data.next_block_id;
            data.next_block_id += 1;
            data.blocks.insert(block_id, block.clone());
        }

        let metadata = ScheduleMetadata {
            schedule_id: Some(schedule_id),
            schedule_name: schedule.name.clone(),
            upload_timestamp: Utc::now(),
            checksum: schedule.checksum.clone(),
        };

        data.schedule_metadata.insert(schedule_id, metadata);
        data.schedules.insert(schedule_id, schedule);
        
        schedule_id
    }

    /// Add dark periods for a schedule.
    pub fn add_dark_periods(&self, schedule_id: i64, periods: Vec<(f64, f64)>) {
        let mut data = self.data.write().unwrap();
        let mut period_data = Vec::new();
        
        for (start, stop) in periods {
            let period_id = data.next_period_id;
            data.next_period_id += 1;
            period_data.push((period_id, start, stop));
        }
        
        data.dark_periods.insert(schedule_id, period_data);
    }

    /// Set the health status for testing connection failures.
    pub fn set_healthy(&self, healthy: bool) {
        let mut data = self.data.write().unwrap();
        data.is_healthy = healthy;
    }

    /// Clear all data from the repository.
    pub fn clear(&self) {
        let mut data = self.data.write().unwrap();
        *data = TestData {
            is_healthy: data.is_healthy,
            next_schedule_id: 1,
            next_block_id: 1,
            next_period_id: 1,
            ..Default::default()
        };
    }

    /// Get the number of schedules stored.
    pub fn schedule_count(&self) -> usize {
        self.data.read().unwrap().schedules.len()
    }

    /// Check if a schedule exists.
    pub fn has_schedule(&self, schedule_id: i64) -> bool {
        self.data.read().unwrap().schedules.contains_key(&schedule_id)
    }

    /// Helper to check health and return error if unhealthy.
    fn check_health(&self) -> RepositoryResult<()> {
        let data = self.data.read().unwrap();
        if !data.is_healthy {
            return Err(RepositoryError::ConnectionError(
                "Database is not healthy".to_string(),
            ));
        }
        Ok(())
    }

    /// Helper to get a schedule or return NotFound error.
    fn get_schedule_impl(&self, schedule_id: i64) -> RepositoryResult<Schedule> {
        let data = self.data.read().unwrap();
        data.schedules
            .get(&schedule_id)
            .cloned()
            .ok_or_else(|| RepositoryError::NotFound(format!("Schedule {} not found", schedule_id)))
    }

    /// Helper for the common deletion pattern.
    fn delete_from_map<T>(&self, map_accessor: impl FnOnce(&mut TestData) -> &mut HashMap<i64, T>, schedule_id: i64) -> usize {
        let mut data = self.data.write().unwrap();
        let existed = map_accessor(&mut data).remove(&schedule_id).is_some();
        if existed { 1 } else { 0 }
    }
}

impl Default for TestRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ScheduleRepository for TestRepository {
    async fn health_check(&self) -> RepositoryResult<bool> {
        let data = self.data.read().unwrap();
        Ok(data.is_healthy)
    }

    async fn store_schedule(&self, schedule: &Schedule) -> RepositoryResult<ScheduleMetadata> {
        self.check_health()?;

        // Use the helper method to add the schedule
        let schedule_id = self.store_schedule_impl(schedule.clone());

        // Retrieve and return the metadata
        let data = self.data.read().unwrap();
        let metadata = data.schedule_metadata.get(&schedule_id).cloned().unwrap();
        
        Ok(metadata)
    }

    async fn get_schedule(&self, schedule_id: i64) -> RepositoryResult<Schedule> {
        self.get_schedule_impl(schedule_id)
    }

    async fn list_schedules(&self) -> RepositoryResult<Vec<ScheduleInfo>> {
        let data = self.data.read().unwrap();
        
        let mut schedules: Vec<ScheduleInfo> = data
            .schedule_metadata
            .iter()
            .map(|(id, meta)| {
                let schedule = data.schedules.get(id).unwrap();
                ScheduleInfo {
                    metadata: ScheduleMetadata {
                        schedule_id: Some(*id),
                        schedule_name: meta.schedule_name.clone(),
                        upload_timestamp: chrono::Utc::now(),
                        checksum: meta.checksum.clone(),
                    },
                    total_blocks: schedule.blocks.len(),
                    scheduled_blocks: schedule.blocks.len(),
                    unscheduled_blocks: 0,
                }
            })
            .collect();

        schedules.sort_by_key(|s| s.metadata.schedule_id.unwrap_or(0));
        Ok(schedules)
    }

    async fn get_schedule_time_range(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Option<(f64, f64)>> {
        let schedule = self.get_schedule_impl(schedule_id)?;

        // Calculate time range from dark periods
        if schedule.dark_periods.is_empty() {
            return Ok(None);
        }

        let min_start = schedule
            .dark_periods
            .iter()
            .map(|p| p.start.value())
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        let max_stop = schedule
            .dark_periods
            .iter()
            .map(|p| p.stop.value())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();

        Ok(Some((min_start, max_stop)))
    }

    async fn get_scheduling_block(
        &self,
        scheduling_block_id: i64,
    ) -> RepositoryResult<SchedulingBlock> {
        let data = self.data.read().unwrap();
        
        data.blocks
            .get(&scheduling_block_id)
            .cloned()
            .ok_or_else(|| {
                RepositoryError::NotFound(format!("Scheduling block {} not found", scheduling_block_id))
            })
    }

    async fn get_blocks_for_schedule(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<SchedulingBlock>> {
        let schedule = self.get_schedule_impl(schedule_id)?;
        Ok(schedule.blocks.clone())
    }

    async fn fetch_dark_periods(&self, schedule_id: i64) -> RepositoryResult<Vec<(f64, f64)>> {
        let data = self.data.read().unwrap();
        
        // Convert from 3-tuple to 2-tuple (id, start, stop) -> (start, stop)
        Ok(data
            .dark_periods
            .get(&schedule_id)
            .map(|periods| periods.iter().map(|(_, start, stop)| (*start, *stop)).collect())
            .unwrap_or_default())
    }

    async fn fetch_possible_periods(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<(i64, f64, f64)>> {
        let data = self.data.read().unwrap();
        
        Ok(data
            .possible_periods
            .get(&schedule_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn populate_schedule_analytics(&self, schedule_id: i64) -> RepositoryResult<usize> {
        self.get_schedule_impl(schedule_id)?;
        
        let mut data = self.data.write().unwrap();
        data.analytics_exists.insert(schedule_id, true);
        Ok(1) // Simulated row count
    }

    async fn delete_schedule_analytics(&self, schedule_id: i64) -> RepositoryResult<usize> {
        Ok(self.delete_from_map(|d| &mut d.analytics_exists, schedule_id))
    }

    async fn has_analytics_data(&self, schedule_id: i64) -> RepositoryResult<bool> {
        let data = self.data.read().unwrap();
        Ok(data.analytics_exists.get(&schedule_id).copied().unwrap_or(false))
    }

    async fn fetch_analytics_blocks_for_sky_map(
        &self,
        _schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::LightweightBlock>> {
        Ok(vec![])
    }

    async fn fetch_analytics_blocks_for_distribution(
        &self,
        _schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::DistributionBlock>> {
        Ok(vec![])
    }

    async fn populate_summary_analytics(
        &self,
        schedule_id: i64,
        _n_bins: usize,
    ) -> RepositoryResult<()> {
        self.get_schedule_impl(schedule_id)?;
        
        let mut data = self.data.write().unwrap();

        // Create dummy summary
        let summary = analytics::ScheduleSummary {
            schedule_id,
            total_blocks: 100,
            scheduled_blocks: 95,
            unscheduled_blocks: 5,
            impossible_blocks: 5,
            scheduling_rate: 0.95,
            priority_min: Some(0.0),
            priority_max: Some(1.0),
            priority_mean: Some(0.5),
            priority_median: Some(0.5),
            priority_scheduled_mean: Some(0.6),
            priority_unscheduled_mean: Some(0.3),
            visibility_total_hours: 500.0,
            visibility_mean_hours: Some(5.0),
            requested_total_hours: 250.0,
            requested_mean_hours: Some(2.5),
            scheduled_total_hours: 237.5,
            corr_priority_visibility: Some(0.5),
            corr_priority_requested: Some(0.3),
            corr_visibility_requested: Some(0.7),
            conflict_count: 10,
        };

        data.summary_analytics.insert(schedule_id, summary);
        Ok(())
    }

    async fn fetch_schedule_summary(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Option<analytics::ScheduleSummary>> {
        let data = self.data.read().unwrap();
        Ok(data.summary_analytics.get(&schedule_id).cloned())
    }

    async fn fetch_priority_rates(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<analytics::PriorityRate>> {
        let data = self.data.read().unwrap();
        Ok(data
            .priority_rates
            .get(&schedule_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn fetch_visibility_bins(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<analytics::VisibilityBin>> {
        let data = self.data.read().unwrap();
        Ok(data
            .visibility_bins
            .get(&schedule_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn fetch_heatmap_bins(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<analytics::HeatmapBinData>> {
        let data = self.data.read().unwrap();
        Ok(data
            .heatmap_bins
            .get(&schedule_id)
            .cloned()
            .unwrap_or_default())
    }

    async fn has_summary_analytics(&self, schedule_id: i64) -> RepositoryResult<bool> {
        let data = self.data.read().unwrap();
        Ok(data.summary_analytics.contains_key(&schedule_id))
    }

    async fn delete_summary_analytics(&self, schedule_id: i64) -> RepositoryResult<usize> {
        Ok(self.delete_from_map(|d| &mut d.summary_analytics, schedule_id))
    }

    async fn populate_visibility_time_bins(
        &self,
        schedule_id: i64,
        _bin_duration_seconds: Option<i64>,
    ) -> RepositoryResult<(usize, usize)> {
        self.get_schedule_impl(schedule_id)?;
        
        let mut data = self.data.write().unwrap();

        // Create dummy bins
        data.visibility_time_bins.insert(schedule_id, vec![]);
        Ok((1, 0)) // metadata rows, bin rows
    }

    async fn fetch_visibility_histogram_from_analytics(
        &self,
        _schedule_id: i64,
        _start_unix: i64,
        _end_unix: i64,
        _target_bin_duration_seconds: i64,
    ) -> RepositoryResult<Vec<crate::db::models::VisibilityBin>> {
        Ok(vec![])
    }

    async fn fetch_visibility_metadata(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Option<analytics::VisibilityTimeMetadata>> {
        let data = self.data.read().unwrap();
        Ok(data.visibility_metadata.get(&schedule_id).cloned())
    }

    async fn has_visibility_time_bins(&self, schedule_id: i64) -> RepositoryResult<bool> {
        let data = self.data.read().unwrap();
        Ok(data.visibility_time_bins.contains_key(&schedule_id))
    }

    async fn delete_visibility_time_bins(&self, schedule_id: i64) -> RepositoryResult<usize> {
        Ok(self.delete_from_map(|d| &mut d.visibility_time_bins, schedule_id))
    }

    async fn insert_validation_results(
        &self,
        results: &[ValidationResult],
    ) -> RepositoryResult<usize> {
        if results.is_empty() {
            return Ok(0);
        }

        let mut data = self.data.write().unwrap();
        let schedule_id = results[0].schedule_id;

        // Convert to ValidationReportData
        let issues: Vec<validation::ValidationIssue> = results
            .iter()
            .map(|r| validation::ValidationIssue {
                block_id: r.scheduling_block_id,
                original_block_id: None,
                issue_type: r.issue_type.clone().unwrap_or_default(),
                category: format!("{:?}", r.issue_category.as_ref().unwrap_or(&crate::services::validation::IssueCategory::Visibility)),
                criticality: format!("{:?}", r.criticality.as_ref().unwrap_or(&crate::services::validation::Criticality::High)),
                field_name: r.field_name.clone(),
                current_value: r.current_value.clone(),
                expected_value: r.expected_value.clone(),
                description: r.description.clone().unwrap_or_default(),
            })
            .collect();

        let report = validation::ValidationReportData {
            schedule_id,
            total_blocks: issues.len(),
            valid_blocks: 0,
            impossible_blocks: Vec::new(),
            validation_errors: issues,
            validation_warnings: Vec::new(),
        };

        data.validation_results.insert(schedule_id, report);
        Ok(results.len())
    }

    async fn fetch_validation_results(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<validation::ValidationReportData> {
        let data = self.data.read().unwrap();
        
        data.validation_results
            .get(&schedule_id)
            .cloned()
            .ok_or_else(|| {
                RepositoryError::NotFound(format!(
                    "No validation results for schedule {}",
                    schedule_id
                ))
            })
    }

    async fn has_validation_results(&self, schedule_id: i64) -> RepositoryResult<bool> {
        let data = self.data.read().unwrap();
        Ok(data.validation_results.contains_key(&schedule_id))
    }

    async fn delete_validation_results(&self, schedule_id: i64) -> RepositoryResult<u64> {
        Ok(self.delete_from_map(|d| &mut d.validation_results, schedule_id) as u64)
    }

    async fn fetch_lightweight_blocks(
        &self,
        _schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::LightweightBlock>> {
        Ok(vec![])
    }

    async fn fetch_insights_blocks(
        &self,
        _schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::InsightsBlock>> {
        Ok(vec![])
    }

    async fn fetch_trends_blocks(
        &self,
        _schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::TrendsBlock>> {
        Ok(vec![])
    }

    async fn fetch_visibility_map_data(
        &self,
        _schedule_id: i64,
    ) -> RepositoryResult<crate::db::models::VisibilityMapData> {
        Ok(crate::db::models::VisibilityMapData {
            blocks: vec![],
            priority_min: 0.0,
            priority_max: 1.0,
            total_count: 0,
            scheduled_count: 0,
        })
    }

    async fn fetch_blocks_for_histogram(
        &self,
        _schedule_id: i64,
        _priority_min: Option<i32>,
        _priority_max: Option<i32>,
        _block_ids: Option<Vec<i64>>,
    ) -> RepositoryResult<Vec<crate::db::models::BlockHistogramData>> {
        Ok(vec![])
    }

    async fn fetch_schedule_timeline_blocks(
        &self,
        _schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::ScheduleTimelineBlock>> {
        Ok(vec![])
    }

    async fn fetch_compare_blocks(
        &self,
        _schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::CompareBlock>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let repo = TestRepository::new();
        assert!(repo.health_check().await.unwrap());
        
        repo.set_healthy(false);
        assert!(!repo.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_store_and_retrieve_schedule() {
        let repo = TestRepository::new();
        
        let schedule = Schedule {
            id: None,
            name: "Test Schedule".to_string(),
            blocks: vec![],
            dark_periods: vec![],
            checksum: "test123".to_string(),
        };

        let metadata = repo.store_schedule(&schedule).await.unwrap();
        assert!(metadata.schedule_id.is_some());
        
        let retrieved = repo.get_schedule(metadata.schedule_id.unwrap()).await.unwrap();
        assert_eq!(retrieved.name, schedule.name);
    }

    #[tokio::test]
    async fn test_list_schedules() {
        let repo = TestRepository::new();
        
        let schedule1 = Schedule {
            id: None,
            name: "Schedule 1".to_string(),
            blocks: vec![],
            dark_periods: vec![],
            checksum: "hash1".to_string(),
        };
        
        let schedule2 = Schedule {
            id: None,
            name: "Schedule 2".to_string(),
            blocks: vec![],
            dark_periods: vec![],
            checksum: "hash2".to_string(),
        };

        repo.store_schedule(&schedule1).await.unwrap();
        repo.store_schedule(&schedule2).await.unwrap();
        
        let schedules = repo.list_schedules().await.unwrap();
        assert_eq!(schedules.len(), 2);
    }

    #[tokio::test]
    async fn test_not_found_error() {
        let repo = TestRepository::new();
        
        let result = repo.get_schedule(999).await;
        assert!(matches!(result, Err(RepositoryError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_analytics_operations() {
        let repo = TestRepository::new();
        
        let schedule = Schedule {
            id: None,
            name: "Test".to_string(),
            blocks: vec![],
            dark_periods: vec![],
            checksum: "test".to_string(),
        };

        let metadata = repo.store_schedule(&schedule).await.unwrap();
        let schedule_id = metadata.schedule_id.unwrap();

        assert!(!repo.has_analytics_data(schedule_id).await.unwrap());
        
        repo.populate_schedule_analytics(schedule_id).await.unwrap();
        assert!(repo.has_analytics_data(schedule_id).await.unwrap());
        
        repo.delete_schedule_analytics(schedule_id).await.unwrap();
        assert!(!repo.has_analytics_data(schedule_id).await.unwrap());
    }
}
