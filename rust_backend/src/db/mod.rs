//! Database module for Azure SQL Server integration.
//!
//! This module provides connection pooling, health checks, and CRUD operations
//! for schedule data storage in Azure SQL Server database.

pub mod checksum;
pub mod config;
pub mod models;
pub mod operations;
pub mod pool;

pub use checksum::calculate_checksum;
pub use config::{DbAuthMethod, DbConfig};
pub use models::{ScheduleInfo, ScheduleMetadata};
pub use pool::DbPool;
