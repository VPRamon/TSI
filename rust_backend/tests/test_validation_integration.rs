//! Integration tests for validation functionality across the schedule upload pipeline.
//!
//! These tests ensure that:
//! 1. Validation service correctly identifies issues
//! 2. Validation results are properly stored
//! 3. Validation integrates correctly with analytics pipeline
//! 4. All validation rules work as expected

use siderust::{
    astro::ModifiedJulianDate, coordinates::spherical::direction::ICRS, units::angular::Degrees,
    units::time::Seconds,
};
use tsi_rust::db::{
    models::{Constraints, Period, Schedule, SchedulingBlock, SchedulingBlockId},
    repositories::LocalRepository,
    repository::{AnalyticsRepository, ScheduleRepository, ValidationRepository},
};
use tsi_rust::services::validation::{
    validate_block, validate_blocks, BlockForValidation, Criticality, IssueCategory,
    ValidationStatus,
};

// ==================== Helper Functions ====================

fn create_test_block(
    id: i64,
    priority: f64,
    ra_deg: f64,
    dec_deg: f64,
    min_obs_sec: f64,
    req_dur_sec: f64,
    visibility_hours: f64,
) -> SchedulingBlock {
    SchedulingBlock {
        id: SchedulingBlockId(id),
        original_block_id: Some(format!("TEST_{}", id)),
        target: ICRS::new(Degrees::new(ra_deg), Degrees::new(dec_deg)),
        constraints: Constraints {
            min_alt: Degrees::new(30.0),
            max_alt: Degrees::new(80.0),
            min_az: Degrees::new(0.0),
            max_az: Degrees::new(360.0),
            fixed_time: None,
        },
        priority,
        min_observation: Seconds::new(min_obs_sec),
        requested_duration: Seconds::new(req_dur_sec),
        visibility_periods: if visibility_hours > 0.0 {
            vec![Period::new(
                ModifiedJulianDate::new(60000.0),
                ModifiedJulianDate::new(60000.0 + visibility_hours / 24.0),
            )
            .unwrap()]
        } else {
            vec![]
        },
        scheduled_period: None,
    }
}

fn create_validation_input(block: &SchedulingBlock) -> BlockForValidation {
    BlockForValidation {
        schedule_id: 1,
        scheduling_block_id: block.id.0,
        priority: block.priority,
        requested_duration_sec: block.requested_duration.value() as i32,
        min_observation_sec: block.min_observation.value() as i32,
        total_visibility_hours: block
            .visibility_periods
            .iter()
            .map(|p| p.duration().value() * 24.0)
            .sum(),
        min_alt_deg: Some(block.constraints.min_alt.value()),
        max_alt_deg: Some(block.constraints.max_alt.value()),
        constraint_start_mjd: None,
        constraint_stop_mjd: None,
        scheduled_start_mjd: None,
        scheduled_stop_mjd: None,
        target_ra_deg: block
            .target
            .ra()
            .to::<siderust::units::angular::Degree>()
            .value(),
        target_dec_deg: block
            .target
            .dec()
            .to::<siderust::units::angular::Degree>()
            .value(),
    }
}

// ==================== Validation Service Tests ====================

#[test]
fn test_validation_rule_zero_visibility() {
    let input = BlockForValidation {
        schedule_id: 1,
        scheduling_block_id: 100,
        priority: 5.0,
        requested_duration_sec: 3600,
        min_observation_sec: 600,
        total_visibility_hours: 0.0, // Zero visibility
        min_alt_deg: Some(30.0),
        max_alt_deg: Some(80.0),
        constraint_start_mjd: None,
        constraint_stop_mjd: None,
        scheduled_start_mjd: None,
        scheduled_stop_mjd: None,
        target_ra_deg: 180.0,
        target_dec_deg: 45.0,
    };

    let results = validate_block(&input);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, ValidationStatus::Impossible);
    assert_eq!(results[0].issue_category, Some(IssueCategory::Visibility));
    assert!(results[0]
        .description
        .as_ref()
        .unwrap()
        .contains("no time windows"));
}

#[test]
fn test_validation_rule_insufficient_visibility() {
    let input = BlockForValidation {
        schedule_id: 1,
        scheduling_block_id: 101,
        priority: 5.0,
        requested_duration_sec: 7200, // 2 hours requested
        min_observation_sec: 600,
        total_visibility_hours: 1.0, // Only 1 hour available
        min_alt_deg: Some(30.0),
        max_alt_deg: Some(80.0),
        constraint_start_mjd: None,
        constraint_stop_mjd: None,
        scheduled_start_mjd: None,
        scheduled_stop_mjd: None,
        target_ra_deg: 180.0,
        target_dec_deg: 45.0,
    };

    let results = validate_block(&input);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, ValidationStatus::Impossible);
    assert!(results[0]
        .issue_type
        .as_ref()
        .unwrap()
        .contains("Visibility less than requested"));
}

#[test]
fn test_validation_rule_negative_priority() {
    let input = BlockForValidation {
        schedule_id: 1,
        scheduling_block_id: 102,
        priority: -2.5, // Negative priority (error)
        requested_duration_sec: 3600,
        min_observation_sec: 600,
        total_visibility_hours: 5.0,
        min_alt_deg: Some(30.0),
        max_alt_deg: Some(80.0),
        constraint_start_mjd: None,
        constraint_stop_mjd: None,
        scheduled_start_mjd: None,
        scheduled_stop_mjd: None,
        target_ra_deg: 180.0,
        target_dec_deg: 45.0,
    };

    let results = validate_block(&input);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, ValidationStatus::Error);
    assert_eq!(results[0].criticality, Some(Criticality::High));
    assert_eq!(results[0].issue_category, Some(IssueCategory::Priority));
}

#[test]
fn test_validation_rule_invalid_coordinates() {
    // Invalid RA (>= 360)
    let input_ra = BlockForValidation {
        schedule_id: 1,
        scheduling_block_id: 103,
        priority: 5.0,
        requested_duration_sec: 3600,
        min_observation_sec: 600,
        total_visibility_hours: 5.0,
        min_alt_deg: Some(30.0),
        max_alt_deg: Some(80.0),
        constraint_start_mjd: None,
        constraint_stop_mjd: None,
        scheduled_start_mjd: None,
        scheduled_stop_mjd: None,
        target_ra_deg: 365.0, // Invalid
        target_dec_deg: 45.0,
    };

    let results_ra = validate_block(&input_ra);
    assert!(results_ra.iter().any(|r| {
        r.status == ValidationStatus::Error
            && r.issue_category == Some(IssueCategory::Coordinate)
            && r.issue_type.as_ref().unwrap().contains("Right Ascension")
    }));

    // Invalid Dec (> 90)
    let input_dec = BlockForValidation {
        schedule_id: 1,
        scheduling_block_id: 104,
        priority: 5.0,
        requested_duration_sec: 3600,
        min_observation_sec: 600,
        total_visibility_hours: 5.0,
        min_alt_deg: Some(30.0),
        max_alt_deg: Some(80.0),
        constraint_start_mjd: None,
        constraint_stop_mjd: None,
        scheduled_start_mjd: None,
        scheduled_stop_mjd: None,
        target_ra_deg: 180.0,
        target_dec_deg: 95.0, // Invalid
    };

    let results_dec = validate_block(&input_dec);
    assert!(results_dec.iter().any(|r| {
        r.status == ValidationStatus::Error
            && r.issue_category == Some(IssueCategory::Coordinate)
            && r.issue_type.as_ref().unwrap().contains("Declination")
    }));
}

#[test]
fn test_validation_rule_invalid_altitude_constraints() {
    let input = BlockForValidation {
        schedule_id: 1,
        scheduling_block_id: 105,
        priority: 5.0,
        requested_duration_sec: 3600,
        min_observation_sec: 600,
        total_visibility_hours: 5.0,
        min_alt_deg: Some(80.0), // Min > Max
        max_alt_deg: Some(30.0),
        constraint_start_mjd: None,
        constraint_stop_mjd: None,
        scheduled_start_mjd: None,
        scheduled_stop_mjd: None,
        target_ra_deg: 180.0,
        target_dec_deg: 45.0,
    };

    let results = validate_block(&input);

    assert!(results.iter().any(|r| {
        r.status == ValidationStatus::Error
            && r.issue_category == Some(IssueCategory::Constraint)
            && r.issue_type
                .as_ref()
                .unwrap()
                .contains("elevation constraint")
    }));
}

#[test]
fn test_validation_rule_narrow_elevation_warning() {
    let input = BlockForValidation {
        schedule_id: 1,
        scheduling_block_id: 106,
        priority: 5.0,
        requested_duration_sec: 3600,
        min_observation_sec: 600,
        total_visibility_hours: 5.0,
        min_alt_deg: Some(30.0),
        max_alt_deg: Some(32.0), // Only 2° range
        constraint_start_mjd: None,
        constraint_stop_mjd: None,
        scheduled_start_mjd: None,
        scheduled_stop_mjd: None,
        target_ra_deg: 180.0,
        target_dec_deg: 45.0,
    };

    let results = validate_block(&input);

    assert!(results.iter().any(|r| {
        r.status == ValidationStatus::Warning
            && r.issue_type.as_ref().unwrap().contains("narrow elevation")
    }));
}

#[test]
fn test_validation_rule_valid_block() {
    let input = BlockForValidation {
        schedule_id: 1,
        scheduling_block_id: 107,
        priority: 5.0,
        requested_duration_sec: 3600,
        min_observation_sec: 600,
        total_visibility_hours: 5.0,
        min_alt_deg: Some(30.0),
        max_alt_deg: Some(80.0),
        constraint_start_mjd: None,
        constraint_stop_mjd: None,
        scheduled_start_mjd: None,
        scheduled_stop_mjd: None,
        target_ra_deg: 180.0,
        target_dec_deg: 45.0,
    };

    let results = validate_block(&input);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, ValidationStatus::Valid);
}

#[test]
fn test_validation_multiple_issues_per_block() {
    // Block with multiple issues
    let input = BlockForValidation {
        schedule_id: 1,
        scheduling_block_id: 108,
        priority: -1.0,              // Issue 1: Negative priority
        requested_duration_sec: 0,   // Issue 2: Zero duration
        min_observation_sec: -100,   // Issue 3: Negative min obs
        total_visibility_hours: 0.0, // Issue 4: Zero visibility
        min_alt_deg: Some(30.0),
        max_alt_deg: Some(80.0),
        constraint_start_mjd: None,
        constraint_stop_mjd: None,
        scheduled_start_mjd: None,
        scheduled_stop_mjd: None,
        target_ra_deg: 400.0,  // Issue 5: Invalid RA
        target_dec_deg: 100.0, // Issue 6: Invalid Dec
    };

    let results = validate_block(&input);

    // Should have multiple issues
    assert!(results.len() > 1);

    // Check for various issue types
    assert!(results
        .iter()
        .any(|r| r.issue_category == Some(IssueCategory::Visibility)));
    assert!(results
        .iter()
        .any(|r| r.issue_category == Some(IssueCategory::Priority)));
    assert!(results
        .iter()
        .any(|r| r.issue_category == Some(IssueCategory::Duration)));
    assert!(results
        .iter()
        .any(|r| r.issue_category == Some(IssueCategory::Coordinate)));
}

#[test]
fn test_validation_batch_processing() {
    let blocks = vec![
        BlockForValidation {
            schedule_id: 1,
            scheduling_block_id: 200,
            priority: 5.0,
            requested_duration_sec: 3600,
            min_observation_sec: 600,
            total_visibility_hours: 5.0,
            min_alt_deg: Some(30.0),
            max_alt_deg: Some(80.0),
            constraint_start_mjd: None,
            constraint_stop_mjd: None,
            scheduled_start_mjd: None,
            scheduled_stop_mjd: None,
            target_ra_deg: 180.0,
            target_dec_deg: 45.0,
        },
        BlockForValidation {
            schedule_id: 1,
            scheduling_block_id: 201,
            priority: 5.0,
            requested_duration_sec: 3600,
            min_observation_sec: 600,
            total_visibility_hours: 0.0, // Zero visibility (impossible)
            min_alt_deg: Some(30.0),
            max_alt_deg: Some(80.0),
            constraint_start_mjd: None,
            constraint_stop_mjd: None,
            scheduled_start_mjd: None,
            scheduled_stop_mjd: None,
            target_ra_deg: 180.0,
            target_dec_deg: 45.0,
        },
        BlockForValidation {
            schedule_id: 1,
            scheduling_block_id: 202,
            priority: -1.0, // Negative priority (error)
            requested_duration_sec: 3600,
            min_observation_sec: 600,
            total_visibility_hours: 5.0,
            min_alt_deg: Some(30.0),
            max_alt_deg: Some(80.0),
            constraint_start_mjd: None,
            constraint_stop_mjd: None,
            scheduled_start_mjd: None,
            scheduled_stop_mjd: None,
            target_ra_deg: 180.0,
            target_dec_deg: 45.0,
        },
    ];

    let results = validate_blocks(&blocks);

    // Should have one result per block minimum
    assert!(results.len() >= 3);

    // Check that different statuses are present
    let has_valid = results.iter().any(|r| r.status == ValidationStatus::Valid);
    let has_impossible = results
        .iter()
        .any(|r| r.status == ValidationStatus::Impossible);
    let has_error = results.iter().any(|r| r.status == ValidationStatus::Error);

    assert!(has_valid, "Should have at least one valid block");
    assert!(has_impossible, "Should have at least one impossible block");
    assert!(has_error, "Should have at least one error");
}

// ==================== Repository Integration Tests ====================

#[tokio::test]
async fn test_local_repository_validation_storage() {
    let repo = LocalRepository::new();

    // Create schedule with some blocks
    let schedule = Schedule {
        id: None,
        name: "Validation Test Schedule".to_string(),
        blocks: vec![
            create_test_block(1, 5.0, 180.0, 45.0, 600.0, 3600.0, 5.0),
            create_test_block(2, 3.0, 90.0, 30.0, 600.0, 3600.0, 0.0), // Zero visibility
            create_test_block(3, -1.0, 270.0, 60.0, 600.0, 3600.0, 5.0), // Negative priority
        ],
        dark_periods: vec![],
        checksum: "validation_test_123".to_string(),
    };

    let metadata = repo.store_schedule(&schedule).await.unwrap();
    let schedule_id = metadata.schedule_id.unwrap();

    // Initially no validation results
    assert!(!repo.has_validation_results(schedule_id).await.unwrap());

    // Populate analytics (which includes validation)
    let analytics_count = repo.populate_schedule_analytics(schedule_id).await.unwrap();
    assert_eq!(analytics_count, 3); // Should return block count

    // Now should have validation results
    assert!(repo.has_validation_results(schedule_id).await.unwrap());

    // Fetch and verify validation results
    let validation_report = repo.fetch_validation_results(schedule_id).await.unwrap();

    assert_eq!(validation_report.schedule_id, schedule_id);
    assert_eq!(validation_report.total_blocks, 3);

    // Should have at least one valid, one impossible, and one error
    assert!(validation_report.valid_blocks > 0);
    assert!(!validation_report.impossible_blocks.is_empty());
    assert!(!validation_report.validation_errors.is_empty());

    // Check issue details
    for issue in &validation_report.impossible_blocks {
        assert!(!issue.category.is_empty());
        assert!(!issue.issue_type.is_empty());
        assert!(!issue.description.is_empty());
    }

    // Delete validation results
    let deleted_count = repo.delete_validation_results(schedule_id).await.unwrap();
    assert_eq!(deleted_count, 1); // One report deleted
    assert!(!repo.has_validation_results(schedule_id).await.unwrap());
}

#[tokio::test]
async fn test_validation_report_structure() {
    let repo = LocalRepository::new();

    let schedule = Schedule {
        id: None,
        name: "Report Structure Test".to_string(),
        blocks: vec![
            // Valid block
            create_test_block(1, 5.0, 180.0, 45.0, 600.0, 3600.0, 5.0),
            // Impossible (zero visibility)
            create_test_block(2, 5.0, 180.0, 45.0, 600.0, 3600.0, 0.0),
            // Error (negative priority)
            create_test_block(3, -1.0, 180.0, 45.0, 600.0, 3600.0, 5.0),
            // Warning (narrow elevation range) - need to test this differently
        ],
        dark_periods: vec![],
        checksum: "report_test_456".to_string(),
    };

    let metadata = repo.store_schedule(&schedule).await.unwrap();
    let schedule_id = metadata.schedule_id.unwrap();

    repo.populate_schedule_analytics(schedule_id).await.unwrap();

    let report = repo.fetch_validation_results(schedule_id).await.unwrap();

    // Verify report structure
    println!("Report: {:?}", report);
    println!("Valid blocks: {}", report.valid_blocks);
    println!("Impossible blocks: {}", report.impossible_blocks.len());
    println!("Validation errors: {}", report.validation_errors.len());
    println!("Validation warnings: {}", report.validation_warnings.len());

    assert_eq!(report.total_blocks, 3);
    assert!(
        report.valid_blocks >= 1,
        "Should have at least 1 valid block"
    );
    assert!(
        !report.impossible_blocks.is_empty(),
        "Should have impossible blocks"
    );
    assert!(
        !report.validation_errors.is_empty(),
        "Should have validation errors"
    );

    // Verify issue fields are properly populated
    if let Some(impossible) = report.impossible_blocks.first() {
        assert!(impossible.block_id > 0);
        assert!(!impossible.issue_type.is_empty());
        assert!(!impossible.category.is_empty());
        assert!(!impossible.description.is_empty());
        // Should use as_str() format, not Debug
        assert!(!impossible.category.contains("IssueCategory"));
        assert!(!impossible.criticality.contains("Criticality"));
    }

    if let Some(error) = report.validation_errors.first() {
        assert!(error.block_id > 0);
        assert!(!error.issue_type.is_empty());
        assert!(!error.category.is_empty());
        assert!(!error.criticality.is_empty());
        assert!(!error.description.is_empty());
        // Should use as_str() format, not Debug
        assert!(!error.category.contains("IssueCategory"));
        assert!(!error.criticality.contains("Criticality"));
    }
}

#[tokio::test]
async fn test_validation_empty_schedule() {
    let repo = LocalRepository::new();

    let schedule = Schedule {
        id: None,
        name: "Empty Schedule".to_string(),
        blocks: vec![],
        dark_periods: vec![],
        checksum: "empty_123".to_string(),
    };

    let metadata = repo.store_schedule(&schedule).await.unwrap();
    let schedule_id = metadata.schedule_id.unwrap();

    let analytics_count = repo.populate_schedule_analytics(schedule_id).await.unwrap();
    assert_eq!(analytics_count, 0);

    // Should still create validation results (even if empty)
    assert!(repo.has_validation_results(schedule_id).await.unwrap());

    let report = repo.fetch_validation_results(schedule_id).await.unwrap();
    assert_eq!(report.total_blocks, 0);
    assert_eq!(report.valid_blocks, 0);
    assert!(report.impossible_blocks.is_empty());
    assert!(report.validation_errors.is_empty());
}

#[tokio::test]
async fn test_validation_all_valid_blocks() {
    let repo = LocalRepository::new();

    let schedule = Schedule {
        id: None,
        name: "All Valid Schedule".to_string(),
        blocks: vec![
            create_test_block(1, 5.0, 180.0, 45.0, 600.0, 3600.0, 5.0),
            create_test_block(2, 3.0, 90.0, 30.0, 600.0, 3600.0, 5.0),
            create_test_block(3, 7.0, 270.0, 60.0, 600.0, 3600.0, 5.0),
        ],
        dark_periods: vec![],
        checksum: "all_valid_789".to_string(),
    };

    let metadata = repo.store_schedule(&schedule).await.unwrap();
    let schedule_id = metadata.schedule_id.unwrap();

    repo.populate_schedule_analytics(schedule_id).await.unwrap();

    let report = repo.fetch_validation_results(schedule_id).await.unwrap();

    assert_eq!(report.total_blocks, 3);
    assert_eq!(report.valid_blocks, 3);
    assert!(report.impossible_blocks.is_empty());
    assert!(report.validation_errors.is_empty());
    assert!(report.validation_warnings.is_empty());
}
