//! Parsers for telescope schedule data formats.
//!
//! This module provides parsers for various input formats used in telescope scheduling,
//! including JSON schedules, CSV files, visibility period data, and dark period constraints.
//!
//! # Parsers
//!
//! - [`json_parser`]: Parse JSON-formatted scheduling blocks
//! - [`csv_parser`]: Parse CSV-formatted schedule files
//! - [`dark_periods_parser`]: Parse dark period constraint files
//! - [`visibility`]: Parse visibility period strings and batch data
//!
//! # Example
//!
//! ```no_run
//! use tsi_rust::parsing::json_parser::parse_schedule_json;
//! use std::path::Path;
//!
//! let blocks = parse_schedule_json(Path::new(\"schedule.json\"))
//!     .expect(\"Failed to parse schedule\");
//! ```

pub mod visibility;
pub mod json_parser;
pub mod csv_parser;
pub mod dark_periods_parser;

#[cfg(test)]
mod json_parser_tests;
#[cfg(test)]
mod csv_parser_tests;
#[cfg(test)]
mod dark_periods_parser_tests;

pub use visibility::VisibilityParser;
