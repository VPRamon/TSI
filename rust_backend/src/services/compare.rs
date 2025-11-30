use crate::db::models::{CompareBlock, CompareData, CompareStats, SchedulingChange};
use crate::db::operations;
use pyo3::prelude::*;
use std::collections::{HashMap, HashSet};
use tokio::runtime::Runtime;

/// Compute statistics for a set of blocks.
fn compute_stats(blocks: &[CompareBlock]) -> CompareStats {
    let scheduled_blocks: Vec<&CompareBlock> = blocks.iter().filter(|b| b.scheduled).collect();
    
    let scheduled_count = scheduled_blocks.len();
    let unscheduled_count = blocks.len() - scheduled_count;
    
    if scheduled_blocks.is_empty() {
        return CompareStats {
            scheduled_count,
            unscheduled_count,
            total_priority: 0.0,
            mean_priority: 0.0,
            median_priority: 0.0,
            total_hours: 0.0,
        };
    }
    
    let priorities: Vec<f64> = scheduled_blocks.iter().map(|b| b.priority).collect();
    let total_priority: f64 = priorities.iter().sum();
    let mean_priority = total_priority / scheduled_count as f64;
    
    let mut sorted_priorities = priorities.clone();
    sorted_priorities.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_priority = if scheduled_count % 2 == 0 {
        (sorted_priorities[scheduled_count / 2 - 1] + sorted_priorities[scheduled_count / 2]) / 2.0
    } else {
        sorted_priorities[scheduled_count / 2]
    };
    
    let total_hours: f64 = scheduled_blocks.iter().map(|b| b.requested_hours).sum();
    
    CompareStats {
        scheduled_count,
        unscheduled_count,
        total_priority,
        mean_priority,
        median_priority,
        total_hours,
    }
}

/// Compute comparison data from two sets of blocks.
/// This function takes blocks from both schedules and computes all necessary statistics and changes.
pub fn compute_compare_data(
    current_blocks: Vec<CompareBlock>,
    comparison_blocks: Vec<CompareBlock>,
    current_name: String,
    comparison_name: String,
) -> Result<CompareData, String> {
    // Create ID sets for comparison
    let current_ids: HashSet<String> = current_blocks
        .iter()
        .map(|b| b.scheduling_block_id.clone())
        .collect();
    
    let comparison_ids: HashSet<String> = comparison_blocks
        .iter()
        .map(|b| b.scheduling_block_id.clone())
        .collect();
    
    // Find differences
    let only_in_current: Vec<String> = current_ids
        .difference(&comparison_ids)
        .cloned()
        .collect();
    
    let only_in_comparison: Vec<String> = comparison_ids
        .difference(&current_ids)
        .cloned()
        .collect();
    
    let common_ids: Vec<String> = current_ids
        .intersection(&comparison_ids)
        .cloned()
        .collect();
    
    // Create maps for efficient lookup
    let current_map: HashMap<String, &CompareBlock> = current_blocks
        .iter()
        .map(|b| (b.scheduling_block_id.clone(), b))
        .collect();
    
    let comparison_map: HashMap<String, &CompareBlock> = comparison_blocks
        .iter()
        .map(|b| (b.scheduling_block_id.clone(), b))
        .collect();
    
    // Find scheduling changes
    let mut scheduling_changes = Vec::new();
    for id in &common_ids {
        if let (Some(current_block), Some(comparison_block)) = 
            (current_map.get(id), comparison_map.get(id)) {
            
            // Newly scheduled: was unscheduled in current, scheduled in comparison
            if !current_block.scheduled && comparison_block.scheduled {
                scheduling_changes.push(SchedulingChange {
                    scheduling_block_id: id.clone(),
                    priority: comparison_block.priority,
                    change_type: "newly_scheduled".to_string(),
                });
            }
            
            // Newly unscheduled: was scheduled in current, unscheduled in comparison
            if current_block.scheduled && !comparison_block.scheduled {
                scheduling_changes.push(SchedulingChange {
                    scheduling_block_id: id.clone(),
                    priority: current_block.priority,
                    change_type: "newly_unscheduled".to_string(),
                });
            }
        }
    }
    
    // Compute statistics
    let current_stats = compute_stats(&current_blocks);
    let comparison_stats = compute_stats(&comparison_blocks);
    
    Ok(CompareData {
        current_blocks,
        comparison_blocks,
        current_stats,
        comparison_stats,
        common_ids,
        only_in_current,
        only_in_comparison,
        scheduling_changes,
        current_name,
        comparison_name,
    })
}

/// Get comparison data from the database by fetching both schedules.
pub async fn get_compare_data(
    current_schedule_id: i64,
    comparison_schedule_id: i64,
    current_name: String,
    comparison_name: String,
) -> Result<CompareData, String> {
    // Fetch blocks from both schedules
    let current_blocks = operations::fetch_compare_blocks(current_schedule_id).await?;
    let comparison_blocks = operations::fetch_compare_blocks(comparison_schedule_id).await?;
    
    compute_compare_data(current_blocks, comparison_blocks, current_name, comparison_name)
}

/// Python binding for get_compare_data.
/// Fetches and compares two schedules from the database.
#[pyfunction]
pub fn py_get_compare_data(
    current_schedule_id: i64,
    comparison_schedule_id: i64,
    current_name: String,
    comparison_name: String,
) -> PyResult<CompareData> {
    let runtime = Runtime::new().map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
            "Failed to create async runtime: {}",
            e
        ))
    })?;
    
    runtime
        .block_on(get_compare_data(
            current_schedule_id,
            comparison_schedule_id,
            current_name,
            comparison_name,
        ))
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
}

/// Python binding to compute comparison from already-loaded block lists.
/// This is useful when one schedule is from a file upload rather than the database.
#[pyfunction]
pub fn py_compute_compare_data(
    current_blocks: Vec<CompareBlock>,
    comparison_blocks: Vec<CompareBlock>,
    current_name: String,
    comparison_name: String,
) -> PyResult<CompareData> {
    compute_compare_data(current_blocks, comparison_blocks, current_name, comparison_name)
        .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e))
}
