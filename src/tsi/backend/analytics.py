"""
TSI Backend Analytics - Metrics and scheduling analytics.

This module contains functions for computing schedule metrics,
finding conflicts, and optimization routines.
"""

from __future__ import annotations

from io import StringIO
from typing import TYPE_CHECKING, Any, cast

import pandas as pd

if TYPE_CHECKING:
    pass

# Import Rust module
import tsi_rust


def _df_to_json(df: pd.DataFrame) -> str:
    """Convert pandas DataFrame to JSON string for Rust."""
    return df.to_json(orient="records")


def _json_to_df(json_str: str) -> pd.DataFrame:
    """Convert JSON string from Rust to pandas DataFrame."""
    return pd.read_json(StringIO(json_str), orient="records")


def get_top_observations(
    df: pd.DataFrame,
    n: int = 10,
    by: str = "priority",
) -> pd.DataFrame:
    """
    Get top N observations sorted by specified column.

    Args:
        df: DataFrame with schedule data
        n: Number of top observations to return
        by: Column to sort by (default: 'priority')

    Returns:
        pandas DataFrame with top observations

    Example:
        >>> top = get_top_observations(df, n=5)
        >>> print(top[['schedulingBlockId', 'priority']])
    """
    json_str = _df_to_json(df)
    result_json: str = tsi_rust.py_get_top_observations(json_str, by, n)
    return _json_to_df(result_json)


def find_conflicts(df: pd.DataFrame) -> pd.DataFrame:
    """
    Find scheduling conflicts (overlapping observations).

    Args:
        df: DataFrame with schedule data

    Returns:
        pandas DataFrame with conflicts (observation pairs with overlaps)

    Example:
        >>> conflicts = find_conflicts(df)
        >>> if len(conflicts) > 0:
        >>>     print(f"Found {len(conflicts)} conflicts")
    """
    json_str = _df_to_json(df)
    result = tsi_rust.py_find_conflicts(json_str)

    # Rust backend returns list of conflict objects
    if isinstance(result, list):
        if not result:
            return pd.DataFrame()
        # Convert conflict objects to dicts
        conflict_dicts = [
            {
                "scheduling_block_id": c.scheduling_block_id,
                "priority": c.priority,
                "scheduled_start": c.scheduled_start,
                "scheduled_stop": c.scheduled_stop,
                "conflict_reasons": c.conflict_reasons,
            }
            for c in result
        ]
        return pd.DataFrame(conflict_dicts)

    return pd.DataFrame()
