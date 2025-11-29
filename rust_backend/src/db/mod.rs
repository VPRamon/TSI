//! Database module for Azure SQL Server integration.
//!
//! This module provides connection pooling, health checks, and CRUD operations
//! for schedule data storage in Azure SQL Server database.

pub mod config;
pub mod pool;
pub mod operations;
pub mod models;
pub mod checksum;

pub use config::{DbAuthMethod, DbConfig};
pub use pool::DbPool;
pub use models::{ScheduleMetadata, ScheduleInfo};
pub use checksum::calculate_checksum;
