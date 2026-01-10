use tsi_rust::api::{Schedule, ScheduleId};
use tsi_rust::db::repositories::LocalRepository;
use tsi_rust::db::services;
use tsi_rust::routes;

fn create_minimal_schedule(name: &str) -> Schedule {
    Schedule {
        id: None,
        name: name.to_string(),
        blocks: vec![],
        dark_periods: vec![],
        checksum: format!("test_{}", name),
    }
}

#[tokio::test]
async fn test_landing_list_schedules() {
    let repo = LocalRepository::new();
    let schedule = create_minimal_schedule("test1");
    let _ = services::store_schedule(&repo, &schedule).await;

    let schedules = services::list_schedules(&repo).await.unwrap();
    assert!(schedules.len() > 0);
}

#[test]
fn test_routes_module_exists() {
    // Ensure routes module compiles and exports expected constants
    assert_eq!(routes::compare::GET_COMPARE_DATA, "get_compare_data");
    assert_eq!(routes::distribution::GET_DISTRIBUTION_DATA, "get_distribution_data");
    assert_eq!(routes::insights::GET_INSIGHTS_DATA, "get_insights_data");
    assert_eq!(routes::skymap::GET_SKY_MAP_DATA, "get_sky_map_data");
    assert_eq!(routes::timeline::GET_SCHEDULE_TIMELINE_DATA, "get_schedule_timeline_data");
    assert_eq!(routes::trends::GET_TRENDS_DATA, "get_trends_data");
    assert_eq!(routes::validation::GET_VALIDATION_REPORT, "get_validation_report");
    assert_eq!(routes::landing::LIST_SCHEDULES, "list_schedules");
    assert_eq!(routes::landing::POST_SCHEDULE, "store_schedule");
}

#[test]
fn test_schedule_info_creation() {
    let info = routes::landing::ScheduleInfo {
        schedule_id: ScheduleId::new(1),
        schedule_name: "test".to_string(),
    };
    assert_eq!(info.schedule_id.value(), 1);
    assert_eq!(info.schedule_name, "test");
}

#[test]
fn test_compare_block_basic() {
    let block = routes::compare::CompareBlock {
        scheduling_block_id: "test".to_string(),
        priority: 5.0,
        scheduled: true,
        requested_hours: qtty::Hours::new(1.0),
    };
    assert_eq!(block.priority, 5.0);
    assert!(block.scheduled);
    assert_eq!(block.requested_hours.value(), 1.0);
}

#[test]
fn test_distribution_block_basic() {
    let block = routes::distribution::DistributionBlock {
        priority: 5.0,
        total_visibility_hours: qtty::Hours::new(10.0),
        requested_hours: qtty::Hours::new(2.0),
        elevation_range_deg: qtty::Degrees::new(30.0),
        scheduled: true,
    };
    assert_eq!(block.priority, 5.0);
    assert!(block.scheduled);
}

#[test]
fn test_priority_bin_info() {
    let bin = routes::skymap::PriorityBinInfo {
        label: "Bin 1".to_string(),
        min_priority: 0.0,
        max_priority: 2.5,
        color: "#ff0000".to_string(),
    };
    assert_eq!(bin.min_priority, 0.0);
    assert_eq!(bin.max_priority, 2.5);
    assert_eq!(bin.color, "#ff0000");
}

#[test]
fn test_route_constants_are_strings() {
    // Verify all route constants are strings (prevents typos)
    let _: &str = routes::compare::GET_COMPARE_DATA;
    let _: &str = routes::distribution::GET_DISTRIBUTION_DATA;
    let _: &str = routes::insights::GET_INSIGHTS_DATA;
    let _: &str = routes::skymap::GET_SKY_MAP_DATA;
    let _: &str = routes::timeline::GET_SCHEDULE_TIMELINE_DATA;
    let _: &str = routes::trends::GET_TRENDS_DATA;
    let _: &str = routes::validation::GET_VALIDATION_REPORT;
    let _: &str = routes::landing::LIST_SCHEDULES;
    let _: &str = routes::landing::POST_SCHEDULE;
}
