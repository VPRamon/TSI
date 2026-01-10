//! Integration tests for routes/visibility.rs and routes/landing.rs
//! These tests exercise route module structure without requiring Python runtime.

use tsi_rust::routes;

#[test]
fn test_visibility_constants_values() {
    assert_eq!(
        routes::visibility::GET_VISIBILITY_MAP_DATA,
        "get_visibility_map_data"
    );
    assert_eq!(
        routes::visibility::GET_SCHEDULE_TIME_RANGE,
        "get_schedule_time_range"
    );
    assert_eq!(
        routes::visibility::GET_VISIBILITY_HISTOGRAM,
        "get_visibility_histogram"
    );
}

#[test]
fn test_landing_constants_values() {
    assert_eq!(routes::landing::LIST_SCHEDULES, "list_schedules");
    assert_eq!(routes::landing::POST_SCHEDULE, "store_schedule");
}

#[test]
fn test_visibility_block_summary_creation() {
    use tsi_rust::routes::visibility::VisibilityBlockSummary;

    let summary = VisibilityBlockSummary {
        scheduling_block_id: 1,
        original_block_id: "block-1".to_string(),
        priority: 5.0,
        num_visibility_periods: 3,
        scheduled: true,
    };

    assert_eq!(summary.scheduling_block_id, 1);
    assert_eq!(summary.original_block_id, "block-1");
    assert_eq!(summary.priority, 5.0);
    assert_eq!(summary.num_visibility_periods, 3);
    assert!(summary.scheduled);
}

#[test]
fn test_visibility_map_data_creation() {
    use tsi_rust::routes::visibility::{VisibilityBlockSummary, VisibilityMapData};

    let block = VisibilityBlockSummary {
        scheduling_block_id: 1,
        original_block_id: "block-1".to_string(),
        priority: 5.0,
        num_visibility_periods: 3,
        scheduled: false,
    };

    let data = VisibilityMapData {
        blocks: vec![block],
        priority_min: 1.0,
        priority_max: 10.0,
        total_count: 1,
        scheduled_count: 0,
    };

    assert_eq!(data.blocks.len(), 1);
    assert_eq!(data.priority_min, 1.0);
    assert_eq!(data.priority_max, 10.0);
    assert_eq!(data.total_count, 1);
    assert_eq!(data.scheduled_count, 0);
}

#[test]
fn test_schedule_info_creation() {
    use tsi_rust::api::ScheduleId;
    use tsi_rust::routes::landing::ScheduleInfo;

    let info = ScheduleInfo {
        schedule_id: ScheduleId::new(42),
        schedule_name: "My Schedule".to_string(),
    };

    assert_eq!(info.schedule_id.value(), 42);
    assert_eq!(info.schedule_name, "My Schedule");
}

#[test]
fn test_visibility_block_summary_serialization() {
    use tsi_rust::routes::visibility::VisibilityBlockSummary;

    let summary = VisibilityBlockSummary {
        scheduling_block_id: 1,
        original_block_id: "block-1".to_string(),
        priority: 5.0,
        num_visibility_periods: 3,
        scheduled: true,
    };

    let json = serde_json::to_string(&summary).unwrap();
    assert!(json.contains("\"scheduling_block_id\":1"));
    assert!(json.contains("\"priority\":5.0"));
    assert!(json.contains("\"scheduled\":true"));
}

#[test]
fn test_visibility_map_data_serialization() {
    use tsi_rust::routes::visibility::VisibilityMapData;

    let data = VisibilityMapData {
        blocks: vec![],
        priority_min: 0.0,
        priority_max: 10.0,
        total_count: 0,
        scheduled_count: 0,
    };

    let json = serde_json::to_string(&data).unwrap();
    assert!(json.contains("\"priority_min\":0.0"));
    assert!(json.contains("\"total_count\":0"));
}

#[test]
fn test_schedule_info_serialization() {
    use tsi_rust::api::ScheduleId;
    use tsi_rust::routes::landing::ScheduleInfo;

    let info = ScheduleInfo {
        schedule_id: ScheduleId::new(42),
        schedule_name: "Test".to_string(),
    };

    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("\"schedule_name\":\"Test\""));
}

#[test]
fn test_visibility_block_summary_clone() {
    use tsi_rust::routes::visibility::VisibilityBlockSummary;

    let summary = VisibilityBlockSummary {
        scheduling_block_id: 1,
        original_block_id: "test".to_string(),
        priority: 3.0,
        num_visibility_periods: 5,
        scheduled: false,
    };

    let cloned = summary.clone();
    assert_eq!(cloned.scheduling_block_id, summary.scheduling_block_id);
    assert_eq!(cloned.priority, summary.priority);
}

#[test]
fn test_visibility_map_data_clone() {
    use tsi_rust::routes::visibility::VisibilityMapData;

    let data = VisibilityMapData {
        blocks: vec![],
        priority_min: 0.0,
        priority_max: 10.0,
        total_count: 0,
        scheduled_count: 0,
    };

    let cloned = data.clone();
    assert_eq!(cloned.total_count, data.total_count);
}

#[test]
fn test_schedule_info_clone() {
    use tsi_rust::api::ScheduleId;
    use tsi_rust::routes::landing::ScheduleInfo;

    let info = ScheduleInfo {
        schedule_id: ScheduleId::new(1),
        schedule_name: "Test".to_string(),
    };

    let cloned = info.clone();
    assert_eq!(cloned.schedule_id.value(), info.schedule_id.value());
}

#[test]
fn test_visibility_block_summary_debug() {
    use tsi_rust::routes::visibility::VisibilityBlockSummary;

    let summary = VisibilityBlockSummary {
        scheduling_block_id: 1,
        original_block_id: "test".to_string(),
        priority: 3.0,
        num_visibility_periods: 5,
        scheduled: false,
    };

    let debug_str = format!("{:?}", summary);
    assert!(debug_str.contains("VisibilityBlockSummary"));
}

#[test]
fn test_visibility_map_data_debug() {
    use tsi_rust::routes::visibility::VisibilityMapData;

    let data = VisibilityMapData {
        blocks: vec![],
        priority_min: 0.0,
        priority_max: 10.0,
        total_count: 0,
        scheduled_count: 0,
    };

    let debug_str = format!("{:?}", data);
    assert!(debug_str.contains("VisibilityMapData"));
}

#[test]
fn test_schedule_info_debug() {
    use tsi_rust::api::ScheduleId;
    use tsi_rust::routes::landing::ScheduleInfo;

    let info = ScheduleInfo {
        schedule_id: ScheduleId::new(1),
        schedule_name: "Test".to_string(),
    };

    let debug_str = format!("{:?}", info);
    assert!(debug_str.contains("ScheduleInfo"));
}
