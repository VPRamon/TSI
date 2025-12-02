//! High-level database service layer.
//!
//! This module provides repository-agnostic database operations that work with
//! any implementation of the `ScheduleRepository` trait. These functions contain
//! business logic such as checksum validation, analytics population, and data
//! integrity checks that should be consistent regardless of the storage backend.
//!
//! # Architecture
//!
//! The database module follows a layered architecture:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │  Application Layer (Python bindings, REST API, etc.)    │
//! └───────────────────┬─────────────────────────────────────┘
//!                     │
//! ┌───────────────────▼─────────────────────────────────────┐
//! │  Service Layer (services.rs) - Business Logic           │
//! │  - Checksum validation                                   │
//! │  - Analytics population orchestration                    │
//! │  - Cross-cutting concerns                                │
//! └───────────────────┬─────────────────────────────────────┘
//!                     │
//! ┌───────────────────▼─────────────────────────────────────┐
//! │  Repository Trait (repository.rs) - Abstract Interface  │
//! └───────────────────┬─────────────────────────────────────┘
//!                     │
//!     ┌───────────────┴────────────────┐
//!     │                                 │
//! ┌───▼──────────────┐     ┌──────────▼──────────────┐
//! │ Azure Repository │     │  Test Repository        │
//! │ (SQL queries)    │     │  (in-memory)            │
//! └──────────────────┘     └─────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```no_run
//! use tsi_rust::db::{services, factory::RepositoryFactory, factory::RepositoryType, DbConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create repository
//!     let config = DbConfig::from_env()?;
//!     let repo = RepositoryFactory::create(RepositoryType::Azure, Some(&config)).await?;
//!     
//!     // Use service layer functions
//!     let schedules = services::list_schedules(repo.as_ref()).await?;
//!     println!("Found {} schedules", schedules.len());
//!     
//!     Ok(())
//! }
//! ```

use log::{info, warn};

use super::models::{Schedule, ScheduleInfo, ScheduleMetadata, SchedulingBlock};
use super::repository::{RepositoryResult, ScheduleRepository};

// ==================== Health & Connection ====================

/// Check if the database connection is healthy.
///
/// This is a simple pass-through to the repository's health check.
///
/// # Arguments
/// * `repo` - Repository implementation
///
/// # Returns
/// * `Ok(true)` if connection is healthy
/// * `Err` if check fails
pub async fn health_check(repo: &dyn ScheduleRepository) -> RepositoryResult<bool> {
    repo.health_check().await
}

// ==================== Schedule Operations ====================

/// Store a new schedule in the database with full business logic.
///
/// This function orchestrates the complete schedule storage process:
/// 1. Check if schedule with same checksum already exists (deduplication)
/// 2. If exists, ensure analytics are populated and return existing metadata
/// 3. If new, store the complete schedule (blocks, dark periods, etc.)
/// 4. Populate analytics tables (best-effort, failures are logged but not fatal)
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `schedule` - The schedule to store
///
/// # Returns
/// * `Ok(ScheduleMetadata)` - Metadata of stored schedule (new or existing)
/// * `Err` if storage fails
pub async fn store_schedule(
    repo: &dyn ScheduleRepository,
    schedule: &Schedule,
) -> RepositoryResult<ScheduleMetadata> {
    store_schedule_with_options(repo, schedule, true, false).await
}

/// Store a new schedule with optional analytics population.
///
/// This function provides control over analytics computation, allowing fast uploads
/// when analytics are not immediately needed.
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `schedule` - The schedule to store
/// * `populate_analytics` - If true, populate block and summary analytics (recommended)
/// * `skip_time_bins` - If true, skip visibility time bin computation (expensive, ~minutes for large schedules)
///
/// # Returns
/// * `Ok(ScheduleMetadata)` - Metadata of stored schedule (new or existing)
/// * `Err` if storage fails
///
/// # Performance Note
/// For large schedules (>1000 blocks), consider:
/// - `populate_analytics=true, skip_time_bins=true` for fast upload with basic analytics
/// - `populate_analytics=false, skip_time_bins=true` for fastest upload (compute analytics later)
pub async fn store_schedule_with_options(
    repo: &dyn ScheduleRepository,
    schedule: &Schedule,
    populate_analytics: bool,
    skip_time_bins: bool,
) -> RepositoryResult<ScheduleMetadata> {
    info!(
        "Service layer: storing schedule '{}' (checksum {}, {} blocks, analytics={}, skip_bins={})",
        schedule.name,
        schedule.checksum,
        schedule.blocks.len(),
        populate_analytics,
        skip_time_bins
    );

    // Try to store the schedule
    let metadata = repo.store_schedule(schedule).await?;

    // Optionally populate analytics for the schedule (best-effort)
    if populate_analytics {
        if let Some(schedule_id) = metadata.schedule_id {
            info!(
                "Service layer: populating analytics for schedule_id={}",
                schedule_id
            );

            // Phase 1: Block-level analytics (FAST - required for dashboard)
            let start = std::time::Instant::now();
            match repo.populate_schedule_analytics(schedule_id).await {
                Ok(analytics_count) => {
                    info!(
                        "Service layer: ✓ Phase 1/3: Populated {} analytics rows in {:.2}s",
                        analytics_count,
                        start.elapsed().as_secs_f64()
                    );
                }
                Err(e) => {
                    warn!(
                        "Service layer: failed to populate block-level analytics for schedule_id={}: {}",
                        schedule_id, e
                    );
                }
            }

            // Phase 2: Summary analytics (FAST - 10 bins for histograms)
            let start = std::time::Instant::now();
            match repo.populate_summary_analytics(schedule_id, 10).await {
                Ok(()) => {
                    info!(
                        "Service layer: ✓ Phase 2/3: Populated summary analytics in {:.2}s",
                        start.elapsed().as_secs_f64()
                    );
                }
                Err(e) => {
                    warn!(
                        "Service layer: failed to populate summary analytics for schedule_id={}: {}",
                        schedule_id, e
                    );
                }
            }

            // Phase 3: Visibility time bins (SLOW - optional, can be computed later)
            if !skip_time_bins {
                let start = std::time::Instant::now();
                info!("Service layer: Phase 3/3: Computing visibility time bins (this may take several minutes for large schedules)...");
                match repo
                    .populate_visibility_time_bins(schedule_id, Some(900))
                    .await
                {
                    Ok((metadata_count, bins_count)) => {
                        info!(
                            "Service layer: ✓ Phase 3/3: Populated {} visibility metadata and {} time bins in {:.2}s",
                            metadata_count, bins_count, start.elapsed().as_secs_f64()
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Service layer: failed to populate visibility time bins for schedule_id={}: {}",
                            schedule_id, e
                        );
                    }
                }
            } else {
                info!("Service layer: Phase 3/3: Skipped visibility time bins (skip_time_bins=true)");
            }
        }
    } else {
        info!("Service layer: Skipped analytics population (populate_analytics=false)");
    }

    Ok(metadata)
}

/// Retrieve a complete schedule by ID.
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `schedule_id` - The ID of the schedule to retrieve
///
/// # Returns
/// * `Ok(Schedule)` - The complete schedule with all blocks and dark periods
/// * `Err` if schedule not found or retrieval fails
pub async fn get_schedule(
    repo: &dyn ScheduleRepository,
    schedule_id: i64,
) -> RepositoryResult<Schedule> {
    info!("Service layer: loading schedule by id {}", schedule_id);
    repo.get_schedule(schedule_id).await
}

/// Retrieve a schedule by name.
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `schedule_name` - The name of the schedule to retrieve
///
/// # Returns
/// * `Ok(Schedule)` - The complete schedule
/// * `Err` if schedule not found or retrieval fails
pub async fn get_schedule_by_name(
    repo: &dyn ScheduleRepository,
    schedule_name: &str,
) -> RepositoryResult<Schedule> {
    info!("Service layer: loading schedule '{}'", schedule_name);
    repo.get_schedule_by_name(schedule_name).await
}

/// List all schedules with basic metadata.
///
/// # Arguments
/// * `repo` - Repository implementation
///
/// # Returns
/// * `Ok(Vec<ScheduleInfo>)` - List of schedule metadata
/// * `Err` if query fails
pub async fn list_schedules(repo: &dyn ScheduleRepository) -> RepositoryResult<Vec<ScheduleInfo>> {
    info!("Service layer: listing all schedules");
    repo.list_schedules().await
}

/// Get the time range covered by a schedule.
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `schedule_id` - The ID of the schedule
///
/// # Returns
/// * `Ok(Some((start_mjd, stop_mjd)))` - Time range in Modified Julian Date
/// * `Ok(None)` - If schedule has no time constraints
/// * `Err` if query fails
pub async fn get_schedule_time_range(
    repo: &dyn ScheduleRepository,
    schedule_id: i64,
) -> RepositoryResult<Option<(f64, f64)>> {
    repo.get_schedule_time_range(schedule_id).await
}

// ==================== Scheduling Block Operations ====================

/// Get a single scheduling block by ID.
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `scheduling_block_id` - The ID of the block
///
/// # Returns
/// * `Ok(SchedulingBlock)` - The scheduling block with all details
/// * `Err` if block not found or query fails
pub async fn get_scheduling_block(
    repo: &dyn ScheduleRepository,
    scheduling_block_id: i64,
) -> RepositoryResult<SchedulingBlock> {
    repo.get_scheduling_block(scheduling_block_id).await
}

/// Get all scheduling blocks for a schedule.
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `schedule_id` - The ID of the schedule
///
/// # Returns
/// * `Ok(Vec<SchedulingBlock>)` - List of all blocks
/// * `Err` if query fails
pub async fn get_blocks_for_schedule(
    repo: &dyn ScheduleRepository,
    schedule_id: i64,
) -> RepositoryResult<Vec<SchedulingBlock>> {
    repo.get_blocks_for_schedule(schedule_id).await
}

// ==================== Dark Periods & Possible Periods ====================

/// Fetch dark periods (observing windows) for a schedule.
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `schedule_id` - The ID of the schedule
///
/// # Returns
/// * `Ok(Vec<(f64, f64)>)` - List of (start_mjd, stop_mjd)
/// * `Err` if query fails
pub async fn fetch_dark_periods(
    repo: &dyn ScheduleRepository,
    schedule_id: i64,
) -> RepositoryResult<Vec<(f64, f64)>> {
    repo.fetch_dark_periods(schedule_id).await
}

/// Fetch possible observation periods for a schedule.
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `schedule_id` - The ID of the schedule
///
/// # Returns
/// * `Ok(Vec<(i64, f64, f64)>)` - List of (block_id, start_mjd, stop_mjd)
/// * `Err` if query fails
pub async fn fetch_possible_periods(
    repo: &dyn ScheduleRepository,
    schedule_id: i64,
) -> RepositoryResult<Vec<(i64, f64, f64)>> {
    repo.fetch_possible_periods(schedule_id).await
}

// ==================== Analytics Operations ====================

/// Ensure analytics are populated for a schedule.
///
/// This checks if analytics exist and populates them if needed.
/// This is useful when working with schedules that may have been
/// uploaded before analytics were implemented.
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `schedule_id` - The ID of the schedule
///
/// # Returns
/// * `Ok(())` if analytics are available
/// * `Err` if population fails
pub async fn ensure_analytics(
    repo: &dyn ScheduleRepository,
    schedule_id: i64,
) -> RepositoryResult<()> {
    if !repo.has_analytics_data(schedule_id).await? {
        info!(
            "Service layer: analytics missing for schedule_id={}, populating...",
            schedule_id
        );
        repo.populate_schedule_analytics(schedule_id).await?;
        repo.populate_summary_analytics(schedule_id, 10).await?;
        repo.populate_visibility_time_bins(schedule_id, Some(900))
            .await?;
    }
    Ok(())
}

/// Check if analytics data exists for a schedule.
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `schedule_id` - The ID of the schedule
///
/// # Returns
/// * `Ok(bool)` - True if analytics exist
/// * `Err` if query fails
pub async fn has_analytics_data(
    repo: &dyn ScheduleRepository,
    schedule_id: i64,
) -> RepositoryResult<bool> {
    repo.has_analytics_data(schedule_id).await
}

/// Check if summary analytics exist for a schedule.
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `schedule_id` - The ID of the schedule
///
/// # Returns
/// * `Ok(bool)` - True if summary analytics exist
/// * `Err` if query fails
pub async fn has_summary_analytics(
    repo: &dyn ScheduleRepository,
    schedule_id: i64,
) -> RepositoryResult<bool> {
    repo.has_summary_analytics(schedule_id).await
}

/// Check if visibility time bins exist for a schedule.
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `schedule_id` - The ID of the schedule
///
/// # Returns
/// * `Ok(bool)` - True if visibility time bins exist
/// * `Err` if query fails
pub async fn has_visibility_time_bins(
    repo: &dyn ScheduleRepository,
    schedule_id: i64,
) -> RepositoryResult<bool> {
    repo.has_visibility_time_bins(schedule_id).await
}
