use serde::{Deserialize, Serialize};

// =========================================================
// Insights types
// =========================================================

/// Block data for insights analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsBlock {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub total_visibility_hours: qtty::Hours,
    pub requested_hours: qtty::Hours,
    pub elevation_range_deg: qtty::Degrees,
    pub scheduled: bool,
    pub scheduled_start_mjd: Option<crate::api::ModifiedJulianDate>,
    pub scheduled_stop_mjd: Option<crate::api::ModifiedJulianDate>,
}

/// Analytics metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsMetrics {
    pub total_observations: usize,
    pub scheduled_count: usize,
    pub unscheduled_count: usize,
    pub scheduling_rate: f64,
    pub mean_priority: f64,
    pub median_priority: f64,
    pub mean_priority_scheduled: f64,
    pub mean_priority_unscheduled: f64,
    pub total_visibility_hours: qtty::Hours,
    pub mean_requested_hours: qtty::Hours,
}

/// Correlation entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationEntry {
    pub variable1: String,
    pub variable2: String,
    pub correlation: f64,
}

/// Conflict record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub block_id_1: String, // Original ID from JSON
    pub block_id_2: String, // Original ID from JSON
    pub start_time_1: crate::api::ModifiedJulianDate,
    pub stop_time_1: crate::api::ModifiedJulianDate,
    pub start_time_2: crate::api::ModifiedJulianDate,
    pub stop_time_2: crate::api::ModifiedJulianDate,
    pub overlap_hours: qtty::Hours,
}

/// Top observation entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopObservation {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub total_visibility_hours: qtty::Hours,
    pub requested_hours: qtty::Hours,
    pub scheduled: bool,
}

/// Complete insights dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsightsData {
    pub blocks: Vec<InsightsBlock>,
    pub metrics: AnalyticsMetrics,
    pub correlations: Vec<CorrelationEntry>,
    pub top_priority: Vec<TopObservation>,
    pub top_visibility: Vec<TopObservation>,
    pub conflicts: Vec<ConflictRecord>,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub impossible_count: usize,
}

/// Route function name constant for insights
pub const GET_INSIGHTS_DATA: &str = "get_insights_data";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insights_block_clone() {
        let block = InsightsBlock {
            scheduling_block_id: 42,
            original_block_id: "obs-001".to_string(),
            priority: 8.0,
            total_visibility_hours: qtty::Hours::new(20.0),
            requested_hours: qtty::Hours::new(4.0),
            elevation_range_deg: qtty::Degrees::new(60.0),
            scheduled: true,
            scheduled_start_mjd: Some(crate::api::ModifiedJulianDate::new(59000.0)),
            scheduled_stop_mjd: Some(crate::api::ModifiedJulianDate::new(59001.0)),
        };
        let cloned = block.clone();
        assert_eq!(cloned.priority, 8.0);
    }

    #[test]
    fn test_insights_block_debug() {
        let block = InsightsBlock {
            scheduling_block_id: 42,
            original_block_id: "obs-001".to_string(),
            priority: 8.0,
            total_visibility_hours: qtty::Hours::new(20.0),
            requested_hours: qtty::Hours::new(4.0),
            elevation_range_deg: qtty::Degrees::new(60.0),
            scheduled: true,
            scheduled_start_mjd: Some(crate::api::ModifiedJulianDate::new(59000.0)),
            scheduled_stop_mjd: Some(crate::api::ModifiedJulianDate::new(59001.0)),
        };
        let debug_str = format!("{:?}", block);
        assert!(debug_str.contains("InsightsBlock"));
    }

    #[test]
    fn test_analytics_metrics_clone() {
        let metrics = AnalyticsMetrics {
            total_observations: 100,
            scheduled_count: 60,
            unscheduled_count: 40,
            scheduling_rate: 0.6,
            mean_priority: 5.5,
            median_priority: 5.0,
            mean_priority_scheduled: 6.0,
            mean_priority_unscheduled: 4.5,
            total_visibility_hours: qtty::Hours::new(500.0),
            mean_requested_hours: qtty::Hours::new(2.5),
        };
        let cloned = metrics.clone();
        assert_eq!(cloned.total_observations, 100);
    }

    #[test]
    fn test_analytics_metrics_debug() {
        let metrics = AnalyticsMetrics {
            total_observations: 100,
            scheduled_count: 60,
            unscheduled_count: 40,
            scheduling_rate: 0.6,
            mean_priority: 5.5,
            median_priority: 5.0,
            mean_priority_scheduled: 6.0,
            mean_priority_unscheduled: 4.5,
            total_visibility_hours: qtty::Hours::new(500.0),
            mean_requested_hours: qtty::Hours::new(2.5),
        };
        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("AnalyticsMetrics"));
    }

    #[test]
    fn test_correlation_entry_debug() {
        let corr = CorrelationEntry {
            variable1: "priority".to_string(),
            variable2: "scheduled".to_string(),
            correlation: 0.85,
        };
        let debug_str = format!("{:?}", corr);
        assert!(debug_str.contains("CorrelationEntry"));
    }

    #[test]
    fn test_conflict_record_debug() {
        let conflict = ConflictRecord {
            block_id_1: "block-1".to_string(),
            block_id_2: "block-2".to_string(),
            start_time_1: crate::api::ModifiedJulianDate::new(59000.0),
            stop_time_1: crate::api::ModifiedJulianDate::new(59001.0),
            start_time_2: crate::api::ModifiedJulianDate::new(59000.5),
            stop_time_2: crate::api::ModifiedJulianDate::new(59001.5),
            overlap_hours: qtty::Hours::new(12.0),
        };
        let debug_str = format!("{:?}", conflict);
        assert!(debug_str.contains("ConflictRecord"));
    }

    #[test]
    fn test_top_observation_debug() {
        let obs = TopObservation {
            scheduling_block_id: 1,
            original_block_id: "top-1".to_string(),
            priority: 10.0,
            total_visibility_hours: qtty::Hours::new(100.0),
            requested_hours: qtty::Hours::new(5.0),
            scheduled: true,
        };
        let debug_str = format!("{:?}", obs);
        assert!(debug_str.contains("TopObservation"));
    }

    #[test]
    fn test_insights_data_debug() {
        let data = InsightsData {
            blocks: vec![],
            metrics: AnalyticsMetrics {
                total_observations: 0,
                scheduled_count: 0,
                unscheduled_count: 0,
                scheduling_rate: 0.0,
                mean_priority: 0.0,
                median_priority: 0.0,
                mean_priority_scheduled: 0.0,
                mean_priority_unscheduled: 0.0,
                total_visibility_hours: qtty::Hours::new(0.0),
                mean_requested_hours: qtty::Hours::new(0.0),
            },
            correlations: vec![],
            top_priority: vec![],
            top_visibility: vec![],
            conflicts: vec![],
            total_count: 0,
            scheduled_count: 0,
            impossible_count: 0,
        };
        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("InsightsData"));
    }

    #[test]
    fn test_const_value() {
        assert_eq!(GET_INSIGHTS_DATA, "get_insights_data");
    }
}
