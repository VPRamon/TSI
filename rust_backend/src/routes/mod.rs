pub mod landing;
pub mod validation;
pub mod skymap;
pub mod visibility;
pub mod distribution;
pub mod timeline;
pub mod insights;
pub mod trends;
pub mod compare;

use pyo3::prelude::*;
use crate::api::types as api;

/// Register all route-specific functions, classes and constants with the Python module.
/// This centralizes ownership of route registrations inside the `routes` module.
pub fn register_route_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
	// Route functions
	m.add_function(wrap_pyfunction!(landing::store_schedule, m)?)?;
	m.add_function(wrap_pyfunction!(landing::list_schedules, m)?)?;
	m.add_function(wrap_pyfunction!(validation::get_validation_report, m)?)?;

	m.add_function(wrap_pyfunction!(skymap::get_sky_map_data, m)?)?;
	m.add_function(wrap_pyfunction!(distribution::get_distribution_data, m)?)?;
	m.add_function(wrap_pyfunction!(timeline::get_schedule_timeline_data, m)?)?;
	m.add_function(wrap_pyfunction!(insights::get_insights_data, m)?)?;
	m.add_function(wrap_pyfunction!(trends::get_trends_data, m)?)?;
	m.add_function(wrap_pyfunction!(compare::get_compare_data, m)?)?;
	m.add_function(wrap_pyfunction!(visibility::get_visibility_map_data, m)?)?;

	// Route-related classes (types defined in routes and re-exported via api::types)
	m.add_class::<api::SkyMapData>()?;
	m.add_class::<api::DistributionBlock>()?;
	m.add_class::<api::DistributionStats>()?;
	m.add_class::<api::DistributionData>()?;
	m.add_class::<api::ScheduleTimelineBlock>()?;
	m.add_class::<api::ScheduleTimelineData>()?;
	m.add_class::<api::InsightsBlock>()?;
	m.add_class::<api::AnalyticsMetrics>()?;
	m.add_class::<api::CorrelationEntry>()?;
	m.add_class::<api::ConflictRecord>()?;
	m.add_class::<api::TopObservation>()?;
	m.add_class::<api::InsightsData>()?;
	m.add_class::<api::TrendsBlock>()?;
	m.add_class::<api::EmpiricalRatePoint>()?;
	m.add_class::<api::SmoothedPoint>()?;
	m.add_class::<api::HeatmapBin>()?;
	m.add_class::<api::TrendsMetrics>()?;
	m.add_class::<api::TrendsData>()?;
	m.add_class::<api::CompareBlock>()?;
	m.add_class::<api::CompareStats>()?;
	m.add_class::<api::SchedulingChange>()?;
	m.add_class::<api::CompareData>()?;
	m.add_class::<api::VisibilityMapData>()?;

	// Route name constants
	m.add("LIST_SCHEDULES", landing::LIST_SCHEDULES)?;
	m.add("POST_SCHEDULE", landing::POST_SCHEDULE)?;
	m.add("GET_VALIDATION_REPORT", validation::GET_VALIDATION_REPORT)?;
	m.add("GET_SKY_MAP_DATA", skymap::GET_SKY_MAP_DATA)?;
	m.add("GET_DISTRIBUTION_DATA", distribution::GET_DISTRIBUTION_DATA)?;
	m.add("GET_SCHEDULE_TIMELINE_DATA", timeline::GET_SCHEDULE_TIMELINE_DATA)?;
	m.add("GET_INSIGHTS_DATA", insights::GET_INSIGHTS_DATA)?;
	m.add("GET_TRENDS_DATA", trends::GET_TRENDS_DATA)?;
	m.add("GET_COMPARE_DATA", compare::GET_COMPARE_DATA)?;
	m.add("GET_VISIBILITY_MAP_DATA", visibility::GET_VISIBILITY_MAP_DATA)?;

	Ok(())
}

