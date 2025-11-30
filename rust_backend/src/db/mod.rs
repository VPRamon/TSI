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

pub use checksum::calculate_checksum;
pub use config::{DbAuthMethod, DbConfig};
pub use models::{ScheduleInfo, ScheduleMetadata};
pub use pool::DbPool;
