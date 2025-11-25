//! Schedule validation with detailed error and warning reporting.
//!
//! This module validates scheduling block data for completeness, consistency,
//! and correctness. It checks for missing required fields, invalid values,
//! duplicate IDs, and other data quality issues.

use polars::prelude::*;
use serde::{Deserialize, Serialize};

use crate::core::domain::SchedulingBlock;

/// Comprehensive validation result with categorized issues and statistics.
///
/// Contains validation status, lists of errors and warnings, and summary
/// statistics about the validated dataset. Errors make `is_valid` false,
/// while warnings are informational but don't fail validation.
///
/// # Fields
///
/// * `is_valid` - `false` if any errors were found, `true` otherwise
/// * `errors` - Critical issues that prevent processing (e.g., missing required fields)
/// * `warnings` - Non-critical issues that should be reviewed (e.g., unusual priority values)
/// * `stats` - Summary statistics about the validated data
///
/// # Examples
///
/// ```
/// use tsi_rust::preprocessing::validator::{ValidationResult, ScheduleValidator};
///
/// let mut result = ValidationResult::new();
/// assert!(result.is_valid);
///
/// result.add_error("Missing required field".to_string());
/// assert!(!result.is_valid);
/// assert_eq!(result.errors.len(), 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub stats: ValidationStats,
}

/// Summary statistics computed during validation.
///
/// Provides counts of various data characteristics and quality metrics,
/// including scheduling status, missing data, and detected issues.
///
/// # Fields
///
/// * `total_blocks` - Total number of scheduling blocks validated
/// * `scheduled_blocks` - Count of blocks with assigned observation times
/// * `unscheduled_blocks` - Count of blocks without assigned times
/// * `missing_coordinates` - Count of blocks lacking target coordinates
/// * `missing_constraints` - Count of blocks missing elevation/azimuth constraints
/// * `duplicate_ids` - Number of duplicate scheduling block IDs found
/// * `invalid_priorities` - Count of priorities outside the valid range
/// * `invalid_durations` - Count of non-positive durations
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
    /// Creates a new validation result with valid status and empty error/warning lists.
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::preprocessing::validator::ValidationResult;
    ///
    /// let result = ValidationResult::new();
    /// assert!(result.is_valid);
    /// assert!(result.errors.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            stats: ValidationStats::default(),
        }
    }
    
    /// Adds a critical error and marks the result as invalid.
    ///
    /// After calling this method, `is_valid` will be `false`.
    ///
    /// # Arguments
    ///
    /// * `error` - Error message describing the validation failure
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::preprocessing::validator::ValidationResult;
    ///
    /// let mut result = ValidationResult::new();
    /// result.add_error("Missing schedulingBlockId".to_string());
    /// assert!(!result.is_valid);
    /// ```
    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }
    
    /// Adds a non-critical warning without invalidating the result.
    ///
    /// Warnings indicate potential issues that don't prevent processing
    /// but should be reviewed.
    ///
    /// # Arguments
    ///
    /// * `warning` - Warning message describing the potential issue
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::preprocessing::validator::ValidationResult;
    ///
    /// let mut result = ValidationResult::new();
    /// result.add_warning("Unusual priority value: 25.0".to_string());
    /// assert!(result.is_valid);  // Still valid despite warning
    /// assert_eq!(result.warnings.len(), 1);
    /// ```
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

/// Validator for telescope scheduling data.
///
/// `ScheduleValidator` provides validation logic for both structured
/// `SchedulingBlock` collections and Polars DataFrames. It checks data
/// completeness, value ranges, uniqueness constraints, and schema requirements.
///
/// # Examples
///
/// ```no_run
/// use tsi_rust::preprocessing::validator::ScheduleValidator;
/// use tsi_rust::core::domain::SchedulingBlock;
///
/// # fn example(blocks: &[SchedulingBlock]) {
/// let result = ScheduleValidator::validate_blocks(blocks);
/// if !result.is_valid {
///     eprintln!("Validation failed: {:?}", result.errors);
/// }
/// println!("Validated {} blocks", result.stats.total_blocks);
/// # }
/// ```
pub struct ScheduleValidator;

impl ScheduleValidator {
    /// Validates a collection of scheduling blocks.
    ///
    /// Performs comprehensive validation including:
    /// - Duplicate ID detection
    /// - Priority range checks (0-20)
    /// - Duration positivity checks
    /// - Missing coordinates detection
    /// - Missing constraints detection
    ///
    /// # Arguments
    ///
    /// * `blocks` - Slice of scheduling blocks to validate
    ///
    /// # Returns
    ///
    /// `ValidationResult` containing all errors, warnings, and statistics.
    ///
    /// # Examples
    ///
    /// ```
    /// use tsi_rust::preprocessing::validator::ScheduleValidator;
    /// use tsi_rust::core::domain::SchedulingBlock;
    /// use siderust::units::time::Seconds;
    ///
    /// let blocks = vec![
    ///     SchedulingBlock {
    ///         scheduling_block_id: "SB001".to_string(),
    ///         priority: 10.0,
    ///         requested_duration: Seconds::new(3600.0),
    ///         min_observation_time: Seconds::new(1800.0),
    ///         coordinates: None,
    ///         fixed_time: None,
    ///         min_azimuth_angle: None,
    ///         max_azimuth_angle: None,
    ///         min_elevation_angle: None,
    ///         max_elevation_angle: None,
    ///         scheduled_period: None,
    ///         visibility_periods: vec![],
    ///     }
    /// ];
    ///
    /// let result = ScheduleValidator::validate_blocks(&blocks);
    /// assert_eq!(result.stats.total_blocks, 1);
    /// ```
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
    
    /// Validates a Polars DataFrame containing schedule data.
    ///
    /// Checks DataFrame schema, column presence, value ranges, and data quality.
    /// Requires columns: `schedulingBlockId`, `priority`, `requestedDurationSec`.
    ///
    /// # Arguments
    ///
    /// * `df` - DataFrame to validate
    ///
    /// # Returns
    ///
    /// `ValidationResult` with errors if schema is invalid or data has quality issues.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tsi_rust::preprocessing::validator::ScheduleValidator;
    /// use polars::prelude::*;
    ///
    /// # fn example(df: &DataFrame) {
    /// let result = ScheduleValidator::validate_dataframe(df);
    /// if !result.is_valid {
    ///     for error in &result.errors {
    ///         eprintln!("Error: {}", error);
    ///     }
    /// }
    /// # }
    /// ```
    ///
    /// # Error Conditions
    ///
    /// - Missing required columns
    /// - Duplicate scheduling block IDs
    /// - Invalid priorities (< 0 or > 20)
    /// - Non-positive durations
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
        if block.coordinates.is_none() {
            result.stats.missing_coordinates += 1;
        }
        
        // Check constraints
        if block.min_elevation_angle.is_none() || block.max_elevation_angle.is_none() {
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
        if block.requested_duration.value() <= 0.0 {
            result.stats.invalid_durations += 1;
            result.add_error(format!(
                "Block {} has invalid duration: {}",
                block.scheduling_block_id, block.requested_duration.value()
            ));
        }
        
        // Validate elevation range
        if let (Some(min_el), Some(max_el)) = (block.min_elevation_angle, block.max_elevation_angle) {
            let min_el_val = min_el.value();
            let max_el_val = max_el.value();
            if min_el_val < 0.0 || min_el_val > 90.0 {
                result.add_warning(format!(
                    "Block {} has invalid min elevation: {}",
                    block.scheduling_block_id, min_el_val
                ));
            }
            if max_el_val < 0.0 || max_el_val > 90.0 {
                result.add_warning(format!(
                    "Block {} has invalid max elevation: {}",
                    block.scheduling_block_id, max_el_val
                ));
            }
            if min_el_val >= max_el_val {
                result.add_error(format!(
                    "Block {} has min elevation >= max elevation: {} >= {}",
                    block.scheduling_block_id, min_el_val, max_el_val
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::domain::{Period, SchedulingBlock};
    use siderust::astro::ModifiedJulianDate;
    use siderust::coordinates::spherical::direction::ICRS;
    use siderust::units::{Degrees, Seconds};

    #[test]
    fn test_validate_valid_block() {
        let block = SchedulingBlock {
            scheduling_block_id: "test-001".to_string(),
            priority: 10.0,
            requested_duration: Seconds::new(3600.0),
            min_observation_time: Seconds::new(1800.0),
            fixed_time: None,
            coordinates: Some(ICRS::new(Degrees::new(180.0), Degrees::new(45.0))),
            min_azimuth_angle: Some(Degrees::new(0.0)),
            max_azimuth_angle: Some(Degrees::new(360.0)),
            min_elevation_angle: Some(Degrees::new(30.0)),
            max_elevation_angle: Some(Degrees::new(80.0)),
            scheduled_period: Some(Period::new(
                ModifiedJulianDate::new(59580.0),
                ModifiedJulianDate::new(59580.5),
            )),
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
            requested_duration: Seconds::new(-100.0),  // Invalid: negative
            min_observation_time: Seconds::new(0.0),
            fixed_time: None,
            coordinates: None,  // Missing
            min_azimuth_angle: Some(Degrees::new(0.0)),
            max_azimuth_angle: Some(Degrees::new(360.0)),
            min_elevation_angle: Some(Degrees::new(80.0)),
            max_elevation_angle: Some(Degrees::new(30.0)),  // Invalid: max < min
            scheduled_period: None,
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
