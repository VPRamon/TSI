//! # API Module
//!
//! This module serves as the sole entry point for Python (Streamlit) integration.
//! It provides a stable API layer that isolates Python bindings (PyO3) from internal
//! Rust implementations, allowing free evolution of:
//!
//! - Internal models and data structures
//! - Database schemas and repository implementations
//! - qtty library usage and type system
//! - siderust astronomy calculations
//!
//! ## Architecture
//!
//! - [`types`]: Python-facing DTOs with `#[pyclass]` derives (PyO3-compatible primitives only)
//! - [`conversions`]: Type conversion layer between internal models and Python DTOs
//! - [`streamlit`]: `#[pyfunction]` exports wrapping service/database calls
//!
//! ## Design Principles
//!
//! 1. **Isolation**: PyO3 dependencies only in this module
//! 2. **Stability**: API changes are explicit and versioned
//! 3. **Conversion**: All qtty types (MJD, Degrees) → primitives (f64) at boundary
//! 4. **Simplicity**: DTOs mirror what Streamlit actually needs, not internal complexity

pub mod conversions;
pub mod streamlit;
pub mod types;

// Re-export for convenience
pub use streamlit::register_api_functions;
pub use types::*;
