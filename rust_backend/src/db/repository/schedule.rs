//! Core schedule repository trait for CRUD operations.
//!
//! This trait defines the fundamental database operations for schedules,
//! scheduling blocks, dark periods, and possible periods.

use async_trait::async_trait;

use super::error::RepositoryResult;
use crate::db::models::{Period, Schedule, ScheduleMetadata, SchedulingBlock};
use crate::api::*;
/// Repository trait for core schedule database operations.
///
/// This trait handles the basic CRUD (Create, Read, Update, Delete) operations
/// for schedules and their associated data. It does not include analytics or
/// specialized query operations, which are in separate traits.
///
/// # Thread Safety
/// Implementations must be `Send + Sync` to work with async Rust.
#[async_trait]
pub trait ScheduleRepository: Send + Sync {
    // ==================== Health & Connection ====================

    /// Check if the database connection is healthy.
    ///
    /// # Returns
    /// - `Ok(true)` if connection is healthy
    /// - `Ok(false)` if connection is unhealthy but no error occurred
    /// - `Err(RepositoryError)` if an error occurred during the check
    async fn health_check(&self) -> RepositoryResult<bool>;

    // ==================== Schedule Operations ====================

    /// Store a new schedule in the database.
    ///
    /// # Arguments
    /// * `schedule` - The schedule to store, including all blocks and metadata
    ///
    /// # Returns
    /// * `Ok(ScheduleMetadata)` - Metadata of the stored schedule including assigned ID
    /// * `Err(RepositoryError)` - If the operation fails
    async fn store_schedule(&self, schedule: &Schedule) -> RepositoryResult<ScheduleMetadata>;

    /// Retrieve a complete schedule by ID.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule to retrieve
    ///
    /// # Returns
    /// * `Ok(Schedule)` - The complete schedule with all blocks and dark periods
    /// * `Err(RepositoryError::NotFound)` - If the schedule doesn't exist
    /// * `Err(RepositoryError)` - If the operation fails
    async fn get_schedule(&self, schedule_id: i64) -> RepositoryResult<Schedule>;

    /// List all schedules with basic metadata.
    ///
    /// # Returns
    /// * `Ok(Vec<ScheduleInfo>)` - List of schedule metadata (id, name, timestamp, etc.)
    /// * `Err(RepositoryError)` - If the operation fails
    async fn list_schedules(&self) -> RepositoryResult<Vec<ScheduleInfo>>;

    /// Get the time range covered by a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Some(Period))` - Time range as a Period
    /// * `Ok(None)` - If the schedule has no time constraints
    /// * `Err(RepositoryError)` - If the operation fails
    async fn get_schedule_time_range(&self, schedule_id: i64) -> RepositoryResult<Option<Period>>;

    // ==================== Scheduling Block Operations ====================

    /// Get a single scheduling block by ID.
    ///
    /// # Arguments
    /// * `scheduling_block_id` - The ID of the block to retrieve
    ///
    /// # Returns
    /// * `Ok(SchedulingBlock)` - The scheduling block with all details
    /// * `Err(RepositoryError::NotFound)` - If the block doesn't exist
    /// * `Err(RepositoryError)` - If the operation fails
    async fn get_scheduling_block(
        &self,
        scheduling_block_id: i64,
    ) -> RepositoryResult<SchedulingBlock>;

    /// Get all scheduling blocks for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<SchedulingBlock>)` - List of all blocks in the schedule
    /// * `Err(RepositoryError)` - If the operation fails
    async fn get_blocks_for_schedule(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<SchedulingBlock>>;

    // ==================== Dark Periods & Possible Periods ====================

    /// Fetch dark periods (observing windows) for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<Period>)` - List of dark periods
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_dark_periods(&self, schedule_id: i64) -> RepositoryResult<Vec<Period>>;

    /// Fetch possible observation periods for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<Period>)` - List of visibility periods
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_possible_periods(&self, schedule_id: i64) -> RepositoryResult<Vec<Period>>;
}
