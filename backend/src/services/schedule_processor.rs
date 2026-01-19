//! Async schedule processing service.
//!
//! Handles heavy schedule processing tasks (parsing, astronomical night computation,
//! possible_periods handling) in the background, emitting progress logs.

use crate::api::ScheduleId;
use crate::db::repository::FullRepository;
use crate::db::services as db_services;
use crate::models;
use crate::services::job_tracker::{JobTracker, LogLevel};
use std::sync::Arc;

/// Process a schedule asynchronously: parse, compute nights, store, and populate analytics.
///
/// This function is designed to be spawned as a background task. It logs progress
/// to the job tracker so users can see what's happening via SSE.
///
/// # Arguments
/// * `job_id` - The job ID for tracking progress
/// * `tracker` - Job tracker for logging
/// * `repo` - Repository for storing the schedule
/// * `schedule_name` - Name for the schedule
/// * `schedule_json` - JSON string of the schedule
/// * `populate_analytics` - Whether to populate analytics after storing
///
/// # Returns
/// * Schedule ID on success, or error message on failure
pub async fn process_schedule_async(
    job_id: String,
    tracker: JobTracker,
    repo: Arc<dyn FullRepository>,
    schedule_name: String,
    schedule_json: String,
    populate_analytics: bool,
) -> Result<ScheduleId, String> {
    tracker.log(&job_id, LogLevel::Info, "Starting schedule processing...");
    
    // Step 1: Parse schedule JSON
    tracker.log(&job_id, LogLevel::Info, "Parsing schedule JSON...");
    let schedule = match tokio::task::spawn_blocking({
        let schedule_json = schedule_json.clone();
        let schedule_name = schedule_name.clone();
        move || models::schedule::parse_schedule_json_str(&schedule_json)
            .map(|mut s| {
                if s.name.is_empty() {
                    s.name = schedule_name;
                }
                s
            })
    })
    .await
    {
        Ok(Ok(schedule)) => {
            tracker.log(
                &job_id,
                LogLevel::Success,
                format!("✓ Parsed schedule with {} blocks", schedule.blocks.len()),
            );
            schedule
        }
        Ok(Err(e)) => {
            let msg = format!("Failed to parse schedule: {}", e);
            tracker.fail_job(&job_id, &msg);
            return Err(msg);
        }
        Err(e) => {
            let msg = format!("Parse task panic: {}", e);
            tracker.fail_job(&job_id, &msg);
            return Err(msg);
        }
    };

    // Log astronomical nights computation
    tracker.log(
        &job_id,
        LogLevel::Info,
        format!(
            "✓ Computed {} astronomical night periods",
            schedule.astronomical_nights.len()
        ),
    );

    // Log possible_periods handling if present
    let visibility_count: usize = schedule
        .blocks
        .iter()
        .map(|b| b.visibility_periods.len())
        .sum();
    if visibility_count > 0 {
        tracker.log(
            &job_id,
            LogLevel::Info,
            format!(
                "✓ Loaded {} visibility periods from possible_periods map",
                visibility_count
            ),
        );
    }

    // Step 2: Store schedule
    tracker.log(&job_id, LogLevel::Info, "Storing schedule in repository...");
    let metadata = match db_services::store_schedule_with_options(
        repo.as_ref(),
        &schedule,
        populate_analytics,
    )
    .await
    {
        Ok(metadata) => {
            tracker.log(
                &job_id,
                LogLevel::Success,
                format!("✓ Stored schedule (ID: {})", metadata.schedule_id.value()),
            );
            metadata
        }
        Err(e) => {
            let msg = format!("Failed to store schedule: {}", e);
            tracker.fail_job(&job_id, &msg);
            return Err(msg);
        }
    };

    // Step 3: Log analytics population if enabled
    if populate_analytics {
        tracker.log(
            &job_id,
            LogLevel::Info,
            "✓ Analytics populated successfully",
        );
    }

    // Mark job as complete
    tracker.log(
        &job_id,
        LogLevel::Success,
        format!("✅ Schedule processing complete! ID: {}", metadata.schedule_id.value()),
    );
    
    let result = serde_json::json!({
        "schedule_id": metadata.schedule_id.value(),
        "schedule_name": metadata.schedule_name,
    });
    tracker.complete_job(&job_id, Some(result));

    Ok(metadata.schedule_id)
}
