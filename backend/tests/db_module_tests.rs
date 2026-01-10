//! Tests for database module exports and service layer functions.

use tsi_rust::db;

#[test]
fn test_db_module_exports_checksum_function() {
    // Verify that calculate_checksum is exported from the db module
    let data = "test data";
    let checksum = db::calculate_checksum(data);
    assert!(!checksum.is_empty());
    assert_eq!(checksum.len(), 64); // SHA-256 produces 64 hex characters
}

#[test]
fn test_db_module_has_service_functions() {
    // Verify all service functions are exported
    // These are compile-time checks - if this compiles, the exports work
    let _: fn() = || {
        // Just verify these symbols exist
        let _ = db::health_check::<db::repositories::LocalRepository>;
        let _ = db::list_schedules::<db::repositories::LocalRepository>;
        let _ = db::get_schedule::<db::repositories::LocalRepository>;
        let _ = db::get_schedule_time_range::<db::repositories::LocalRepository>;
        let _ = db::store_schedule::<db::repositories::LocalRepository>;
    };
}

#[test]
fn test_repository_config_can_be_created() {
    // Test that RepositoryConfig type is exported and is accessible
    use tsi_rust::db::RepositoryConfig;

    // RepositoryConfig is an enum that can be created
    let _: Option<RepositoryConfig> = None;
}

#[cfg(feature = "postgres-repo")]
#[test]
fn test_postgres_config_type_is_exported() {
    // Verify PostgresConfig is exported when feature is enabled
    use tsi_rust::db::PostgresConfig;

    // This is a compile-time check
    let _: Option<PostgresConfig> = None;
}

#[cfg(feature = "postgres-repo")]
#[test]
fn test_pool_stats_type_is_exported() {
    // Verify PoolStats is exported when feature is enabled
    use tsi_rust::db::PoolStats;

    // This is a compile-time check
    let _: Option<PoolStats> = None;
}

#[cfg(not(feature = "postgres-repo"))]
#[test]
fn test_postgres_config_fallback_exists() {
    // Verify PostgresConfig fallback type exists when feature is disabled
    use tsi_rust::db::PostgresConfig;

    // This is a compile-time check
    let _: Option<PostgresConfig> = None;
}

#[cfg(not(feature = "postgres-repo"))]
#[test]
fn test_pool_stats_fallback_exists() {
    // Verify PoolStats fallback type exists when feature is disabled
    use tsi_rust::db::PoolStats;

    let stats = PoolStats::default();
    // Just verify it can be created
    let _ = format!("{:?}", stats);
}

#[test]
fn test_checksum_consistency() {
    // Verify checksum is deterministic
    let data = "consistent data";
    let checksum1 = db::calculate_checksum(data);
    let checksum2 = db::calculate_checksum(data);
    assert_eq!(checksum1, checksum2);
}

#[test]
fn test_checksum_different_for_different_data() {
    let data1 = "data one";
    let data2 = "data two";
    let checksum1 = db::calculate_checksum(data1);
    let checksum2 = db::calculate_checksum(data2);
    assert_ne!(checksum1, checksum2);
}
