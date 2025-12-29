//! Repository implementations module.
//!
//! This module contains different implementations of the `ScheduleRepository` trait:
//! - `azure`: Azure SQL Server implementation for production use
//! - `local`: In-memory implementation for unit testing and local development

pub mod azure;
pub mod local;
pub mod postgres;

pub use azure::AzureRepository;
pub use local::LocalRepository;
pub use postgres::PostgresRepository;
