//! Validation service for scheduling blocks.
//!
//! This module implements data validation rules that are executed during the ETL Transform stage.
//! Validation results are persisted to the database and used to:
//! 1. Filter out impossible-to-schedule blocks from analytics queries
//! 2. Provide detailed validation reports to users
//!
//! Validation rules include:
//! - Visibility checks (zero visibility, insufficient visibility)
//! - Constraint validation (negative values, invalid ranges)
//! - Coordinate validation (RA/Dec ranges)
//! - Temporal constraint validation (scheduled periods vs constraints)

/// Validation status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationStatus {
    Valid,
    Impossible,
    Error,
    Warning,
}

impl ValidationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ValidationStatus::Valid => "valid",
            ValidationStatus::Impossible => "impossible",
            ValidationStatus::Error => "error",
            ValidationStatus::Warning => "warning",
        }
    }
}

/// Criticality level for validation issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Criticality {
    Low,
    Medium,
    High,
    Critical,
}

impl Criticality {
    pub fn as_str(&self) -> &'static str {
        match self {
            Criticality::Low => "Low",
            Criticality::Medium => "Medium",
            Criticality::High => "High",
            Criticality::Critical => "Critical",
        }
    }
}

/// Issue category for grouping validation problems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueCategory {
    Visibility,
    Constraint,
    Coordinate,
    Priority,
    Duration,
    ScheduledPeriod,
}

impl IssueCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            IssueCategory::Visibility => "visibility",
            IssueCategory::Constraint => "constraint",
            IssueCategory::Coordinate => "coordinate",
            IssueCategory::Priority => "priority",
            IssueCategory::Duration => "duration",
            IssueCategory::ScheduledPeriod => "scheduled_period",
        }
    }
}

/// A single validation result for a scheduling block
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub schedule_id: i64,
    pub scheduling_block_id: i64,
    pub status: ValidationStatus,
    pub issue_type: Option<String>,
    pub issue_category: Option<IssueCategory>,
    pub criticality: Option<Criticality>,
    pub field_name: Option<String>,
    pub current_value: Option<String>,
    pub expected_value: Option<String>,
    pub description: Option<String>,
}

impl ValidationResult {
    /// Create a validation result indicating the block is valid
    pub fn valid(schedule_id: i64, scheduling_block_id: i64) -> Self {
        Self {
            schedule_id,
            scheduling_block_id,
            status: ValidationStatus::Valid,
            issue_type: None,
            issue_category: None,
            criticality: None,
            field_name: None,
            current_value: None,
            expected_value: None,
            description: None,
        }
    }

    /// Create a validation result for an impossible block (cannot be scheduled)
    pub fn impossible(
        schedule_id: i64,
        scheduling_block_id: i64,
        issue_type: String,
        category: IssueCategory,
        description: String,
        field_name: Option<String>,
        current_value: Option<String>,
        expected_value: Option<String>,
    ) -> Self {
        Self {
            schedule_id,
            scheduling_block_id,
            status: ValidationStatus::Impossible,
            issue_type: Some(issue_type),
            issue_category: Some(category),
            criticality: Some(Criticality::Critical),
            field_name,
            current_value,
            expected_value,
            description: Some(description),
        }
    }

    /// Create a validation error result
    pub fn error(
        schedule_id: i64,
        scheduling_block_id: i64,
        issue_type: String,
        category: IssueCategory,
        criticality: Criticality,
        description: String,
        field_name: Option<String>,
        current_value: Option<String>,
        expected_value: Option<String>,
    ) -> Self {
        Self {
            schedule_id,
            scheduling_block_id,
            status: ValidationStatus::Error,
            issue_type: Some(issue_type),
            issue_category: Some(category),
            criticality: Some(criticality),
            field_name,
            current_value,
            expected_value,
            description: Some(description),
        }
    }

    /// Create a validation warning result
    pub fn warning(
        schedule_id: i64,
        scheduling_block_id: i64,
        issue_type: String,
        category: IssueCategory,
        criticality: Criticality,
        description: String,
        field_name: Option<String>,
        current_value: Option<String>,
    ) -> Self {
        Self {
            schedule_id,
            scheduling_block_id,
            status: ValidationStatus::Warning,
            issue_type: Some(issue_type),
            issue_category: Some(category),
            criticality: Some(criticality),
            field_name,
            current_value,
            expected_value: None,
            description: Some(description),
        }
    }
}

/// Data structure for a scheduling block being validated
#[derive(Debug, Clone)]
pub struct BlockForValidation {
    pub schedule_id: i64,
    pub scheduling_block_id: i64,
    pub priority: f64,
    pub requested_duration_sec: i32,
    pub min_observation_sec: i32,
    pub total_visibility_hours: f64,
    pub min_alt_deg: Option<f64>,
    pub max_alt_deg: Option<f64>,
    pub constraint_start_mjd: Option<f64>,
    pub constraint_stop_mjd: Option<f64>,
    pub scheduled_start_mjd: Option<f64>,
    pub scheduled_stop_mjd: Option<f64>,
    pub target_ra_deg: f64,
    pub target_dec_deg: f64,
}

/// Validate a single scheduling block
///
/// Returns a vector of validation results (may be multiple issues per block)
pub fn validate_block(block: &BlockForValidation) -> Vec<ValidationResult> {
    let mut results = Vec::new();

    // === CRITICAL: Visibility Checks ===
    
    // Check 1: Zero visibility (impossible to schedule)
    if block.total_visibility_hours < 0.001 {
        // Less than ~3.6 seconds
        results.push(ValidationResult::impossible(
            block.schedule_id,
            block.scheduling_block_id,
            "No visibility periods available".to_string(),
            IssueCategory::Visibility,
            "This block has no time windows when it is visible from the telescope site".to_string(),
            Some("total_visibility_hours".to_string()),
            Some(format!("{:.6}", block.total_visibility_hours)),
            Some("> 0".to_string()),
        ));
    }
    // Check 2: Insufficient visibility for requested duration
    else {
        let requested_hours = block.requested_duration_sec as f64 / 3600.0;
        if block.total_visibility_hours < requested_hours {
            results.push(ValidationResult::impossible(
                block.schedule_id,
                block.scheduling_block_id,
                "Visibility less than requested duration".to_string(),
                IssueCategory::Visibility,
                format!(
                    "Needs {:.2}h but only {:.2}h available",
                    requested_hours, block.total_visibility_hours
                ),
                Some("total_visibility_hours".to_string()),
                Some(format!("{:.2}", block.total_visibility_hours)),
                Some(format!(">= {:.2}", requested_hours)),
            ));
        }

        // Check 3: Insufficient visibility for minimum observation time
        let min_observation_hours = block.min_observation_sec as f64 / 3600.0;
        if block.total_visibility_hours < min_observation_hours {
            results.push(ValidationResult::impossible(
                block.schedule_id,
                block.scheduling_block_id,
                "Visibility less than minimum observation time".to_string(),
                IssueCategory::Visibility,
                format!(
                    "Minimum {:.2}h required but only {:.2}h available",
                    min_observation_hours, block.total_visibility_hours
                ),
                Some("total_visibility_hours".to_string()),
                Some(format!("{:.2}", block.total_visibility_hours)),
                Some(format!(">= {:.2}", min_observation_hours)),
            ));
        }
    }

    // === HIGH: Constraint Validation Errors ===

    // Check 4: Negative priority
    if block.priority < 0.0 {
        results.push(ValidationResult::error(
            block.schedule_id,
            block.scheduling_block_id,
            "Negative priority".to_string(),
            IssueCategory::Priority,
            Criticality::High,
            "Priority values must be non-negative".to_string(),
            Some("priority".to_string()),
            Some(format!("{:.2}", block.priority)),
            Some(">= 0".to_string()),
        ));
    }

    // Check 5: Negative requested duration
    if block.requested_duration_sec < 0 {
        results.push(ValidationResult::error(
            block.schedule_id,
            block.scheduling_block_id,
            "Negative requested duration".to_string(),
            IssueCategory::Duration,
            Criticality::High,
            "Requested duration must be a positive value".to_string(),
            Some("requested_duration_sec".to_string()),
            Some(block.requested_duration_sec.to_string()),
            Some("> 0".to_string()),
        ));
    }

    // Check 6: Negative minimum observation time
    if block.min_observation_sec < 0 {
        results.push(ValidationResult::error(
            block.schedule_id,
            block.scheduling_block_id,
            "Negative minimum observation time".to_string(),
            IssueCategory::Duration,
            Criticality::High,
            "Minimum observation time must be a positive value".to_string(),
            Some("min_observation_sec".to_string()),
            Some(block.min_observation_sec.to_string()),
            Some("> 0".to_string()),
        ));
    }

    // Check 7: Min observation time exceeds requested duration
    if block.min_observation_sec > block.requested_duration_sec {
        results.push(ValidationResult::error(
            block.schedule_id,
            block.scheduling_block_id,
            "Minimum observation time exceeds requested duration".to_string(),
            IssueCategory::Duration,
            Criticality::High,
            format!(
                "Minimum observation time ({:.2}h) cannot be greater than requested duration ({:.2}h)",
                block.min_observation_sec as f64 / 3600.0,
                block.requested_duration_sec as f64 / 3600.0
            ),
            Some("min_observation_sec".to_string()),
            Some(block.min_observation_sec.to_string()),
            Some(format!("<= {}", block.requested_duration_sec)),
        ));
    }

    // === MEDIUM: Coordinate Validation ===

    // Check 8: Invalid RA range (should be 0-360 degrees)
    if block.target_ra_deg < 0.0 || block.target_ra_deg >= 360.0 {
        results.push(ValidationResult::error(
            block.schedule_id,
            block.scheduling_block_id,
            "Invalid Right Ascension".to_string(),
            IssueCategory::Coordinate,
            Criticality::Medium,
            format!("Right Ascension {:.2}° is outside valid range", block.target_ra_deg),
            Some("target_ra_deg".to_string()),
            Some(format!("{:.2}", block.target_ra_deg)),
            Some("0-360".to_string()),
        ));
    }

    // Check 9: Invalid Dec range (should be -90 to +90 degrees)
    if block.target_dec_deg < -90.0 || block.target_dec_deg > 90.0 {
        results.push(ValidationResult::error(
            block.schedule_id,
            block.scheduling_block_id,
            "Invalid Declination".to_string(),
            IssueCategory::Coordinate,
            Criticality::Medium,
            format!("Declination {:.2}° is outside valid range", block.target_dec_deg),
            Some("target_dec_deg".to_string()),
            Some(format!("{:.2}", block.target_dec_deg)),
            Some("-90 to +90".to_string()),
        ));
    }

    // Check 10: Invalid elevation constraints
    if let (Some(min_alt), Some(max_alt)) = (block.min_alt_deg, block.max_alt_deg) {
        if min_alt > max_alt {
            results.push(ValidationResult::error(
                block.schedule_id,
                block.scheduling_block_id,
                "Invalid elevation constraint range".to_string(),
                IssueCategory::Constraint,
                Criticality::Medium,
                format!("Minimum altitude ({:.1}°) exceeds maximum altitude ({:.1}°)", min_alt, max_alt),
                Some("min_alt_deg".to_string()),
                Some(format!("{:.1}", min_alt)),
                Some(format!("<= {:.1}", max_alt)),
            ));
        }

        let elevation_range = max_alt - min_alt;
        
        // Check 11: Physically impossible elevation range
        if elevation_range < 0.0 || elevation_range > 180.0 {
            results.push(ValidationResult::error(
                block.schedule_id,
                block.scheduling_block_id,
                "Physically impossible elevation range".to_string(),
                IssueCategory::Constraint,
                Criticality::Medium,
                format!("Elevation range {:.1}° is physically impossible", elevation_range),
                Some("elevation_range".to_string()),
                Some(format!("{:.1}", elevation_range)),
                Some("0-180".to_string()),
            ));
        }
        // Check 12: Warning for very narrow elevation ranges
        else if elevation_range > 0.0 && elevation_range < 5.0 {
            results.push(ValidationResult::warning(
                block.schedule_id,
                block.scheduling_block_id,
                "Very narrow elevation range".to_string(),
                IssueCategory::Constraint,
                Criticality::Medium,
                format!("Elevation range of {:.1}° may make scheduling difficult", elevation_range),
                Some("elevation_range".to_string()),
                Some(format!("{:.1}", elevation_range)),
            ));
        }
    }

    // Check 13: Invalid time constraint range
    if let (Some(start_mjd), Some(stop_mjd)) = (block.constraint_start_mjd, block.constraint_stop_mjd) {
        if start_mjd > stop_mjd {
            results.push(ValidationResult::error(
                block.schedule_id,
                block.scheduling_block_id,
                "Invalid time constraint range".to_string(),
                IssueCategory::Constraint,
                Criticality::High,
                format!("Constraint start time ({:.2} MJD) is after stop time ({:.2} MJD)", start_mjd, stop_mjd),
                Some("constraint_start_mjd".to_string()),
                Some(format!("{:.2}", start_mjd)),
                Some(format!("<= {:.2}", stop_mjd)),
            ));
        }

        // Check 14: Time constraint duration less than requested duration
        let constraint_duration_days = stop_mjd - start_mjd;
        let constraint_duration_hours = constraint_duration_days * 24.0;
        let requested_hours = block.requested_duration_sec as f64 / 3600.0;
        // Add small tolerance to account for floating point precision (0.001 hours = 3.6 seconds)
        let tolerance_hours = 0.001;

        if constraint_duration_hours + tolerance_hours < requested_hours {
            results.push(ValidationResult::error(
                block.schedule_id,
                block.scheduling_block_id,
                "Time constraint duration less than requested duration".to_string(),
                IssueCategory::Constraint,
                Criticality::High,
                format!(
                    "Time constraint allows {:.2}h but {:.2}h requested",
                    constraint_duration_hours, requested_hours
                ),
                Some("constraint_duration".to_string()),
                Some(format!("{:.2}h", constraint_duration_hours)),
                Some(format!(">= {:.2}h", requested_hours)),
            ));
        }
    }

    // === SCHEDULED PERIOD VALIDATION ===

    if let (Some(sched_start), Some(sched_stop)) = (block.scheduled_start_mjd, block.scheduled_stop_mjd) {
        // Check 15: Invalid scheduled period range
        if sched_start > sched_stop {
            results.push(ValidationResult::error(
                block.schedule_id,
                block.scheduling_block_id,
                "Invalid scheduled period".to_string(),
                IssueCategory::ScheduledPeriod,
                Criticality::High,
                format!("Scheduled start time ({:.2} MJD) is after stop time ({:.2} MJD)", sched_start, sched_stop),
                Some("scheduled_start_mjd".to_string()),
                Some(format!("{:.2}", sched_start)),
                Some(format!("<= {:.2}", sched_stop)),
            ));
        }

        // Check 16: Scheduled duration exceeds requested duration
        let scheduled_duration_days = sched_stop - sched_start;
        let scheduled_duration_hours = scheduled_duration_days * 24.0;
        let requested_hours = block.requested_duration_sec as f64 / 3600.0;
        // Allow 1% tolerance for rounding and scheduler flexibility
        let tolerance_factor = 1.01;

        if scheduled_duration_hours > requested_hours * tolerance_factor {
            results.push(ValidationResult::warning(
                block.schedule_id,
                block.scheduling_block_id,
                "Scheduled duration exceeds requested duration".to_string(),
                IssueCategory::ScheduledPeriod,
                Criticality::Low,
                format!(
                    "Scheduled for {:.2}h but only {:.2}h requested",
                    scheduled_duration_hours, requested_hours
                ),
                Some("scheduled_duration".to_string()),
                Some(format!("{:.2}h", scheduled_duration_hours)),
            ));
        }

        // Check 17: Scheduled period outside time constraint
        if let (Some(constraint_start), Some(constraint_stop)) = 
            (block.constraint_start_mjd, block.constraint_stop_mjd) {
            // Add small epsilon tolerance for floating point comparisons (0.0001 days ≈ 8.64 seconds)
            let epsilon = 0.0001;
            
            if sched_start < constraint_start - epsilon || sched_stop > constraint_stop + epsilon {
                results.push(ValidationResult::error(
                    block.schedule_id,
                    block.scheduling_block_id,
                    "Scheduled period outside time constraint".to_string(),
                    IssueCategory::ScheduledPeriod,
                    Criticality::High,
                    format!(
                        "Scheduled [{:.2}, {:.2}] MJD is outside constraint [{:.2}, {:.2}] MJD",
                        sched_start, sched_stop, constraint_start, constraint_stop
                    ),
                    Some("scheduled_period".to_string()),
                    Some(format!("[{:.2}, {:.2}]", sched_start, sched_stop)),
                    Some(format!("[{:.2}, {:.2}]", constraint_start, constraint_stop)),
                ));
            }
        }
    }

    // If no issues found, mark as valid
    if results.is_empty() {
        results.push(ValidationResult::valid(block.schedule_id, block.scheduling_block_id));
    }

    results
}

/// Validate multiple blocks in batch
pub fn validate_blocks(blocks: &[BlockForValidation]) -> Vec<ValidationResult> {
    blocks
        .iter()
        .flat_map(|block| validate_block(block))
        .collect()
}
