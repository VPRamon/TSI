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
//!     ┌──────────────────────────────────────────────┐
//!     │             Local Repository                  │
//!     │               (in-memory)                     │
//!     └──────────────────────────────────────────────┘
//! ```
//!
//! # Repository Pattern
//! The module includes:
//! - `services`: High-level business logic functions (use these in your application!)
//! - `repository`: Trait definition for database operations
//! - `repositories::postgres`: Postgres implementation with Diesel ORM
//! - `repositories::local`: In-memory implementation for unit testing and local development
//! - `factory`: Factory for creating repository instances
//!
//! # Recommended Usage
//!
//! **For new code, use the service layer:**
//! ```ignore
//! use tsi_rust::db::{services, factory, PostgresConfig, RepositoryType};
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = PostgresConfig::from_env()?;
//!     let repo = factory::RepositoryFactory::create(RepositoryType::Postgres, Some(&config)).await?;
//!     
//!     // Use service layer functions
//!     let schedules = services::list_schedules(repo.as_ref()).await?;
//!     Ok(())
//! }
//! ```
//!
//! # Postgres Implementation
//! PostgreSQL-specific code is in `repositories::postgres`.

// Feature flag priority: postgres > local
// When multiple features are enabled (e.g., --all-features), postgres takes precedence.
#[cfg(not(any(feature = "postgres-repo", feature = "local-repo")))]
compile_error!("Enable at least one repository backend feature.");

pub mod checksum;
pub mod factory;
pub mod models;
pub mod repo_config;
pub mod repositories;
pub mod repository;
pub mod services;

#[cfg(test)]
#[path = "services_tests.rs"]
mod services_tests;
// Postgres config is colocated with the repository implementation.
#[cfg(feature = "postgres-repo")]
pub use repositories::postgres::{PoolStats, PostgresConfig};
#[cfg(not(feature = "postgres-repo"))]
#[derive(Debug, Clone)]
pub struct PostgresConfig {
    _private: (),
}
#[cfg(not(feature = "postgres-repo"))]
#[derive(Debug, Clone, Default)]
pub struct PoolStats {
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
// `ScheduleMetadata` removed; use `crate::api::ScheduleInfo` for lightweight listings.
pub use repo_config::RepositoryConfig;

// Repository trait and implementations
pub use factory::{RepositoryBuilder, RepositoryFactory, RepositoryType};
pub use repositories::LocalRepository;
#[cfg(feature = "postgres-repo")]
pub use repositories::PostgresRepository;
pub use repository::{
    AnalyticsRepository, ErrorContext, FullRepository, RepositoryError, RepositoryResult,
    ScheduleRepository, ValidationRepository, VisualizationRepository,
};

use anyhow::{Context, Result};
use std::sync::{Arc, OnceLock};
#[cfg(feature = "postgres-repo")]
use tokio::runtime::Runtime;

/// Global repository instance initialized once per process.
static REPOSITORY: OnceLock<Arc<dyn FullRepository>> = OnceLock::new();

// Priority: postgres > local (when --all-features is used)
#[cfg(feature = "postgres-repo")]
async fn create_selected_repository() -> RepositoryResult<Arc<dyn FullRepository>> {
    let config = PostgresConfig::from_env().map_err(RepositoryError::ConfigurationError)?;
    let repo = RepositoryFactory::create_postgres(&config).await?;
    Ok(repo as Arc<dyn FullRepository>)
}

#[cfg(all(feature = "local-repo", not(feature = "postgres-repo")))]
fn create_selected_repository() -> RepositoryResult<Arc<dyn FullRepository>> {
    Ok(RepositoryFactory::create_local())
}

/// Initialize the global repository singleton for the selected backend.
#[cfg(feature = "postgres-repo")]
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
#[cfg(all(feature = "local-repo", not(feature = "postgres-repo")))]
pub fn init_repository() -> Result<()> {
    if REPOSITORY.get().is_some() {
        return Ok(());
    }

    let repo = create_selected_repository()?;
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
