//! # TSI Rust Backend
//!
//! High-performance telescope scheduling analysis engine.
//!
//! This crate provides a Rust-based backend for the Telescope Scheduling Intelligence (TSI)
//! system, offering efficient parsing, preprocessing, validation, and analysis of astronomical
//! observation schedules. The backend exposes a REST API via Axum for the React frontend.
//!
//! ## Features
//!
//! - **Data Loading**: Parse observation schedules from JSON format
//! - **Preprocessing**: Validate, enrich, and transform scheduling data
//! - **Analysis**: Compute metrics, correlations, and identify scheduling conflicts
//! - **Time Handling**: Modified Julian Date (MJD) conversions and time period management
//! - **Visibility Computation**: Integration with visibility period data
//! - **HTTP API**: RESTful endpoints for frontend integration
//!
//! ## Architecture
//!
//! The crate is organized into several logical modules:
//!
//! - [`api`]: Data Transfer Objects (DTOs) for API responses
//! - [`db`]: Database operations, repository pattern, and persistence layer
//! - [`services`]: High-level business logic and visualization services
//! - [`http`]: Axum-based HTTP server and request handlers
//! - [`routes`]: Route-specific data types and business logic
//!

// Allow large error types - RepositoryError contains rich context for debugging
#![allow(clippy::result_large_err)]
//! ## Performance
//!
//! This Rust backend is designed for high-performance batch processing of large
//! observation schedules. Key optimizations include:
//!
//! - Zero-copy parsing where possible
//! - Efficient JSON-based data processing with serde_json
//! - Parallel batch operations
//! - Minimal allocations in hot paths

pub mod api;

pub mod db;
pub mod models;

pub mod routes;

pub mod services;

#[cfg(feature = "http-server")]
pub mod http;

/// Re-export the exact `qtty` version used internally so downstream adapter
/// crates avoid duplicate-type issues.
pub use qtty;

/// Re-export `siderust` so adapter crates can use coordinate types directly.
pub use siderust;

/// Configure the global Rayon thread pool to avoid oversubscribing CPUs
/// (Rayon defaults to one thread per logical core, which competes with
/// the Tokio runtime + Diesel pool threads inside the same container).
///
/// Caps the pool at `max(1, num_cpus - 1)` and respects the
/// `RAYON_NUM_THREADS` env var if it is already set. Safe to call
/// multiple times — only the first call wins.
pub fn configure_rayon_thread_pool() {
    if std::env::var_os("RAYON_NUM_THREADS").is_some() {
        return;
    }
    let cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2);
    let target = cpus.saturating_sub(1).max(1);
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(target)
        .build_global();
}
