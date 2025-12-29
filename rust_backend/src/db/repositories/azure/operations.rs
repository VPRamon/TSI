// Azure operations implementation removed — placeholder
#![allow(dead_code, unused_variables)]

use crate::api::{
    CompareBlock, DistributionBlock, InsightsBlock, LightweightBlock, Period, Schedule, ScheduleId,
    ScheduleInfo, ScheduleTimelineBlock, SchedulingBlock, TrendsBlock, VisibilityMapData,
};

/// Placeholder indicating implementation removed.
pub(crate) fn _azure_operations_todo() -> ! {
    todo!("Azure operations implementation removed — TODO: re-implement")
}

pub async fn health_check() -> Result<bool, String> {
    todo!("Azure placeholder: health_check")
}

pub async fn store_schedule(_schedule: &Schedule) -> Result<ScheduleInfo, String> {
    todo!("Azure placeholder: store_schedule")
}

pub async fn get_schedule(
    _schedule_id: Option<ScheduleId>,
    _schedule_name: Option<&str>,
) -> Result<Schedule, String> {
    todo!("Azure placeholder: get_schedule")
}

pub async fn list_schedules() -> Result<Vec<ScheduleInfo>, String> {
    todo!("Azure placeholder: list_schedules")
}

pub async fn get_scheduling_block(_sb_id: i64) -> Result<SchedulingBlock, String> {
    todo!("Azure placeholder: get_scheduling_block")
}

pub async fn get_blocks_for_schedule(
    _schedule_id: ScheduleId,
) -> Result<Vec<SchedulingBlock>, String> {
    todo!("Azure placeholder: get_blocks_for_schedule")
}

pub async fn fetch_dark_periods_public(
    _schedule_id: Option<ScheduleId>,
) -> Result<Vec<Period>, String> {
    todo!("Azure placeholder: fetch_dark_periods_public")
}

pub async fn fetch_possible_periods(_schedule_id: ScheduleId) -> Result<Vec<Period>, String> {
    todo!("Azure placeholder: fetch_possible_periods")
}

pub async fn fetch_lightweight_blocks(_schedule_id: i64) -> Result<Vec<LightweightBlock>, String> {
    todo!("Azure placeholder: fetch_lightweight_blocks")
}

pub async fn fetch_distribution_blocks(
    _schedule_id: i64,
) -> Result<Vec<DistributionBlock>, String> {
    todo!("Azure placeholder: fetch_distribution_blocks")
}

pub async fn fetch_insights_blocks(_schedule_id: i64) -> Result<Vec<InsightsBlock>, String> {
    todo!("Azure placeholder: fetch_insights_blocks")
}

pub async fn fetch_trends_blocks(_schedule_id: i64) -> Result<Vec<TrendsBlock>, String> {
    todo!("Azure placeholder: fetch_trends_blocks")
}

pub async fn fetch_visibility_map_data(
    _schedule_id: ScheduleId,
) -> Result<VisibilityMapData, String> {
    todo!("Azure placeholder: fetch_visibility_map_data")
}

pub async fn get_schedule_time_range(_schedule_id: ScheduleId) -> Result<Option<Period>, String> {
    todo!("Azure placeholder: get_schedule_time_range")
}

pub async fn fetch_schedule_timeline_blocks(
    _schedule_id: ScheduleId,
) -> Result<Vec<ScheduleTimelineBlock>, String> {
    todo!("Azure placeholder: fetch_schedule_timeline_blocks")
}

pub async fn fetch_compare_blocks(_schedule_id: ScheduleId) -> Result<Vec<CompareBlock>, String> {
    todo!("Azure placeholder: fetch_compare_blocks")
}
