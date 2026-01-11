//! Edge case tests for API types.
//!
//! These tests cover boundary conditions, invalid constraints, extreme values,
//! and other edge cases in Period, Constraints, and other API types.

use tsi_rust::api::{
    Constraints, ConstraintsId, Period, ScheduleId, SchedulingBlock, SchedulingBlockId, TargetId,
};
use tsi_rust::models::ModifiedJulianDate;

// =========================================================
// Period Edge Cases
// =========================================================

#[test]
fn test_period_boundary_start_equals_stop() {
    // Period where start equals stop (zero duration)
    let period = Period {
        start: ModifiedJulianDate::new(59580.0),
        stop: ModifiedJulianDate::new(59580.0),
    };

    let duration = period.duration();
    assert_eq!(duration.value(), 0.0);
}

#[test]
fn test_period_inverted_start_after_stop() {
    // Period where start is after stop (negative duration)
    let period = Period {
        start: ModifiedJulianDate::new(59582.0),
        stop: ModifiedJulianDate::new(59580.0),
    };

    let duration = period.duration();
    assert!(duration.value() < 0.0);
    assert_eq!(duration.value(), -2.0);
}

#[test]
fn test_period_new_validates_ordering() {
    // Period::new should return None for invalid ordering
    let valid = Period::new(
        ModifiedJulianDate::new(59580.0),
        ModifiedJulianDate::new(59582.0),
    );
    assert!(valid.is_some());

    let invalid = Period::new(
        ModifiedJulianDate::new(59582.0),
        ModifiedJulianDate::new(59580.0),
    );
    assert!(invalid.is_none());
}

#[test]
fn test_period_very_small_duration() {
    // Extremely small time period (microseconds in MJD)
    let period = Period {
        start: ModifiedJulianDate::new(59580.0),
        stop: ModifiedJulianDate::new(59580.0 + 1e-9), // ~86 nanoseconds
    };

    let duration = period.duration();
    assert!(duration.value() > 0.0);
    assert!(duration.value() < 1e-8);
}

#[test]
fn test_period_very_large_duration() {
    // Period spanning thousands of days
    let period = Period {
        start: ModifiedJulianDate::new(50000.0),
        stop: ModifiedJulianDate::new(60000.0),
    };

    let duration = period.duration();
    assert_eq!(duration.value(), 10000.0);
}

#[test]
fn test_period_contains_boundary_inclusive_start() {
    let period = Period {
        start: ModifiedJulianDate::new(59580.0),
        stop: ModifiedJulianDate::new(59582.0),
    };

    // Start is inclusive
    assert!(period.contains(ModifiedJulianDate::new(59580.0)));
}

#[test]
fn test_period_contains_boundary_exclusive_end() {
    let period = Period {
        start: ModifiedJulianDate::new(59580.0),
        stop: ModifiedJulianDate::new(59582.0),
    };

    // End is exclusive
    assert!(!period.contains(ModifiedJulianDate::new(59582.0)));
}

#[test]
fn test_period_contains_just_before_end() {
    let period = Period {
        start: ModifiedJulianDate::new(59580.0),
        stop: ModifiedJulianDate::new(59582.0),
    };

    // Just before end should be included
    assert!(period.contains(ModifiedJulianDate::new(59581.9999)));
}

#[test]
fn test_period_overlaps_exact_boundaries() {
    // Periods that touch at boundaries
    let period1 = Period {
        start: ModifiedJulianDate::new(59580.0),
        stop: ModifiedJulianDate::new(59582.0),
    };

    let period2 = Period {
        start: ModifiedJulianDate::new(59582.0),
        stop: ModifiedJulianDate::new(59584.0),
    };

    // Should not overlap (period1 ends where period2 starts)
    assert!(!period1.overlaps(&period2));
    assert!(!period2.overlaps(&period1));
}

#[test]
fn test_period_overlaps_partial() {
    let period1 = Period {
        start: ModifiedJulianDate::new(59580.0),
        stop: ModifiedJulianDate::new(59582.0),
    };

    let period2 = Period {
        start: ModifiedJulianDate::new(59581.0),
        stop: ModifiedJulianDate::new(59583.0),
    };

    assert!(period1.overlaps(&period2));
    assert!(period2.overlaps(&period1));
}

#[test]
fn test_period_overlaps_complete_containment() {
    let period1 = Period {
        start: ModifiedJulianDate::new(59580.0),
        stop: ModifiedJulianDate::new(59585.0),
    };

    let period2 = Period {
        start: ModifiedJulianDate::new(59581.0),
        stop: ModifiedJulianDate::new(59582.0),
    };

    // period2 is completely inside period1
    assert!(period1.overlaps(&period2));
    assert!(period2.overlaps(&period1));
}

#[test]
fn test_period_negative_mjd() {
    // MJD can be negative for dates before 1858-11-17
    let period = Period {
        start: ModifiedJulianDate::new(-1000.0),
        stop: ModifiedJulianDate::new(-500.0),
    };

    let duration = period.duration();
    assert_eq!(duration.value(), 500.0);
}

#[test]
fn test_period_extreme_mjd_values() {
    // Test with very large MJD values
    let period = Period {
        start: ModifiedJulianDate::new(100000.0),
        stop: ModifiedJulianDate::new(100001.0),
    };

    assert!(period.contains(ModifiedJulianDate::new(100000.5)));
}

// =========================================================
// Constraints Edge Cases
// =========================================================

#[test]
fn test_constraints_altitude_minimum_range() {
    // Minimum altitude range (0 degrees)
    let c = Constraints {
        min_alt: qtty::Degrees::new(45.0),
        max_alt: qtty::Degrees::new(45.0),
        min_az: qtty::Degrees::new(0.0),
        max_az: qtty::Degrees::new(360.0),
        fixed_time: None,
    };

    assert_eq!(c.min_alt.value(), c.max_alt.value());
}

#[test]
fn test_constraints_altitude_full_range() {
    // Full altitude range (0-90 degrees)
    let c = Constraints {
        min_alt: qtty::Degrees::new(0.0),
        max_alt: qtty::Degrees::new(90.0),
        min_az: qtty::Degrees::new(0.0),
        max_az: qtty::Degrees::new(360.0),
        fixed_time: None,
    };

    assert_eq!(c.max_alt.value() - c.min_alt.value(), 90.0);
}

#[test]
fn test_constraints_altitude_inverted() {
    // Invalid: max_alt < min_alt
    let c = Constraints {
        min_alt: qtty::Degrees::new(85.0),
        max_alt: qtty::Degrees::new(30.0),
        min_az: qtty::Degrees::new(0.0),
        max_az: qtty::Degrees::new(360.0),
        fixed_time: None,
    };

    // Should construct but be invalid
    assert!(c.min_alt.value() > c.max_alt.value());
}

#[test]
fn test_constraints_altitude_negative() {
    // Negative altitudes (below horizon)
    let c = Constraints {
        min_alt: qtty::Degrees::new(-10.0),
        max_alt: qtty::Degrees::new(20.0),
        min_az: qtty::Degrees::new(0.0),
        max_az: qtty::Degrees::new(360.0),
        fixed_time: None,
    };

    assert!(c.min_alt.value() < 0.0);
}

#[test]
fn test_constraints_altitude_above_90() {
    // Altitude above 90 degrees (invalid but tests edge case)
    let c = Constraints {
        min_alt: qtty::Degrees::new(85.0),
        max_alt: qtty::Degrees::new(120.0),
        min_az: qtty::Degrees::new(0.0),
        max_az: qtty::Degrees::new(360.0),
        fixed_time: None,
    };

    assert!(c.max_alt.value() > 90.0);
}

#[test]
fn test_constraints_azimuth_full_circle() {
    // Full azimuth range
    let c = Constraints {
        min_alt: qtty::Degrees::new(30.0),
        max_alt: qtty::Degrees::new(85.0),
        min_az: qtty::Degrees::new(0.0),
        max_az: qtty::Degrees::new(360.0),
        fixed_time: None,
    };

    assert_eq!(c.max_az.value() - c.min_az.value(), 360.0);
}

#[test]
fn test_constraints_azimuth_narrow_range() {
    // Very narrow azimuth range (1 degree)
    let c = Constraints {
        min_alt: qtty::Degrees::new(30.0),
        max_alt: qtty::Degrees::new(85.0),
        min_az: qtty::Degrees::new(180.0),
        max_az: qtty::Degrees::new(181.0),
        fixed_time: None,
    };

    assert_eq!(c.max_az.value() - c.min_az.value(), 1.0);
}

#[test]
fn test_constraints_azimuth_wrapping() {
    // Azimuth range that crosses 0/360 boundary
    let c = Constraints {
        min_alt: qtty::Degrees::new(30.0),
        max_alt: qtty::Degrees::new(85.0),
        min_az: qtty::Degrees::new(350.0),
        max_az: qtty::Degrees::new(10.0),
        fixed_time: None,
    };

    // This represents a valid wrapping constraint
    assert!(c.min_az.value() > c.max_az.value());
}

#[test]
fn test_constraints_azimuth_negative() {
    // Negative azimuth values
    let c = Constraints {
        min_alt: qtty::Degrees::new(30.0),
        max_alt: qtty::Degrees::new(85.0),
        min_az: qtty::Degrees::new(-10.0),
        max_az: qtty::Degrees::new(10.0),
        fixed_time: None,
    };

    assert!(c.min_az.value() < 0.0);
}

#[test]
fn test_constraints_azimuth_above_360() {
    // Azimuth values above 360
    let c = Constraints {
        min_alt: qtty::Degrees::new(30.0),
        max_alt: qtty::Degrees::new(85.0),
        min_az: qtty::Degrees::new(350.0),
        max_az: qtty::Degrees::new(370.0),
        fixed_time: None,
    };

    assert!(c.max_az.value() > 360.0);
}

#[test]
fn test_constraints_with_fixed_time_zero_duration() {
    // Fixed time with zero duration
    let fixed = Period {
        start: ModifiedJulianDate::new(59580.0),
        stop: ModifiedJulianDate::new(59580.0),
    };

    let c = Constraints {
        min_alt: qtty::Degrees::new(30.0),
        max_alt: qtty::Degrees::new(85.0),
        min_az: qtty::Degrees::new(0.0),
        max_az: qtty::Degrees::new(360.0),
        fixed_time: Some(fixed),
    };

    assert!(c.fixed_time.is_some());
    let ft = c.fixed_time.unwrap();
    assert_eq!(ft.duration().value(), 0.0);
}

#[test]
fn test_constraints_all_zero_values() {
    // All constraint values at zero
    let c = Constraints {
        min_alt: qtty::Degrees::new(0.0),
        max_alt: qtty::Degrees::new(0.0),
        min_az: qtty::Degrees::new(0.0),
        max_az: qtty::Degrees::new(0.0),
        fixed_time: None,
    };

    assert_eq!(c.min_alt.value(), 0.0);
    assert_eq!(c.max_alt.value(), 0.0);
    assert_eq!(c.min_az.value(), 0.0);
    assert_eq!(c.max_az.value(), 0.0);
}

#[test]
fn test_constraints_extreme_values() {
    // Extreme constraint values
    let c = Constraints {
        min_alt: qtty::Degrees::new(-180.0),
        max_alt: qtty::Degrees::new(180.0),
        min_az: qtty::Degrees::new(-720.0),
        max_az: qtty::Degrees::new(720.0),
        fixed_time: None,
    };

    assert!(c.min_alt.value() < -90.0);
    assert!(c.max_alt.value() > 90.0);
    assert!(c.min_az.value() < 0.0);
    assert!(c.max_az.value() > 360.0);
}

// =========================================================
// SchedulingBlock Edge Cases
// =========================================================

#[test]
fn test_scheduling_block_zero_priority() {
    let block = SchedulingBlock {
        id: SchedulingBlockId::new(1),
        original_block_id: Some("block_001".to_string()),
        target_ra: qtty::Degrees::new(45.0),
        target_dec: qtty::Degrees::new(-30.0),
        constraints: Constraints {
            min_alt: qtty::Degrees::new(30.0),
            max_alt: qtty::Degrees::new(85.0),
            min_az: qtty::Degrees::new(0.0),
            max_az: qtty::Degrees::new(360.0),
            fixed_time: None,
        },
        priority: 0.0,
        min_observation: qtty::Seconds::new(60.0),
        requested_duration: qtty::Seconds::new(3600.0),
        visibility_periods: vec![],
        scheduled_period: None,
    };

    assert_eq!(block.priority, 0.0);
}

#[test]
fn test_scheduling_block_negative_priority() {
    let block = SchedulingBlock {
        id: SchedulingBlockId::new(1),
        original_block_id: Some("block_001".to_string()),
        target_ra: qtty::Degrees::new(45.0),
        target_dec: qtty::Degrees::new(-30.0),
        constraints: Constraints {
            min_alt: qtty::Degrees::new(30.0),
            max_alt: qtty::Degrees::new(85.0),
            min_az: qtty::Degrees::new(0.0),
            max_az: qtty::Degrees::new(360.0),
            fixed_time: None,
        },
        priority: -5.0,
        min_observation: qtty::Seconds::new(60.0),
        requested_duration: qtty::Seconds::new(3600.0),
        visibility_periods: vec![],
        scheduled_period: None,
    };

    assert!(block.priority < 0.0);
}

#[test]
fn test_scheduling_block_very_high_priority() {
    let block = SchedulingBlock {
        id: SchedulingBlockId::new(1),
        original_block_id: Some("block_001".to_string()),
        target_ra: qtty::Degrees::new(45.0),
        target_dec: qtty::Degrees::new(-30.0),
        constraints: Constraints {
            min_alt: qtty::Degrees::new(30.0),
            max_alt: qtty::Degrees::new(85.0),
            min_az: qtty::Degrees::new(0.0),
            max_az: qtty::Degrees::new(360.0),
            fixed_time: None,
        },
        priority: 1000000.0,
        min_observation: qtty::Seconds::new(60.0),
        requested_duration: qtty::Seconds::new(3600.0),
        visibility_periods: vec![],
        scheduled_period: None,
    };

    assert!(block.priority > 100000.0);
}

#[test]
fn test_scheduling_block_ra_boundaries() {
    // RA at 0 degrees
    let block1 = SchedulingBlock {
        id: SchedulingBlockId::new(1),
        original_block_id: Some("block_ra0".to_string()),
        target_ra: qtty::Degrees::new(0.0),
        target_dec: qtty::Degrees::new(0.0),
        constraints: Constraints {
            min_alt: qtty::Degrees::new(30.0),
            max_alt: qtty::Degrees::new(85.0),
            min_az: qtty::Degrees::new(0.0),
            max_az: qtty::Degrees::new(360.0),
            fixed_time: None,
        },
        priority: 5.0,
        min_observation: qtty::Seconds::new(60.0),
        requested_duration: qtty::Seconds::new(3600.0),
        visibility_periods: vec![],
        scheduled_period: None,
    };

    // RA at 360 degrees (equivalent to 0)
    let block2 = SchedulingBlock {
        id: SchedulingBlockId::new(2),
        original_block_id: Some("block_ra360".to_string()),
        target_ra: qtty::Degrees::new(360.0),
        target_dec: qtty::Degrees::new(0.0),
        constraints: Constraints {
            min_alt: qtty::Degrees::new(30.0),
            max_alt: qtty::Degrees::new(85.0),
            min_az: qtty::Degrees::new(0.0),
            max_az: qtty::Degrees::new(360.0),
            fixed_time: None,
        },
        priority: 5.0,
        min_observation: qtty::Seconds::new(60.0),
        requested_duration: qtty::Seconds::new(3600.0),
        visibility_periods: vec![],
        scheduled_period: None,
    };

    assert_eq!(block1.target_ra.value(), 0.0);
    assert_eq!(block2.target_ra.value(), 360.0);
}

#[test]
fn test_scheduling_block_dec_boundaries() {
    // Dec at -90 degrees (south pole)
    let block1 = SchedulingBlock {
        id: SchedulingBlockId::new(1),
        original_block_id: Some("south_pole".to_string()),
        target_ra: qtty::Degrees::new(0.0),
        target_dec: qtty::Degrees::new(-90.0),
        constraints: Constraints {
            min_alt: qtty::Degrees::new(30.0),
            max_alt: qtty::Degrees::new(85.0),
            min_az: qtty::Degrees::new(0.0),
            max_az: qtty::Degrees::new(360.0),
            fixed_time: None,
        },
        priority: 5.0,
        min_observation: qtty::Seconds::new(60.0),
        requested_duration: qtty::Seconds::new(3600.0),
        visibility_periods: vec![],
        scheduled_period: None,
    };

    // Dec at +90 degrees (north pole)
    let block2 = SchedulingBlock {
        id: SchedulingBlockId::new(2),
        original_block_id: Some("north_pole".to_string()),
        target_ra: qtty::Degrees::new(0.0),
        target_dec: qtty::Degrees::new(90.0),
        constraints: Constraints {
            min_alt: qtty::Degrees::new(30.0),
            max_alt: qtty::Degrees::new(85.0),
            min_az: qtty::Degrees::new(0.0),
            max_az: qtty::Degrees::new(360.0),
            fixed_time: None,
        },
        priority: 5.0,
        min_observation: qtty::Seconds::new(60.0),
        requested_duration: qtty::Seconds::new(3600.0),
        visibility_periods: vec![],
        scheduled_period: None,
    };

    assert_eq!(block1.target_dec.value(), -90.0);
    assert_eq!(block2.target_dec.value(), 90.0);
}

#[test]
fn test_scheduling_block_zero_observation_time() {
    let block = SchedulingBlock {
        id: SchedulingBlockId::new(1),
        original_block_id: Some("zero_obs".to_string()),
        target_ra: qtty::Degrees::new(45.0),
        target_dec: qtty::Degrees::new(-30.0),
        constraints: Constraints {
            min_alt: qtty::Degrees::new(30.0),
            max_alt: qtty::Degrees::new(85.0),
            min_az: qtty::Degrees::new(0.0),
            max_az: qtty::Degrees::new(360.0),
            fixed_time: None,
        },
        priority: 5.0,
        min_observation: qtty::Seconds::new(0.0),
        requested_duration: qtty::Seconds::new(0.0),
        visibility_periods: vec![],
        scheduled_period: None,
    };

    assert_eq!(block.min_observation.value(), 0.0);
    assert_eq!(block.requested_duration.value(), 0.0);
}

#[test]
fn test_scheduling_block_very_long_observation() {
    let block = SchedulingBlock {
        id: SchedulingBlockId::new(1),
        original_block_id: Some("long_obs".to_string()),
        target_ra: qtty::Degrees::new(45.0),
        target_dec: qtty::Degrees::new(-30.0),
        constraints: Constraints {
            min_alt: qtty::Degrees::new(30.0),
            max_alt: qtty::Degrees::new(85.0),
            min_az: qtty::Degrees::new(0.0),
            max_az: qtty::Degrees::new(360.0),
            fixed_time: None,
        },
        priority: 5.0,
        min_observation: qtty::Seconds::new(3600.0),
        requested_duration: qtty::Seconds::new(86400.0 * 30.0), // 30 days
        visibility_periods: vec![],
        scheduled_period: None,
    };

    assert_eq!(block.requested_duration.value(), 86400.0 * 30.0);
}

#[test]
fn test_scheduling_block_many_visibility_periods() {
    // Block with hundreds of visibility periods
    let visibility_periods: Vec<Period> = (0..500)
        .map(|i| Period {
            start: ModifiedJulianDate::new(59580.0 + i as f64),
            stop: ModifiedJulianDate::new(59580.5 + i as f64),
        })
        .collect();

    let block = SchedulingBlock {
        id: SchedulingBlockId::new(1),
        original_block_id: Some("many_periods".to_string()),
        target_ra: qtty::Degrees::new(45.0),
        target_dec: qtty::Degrees::new(-30.0),
        constraints: Constraints {
            min_alt: qtty::Degrees::new(30.0),
            max_alt: qtty::Degrees::new(85.0),
            min_az: qtty::Degrees::new(0.0),
            max_az: qtty::Degrees::new(360.0),
            fixed_time: None,
        },
        priority: 5.0,
        min_observation: qtty::Seconds::new(60.0),
        requested_duration: qtty::Seconds::new(3600.0),
        visibility_periods,
        scheduled_period: None,
    };

    assert_eq!(block.visibility_periods.len(), 500);
}

// =========================================================
// ID Type Edge Cases
// =========================================================

#[test]
fn test_schedule_id_zero() {
    let id = ScheduleId::new(0);
    assert_eq!(id.value(), 0);
}

#[test]
fn test_schedule_id_negative() {
    let id = ScheduleId::new(-1);
    assert_eq!(id.value(), -1);
}

#[test]
fn test_schedule_id_max_value() {
    let id = ScheduleId::new(i64::MAX);
    assert_eq!(id.value(), i64::MAX);
}

#[test]
fn test_schedule_id_min_value() {
    let id = ScheduleId::new(i64::MIN);
    assert_eq!(id.value(), i64::MIN);
}

#[test]
fn test_target_id_edge_values() {
    assert_eq!(TargetId::new(0).value(), 0);
    assert_eq!(TargetId::new(i64::MAX).value(), i64::MAX);
    assert_eq!(TargetId::new(i64::MIN).value(), i64::MIN);
}

#[test]
fn test_constraints_id_edge_values() {
    assert_eq!(ConstraintsId::new(0).value(), 0);
    assert_eq!(ConstraintsId::new(i64::MAX).value(), i64::MAX);
    assert_eq!(ConstraintsId::new(i64::MIN).value(), i64::MIN);
}

#[test]
fn test_scheduling_block_id_edge_values() {
    assert_eq!(SchedulingBlockId::new(0).value(), 0);
    assert_eq!(SchedulingBlockId::new(i64::MAX).value(), i64::MAX);
    assert_eq!(SchedulingBlockId::new(i64::MIN).value(), i64::MIN);
}
