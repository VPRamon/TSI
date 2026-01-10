//! Tests for API types including Period, Constraints, and ID types.

use tsi_rust::api::*;
use tsi_rust::models::ModifiedJulianDate;

#[test]
fn test_period_creation() {
    let start = ModifiedJulianDate::new(59000.0);
    let stop = ModifiedJulianDate::new(59001.0);
    
    let period = Period::new(start, stop);
    assert!(period.is_some());
    
    let period = period.unwrap();
    assert_eq!(period.start.value(), 59000.0);
    assert_eq!(period.stop.value(), 59001.0);
}

#[test]
fn test_period_invalid_order() {
    let start = ModifiedJulianDate::new(59001.0);
    let stop = ModifiedJulianDate::new(59000.0);
    
    let period = Period::new(start, stop);
    assert!(period.is_none());
}

#[test]
fn test_period_duration() {
    let start = ModifiedJulianDate::new(59000.0);
    let stop = ModifiedJulianDate::new(59003.5);
    
    let period = Period::new(start, stop).unwrap();
    let duration = period.duration();
    assert_eq!(duration.value(), 3.5);
}

#[test]
fn test_period_contains() {
    let start = ModifiedJulianDate::new(59000.0);
    let stop = ModifiedJulianDate::new(59001.0);
    let period = Period::new(start, stop).unwrap();
    
    assert!(period.contains(ModifiedJulianDate::new(59000.0)));
    assert!(period.contains(ModifiedJulianDate::new(59000.5)));
    assert!(!period.contains(ModifiedJulianDate::new(59001.0))); // Exclusive end
    assert!(!period.contains(ModifiedJulianDate::new(58999.0)));
}

#[test]
fn test_period_contains_mjd() {
    let period = Period::py_new(59000.0, 59001.0);
    
    assert!(period.contains_mjd(59000.0));
    assert!(period.contains_mjd(59000.5));
    assert!(period.contains_mjd(59001.0)); // Python version is inclusive
    assert!(!period.contains_mjd(58999.0));
    assert!(!period.contains_mjd(59002.0));
}

#[test]
fn test_period_overlaps() {
    let period1 = Period::new(
        ModifiedJulianDate::new(59000.0),
        ModifiedJulianDate::new(59002.0),
    ).unwrap();
    
    let period2 = Period::new(
        ModifiedJulianDate::new(59001.0),
        ModifiedJulianDate::new(59003.0),
    ).unwrap();
    
    assert!(period1.overlaps(&period2));
    assert!(period2.overlaps(&period1));
    
    let period3 = Period::new(
        ModifiedJulianDate::new(59005.0),
        ModifiedJulianDate::new(59006.0),
    ).unwrap();
    
    assert!(!period1.overlaps(&period3));
    assert!(!period3.overlaps(&period1));
}

#[test]
fn test_period_py_new() {
    let period = Period::py_new(59000.0, 59001.0);
    assert_eq!(period.start_mjd(), 59000.0);
    assert_eq!(period.stop_mjd(), 59001.0);
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
    let period = Period::py_new(59000.0, 59001.0);
    let constraints = Constraints::new(
        qtty::Degrees::new(20.0),
        qtty::Degrees::new(80.0),
        qtty::Degrees::new(0.0),
        qtty::Degrees::new(360.0),
        Some(period),
    );
    
    assert!(constraints.fixed_time.is_some());
    let fixed_time = constraints.fixed_time.unwrap();
    assert_eq!(fixed_time.start_mjd(), 59000.0);
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
    let period = Period::py_new(59000.0, 59001.0);
    let json = serde_json::to_string(&period).unwrap();
    assert!(json.contains("59000"));
    assert!(json.contains("59001"));
}

#[test]
fn test_period_deserialization() {
    let json = r#"{"start":59000.0,"stop":59001.0}"#;
    let period: Period = serde_json::from_str(json).unwrap();
    assert_eq!(period.start_mjd(), 59000.0);
    assert_eq!(period.stop_mjd(), 59001.0);
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

