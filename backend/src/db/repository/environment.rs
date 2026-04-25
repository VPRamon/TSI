//! Environment repository trait and operations.
//!
//! Environments group schedules that share the same structure
//! (location, period, and set of blocks) to enable preschedule caching.

use async_trait::async_trait;

use super::error::RepositoryResult;
use crate::api::{EnvironmentId, EnvironmentInfo, EnvironmentStructure, ScheduleId};

/// Repository operations for environments.
///
/// Environments allow multiple schedules with identical structure to share
/// cached astronomical night and visibility computations.
#[async_trait]
pub trait EnvironmentRepository: Send + Sync {
    /// List all environments with their assigned schedules.
    async fn list_environments(&self) -> RepositoryResult<Vec<EnvironmentInfo>>;

    /// Get a single environment by ID.
    ///
    /// Returns `Ok(None)` if the environment doesn't exist.
    async fn get_environment(&self, id: EnvironmentId)
        -> RepositoryResult<Option<EnvironmentInfo>>;

    /// Create a new environment with the given name.
    ///
    /// Returns an error if an environment with this name already exists (case-insensitive).
    async fn create_environment(&self, name: &str) -> RepositoryResult<EnvironmentInfo>;

    /// Delete an environment and unassign all its schedules.
    ///
    /// Returns an error if the environment doesn't exist.
    async fn delete_environment(&self, id: EnvironmentId) -> RepositoryResult<()>;

    /// Initialize an environment's structure and preschedule cache.
    ///
    /// If the environment is uninitialized (structure is `None`), sets the structure
    /// and stores the preschedule payload.
    ///
    /// If the environment is already initialized:
    /// - If the new structure matches the existing one, updates only the preschedule payload.
    /// - If the structure differs, returns a validation error.
    async fn initialise_environment(
        &self,
        id: EnvironmentId,
        structure: &EnvironmentStructure,
        preschedule: &serde_json::Value,
    ) -> RepositoryResult<()>;

    /// Assign a schedule to an environment.
    ///
    /// Returns an error if the schedule doesn't exist.
    /// Overwrites any previous environment assignment for this schedule.
    async fn assign_schedule(
        &self,
        schedule_id: ScheduleId,
        env_id: EnvironmentId,
    ) -> RepositoryResult<()>;

    /// Remove a schedule's environment assignment.
    ///
    /// This is a no-op if the schedule wasn't assigned to any environment.
    async fn unassign_schedule(&self, schedule_id: ScheduleId) -> RepositoryResult<()>;

    /// Get the cached preschedule payload for an environment.
    ///
    /// Returns `Ok(None)` if the environment exists but has no preschedule,
    /// or if the environment doesn't exist.
    async fn get_preschedule(
        &self,
        env_id: EnvironmentId,
    ) -> RepositoryResult<Option<serde_json::Value>>;
}
