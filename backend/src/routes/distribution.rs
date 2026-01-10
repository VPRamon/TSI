use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

// =========================================================
// Distribution types + route
// =========================================================

/// Block data for distribution analysis.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionBlock {
    pub priority: f64,
    pub total_visibility_hours: qtty::Hours,
    pub requested_hours: qtty::Hours,
    pub elevation_range_deg: qtty::Degrees,
    pub scheduled: bool,
}

/// Distribution statistics.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionStats {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub std_dev: f64,
    pub min: f64,
    pub max: f64,
    pub sum: f64,
}

/// Complete distribution dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionData {
    pub blocks: Vec<DistributionBlock>,
    pub priority_stats: DistributionStats,
    pub visibility_stats: DistributionStats,
    pub requested_hours_stats: DistributionStats,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub impossible_count: usize,
}

/// Route function name constant for distribution data
pub const GET_DISTRIBUTION_DATA: &str = "get_distribution_data";

/// Get distribution visualization data (wraps service call)
#[pyfunction]
pub fn get_distribution_data(
    schedule_id: crate::api::ScheduleId,
) -> PyResult<crate::api::DistributionData> {
    let data = crate::services::py_get_distribution_data_analytics(schedule_id)?;
    Ok(data)
}

/// Register distribution functions, classes and constants with the Python module.
pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_distribution_data, m)?)?;
    m.add_class::<DistributionBlock>()?;
    m.add_class::<DistributionStats>()?;
    m.add_class::<DistributionData>()?;
    m.add("GET_DISTRIBUTION_DATA", GET_DISTRIBUTION_DATA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_distribution_block_clone() {
        let block = DistributionBlock {
            priority: 7.0,
            total_visibility_hours: qtty::Hours::new(15.0),
            requested_hours: qtty::Hours::new(3.0),
            elevation_range_deg: qtty::Degrees::new(45.0),
            scheduled: false,
        };
        let cloned = block.clone();
        assert_eq!(cloned.priority, 7.0);
    }

    #[test]
    fn test_distribution_block_debug() {
        let block = DistributionBlock {
            priority: 7.0,
            total_visibility_hours: qtty::Hours::new(15.0),
            requested_hours: qtty::Hours::new(3.0),
            elevation_range_deg: qtty::Degrees::new(45.0),
            scheduled: false,
        };
        let debug_str = format!("{:?}", block);
        assert!(debug_str.contains("DistributionBlock"));
    }

    #[test]
    fn test_distribution_stats_clone() {
        let stats = DistributionStats {
            count: 50,
            mean: 5.5,
            median: 5.0,
            std_dev: 1.2,
            min: 2.0,
            max: 10.0,
            sum: 275.0,
        };
        let cloned = stats.clone();
        assert_eq!(cloned.count, 50);
    }

    #[test]
    fn test_distribution_stats_debug() {
        let stats = DistributionStats {
            count: 50,
            mean: 5.5,
            median: 5.0,
            std_dev: 1.2,
            min: 2.0,
            max: 10.0,
            sum: 275.0,
        };
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("DistributionStats"));
    }

    #[test]
    fn test_distribution_data_debug() {
        let data = DistributionData {
            blocks: vec![],
            priority_stats: DistributionStats {
                count: 0,
                mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
                sum: 0.0,
            },
            visibility_stats: DistributionStats {
                count: 0,
                mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
                sum: 0.0,
            },
            requested_hours_stats: DistributionStats {
                count: 0,
                mean: 0.0,
                median: 0.0,
                std_dev: 0.0,
                min: 0.0,
                max: 0.0,
                sum: 0.0,
            },
            total_count: 0,
            scheduled_count: 0,
            unscheduled_count: 0,
            impossible_count: 0,
        };
        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("DistributionData"));
    }

    #[test]
    fn test_const_value() {
        assert_eq!(GET_DISTRIBUTION_DATA, "get_distribution_data");
    }
}
