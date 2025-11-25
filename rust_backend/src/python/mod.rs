//! Python bindings for TSI Rust backend functionality.
//!
//! This module exposes Rust functions and types to Python via PyO3, providing
//! high-performance telescope scheduling operations to Python applications.
//!
//! # Modules
//!
//! - [`loaders`]: Data loading functions for JSON and CSV schedules
//! - [`preprocessing`]: Schedule preprocessing and validation
//! - [`algorithms`]: Analytics and optimization algorithms
//! - [`transformations`]: Data transformation utilities
//! - [`time_bindings`]: Time conversion functions (MJD ↔ datetime)
//!
//! # Python API
//!
//! All functions are available in the `tsi_rust` Python module after installation.
//! See individual module documentation for usage examples.

pub mod loaders;
pub mod preprocessing;
pub mod algorithms;
pub mod transformations;
pub mod time_bindings;

pub use loaders::*;
pub use preprocessing::*;
pub use algorithms::*;
pub use transformations::*;
pub use time_bindings::*;
