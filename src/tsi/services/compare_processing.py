"""Compare schedules page data processing services."""

from __future__ import annotations

import pandas as pd


def calculate_observation_gaps(df: pd.DataFrame) -> tuple[int, float, float]:
    """
    Calculate gaps statistics between scheduled observations.
    
    A gap is defined as any time period between consecutive scheduled observations.
    
    Args:
        df: DataFrame with scheduled observations containing scheduled_start_dt and scheduled_stop_dt
        
    Returns:
        Tuple of (num_gaps, mean_gap_hours, median_gap_hours)
    """
    if len(df) <= 1:  # Need at least 2 observations to have a gap
        return 0, 0.0, 0.0
    
    # Check if we have the necessary datetime columns
    if "scheduled_start_dt" not in df.columns or "scheduled_stop_dt" not in df.columns:
        return 0, 0.0, 0.0
    
    # Filter out rows with null datetime values
    valid_df = df.dropna(subset=["scheduled_start_dt", "scheduled_stop_dt"]).copy()
    
    if len(valid_df) <= 1:
        return 0, 0.0, 0.0
    
    # Sort by start time
    sorted_df = valid_df.sort_values("scheduled_start_dt").reset_index(drop=True)
    
    # Calculate gaps and their durations
    gaps = 0
    gap_durations = []  # in hours
    
    for i in range(len(sorted_df) - 1):
        current_end = sorted_df.iloc[i]["scheduled_stop_dt"]
        next_start = sorted_df.iloc[i + 1]["scheduled_start_dt"]
        
        # If there's a gap between observations
        if next_start > current_end:
            gaps += 1
            gap_duration_hours = (next_start - current_end).total_seconds() / 3600
            gap_durations.append(gap_duration_hours)
    
    # Calculate mean and median
    mean_gap = sum(gap_durations) / len(gap_durations) if gap_durations else 0.0
    median_gap = sorted(gap_durations)[len(gap_durations) // 2] if gap_durations else 0.0
    
    return gaps, mean_gap, median_gap
