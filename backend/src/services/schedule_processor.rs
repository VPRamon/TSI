//! Async schedule processing service.
//!
//! Handles heavy schedule processing tasks (parsing, astronomical night computation,
//! possible_periods handling) in the background, emitting progress logs.

use crate::api::ScheduleId;
use crate::db::repository::FullRepository;
use crate::db::services as db_services;
use crate::services::job_tracker::{JobTracker, LogLevel};
use crate::services::ScheduleImportAdapter;
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
    import_adapter: Arc<dyn ScheduleImportAdapter>,
    schedule_name: String,
    schedule_json: String,
    populate_analytics: bool,
) -> Result<ScheduleId, String> {
    let adapter_name = import_adapter.name();
    tracker.log(
        &job_id,
        LogLevel::Info,
        format!("Starting schedule import via {adapter_name}..."),
    );

    // Step 1: Parse schedule JSON
    tracker.log(
        &job_id,
        LogLevel::Info,
        format!("Parsing import payload with {adapter_name}..."),
    );
    let schedule = match tokio::task::spawn_blocking({
        let schedule_json = schedule_json.clone();
        let schedule_name = schedule_name.clone();
        let import_adapter = Arc::clone(&import_adapter);
        move || {
            import_adapter.parse_schedule(&schedule_json).map(|mut s| {
                if s.name.is_empty() {
                    s.name = schedule_name;
                }
                s
            })
        }
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
            let msg = format!("Failed to import schedule: {}", e);
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

    // Log visibility period counts (provided via possible_periods or computed in backend).
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
                "✓ {} visibility periods ready ({} blocks with periods)",
                visibility_count,
                schedule
                    .blocks
                    .iter()
                    .filter(|b| !b.visibility_periods.is_empty())
                    .count()
            ),
        );
    } else {
        tracker.log(
            &job_id,
            LogLevel::Info,
            "✓ No visibility periods (all blocks without visibility data)",
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
        format!(
            "✅ Schedule processing complete! ID: {}",
            metadata.schedule_id.value()
        ),
    );

    let result = serde_json::json!({
        "schedule_id": metadata.schedule_id.value(),
        "schedule_name": metadata.schedule_name,
    });
    tracker.complete_job(&job_id, Some(result));

    Ok(metadata.schedule_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use qtty::{Degrees, Meters};
    use siderust::coordinates::centers::Geodetic;
    use siderust::coordinates::frames::ECEF;
    use crate::api::{ModifiedJulianDate, Period, Schedule};
    use crate::db::repositories::LocalRepository;
    use crate::services::job_tracker::JobStatus;

    #[derive(Debug)]
    struct StubImportAdapter {
        schedule: Option<Schedule>,
        error_message: Option<&'static str>,
    }

    impl ScheduleImportAdapter for StubImportAdapter {
        fn name(&self) -> &'static str {
            "stub-import"
        }

        fn parse_schedule(&self, _raw_payload: &str) -> anyhow::Result<Schedule> {
            if let Some(message) = self.error_message {
                anyhow::bail!(message);
            }

            self.schedule
                .clone()
                .ok_or_else(|| anyhow::anyhow!("missing stub schedule"))
        }
    }

    fn make_schedule(name: &str) -> Schedule {
        Schedule {
            id: None,
            name: name.to_string(),
            checksum: format!("checksum-{name}"),
            schedule_period: Period {
                start: ModifiedJulianDate::new(60000.0),
                end: ModifiedJulianDate::new(60001.0),
            },
            dark_periods: vec![],
            geographic_location: Geodetic::<ECEF>::new(
                Degrees::new(-17.8892),
                Degrees::new(28.7624),
                Meters::new(2396.0),
            ),
            astronomical_nights: vec![],
            blocks: vec![],
        }
    }

    #[tokio::test]
    async fn process_schedule_uses_injected_import_adapter() {
        let tracker = JobTracker::new();
        let job_id = tracker.create_job();
        let repo = Arc::new(LocalRepository::new()) as Arc<dyn FullRepository>;
        let adapter = Arc::new(StubImportAdapter {
            schedule: Some(make_schedule("")),
            error_message: None,
        }) as Arc<dyn ScheduleImportAdapter>;

        let result = process_schedule_async(
            job_id.clone(),
            tracker.clone(),
            Arc::clone(&repo),
            adapter,
            "fallback-name".to_string(),
            "not valid native schedule json".to_string(),
            false,
        )
        .await;

        assert!(result.is_ok());

        let schedules = db_services::list_schedules(repo.as_ref()).await.unwrap();
        assert_eq!(schedules.len(), 1);
        assert_eq!(schedules[0].schedule_name, "fallback-name");

        let job = tracker.get_job(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Completed);
    }

    #[tokio::test]
    async fn process_schedule_surfaces_adapter_errors() {
        let tracker = JobTracker::new();
        let job_id = tracker.create_job();
        let repo = Arc::new(LocalRepository::new()) as Arc<dyn FullRepository>;
        let adapter = Arc::new(StubImportAdapter {
            schedule: None,
            error_message: Some("adapter rejected payload"),
        }) as Arc<dyn ScheduleImportAdapter>;

        let result = process_schedule_async(
            job_id.clone(),
            tracker.clone(),
            repo,
            adapter,
            "ignored-name".to_string(),
            "{}".to_string(),
            false,
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("adapter rejected payload"));

        let job = tracker.get_job(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Failed);
    }
}
