use crate::api::ScheduleId;
use serde::{Deserialize, Serialize};

/// Schedule information with block counts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleInfo {
    pub schedule_id: ScheduleId,
    pub schedule_name: String,
}

pub const LIST_SCHEDULES: &str = "list_schedules";
pub const POST_SCHEDULE: &str = "store_schedule";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schedule_info_clone() {
        let info = ScheduleInfo {
            schedule_id: ScheduleId::new(123),
            schedule_name: "Test Schedule".to_string(),
        };
        let cloned = info.clone();
        assert_eq!(cloned.schedule_id.value(), 123);
        assert_eq!(cloned.schedule_name, "Test Schedule");
    }

    #[test]
    fn test_schedule_info_debug() {
        let info = ScheduleInfo {
            schedule_id: ScheduleId::new(123),
            schedule_name: "Test Schedule".to_string(),
        };
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("ScheduleInfo"));
    }

    #[test]
    fn test_const_values() {
        assert_eq!(LIST_SCHEDULES, "list_schedules");
        assert_eq!(POST_SCHEDULE, "store_schedule");
    }
}
