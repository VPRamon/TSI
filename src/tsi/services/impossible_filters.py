"""Impossible observation filtering logic (consolidated)."""

from __future__ import annotations

import pandas as pd


def check_filter_support(df: pd.DataFrame) -> bool:
    """
    Check if impossible observation filtering is supported.
    
    Requires visibility hours and at least one duration constraint column.
    
    Args:
        df: Source DataFrame
    
    Returns:
        True if filtering is supported, False otherwise
    """
    has_visibility = "total_visibility_hours" in df.columns
    has_duration_constraint = (
        "minObservationTimeInSec" in df.columns or "requested_hours" in df.columns
    )
    
    return has_visibility and has_duration_constraint


def compute_impossible_mask(
    df: pd.DataFrame, 
    tolerance_sec: float = 1.0
) -> pd.Series | None:
    """
    Compute mask for impossible observations.
    
    An observation is impossible if its required duration exceeds total visibility.
    Checks both minimum observation time and requested duration.
    
    Args:
        df: Source DataFrame
        tolerance_sec: Tolerance in seconds for comparison (default: 1 second)
    
    Returns:
        Boolean Series marking impossible observations, or None if not supported
    """
    if not check_filter_support(df):
        return None
    
    visibility_secs = df["total_visibility_hours"] * 3600.0
    impossible_conditions = []
    
    # Check minimum observation time constraint
    if "minObservationTimeInSec" in df.columns:
        min_duration_secs = df["minObservationTimeInSec"].fillna(0)
        impossible_conditions.append(
            (min_duration_secs - tolerance_sec > visibility_secs).fillna(False)
        )
    
    # Check requested duration constraint
    if "requested_hours" in df.columns:
        requested_secs = df["requested_hours"] * 3600.0
        impossible_conditions.append(
            (requested_secs - tolerance_sec > visibility_secs).fillna(False)
        )
    
    # An observation is impossible if ANY of the conditions is true
    if not impossible_conditions:
        return None
    
    impossible_mask = impossible_conditions[0]
    for condition in impossible_conditions[1:]:
        impossible_mask = impossible_mask | condition
    
    return impossible_mask


def filter_impossible_observations(
    df: pd.DataFrame, 
    filter_mode: str,
    tolerance_sec: float = 1.0
) -> pd.DataFrame:
    """
    Filter out impossible observations based on visibility constraints.
    
    An observation is considered impossible if either:
    - Minimum observation time exceeds total visibility hours
    - Requested duration exceeds total visibility hours
    
    Args:
        df: Source DataFrame
        filter_mode: One of "all" or "exclude_impossible"
        tolerance_sec: Tolerance in seconds for comparison
    
    Returns:
        Filtered DataFrame (view if no filtering, copy if filtered)
    """
    if filter_mode == "all":
        return df
    
    impossible_mask = compute_impossible_mask(df, tolerance_sec)
    
    if impossible_mask is None:
        return df
    
    # Return filtered copy
    return df[~impossible_mask].copy()


def apply_insights_filter(
    df: pd.DataFrame,
    filter_mode: str,
    impossible_mask: pd.Series | None = None,
    tolerance_sec: float = 1.0,
) -> pd.DataFrame:
    """
    Apply filtering based on user selection.
    
    Args:
        df: Source DataFrame
        filter_mode: Filter mode ('all' or 'exclude_impossible')
        impossible_mask: Pre-computed impossible observation mask (optional)
        tolerance_sec: Tolerance in seconds for comparison
    
    Returns:
        Filtered DataFrame
    """
    if filter_mode == "all":
        return df
    
    if filter_mode == "exclude_impossible":
        if impossible_mask is None:
            impossible_mask = compute_impossible_mask(df, tolerance_sec)
        
        if impossible_mask is not None:
            return df[~impossible_mask].copy()
    
    return df
