"""
TSI Backend Transformations - Filtering and data transformations.

This module contains functions for filtering, validating,
and transforming schedule DataFrames.
"""

from __future__ import annotations

from datetime import datetime, timezone
from io import StringIO
from typing import TYPE_CHECKING, Any, Literal, cast

import pandas as pd

if TYPE_CHECKING:
    pass

# Import Rust module
import tsi_rust

# Local Python utils replacing small legacy Rust helpers
from . import utils as _utils


def _df_to_json(df: pd.DataFrame) -> str:
    """Convert pandas DataFrame to JSON string for Rust."""
    return df.to_json(orient="records")


def _json_to_df(json_str: str) -> pd.DataFrame:
    """Convert JSON string from Rust to pandas DataFrame."""
    return pd.read_json(StringIO(json_str), orient="records")


def filter_by_priority(
    df: pd.DataFrame,
    min_priority: float = 0.0,
    max_priority: float = 100.0,
    use_pandas: bool = True,
) -> pd.DataFrame:
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
    json_str = _df_to_json(df)
    result_json: str = _utils.py_filter_by_range(
        json_str, "priority", min_priority, max_priority
    )
    return _json_to_df(result_json)


def filter_by_scheduled(
    df: pd.DataFrame,
    filter_type: Literal["All", "Scheduled", "Unscheduled"] = "All",
    use_pandas: bool = True,
) -> pd.DataFrame:
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
    json_str = _df_to_json(df)
    result_json: str = _utils.py_filter_by_scheduled(json_str, filter_type)
    return _json_to_df(result_json)


def filter_dataframe(
    df: pd.DataFrame,
    priority_min: float = 0.0,
    priority_max: float = 100.0,
    scheduled_filter: Literal["All", "Scheduled", "Unscheduled"] = "All",
    priority_bins: list[str] | None = None,
    block_ids: list[str] | None = None,
    use_pandas: bool = True,
) -> pd.DataFrame:
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
    json_str = _df_to_json(df)
    result_json: str = _utils.py_filter_dataframe(
        json_str, priority_min, priority_max, scheduled_filter, priority_bins, block_ids
    )
    return _json_to_df(result_json)


def remove_duplicates(
    df: pd.DataFrame,
    subset: list[str] | None = None,
    keep: Literal["first", "last", "none"] = "first",
    use_pandas: bool = True,
) -> pd.DataFrame:
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
    json_str = _df_to_json(df)
    result_json: str = _utils.py_remove_duplicates(json_str, subset, keep)
    return _json_to_df(result_json)


def remove_missing_coordinates(
    df: pd.DataFrame,
    use_pandas: bool = True,
) -> pd.DataFrame:
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
    json_str = _df_to_json(df)
    result_json: str = _utils.py_remove_missing_coordinates(json_str)
    return _json_to_df(result_json)


def validate_dataframe(df: pd.DataFrame) -> tuple[bool, list[str]]:
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
    json_str = _df_to_json(df)
    return cast(tuple[bool, list[str]], _utils.py_validate_dataframe(json_str))


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
    secs = (mjd - 40587.0) * 86400.0
    return datetime.fromtimestamp(secs, timezone.utc)


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
    tzinfo = dt.tzinfo
    if tzinfo is None:
        dt = dt.replace(tzinfo=timezone.utc)
    else:
        dt = dt.astimezone(timezone.utc)
    timestamp = dt.timestamp()
    return timestamp / 86400.0 + 40587.0


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
    return cast(list[tuple[datetime, datetime]], _utils.parse_visibility_periods(periods))
