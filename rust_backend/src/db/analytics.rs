//! Analytics ETL module for pre-computing and denormalizing schedule data.
//!
//! This module implements the ETL (Extract, Transform, Load) logic that populates
//! the `analytics.schedule_blocks_analytics` table after schedule uploads.
//!
//! The analytics table pre-computes:
//! - Priority buckets (quartiles based on schedule's priority range)
//! - Total visibility hours (parsed from visibility_periods_json)
//! - Denormalized fields from targets, constraints, and scheduling blocks
//!
//! This eliminates expensive JOINs and JSON parsing on every page load.

use log::{debug, info, warn};
use tiberius::Query;

use super::pool;

/// Populate the analytics table for a schedule.
///
/// This function:
/// 1. Deletes existing analytics rows for the schedule (idempotent)
/// 2. Computes priority range for bucket calculation
/// 3. Extracts and transforms data from normalized tables
/// 4. Bulk inserts into analytics.schedule_blocks_analytics
///
/// # Arguments
/// * `schedule_id` - The ID of the schedule to process
///
/// # Returns
/// * `Ok(usize)` - Number of rows inserted
/// * `Err(String)` - Error description if the operation fails
pub async fn populate_schedule_analytics(schedule_id: i64) -> Result<usize, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    info!("Populating analytics table for schedule_id={}", schedule_id);

    // First, delete existing analytics for this schedule
    let delete_sql = "DELETE FROM analytics.schedule_blocks_analytics WHERE schedule_id = @P1";
    let mut delete_query = Query::new(delete_sql);
    delete_query.bind(schedule_id);

    let delete_result = delete_query
        .execute(&mut *conn)
        .await
        .map_err(|e| format!("Failed to delete existing analytics: {e}"))?;

    let deleted_count = delete_result.rows_affected().iter().sum::<u64>();
    if deleted_count > 0 {
        debug!(
            "Deleted {} existing analytics rows for schedule_id={}",
            deleted_count, schedule_id
        );
    }

    // Compute priority range for bucket calculation
    let priority_range_sql = r#"
        SELECT 
            MIN(sb.priority) as priority_min,
            MAX(sb.priority) as priority_max
        FROM dbo.schedule_scheduling_blocks ssb
        JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
        WHERE ssb.schedule_id = @P1
    "#;

    let mut range_query = Query::new(priority_range_sql);
    range_query.bind(schedule_id);

    let range_stream = range_query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to query priority range: {e}"))?;

    let range_row = range_stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read priority range: {e}"))?;

    let (priority_min, priority_max) = match range_row {
        Some(row) => {
            let min_val: Option<f64> = row.get(0);
            let max_val: Option<f64> = row.get(1);
            (min_val.unwrap_or(0.0), max_val.unwrap_or(10.0))
        }
        None => {
            warn!("No blocks found for schedule_id={}", schedule_id);
            return Ok(0);
        }
    };

    let priority_range = if (priority_max - priority_min).abs() < f64::EPSILON {
        None // Single value, will use bucket 2
    } else {
        Some(priority_max - priority_min)
    };

    debug!(
        "Priority range for schedule_id={}: min={}, max={}, range={:?}",
        schedule_id, priority_min, priority_max, priority_range
    );

    // Fetch all blocks with denormalized data
    let fetch_sql = r#"
        SELECT 
            ssb.schedule_id,
            sb.scheduling_block_id,
            t.ra_deg,
            t.dec_deg,
            sb.priority,
            sb.requested_duration_sec,
            sb.min_observation_sec,
            ac.min_alt_deg,
            ac.max_alt_deg,
            azc.min_az_deg,
            azc.max_az_deg,
            c.fixed_time_start_mjd,
            c.fixed_time_stop_mjd,
            ssb.start_time_mjd,
            ssb.stop_time_mjd,
            sb.visibility_periods_json
        FROM dbo.schedule_scheduling_blocks ssb
        JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
        JOIN dbo.targets t ON sb.target_id = t.target_id
        LEFT JOIN dbo.constraints c ON sb.constraints_id = c.constraints_id
        LEFT JOIN dbo.altitude_constraints ac ON c.altitude_constraints_id = ac.altitude_constraints_id
        LEFT JOIN dbo.azimuth_constraints azc ON c.azimuth_constraints_id = azc.azimuth_constraints_id
        WHERE ssb.schedule_id = @P1
        ORDER BY sb.scheduling_block_id
    "#;

    let mut fetch_query = Query::new(fetch_sql);
    fetch_query.bind(schedule_id);

    let fetch_stream = fetch_query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch blocks for analytics: {e}"))?;

    let rows = fetch_stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read blocks: {e}"))?;

    if rows.is_empty() {
        info!(
            "No blocks found for schedule_id={}, analytics table empty",
            schedule_id
        );
        return Ok(0);
    }

    // Process rows and prepare batch insert
    let mut analytics_rows: Vec<AnalyticsRow> = Vec::with_capacity(rows.len());

    for row in &rows {
        let schedule_id: i64 = row.get::<i64, _>(0).unwrap_or(0);
        let scheduling_block_id: i64 = row.get::<i64, _>(1).unwrap_or(0);
        let ra_deg: f64 = row.get::<f64, _>(2).unwrap_or(0.0);
        let dec_deg: f64 = row.get::<f64, _>(3).unwrap_or(0.0);
        let priority: f64 = row.get::<f64, _>(4).unwrap_or(0.0);
        let requested_duration_sec: i32 = row.get::<i32, _>(5).unwrap_or(0);
        let min_observation_sec: i32 = row.get::<i32, _>(6).unwrap_or(0);
        let min_alt_deg: Option<f64> = row.get(7);
        let max_alt_deg: Option<f64> = row.get(8);
        let min_az_deg: Option<f64> = row.get(9);
        let max_az_deg: Option<f64> = row.get(10);
        let fixed_time_start: Option<f64> = row.get(11);
        let fixed_time_stop: Option<f64> = row.get(12);
        let scheduled_start: Option<f64> = row.get(13);
        let scheduled_stop: Option<f64> = row.get(14);
        let visibility_json: Option<&str> = row.get(15);

        // Compute priority bucket
        let priority_bucket = compute_priority_bucket(priority, priority_min, priority_range);

        // Parse visibility JSON
        let (total_visibility_hours, visibility_period_count) =
            parse_visibility_periods(visibility_json);

        let is_scheduled = scheduled_start.is_some() && scheduled_stop.is_some();

        analytics_rows.push(AnalyticsRow {
            schedule_id,
            scheduling_block_id,
            target_ra_deg: ra_deg,
            target_dec_deg: dec_deg,
            priority,
            priority_bucket,
            requested_duration_sec,
            min_observation_sec,
            min_altitude_deg: min_alt_deg,
            max_altitude_deg: max_alt_deg,
            min_azimuth_deg: min_az_deg,
            max_azimuth_deg: max_az_deg,
            fixed_time_start_mjd: fixed_time_start,
            fixed_time_stop_mjd: fixed_time_stop,
            is_scheduled,
            scheduled_start_mjd: scheduled_start,
            scheduled_stop_mjd: scheduled_stop,
            total_visibility_hours,
            visibility_period_count,
        });
    }

    // Bulk insert analytics rows
    let inserted = bulk_insert_analytics(&mut conn, &analytics_rows).await?;

    info!(
        "Populated {} analytics rows for schedule_id={}",
        inserted, schedule_id
    );

    Ok(inserted)
}

/// Delete analytics data for a schedule.
///
/// # Arguments
/// * `schedule_id` - The ID of the schedule whose analytics should be deleted
///
/// # Returns
/// * `Ok(usize)` - Number of rows deleted
/// * `Err(String)` - Error description if the operation fails
pub async fn delete_schedule_analytics(schedule_id: i64) -> Result<usize, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = "DELETE FROM analytics.schedule_blocks_analytics WHERE schedule_id = @P1";
    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let result = query
        .execute(&mut *conn)
        .await
        .map_err(|e| format!("Failed to delete analytics: {e}"))?;

    let deleted = result.rows_affected().iter().sum::<u64>() as usize;
    info!(
        "Deleted {} analytics rows for schedule_id={}",
        deleted, schedule_id
    );

    Ok(deleted)
}

/// Compute priority bucket (1-4) based on quartiles.
fn compute_priority_bucket(priority: f64, priority_min: f64, priority_range: Option<f64>) -> u8 {
    match priority_range {
        None => 2, // Single value = medium
        Some(range) => {
            let normalized = (priority - priority_min) / range;
            if normalized >= 0.75 {
                4 // High
            } else if normalized >= 0.50 {
                3 // Medium-High
            } else if normalized >= 0.25 {
                2 // Medium-Low
            } else {
                1 // Low
            }
        }
    }
}

/// Parse visibility periods JSON to compute total hours and period count.
fn parse_visibility_periods(json: Option<&str>) -> (f64, i32) {
    let Some(json_str) = json else {
        return (0.0, 0);
    };

    if json_str.is_empty() || json_str == "null" {
        return (0.0, 0);
    }

    match serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
        Ok(periods) => {
            let count = periods.len() as i32;
            let total_hours = periods.iter().fold(0.0, |acc, period| {
                let start = period["start"].as_f64().unwrap_or(0.0);
                let stop = period["stop"].as_f64().unwrap_or(0.0);
                let duration_days = stop - start;
                acc + (duration_days * 24.0)
            });
            (total_hours, count)
        }
        Err(_) => (0.0, 0),
    }
}

/// Internal struct for holding analytics row data during processing.
struct AnalyticsRow {
    schedule_id: i64,
    scheduling_block_id: i64,
    target_ra_deg: f64,
    target_dec_deg: f64,
    priority: f64,
    priority_bucket: u8,
    requested_duration_sec: i32,
    min_observation_sec: i32,
    min_altitude_deg: Option<f64>,
    max_altitude_deg: Option<f64>,
    min_azimuth_deg: Option<f64>,
    max_azimuth_deg: Option<f64>,
    fixed_time_start_mjd: Option<f64>,
    fixed_time_stop_mjd: Option<f64>,
    is_scheduled: bool,
    scheduled_start_mjd: Option<f64>,
    scheduled_stop_mjd: Option<f64>,
    total_visibility_hours: f64,
    visibility_period_count: i32,
}

type DbClient = tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>;

/// Bulk insert analytics rows in batches.
async fn bulk_insert_analytics(conn: &mut DbClient, rows: &[AnalyticsRow]) -> Result<usize, String> {
    if rows.is_empty() {
        return Ok(0);
    }

    // SQL Server parameter limit: ~2100 params
    // Each row uses 18 params, so max batch = 2100/18 ≈ 116
    // Use 100 for safety margin
    const BATCH_SIZE: usize = 100;

    let mut total_inserted = 0;

    for chunk in rows.chunks(BATCH_SIZE) {
        let mut values_clauses = Vec::with_capacity(chunk.len());
        for (i, _) in chunk.iter().enumerate() {
            let base = i * 18;
            values_clauses.push(format!(
                "(@P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{})",
                base + 1, base + 2, base + 3, base + 4, base + 5, base + 6,
                base + 7, base + 8, base + 9, base + 10, base + 11, base + 12,
                base + 13, base + 14, base + 15, base + 16, base + 17, base + 18
            ));
        }

        let sql = format!(
            r#"
            INSERT INTO analytics.schedule_blocks_analytics (
                schedule_id,
                scheduling_block_id,
                target_ra_deg,
                target_dec_deg,
                priority,
                priority_bucket,
                requested_duration_sec,
                min_observation_sec,
                min_altitude_deg,
                max_altitude_deg,
                min_azimuth_deg,
                max_azimuth_deg,
                fixed_time_start_mjd,
                fixed_time_stop_mjd,
                is_scheduled,
                scheduled_start_mjd,
                scheduled_stop_mjd,
                total_visibility_hours,
                visibility_period_count
            ) VALUES {}
            "#,
            values_clauses.join(", ")
        );

        let mut insert = Query::new(sql);

        for row in chunk {
            insert.bind(row.schedule_id);
            insert.bind(row.scheduling_block_id);
            insert.bind(row.target_ra_deg);
            insert.bind(row.target_dec_deg);
            insert.bind(row.priority);
            insert.bind(row.priority_bucket as i16); // TINYINT
            insert.bind(row.requested_duration_sec);
            insert.bind(row.min_observation_sec);
            insert.bind(row.min_altitude_deg);
            insert.bind(row.max_altitude_deg);
            insert.bind(row.min_azimuth_deg);
            insert.bind(row.max_azimuth_deg);
            insert.bind(row.fixed_time_start_mjd);
            insert.bind(row.fixed_time_stop_mjd);
            insert.bind(row.is_scheduled);
            insert.bind(row.scheduled_start_mjd);
            insert.bind(row.scheduled_stop_mjd);
            insert.bind(row.total_visibility_hours);
            insert.bind(row.visibility_period_count);
        }

        let result = insert
            .execute(&mut *conn)
            .await
            .map_err(|e| format!("Failed to bulk insert analytics: {e}"))?;

        total_inserted += result.rows_affected().iter().sum::<u64>() as usize;
    }

    Ok(total_inserted)
}

/// Fetch lightweight blocks from the analytics table for Sky Map.
/// This replaces the join-based fetch_lightweight_blocks function.
pub async fn fetch_analytics_blocks_for_sky_map(
    schedule_id: i64,
) -> Result<Vec<super::models::LightweightBlock>, String> {
    use super::models::LightweightBlock;
    use siderust::astro::ModifiedJulianDate;

    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = r#"
        SELECT 
            scheduling_block_id,
            priority,
            requested_duration_sec,
            target_ra_deg,
            target_dec_deg,
            scheduled_start_mjd,
            scheduled_stop_mjd
        FROM analytics.schedule_blocks_analytics
        WHERE schedule_id = @P1
        ORDER BY scheduling_block_id
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch analytics blocks: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read analytics blocks: {e}"))?;

    let mut blocks = Vec::with_capacity(rows.len());

    for row in rows {
        let id: i64 = row
            .get::<i64, _>(0)
            .ok_or_else(|| "scheduling_block_id is NULL".to_string())?;
        let priority: f64 = row
            .get::<f64, _>(1)
            .ok_or_else(|| "priority is NULL".to_string())?;
        let requested_duration: i32 = row
            .get::<i32, _>(2)
            .ok_or_else(|| "requested_duration_sec is NULL".to_string())?;
        let ra: f64 = row
            .get::<f64, _>(3)
            .ok_or_else(|| "target_ra_deg is NULL".to_string())?;
        let dec: f64 = row
            .get::<f64, _>(4)
            .ok_or_else(|| "target_dec_deg is NULL".to_string())?;

        let scheduled_period = match (row.get::<f64, _>(5), row.get::<f64, _>(6)) {
            (Some(start_mjd), Some(stop_mjd)) => super::models::Period::new(
                ModifiedJulianDate::new(start_mjd),
                ModifiedJulianDate::new(stop_mjd),
            ),
            _ => None,
        };

        blocks.push(LightweightBlock {
            id: super::models::SchedulingBlockId(id),
            priority,
            priority_bin: String::new(), // Will be computed by service layer
            requested_duration_seconds: requested_duration as f64,
            target_ra_deg: ra,
            target_dec_deg: dec,
            scheduled_period,
        });
    }

    Ok(blocks)
}

/// Fetch distribution blocks from the analytics table.
/// This replaces the join-based fetch_distribution_blocks function.
pub async fn fetch_analytics_blocks_for_distribution(
    schedule_id: i64,
) -> Result<Vec<super::models::DistributionBlock>, String> {
    use super::models::DistributionBlock;

    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = r#"
        SELECT 
            priority,
            total_visibility_hours,
            requested_hours,
            elevation_range_deg,
            is_scheduled
        FROM analytics.schedule_blocks_analytics
        WHERE schedule_id = @P1
        ORDER BY scheduling_block_id
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch analytics blocks: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read analytics blocks: {e}"))?;

    let mut blocks = Vec::with_capacity(rows.len());

    for row in rows {
        let priority: f64 = row.get::<f64, _>(0).unwrap_or(0.0);
        let total_visibility_hours: f64 = row.get::<f64, _>(1).unwrap_or(0.0);
        let requested_hours: f64 = row.get::<f64, _>(2).unwrap_or(0.0);
        let elevation_range_deg: f64 = row.get::<f64, _>(3).unwrap_or(90.0);
        let is_scheduled: bool = row.get::<bool, _>(4).unwrap_or(false);

        blocks.push(DistributionBlock {
            priority,
            total_visibility_hours,
            requested_hours,
            elevation_range_deg,
            scheduled: is_scheduled,
        });
    }

    Ok(blocks)
}

/// Check if analytics data exists for a schedule.
pub async fn has_analytics_data(schedule_id: i64) -> Result<bool, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = r#"
        SELECT COUNT(*) FROM analytics.schedule_blocks_analytics
        WHERE schedule_id = @P1
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to check analytics: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read count: {e}"))?;

    match row {
        Some(r) => {
            let count: i32 = r.get(0).unwrap_or(0);
            Ok(count > 0)
        }
        None => Ok(false),
    }
}

// =============================================================================
// Phase 2: Summary Analytics Tables
// =============================================================================

use pyo3::prelude::*;

/// Summary metrics for a schedule (matches schedule_summary_analytics table).
#[derive(Debug, Clone)]
#[pyclass(get_all)]
pub struct ScheduleSummary {
    pub schedule_id: i64,
    pub total_blocks: i32,
    pub scheduled_blocks: i32,
    pub unscheduled_blocks: i32,
    pub impossible_blocks: i32,
    pub scheduling_rate: f64,
    pub priority_min: Option<f64>,
    pub priority_max: Option<f64>,
    pub priority_mean: Option<f64>,
    pub priority_median: Option<f64>,
    pub priority_scheduled_mean: Option<f64>,
    pub priority_unscheduled_mean: Option<f64>,
    pub visibility_total_hours: f64,
    pub visibility_mean_hours: Option<f64>,
    pub requested_total_hours: f64,
    pub requested_mean_hours: Option<f64>,
    pub scheduled_total_hours: f64,
    pub corr_priority_visibility: Option<f64>,
    pub corr_priority_requested: Option<f64>,
    pub corr_visibility_requested: Option<f64>,
    pub conflict_count: i32,
}

/// Priority-level rate data (matches schedule_priority_rates table).
#[derive(Debug, Clone)]
#[pyclass(get_all)]
pub struct PriorityRate {
    pub priority_value: i32,
    pub total_count: i32,
    pub scheduled_count: i32,
    pub scheduling_rate: f64,
    pub visibility_mean_hours: Option<f64>,
    pub requested_mean_hours: Option<f64>,
}

/// Visibility bin data (matches schedule_visibility_bins table).
#[derive(Debug, Clone)]
#[pyclass(get_all)]
pub struct VisibilityBin {
    pub bin_index: i32,
    pub bin_min_hours: f64,
    pub bin_max_hours: f64,
    pub bin_mid_hours: f64,
    pub total_count: i32,
    pub scheduled_count: i32,
    pub scheduling_rate: f64,
}

/// Heatmap bin data (matches schedule_heatmap_bins table).
#[derive(Debug, Clone)]
#[pyclass(get_all)]
pub struct HeatmapBinData {
    pub visibility_mid_hours: f64,
    pub time_mid_hours: f64,
    pub total_count: i32,
    pub scheduled_count: i32,
    pub scheduling_rate: f64,
}

/// Populate summary analytics tables for a schedule.
///
/// This function populates:
/// - schedule_summary_analytics: Overall schedule metrics
/// - schedule_priority_rates: Per-priority scheduling rates
/// - schedule_visibility_bins: Visibility-based rate bins
/// - schedule_heatmap_bins: 2D visibility x time bins
///
/// Requires that schedule_blocks_analytics is already populated.
pub async fn populate_summary_analytics(schedule_id: i64, n_bins: usize) -> Result<(), String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    info!(
        "Populating summary analytics for schedule_id={}",
        schedule_id
    );

    // First, check if block-level analytics exist
    let check_sql = "SELECT COUNT(*) FROM analytics.schedule_blocks_analytics WHERE schedule_id = @P1";
    let mut check_query = Query::new(check_sql);
    check_query.bind(schedule_id);

    let check_stream = check_query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to check block analytics: {e}"))?;

    let check_row = check_stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read count: {e}"))?;

    let block_count: i32 = check_row
        .and_then(|r| r.get(0))
        .unwrap_or(0);

    if block_count == 0 {
        warn!(
            "No block-level analytics for schedule_id={}, skipping summary",
            schedule_id
        );
        return Ok(());
    }

    // Delete existing summaries
    for table in &[
        "analytics.schedule_summary_analytics",
        "analytics.schedule_priority_rates",
        "analytics.schedule_visibility_bins",
        "analytics.schedule_heatmap_bins",
    ] {
        let delete_sql = format!("DELETE FROM {} WHERE schedule_id = @P1", table);
        let mut delete_query = Query::new(delete_sql);
        delete_query.bind(schedule_id);
        delete_query
            .execute(&mut *conn)
            .await
            .map_err(|e| format!("Failed to delete from {}: {}", table, e))?;
    }

    // Populate schedule_summary_analytics
    populate_schedule_summary(&mut conn, schedule_id).await?;

    // Populate schedule_priority_rates
    populate_priority_rates(&mut conn, schedule_id).await?;

    // Populate schedule_visibility_bins
    populate_visibility_bins(&mut conn, schedule_id, n_bins).await?;

    // Populate schedule_heatmap_bins
    populate_heatmap_bins(&mut conn, schedule_id, n_bins).await?;

    info!(
        "Completed summary analytics for schedule_id={}",
        schedule_id
    );

    Ok(())
}

// Note: DbClient is already defined earlier in this file

async fn populate_schedule_summary(conn: &mut DbClient, schedule_id: i64) -> Result<(), String> {
    let sql = r#"
        INSERT INTO analytics.schedule_summary_analytics (
            schedule_id,
            total_blocks,
            scheduled_blocks,
            unscheduled_blocks,
            impossible_blocks,
            scheduling_rate,
            priority_min,
            priority_max,
            priority_mean,
            priority_scheduled_mean,
            priority_unscheduled_mean,
            visibility_total_hours,
            visibility_mean_hours,
            visibility_min_hours,
            visibility_max_hours,
            requested_total_hours,
            requested_mean_hours,
            requested_min_hours,
            requested_max_hours,
            scheduled_total_hours,
            ra_min,
            ra_max,
            dec_min,
            dec_max,
            scheduled_time_min_mjd,
            scheduled_time_max_mjd
        )
        SELECT
            @P1,
            COUNT(*),
            SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END),
            SUM(CASE WHEN is_scheduled = 0 THEN 1 ELSE 0 END),
            SUM(CASE WHEN is_impossible = 1 THEN 1 ELSE 0 END),
            CAST(SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END) AS FLOAT) / NULLIF(COUNT(*), 0),
            MIN(priority),
            MAX(priority),
            AVG(priority),
            AVG(CASE WHEN is_scheduled = 1 THEN priority END),
            AVG(CASE WHEN is_scheduled = 0 THEN priority END),
            SUM(total_visibility_hours),
            AVG(total_visibility_hours),
            MIN(total_visibility_hours),
            MAX(total_visibility_hours),
            SUM(requested_hours),
            AVG(requested_hours),
            MIN(requested_hours),
            MAX(requested_hours),
            SUM(CASE WHEN is_scheduled = 1 THEN COALESCE(scheduled_duration_sec, 0) / 3600.0 ELSE 0 END),
            MIN(target_ra_deg),
            MAX(target_ra_deg),
            MIN(target_dec_deg),
            MAX(target_dec_deg),
            MIN(scheduled_start_mjd),
            MAX(scheduled_stop_mjd)
        FROM analytics.schedule_blocks_analytics
        WHERE schedule_id = @P1
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    query
        .execute(&mut *conn)
        .await
        .map_err(|e| format!("Failed to insert schedule summary: {e}"))?;

    debug!("Inserted schedule summary for schedule_id={}", schedule_id);
    Ok(())
}

async fn populate_priority_rates(conn: &mut DbClient, schedule_id: i64) -> Result<(), String> {
    let sql = r#"
        INSERT INTO analytics.schedule_priority_rates (
            schedule_id,
            priority_value,
            total_count,
            scheduled_count,
            unscheduled_count,
            impossible_count,
            scheduling_rate,
            visibility_mean_hours,
            visibility_total_hours,
            requested_mean_hours,
            requested_total_hours
        )
        SELECT
            @P1,
            CAST(ROUND(priority, 0) AS INT),
            COUNT(*),
            SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END),
            SUM(CASE WHEN is_scheduled = 0 THEN 1 ELSE 0 END),
            SUM(CASE WHEN is_impossible = 1 THEN 1 ELSE 0 END),
            CAST(SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END) AS FLOAT) / NULLIF(COUNT(*), 0),
            AVG(total_visibility_hours),
            SUM(total_visibility_hours),
            AVG(requested_hours),
            SUM(requested_hours)
        FROM analytics.schedule_blocks_analytics
        WHERE schedule_id = @P1
        GROUP BY CAST(ROUND(priority, 0) AS INT)
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let result = query
        .execute(&mut *conn)
        .await
        .map_err(|e| format!("Failed to insert priority rates: {e}"))?;

    let rows = result.rows_affected().iter().sum::<u64>();
    debug!(
        "Inserted {} priority rates for schedule_id={}",
        rows, schedule_id
    );
    Ok(())
}

async fn populate_visibility_bins(
    conn: &mut DbClient,
    schedule_id: i64,
    n_bins: usize,
) -> Result<(), String> {
    // First, get visibility range
    let range_sql = r#"
        SELECT MIN(total_visibility_hours), MAX(total_visibility_hours)
        FROM analytics.schedule_blocks_analytics
        WHERE schedule_id = @P1
    "#;

    let mut range_query = Query::new(range_sql);
    range_query.bind(schedule_id);

    let range_stream = range_query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to get visibility range: {e}"))?;

    let range_row = range_stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read visibility range: {e}"))?;

    let (vis_min, vis_max) = match range_row {
        Some(row) => {
            let min_val: Option<f64> = row.get(0);
            let max_val: Option<f64> = row.get(1);
            (min_val.unwrap_or(0.0), max_val.unwrap_or(0.0))
        }
        None => return Ok(()),
    };

    if (vis_max - vis_min).abs() < f64::EPSILON {
        // Single value, create one bin
        let sql = r#"
            INSERT INTO analytics.schedule_visibility_bins (
                schedule_id, bin_index, bin_min_hours, bin_max_hours, bin_mid_hours,
                total_count, scheduled_count, scheduling_rate, priority_mean
            )
            SELECT
                @P1, 0, @P2, @P3, @P4,
                COUNT(*),
                SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END),
                CAST(SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END) AS FLOAT) / NULLIF(COUNT(*), 0),
                AVG(priority)
            FROM analytics.schedule_blocks_analytics
            WHERE schedule_id = @P1
        "#;

        let mut query = Query::new(sql);
        query.bind(schedule_id);
        query.bind(vis_min);
        query.bind(vis_max);
        query.bind(vis_min);

        query
            .execute(&mut *conn)
            .await
            .map_err(|e| format!("Failed to insert single visibility bin: {e}"))?;

        return Ok(());
    }

    let bin_width = (vis_max - vis_min) / n_bins as f64;

    // Insert bins using a loop (SQL Server doesn't have generate_series)
    for i in 0..n_bins {
        let bin_min = vis_min + (i as f64 * bin_width);
        let bin_max = if i == n_bins - 1 {
            vis_max + 0.001 // Include the max value in last bin
        } else {
            vis_min + ((i + 1) as f64 * bin_width)
        };
        let bin_mid = (bin_min + bin_max) / 2.0;

        let sql = r#"
            INSERT INTO analytics.schedule_visibility_bins (
                schedule_id, bin_index, bin_min_hours, bin_max_hours, bin_mid_hours,
                total_count, scheduled_count, scheduling_rate, priority_mean
            )
            SELECT
                @P1, @P2, @P3, @P4, @P5,
                COUNT(*),
                SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END),
                CAST(SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END) AS FLOAT) / NULLIF(COUNT(*), 0),
                AVG(priority)
            FROM analytics.schedule_blocks_analytics
            WHERE schedule_id = @P1
              AND total_visibility_hours >= @P3
              AND total_visibility_hours < @P4
            HAVING COUNT(*) > 0
        "#;

        let mut query = Query::new(sql);
        query.bind(schedule_id);
        query.bind(i as i32);
        query.bind(bin_min);
        query.bind(bin_max);
        query.bind(bin_mid);

        query
            .execute(&mut *conn)
            .await
            .map_err(|e| format!("Failed to insert visibility bin {}: {}", i, e))?;
    }

    debug!(
        "Inserted visibility bins for schedule_id={}",
        schedule_id
    );
    Ok(())
}

async fn populate_heatmap_bins(
    conn: &mut DbClient,
    schedule_id: i64,
    n_bins: usize,
) -> Result<(), String> {
    // Get ranges for both dimensions
    let range_sql = r#"
        SELECT 
            MIN(total_visibility_hours), MAX(total_visibility_hours),
            MIN(requested_hours), MAX(requested_hours)
        FROM analytics.schedule_blocks_analytics
        WHERE schedule_id = @P1
    "#;

    let mut range_query = Query::new(range_sql);
    range_query.bind(schedule_id);

    let range_stream = range_query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to get heatmap ranges: {e}"))?;

    let range_row = range_stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read heatmap ranges: {e}"))?;

    let (vis_min, vis_max, time_min, time_max) = match range_row {
        Some(row) => {
            let vis_min: Option<f64> = row.get(0);
            let vis_max: Option<f64> = row.get(1);
            let time_min: Option<f64> = row.get(2);
            let time_max: Option<f64> = row.get(3);
            (
                vis_min.unwrap_or(0.0),
                vis_max.unwrap_or(0.0),
                time_min.unwrap_or(0.0),
                time_max.unwrap_or(0.0),
            )
        }
        None => return Ok(()),
    };

    if (vis_max - vis_min).abs() < f64::EPSILON || (time_max - time_min).abs() < f64::EPSILON {
        return Ok(()); // Can't create 2D bins with no range
    }

    let vis_width = (vis_max - vis_min) / n_bins as f64;
    let time_width = (time_max - time_min) / n_bins as f64;

    // Insert 2D bins
    for vi in 0..n_bins {
        let vis_bin_min = vis_min + (vi as f64 * vis_width);
        let vis_bin_max = if vi == n_bins - 1 {
            vis_max + 0.001
        } else {
            vis_min + ((vi + 1) as f64 * vis_width)
        };

        for ti in 0..n_bins {
            let time_bin_min = time_min + (ti as f64 * time_width);
            let time_bin_max = if ti == n_bins - 1 {
                time_max + 0.001
            } else {
                time_min + ((ti + 1) as f64 * time_width)
            };

            let sql = r#"
                INSERT INTO analytics.schedule_heatmap_bins (
                    schedule_id, visibility_bin_index, time_bin_index,
                    visibility_mid_hours, time_mid_hours,
                    total_count, scheduled_count, scheduling_rate
                )
                SELECT
                    @P1, @P2, @P3,
                    AVG(total_visibility_hours), AVG(requested_hours),
                    COUNT(*),
                    SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END),
                    CAST(SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END) AS FLOAT) / NULLIF(COUNT(*), 0)
                FROM analytics.schedule_blocks_analytics
                WHERE schedule_id = @P1
                  AND total_visibility_hours >= @P4 AND total_visibility_hours < @P5
                  AND requested_hours >= @P6 AND requested_hours < @P7
                HAVING COUNT(*) > 0
            "#;

            let mut query = Query::new(sql);
            query.bind(schedule_id);
            query.bind(vi as i32);
            query.bind(ti as i32);
            query.bind(vis_bin_min);
            query.bind(vis_bin_max);
            query.bind(time_bin_min);
            query.bind(time_bin_max);

            query
                .execute(&mut *conn)
                .await
                .map_err(|e| format!("Failed to insert heatmap bin ({}, {}): {}", vi, ti, e))?;
        }
    }

    debug!("Inserted heatmap bins for schedule_id={}", schedule_id);
    Ok(())
}

/// Fetch schedule summary from analytics table.
pub async fn fetch_schedule_summary(schedule_id: i64) -> Result<Option<ScheduleSummary>, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = r#"
        SELECT
            schedule_id,
            total_blocks,
            scheduled_blocks,
            unscheduled_blocks,
            impossible_blocks,
            scheduling_rate,
            priority_min,
            priority_max,
            priority_mean,
            priority_median,
            priority_scheduled_mean,
            priority_unscheduled_mean,
            visibility_total_hours,
            visibility_mean_hours,
            requested_total_hours,
            requested_mean_hours,
            scheduled_total_hours,
            corr_priority_visibility,
            corr_priority_requested,
            corr_visibility_requested,
            conflict_count
        FROM analytics.schedule_summary_analytics
        WHERE schedule_id = @P1
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch schedule summary: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read schedule summary: {e}"))?;

    match row {
        Some(r) => Ok(Some(ScheduleSummary {
            schedule_id: r.get::<i64, _>(0).unwrap_or(0),
            total_blocks: r.get::<i32, _>(1).unwrap_or(0),
            scheduled_blocks: r.get::<i32, _>(2).unwrap_or(0),
            unscheduled_blocks: r.get::<i32, _>(3).unwrap_or(0),
            impossible_blocks: r.get::<i32, _>(4).unwrap_or(0),
            scheduling_rate: r.get::<f64, _>(5).unwrap_or(0.0),
            priority_min: r.get(6),
            priority_max: r.get(7),
            priority_mean: r.get(8),
            priority_median: r.get(9),
            priority_scheduled_mean: r.get(10),
            priority_unscheduled_mean: r.get(11),
            visibility_total_hours: r.get::<f64, _>(12).unwrap_or(0.0),
            visibility_mean_hours: r.get(13),
            requested_total_hours: r.get::<f64, _>(14).unwrap_or(0.0),
            requested_mean_hours: r.get(15),
            scheduled_total_hours: r.get::<f64, _>(16).unwrap_or(0.0),
            corr_priority_visibility: r.get(17),
            corr_priority_requested: r.get(18),
            corr_visibility_requested: r.get(19),
            conflict_count: r.get::<i32, _>(20).unwrap_or(0),
        })),
        None => Ok(None),
    }
}

/// Fetch priority rates from analytics table.
pub async fn fetch_priority_rates(schedule_id: i64) -> Result<Vec<PriorityRate>, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = r#"
        SELECT
            priority_value,
            total_count,
            scheduled_count,
            scheduling_rate,
            visibility_mean_hours,
            requested_mean_hours
        FROM analytics.schedule_priority_rates
        WHERE schedule_id = @P1
        ORDER BY priority_value
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch priority rates: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read priority rates: {e}"))?;

    let rates = rows
        .iter()
        .map(|r| PriorityRate {
            priority_value: r.get::<i32, _>(0).unwrap_or(0),
            total_count: r.get::<i32, _>(1).unwrap_or(0),
            scheduled_count: r.get::<i32, _>(2).unwrap_or(0),
            scheduling_rate: r.get::<f64, _>(3).unwrap_or(0.0),
            visibility_mean_hours: r.get(4),
            requested_mean_hours: r.get(5),
        })
        .collect();

    Ok(rates)
}

/// Fetch visibility bins from analytics table.
pub async fn fetch_visibility_bins(schedule_id: i64) -> Result<Vec<VisibilityBin>, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = r#"
        SELECT
            bin_index,
            bin_min_hours,
            bin_max_hours,
            bin_mid_hours,
            total_count,
            scheduled_count,
            scheduling_rate
        FROM analytics.schedule_visibility_bins
        WHERE schedule_id = @P1
        ORDER BY bin_index
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch visibility bins: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read visibility bins: {e}"))?;

    let bins = rows
        .iter()
        .map(|r| VisibilityBin {
            bin_index: r.get::<i32, _>(0).unwrap_or(0),
            bin_min_hours: r.get::<f64, _>(1).unwrap_or(0.0),
            bin_max_hours: r.get::<f64, _>(2).unwrap_or(0.0),
            bin_mid_hours: r.get::<f64, _>(3).unwrap_or(0.0),
            total_count: r.get::<i32, _>(4).unwrap_or(0),
            scheduled_count: r.get::<i32, _>(5).unwrap_or(0),
            scheduling_rate: r.get::<f64, _>(6).unwrap_or(0.0),
        })
        .collect();

    Ok(bins)
}

/// Fetch heatmap bins from analytics table.
pub async fn fetch_heatmap_bins(schedule_id: i64) -> Result<Vec<HeatmapBinData>, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = r#"
        SELECT
            visibility_mid_hours,
            time_mid_hours,
            total_count,
            scheduled_count,
            scheduling_rate
        FROM analytics.schedule_heatmap_bins
        WHERE schedule_id = @P1
        ORDER BY visibility_bin_index, time_bin_index
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch heatmap bins: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read heatmap bins: {e}"))?;

    let bins = rows
        .iter()
        .map(|r| HeatmapBinData {
            visibility_mid_hours: r.get::<f64, _>(0).unwrap_or(0.0),
            time_mid_hours: r.get::<f64, _>(1).unwrap_or(0.0),
            total_count: r.get::<i32, _>(2).unwrap_or(0),
            scheduled_count: r.get::<i32, _>(3).unwrap_or(0),
            scheduling_rate: r.get::<f64, _>(4).unwrap_or(0.0),
        })
        .collect();

    Ok(bins)
}

/// Check if summary analytics data exists for a schedule.
pub async fn has_summary_analytics(schedule_id: i64) -> Result<bool, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = r#"
        SELECT COUNT(*) FROM analytics.schedule_summary_analytics
        WHERE schedule_id = @P1
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to check summary analytics: {e}"))?;

    let row = stream
        .into_row()
        .await
        .map_err(|e| format!("Failed to read count: {e}"))?;

    match row {
        Some(r) => {
            let count: i32 = r.get(0).unwrap_or(0);
            Ok(count > 0)
        }
        None => Ok(false),
    }
}

/// Delete summary analytics for a schedule.
pub async fn delete_summary_analytics(schedule_id: i64) -> Result<usize, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let mut total_deleted = 0u64;

    for table in &[
        "analytics.schedule_summary_analytics",
        "analytics.schedule_priority_rates",
        "analytics.schedule_visibility_bins",
        "analytics.schedule_heatmap_bins",
    ] {
        let sql = format!("DELETE FROM {} WHERE schedule_id = @P1", table);
        let mut query = Query::new(sql);
        query.bind(schedule_id);

        let result = query
            .execute(&mut *conn)
            .await
            .map_err(|e| format!("Failed to delete from {}: {}", table, e))?;

        total_deleted += result.rows_affected().iter().sum::<u64>();
    }

    info!(
        "Deleted {} summary analytics rows for schedule_id={}",
        total_deleted, schedule_id
    );

    Ok(total_deleted as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_priority_bucket() {
        // Test with range
        assert_eq!(compute_priority_bucket(0.0, 0.0, Some(10.0)), 1); // Low
        assert_eq!(compute_priority_bucket(2.4, 0.0, Some(10.0)), 1); // Low
        assert_eq!(compute_priority_bucket(2.5, 0.0, Some(10.0)), 2); // Medium-Low
        assert_eq!(compute_priority_bucket(5.0, 0.0, Some(10.0)), 3); // Medium-High
        assert_eq!(compute_priority_bucket(7.5, 0.0, Some(10.0)), 4); // High
        assert_eq!(compute_priority_bucket(10.0, 0.0, Some(10.0)), 4); // High

        // Test with no range (single value)
        assert_eq!(compute_priority_bucket(5.0, 5.0, None), 2); // Default to medium
    }

    #[test]
    fn test_parse_visibility_periods() {
        // Empty/null cases
        assert_eq!(parse_visibility_periods(None), (0.0, 0));
        assert_eq!(parse_visibility_periods(Some("")), (0.0, 0));
        assert_eq!(parse_visibility_periods(Some("null")), (0.0, 0));

        // Valid JSON
        let json = r#"[{"start": 60000.0, "stop": 60001.0}, {"start": 60002.0, "stop": 60002.5}]"#;
        let (hours, count) = parse_visibility_periods(Some(json));
        assert_eq!(count, 2);
        // 1 day + 0.5 day = 1.5 days = 36 hours
        assert!((hours - 36.0).abs() < 0.01);

        // Invalid JSON
        assert_eq!(parse_visibility_periods(Some("invalid")), (0.0, 0));
    }
}
