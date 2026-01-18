pub mod compare;
pub mod distribution;
pub mod insights;
pub mod landing;
pub mod skymap;
pub mod timeline;
pub mod trends;
pub mod validation;
pub mod visibility;

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_structure() {
        // Test that all route module constants are accessible
        assert_eq!(super::compare::GET_COMPARE_DATA, "get_compare_data");
        assert_eq!(
            super::distribution::GET_DISTRIBUTION_DATA,
            "get_distribution_data"
        );
        assert_eq!(super::insights::GET_INSIGHTS_DATA, "get_insights_data");
        assert_eq!(super::skymap::GET_SKY_MAP_DATA, "get_sky_map_data");
        assert_eq!(
            super::timeline::GET_SCHEDULE_TIMELINE_DATA,
            "get_schedule_timeline_data"
        );
        assert_eq!(super::trends::GET_TRENDS_DATA, "get_trends_data");
        assert_eq!(
            super::validation::GET_VALIDATION_REPORT,
            "get_validation_report"
        );
        assert_eq!(super::landing::LIST_SCHEDULES, "list_schedules");
        assert_eq!(super::landing::POST_SCHEDULE, "store_schedule");
        assert_eq!(
            super::visibility::GET_VISIBILITY_MAP_DATA,
            "get_visibility_map_data"
        );
        assert_eq!(
            super::visibility::GET_SCHEDULE_TIME_RANGE,
            "get_schedule_time_range"
        );
        assert_eq!(
            super::visibility::GET_VISIBILITY_HISTOGRAM,
            "get_visibility_histogram"
        );
    }
}
