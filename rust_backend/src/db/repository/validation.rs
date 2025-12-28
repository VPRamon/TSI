//! Validation repository trait for managing validation results.
//!
//! This trait defines operations for storing, retrieving, and managing
//! schedule validation results and reports.

use async_trait::async_trait;

use super::error::RepositoryResult;
use crate::services::validation::ValidationResult;

/// Repository trait for validation operations.
///
/// This trait handles storage and retrieval of validation results,
/// which track issues found during schedule validation.
///
/// # Thread Safety
/// Implementations must be `Send + Sync` to work with async Rust.
#[async_trait]
pub trait ValidationRepository: Send + Sync {
    /// Insert validation results for a schedule.
    ///
    /// # Arguments
    /// * `results` - Validation results to store
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of validation records inserted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn insert_validation_results(
        &self,
        results: &[ValidationResult],
    ) -> RepositoryResult<usize>;

    /// Fetch validation results for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(ValidationReport)` - Validation report with all issues
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_validation_results(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<crate::api::ValidationReport>;

    /// Check if validation results exist for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(bool)` - True if validation results exist
    /// * `Err(RepositoryError)` - If the operation fails
    async fn has_validation_results(&self, schedule_id: crate::api::ScheduleId) -> RepositoryResult<bool>;

    /// Delete validation results for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(u64)` - Number of rows deleted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn delete_validation_results(&self, schedule_id: crate::api::ScheduleId) -> RepositoryResult<u64>;
}
