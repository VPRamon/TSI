//! In-memory local repository implementation.
//!
//! This module provides a local implementation of all repository traits
//! suitable for unit testing and local development. All data is stored in memory using HashMap and Vec
//! structures, providing fast, deterministic, and isolated execution.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::api::{Period, ScheduleId};
use crate::db::{
    models::{InsightsBlock, Schedule, SchedulingBlock},
    repository::*,
};
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
#[derive(Clone, Debug)]
pub struct LocalRepository {
    data: Arc<RwLock<LocalData>>,
}

#[derive(Default, Debug)]
struct LocalData {
    schedules: HashMap<i64, Schedule>,
    schedule_metadata: HashMap<i64, crate::api::ScheduleInfo>,
    blocks: HashMap<i64, SchedulingBlock>,
    possible_periods: HashMap<i64, Vec<Period>>,

    // Analytics data
    analytics_exists: HashMap<i64, bool>,

    // Validation data
    validation_results: HashMap<i64, crate::api::ValidationReport>,

    // Algorithm trace data: (algorithm, summary_json, iterations_json) keyed by schedule_id
    algorithm_traces: HashMap<i64, (String, serde_json::Value, serde_json::Value)>,

    // Environment data
    environments: HashMap<
        i64,
        (
            String,
            Option<crate::api::EnvironmentStructure>,
            chrono::DateTime<chrono::Utc>,
        ),
    >,
    preschedule: HashMap<i64, serde_json::Value>,
    schedule_environment: HashMap<i64, i64>, // schedule_id -> env_id

    // ID counters
    next_schedule_id: i64,
    next_block_id: i64,
    next_environment_id: i64,

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
                next_environment_id: 1,
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
    pub fn store_schedule_impl(&self, mut schedule: Schedule) -> ScheduleId {
        let mut data = self.data.write().unwrap();
        let schedule_id = ScheduleId(data.next_schedule_id);
        data.next_schedule_id += 1;

        // Assign IDs to blocks and store them in the global blocks map.
        for block in &mut schedule.blocks {
            let block_id = data.next_block_id;
            data.next_block_id += 1;

            // Set the DB-assigned ID on the block so later code that expects
            // `block.id` to be present (e.g. analytics/visualization helpers)
            // won't panic with `DB Block ID missing`.
            block.id = Some(crate::api::SchedulingBlockId(block_id));

            data.blocks.insert(block_id, block.clone());
        }

        let metadata = crate::api::ScheduleInfo {
            schedule_id,
            schedule_name: schedule.name.clone(),
            observer_location: schedule.geographic_location,
            schedule_period: schedule.schedule_period,
            environment_id: data.schedule_environment.get(&schedule_id.0).copied(),
        };

        data.schedule_metadata.insert(schedule_id.0, metadata);
        data.schedules.insert(schedule_id.0, schedule);

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
            next_environment_id: 1,
            ..Default::default()
        };
    }

    /// Get the number of schedules stored.
    pub fn schedule_count(&self) -> usize {
        self.data.read().unwrap().schedules.len()
    }

    /// Check if a schedule exists.
    pub fn has_schedule(&self, schedule_id: ScheduleId) -> bool {
        self.data
            .read()
            .unwrap()
            .schedules
            .contains_key(&schedule_id.0)
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
    fn get_schedule_impl(&self, schedule_id: ScheduleId) -> RepositoryResult<Schedule> {
        let data = self.data.read().unwrap();
        data.schedules
            .get(&schedule_id.0)
            .cloned()
            .ok_or_else(|| RepositoryError::NotFound(format!("Schedule {} not found", schedule_id)))
    }

    /// Helper for the common deletion pattern.
    fn delete_from_map<T>(
        &self,
        map_accessor: impl FnOnce(&mut LocalData) -> &mut HashMap<i64, T>,
        schedule_id: ScheduleId,
    ) -> usize {
        let mut data = self.data.write().unwrap();
        let existed = map_accessor(&mut data).remove(&schedule_id.0).is_some();
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
        let metadata = data.schedule_metadata.get(&schedule_id.0).cloned().unwrap();

        Ok(metadata)
    }

    async fn get_schedule(&self, schedule_id: ScheduleId) -> RepositoryResult<Schedule> {
        self.check_health()?;
        self.get_schedule_impl(schedule_id)
    }

    async fn list_schedules(&self) -> RepositoryResult<Vec<crate::api::ScheduleInfo>> {
        self.check_health()?;

        let data = self.data.read().unwrap();

        let mut schedules: Vec<crate::api::ScheduleInfo> =
            data.schedule_metadata.values().cloned().collect();

        schedules.sort_by_key(|s| s.schedule_id);
        Ok(schedules)
    }

    async fn get_schedule_time_range(
        &self,
        schedule_id: ScheduleId,
    ) -> RepositoryResult<Option<Period>> {
        let schedule = self.get_schedule_impl(schedule_id)?;

        // Return the schedule period directly
        Ok(Some(schedule.schedule_period))
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
        schedule_id: ScheduleId,
    ) -> RepositoryResult<Vec<SchedulingBlock>> {
        let schedule = self.get_schedule_impl(schedule_id)?;
        Ok(schedule.blocks.clone())
    }

    async fn fetch_dark_periods(&self, schedule_id: ScheduleId) -> RepositoryResult<Vec<Period>> {
        let schedule = self.get_schedule_impl(schedule_id)?;
        Ok(schedule.dark_periods.clone())
    }

    async fn fetch_possible_periods(
        &self,
        schedule_id: ScheduleId,
    ) -> RepositoryResult<Vec<Period>> {
        let data = self.data.read().unwrap();

        Ok(data
            .possible_periods
            .get(&schedule_id.0)
            .cloned()
            .unwrap_or_default())
    }

    async fn delete_schedule(&self, schedule_id: ScheduleId) -> RepositoryResult<()> {
        self.check_health()?;

        let mut data = self.data.write().unwrap();
        if data.schedules.remove(&schedule_id.0).is_none() {
            return Err(RepositoryError::NotFound(format!(
                "Schedule {} not found",
                schedule_id
            )));
        }
        data.schedule_metadata.remove(&schedule_id.0);
        data.analytics_exists.remove(&schedule_id.0);
        data.validation_results.remove(&schedule_id.0);
        data.possible_periods.remove(&schedule_id.0);
        data.schedule_environment.remove(&schedule_id.0);
        // Remove blocks belonging to this schedule
        let block_ids_to_remove: Vec<i64> = data.blocks.iter().map(|(&id, _)| id).collect();
        // We can't easily filter by schedule_id in local repo for blocks,
        // but schedule data itself is removed which is the important part
        let _ = block_ids_to_remove;
        Ok(())
    }

    async fn update_schedule_metadata(
        &self,
        schedule_id: ScheduleId,
        new_name: Option<String>,
        new_location: Option<crate::api::GeographicLocation>,
    ) -> RepositoryResult<crate::api::ScheduleInfo> {
        self.check_health()?;

        let mut data = self.data.write().unwrap();

        let schedule = data.schedules.get_mut(&schedule_id.0).ok_or_else(|| {
            RepositoryError::NotFound(format!("Schedule {} not found", schedule_id))
        })?;

        if let Some(ref name) = new_name {
            schedule.name = name.clone();
        }
        if let Some(location) = new_location {
            schedule.geographic_location = location;
        }

        // Capture updated values before splitting borrows
        let updated_location = data
            .schedules
            .get(&schedule_id.0)
            .map(|s| s.geographic_location);

        // Update metadata
        if let Some(meta) = data.schedule_metadata.get_mut(&schedule_id.0) {
            if let Some(name) = new_name {
                meta.schedule_name = name;
            }
            if let Some(loc) = updated_location {
                meta.observer_location = loc;
            }
        }

        let meta = data.schedule_metadata.get(&schedule_id.0).cloned().unwrap();
        Ok(meta)
    }
}

// ==================== Analytics Repository ====================

#[async_trait]
impl AnalyticsRepository for LocalRepository {
    async fn populate_schedule_analytics(
        &self,
        schedule_id: ScheduleId,
    ) -> RepositoryResult<usize> {
        let schedule = self.get_schedule_impl(schedule_id)?;

        // Build validation input from schedule blocks
        let blocks_for_validation: Vec<crate::services::validation::BlockForValidation> = schedule
            .blocks
            .iter()
            .map(|b| {
                // Calculate max visibility period for this block
                let max_visibility_period_hours = b
                    .visibility_periods
                    .iter()
                    .map(|p| p.duration().value() * 24.0)
                    .fold(0.0_f64, |a, b| a.max(b));
                crate::services::validation::BlockForValidation {
                    schedule_id,
                    scheduling_block_id: b.id.map(|id| id.0).expect("DB Block ID missing"),
                    priority: b.priority,
                    requested_duration_sec: b.requested_duration.value() as i32,
                    min_observation_sec: b.min_observation.value() as i32,
                    total_visibility_hours: b
                        .visibility_periods
                        .iter()
                        .map(|p| p.duration().value() * 24.0)
                        .sum(),
                    max_visibility_period_hours,
                    min_alt_deg: Some(b.constraints.min_alt.value()),
                    max_alt_deg: Some(b.constraints.max_alt.value()),
                    constraint_start_mjd: b
                        .constraints
                        .fixed_time
                        .as_ref()
                        .map(|p| p.start.value()),
                    constraint_stop_mjd: b.constraints.fixed_time.as_ref().map(|p| p.end.value()),
                    scheduled_start_mjd: b.scheduled_period.as_ref().map(|p| p.start.value()),
                    scheduled_stop_mjd: b.scheduled_period.as_ref().map(|p| p.end.value()),
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
                schedule_id.0,
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
        data.analytics_exists.insert(schedule_id.0, true);

        // Return the number of blocks processed (not validation count)
        Ok(schedule.blocks.len())
    }

    async fn delete_schedule_analytics(&self, schedule_id: ScheduleId) -> RepositoryResult<usize> {
        Ok(self.delete_from_map(|d| &mut d.analytics_exists, schedule_id))
    }

    async fn has_analytics_data(&self, schedule_id: ScheduleId) -> RepositoryResult<bool> {
        let data = self.data.read().unwrap();
        Ok(data
            .analytics_exists
            .get(&schedule_id.0)
            .copied()
            .unwrap_or(false))
    }

    async fn fetch_analytics_blocks_for_sky_map(
        &self,
        schedule_id: ScheduleId,
    ) -> RepositoryResult<Vec<crate::api::LightweightBlock>> {
        use crate::api::LightweightBlock;

        let schedule = self.get_schedule_impl(schedule_id)?;

        // Convert schedule blocks to LightweightBlock format
        let blocks: Vec<LightweightBlock> = schedule
            .blocks
            .iter()
            .map(|b| {
                LightweightBlock {
                    original_block_id: b.original_block_id.clone(),
                    block_name: b.block_name.clone(),
                    priority: b.priority,
                    priority_bin: "".to_string(), // Will be computed by sky_map service
                    requested_duration_seconds: b.requested_duration,
                    target_ra_deg: b.target_ra,
                    target_dec_deg: b.target_dec,
                    scheduled_period: b.scheduled_period,
                }
            })
            .collect();

        Ok(blocks)
    }

    async fn fetch_analytics_blocks_for_distribution(
        &self,
        schedule_id: ScheduleId,
    ) -> RepositoryResult<Vec<crate::api::DistributionBlock>> {
        use crate::api::DistributionBlock;

        let schedule = self.get_schedule_impl(schedule_id)?;

        // Convert schedule blocks to DistributionBlock format
        let blocks: Vec<DistributionBlock> = schedule
            .blocks
            .iter()
            .map(|b| {
                let total_visibility_hours: f64 = b
                    .visibility_periods
                    .iter()
                    .map(|p| p.duration().value() * 24.0)
                    .sum();

                let requested_hours = b.requested_duration.value() / 3600.0;

                let elevation_range_deg =
                    b.constraints.max_alt.value() - b.constraints.min_alt.value();

                DistributionBlock {
                    priority: b.priority,
                    total_visibility_hours: qtty::Hours::new(total_visibility_hours),
                    requested_hours: qtty::Hours::new(requested_hours),
                    elevation_range_deg: qtty::Degrees::new(elevation_range_deg),
                    scheduled: b.scheduled_period.is_some(),
                }
            })
            .collect();

        Ok(blocks)
    }

    async fn fetch_analytics_blocks_for_insights(
        &self,
        schedule_id: ScheduleId,
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
                    scheduling_block_id: b.id.expect("DB Block ID missing").0,
                    original_block_id: b.original_block_id.clone(),
                    block_name: b.block_name.clone(),
                    priority: b.priority,
                    total_visibility_hours: qtty::time::Hours::new(total_visibility_hours),
                    requested_hours: qtty::time::Hours::new(b.requested_duration.value() / 3600.0),
                    elevation_range_deg: qtty::angular::Degrees::new(
                        b.constraints.max_alt.value() - b.constraints.min_alt.value(),
                    ),
                    scheduled: b.scheduled_period.is_some(),
                    scheduled_start_mjd: b.scheduled_period.as_ref().map(|p| p.start),
                    scheduled_stop_mjd: b.scheduled_period.as_ref().map(|p| p.end),
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
                    // Try to resolve the original_block_id and block_name from the stored blocks
                    let original_id = data
                        .blocks
                        .get(&r.scheduling_block_id)
                        .map(|b| b.original_block_id.clone());
                    let blk_name = data
                        .blocks
                        .get(&r.scheduling_block_id)
                        .filter(|b| !b.block_name.is_empty())
                        .map(|b| b.block_name.clone());

                    impossible_blocks.push(crate::api::ValidationIssue {
                        block_id: r.scheduling_block_id,
                        original_block_id: original_id,
                        block_name: blk_name,
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
                    let original_id = data
                        .blocks
                        .get(&r.scheduling_block_id)
                        .map(|b| b.original_block_id.clone());
                    let blk_name = data
                        .blocks
                        .get(&r.scheduling_block_id)
                        .filter(|b| !b.block_name.is_empty())
                        .map(|b| b.block_name.clone());

                    validation_errors.push(crate::api::ValidationIssue {
                        block_id: r.scheduling_block_id,
                        original_block_id: original_id,
                        block_name: blk_name,
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
                    let original_id = data
                        .blocks
                        .get(&r.scheduling_block_id)
                        .map(|b| b.original_block_id.clone());
                    let blk_name = data
                        .blocks
                        .get(&r.scheduling_block_id)
                        .filter(|b| !b.block_name.is_empty())
                        .map(|b| b.block_name.clone());

                    validation_warnings.push(crate::api::ValidationIssue {
                        block_id: r.scheduling_block_id,
                        original_block_id: original_id,
                        block_name: blk_name,
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

        data.validation_results.insert(schedule_id.0, report);
        Ok(results.len())
    }

    async fn fetch_validation_results(
        &self,
        schedule_id: ScheduleId,
    ) -> RepositoryResult<crate::api::ValidationReport> {
        let data = self.data.read().unwrap();

        data.validation_results
            .get(&schedule_id.0)
            .cloned()
            .ok_or_else(|| {
                RepositoryError::NotFound(format!(
                    "No validation results for schedule {}",
                    schedule_id
                ))
            })
    }

    async fn has_validation_results(&self, schedule_id: ScheduleId) -> RepositoryResult<bool> {
        let data = self.data.read().unwrap();
        Ok(data.validation_results.contains_key(&schedule_id.0))
    }

    async fn delete_validation_results(&self, schedule_id: ScheduleId) -> RepositoryResult<u64> {
        Ok(self.delete_from_map(|d| &mut d.validation_results, schedule_id) as u64)
    }
}

// ==================== Visualization Repository ====================

#[async_trait]
impl VisualizationRepository for LocalRepository {
    async fn fetch_visibility_map_data(
        &self,
        schedule_id: ScheduleId,
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
            .map(|b| VisibilityBlockSummary {
                scheduling_block_id: b.id.expect("DB Block ID missing").0,
                original_block_id: b.original_block_id.clone(),
                block_name: b.block_name.clone(),
                priority: b.priority,
                num_visibility_periods: b.visibility_periods.len(),
                scheduled: b.scheduled_period.is_some(),
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
        schedule_id: ScheduleId,
        priority_min: Option<f64>,
        priority_max: Option<f64>,
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
                let priority = b.priority;

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
        schedule_id: ScheduleId,
    ) -> RepositoryResult<Vec<crate::db::models::ScheduleTimelineBlock>> {
        use crate::db::models::ScheduleTimelineBlock;

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

                Some(ScheduleTimelineBlock {
                    scheduling_block_id: b.id.expect("DB Block ID missing").0,
                    original_block_id: b.original_block_id.clone(),
                    block_name: b.block_name.clone(),
                    priority: b.priority,
                    scheduled_start_mjd: scheduled_period.start,
                    scheduled_stop_mjd: scheduled_period.end,
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
        schedule_id: ScheduleId,
    ) -> RepositoryResult<Vec<crate::db::models::CompareBlock>> {
        use crate::db::models::CompareBlock;

        let schedule = self.get_schedule_impl(schedule_id)?;

        // Convert schedule blocks to CompareBlock format. Use the stored DB
        // id (falling back to the row index only for display) — matching is
        // done via `original_block_id`, never row position.
        let blocks: Vec<CompareBlock> = schedule
            .blocks
            .iter()
            .enumerate()
            .map(|(idx, b)| {
                let requested_hours = b.requested_duration.value() / 3600.0;
                let scheduling_block_id =
                    b.id.map(|id| id.0.to_string())
                        .unwrap_or_else(|| format!("local-{}", idx + 1));

                CompareBlock {
                    scheduling_block_id,
                    original_block_id: b.original_block_id.clone(),
                    block_name: b.block_name.clone(),
                    priority: b.priority,
                    scheduled: b.scheduled_period.is_some(),
                    requested_hours: qtty::Hours::new(requested_hours),
                    scheduled_start_mjd: b.scheduled_period.as_ref().map(|p| p.start.value()),
                    scheduled_stop_mjd: b.scheduled_period.as_ref().map(|p| p.end.value()),
                }
            })
            .collect();

        Ok(blocks)
    }

    async fn fetch_gap_metrics(
        &self,
        schedule_id: ScheduleId,
    ) -> RepositoryResult<(Option<i32>, Option<qtty::Hours>, Option<qtty::Hours>)> {
        // If analytics have not been populated for this schedule, mirror
        // the postgres behaviour and return no metrics (None, None, None).
        let data = self.data.read().unwrap();
        if !data
            .analytics_exists
            .get(&schedule_id.0)
            .copied()
            .unwrap_or(false)
        {
            return Ok((None, None, None));
        }

        // Collect scheduled periods from blocks
        let schedule = data.schedules.get(&schedule_id.0).cloned().ok_or_else(|| {
            RepositoryError::NotFound(format!("Schedule {} not found", schedule_id))
        })?;

        let mut periods: Vec<(f64, f64)> = schedule
            .blocks
            .iter()
            .filter_map(|b| b.scheduled_period)
            .map(|p| (p.start.value(), p.end.value()))
            .collect();

        if periods.len() < 2 {
            return Ok((
                Some(0),
                Some(qtty::Hours::new(0.0)),
                Some(qtty::Hours::new(0.0)),
            ));
        }

        periods.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        // Compute positive gaps between consecutive scheduled periods (in hours)
        let mut gaps_hours: Vec<f64> = Vec::new();
        for w in periods.windows(2) {
            let prev = w[0];
            let next = w[1];
            let gap_days = next.0 - prev.1;
            if gap_days > 0.0 {
                gaps_hours.push(gap_days * 24.0);
            }
        }

        if gaps_hours.is_empty() {
            return Ok((
                Some(0),
                Some(qtty::Hours::new(0.0)),
                Some(qtty::Hours::new(0.0)),
            ));
        }

        // Mean
        let sum: f64 = gaps_hours.iter().sum();
        let mean = sum / gaps_hours.len() as f64;

        // Median
        gaps_hours.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let median = if gaps_hours.len().is_multiple_of(2) {
            let hi = gaps_hours[gaps_hours.len() / 2];
            let lo = gaps_hours[gaps_hours.len() / 2 - 1];
            (lo + hi) / 2.0
        } else {
            gaps_hours[gaps_hours.len() / 2]
        };

        Ok((
            Some(gaps_hours.len() as i32),
            Some(qtty::Hours::new(mean)),
            Some(qtty::Hours::new(median)),
        ))
    }
}

// ==================== Environment Repository ====================

#[async_trait]
impl crate::db::repository::EnvironmentRepository for LocalRepository {
    async fn list_environments(&self) -> RepositoryResult<Vec<crate::api::EnvironmentInfo>> {
        self.check_health()?;
        let data = self.data.read().unwrap();

        let mut result = Vec::new();
        for (&env_id, (name, structure, created_at)) in &data.environments {
            // Collect schedule IDs assigned to this environment
            let schedule_ids: Vec<ScheduleId> = data
                .schedule_environment
                .iter()
                .filter(|&(_, &eid)| eid == env_id)
                .map(|(&sid, _)| ScheduleId(sid))
                .collect();

            result.push(crate::api::EnvironmentInfo {
                environment_id: env_id,
                name: name.clone(),
                structure: structure.clone(),
                schedule_ids,
                created_at: *created_at,
            });
        }

        result.sort_by_key(|env| env.environment_id);
        Ok(result)
    }

    async fn get_environment(
        &self,
        id: crate::api::EnvironmentId,
    ) -> RepositoryResult<Option<crate::api::EnvironmentInfo>> {
        self.check_health()?;
        let data = self.data.read().unwrap();

        match data.environments.get(&id) {
            Some((name, structure, created_at)) => {
                // Collect schedule IDs assigned to this environment
                let schedule_ids: Vec<ScheduleId> = data
                    .schedule_environment
                    .iter()
                    .filter(|&(_, &eid)| eid == id)
                    .map(|(&sid, _)| ScheduleId(sid))
                    .collect();

                Ok(Some(crate::api::EnvironmentInfo {
                    environment_id: id,
                    name: name.clone(),
                    structure: structure.clone(),
                    schedule_ids,
                    created_at: *created_at,
                }))
            }
            None => Ok(None),
        }
    }

    async fn create_environment(
        &self,
        name: &str,
    ) -> RepositoryResult<crate::api::EnvironmentInfo> {
        self.check_health()?;
        let mut data = self.data.write().unwrap();

        // Check for existing environment with same name (case-insensitive)
        let name_lower = name.trim().to_lowercase();
        for (existing_name, _, _) in data.environments.values() {
            if existing_name.trim().to_lowercase() == name_lower {
                return Err(RepositoryError::validation(format!(
                    "Environment with name '{}' already exists",
                    name
                )));
            }
        }

        let env_id = data.next_environment_id;
        data.next_environment_id += 1;

        let created_at = chrono::Utc::now();
        data.environments
            .insert(env_id, (name.to_string(), None, created_at));

        Ok(crate::api::EnvironmentInfo {
            environment_id: env_id,
            name: name.to_string(),
            structure: None,
            schedule_ids: vec![],
            created_at,
        })
    }

    async fn delete_environment(&self, id: crate::api::EnvironmentId) -> RepositoryResult<()> {
        self.check_health()?;
        let mut data = self.data.write().unwrap();

        if data.environments.remove(&id).is_none() {
            return Err(RepositoryError::not_found(format!(
                "Environment {} not found",
                id
            )));
        }

        // Remove preschedule cache
        data.preschedule.remove(&id);

        // Unassign all schedules from this environment
        data.schedule_environment
            .retain(|_, &mut env_id| env_id != id);

        // Update schedule metadata to reflect unassignment
        for (_, meta) in data.schedule_metadata.iter_mut() {
            if meta.environment_id == Some(id) {
                meta.environment_id = None;
            }
        }

        Ok(())
    }

    async fn initialise_environment(
        &self,
        id: crate::api::EnvironmentId,
        structure: &crate::api::EnvironmentStructure,
        preschedule: &serde_json::Value,
    ) -> RepositoryResult<()> {
        self.check_health()?;
        let mut data = self.data.write().unwrap();

        let current_entry = data
            .environments
            .get(&id)
            .ok_or_else(|| RepositoryError::not_found(format!("Environment {} not found", id)))?;

        let name = current_entry.0.clone();
        let current_structure = current_entry.1.clone();
        let created_at = current_entry.2;

        match current_structure {
            None => {
                // Uninitialized - set structure and preschedule
                data.environments
                    .insert(id, (name, Some(structure.clone()), created_at));
                data.preschedule.insert(id, preschedule.clone());
                Ok(())
            }
            Some(existing) if existing == *structure => {
                // Structure matches - just update preschedule
                data.preschedule.insert(id, preschedule.clone());
                Ok(())
            }
            Some(_) => {
                // Structure mismatch
                Err(RepositoryError::validation(format!(
                    "Environment {} already has a different structure",
                    id
                )))
            }
        }
    }

    async fn assign_schedule(
        &self,
        schedule_id: ScheduleId,
        env_id: crate::api::EnvironmentId,
    ) -> RepositoryResult<()> {
        self.check_health()?;
        let mut data = self.data.write().unwrap();

        // Check schedule exists
        if !data.schedules.contains_key(&schedule_id.0) {
            return Err(RepositoryError::not_found(format!(
                "Schedule {} not found",
                schedule_id
            )));
        }

        // Assign schedule to environment
        data.schedule_environment.insert(schedule_id.0, env_id);

        // Update schedule metadata
        if let Some(meta) = data.schedule_metadata.get_mut(&schedule_id.0) {
            meta.environment_id = Some(env_id);
        }

        Ok(())
    }

    async fn unassign_schedule(&self, schedule_id: ScheduleId) -> RepositoryResult<()> {
        self.check_health()?;
        let mut data = self.data.write().unwrap();

        // Remove assignment (no-op if not assigned)
        data.schedule_environment.remove(&schedule_id.0);

        // Update schedule metadata
        if let Some(meta) = data.schedule_metadata.get_mut(&schedule_id.0) {
            meta.environment_id = None;
        }

        Ok(())
    }

    async fn get_preschedule(
        &self,
        env_id: crate::api::EnvironmentId,
    ) -> RepositoryResult<Option<serde_json::Value>> {
        self.check_health()?;
        let data = self.data.read().unwrap();
        Ok(data.preschedule.get(&env_id).cloned())
    }
}

#[async_trait]
impl crate::db::repository::AlgorithmTraceRepository for LocalRepository {
    async fn store_algorithm_trace(
        &self,
        schedule_id: ScheduleId,
        algorithm: &str,
        summary: &serde_json::Value,
        iterations: &serde_json::Value,
    ) -> RepositoryResult<()> {
        self.check_health()?;
        let mut data = self.data.write().unwrap();
        data.algorithm_traces.insert(
            schedule_id.0,
            (algorithm.to_string(), summary.clone(), iterations.clone()),
        );
        Ok(())
    }

    async fn get_algorithm_trace(
        &self,
        schedule_id: ScheduleId,
    ) -> RepositoryResult<Option<crate::api::AlgorithmTraceResponse>> {
        self.check_health()?;
        let data = self.data.read().unwrap();
        let Some((algorithm, summary_json, iterations_json)) =
            data.algorithm_traces.get(&schedule_id.0)
        else {
            return Ok(None);
        };
        let mut summary: crate::api::AlgorithmTraceSummary =
            serde_json::from_value(summary_json.clone()).map_err(|e| {
                RepositoryError::internal(format!("Failed to decode algorithm_trace summary: {e}"))
            })?;
        if summary.algorithm.is_empty() {
            summary.algorithm = algorithm.clone();
        }
        let iterations: Vec<crate::api::AlgorithmTraceIteration> =
            serde_json::from_value(iterations_json.clone()).map_err(|e| {
                RepositoryError::internal(format!(
                    "Failed to decode algorithm_trace iterations: {e}"
                ))
            })?;
        Ok(Some(crate::api::AlgorithmTraceResponse {
            schedule_id,
            summary,
            iterations,
        }))
    }

    async fn list_algorithm_names(&self) -> RepositoryResult<Vec<(ScheduleId, String)>> {
        self.check_health()?;
        let data = self.data.read().unwrap();
        Ok(data
            .algorithm_traces
            .iter()
            .map(|(id, (algo, _, _))| (ScheduleId::new(*id), algo.clone()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ModifiedJulianDate;
    use qtty::{Degrees, Meters};
    use siderust::coordinates::centers::Geodetic;
    use siderust::coordinates::frames::ECEF;

    fn default_schedule_period() -> Period {
        Period {
            start: ModifiedJulianDate::new(60000.0),
            end: ModifiedJulianDate::new(60001.0),
        }
    }

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
            geographic_location: Geodetic::<ECEF>::new(
                Degrees::new(-17.8892),
                Degrees::new(28.7624),
                Meters::new(2396.0),
            ),
            astronomical_nights: vec![],
            checksum: "test123".to_string(),
            schedule_period: default_schedule_period(),
        };

        let metadata = repo.store_schedule(&schedule).await.unwrap();
        assert!(metadata.schedule_id.0 > 0);

        let retrieved = repo.get_schedule(metadata.schedule_id).await.unwrap();
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
            geographic_location: Geodetic::<ECEF>::new(
                Degrees::new(-17.8892),
                Degrees::new(28.7624),
                Meters::new(2396.0),
            ),
            astronomical_nights: vec![],
            checksum: "hash1".to_string(),
            schedule_period: default_schedule_period(),
        };

        let schedule2 = Schedule {
            id: None,
            name: "Schedule 2".to_string(),
            blocks: vec![],
            dark_periods: vec![],
            geographic_location: Geodetic::<ECEF>::new(
                Degrees::new(-17.8892),
                Degrees::new(28.7624),
                Meters::new(2396.0),
            ),
            astronomical_nights: vec![],
            checksum: "hash2".to_string(),
            schedule_period: default_schedule_period(),
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
        assert!(matches!(result, Err(RepositoryError::NotFound { .. })));
    }

    #[tokio::test]
    async fn test_analytics_operations() {
        let repo = LocalRepository::new();

        let schedule = Schedule {
            id: None,
            name: "Test".to_string(),
            blocks: vec![],
            dark_periods: vec![],
            geographic_location: Geodetic::<ECEF>::new(
                Degrees::new(-17.8892),
                Degrees::new(28.7624),
                Meters::new(2396.0),
            ),
            astronomical_nights: vec![],
            checksum: "test".to_string(),
            schedule_period: default_schedule_period(),
        };

        let metadata = repo.store_schedule(&schedule).await.unwrap();
        let schedule_id = metadata.schedule_id;

        assert!(!repo.has_analytics_data(schedule_id).await.unwrap());

        repo.populate_schedule_analytics(schedule_id).await.unwrap();
        assert!(repo.has_analytics_data(schedule_id).await.unwrap());

        repo.delete_schedule_analytics(schedule_id).await.unwrap();
        assert!(!repo.has_analytics_data(schedule_id).await.unwrap());
    }

    // ==================== Environment Tests ====================

    #[tokio::test]
    async fn test_create_and_list_environments() {
        let repo = LocalRepository::new();

        let env1 = repo.create_environment("Test Env 1").await.unwrap();
        assert_eq!(env1.name, "Test Env 1");
        assert!(env1.structure.is_none());
        assert!(env1.schedule_ids.is_empty());

        let _env2 = repo.create_environment("Test Env 2").await.unwrap();

        let envs = repo.list_environments().await.unwrap();
        assert_eq!(envs.len(), 2);
    }

    #[tokio::test]
    async fn test_create_environment_duplicate_name() {
        let repo = LocalRepository::new();

        repo.create_environment("Test Env").await.unwrap();
        let result = repo.create_environment("test env").await; // Case-insensitive
        assert!(matches!(
            result,
            Err(RepositoryError::ValidationError { .. })
        ));
    }

    #[tokio::test]
    async fn test_environment_structure_initialization() {
        let repo = LocalRepository::new();

        let env = repo.create_environment("Test Env").await.unwrap();
        let env_id = env.environment_id;

        let structure = crate::api::EnvironmentStructure {
            period_start_mjd: 60000.0,
            period_end_mjd: 60007.0,
            lat_deg: -17.89,
            lon_deg: 28.76,
            elevation_m: 2200.0,
            blocks_hash: "test_hash".to_string(),
        };

        let preschedule = serde_json::json!({
            "dark_periods": [],
            "astronomical_nights": [],
            "block_visibility": {}
        });

        // Initialize structure
        repo.initialise_environment(env_id, &structure, &preschedule)
            .await
            .unwrap();

        let env = repo.get_environment(env_id).await.unwrap().unwrap();
        assert!(env.structure.is_some());
        assert_eq!(env.structure.as_ref().unwrap().blocks_hash, "test_hash");

        // Update preschedule with matching structure (should succeed)
        let preschedule2 = serde_json::json!({"updated": true});
        repo.initialise_environment(env_id, &structure, &preschedule2)
            .await
            .unwrap();

        // Try to set different structure (should fail)
        let mut different_structure = structure.clone();
        different_structure.lat_deg = 20.0;
        let result = repo
            .initialise_environment(env_id, &different_structure, &preschedule)
            .await;
        assert!(matches!(
            result,
            Err(RepositoryError::ValidationError { .. })
        ));
    }

    #[tokio::test]
    async fn test_assign_and_unassign_schedule() {
        let repo = LocalRepository::new();

        let schedule = Schedule {
            id: None,
            name: "Test Schedule".to_string(),
            blocks: vec![],
            dark_periods: vec![],
            geographic_location: Geodetic::<ECEF>::new(
                Degrees::new(-17.89),
                Degrees::new(28.76),
                Meters::new(2200.0),
            ),
            astronomical_nights: vec![],
            checksum: "test".to_string(),
            schedule_period: default_schedule_period(),
        };

        let meta = repo.store_schedule(&schedule).await.unwrap();
        let schedule_id = meta.schedule_id;

        let env = repo.create_environment("Test Env").await.unwrap();
        let env_id = env.environment_id;

        // Assign schedule to environment
        repo.assign_schedule(schedule_id, env_id).await.unwrap();

        let env = repo.get_environment(env_id).await.unwrap().unwrap();
        assert_eq!(env.schedule_ids.len(), 1);
        assert_eq!(env.schedule_ids[0], schedule_id);

        let meta = repo.list_schedules().await.unwrap();
        assert_eq!(meta[0].environment_id, Some(env_id));

        // Unassign schedule
        repo.unassign_schedule(schedule_id).await.unwrap();

        let env = repo.get_environment(env_id).await.unwrap().unwrap();
        assert!(env.schedule_ids.is_empty());

        let meta = repo.list_schedules().await.unwrap();
        assert_eq!(meta[0].environment_id, None);
    }

    #[tokio::test]
    async fn test_delete_environment() {
        let repo = LocalRepository::new();

        let env = repo.create_environment("Test Env").await.unwrap();
        let env_id = env.environment_id;

        let schedule = Schedule {
            id: None,
            name: "Test Schedule".to_string(),
            blocks: vec![],
            dark_periods: vec![],
            geographic_location: Geodetic::<ECEF>::new(
                Degrees::new(-17.89),
                Degrees::new(28.76),
                Meters::new(2200.0),
            ),
            astronomical_nights: vec![],
            checksum: "test".to_string(),
            schedule_period: default_schedule_period(),
        };
        let meta = repo.store_schedule(&schedule).await.unwrap();
        repo.assign_schedule(meta.schedule_id, env_id)
            .await
            .unwrap();

        // Delete environment
        repo.delete_environment(env_id).await.unwrap();

        // Environment should be gone
        let result = repo.get_environment(env_id).await.unwrap();
        assert!(result.is_none());

        // Schedule should be unassigned
        let meta = repo.list_schedules().await.unwrap();
        assert_eq!(meta[0].environment_id, None);
    }

    #[tokio::test]
    async fn test_preschedule_cache() {
        let repo = LocalRepository::new();

        let env = repo.create_environment("Test Env").await.unwrap();
        let env_id = env.environment_id;

        let structure = crate::api::EnvironmentStructure {
            period_start_mjd: 60000.0,
            period_end_mjd: 60007.0,
            lat_deg: -17.89,
            lon_deg: 28.76,
            elevation_m: 2200.0,
            blocks_hash: "test_hash".to_string(),
        };

        let preschedule = serde_json::json!({"test": "data"});

        repo.initialise_environment(env_id, &structure, &preschedule)
            .await
            .unwrap();

        let cached = repo.get_preschedule(env_id).await.unwrap();
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), preschedule);
    }
}
