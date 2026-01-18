//! Postgres repository implementation using Diesel.
//!
//! This module implements the repository traits against a Postgres database
//! following the schema defined in `docs/POSTGRES_ETL_DB_DESIGN.md`.
//!
//! ## Features
//!
//! - Connection pooling with r2d2
//! - Automatic retry for transient failures
//! - Connection health monitoring
//! - Automatic migration execution
//!
//! ## Configuration
//!
//! Environment variables:
//! - `DATABASE_URL` or `PG_DATABASE_URL`: Connection string (required)
//! - `PG_POOL_MAX`: Maximum pool size (default: 10)
//! - `PG_POOL_MIN`: Minimum pool size (default: 1)
//! - `PG_CONN_TIMEOUT_SEC`: Connection timeout in seconds (default: 30)
//! - `PG_IDLE_TIMEOUT_SEC`: Idle connection timeout in seconds (default: 600)
//! - `PG_MAX_RETRIES`: Maximum retry attempts for transient failures (default: 3)
//! - `PG_RETRY_DELAY_MS`: Initial retry delay in milliseconds (default: 100)

use async_trait::async_trait;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_query;
use diesel::upsert::excluded;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::task;

use crate::api::{
    CompareBlock, Constraints, DistributionBlock, InsightsBlock, LightweightBlock,
    ModifiedJulianDate, Period, Schedule, ScheduleId, ScheduleInfo, ScheduleTimelineBlock,
    SchedulingBlock, SchedulingBlockId, VisibilityBlockSummary, VisibilityMapData,
};
use crate::db::repository::{
    AnalyticsRepository, ErrorContext, RepositoryError, RepositoryResult, ScheduleRepository,
    ValidationRepository, VisualizationRepository,
};
use crate::services::validation::{
    validate_blocks, BlockForValidation, ValidationResult, ValidationStatus,
};

mod models;
mod schema;

use models::*;
use schema::*;

type PgPool = Pool<ConnectionManager<PgConnection>>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/db/repositories/postgres/migrations");

/// Configuration for connecting to Postgres.
#[derive(Debug, Clone)]
pub struct PostgresConfig {
    /// Database connection URL
    pub database_url: String,
    /// Maximum number of connections in the pool
    pub max_pool_size: u32,
    /// Minimum number of connections in the pool
    pub min_pool_size: u32,
    /// Connection timeout in seconds
    pub connection_timeout_sec: u64,
    /// Idle connection timeout in seconds
    pub idle_timeout_sec: u64,
    /// Maximum number of retry attempts for transient failures
    pub max_retries: u32,
    /// Initial retry delay in milliseconds (doubles with each retry)
    pub retry_delay_ms: u64,
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            database_url: String::new(),
            max_pool_size: 10,
            min_pool_size: 1,
            connection_timeout_sec: 30,
            idle_timeout_sec: 600,
            max_retries: 3,
            retry_delay_ms: 100,
        }
    }
}

impl PostgresConfig {
    /// Create configuration from environment variables.
    ///
    /// # Environment Variables
    /// - `DATABASE_URL` or `PG_DATABASE_URL`: Connection string (required)
    /// - `PG_POOL_MAX`: Maximum pool size (default: 10)
    /// - `PG_POOL_MIN`: Minimum pool size (default: 1)
    /// - `PG_CONN_TIMEOUT_SEC`: Connection timeout in seconds (default: 30)
    /// - `PG_IDLE_TIMEOUT_SEC`: Idle connection timeout in seconds (default: 600)
    /// - `PG_MAX_RETRIES`: Maximum retry attempts (default: 3)
    /// - `PG_RETRY_DELAY_MS`: Initial retry delay in milliseconds (default: 100)
    pub fn from_env() -> Result<Self, String> {
        let database_url = std::env::var("DATABASE_URL")
            .or_else(|_| std::env::var("PG_DATABASE_URL"))
            .map_err(|_| "DATABASE_URL or PG_DATABASE_URL must be set".to_string())?;

        let max_pool_size = std::env::var("PG_POOL_MAX")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(10);

        let min_pool_size = std::env::var("PG_POOL_MIN")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(1);

        let connection_timeout_sec = std::env::var("PG_CONN_TIMEOUT_SEC")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(30);

        let idle_timeout_sec = std::env::var("PG_IDLE_TIMEOUT_SEC")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(600);

        let max_retries = std::env::var("PG_MAX_RETRIES")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(3);

        let retry_delay_ms = std::env::var("PG_RETRY_DELAY_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(100);

        Ok(Self {
            database_url,
            max_pool_size,
            min_pool_size,
            connection_timeout_sec,
            idle_timeout_sec,
            max_retries,
            retry_delay_ms,
        })
    }

    /// Create a new configuration with a database URL.
    pub fn with_url(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            ..Default::default()
        }
    }
}

/// Pool health statistics.
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
    /// Number of connections currently in use
    pub connections_in_use: u32,
    /// Number of idle connections
    pub idle_connections: u32,
    /// Total number of connections in the pool
    pub total_connections: u32,
    /// Maximum pool size
    pub max_size: u32,
    /// Total successful queries executed
    pub total_queries: u64,
    /// Total failed queries
    pub failed_queries: u64,
    /// Total retried operations
    pub retried_operations: u64,
}

/// Diesel-backed repository for Postgres.
///
/// This repository implementation provides:
/// - Connection pooling with configurable limits
/// - Automatic retry for transient failures
/// - Health monitoring and statistics
/// - Automatic schema migrations
#[derive(Clone, Debug)]
pub struct PostgresRepository {
    pool: PgPool,
    config: PostgresConfig,
    // Metrics counters
    total_queries: std::sync::Arc<AtomicU64>,
    failed_queries: std::sync::Arc<AtomicU64>,
    retried_operations: std::sync::Arc<AtomicU64>,
}

impl PostgresRepository {
    /// Create a new repository and run pending migrations.
    ///
    /// # Arguments
    /// * `config` - Database configuration
    ///
    /// # Returns
    /// * `Ok(PostgresRepository)` on success
    /// * `Err(RepositoryError)` if connection or migration fails
    pub fn new(config: PostgresConfig) -> RepositoryResult<Self> {
        let manager = ConnectionManager::<PgConnection>::new(&config.database_url);

        let pool = Pool::builder()
            .max_size(config.max_pool_size)
            .min_idle(Some(config.min_pool_size))
            .connection_timeout(Duration::from_secs(config.connection_timeout_sec))
            .idle_timeout(Some(Duration::from_secs(config.idle_timeout_sec)))
            .test_on_check_out(true) // Validate connections before use
            .build(manager)
            .map_err(|e| {
                RepositoryError::connection_with_context(
                    e.to_string(),
                    ErrorContext::new("create_pool")
                        .with_details(format!("max_size={}", config.max_pool_size)),
                )
            })?;

        // Run migrations once during initialization
        {
            let mut conn = pool.get().map_err(|e| {
                RepositoryError::connection_with_context(
                    e.to_string(),
                    ErrorContext::new("get_connection_for_migrations"),
                )
            })?;
            Self::run_migrations(&mut conn)?;
        }

        Ok(Self {
            pool,
            config,
            total_queries: std::sync::Arc::new(AtomicU64::new(0)),
            failed_queries: std::sync::Arc::new(AtomicU64::new(0)),
            retried_operations: std::sync::Arc::new(AtomicU64::new(0)),
        })
    }

    /// Run pending database migrations.
    fn run_migrations(conn: &mut PgConnection) -> RepositoryResult<()> {
        conn.run_pending_migrations(MIGRATIONS).map_err(|e| {
            RepositoryError::internal_with_context(
                format!("Migration failed: {}", e),
                ErrorContext::new("run_migrations"),
            )
        })?;

        Ok(())
    }

    /// Execute a database operation with automatic retry for transient failures.
    ///
    /// This method will retry the operation up to `max_retries` times if a
    /// retryable error occurs (connection errors, timeouts, serialization failures).
    async fn with_conn<T, F>(&self, f: F) -> RepositoryResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut PgConnection) -> RepositoryResult<T> + Send + 'static + Clone,
    {
        let pool = self.pool.clone();
        let max_retries = self.config.max_retries;
        let retry_delay_ms = self.config.retry_delay_ms;
        let total_queries = self.total_queries.clone();
        let failed_queries = self.failed_queries.clone();
        let retried_operations = self.retried_operations.clone();

        task::spawn_blocking(move || {
            let mut last_error = None;
            let mut retry_delay = Duration::from_millis(retry_delay_ms);

            for attempt in 0..=max_retries {
                if attempt > 0 {
                    retried_operations.fetch_add(1, Ordering::Relaxed);
                    std::thread::sleep(retry_delay);
                    retry_delay *= 2; // Exponential backoff
                }

                // Get connection
                let mut conn = match pool.get() {
                    Ok(c) => c,
                    Err(e) => {
                        let err = RepositoryError::connection_with_context(
                            e.to_string(),
                            ErrorContext::new("get_connection")
                                .with_details(format!("attempt={}", attempt + 1))
                                .retryable(),
                        );
                        if attempt < max_retries {
                            last_error = Some(err);
                            continue;
                        }
                        failed_queries.fetch_add(1, Ordering::Relaxed);
                        return Err(err);
                    }
                };

                // Execute the operation
                total_queries.fetch_add(1, Ordering::Relaxed);
                match f.clone()(&mut conn) {
                    Ok(result) => return Ok(result),
                    Err(e) if e.is_retryable() && attempt < max_retries => {
                        last_error = Some(e);
                        continue;
                    }
                    Err(e) => {
                        failed_queries.fetch_add(1, Ordering::Relaxed);
                        return Err(e);
                    }
                }
            }

            failed_queries.fetch_add(1, Ordering::Relaxed);
            Err(last_error.unwrap_or_else(|| {
                RepositoryError::internal("Max retries exceeded with no error captured")
            }))
        })
        .await
        .map_err(|e| {
            RepositoryError::internal_with_context(
                format!("Task join error: {}", e),
                ErrorContext::new("spawn_blocking"),
            )
        })?
    }

    /// Get pool health statistics.
    ///
    /// Returns current pool state and query statistics for monitoring.
    pub fn get_pool_stats(&self) -> PoolStats {
        let state = self.pool.state();
        PoolStats {
            connections_in_use: state.connections - state.idle_connections,
            idle_connections: state.idle_connections,
            total_connections: state.connections,
            max_size: self.config.max_pool_size,
            total_queries: self.total_queries.load(Ordering::Relaxed),
            failed_queries: self.failed_queries.load(Ordering::Relaxed),
            retried_operations: self.retried_operations.load(Ordering::Relaxed),
        }
    }

    /// Check if the database connection is healthy.
    ///
    /// Performs a simple query to verify connectivity.
    pub async fn is_healthy(&self) -> bool {
        self.health_check().await.unwrap_or(false)
    }

    /// Get detailed health information.
    ///
    /// Returns a tuple of (is_healthy, latency_ms, error_message).
    pub async fn health_check_detailed(&self) -> (bool, Option<u64>, Option<String>) {
        let start = Instant::now();
        match self.health_check().await {
            Ok(true) => (true, Some(start.elapsed().as_millis() as u64), None),
            Ok(false) => (
                false,
                Some(start.elapsed().as_millis() as u64),
                Some("Health check returned false".to_string()),
            ),
            Err(e) => (
                false,
                Some(start.elapsed().as_millis() as u64),
                Some(e.to_string()),
            ),
        }
    }
}

fn map_diesel_error(err: diesel::result::Error) -> RepositoryError {
    RepositoryError::from(err)
}

fn periods_to_json(periods: &[Period]) -> Value {
    serde_json::to_value(periods).unwrap_or_else(|_| json!([]))
}

fn scheduled_period_to_json(period: &Option<Period>) -> Value {
    match period {
        Some(p) => periods_to_json(std::slice::from_ref(p)),
        None => json!([]),
    }
}

fn period_to_json(period: &Period) -> Value {
    json!({
        "start": period.start.value(),
        "stop": period.stop.value()
    })
}

fn value_to_periods(value: &Value) -> RepositoryResult<Vec<Period>> {
    serde_json::from_value(value.clone())
        .map_err(|e| RepositoryError::InternalError(format!("Failed to parse period JSON: {e}")))
}

fn value_to_single_period(value: &Value) -> RepositoryResult<Option<Period>> {
    let mut periods: Vec<Period> = value_to_periods(value)?;
    Ok(periods.pop())
}

fn json_to_period(value: &Value) -> RepositoryResult<Option<Period>> {
    if value.is_null() {
        return Ok(None);
    }
    serde_json::from_value(value.clone())
        .map(Some)
        .map_err(|e| {
            RepositoryError::InternalError(format!("Failed to parse schedule_period JSON: {e}"))
        })
}

fn compute_possible_periods_json(blocks: &[SchedulingBlock]) -> Value {
    let mut all_periods: Vec<Period> = Vec::new();
    for block in blocks {
        all_periods.extend(block.visibility_periods.clone());
    }
    periods_to_json(&all_periods)
}

fn priority_bucket(priority: f64) -> i16 {
    // Simple quartile-style bucket across 0-10 range
    if priority < 2.5 {
        1
    } else if priority < 5.0 {
        2
    } else if priority < 7.5 {
        3
    } else {
        4
    }
}

fn row_to_block(row: ScheduleBlockRow) -> RepositoryResult<SchedulingBlock> {
    let constraints = Constraints {
        min_alt: row
            .min_altitude_deg
            .unwrap_or_else(|| qtty::Degrees::new(0.0)),
        max_alt: row
            .max_altitude_deg
            .unwrap_or_else(|| qtty::Degrees::new(0.0)),
        min_az: row
            .min_azimuth_deg
            .unwrap_or_else(|| qtty::Degrees::new(0.0)),
        max_az: row
            .max_azimuth_deg
            .unwrap_or_else(|| qtty::Degrees::new(0.0)),
        fixed_time: match (row.constraint_start_mjd, row.constraint_stop_mjd) {
            (Some(start), Some(stop)) => Some(Period {
                start: ModifiedJulianDate::new(start),
                stop: ModifiedJulianDate::new(stop),
            }),
            _ => None,
        },
    };

    let scheduled_period = value_to_single_period(&row.scheduled_periods_json)?;
    let visibility_periods = value_to_periods(&row.visibility_periods_json)?;

    Ok(SchedulingBlock {
        id: Some(SchedulingBlockId(row.scheduling_block_id)),
        original_block_id: row.original_block_id.unwrap_or_default(),
        target_ra: row.target_ra_deg,
        target_dec: row.target_dec_deg,
        constraints,
        priority: row.priority,
        min_observation: (row.min_observation_sec as f64).into(),
        requested_duration: (row.requested_duration_sec as f64).into(),
        visibility_periods,
        scheduled_period,
    })
}

fn build_schedule_from_rows(
    schedule_row: ScheduleRow,
    block_rows: Vec<ScheduleBlockRow>,
) -> RepositoryResult<Schedule> {
    let dark_periods = value_to_periods(&schedule_row.dark_periods_json)?;
    let schedule_period = json_to_period(&schedule_row.schedule_period_json)?.ok_or_else(|| {
        RepositoryError::InternalError("schedule_period_json is required but was null".to_string())
    })?;
    let geographic_location: crate::api::GeographicLocation =
        serde_json::from_value(schedule_row.observer_location_json.clone()).map_err(|e| {
            RepositoryError::InternalError(format!("Failed to parse observer location: {}", e))
        })?;
    let astronomical_nights = value_to_periods(&schedule_row.astronomical_night_periods_json)?;
    let mut blocks = Vec::with_capacity(block_rows.len());
    for row in block_rows {
        blocks.push(row_to_block(row)?);
    }

    Ok(Schedule {
        id: Some(schedule_row.schedule_id),
        name: schedule_row.schedule_name,
        checksum: schedule_row.checksum,
        schedule_period,
        dark_periods,
        geographic_location,
        astronomical_nights,
        blocks,
    })
}

fn compute_summary_metrics(
    schedule_id: i64,
    block_rows: &[ScheduleBlockRow],
    analytics_rows: &[NewScheduleBlockAnalyticsRow],
) -> NewScheduleSummaryAnalyticsRow {
    let total_blocks = block_rows.len() as i32;
    let scheduled_blocks = analytics_rows.iter().filter(|r| r.scheduled).count() as i32;
    let unscheduled_blocks = total_blocks - scheduled_blocks;
    let impossible_blocks = analytics_rows
        .iter()
        .filter(|r| r.validation_impossible)
        .count() as i32;

    let scheduling_rate = if total_blocks > 0 {
        scheduled_blocks as f64 / total_blocks as f64
    } else {
        0.0
    };

    let priorities: Vec<f64> = block_rows.iter().map(|b| b.priority).collect();
    let priority_mean = if priorities.is_empty() {
        None
    } else {
        Some(priorities.iter().sum::<f64>() / priorities.len() as f64)
    };

    let priority_median = if priorities.is_empty() {
        None
    } else {
        let mut sorted = priorities.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sorted.len() / 2;
        let median = if sorted.len().is_multiple_of(2) {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        };
        Some(median)
    };

    let scheduled_priorities: Vec<f64> = block_rows
        .iter()
        .zip(analytics_rows.iter())
        .filter(|(_, a)| a.scheduled)
        .map(|(b, _)| b.priority)
        .collect();
    let priority_scheduled_mean = if scheduled_priorities.is_empty() {
        None
    } else {
        Some(scheduled_priorities.iter().sum::<f64>() / scheduled_priorities.len() as f64)
    };

    let unscheduled_priorities: Vec<f64> = block_rows
        .iter()
        .zip(analytics_rows.iter())
        .filter(|(_, a)| !a.scheduled)
        .map(|(b, _)| b.priority)
        .collect();
    let priority_unscheduled_mean = if unscheduled_priorities.is_empty() {
        None
    } else {
        Some(unscheduled_priorities.iter().sum::<f64>() / unscheduled_priorities.len() as f64)
    };

    let visibility_total_hours = qtty::Hours::new(
        analytics_rows
            .iter()
            .map(|r| r.total_visibility_hours.value())
            .sum::<f64>(),
    );
    let requested_mean_hours = if analytics_rows.is_empty() {
        None
    } else {
        Some(qtty::Hours::new(
            analytics_rows
                .iter()
                .map(|r| r.requested_hours.value())
                .sum::<f64>()
                / analytics_rows.len() as f64,
        ))
    };

    // Compute gap statistics from scheduled blocks
    let (gap_count, gap_mean_hours, gap_median_hours) = compute_gap_statistics(block_rows);

    NewScheduleSummaryAnalyticsRow {
        schedule_id,
        total_blocks,
        scheduled_blocks,
        unscheduled_blocks,
        impossible_blocks,
        scheduling_rate,
        priority_mean,
        priority_median,
        priority_scheduled_mean,
        priority_unscheduled_mean,
        visibility_total_hours,
        requested_mean_hours,
        gap_count,
        gap_mean_hours,
        gap_median_hours,
    }
}

fn compute_gap_statistics(
    block_rows: &[ScheduleBlockRow],
) -> (Option<i32>, Option<qtty::Hours>, Option<qtty::Hours>) {
    use serde_json::Value;

    // Extract all scheduled periods and sort by start time
    let mut scheduled_periods: Vec<(f64, f64)> = Vec::new();

    for block in block_rows {
        // Parse scheduled_periods_json - it could be a single period or array
        if let Ok(json_val) = serde_json::from_value::<Value>(block.scheduled_periods_json.clone())
        {
            if let Some(obj) = json_val.as_object() {
                // Single period: {"start": mjd, "stop": mjd}
                if let (Some(start), Some(stop)) = (obj.get("start"), obj.get("stop")) {
                    if let (Some(start_f), Some(stop_f)) = (start.as_f64(), stop.as_f64()) {
                        scheduled_periods.push((start_f, stop_f));
                    }
                }
            } else if let Some(arr) = json_val.as_array() {
                // Array of periods: [{"start": mjd, "stop": mjd}, ...]
                for period in arr {
                    if let Some(obj) = period.as_object() {
                        if let (Some(start), Some(stop)) = (obj.get("start"), obj.get("stop")) {
                            if let (Some(start_f), Some(stop_f)) = (start.as_f64(), stop.as_f64()) {
                                scheduled_periods.push((start_f, stop_f));
                            }
                        }
                    }
                }
            }
        }
    }

    // Sort by start time
    scheduled_periods.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // Compute gaps between consecutive observations
    let mut gaps_hours: Vec<f64> = Vec::new();
    for window in scheduled_periods.windows(2) {
        let (_, end1) = window[0];
        let (start2, _) = window[1];
        let gap_days = start2 - end1;
        if gap_days > 0.0 {
            gaps_hours.push(gap_days * 24.0); // Convert days to hours
        }
    }

    if gaps_hours.is_empty() {
        return (None, None, None);
    }

    let gap_count = gaps_hours.len() as i32;
    let gap_mean = gaps_hours.iter().sum::<f64>() / gaps_hours.len() as f64;

    let gap_median = {
        let mut sorted = gaps_hours.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mid = sorted.len() / 2;
        if sorted.len().is_multiple_of(2) {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        }
    };

    (
        Some(gap_count),
        Some(qtty::Hours::new(gap_mean)),
        Some(qtty::Hours::new(gap_median)),
    )
}

#[async_trait]
impl ScheduleRepository for PostgresRepository {
    async fn health_check(&self) -> RepositoryResult<bool> {
        self.with_conn(|conn| {
            sql_query("SELECT 1")
                .execute(conn)
                .map(|_| true)
                .map_err(map_diesel_error)
        })
        .await
    }

    async fn store_schedule(
        &self,
        schedule: &Schedule,
    ) -> RepositoryResult<crate::api::ScheduleInfo> {
        let schedule = schedule.clone();
        self.with_conn(move |conn| {
            conn.transaction(|tx| {
                // Idempotency: return existing schedule if checksum matches
                if let Ok(existing) = schedules::table
                    .filter(schedules::checksum.eq(&schedule.checksum))
                    .select(ScheduleRow::as_select())
                    .first::<ScheduleRow>(tx)
                {
                    return Ok(ScheduleInfo {
                        schedule_id: ScheduleId(existing.schedule_id),
                        schedule_name: existing.schedule_name,
                    });
                }

                let new_schedule = NewScheduleRow {
                    schedule_name: schedule.name.clone(),
                    checksum: schedule.checksum.clone(),
                    dark_periods_json: periods_to_json(&schedule.dark_periods),
                    possible_periods_json: compute_possible_periods_json(&schedule.blocks),
                    raw_schedule_json: serde_json::to_value(&schedule).ok(),
                    schedule_period_json: period_to_json(&schedule.schedule_period),
                    observer_location_json: serde_json::to_value(&schedule.geographic_location)
                        .map_err(|e| {
                            map_diesel_error(diesel::result::Error::SerializationError(Box::new(e)))
                        })?,
                    astronomical_night_periods_json: periods_to_json(&schedule.astronomical_nights),
                };

                let inserted: ScheduleRow = diesel::insert_into(schedules::table)
                    .values(&new_schedule)
                    .returning(ScheduleRow::as_returning())
                    .get_result(tx)
                    .map_err(map_diesel_error)?;

                let block_rows: Vec<NewScheduleBlockRow> = schedule
                    .blocks
                    .iter()
                    .enumerate()
                    .map(|(idx, b)| NewScheduleBlockRow {
                        schedule_id: inserted.schedule_id,
                        // Use client-provided id if present, otherwise generate from index
                        source_block_id: idx as i64 + 1, // ID generated by DB
                        original_block_id: Some(b.original_block_id.clone()),
                        priority: b.priority,
                        requested_duration_sec: b.requested_duration.value() as i32,
                        min_observation_sec: b.min_observation.value() as i32,
                        target_ra_deg: b.target_ra,
                        target_dec_deg: b.target_dec,
                        min_altitude_deg: Some(b.constraints.min_alt),
                        max_altitude_deg: Some(b.constraints.max_alt),
                        min_azimuth_deg: Some(b.constraints.min_az),
                        max_azimuth_deg: Some(b.constraints.max_az),
                        constraint_start_mjd: b
                            .constraints
                            .fixed_time
                            .as_ref()
                            .map(|p| p.start.value()),
                        constraint_stop_mjd: b
                            .constraints
                            .fixed_time
                            .as_ref()
                            .map(|p| p.stop.value()),
                        visibility_periods_json: periods_to_json(&b.visibility_periods),
                        scheduled_periods_json: scheduled_period_to_json(&b.scheduled_period),
                    })
                    .collect();

                if !block_rows.is_empty() {
                    // Insert blocks in chunks to avoid exceeding Postgres parameter limits
                    // (very large schedules can generate more than 65535 parameters in a single
                    // prepared statement). Choose a conservative chunk size.
                    let chunk_size: usize = 1000;
                    for chunk in block_rows.chunks(chunk_size) {
                        diesel::insert_into(schedule_blocks::table)
                            .values(chunk)
                            .execute(tx)
                            .map_err(map_diesel_error)?;
                    }
                }

                Ok(ScheduleInfo {
                    schedule_id: ScheduleId(inserted.schedule_id),
                    schedule_name: inserted.schedule_name,
                })
            })
        })
        .await
    }

    async fn get_schedule(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Schedule> {
        self.with_conn(move |conn| {
            let schedule_row = schedules::table
                .filter(schedules::schedule_id.eq(schedule_id.0))
                .select(ScheduleRow::as_select())
                .first::<ScheduleRow>(conn)
                .map_err(map_diesel_error)?;

            let block_rows = schedule_blocks::table
                .filter(schedule_blocks::schedule_id.eq(schedule_id.0))
                .select(ScheduleBlockRow::as_select())
                .load::<ScheduleBlockRow>(conn)
                .map_err(map_diesel_error)?;

            build_schedule_from_rows(schedule_row, block_rows)
        })
        .await
    }

    async fn list_schedules(&self) -> RepositoryResult<Vec<crate::api::ScheduleInfo>> {
        self.with_conn(|conn| {
            let rows: Vec<(i64, String)> = schedules::table
                .select((schedules::schedule_id, schedules::schedule_name))
                .order(schedules::uploaded_at.desc())
                .load(conn)
                .map_err(map_diesel_error)?;

            Ok(rows
                .into_iter()
                .map(|(id, name)| ScheduleInfo {
                    schedule_id: ScheduleId(id),
                    schedule_name: name,
                })
                .collect())
        })
        .await
    }

    async fn get_schedule_time_range(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Option<Period>> {
        self.with_conn(move |conn| {
            let schedule_period_json: Value = schedules::table
                .filter(schedules::schedule_id.eq(schedule_id.0))
                .select(schedules::schedule_period_json)
                .first(conn)
                .map_err(map_diesel_error)?;

            Ok(Some(json_to_period(&schedule_period_json)?.ok_or_else(
                || {
                    RepositoryError::InternalError(
                        "schedule_period_json is required but was null".to_string(),
                    )
                },
            )?))
        })
        .await
    }

    async fn get_scheduling_block(
        &self,
        scheduling_block_id: i64,
    ) -> RepositoryResult<SchedulingBlock> {
        self.with_conn(move |conn| {
            let row = schedule_blocks::table
                .filter(schedule_blocks::scheduling_block_id.eq(scheduling_block_id))
                .select(ScheduleBlockRow::as_select())
                .first::<ScheduleBlockRow>(conn)
                .map_err(map_diesel_error)?;
            row_to_block(row)
        })
        .await
    }

    async fn get_blocks_for_schedule(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<SchedulingBlock>> {
        self.with_conn(move |conn| {
            let block_rows = schedule_blocks::table
                .filter(schedule_blocks::schedule_id.eq(schedule_id.0))
                .select(ScheduleBlockRow::as_select())
                .order(schedule_blocks::scheduling_block_id.asc())
                .load::<ScheduleBlockRow>(conn)
                .map_err(map_diesel_error)?;

            let mut blocks = Vec::with_capacity(block_rows.len());
            for row in block_rows {
                blocks.push(row_to_block(row)?);
            }
            Ok(blocks)
        })
        .await
    }

    async fn fetch_dark_periods(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<Period>> {
        self.with_conn(move |conn| {
            let dark_periods_json: Value = schedules::table
                .filter(schedules::schedule_id.eq(schedule_id.0))
                .select(schedules::dark_periods_json)
                .first(conn)
                .map_err(map_diesel_error)?;
            value_to_periods(&dark_periods_json)
        })
        .await
    }

    async fn fetch_possible_periods(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<Period>> {
        self.with_conn(move |conn| {
            let possible_json: Value = schedules::table
                .filter(schedules::schedule_id.eq(schedule_id.0))
                .select(schedules::possible_periods_json)
                .first(conn)
                .map_err(map_diesel_error)?;

            let mut possible_periods = value_to_periods(&possible_json)?;
            if possible_periods.is_empty() {
                let visibility_jsons: Vec<Value> = schedule_blocks::table
                    .filter(schedule_blocks::schedule_id.eq(schedule_id.0))
                    .select(schedule_blocks::visibility_periods_json)
                    .load(conn)
                    .map_err(map_diesel_error)?;

                for v in visibility_jsons {
                    possible_periods.extend(value_to_periods(&v)?);
                }
            }

            Ok(possible_periods)
        })
        .await
    }
}

#[async_trait]
impl AnalyticsRepository for PostgresRepository {
    async fn populate_schedule_analytics(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<usize> {
        self.with_conn(move |conn| {
            conn.transaction(|tx| {
                let block_rows = schedule_blocks::table
                    .filter(schedule_blocks::schedule_id.eq(schedule_id.0))
                    .select(ScheduleBlockRow::as_select())
                    .load::<ScheduleBlockRow>(tx)
                    .map_err(map_diesel_error)?;

                if block_rows.is_empty() {
                    // Keep derived tables consistent even for empty schedules.
                    diesel::delete(
                        schedule_validation_results::table
                            .filter(schedule_validation_results::schedule_id.eq(schedule_id.0)),
                    )
                    .execute(tx)
                    .map_err(map_diesel_error)?;
                    return Ok(0);
                }

                let mut analytics_rows: Vec<NewScheduleBlockAnalyticsRow> =
                    Vec::with_capacity(block_rows.len());
                let mut blocks_for_validation: Vec<BlockForValidation> =
                    Vec::with_capacity(block_rows.len());

                for row in &block_rows {
                    let visibility_periods = value_to_periods(&row.visibility_periods_json)?;
                    let total_visibility_hours: f64 = visibility_periods
                        .iter()
                        .map(|p| p.duration().value() * 24.0)
                        .sum();
                    let max_visibility_period_hours: f64 = visibility_periods
                        .iter()
                        .map(|p| p.duration().value() * 24.0)
                        .fold(0.0_f64, |a, b| a.max(b));

                    let scheduled_period = value_to_single_period(&row.scheduled_periods_json)?;
                    let (scheduled, scheduled_start, scheduled_stop) =
                        if let Some(p) = scheduled_period {
                            (true, Some(p.start.value()), Some(p.stop.value()))
                        } else {
                            (false, None, None)
                        };

                    let elevation_range = match (row.min_altitude_deg, row.max_altitude_deg) {
                        (Some(min), Some(max)) => Some(max - min),
                        _ => None,
                    };

                    blocks_for_validation.push(BlockForValidation {
                        schedule_id,
                        scheduling_block_id: row.scheduling_block_id,
                        priority: row.priority,
                        requested_duration_sec: row.requested_duration_sec,
                        min_observation_sec: row.min_observation_sec,
                        total_visibility_hours,
                        max_visibility_period_hours,
                        min_alt_deg: row.min_altitude_deg.map(|d| d.value()),
                        max_alt_deg: row.max_altitude_deg.map(|d| d.value()),
                        constraint_start_mjd: row.constraint_start_mjd,
                        constraint_stop_mjd: row.constraint_stop_mjd,
                        scheduled_start_mjd: scheduled_start,
                        scheduled_stop_mjd: scheduled_stop,
                        target_ra_deg: row.target_ra_deg.value(),
                        target_dec_deg: row.target_dec_deg.value(),
                    });

                    analytics_rows.push(NewScheduleBlockAnalyticsRow {
                        schedule_id: schedule_id.0,
                        scheduling_block_id: row.scheduling_block_id,
                        priority_bucket: priority_bucket(row.priority),
                        requested_hours: qtty::Hours::new(
                            row.requested_duration_sec as f64 / 3600.0,
                        ),
                        total_visibility_hours: qtty::Hours::new(total_visibility_hours),
                        num_visibility_periods: visibility_periods.len() as i32,
                        elevation_range_deg: elevation_range,
                        scheduled,
                        scheduled_start_mjd: scheduled_start,
                        scheduled_stop_mjd: scheduled_stop,
                        validation_impossible: false,
                    });
                }

                // Run validation as part of ETL so the dashboard can show the Validation Report.
                let validation_results: Vec<ValidationResult> =
                    validate_blocks(&blocks_for_validation);
                let impossible_block_ids: HashSet<i64> = validation_results
                    .iter()
                    .filter(|r| matches!(r.status, ValidationStatus::Impossible))
                    .map(|r| r.scheduling_block_id)
                    .collect();

                for row in &mut analytics_rows {
                    row.validation_impossible =
                        impossible_block_ids.contains(&row.scheduling_block_id);
                }

                diesel::insert_into(schedule_block_analytics::table)
                    .values(&analytics_rows)
                    .on_conflict((
                        schedule_block_analytics::schedule_id,
                        schedule_block_analytics::scheduling_block_id,
                    ))
                    .do_update()
                    .set((
                        schedule_block_analytics::priority_bucket
                            .eq(excluded(schedule_block_analytics::priority_bucket)),
                        schedule_block_analytics::requested_hours
                            .eq(excluded(schedule_block_analytics::requested_hours)),
                        schedule_block_analytics::total_visibility_hours
                            .eq(excluded(schedule_block_analytics::total_visibility_hours)),
                        schedule_block_analytics::num_visibility_periods
                            .eq(excluded(schedule_block_analytics::num_visibility_periods)),
                        schedule_block_analytics::elevation_range_deg
                            .eq(excluded(schedule_block_analytics::elevation_range_deg)),
                        schedule_block_analytics::scheduled
                            .eq(excluded(schedule_block_analytics::scheduled)),
                        schedule_block_analytics::scheduled_start_mjd
                            .eq(excluded(schedule_block_analytics::scheduled_start_mjd)),
                        schedule_block_analytics::scheduled_stop_mjd
                            .eq(excluded(schedule_block_analytics::scheduled_stop_mjd)),
                        schedule_block_analytics::validation_impossible
                            .eq(excluded(schedule_block_analytics::validation_impossible)),
                    ))
                    .execute(tx)
                    .map_err(map_diesel_error)?;

                // Persist validation results (one-or-more per block, including "valid").
                diesel::delete(
                    schedule_validation_results::table
                        .filter(schedule_validation_results::schedule_id.eq(schedule_id.0)),
                )
                .execute(tx)
                .map_err(map_diesel_error)?;

                let new_validation_rows: Vec<NewScheduleValidationResultRow> = validation_results
                    .iter()
                    .map(|r| NewScheduleValidationResultRow {
                        schedule_id: r.schedule_id.0,
                        scheduling_block_id: r.scheduling_block_id,
                        status: r.status.as_str().to_string(),
                        issue_type: r.issue_type.clone(),
                        issue_category: r.issue_category.map(|c| c.as_str().to_string()),
                        criticality: r.criticality.map(|c| c.as_str().to_string()),
                        field_name: r.field_name.clone(),
                        current_value: r.current_value.clone(),
                        expected_value: r.expected_value.clone(),
                        description: r.description.clone(),
                    })
                    .collect();

                if !new_validation_rows.is_empty() {
                    diesel::insert_into(schedule_validation_results::table)
                        .values(&new_validation_rows)
                        .execute(tx)
                        .map_err(map_diesel_error)?;
                }

                let summary_row =
                    compute_summary_metrics(schedule_id.0, &block_rows, &analytics_rows);

                diesel::insert_into(schedule_summary_analytics::table)
                    .values(&summary_row)
                    .on_conflict(schedule_summary_analytics::schedule_id)
                    .do_update()
                    .set((
                        schedule_summary_analytics::total_blocks
                            .eq(excluded(schedule_summary_analytics::total_blocks)),
                        schedule_summary_analytics::scheduled_blocks
                            .eq(excluded(schedule_summary_analytics::scheduled_blocks)),
                        schedule_summary_analytics::unscheduled_blocks
                            .eq(excluded(schedule_summary_analytics::unscheduled_blocks)),
                        schedule_summary_analytics::impossible_blocks
                            .eq(excluded(schedule_summary_analytics::impossible_blocks)),
                        schedule_summary_analytics::scheduling_rate
                            .eq(excluded(schedule_summary_analytics::scheduling_rate)),
                        schedule_summary_analytics::priority_mean
                            .eq(excluded(schedule_summary_analytics::priority_mean)),
                        schedule_summary_analytics::priority_median
                            .eq(excluded(schedule_summary_analytics::priority_median)),
                        schedule_summary_analytics::priority_scheduled_mean.eq(excluded(
                            schedule_summary_analytics::priority_scheduled_mean,
                        )),
                        schedule_summary_analytics::priority_unscheduled_mean.eq(excluded(
                            schedule_summary_analytics::priority_unscheduled_mean,
                        )),
                        schedule_summary_analytics::visibility_total_hours
                            .eq(excluded(schedule_summary_analytics::visibility_total_hours)),
                        schedule_summary_analytics::requested_mean_hours
                            .eq(excluded(schedule_summary_analytics::requested_mean_hours)),
                        schedule_summary_analytics::gap_count
                            .eq(excluded(schedule_summary_analytics::gap_count)),
                        schedule_summary_analytics::gap_mean_hours
                            .eq(excluded(schedule_summary_analytics::gap_mean_hours)),
                        schedule_summary_analytics::gap_median_hours
                            .eq(excluded(schedule_summary_analytics::gap_median_hours)),
                    ))
                    .execute(tx)
                    .map_err(map_diesel_error)?;

                Ok(analytics_rows.len())
            })
        })
        .await
    }

    async fn delete_schedule_analytics(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<usize> {
        self.with_conn(move |conn| {
            conn.transaction(|tx| {
                diesel::delete(
                    schedule_summary_analytics::table
                        .filter(schedule_summary_analytics::schedule_id.eq(schedule_id.0)),
                )
                .execute(tx)
                .map_err(map_diesel_error)?;

                let deleted = diesel::delete(
                    schedule_block_analytics::table
                        .filter(schedule_block_analytics::schedule_id.eq(schedule_id.0)),
                )
                .execute(tx)
                .map_err(map_diesel_error)?;

                Ok(deleted)
            })
        })
        .await
    }

    async fn has_analytics_data(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<bool> {
        self.with_conn(move |conn| {
            use diesel::dsl::count_star;
            let count: i64 = schedule_block_analytics::table
                .filter(schedule_block_analytics::schedule_id.eq(schedule_id.0))
                .select(count_star())
                .first(conn)
                .map_err(map_diesel_error)?;
            Ok(count > 0)
        })
        .await
    }

    async fn fetch_analytics_blocks_for_sky_map(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<LightweightBlock>> {
        self.with_conn(move |conn| {
            let rows = schedule_blocks::table
                .inner_join(
                    schedule_block_analytics::table
                        .on(schedule_block_analytics::scheduling_block_id
                            .eq(schedule_blocks::scheduling_block_id)),
                )
                .filter(schedule_blocks::schedule_id.eq(schedule_id.0))
                .select((
                    schedule_blocks::scheduling_block_id,
                    schedule_blocks::source_block_id,
                    schedule_blocks::original_block_id,
                    schedule_blocks::priority,
                    schedule_blocks::requested_duration_sec,
                    schedule_blocks::target_ra_deg,
                    schedule_blocks::target_dec_deg,
                    schedule_block_analytics::scheduled_start_mjd,
                    schedule_block_analytics::scheduled_stop_mjd,
                ))
                .load::<(
                    i64,
                    i64,
                    Option<String>,
                    f64,
                    i32,
                    f64,
                    f64,
                    Option<f64>,
                    Option<f64>,
                )>(conn)
                .map_err(map_diesel_error)?;

            let blocks = rows
                .into_iter()
                .map(
                    |(
                        _block_id,
                        source_block_id,
                        original_block_id,
                        priority,
                        requested_duration_sec,
                        ra,
                        dec,
                        scheduled_start,
                        scheduled_stop,
                    )| {
                        let scheduled_period = match (scheduled_start, scheduled_stop) {
                            (Some(s), Some(e)) => Some(Period {
                                start: ModifiedJulianDate::new(s),
                                stop: ModifiedJulianDate::new(e),
                            }),
                            _ => None,
                        };

                        LightweightBlock {
                            original_block_id: original_block_id
                                .unwrap_or_else(|| source_block_id.to_string()),
                            priority,
                            priority_bin: String::new(),
                            requested_duration_seconds: (requested_duration_sec as f64).into(),
                            target_ra_deg: ra.into(),
                            target_dec_deg: dec.into(),
                            scheduled_period,
                        }
                    },
                )
                .collect();

            Ok(blocks)
        })
        .await
    }

    async fn fetch_analytics_blocks_for_distribution(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<DistributionBlock>> {
        self.with_conn(move |conn| {
            let rows = schedule_blocks::table
                .inner_join(
                    schedule_block_analytics::table
                        .on(schedule_block_analytics::scheduling_block_id
                            .eq(schedule_blocks::scheduling_block_id)),
                )
                .filter(schedule_blocks::schedule_id.eq(schedule_id.0))
                .select((
                    schedule_blocks::priority,
                    schedule_block_analytics::total_visibility_hours,
                    schedule_block_analytics::requested_hours,
                    schedule_block_analytics::elevation_range_deg,
                    schedule_block_analytics::scheduled,
                ))
                .load::<(f64, f64, f64, Option<f64>, bool)>(conn)
                .map_err(map_diesel_error)?;

            let blocks = rows
                .into_iter()
                .map(
                    |(priority, total_vis, requested, elevation, scheduled)| DistributionBlock {
                        priority,
                        total_visibility_hours: total_vis.into(),
                        requested_hours: requested.into(),
                        elevation_range_deg: elevation.unwrap_or(0.0).into(),
                        scheduled,
                    },
                )
                .collect();
            Ok(blocks)
        })
        .await
    }

    async fn fetch_analytics_blocks_for_insights(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<InsightsBlock>> {
        self.with_conn(move |conn| {
            let rows = schedule_blocks::table
                .inner_join(
                    schedule_block_analytics::table
                        .on(schedule_block_analytics::scheduling_block_id
                            .eq(schedule_blocks::scheduling_block_id)),
                )
                .filter(schedule_blocks::schedule_id.eq(schedule_id.0))
                .select((
                    schedule_blocks::scheduling_block_id,
                    schedule_blocks::source_block_id,
                    schedule_blocks::original_block_id,
                    schedule_blocks::priority,
                    schedule_block_analytics::total_visibility_hours,
                    schedule_block_analytics::requested_hours,
                    schedule_block_analytics::elevation_range_deg,
                    schedule_block_analytics::scheduled,
                    schedule_block_analytics::scheduled_start_mjd,
                    schedule_block_analytics::scheduled_stop_mjd,
                ))
                .load::<(
                    i64,
                    i64,
                    Option<String>,
                    f64,
                    f64,
                    f64,
                    Option<f64>,
                    bool,
                    Option<f64>,
                    Option<f64>,
                )>(conn)
                .map_err(map_diesel_error)?;

            let blocks = rows
                .into_iter()
                .map(
                    |(
                        block_id,
                        source_block_id,
                        original_block_id,
                        priority,
                        total_vis_hours,
                        requested_hours,
                        elevation_range,
                        scheduled,
                        start_mjd,
                        stop_mjd,
                    )| InsightsBlock {
                        scheduling_block_id: block_id,
                        original_block_id: original_block_id
                            .unwrap_or_else(|| source_block_id.to_string()),
                        priority,
                        total_visibility_hours: qtty::time::Hours::new(total_vis_hours),
                        requested_hours: qtty::time::Hours::new(requested_hours),
                        elevation_range_deg: qtty::Degrees::new(elevation_range.unwrap_or(0.0)),
                        scheduled,
                        scheduled_start_mjd: start_mjd.map(ModifiedJulianDate::new),
                        scheduled_stop_mjd: stop_mjd.map(ModifiedJulianDate::new),
                    },
                )
                .collect();

            Ok(blocks)
        })
        .await
    }
}

#[async_trait]
impl ValidationRepository for PostgresRepository {
    async fn insert_validation_results(
        &self,
        results: &[ValidationResult],
    ) -> RepositoryResult<usize> {
        let results = results.to_vec();
        self.with_conn(move |conn| {
            conn.transaction(|tx| {
                if results.is_empty() {
                    return Ok(0);
                }

                let schedule_id = results[0].schedule_id.0;

                diesel::delete(
                    schedule_validation_results::table
                        .filter(schedule_validation_results::schedule_id.eq(schedule_id)),
                )
                .execute(tx)
                .map_err(map_diesel_error)?;

                let new_rows: Vec<NewScheduleValidationResultRow> = results
                    .iter()
                    .map(|r| NewScheduleValidationResultRow {
                        schedule_id: r.schedule_id.0,
                        scheduling_block_id: r.scheduling_block_id,
                        status: r.status.as_str().to_string(),
                        issue_type: r.issue_type.clone(),
                        issue_category: r.issue_category.map(|c| c.as_str().to_string()),
                        criticality: r.criticality.map(|c| c.as_str().to_string()),
                        field_name: r.field_name.clone(),
                        current_value: r.current_value.clone(),
                        expected_value: r.expected_value.clone(),
                        description: r.description.clone(),
                    })
                    .collect();

                let inserted = diesel::insert_into(schedule_validation_results::table)
                    .values(&new_rows)
                    .execute(tx)
                    .map_err(map_diesel_error)?;

                // Reset validation flags before marking impossible blocks
                diesel::update(
                    schedule_block_analytics::table
                        .filter(schedule_block_analytics::schedule_id.eq(schedule_id)),
                )
                .set(schedule_block_analytics::validation_impossible.eq(false))
                .execute(tx)
                .map_err(map_diesel_error)?;

                let impossible_block_ids: Vec<i64> = results
                    .iter()
                    .filter(|r| matches!(r.status, ValidationStatus::Impossible))
                    .map(|r| r.scheduling_block_id)
                    .collect();

                if !impossible_block_ids.is_empty() {
                    diesel::update(
                        schedule_block_analytics::table
                            .filter(schedule_block_analytics::schedule_id.eq(schedule_id))
                            .filter(
                                schedule_block_analytics::scheduling_block_id
                                    .eq_any(&impossible_block_ids),
                            ),
                    )
                    .set(schedule_block_analytics::validation_impossible.eq(true))
                    .execute(tx)
                    .map_err(map_diesel_error)?;
                }

                Ok(inserted)
            })
        })
        .await
    }

    async fn fetch_validation_results(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<crate::api::ValidationReport> {
        self.with_conn(move |conn| {
            let block_id_map: std::collections::HashMap<i64, Option<String>> =
                schedule_blocks::table
                    .filter(schedule_blocks::schedule_id.eq(schedule_id.0))
                    .select((
                        schedule_blocks::scheduling_block_id,
                        schedule_blocks::original_block_id,
                    ))
                    .load::<(i64, Option<String>)>(conn)
                    .map_err(map_diesel_error)?
                    .into_iter()
                    .collect();

            let mut rows = schedule_validation_results::table
                .filter(schedule_validation_results::schedule_id.eq(schedule_id.0))
                .select(ScheduleValidationResultRow::as_select())
                .order(schedule_validation_results::validation_id.asc())
                .load::<ScheduleValidationResultRow>(conn)
                .map_err(map_diesel_error)?;

            if rows.is_empty() {
                // If validation hasn't been populated yet, return an "all valid" empty report
                // instead of hard failing the UI.
                let total_blocks = block_id_map.len();
                return Ok(crate::api::ValidationReport {
                    schedule_id,
                    total_blocks,
                    valid_blocks: total_blocks,
                    impossible_blocks: Vec::new(),
                    validation_errors: Vec::new(),
                    validation_warnings: Vec::new(),
                });
            }

            let mut impossible_blocks = Vec::new();
            let mut validation_errors = Vec::new();
            let mut validation_warnings = Vec::new();
            let mut valid_blocks = 0usize;

            for row in rows.drain(..) {
                let original_block_id = block_id_map
                    .get(&row.scheduling_block_id)
                    .cloned()
                    .unwrap_or(None);

                let issue = crate::api::ValidationIssue {
                    block_id: row.scheduling_block_id,
                    original_block_id,
                    issue_type: row.issue_type.unwrap_or_default(),
                    category: row.issue_category.unwrap_or_default(),
                    criticality: row.criticality.unwrap_or_default(),
                    field_name: row.field_name.clone(),
                    current_value: row.current_value.clone(),
                    expected_value: row.expected_value.clone(),
                    description: row.description.unwrap_or_default(),
                };

                match row.status.as_str() {
                    s if s == ValidationStatus::Valid.as_str() => {
                        valid_blocks += 1;
                    }
                    s if s == ValidationStatus::Impossible.as_str() => {
                        impossible_blocks.push(issue);
                    }
                    s if s == ValidationStatus::Warning.as_str() => {
                        validation_warnings.push(issue);
                    }
                    _ => {
                        validation_errors.push(issue);
                    }
                }
            }

            let total_blocks = block_id_map.len();

            Ok(crate::api::ValidationReport {
                schedule_id,
                total_blocks,
                valid_blocks,
                impossible_blocks,
                validation_errors,
                validation_warnings,
            })
        })
        .await
    }

    async fn has_validation_results(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<bool> {
        self.with_conn(move |conn| {
            use diesel::dsl::count_star;
            let count: i64 = schedule_validation_results::table
                .filter(schedule_validation_results::schedule_id.eq(schedule_id.0))
                .select(count_star())
                .first(conn)
                .map_err(map_diesel_error)?;
            Ok(count > 0)
        })
        .await
    }

    async fn delete_validation_results(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<u64> {
        self.with_conn(move |conn| {
            let deleted = diesel::delete(
                schedule_validation_results::table
                    .filter(schedule_validation_results::schedule_id.eq(schedule_id.0)),
            )
            .execute(conn)
            .map_err(map_diesel_error)?;
            Ok(deleted as u64)
        })
        .await
    }
}

#[async_trait]
impl VisualizationRepository for PostgresRepository {
    async fn fetch_schedule_timeline_blocks(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<ScheduleTimelineBlock>> {
        self.with_conn(move |conn| {
            let rows = schedule_blocks::table
                .inner_join(
                    schedule_block_analytics::table
                        .on(schedule_block_analytics::scheduling_block_id
                            .eq(schedule_blocks::scheduling_block_id)),
                )
                .filter(schedule_blocks::schedule_id.eq(schedule_id.0))
                .filter(schedule_block_analytics::scheduled.eq(true))
                .select((
                    schedule_blocks::scheduling_block_id,
                    schedule_blocks::source_block_id,
                    schedule_blocks::original_block_id,
                    schedule_blocks::priority,
                    schedule_block_analytics::scheduled_start_mjd,
                    schedule_block_analytics::scheduled_stop_mjd,
                    schedule_blocks::target_ra_deg,
                    schedule_blocks::target_dec_deg,
                    schedule_block_analytics::requested_hours,
                    schedule_block_analytics::total_visibility_hours,
                    schedule_block_analytics::num_visibility_periods,
                ))
                .load::<(
                    i64,
                    i64,
                    Option<String>,
                    f64,
                    Option<f64>,
                    Option<f64>,
                    f64,
                    f64,
                    f64,
                    f64,
                    i32,
                )>(conn)
                .map_err(map_diesel_error)?;

            let blocks = rows
                .into_iter()
                .filter_map(
                    |(
                        block_id,
                        source_block_id,
                        original_block_id,
                        priority,
                        start,
                        stop,
                        ra_deg,
                        dec_deg,
                        requested_hours,
                        total_visibility_hours,
                        num_visibility_periods,
                    )| {
                        let scheduled_start_mjd = start?;
                        let scheduled_stop_mjd = stop?;
                        Some(ScheduleTimelineBlock {
                            scheduling_block_id: block_id,
                            original_block_id: original_block_id
                                .unwrap_or_else(|| source_block_id.to_string()),
                            priority,
                            scheduled_start_mjd: ModifiedJulianDate::new(scheduled_start_mjd),
                            scheduled_stop_mjd: ModifiedJulianDate::new(scheduled_stop_mjd),
                            ra_deg: qtty::Degrees::new(ra_deg),
                            dec_deg: qtty::Degrees::new(dec_deg),
                            requested_hours: qtty::time::Hours::new(requested_hours),
                            total_visibility_hours: qtty::time::Hours::new(total_visibility_hours),
                            num_visibility_periods: num_visibility_periods as usize,
                        })
                    },
                )
                .collect();

            Ok(blocks)
        })
        .await
    }

    async fn fetch_compare_blocks(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<Vec<CompareBlock>> {
        self.with_conn(move |conn| {
            let rows = schedule_blocks::table
                .inner_join(
                    schedule_block_analytics::table
                        .on(schedule_block_analytics::scheduling_block_id
                            .eq(schedule_blocks::scheduling_block_id)),
                )
                .filter(schedule_blocks::schedule_id.eq(schedule_id.0))
                .select((
                    schedule_blocks::scheduling_block_id,
                    schedule_blocks::priority,
                    schedule_block_analytics::scheduled,
                    schedule_block_analytics::requested_hours,
                ))
                .load::<(i64, f64, bool, f64)>(conn)
                .map_err(map_diesel_error)?;

            let blocks = rows
                .into_iter()
                .map(
                    |(block_id, priority, scheduled, requested_hours)| CompareBlock {
                        scheduling_block_id: block_id.to_string(),
                        priority,
                        scheduled,
                        requested_hours: requested_hours.into(),
                    },
                )
                .collect();

            Ok(blocks)
        })
        .await
    }

    async fn fetch_visibility_map_data(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<VisibilityMapData> {
        self.with_conn(move |conn| {
            let rows = schedule_blocks::table
                .inner_join(
                    schedule_block_analytics::table
                        .on(schedule_block_analytics::scheduling_block_id
                            .eq(schedule_blocks::scheduling_block_id)),
                )
                .filter(schedule_blocks::schedule_id.eq(schedule_id.0))
                .select((
                    schedule_blocks::scheduling_block_id,
                    schedule_blocks::source_block_id,
                    schedule_blocks::original_block_id,
                    schedule_blocks::priority,
                    schedule_block_analytics::num_visibility_periods,
                    schedule_block_analytics::scheduled,
                ))
                .load::<(i64, i64, Option<String>, f64, i32, bool)>(conn)
                .map_err(map_diesel_error)?;

            if rows.is_empty() {
                return Ok(VisibilityMapData {
                    blocks: vec![],
                    priority_min: 0.0,
                    priority_max: 1.0,
                    total_count: 0,
                    scheduled_count: 0,
                });
            }

            let mut priority_min = f64::MAX;
            let mut priority_max = f64::MIN;
            let mut scheduled_count = 0usize;

            let blocks: Vec<VisibilityBlockSummary> = rows
                .into_iter()
                .map(
                    |(
                        block_id,
                        source_block_id,
                        original_block_id,
                        priority,
                        num_periods,
                        scheduled,
                    )| {
                        priority_min = priority_min.min(priority);
                        priority_max = priority_max.max(priority);
                        if scheduled {
                            scheduled_count += 1;
                        }

                        VisibilityBlockSummary {
                            scheduling_block_id: block_id,
                            original_block_id: original_block_id
                                .unwrap_or_else(|| source_block_id.to_string()),
                            priority,
                            num_visibility_periods: num_periods as usize,
                            scheduled,
                        }
                    },
                )
                .collect();
            let total_count = blocks.len();

            Ok(VisibilityMapData {
                blocks,
                priority_min,
                priority_max,
                total_count,
                scheduled_count,
            })
        })
        .await
    }

    async fn fetch_blocks_for_histogram(
        &self,
        schedule_id: crate::api::ScheduleId,
        priority_min: Option<i32>,
        priority_max: Option<i32>,
        block_ids: Option<Vec<i64>>,
    ) -> RepositoryResult<Vec<crate::db::models::BlockHistogramData>> {
        self.with_conn(move |conn| {
            let mut query = schedule_blocks::table
                .filter(schedule_blocks::schedule_id.eq(schedule_id.0))
                .into_boxed();

            if let Some(ref ids) = block_ids {
                query = query.filter(schedule_blocks::scheduling_block_id.eq_any(ids));
            }
            if let Some(min_p) = priority_min {
                query = query.filter(schedule_blocks::priority.ge(min_p as f64));
            }
            if let Some(max_p) = priority_max {
                query = query.filter(schedule_blocks::priority.le(max_p as f64));
            }

            let rows = query
                .select((
                    schedule_blocks::scheduling_block_id,
                    schedule_blocks::priority,
                    schedule_blocks::visibility_periods_json,
                ))
                .load::<(i64, f64, Value)>(conn)
                .map_err(map_diesel_error)?;

            let blocks = rows
                .into_iter()
                .map(|(block_id, priority, visibility_json)| {
                    let visibility_periods = value_to_periods(&visibility_json).ok();
                    crate::db::models::BlockHistogramData {
                        scheduling_block_id: block_id,
                        priority: priority as i32,
                        visibility_periods,
                    }
                })
                .collect();

            Ok(blocks)
        })
        .await
    }

    async fn fetch_gap_metrics(
        &self,
        schedule_id: crate::api::ScheduleId,
    ) -> RepositoryResult<(Option<i32>, Option<qtty::Hours>, Option<qtty::Hours>)> {
        self.with_conn(move |conn| {
            let result = schedule_summary_analytics::table
                .filter(schedule_summary_analytics::schedule_id.eq(schedule_id.0))
                .select((
                    schedule_summary_analytics::gap_count,
                    schedule_summary_analytics::gap_mean_hours,
                    schedule_summary_analytics::gap_median_hours,
                ))
                .first::<(Option<i32>, Option<f64>, Option<f64>)>(conn)
                .optional()
                .map_err(map_diesel_error)?;

            match result {
                Some((gap_count, gap_mean, gap_median)) => Ok((
                    gap_count,
                    gap_mean.map(qtty::Hours::new),
                    gap_median.map(qtty::Hours::new),
                )),
                None => Ok((None, None, None)),
            }
        })
        .await
    }
}
