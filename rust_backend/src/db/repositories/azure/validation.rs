// Azure validation implementation removed — placeholder
#![allow(clippy::all)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unreachable_code)]

use crate::api::{ScheduleId, ValidationReport};
use crate::services::validation::ValidationResult;

/// Placeholder indicating implementation removed.
pub(crate) fn _azure_validation_todo() -> ! {
    todo!("Azure validation implementation removed — TODO: re-implement")
}

pub async fn insert_validation_results(_results: &[ValidationResult]) -> Result<usize, String> {
    todo!("Azure placeholder: insert_validation_results")
}

pub async fn update_validation_impossible_flags(_schedule_id: ScheduleId) -> Result<usize, String> {
    todo!("Azure placeholder: update_validation_impossible_flags")
}

pub async fn fetch_validation_results(
    _schedule_id: ScheduleId,
) -> Result<ValidationReport, String> {
    todo!("Azure placeholder: fetch_validation_results")
}

pub async fn has_validation_results(_schedule_id: ScheduleId) -> Result<bool, String> {
    todo!("Azure placeholder: has_validation_results")
}

pub async fn delete_validation_results(_schedule_id: ScheduleId) -> Result<u64, String> {
    todo!("Azure placeholder: delete_validation_results")
}
