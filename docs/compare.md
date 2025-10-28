# Compare Schedules

Upload a second schedule to compare against the one currently loaded in the app.

## Workflow

1) Load a base schedule on the landing page (CSV or JSON).
2) Open Compare and upload a comparison `schedule.json` (optional: `possible_periods.json`).
3) The comparison schedule is processed and normalized with the same pipeline as the base one.

## Matching and validation

- Block IDs are matched via `schedulingBlockId`.
- The app reports discrepancies:
  - Blocks only in base
  - Blocks only in comparison
- Comparison proceeds on the intersection of block IDs ("common blocks").

## What you see

Summary tables
- Priority & Scheduling Metrics (per schedule):
  - Scheduled Blocks, Total Priority Sum, Mean/Median Priority, Newly Scheduled/Unscheduled
  - Delta labels show percent change (green/red)
- Time Metrics (when `requested_hours` is available):
  - Total/Mean/Median planned time
  - Gaps between observations (count, mean/median hours) derived from `scheduled_start_dt`/`scheduled_stop_dt`
  - Gap deltas use inverse coloring (more gaps = red)

Visualizations
- Priority Distribution Comparison (overlaid histograms)
- Scheduling Status Breakdown (grouped bars)
- Scheduling Changes (two histograms: newly scheduled vs newly unscheduled priorities)
- Planned Time Distribution (box plots)

Details
- Expandable tables listing newly scheduled and newly unscheduled blocks; include target names and MJD periods when available.

## Notes

- Comparison inputs are cached to avoid reprocessing when uploading the same file again.
- If there are no common blocks, the page halts with a clear message.
