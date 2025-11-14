//! CSV loader using Polars for efficient data loading
//! 
//! This module provides functions to load scheduling blocks from CSV files
//! or in-memory byte arrays. It uses the common parser to eliminate code duplication.

use anyhow::{Context, Result};
use polars::prelude::*;
use std::path::Path;

use crate::models::schedule::SchedulingBlock;
use super::parser::CsvParser;

/// Load scheduling blocks from a CSV file
/// 
/// # Arguments
/// * `path` - Path to the CSV file
/// 
/// # Returns
/// Vector of scheduling blocks parsed from the CSV
/// 
/// # Errors
/// Returns an error if the file cannot be read or parsed
/// 
/// # Example
/// ```no_run
/// use tsi_backend::loaders::load_csv;
/// use std::path::Path;
/// 
/// let blocks = load_csv(Path::new("data/schedule.csv")).unwrap();
/// println!("Loaded {} blocks", blocks.len());
/// ```
pub fn load_csv<P: AsRef<Path>>(path: P) -> Result<Vec<SchedulingBlock>> {
    let df = CsvReader::from_path(path.as_ref())?
        .finish()
        .context("Failed to read CSV file")?;

    CsvParser::parse_dataframe(df)
}

/// Load scheduling blocks from CSV bytes (for file uploads)
/// 
/// # Arguments
/// * `data` - Byte slice containing CSV data
/// 
/// # Returns
/// Vector of scheduling blocks parsed from the CSV
/// 
/// # Errors
/// Returns an error if the data cannot be parsed
/// 
/// # Example
/// ```no_run
/// use tsi_backend::loaders::load_csv_from_bytes;
/// 
/// let csv_data = b"schedulingBlockId,priority,...";
/// let blocks = load_csv_from_bytes(csv_data).unwrap();
/// ```
pub fn load_csv_from_bytes(data: &[u8]) -> Result<Vec<SchedulingBlock>> {
    let cursor = std::io::Cursor::new(data);
    let df = CsvReader::new(cursor)
        .finish()
        .context("Failed to read CSV from bytes")?;

    CsvParser::parse_dataframe(df)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_csv_from_empty_data() {
        let empty_data = b"";
        let result = load_csv_from_bytes(empty_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_csv_missing_columns() {
        let csv_data = b"col1,col2\nval1,val2\n";
        let result = load_csv_from_bytes(csv_data);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Missing required column"));
        }
    }
}
