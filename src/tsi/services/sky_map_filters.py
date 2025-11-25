"""Sky Map data filtering and processing services."""

from __future__ import annotations

from collections.abc import Sequence
from datetime import datetime

import pandas as pd


def filter_dataframe(
    df: pd.DataFrame,
    priority_range: tuple[float, float],
    scheduled_filter: str,
    selected_bins: Sequence[str],
    schedule_window: tuple[datetime, datetime] | None,
) -> pd.DataFrame:
    """
    Apply priority, bin, scheduled status and time filters to the dataframe.

    Args:
        df: Source DataFrame
        priority_range: Tuple of (min_priority, max_priority)
        scheduled_filter: One of "All", "Scheduled", "Unscheduled"
        selected_bins: Sequence of priority bins to include
        schedule_window: Optional tuple of (start_datetime, end_datetime)

    Returns:
        Filtered DataFrame copy
    """
    # Use boolean indexing without copying until the final result
    mask = (df["priority"] >= priority_range[0]) & (df["priority"] <= priority_range[1])

    if selected_bins:
        mask &= df["priority_bin"].isin(selected_bins)

    if scheduled_filter == "Scheduled":
        mask &= df["scheduled_flag"]
    elif scheduled_filter == "Unscheduled":
        mask &= ~df["scheduled_flag"]

    # Apply time window filter if specified
    if schedule_window:
        start_ts = to_utc_timestamp(schedule_window[0])
        end_ts = to_utc_timestamp(schedule_window[1])

        scheduled_mask = (
            df["scheduled_flag"]
            & df["scheduled_start_dt"].notna()
            & (df["scheduled_start_dt"] >= start_ts)
            & (df["scheduled_start_dt"] <= end_ts)
        )

        if scheduled_filter == "All":
            # Include all unscheduled + scheduled within window
            unscheduled_mask = ~df["scheduled_flag"]
            mask &= unscheduled_mask | scheduled_mask
        elif scheduled_filter == "Scheduled":
            mask &= scheduled_mask
        # For unscheduled, window doesn't apply - already filtered above

    # Only create copy at the end when returning filtered result
    return df[mask].copy()


def prepare_priority_bins(df: pd.DataFrame) -> tuple[pd.DataFrame, list[str]]:
    """
    Prepare priority bins column and return list of bins.

    Args:
        df: Source DataFrame

    Returns:
        Tuple of (modified DataFrame, list of priority bins)
    """
    # Convert priority_bin to string, handling NaN values
    if df["priority_bin"].dtype != "string":
        df = df.copy()  # Only copy when we need to modify
        df["priority_bin"] = df["priority_bin"].astype("string").fillna("No priority")

    priority_bins = df["priority_bin"].dropna().unique().tolist()
    return df, priority_bins


def build_palette(labels: Sequence[str]) -> dict:
    """
    Generate a simple color palette for categorical bins.

    Args:
        labels: Sequence of category labels

    Returns:
        Dictionary mapping labels to colors
    """
    base_colors = [
        "#1f77b4",
        "#ff7f0e",
        "#2ca02c",
        "#d62728",
        "#9467bd",
        "#8c564b",
        "#e377c2",
        "#7f7f7f",
        "#bcbd22",
        "#17becf",
    ]
    palette = {}
    for idx, label in enumerate(labels):
        palette[label] = base_colors[idx % len(base_colors)]
    return palette


def to_utc_timestamp(value: datetime) -> pd.Timestamp:
    """
    Convert naive or aware datetime to UTC pandas Timestamp.

    Args:
        value: Python datetime object

    Returns:
        UTC pandas Timestamp
    """
    ts = pd.Timestamp(value)
    if ts.tzinfo is None:
        ts = ts.tz_localize("UTC")
    else:
        ts = ts.tz_convert("UTC")
    return ts
