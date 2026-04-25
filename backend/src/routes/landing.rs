use crate::api::{GeographicLocation, Period, ScheduleId};
use serde::{Deserialize, Serialize};

/// Schedule information returned by the list endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleInfo {
    pub schedule_id: ScheduleId,
    pub schedule_name: String,
    /// Geographic location of the observatory.
    pub observer_location: GeographicLocation,
    /// Overall time window of the schedule in MJD.
    pub schedule_period: Period,
    /// Optional environment assignment.
    #[serde(default)]
    pub environment_id: Option<i64>,
}

pub const LIST_SCHEDULES: &str = "list_schedules";
pub const POST_SCHEDULE: &str = "store_schedule";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::ModifiedJulianDate;
    use qtty::{Degrees, Meters};
    use siderust::coordinates::centers::Geodetic;
    use siderust::coordinates::frames::ECEF;

    fn test_info() -> ScheduleInfo {
        ScheduleInfo {
            schedule_id: ScheduleId::new(123),
            schedule_name: "Test Schedule".to_string(),
            observer_location: Geodetic::<ECEF>::new(
                Degrees::new(-17.89),
                Degrees::new(28.76),
                Meters::new(2200.0),
            ),
            schedule_period: Period {
                start: ModifiedJulianDate::new(60000.0),
                end: ModifiedJulianDate::new(60007.0),
            },
            environment_id: None,
        }
    }

    #[test]
    fn test_schedule_info_clone() {
        let info = test_info();
        let cloned = info.clone();
        assert_eq!(cloned.schedule_id.value(), 123);
        assert_eq!(cloned.schedule_name, "Test Schedule");
    }

    #[test]
    fn test_schedule_info_debug() {
        let info = test_info();
        let debug_str = format!("{:?}", info);
        assert!(debug_str.contains("ScheduleInfo"));
    }

    #[test]
    fn test_const_values() {
        assert_eq!(LIST_SCHEDULES, "list_schedules");
        assert_eq!(POST_SCHEDULE, "store_schedule");
    }
}
