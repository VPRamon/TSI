//! Generic conversions shared across multiple routes.
//!
//! This module contains conversions that are not owned by a single
//! route and are therefore placed under `routes::conversions`.

use crate::api::types as api;
use crate::db::models;

// =========================================================
// Core Schedule Types - Internal to API
// =========================================================

impl From<models::ScheduleId> for api::ScheduleId {
    fn from(id: models::ScheduleId) -> Self {
        api::ScheduleId(id.0)
    }
}

impl From<models::TargetId> for api::TargetId {
    fn from(id: models::TargetId) -> Self {
        api::TargetId(id.0)
    }
}

impl From<models::ConstraintsId> for api::ConstraintsId {
    fn from(id: models::ConstraintsId) -> Self {
        api::ConstraintsId(id.0)
    }
}

impl From<models::SchedulingBlockId> for api::SchedulingBlockId {
    fn from(id: models::SchedulingBlockId) -> Self {
        api::SchedulingBlockId(id.0)
    }
}

impl From<api::ScheduleId> for models::ScheduleId {
    fn from(id: api::ScheduleId) -> Self {
        models::ScheduleId(id.0)
    }
}

impl From<api::TargetId> for models::TargetId {
    fn from(id: api::TargetId) -> Self {
        models::TargetId(id.0)
    }
}

impl From<api::ConstraintsId> for models::ConstraintsId {
    fn from(id: api::ConstraintsId) -> Self {
        models::ConstraintsId(id.0)
    }
}

impl From<api::SchedulingBlockId> for models::SchedulingBlockId {
    fn from(id: api::SchedulingBlockId) -> Self {
        models::SchedulingBlockId(id.0)
    }
}

impl From<&models::Period> for api::Period {
    fn from(period: &models::Period) -> Self {
        api::Period {
            start: period.start.value(),
            stop: period.stop.value(),
        }
    }
}

impl From<&models::Constraints> for api::Constraints {
    fn from(constraints: &models::Constraints) -> Self {
        api::Constraints {
            min_alt: constraints.min_alt.value(),
            max_alt: constraints.max_alt.value(),
            min_az: constraints.min_az.value(),
            max_az: constraints.max_az.value(),
            fixed_time: constraints.fixed_time.as_ref().map(|p| p.into()),
        }
    }
}

impl From<&models::SchedulingBlock> for api::SchedulingBlock {
    fn from(block: &models::SchedulingBlock) -> Self {
        api::SchedulingBlock {
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

impl From<&models::Schedule> for api::Schedule {
    fn from(schedule: &models::Schedule) -> Self {
        api::Schedule {
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

impl From<&models::ScheduleMetadata> for api::ScheduleMetadata {
    fn from(metadata: &models::ScheduleMetadata) -> Self {
        api::ScheduleMetadata {
            schedule_id: metadata.schedule_id,
            schedule_name: metadata.schedule_name.clone(),
            upload_timestamp: metadata.upload_timestamp.to_rfc3339(),
            checksum: metadata.checksum.clone(),
        }
    }
}

// LightweightBlock conversion (used by visualization code)
impl From<&crate::api::LightweightBlock> for api::LightweightBlock {
    fn from(block: &crate::api::LightweightBlock) -> Self {
        api::LightweightBlock {
            original_block_id: block.original_block_id.clone(),
            priority: block.priority,
            priority_bin: block.priority_bin.clone(),
            requested_duration_seconds: block.requested_duration_seconds,
            target_ra_deg: block.target_ra_deg,
            target_dec_deg: block.target_dec_deg,
            scheduled_period: block.scheduled_period.clone(),
        }
    }
}

// Algorithm result conversion (shared)
impl From<&crate::algorithms::SchedulingConflict> for api::SchedulingConflict {
    fn from(conflict: &crate::algorithms::SchedulingConflict) -> Self {
        api::SchedulingConflict {
            block_id_1: conflict.scheduling_block_id.clone(),
            block_id_2: String::new(),
            overlap_start: 0.0,
            overlap_end: 0.0,
            overlap_duration_hours: 0.0,
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
