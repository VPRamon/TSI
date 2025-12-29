//! Database module for schedule data storage.
//!
//! This module provides abstractions for database operations via the Repository pattern,
//! allowing different storage backends to be swapped easily.
//!
//! # Architecture
//!
//! The database module follows a layered architecture:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │  Application Layer (Python bindings, REST API, etc.)    │
//! └───────────────────┬─────────────────────────────────────┘
//!                     │
//! ┌───────────────────▼─────────────────────────────────────┐
//! │  Service Layer (services.rs) - Business Logic           │
//! │  - Checksum validation                                   │
//! │  - Analytics population orchestration                    │
//! │  - Cross-cutting concerns                                │
//! └───────────────────┬─────────────────────────────────────┘
//!                     │
//! ┌───────────────────▼─────────────────────────────────────┐
//! │  Repository Trait (repository.rs) - Abstract Interface  │
//! └───────────────────┬─────────────────────────────────────┘
//!                     │
//!     ┌───────────────┴────────────────┐
//!     │                                 │
//! ┌───▼──────────────┐     ┌──────────▼──────────────┐
//! │ Azure Repository │     │  Local Repository       │
//! │ (SQL queries)    │     │  (in-memory)            │
//! └──────────────────┘     └─────────────────────────┘
//! ```
//!
//! # Repository Pattern
//! The module includes:
//! - `services`: High-level business logic functions (use these in your application!)
//! - `repository`: Trait definition for database operations
//! - `repositories::azure`: Azure SQL Server implementation (with operations, analytics, validation)
//! - `repositories::local`: In-memory implementation for unit testing and local development
//! - `factory`: Factory for creating repository instances
//!
//! # Recommended Usage
//!
//! **For new code, use the service layer:**
//! ```no_run
//! use tsi_rust::db::{services, factory, DbConfig};
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = DbConfig::from_env()?;
//!     let repo = factory::RepositoryFactory::create_azure(&config).await?;
//!     
//!     // Use service layer functions
//!     let schedules = services::list_schedules(repo.as_ref()).await?;
//!     Ok(())
//! }
//! ```
//!
//! # Azure Implementation
//! Azure SQL Server specific code is in `repositories::azure`:
//! - `operations`: Direct database CRUD operations (low-level, Azure-specific)
//! - `analytics`: Analytics ETL operations
//! - `validation`: Validation result storage
//! - `pool`: Connection pooling

#[cfg(all(feature = "azure-repo", feature = "postgres-repo"))]
compile_error!("Enable only one repository backend feature at a time.");
#[cfg(all(feature = "azure-repo", feature = "local-repo"))]
compile_error!("Enable only one repository backend feature at a time.");
#[cfg(all(feature = "postgres-repo", feature = "local-repo"))]
compile_error!("Enable only one repository backend feature at a time.");
#[cfg(not(any(feature = "azure-repo", feature = "postgres-repo", feature = "local-repo")))]
compile_error!("Enable exactly one repository backend feature.");

pub mod checksum;
pub mod config;
pub mod factory;
pub mod repo_config;
pub mod repositories;
pub mod repository;
pub mod services;
// Postgres config is colocated with the repository implementation.
#[cfg(feature = "postgres-repo")]
pub use repositories::postgres::PostgresConfig;
#[cfg(not(feature = "postgres-repo"))]
#[derive(Debug, Clone)]
pub struct PostgresConfig {
    _private: (),
}

// ==================== Service Layer (Recommended for new code) ====================
// Use these high-level functions that work with any repository implementation

pub use services::{
    ensure_analytics, fetch_dark_periods, fetch_possible_periods, get_blocks_for_schedule,
    get_schedule, get_schedule_time_range, get_scheduling_block, has_analytics_data, health_check,
    list_schedules, store_schedule,
};

// ==================== Repository Pattern Exports ====================

pub use checksum::calculate_checksum;
pub use config::{DbAuthMethod, DbConfig};
// `ScheduleMetadata` removed; use `crate::api::ScheduleInfo` for lightweight listings.
pub use repo_config::RepositoryConfig;

// Repository trait and implementations
pub use factory::{RepositoryBuilder, RepositoryFactory, RepositoryType};
#[cfg(feature = "azure-repo")]
pub use repositories::AzureRepository;
pub use repositories::LocalRepository;
#[cfg(feature = "postgres-repo")]
pub use repositories::PostgresRepository;
pub use repository::{
    AnalyticsRepository, FullRepository, RepositoryError, RepositoryResult, ScheduleRepository,
    ValidationRepository, VisualizationRepository,
};

// ==================== Backward Compatibility ====================
// These exports are kept for existing code paths that still depend on
// the Azure-specific modules.

// Re-export Azure module functions and types for compatibility
#[cfg(feature = "azure-repo")]
pub use repositories::azure::{analytics, operations, pool, validation};

// Validation
#[cfg(feature = "azure-repo")]
pub use repositories::azure::validation::{
    delete_validation_results, fetch_validation_results, has_validation_results,
    insert_validation_results,
};

// Database pool type
#[cfg(feature = "azure-repo")]
pub use repositories::azure::pool::DbPool;

use anyhow::Result;
#[cfg(any(feature = "azure-repo", feature = "postgres-repo"))]
use anyhow::Context;
use std::sync::{Arc, OnceLock};
#[cfg(any(feature = "azure-repo", feature = "postgres-repo"))]
use tokio::runtime::Runtime;

/// Global repository instance initialized once per process.
static REPOSITORY: OnceLock<Arc<dyn FullRepository>> = OnceLock::new();

#[cfg(feature = "azure-repo")]
async fn create_selected_repository() -> RepositoryResult<Arc<dyn FullRepository>> {
    let config = DbConfig::from_env().map_err(RepositoryError::ConfigurationError)?;
    let repo = RepositoryFactory::create_azure(&config).await?;
    Ok(repo as Arc<dyn FullRepository>)
}

#[cfg(feature = "postgres-repo")]
async fn create_selected_repository() -> RepositoryResult<Arc<dyn FullRepository>> {
    let config = PostgresConfig::from_env().map_err(RepositoryError::ConfigurationError)?;
    let repo = RepositoryFactory::create_postgres(&config).await?;
    Ok(repo as Arc<dyn FullRepository>)
}

#[cfg(feature = "local-repo")]
fn create_selected_repository() -> RepositoryResult<Arc<dyn FullRepository>> {
    Ok(RepositoryFactory::create_local())
}

/// Initialize the global repository singleton for the selected backend.
#[cfg(any(feature = "azure-repo", feature = "postgres-repo"))]
pub fn init_repository() -> Result<()> {
    if REPOSITORY.get().is_some() {
        return Ok(());
    }

    let runtime = Runtime::new().context("Failed to create async runtime for repository init")?;
    let repo = runtime
        .block_on(create_selected_repository())
        .map_err(|e| anyhow::Error::msg(e.to_string()))?;
    let _ = REPOSITORY.set(repo);
    Ok(())
}

/// Initialize the global repository singleton for the selected backend.
#[cfg(feature = "local-repo")]
pub fn init_repository() -> Result<()> {
    if REPOSITORY.get().is_some() {
        return Ok(());
    }

    let repo = create_selected_repository().map_err(|e| anyhow::Error::msg(e.to_string()))?;
    let _ = REPOSITORY.set(repo);
    Ok(())
}

/// Get a reference to the global repository instance.
pub fn get_repository() -> Result<&'static Arc<dyn FullRepository>> {
    if REPOSITORY.get().is_none() {
        let _ = init_repository();
    }

    REPOSITORY
        .get()
        .context("Database not initialized. Call init_repository() first.")
}
