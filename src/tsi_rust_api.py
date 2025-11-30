"""
TSI Rust Backend - Python Integration Layer

High-level Python API for the Telescope Scheduling Intelligence Rust backend.
Provides ergonomic wrappers and automatic type conversions between Rust and Python.

Example:
    >>> from tsi_rust_api import TSIBackend
    >>>
    >>> # Initialize backend
    >>> backend = TSIBackend()
    >>>
    >>> # Load schedule data
    >>> df = backend.load_schedule("data/schedule.csv")
    >>> print(f"Loaded {len(df)} observations")
    >>>
    >>> # Compute analytics
    >>> metrics = backend.compute_metrics(df)
    >>> print(f"Scheduled: {metrics.scheduled_count}/{metrics.total_observations}")
    >>>
    >>> # Filter data
    >>> high_priority = backend.filter_by_priority(df, min_priority=15.0)
    >>> print(f"High priority observations: {len(high_priority)}")
"""

from __future__ import annotations

import logging
from datetime import datetime
from pathlib import Path
from typing import Any, Literal, cast

import pandas as pd
import polars as pl

from tsi.exceptions import BackendUnavailableError, DataLoadError

logger = logging.getLogger(__name__)

try:
    import tsi_rust
except ImportError as e:
    raise BackendUnavailableError(
        "tsi_rust module not found. Please compile the Rust backend with: maturin develop --release",
        details={"install_command": "maturin develop --release"}
    ) from e


class TSIBackend:
    """
    High-level Python interface to the TSI Rust backend.

    Provides ergonomic methods for loading, processing, and analyzing
    telescope scheduling data with automatic type conversions.

    Example:
        >>> backend = TSIBackend()
        >>> df = backend.load_schedule("data/schedule.csv")
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
            >>> df = backend.load_schedule("data/schedule.csv")
            >>> print(df.columns)
        """
        path = Path(path)

        if format == "auto":
            format = "json" if path.suffix == ".json" else "csv"

        if format == "csv":
            df_polars: pl.DataFrame = tsi_rust.load_schedule_from_csv(str(path))
        elif format == "json":
            df_polars = tsi_rust.load_schedule_from_json(str(path))
        else:
            raise ValueError(f"Unknown format: {format}")

        return cast(pd.DataFrame, df_polars.to_pandas()) if self.use_pandas else df_polars

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
        if format == "json":
            df_polars: pl.DataFrame = tsi_rust.load_schedule_from_json_str(content)
        else:
            raise ValueError(f"Format {format} not supported for string loading")

        return cast(pd.DataFrame, df_polars.to_pandas()) if self.use_pandas else df_polars

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
        df_polars = self._to_polars(df)
        validation_result = tsi_rust.py_validate_schedule(df_polars)
        return cast(dict[str, Any], validation_result.to_dict())

    # ===== Analytics & Algorithms =====

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
        df_polars = self._to_polars(df)
        analytics = tsi_rust.py_compute_metrics(df_polars)
        return cast(dict[str, Any], analytics.to_dict())

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
        df_polars = self._to_polars(df)
        result: pl.DataFrame = tsi_rust.py_get_top_observations(
            df_polars, by, n
        )  # Correct order: df, by, n
        return cast(pd.DataFrame, result.to_pandas()) if self.use_pandas else result

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
        df_polars = self._to_polars(df)
        result = tsi_rust.py_find_conflicts(df_polars)

        # Rust backend may return list or DataFrame depending on version
        if isinstance(result, list):
            # Convert list to DataFrame
            if not result:
                # Empty conflicts - return empty DataFrame
                return pd.DataFrame() if self.use_pandas else pl.DataFrame()
            return pd.DataFrame(result) if self.use_pandas else pl.DataFrame(result)

        # Result is already a DataFrame
        result_df: pl.DataFrame = result
        return cast(pd.DataFrame, result_df.to_pandas()) if self.use_pandas else result_df

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
        df_polars = self._to_polars(df)
        opt_result = tsi_rust.py_greedy_schedule(df_polars, max_iterations)
        return cast(dict[str, Any], opt_result.to_dict())

    # ===== Transformations & Filtering =====

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
        df_polars = self._to_polars(df)
        result: pl.DataFrame = tsi_rust.py_filter_by_range(
            df_polars, "priority", min_priority, max_priority
        )
        return cast(pd.DataFrame, result.to_pandas()) if self.use_pandas else result

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
        df_polars = self._to_polars(df)
        result: pl.DataFrame = tsi_rust.py_filter_by_scheduled(df_polars, filter_type)
        return cast(pd.DataFrame, result.to_pandas()) if self.use_pandas else result

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
        df_polars = self._to_polars(df)
        result: pl.DataFrame = tsi_rust.py_filter_dataframe(
            df_polars, priority_min, priority_max, scheduled_filter, priority_bins, block_ids
        )
        return cast(pd.DataFrame, result.to_pandas()) if self.use_pandas else result

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
        df_polars = self._to_polars(df)
        result: pl.DataFrame = tsi_rust.py_remove_duplicates(df_polars, subset, keep)
        return cast(pd.DataFrame, result.to_pandas()) if self.use_pandas else result

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
        df_polars = self._to_polars(df)
        result: pl.DataFrame = tsi_rust.py_remove_missing_coordinates(df_polars)
        return cast(pd.DataFrame, result.to_pandas()) if self.use_pandas else result

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
        df_polars = self._to_polars(df)
        return cast(tuple[bool, list[str]], tsi_rust.py_validate_dataframe(df_polars))

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

    def _to_polars(self, df: pd.DataFrame | pl.DataFrame) -> pl.DataFrame:
        """Convert DataFrame to Polars format if needed."""
        if isinstance(df, pd.DataFrame):
            return pl.from_pandas(df)
        return df

    def __repr__(self) -> str:
        return f"TSIBackend(use_pandas={self.use_pandas})"


# Convenience functions for quick access
def load_schedule(path: str | Path, **kwargs: Any) -> pd.DataFrame:
    """Quick function to load schedule data. Returns pandas DataFrame."""
    backend = TSIBackend(use_pandas=True)
    return cast(pd.DataFrame, backend.load_schedule(path, **kwargs))


def load_dark_periods(path: str | Path) -> pd.DataFrame:
    """
    Quick function to load dark periods data.

    Args:
        path: Path to dark_periods.json file

    Returns:
        pandas DataFrame with columns: start_dt, stop_dt, start_mjd, stop_mjd,
        duration_hours, months

    Example:
        >>> from tsi_rust_api import load_dark_periods
        >>> df = load_dark_periods("data/dark_periods.json")
        >>> print(f"Loaded {len(df)} dark periods")
    """
    df_polars = tsi_rust.load_dark_periods(str(path))
    return cast(pd.DataFrame, df_polars.to_pandas())


def compute_metrics(df: pd.DataFrame | pl.DataFrame) -> dict[str, Any]:
    """Quick function to compute scheduling metrics."""
    backend = TSIBackend()
    return backend.compute_metrics(df)


def filter_by_priority(
    df: pd.DataFrame | pl.DataFrame, min_priority: float = 0.0, max_priority: float = 100.0
) -> pd.DataFrame:
    """Quick function to filter by priority range."""
    backend = TSIBackend(use_pandas=True)
    return cast(pd.DataFrame, backend.filter_by_priority(df, min_priority, max_priority))


# Version info
__version__ = "0.1.0"
__all__ = [
    "TSIBackend",
    "load_schedule",
    "load_dark_periods",
    "compute_metrics",
    "filter_by_priority",
]
