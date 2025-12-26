//! Generic conversions shared across multiple routes.
//!
//! This module contains conversions that are not owned by a single
//! route and are therefore placed under `routes::conversions`.

impl From<&crate::db::models::Period> for crate::api::Period {
    fn from(period: &crate::db::models::Period) -> Self {
        crate::api::Period {
            start: period.start.value(),
            stop: period.stop.value(),
        }
    }
}

impl From<&crate::db::models::Constraints> for crate::api::Constraints {
    fn from(constraints: &crate::db::models::Constraints) -> Self {
        crate::api::Constraints {
            min_alt: constraints.min_alt.value(),
            max_alt: constraints.max_alt.value(),
            min_az: constraints.min_az.value(),
            max_az: constraints.max_az.value(),
            fixed_time: constraints.fixed_time.as_ref().map(|p| p.into()),
        }
    }
}

impl From<&crate::db::models::SchedulingBlock> for crate::api::SchedulingBlock {
    fn from(block: &crate::db::models::SchedulingBlock) -> Self {
        crate::api::SchedulingBlock {
            id: block.id.0,
            original_block_id: block.original_block_id.clone(),
            target_ra: block.target_ra.value(),
            target_dec: block.target_dec.value(),
            constraints: (&block.constraints).into(),
            priority: block.priority,
            min_observation: block.min_observation.value(),
            requested_duration: block.requested_duration.value(),
            visibility_periods: block.visibility_periods.iter().map(|p| p.into()).collect(),
            scheduled_period: block.scheduled_period.as_ref().map(|p| p.into()),
        }
    }
}

impl From<&crate::db::models::Schedule> for crate::api::Schedule {
    fn from(schedule: &crate::db::models::Schedule) -> Self {
        crate::api::Schedule {
            id: schedule.id.map(|id| id.0),
            name: schedule.name.clone(),
            checksum: schedule.checksum.clone(),
            dark_periods: schedule
                .dark_periods
                .iter()
                .map(|p| p.into())
                .collect(),
            blocks: schedule.blocks.iter().map(|b| b.into()).collect(),
        }
    }
}

// Helper functions for collections
pub fn convert_vec<T, U>(items: &[T]) -> Vec<U>
where
    U: for<'a> From<&'a T>,
{
    items.iter().map(|item| item.into()).collect()
}

pub fn convert_option<T, U>(item: &Option<T>) -> Option<U>
where
    U: for<'a> From<&'a T>,
{
    item.as_ref().map(|i| i.into())
}
