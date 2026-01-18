//! HTTP server module for the TSI backend.
//!
//! This module provides an axum-based HTTP server that exposes the TSI backend
//! as a REST API. It reuses the existing service layer, repository pattern,
//! and DTOs from the core library.
//!
//! # Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────┐
//! │  HTTP Layer (axum handlers)                               │
//! │  - Request parsing and validation                         │
//! │  - JSON serialization/deserialization                     │
//! │  - CORS, compression, error handling                      │
//! └───────────────────┬──────────────────────────────────────┘
//!                     │
//! ┌───────────────────▼──────────────────────────────────────┐
//! │  Service Layer (existing services.rs)                     │
//! │  - Business logic                                         │
//! │  - Analytics computation                                  │
//! └───────────────────┬──────────────────────────────────────┘
//!                     │
//! ┌───────────────────▼──────────────────────────────────────┐
//! │  Repository Layer (existing db/)                          │
//! │  - Data persistence                                       │
//! │  - LocalRepository / PostgresRepository                   │
//! └──────────────────────────────────────────────────────────┘
//! ```

#[cfg(feature = "http-server")]
pub mod handlers;

#[cfg(feature = "http-server")]
pub mod router;

#[cfg(feature = "http-server")]
pub mod state;

#[cfg(feature = "http-server")]
pub mod error;

#[cfg(feature = "http-server")]
pub mod dto;

#[cfg(feature = "http-server")]
pub use router::create_router;

#[cfg(feature = "http-server")]
pub use state::AppState;
