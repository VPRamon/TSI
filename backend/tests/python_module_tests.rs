//! Tests for Python module bindings and API surface.
//!
//! These tests verify the module structure without requiring Python runtime.

#[test]
fn test_module_exports_exist() {
    // Test that the API module exists and compiles
    // This is a compile-time check
    let _: fn() = || {
        let _ = tsi_rust::api::ScheduleId::new(1);
        let _ = tsi_rust::api::TargetId::new(1);
        let _ = tsi_rust::api::ConstraintsId::new(1);
        let _ = tsi_rust::api::SchedulingBlockId::new(1);
    };
}

#[test]
fn test_route_modules_exist() {
    // Verify route modules are accessible by checking types exist
    let _: fn() = || {
        // Just verify these types exist and can be imported
        let _: Option<tsi_rust::routes::compare::CompareBlock> = None;
        let _: Option<tsi_rust::routes::distribution::DistributionBlock> = None;
        let _: Option<tsi_rust::routes::insights::InsightsBlock> = None;
        let _: Option<tsi_rust::routes::skymap::LightweightBlock> = None;
        let _: Option<tsi_rust::routes::timeline::ScheduleTimelineBlock> = None;
        let _: Option<tsi_rust::routes::trends::TrendsBlock> = None;
        let _: Option<tsi_rust::routes::validation::ValidationIssue> = None;
    };
}

#[test]
fn test_service_modules_exist() {
    // Verify service modules are accessible
    let _: fn() = || {
        use tsi_rust::services;
        let _ = services::compare::compute_compare_data;
        let _ = services::distributions::compute_distribution_data;
        let _ = services::sky_map::compute_sky_map_data;
    };
}

#[test]
fn test_db_modules_exist() {
    // Verify db modules are accessible
    let _: fn() = || {
        use tsi_rust::db;
        let _ = db::calculate_checksum;
        let _ = db::factory::RepositoryFactory::create_local;
    };
}
