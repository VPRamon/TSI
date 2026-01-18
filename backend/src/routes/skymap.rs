use serde::{Deserialize, Serialize};

/// Priority bin information for sky map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityBinInfo {
    pub label: String,
    pub min_priority: f64,
    pub max_priority: f64,
    pub color: String,
}

/// Minimal block data for visualization queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightweightBlock {
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub priority_bin: String,
    pub requested_duration_seconds: qtty::Seconds,
    pub target_ra_deg: qtty::Degrees,
    pub target_dec_deg: qtty::Degrees,
    pub scheduled_period: Option<crate::api::Period>,
}

/// Sky map visualization data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkyMapData {
    pub blocks: Vec<LightweightBlock>,
    pub priority_bins: Vec<crate::api::PriorityBinInfo>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub ra_min: qtty::Degrees,
    pub ra_max: qtty::Degrees,
    pub dec_min: qtty::Degrees,
    pub dec_max: qtty::Degrees,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub scheduled_time_min: Option<f64>,
    pub scheduled_time_max: Option<f64>,
}

/// Route function name constant
pub const GET_SKY_MAP_DATA: &str = "get_sky_map_data";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_bin_info_clone() {
        let bin = PriorityBinInfo {
            label: "High".to_string(),
            min_priority: 7.5,
            max_priority: 10.0,
            color: "#ff0000".to_string(),
        };
        let cloned = bin.clone();
        assert_eq!(cloned.min_priority, 7.5);
    }

    #[test]
    fn test_priority_bin_info_debug() {
        let bin = PriorityBinInfo {
            label: "High".to_string(),
            min_priority: 7.5,
            max_priority: 10.0,
            color: "#ff0000".to_string(),
        };
        let debug_str = format!("{:?}", bin);
        assert!(debug_str.contains("PriorityBinInfo"));
    }

    #[test]
    fn test_lightweight_block_clone() {
        let block = LightweightBlock {
            original_block_id: "light-1".to_string(),
            priority: 6.0,
            priority_bin: "Medium".to_string(),
            requested_duration_seconds: qtty::Seconds::new(3600.0),
            target_ra_deg: qtty::Degrees::new(180.0),
            target_dec_deg: qtty::Degrees::new(45.0),
            scheduled_period: None,
        };
        let cloned = block.clone();
        assert_eq!(cloned.priority, 6.0);
    }

    #[test]
    fn test_lightweight_block_debug() {
        let block = LightweightBlock {
            original_block_id: "light-1".to_string(),
            priority: 6.0,
            priority_bin: "Medium".to_string(),
            requested_duration_seconds: qtty::Seconds::new(3600.0),
            target_ra_deg: qtty::Degrees::new(180.0),
            target_dec_deg: qtty::Degrees::new(45.0),
            scheduled_period: None,
        };
        let debug_str = format!("{:?}", block);
        assert!(debug_str.contains("LightweightBlock"));
    }

    #[test]
    fn test_sky_map_data_debug() {
        let data = SkyMapData {
            blocks: vec![],
            priority_bins: vec![],
            priority_min: 0.0,
            priority_max: 10.0,
            ra_min: qtty::Degrees::new(0.0),
            ra_max: qtty::Degrees::new(360.0),
            dec_min: qtty::Degrees::new(-90.0),
            dec_max: qtty::Degrees::new(90.0),
            total_count: 0,
            scheduled_count: 0,
            scheduled_time_min: None,
            scheduled_time_max: None,
        };
        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("SkyMapData"));
    }

    #[test]
    fn test_const_value() {
        assert_eq!(GET_SKY_MAP_DATA, "get_sky_map_data");
    }
}
