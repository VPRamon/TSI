//! Postgres repository implementation using Diesel.
//!
//! This module implements the repository traits against a Postgres database
//! following the schema defined in `docs/POSTGRES_ETL_DB_DESIGN.md`.

use async_trait::async_trait;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_query;
use diesel::upsert::excluded;
use diesel_migrations::{FileBasedMigrations, MigrationHarness};
use serde_json::{json, Value};
use tokio::task;

use crate::api::{
    CompareBlock, Constraints, DistributionBlock, InsightsBlock, LightweightBlock,
    ModifiedJulianDate, Period, Schedule, ScheduleId, ScheduleInfo, ScheduleTimelineBlock,
    SchedulingBlock, VisibilityBlockSummary, VisibilityMapData,
};
use crate::db::repository::{
    AnalyticsRepository, RepositoryError, RepositoryResult, ScheduleRepository,
    ValidationRepository, VisualizationRepository,
};
use crate::services::validation::{ValidationResult, ValidationStatus};

mod models;
mod schema;

use models::*;
use schema::*;

type PgPool = Pool<ConnectionManager<PgConnection>>;

/// Configuration for connecting to Postgres.
#[derive(Debug, Clone)]
pub struct PostgresConfig {
    pub database_url: String,
    pub max_pool_size: u32,
}

impl PostgresConfig {
    pub fn from_env() -> Result<Self, String> {
        let database_url = std::env::var("DATABASE_URL")
            .or_else(|_| std::env::var("PG_DATABASE_URL"))
            .map_err(|_| "DATABASE_URL or PG_DATABASE_URL must be set".to_string())?;

        let max_pool_size = std::env::var("PG_POOL_MAX")
            .ok()
            .and_then(|v| v.parse::<u32>().ok())
            .unwrap_or(10);

        Ok(Self {
            database_url,
            max_pool_size,
        })
    }
}

/// Diesel-backed repository for Postgres.
#[derive(Clone)]
pub struct PostgresRepository {
    pool: PgPool,
}

impl PostgresRepository {
    /// Create a new repository and run pending migrations.
    pub fn new(config: PostgresConfig) -> RepositoryResult<Self> {
        let manager = ConnectionManager::<PgConnection>::new(config.database_url);
        let pool = Pool::builder()
            .max_size(config.max_pool_size)
            .build(manager)
            .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;

        // Run migrations once during initialization.
        {
            let mut conn = pool
                .get()
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            Self::run_migrations(&mut conn)?;
        }

        Ok(Self { pool })
    }

    fn run_migrations(conn: &mut PgConnection) -> RepositoryResult<()> {
        let migrations =
            FileBasedMigrations::from_path(format!("{}/migrations", env!("CARGO_MANIFEST_DIR")))
                .map_err(|e| {
                    RepositoryError::InternalError(format!("Migrations not found: {e}"))
                })?;

        conn.run_pending_migrations(migrations)
            .map_err(|e| RepositoryError::InternalError(format!("Migration error: {e}")))?;
        Ok(())
    }

    async fn with_conn<T, F>(&self, f: F) -> RepositoryResult<T>
    where
        T: Send + 'static,
        F: FnOnce(&mut PgConnection) -> RepositoryResult<T> + Send + 'static,
    {
        let pool = self.pool.clone();
        task::spawn_blocking(move || {
            let mut conn = pool
                .get()
                .map_err(|e| RepositoryError::ConnectionError(e.to_string()))?;
            f(&mut conn)
        })
        .await
        .map_err(|e| RepositoryError::InternalError(e.to_string()))?
    }
}

fn map_diesel_error(err: diesel::result::Error) -> RepositoryError {
    match err {
        diesel::result::Error::NotFound => RepositoryError::NotFound("Record not found".into()),
        other => RepositoryError::QueryError(other.to_string()),
    }
}

fn periods_to_json(periods: &[Period]) -> Value {
    serde_json::to_value(periods).unwrap_or_else(|_| json!([]))
}

fn scheduled_period_to_json(period: &Option<Period>) -> Value {
    match period {
        Some(p) => periods_to_json(&[p.clone()]),
        None => json!([]),
    }
}

fn value_to_periods(value: &Value) -> RepositoryResult<Vec<Period>> {
    serde_json::from_value(value.clone())
        .map_err(|e| RepositoryError::InternalError(format!("Failed to parse period JSON: {e}")))
}

fn value_to_single_period(value: &Value) -> RepositoryResult<Option<Period>> {
    let mut periods: Vec<Period> = value_to_periods(value)?;
    Ok(periods.pop())
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
        min_alt: row.min_altitude_deg.unwrap_or(0.0),
        max_alt: row.max_altitude_deg.unwrap_or(0.0),
        min_az: row.min_azimuth_deg.unwrap_or(0.0),
        max_az: row.max_azimuth_deg.unwrap_or(0.0),
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
        id: row.scheduling_block_id,
        original_block_id: row
            .original_block_id
            .or_else(|| Some(row.source_block_id.to_string())),
        target_ra: row.target_ra_deg,
        target_dec: row.target_dec_deg,
        constraints,
        priority: row.priority,
        min_observation: row.min_observation_sec as f64,
        requested_duration: row.requested_duration_sec as f64,
        visibility_periods,
        scheduled_period,
    })
}

fn build_schedule_from_rows(
    schedule_row: ScheduleRow,
    block_rows: Vec<ScheduleBlockRow>,
) -> RepositoryResult<Schedule> {
    let dark_periods = value_to_periods(&schedule_row.dark_periods_json)?;
    let mut blocks = Vec::with_capacity(block_rows.len());
    for row in block_rows {
        blocks.push(row_to_block(row)?);
    }

    Ok(Schedule {
        id: Some(schedule_row.schedule_id),
        name: schedule_row.schedule_name,
        checksum: schedule_row.checksum,
        dark_periods,
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
        let median = if sorted.len() % 2 == 0 {
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

    let visibility_total_hours = analytics_rows
        .iter()
        .map(|r| r.total_visibility_hours)
        .sum::<f64>();
    let requested_mean_hours = if analytics_rows.is_empty() {
        None
    } else {
        Some(
            analytics_rows
                .iter()
                .map(|r| r.requested_hours)
                .sum::<f64>()
                / analytics_rows.len() as f64,
        )
    };

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
    }
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
                };

                let inserted: ScheduleRow = diesel::insert_into(schedules::table)
                    .values(&new_schedule)
                    .returning(ScheduleRow::as_returning())
                    .get_result(tx)
                    .map_err(map_diesel_error)?;

                let block_rows: Vec<NewScheduleBlockRow> = schedule
                    .blocks
                    .iter()
                    .map(|b| NewScheduleBlockRow {
                        schedule_id: inserted.schedule_id,
                        source_block_id: b.id,
                        original_block_id: b.original_block_id.clone(),
                        priority: b.priority,
                        requested_duration_sec: b.requested_duration as i32,
                        min_observation_sec: b.min_observation as i32,
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
                    diesel::insert_into(schedule_blocks::table)
                        .values(&block_rows)
                        .execute(tx)
                        .map_err(map_diesel_error)?;
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
            let dark_periods_json: Value = schedules::table
                .filter(schedules::schedule_id.eq(schedule_id.0))
                .select(schedules::dark_periods_json)
                .first(conn)
                .map_err(map_diesel_error)?;

            let dark_periods = value_to_periods(&dark_periods_json)?;
            if dark_periods.is_empty() {
                return Ok(None);
            }

            let min_start = dark_periods
                .iter()
                .map(|p| p.start.value())
                .fold(f64::INFINITY, f64::min);
            let max_stop = dark_periods
                .iter()
                .map(|p| p.stop.value())
                .fold(f64::NEG_INFINITY, f64::max);

            Ok(Some(Period {
                start: ModifiedJulianDate::new(min_start),
                stop: ModifiedJulianDate::new(max_stop),
            }))
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
                    return Ok(0);
                }

                let mut analytics_rows: Vec<NewScheduleBlockAnalyticsRow> =
                    Vec::with_capacity(block_rows.len());

                for row in &block_rows {
                    let visibility_periods = value_to_periods(&row.visibility_periods_json)?;
                    let total_visibility_hours: f64 = visibility_periods
                        .iter()
                        .map(|p| p.duration().value() * 24.0)
                        .sum();

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

                    analytics_rows.push(NewScheduleBlockAnalyticsRow {
                        schedule_id: schedule_id.0,
                        scheduling_block_id: row.scheduling_block_id,
                        priority_bucket: priority_bucket(row.priority),
                        requested_hours: row.requested_duration_sec as f64 / 3600.0,
                        total_visibility_hours,
                        num_visibility_periods: visibility_periods.len() as i32,
                        elevation_range_deg: elevation_range,
                        scheduled,
                        scheduled_start_mjd: scheduled_start,
                        scheduled_stop_mjd: scheduled_stop,
                        validation_impossible: false,
                    });
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
                            requested_duration_seconds: requested_duration_sec as f64,
                            target_ra_deg: ra,
                            target_dec_deg: dec,
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
                        total_visibility_hours: total_vis,
                        requested_hours: requested,
                        elevation_range_deg: elevation.unwrap_or(0.0),
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
            let mut rows = schedule_validation_results::table
                .filter(schedule_validation_results::schedule_id.eq(schedule_id.0))
                .select(ScheduleValidationResultRow::as_select())
                .order(schedule_validation_results::validation_id.asc())
                .load::<ScheduleValidationResultRow>(conn)
                .map_err(map_diesel_error)?;

            if rows.is_empty() {
                return Err(RepositoryError::NotFound(format!(
                    "No validation results for schedule {}",
                    schedule_id
                )));
            }

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
                        requested_hours,
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
}
