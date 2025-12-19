"""
TSI Backend Core - Main TSIBackend class and initialization.

This module contains the core TSIBackend class that provides the primary
interface to the Rust backend. It handles initialization, DataFrame
conversion, and coordination between the various backend operations.

Example:
    >>> from tsi.backend import TSIBackend
    >>> backend = TSIBackend()
    >>> df = backend.load_schedule("data/schedule.json")
"""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING

import pandas as pd
import polars as pl

from tsi.exceptions import ServerError

if TYPE_CHECKING:
    from datetime import datetime
    from pathlib import Path
    from typing import Any, Literal

logger = logging.getLogger(__name__)

# Import Rust backend module
try:
    import tsi_rust
except ImportError as e:
    raise ServerError(
        "tsi_rust module not found. Please compile the Rust backend with: maturin develop --release",
        details={"install_command": "maturin develop --release"},
    ) from e


class TSIBackend:
    """
    High-level Python interface to the TSI Rust backend.

    Provides ergonomic methods for loading, processing, and analyzing
    telescope scheduling data with automatic type conversions.

    The class is organized into logical method groups:
    - Data Loading: load_schedule, load_schedule_from_string
    - Preprocessing: validate_schedule
    - Analytics: compute_metrics, get_top_observations, find_conflicts, greedy_schedule
    - Transformations: filter_by_priority, filter_by_scheduled, filter_dataframe,
                       remove_duplicates, remove_missing_coordinates, validate_dataframe
    - Time Conversions: mjd_to_datetime, datetime_to_mjd, parse_visibility_periods

    Example:
        >>> backend = TSIBackend()
        >>> df = backend.load_schedule("data/schedule.json")
        >>> metrics = backend.compute_metrics(df)
    """

    def __init__(self, use_pandas: bool = True):
        """
        Initialize TSI backend.

        Args:
            use_pandas: If True, return pandas DataFrames. If False, return Polars DataFrames.
        """
        self.use_pandas = use_pandas

    # ===== Data Loading =====
    # Implemented in tsi.backend.loaders, delegated here for convenience

    def load_schedule(
        self, path: str | Path, format: Literal["auto", "csv", "json"] = "auto"
    ) -> pd.DataFrame | pl.DataFrame:
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
        from tsi.backend.loaders import load_schedule_file

        return load_schedule_file(path, format=format, use_pandas=self.use_pandas)

    def load_schedule_from_string(
        self, content: str, format: Literal["csv", "json"] = "json"
    ) -> pd.DataFrame | pl.DataFrame:
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
        from tsi.backend.loaders import load_schedule_from_string

        return load_schedule_from_string(content, format=format, use_pandas=self.use_pandas)

    # ===== Preprocessing =====

    def validate_schedule(self, df: pd.DataFrame | pl.DataFrame) -> dict[str, Any]:
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
        from typing import cast

        df_polars = self._to_polars(df)
        validation_result = tsi_rust.py_validate_schedule(df_polars)
        return cast(dict[str, Any], validation_result.to_dict())

    # ===== Analytics & Algorithms =====
    # Implemented in tsi.backend.analytics, delegated here for convenience

    def compute_metrics(self, df: pd.DataFrame | pl.DataFrame) -> dict[str, Any]:
        """
        Compute comprehensive scheduling metrics.

        Args:
            df: DataFrame with scheduling data

        Returns:
            Dictionary with metrics (total_observations, scheduled_count,
            mean_priority, median_priority, etc.)

        Example:
            >>> metrics = backend.compute_metrics(df)
            >>> print(f"Scheduled: {metrics['scheduled_percentage']:.1f}%")
        """
        from tsi.backend.analytics import compute_metrics

        return compute_metrics(self._to_polars(df))

    def get_top_observations(
        self, df: pd.DataFrame | pl.DataFrame, n: int = 10, by: str = "priority"
    ) -> pd.DataFrame | pl.DataFrame:
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
        from tsi.backend.analytics import get_top_observations

        # get_top_observations already returns pandas DataFrame
        return get_top_observations(self._to_polars(df), n=n, by=by)

    def find_conflicts(self, df: pd.DataFrame | pl.DataFrame) -> pd.DataFrame | pl.DataFrame:
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
        from tsi.backend.analytics import find_conflicts

        # find_conflicts already returns pandas DataFrame
        return find_conflicts(self._to_polars(df))

    def greedy_schedule(
        self, df: pd.DataFrame | pl.DataFrame, max_iterations: int = 1000
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
            >>> result = backend.greedy_schedule(df, max_iterations=500)
            >>> print(f"Selected {len(result['selected_ids'])} observations")
        """
        from tsi.backend.analytics import greedy_schedule

        return greedy_schedule(self._to_polars(df), max_iterations=max_iterations)

    # ===== Transformations & Filtering =====
    # Implemented in tsi.backend.transformations, delegated here for convenience

    def filter_by_priority(
        self,
        df: pd.DataFrame | pl.DataFrame,
        min_priority: float = 0.0,
        max_priority: float = 100.0,
    ) -> pd.DataFrame | pl.DataFrame:
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
        from tsi.backend.transformations import filter_by_priority

        return filter_by_priority(
            self._to_polars(df), min_priority, max_priority, use_pandas=self.use_pandas
        )

    def filter_by_scheduled(
        self,
        df: pd.DataFrame | pl.DataFrame,
        filter_type: Literal["All", "Scheduled", "Unscheduled"] = "All",
    ) -> pd.DataFrame | pl.DataFrame:
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
        from tsi.backend.transformations import filter_by_scheduled

        return filter_by_scheduled(self._to_polars(df), filter_type, use_pandas=self.use_pandas)

    def filter_dataframe(
        self,
        df: pd.DataFrame | pl.DataFrame,
        priority_min: float = 0.0,
        priority_max: float = 100.0,
        scheduled_filter: Literal["All", "Scheduled", "Unscheduled"] = "All",
        priority_bins: list[str] | None = None,
        block_ids: list[str] | None = None,
    ) -> pd.DataFrame | pl.DataFrame:
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
        from tsi.backend.transformations import filter_dataframe

        return filter_dataframe(
            self._to_polars(df),
            priority_min=priority_min,
            priority_max=priority_max,
            scheduled_filter=scheduled_filter,
            priority_bins=priority_bins,
            block_ids=block_ids,
            use_pandas=self.use_pandas,
        )

    def remove_duplicates(
        self,
        df: pd.DataFrame | pl.DataFrame,
        subset: list[str] | None = None,
        keep: Literal["first", "last", "none"] = "first",
    ) -> pd.DataFrame | pl.DataFrame:
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
        from tsi.backend.transformations import remove_duplicates

        return remove_duplicates(
            self._to_polars(df), subset=subset, keep=keep, use_pandas=self.use_pandas
        )

    def remove_missing_coordinates(
        self, df: pd.DataFrame | pl.DataFrame
    ) -> pd.DataFrame | pl.DataFrame:
        """
        Remove observations with missing RA or Dec coordinates.

        Args:
            df: DataFrame to clean

        Returns:
            DataFrame with complete coordinates only

        Example:
            >>> valid_coords = backend.remove_missing_coordinates(df)
        """
        from tsi.backend.transformations import remove_missing_coordinates

        return remove_missing_coordinates(self._to_polars(df), use_pandas=self.use_pandas)

    def validate_dataframe(self, df: pd.DataFrame | pl.DataFrame) -> tuple[bool, list[str]]:
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
        from tsi.backend.transformations import validate_dataframe

        return validate_dataframe(self._to_polars(df))

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
        from typing import cast

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
        from typing import cast

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
        from typing import cast

        return cast(list[tuple[Any, Any]], tsi_rust.parse_visibility_periods(visibility_str))

    # ===== Utilities =====

    def _to_polars(self, df: pd.DataFrame | pl.DataFrame) -> pl.DataFrame:
        """Convert DataFrame to Polars format if needed."""
        if isinstance(df, pd.DataFrame):
            return pl.from_pandas(df)
        return df

    def __repr__(self) -> str:
        return f"TSIBackend(use_pandas={self.use_pandas})"


# Expose the Rust module for direct access when needed
def get_rust_module():
    """Get direct access to the tsi_rust module."""
    return tsi_rust
