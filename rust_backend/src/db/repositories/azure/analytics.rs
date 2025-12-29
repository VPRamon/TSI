// Azure analytics implementation removed — placeholder
#![allow(dead_code, unused_variables)]

use crate::api::{
    DistributionBlock, InsightsBlock, LightweightBlock, ScheduleId, ScheduleTimelineBlock,
    TrendsBlock, VisibilityMapData,
};

/// Placeholder indicating implementation removed.
pub(crate) fn _azure_analytics_todo() -> ! {
    todo!("Azure analytics implementation removed — TODO: re-implement")
}

pub async fn populate_schedule_analytics(_schedule_id: ScheduleId) -> Result<usize, String> {
    todo!("Azure placeholder: populate_schedule_analytics")
}

pub async fn delete_schedule_analytics(_schedule_id: ScheduleId) -> Result<usize, String> {
    todo!("Azure placeholder: delete_schedule_analytics")
}

pub async fn fetch_analytics_blocks_for_sky_map(
    _schedule_id: ScheduleId,
) -> Result<Vec<LightweightBlock>, String> {
    todo!("Azure placeholder: fetch_analytics_blocks_for_sky_map")
}

pub async fn fetch_analytics_blocks_for_distribution(
    _schedule_id: ScheduleId,
) -> Result<Vec<DistributionBlock>, String> {
    todo!("Azure placeholder: fetch_analytics_blocks_for_distribution")
}

pub async fn fetch_analytics_blocks_for_timeline(
    _schedule_id: ScheduleId,
) -> Result<Vec<ScheduleTimelineBlock>, String> {
    todo!("Azure placeholder: fetch_analytics_blocks_for_timeline")
}

pub async fn fetch_analytics_blocks_for_visibility_map(
    _schedule_id: ScheduleId,
) -> Result<VisibilityMapData, String> {
    todo!("Azure placeholder: fetch_analytics_blocks_for_visibility_map")
}

pub async fn fetch_analytics_blocks_for_insights(
    _schedule_id: ScheduleId,
) -> Result<Vec<InsightsBlock>, String> {
    todo!("Azure placeholder: fetch_analytics_blocks_for_insights")
}

pub async fn fetch_analytics_blocks_for_trends(
    _schedule_id: ScheduleId,
) -> Result<Vec<TrendsBlock>, String> {
    todo!("Azure placeholder: fetch_analytics_blocks_for_trends")
}

pub async fn has_analytics_data(_schedule_id: ScheduleId) -> Result<bool, String> {
    todo!("Azure placeholder: has_analytics_data")
}

pub async fn populate_summary_analytics(
    _schedule_id: ScheduleId,
    _n_bins: usize,
) -> Result<(), String> {
    todo!("Azure placeholder: populate_summary_analytics")
}

pub async fn has_summary_analytics(_schedule_id: ScheduleId) -> Result<bool, String> {
    todo!("Azure placeholder: has_summary_analytics")
}

pub async fn delete_summary_analytics(_schedule_id: ScheduleId) -> Result<usize, String> {
    todo!("Azure placeholder: delete_summary_analytics")
}

pub async fn populate_visibility_time_bins(
    _schedule_id: i64,
    _bin_duration_seconds: Option<i64>,
) -> Result<(usize, usize), String> {
    todo!("Azure placeholder: populate_visibility_time_bins")
}

pub async fn delete_visibility_time_bins(_schedule_id: i64) -> Result<usize, String> {
    todo!("Azure placeholder: delete_visibility_time_bins")
}

pub async fn has_visibility_time_bins(_schedule_id: i64) -> Result<bool, String> {
    todo!("Azure placeholder: has_visibility_time_bins")
}
