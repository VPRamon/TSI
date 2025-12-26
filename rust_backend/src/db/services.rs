//! High-level database service layer.
//!
//! This module provides repository-agnostic database operations that work with
//! any implementation of the repository traits. These functions contain
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
//! │  Repository Traits (repository/) - Abstract Interface    │
//! │  - ScheduleRepository (core CRUD)                        │
//! │  - AnalyticsRepository (analytics ops)                   │
//! │  - ValidationRepository (validation)                     │
//! │  - VisualizationRepository (dashboard queries)           │
//! └───────────────────┬─────────────────────────────────────┘
//!                     │
//!     ┌───────────────┴────────────────┐
//!     │                                 │
//! ┌───▼──────────────┐     ┌──────────▼──────────────┐
//! │ Azure Repository │     │ Local Repository        │
//! │ (SQL queries)    │     │ (in-memory)             │
//! └──────────────────┘     └─────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ```no_run
//! use tsi_rust::db::{services, repositories::LocalRepository};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create local repository
//!     let repo = LocalRepository::new();
//!     
//!     // Use service layer functions
//!     let schedules = services::list_schedules(&repo).await?;
//!     println!("Found {} schedules", schedules.len());
//!     
//!     Ok(())
//! }
//! ```

use log::{info, warn};

use super::models::{Period, Schedule, SchedulingBlock};
use super::repository::{FullRepository, RepositoryResult};
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
pub async fn health_check<R: FullRepository>(repo: &R) -> RepositoryResult<bool> {
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
/// * `Ok(crate::api::ScheduleInfo)` - Metadata of stored schedule (new or existing)
/// * `Err` if storage fails
pub async fn store_schedule<R: FullRepository>(
    repo: &R,
    schedule: &Schedule,
) -> RepositoryResult<crate::api::ScheduleInfo> {
    store_schedule_with_options(repo, schedule, true).await
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
/// * `Ok(crate::api::ScheduleInfo)` - Metadata of stored schedule (new or existing)
/// * `Err` if storage fails
///
/// # Performance Note
/// For large schedules (>1000 blocks), consider:
/// - `populate_analytics=true, skip_time_bins=true` for fast upload with basic analytics
/// - `populate_analytics=false, skip_time_bins=true` for fastest upload (compute analytics later)
pub async fn store_schedule_with_options<R: FullRepository>(
    repo: &R,
    schedule: &Schedule,
    populate_analytics: bool,
) -> RepositoryResult<crate::api::ScheduleInfo> {
    info!(
        "Service layer: storing schedule '{}' (checksum {}, {} blocks, analytics={})",
        schedule.name,
        schedule.checksum,
        schedule.blocks.len(),
        populate_analytics,
    );

    // Try to store the schedule
    let metadata = repo.store_schedule(schedule).await?;

    // Optionally populate analytics for the schedule (best-effort)
    if populate_analytics {
        let schedule_id = metadata.schedule_id;
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
pub async fn get_schedule<R: FullRepository>(
    repo: &R,
    schedule_id: i64,
) -> RepositoryResult<Schedule> {
    info!("Service layer: loading schedule by id {}", schedule_id);
    repo.get_schedule(schedule_id).await
}

/// List all schedules with basic metadata.
///
/// # Arguments
/// * `repo` - Repository implementation
///
/// # Returns
/// * `Ok(Vec<crate::api::ScheduleInfo>)` - List of schedule metadata
/// * `Err` if query fails
pub async fn list_schedules<R: FullRepository>(repo: &R) -> RepositoryResult<Vec<crate::api::ScheduleInfo>> {
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
/// * `Ok(Some(Period))` - Time range as a Period
/// * `Ok(None)` - If schedule has no time constraints
/// * `Err` if query fails
pub async fn get_schedule_time_range<R: FullRepository>(
    repo: &R,
    schedule_id: i64,
) -> RepositoryResult<Option<Period>> {
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
pub async fn get_scheduling_block<R: FullRepository>(
    repo: &R,
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
pub async fn get_blocks_for_schedule<R: FullRepository>(
    repo: &R,
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
/// * `Ok(Vec<Period>)` - List of dark periods
/// * `Err` if query fails
pub async fn fetch_dark_periods<R: FullRepository>(
    repo: &R,
    schedule_id: i64,
) -> RepositoryResult<Vec<Period>> {
    repo.fetch_dark_periods(schedule_id).await
}

/// Fetch possible observation periods for a schedule.
///
/// # Arguments
/// * `repo` - Repository implementation
/// * `schedule_id` - The ID of the schedule
///
/// # Returns
/// * `Ok(Vec<Period>)` - List of visibility periods
/// * `Err` if query fails
pub async fn fetch_possible_periods<R: FullRepository>(
    repo: &R,
    schedule_id: i64,
) -> RepositoryResult<Vec<Period>> {
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
pub async fn ensure_analytics<R: FullRepository>(
    repo: &R,
    schedule_id: i64,
) -> RepositoryResult<()> {
    if !repo.has_analytics_data(schedule_id).await? {
        info!(
            "Service layer: analytics missing for schedule_id={}, populating...",
            schedule_id
        );
        repo.populate_schedule_analytics(schedule_id).await?;
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
pub async fn has_analytics_data<R: FullRepository>(
    repo: &R,
    schedule_id: i64,
) -> RepositoryResult<bool> {
    repo.has_analytics_data(schedule_id).await
}


// =============================================================================
// Schedule Parsing and Storage Helpers
// =============================================================================

/// Parse schedule from JSON strings.
///
/// This helper function reads dark periods and parses the schedule JSON,
/// setting the schedule name.
///
/// # Arguments
/// * `schedule_name` - Name to assign to the schedule
/// * `schedule_json` - JSON string containing schedule data
/// * `visibility_json` - Optional JSON string containing visibility periods
///
/// # Returns
/// * `Ok(Schedule)` - Parsed schedule
/// * `Err` if parsing fails or dark_periods.json cannot be read
///
/// # Errors
/// Returns an error if:
/// - The dark_periods.json file cannot be read
/// - The JSON parsing fails
pub fn parse_schedule_from_json(
    schedule_name: &str,
    schedule_json: &str,
    visibility_json: Option<&str>,
) -> anyhow::Result<crate::db::models::Schedule> {
    use anyhow::Context;

    let dark_periods = std::fs::read_to_string("data/dark_periods.json")
        .context("Failed to read dark_periods.json")?;

    let mut schedule: crate::db::models::Schedule =
        crate::db::models::schedule::parse_schedule_json_str(
            schedule_json,
            visibility_json,
            dark_periods.as_str(),
        )
        .context("Failed to parse schedule")?;
    
    schedule.name = schedule_name.to_string();

    Ok(schedule)
}

/// Store schedule in database with options (synchronous wrapper).
///
/// This helper function handles the async runtime creation and repository
/// interaction for storing a schedule. It blocks on the async operation,
/// making it suitable for use in synchronous contexts like Python bindings.
///
/// # Arguments
/// * `schedule` - The schedule to store
/// * `populate_analytics` - Whether to run Phase 1 analytics ETL
/// * `skip_time_bins` - Whether to skip Phase 3 time bin population
///
/// # Returns
/// * `Ok(crate::api::ScheduleInfo)` - Metadata of the stored schedule
/// * `Err` if storage or analytics population fails
///
/// # Errors
/// Returns an error if:
/// - The async runtime cannot be created
/// - The repository is not initialized
/// - The schedule storage fails
/// - Analytics population fails (if requested)
pub fn store_schedule_sync(
    schedule: &crate::db::models::Schedule,
    populate_analytics: bool,
) -> anyhow::Result<crate::api::ScheduleInfo> {
    use anyhow::Context;
    use tokio::runtime::Runtime;

    let runtime = Runtime::new().context("Failed to create async runtime")?;
    let repo = crate::db::get_repository().context("Repository not initialized")?;
    
    Ok(runtime.block_on(store_schedule_with_options(
        repo.as_ref(),
        schedule,
        populate_analytics,
    ))?)
}
