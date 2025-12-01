//! Repository trait for abstracting database operations.
//!
//! This trait defines the interface for all database operations, allowing
//! different implementations (Azure SQL Server, in-memory mock, etc.) to be
//! swapped via dependency injection.

use async_trait::async_trait;

use super::models::{Schedule, ScheduleInfo, ScheduleMetadata, SchedulingBlock};
use super::validation::ValidationReportData;
use super::analytics::{
    HeatmapBinData, PriorityRate, ScheduleSummary, VisibilityBin, VisibilityTimeBin,
    VisibilityTimeMetadata,
};
use crate::services::validation::ValidationResult;

/// Result type for repository operations
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// Error type for repository operations
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Query error: {0}")]
    QueryError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Data validation error: {0}")]
    ValidationError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<String> for RepositoryError {
    fn from(s: String) -> Self {
        RepositoryError::InternalError(s)
    }
}

impl From<&str> for RepositoryError {
    fn from(s: &str) -> Self {
        RepositoryError::InternalError(s.to_string())
    }
}

/// Repository trait for schedule database operations.
///
/// This trait defines all database operations needed for the scheduling system.
/// Implementations can use different backends (Azure SQL Server, PostgreSQL,
/// in-memory storage, etc.).
///
/// # Thread Safety
/// Implementations must be `Send + Sync` to work with async Rust and allow
/// sharing across threads.
///
/// # Error Handling
/// All methods return `RepositoryResult<T>` which wraps either the expected
/// return type or a `RepositoryError` describing what went wrong.
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
    /// * `Ok(Some((start_mjd, stop_mjd)))` - Time range in Modified Julian Date
    /// * `Ok(None)` - If the schedule has no time constraints
    /// * `Err(RepositoryError)` - If the operation fails
    async fn get_schedule_time_range(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Option<(f64, f64)>>;

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
    /// * `Ok(Vec<(i64, f64, f64)>)` - List of (period_id, start_mjd, stop_mjd)
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_dark_periods(&self, schedule_id: i64) -> RepositoryResult<Vec<(f64, f64)>>;

    /// Fetch possible observation periods for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<(i64, f64, f64)>)` - List of (period_id, start_mjd, stop_mjd)
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_possible_periods(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<(i64, f64, f64)>>;

    // ==================== Analytics Operations ====================

    /// Populate the analytics table for a schedule.
    ///
    /// This pre-computes denormalized data for faster queries.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule to analyze
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of analytics rows inserted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn populate_schedule_analytics(&self, schedule_id: i64) -> RepositoryResult<usize>;

    /// Delete analytics data for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of rows deleted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn delete_schedule_analytics(&self, schedule_id: i64) -> RepositoryResult<usize>;

    /// Check if analytics data exists for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(bool)` - True if analytics data exists
    /// * `Err(RepositoryError)` - If the operation fails
    async fn has_analytics_data(&self, schedule_id: i64) -> RepositoryResult<bool>;

    /// Fetch analytics blocks for sky map visualization.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<LightweightBlock>)` - Blocks optimized for sky map display
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_analytics_blocks_for_sky_map(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<super::models::LightweightBlock>>;

    /// Fetch analytics blocks for distribution charts.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<LightweightBlock>)` - Blocks for distribution analysis
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_analytics_blocks_for_distribution(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<super::models::DistributionBlock>>;

    // ==================== Summary Analytics ====================

    /// Populate summary analytics (aggregated statistics) for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    /// * `n_bins` - Number of bins for histogram data
    ///
    /// # Returns
    /// * `Ok(())` - If successful
    /// * `Err(RepositoryError)` - If the operation fails
    async fn populate_summary_analytics(
        &self,
        schedule_id: i64,
        n_bins: usize,
    ) -> RepositoryResult<()>;

    /// Fetch schedule summary statistics.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Some(ScheduleSummary))` - Summary statistics if available
    /// * `Ok(None)` - If no summary exists
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_schedule_summary(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Option<ScheduleSummary>>;

    /// Fetch priority rate distribution.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<PriorityRate>)` - Priority distribution data
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_priority_rates(&self, schedule_id: i64)
        -> RepositoryResult<Vec<PriorityRate>>;

    /// Fetch visibility bins for histogram.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<VisibilityBin>)` - Visibility histogram data
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_visibility_bins(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<VisibilityBin>>;

    /// Fetch heatmap bin data.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<HeatmapBinData>)` - Heatmap data
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_heatmap_bins(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<HeatmapBinData>>;

    /// Check if summary analytics exist for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(bool)` - True if summary analytics exist
    /// * `Err(RepositoryError)` - If the operation fails
    async fn has_summary_analytics(&self, schedule_id: i64) -> RepositoryResult<bool>;

    /// Delete summary analytics for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of rows deleted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn delete_summary_analytics(&self, schedule_id: i64) -> RepositoryResult<usize>;

    // ==================== Visibility Time Bins ====================

    /// Populate visibility time bins for histogram analysis.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    /// * `bin_duration_seconds` - Optional bin duration (default: 900 seconds)
    ///
    /// # Returns
    /// * `Ok((usize, usize))` - Tuple of (metadata_rows, bin_rows) inserted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn populate_visibility_time_bins(
        &self,
        schedule_id: i64,
        bin_duration_seconds: Option<i64>,
    ) -> RepositoryResult<(usize, usize)>;

    /// Fetch visibility histogram data from analytics.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    /// * `start_unix` - Start time in Unix seconds
    /// * `end_unix` - End time in Unix seconds
    /// * `target_bin_duration_seconds` - Target bin duration
    ///
    /// # Returns
    /// * `Ok(Vec<VisibilityBin>)` - Histogram bins
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_visibility_histogram_from_analytics(
        &self,
        schedule_id: i64,
        start_unix: i64,
        end_unix: i64,
        target_bin_duration_seconds: i64,
    ) -> RepositoryResult<Vec<super::models::VisibilityBin>>;

    /// Fetch visibility metadata (range, bins, etc.).
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Some(VisibilityTimeMetadata))` - Metadata if available
    /// * `Ok(None)` - If no metadata exists
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_visibility_metadata(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Option<VisibilityTimeMetadata>>;

    /// Check if visibility time bins exist for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(bool)` - True if bins exist
    /// * `Err(RepositoryError)` - If the operation fails
    async fn has_visibility_time_bins(&self, schedule_id: i64) -> RepositoryResult<bool>;

    /// Delete visibility time bins for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(usize)` - Number of rows deleted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn delete_visibility_time_bins(&self, schedule_id: i64) -> RepositoryResult<usize>;

    // ==================== Validation Operations ====================

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
    /// * `Ok(ValidationReportData)` - Validation report with all issues
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_validation_results(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<ValidationReportData>;

    /// Check if validation results exist for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(bool)` - True if validation results exist
    /// * `Err(RepositoryError)` - If the operation fails
    async fn has_validation_results(&self, schedule_id: i64) -> RepositoryResult<bool>;

    /// Delete validation results for a schedule.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(u64)` - Number of rows deleted
    /// * `Err(RepositoryError)` - If the operation fails
    async fn delete_validation_results(&self, schedule_id: i64) -> RepositoryResult<u64>;

    // ==================== Specialized Query Operations ====================

    /// Fetch lightweight blocks (minimal data) for processing.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<LightweightBlock>)` - Blocks with minimal fields populated
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_lightweight_blocks(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<super::models::LightweightBlock>>;

    /// Fetch blocks optimized for insights display.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<InsightsBlock>)` - Blocks for insights view
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_insights_blocks(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<super::models::InsightsBlock>>;

    /// Fetch blocks optimized for trends analysis.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<TrendsBlock>)` - Blocks for trends analysis
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_trends_blocks(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<super::models::TrendsBlock>>;

    /// Fetch blocks for visibility map visualization.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<VisibilityMapBlock>)` - Blocks for visibility map
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_visibility_map_data(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<super::models::VisibilityMapData>;

    /// Fetch blocks for histogram generation.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    /// * `priority_min` - Optional minimum priority filter
    /// * `priority_max` - Optional maximum priority filter
    /// * `block_ids` - Optional specific block IDs to fetch
    ///
    /// # Returns
    /// * `Ok(Vec<BlockHistogramData>)` - Blocks for histogram
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_blocks_for_histogram(
        &self,
        schedule_id: i64,
        priority_min: Option<i32>,
        priority_max: Option<i32>,
        block_ids: Option<Vec<i64>>,
    ) -> RepositoryResult<Vec<super::models::BlockHistogramData>>;

    /// Fetch blocks for timeline visualization.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<TimelineBlock>)` - Blocks for timeline
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_schedule_timeline_blocks(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<super::models::ScheduleTimelineBlock>>;

    /// Fetch blocks for comparison view.
    ///
    /// # Arguments
    /// * `schedule_id` - The ID of the schedule
    ///
    /// # Returns
    /// * `Ok(Vec<ComparisonBlock>)` - Blocks for comparison
    /// * `Err(RepositoryError)` - If the operation fails
    async fn fetch_compare_blocks(
        &self,
        schedule_id: i64,
    ) -> RepositoryResult<Vec<super::models::CompareBlock>>;
}
