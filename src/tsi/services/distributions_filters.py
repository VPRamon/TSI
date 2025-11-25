"""Distribution data filtering services."""

from __future__ import annotations

import pandas as pd


def filter_impossible_observations(df: pd.DataFrame, filter_mode: str) -> pd.DataFrame:
    """
    Filter out impossible observations based on visibility constraints.

    An observation is considered impossible if either:
    - Minimum observation time exceeds total visibility hours
    - Requested duration exceeds total visibility hours

    Args:
        df: Source DataFrame
        filter_mode: One of "all" or "exclude_impossible"

    Returns:
        Filtered DataFrame (view if no filtering, copy if filtered)
    """
    if filter_mode == "all":
        return df

    impossible_mask = compute_impossible_mask(df)
    
    if impossible_mask is None:
        return df

    # Return filtered copy
    return df[~impossible_mask].copy()


def compute_impossible_mask(df: pd.DataFrame) -> pd.Series | None:
    """
    Compute boolean mask identifying impossible observations.

    Args:
        df: Source DataFrame

    Returns:
        Boolean Series where True indicates impossible observations, or None if not applicable
    """
    if not check_filter_support(df):
        return None

    TOLERANCE_SEC = 1
    visibility_secs = df["total_visibility_hours"] * 3600.0

    # Check both minimum observation time and requested duration
    impossible_conditions = []

    if "minObservationTimeInSec" in df.columns:
        min_duration_secs = df["minObservationTimeInSec"].fillna(0)
        impossible_conditions.append(
            (min_duration_secs - TOLERANCE_SEC > visibility_secs).fillna(False)
        )

    if "requested_hours" in df.columns:
        requested_secs = df["requested_hours"] * 3600.0
        impossible_conditions.append(
            (requested_secs - TOLERANCE_SEC > visibility_secs).fillna(False)
        )

    # An observation is impossible if ANY of the conditions is true
    if not impossible_conditions:
        return None

    impossible_mask = impossible_conditions[0]
    for condition in impossible_conditions[1:]:
        impossible_mask = impossible_mask | condition

    return impossible_mask


def check_filter_support(df: pd.DataFrame) -> bool:
    """
    Check if filtering by impossible observations is supported.

    Args:
        df: The DataFrame to check

    Returns:
        True if the necessary columns exist for filtering
    """
    return "total_visibility_hours" in df.columns and (
        "minObservationTimeInSec" in df.columns or "requested_hours" in df.columns
    )
