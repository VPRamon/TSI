//! Database module for Azure SQL Server integration.
//!
//! This module provides connection pooling, health checks, and CRUD operations
//! for schedule data storage in Azure SQL Server database.

pub mod analytics;
pub mod checksum;
pub mod config;
pub mod models;
pub mod operations;
pub mod pool;

pub use analytics::{
    delete_schedule_analytics, fetch_analytics_blocks_for_distribution,
    fetch_analytics_blocks_for_sky_map, has_analytics_data, populate_schedule_analytics,
};
pub use checksum::calculate_checksum;
pub use config::{DbAuthMethod, DbConfig};
pub use models::{ScheduleInfo, ScheduleMetadata};
pub use pool::DbPool;
