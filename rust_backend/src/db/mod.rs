//! Database module for schedule data storage.
//!
//! This module provides abstractions for database operations via the Repository pattern,
//! allowing different storage backends to be swapped easily. It also maintains the legacy
//! direct database access functions for backward compatibility.
//!
//! # Repository Pattern
//! The module now includes:
//! - `repository`: Trait definition for database operations
//! - `repositories::azure`: Azure SQL Server implementation
//! - `repositories::test`: In-memory mock for unit testing
//!
//! # Legacy Direct Access
//! The existing database modules remain available for backward compatibility:
//! - `operations`: Direct database CRUD operations
//! - `analytics`: Analytics ETL operations
//! - `validation`: Validation result storage
//! - `pool`: Connection pooling

pub mod analytics;
pub mod checksum;
pub mod config;
pub mod factory;
pub mod models;
pub mod operations;
pub mod pool;
pub mod repository;
pub mod repositories;
pub mod validation;

// Phase 1: Block-level analytics
pub use analytics::{
    delete_schedule_analytics, fetch_analytics_blocks_for_distribution,
    fetch_analytics_blocks_for_sky_map, has_analytics_data, populate_schedule_analytics,
};

// Phase 2: Summary analytics
pub use analytics::{
    delete_summary_analytics, fetch_heatmap_bins, fetch_priority_rates, fetch_schedule_summary,
    fetch_visibility_bins, has_summary_analytics, populate_summary_analytics,
    HeatmapBinData, PriorityRate, ScheduleSummary, VisibilityBin,
};

// Phase 3: Visibility time bins
pub use analytics::{
    delete_visibility_time_bins, fetch_visibility_histogram_from_analytics,
    fetch_visibility_metadata, has_visibility_time_bins, populate_visibility_time_bins,
    VisibilityTimeBin, VisibilityTimeMetadata,
};

// Validation
pub use validation::{
    delete_validation_results, fetch_validation_results, has_validation_results,
    insert_validation_results, ValidationIssue, ValidationReportData,
};

pub use checksum::calculate_checksum;
pub use config::{DbAuthMethod, DbConfig};
pub use models::{ScheduleInfo, ScheduleMetadata};
pub use pool::DbPool;

// Repository pattern exports
pub use repository::{RepositoryError, RepositoryResult, ScheduleRepository};
pub use repositories::{AzureRepository, TestRepository};
pub use factory::{RepositoryBuilder, RepositoryFactory, RepositoryType};
