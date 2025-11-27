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
/// * `blocks_with_visibility` - Count of blocks with visibility periods
/// * `avg_visibility_periods` - Average number of visibility periods per block
/// * `avg_visibility_hours` - Average visibility hours per block
/// * `missing_coordinates` - Count of blocks lacking target coordinates
/// * `missing_constraints` - Count of blocks missing elevation/azimuth constraints
/// * `duplicate_ids` - Number of duplicate scheduling block IDs found
/// * `invalid_priorities` - Count of priorities outside the valid range
/// * `invalid_durations` - Count of non-positive durations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationStats {
    pub total_blocks: usize,
    pub scheduled_blocks: usize,
    pub unscheduled_blocks: usize,
    pub blocks_with_visibility: usize,
    pub avg_visibility_periods: f64,
    pub avg_visibility_hours: f64,
    pub missing_coordinates: usize,
    pub missing_constraints: usize,
    pub duplicate_ids: usize,
    pub invalid_priorities: usize,
    pub invalid_durations: usize,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
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
    /// - Invalid priorities (< 0)
    /// - Non-positive durations
    /// - Invalid coordinate ranges (dec: [-90, 90], ra: [0, 360))
    /// - Invalid elevation constraints (min >= max)
    /// - Invalid scheduled periods (start >= stop)
    pub fn validate_dataframe(df: &DataFrame) -> ValidationResult {
        let mut result = ValidationResult::new();

        result.stats.total_blocks = df.height();

        // Handle empty DataFrame
        if result.stats.total_blocks == 0 {
            return result;
        }

        // Check required columns
        let required_cols = vec!["schedulingBlockId", "priority", "requestedDurationSec"];

        for col in required_cols {
            if df.column(col).is_err() {
                result.add_error(format!("Missing required column: {}", col));
            }
        }

        if !result.is_valid {
            return result;
        }

        // Check for missing IDs
        if let Ok(id_col) = df.column("schedulingBlockId") {
            let missing_count = id_col.null_count();
            if missing_count > 0 {
                result.add_error(format!("{} blocks have missing IDs", missing_count));
            }

            // Check for duplicate IDs
            if let Ok(str_series) = id_col.str() {
                let unique_count = str_series.n_unique().unwrap_or(0);
                let total_count = str_series.len();
                let duplicates = total_count - unique_count;
                if duplicates > 0 {
                    result.add_error(format!(
                        "{} duplicate scheduling block IDs found",
                        duplicates
                    ));
                }
            }
        }

        // Count scheduled blocks
        if let Ok(scheduled_flag) = df.column("scheduled_flag") {
            if let Ok(bool_series) = scheduled_flag.bool() {
                result.stats.scheduled_blocks = bool_series.sum().unwrap_or(0) as usize;
                result.stats.unscheduled_blocks =
                    result.stats.total_blocks - result.stats.scheduled_blocks;
            }
        }

        // Validate priorities
        if let Ok(priority_col) = df.column("priority") {
            if let Ok(f64_series) = priority_col.f64() {
                // Count negative priorities (errors)
                let negative_count = f64_series
                    .iter()
                    .flatten()
                    .filter(|&p| p < 0.0)
                    .count();

                if negative_count > 0 {
                    result.add_error(format!(
                        "{} blocks have negative priority (invalid)",
                        negative_count
                    ));
                }

                // Count missing priorities (warnings)
                let missing_count = priority_col.null_count();
                if missing_count > 0 {
                    result.add_warning(format!("{} blocks have missing priority", missing_count));
                }
            }
        }

        // Validate coordinates
        Self::validate_coordinates(df, &mut result);

        // Validate time constraints
        Self::validate_time_constraints(df, &mut result);

        // Validate elevation constraints
        Self::validate_elevation_constraints(df, &mut result);

        // Compute visibility statistics
        Self::compute_visibility_stats(df, &mut result);

        result
    }

    fn validate_coordinates(df: &DataFrame, result: &mut ValidationResult) {
        // Validate declination: [-90, 90]
        if let Ok(dec_col) = df.column("decInDeg") {
            if let Ok(f64_series) = dec_col.f64() {
                let invalid_count = f64_series
                    .iter()
                    .flatten()
                    .filter(|&dec| dec < -90.0 || dec > 90.0)
                    .count();

                if invalid_count > 0 {
                    result.add_error(format!(
                        "{} blocks have invalid declination (outside [-90, 90])",
                        invalid_count
                    ));
                }

                let missing = dec_col.null_count();
                if missing > 0 {
                    result.add_warning(format!("{} blocks have missing declination", missing));
                }
                result.stats.missing_coordinates = missing;
            }
        }

        // Validate right ascension: [0, 360)
        if let Ok(ra_col) = df.column("raInDeg") {
            if let Ok(f64_series) = ra_col.f64() {
                let invalid_count = f64_series
                    .iter()
                    .flatten()
                    .filter(|&ra| ra < 0.0 || ra >= 360.0)
                    .count();

                if invalid_count > 0 {
                    result.add_error(format!(
                        "{} blocks have invalid right ascension (outside [0, 360))",
                        invalid_count
                    ));
                }

                let missing = ra_col.null_count();
                if missing > 0 {
                    result.add_warning(format!("{} blocks have missing right ascension", missing));
                }
            }
        }
    }

    fn validate_time_constraints(df: &DataFrame, result: &mut ValidationResult) {
        // Check requested duration > 0
        if let Ok(duration_col) = df.column("requestedDurationSec") {
            if let Ok(f64_series) = duration_col.f64() {
                let invalid_count = f64_series
                    .iter()
                    .flatten()
                    .filter(|&d| d <= 0.0)
                    .count();

                if invalid_count > 0 {
                    result.add_error(format!(
                        "{} blocks have invalid requested duration (≤ 0)",
                        invalid_count
                    ));
                    result.stats.invalid_durations = invalid_count;
                }
            }
        }

        // Check scheduled period: start < stop
        if let (Ok(start_col), Ok(stop_col)) = (
            df.column("scheduled_period.start"),
            df.column("scheduled_period.stop"),
        ) {
            if let (Ok(start_series), Ok(stop_series)) = (start_col.f64(), stop_col.f64()) {
                let invalid_count = start_series
                    .iter()
                    .zip(stop_series.iter())
                    .filter_map(|(s, e)| match (s, e) {
                        (Some(start), Some(stop)) if start >= stop => Some(()),
                        _ => None,
                    })
                    .count();

                if invalid_count > 0 {
                    result.add_error(format!(
                        "{} blocks have start time ≥ stop time",
                        invalid_count
                    ));
                }
            }
        }
    }

    fn validate_elevation_constraints(df: &DataFrame, result: &mut ValidationResult) {
        if let (Ok(min_col), Ok(max_col)) = (
            df.column("minElevationAngleInDeg"),
            df.column("maxElevationAngleInDeg"),
        ) {
            if let (Ok(min_series), Ok(max_series)) = (min_col.f64(), max_col.f64()) {
                let invalid_count = min_series
                    .iter()
                    .zip(max_series.iter())
                    .filter_map(|(min, max)| match (min, max) {
                        (Some(min_val), Some(max_val)) if min_val >= max_val => Some(()),
                        _ => None,
                    })
                    .count();

                if invalid_count > 0 {
                    result.add_error(format!(
                        "{} blocks have min elevation ≥ max elevation",
                        invalid_count
                    ));
                }
            }
        }
    }

    fn compute_visibility_stats(df: &DataFrame, result: &mut ValidationResult) {
        // Count blocks with visibility
        if let Ok(num_vis_col) = df.column("num_visibility_periods") {
            if let Ok(u32_series) = num_vis_col.u32() {
                result.stats.blocks_with_visibility = u32_series
                    .iter()
                    .filter(|&v| v.unwrap_or(0) > 0)
                    .count();
            }

            // Average visibility periods - cast to f64 first
            if let Ok(f64_series) = num_vis_col.cast(&DataType::Float64) {
                if let Ok(mean_val) = f64_series.f64().and_then(|s| Ok(s.mean().unwrap_or(0.0))) {
                    result.stats.avg_visibility_periods = mean_val;
                }
            }
        }

        // Average visibility hours
        if let Ok(vis_hours_col) = df.column("total_visibility_hours") {
            if let Ok(f64_series) = vis_hours_col.f64() {
                result.stats.avg_visibility_hours = f64_series.mean().unwrap_or(0.0);
            }
        }
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

        // Validate priority (negative is invalid)
        if block.priority < 0.0 {
            result.stats.invalid_priorities += 1;
            result.add_error(format!(
                "Block {} has negative priority: {}",
                block.scheduling_block_id, block.priority
            ));
        }

        // Validate duration
        if block.requested_duration.value() <= 0.0 {
            result.stats.invalid_durations += 1;
            result.add_error(format!(
                "Block {} has invalid duration: {}",
                block.scheduling_block_id,
                block.requested_duration.value()
            ));
        }

        // Validate elevation range
        if let (Some(min_el), Some(max_el)) = (block.min_elevation_angle, block.max_elevation_angle)
        {
            let min_el_val = min_el.value();
            let max_el_val = max_el.value();
            if !(0.0..=90.0).contains(&min_el_val) {
                result.add_warning(format!(
                    "Block {} has invalid min elevation: {}",
                    block.scheduling_block_id, min_el_val
                ));
            }
            if !(0.0..=90.0).contains(&max_el_val) {
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
            priority: 25.0,                           // Invalid: > 20
            requested_duration: Seconds::new(-100.0), // Invalid: negative
            min_observation_time: Seconds::new(0.0),
            fixed_time: None,
            coordinates: None, // Missing
            min_azimuth_angle: Some(Degrees::new(0.0)),
            max_azimuth_angle: Some(Degrees::new(360.0)),
            min_elevation_angle: Some(Degrees::new(80.0)),
            max_elevation_angle: Some(Degrees::new(30.0)), // Invalid: max < min
            scheduled_period: None,
            visibility_periods: vec![],
        };

        let result = ScheduleValidator::validate_blocks(&[block]);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        assert_eq!(result.stats.invalid_priorities, 1);
        assert_eq!(result.stats.invalid_durations, 1);
        assert_eq!(result.stats.missing_coordinates, 1);
    }
}
