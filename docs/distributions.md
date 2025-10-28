# Distributions

Statistical visualizations of priority, visibility, requested time, elevation range, and scheduling status.

## Data requirements

- `priority`, `requested_hours`, `total_visibility_hours`, `elevation_range_deg`, `scheduled_flag`
- Optional (for the "Exclude impossible" filter): `minObservationTimeInSec`, `total_visibility_hours`

## Controls

- Filter mode (top-right):
  - All observations (default)
  - Only possible observations: excludes rows where `minObservationTimeInSec` > `total_visibility_hours` × 3600 (with 1s tolerance)
- Priority bins: fixed in code (20) for histogram generation

## What you see

- Priority Distribution (histogram)
- Visibility Hours (histogram)
- Elevation Constraint Range (histogram of `elevation_range_deg`)
- Requested Duration (histogram of `requested_hours`)
- Scheduling Status (bar chart of scheduled vs unscheduled counts)
- Priority Comparison by Scheduling Status (violin plot)
- Statistical Summary:
  - Priority: mean, median, std
  - Visibility Hours: mean, median, total
  - Requested Hours: mean, median, total

## Notes

- The "Only possible" filter is available only if both `minObservationTimeInSec` and `total_visibility_hours` are present.
- Plots are built by `tsi.plots.distributions.build_figures` and rendered with Plotly.
