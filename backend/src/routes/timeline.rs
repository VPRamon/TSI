use serde::{Deserialize, Serialize};

// =========================================================
// Schedule timeline types
// =========================================================

/// Timeline block data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTimelineBlock {
    pub scheduling_block_id: i64, // Internal DB ID (for internal operations)
    pub original_block_id: String, // Original ID from JSON (shown to user)
    pub priority: f64,
    pub scheduled_start_mjd: crate::api::ModifiedJulianDate,
    pub scheduled_stop_mjd: crate::api::ModifiedJulianDate,
    pub ra_deg: qtty::Degrees,
    pub dec_deg: qtty::Degrees,
    pub requested_hours: qtty::Hours,
    pub total_visibility_hours: qtty::Hours,
    pub num_visibility_periods: usize,
}

/// Schedule timeline dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleTimelineData {
    pub blocks: Vec<ScheduleTimelineBlock>,
    pub priority_min: f64,
    pub priority_max: f64,
    pub total_count: usize,
    pub scheduled_count: usize,
    pub unique_months: Vec<String>,
    pub dark_periods: Vec<crate::api::Period>,
}

/// Route function name constant for schedule timeline
pub const GET_SCHEDULE_TIMELINE_DATA: &str = "get_schedule_timeline_data";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_timeline_block_clone() {
        let block = ScheduleTimelineBlock {
            scheduling_block_id: 10,
            original_block_id: "timeline-1".to_string(),
            priority: 7.5,
            scheduled_start_mjd: crate::api::ModifiedJulianDate::new(59100.0),
            scheduled_stop_mjd: crate::api::ModifiedJulianDate::new(59101.0),
            ra_deg: qtty::Degrees::new(90.0),
            dec_deg: qtty::Degrees::new(30.0),
            requested_hours: qtty::Hours::new(3.0),
            total_visibility_hours: qtty::Hours::new(12.0),
            num_visibility_periods: 5,
        };
        let cloned = block.clone();
        assert_eq!(cloned.priority, 7.5);
    }

    #[test]
    fn test_schedule_timeline_block_debug() {
        let block = ScheduleTimelineBlock {
            scheduling_block_id: 10,
            original_block_id: "timeline-1".to_string(),
            priority: 7.5,
            scheduled_start_mjd: crate::api::ModifiedJulianDate::new(59100.0),
            scheduled_stop_mjd: crate::api::ModifiedJulianDate::new(59101.0),
            ra_deg: qtty::Degrees::new(90.0),
            dec_deg: qtty::Degrees::new(30.0),
            requested_hours: qtty::Hours::new(3.0),
            total_visibility_hours: qtty::Hours::new(12.0),
            num_visibility_periods: 5,
        };
        let debug_str = format!("{:?}", block);
        assert!(debug_str.contains("ScheduleTimelineBlock"));
    }

    #[test]
    fn test_schedule_timeline_data_debug() {
        let data = ScheduleTimelineData {
            blocks: vec![],
            priority_min: 0.0,
            priority_max: 10.0,
            total_count: 0,
            scheduled_count: 0,
            unique_months: vec![],
            dark_periods: vec![],
        };
        let debug_str = format!("{:?}", data);
        assert!(debug_str.contains("ScheduleTimelineData"));
    }

    #[test]
    fn test_const_value() {
        assert_eq!(GET_SCHEDULE_TIMELINE_DATA, "get_schedule_timeline_data");
    }
}
