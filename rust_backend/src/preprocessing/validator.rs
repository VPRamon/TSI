use polars::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::domain::SchedulingBlock;

/// Validation result containing errors, warnings and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub stats: ValidationStats,
}

/// Statistics about the validated data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    pub total_blocks: usize,
    pub scheduled_blocks: usize,
    pub unscheduled_blocks: usize,
    pub missing_coordinates: usize,
    pub missing_constraints: usize,
    pub duplicate_ids: usize,
    pub invalid_priorities: usize,
    pub invalid_durations: usize,
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            stats: ValidationStats::default(),
        }
    }
    
    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }
    
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

impl Default for ValidationStats {
    fn default() -> Self {
        Self {
            total_blocks: 0,
            scheduled_blocks: 0,
            unscheduled_blocks: 0,
            missing_coordinates: 0,
            missing_constraints: 0,
            duplicate_ids: 0,
            invalid_priorities: 0,
            invalid_durations: 0,
        }
    }
}

/// Validator for scheduling data
pub struct ScheduleValidator;

impl ScheduleValidator {
    /// Validate a list of scheduling blocks
    pub fn validate_blocks(blocks: &[SchedulingBlock]) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        result.stats.total_blocks = blocks.len();
        
        // Check for duplicates
        result.stats.duplicate_ids = Self::check_duplicates(blocks, &mut result);
        
        // Validate each block
        for block in blocks {
            Self::validate_block(block, &mut result);
        }
        
        result
    }
    
    /// Validate a Polars DataFrame
    pub fn validate_dataframe(df: &DataFrame) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        result.stats.total_blocks = df.height();
        
        // Check required columns
        let required_cols = vec![
            "schedulingBlockId",
            "priority",
            "requestedDurationSec",
        ];
        
        for col in required_cols {
            if df.column(col).is_err() {
                result.add_error(format!("Missing required column: {}", col));
            }
        }
        
        if !result.is_valid {
            return result;
        }
        
        // Count scheduled blocks
        if let Ok(scheduled_flag) = df.column("scheduled_flag") {
            if let Ok(bool_series) = scheduled_flag.bool() {
                result.stats.scheduled_blocks = bool_series.sum().unwrap_or(0) as usize;
                result.stats.unscheduled_blocks = result.stats.total_blocks - result.stats.scheduled_blocks;
            }
        }
        
        // Check for missing coordinates
        if let Ok(ra_col) = df.column("raInDeg") {
            if let Ok(f64_series) = ra_col.f64() {
                result.stats.missing_coordinates = f64_series.null_count();
            }
        }
        
        // Validate priorities
        if let Ok(priority_col) = df.column("priority") {
            if let Ok(f64_series) = priority_col.f64() {
                for val in f64_series.into_iter().flatten() {
                    if val < 0.0 || val > 20.0 {
                        result.stats.invalid_priorities += 1;
                        if result.stats.invalid_priorities <= 5 {
                            result.add_warning(format!("Invalid priority value: {}", val));
                        }
                    }
                }
                
                if result.stats.invalid_priorities > 5 {
                    result.add_warning(format!(
                        "Total invalid priorities: {} (showing first 5)",
                        result.stats.invalid_priorities
                    ));
                }
            }
        }
        
        // Validate durations
        if let Ok(duration_col) = df.column("requestedDurationSec") {
            if let Ok(f64_series) = duration_col.f64() {
                for val in f64_series.into_iter().flatten() {
                    if val <= 0.0 {
                        result.stats.invalid_durations += 1;
                        if result.stats.invalid_durations <= 5 {
                            result.add_error(format!("Invalid duration (must be > 0): {}", val));
                        }
                    }
                }
                
                if result.stats.invalid_durations > 5 {
                    result.add_error(format!(
                        "Total invalid durations: {} (showing first 5)",
                        result.stats.invalid_durations
                    ));
                }
            }
        }
        
        // Check for duplicate IDs
        if let Ok(id_col) = df.column("schedulingBlockId") {
            if let Ok(str_series) = id_col.str() {
                let unique_count = str_series.n_unique().unwrap_or(0);
                let total_count = str_series.len();
                result.stats.duplicate_ids = total_count - unique_count;
                
                if result.stats.duplicate_ids > 0 {
                    result.add_error(format!(
                        "Found {} duplicate scheduling block IDs",
                        result.stats.duplicate_ids
                    ));
                }
            }
        }
        
        result
    }
    
    fn validate_block(block: &SchedulingBlock, result: &mut ValidationResult) {
        // Count scheduled/unscheduled
        if block.is_scheduled() {
            result.stats.scheduled_blocks += 1;
        } else {
            result.stats.unscheduled_blocks += 1;
        }
        
        // Check coordinates
        if block.ra_in_deg.is_none() || block.dec_in_deg.is_none() {
            result.stats.missing_coordinates += 1;
        }
        
        // Check constraints
        if block.min_elevation_angle_in_deg.is_none() || block.max_elevation_angle_in_deg.is_none() {
            result.stats.missing_constraints += 1;
        }
        
        // Validate priority
        if block.priority < 0.0 || block.priority > 20.0 {
            result.stats.invalid_priorities += 1;
            result.add_warning(format!(
                "Block {} has invalid priority: {}",
                block.scheduling_block_id, block.priority
            ));
        }
        
        // Validate duration
        if block.requested_duration_sec <= 0.0 {
            result.stats.invalid_durations += 1;
            result.add_error(format!(
                "Block {} has invalid duration: {}",
                block.scheduling_block_id, block.requested_duration_sec
            ));
        }
        
        // Validate elevation range
        if let (Some(min_el), Some(max_el)) = (block.min_elevation_angle_in_deg, block.max_elevation_angle_in_deg) {
            if min_el < 0.0 || min_el > 90.0 {
                result.add_warning(format!(
                    "Block {} has invalid min elevation: {}",
                    block.scheduling_block_id, min_el
                ));
            }
            if max_el < 0.0 || max_el > 90.0 {
                result.add_warning(format!(
                    "Block {} has invalid max elevation: {}",
                    block.scheduling_block_id, max_el
                ));
            }
            if min_el >= max_el {
                result.add_error(format!(
                    "Block {} has min elevation >= max elevation: {} >= {}",
                    block.scheduling_block_id, min_el, max_el
                ));
            }
        }
    }
    
    fn check_duplicates(blocks: &[SchedulingBlock], result: &mut ValidationResult) -> usize {
        use std::collections::HashSet;
        
        let mut seen = HashSet::new();
        let mut duplicates = 0;
        
        for block in blocks {
            if !seen.insert(&block.scheduling_block_id) {
                duplicates += 1;
                if duplicates <= 5 {
                    result.add_error(format!(
                        "Duplicate scheduling block ID: {}",
                        block.scheduling_block_id
                    ));
                }
            }
        }
        
        if duplicates > 5 {
            result.add_error(format!(
                "Total duplicate IDs: {} (showing first 5)",
                duplicates
            ));
        }
        
        duplicates
    }
}

#[cfg(all(test, not(feature = "extension-module")))]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_valid_block() {
        let block = SchedulingBlock {
            scheduling_block_id: "test-001".to_string(),
            priority: 10.0,
            requested_duration_sec: 3600.0,
            min_observation_time_sec: Some(1800.0),
            fixed_start_time: None,
            fixed_stop_time: None,
            ra_in_deg: Some(180.0),
            dec_in_deg: Some(45.0),
            min_azimuth_angle_in_deg: Some(0.0),
            max_azimuth_angle_in_deg: Some(360.0),
            min_elevation_angle_in_deg: Some(30.0),
            max_elevation_angle_in_deg: Some(80.0),
            scheduled_start: Some(59580.0),
            scheduled_stop: Some(59580.5),
            visibility_periods: vec![],
        };
        
        let result = ScheduleValidator::validate_blocks(&[block]);
        assert!(result.is_valid);
        assert_eq!(result.errors.len(), 0);
        assert_eq!(result.stats.total_blocks, 1);
        assert_eq!(result.stats.scheduled_blocks, 1);
    }
    
    #[test]
    fn test_validate_invalid_block() {
        let block = SchedulingBlock {
            scheduling_block_id: "test-002".to_string(),
            priority: 25.0,  // Invalid: > 20
            requested_duration_sec: -100.0,  // Invalid: negative
            min_observation_time_sec: None,
            fixed_start_time: None,
            fixed_stop_time: None,
            ra_in_deg: None,  // Missing
            dec_in_deg: None,  // Missing
            min_azimuth_angle_in_deg: Some(0.0),
            max_azimuth_angle_in_deg: Some(360.0),
            min_elevation_angle_in_deg: Some(80.0),
            max_elevation_angle_in_deg: Some(30.0),  // Invalid: max < min
            scheduled_start: None,
            scheduled_stop: None,
            visibility_periods: vec![],
        };
        
        let result = ScheduleValidator::validate_blocks(&[block]);
        assert!(!result.is_valid);
        assert!(result.errors.len() > 0);
        assert_eq!(result.stats.invalid_priorities, 1);
        assert_eq!(result.stats.invalid_durations, 1);
        assert_eq!(result.stats.missing_coordinates, 1);
    }
}
