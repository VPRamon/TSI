use chrono::{DateTime, Utc};
use diesel::prelude::*;
use qtty::{Degrees, Hours};
use serde_json::Value;

use super::schema::{
    schedule_block_analytics, schedule_blocks, schedule_summary_analytics,
    schedule_validation_results, schedules,
};

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = schedules)]
#[allow(dead_code)] // Some fields used only for database operations
pub struct ScheduleRow {
    pub schedule_id: i64,
    pub schedule_name: String,
    pub checksum: String,
    pub uploaded_at: DateTime<Utc>,
    pub dark_periods_json: Value,
    pub possible_periods_json: Value,
    pub raw_schedule_json: Option<Value>,
    pub schedule_period_json: Value,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schedules)]
pub struct NewScheduleRow {
    pub schedule_name: String,
    pub checksum: String,
    pub dark_periods_json: Value,
    pub possible_periods_json: Value,
    pub raw_schedule_json: Option<Value>,
    pub schedule_period_json: Value,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = schedule_blocks)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[allow(dead_code)] // Some fields used only for database operations
pub struct ScheduleBlockRow {
    pub scheduling_block_id: i64,
    pub schedule_id: i64,
    pub source_block_id: i64,
    pub original_block_id: Option<String>,
    pub priority: f64,
    pub requested_duration_sec: i32,
    pub min_observation_sec: i32,
    pub target_ra_deg: Degrees,
    pub target_dec_deg: Degrees,
    pub min_altitude_deg: Option<Degrees>,
    pub max_altitude_deg: Option<Degrees>,
    pub min_azimuth_deg: Option<Degrees>,
    pub max_azimuth_deg: Option<Degrees>,
    pub constraint_start_mjd: Option<f64>,
    pub constraint_stop_mjd: Option<f64>,
    pub visibility_periods_json: Value,
    pub scheduled_periods_json: Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schedule_blocks)]
pub struct NewScheduleBlockRow {
    pub schedule_id: i64,
    pub source_block_id: i64,
    pub original_block_id: Option<String>,
    pub priority: f64,
    pub requested_duration_sec: i32,
    pub min_observation_sec: i32,
    pub target_ra_deg: Degrees,
    pub target_dec_deg: Degrees,
    pub min_altitude_deg: Option<Degrees>,
    pub max_altitude_deg: Option<Degrees>,
    pub min_azimuth_deg: Option<Degrees>,
    pub max_azimuth_deg: Option<Degrees>,
    pub constraint_start_mjd: Option<f64>,
    pub constraint_stop_mjd: Option<f64>,
    pub visibility_periods_json: Value,
    pub scheduled_periods_json: Value,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = schedule_block_analytics)]
// Analytics table row structure (for future use)
#[allow(dead_code)]
pub struct ScheduleBlockAnalyticsRow {
    pub schedule_id: i64,
    pub scheduling_block_id: i64,
    pub priority_bucket: i16,
    pub requested_hours: Hours,
    pub total_visibility_hours: Hours,
    pub num_visibility_periods: i32,
    pub elevation_range_deg: Option<Degrees>,
    pub scheduled: bool,
    pub scheduled_start_mjd: Option<f64>,
    pub scheduled_stop_mjd: Option<f64>,
    pub validation_impossible: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schedule_block_analytics)]
pub struct NewScheduleBlockAnalyticsRow {
    pub schedule_id: i64,
    pub scheduling_block_id: i64,
    pub priority_bucket: i16,
    pub requested_hours: Hours,
    pub total_visibility_hours: Hours,
    pub num_visibility_periods: i32,
    pub elevation_range_deg: Option<Degrees>,
    pub scheduled: bool,
    pub scheduled_start_mjd: Option<f64>,
    pub scheduled_stop_mjd: Option<f64>,
    pub validation_impossible: bool,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = schedule_summary_analytics)]
// Summary analytics table row structure (for future use)
#[allow(dead_code)]
pub struct ScheduleSummaryAnalyticsRow {
    pub schedule_id: i64,
    pub total_blocks: i32,
    pub scheduled_blocks: i32,
    pub unscheduled_blocks: i32,
    pub impossible_blocks: i32,
    pub scheduling_rate: f64,
    pub priority_mean: Option<f64>,
    pub priority_median: Option<f64>,
    pub priority_scheduled_mean: Option<f64>,
    pub priority_unscheduled_mean: Option<f64>,
    pub visibility_total_hours: Hours,
    pub requested_mean_hours: Option<Hours>,
    pub gap_count: Option<i32>,
    pub gap_mean_hours: Option<Hours>,
    pub gap_median_hours: Option<Hours>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schedule_summary_analytics)]
pub struct NewScheduleSummaryAnalyticsRow {
    pub schedule_id: i64,
    pub total_blocks: i32,
    pub scheduled_blocks: i32,
    pub unscheduled_blocks: i32,
    pub impossible_blocks: i32,
    pub scheduling_rate: f64,
    pub priority_mean: Option<f64>,
    pub priority_median: Option<f64>,
    pub priority_scheduled_mean: Option<f64>,
    pub priority_unscheduled_mean: Option<f64>,
    pub visibility_total_hours: Hours,
    pub requested_mean_hours: Option<Hours>,
    pub gap_count: Option<i32>,
    pub gap_mean_hours: Option<Hours>,
    pub gap_median_hours: Option<Hours>,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = schedule_validation_results)]
#[allow(dead_code)] // Some fields used only for database operations
pub struct ScheduleValidationResultRow {
    pub validation_id: i64,
    pub schedule_id: i64,
    pub scheduling_block_id: i64,
    pub status: String,
    pub issue_type: Option<String>,
    pub issue_category: Option<String>,
    pub criticality: Option<String>,
    pub field_name: Option<String>,
    pub current_value: Option<String>,
    pub expected_value: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = schedule_validation_results)]
pub struct NewScheduleValidationResultRow {
    pub schedule_id: i64,
    pub scheduling_block_id: i64,
    pub status: String,
    pub issue_type: Option<String>,
    pub issue_category: Option<String>,
    pub criticality: Option<String>,
    pub field_name: Option<String>,
    pub current_value: Option<String>,
    pub expected_value: Option<String>,
    pub description: Option<String>,
}
