"""Compare schedules page data processing services."""

from __future__ import annotations

import numpy as np
import pandas as pd


def calculate_observation_gaps(df: pd.DataFrame) -> tuple[int, float, float]:
    """
    Calculate gaps statistics between scheduled observations (vectorized).

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
    valid_df = df.dropna(subset=["scheduled_start_dt", "scheduled_stop_dt"])

    if len(valid_df) <= 1:
        return 0, 0.0, 0.0

    # Sort by start time and reset index for clean indexing
    sorted_df = valid_df.sort_values("scheduled_start_dt").reset_index(drop=True)

    # Vectorized gap calculation:
    # Gap exists when next observation starts after current one ends
    current_ends = sorted_df["scheduled_stop_dt"].iloc[:-1].values
    next_starts = sorted_df["scheduled_start_dt"].iloc[1:].values

    # Calculate time differences in nanoseconds, then convert to hours
    time_diffs = (next_starts - current_ends).astype('timedelta64[ns]')
    gap_hours = time_diffs.astype(float) / (3600 * 1e9)  # Convert ns to hours

    # Only keep positive gaps (where next starts after current ends)
    positive_gaps = gap_hours[gap_hours > 0]

    num_gaps = len(positive_gaps)

    if num_gaps == 0:
        return 0, 0.0, 0.0

    mean_gap = float(np.mean(positive_gaps))
    median_gap = float(np.median(positive_gaps))

    return num_gaps, mean_gap, median_gap

