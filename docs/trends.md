# Scheduling Trends

Analyze scheduling probability as a function of visibility, priority, and requested time, including interactions via a logistic model.

## Data requirements

Required: `priority`, `total_visibility_hours`, `requested_hours`, `scheduled_flag`

## Controls (sidebar)

- Data Filters
  - Visibility range (hours)
  - Requested time range (hours)
  - Priority levels (multi-select)
- Plot Configuration
  - Plot library: Altair | Plotly
  - Number of bins (for hist/heatmap): 5–20
  - Smoothing bandwidth (LOESS-like): 0.1–0.6
- Logistic Model
  - Exclude visibility = 0 for model (checkbox)
  - Class weighting: balanced | None
  - Fixed requested time for prediction (slider)

## Sections

1) Empirical proportions by priority
- Bar chart of empirical scheduling rates by priority. Tooltip includes sample counts.

2) Smoothed curves (trends)
- Visibility → Scheduling rate
- Requested time → Scheduling rate
- Weighted moving-average smoothing; library selectable.

3) Heatmap: Visibility × Priority
- 2D heatmap of mean empirical scheduling rate across visibility (X) and priority (Y).

4) Logistic model with interactions
- Model metrics: n samples, n scheduled, accuracy, AUC
- Predicted probability vs visibility (curves by priority) at a fixed requested time
- Expander with model details: features, interactions, preprocessing, weighting

## Notes

- Heavy computations are cached across interactions for a responsive UI.
- If too few rows remain after filtering (< 10), the page asks to widen filters.
