//! Repository implementations module.
//!
//! This module contains different implementations of the `ScheduleRepository` trait:
//! - `postgres`: PostgreSQL implementation with Diesel ORM
//! - `local`: In-memory implementation for unit testing and local development
pub mod local;
#[cfg(feature = "postgres-repo")]
pub mod postgres;

pub use local::LocalRepository;
#[cfg(feature = "postgres-repo")]
pub use postgres::{PoolStats, PostgresConfig, PostgresRepository};
