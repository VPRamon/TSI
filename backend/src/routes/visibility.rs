use serde::{Deserialize, Serialize};

// =========================================================
// Visibility Map types
// =========================================================

/// Block summary for visibility map.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityBlockSummary {
    pub scheduling_block_id: i64,
    pub original_block_id: String,
    pub priority: f64,
    pub num_visibility_periods: usize,
    pub scheduled: bool,
}

/// Visibility map visualization data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibilityMapData {
    pub blocks: Vec<VisibilityBlockSummary>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub total_count: usize,
    pub scheduled_count: usize,
}

/// Route function name constant for visibility map
pub const GET_VISIBILITY_MAP_DATA: &str = "get_visibility_map_data";
/// Route function name constant for schedule time range
pub const GET_SCHEDULE_TIME_RANGE: &str = "get_schedule_time_range";
/// Route function name constant for visibility histogram
pub const GET_VISIBILITY_HISTOGRAM: &str = "get_visibility_histogram";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visibility_block_summary_clone() {
        let summary = VisibilityBlockSummary {
            scheduling_block_id: 25,
            original_block_id: "vis-1".to_string(),
            priority: 8.5,
            num_visibility_periods: 10,
            scheduled: true,
        };
        let cloned = summary.clone();
        assert_eq!(cloned.priority, 8.5);
    }

    #[test]
    fn test_visibility_block_summary_debug() {
        let summary = VisibilityBlockSummary {
            scheduling_block_id: 25,
            original_block_id: "vis-1".to_string(),
            priority: 8.5,
            num_visibility_periods: 10,
            scheduled: true,
        };
        let debug_str = format!("{:?}", summary);
        assert!(debug_str.contains("VisibilityBlockSummary"));
    }

    #[test]
    fn test_visibility_map_data_clone() {
        let data = VisibilityMapData {
            blocks: vec![],
            priority_min: 0.0,
            priority_max: 10.0,
            total_count: 0,
            scheduled_count: 0,
        };
        let cloned = data.clone();
        assert_eq!(cloned.total_count, 0);
    }

    #[test]
    fn test_visibility_map_data_debug() {
        let data = VisibilityMapData {
            blocks: vec![],
            priority_min: 0.0,
            priority_max: 10.0,
            total_count: 0,
            scheduled_count: 0,
        };
        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("VisibilityMapData"));
    }

    #[test]
    fn test_const_values() {
        assert_eq!(GET_VISIBILITY_MAP_DATA, "get_visibility_map_data");
        assert_eq!(GET_SCHEDULE_TIME_RANGE, "get_schedule_time_range");
        assert_eq!(GET_VISIBILITY_HISTOGRAM, "get_visibility_histogram");
    }
}
