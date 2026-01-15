use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

// =========================================================
// Trends types + route
// =========================================================

/// Block data for trends analysis.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsBlock {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub total_visibility_hours: qtty::Hours,
    pub requested_hours: qtty::Hours,
    pub scheduled: bool,
}

/// Empirical scheduling rate point.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmpiricalRatePoint {
    pub bin_label: String,
    pub mid_value: f64,
    pub scheduled_rate: f64,
    pub count: usize,
}

/// Smoothed trend point.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmoothedPoint {
    pub x: f64,
    pub y_smoothed: f64,
    pub n_samples: usize,
}

/// Heatmap bin data.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeatmapBin {
    pub visibility_mid: f64,
    pub time_mid: f64,
    pub scheduled_rate: f64,
    pub count: usize,
}

/// Trends metrics summary.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsMetrics {
    pub total_count: usize,
    pub scheduled_count: usize,
    pub scheduling_rate: f64,
    pub zero_visibility_count: usize,
    pub priority_min: f64,
    pub priority_max: f64,
    pub priority_mean: f64,
    pub visibility_min: f64,
    pub visibility_max: f64,
    pub visibility_mean: f64,
    pub time_min: f64,
    pub time_max: f64,
    pub time_mean: f64,
}

/// Complete trends dataset.
#[pyclass(module = "tsi_rust_api", get_all)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendsData {
    pub blocks: Vec<TrendsBlock>,
    pub metrics: TrendsMetrics,
    pub by_priority: Vec<EmpiricalRatePoint>,
    pub by_visibility: Vec<EmpiricalRatePoint>,
    pub by_time: Vec<EmpiricalRatePoint>,
    pub smoothed_visibility: Vec<SmoothedPoint>,
    pub smoothed_time: Vec<SmoothedPoint>,
    pub heatmap_bins: Vec<HeatmapBin>,
    pub priority_values: Vec<f64>,
}

/// Route function name constant for trends
pub const GET_TRENDS_DATA: &str = "get_trends_data";

/// Get trends analysis data (wraps service call).
/// Accepts optional parameters from Python and uses sensible defaults.
#[pyfunction]
#[allow(clippy::too_many_arguments)]
pub fn get_trends_data(
    schedule_id: crate::api::ScheduleId,
    n_bins: Option<i64>,
    bandwidth: Option<f64>,
    n_smooth_points: Option<i64>,
) -> PyResult<crate::api::TrendsData> {
    let n_bins = n_bins.unwrap_or(10) as usize;
    let bandwidth = bandwidth.unwrap_or(0.5);
    let n_smooth_points = n_smooth_points.unwrap_or(12) as usize;

    let data =
        crate::services::py_get_trends_data(schedule_id, n_bins, bandwidth, n_smooth_points)?;
    Ok(data)
}

/// Register trends-related functions, classes, and constants.
pub fn register_routes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_trends_data, m)?)?;
    m.add_class::<TrendsBlock>()?;
    m.add_class::<EmpiricalRatePoint>()?;
    m.add_class::<SmoothedPoint>()?;
    m.add_class::<HeatmapBin>()?;
    m.add_class::<TrendsMetrics>()?;
    m.add_class::<TrendsData>()?;
    m.add("GET_TRENDS_DATA", GET_TRENDS_DATA)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trends_block_clone() {
        let block = TrendsBlock {
            scheduling_block_id: 5,
            original_block_id: "trend-1".to_string(),
            priority: 4.5,
            total_visibility_hours: qtty::Hours::new(18.0),
            requested_hours: qtty::Hours::new(2.0),
            scheduled: false,
        };
        let cloned = block.clone();
        assert_eq!(cloned.priority, 4.5);
    }

    #[test]
    fn test_trends_block_debug() {
        let block = TrendsBlock {
            scheduling_block_id: 5,
            original_block_id: "trend-1".to_string(),
            priority: 4.5,
            total_visibility_hours: qtty::Hours::new(18.0),
            requested_hours: qtty::Hours::new(2.0),
            scheduled: false,
        };
        let debug_str = format!("{:?}", block);
        assert!(debug_str.contains("TrendsBlock"));
    }

    #[test]
    fn test_empirical_rate_point_clone() {
        let point = EmpiricalRatePoint {
            bin_label: "5.0-7.5".to_string(),
            mid_value: 6.25,
            scheduled_rate: 0.75,
            count: 20,
        };
        let cloned = point.clone();
        assert_eq!(cloned.scheduled_rate, 0.75);
    }

    #[test]
    fn test_empirical_rate_point_debug() {
        let point = EmpiricalRatePoint {
            bin_label: "5.0-7.5".to_string(),
            mid_value: 6.25,
            scheduled_rate: 0.75,
            count: 20,
        };
        let debug_str = format!("{:?}", point);
        assert!(debug_str.contains("EmpiricalRatePoint"));
    }

    #[test]
    fn test_smoothed_point_debug() {
        let point = SmoothedPoint {
            x: 5.0,
            y_smoothed: 0.8,
            n_samples: 15,
        };
        let debug_str = format!("{:?}", point);
        assert!(debug_str.contains("SmoothedPoint"));
    }

    #[test]
    fn test_heatmap_bin_debug() {
        let bin = HeatmapBin {
            visibility_mid: 10.0,
            time_mid: 2.5,
            scheduled_rate: 0.65,
            count: 30,
        };
        let debug_str = format!("{:?}", bin);
        assert!(debug_str.contains("HeatmapBin"));
    }

    #[test]
    fn test_trends_metrics_debug() {
        let metrics = TrendsMetrics {
            total_count: 200,
            scheduled_count: 120,
            scheduling_rate: 0.6,
            zero_visibility_count: 10,
            priority_min: 0.0,
            priority_max: 10.0,
            priority_mean: 5.5,
            visibility_min: 0.0,
            visibility_max: 100.0,
            visibility_mean: 25.0,
            time_min: 0.0,
            time_max: 10.0,
            time_mean: 5.0,
        };
        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("TrendsMetrics"));
    }

    #[test]
    fn test_trends_data_debug() {
        let data = TrendsData {
            blocks: vec![],
            metrics: TrendsMetrics {
                total_count: 0,
                scheduled_count: 0,
                scheduling_rate: 0.0,
                zero_visibility_count: 0,
                priority_min: 0.0,
                priority_max: 0.0,
                priority_mean: 0.0,
                visibility_min: 0.0,
                visibility_max: 0.0,
                visibility_mean: 0.0,
                time_min: 0.0,
                time_max: 0.0,
                time_mean: 0.0,
            },
            by_priority: vec![],
            by_visibility: vec![],
            by_time: vec![],
            smoothed_visibility: vec![],
            smoothed_time: vec![],
            heatmap_bins: vec![],
            priority_values: vec![],
        };
        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("TrendsData"));
    }

    #[test]
    fn test_const_value() {
        assert_eq!(GET_TRENDS_DATA, "get_trends_data");
    }
}
