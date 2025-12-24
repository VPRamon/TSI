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
pub mod models;
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
    get_schedule, get_schedule_time_range, get_scheduling_block, has_analytics_data,
    has_summary_analytics, has_visibility_time_bins, health_check, list_schedules, store_schedule,
};

// ==================== Repository Pattern Exports ====================

pub use checksum::calculate_checksum;
pub use config::{DbAuthMethod, DbConfig};
pub use models::{ScheduleInfo, ScheduleMetadata};
pub use repo_config::RepositoryConfig;

// Repository trait and implementations
pub use factory::{RepositoryBuilder, RepositoryFactory, RepositoryType};
pub use repositories::{AzureRepository, LocalRepository};
pub use repository::{
    AnalyticsRepository, FullRepository, RepositoryError, RepositoryResult, ScheduleRepository,
    ValidationRepository, VisualizationRepository,
};

// ==================== Backward Compatibility (Legacy) ====================
// These exports are for backward compatibility with existing code.
// New code should use the service layer functions above.

// Re-export Azure module functions and types for backward compatibility
pub use repositories::azure::{
    analytics, operations, pool, validation, HeatmapBinData, PriorityRate, ScheduleSummary,
    VisibilityBin, VisibilityTimeBin, VisibilityTimeMetadata,
};

// Phase 1: Block-level analytics (backward compatibility)
pub use repositories::azure::analytics::{
    delete_schedule_analytics as delete_schedule_analytics_legacy,
    fetch_analytics_blocks_for_distribution, fetch_analytics_blocks_for_sky_map,
    has_analytics_data as has_analytics_data_legacy,
    populate_schedule_analytics as populate_schedule_analytics_legacy,
};

// Phase 2: Summary analytics (backward compatibility)
pub use repositories::azure::analytics::{
    delete_summary_analytics as delete_summary_analytics_legacy, fetch_heatmap_bins,
    fetch_priority_rates, fetch_schedule_summary, fetch_visibility_bins,
    has_summary_analytics as has_summary_analytics_legacy,
    populate_summary_analytics as populate_summary_analytics_legacy,
};

// Phase 3: Visibility time bins (backward compatibility)
pub use repositories::azure::analytics::{
    delete_visibility_time_bins as delete_visibility_time_bins_legacy,
    fetch_visibility_histogram_from_analytics, fetch_visibility_metadata,
    has_visibility_time_bins as has_visibility_time_bins_legacy,
    populate_visibility_time_bins as populate_visibility_time_bins_legacy,
};

// Validation
pub use repositories::azure::validation::{
    delete_validation_results, fetch_validation_results, has_validation_results,
    insert_validation_results,
};

// Database pool type
pub use repositories::azure::pool::DbPool;
