//! In-memory local repository implementation.
//!
//! This module provides a local implementation of all repository traits
//! suitable for unit testing and local development. All data is stored in memory using HashMap and Vec
//! structures, providing fast, deterministic, and isolated execution.

use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::db::{
    analytics,
    models::{Period, Schedule, ScheduleInfo, ScheduleMetadata, SchedulingBlock},
    repository::*,
    validation,
};
use crate::services::validation::ValidationResult;
use siderust::astro::ModifiedJulianDate;

/// In-memory local repository.
///
/// This implementation stores all data in memory using HashMaps and Vecs,
/// making it ideal for unit tests and local development that need isolation and speed.
///
/// # Example
/// ```
/// use tsi_rust::db::repositories::LocalRepository;
///
/// #[tokio::test]
/// async fn test_schedule_storage() {
///     let repo = LocalRepository::new();
///     
///     // Pre-populate with test data
///     repo.store_schedule_impl(/* ... */);
///     
///     let schedules = repo.list_schedules().await.unwrap();
///     assert_eq!(schedules.len(), 1);
/// }
/// ```
#[derive(Clone)]
pub struct LocalRepository {
    data: Arc<RwLock<LocalData>>,
}

#[derive(Default)]
struct LocalData {
    schedules: HashMap<i64, Schedule>,
    schedule_metadata: HashMap<i64, ScheduleMetadata>,
    blocks: HashMap<i64, SchedulingBlock>,
    possible_periods: HashMap<i64, Vec<Period>>,

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

    // Connection health
    is_healthy: bool,
}

impl LocalRepository {
    /// Create a new empty local repository.
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(LocalData {
                is_healthy: true,
                next_schedule_id: 1,
                next_block_id: 1,
                ..Default::default()
            })),
        }
    }

    /// Add a schedule to the repository.
    ///
    /// This is a helper method for setting up data. The schedule will be
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

    /// Set the health status for testing connection failures.
    pub fn set_healthy(&self, healthy: bool) {
        let mut data = self.data.write().unwrap();
        data.is_healthy = healthy;
    }

    /// Clear all data from the repository.
    pub fn clear(&self) {
        let mut data = self.data.write().unwrap();
        *data = LocalData {
            is_healthy: data.is_healthy,
            next_schedule_id: 1,
            next_block_id: 1,
            ..Default::default()
        };
    }

    /// Get the number of schedules stored.
    pub fn schedule_count(&self) -> usize {
        self.data.read().unwrap().schedules.len()
    }

    /// Check if a schedule exists.
    pub fn has_schedule(&self, schedule_id: i64) -> bool {
        self.data
            .read()
            .unwrap()
            .schedules
            .contains_key(&schedule_id)
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
    fn delete_from_map<T>(
        &self,
        map_accessor: impl FnOnce(&mut LocalData) -> &mut HashMap<i64, T>,
        schedule_id: i64,
    ) -> usize {
        let mut data = self.data.write().unwrap();
        let existed = map_accessor(&mut data).remove(&schedule_id).is_some();
        if existed {
            1
        } else {
            0
        }
    }
}

impl Default for LocalRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ScheduleRepository for LocalRepository {
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

    async fn get_schedule_time_range(&self, schedule_id: i64) -> RepositoryResult<Option<Period>> {
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

        Ok(Period::new(
            ModifiedJulianDate::new(min_start),
            ModifiedJulianDate::new(max_stop),
        ))
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
                RepositoryError::NotFound(format!(
                    "Scheduling block {} not found",
                    scheduling_block_id
                ))
            })
    }

    async fn get_blocks_for_schedule(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<SchedulingBlock>> {
        let schedule = self.get_schedule_impl(schedule_id)?;
        Ok(schedule.blocks.clone())
    }

    async fn fetch_dark_periods(&self, schedule_id: i64) -> RepositoryResult<Vec<Period>> {
        let schedule = self.get_schedule_impl(schedule_id)?;
        Ok(schedule.dark_periods.clone())
    }

    async fn fetch_possible_periods(&self, schedule_id: i64) -> RepositoryResult<Vec<Period>> {
        let data = self.data.read().unwrap();

        Ok(data
            .possible_periods
            .get(&schedule_id)
            .cloned()
            .unwrap_or_default())
    }
}

// ==================== Analytics Repository ====================

#[async_trait]
impl AnalyticsRepository for LocalRepository {
    async fn populate_schedule_analytics(&self, schedule_id: i64) -> RepositoryResult<usize> {
        let schedule = self.get_schedule_impl(schedule_id)?;

        // Build validation input from schedule blocks
        let blocks_for_validation: Vec<crate::services::validation::BlockForValidation> = schedule
            .blocks
            .iter()
            .map(|b| {
                crate::services::validation::BlockForValidation {
                    schedule_id,
                    scheduling_block_id: b.id.0, // Use actual block ID
                    priority: b.priority,
                    requested_duration_sec: b.requested_duration.value() as i32,
                    min_observation_sec: b.min_observation.value() as i32,
                    total_visibility_hours: b
                        .visibility_periods
                        .iter()
                        .map(|p| p.duration().value() * 24.0)
                        .sum(),
                    min_alt_deg: Some(b.constraints.min_alt.value()),
                    max_alt_deg: Some(b.constraints.max_alt.value()),
                    constraint_start_mjd: b
                        .constraints
                        .fixed_time
                        .as_ref()
                        .map(|p| p.start.value()),
                    constraint_stop_mjd: b.constraints.fixed_time.as_ref().map(|p| p.stop.value()),
                    scheduled_start_mjd: b.scheduled_period.as_ref().map(|p| p.start.value()),
                    scheduled_stop_mjd: b.scheduled_period.as_ref().map(|p| p.stop.value()),
                    target_ra_deg: b.target_ra.value(),
                    target_dec_deg: b.target_dec.value(),
                }
            })
            .collect();

        // Run validation (even for empty schedules to create empty report)
        let validation_results = if blocks_for_validation.is_empty() {
            // Create empty validation report for empty schedules
            let mut data = self.data.write().unwrap();
            data.validation_results.insert(
                schedule_id,
                validation::ValidationReportData {
                    schedule_id,
                    total_blocks: 0,
                    valid_blocks: 0,
                    impossible_blocks: Vec::new(),
                    validation_errors: Vec::new(),
                    validation_warnings: Vec::new(),
                },
            );
            Vec::new()
        } else {
            crate::services::validation::validate_blocks(&blocks_for_validation)
        };

        // Store validation results if non-empty
        if !validation_results.is_empty() {
            let results_to_insert = validation_results.clone();
            let _validation_count =
                ValidationRepository::insert_validation_results(self, &results_to_insert).await?;
        }

        // Mark analytics as populated
        let mut data = self.data.write().unwrap();
        data.analytics_exists.insert(schedule_id, true);

        // Return the number of blocks processed (not validation count)
        Ok(schedule.blocks.len())
    }

    async fn delete_schedule_analytics(&self, schedule_id: i64) -> RepositoryResult<usize> {
        Ok(self.delete_from_map(|d| &mut d.analytics_exists, schedule_id))
    }

    async fn has_analytics_data(&self, schedule_id: i64) -> RepositoryResult<bool> {
        let data = self.data.read().unwrap();
        Ok(data
            .analytics_exists
            .get(&schedule_id)
            .copied()
            .unwrap_or(false))
    }

    async fn fetch_analytics_blocks_for_sky_map(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::LightweightBlock>> {
        use crate::db::models::LightweightBlock;

        let schedule = self.get_schedule_impl(schedule_id)?;

        // Convert schedule blocks to LightweightBlock format
        let blocks: Vec<LightweightBlock> = schedule
            .blocks
            .iter()
            .enumerate()
            .map(|(idx, b)| {
                // Use original_block_id if available, otherwise fallback to internal ID
                let original_block_id = b
                    .original_block_id
                    .clone()
                    .unwrap_or_else(|| format!("{}", idx + 1));

                LightweightBlock {
                    original_block_id,
                    priority: b.priority,
                    priority_bin: "".to_string(), // Will be computed by sky_map service
                    requested_duration_seconds: b.requested_duration.value(),
                    target_ra_deg: b.target_ra.value(),
                    target_dec_deg: b.target_dec.value(),
                    scheduled_period: b.scheduled_period,
                }
            })
            .collect();

        Ok(blocks)
    }

    async fn fetch_analytics_blocks_for_distribution(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::DistributionBlock>> {
        use crate::db::models::DistributionBlock;

        let schedule = self.get_schedule_impl(schedule_id)?;

        // Convert schedule blocks to DistributionBlock format
        let blocks: Vec<DistributionBlock> = schedule
            .blocks
            .iter()
            .map(|b| {
                let total_visibility_hours = b
                    .visibility_periods
                    .iter()
                    .map(|p| p.duration().value() * 24.0)
                    .sum();

                let requested_hours = b.requested_duration.value() / 3600.0;

                let elevation_range_deg =
                    b.constraints.max_alt.value() - b.constraints.min_alt.value();

                DistributionBlock {
                    priority: b.priority,
                    total_visibility_hours,
                    requested_hours,
                    elevation_range_deg,
                    scheduled: b.scheduled_period.is_some(),
                }
            })
            .collect();

        Ok(blocks)
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
}

// ==================== Validation Repository ====================

#[async_trait]
impl ValidationRepository for LocalRepository {
    async fn insert_validation_results(
        &self,
        results: &[ValidationResult],
    ) -> RepositoryResult<usize> {
        if results.is_empty() {
            return Ok(0);
        }

        let mut data = self.data.write().unwrap();
        let schedule_id = results[0].schedule_id;

        // Separate results by status
        let mut impossible_blocks = Vec::new();
        let mut validation_errors = Vec::new();
        let mut validation_warnings = Vec::new();
        let mut valid_count = 0;

        for r in results {
            use crate::services::validation::ValidationStatus;

            match r.status {
                ValidationStatus::Valid => {
                    valid_count += 1;
                }
                ValidationStatus::Impossible => {
                    impossible_blocks.push(validation::ValidationIssue {
                        block_id: r.scheduling_block_id,
                        original_block_id: None,
                        issue_type: r.issue_type.clone().unwrap_or_default(),
                        category: r
                            .issue_category
                            .as_ref()
                            .map(|c| c.as_str().to_string())
                            .unwrap_or_default(),
                        criticality: r
                            .criticality
                            .as_ref()
                            .map(|c| c.as_str().to_string())
                            .unwrap_or_default(),
                        field_name: r.field_name.clone(),
                        current_value: r.current_value.clone(),
                        expected_value: r.expected_value.clone(),
                        description: r.description.clone().unwrap_or_default(),
                    });
                }
                ValidationStatus::Error => {
                    validation_errors.push(validation::ValidationIssue {
                        block_id: r.scheduling_block_id,
                        original_block_id: None,
                        issue_type: r.issue_type.clone().unwrap_or_default(),
                        category: r
                            .issue_category
                            .as_ref()
                            .map(|c| c.as_str().to_string())
                            .unwrap_or_default(),
                        criticality: r
                            .criticality
                            .as_ref()
                            .map(|c| c.as_str().to_string())
                            .unwrap_or_default(),
                        field_name: r.field_name.clone(),
                        current_value: r.current_value.clone(),
                        expected_value: r.expected_value.clone(),
                        description: r.description.clone().unwrap_or_default(),
                    });
                }
                ValidationStatus::Warning => {
                    validation_warnings.push(validation::ValidationIssue {
                        block_id: r.scheduling_block_id,
                        original_block_id: None,
                        issue_type: r.issue_type.clone().unwrap_or_default(),
                        category: r
                            .issue_category
                            .as_ref()
                            .map(|c| c.as_str().to_string())
                            .unwrap_or_default(),
                        criticality: r
                            .criticality
                            .as_ref()
                            .map(|c| c.as_str().to_string())
                            .unwrap_or_default(),
                        field_name: r.field_name.clone(),
                        current_value: r.current_value.clone(),
                        expected_value: r.expected_value.clone(),
                        description: r.description.clone().unwrap_or_default(),
                    });
                }
            }
        }

        // Get unique block count
        let unique_blocks: std::collections::HashSet<i64> =
            results.iter().map(|r| r.scheduling_block_id).collect();
        let total_blocks = unique_blocks.len();

        let report = validation::ValidationReportData {
            schedule_id,
            total_blocks,
            valid_blocks: valid_count,
            impossible_blocks,
            validation_errors,
            validation_warnings,
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
}

// ==================== Visualization Repository ====================

#[async_trait]
impl VisualizationRepository for LocalRepository {
    async fn fetch_visibility_map_data(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<crate::db::models::VisibilityMapData> {
        use crate::db::models::VisibilityBlockSummary;

        let schedule = self.get_schedule_impl(schedule_id)?;

        if schedule.blocks.is_empty() {
            return Ok(crate::db::models::VisibilityMapData {
                blocks: vec![],
                priority_min: 0.0,
                priority_max: 1.0,
                total_count: 0,
                scheduled_count: 0,
            });
        }

        // Convert schedule blocks to VisibilityBlockSummary
        let blocks: Vec<VisibilityBlockSummary> = schedule
            .blocks
            .iter()
            .enumerate()
            .map(|(idx, b)| {
                // Use original_block_id if available, otherwise fallback to internal ID
                let original_block_id = b
                    .original_block_id
                    .clone()
                    .unwrap_or_else(|| format!("{}", idx + 1));

                VisibilityBlockSummary {
                    scheduling_block_id: idx as i64 + 1,
                    original_block_id,
                    priority: b.priority,
                    num_visibility_periods: b.visibility_periods.len(),
                    scheduled: b.scheduled_period.is_some(),
                }
            })
            .collect();

        // Compute statistics
        let priority_min = blocks
            .iter()
            .map(|b| b.priority)
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let priority_max = blocks
            .iter()
            .map(|b| b.priority)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(1.0);
        let total_count = blocks.len();
        let scheduled_count = blocks.iter().filter(|b| b.scheduled).count();

        Ok(crate::db::models::VisibilityMapData {
            blocks,
            priority_min,
            priority_max,
            total_count,
            scheduled_count,
        })
    }

    async fn fetch_blocks_for_histogram(
        &self,
        schedule_id: i64,
        priority_min: Option<i32>,
        priority_max: Option<i32>,
        block_ids: Option<Vec<i64>>,
    ) -> RepositoryResult<Vec<crate::db::models::BlockHistogramData>> {
        use crate::db::models::BlockHistogramData;

        let schedule = self.get_schedule_impl(schedule_id)?;

        // Convert schedule blocks to BlockHistogramData format
        let blocks: Vec<BlockHistogramData> = schedule
            .blocks
            .iter()
            .enumerate()
            .filter_map(|(idx, b)| {
                let block_id = idx as i64 + 1;
                let priority = b.priority as i32;

                // Apply filters if provided
                if let Some(block_ids_filter) = &block_ids {
                    if !block_ids_filter.contains(&block_id) {
                        return None;
                    }
                }

                if let Some(min_priority) = priority_min {
                    if priority < min_priority {
                        return None;
                    }
                }

                if let Some(max_priority) = priority_max {
                    if priority > max_priority {
                        return None;
                    }
                }

                // Convert visibility periods to JSON string
                let visibility_periods_json = if !b.visibility_periods.is_empty() {
                    let periods: Vec<serde_json::Value> = b
                        .visibility_periods
                        .iter()
                        .map(|p| {
                            serde_json::json!({
                                "start": p.start.value(),
                                "stop": p.stop.value()
                            })
                        })
                        .collect();
                    Some(serde_json::to_string(&periods).unwrap_or_default())
                } else {
                    None
                };

                Some(BlockHistogramData {
                    scheduling_block_id: block_id,
                    priority,
                    visibility_periods_json,
                })
            })
            .collect();

        Ok(blocks)
    }

    async fn fetch_schedule_timeline_blocks(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::ScheduleTimelineBlock>> {
        use crate::db::models::ScheduleTimelineBlock;

        let schedule = self.get_schedule_impl(schedule_id)?;

        // Convert schedule blocks to ScheduleTimelineBlock format
        let blocks: Vec<ScheduleTimelineBlock> = schedule
            .blocks
            .iter()
            .enumerate()
            .filter_map(|(idx, b)| {
                // Only include scheduled blocks
                let scheduled_period = b.scheduled_period.as_ref()?;

                let total_visibility_hours = b
                    .visibility_periods
                    .iter()
                    .map(|p| p.duration().value() * 24.0)
                    .sum();

                let requested_hours = b.requested_duration.value() / 3600.0;

                // Use original_block_id if available, otherwise fallback to internal ID
                let original_block_id = b
                    .original_block_id
                    .clone()
                    .unwrap_or_else(|| format!("{}", idx + 1));

                Some(ScheduleTimelineBlock {
                    scheduling_block_id: idx as i64 + 1,
                    original_block_id,
                    priority: b.priority,
                    scheduled_start_mjd: scheduled_period.start.value(),
                    scheduled_stop_mjd: scheduled_period.stop.value(),
                    ra_deg: b.target_ra.value(),
                    dec_deg: b.target_dec.value(),
                    requested_hours,
                    total_visibility_hours,
                    num_visibility_periods: b.visibility_periods.len(),
                })
            })
            .collect();

        Ok(blocks)
    }

    async fn fetch_compare_blocks(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<crate::db::models::CompareBlock>> {
        use crate::db::models::CompareBlock;

        let schedule = self.get_schedule_impl(schedule_id)?;

        // Convert schedule blocks to CompareBlock format
        let blocks: Vec<CompareBlock> = schedule
            .blocks
            .iter()
            .enumerate()
            .map(|(idx, b)| {
                let requested_hours = b.requested_duration.value() / 3600.0;

                CompareBlock {
                    scheduling_block_id: format!("{}", idx + 1),
                    priority: b.priority,
                    scheduled: b.scheduled_period.is_some(),
                    requested_hours,
                }
            })
            .collect();

        Ok(blocks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_check() {
        let repo = LocalRepository::new();
        assert!(repo.health_check().await.unwrap());

        repo.set_healthy(false);
        assert!(!repo.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_store_and_retrieve_schedule() {
        let repo = LocalRepository::new();

        let schedule = Schedule {
            id: None,
            name: "Test Schedule".to_string(),
            blocks: vec![],
            dark_periods: vec![],
            checksum: "test123".to_string(),
        };

        let metadata = repo.store_schedule(&schedule).await.unwrap();
        assert!(metadata.schedule_id.is_some());

        let retrieved = repo
            .get_schedule(metadata.schedule_id.unwrap())
            .await
            .unwrap();
        assert_eq!(retrieved.name, schedule.name);
    }

    #[tokio::test]
    async fn test_list_schedules() {
        let repo = LocalRepository::new();

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
        let repo = LocalRepository::new();

        let result = repo.get_schedule(999).await;
        assert!(matches!(result, Err(RepositoryError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_analytics_operations() {
        let repo = LocalRepository::new();

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
