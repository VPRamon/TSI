# Visibility Map & Schedule

Histogram showing how many targets are visible over time. Optimized for large datasets.

## Data requirements

- `total_visibility_hours`, `num_visibility_periods`
- `priority`, `priority_bin`
- `scheduled_flag`

## Controls

Sidebar
- Priority Range: min–max slider (defaults to data range)
- Reset filters

Main panel (expander "Histogram Settings")
- Bin Size Mode: Number of bins | Fixed duration
  - Number of bins: integer [10, 500] (default 100)
  - Fixed duration: width + unit (Minutes | Hours | Days)

## What you see

- Visibility Histogram (Plotly):
  - X: time (UTC)
  - Y: count of targets visible in each time bin
  - Color intensity encodes density (darker = more targets)
- Metrics:
  - Total Blocks, Filtered Blocks, Scheduled, Avg Visibility Periods

## Notes

- Data is filtered by the selected priority range using `tsi.services.loaders.get_filtered_dataframe`.
- You can tune binning either by specifying a bin count or a fixed bin width in minutes/hours/days.
