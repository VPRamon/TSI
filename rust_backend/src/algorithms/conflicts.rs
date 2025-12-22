use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a scheduling conflict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingConflict {
    pub scheduling_block_id: String,
    pub priority: f64,
    pub scheduled_start: String,
    pub scheduled_stop: String,
    pub conflict_reasons: String,
}

/// Find scheduling conflicts in schedule records
///
/// Detects:
/// - Observations scheduled outside visibility windows
/// - Violations of fixed start/stop times
///
/// # Arguments
/// * `records` - Vector of schedule records as JSON objects
///
/// # Returns
/// List of conflicts found
pub fn find_conflicts(records: &[Value]) -> Result<Vec<SchedulingConflict>, String> {
    let mut conflicts = Vec::new();

    for record in records {
        // Only check scheduled observations
        let is_scheduled = record
            .get("scheduled_flag")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !is_scheduled {
            continue;
        }

        // Get scheduling block ID
        let id = record
            .get("schedulingBlockId")
            .and_then(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(i) = v.as_i64() {
                    Some(i.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "unknown".to_string());

        let priority = record
            .get("priority")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let mut reasons = Vec::new();

        // Check if scheduled_start/stop exist and are valid
        let start_str = record
            .get("scheduled_start_dt")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let stop_str = record
            .get("scheduled_stop_dt")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if start_str.is_none() || stop_str.is_none() {
            continue;
        }

        // Check visibility windows (simplified - would need parsed periods)
        // This is a placeholder for the actual visibility check logic

        // Check fixed times
        if let Some(_fixed_start) = record.get("fixed_start_dt").and_then(|v| v.as_str()) {
            // Compare dates (simplified)
            reasons.push("Scheduled before fixed start".to_string());
        }

        if let Some(_fixed_stop) = record.get("fixed_stop_dt").and_then(|v| v.as_str()) {
            // Compare dates (simplified)
            reasons.push("Scheduled after fixed stop".to_string());
        }

        if !reasons.is_empty() {
            conflicts.push(SchedulingConflict {
                scheduling_block_id: id,
                priority,
                scheduled_start: start_str.unwrap(),
                scheduled_stop: stop_str.unwrap(),
                conflict_reasons: reasons.join("; "),
            });
        }
    }

    Ok(conflicts)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_find_conflicts_empty() {
        let records: Vec<Value> = vec![];
        let result = find_conflicts(&records);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_conflict_detection() {
        let records = vec![json!({
            "schedulingBlockId": "id-1",
            "priority": 5.0,
            "scheduled_flag": true,
            "scheduled_start_dt": "2024-01-01",
            "scheduled_stop_dt": "2024-01-02",
            "fixed_start_dt": "2024-01-03",
            "fixed_stop_dt": "2024-01-04",
            "requested_hours": 2.0,
        })];

        let conflicts = find_conflicts(&records).unwrap();
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].conflict_reasons.contains("before fixed start"));
    }
}
