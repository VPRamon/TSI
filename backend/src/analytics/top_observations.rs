/// Top observations ranking and filtering
use crate::models::schedule::SchedulingBlock;
use serde::{Deserialize, Serialize};

/// Sorting criteria for observations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    Priority,
    RequestedHours,
    VisibilityHours,
    ElevationRange,
}

/// Sorting order
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Simplified observation for ranking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankedObservation {
    pub scheduling_block_id: String,
    pub priority: f64,
    pub requested_hours: f64,
    pub total_visibility_hours: f64,
    pub elevation_range_deg: f64,
    pub scheduled_flag: bool,
    pub priority_bin: String,
}

impl From<&SchedulingBlock> for RankedObservation {
    fn from(block: &SchedulingBlock) -> Self {
        Self {
            scheduling_block_id: block.scheduling_block_id.clone(),
            priority: block.priority,
            requested_hours: block.requested_hours,
            total_visibility_hours: block.total_visibility_hours,
            elevation_range_deg: block.elevation_range_deg,
            scheduled_flag: block.scheduled_flag,
            priority_bin: format!("{}", block.priority_bin),
        }
    }
}

/// Get top N observations sorted by specified criteria
pub fn get_top_observations(
    blocks: &[SchedulingBlock],
    sort_by: SortBy,
    order: SortOrder,
    limit: usize,
    scheduled_only: Option<bool>,
) -> Vec<RankedObservation> {
    // Filter by scheduled status if specified
    let filtered: Vec<&SchedulingBlock> = if let Some(scheduled) = scheduled_only {
        blocks.iter().filter(|b| b.scheduled_flag == scheduled).collect()
    } else {
        blocks.iter().collect()
    };
    
    // Convert to ranked observations
    let mut ranked: Vec<RankedObservation> = filtered.iter().map(|b| (*b).into()).collect();
    
    // Sort by specified criteria
    match sort_by {
        SortBy::Priority => {
            ranked.sort_by(|a, b| {
                let cmp = a.priority.partial_cmp(&b.priority).unwrap_or(std::cmp::Ordering::Equal);
                if order == SortOrder::Ascending { cmp } else { cmp.reverse() }
            });
        }
        SortBy::RequestedHours => {
            ranked.sort_by(|a, b| {
                let cmp = a.requested_hours.partial_cmp(&b.requested_hours).unwrap_or(std::cmp::Ordering::Equal);
                if order == SortOrder::Ascending { cmp } else { cmp.reverse() }
            });
        }
        SortBy::VisibilityHours => {
            ranked.sort_by(|a, b| {
                let cmp = a.total_visibility_hours.partial_cmp(&b.total_visibility_hours).unwrap_or(std::cmp::Ordering::Equal);
                if order == SortOrder::Ascending { cmp } else { cmp.reverse() }
            });
        }
        SortBy::ElevationRange => {
            ranked.sort_by(|a, b| {
                let cmp = a.elevation_range_deg.partial_cmp(&b.elevation_range_deg).unwrap_or(std::cmp::Ordering::Equal);
                if order == SortOrder::Ascending { cmp } else { cmp.reverse() }
            });
        }
    }
    
    // Take top N
    ranked.truncate(limit);
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::schedule::{PriorityBin, VisibilityPeriod};

    fn create_test_block(id: &str, priority: f64, requested: f64, visibility: f64) -> SchedulingBlock {
        SchedulingBlock {
            scheduling_block_id: id.to_string(),
            priority,
            min_observation_time_in_sec: 1200.0,
            requested_duration_sec: requested * 3600.0,
            fixed_start_time: None,
            fixed_stop_time: None,
            dec_in_deg: 0.0,
            ra_in_deg: 0.0,
            min_azimuth_angle_in_deg: 0.0,
            max_azimuth_angle_in_deg: 360.0,
            min_elevation_angle_in_deg: 60.0,
            max_elevation_angle_in_deg: 90.0,
            scheduled_period_start: if priority > 5.0 { Some(61892.0) } else { None },
            scheduled_period_stop: if priority > 5.0 { Some(61892.1) } else { None },
            visibility: vec![VisibilityPeriod {
                start: 61892.0,
                stop: 61892.0 + visibility / 24.0,
            }],
            num_visibility_periods: 1,
            total_visibility_hours: visibility,
            priority_bin: PriorityBin::from_priority(priority),
            scheduled_flag: priority > 5.0,
            requested_hours: requested,
            elevation_range_deg: 30.0,
        }
    }

    #[test]
    fn test_sort_by_priority() {
        let blocks = vec![
            create_test_block("1", 5.0, 1.0, 10.0),
            create_test_block("2", 10.0, 2.0, 12.0),
            create_test_block("3", 3.0, 1.5, 8.0),
        ];
        
        let top = get_top_observations(&blocks, SortBy::Priority, SortOrder::Descending, 2, None);
        
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].scheduling_block_id, "2");
        assert_eq!(top[1].scheduling_block_id, "1");
    }

    #[test]
    fn test_filter_scheduled() {
        let blocks = vec![
            create_test_block("1", 5.0, 1.0, 10.0),   // Not scheduled (priority <= 5)
            create_test_block("2", 10.0, 2.0, 12.0),  // Scheduled
            create_test_block("3", 8.0, 1.5, 8.0),    // Scheduled
        ];
        
        let scheduled = get_top_observations(&blocks, SortBy::Priority, SortOrder::Descending, 10, Some(true));
        
        assert_eq!(scheduled.len(), 2);
        assert!(scheduled.iter().all(|o| o.scheduled_flag));
    }
}
