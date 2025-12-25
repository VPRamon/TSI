//! Azure SQL Server implementation module.
//!
//! This module contains all Azure SQL Server-specific database operations,
//! including connection pooling, CRUD operations, analytics, and validation.

pub mod analytics;
pub mod operations;
pub mod pool;
pub mod repository;
pub mod validation;

// Re-export the main repository implementation
pub use repository::AzureRepository;

// Re-export pool functions for backward compatibility
pub use pool::{build_tiberius_config, get_pool, init_pool};

// Re-export analytics types from api module (now centralized there)
pub use crate::api::types::{
    HeatmapBinData, PriorityRate, ScheduleSummary, VisibilityBinData,
    VisibilityTimeMetadata,
};
