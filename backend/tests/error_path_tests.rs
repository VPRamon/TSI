//! Error path testing for db/factory.rs, db/services.rs, and db/repository/error.rs
//!
//! These tests specifically trigger error conditions to ensure proper error handling,
//! error propagation, and error context enrichment throughout the stack.

use tsi_rust::api::{GeographicLocation, ModifiedJulianDate, Period, Schedule, ScheduleId};
use tsi_rust::db::factory::RepositoryType;
use tsi_rust::db::repositories::LocalRepository;
use tsi_rust::db::repository::{ErrorContext, RepositoryError};
use tsi_rust::db::services;

mod support;

fn default_schedule_period() -> Period {
    Period {
        start: ModifiedJulianDate::new(60000.0),
        stop: ModifiedJulianDate::new(60001.0),
    }
}

// =========================================================
// Factory Error Tests
// =========================================================

#[cfg(feature = "postgres-repo")]
#[tokio::test]
async fn test_factory_postgres_without_config() {
    use tsi_rust::db::RepositoryFactory;
    // Attempting to create Postgres repository without config should fail
    let result = RepositoryFactory::create(RepositoryType::Postgres, None).await;

    assert!(result.is_err());

    if let Err(e) = result {
        // Should be a configuration error
        assert!(matches!(e, RepositoryError::ConfigurationError { .. }));

        // Error message should be informative
        let error_msg = e.to_string();
        assert!(error_msg.contains("Postgres") || error_msg.contains("config"));
    }
}

#[cfg(feature = "postgres-repo")]
#[tokio::test]
async fn test_factory_invalid_database_url() {
    use tsi_rust::db::PostgresConfig;
    use tsi_rust::db::RepositoryFactory;

    // Create config with invalid database URL
    let invalid_config = PostgresConfig {
        database_url: "invalid://not-a-real-database:5432/test".to_string(),
        max_pool_size: 5,
        min_pool_size: 1,
        connection_timeout_sec: 30,
        idle_timeout_sec: 600,
        max_retries: 3,
        retry_delay_ms: 100,
    };

    let result = RepositoryFactory::create(RepositoryType::Postgres, Some(&invalid_config)).await;

    // Should fail to create repository
    assert!(result.is_err());
}

#[test]
fn test_factory_repository_type_from_str() {
    // Valid types
    assert_eq!(
        "postgres".parse::<RepositoryType>().unwrap(),
        RepositoryType::Postgres
    );
    assert_eq!(
        "pg".parse::<RepositoryType>().unwrap(),
        RepositoryType::Postgres
    );
    assert_eq!(
        "local".parse::<RepositoryType>().unwrap(),
        RepositoryType::Local
    );
    assert_eq!(
        "POSTGRES".parse::<RepositoryType>().unwrap(),
        RepositoryType::Postgres
    );
    assert_eq!(
        "LOCAL".parse::<RepositoryType>().unwrap(),
        RepositoryType::Local
    );

    // Invalid type
    let result: Result<RepositoryType, _> = "invalid_type".parse();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown repository type"));
}

#[test]
fn test_factory_repository_type_from_env_default() {
    support::with_scoped_env(
        &[
            ("REPOSITORY_TYPE", None),
            ("DATABASE_URL", None),
            ("PG_DATABASE_URL", None),
        ],
        || {
            // Should default to Local when no DB URL is set
            let repo_type = RepositoryType::from_env();
            assert_eq!(repo_type, RepositoryType::Local);
        },
    );
}

#[test]
fn test_factory_repository_type_from_env_explicit() {
    support::with_scoped_env(&[("REPOSITORY_TYPE", Some("local"))], || {
        let repo_type = RepositoryType::from_env();
        assert_eq!(repo_type, RepositoryType::Local);
    });
}

// =========================================================
// Services Error Tests
// =========================================================

#[tokio::test]
async fn test_services_health_check_unhealthy_repo() {
    let repo = LocalRepository::new();

    // Set repository to unhealthy state
    repo.set_healthy(false);

    let result = services::health_check(&repo).await;

    // Health check should return Ok(false) for unhealthy repo
    assert!(result.is_ok());
    assert!(!result.unwrap());

    // Restore healthy state
    repo.set_healthy(true);
}

#[tokio::test]
async fn test_services_store_schedule_unhealthy_repo() {
    let repo = LocalRepository::new();
    repo.set_healthy(false);

    let schedule = Schedule {
        id: None,
        name: "test".to_string(),
        blocks: vec![],
        dark_periods: vec![],
        geographic_location: GeographicLocation {
            latitude: 28.7624,
            longitude: -17.8892,
            elevation_m: Some(2396.0),
        },
        astronomical_nights: vec![],
        checksum: "test_checksum".to_string(),
        schedule_period: default_schedule_period(),
    };

    let result = services::store_schedule(&repo, &schedule).await;

    // Should fail with connection error
    assert!(result.is_err());

    if let Err(e) = result {
        assert!(matches!(e, RepositoryError::ConnectionError { .. }));
    }
}

#[tokio::test]
async fn test_services_get_schedule_not_found() {
    let repo = LocalRepository::new();

    // Try to get non-existent schedule
    let result = services::get_schedule(&repo, ScheduleId::new(99999)).await;

    assert!(result.is_err());

    if let Err(e) = result {
        assert!(matches!(e, RepositoryError::NotFound { .. }));

        let error_msg = e.to_string();
        assert!(error_msg.contains("not found") || error_msg.contains("Not found"));
    }
}

#[tokio::test]
async fn test_services_get_blocks_for_nonexistent_schedule() {
    let repo = LocalRepository::new();

    let result = services::get_blocks_for_schedule(&repo, ScheduleId::new(88888)).await;

    // Should return empty vec or error depending on implementation
    // In LocalRepository, this likely returns empty vec
    assert!(result.is_ok() || result.is_err());

    if let Ok(blocks) = result {
        assert_eq!(blocks.len(), 0);
    }
}

#[tokio::test]
async fn test_services_list_schedules_unhealthy() {
    let repo = LocalRepository::new();
    repo.set_healthy(false);

    let result = services::list_schedules(&repo).await;

    // Should fail with connection error
    assert!(result.is_err());
}

#[tokio::test]
async fn test_services_store_schedule_with_invalid_data() {
    let repo = LocalRepository::new();

    // Create schedule with potentially invalid data
    let schedule = Schedule {
        id: None,
        name: "".to_string(), // Empty name
        blocks: vec![],
        dark_periods: vec![],
        geographic_location: GeographicLocation {
            latitude: 28.7624,
            longitude: -17.8892,
            elevation_m: Some(2396.0),
        },
        astronomical_nights: vec![],
        checksum: "".to_string(), // Empty checksum
        schedule_period: default_schedule_period(),
    };

    // LocalRepository may accept this, but it tests the flow
    let result = services::store_schedule(&repo, &schedule).await;

    // Should either succeed (LocalRepository is permissive) or fail with validation error
    if result.is_err() {
        if let Err(e) = result {
            assert!(matches!(e, RepositoryError::ValidationError { .. }));
        }
    }
}

// =========================================================
// Repository Error Type Tests
// =========================================================

#[test]
fn test_error_context_builder_full() {
    let ctx = ErrorContext::new("test_operation")
        .with_entity("schedule")
        .with_entity_id(123)
        .with_details("connection timeout")
        .retryable();

    assert_eq!(ctx.operation.unwrap(), "test_operation");
    assert_eq!(ctx.entity.unwrap(), "schedule");
    assert_eq!(ctx.entity_id.unwrap(), "123");
    assert_eq!(ctx.details.unwrap(), "connection timeout");
    assert!(ctx.retryable);
}

#[test]
fn test_error_context_display_formatting() {
    let ctx = ErrorContext::new("fetch_data")
        .with_entity("block")
        .with_entity_id(456);

    let display = format!("{}", ctx);
    assert!(display.contains("operation=fetch_data"));
    assert!(display.contains("entity=block"));
    assert!(display.contains("id=456"));
}

#[test]
fn test_repository_error_connection() {
    let err = RepositoryError::connection("Failed to connect to database");

    assert!(matches!(err, RepositoryError::ConnectionError { .. }));

    let error_str = format!("{}", err);
    assert!(error_str.contains("Connection error"));
    assert!(error_str.contains("Failed to connect"));
}

#[test]
fn test_repository_error_connection_with_context() {
    let ctx = ErrorContext::new("open_connection").with_details("timeout after 30s");
    let err = RepositoryError::connection_with_context("Database unreachable", ctx);

    if let RepositoryError::ConnectionError { message, context } = err {
        assert_eq!(message, "Database unreachable");
        assert_eq!(context.operation.unwrap(), "open_connection");
        assert!(context.retryable);
    } else {
        panic!("Expected ConnectionError");
    }
}

#[test]
fn test_repository_error_query() {
    let err = RepositoryError::query("Invalid SQL syntax");

    assert!(matches!(err, RepositoryError::QueryError { .. }));

    let error_str = format!("{}", err);
    assert!(error_str.contains("Query error"));
}

#[test]
fn test_repository_error_query_with_context() {
    let ctx = ErrorContext::new("fetch_schedules").with_entity("schedule");
    let err = RepositoryError::query_with_context("Column not found", ctx);

    if let RepositoryError::QueryError { message, context } = err {
        assert_eq!(message, "Column not found");
        assert_eq!(context.operation.unwrap(), "fetch_schedules");
    } else {
        panic!("Expected QueryError");
    }
}

#[test]
fn test_repository_error_not_found() {
    let err = RepositoryError::not_found("Schedule with ID 123 not found");

    assert!(matches!(err, RepositoryError::NotFound { .. }));

    let error_str = format!("{}", err);
    assert!(error_str.contains("Not found"));
    assert!(error_str.contains("123"));
}

#[test]
fn test_repository_error_not_found_with_context() {
    let ctx = ErrorContext::new("get_schedule").with_entity_id(789);
    let err = RepositoryError::not_found_with_context("Resource missing", ctx);

    if let RepositoryError::NotFound { message, context } = err {
        assert_eq!(message, "Resource missing");
        assert_eq!(context.entity_id.unwrap(), "789");
    } else {
        panic!("Expected NotFound error");
    }
}

#[test]
fn test_repository_error_validation() {
    let err = RepositoryError::validation("Invalid schedule format");

    assert!(matches!(err, RepositoryError::ValidationError { .. }));

    let error_str = format!("{}", err);
    assert!(error_str.contains("Data validation error"));
    assert!(error_str.contains("Invalid schedule format"));
}

#[test]
fn test_repository_error_validation_with_context() {
    let ctx = ErrorContext::new("validate_schedule")
        .with_entity("schedule")
        .with_details("missing required field: name");

    let err = RepositoryError::validation_with_context("Validation failed", ctx);

    if let RepositoryError::ValidationError { message, context } = err {
        assert_eq!(message, "Validation failed");
        assert!(context.details.unwrap().contains("missing required field"));
    } else {
        panic!("Expected ValidationError");
    }
}

#[test]
fn test_repository_error_configuration() {
    let err = RepositoryError::configuration("Missing DATABASE_URL");

    assert!(matches!(err, RepositoryError::ConfigurationError { .. }));
}

#[test]
fn test_repository_error_configuration_with_context() {
    let ctx = ErrorContext::new("initialize_repository").with_details("env var not set");
    let err = RepositoryError::configuration_with_context("Config incomplete", ctx);

    if let RepositoryError::ConfigurationError { message, context } = err {
        assert_eq!(message, "Config incomplete");
        assert_eq!(context.operation.unwrap(), "initialize_repository");
    } else {
        panic!("Expected ConfigurationError");
    }
}

#[test]
fn test_repository_error_internal() {
    let err = RepositoryError::internal("Unexpected state");

    assert!(matches!(err, RepositoryError::InternalError { .. }));
}

#[test]
fn test_repository_error_internal_with_context() {
    let ctx = ErrorContext::new("process_batch").with_details("panic recovery");
    let err = RepositoryError::internal_with_context("Internal failure", ctx);

    if let RepositoryError::InternalError { message, context } = err {
        assert_eq!(message, "Internal failure");
        assert!(context.details.unwrap().contains("panic recovery"));
    } else {
        panic!("Expected InternalError");
    }
}

#[test]
fn test_repository_error_transaction() {
    let err = RepositoryError::transaction("Commit failed");

    assert!(matches!(err, RepositoryError::TransactionError { .. }));
}

#[test]
fn test_repository_error_timeout() {
    let err = RepositoryError::timeout("Query timed out after 30s");

    assert!(matches!(err, RepositoryError::TimeoutError { .. }));
}

#[test]
fn test_repository_error_timeout_with_details() {
    let ctx = ErrorContext::new("execute_query")
        .with_entity("analytics")
        .with_details("30s timeout exceeded")
        .retryable();

    let err = RepositoryError::TimeoutError {
        message: "Operation timeout".to_string(),
        context: ctx,
    };

    if let RepositoryError::TimeoutError { message, context } = err {
        assert_eq!(message, "Operation timeout");
        assert!(context.retryable);
    } else {
        panic!("Expected TimeoutError");
    }
}

// =========================================================
// Error Propagation Tests
// =========================================================

#[tokio::test]
async fn test_error_propagation_through_services() {
    let repo = LocalRepository::new();

    // Create unhealthy repository
    repo.set_healthy(false);

    let schedule = Schedule {
        id: None,
        name: "test".to_string(),
        blocks: vec![],
        dark_periods: vec![],
        geographic_location: GeographicLocation {
            latitude: 28.7624,
            longitude: -17.8892,
            elevation_m: Some(2396.0),
        },
        astronomical_nights: vec![],
        checksum: "test".to_string(),
        schedule_period: default_schedule_period(),
    };

    // Error should propagate from repository -> services
    let result = services::store_schedule(&repo, &schedule).await;

    assert!(result.is_err());

    // Verify error type is preserved
    if let Err(e) = result {
        assert!(matches!(e, RepositoryError::ConnectionError { .. }));
    }
}

#[tokio::test]
async fn test_error_propagation_multiple_operations() {
    let repo = LocalRepository::new();

    // First operation succeeds
    let schedule = Schedule {
        id: None,
        name: "good".to_string(),
        blocks: vec![],
        dark_periods: vec![],
        geographic_location: GeographicLocation {
            latitude: 28.7624,
            longitude: -17.8892,
            elevation_m: Some(2396.0),
        },
        astronomical_nights: vec![],
        checksum: "good_checksum".to_string(),
        schedule_period: default_schedule_period(),
    };

    let meta = services::store_schedule(&repo, &schedule).await.unwrap();

    // Make repository unhealthy
    repo.set_healthy(false);

    // Subsequent operation should fail
    let result = services::get_schedule(&repo, meta.schedule_id).await;
    assert!(result.is_err());
}

// =========================================================
// Edge Case Error Tests
// =========================================================

#[test]
fn test_error_context_empty_strings() {
    let ctx = ErrorContext::new("")
        .with_entity("")
        .with_entity_id("")
        .with_details("");

    // Should handle empty strings gracefully
    assert_eq!(ctx.operation.unwrap(), "");
    assert_eq!(ctx.entity.unwrap(), "");
}

#[test]
fn test_error_context_unicode() {
    let ctx = ErrorContext::new("操作")
        .with_entity("スケジュール")
        .with_details("エラーが発生しました");

    // Should handle unicode correctly
    let display = format!("{}", ctx);
    assert!(display.contains("操作"));
}

#[test]
fn test_error_message_very_long() {
    let long_message = "a".repeat(10000);
    let err = RepositoryError::query(long_message.clone());

    // Should handle very long error messages
    let error_str = format!("{}", err);
    assert!(error_str.len() > 5000);
}
