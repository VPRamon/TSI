//! Integration tests for the PostgreSQL repository implementation.
//!
//! These tests verify that the PostgresRepository correctly implements all
//! repository traits and handles edge cases properly.
//!
//! # Running Tests
//!
//! These tests require a running PostgreSQL instance. Set the following
//! environment variable before running:
//!
//! ```bash
//! export DATABASE_URL="postgresql://user:password@localhost:5432/tsi_test"
//! cargo test --features postgres-repo postgres_repository_tests -- --test-threads=1
//! ```
//!
//! Note: Tests should run with `--test-threads=1` to avoid conflicts when
//! using a shared test database.

#![cfg(feature = "postgres-repo")]

use std::sync::Arc;

use tsi_rust::api::{
    Constraints, ModifiedJulianDate, Period, Schedule, ScheduleId, SchedulingBlock,
    SchedulingBlockId,
};
use tsi_rust::db::repositories::postgres::{PostgresConfig, PostgresRepository};
use tsi_rust::db::{
    AnalyticsRepository, ErrorContext, RepositoryError, ScheduleRepository, ValidationRepository,
    VisualizationRepository,
};
use tsi_rust::services::validation::{ValidationResult, ValidationStatus};

/// Helper function to create a test PostgresConfig.
/// Uses DATABASE_URL from environment or skips the test.
fn get_test_config() -> Option<PostgresConfig> {
    match PostgresConfig::from_env() {
        Ok(mut config) => {
            // Use smaller pool for tests
            config.max_pool_size = 5;
            config.min_pool_size = 1;
            config.max_retries = 2;
            config.retry_delay_ms = 50;
            Some(config)
        }
        Err(_) => {
            eprintln!("DATABASE_URL not set, skipping postgres tests");
            None
        }
    }
}

/// Create a test repository, or skip if database is not available.
fn create_test_repo() -> Option<PostgresRepository> {
    let config = get_test_config()?;
    match PostgresRepository::new(config) {
        Ok(repo) => Some(repo),
        Err(e) => {
            eprintln!("Failed to create postgres repo: {}, skipping tests", e);
            None
        }
    }
}

/// Helper to create a test schedule with blocks.
fn create_test_schedule(name: &str, checksum: &str, num_blocks: usize) -> Schedule {
    let dark_periods = vec![Period {
        start: ModifiedJulianDate::new(60000.0),
        stop: ModifiedJulianDate::new(60001.0),
    }];

    let blocks: Vec<SchedulingBlock> = (0..num_blocks)
        .map(|i| {
            let visibility_periods = vec![Period {
                start: ModifiedJulianDate::new(60000.0 + (i as f64 * 0.1)),
                stop: ModifiedJulianDate::new(60000.5 + (i as f64 * 0.1)),
            }];

            let scheduled_period = if i % 2 == 0 {
                Some(Period {
                    start: ModifiedJulianDate::new(60000.1 + (i as f64 * 0.1)),
                    stop: ModifiedJulianDate::new(60000.2 + (i as f64 * 0.1)),
                })
            } else {
                None
            };

            SchedulingBlock {
                id: SchedulingBlockId(i as i64 + 1),
                original_block_id: Some(format!("OB-{}", i + 1)),
                target_ra: (45.0 + i as f64).into(),
                target_dec: (30.0 - i as f64).into(),
                constraints: Constraints {
                    min_alt: 20.0.into(),
                    max_alt: 85.0.into(),
                    min_az: 0.0.into(),
                    max_az: 360.0.into(),
                    fixed_time: None,
                },
                priority: (i as f64 + 1.0) * 0.5,
                min_observation: 300.0.into(),
                requested_duration: 3600.0.into(),
                visibility_periods,
                scheduled_period,
            }
        })
        .collect();

    Schedule {
        id: None,
        name: name.to_string(),
        checksum: checksum.to_string(),
        dark_periods,
        blocks,
    }
}

/// Generate a unique checksum for each test run.
fn unique_checksum(base: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{}_{}", base, timestamp)
}

// ============================================================================
// Health Check Tests
// ============================================================================

#[tokio::test]
async fn test_postgres_health_check() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let result = repo.health_check().await;
    assert!(result.is_ok(), "Health check should succeed");
    assert!(result.unwrap(), "Health check should return true");
}

#[tokio::test]
async fn test_postgres_health_check_detailed() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let (healthy, latency, error) = repo.health_check_detailed().await;
    assert!(healthy, "Should be healthy");
    assert!(latency.is_some(), "Should have latency");
    assert!(error.is_none(), "Should have no error");

    let latency_ms = latency.unwrap();
    assert!(latency_ms < 5000, "Latency should be reasonable (< 5s)");
}

#[tokio::test]
async fn test_postgres_pool_stats() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let stats = repo.get_pool_stats();
    assert!(stats.max_size > 0, "Max size should be positive");
    assert!(
        stats.total_connections <= stats.max_size,
        "Total should not exceed max"
    );
}

// ============================================================================
// Schedule CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_postgres_store_and_retrieve_schedule() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("store_retrieve");
    let schedule = create_test_schedule("Test Schedule", &checksum, 3);

    // Store
    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");
    assert_eq!(metadata.schedule_name, "Test Schedule");

    // Retrieve
    let retrieved = repo
        .get_schedule(metadata.schedule_id)
        .await
        .expect("Should retrieve schedule");

    assert_eq!(retrieved.name, schedule.name);
    assert_eq!(retrieved.checksum, schedule.checksum);
    assert_eq!(retrieved.blocks.len(), 3);
    assert_eq!(retrieved.dark_periods.len(), 1);
}

#[tokio::test]
async fn test_postgres_idempotent_store() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("idempotent");
    let schedule = create_test_schedule("Idempotent Test", &checksum, 2);

    // Store twice with same checksum
    let first = repo
        .store_schedule(&schedule)
        .await
        .expect("First store should succeed");

    let second = repo
        .store_schedule(&schedule)
        .await
        .expect("Second store should succeed");

    // Should return the same schedule ID
    assert_eq!(first.schedule_id, second.schedule_id);
}

#[tokio::test]
async fn test_postgres_list_schedules() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    // Store a few schedules with unique checksums
    for i in 0..3 {
        let checksum = unique_checksum(&format!("list_{}", i));
        let schedule = create_test_schedule(&format!("List Schedule {}", i), &checksum, 1);
        repo.store_schedule(&schedule)
            .await
            .expect("Should store schedule");
    }

    // List should include our schedules
    let schedules = repo.list_schedules().await.expect("Should list schedules");
    assert!(schedules.len() >= 3, "Should have at least 3 schedules");
}

#[tokio::test]
async fn test_postgres_get_schedule_not_found() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let result = repo.get_schedule(ScheduleId(999999999)).await;
    assert!(result.is_err());

    match result.unwrap_err() {
        RepositoryError::NotFound { .. } => {} // Expected
        other => panic!("Expected NotFound error, got: {:?}", other),
    }
}

#[tokio::test]
async fn test_postgres_get_blocks_for_schedule() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("get_blocks");
    let schedule = create_test_schedule("Blocks Test", &checksum, 5);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    let blocks = repo
        .get_blocks_for_schedule(metadata.schedule_id)
        .await
        .expect("Should get blocks");

    assert_eq!(blocks.len(), 5);

    // Verify block data
    for (i, block) in blocks.iter().enumerate() {
        assert_eq!(block.original_block_id, Some(format!("OB-{}", i + 1)));
    }
}

#[tokio::test]
async fn test_postgres_get_scheduling_block() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("get_single_block");
    let schedule = create_test_schedule("Single Block Test", &checksum, 3);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    // Get all blocks to find a valid ID
    let blocks = repo
        .get_blocks_for_schedule(metadata.schedule_id)
        .await
        .expect("Should get blocks");

    let first_block_id = blocks[0].id.0;

    // Retrieve single block
    let block = repo
        .get_scheduling_block(first_block_id)
        .await
        .expect("Should get single block");

    assert_eq!(block.id.0, first_block_id);
}

#[tokio::test]
async fn test_postgres_fetch_dark_periods() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("dark_periods");
    let schedule = create_test_schedule("Dark Periods Test", &checksum, 1);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    let dark_periods = repo
        .fetch_dark_periods(metadata.schedule_id)
        .await
        .expect("Should fetch dark periods");

    assert_eq!(dark_periods.len(), 1);
    assert!((dark_periods[0].start.value() - 60000.0).abs() < 0.001);
}

#[tokio::test]
async fn test_postgres_get_schedule_time_range() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("time_range");
    let schedule = create_test_schedule("Time Range Test", &checksum, 1);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    let time_range = repo
        .get_schedule_time_range(metadata.schedule_id)
        .await
        .expect("Should get time range");

    assert!(time_range.is_some());
    let range = time_range.unwrap();
    assert!((range.start.value() - 60000.0).abs() < 0.001);
    assert!((range.stop.value() - 60001.0).abs() < 0.001);
}

// ============================================================================
// Analytics Tests
// ============================================================================

#[tokio::test]
async fn test_postgres_analytics_lifecycle() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("analytics_lifecycle");
    let schedule = create_test_schedule("Analytics Test", &checksum, 5);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    let schedule_id = metadata.schedule_id;

    // Initially no analytics
    assert!(!repo
        .has_analytics_data(schedule_id)
        .await
        .expect("has_analytics_data should work"));

    // Populate analytics
    let rows = repo
        .populate_schedule_analytics(schedule_id)
        .await
        .expect("Should populate analytics");
    assert_eq!(rows, 5, "Should have processed 5 blocks");

    // Validation results should be generated as part of ETL analytics population
    assert!(repo
        .has_validation_results(schedule_id)
        .await
        .expect("has_validation_results should work"));

    let report = repo
        .fetch_validation_results(schedule_id)
        .await
        .expect("Should fetch validation results");
    assert_eq!(report.total_blocks, 5);
    assert_eq!(report.valid_blocks, 5);

    // Now has analytics
    assert!(repo
        .has_analytics_data(schedule_id)
        .await
        .expect("has_analytics_data should work"));

    // Delete analytics
    let deleted = repo
        .delete_schedule_analytics(schedule_id)
        .await
        .expect("Should delete analytics");
    assert!(deleted > 0, "Should have deleted analytics entries");

    // No longer has analytics
    assert!(!repo
        .has_analytics_data(schedule_id)
        .await
        .expect("has_analytics_data should work"));
}

#[tokio::test]
async fn test_postgres_fetch_sky_map_blocks() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("sky_map");
    let schedule = create_test_schedule("Sky Map Test", &checksum, 4);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    repo.populate_schedule_analytics(metadata.schedule_id)
        .await
        .expect("Should populate analytics");

    let blocks = repo
        .fetch_analytics_blocks_for_sky_map(metadata.schedule_id)
        .await
        .expect("Should fetch sky map blocks");

    assert_eq!(blocks.len(), 4);
    for block in &blocks {
        assert!(block.target_ra_deg.value() >= 0.0);
        assert!(block.target_dec_deg.value() >= -90.0);
    }
}

#[tokio::test]
async fn test_postgres_fetch_distribution_blocks() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("distribution");
    let schedule = create_test_schedule("Distribution Test", &checksum, 4);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    repo.populate_schedule_analytics(metadata.schedule_id)
        .await
        .expect("Should populate analytics");

    let blocks = repo
        .fetch_analytics_blocks_for_distribution(metadata.schedule_id)
        .await
        .expect("Should fetch distribution blocks");

    assert_eq!(blocks.len(), 4);
    for block in &blocks {
        assert!(block.priority >= 0.0);
        assert!(block.total_visibility_hours.value() >= 0.0);
    }
}

#[tokio::test]
async fn test_postgres_fetch_insights_blocks() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("insights");
    let schedule = create_test_schedule("Insights Test", &checksum, 4);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    repo.populate_schedule_analytics(metadata.schedule_id)
        .await
        .expect("Should populate analytics");

    let blocks = repo
        .fetch_analytics_blocks_for_insights(metadata.schedule_id)
        .await
        .expect("Should fetch insights blocks");

    assert_eq!(blocks.len(), 4);

    // Check that scheduled blocks have correct data
    let scheduled_blocks: Vec<_> = blocks.iter().filter(|b| b.scheduled).collect();
    let unscheduled_blocks: Vec<_> = blocks.iter().filter(|b| !b.scheduled).collect();

    assert_eq!(scheduled_blocks.len(), 2); // Every other block is scheduled
    assert_eq!(unscheduled_blocks.len(), 2);
}

// ============================================================================
// Validation Tests
// ============================================================================

#[tokio::test]
async fn test_postgres_validation_lifecycle() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("validation");
    let schedule = create_test_schedule("Validation Test", &checksum, 3);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    let schedule_id = metadata.schedule_id;

    // Populate analytics first (validation results reference block analytics)
    repo.populate_schedule_analytics(schedule_id)
        .await
        .expect("Should populate analytics");

    // Get blocks to get valid block IDs
    let blocks = repo
        .get_blocks_for_schedule(schedule_id)
        .await
        .expect("Should get blocks");

    // Initially no validation results
    assert!(!repo
        .has_validation_results(schedule_id)
        .await
        .expect("has_validation_results should work"));

    // Insert validation results
    let results: Vec<ValidationResult> = blocks
        .iter()
        .enumerate()
        .map(|(i, b)| ValidationResult {
            schedule_id,
            scheduling_block_id: b.id.0,
            status: if i == 0 {
                ValidationStatus::Impossible
            } else {
                ValidationStatus::Valid
            },
            issue_type: if i == 0 {
                Some("no_visibility".to_string())
            } else {
                None
            },
            issue_category: None,
            criticality: None,
            field_name: None,
            current_value: None,
            expected_value: None,
            description: if i == 0 {
                Some("Block has no visibility".to_string())
            } else {
                None
            },
        })
        .collect();

    let inserted = repo
        .insert_validation_results(&results)
        .await
        .expect("Should insert validation results");
    assert_eq!(inserted, 3);

    // Now has validation results
    assert!(repo
        .has_validation_results(schedule_id)
        .await
        .expect("has_validation_results should work"));

    // Fetch validation report
    let report = repo
        .fetch_validation_results(schedule_id)
        .await
        .expect("Should fetch validation results");

    assert_eq!(report.total_blocks, 3);
    assert_eq!(report.impossible_blocks.len(), 1);
    assert_eq!(report.valid_blocks, 2);

    // Delete validation results
    let deleted = repo
        .delete_validation_results(schedule_id)
        .await
        .expect("Should delete validation results");
    assert!(deleted > 0);

    // No longer has validation results
    assert!(!repo
        .has_validation_results(schedule_id)
        .await
        .expect("has_validation_results should work"));
}

// ============================================================================
// Visualization Tests
// ============================================================================

#[tokio::test]
async fn test_postgres_fetch_timeline_blocks() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("timeline");
    let schedule = create_test_schedule("Timeline Test", &checksum, 6);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    repo.populate_schedule_analytics(metadata.schedule_id)
        .await
        .expect("Should populate analytics");

    let blocks = repo
        .fetch_schedule_timeline_blocks(metadata.schedule_id)
        .await
        .expect("Should fetch timeline blocks");

    // Only scheduled blocks appear in timeline (every other block)
    assert_eq!(blocks.len(), 3);

    for block in &blocks {
        assert!(block.scheduled_start_mjd.value() < block.scheduled_stop_mjd.value());
    }
}

#[tokio::test]
async fn test_postgres_fetch_compare_blocks() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("compare");
    let schedule = create_test_schedule("Compare Test", &checksum, 4);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    repo.populate_schedule_analytics(metadata.schedule_id)
        .await
        .expect("Should populate analytics");

    let blocks = repo
        .fetch_compare_blocks(metadata.schedule_id)
        .await
        .expect("Should fetch compare blocks");

    assert_eq!(blocks.len(), 4);

    for block in &blocks {
        assert!(block.priority >= 0.0);
        assert!(block.requested_hours.value() > 0.0);
    }
}

#[tokio::test]
async fn test_postgres_fetch_visibility_map_data() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("visibility_map");
    let schedule = create_test_schedule("Visibility Map Test", &checksum, 5);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    repo.populate_schedule_analytics(metadata.schedule_id)
        .await
        .expect("Should populate analytics");

    let data = repo
        .fetch_visibility_map_data(metadata.schedule_id)
        .await
        .expect("Should fetch visibility map data");

    assert_eq!(data.total_count, 5);
    assert!(data.scheduled_count > 0);
    assert!(data.priority_min <= data.priority_max);
    assert_eq!(data.blocks.len(), 5);
}

#[tokio::test]
async fn test_postgres_fetch_histogram_blocks() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("histogram");
    let schedule = create_test_schedule("Histogram Test", &checksum, 10);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    // No analytics needed for histogram - uses raw block data
    let blocks = repo
        .fetch_blocks_for_histogram(metadata.schedule_id, None, None, None)
        .await
        .expect("Should fetch histogram blocks");

    assert_eq!(blocks.len(), 10);

    // Test with filters
    let filtered = repo
        .fetch_blocks_for_histogram(metadata.schedule_id, Some(1), Some(3), None)
        .await
        .expect("Should fetch filtered histogram blocks");

    // Priorities in our test data range from 0.5 to 5.0
    // Filtering 1-3 should give us blocks with priorities 1.0, 1.5, 2.0, 2.5, 3.0
    assert!(filtered.len() < 10, "Filter should reduce results");
}

// ============================================================================
// Concurrent Access Tests
// ============================================================================

#[tokio::test]
async fn test_postgres_concurrent_reads() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    let checksum = unique_checksum("concurrent_reads");
    let schedule = create_test_schedule("Concurrent Test", &checksum, 3);

    let metadata = repo
        .store_schedule(&schedule)
        .await
        .expect("Should store schedule");

    repo.populate_schedule_analytics(metadata.schedule_id)
        .await
        .expect("Should populate analytics");

    let repo = Arc::new(repo);
    let schedule_id = metadata.schedule_id;

    // Spawn multiple concurrent reads
    let mut handles = Vec::new();
    for _ in 0..10 {
        let repo_clone = repo.clone();
        handles.push(tokio::spawn(async move {
            repo_clone.get_schedule(schedule_id).await
        }));
    }

    // All reads should succeed
    for handle in handles {
        let result = handle.await.expect("Task should complete");
        assert!(result.is_ok(), "Read should succeed");
    }
}

#[tokio::test]
async fn test_postgres_retry_on_transient_failure() {
    let Some(repo) = create_test_repo() else {
        return;
    };

    // The retry logic is exercised when connections are temporarily unavailable.
    // We can't easily simulate that, but we can verify the retry stats work.

    let stats_before = repo.get_pool_stats();

    // Perform some operations
    let checksum = unique_checksum("retry_test");
    let schedule = create_test_schedule("Retry Test", &checksum, 1);
    let _ = repo.store_schedule(&schedule).await;

    let stats_after = repo.get_pool_stats();

    assert!(
        stats_after.total_queries > stats_before.total_queries,
        "Query count should increase"
    );
}

// ============================================================================
// Error Context Tests
// ============================================================================

#[test]
fn test_error_context_display() {
    let ctx = ErrorContext::new("test_operation")
        .with_entity("schedule")
        .with_entity_id(123)
        .with_details("some details")
        .retryable();

    let display = format!("{}", ctx);
    assert!(display.contains("operation=test_operation"));
    assert!(display.contains("entity=schedule"));
    assert!(display.contains("id=123"));
    assert!(display.contains("details=some details"));
    assert!(display.contains("retryable=true"));
}

#[test]
fn test_repository_error_is_retryable() {
    let conn_err = RepositoryError::connection("test");
    assert!(conn_err.is_retryable());

    let timeout_err = RepositoryError::timeout("test");
    assert!(timeout_err.is_retryable());

    let not_found_err = RepositoryError::not_found("test");
    assert!(!not_found_err.is_retryable());

    let query_err = RepositoryError::query("test");
    assert!(!query_err.is_retryable());
}

#[test]
fn test_repository_error_with_operation() {
    let err = RepositoryError::query("test error").with_operation("fetch_blocks");

    let ctx = err.context();
    assert_eq!(ctx.operation, Some("fetch_blocks".to_string()));
}
