use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::core::domain::{SchedulingBlock, Period};
use crate::time::mjd::mjd_to_epoch;

/// Raw structure for visibility period from JSON
#[derive(Debug, Deserialize)]
struct RawVisibilityPeriod {
    #[serde(rename = "startTime")]
    start_time: TimeValue,
    #[serde(rename = "stopTime")]
    stop_time: TimeValue,
}

#[derive(Debug, Deserialize)]
struct TimeValue {
    value: f64,  // MJD
}

/// Container for visibility JSON file
#[derive(Debug, Deserialize)]
struct VisibilityJson {
    #[serde(rename = "SchedulingBlock")]
    scheduling_blocks: HashMap<String, Vec<RawVisibilityPeriod>>,
}

/// Enricher for adding visibility data to scheduling blocks
pub struct ScheduleEnricher {
    visibility_data: Option<HashMap<String, Vec<Period>>>,
}

impl ScheduleEnricher {
    /// Create a new enricher without visibility data
    pub fn new() -> Self {
        Self {
            visibility_data: None,
        }
    }
    
    /// Create an enricher with visibility data from a file
    pub fn with_visibility_file(path: &Path) -> Result<Self> {
        let visibility_data = Self::load_visibility_file(path)?;
        Ok(Self {
            visibility_data: Some(visibility_data),
        })
    }
    
    /// Create an enricher with visibility data from a JSON string
    pub fn with_visibility_str(json_str: &str) -> Result<Self> {
        let visibility_data = Self::parse_visibility_json(json_str)?;
        Ok(Self {
            visibility_data: Some(visibility_data),
        })
    }
    
    /// Load visibility data from a JSON file
    fn load_visibility_file(path: &Path) -> Result<HashMap<String, Vec<Period>>> {
        let json_content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read visibility file: {}", path.display()))?;
        
        Self::parse_visibility_json(&json_content)
    }
    
    /// Parse visibility JSON into a HashMap
    fn parse_visibility_json(json_str: &str) -> Result<HashMap<String, Vec<Period>>> {
        let visibility_json: VisibilityJson = serde_json::from_str(json_str)
            .context("Failed to parse visibility JSON")?;
        
        let mut result = HashMap::new();
        
        for (sb_id, raw_periods) in visibility_json.scheduling_blocks {
            let periods: Vec<Period> = raw_periods
                .into_iter()
                .map(|raw| {
                    let start = mjd_to_epoch(raw.start_time.value);
                    let stop = mjd_to_epoch(raw.stop_time.value);
                    Period::new(start, stop)
                })
                .collect();
            
            result.insert(sb_id, periods);
        }
        
        Ok(result)
    }
    
    /// Enrich a single scheduling block with visibility data
    pub fn enrich_block(&self, block: &mut SchedulingBlock) {
        if let Some(ref visibility_data) = self.visibility_data {
            if let Some(periods) = visibility_data.get(&block.scheduling_block_id) {
                block.visibility_periods = periods.clone();
            }
        }
    }
    
    /// Enrich multiple scheduling blocks with visibility data
    pub fn enrich_blocks(&self, blocks: &mut [SchedulingBlock]) {
        for block in blocks.iter_mut() {
            self.enrich_block(block);
        }
    }
    
    /// Get the number of blocks with visibility data
    pub fn visibility_block_count(&self) -> usize {
        self.visibility_data
            .as_ref()
            .map(|data| data.len())
            .unwrap_or(0)
    }
}

impl Default for ScheduleEnricher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, not(feature = "extension-module")))]
mod tests {
    use super::*;

    #[test]
    fn test_parse_visibility_json() {
        let json = r#"{
            "SchedulingBlock": {
                "1000004990": [
                    {
                        "startTime": {"value": 61892.19975694455},
                        "stopTime": {"value": 61892.21081296308}
                    },
                    {
                        "startTime": {"value": 61893.19702662015},
                        "stopTime": {"value": 61893.21006319439}
                    }
                ],
                "1000004991": [
                    {
                        "startTime": {"value": 61894.19429606479},
                        "stopTime": {"value": 61894.20932361111}
                    }
                ]
            }
        }"#;

        let enricher = ScheduleEnricher::with_visibility_str(json).unwrap();
        assert_eq!(enricher.visibility_block_count(), 2);
        
        // Check that the data was parsed correctly
        if let Some(ref vis_data) = enricher.visibility_data {
            let periods_1 = vis_data.get("1000004990").unwrap();
            assert_eq!(periods_1.len(), 2);
            
            let periods_2 = vis_data.get("1000004991").unwrap();
            assert_eq!(periods_2.len(), 1);
        }
    }
    
    #[test]
    fn test_enrich_block() {
        use siderust::astro::ModifiedJulianDate;
        use siderust::coordinates::spherical::direction::ICRS;
        use siderust::units::{time::*, angular::Degrees};
        
        let json = r#"{
            "SchedulingBlock": {
                "test-001": [
                    {
                        "startTime": {"value": 59580.0},
                        "stopTime": {"value": 59581.0}
                    }
                ]
            }
        }"#;

        let enricher = ScheduleEnricher::with_visibility_str(json).unwrap();
        
        let mut block = SchedulingBlock {
            scheduling_block_id: "test-001".to_string(),
            priority: 10.0,
            requested_duration: Seconds::new(3600.0),
            min_observation_time: Seconds::new(0.0),
            fixed_time: None,
            coordinates: Some(ICRS::new(Degrees::new(180.0), Degrees::new(45.0))),
            min_azimuth_angle: Some(Degrees::new(0.0)),
            max_azimuth_angle: Some(Degrees::new(360.0)),
            min_elevation_angle: Some(Degrees::new(30.0)),
            max_elevation_angle: Some(Degrees::new(80.0)),
            scheduled_period: None,
            visibility_periods: vec![],
        };
        
        assert_eq!(block.visibility_periods.len(), 0);
        
        enricher.enrich_block(&mut block);
        
        assert_eq!(block.visibility_periods.len(), 1);
        assert!(block.total_visibility_hours().value() > 0.0);
    }
}
