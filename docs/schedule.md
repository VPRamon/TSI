# Scheduled Timeline

Monthly view of scheduled observations with optional nighttime (dark) vs daytime overlays.

## Data requirements

- `scheduled_flag`
- `scheduled_start_dt`, `scheduled_stop_dt` (UTC timestamps)
- `priority`
- Recommended: `requested_hours`, `total_visibility_hours`, `num_visibility_periods`, `raInDeg`, `decInDeg`
- Optional overlay: dark periods DataFrame with `start_dt`, `stop_dt`, `duration_hours`, `months`

## What you see

- Timeline by month (Plotly):
  - Each month is a row; observations are rectangles clipped to that month’s days (1–31)
  - Color encodes priority on a Viridis scale
  - If dark periods are loaded:
    - Nighttime (observable) intervals shown in bluish bands
    - Daytime (non‑observable) intervals shown in light yellow bands
    - Both are toggleable in the legend
- Observation Details table (searchable, filterable):
  - Filters: Search by ID, Month selector, Minimum priority
  - Columns include Month/Day, Start/End times, Priority, Duration, optional RA/Dec and visibility stats
  - Download as CSV
- Metrics:
  - Scheduled blocks, Total hours, Average duration, Months covered
  - Time range caption (UTC)

## Behavior and constraints

- If the dataset contains no scheduled rows or missing datetimes, the page shows guidance messages.
- Observations spanning multiple months are split and rendered in the corresponding month rows.
