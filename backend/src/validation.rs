//! Input validation utilities for API request parameters
//! 
//! This module provides validation functions and types to ensure
//! API requests contain valid data before processing.

use crate::error::{BackendError, Result};

/// Validate that a numeric parameter is within a specified range
/// 
/// # Arguments
/// * `value` - The value to validate
/// * `min` - Minimum allowed value (inclusive)
/// * `max` - Maximum allowed value (inclusive)
/// * `param_name` - Name of the parameter for error messages
/// 
/// # Returns
/// Ok(()) if valid, Err with descriptive message otherwise
pub fn validate_range<T: PartialOrd + std::fmt::Display>(
    value: T,
    min: T,
    max: T,
    param_name: &str,
) -> Result<()> {
    if value < min || value > max {
        return Err(BackendError::InvalidParameter(format!(
            "{} must be between {} and {}, got {}",
            param_name, min, max, value
        )));
    }
    Ok(())
}

/// Validate that a string parameter is not empty
pub fn validate_not_empty(value: &str, param_name: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(BackendError::InvalidParameter(format!(
            "{} cannot be empty",
            param_name
        )));
    }
    Ok(())
}

/// Validate that a column name is valid
pub fn validate_column_name(column: &str) -> Result<()> {
    const VALID_COLUMNS: &[&str] = &[
        "priority",
        "total_visibility_hours",
        "visibility_hours",
        "requested_hours",
        "elevation_range_deg",
        "elevation_range",
        "min_elevation_angle_in_deg",
        "max_elevation_angle_in_deg",
        "ra_in_deg",
        "dec_in_deg",
        "min_azimuth_angle_in_deg",
        "max_azimuth_angle_in_deg",
    ];

    if !VALID_COLUMNS.contains(&column) {
        return Err(BackendError::InvalidColumn(format!(
            "Unknown column '{}'. Valid columns: {}",
            column,
            VALID_COLUMNS.join(", ")
        )));
    }

    Ok(())
}

/// Validate that sort_by parameter is valid
pub fn validate_sort_by(sort_by: &str) -> Result<()> {
    const VALID_SORT_BY: &[&str] = &[
        "priority",
        "requested_hours",
        "requested",
        "visibility_hours",
        "visibility",
        "elevation_range",
        "elevation",
    ];

    if !VALID_SORT_BY.contains(&sort_by) {
        return Err(BackendError::InvalidParameter(format!(
            "Invalid sort_by '{}'. Valid options: {}",
            sort_by,
            VALID_SORT_BY.join(", ")
        )));
    }

    Ok(())
}

/// Validate that sort order is valid
pub fn validate_sort_order(order: &str) -> Result<()> {
    const VALID_ORDERS: &[&str] = &["asc", "ascending", "desc", "descending"];

    if !VALID_ORDERS.contains(&order) {
        return Err(BackendError::InvalidParameter(format!(
            "Invalid order '{}'. Valid options: {}",
            order,
            VALID_ORDERS.join(", ")
        )));
    }

    Ok(())
}

/// Validate file size is within limits
/// 
/// # Arguments
/// * `size` - Size in bytes
/// * `max_size` - Maximum allowed size in bytes
/// 
/// # Returns
/// Ok(()) if valid, Err if file is too large
pub fn validate_file_size(size: usize, max_size: usize) -> Result<()> {
    if size > max_size {
        return Err(BackendError::FileTooLarge {
            size,
            limit: max_size,
        });
    }
    Ok(())
}

/// Validate that bins parameter is reasonable
pub fn validate_bins(bins: usize) -> Result<()> {
    validate_range(bins, 1, 100, "bins")
}

/// Validate that limit parameter is reasonable
pub fn validate_limit(n: usize) -> Result<()> {
    validate_range(n, 1, 1000, "n")
}

/// Validate metric type for trends analysis
pub fn validate_metric(metric: &str) -> Result<()> {
    const VALID_METRICS: &[&str] = &[
        "scheduling_rate",
        "utilization",
        "priority_distribution",
        "avg_priority",
    ];

    if !VALID_METRICS.contains(&metric) {
        return Err(BackendError::InvalidParameter(format!(
            "Invalid metric '{}'. Valid options: {}",
            metric,
            VALID_METRICS.join(", ")
        )));
    }

    Ok(())
}

/// Validate group_by parameter for trends
pub fn validate_group_by(group_by: &str) -> Result<()> {
    const VALID_GROUP_BY: &[&str] = &["month", "week", "day"];

    if !VALID_GROUP_BY.contains(&group_by) {
        return Err(BackendError::InvalidParameter(format!(
            "Invalid group_by '{}'. Valid options: {}",
            group_by,
            VALID_GROUP_BY.join(", ")
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_range_valid() {
        assert!(validate_range(5, 1, 10, "test").is_ok());
        assert!(validate_range(1, 1, 10, "test").is_ok());
        assert!(validate_range(10, 1, 10, "test").is_ok());
    }

    #[test]
    fn test_validate_range_invalid() {
        assert!(validate_range(0, 1, 10, "test").is_err());
        assert!(validate_range(11, 1, 10, "test").is_err());
    }

    #[test]
    fn test_validate_not_empty() {
        assert!(validate_not_empty("hello", "test").is_ok());
        assert!(validate_not_empty("", "test").is_err());
        assert!(validate_not_empty("   ", "test").is_err());
    }

    #[test]
    fn test_validate_column_name() {
        assert!(validate_column_name("priority").is_ok());
        assert!(validate_column_name("requested_hours").is_ok());
        assert!(validate_column_name("invalid_column").is_err());
    }

    #[test]
    fn test_validate_sort_by() {
        assert!(validate_sort_by("priority").is_ok());
        assert!(validate_sort_by("requested_hours").is_ok());
        assert!(validate_sort_by("invalid").is_err());
    }

    #[test]
    fn test_validate_file_size() {
        assert!(validate_file_size(1000, 10000).is_ok());
        assert!(validate_file_size(10001, 10000).is_err());
    }

    #[test]
    fn test_validate_bins() {
        assert!(validate_bins(20).is_ok());
        assert!(validate_bins(0).is_err());
        assert!(validate_bins(101).is_err());
    }

    #[test]
    fn test_validate_metric() {
        assert!(validate_metric("scheduling_rate").is_ok());
        assert!(validate_metric("utilization").is_ok());
        assert!(validate_metric("invalid_metric").is_err());
    }
}
