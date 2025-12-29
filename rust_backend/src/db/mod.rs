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

pub mod checksum;
pub mod config;
pub mod factory;
pub mod repo_config;
pub mod repositories;
pub mod repository;
pub mod repository_manager;
pub mod services;

// ==================== Service Layer (Recommended for new code) ====================
// Use these high-level functions that work with any repository implementation

pub use repository_manager::{get_repository, init_repository};

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
pub use repositories::{AzureRepository, LocalRepository};
pub use repository::{
    AnalyticsRepository, FullRepository, RepositoryError, RepositoryResult, ScheduleRepository,
    ValidationRepository, VisualizationRepository,
};

// ==================== Backward Compatibility ====================
// These exports are kept for existing code paths that still depend on
// the Azure-specific modules.

// Re-export Azure module functions and types for compatibility
pub use repositories::azure::{analytics, operations, pool, validation};

// Validation
pub use repositories::azure::validation::{
    delete_validation_results, fetch_validation_results, has_validation_results,
    insert_validation_results,
};

// Database pool type
pub use repositories::azure::pool::DbPool;
