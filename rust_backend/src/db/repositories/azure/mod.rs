//! Azure SQL Server implementation module.
//!
//! This module contains all Azure SQL Server-specific database operations,
//! including connection pooling, CRUD operations, analytics, and validation.

pub mod analytics;
#[cfg(any())] // Disabled - non-functional stub with compilation errors
pub mod operations;
pub mod pool;
pub mod repository;
pub mod validation;

// Re-export the main repository implementation
pub use repository::AzureRepository;

// Re-export pool functions for backward compatibility
pub use pool::{build_tiberius_config, get_pool, init_pool};
