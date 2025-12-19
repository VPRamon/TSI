#![allow(clippy::redundant_closure)]
#![allow(clippy::len_zero)]

use crate::db::models::{Period, ScheduleTimelineBlock, ScheduleTimelineData};
use pyo3::prelude::*;
use std::collections::HashSet;
use tokio::runtime::Runtime;

// Import the global repository accessor
use crate::db::repository::VisualizationRepository;
use crate::python::database::get_repository;

/// Compute schedule timeline data with statistics and metadata.
/// This function takes the raw blocks and computes everything needed for visualization.
pub fn compute_schedule_timeline_data(
    blocks: Vec<ScheduleTimelineBlock>,
    dark_periods: Vec<Period>,
) -> Result<ScheduleTimelineData, String> {
    if blocks.is_empty() {
        return Ok(ScheduleTimelineData {
            blocks: vec![],
            priority_min: 0.0,
            priority_max: 10.0,
            total_count: 0,
            scheduled_count: 0,
            unique_months: vec![],
            dark_periods: dark_periods
                .into_iter()
                .map(|p| (p.start.value(), p.stop.value()))
                .collect(),
        });
    }

    // Compute statistics
    let mut priority_min = f64::MAX;
    let mut priority_max = f64::MIN;
    let mut unique_months = HashSet::new();

    for block in &blocks {
        priority_min = priority_min.min(block.priority);
        priority_max = priority_max.max(block.priority);

        // Convert MJD to year-month string
        // MJD 0 = November 17, 1858
        // Unix epoch (Jan 1, 1970) = MJD 40587
        let unix_timestamp = (block.scheduled_start_mjd - 40587.0) * 86400.0;
        let datetime = chrono::DateTime::from_timestamp(unix_timestamp as i64, 0);

        if let Some(dt) = datetime {
            let month_label = dt.format("%Y-%m").to_string();
            unique_months.insert(month_label);
        }
    }

    // Sort unique months
    let mut sorted_months: Vec<String> = unique_months.into_iter().collect();
    sorted_months.sort();

    // Handle edge cases
    if !priority_min.is_finite() {
        priority_min = 0.0;
    }
    if !priority_max.is_finite() {
        priority_max = 10.0;
    }
    if priority_max <= priority_min {
        priority_max = priority_min + 1.0;
    }

    Ok(ScheduleTimelineData {
        blocks: blocks.clone(),
        priority_min,
        priority_max,
        total_count: blocks.len(),
        scheduled_count: blocks.len(),
        unique_months: sorted_months,
        dark_periods: dark_periods
            .into_iter()
            .map(|p| (p.start.value(), p.stop.value()))
            .collect(),
    })
}

/// Get complete schedule timeline data with computed statistics and metadata.
/// This function orchestrates fetching blocks and dark periods from the database
/// and computing the timeline data.
///
/// Uses the analytics table for optimal performance when available.
pub async fn get_schedule_timeline_data(schedule_id: i64) -> Result<ScheduleTimelineData, String> {
    // Get the initialized repository
    let repo = get_repository().map_err(|e| format!("Failed to get repository: {}", e))?;

    // Fetch timeline blocks from visualization repository
    let blocks = repo
        .fetch_schedule_timeline_blocks(schedule_id)
        .await
        .map_err(|e| format!("Failed to fetch timeline blocks: {}", e))?;

    // Note: dark_periods is currently Azure-specific. For LocalRepository,
    // this will return an empty vec which is acceptable.
    let dark_periods = vec![]; // TODO: Add dark periods support to LocalRepository

    compute_schedule_timeline_data(blocks, dark_periods)
}

/// Get complete schedule timeline data with computed statistics and metadata.
/// This is the main function for the schedule timeline feature, computing all statistics
/// on the Rust side for maximum performance.
#[pyfunction]
pub fn py_get_schedule_timeline_data(schedule_id: i64) -> PyResult<ScheduleTimelineData> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;

    runtime
        .block_on(get_schedule_timeline_data(schedule_id))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_timeline_data_empty() {
        let result = compute_schedule_timeline_data(vec![], vec![]).unwrap();
        assert_eq!(result.total_count, 0);
        assert_eq!(result.scheduled_count, 0);
        assert_eq!(result.unique_months.len(), 0);
    }

    #[test]
    fn test_compute_timeline_data() {
        let blocks = vec![
            ScheduleTimelineBlock {
                scheduling_block_id: 1,
                original_block_id: "SB001".to_string(),
                priority: 5.0,
                scheduled_start_mjd: 59000.0, // 2020-05-10
                scheduled_stop_mjd: 59001.0,
                ra_deg: 180.0,
                dec_deg: 45.0,
                requested_hours: 1.0,
                total_visibility_hours: 5.0,
                num_visibility_periods: 3,
            },
            ScheduleTimelineBlock {
                scheduling_block_id: 2,
                original_block_id: "SB002".to_string(),
                priority: 8.0,
                scheduled_start_mjd: 59030.0, // 2020-06-09
                scheduled_stop_mjd: 59031.0,
                ra_deg: 200.0,
                dec_deg: -30.0,
                requested_hours: 2.0,
                total_visibility_hours: 8.0,
                num_visibility_periods: 4,
            },
        ];

        let result = compute_schedule_timeline_data(blocks, vec![]).unwrap();

        assert_eq!(result.total_count, 2);
        assert_eq!(result.scheduled_count, 2);
        assert_eq!(result.priority_min, 5.0);
        assert_eq!(result.priority_max, 8.0);
        assert!(result.unique_months.len() >= 1); // At least one month
    }
}
