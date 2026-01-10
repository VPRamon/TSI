//! Tests for db::factory module - repository creation and configuration.

use std::str::FromStr;
use std::sync::Arc;
use tsi_rust::db::factory::{RepositoryFactory, RepositoryType};

#[test]
fn test_repository_type_from_str_postgres() {
    let rt = RepositoryType::from_str("postgres").unwrap();
    assert_eq!(rt, RepositoryType::Postgres);
    
    let rt = RepositoryType::from_str("POSTGRES").unwrap();
    assert_eq!(rt, RepositoryType::Postgres);
    
    let rt = RepositoryType::from_str("pg").unwrap();
    assert_eq!(rt, RepositoryType::Postgres);
}

#[test]
fn test_repository_type_from_str_local() {
    let rt = RepositoryType::from_str("local").unwrap();
    assert_eq!(rt, RepositoryType::Local);
    
    let rt = RepositoryType::from_str("LOCAL").unwrap();
    assert_eq!(rt, RepositoryType::Local);
}

#[test]
fn test_repository_type_from_str_invalid() {
    let result = RepositoryType::from_str("invalid");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown repository type"));
}

#[test]
fn test_repository_type_from_env_default() {
    // Clear relevant environment variables for consistent testing
    std::env::remove_var("REPOSITORY_TYPE");
    std::env::remove_var("DATABASE_URL");
    std::env::remove_var("PG_DATABASE_URL");
    
    let rt = RepositoryType::from_env();
    assert_eq!(rt, RepositoryType::Local);
}

#[test]
fn test_repository_type_from_env_with_database_url() {
    std::env::remove_var("REPOSITORY_TYPE");
    std::env::set_var("DATABASE_URL", "postgres://localhost/test");
    
    let rt = RepositoryType::from_env();
    assert_eq!(rt, RepositoryType::Postgres);
    
    std::env::remove_var("DATABASE_URL");
}

#[test]
fn test_repository_type_from_env_with_pg_database_url() {
    std::env::remove_var("REPOSITORY_TYPE");
    std::env::remove_var("DATABASE_URL");
    std::env::set_var("PG_DATABASE_URL", "postgres://localhost/test");
    
    let rt = RepositoryType::from_env();
    assert_eq!(rt, RepositoryType::Postgres);
    
    std::env::remove_var("PG_DATABASE_URL");
}

#[test]
fn test_repository_type_from_env_explicit() {
    std::env::set_var("REPOSITORY_TYPE", "local");
    
    let rt = RepositoryType::from_env();
    assert_eq!(rt, RepositoryType::Local);
    
    std::env::remove_var("REPOSITORY_TYPE");
}

#[test]
fn test_repository_type_from_env_explicit_postgres() {
    std::env::set_var("REPOSITORY_TYPE", "postgres");
    
    let rt = RepositoryType::from_env();
    assert_eq!(rt, RepositoryType::Postgres);
    
    std::env::remove_var("REPOSITORY_TYPE");
}

#[test]
fn test_repository_type_from_env_invalid_defaults_to_local() {
    std::env::set_var("REPOSITORY_TYPE", "invalid");
    std::env::remove_var("DATABASE_URL");
    std::env::remove_var("PG_DATABASE_URL");
    
    let rt = RepositoryType::from_env();
    assert_eq!(rt, RepositoryType::Local);
    
    std::env::remove_var("REPOSITORY_TYPE");
}

#[test]
fn test_create_local_repository() {
    let repo = RepositoryFactory::create_local();
    // Just verify the repository was created successfully
    let ptr = Arc::as_ptr(&repo) as *const ();
    assert!(!ptr.is_null());
}

#[tokio::test]
async fn test_create_local_via_factory() {
    let result = RepositoryFactory::create(RepositoryType::Local, None).await;
    assert!(result.is_ok());
}

#[cfg(feature = "postgres-repo")]
#[tokio::test]
async fn test_create_postgres_without_config_fails() {
    let result = RepositoryFactory::create(RepositoryType::Postgres, None).await;
    assert!(result.is_err());
    assert!(result.err().unwrap().to_string().contains("requires PostgresConfig"));
}

#[cfg(not(feature = "postgres-repo"))]
#[tokio::test]
async fn test_create_postgres_without_feature_fails() {
    let result = RepositoryFactory::create(RepositoryType::Postgres, None).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("feature not enabled"));
}

#[test]
fn test_repository_type_debug() {
    let rt = RepositoryType::Local;
    let debug_str = format!("{:?}", rt);
    assert!(debug_str.contains("Local"));
}

#[test]
fn test_repository_type_clone() {
    let rt1 = RepositoryType::Postgres;
    let rt2 = rt1.clone();
    assert_eq!(rt1, rt2);
}

#[test]
fn test_repository_type_copy() {
    let rt1 = RepositoryType::Local;
    let rt2 = rt1;
    assert_eq!(rt1, rt2);
}

#[test]
fn test_repository_type_partial_eq() {
    assert_eq!(RepositoryType::Local, RepositoryType::Local);
    assert_eq!(RepositoryType::Postgres, RepositoryType::Postgres);
    assert_ne!(RepositoryType::Local, RepositoryType::Postgres);
}
