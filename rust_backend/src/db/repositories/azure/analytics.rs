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

#![allow(clippy::type_complexity)]
#![allow(clippy::collapsible_if)]

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
            sb.original_block_id,
            t.ra_deg,
            t.dec_deg,
            sb.priority,
            sb.requested_duration_sec,
            sb.min_observation_sec,
            ac.min_alt_deg,
            ac.max_alt_deg,
            azc.min_az_deg,
            azc.max_az_deg,
            c.start_time_mjd as constraint_start_mjd,
            c.stop_time_mjd as constraint_stop_mjd,
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
        let original_block_id: Option<&str> = row.get(2);
        let ra_deg: f64 = row.get::<f64, _>(3).unwrap_or(0.0);
        let dec_deg: f64 = row.get::<f64, _>(4).unwrap_or(0.0);
        let priority: f64 = row.get::<f64, _>(5).unwrap_or(0.0);
        let requested_duration_sec: i32 = row.get::<i32, _>(6).unwrap_or(0);
        let min_observation_sec: i32 = row.get::<i32, _>(7).unwrap_or(0);
        let min_alt_deg: Option<f64> = row.get(8);
        let max_alt_deg: Option<f64> = row.get(9);
        let min_az_deg: Option<f64> = row.get(10);
        let max_az_deg: Option<f64> = row.get(11);
        let constraint_start: Option<f64> = row.get(12);
        let constraint_stop: Option<f64> = row.get(13);
        let scheduled_start: Option<f64> = row.get(14);
        let scheduled_stop: Option<f64> = row.get(15);
        let visibility_json: Option<&str> = row.get(16);

        // Compute priority bucket
        let priority_bucket = compute_priority_bucket(priority, priority_min, priority_range);

        // Parse visibility JSON
        let (total_visibility_hours, visibility_period_count) =
            parse_visibility_periods(visibility_json);

        let is_scheduled = scheduled_start.is_some() && scheduled_stop.is_some();

        analytics_rows.push(AnalyticsRow {
            schedule_id,
            scheduling_block_id,
            original_block_id: original_block_id.map(|s| s.to_string()),
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
            constraint_start_mjd: constraint_start,
            constraint_stop_mjd: constraint_stop,
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

    // ============================================================================
    // Phase 4: Data Validation (NEW)
    // Validate all blocks and persist validation results
    // ============================================================================

    info!("Starting validation for schedule_id={}", schedule_id);

    // Convert analytics rows to validation input format
    let blocks_for_validation: Vec<crate::services::validation::BlockForValidation> =
        analytics_rows
            .iter()
            .map(|row| crate::services::validation::BlockForValidation {
                schedule_id: row.schedule_id,
                scheduling_block_id: row.scheduling_block_id,
                priority: row.priority,
                requested_duration_sec: row.requested_duration_sec,
                min_observation_sec: row.min_observation_sec,
                total_visibility_hours: row.total_visibility_hours,
                min_alt_deg: row.min_altitude_deg,
                max_alt_deg: row.max_altitude_deg,
                constraint_start_mjd: row.constraint_start_mjd,
                constraint_stop_mjd: row.constraint_stop_mjd,
                scheduled_start_mjd: row.scheduled_start_mjd,
                scheduled_stop_mjd: row.scheduled_stop_mjd,
                target_ra_deg: row.target_ra_deg,
                target_dec_deg: row.target_dec_deg,
            })
            .collect();

    // Run validation
    let validation_results = crate::services::validation::validate_blocks(&blocks_for_validation);

    // Persist validation results
    match super::validation::insert_validation_results(&validation_results).await {
        Ok(count) => {
            info!(
                "Inserted {} validation results for schedule_id={}",
                count, schedule_id
            );
        }
        Err(e) => {
            log::warn!("Failed to insert validation results: {}", e);
        }
    }

    // Update validation_impossible flags in analytics table
    match super::validation::update_validation_impossible_flags(schedule_id).await {
        Ok(count) => {
            info!(
                "Updated {} validation_impossible flags for schedule_id={}",
                count, schedule_id
            );
        }
        Err(e) => {
            log::warn!("Failed to update validation_impossible flags: {}", e);
        }
    }

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
pub async fn fetch_analytics_blocks_for_sky_map(
    schedule_id: i64,
) -> Result<Vec<crate::api::LightweightBlock>, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = r#"
        SELECT 
            scheduling_block_id,
            COALESCE(original_block_id, CAST(scheduling_block_id AS NVARCHAR(256))),
            priority,
            requested_duration_sec,
            target_ra_deg,
            target_dec_deg,
            scheduled_start_mjd,
            scheduled_stop_mjd
        FROM analytics.schedule_blocks_analytics
        WHERE schedule_id = @P1
          AND COALESCE(validation_impossible, is_impossible) = 0
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
        let _id: i64 = row
            .get::<i64, _>(0)
            .ok_or_else(|| "scheduling_block_id is NULL".to_string())?;

        let original_block_id: String = row
            .get::<&str, _>(1)
            .ok_or_else(|| "original_block_id is NULL".to_string())?
            .to_string();

        let priority: f64 = row
            .get::<f64, _>(2)
            .ok_or_else(|| "priority is NULL".to_string())?;
        let requested_duration: i32 = row
            .get::<i32, _>(3)
            .ok_or_else(|| "requested_duration_sec is NULL".to_string())?;
        let ra: f64 = row
            .get::<f64, _>(4)
            .ok_or_else(|| "target_ra_deg is NULL".to_string())?;
        let dec: f64 = row
            .get::<f64, _>(5)
            .ok_or_else(|| "target_dec_deg is NULL".to_string())?;

        let scheduled_period = match (row.get::<f64, _>(6), row.get::<f64, _>(7)) {
            (Some(start_mjd), Some(stop_mjd)) => Some(crate::api::Period {
                start: start_mjd,
                stop: stop_mjd,
            }),
            _ => None,
        };

        blocks.push(crate::api::LightweightBlock {
            original_block_id,
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
) -> Result<Vec<crate::api::DistributionBlock>, String> {
    use crate::api::DistributionBlock;

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
          AND COALESCE(validation_impossible, is_impossible) = 0
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

/// Fetch schedule timeline blocks from the analytics table.
/// This is much faster than fetch_schedule_timeline_blocks as it avoids JOINs
/// and uses pre-computed visibility metrics.
pub async fn fetch_analytics_blocks_for_timeline(
    schedule_id: i64,
) -> Result<Vec<crate::db::models::ScheduleTimelineBlock>, String> {
    use crate::db::models::ScheduleTimelineBlock;

    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    // Only select scheduled blocks (where scheduled times exist)
    let sql = r#"
        SELECT 
            a.scheduling_block_id,
            COALESCE(a.original_block_id, CAST(a.scheduling_block_id AS NVARCHAR(256))),
            a.priority,
            a.scheduled_start_mjd,
            a.scheduled_stop_mjd,
            a.target_ra_deg,
            a.target_dec_deg,
            a.requested_hours,
            a.total_visibility_hours,
            a.visibility_period_count
        FROM analytics.schedule_blocks_analytics a
        WHERE a.schedule_id = @P1
          AND a.is_scheduled = 1
          AND a.scheduled_start_mjd IS NOT NULL
          AND a.scheduled_stop_mjd IS NOT NULL
          AND COALESCE(a.validation_impossible, a.is_impossible) = 0
        ORDER BY a.scheduled_start_mjd
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch timeline blocks from analytics: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read timeline blocks: {e}"))?;

    let mut blocks = Vec::with_capacity(rows.len());

    for row in rows {
        let scheduling_block_id: i64 = row
            .get::<i64, _>(0)
            .ok_or_else(|| "scheduling_block_id is NULL".to_string())?;

        let original_block_id: String = row
            .get::<&str, _>(1)
            .ok_or_else(|| "original_block_id is NULL".to_string())?
            .to_string();

        let priority: f64 = row.get::<f64, _>(2).unwrap_or(0.0);
        let scheduled_start_mjd: f64 = row
            .get::<f64, _>(3)
            .ok_or_else(|| "scheduled_start_mjd is NULL".to_string())?;
        let scheduled_stop_mjd: f64 = row
            .get::<f64, _>(4)
            .ok_or_else(|| "scheduled_stop_mjd is NULL".to_string())?;
        let ra_deg: f64 = row.get::<f64, _>(5).unwrap_or(0.0);
        let dec_deg: f64 = row.get::<f64, _>(6).unwrap_or(0.0);
        let requested_hours: f64 = row.get::<f64, _>(7).unwrap_or(0.0);
        let total_visibility_hours: f64 = row.get::<f64, _>(8).unwrap_or(0.0);
        let visibility_period_count: i32 = row.get::<i32, _>(9).unwrap_or(0);

        blocks.push(ScheduleTimelineBlock {
            scheduling_block_id,
            original_block_id,
            priority,
            scheduled_start_mjd: siderust::astro::ModifiedJulianDate::new(scheduled_start_mjd),
            scheduled_stop_mjd: siderust::astro::ModifiedJulianDate::new(scheduled_stop_mjd),
            ra_deg: qtty::angular::Degrees::new(ra_deg),
            dec_deg: qtty::angular::Degrees::new(dec_deg),
            requested_hours: qtty::time::Hours::new(requested_hours),
            total_visibility_hours: qtty::time::Hours::new(total_visibility_hours),
            num_visibility_periods: visibility_period_count as usize,
        });
    }

    debug!(
        "Fetched {} timeline blocks from analytics for schedule_id={}",
        blocks.len(),
        schedule_id
    );

    Ok(blocks)
}

/// Fetch visibility map data from the analytics table.
/// This is much faster than fetch_visibility_map_data as it avoids JOINs
/// and JSON parsing, using pre-computed visibility metrics instead.
pub async fn fetch_analytics_blocks_for_visibility_map(
    schedule_id: i64,
) -> Result<crate::api::VisibilityMapData, String> {
    use crate::api::{VisibilityBlockSummary, VisibilityMapData};

    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = r#"
        SELECT 
            scheduling_block_id,
            COALESCE(original_block_id, CAST(scheduling_block_id AS NVARCHAR(256))),
            priority,
            visibility_period_count,
            is_scheduled
        FROM analytics.schedule_blocks_analytics
        WHERE schedule_id = @P1
          AND COALESCE(validation_impossible, is_impossible) = 0
        ORDER BY scheduling_block_id
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch visibility blocks from analytics: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read visibility blocks: {e}"))?;

    let mut blocks = Vec::with_capacity(rows.len());
    let mut priority_min = f64::INFINITY;
    let mut priority_max = f64::NEG_INFINITY;
    let mut scheduled_count = 0usize;

    for row in rows {
        let scheduling_block_id: i64 = row
            .get::<i64, _>(0)
            .ok_or_else(|| "scheduling_block_id is NULL".to_string())?;

        let original_block_id: String = row
            .get::<&str, _>(1)
            .ok_or_else(|| "original_block_id is NULL".to_string())?
            .to_string();

        let priority: f64 = row
            .get::<f64, _>(2)
            .ok_or_else(|| "priority is NULL".to_string())?;
        let num_visibility_periods: i32 = row.get::<i32, _>(3).unwrap_or(0);
        let scheduled: bool = row.get::<bool, _>(4).unwrap_or(false);

        if scheduled {
            scheduled_count += 1;
        }

        priority_min = priority_min.min(priority);
        priority_max = priority_max.max(priority);

        blocks.push(VisibilityBlockSummary {
            scheduling_block_id,
            original_block_id,
            priority,
            num_visibility_periods: num_visibility_periods as usize,
            scheduled,
        });
    }

    // Handle empty datasets gracefully
    if !priority_min.is_finite() {
        priority_min = 0.0;
    }
    if !priority_max.is_finite() {
        priority_max = 0.0;
    }

    debug!(
        "Fetched {} visibility blocks from analytics for schedule_id={}",
        blocks.len(),
        schedule_id
    );

    Ok(VisibilityMapData {
        total_count: blocks.len(),
        blocks,
        priority_min,
        priority_max,
        scheduled_count,
    })
}

/// Fetch insights blocks from the analytics table.
/// This is much faster than fetch_insights_blocks as it avoids JOINs
/// and JSON parsing, using pre-computed metrics instead.
pub async fn fetch_analytics_blocks_for_insights(
    schedule_id: i64,
) -> Result<Vec<crate::db::models::InsightsBlock>, String> {
    use crate::db::models::InsightsBlock;

    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = r#"
        SELECT 
            a.scheduling_block_id,
            COALESCE(a.original_block_id, CAST(a.scheduling_block_id AS NVARCHAR(256))),
            a.priority,
            a.total_visibility_hours,
            a.requested_hours,
            a.elevation_range_deg,
            a.is_scheduled,
            a.scheduled_start_mjd,
            a.scheduled_stop_mjd
        FROM analytics.schedule_blocks_analytics a
        WHERE a.schedule_id = @P1
          AND COALESCE(a.validation_impossible, a.is_impossible) = 0
        ORDER BY a.scheduling_block_id
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch insights blocks from analytics: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read insights blocks: {e}"))?;

    let mut blocks = Vec::with_capacity(rows.len());

    for row in rows {
        let scheduling_block_id: i64 = row
            .get::<i64, _>(0)
            .ok_or_else(|| "scheduling_block_id is NULL".to_string())?;

        let original_block_id: String = row
            .get::<&str, _>(1)
            .ok_or_else(|| "original_block_id is NULL".to_string())?
            .to_string();

        let priority: f64 = row.get::<f64, _>(2).unwrap_or(0.0);
        let total_visibility_hours: f64 = row.get::<f64, _>(3).unwrap_or(0.0);
        let requested_hours: f64 = row.get::<f64, _>(4).unwrap_or(0.0);
        let elevation_range_deg: f64 = row.get::<f64, _>(5).unwrap_or(90.0);
        let scheduled: bool = row.get::<bool, _>(6).unwrap_or(false);
        let scheduled_start_mjd: Option<f64> = row.get::<f64, _>(7);
        let scheduled_stop_mjd: Option<f64> = row.get::<f64, _>(8);

        blocks.push(InsightsBlock {
            scheduling_block_id,
            original_block_id,
            priority,
            total_visibility_hours: qtty::time::Hours::new(total_visibility_hours),
            requested_hours: qtty::time::Hours::new(requested_hours),
            elevation_range_deg: qtty::angular::Degrees::new(elevation_range_deg),
            scheduled,
            scheduled_start_mjd: scheduled_start_mjd.map(siderust::astro::ModifiedJulianDate::new),
            scheduled_stop_mjd: scheduled_stop_mjd.map(siderust::astro::ModifiedJulianDate::new),
        });
    }

    debug!(
        "Fetched {} insights blocks from analytics for schedule_id={}",
        blocks.len(),
        schedule_id
    );

    Ok(blocks)
}

/// Fetch trends blocks from the analytics table.
/// This is much faster than fetch_trends_blocks as it avoids JOINs
/// and JSON parsing, using pre-computed metrics instead.
pub async fn fetch_analytics_blocks_for_trends(
    schedule_id: i64,
) -> Result<Vec<crate::db::models::TrendsBlock>, String> {
    use crate::db::models::TrendsBlock;

    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = r#"
        SELECT 
            a.scheduling_block_id,
            COALESCE(a.original_block_id, CAST(a.scheduling_block_id AS NVARCHAR(256))),
            a.priority,
            a.total_visibility_hours,
            a.requested_hours,
            a.is_scheduled
        FROM analytics.schedule_blocks_analytics a
        WHERE a.schedule_id = @P1
          AND COALESCE(a.validation_impossible, a.is_impossible) = 0
        ORDER BY a.scheduling_block_id
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch trends blocks from analytics: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read trends blocks: {e}"))?;

    let mut blocks = Vec::with_capacity(rows.len());

    for row in rows {
        let scheduling_block_id: i64 = row
            .get::<i64, _>(0)
            .ok_or_else(|| "scheduling_block_id is NULL".to_string())?;

        let original_block_id: String = row
            .get::<&str, _>(1)
            .ok_or_else(|| "original_block_id is NULL".to_string())?
            .to_string();

        let priority: f64 = row.get::<f64, _>(2).unwrap_or(0.0);
        let total_visibility_hours: f64 = row.get::<f64, _>(3).unwrap_or(0.0);
        let requested_hours: f64 = row.get::<f64, _>(4).unwrap_or(0.0);
        let scheduled: bool = row.get::<bool, _>(5).unwrap_or(false);

        blocks.push(TrendsBlock {
            scheduling_block_id,
            original_block_id,
            priority,
            total_visibility_hours: qtty::time::Hours::new(total_visibility_hours),
            requested_hours: qtty::time::Hours::new(requested_hours),
            scheduled,
        });
    }

    debug!(
        "Fetched {} trends blocks from analytics for schedule_id={}",
        blocks.len(),
        schedule_id
    );

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
    let check_sql =
        "SELECT COUNT(*) FROM analytics.schedule_blocks_analytics WHERE schedule_id = @P1";
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

    let block_count: i32 = check_row.and_then(|r| r.get(0)).unwrap_or(0);

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
pub async fn populate_visibility_time_bins(
    schedule_id: i64,
    bin_duration_seconds: Option<i64>,
) -> Result<(usize, usize), String> {
    let start_time = std::time::Instant::now();
    let bin_duration = bin_duration_seconds.unwrap_or(DEFAULT_VISIBILITY_BIN_DURATION_SECONDS);

    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    info!(
        "Populating visibility time bins for schedule_id={} with {}s bins",
        schedule_id, bin_duration
    );

    // Delete existing data for this schedule
    let delete_start = std::time::Instant::now();
    delete_visibility_time_bins_internal(&mut conn, schedule_id).await?;
    debug!(
        "Deleted existing bins in {:.2}s",
        delete_start.elapsed().as_secs_f64()
    );

    // Fetch all blocks with visibility data
    let fetch_start = std::time::Instant::now();
    let sql = r#"
        SELECT 
            sb.scheduling_block_id,
            sb.priority,
            sb.visibility_periods_json,
            CASE WHEN ssb.start_time_mjd IS NOT NULL THEN 1 ELSE 0 END as is_scheduled
        FROM dbo.schedule_scheduling_blocks ssb
        JOIN dbo.scheduling_blocks sb ON ssb.scheduling_block_id = sb.scheduling_block_id
        WHERE ssb.schedule_id = @P1
        ORDER BY sb.scheduling_block_id
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to fetch blocks: {e}"))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|e| format!("Failed to read blocks: {e}"))?;

    if rows.is_empty() {
        warn!("No blocks found for schedule_id={}", schedule_id);
        return Ok((0, 0));
    }

    info!(
        "Fetched {} blocks in {:.2}s, processing visibility periods...",
        rows.len(),
        fetch_start.elapsed().as_secs_f64()
    );

    // First pass: determine priority range and time range
    let parse_start = std::time::Instant::now();
    let mut priority_min = f64::INFINITY;
    let mut priority_max = f64::NEG_INFINITY;
    let mut time_min = i64::MAX;
    let mut time_max = i64::MIN;
    let mut blocks_with_visibility = 0;

    let mut raw_blocks: Vec<(i64, f64, Option<String>, bool)> = Vec::with_capacity(rows.len());

    for row in &rows {
        let block_id: i64 = row.get(0).ok_or("scheduling_block_id is NULL")?;
        let priority: f64 = row.get::<f64, _>(1).unwrap_or(0.0);
        let visibility_json: Option<&str> = row.get(2);
        let is_scheduled: i32 = row.get(3).unwrap_or(0);

        priority_min = priority_min.min(priority);
        priority_max = priority_max.max(priority);

        if let Some(json_str) = visibility_json {
            if !json_str.is_empty() && json_str != "null" {
                let periods = parse_visibility_periods_for_binning(json_str);
                if !periods.is_empty() {
                    blocks_with_visibility += 1;
                    for p in &periods {
                        time_min = time_min.min(p.start_unix);
                        time_max = time_max.max(p.end_unix);
                    }
                }
            }
        }

        raw_blocks.push((
            block_id,
            priority,
            visibility_json.map(|s| s.to_string()),
            is_scheduled != 0,
        ));
    }

    if time_min >= time_max {
        warn!(
            "No valid visibility periods for schedule_id={}",
            schedule_id
        );
        return Ok((0, 0));
    }

    let priority_range = if (priority_max - priority_min).abs() < f64::EPSILON {
        None
    } else {
        Some(priority_max - priority_min)
    };

    // Calculate number of bins
    let time_range = time_max - time_min;
    let num_bins = ((time_range + bin_duration - 1) / bin_duration) as usize;

    info!(
        "Parsed visibility periods in {:.2}s: {} blocks with visibility, time range {}s ({} days), {} bins",
        parse_start.elapsed().as_secs_f64(),
        blocks_with_visibility,
        time_range,
        time_range / 86400,
        num_bins
    );

    // Second pass: parse visibility and compute quartiles
    let mut blocks: Vec<BlockVisibilityData> = Vec::with_capacity(raw_blocks.len());

    for (block_id, priority, visibility_json, is_scheduled) in raw_blocks {
        let quartile = compute_priority_quartile(priority, priority_min, priority_range);
        let periods = visibility_json
            .as_deref()
            .map(parse_visibility_periods_for_binning)
            .unwrap_or_default();

        if !periods.is_empty() {
            blocks.push(BlockVisibilityData {
                scheduling_block_id: block_id,
                priority,
                priority_quartile: quartile,
                is_scheduled,
                periods,
            });
        }
    }

    // Initialize bins
    let bin_init_start = std::time::Instant::now();
    let mut bin_data: Vec<(i64, i64, std::collections::HashSet<i64>, [i32; 4], i32, i32)> = (0
        ..num_bins)
        .map(|i| {
            let bin_start = time_min + (i as i64) * bin_duration;
            let bin_end = std::cmp::min(bin_start + bin_duration, time_max);
            (
                bin_start,
                bin_end,
                std::collections::HashSet::new(),
                [0; 4],
                0,
                0,
            )
        })
        .collect();
    debug!(
        "Initialized {} bins in {:.2}s",
        num_bins,
        bin_init_start.elapsed().as_secs_f64()
    );

    // Populate bins with block visibility (this is the O(n*m) operation)
    info!("Processing block-to-bin assignments (may take 1-2 minutes for large schedules)...");
    let bin_pop_start = std::time::Instant::now();
    let mut total_assignments = 0usize;

    for (idx, block) in blocks.iter().enumerate() {
        if idx > 0 && idx % 100 == 0 {
            debug!(
                "  Progress: {}/{} blocks processed ({:.1}%)",
                idx,
                blocks.len(),
                (idx as f64 / blocks.len() as f64) * 100.0
            );
        }

        for period in &block.periods {
            // Find overlapping bins
            let start_bin = ((period.start_unix - time_min) / bin_duration).max(0) as usize;
            let end_bin = (((period.end_unix - time_min) + bin_duration - 1) / bin_duration)
                .min(num_bins as i64) as usize;

            for bin_idx in start_bin..end_bin {
                if bin_idx >= bin_data.len() {
                    continue;
                }

                let (
                    bin_start,
                    bin_end,
                    ref mut block_ids,
                    ref mut quartile_counts,
                    ref mut sched_count,
                    ref mut unsched_count,
                ) = bin_data[bin_idx];

                // Check if period actually overlaps with this bin
                if period.start_unix < bin_end && period.end_unix > bin_start {
                    if block_ids.insert(block.scheduling_block_id) {
                        // First time this block is added to this bin
                        total_assignments += 1;
                        let q_idx = (block.priority_quartile - 1) as usize;
                        if q_idx < 4 {
                            quartile_counts[q_idx] += 1;
                        }
                        if block.is_scheduled {
                            *sched_count += 1;
                        } else {
                            *unsched_count += 1;
                        }
                    }
                }
            }
        }
    }

    info!(
        "Completed bin population in {:.2}s: {} block-to-bin assignments",
        bin_pop_start.elapsed().as_secs_f64(),
        total_assignments
    );

    // Insert metadata
    let mut max_visible = 0i32;
    let mut total_visible = 0i64;

    for (_, _, ref block_ids, _, _, _) in &bin_data {
        let count = block_ids.len() as i32;
        max_visible = max_visible.max(count);
        total_visible += count as i64;
    }

    let mean_visible = if num_bins > 0 {
        Some(total_visible as f64 / num_bins as f64)
    } else {
        None
    };

    let etl_duration_ms = start_time.elapsed().as_millis() as i32;

    let metadata_sql = r#"
        INSERT INTO analytics.schedule_visibility_metadata (
            schedule_id, time_range_start_unix, time_range_end_unix,
            bin_duration_seconds, total_bins, total_blocks, blocks_with_visibility,
            priority_min, priority_max, max_visible_in_bin, mean_visible_per_bin,
            etl_duration_ms
        ) VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10, @P11, @P12)
    "#;

    let mut metadata_query = Query::new(metadata_sql);
    metadata_query.bind(schedule_id);
    metadata_query.bind(time_min);
    metadata_query.bind(time_max);
    metadata_query.bind(bin_duration as i32);
    metadata_query.bind(num_bins as i32);
    metadata_query.bind(rows.len() as i32);
    metadata_query.bind(blocks_with_visibility);
    metadata_query.bind(priority_min);
    metadata_query.bind(priority_max);
    metadata_query.bind(max_visible);
    metadata_query.bind(mean_visible);
    metadata_query.bind(etl_duration_ms);

    metadata_query
        .execute(&mut *conn)
        .await
        .map_err(|e| format!("Failed to insert visibility metadata: {e}"))?;

    // Batch insert bins
    let bins_inserted = bulk_insert_visibility_bins(&mut conn, schedule_id, &bin_data).await?;

    info!(
        "Populated {} visibility time bins for schedule_id={} in {}ms",
        bins_inserted, schedule_id, etl_duration_ms
    );

    Ok((1, bins_inserted))
}

/// Bulk insert visibility time bins.
async fn bulk_insert_visibility_bins(
    conn: &mut DbClient,
    schedule_id: i64,
    bin_data: &[(i64, i64, std::collections::HashSet<i64>, [i32; 4], i32, i32)],
) -> Result<usize, String> {
    if bin_data.is_empty() {
        return Ok(0);
    }

    // SQL Server parameter limit: ~2100 params
    // Each row uses 11 params, so max batch = 2100/11 ≈ 190
    const BATCH_SIZE: usize = 150;

    let mut total_inserted = 0;

    for (chunk_idx, chunk) in bin_data.chunks(BATCH_SIZE).enumerate() {
        let mut values_clauses = Vec::with_capacity(chunk.len());
        for (i, _) in chunk.iter().enumerate() {
            let base = i * 11;
            values_clauses.push(format!(
                "(@P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{}, @P{})",
                base + 1,
                base + 2,
                base + 3,
                base + 4,
                base + 5,
                base + 6,
                base + 7,
                base + 8,
                base + 9,
                base + 10,
                base + 11
            ));
        }

        let sql = format!(
            r#"
            INSERT INTO analytics.schedule_visibility_time_bins (
                schedule_id, bin_start_unix, bin_end_unix, bin_index,
                total_visible_count, priority_q1_count, priority_q2_count,
                priority_q3_count, priority_q4_count, scheduled_visible_count,
                unscheduled_visible_count
            ) VALUES {}
            "#,
            values_clauses.join(", ")
        );

        let mut query = Query::new(sql);

        for (
            bin_idx,
            (bin_start, bin_end, block_ids, quartile_counts, sched_count, unsched_count),
        ) in chunk.iter().enumerate()
        {
            let global_bin_idx = chunk_idx * BATCH_SIZE + bin_idx;
            query.bind(schedule_id);
            query.bind(*bin_start);
            query.bind(*bin_end);
            query.bind(global_bin_idx as i32);
            query.bind(block_ids.len() as i32);
            query.bind(quartile_counts[0]);
            query.bind(quartile_counts[1]);
            query.bind(quartile_counts[2]);
            query.bind(quartile_counts[3]);
            query.bind(*sched_count);
            query.bind(*unsched_count);
        }

        query
            .execute(&mut *conn)
            .await
            .map_err(|e| format!("Failed to insert visibility bins batch: {e}"))?;

        total_inserted += chunk.len();
    }

    Ok(total_inserted)
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
pub async fn delete_visibility_time_bins(schedule_id: i64) -> Result<usize, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    delete_visibility_time_bins_internal(&mut conn, schedule_id).await
}

/// Check if visibility time bins exist for a schedule.
pub async fn has_visibility_time_bins(schedule_id: i64) -> Result<bool, String> {
    let pool = pool::get_pool()?;
    let mut conn = pool
        .get()
        .await
        .map_err(|e| format!("Failed to get connection: {e}"))?;

    let sql = r#"
        SELECT COUNT(*) FROM analytics.schedule_visibility_metadata
        WHERE schedule_id = @P1
    "#;

    let mut query = Query::new(sql);
    query.bind(schedule_id);

    let stream = query
        .query(&mut *conn)
        .await
        .map_err(|e| format!("Failed to check visibility time bins: {e}"))?;

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
