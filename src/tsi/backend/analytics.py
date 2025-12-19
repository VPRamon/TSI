"""
TSI Backend Analytics - Metrics and scheduling analytics.

This module contains functions for computing schedule metrics,
finding conflicts, and optimization routines.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, Any, cast

import pandas as pd
import polars as pl

if TYPE_CHECKING:
    pass

# Import Rust module
import tsi_rust


def compute_metrics(schedule_id: int) -> dict[str, Any]:
    """
    Compute comprehensive scheduling metrics from pre-computed database analytics.

    This function retrieves pre-computed summary analytics from the database,
    which is much faster than recomputing from a DataFrame.

    Args:
        schedule_id: ID of the schedule to get metrics for

    Returns:
        Dictionary with computed metrics including:
        - total_observations: count of scheduling blocks
        - scheduled_count: number of scheduled observations
        - mean_priority, median_priority: priority statistics
        - scheduling_rate: percentage of scheduled blocks (0.0 to 1.0)

    Example:
        >>> summary = compute_metrics(schedule_id=1)
        >>> print(f"Scheduled: {summary['scheduling_rate']:.1%}")

    Note:
        Requires py_populate_summary_analytics() to have been called first.
        This is automatically done during py_store_schedule() if populate_analytics=True.
    """
    summary = tsi_rust.py_get_schedule_summary(schedule_id)
    if summary is None:
        raise ValueError(
            f"No analytics data found for schedule {schedule_id}. Run py_populate_summary_analytics() first."
        )

    # Convert to dict with field mapping
    return {
        "total_observations": summary.total_blocks,
        "scheduled_count": summary.scheduled_blocks,
        "unscheduled_count": summary.unscheduled_blocks,
        "impossible_count": summary.impossible_blocks,
        "scheduling_rate": summary.scheduling_rate,
        "mean_priority": summary.priority_mean or 0.0,
        "median_priority": summary.priority_median or 0.0,
        "mean_priority_scheduled": summary.priority_scheduled_mean or 0.0,
        "mean_priority_unscheduled": summary.priority_unscheduled_mean or 0.0,
        "total_visibility_hours": summary.visibility_total_hours,
        "mean_requested_hours": summary.requested_mean_hours or 0.0,
    }


def get_top_observations(
    df: pd.DataFrame | pl.DataFrame,
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
    df_polars = _to_polars(df)
    result: pl.DataFrame = tsi_rust.py_get_top_observations(df_polars, by, n)
    return cast(pd.DataFrame, result.to_pandas())


def find_conflicts(df: pd.DataFrame | pl.DataFrame) -> pd.DataFrame:
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
    df_polars = _to_polars(df)
    result = tsi_rust.py_find_conflicts(df_polars)

    # Rust backend may return list or DataFrame depending on version
    if isinstance(result, list):
        if not result:
            return pd.DataFrame()
        return pd.DataFrame(result)

    result_df: pl.DataFrame = result
    return cast(pd.DataFrame, result_df.to_pandas())


def greedy_schedule(
    df: pd.DataFrame | pl.DataFrame,
    max_iterations: int = 1000,
) -> dict[str, Any]:
    """
    Run greedy scheduling optimization.

    Args:
        df: DataFrame with observations to schedule
        max_iterations: Maximum optimization iterations

    Returns:
        Dictionary with optimization results (selected_observations,
        total_duration, iterations_run, etc.)

    Example:
        >>> result = greedy_schedule(df, max_iterations=500)
        >>> print(f"Selected {len(result['selected_ids'])} observations")
    """
    df_polars = _to_polars(df)
    opt_result = tsi_rust.py_greedy_schedule(df_polars, max_iterations)
    return cast(dict[str, Any], opt_result.to_dict())


def compute_priority_range(df: pd.DataFrame | pl.DataFrame) -> tuple[int, int]:
    """
    Compute the range of priority values in the DataFrame.

    Args:
        df: DataFrame with schedule data

    Returns:
        Tuple of (min_priority, max_priority)

    Example:
        >>> pmin, pmax = compute_priority_range(df)
        >>> print(f"Priorities range from {pmin} to {pmax}")
    """
    df_polars = _to_polars(df)

    if "priority" not in df_polars.columns:
        return (1, 1)

    values = df_polars["priority"].drop_nulls()
    if len(values) == 0:
        return (1, 1)

    min_val = values.min()
    max_val = values.max()
    # Cast to int - Polars min/max returns numeric types for numeric columns
    return (
        int(min_val) if min_val is not None else 1,  # type: ignore[arg-type]
        int(max_val) if max_val is not None else 1,  # type: ignore[arg-type]
    )


def _to_polars(df: pd.DataFrame | pl.DataFrame) -> pl.DataFrame:
    """Convert DataFrame to Polars if needed."""
    if isinstance(df, pd.DataFrame):
        return pl.from_pandas(df)
    return df
