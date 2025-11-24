/// Conflict detection for impossible or problematic observations
use crate::models::schedule::SchedulingBlock;
use serde::{Deserialize, Serialize};

/// A conflict detected in the scheduling data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub scheduling_block_id: String,
    pub conflict_type: ConflictType,
    pub description: String,
    pub severity: Severity,
}

/// Type of conflict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConflictType {
    ImpossibleObservation,
    InsufficientVisibility,
    SchedulingAnomaly,
}

/// Severity level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    High,
    Medium,
    Low,
}

/// Result of conflict detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictReport {
    pub total_conflicts: usize,
    pub impossible_observations: usize,
    pub insufficient_visibility: usize,
    pub scheduling_anomalies: usize,
    pub conflicts: Vec<Conflict>,
}

/// Detect all conflicts in the scheduling data
pub fn detect_conflicts(blocks: &[SchedulingBlock], tolerance_sec: f64) -> ConflictReport {
    let mut conflicts = Vec::new();
    
    for block in blocks {
        // Check for impossible observations (visibility < min observation time)
        if block.is_impossible(tolerance_sec) {
            let visibility_hours = block.total_visibility_hours;
            let required_hours = block.requested_duration_sec / 3600.0;
            let min_hours = block.min_observation_time_in_sec / 3600.0;
            
            let description = format!(
                "Observation requires {:.2}h (min {:.2}h) but only {:.2}h visibility available",
                required_hours, min_hours, visibility_hours
            );
            
            conflicts.push(Conflict {
                scheduling_block_id: block.scheduling_block_id.clone(),
                conflict_type: ConflictType::ImpossibleObservation,
                description,
                severity: Severity::High,
            });
        }
        // Check for scheduled but insufficient visibility
        else if block.scheduled_flag {
            if let (Some(start), Some(stop)) = (block.scheduled_period_start, block.scheduled_period_stop) {
                let scheduled_hours = (stop - start) * 24.0;
                let requested_hours = block.requested_hours;
                
                // Check if scheduled time significantly differs from requested
                if (scheduled_hours - requested_hours).abs() > requested_hours * 0.2 {
                    let description = format!(
                        "Scheduled for {:.2}h but requested {:.2}h (±20% tolerance exceeded)",
                        scheduled_hours, requested_hours
                    );
                    
                    conflicts.push(Conflict {
                        scheduling_block_id: block.scheduling_block_id.clone(),
                        conflict_type: ConflictType::SchedulingAnomaly,
                        description,
                        severity: Severity::Medium,
                    });
                }
            }
        }
        // Check for low visibility compared to requested time
        else if !block.scheduled_flag && block.total_visibility_hours < block.requested_hours * 1.5 {
            let description = format!(
                "Only {:.2}h visibility for {:.2}h request (margin < 50%)",
                block.total_visibility_hours, block.requested_hours
            );
            
            conflicts.push(Conflict {
                scheduling_block_id: block.scheduling_block_id.clone(),
                conflict_type: ConflictType::InsufficientVisibility,
                description,
                severity: Severity::Low,
            });
        }
    }
    
    let impossible_observations = conflicts
        .iter()
        .filter(|c| c.conflict_type == ConflictType::ImpossibleObservation)
        .count();
    
    let insufficient_visibility = conflicts
        .iter()
        .filter(|c| c.conflict_type == ConflictType::InsufficientVisibility)
        .count();
    
    let scheduling_anomalies = conflicts
        .iter()
        .filter(|c| c.conflict_type == ConflictType::SchedulingAnomaly)
        .count();
    
    ConflictReport {
        total_conflicts: conflicts.len(),
        impossible_observations,
        insufficient_visibility,
        scheduling_anomalies,
        conflicts,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::schedule::{PriorityBin, VisibilityPeriod};

    #[test]
    fn test_detect_impossible() {
        let block = SchedulingBlock {
            scheduling_block_id: "impossible".to_string(),
            priority: 8.5,
            min_observation_time_in_sec: 36000.0, // 10 hours
            requested_duration_sec: 36000.0,
            fixed_start_time: None,
            fixed_stop_time: None,
            target_name: None,
            target_id: None,
            dec_in_deg: 0.0,
            ra_in_deg: 0.0,
            min_azimuth_angle_in_deg: 0.0,
            max_azimuth_angle_in_deg: 360.0,
            min_elevation_angle_in_deg: 60.0,
            max_elevation_angle_in_deg: 90.0,
            scheduled_period_start: None,
            scheduled_period_stop: None,
            visibility: vec![VisibilityPeriod {
                start: 61892.0,
                stop: 61892.1, // Only 2.4 hours
            }],
            num_visibility_periods: 1,
            total_visibility_hours: 2.4,
            priority_bin: PriorityBin::MediumHigh,
            scheduled_flag: false,
            requested_hours: 10.0,
            elevation_range_deg: 30.0,
        };
        
        let report = detect_conflicts(&[block], 1.0);
        
        assert_eq!(report.total_conflicts, 1);
        assert_eq!(report.impossible_observations, 1);
        assert_eq!(report.conflicts[0].conflict_type, ConflictType::ImpossibleObservation);
        assert_eq!(report.conflicts[0].severity, Severity::High);
    }

    #[test]
    fn test_no_conflicts() {
        let block = SchedulingBlock {
            scheduling_block_id: "good".to_string(),
            priority: 8.5,
            min_observation_time_in_sec: 1200.0,
            requested_duration_sec: 3600.0,
            fixed_start_time: None,
            fixed_stop_time: None,
            target_name: None,
            target_id: None,
            dec_in_deg: 0.0,
            ra_in_deg: 0.0,
            min_azimuth_angle_in_deg: 0.0,
            max_azimuth_angle_in_deg: 360.0,
            min_elevation_angle_in_deg: 60.0,
            max_elevation_angle_in_deg: 90.0,
            scheduled_period_start: Some(61892.0),
            scheduled_period_stop: Some(61892.041666667), // ~1 hour (1/24 day)
            visibility: vec![VisibilityPeriod {
                start: 61892.0,
                stop: 61892.5, // 12 hours
            }],
            num_visibility_periods: 1,
            total_visibility_hours: 12.0,
            priority_bin: PriorityBin::MediumHigh,
            scheduled_flag: true,
            requested_hours: 1.0,
            elevation_range_deg: 30.0,
        };
        
        let report = detect_conflicts(&[block], 1.0);
        
        assert_eq!(report.total_conflicts, 0);
    }
}
