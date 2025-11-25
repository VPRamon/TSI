use anyhow::{Context, Result};
use polars::prelude::*;
use std::path::Path;

use crate::core::domain::SchedulingBlock;
use crate::parsing::csv_parser;
use crate::parsing::dark_periods_parser;
use crate::parsing::json_parser;

/// Represents the source type of schedule data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleSourceType {
    Json,
    Csv,
}

/// Result of loading schedule data
#[derive(Debug)]
pub struct ScheduleLoadResult {
    pub dataframe: DataFrame,
    pub source_type: ScheduleSourceType,
    pub num_blocks: usize,
}

impl ScheduleLoadResult {
    pub fn new(dataframe: DataFrame, source_type: ScheduleSourceType) -> Self {
        let num_blocks = dataframe.height();
        Self {
            dataframe,
            source_type,
            num_blocks,
        }
    }
}

/// Unified interface for loading schedule data from JSON or CSV
pub struct ScheduleLoader;

impl ScheduleLoader {
    /// Load schedule data from a file (auto-detects JSON or CSV)
    pub fn load_from_file(path: &Path) -> Result<ScheduleLoadResult> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .context("File has no extension")?;

        match extension.to_lowercase().as_str() {
            "json" => Self::load_from_json(path),
            "csv" => Self::load_from_csv(path),
            _ => anyhow::bail!("Unsupported file format: {}", extension),
        }
    }

    /// Load schedule data from a JSON file
    pub fn load_from_json(json_path: &Path) -> Result<ScheduleLoadResult> {
        let blocks =
            json_parser::parse_schedule_json(json_path).context("Failed to parse JSON file")?;

        let df = csv_parser::blocks_to_dataframe(&blocks)
            .context("Failed to convert blocks to DataFrame")?;

        Ok(ScheduleLoadResult::new(df, ScheduleSourceType::Json))
    }

    /// Load schedule data from a JSON string
    pub fn load_from_json_str(json_str: &str) -> Result<ScheduleLoadResult> {
        let blocks = json_parser::parse_schedule_json_str(json_str)
            .context("Failed to parse JSON string")?;

        let df = csv_parser::blocks_to_dataframe(&blocks)
            .context("Failed to convert blocks to DataFrame")?;

        Ok(ScheduleLoadResult::new(df, ScheduleSourceType::Json))
    }

    /// Load schedule data from a CSV file
    pub fn load_from_csv(csv_path: &Path) -> Result<ScheduleLoadResult> {
        let df = csv_parser::parse_schedule_csv(csv_path).context("Failed to parse CSV file")?;

        Ok(ScheduleLoadResult::new(df, ScheduleSourceType::Csv))
    }

    /// Load schedule data from CSV and convert to SchedulingBlock structures
    pub fn load_blocks_from_csv(csv_path: &Path) -> Result<Vec<SchedulingBlock>> {
        csv_parser::parse_schedule_csv_to_blocks(csv_path)
    }

    /// Load schedule data from JSON and get SchedulingBlock structures
    pub fn load_blocks_from_json(json_path: &Path) -> Result<Vec<SchedulingBlock>> {
        json_parser::parse_schedule_json(json_path)
    }
}

/// Unified interface for loading dark periods data
pub struct DarkPeriodsLoader;

impl DarkPeriodsLoader {
    /// Load dark periods from a JSON file and return a Polars DataFrame
    pub fn load_from_file(path: &Path) -> Result<DataFrame> {
        let periods = dark_periods_parser::parse_dark_periods_file(path)
            .context("Failed to parse dark periods file")?;

        dark_periods_parser::periods_to_dataframe(periods)
            .context("Failed to convert dark periods to DataFrame")
    }

    /// Load dark periods from a JSON string and return a Polars DataFrame
    pub fn load_from_str(json_str: &str) -> Result<DataFrame> {
        let periods = dark_periods_parser::parse_dark_periods_str(json_str)
            .context("Failed to parse dark periods string")?;

        dark_periods_parser::periods_to_dataframe(periods)
            .context("Failed to convert dark periods to DataFrame")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_from_json_str() {
        let json = r#"{
            "SchedulingBlock": [
                {
                    "schedulingBlockId": 1000004990,
                    "priority": 8.5,
                    "scheduled_period": {
                        "durationInSec": 1200.0,
                        "startTime": {
                            "format": "MJD",
                            "scale": "UTC",
                            "value": 61894.19429606479
                        },
                        "stopTime": {
                            "format": "MJD",
                            "scale": "UTC",
                            "value": 61894.20818495378
                        }
                    },
                    "target": {
                        "id_": 10,
                        "name": "T32",
                        "position_": {
                            "coord": {
                                "celestial": {
                                    "raInDeg": 158.03297990185885,
                                    "decInDeg": -68.02521140748772,
                                    "equinox": 2000.0,
                                    "raProperMotionInMarcsecYear": 0.0,
                                    "decProperMotionInMarcsecYear": 0.0
                                }
                            }
                        }
                    },
                    "schedulingBlockConfiguration_": {
                        "constraints_": {
                            "azimuthConstraint_": {
                                "minAzimuthAngleInDeg": 0.0,
                                "maxAzimuthAngleInDeg": 360.0
                            },
                            "elevationConstraint_": {
                                "minElevationAngleInDeg": 60.0,
                                "maxElevationAngleInDeg": 90.0
                            },
                            "timeConstraint_": {
                                "fixedStartTime": [],
                                "fixedStopTime": [],
                                "minObservationTimeInSec": 1200,
                                "requestedDurationSec": 1200
                            }
                        }
                    }
                }
            ]
        }"#;

        let result = ScheduleLoader::load_from_json_str(json).unwrap();
        assert_eq!(result.source_type, ScheduleSourceType::Json);
        assert_eq!(result.num_blocks, 1);
        assert_eq!(result.dataframe.height(), 1);

        // Verify some columns exist
        let col_names = result.dataframe.get_column_names();
        assert!(col_names.iter().any(|s| s.as_str() == "schedulingBlockId"));
        assert!(col_names.iter().any(|s| s.as_str() == "priority"));
        assert!(col_names.iter().any(|s| s.as_str() == "scheduled_flag"));
    }
}
