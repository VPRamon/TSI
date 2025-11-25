"""Visibility schedule data processing services."""

from __future__ import annotations

import pandas as pd


def get_priority_range(df: pd.DataFrame) -> tuple[float, float]:
    """
    Calculate the priority range from the dataframe.

    Args:
        df: Source DataFrame

    Returns:
        Tuple of (min_priority, max_priority)
    """
    if "priority" in df.columns:
        priority_values = df["priority"].dropna()
        if not priority_values.empty:
            priority_min = float(priority_values.min())
            priority_max = float(priority_values.max())
            if priority_min == priority_max:
                priority_max = priority_min + 1.0
        else:
            priority_min, priority_max = 0.0, 10.0
    else:
        priority_min, priority_max = 0.0, 10.0

    return priority_min, priority_max


def get_all_block_ids(df: pd.DataFrame) -> list:
    """
    Get sorted list of all block IDs from the dataframe.

    Args:
        df: Source DataFrame

    Returns:
        Sorted list of block IDs
    """
    return sorted(df["schedulingBlockId"].dropna().unique())


def compute_effective_priority_range(
    sidebar_range: tuple[float, float],
    settings_range: tuple[float, float],
) -> tuple[float, float]:
    """
    Compute the effective priority range by combining sidebar and settings filters.

    Takes the more restrictive range (intersection of both ranges).

    Args:
        sidebar_range: Priority range from sidebar
        settings_range: Priority range from histogram settings

    Returns:
        Effective priority range (min, max)
    """
    effective_min = max(sidebar_range[0], settings_range[0])
    effective_max = min(sidebar_range[1], settings_range[1])
    return (effective_min, effective_max)
