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

// =========================================================
// Comprehensive tests for all route modules
// =========================================================

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

// Compare module tests
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
fn test_compare_stats_creation() {
    let stats = routes::compare::CompareStats {
        scheduled_count: 10,
        unscheduled_count: 5,
        total_priority: 75.0,
        mean_priority: 5.0,
        median_priority: 4.5,
        total_hours: qtty::Hours::new(20.0),
    };
    assert_eq!(stats.scheduled_count, 10);
    assert_eq!(stats.unscheduled_count, 5);
    assert_eq!(stats.total_priority, 75.0);
}

#[test]
fn test_scheduling_change_creation() {
    let change = routes::compare::SchedulingChange {
        scheduling_block_id: "block-1".to_string(),
        priority: 3.0,
        change_type: "newly_scheduled".to_string(),
    };
    assert_eq!(change.scheduling_block_id, "block-1");
    assert_eq!(change.change_type, "newly_scheduled");
}

#[test]
fn test_compare_data_creation() {
    let data = routes::compare::CompareData {
        current_blocks: vec![],
        comparison_blocks: vec![],
        current_stats: routes::compare::CompareStats {
            scheduled_count: 0,
            unscheduled_count: 0,
            total_priority: 0.0,
            mean_priority: 0.0,
            median_priority: 0.0,
            total_hours: qtty::Hours::new(0.0),
        },
        comparison_stats: routes::compare::CompareStats {
            scheduled_count: 0,
            unscheduled_count: 0,
            total_priority: 0.0,
            mean_priority: 0.0,
            median_priority: 0.0,
            total_hours: qtty::Hours::new(0.0),
        },
        common_ids: vec![],
        only_in_current: vec![],
        only_in_comparison: vec![],
        scheduling_changes: vec![],
        current_name: "Schedule A".to_string(),
        comparison_name: "Schedule B".to_string(),
    };
    assert_eq!(data.current_name, "Schedule A");
    assert_eq!(data.comparison_name, "Schedule B");
}

// Distribution module tests
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
fn test_distribution_stats_creation() {
    let stats = routes::distribution::DistributionStats {
        count: 50,
        mean: 5.5,
        median: 5.0,
        std_dev: 1.2,
        min: 2.0,
        max: 10.0,
        sum: 275.0,
    };
    assert_eq!(stats.count, 50);
    assert_eq!(stats.mean, 5.5);
    assert_eq!(stats.median, 5.0);
}

#[test]
fn test_distribution_data_creation() {
    let data = routes::distribution::DistributionData {
        blocks: vec![],
        priority_stats: routes::distribution::DistributionStats {
            count: 0,
            mean: 0.0,
            median: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            sum: 0.0,
        },
        visibility_stats: routes::distribution::DistributionStats {
            count: 0,
            mean: 0.0,
            median: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            sum: 0.0,
        },
        requested_hours_stats: routes::distribution::DistributionStats {
            count: 0,
            mean: 0.0,
            median: 0.0,
            std_dev: 0.0,
            min: 0.0,
            max: 0.0,
            sum: 0.0,
        },
        total_count: 0,
        scheduled_count: 0,
        unscheduled_count: 0,
        impossible_count: 0,
    };
    assert_eq!(data.total_count, 0);
}

// Insights module tests
#[test]
fn test_insights_block_creation() {
    let block = routes::insights::InsightsBlock {
        scheduling_block_id: 42,
        original_block_id: "obs-001".to_string(),
        priority: 8.0,
        total_visibility_hours: qtty::Hours::new(20.0),
        requested_hours: qtty::Hours::new(4.0),
        elevation_range_deg: qtty::Degrees::new(60.0),
        scheduled: true,
        scheduled_start_mjd: Some(tsi_rust::api::ModifiedJulianDate::new(59000.0)),
        scheduled_stop_mjd: Some(tsi_rust::api::ModifiedJulianDate::new(59001.0)),
    };
    assert_eq!(block.scheduling_block_id, 42);
    assert_eq!(block.original_block_id, "obs-001");
    assert!(block.scheduled);
}

#[test]
fn test_analytics_metrics_creation() {
    let metrics = routes::insights::AnalyticsMetrics {
        total_observations: 100,
        scheduled_count: 60,
        unscheduled_count: 40,
        scheduling_rate: 0.6,
        mean_priority: 5.5,
        median_priority: 5.0,
        mean_priority_scheduled: 6.0,
        mean_priority_unscheduled: 4.5,
        total_visibility_hours: qtty::Hours::new(500.0),
        mean_requested_hours: qtty::Hours::new(2.5),
    };
    assert_eq!(metrics.total_observations, 100);
    assert_eq!(metrics.scheduling_rate, 0.6);
}

#[test]
fn test_correlation_entry_creation() {
    let corr = routes::insights::CorrelationEntry {
        variable1: "priority".to_string(),
        variable2: "scheduled".to_string(),
        correlation: 0.85,
    };
    assert_eq!(corr.variable1, "priority");
    assert_eq!(corr.correlation, 0.85);
}

#[test]
fn test_conflict_record_creation() {
    let conflict = routes::insights::ConflictRecord {
        block_id_1: "block-1".to_string(),
        block_id_2: "block-2".to_string(),
        start_time_1: tsi_rust::api::ModifiedJulianDate::new(59000.0),
        stop_time_1: tsi_rust::api::ModifiedJulianDate::new(59001.0),
        start_time_2: tsi_rust::api::ModifiedJulianDate::new(59000.5),
        stop_time_2: tsi_rust::api::ModifiedJulianDate::new(59001.5),
        overlap_hours: qtty::Hours::new(12.0),
    };
    assert_eq!(conflict.block_id_1, "block-1");
    assert_eq!(conflict.overlap_hours.value(), 12.0);
}

#[test]
fn test_top_observation_creation() {
    let obs = routes::insights::TopObservation {
        scheduling_block_id: 1,
        original_block_id: "top-1".to_string(),
        priority: 10.0,
        total_visibility_hours: qtty::Hours::new(100.0),
        requested_hours: qtty::Hours::new(5.0),
        scheduled: true,
    };
    assert_eq!(obs.priority, 10.0);
    assert_eq!(obs.original_block_id, "top-1");
}

#[test]
fn test_insights_data_creation() {
    let data = routes::insights::InsightsData {
        blocks: vec![],
        metrics: routes::insights::AnalyticsMetrics {
            total_observations: 0,
            scheduled_count: 0,
            unscheduled_count: 0,
            scheduling_rate: 0.0,
            mean_priority: 0.0,
            median_priority: 0.0,
            mean_priority_scheduled: 0.0,
            mean_priority_unscheduled: 0.0,
            total_visibility_hours: qtty::Hours::new(0.0),
            mean_requested_hours: qtty::Hours::new(0.0),
        },
        correlations: vec![],
        top_priority: vec![],
        top_visibility: vec![],
        conflicts: vec![],
        total_count: 0,
        scheduled_count: 0,
        impossible_count: 0,
    };
    assert_eq!(data.total_count, 0);
}

// Landing module tests
#[test]
fn test_schedule_info_creation() {
    let info = routes::landing::ScheduleInfo {
        schedule_id: ScheduleId::new(1),
        schedule_name: "test".to_string(),
    };
    assert_eq!(info.schedule_id.value(), 1);
    assert_eq!(info.schedule_name, "test");
}

// Skymap module tests
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
fn test_lightweight_block_creation() {
    let block = routes::skymap::LightweightBlock {
        original_block_id: "light-1".to_string(),
        priority: 6.0,
        priority_bin: "Medium".to_string(),
        requested_duration_seconds: qtty::Seconds::new(3600.0),
        target_ra_deg: qtty::Degrees::new(180.0),
        target_dec_deg: qtty::Degrees::new(45.0),
        scheduled_period: None,
    };
    assert_eq!(block.priority, 6.0);
    assert_eq!(block.target_ra_deg.value(), 180.0);
}

#[test]
fn test_sky_map_data_creation() {
    let data = routes::skymap::SkyMapData {
        blocks: vec![],
        priority_bins: vec![],
        priority_min: 0.0,
        priority_max: 10.0,
        ra_min: qtty::Degrees::new(0.0),
        ra_max: qtty::Degrees::new(360.0),
        dec_min: qtty::Degrees::new(-90.0),
        dec_max: qtty::Degrees::new(90.0),
        total_count: 0,
        scheduled_count: 0,
        scheduled_time_min: None,
        scheduled_time_max: None,
    };
    assert_eq!(data.priority_min, 0.0);
    assert_eq!(data.total_count, 0);
}

// Timeline module tests
#[test]
fn test_schedule_timeline_block_creation() {
    let block = routes::timeline::ScheduleTimelineBlock {
        scheduling_block_id: 10,
        original_block_id: "timeline-1".to_string(),
        priority: 7.5,
        scheduled_start_mjd: tsi_rust::api::ModifiedJulianDate::new(59100.0),
        scheduled_stop_mjd: tsi_rust::api::ModifiedJulianDate::new(59101.0),
        ra_deg: qtty::Degrees::new(90.0),
        dec_deg: qtty::Degrees::new(30.0),
        requested_hours: qtty::Hours::new(3.0),
        total_visibility_hours: qtty::Hours::new(12.0),
        num_visibility_periods: 5,
    };
    assert_eq!(block.scheduling_block_id, 10);
    assert_eq!(block.priority, 7.5);
    assert_eq!(block.num_visibility_periods, 5);
}

#[test]
fn test_schedule_timeline_data_creation() {
    let data = routes::timeline::ScheduleTimelineData {
        blocks: vec![],
        priority_min: 0.0,
        priority_max: 10.0,
        total_count: 0,
        scheduled_count: 0,
        unique_months: vec![],
        dark_periods: vec![],
    };
    assert_eq!(data.total_count, 0);
    assert_eq!(data.scheduled_count, 0);
}

// Trends module tests
#[test]
fn test_trends_block_creation() {
    let block = routes::trends::TrendsBlock {
        scheduling_block_id: 5,
        original_block_id: "trend-1".to_string(),
        priority: 4.5,
        total_visibility_hours: qtty::Hours::new(18.0),
        requested_hours: qtty::Hours::new(2.0),
        scheduled: false,
    };
    assert_eq!(block.scheduling_block_id, 5);
    assert!(!block.scheduled);
}

#[test]
fn test_empirical_rate_point_creation() {
    let point = routes::trends::EmpiricalRatePoint {
        bin_label: "5.0-7.5".to_string(),
        mid_value: 6.25,
        scheduled_rate: 0.75,
        count: 20,
    };
    assert_eq!(point.bin_label, "5.0-7.5");
    assert_eq!(point.scheduled_rate, 0.75);
}

#[test]
fn test_smoothed_point_creation() {
    let point = routes::trends::SmoothedPoint {
        x: 5.0,
        y_smoothed: 0.8,
        n_samples: 15,
    };
    assert_eq!(point.x, 5.0);
    assert_eq!(point.y_smoothed, 0.8);
}

#[test]
fn test_heatmap_bin_creation() {
    let bin = routes::trends::HeatmapBin {
        visibility_mid: 10.0,
        time_mid: 2.5,
        scheduled_rate: 0.65,
        count: 30,
    };
    assert_eq!(bin.visibility_mid, 10.0);
    assert_eq!(bin.count, 30);
}

#[test]
fn test_trends_metrics_creation() {
    let metrics = routes::trends::TrendsMetrics {
        total_count: 200,
        scheduled_count: 120,
        scheduling_rate: 0.6,
        zero_visibility_count: 10,
        priority_min: 0.0,
        priority_max: 10.0,
        priority_mean: 5.5,
        visibility_min: 0.0,
        visibility_max: 100.0,
        visibility_mean: 25.0,
        time_min: 0.0,
        time_max: 10.0,
        time_mean: 5.0,
    };
    assert_eq!(metrics.total_count, 200);
    assert_eq!(metrics.scheduling_rate, 0.6);
}

#[test]
fn test_trends_data_creation() {
    let data = routes::trends::TrendsData {
        blocks: vec![],
        metrics: routes::trends::TrendsMetrics {
            total_count: 0,
            scheduled_count: 0,
            scheduling_rate: 0.0,
            zero_visibility_count: 0,
            priority_min: 0.0,
            priority_max: 0.0,
            priority_mean: 0.0,
            visibility_min: 0.0,
            visibility_max: 0.0,
            visibility_mean: 0.0,
            time_min: 0.0,
            time_max: 0.0,
            time_mean: 0.0,
        },
        by_priority: vec![],
        by_visibility: vec![],
        by_time: vec![],
        smoothed_visibility: vec![],
        smoothed_time: vec![],
        heatmap_bins: vec![],
        priority_values: vec![],
    };
    assert_eq!(data.metrics.total_count, 0);
}

// Validation module tests
#[test]
fn test_validation_issue_creation() {
    let issue = routes::validation::ValidationIssue {
        block_id: 99,
        original_block_id: Some("val-1".to_string()),
        issue_type: "invalid_coordinates".to_string(),
        category: "error".to_string(),
        criticality: "high".to_string(),
        field_name: Some("ra".to_string()),
        current_value: Some("400.0".to_string()),
        expected_value: Some("0-360".to_string()),
        description: "RA out of range".to_string(),
    };
    assert_eq!(issue.block_id, 99);
    assert_eq!(issue.issue_type, "invalid_coordinates");
}

#[test]
fn test_validation_report_creation() {
    let report = routes::validation::ValidationReport {
        schedule_id: ScheduleId::new(1),
        total_blocks: 100,
        valid_blocks: 95,
        impossible_blocks: vec![],
        validation_errors: vec![],
        validation_warnings: vec![],
    };
    assert_eq!(report.total_blocks, 100);
    assert_eq!(report.valid_blocks, 95);
}

// Visibility module tests
#[test]
fn test_visibility_block_summary_creation() {
    let summary = routes::visibility::VisibilityBlockSummary {
        scheduling_block_id: 25,
        original_block_id: "vis-1".to_string(),
        priority: 8.5,
        num_visibility_periods: 10,
        scheduled: true,
    };
    assert_eq!(summary.scheduling_block_id, 25);
    assert_eq!(summary.priority, 8.5);
    assert!(summary.scheduled);
}

#[test]
fn test_visibility_map_data_creation() {
    let data = routes::visibility::VisibilityMapData {
        blocks: vec![],
        priority_min: 0.0,
        priority_max: 10.0,
        total_count: 0,
        scheduled_count: 0,
    };
    assert_eq!(data.total_count, 0);
    assert_eq!(data.priority_min, 0.0);
}

#[test]
fn test_visibility_constants() {
    assert_eq!(routes::visibility::GET_VISIBILITY_MAP_DATA, "get_visibility_map_data");
    assert_eq!(routes::visibility::GET_SCHEDULE_TIME_RANGE, "get_schedule_time_range");
    assert_eq!(routes::visibility::GET_VISIBILITY_HISTOGRAM, "get_visibility_histogram");
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
