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
//!
//! NOTE: This is a stub implementation. Azure backend has been removed.

#![allow(clippy::all)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use log::debug;
use tiberius::Query;

use super::pool;
// use crate::api::ScheduleId;  // Unused in stub implementation

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
#[allow(unreachable_code)]
pub async fn populate_schedule_analytics(_schedule_id: i64) -> Result<usize, String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics placeholder");
}

/// Delete analytics data for a schedule.
///
/// # Arguments
/// * `schedule_id` - The ID of the schedule whose analytics should be deleted
///
/// # Returns
/// * `Ok(usize)` - Number of rows deleted
/// * `Err(String)` - Error description if the operation fails
#[allow(unreachable_code)]
pub async fn delete_schedule_analytics(_schedule_id: i64) -> Result<usize, String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
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
    original_block_id: Option<String>,
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
    constraint_start_mjd: Option<f64>,
    constraint_stop_mjd: Option<f64>,
    is_scheduled: bool,
    scheduled_start_mjd: Option<f64>,
    scheduled_stop_mjd: Option<f64>,
    total_visibility_hours: f64,
    visibility_period_count: i32,
}

type DbClient = tiberius::Client<tokio_util::compat::Compat<tokio::net::TcpStream>>;

/// Bulk insert analytics rows in batches.
async fn bulk_insert_analytics(
    conn: &mut DbClient,
    rows: &[AnalyticsRow],
) -> Result<usize, String> {
    if rows.is_empty() {
        return Ok(0);
    }

    // SQL Server parameter limit: ~2100 params
    // Each row uses 21 params (added original_block_id), so max batch = 2100/21 = 100
    // Use 95 for safety margin
    const BATCH_SIZE: usize = 95;

    let mut total_inserted = 0;

    for chunk in rows.chunks(BATCH_SIZE) {
        let mut values_clauses = Vec::with_capacity(chunk.len());
        for (i, _) in chunk.iter().enumerate() {
            let base = i * 21;
            values_clauses.push(format!(
                "(@P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{})",
                base + 1, base + 2, base + 3, base + 4, base + 5, base + 6,
                base + 7, base + 8, base + 9, base + 10, base + 11, base + 12,
                base + 13, base + 14, base + 15, base + 16, base + 17, base + 18,
                base + 19, base + 20, base + 21
            ));
        }

        let sql = format!(
            r#"
            INSERT INTO analytics.schedule_blocks_analytics (
                schedule_id,
                scheduling_block_id,
                original_block_id,
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
                constraint_start_mjd,
                constraint_stop_mjd,
                is_scheduled,
                scheduled_start_mjd,
                scheduled_stop_mjd,
                total_visibility_hours,
                visibility_period_count,
                validation_impossible
            ) VALUES {}
            "#,
            values_clauses.join(", ")
        );

        debug!(
            "SQL for batch insert ({} rows): {}",
            chunk.len(),
            &sql[..sql.len().min(500)]
        );

        let mut insert = Query::new(sql);

        for row in chunk {
            insert.bind(row.schedule_id);
            insert.bind(row.scheduling_block_id);
            insert.bind(row.original_block_id.as_deref());
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
            insert.bind(row.constraint_start_mjd);
            insert.bind(row.constraint_stop_mjd);
            insert.bind(row.is_scheduled);
            insert.bind(row.scheduled_start_mjd);
            insert.bind(row.scheduled_stop_mjd);
            insert.bind(row.total_visibility_hours);
            insert.bind(row.visibility_period_count);
            // validation_impossible is set to NULL initially, validation happens in Phase 4
            insert.bind(Option::<bool>::None);
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
#[allow(unreachable_code)]
pub async fn fetch_analytics_blocks_for_sky_map(
    _schedule_id: i64,
) -> Result<Vec<crate::api::LightweightBlock>, String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
}

/// Fetch distribution blocks from the analytics table.
/// This replaces the join-based fetch_distribution_blocks function.
#[allow(unreachable_code)]
pub async fn fetch_analytics_blocks_for_distribution(
    _schedule_id: i64,
) -> Result<Vec<crate::api::DistributionBlock>, String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
}

/// Fetch schedule timeline blocks from the analytics table.
///  This is much faster than fetch_schedule_timeline_blocks as it avoids JOINs
/// and uses pre-computed visibility metrics.
#[allow(unreachable_code)]
pub async fn fetch_analytics_blocks_for_timeline(
    _schedule_id: i64,
) -> Result<Vec<crate::db::models::ScheduleTimelineBlock>, String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
}

/// Fetch visibility map data from the analytics table.
/// This is much faster than fetch_visibility_map_data as it avoids JOINs
/// and JSON parsing, using pre-computed visibility metrics instead.
#[allow(unreachable_code)]
pub async fn fetch_analytics_blocks_for_visibility_map(
    _schedule_id: i64,
) -> Result<crate::api::VisibilityMapData, String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
}

/// Fetch insights blocks from the analytics table.
/// This is much faster than fetch_insights_blocks as it avoids JOINs
/// and JSON parsing, using pre-computed metrics instead.
#[allow(unreachable_code)]
pub async fn fetch_analytics_blocks_for_insights(
    _schedule_id: i64,
) -> Result<Vec<crate::db::models::InsightsBlock>, String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
}

/// Fetch trends blocks from the analytics table.
/// This is much faster than fetch_trends_blocks as it avoids JOINs
/// and JSON parsing, using pre-computed metrics instead.
#[allow(unreachable_code)]
pub async fn fetch_analytics_blocks_for_trends(
    _schedule_id: i64,
) -> Result<Vec<crate::db::models::TrendsBlock>, String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
}

/// Check if analytics data exists for a schedule.
#[allow(unreachable_code)]
pub async fn has_analytics_data(_schedule_id: i64) -> Result<bool, String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
}

// =============================================================================
// Phase 2: Summary Analytics Tables
// =============================================================================

/// Populate summary analytics tables for a schedule.
///
/// This function populates:
/// - schedule_summary_analytics: Overall schedule metrics
/// - schedule_priority_rates: Per-priority scheduling rates
/// - schedule_visibility_bins: Visibility-based rate bins
/// - schedule_heatmap_bins: 2D visibility x time bins
///
/// Requires that schedule_blocks_analytics is already populated.
#[allow(unreachable_code)]
pub async fn populate_summary_analytics(_schedule_id: i64, _n_bins: usize) -> Result<(), String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
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
            SUM(CASE WHEN COALESCE(validation_impossible, is_impossible) = 1 THEN 1 ELSE 0 END),
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
          AND COALESCE(validation_impossible, is_impossible) = 0
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
            SUM(CASE WHEN COALESCE(validation_impossible, is_impossible) = 1 THEN 1 ELSE 0 END),
            CAST(SUM(CASE WHEN is_scheduled = 1 THEN 1 ELSE 0 END) AS FLOAT) / NULLIF(COUNT(*), 0),
            AVG(total_visibility_hours),
            SUM(total_visibility_hours),
            AVG(requested_hours),
            SUM(requested_hours)
        FROM analytics.schedule_blocks_analytics
        WHERE schedule_id = @P1
          AND COALESCE(validation_impossible, is_impossible) = 0
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
          AND COALESCE(validation_impossible, is_impossible) = 0
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
              AND COALESCE(validation_impossible, is_impossible) = 0
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
              AND COALESCE(validation_impossible, is_impossible) = 0
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

    debug!("Inserted visibility bins for schedule_id={}", schedule_id);
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
          AND COALESCE(validation_impossible, is_impossible) = 0
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
                  AND COALESCE(validation_impossible, is_impossible) = 0
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

/// Check if summary analytics data exists for a schedule.
#[allow(unreachable_code)]
pub async fn has_summary_analytics(_schedule_id: i64) -> Result<bool, String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
}

/// Delete summary analytics for a schedule.
#[allow(unreachable_code)]
pub async fn delete_summary_analytics(_schedule_id: i64) -> Result<usize, String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
}

// =============================================================================
// Phase 3: Visibility Time Bins
// =============================================================================

/// Default bin duration for visibility time bins: 15 minutes (900 seconds)
const DEFAULT_VISIBILITY_BIN_DURATION_SECONDS: i64 = 900;

/// MJD epoch (1858-11-17 00:00:00 UTC) as Unix timestamp
const MJD_EPOCH_UNIX: i64 = -3506716800;

/// Convert Modified Julian Date to Unix timestamp (seconds since 1970-01-01)
#[inline]
fn mjd_to_unix(mjd: f64) -> i64 {
    MJD_EPOCH_UNIX + (mjd * 86400.0) as i64
}

/// Parsed visibility period from JSON
#[derive(Debug, Clone)]
struct VisibilityPeriod {
    start_unix: i64,
    end_unix: i64,
}

/// Block data for visibility binning
#[derive(Debug)]
struct BlockVisibilityData {
    scheduling_block_id: i64,
    #[allow(dead_code)]
    priority: f64,
    priority_quartile: u8,
    is_scheduled: bool,
    periods: Vec<VisibilityPeriod>,
}

/// Parse visibility periods JSON into VisibilityPeriod structs.
fn parse_visibility_periods_for_binning(json_str: &str) -> Vec<VisibilityPeriod> {
    let periods_array: Vec<serde_json::Value> = match serde_json::from_str(json_str) {
        Ok(arr) => arr,
        Err(_) => return vec![],
    };

    let mut periods = Vec::with_capacity(periods_array.len());

    for period_obj in periods_array {
        let start_mjd = match period_obj["start"].as_f64() {
            Some(v) => v,
            None => continue,
        };

        let stop_mjd = match period_obj["stop"].as_f64() {
            Some(v) => v,
            None => continue,
        };

        let start_unix = mjd_to_unix(start_mjd);
        let end_unix = mjd_to_unix(stop_mjd);

        if start_unix < end_unix {
            periods.push(VisibilityPeriod {
                start_unix,
                end_unix,
            });
        }
    }

    periods
}

/// Compute priority quartile (1-4) based on the schedule's priority range.
fn compute_priority_quartile(priority: f64, min_priority: f64, range: Option<f64>) -> u8 {
    match range {
        Some(r) if r > 0.0 => {
            let normalized = (priority - min_priority) / r;
            match normalized {
                x if x < 0.25 => 1,
                x if x < 0.50 => 2,
                x if x < 0.75 => 3,
                _ => 4,
            }
        }
        _ => 2, // Default to middle quartile when no range
    }
}

/// Populate visibility time bins for a schedule.
///
/// This function:
/// 1. Deletes existing visibility time bins for the schedule (idempotent)
/// 2. Fetches all blocks with their visibility periods JSON
/// 3. Parses visibility periods and computes which bins each block is visible in
/// 4. Stores pre-computed counts per time bin
///
/// # Arguments
/// * `schedule_id` - The ID of the schedule to process
/// * `bin_duration_seconds` - Duration of each bin in seconds (default: 900 = 15 minutes)
///
/// # Returns
/// * `Ok((metadata_count, bins_count))` - Number of metadata and bin rows inserted
/// * `Err(String)` - Error description if the operation fails
#[allow(unreachable_code)]
pub async fn populate_visibility_time_bins(
    _schedule_id: i64,
    _bin_duration_seconds: Option<i64>,
) -> Result<(usize, usize), String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
}

/// Delete visibility time bins for a schedule (internal helper).
async fn delete_visibility_time_bins_internal(
    conn: &mut DbClient,
    schedule_id: i64,
) -> Result<usize, String> {
    let mut total_deleted = 0u64;

    for table in &[
        "analytics.schedule_visibility_time_bins",
        "analytics.schedule_visibility_metadata",
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

    if total_deleted > 0 {
        debug!(
            "Deleted {} visibility time bin rows for schedule_id={}",
            total_deleted, schedule_id
        );
    }

    Ok(total_deleted as usize)
}

/// Delete visibility time bins for a schedule (public API).
#[allow(unreachable_code)]
pub async fn delete_visibility_time_bins(_schedule_id: i64) -> Result<usize, String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
}

/// Check if visibility time bins exist for a schedule.
#[allow(unreachable_code)]
pub async fn has_visibility_time_bins(_schedule_id: i64) -> Result<bool, String> {
    let _ = pool::get_pool()?;
    todo!("Azure analytics stub");
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
