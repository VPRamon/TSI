//! Repository implementations module.
//!
//! This module contains different implementations of the `ScheduleRepository` trait:
//! - `azure`: Azure SQL Server implementation for production use
//! - `test`: In-memory implementation for unit testing

pub mod azure;
pub mod test;

pub use azure::AzureRepository;
pub use test::TestRepository;
