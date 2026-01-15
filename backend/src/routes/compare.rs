use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

// =========================================================
// Compare types + route
// =========================================================

#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareBlock {
    pub scheduling_block_id: String,
    pub priority: f64,
    pub scheduled: bool,
    pub requested_hours: qtty::Hours,
}

#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareStats {
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub total_priority: f64,
    pub mean_priority: f64,
    pub median_priority: f64,
    pub total_hours: qtty::Hours,
    pub gap_count: Option<i32>,
    pub gap_mean_hours: Option<qtty::Hours>,
    pub gap_median_hours: Option<qtty::Hours>,
}

#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingChange {
    pub scheduling_block_id: String,
    pub priority: f64,
    pub change_type: String,
}

#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareData {
    pub current_blocks: Vec<CompareBlock>,
    pub comparison_blocks: Vec<CompareBlock>,
    pub current_stats: CompareStats,
    pub comparison_stats: CompareStats,
    pub common_ids: Vec<String>,
    pub only_in_current: Vec<String>,
    pub only_in_comparison: Vec<String>,
    pub scheduling_changes: Vec<SchedulingChange>,
    pub current_name: String,
    pub comparison_name: String,
}

pub const GET_COMPARE_DATA: &str = "get_compare_data";

#[pyfunction]
pub fn get_compare_data(
    current_schedule_id: crate::api::ScheduleId,
    comparison_schedule_id: crate::api::ScheduleId,
    current_name: Option<String>,
    comparison_name: Option<String>,
) -> PyResult<crate::api::CompareData> {
    let current_name = current_name.unwrap_or_else(|| "Schedule A".to_string());
    let comparison_name = comparison_name.unwrap_or_else(|| "Schedule B".to_string());

    let data = crate::services::py_get_compare_data(
        current_schedule_id,
        comparison_schedule_id,
        current_name,
        comparison_name,
    )?;
    Ok(data)
}

/// Register compare route functions, classes and constants.
pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_compare_data, m)?)?;
    m.add_class::<CompareBlock>()?;
    m.add_class::<CompareStats>()?;
    m.add_class::<SchedulingChange>()?;
    m.add_class::<CompareData>()?;
    m.add("GET_COMPARE_DATA", GET_COMPARE_DATA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_compare_block_clone() {
        let block = CompareBlock {
            scheduling_block_id: "test-1".to_string(),
            priority: 5.0,
            scheduled: true,
            requested_hours: qtty::Hours::new(2.0),
        };
        let cloned = block.clone();
        assert_eq!(cloned.scheduling_block_id, "test-1");
        assert_eq!(cloned.priority, 5.0);
    }

    #[test]
    fn test_compare_block_debug() {
        let block = CompareBlock {
            scheduling_block_id: "test-1".to_string(),
            priority: 5.0,
            scheduled: true,
            requested_hours: qtty::Hours::new(2.0),
        };
        let debug_str = format!("{:?}", block);
        assert!(debug_str.contains("CompareBlock"));
    }

    #[test]
    fn test_compare_block_serialize() {
        let block = CompareBlock {
            scheduling_block_id: "test-1".to_string(),
            priority: 5.0,
            scheduled: true,
            requested_hours: qtty::Hours::new(2.0),
        };
        let json = serde_json::to_string(&block).unwrap();
        assert!(json.contains("test-1"));
        let deserialized: CompareBlock = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.scheduling_block_id, "test-1");
    }

    #[test]
    fn test_compare_stats_clone() {
        let stats = CompareStats {
            scheduled_count: 10,
            unscheduled_count: 5,
            total_priority: 75.0,
            mean_priority: 5.0,
            median_priority: 4.5,
            total_hours: qtty::Hours::new(20.0),
            gap_count: None,
            gap_mean_hours: None,
            gap_median_hours: None,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.scheduled_count, 10);
    }

    #[test]
    fn test_compare_stats_debug() {
        let stats = CompareStats {
            scheduled_count: 10,
            unscheduled_count: 5,
            total_priority: 75.0,
            mean_priority: 5.0,
            median_priority: 4.5,
            total_hours: qtty::Hours::new(20.0),
            gap_count: None,
            gap_mean_hours: None,
            gap_median_hours: None,
        };
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("CompareStats"));
    }

    #[test]
    fn test_compare_stats_serialize() {
        let stats = CompareStats {
            scheduled_count: 10,
            unscheduled_count: 5,
            total_priority: 75.0,
            mean_priority: 5.0,
            median_priority: 4.5,
            total_hours: qtty::Hours::new(20.0),
            gap_count: None,
            gap_mean_hours: None,
            gap_median_hours: None,
        };
        let json = serde_json::to_string(&stats).unwrap();
        let deserialized: CompareStats = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.scheduled_count, 10);
    }

    #[test]
    fn test_scheduling_change_clone() {
        let change = SchedulingChange {
            scheduling_block_id: "block-1".to_string(),
            priority: 3.0,
            change_type: "newly_scheduled".to_string(),
        };
        let cloned = change.clone();
        assert_eq!(cloned.change_type, "newly_scheduled");
    }

    #[test]
    fn test_scheduling_change_debug() {
        let change = SchedulingChange {
            scheduling_block_id: "block-1".to_string(),
            priority: 3.0,
            change_type: "newly_scheduled".to_string(),
        };
        let debug_str = format!("{:?}", change);
        assert!(debug_str.contains("SchedulingChange"));
    }

    #[test]
    fn test_scheduling_change_serialize() {
        let change = SchedulingChange {
            scheduling_block_id: "block-1".to_string(),
            priority: 3.0,
            change_type: "newly_scheduled".to_string(),
        };
        let json = serde_json::to_string(&change).unwrap();
        let deserialized: SchedulingChange = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.change_type, "newly_scheduled");
    }

    #[test]
    fn test_compare_data_debug() {
        let data = CompareData {
            current_blocks: vec![],
            comparison_blocks: vec![],
            current_stats: CompareStats {
                scheduled_count: 0,
                unscheduled_count: 0,
                total_priority: 0.0,
                mean_priority: 0.0,
                median_priority: 0.0,
                total_hours: qtty::Hours::new(0.0),
                gap_count: None,
                gap_mean_hours: None,
                gap_median_hours: None,
            },
            comparison_stats: CompareStats {
                scheduled_count: 0,
                unscheduled_count: 0,
                total_priority: 0.0,
                mean_priority: 0.0,
                median_priority: 0.0,
                total_hours: qtty::Hours::new(0.0),
                gap_count: None,
                gap_mean_hours: None,
                gap_median_hours: None,
            },
            common_ids: vec![],
            only_in_current: vec![],
            only_in_comparison: vec![],
            scheduling_changes: vec![],
            current_name: "Schedule A".to_string(),
            comparison_name: "Schedule B".to_string(),
        };
        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("CompareData"));
    }

    #[test]
    fn test_compare_data_serialize() {
        let data = CompareData {
            current_blocks: vec![],
            comparison_blocks: vec![],
            current_stats: CompareStats {
                scheduled_count: 0,
                unscheduled_count: 0,
                total_priority: 0.0,
                mean_priority: 0.0,
                median_priority: 0.0,
                total_hours: qtty::Hours::new(0.0),
                gap_count: None,
                gap_mean_hours: None,
                gap_median_hours: None,
            },
            comparison_stats: CompareStats {
                scheduled_count: 0,
                unscheduled_count: 0,
                total_priority: 0.0,
                mean_priority: 0.0,
                median_priority: 0.0,
                total_hours: qtty::Hours::new(0.0),
                gap_count: None,
                gap_mean_hours: None,
                gap_median_hours: None,
            },
            common_ids: vec![],
            only_in_current: vec![],
            only_in_comparison: vec![],
            scheduling_changes: vec![],
            current_name: "Schedule A".to_string(),
            comparison_name: "Schedule B".to_string(),
        };
        let json = serde_json::to_string(&data).unwrap();
        let deserialized: CompareData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.current_name, "Schedule A");
    }

    #[test]
    fn test_const_value() {
        assert_eq!(GET_COMPARE_DATA, "get_compare_data");
    }
}
