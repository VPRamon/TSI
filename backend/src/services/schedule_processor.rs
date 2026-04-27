//! Async schedule processing service.
//!
//! Handles heavy schedule processing tasks (parsing, astronomical night computation,
//! possible_periods handling) in the background, emitting progress logs.

use crate::api::{ModifiedJulianDate, Period, ScheduleId, SchedulingBlock};
use crate::db::repository::FullRepository;
use crate::db::services as db_services;
use crate::http::dto::SchedulePeriodOverride;
use crate::models::schedule::compute_schedule_checksum;
use crate::services::astronomical_night::compute_astronomical_nights;
use crate::services::job_tracker::{JobTracker, LogLevel};
use crate::services::visibility::{compute_block_visibility, VisibilityInput};
use crate::services::ScheduleImportAdapter;
use rayon::prelude::*;
use std::sync::Arc;

/// Validator callback for algorithm trace summaries. Provided by the
/// http extension layer; receives the parsed `algorithm` name and the
/// summary JSON object, returns `Err(human_readable)` to reject the
/// upload.
pub type TraceValidatorFn =
    Arc<dyn Fn(&str, &serde_json::Value) -> Result<(), String> + Send + Sync>;

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
    algorithm_trace_jsonl: Option<String>,
    trace_validator: Option<TraceValidatorFn>,
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
                // User-provided name always takes precedence over the adapter's placeholder.
                if !schedule_name.is_empty() {
                    s.name = schedule_name.clone();
                }
                // Salt the checksum with the name so the same file uploaded under
                // different names produces distinct database entries.
                s.checksum = compute_schedule_checksum(&format!("{}:{}", s.name, schedule_json));
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

    // Step 3: Persist algorithm trace alongside the schedule when supplied.
    if let Some(trace_text) = algorithm_trace_jsonl.as_deref() {
        tracker.log(&job_id, LogLevel::Info, "Persisting algorithm trace...");
        match parse_algorithm_trace_jsonl(trace_text) {
            Ok((algorithm, summary, iterations)) => {
                if let Some(validator) = trace_validator.as_deref() {
                    if let Err(e) = validator(&algorithm, &summary) {
                        let msg =
                            format!("Algorithm trace validation failed for `{algorithm}`: {e}");
                        tracker.fail_job(&job_id, &msg);
                        return Err(msg);
                    }
                }
                let iter_count = iterations.as_array().map(|a| a.len()).unwrap_or(0);
                if let Err(e) = repo
                    .store_algorithm_trace(metadata.schedule_id, &algorithm, &summary, &iterations)
                    .await
                {
                    tracker.log(
                        &job_id,
                        LogLevel::Warning,
                        format!("⚠ Failed to persist algorithm trace ({iter_count} rows): {e}"),
                    );
                } else {
                    tracker.log(
                        &job_id,
                        LogLevel::Success,
                        format!("✓ Stored {algorithm} algorithm trace ({iter_count} iterations)"),
                    );
                }
            }
            Err(e) => {
                tracker.log(
                    &job_id,
                    LogLevel::Warning,
                    format!("⚠ Skipping algorithm trace; failed to parse JSONL: {e}"),
                );
            }
        }
    }

    // Step 4: Log analytics population if enabled
    if populate_analytics {
        tracker.log(&job_id, LogLevel::Info, "Computing analytics...");
        tracker.log(
            &job_id,
            LogLevel::Success,
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
    blocks.par_iter_mut().for_each(|block| {
        block.visibility_periods = compute_block_visibility(&VisibilityInput {
            location,
            schedule_period: period,
            target_ra: block.target_ra,
            target_dec: block.target_dec,
            constraints: &block.constraints,
            min_duration: block.min_observation,
            astronomical_nights: Some(nights),
        });
    });
}

/// Parse a generic algorithm trace in JSONL form into the data persisted by
/// [`AlgorithmTraceRepository::store_algorithm_trace`].
///
/// The trace format is intentionally minimal: each line is a JSON object
/// with a `kind` discriminator. Three kinds are recognised, all others are
/// ignored:
///
/// - `"started"`              → carries the algorithm identifier and its
///   configuration; merged into the summary.
/// - `"iteration_completed"`  → appended to the iterations array (without
///   the `kind` field).
/// - `"summary"`              → run-level aggregates; merged into the
///   summary.
///
/// Returns `(algorithm, summary_object, iterations_array)`. The `algorithm`
/// string is taken from the `algorithm` field of the `started` event and
/// falls back to `"unknown"` when missing.
pub fn parse_algorithm_trace_jsonl(
    text: &str,
) -> Result<(String, serde_json::Value, serde_json::Value), String> {
    let mut summary = serde_json::Map::new();
    let mut iterations: Vec<serde_json::Value> = Vec::new();

    for (line_no, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| format!("line {}: invalid JSON: {e}", line_no + 1))?;
        let kind = value
            .get("kind")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        match kind.as_str() {
            "started" | "summary" => {
                if let serde_json::Value::Object(map) = value {
                    for (k, v) in map {
                        if k == "kind" {
                            continue;
                        }
                        summary.insert(k, v);
                    }
                }
            }
            "iteration_completed" => {
                if let serde_json::Value::Object(mut map) = value {
                    map.remove("kind");
                    iterations.push(serde_json::Value::Object(map));
                }
            }
            _ => {}
        }
    }

    let algorithm = summary
        .get("algorithm")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    Ok((
        algorithm,
        serde_json::Value::Object(summary),
        serde_json::Value::Array(iterations),
    ))
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
            None,
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
            None,
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
            None,
            None,
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
            None,
            None,
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
