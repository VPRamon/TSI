"""Insights page data filtering services."""

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


def compute_impossible_mask(df: pd.DataFrame, tolerance_sec: float = 1.0) -> pd.Series | None:
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


def apply_insights_filter(
    df: pd.DataFrame,
    filter_mode: str,
    impossible_mask: pd.Series | None = None,
) -> pd.DataFrame:
    """
    Apply filtering based on user selection.
    
    Args:
        df: Source DataFrame
        filter_mode: Filter mode ('all' or 'exclude_impossible')
        impossible_mask: Pre-computed impossible observation mask
        
    Returns:
        Filtered DataFrame
    """
    if filter_mode == "all":
        return df
    
    if filter_mode == "exclude_impossible" and impossible_mask is not None:
        return df[~impossible_mask]
    
    return df
