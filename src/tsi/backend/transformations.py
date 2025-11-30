"""
TSI Backend Transformations - Filtering and data transformations.

This module contains functions for filtering, validating,
and transforming schedule DataFrames.
"""

from __future__ import annotations

from datetime import datetime
from typing import Any, Literal, TYPE_CHECKING, cast

import pandas as pd
import polars as pl

if TYPE_CHECKING:
    pass

# Import Rust module
import tsi_rust


def filter_by_priority(
    df: pd.DataFrame | pl.DataFrame,
    min_priority: float = 0.0,
    max_priority: float = 100.0,
    use_pandas: bool = True,
) -> pd.DataFrame | pl.DataFrame:
    """
    Filter observations by priority range.

    Args:
        df: DataFrame to filter
        min_priority: Minimum priority (inclusive)
        max_priority: Maximum priority (inclusive)
        use_pandas: If True, return pandas DataFrame

    Returns:
        Filtered DataFrame

    Example:
        >>> high_priority = filter_by_priority(df, min_priority=15.0)
    """
    df_polars = _to_polars(df)
    result: pl.DataFrame = tsi_rust.py_filter_by_range(
        df_polars, "priority", min_priority, max_priority
    )
    return cast(pd.DataFrame, result.to_pandas()) if use_pandas else result


def filter_by_scheduled(
    df: pd.DataFrame | pl.DataFrame,
    filter_type: Literal["All", "Scheduled", "Unscheduled"] = "All",
    use_pandas: bool = True,
) -> pd.DataFrame | pl.DataFrame:
    """
    Filter observations by scheduling status.

    Args:
        df: DataFrame to filter
        filter_type: 'All', 'Scheduled', or 'Unscheduled'
        use_pandas: If True, return pandas DataFrame

    Returns:
        Filtered DataFrame

    Example:
        >>> unscheduled = filter_by_scheduled(df, "Unscheduled")
        >>> print(f"Unscheduled: {len(unscheduled)}")
    """
    df_polars = _to_polars(df)
    result: pl.DataFrame = tsi_rust.py_filter_by_scheduled(df_polars, filter_type)
    return cast(pd.DataFrame, result.to_pandas()) if use_pandas else result


def filter_dataframe(
    df: pd.DataFrame | pl.DataFrame,
    priority_min: float = 0.0,
    priority_max: float = 100.0,
    scheduled_filter: Literal["All", "Scheduled", "Unscheduled"] = "All",
    priority_bins: list[str] | None = None,
    block_ids: list[str] | None = None,
    use_pandas: bool = True,
) -> pd.DataFrame | pl.DataFrame:
    """
    Apply multiple filters to DataFrame.

    This is the canonical filtering function that combines all filter types.

    Args:
        df: DataFrame to filter
        priority_min: Minimum priority
        priority_max: Maximum priority
        scheduled_filter: Scheduling status filter
        priority_bins: List of priority bins to include
        block_ids: List of scheduling block IDs to include
        use_pandas: If True, return pandas DataFrame

    Returns:
        Filtered DataFrame

    Example:
        >>> filtered = filter_dataframe(
        ...     df,
        ...     priority_min=10.0,
        ...     priority_max=20.0,
        ...     scheduled_filter="Scheduled",
        ...     priority_bins=["High", "Very High"]
        ... )
    """
    df_polars = _to_polars(df)
    result: pl.DataFrame = tsi_rust.py_filter_dataframe(
        df_polars, priority_min, priority_max, scheduled_filter, priority_bins, block_ids
    )
    return cast(pd.DataFrame, result.to_pandas()) if use_pandas else result


def remove_duplicates(
    df: pd.DataFrame | pl.DataFrame,
    subset: list[str] | None = None,
    keep: Literal["first", "last", "none"] = "first",
    use_pandas: bool = True,
) -> pd.DataFrame | pl.DataFrame:
    """
    Remove duplicate rows from DataFrame.

    Args:
        df: DataFrame to clean
        subset: Column names to consider for duplicates (None = all columns)
        keep: Which duplicates to keep ('first', 'last', 'none')
        use_pandas: If True, return pandas DataFrame

    Returns:
        DataFrame with duplicates removed

    Example:
        >>> clean_df = remove_duplicates(df, subset=["schedulingBlockId"])
    """
    df_polars = _to_polars(df)
    result: pl.DataFrame = tsi_rust.py_remove_duplicates(df_polars, subset, keep)
    return cast(pd.DataFrame, result.to_pandas()) if use_pandas else result


def remove_missing_coordinates(
    df: pd.DataFrame | pl.DataFrame,
    use_pandas: bool = True,
) -> pd.DataFrame | pl.DataFrame:
    """
    Remove observations with missing RA or Dec coordinates.

    Args:
        df: DataFrame to clean
        use_pandas: If True, return pandas DataFrame

    Returns:
        DataFrame with complete coordinates only

    Example:
        >>> valid_coords = remove_missing_coordinates(df)
    """
    df_polars = _to_polars(df)
    result: pl.DataFrame = tsi_rust.py_remove_missing_coordinates(df_polars)
    return cast(pd.DataFrame, result.to_pandas()) if use_pandas else result


def validate_dataframe(df: pd.DataFrame | pl.DataFrame) -> tuple[bool, list[str]]:
    """
    Validate DataFrame data quality (coordinates, priorities, etc.).

    Args:
        df: DataFrame to validate

    Returns:
        Tuple of (is_valid, list of issues)

    Example:
        >>> is_valid, issues = validate_dataframe(df)
        >>> if not is_valid:
        ...     for issue in issues:
        ...         print(f"Warning: {issue}")
    """
    df_polars = _to_polars(df)
    return cast(tuple[bool, list[str]], tsi_rust.py_validate_dataframe(df_polars))


# ===== Time Conversions =====


def mjd_to_datetime(mjd: float) -> datetime:
    """
    Convert Modified Julian Date to Python datetime object.

    Args:
        mjd: Modified Julian Date value

    Returns:
        Python datetime object with UTC timezone

    Example:
        >>> dt = mjd_to_datetime(59580.5)
        >>> print(dt)  # 2022-01-01 12:00:00+00:00
    """
    return cast(datetime, tsi_rust.mjd_to_datetime(mjd))


def datetime_to_mjd(dt: datetime) -> float:
    """
    Convert Python datetime object to Modified Julian Date.

    Args:
        dt: Python datetime object (must have timezone info)

    Returns:
        Modified Julian Date value

    Example:
        >>> from datetime import datetime, timezone
        >>> dt = datetime(2022, 1, 1, 12, 0, 0, tzinfo=timezone.utc)
        >>> mjd = datetime_to_mjd(dt)
        >>> print(mjd)  # 59580.5
    """
    return cast(float, tsi_rust.datetime_to_mjd(dt))


def parse_visibility_periods(
    periods: list[dict[str, Any]],
) -> list[tuple[datetime, datetime]]:
    """
    Parse visibility periods from list of dicts to datetime tuples.

    Args:
        periods: List of dicts with 'start' and 'end' datetime strings

    Returns:
        List of (start_datetime, end_datetime) tuples

    Example:
        >>> periods = [{"start": "2022-01-01T00:00:00Z", "end": "2022-01-01T06:00:00Z"}]
        >>> parsed = parse_visibility_periods(periods)
        >>> print(parsed[0][0])  # datetime object
    """
    return cast(list[tuple[datetime, datetime]], tsi_rust.parse_visibility_periods(periods))


def _to_polars(df: pd.DataFrame | pl.DataFrame) -> pl.DataFrame:
    """Convert DataFrame to Polars if needed."""
    if isinstance(df, pd.DataFrame):
        return pl.from_pandas(df)
    return df
