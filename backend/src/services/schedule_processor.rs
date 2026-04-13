//! Async schedule processing service.
//!
//! Handles heavy schedule processing tasks (parsing, astronomical night computation,
//! possible_periods handling) in the background, emitting progress logs.

use crate::api::{ModifiedJulianDate, Period, ScheduleId, SchedulingBlock};
use crate::db::repository::FullRepository;
use crate::db::services as db_services;
use crate::http::dto::SchedulePeriodOverride;
use crate::services::astronomical_night::compute_astronomical_nights;
use crate::services::job_tracker::{JobTracker, LogLevel};
use crate::services::visibility_service::{compute_block_visibility, VisibilityInput};
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
/// * `period_override` - Optional manual schedule period override. When present, replaces the
///   period inferred from the payload and triggers recomputation of astronomical nights and
///   block visibility.
///
/// # Returns
/// * Schedule ID on success, or error message on failure
#[allow(clippy::too_many_arguments)]
pub async fn process_schedule_async(
    job_id: String,
    tracker: JobTracker,
    repo: Arc<dyn FullRepository>,
    import_adapter: Arc<dyn ScheduleImportAdapter>,
    schedule_name: String,
    schedule_json: String,
    populate_analytics: bool,
    period_override: Option<SchedulePeriodOverride>,
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

    // Apply schedule period override when supplied by the caller.
    let mut schedule = schedule;
    if let Some(ref ov) = period_override {
        let override_period = Period {
            start: ModifiedJulianDate::new(ov.start_mjd),
            end: ModifiedJulianDate::new(ov.end_mjd),
        };

        // Validate that no scheduled block falls outside the override window.
        for block in &schedule.blocks {
            if let Some(ref sp) = block.scheduled_period {
                if sp.start.value() < ov.start_mjd || sp.end.value() > ov.end_mjd {
                    let msg = format!(
                        "Block '{}' scheduled period ({:.5}–{:.5}) falls outside the override \
                         window ({:.5}–{:.5})",
                        block.original_block_id,
                        sp.start.value(),
                        sp.end.value(),
                        ov.start_mjd,
                        ov.end_mjd,
                    );
                    tracker.fail_job(&job_id, &msg);
                    return Err(msg);
                }
            }
        }

        tracker.log(
            &job_id,
            LogLevel::Info,
            format!(
                "⚙ Applying schedule period override: MJD {:.5}–{:.5} \
                 (replacing inferred period MJD {:.5}–{:.5})",
                ov.start_mjd,
                ov.end_mjd,
                schedule.schedule_period.start.value(),
                schedule.schedule_period.end.value(),
            ),
        );

        schedule.schedule_period = override_period;

        // Recompute astronomical nights for the new window.
        let location = schedule.geographic_location;
        let new_nights = compute_astronomical_nights(&location, &override_period);
        schedule.astronomical_nights = new_nights.clone();
        schedule.dark_periods = new_nights.clone();

        // Recompute visibility for all blocks using the new period.
        apply_visibility_override(
            &mut schedule.blocks,
            &override_period,
            &new_nights,
            &location,
        );

        tracker.log(
            &job_id,
            LogLevel::Success,
            format!(
                "✓ Period override applied; recomputed {} astronomical nights",
                schedule.astronomical_nights.len()
            ),
        );
    }

    // Log astronomical nights computation (only when no override was applied, to avoid redundancy).
    if period_override.is_none() {
        tracker.log(
            &job_id,
            LogLevel::Info,
            format!(
                "✓ Computed {} astronomical night periods",
                schedule.astronomical_nights.len()
            ),
        );
    }

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

/// Recompute block visibility periods using the supplied `period` and `nights`.
///
/// All existing visibility periods on each block are discarded and replaced.
fn apply_visibility_override(
    blocks: &mut [SchedulingBlock],
    period: &Period,
    nights: &[Period],
    location: &crate::api::GeographicLocation,
) {
    for block in blocks.iter_mut() {
        block.visibility_periods = compute_block_visibility(&VisibilityInput {
            location,
            schedule_period: period,
            target_ra: block.target_ra,
            target_dec: block.target_dec,
            constraints: &block.constraints,
            min_duration: block.min_observation,
            astronomical_nights: Some(nights),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::{ModifiedJulianDate, Period, Schedule};
    use crate::db::repositories::LocalRepository;
    use crate::services::job_tracker::JobStatus;
    use qtty::{Degrees, Meters};
    use siderust::coordinates::centers::Geodetic;
    use siderust::coordinates::frames::ECEF;

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
            None,
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
            None,
        )
        .await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("adapter rejected payload"));

        let job = tracker.get_job(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Failed);
    }

    #[tokio::test]
    async fn period_override_replaces_inferred_schedule_period() {
        let tracker = JobTracker::new();
        let job_id = tracker.create_job();
        let repo = Arc::new(LocalRepository::new()) as Arc<dyn FullRepository>;
        // Schedule parsed by the adapter uses 60000–60001.
        let adapter = Arc::new(StubImportAdapter {
            schedule: Some(make_schedule("override-test")),
            error_message: None,
        }) as Arc<dyn ScheduleImportAdapter>;

        let override_start = 60010.0_f64;
        let override_end = 60020.0_f64;

        let result = process_schedule_async(
            job_id.clone(),
            tracker.clone(),
            Arc::clone(&repo),
            adapter,
            "override-test".to_string(),
            "irrelevant".to_string(),
            false,
            Some(SchedulePeriodOverride {
                start_mjd: override_start,
                end_mjd: override_end,
            }),
        )
        .await;

        assert!(result.is_ok(), "processing should succeed");

        let schedules = db_services::list_schedules(repo.as_ref()).await.unwrap();
        assert_eq!(schedules.len(), 1);

        // Retrieve the stored schedule and confirm the period was overridden.
        let stored = db_services::get_schedule(repo.as_ref(), schedules[0].schedule_id)
            .await
            .unwrap();
        assert!(
            (stored.schedule_period.start.value() - override_start).abs() < 1e-9,
            "stored period start should match override"
        );
        assert!(
            (stored.schedule_period.end.value() - override_end).abs() < 1e-9,
            "stored period end should match override"
        );

        let job = tracker.get_job(&job_id).unwrap();
        assert_eq!(job.status, JobStatus::Completed);
    }

    #[tokio::test]
    async fn period_override_rejects_block_outside_window() {
        use crate::api::{Constraints, SchedulingBlock};
        use qtty::Seconds;

        let tracker = JobTracker::new();
        let job_id = tracker.create_job();
        let repo = Arc::new(LocalRepository::new()) as Arc<dyn FullRepository>;

        // Build a schedule with one block whose scheduled_period is outside the override window.
        let mut sched = make_schedule("reject-test");
        sched.blocks.push(SchedulingBlock {
            id: None,
            original_block_id: "blk-001".to_string(),
            block_name: "outside-block".to_string(),
            target_ra: qtty::Degrees::new(10.0),
            target_dec: qtty::Degrees::new(-30.0),
            constraints: Constraints {
                min_alt: qtty::Degrees::new(30.0),
                max_alt: qtty::Degrees::new(90.0),
                min_az: qtty::Degrees::new(0.0),
                max_az: qtty::Degrees::new(360.0),
                fixed_time: None,
            },
            priority: 5.0,
            min_observation: Seconds::new(1800.0),
            requested_duration: Seconds::new(3600.0),
            visibility_periods: vec![],
            // Block is scheduled at 60025–60026, but override window is 60010–60020.
            scheduled_period: Some(Period {
                start: ModifiedJulianDate::new(60025.0),
                end: ModifiedJulianDate::new(60026.0),
            }),
        });

        let adapter = Arc::new(StubImportAdapter {
            schedule: Some(sched),
            error_message: None,
        }) as Arc<dyn ScheduleImportAdapter>;

        let result = process_schedule_async(
            job_id.clone(),
            tracker.clone(),
            repo,
            adapter,
            "reject-test".to_string(),
            "irrelevant".to_string(),
            false,
            Some(SchedulePeriodOverride {
                start_mjd: 60010.0,
                end_mjd: 60020.0,
            }),
        )
        .await;

        assert!(
            result.is_err(),
            "should fail when block falls outside window"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("falls outside the override window"),
            "error message should mention the window: {err}"
        );
    }
}
