use anyhow::{Context, Result};
use polars::prelude::*;
use std::path::Path;

use crate::core::domain::SchedulingBlock;
use crate::parsing::csv_parser;
use crate::preprocessing::enricher::ScheduleEnricher;
use crate::preprocessing::validator::{ScheduleValidator, ValidationResult};

/// Result of preprocessing operation
pub struct PreprocessResult {
    pub dataframe: DataFrame,
    pub validation: ValidationResult,
    pub total_blocks: usize,
    pub scheduled_blocks: usize,
}

/// Configuration for the preprocessing pipeline
pub struct PreprocessConfig {
    pub validate: bool,
    pub enrich_visibility: bool,
}

impl Default for PreprocessConfig {
    fn default() -> Self {
        Self {
            validate: true,
            enrich_visibility: false,
        }
    }
}

/// Main preprocessing pipeline
pub struct PreprocessPipeline {
    config: PreprocessConfig,
}

impl PreprocessPipeline {
    /// Create a new pipeline with default configuration
    pub fn new() -> Self {
        Self {
            config: PreprocessConfig::default(),
        }
    }
    
    /// Create a pipeline with custom configuration
    pub fn with_config(config: PreprocessConfig) -> Self {
        Self { config }
    }
    
    /// Process a schedule file (JSON or CSV) into a validated DataFrame
    ///
    /// # Arguments
    /// * `schedule_path` - Path to schedule.json or schedule.csv
    /// * `visibility_path` - Optional path to possible_periods.json
    ///
    /// # Returns
    /// PreprocessResult with DataFrame and validation info
    pub fn process(
        &self,
        schedule_path: &Path,
        visibility_path: Option<&Path>,
    ) -> Result<PreprocessResult> {
        // Step 1: Load schedule data
        let mut blocks = self.load_schedule(schedule_path)?;
        
        // Step 2: Enrich with visibility data (if requested)
        if self.config.enrich_visibility {
            if let Some(vis_path) = visibility_path {
                self.enrich_with_visibility(&mut blocks, vis_path)?;
            }
        }
        
        // Step 3: Convert to DataFrame
        let df = csv_parser::blocks_to_dataframe(&blocks)
            .context("Failed to convert blocks to DataFrame")?;
        
        // Step 4: Validate (if requested)
        let validation = if self.config.validate {
            ScheduleValidator::validate_dataframe(&df)
        } else {
            ValidationResult::new()
        };
        
        // Step 5: Collect statistics
        let total_blocks = blocks.len();
        let scheduled_blocks = blocks.iter().filter(|b| b.is_scheduled()).count();
        
        Ok(PreprocessResult {
            dataframe: df,
            validation,
            total_blocks,
            scheduled_blocks,
        })
    }
    
    /// Process from JSON string (useful for testing or API usage)
    pub fn process_json_str(
        &self,
        json_str: &str,
        visibility_json: Option<&str>,
    ) -> Result<PreprocessResult> {
        // Step 1: Parse JSON
        let mut blocks = crate::parsing::json_parser::parse_schedule_json_str(json_str)
            .context("Failed to parse schedule JSON")?;
        
        // Step 2: Enrich with visibility (if provided)
        if self.config.enrich_visibility {
            if let Some(vis_json) = visibility_json {
                let enricher = ScheduleEnricher::with_visibility_str(vis_json)?;
                enricher.enrich_blocks(&mut blocks);
            }
        }
        
        // Step 3: Convert to DataFrame
        let df = csv_parser::blocks_to_dataframe(&blocks)
            .context("Failed to convert blocks to DataFrame")?;
        
        // Step 4: Validate
        let validation = if self.config.validate {
            ScheduleValidator::validate_dataframe(&df)
        } else {
            ValidationResult::new()
        };
        
        // Step 5: Statistics
        let total_blocks = blocks.len();
        let scheduled_blocks = blocks.iter().filter(|b| b.is_scheduled()).count();
        
        Ok(PreprocessResult {
            dataframe: df,
            validation,
            total_blocks,
            scheduled_blocks,
        })
    }
    
    /// Load schedule from file
    fn load_schedule(&self, path: &Path) -> Result<Vec<SchedulingBlock>> {
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .context("File has no extension")?;
        
        match extension.to_lowercase().as_str() {
            "json" => {
                crate::parsing::json_parser::parse_schedule_json(path)
                    .context("Failed to parse JSON file")
            }
            "csv" => {
                crate::parsing::csv_parser::parse_schedule_csv_to_blocks(path)
                    .context("Failed to parse CSV file")
            }
            _ => anyhow::bail!("Unsupported file format: {}", extension),
        }
    }
    
    /// Enrich blocks with visibility data
    fn enrich_with_visibility(
        &self,
        blocks: &mut [SchedulingBlock],
        visibility_path: &Path,
    ) -> Result<()> {
        let enricher = ScheduleEnricher::with_visibility_file(visibility_path)?;
        enricher.enrich_blocks(blocks);
        Ok(())
    }
}

impl Default for PreprocessPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to preprocess a schedule file
pub fn preprocess_schedule(
    schedule_path: &Path,
    visibility_path: Option<&Path>,
    validate: bool,
) -> Result<PreprocessResult> {
    let config = PreprocessConfig {
        validate,
        enrich_visibility: visibility_path.is_some(),
    };
    
    let pipeline = PreprocessPipeline::with_config(config);
    pipeline.process(schedule_path, visibility_path)
}

#[cfg(all(test, not(feature = "extension-module")))]
mod tests {
    use super::*;

    #[test]
    fn test_process_json_str_basic() {
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
                                    "raInDeg": 158.03,
                                    "decInDeg": -68.03,
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

        let pipeline = PreprocessPipeline::new();
        let result = pipeline.process_json_str(json, None).unwrap();
        
        assert_eq!(result.total_blocks, 1);
        assert_eq!(result.scheduled_blocks, 1);
        assert!(result.validation.is_valid);
        assert_eq!(result.dataframe.height(), 1);
    }
    
    #[test]
    fn test_process_with_validation() {
        let json = r#"{
            "SchedulingBlock": [
                {
                    "schedulingBlockId": 1000004990,
                    "priority": 25.0,
                    "scheduled_period": null,
                    "target": {
                        "id_": 10,
                        "name": "T32",
                        "position_": {
                            "coord": {
                                "celestial": {
                                    "raInDeg": 158.03,
                                    "decInDeg": -68.03,
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

        let pipeline = PreprocessPipeline::new();
        let result = pipeline.process_json_str(json, None).unwrap();
        
        // Should have warnings about invalid priority
        assert!(result.validation.warnings.len() > 0);
    }
}
