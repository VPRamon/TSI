#[cfg(test)]
mod tests {
    use crate::api::{ScheduleId, TargetId, ConstraintsId, SchedulingBlockId};

    #[test]
    fn test_schedule_id_new() {
        let id = ScheduleId::new(42);
        assert_eq!(id.value(), 42);
    }

    #[test]
    fn test_schedule_id_equality() {
        let id1 = ScheduleId::new(100);
        let id2 = ScheduleId::new(100);
        let id3 = ScheduleId::new(101);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_schedule_id_ordering() {
        let id1 = ScheduleId::new(1);
        let id2 = ScheduleId::new(2);

        assert!(id1 < id2);
        assert!(id2 > id1);
    }

    #[test]
    fn test_schedule_id_clone() {
        let id1 = ScheduleId::new(123);
        let id2 = id1.clone();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_schedule_id_from_i64() {
        let id = ScheduleId(999);
        assert_eq!(id.0, 999);
    }

    #[test]
    fn test_target_id_new() {
        let id = TargetId::new(55);
        assert_eq!(id.value(), 55);
    }

    #[test]
    fn test_target_id_equality() {
        let id1 = TargetId::new(200);
        let id2 = TargetId::new(200);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_constraints_id_new() {
        let id = ConstraintsId::new(77);
        assert_eq!(id.value(), 77);
    }

    #[test]
    fn test_constraints_id_equality() {
        let id1 = ConstraintsId::new(300);
        let id2 = ConstraintsId::new(300);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_scheduling_block_id_new() {
        let id = SchedulingBlockId::new(88);
        assert_eq!(id.value(), 88);
    }

    #[test]
    fn test_scheduling_block_id_equality() {
        let id1 = SchedulingBlockId::new(400);
        let id2 = SchedulingBlockId::new(400);
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_all_ids_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ScheduleId::new(1));
        set.insert(ScheduleId::new(2));
        set.insert(ScheduleId::new(1)); // Duplicate

        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_schedule_id_negative() {
        let id = ScheduleId::new(-1);
        assert_eq!(id.value(), -1);
    }

    #[test]
    fn test_schedule_id_zero() {
        let id = ScheduleId::new(0);
        assert_eq!(id.value(), 0);
    }
}
