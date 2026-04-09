//! Tests for API types including Period, Constraints, and ID types.

use tsi_rust::api::*;
use tsi_rust::models::ModifiedJulianDate;

#[test]
fn test_period_creation() {
    let start = ModifiedJulianDate::new(59000.0);
    let end = ModifiedJulianDate::new(59001.0);

    let period = Period::new(start, end);
    assert_eq!(period.start.value(), 59000.0);
    assert_eq!(period.end.value(), 59001.0);
}

#[test]
fn test_period_try_new_invalid_order() {
    let start = ModifiedJulianDate::new(59001.0);
    let end = ModifiedJulianDate::new(59000.0);

    assert!(Period::try_new(start, end).is_err());
}

#[test]
fn test_period_duration() {
    let period = Period::new(
        ModifiedJulianDate::new(59000.0),
        ModifiedJulianDate::new(59003.5),
    );
    let duration = period.duration();
    assert_eq!(duration.value(), 3.5);
}

#[test]
fn test_period_contains() {
    let period = Period::new(
        ModifiedJulianDate::new(59000.0),
        ModifiedJulianDate::new(59001.0),
    );
    let t_in = ModifiedJulianDate::new(59000.5);
    let t_at_start = ModifiedJulianDate::new(59000.0);
    let t_at_end = ModifiedJulianDate::new(59001.0);
    let t_before = ModifiedJulianDate::new(58999.0);

    assert!(period.start <= t_in && t_in < period.end);
    assert!(period.start <= t_at_start && t_at_start < period.end);
    assert!(!(period.start <= t_at_end && t_at_end < period.end)); // Exclusive end
    assert!(!(period.start <= t_before && t_before < period.end));
}

#[test]
fn test_period_contains_range() {
    let period = Period::new(
        ModifiedJulianDate::new(59000.0),
        ModifiedJulianDate::new(59001.0),
    );
    // Inclusive range using start <= t <= end
    assert!(period.start.value() <= 59000.0 && 59000.0 <= period.end.value());
    assert!(period.start.value() <= 59000.5 && 59000.5 <= period.end.value());
    assert!(period.start.value() <= 59001.0 && 59001.0 <= period.end.value());
    assert!(!(period.start.value() <= 58999.0 && 58999.0 <= period.end.value()));
    assert!(!(period.start.value() <= 59002.0 && 59002.0 <= period.end.value()));
}

#[test]
fn test_period_overlaps() {
    let period1 = Period::new(
        ModifiedJulianDate::new(59000.0),
        ModifiedJulianDate::new(59002.0),
    );
    let period2 = Period::new(
        ModifiedJulianDate::new(59001.0),
        ModifiedJulianDate::new(59003.0),
    );
    let period3 = Period::new(
        ModifiedJulianDate::new(59005.0),
        ModifiedJulianDate::new(59006.0),
    );

    assert!(period1.intersection(&period2).is_some());
    assert!(period2.intersection(&period1).is_some());
    assert!(period1.intersection(&period3).is_none());
    assert!(period3.intersection(&period1).is_none());
}

#[test]
fn test_period_from_mjd() {
    let period = Period::new(
        ModifiedJulianDate::new(59000.0),
        ModifiedJulianDate::new(59001.0),
    );
    assert_eq!(period.start.value(), 59000.0);
    assert_eq!(period.end.value(), 59001.0);
}

#[test]
fn test_constraints_creation() {
    let constraints = Constraints::new(
        qtty::Degrees::new(20.0),
        qtty::Degrees::new(80.0),
        qtty::Degrees::new(0.0),
        qtty::Degrees::new(360.0),
        None,
    );

    assert_eq!(constraints.min_alt.value(), 20.0);
    assert_eq!(constraints.max_alt.value(), 80.0);
    assert_eq!(constraints.min_az.value(), 0.0);
    assert_eq!(constraints.max_az.value(), 360.0);
    assert!(constraints.fixed_time.is_none());
}

#[test]
fn test_constraints_with_fixed_time() {
    let period = Period::new(ModifiedJulianDate::new(59000.0), ModifiedJulianDate::new(59001.0));
    let constraints = Constraints::new(
        qtty::Degrees::new(20.0),
        qtty::Degrees::new(80.0),
        qtty::Degrees::new(0.0),
        qtty::Degrees::new(360.0),
        Some(period),
    );

    assert!(constraints.fixed_time.is_some());
    let fixed_time = constraints.fixed_time.unwrap();
    assert_eq!(fixed_time.start.value(), 59000.0);
}

#[test]
fn test_constraints_debug() {
    let constraints = Constraints::new(
        qtty::Degrees::new(20.5),
        qtty::Degrees::new(80.3),
        qtty::Degrees::new(0.0),
        qtty::Degrees::new(360.0),
        None,
    );

    let debug = format!("{:?}", constraints);
    assert!(debug.contains("Constraints"));
}

#[test]
fn test_schedule_id_display() {
    let id = ScheduleId::new(42);
    assert_eq!(format!("{}", id), "42");
}

#[test]
fn test_target_id_display() {
    let id = TargetId::new(123);
    assert_eq!(format!("{}", id), "123");
}

#[test]
fn test_constraints_id_display() {
    let id = ConstraintsId::new(456);
    assert_eq!(format!("{}", id), "456");
}

#[test]
fn test_scheduling_block_id_display() {
    let id = SchedulingBlockId::new(789);
    assert_eq!(format!("{}", id), "789");
}

#[test]
fn test_schedule_id_from_i64() {
    let id = ScheduleId::new(42);
    let value: i64 = id.into();
    assert_eq!(value, 42);
}

#[test]
fn test_all_id_types_value_getter() {
    let schedule_id = ScheduleId::new(1);
    let target_id = TargetId::new(2);
    let constraints_id = ConstraintsId::new(3);
    let block_id = SchedulingBlockId::new(4);

    assert_eq!(schedule_id.value(), 1);
    assert_eq!(target_id.value(), 2);
    assert_eq!(constraints_id.value(), 3);
    assert_eq!(block_id.value(), 4);
}

#[test]
fn test_period_serialization() {
    let period = Period::new(ModifiedJulianDate::new(59000.0), ModifiedJulianDate::new(59001.0));
    let json = serde_json::to_string(&period).unwrap();
    assert!(json.contains("59000"));
    assert!(json.contains("59001"));
}

#[test]
fn test_period_deserialization() {
    let json = r#"{"start_mjd":59000.0,"end_mjd":59001.0}"#;
    let period: Period = serde_json::from_str(json).unwrap();
    assert_eq!(period.start.value(), 59000.0);
    assert_eq!(period.end.value(), 59001.0);
}

#[test]
fn test_constraints_serialization() {
    let constraints = Constraints::new(
        qtty::Degrees::new(20.0),
        qtty::Degrees::new(80.0),
        qtty::Degrees::new(0.0),
        qtty::Degrees::new(360.0),
        None,
    );

    let json = serde_json::to_string(&constraints).unwrap();
    assert!(json.contains("20"));
    assert!(json.contains("80"));
}

#[test]
fn test_schedule_id_serialization() {
    let id = ScheduleId::new(42);
    let json = serde_json::to_string(&id).unwrap();
    assert!(json.contains("42"));
}
