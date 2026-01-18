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
