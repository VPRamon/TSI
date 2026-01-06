"""
TSI Rust Backend - Python Integration Layer

This module provides the primary Python interface to the TSI Rust backend.
All backend functionality is exposed through this module.

Example:
    >>> from tsi_rust_api import TSIBackend
    >>>
    >>> # Initialize backend
    >>> backend = TSIBackend()
    >>>
    >>> # Load schedule data
    >>> df = backend.load_schedule("data/schedule.json")
    >>> print(f"Loaded {len(df)} observations")
    >>>
    >>> # Filter data
    >>> high_priority = backend.filter_by_priority(df, min_priority=15.0)
    >>> print(f"High priority observations: {len(high_priority)}")
"""

# mypy: disable-error-code="no-any-return"

from __future__ import annotations

import json
import logging
from datetime import datetime, timezone
from io import StringIO
from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal, cast

import pandas as pd

from tsi.exceptions import ServerError

if TYPE_CHECKING:
    pass

logger = logging.getLogger(__name__)

# Import Rust backend module
try:
    import tsi_rust
except ImportError as e:
    raise ServerError(
        "tsi_rust module not found. Please compile the Rust backend with: maturin develop --release",
        details={"install_command": "maturin develop --release"},
    ) from e


# ===== Internal Helpers =====


def _df_to_json(df: pd.DataFrame) -> str:
    """Convert pandas DataFrame to JSON string for Rust."""
    return df.to_json(orient="records")


def _json_to_df(json_str: str) -> pd.DataFrame:
    """Convert JSON string from Rust to pandas DataFrame."""
    return pd.read_json(StringIO(json_str), orient="records")


# ===== Data Loading Functions =====


def load_schedule_file(
    path: str | Path,
    format: Literal["auto", "json"] = "auto",
    use_pandas: bool = True,
) -> pd.DataFrame:
    """
    Load schedule data from JSON file.

    Args:
        path: Path to the schedule file
        format: File format ('auto' or 'json'). Auto-detects from extension.
        use_pandas: If True, return pandas DataFrame. If False, return Polars DataFrame.

    Returns:
        DataFrame with scheduling blocks and derived columns

    Example:
        >>> df = load_schedule_file("data/schedule.json")
        >>> print(df.columns)
    """
    path = Path(path)

    if format == "auto":
        if path.suffix != ".json":
            raise ValueError(f"Only JSON files are supported. Got: {path.suffix}")
        format = "json"

    if format == "json":
        # Try Rust's JSON string loading via reading file first
        content = path.read_text()
        return load_schedule_from_string(content, format="json", use_pandas=use_pandas)
    else:
        raise ValueError(f"Unknown format: {format}")


def load_schedule_from_string(
    content: str,
    format: Literal["json"] = "json",
    use_pandas: bool = True,
) -> pd.DataFrame:
    """
    Load schedule data from JSON string content.

    Args:
        content: JSON string content
        format: Format of the content ('json' only)
        use_pandas: If True, return pandas DataFrame. If False, return Polars DataFrame.

    Returns:
        DataFrame with scheduling blocks

    Example:
        >>> json_str = '{"SchedulingBlock": [...]}'
        >>> df = load_schedule_from_string(json_str, format="json")
    """
    if format != "json":
        raise ValueError(f"Only JSON format is supported. Got: {format}")

    # Parse JSON and extract scheduling blocks
    data = json.loads(content)
    # Handle different JSON structures
    if "SchedulingBlock" in data:
        blocks = data["SchedulingBlock"]
    elif "schedulingBlocks" in data:
        blocks = data["schedulingBlocks"]
    else:
        blocks = data if isinstance(data, list) else [data]

    df_pandas = pd.DataFrame(blocks)
    return df_pandas


def load_dark_periods(path: str | Path) -> pd.DataFrame:
    """
    Load dark periods data from JSON file.

    Args:
        path: Path to dark_periods.json file

    Returns:
        pandas DataFrame with columns: start_dt, stop_dt, start_mjd, stop_mjd,
        duration_hours, months

    Example:
        >>> df = load_dark_periods("data/dark_periods.json")
        >>> print(f"Loaded {len(df)} dark periods")
    """
    path = Path(path)
    with open(path) as f:
        data = json.load(f)

    # Handle different JSON structures
    if "dark_periods" in data:
        periods = data["dark_periods"]
    elif isinstance(data, list):
        periods = data
    else:
        periods = [data]

    return pd.DataFrame(periods)


# ===== Analytics Functions =====


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
    # Sort by the specified column in descending order and take top n
    if by not in df.columns:
        raise ValueError(f"Column '{by}' not found in DataFrame")
    
    sorted_df = df.sort_values(by=by, ascending=False)
    return sorted_df.head(n).reset_index(drop=True)


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
    # Simple implementation: check for overlapping time ranges
    conflicts = []
    
    # Check if we have the required columns
    if "scheduled_start" not in df.columns or "scheduled_stop" not in df.columns:
        # Return empty DataFrame if columns don't exist
        return pd.DataFrame(
            columns=["scheduling_block_id", "priority", "scheduled_start", "scheduled_stop", "conflict_reasons"]
        )
    
    # Check for overlaps
    for i in range(len(df)):
        for j in range(i + 1, len(df)):
            row_i = df.iloc[i]
            row_j = df.iloc[j]
            
            start_i = row_i.get("scheduled_start")
            stop_i = row_i.get("scheduled_stop")
            start_j = row_j.get("scheduled_start")
            stop_j = row_j.get("scheduled_stop")
            
            # Skip if any times are missing
            if pd.isna(start_i) or pd.isna(stop_i) or pd.isna(start_j) or pd.isna(stop_j):
                continue
            
            # Check for overlap
            if start_i < stop_j and start_j < stop_i:
                conflicts.append(
                    {
                        "scheduling_block_id": row_i.get("schedulingBlockId", "unknown"),
                        "priority": row_i.get("priority", 0.0),
                        "scheduled_start": start_i,
                        "scheduled_stop": stop_i,
                        "conflict_reasons": [f"Overlaps with block {row_j.get('schedulingBlockId', 'unknown')}"],
                    }
                )
    
    return pd.DataFrame(conflicts)


# ===== Filter/Transform Functions =====


def _filter_by_range(json_str: str, column: str, min_val: float, max_val: float) -> str:
    """Internal: Filter records by numeric range."""
    records = json.loads(json_str)
    filtered = []
    for r in records:
        v = r.get(column)
        try:
            fv = float(v) if v is not None else None
        except (TypeError, ValueError):
            fv = None
        if fv is not None and min_val <= fv <= max_val:
            filtered.append(r)
    return json.dumps(filtered)


def _filter_by_scheduled_internal(json_str: str, filter_type: str) -> str:
    """Internal: Filter by scheduled status."""
    records = json.loads(json_str)
    if filter_type == "All":
        filtered = records
    elif filter_type == "Scheduled":
        filtered = [r for r in records if r.get("wasScheduled") is True]
    elif filter_type == "Unscheduled":
        filtered = [r for r in records if not r.get("wasScheduled", False)]
    else:
        filtered = records
    return json.dumps(filtered)


def _filter_dataframe_internal(
    json_str: str,
    priority_min: float,
    priority_max: float,
    scheduled_filter: str,
    priority_bins: list[str] | None = None,
    block_ids: list[str] | None = None,
) -> str:
    """Internal: Apply multiple filters."""
    records = json.loads(json_str)

    # Priority range
    filtered = []
    for r in records:
        try:
            p = float(r.get("priority")) if r.get("priority") is not None else None
        except (TypeError, ValueError):
            p = None
        if p is not None and priority_min <= p <= priority_max:
            filtered.append(r)

    # Scheduled filter
    if scheduled_filter == "Scheduled":
        filtered = [r for r in filtered if r.get("wasScheduled") is True]
    elif scheduled_filter == "Unscheduled":
        filtered = [r for r in filtered if not r.get("wasScheduled", False)]

    # Priority bins
    if priority_bins:
        filtered = [r for r in filtered if r.get("priorityBin") in priority_bins]

    # Block IDs
    if block_ids:
        filtered = [r for r in filtered if r.get("schedulingBlockId") in block_ids]

    return json.dumps(filtered)


def _remove_duplicates_internal(
    json_str: str, subset: list[str] | None = None, keep: str | None = "first"
) -> str:
    """Internal: Remove duplicate records."""
    records = json.loads(json_str)
    keep = keep or "first"

    def key_of(r: dict[str, Any]) -> str:
        if subset:
            parts = [str(r.get(c)) for c in subset]
            return "|".join(parts)
        return json.dumps(r, sort_keys=True)

    if keep == "first":
        seen = set()
        out = []
        for r in records:
            k = key_of(r)
            if k not in seen:
                seen.add(k)
                out.append(r)
        return json.dumps(out)

    if keep == "last":
        seen = set()
        out_rev = []
        for r in reversed(records):
            k = key_of(r)
            if k not in seen:
                seen.add(k)
                out_rev.append(r)
        return json.dumps(list(reversed(out_rev)))

    # keep == "none"
    counts: dict[str, int] = {}
    for r in records:
        counts[key_of(r)] = counts.get(key_of(r), 0) + 1
    out = [r for r in records if counts[key_of(r)] == 1]
    return json.dumps(out)


def _remove_missing_coordinates_internal(json_str: str) -> str:
    """Internal: Remove records with missing coordinates."""
    records = json.loads(json_str)
    filtered = []
    for r in records:
        try:
            has_ra = r.get("raDeg") is not None and float(r.get("raDeg")) is not None
            has_dec = r.get("decDeg") is not None and float(r.get("decDeg")) is not None
        except (TypeError, ValueError):
            has_ra = False
            has_dec = False
        if has_ra and has_dec:
            filtered.append(r)
    return json.dumps(filtered)


def _validate_dataframe_internal(json_str: str) -> tuple[bool, list[str]]:
    """Internal: Validate data quality."""
    records = json.loads(json_str)
    issues: list[str] = []

    missing_coords = 0
    for r in records:
        try:
            has_ra = r.get("raDeg") is not None and float(r.get("raDeg")) is not None
            has_dec = r.get("decDeg") is not None and float(r.get("decDeg")) is not None
        except (TypeError, ValueError):
            has_ra = False
            has_dec = False
        if not (has_ra and has_dec):
            missing_coords += 1

    if missing_coords > 0:
        issues.append(f"{missing_coords} observations with missing coordinates")

    invalid_priorities = 0
    for r in records:
        try:
            p = float(r.get("priority"))
            if p < 0.0 or p > 100.0:
                invalid_priorities += 1
        except (TypeError, ValueError):
            invalid_priorities += 1

    if invalid_priorities > 0:
        issues.append(f"{invalid_priorities} observations with invalid priorities")

    return (len(issues) == 0, issues)


def filter_by_priority_func(
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
        >>> high_priority = filter_by_priority_func(df, min_priority=15.0)
    """
    json_str = _df_to_json(df)
    result_json: str = _filter_by_range(json_str, "priority", min_priority, max_priority)
    return _json_to_df(result_json)


def filter_by_scheduled_func(
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
        >>> unscheduled = filter_by_scheduled_func(df, "Unscheduled")
        >>> print(f"Unscheduled: {len(unscheduled)}")
    """
    json_str = _df_to_json(df)
    result_json: str = _filter_by_scheduled_internal(json_str, filter_type)
    return _json_to_df(result_json)


def filter_dataframe_func(
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
        >>> filtered = filter_dataframe_func(
        ...     df,
        ...     priority_min=10.0,
        ...     priority_max=20.0,
        ...     scheduled_filter="Scheduled",
        ...     priority_bins=["High", "Very High"]
        ... )
    """
    json_str = _df_to_json(df)
    result_json: str = _filter_dataframe_internal(
        json_str, priority_min, priority_max, scheduled_filter, priority_bins, block_ids
    )
    return _json_to_df(result_json)


def remove_duplicates_func(
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
        >>> clean_df = remove_duplicates_func(df, subset=["schedulingBlockId"])
    """
    json_str = _df_to_json(df)
    result_json: str = _remove_duplicates_internal(json_str, subset, keep)
    return _json_to_df(result_json)


def remove_missing_coordinates_func(
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
        >>> valid_coords = remove_missing_coordinates_func(df)
    """
    json_str = _df_to_json(df)
    result_json: str = _remove_missing_coordinates_internal(json_str)
    return _json_to_df(result_json)


def validate_dataframe_func(df: pd.DataFrame) -> tuple[bool, list[str]]:
    """
    Validate DataFrame data quality (coordinates, priorities, etc.).

    Args:
        df: DataFrame to validate

    Returns:
        Tuple of (is_valid, list of issues)

    Example:
        >>> is_valid, issues = validate_dataframe_func(df)
        >>> if not is_valid:
        ...     for issue in issues:
        ...         print(f"Warning: {issue}")
    """
    json_str = _df_to_json(df)
    return _validate_dataframe_internal(json_str)


# ===== TSIBackend Class =====


class TSIBackend:
    """
    High-level Python interface to the TSI Rust backend.

    Provides ergonomic methods for loading, processing, and analyzing
    telescope scheduling data with automatic type conversions.

    The class is organized into logical method groups:
    - Data Loading: load_schedule, load_schedule_from_string
    - Preprocessing: validate_schedule
    - Analytics: get_top_observations, find_conflicts
    - Transformations: filter_by_priority, filter_by_scheduled, filter_dataframe,
                       remove_duplicates, remove_missing_coordinates, validate_dataframe
    - Time Conversions: mjd_to_datetime, datetime_to_mjd, parse_visibility_periods

    Example:
        >>> backend = TSIBackend()
        >>> df = backend.load_schedule("data/schedule.json")
    """

    def __init__(self, use_pandas: bool = True):
        """
        Initialize TSI backend.

        Args:
            use_pandas: If True, return pandas DataFrames. If False, return Polars DataFrames.
        """
        self.use_pandas = use_pandas

    # ===== Data Loading =====

    def load_schedule(
        self, path: str | Path, format: Literal["auto", "csv", "json"] = "auto"
    ) -> pd.DataFrame:
        """
        Load schedule data from CSV or JSON file.

        Args:
            path: Path to the schedule file
            format: File format ('auto', 'csv', or 'json'). Auto-detects from extension.

        Returns:
            DataFrame with scheduling blocks and derived columns

        Example:
            >>> df = backend.load_schedule("data/schedule.json")
            >>> print(df.columns)
        """
        return load_schedule_file(path, format=format, use_pandas=self.use_pandas)

    def load_schedule_from_string(
        self, content: str, format: Literal["csv", "json"] = "json"
    ) -> pd.DataFrame:
        """
        Load schedule data from string content.

        Args:
            content: JSON or CSV string content
            format: Format of the content ('csv' or 'json')

        Returns:
            DataFrame with scheduling blocks

        Example:
            >>> json_str = '{"schedulingBlocks": [...]}'
            >>> df = backend.load_schedule_from_string(json_str, format="json")
        """
        return load_schedule_from_string(content, format=format, use_pandas=self.use_pandas)

    # ===== Preprocessing =====

    def validate_schedule(self, df: pd.DataFrame) -> dict[str, Any]:
        """
        Validate schedule data structure and quality.

        Args:
            df: DataFrame to validate

        Returns:
            Dictionary with validation results (is_valid, warnings, errors)

        Example:
            >>> result = backend.validate_schedule(df)
            >>> if not result['is_valid']:
            ...     print(result['errors'])
        """
        df_polars = df
        validation_result = tsi_rust.py_validate_schedule(df_polars)
        return cast(dict[str, Any], validation_result.to_dict())

    # ===== Analytics & Algorithms =====

    def get_top_observations(
        self, df: pd.DataFrame, n: int = 10, by: str = "priority"
    ) -> pd.DataFrame:
        """
        Get top N observations sorted by specified column.

        Args:
            df: DataFrame with scheduling data
            n: Number of top observations to return
            by: Column to sort by (default: 'priority')

        Returns:
            DataFrame with top N observations

        Example:
            >>> top_10 = backend.get_top_observations(df, n=10)
            >>> print(top_10[['schedulingBlockId', 'priority']])
        """
        return get_top_observations(df, n=n, by=by)

    def find_conflicts(self, df: pd.DataFrame) -> pd.DataFrame:
        """
        Find scheduling conflicts (overlapping observations).

        Args:
            df: DataFrame with scheduling data

        Returns:
            DataFrame with conflicts (observation pairs with overlaps)

        Example:
            >>> conflicts = backend.find_conflicts(df)
            >>> print(f"Found {len(conflicts)} conflicts")
        """
        return find_conflicts(df)

    # ===== Transformations & Filtering =====

    def filter_by_priority(
        self,
        df: pd.DataFrame,
        min_priority: float = 0.0,
        max_priority: float = 100.0,
    ) -> pd.DataFrame:
        """
        Filter observations by priority range.

        Args:
            df: DataFrame to filter
            min_priority: Minimum priority (inclusive)
            max_priority: Maximum priority (inclusive)

        Returns:
            Filtered DataFrame

        Example:
            >>> high_priority = backend.filter_by_priority(df, min_priority=15.0)
        """
        return filter_by_priority_func(df, min_priority, max_priority, use_pandas=self.use_pandas)

    def filter_by_scheduled(
        self,
        df: pd.DataFrame,
        filter_type: Literal["All", "Scheduled", "Unscheduled"] = "All",
    ) -> pd.DataFrame:
        """
        Filter observations by scheduling status.

        Args:
            df: DataFrame to filter
            filter_type: 'All', 'Scheduled', or 'Unscheduled'

        Returns:
            Filtered DataFrame

        Example:
            >>> unscheduled = backend.filter_by_scheduled(df, "Unscheduled")
            >>> print(f"Unscheduled: {len(unscheduled)}")
        """
        return filter_by_scheduled_func(df, filter_type, use_pandas=self.use_pandas)

    def filter_dataframe(
        self,
        df: pd.DataFrame,
        priority_min: float = 0.0,
        priority_max: float = 100.0,
        scheduled_filter: Literal["All", "Scheduled", "Unscheduled"] = "All",
        priority_bins: list[str] | None = None,
        block_ids: list[str] | None = None,
    ) -> pd.DataFrame:
        """
        Apply multiple filters to DataFrame.

        Args:
            df: DataFrame to filter
            priority_min: Minimum priority
            priority_max: Maximum priority
            scheduled_filter: Scheduling status filter
            priority_bins: List of priority bins to include
            block_ids: List of scheduling block IDs to include

        Returns:
            Filtered DataFrame

        Example:
            >>> filtered = backend.filter_dataframe(
            ...     df,
            ...     priority_min=10.0,
            ...     priority_max=20.0,
            ...     scheduled_filter="Scheduled",
            ...     priority_bins=["High", "Very High"]
            ... )
        """
        return filter_dataframe_func(
            df,
            priority_min=priority_min,
            priority_max=priority_max,
            scheduled_filter=scheduled_filter,
            priority_bins=priority_bins,
            block_ids=block_ids,
            use_pandas=self.use_pandas,
        )

    def remove_duplicates(
        self,
        df: pd.DataFrame,
        subset: list[str] | None = None,
        keep: Literal["first", "last", "none"] = "first",
    ) -> pd.DataFrame:
        """
        Remove duplicate rows from DataFrame.

        Args:
            df: DataFrame to clean
            subset: Column names to consider for duplicates (None = all columns)
            keep: Which duplicates to keep ('first', 'last', 'none')

        Returns:
            DataFrame with duplicates removed

        Example:
            >>> clean_df = backend.remove_duplicates(df, subset=["schedulingBlockId"])
        """
        return remove_duplicates_func(df, subset=subset, keep=keep, use_pandas=self.use_pandas)

    def remove_missing_coordinates(self, df: pd.DataFrame) -> pd.DataFrame:
        """
        Remove observations with missing RA or Dec coordinates.

        Args:
            df: DataFrame to clean

        Returns:
            DataFrame with complete coordinates only

        Example:
            >>> valid_coords = backend.remove_missing_coordinates(df)
        """
        return remove_missing_coordinates_func(df, use_pandas=self.use_pandas)

    def validate_dataframe(self, df: pd.DataFrame) -> tuple[bool, list[str]]:
        """
        Validate DataFrame data quality (coordinates, priorities, etc.).

        Args:
            df: DataFrame to validate

        Returns:
            Tuple of (is_valid, list of issues)

        Example:
            >>> is_valid, issues = backend.validate_dataframe(df)
            >>> if not is_valid:
            ...     for issue in issues:
            ...         print(f"Warning: {issue}")
        """
        return validate_dataframe_func(df)

    # ===== Time Conversions =====

    @staticmethod
    def mjd_to_datetime(mjd: float) -> datetime:
        """
        Convert Modified Julian Date to Python datetime object.

        Args:
            mjd: Modified Julian Date value

        Returns:
            Python datetime object with UTC timezone

        Example:
            >>> dt = TSIBackend.mjd_to_datetime(59580.5)
            >>> print(dt)  # 2022-01-01 12:00:00+00:00
        """
        return cast(datetime, tsi_rust.mjd_to_datetime(mjd))

    @staticmethod
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
            >>> mjd = TSIBackend.datetime_to_mjd(dt)
            >>> print(mjd)  # 59580.5
        """
        return cast(float, tsi_rust.datetime_to_mjd(dt))

    @staticmethod
    def parse_visibility_periods(visibility_str: str) -> list[tuple[Any, Any]]:
        """
        Parse visibility period string.

        Args:
            visibility_str: String representation of visibility periods

        Returns:
            List of (start_datetime, stop_datetime) tuples
        """
        return cast(list[tuple[Any, Any]], tsi_rust.parse_visibility_periods(visibility_str))

    # ===== Utilities =====

    def __repr__(self) -> str:
        return f"TSIBackend(use_pandas={self.use_pandas})"


# ===== Convenience Functions (Backward Compatibility) =====


def load_schedule(path: str | Path, **kwargs: Any) -> pd.DataFrame:
    """
    Quick function to load schedule data. Returns pandas DataFrame.
    """
    return load_schedule_file(path, use_pandas=True, **kwargs)


def filter_by_priority(
    df: pd.DataFrame, min_priority: float = 0.0, max_priority: float = 100.0
) -> pd.DataFrame:
    """
    Quick function to filter by priority range.
    """
    return filter_by_priority_func(df, min_priority, max_priority, use_pandas=True)


# Version info
__version__ = "0.1.0"
__all__ = [
    "TSIBackend",
    "load_schedule",
    "load_schedule_file",
    "load_schedule_from_string",
    "load_dark_periods",
    "filter_by_priority",
    "filter_by_priority_func",
    "filter_by_scheduled_func",
    "filter_dataframe_func",
    "remove_duplicates_func",
    "remove_missing_coordinates_func",
    "validate_dataframe_func",
    "get_top_observations",
    "find_conflicts",
]
