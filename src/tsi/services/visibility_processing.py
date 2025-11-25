"""Visibility schedule data processing services."""

from __future__ import annotations

import pandas as pd

from core.domain.priority import get_priority_range


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
