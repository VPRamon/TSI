//! Tests for db::repository::error module.

use tsi_rust::db::repository::{ErrorContext, RepositoryError};

#[test]
fn test_error_context_new() {
    let ctx = ErrorContext::new("test_operation");
    assert_eq!(ctx.operation, Some("test_operation".to_string()));
    assert!(ctx.entity.is_none());
    assert!(ctx.entity_id.is_none());
    assert!(ctx.details.is_none());
    assert!(!ctx.retryable);
}

#[test]
fn test_error_context_with_entity() {
    let ctx = ErrorContext::new("op").with_entity("schedule");
    assert_eq!(ctx.entity, Some("schedule".to_string()));
}

#[test]
fn test_error_context_with_entity_id() {
    let ctx = ErrorContext::new("op").with_entity_id(123);
    assert_eq!(ctx.entity_id, Some("123".to_string()));
}

#[test]
fn test_error_context_with_details() {
    let ctx = ErrorContext::new("op").with_details("some details");
    assert_eq!(ctx.details, Some("some details".to_string()));
}

#[test]
fn test_error_context_retryable() {
    let ctx = ErrorContext::new("op").retryable();
    assert!(ctx.retryable);
}

#[test]
fn test_error_context_chaining() {
    let ctx = ErrorContext::new("store_schedule")
        .with_entity("schedule")
        .with_entity_id(42)
        .with_details("timeout occurred")
        .retryable();

    assert_eq!(ctx.operation, Some("store_schedule".to_string()));
    assert_eq!(ctx.entity, Some("schedule".to_string()));
    assert_eq!(ctx.entity_id, Some("42".to_string()));
    assert_eq!(ctx.details, Some("timeout occurred".to_string()));
    assert!(ctx.retryable);
}

#[test]
fn test_error_context_display() {
    let ctx = ErrorContext::new("test_op")
        .with_entity("test_entity")
        .with_entity_id("123");

    let display = format!("{}", ctx);
    assert!(display.contains("operation=test_op"));
    assert!(display.contains("entity=test_entity"));
    assert!(display.contains("id=123"));
}

#[test]
fn test_error_context_display_retryable() {
    let ctx = ErrorContext::new("op").retryable();
    let display = format!("{}", ctx);
    assert!(display.contains("retryable=true"));
}

#[test]
fn test_error_context_display_with_details() {
    let ctx = ErrorContext::new("op").with_details("extra info");
    let display = format!("{}", ctx);
    assert!(display.contains("details=extra info"));
}

#[test]
fn test_error_context_default() {
    let ctx = ErrorContext::default();
    assert!(ctx.operation.is_none());
    assert!(ctx.entity.is_none());
    assert!(ctx.entity_id.is_none());
    assert!(ctx.details.is_none());
    assert!(!ctx.retryable);
}

#[test]
fn test_error_context_clone() {
    let ctx1 = ErrorContext::new("op").with_entity("entity");
    let ctx2 = ctx1.clone();
    assert_eq!(ctx1.operation, ctx2.operation);
    assert_eq!(ctx1.entity, ctx2.entity);
}

#[test]
fn test_error_context_debug() {
    let ctx = ErrorContext::new("test");
    let debug_str = format!("{:?}", ctx);
    assert!(debug_str.contains("ErrorContext"));
}

#[test]
fn test_repository_error_connection() {
    let err = RepositoryError::connection("connection failed");
    assert!(err.to_string().contains("Connection error"));
    assert!(err.to_string().contains("connection failed"));
}

#[test]
fn test_repository_error_connection_with_context() {
    let ctx = ErrorContext::new("connect").with_entity("database");
    let err = RepositoryError::connection_with_context("failed to connect", ctx);
    let err_str = err.to_string();
    assert!(err_str.contains("Connection error"));
    assert!(err_str.contains("failed to connect"));
    assert!(err_str.contains("operation=connect"));
}

#[test]
fn test_repository_error_query() {
    let err = RepositoryError::query("invalid SQL");
    assert!(err.to_string().contains("Query error"));
    assert!(err.to_string().contains("invalid SQL"));
}

#[test]
fn test_repository_error_not_found() {
    let err = RepositoryError::not_found("schedule not found");
    assert!(err.to_string().contains("Not found"));
    assert!(err.to_string().contains("schedule not found"));
}

#[test]
fn test_repository_error_validation() {
    let err = RepositoryError::validation("invalid data");
    assert!(err.to_string().contains("validation error"));
    assert!(err.to_string().contains("invalid data"));
}

#[test]
fn test_repository_error_configuration() {
    let err = RepositoryError::configuration("missing config");
    assert!(err.to_string().contains("Configuration error"));
    assert!(err.to_string().contains("missing config"));
}

#[test]
fn test_repository_error_internal() {
    let err = RepositoryError::internal("unexpected state");
    assert!(err.to_string().contains("Internal error"));
    assert!(err.to_string().contains("unexpected state"));
}

#[test]
fn test_repository_error_transaction() {
    let err = RepositoryError::transaction("commit failed");
    assert!(err.to_string().contains("Transaction error"));
    assert!(err.to_string().contains("commit failed"));
}

#[test]
fn test_repository_error_timeout() {
    let err = RepositoryError::timeout("operation timed out");
    assert!(err.to_string().contains("Timeout error"));
    assert!(err.to_string().contains("operation timed out"));
}

#[test]
fn test_repository_error_is_retryable_connection() {
    let err = RepositoryError::connection("temp failure");
    assert!(err.is_retryable());
}

#[test]
fn test_repository_error_is_retryable_timeout() {
    let err = RepositoryError::timeout("timeout");
    assert!(err.is_retryable());
}

#[test]
fn test_repository_error_is_retryable_not_found() {
    let err = RepositoryError::not_found("missing");
    assert!(!err.is_retryable());
}

#[test]
fn test_repository_error_is_retryable_validation() {
    let err = RepositoryError::validation("invalid");
    assert!(!err.is_retryable());
}

#[test]
fn test_repository_error_with_operation() {
    let err = RepositoryError::query("error").with_operation("fetch_schedule");
    let err_str = err.to_string();
    assert!(err_str.contains("operation=fetch_schedule"));
}

#[test]
fn test_repository_error_debug() {
    let err = RepositoryError::internal("test");
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("InternalError"));
}

#[test]
fn test_repository_result_ok() {
    use tsi_rust::db::repository::RepositoryResult;
    let result: RepositoryResult<i32> = Ok(42);
    assert!(result.is_ok());
    assert_eq!(*result.as_ref().unwrap(), 42);
}

#[test]
fn test_repository_result_err() {
    use tsi_rust::db::repository::RepositoryResult;
    let result: RepositoryResult<i32> = Err(RepositoryError::not_found("test"));
    assert!(result.is_err());
}
