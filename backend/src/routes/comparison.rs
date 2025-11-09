/// Comparison endpoints for comparing two datasets
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

use crate::{
    models::schedule::{DatasetMetadata, SchedulingBlock},
    state::AppState,
};

/// Response for comparison upload
#[derive(Debug, Serialize)]
pub struct ComparisonUploadResponse {
    pub message: String,
    pub metadata: DatasetMetadata,
}

/// Upload a comparison dataset (CSV format)
pub async fn upload_comparison_csv(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Extract the file from multipart form data
    let mut file_data = Vec::new();
    let mut filename = String::from("comparison.csv");

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        if let Some(name) = field.file_name() {
            filename = name.to_string();
        }

        if let Ok(data) = field.bytes().await {
            file_data = data.to_vec();
        }
    }

    if file_data.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "No file data provided".to_string(),
        ));
    }

    // Write to temp file (Polars reads from file path)
    let temp_path = std::env::temp_dir().join(format!("comparison_{}", &filename));
    std::fs::write(&temp_path, &file_data).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to write temp file: {}", e),
        )
    })?;

    // Load CSV using the existing loader that reads from file path
    let blocks = crate::loaders::load_csv(&temp_path).map_err(|e| {
        let _ = std::fs::remove_file(&temp_path);
        (
            StatusCode::BAD_REQUEST,
            format!("Failed to parse CSV: {}", e),
        )
    })?;

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_path);

    if blocks.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "CSV file contains no data".to_string(),
        ));
    }

    // Store as comparison dataset
    let metadata = state
        .load_comparison_dataset(blocks, filename.clone())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    let response = ComparisonUploadResponse {
        message: format!("Comparison dataset '{}' uploaded successfully", filename),
        metadata,
    };

    Ok((StatusCode::OK, Json(response)))
}

/// Statistics for a single dataset
#[derive(Debug, Serialize)]
pub struct DatasetStats {
    pub filename: String,
    pub total_blocks: usize,
    pub scheduled_blocks: usize,
    pub unscheduled_blocks: usize,
    pub scheduling_rate: f64,
    pub total_requested_hours: f64,
    pub total_scheduled_hours: f64,
    pub total_visibility_hours: f64,
    pub utilization: f64,
    pub avg_priority: f64,
    pub avg_requested_hours: f64,
    pub avg_visibility_hours: f64,
}

impl DatasetStats {
    fn from_blocks(blocks: &[SchedulingBlock], filename: String) -> Self {
        let total_blocks = blocks.len();
        let scheduled_blocks = blocks.iter().filter(|b| b.scheduled_flag).count();
        let unscheduled_blocks = total_blocks - scheduled_blocks;

        let scheduling_rate = if total_blocks > 0 {
            (scheduled_blocks as f64 / total_blocks as f64) * 100.0
        } else {
            0.0
        };

        let total_requested_hours: f64 = blocks.iter().map(|b| b.requested_hours).sum();
        let total_scheduled_hours: f64 = blocks
            .iter()
            .filter(|b| b.scheduled_flag)
            .map(|b| b.requested_hours)
            .sum();
        let total_visibility_hours: f64 = blocks.iter().map(|b| b.total_visibility_hours).sum();

        let utilization = if total_visibility_hours > 0.0 {
            (total_scheduled_hours / total_visibility_hours) * 100.0
        } else {
            0.0
        };

        let avg_priority = if total_blocks > 0 {
            blocks.iter().map(|b| b.priority).sum::<f64>() / total_blocks as f64
        } else {
            0.0
        };

        let avg_requested_hours = if total_blocks > 0 {
            total_requested_hours / total_blocks as f64
        } else {
            0.0
        };

        let avg_visibility_hours = if total_blocks > 0 {
            total_visibility_hours / total_blocks as f64
        } else {
            0.0
        };

        Self {
            filename,
            total_blocks,
            scheduled_blocks,
            unscheduled_blocks,
            scheduling_rate,
            total_requested_hours,
            total_scheduled_hours,
            total_visibility_hours,
            utilization,
            avg_priority,
            avg_requested_hours,
            avg_visibility_hours,
        }
    }
}

/// Difference metrics between two datasets
#[derive(Debug, Serialize)]
pub struct DiffMetrics {
    pub blocks_added: usize,
    pub blocks_removed: usize,
    pub blocks_unchanged: usize,
    pub blocks_modified: usize,
    pub newly_scheduled: usize,
    pub newly_unscheduled: usize,
    pub scheduling_rate_diff: f64,
    pub utilization_diff: f64,
    pub avg_priority_diff: f64,
}

/// Block change information
#[derive(Debug, Serialize)]
pub struct BlockChange {
    pub scheduling_block_id: String,
    pub change_type: ChangeType,
    pub primary_scheduled: Option<bool>,
    pub comparison_scheduled: Option<bool>,
    pub primary_priority: Option<f64>,
    pub comparison_priority: Option<f64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    Unchanged,
}

/// Complete comparison response
#[derive(Debug, Serialize)]
pub struct ComparisonResponse {
    pub primary: DatasetStats,
    pub comparison: DatasetStats,
    pub diff: DiffMetrics,
    pub changes: Vec<BlockChange>,
}

/// Compare primary and comparison datasets
pub async fn get_comparison(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Get both datasets
    let (primary_blocks, primary_meta) = state
        .get_dataset()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or((
            StatusCode::NOT_FOUND,
            "No primary dataset loaded".to_string(),
        ))?;

    let (comparison_blocks, comparison_meta) = state
        .get_comparison_dataset()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?
        .ok_or((
            StatusCode::NOT_FOUND,
            "No comparison dataset loaded".to_string(),
        ))?;

    // Compute statistics for each dataset
    let primary_stats = DatasetStats::from_blocks(&primary_blocks, primary_meta.filename);
    let comparison_stats =
        DatasetStats::from_blocks(&comparison_blocks, comparison_meta.filename);

    // Create lookup maps for comparison
    let primary_map: HashMap<String, &SchedulingBlock> = primary_blocks
        .iter()
        .map(|b| (b.scheduling_block_id.clone(), b))
        .collect();

    let comparison_map: HashMap<String, &SchedulingBlock> = comparison_blocks
        .iter()
        .map(|b| (b.scheduling_block_id.clone(), b))
        .collect();

    // Find all unique block IDs
    let all_ids: HashSet<String> = primary_map
        .keys()
        .chain(comparison_map.keys())
        .cloned()
        .collect();

    // Analyze changes
    let mut changes = Vec::new();
    let mut blocks_added = 0;
    let mut blocks_removed = 0;
    let mut blocks_unchanged = 0;
    let mut blocks_modified = 0;
    let mut newly_scheduled = 0;
    let mut newly_unscheduled = 0;

    for id in all_ids {
        let primary_block = primary_map.get(&id);
        let comparison_block = comparison_map.get(&id);

        match (primary_block, comparison_block) {
            (Some(p), Some(c)) => {
                // Block exists in both datasets
                let scheduling_changed = p.scheduled_flag != c.scheduled_flag;
                let priority_changed = (p.priority - c.priority).abs() > 0.001;

                if scheduling_changed || priority_changed {
                    blocks_modified += 1;

                    if scheduling_changed {
                        if c.scheduled_flag && !p.scheduled_flag {
                            newly_scheduled += 1;
                        } else if !c.scheduled_flag && p.scheduled_flag {
                            newly_unscheduled += 1;
                        }
                    }

                    changes.push(BlockChange {
                        scheduling_block_id: id.clone(),
                        change_type: ChangeType::Modified,
                        primary_scheduled: Some(p.scheduled_flag),
                        comparison_scheduled: Some(c.scheduled_flag),
                        primary_priority: Some(p.priority),
                        comparison_priority: Some(c.priority),
                    });
                } else {
                    blocks_unchanged += 1;
                }
            }
            (None, Some(c)) => {
                // Block only in comparison (added)
                blocks_added += 1;
                changes.push(BlockChange {
                    scheduling_block_id: id.clone(),
                    change_type: ChangeType::Added,
                    primary_scheduled: None,
                    comparison_scheduled: Some(c.scheduled_flag),
                    primary_priority: None,
                    comparison_priority: Some(c.priority),
                });
            }
            (Some(p), None) => {
                // Block only in primary (removed)
                blocks_removed += 1;
                changes.push(BlockChange {
                    scheduling_block_id: id.clone(),
                    change_type: ChangeType::Removed,
                    primary_scheduled: Some(p.scheduled_flag),
                    comparison_scheduled: None,
                    primary_priority: Some(p.priority),
                    comparison_priority: None,
                });
            }
            (None, None) => unreachable!(),
        }
    }

    // Compute diff metrics
    let diff = DiffMetrics {
        blocks_added,
        blocks_removed,
        blocks_unchanged,
        blocks_modified,
        newly_scheduled,
        newly_unscheduled,
        scheduling_rate_diff: comparison_stats.scheduling_rate - primary_stats.scheduling_rate,
        utilization_diff: comparison_stats.utilization - primary_stats.utilization,
        avg_priority_diff: comparison_stats.avg_priority - primary_stats.avg_priority,
    };

    let response = ComparisonResponse {
        primary: primary_stats,
        comparison: comparison_stats,
        diff,
        changes,
    };

    Ok(Json(response))
}

/// Clear the comparison dataset
pub async fn clear_comparison(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    state
        .clear_comparison_dataset()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "Comparison dataset cleared successfully"
        })),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::schedule::{PriorityBin, VisibilityPeriod};

    fn create_test_block(id: &str, scheduled: bool, priority: f64) -> SchedulingBlock {
        SchedulingBlock {
            scheduling_block_id: id.to_string(),
            priority,
            min_observation_time_in_sec: 1200.0,
            requested_duration_sec: 3600.0,
            fixed_start_time: None,
            fixed_stop_time: None,
            dec_in_deg: 0.0,
            ra_in_deg: 0.0,
            min_azimuth_angle_in_deg: 0.0,
            max_azimuth_angle_in_deg: 360.0,
            min_elevation_angle_in_deg: 60.0,
            max_elevation_angle_in_deg: 90.0,
            scheduled_period_start: if scheduled { Some(61892.0) } else { None },
            scheduled_period_stop: if scheduled { Some(61893.0) } else { None },
            visibility: vec![VisibilityPeriod {
                start: 61892.0,
                stop: 61893.0,
            }],
            num_visibility_periods: 1,
            total_visibility_hours: 24.0,
            priority_bin: PriorityBin::MediumHigh,
            scheduled_flag: scheduled,
            requested_hours: 1.0,
            elevation_range_deg: 30.0,
        }
    }

    #[test]
    fn test_dataset_stats() {
        let blocks = vec![
            create_test_block("block1", true, 10.0),
            create_test_block("block2", false, 20.0),
            create_test_block("block3", true, 15.0),
        ];

        let stats = DatasetStats::from_blocks(&blocks, "test.csv".to_string());

        assert_eq!(stats.total_blocks, 3);
        assert_eq!(stats.scheduled_blocks, 2);
        assert_eq!(stats.unscheduled_blocks, 1);
        assert!((stats.scheduling_rate - 66.666).abs() < 0.1);
        assert_eq!(stats.total_requested_hours, 3.0);
        assert_eq!(stats.total_scheduled_hours, 2.0);
        assert_eq!(stats.total_visibility_hours, 72.0);
        assert!((stats.avg_priority - 15.0).abs() < 0.1);
    }
}
