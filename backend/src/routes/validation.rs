use serde::{Deserialize, Serialize};

/// Validation issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub block_id: i64,
    pub original_block_id: Option<String>,
    pub issue_type: String,
    pub category: String,
    pub criticality: String,
    pub field_name: Option<String>,
    pub current_value: Option<String>,
    pub expected_value: Option<String>,
    pub description: String,
}

/// Validation report data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub schedule_id: crate::api::ScheduleId,
    pub total_blocks: usize,
    pub valid_blocks: usize,
    pub impossible_blocks: Vec<ValidationIssue>,
    pub validation_errors: Vec<ValidationIssue>,
    pub validation_warnings: Vec<ValidationIssue>,
}

/// Validation route function name constant
pub const GET_VALIDATION_REPORT: &str = "get_validation_report";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_issue_clone() {
        let issue = ValidationIssue {
            block_id: 99,
            original_block_id: Some("val-1".to_string()),
            issue_type: "invalid_coordinates".to_string(),
            category: "error".to_string(),
            criticality: "high".to_string(),
            field_name: Some("ra".to_string()),
            current_value: Some("400.0".to_string()),
            expected_value: Some("0-360".to_string()),
            description: "RA out of range".to_string(),
        };
        let cloned = issue.clone();
        assert_eq!(cloned.issue_type, "invalid_coordinates");
    }

    #[test]
    fn test_validation_issue_debug() {
        let issue = ValidationIssue {
            block_id: 99,
            original_block_id: Some("val-1".to_string()),
            issue_type: "invalid_coordinates".to_string(),
            category: "error".to_string(),
            criticality: "high".to_string(),
            field_name: Some("ra".to_string()),
            current_value: Some("400.0".to_string()),
            expected_value: Some("0-360".to_string()),
            description: "RA out of range".to_string(),
        };
        let debug_str = format!("{:?}", issue);
        assert!(debug_str.contains("ValidationIssue"));
    }

    #[test]
    fn test_validation_report_clone() {
        let report = ValidationReport {
            schedule_id: crate::api::ScheduleId::new(1),
            total_blocks: 100,
            valid_blocks: 95,
            impossible_blocks: vec![],
            validation_errors: vec![],
            validation_warnings: vec![],
        };
        let cloned = report.clone();
        assert_eq!(cloned.total_blocks, 100);
    }

    #[test]
    fn test_validation_report_debug() {
        let report = ValidationReport {
            schedule_id: crate::api::ScheduleId::new(1),
            total_blocks: 100,
            valid_blocks: 95,
            impossible_blocks: vec![],
            validation_errors: vec![],
            validation_warnings: vec![],
        };
        let debug_str = format!("{:?}", report);
        assert!(debug_str.contains("ValidationReport"));
    }

    #[test]
    fn test_const_value() {
        assert_eq!(GET_VALIDATION_REPORT, "get_validation_report");
    }
}
