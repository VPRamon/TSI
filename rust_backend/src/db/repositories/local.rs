//! In-memory local repository implementation.
//!
//! This module provides a local implementation of all repository traits
//! suitable for unit testing and local development. All data is stored in memory using HashMap and Vec
//! structures, providing fast, deterministic, and isolated execution.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::api::ModifiedJulianDate;
use crate::api::*;
use crate::db::repository::*;
use crate::services::validation::ValidationResult;

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

struct LocalData {
    schedules: HashMap<crate::api::ScheduleId, Schedule>,
    schedule_metadata: HashMap<crate::api::ScheduleId, crate::api::ScheduleInfo>,
    blocks: HashMap<i64, SchedulingBlock>,
    possible_periods: HashMap<crate::api::ScheduleId, Vec<Period>>,

    // Analytics data
    analytics_exists: HashMap<crate::api::ScheduleId, bool>,

    // Validation data
    validation_results: HashMap<crate::api::ScheduleId, crate::api::ValidationReport>,

    // ID counters
    next_schedule_id: crate::api::ScheduleId,
    next_block_id: i64,

    // Connection health
    is_healthy: bool,
}

impl Default for LocalData {
    fn default() -> Self {
        Self {
            schedules: HashMap::new(),
            schedule_metadata: HashMap::new(),
            blocks: HashMap::new(),
            possible_periods: HashMap::new(),
            analytics_exists: HashMap::new(),
            validation_results: HashMap::new(),
            next_schedule_id: crate::api::ScheduleId(1),
            next_block_id: 1,
            is_healthy: true,
        }
    }
}

impl LocalRepository {
    /// Create a new empty local repository.
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(LocalData {
                is_healthy: true,
                next_schedule_id: crate::api::ScheduleId(1),
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
    pub fn store_schedule_impl(&self, mut schedule: Schedule) -> crate::api::ScheduleId {
        let mut data = self.data.write().unwrap();
        let schedule_id = data.next_schedule_id;
        data.next_schedule_id = crate::api::ScheduleId(data.next_schedule_id.0 + 1);

        // Assign IDs to blocks
        for block in &mut schedule.blocks {
            let block_id = data.next_block_id;
            data.next_block_id += 1;
            data.blocks.insert(block_id, block.clone());
        }

        let metadata = crate::api::ScheduleInfo {
            schedule_id,
            schedule_name: schedule.name.clone(),
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
            next_schedule_id: crate::api::ScheduleId(1),
            next_block_id: 1,
            ..Default::default()
        };
    }

    /// Get the number of schedules stored.
    pub fn schedule_count(&self) -> usize {
        self.data.read().unwrap().schedules.len()
    }

    /// Check if a schedule exists.
    pub fn has_schedule(&self, schedule_id: crate::api::ScheduleId) -> bool {
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
    fn get_schedule_impl(&self, schedule_id: crate::api::ScheduleId) -> RepositoryResult<Schedule> {
        let data = self.data.read().unwrap();
        data.schedules.get(&schedule_id).cloned().ok_or_else(|| {
            RepositoryError::NotFound(format!("Schedule {} not found", schedule_id.0))
        })
    }

    /// Helper for the common deletion pattern.
    fn delete_from_map<T>(
        &self,
        map_accessor: impl FnOnce(&mut LocalData) -> &mut HashMap<crate::api::ScheduleId, T>,
        schedule_id: crate::api::ScheduleId,
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

    async fn store_schedule(
        &self,
        schedule: &Schedule,
    ) -> RepositoryResult<crate::api::ScheduleInfo> {
        self.check_health()?;

        // Use the helper method to add the schedule
        let schedule_id = self.store_schedule_impl(schedule.clone());

        // Retrieve and return the metadata
        let data = self.data.read().unwrap();
        let metadata = data.schedule_metadata.get(&schedule_id).cloned().unwrap();

        Ok(metadata)
    }

    async fn get_schedule(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Schedule> {
        self.get_schedule_impl(schedule_id)
    }

    async fn list_schedules(&self) -> RepositoryResult<Vec<crate::api::ScheduleInfo>> {
        let data = self.data.read().unwrap();

        let mut schedules: Vec<crate::api::ScheduleInfo> =
            data.schedule_metadata.values().cloned().collect();

        schedules.sort_by_key(|s| s.schedule_id);
        Ok(schedules)
    }

    async fn get_schedule_time_range(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Option<Period>> {
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
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<SchedulingBlock>> {
        let schedule = self.get_schedule_impl(schedule_id)?;
        Ok(schedule.blocks.clone())
    }

    async fn fetch_dark_periods(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<Period>> {
        let schedule = self.get_schedule_impl(schedule_id)?;
        Ok(schedule.dark_periods.clone())
    }

    async fn fetch_possible_periods(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<Period>> {
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
    async fn populate_schedule_analytics(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<usize> {
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
                crate::api::ValidationReport {
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

    async fn delete_schedule_analytics(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<usize> {
        Ok(self.delete_from_map(|d| &mut d.analytics_exists, schedule_id))
    }

    async fn has_analytics_data(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<bool> {
        let data = self.data.read().unwrap();
        Ok(data
            .analytics_exists
            .get(&schedule_id)
            .copied()
            .unwrap_or(false))
    }

    async fn fetch_analytics_blocks_for_sky_map(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<crate::api::LightweightBlock>> {
        use crate::api::LightweightBlock;

        let schedule = self.get_schedule_impl(schedule_id)?;

        // Convert schedule blocks to LightweightBlock format
        let blocks: Vec<LightweightBlock> = schedule
            .blocks
            .iter()
            .map(|b| {
                // Use original_block_id if available, otherwise fallback to the parsed block id
                let original_block_id = b
                    .original_block_id
                    .clone()
                    .unwrap_or_else(|| "-".to_string());

                LightweightBlock {
                    original_block_id,
                    priority: b.priority,
                    priority_bin: "".to_string(), // Will be computed by sky_map service
                    requested_duration_seconds: b.requested_duration.value(),
                    target_ra_deg: b.target_ra.value(),
                    target_dec_deg: b.target_dec.value(),
                    scheduled_period: b.scheduled_period.as_ref().map(|p| p.clone()),
                }
            })
            .collect();

        Ok(blocks)
    }

    async fn fetch_analytics_blocks_for_distribution(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<crate::api::DistributionBlock>> {
        use crate::api::DistributionBlock;

        let schedule = self.get_schedule_impl(schedule_id)?;

        // Convert schedule blocks to DistributionBlock format
        let blocks: Vec<DistributionBlock> = schedule
            .blocks
            .iter()
            .map(|b| {
                let total_visibility_hours_f64: f64 = b
                    .visibility_periods
                    .iter()
                    .map(|p| p.duration().value() * 24.0)
                    .sum();

                let requested_hours_f64 = b.requested_duration.value() / 3600.0;

                let elevation_range_deg =
                    b.constraints.max_alt.value() - b.constraints.min_alt.value();

                DistributionBlock {
                    priority: b.priority,
                    total_visibility_hours: total_visibility_hours_f64,
                    requested_hours: requested_hours_f64,
                    elevation_range_deg,
                    scheduled: b.scheduled_period.is_some(),
                }
            })
            .collect();

        Ok(blocks)
    }

    async fn fetch_analytics_blocks_for_insights(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<InsightsBlock>> {
        let schedule = self.get_schedule_impl(schedule_id)?;

        let blocks = schedule
            .blocks
            .iter()
            .map(|b| {
                let total_visibility_hours: f64 = b
                    .visibility_periods
                    .iter()
                    .map(|p| p.duration().value() * 24.0)
                    .sum();

                InsightsBlock {
                    scheduling_block_id: b.id.0,
                    original_block_id: b
                        .original_block_id
                        .clone()
                        .unwrap_or_else(|| b.id.0.to_string()),
                    priority: b.priority,
                    total_visibility_hours: qtty::time::Hours::new(total_visibility_hours),
                    requested_hours: qtty::time::Hours::new(b.requested_duration.value() / 3600.0),
                    elevation_range_deg: qtty::angular::Degrees::new(
                        b.constraints.max_alt.value() - b.constraints.min_alt.value(),
                    ),
                    scheduled: b.scheduled_period.is_some(),
                    scheduled_start_mjd: b.scheduled_period.as_ref().map(|p| p.start.clone()),
                    scheduled_stop_mjd: b.scheduled_period.as_ref().map(|p| p.stop.clone()),
                }
            })
            .collect();

        Ok(blocks)
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
                    impossible_blocks.push(crate::api::ValidationIssue {
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
                    validation_errors.push(crate::api::ValidationIssue {
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
                    validation_warnings.push(crate::api::ValidationIssue {
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

        let report = crate::api::ValidationReport {
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
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<crate::api::ValidationReport> {
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

    async fn has_validation_results(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<bool> {
        let data = self.data.read().unwrap();
        Ok(data.validation_results.contains_key(&schedule_id))
    }

    async fn delete_validation_results(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<u64> {
        Ok(self.delete_from_map(|d| &mut d.validation_results, schedule_id) as u64)
    }
}

// ==================== Visualization Repository ====================

#[async_trait]
impl VisualizationRepository for LocalRepository {
    async fn fetch_visibility_map_data(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<crate::api::VisibilityMapData> {
        use crate::api::VisibilityBlockSummary;

        let schedule = self.get_schedule_impl(schedule_id)?;

        if schedule.blocks.is_empty() {
            return Ok(crate::api::VisibilityMapData {
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
            .map(|b| {
                // Use original_block_id if available, otherwise fallback to the parsed block id
                let original_block_id = b
                    .original_block_id
                    .clone()
                    .unwrap_or_else(|| b.id.0.to_string());

                VisibilityBlockSummary {
                    scheduling_block_id: b.id.0,
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

        Ok(crate::api::VisibilityMapData {
            blocks,
            priority_min,
            priority_max,
            total_count,
            scheduled_count,
        })
    }

    async fn fetch_blocks_for_histogram(
        &self,
        schedule_id: crate::api::ScheduleId,
        priority_min: Option<i32>,
        priority_max: Option<i32>,
        block_ids: Option<Vec<i64>>,
    ) -> RepositoryResult<Vec<crate::services::visibility::BlockHistogramData>> {
        use crate::services::visibility::BlockHistogramData;

        let schedule = self.get_schedule_impl(schedule_id)?;

        // Convert schedule blocks to BlockHistogramData format
        let blocks: Vec<BlockHistogramData> = schedule
            .blocks
            .iter()
            .filter_map(|b| {
                let block_id = b.id.0;
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

                // Return visibility periods directly as Vec<Period>
                let visibility_periods = if !b.visibility_periods.is_empty() {
                    Some(b.visibility_periods.clone())
                } else {
                    None
                };

                Some(BlockHistogramData {
                    scheduling_block_id: block_id,
                    priority,
                    visibility_periods,
                })
            })
            .collect();

        Ok(blocks)
    }

    async fn fetch_schedule_timeline_blocks(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<crate::api::ScheduleTimelineBlock>> {
        use crate::api::ScheduleTimelineBlock;

        let schedule = self.get_schedule_impl(schedule_id)?;

        // Convert schedule blocks to ScheduleTimelineBlock format
        let blocks: Vec<ScheduleTimelineBlock> = schedule
            .blocks
            .iter()
            .filter_map(|b| {
                // Only include scheduled blocks
                let scheduled_period = b.scheduled_period.as_ref()?;

                let total_visibility_hours = b
                    .visibility_periods
                    .iter()
                    .map(|p| p.duration().value() * 24.0)
                    .sum();

                let requested_hours = b.requested_duration.value() / 3600.0;

                // Use original_block_id if available, otherwise fallback to the parsed block id
                let original_block_id = b
                    .original_block_id
                    .clone()
                    .unwrap_or_else(|| b.id.0.to_string());

                Some(ScheduleTimelineBlock {
                    scheduling_block_id: b.id.0,
                    original_block_id,
                    priority: b.priority,
                    scheduled_start_mjd: scheduled_period.start.clone(),
                    scheduled_stop_mjd: scheduled_period.stop.clone(),
                    ra_deg: b.target_ra,
                    dec_deg: b.target_dec,
                    requested_hours: qtty::time::Hours::new(requested_hours),
                    total_visibility_hours: qtty::time::Hours::new(total_visibility_hours),
                    num_visibility_periods: b.visibility_periods.len(),
                })
            })
            .collect();

        Ok(blocks)
    }

    async fn fetch_compare_blocks(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<crate::api::CompareBlock>> {
        use crate::api::CompareBlock;

        let schedule = self.get_schedule_impl(schedule_id)?;

        // Convert schedule blocks to CompareBlock format
        let blocks: Vec<CompareBlock> = schedule
            .blocks
            .iter()
            .map(|b| {
                let requested_hours_f64 = b.requested_duration.value() / 3600.0;

                CompareBlock {
                    scheduling_block_id: b.id.0.to_string(),
                    priority: b.priority,
                    scheduled: b.scheduled_period.is_some(),
                    requested_hours: qtty::time::Hours::new(requested_hours_f64),
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
        let schedule_id = metadata.schedule_id;

        let retrieved = repo
            .get_schedule(schedule_id)
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

        let result = repo.get_schedule(ScheduleId(999)).await;
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
        let schedule_id = metadata.schedule_id;

        assert!(!repo.has_analytics_data(schedule_id).await.unwrap());

        repo.populate_schedule_analytics(schedule_id).await.unwrap();
        assert!(repo.has_analytics_data(schedule_id).await.unwrap());

        repo.delete_schedule_analytics(schedule_id).await.unwrap();
        assert!(!repo.has_analytics_data(schedule_id).await.unwrap());
    }
}
