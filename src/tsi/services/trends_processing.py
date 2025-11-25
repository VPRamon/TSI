"""Scheduling trends page data processing services."""

from __future__ import annotations

import pandas as pd


def validate_required_columns(df: pd.DataFrame) -> tuple[bool, list[str]]:
    """
    Validate that required columns exist for trends analysis.
    
    Args:
        df: Source DataFrame
        
    Returns:
        Tuple of (is_valid, missing_columns)
    """
    required_cols = ["priority", "total_visibility_hours", "requested_hours", "scheduled_flag"]
    missing_cols = [col for col in required_cols if col not in df.columns]

    return len(missing_cols) == 0, missing_cols


def apply_trends_filters(
    df: pd.DataFrame,
    vis_range: tuple[float, float],
    time_range: tuple[float, float],
    selected_priorities: list,
) -> pd.DataFrame:
    """
    Apply filters to DataFrame for trends analysis.
    
    Args:
        df: Source DataFrame
        vis_range: (min, max) visibility range in hours
        time_range: (min, max) requested time range in hours
        selected_priorities: List of priority values to include
        
    Returns:
        Filtered DataFrame
    """
    return df[
        (df["total_visibility_hours"] >= vis_range[0])
        & (df["total_visibility_hours"] <= vis_range[1])
        & (df["requested_hours"] >= time_range[0])
        & (df["requested_hours"] <= time_range[1])
        & (df["priority"].isin(selected_priorities))
    ].copy()
