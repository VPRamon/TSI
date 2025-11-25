//! High-level data loading utilities.
//!
//! This module provides convenient loaders that combine parsing logic with
//! domain model construction and DataFrame conversion. These loaders handle
//! format detection, error context, and produce ready-to-use data structures.
//!
//! # Example
//!
//! ```no_run
//! use tsi_rust::io::loaders::ScheduleLoader;
//! use std::path::Path;
//!
//! let result = ScheduleLoader::load_from_file(Path::new(\"schedule.json\"))
//!     .expect(\"Failed to load\");
//! println!(\"Loaded {} blocks\", result.blocks.len());
//! ```

pub mod loaders;

#[cfg(test)]
mod loaders_tests;

pub use loaders::{ScheduleLoader, ScheduleLoadResult, ScheduleSourceType};
