# Sky Map

Visualize targets in celestial coordinates (RA/Dec) with filtering by priority, status, and time.

## Data requirements

Required columns:
- `raInDeg`, `decInDeg`
- `priority`, `priority_bin`
- `requested_hours`
- `scheduled_flag`
- Optional for time filter: `scheduled_start_dt` (UTC timestamp)

## Controls

- Scheduling Status: All | Scheduled | Unscheduled
- Color by: Priority (priority_bin) | Status (scheduled_flag)
- Priority Range: min–max slider based on data
- Scheduled window (UTC): time range slider (only when scheduled_start_dt exists)
- Invert RA axis: toggle
- Reset filters: resets all page/state filters

## What you see

- RA/Dec scatter plot (Plotly)
  - Color: selected category
  - Size: `requested_hours`
  - Optional palette for original `priority_bin` labels
- Subset summary below the plot:
  - Observations shown (count)
  - RA coverage (max–min)
  - Dec coverage (max–min)
  - Share scheduled (percentage)

## Notes

- Priority range defaults to the dataset’s min/max (not hard‑coded to 0–10).
- When a scheduled window is set and Status=All, the view keeps unscheduled blocks and clips scheduled ones to the time window.
- If required columns are missing, the page will report a descriptive error.
