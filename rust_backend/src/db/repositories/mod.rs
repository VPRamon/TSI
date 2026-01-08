//! Repository implementations module.
//!
//! This module contains different implementations of the `ScheduleRepository` trait:
//! - `azure`: Azure SQL Server implementation for production use
//! - `postgres`: PostgreSQL implementation with Diesel ORM
//! - `local`: In-memory implementation for unit testing and local development

#[cfg(feature = "azure-repo")]
pub mod azure;
pub mod local;
#[cfg(feature = "postgres-repo")]
pub mod postgres;

#[cfg(feature = "azure-repo")]
pub use azure::AzureRepository;
pub use local::LocalRepository;
#[cfg(feature = "postgres-repo")]
pub use postgres::{PoolStats, PostgresConfig, PostgresRepository};
