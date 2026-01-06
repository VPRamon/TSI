"""Unified backend service facade.

This module provides a single entry point for all backend operations,
combining both remote (backend_client) and local (TSIBackend) functionality.
It eliminates the need for callers to choose between two separate backend
interfaces and simplifies the import structure.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import TYPE_CHECKING, Any, Literal, cast

import pandas as pd
from numpy import int64

from app_config import get_settings
from tsi.error_handling import log_error, with_retry
from tsi.exceptions import ServerError
import tsi_rust as api
import tsi_rust_api

if TYPE_CHECKING:
    from tsi_rust import (
        CompareData,
        DistributionData,
        InsightsData,
        ScheduleTimelineData,
        SkyMapData,
        ValidationReport,
        VisibilityMapData,
    )

logger = logging.getLogger(__name__)


# ============================================================================
# Shared Backend Instance
# ============================================================================

# Single shared TSIBackend instance for DataFrame operations
_BACKEND = tsi_rust_api.TSIBackend(use_pandas=True)


# ============================================================================
# Schedule Reference Type
# ============================================================================

@dataclass
class ScheduleSummary:
    """Minimal schedule metadata used across the UI."""

    id: int
    name: str

    @property
    def ref(self) -> api.ScheduleId:
        return api.ScheduleId(int(self.id))


# ============================================================================
# Backend Service Facade
# ============================================================================

class BackendService:
    """
    Unified backend service combining remote and local operations.

    This facade provides a single entry point for:
    - Remote backend calls (upload, list, get data from stored schedules)
    - Local DataFrame operations (filter, validate, transform)
    - Time conversions (MJD <-> datetime)
    - Data loading (from files or file-like objects)

    Usage:
        >>> backend = BackendService()
        >>> # Upload schedule
        >>> summary = backend.upload_schedule("My Schedule", json_data)
        >>> # Load and filter DataFrame
        >>> df = backend.load_schedule("data/schedule.json")
        >>> filtered = backend.filter_by_priority(df, min_priority=15.0)
        >>> # Convert times
        >>> dt = backend.mjd_to_datetime(59580.5)
    """

    def __init__(self):
        """Initialize the backend service."""
        self._local_backend = _BACKEND

    # ========================================================================
    # Remote Backend Operations (Schedule Management)
    # ========================================================================

    def _import_rust(self) -> Any:
        """Import the Rust backend module, respecting configuration flags."""
        settings = get_settings()

        if not settings.enable_rust_backend:
            raise ServerError(
                "Rust backend is disabled in configuration",
                details={"enable_rust_backend": False},
            )

        try:
            import tsi_rust  # type: ignore[import-not-found]
        except ImportError as e:
            raise ServerError(
                "Rust backend is not available. Please compile the extension before using backend features.",
                details={"install_command": "maturin develop --release"},
            ) from e
        return tsi_rust

    def _rust_call(self, method: str, *args: Any) -> Any:
        """Call a Rust backend function by name with consistent error handling."""
        try:
            rust = self._import_rust()
            return getattr(rust, method)(*args)
        except ServerError:
            raise
        except AttributeError as e:
            raise ServerError(
                f"Rust backend method '{method}' not found", details={"method": method}
            ) from e
        except Exception as e:
            log_error(
                e,
                f"Rust backend call '{method}' failed",
                extra={"method": method, "args_count": len(args)},
            )
            raise

    def _to_schedule_id(self, schedule_ref: ScheduleSummary | api.ScheduleId | int | int64) -> api.ScheduleId:
        """Normalize any schedule reference to the ScheduleId wrapper."""
        if isinstance(schedule_ref, ScheduleSummary):
            return schedule_ref.ref
        if isinstance(schedule_ref, api.ScheduleId):
            return schedule_ref
        return api.ScheduleId(int(schedule_ref))

    def _to_int(self, schedule_id: api.ScheduleId | int | int64 | Any) -> int:
        """Normalize a backend ScheduleId or numeric-like value to a plain int."""
        if isinstance(schedule_id, api.ScheduleId):
            return int(schedule_id.value)
        return int(schedule_id)

    @with_retry(max_attempts=3, backoff_factor=1.5)
    def upload_schedule(
        self,
        schedule_name: str,
        schedule_json: str,
        visibility_json: str | None = None,
    ) -> ScheduleSummary:
        """Upload and store a schedule via the backend."""
        try:
            result = self._rust_call(
                api.POST_SCHEDULE,
                schedule_name,
                schedule_json,
                visibility_json,
            )
            schedule_id = self._to_int(result)
            return ScheduleSummary(id=schedule_id, name=schedule_name)
        except Exception as e:
            raise ServerError(
                f"Failed to store schedule '{schedule_name}'",
                details={"schedule_name": schedule_name, "error": str(e)},
            ) from e

    @with_retry(max_attempts=3, backoff_factor=1.5)
    def list_schedules(self) -> list[ScheduleSummary]:
        """List available schedules using the backend."""
        raw_schedules = self._rust_call(api.LIST_SCHEDULES)
        summaries: list[ScheduleSummary] = []

        for raw in raw_schedules:
            schedule_id = self._extract_field(raw, "schedule_id")
            schedule_name = self._extract_field(raw, "schedule_name") or self._extract_field(raw, "name")

            if schedule_id is None:
                logger.warning("Skipping schedule entry with no id: %s", raw)
                continue

            summaries.append(
                ScheduleSummary(
                    id=self._to_int(schedule_id),
                    name=str(schedule_name) if schedule_name is not None else f"Schedule {self._to_int(schedule_id)}",
                )
            )

        return summaries

    def _extract_field(self, obj: Any, field: str) -> Any:
        """Extract attribute or dict key safely from backend objects."""
        if hasattr(obj, field):
            return getattr(obj, field)
        if isinstance(obj, dict):
            return obj.get(field)
        return None

    def get_sky_map_data(self, schedule_ref: ScheduleSummary | api.ScheduleId | int) -> SkyMapData:
        """Get complete sky map data with computed bins and metadata."""
        return self._rust_call(api.GET_SKY_MAP_DATA, self._to_schedule_id(schedule_ref))

    def get_visibility_map_data(
        self, schedule_ref: ScheduleSummary | api.ScheduleId | int
    ) -> VisibilityMapData:
        """Fetch visibility map metadata and block summaries from the backend."""
        return self._rust_call(api.GET_VISIBILITY_MAP_DATA, self._to_schedule_id(schedule_ref))

    def get_distribution_data(
        self, schedule_ref: ScheduleSummary | api.ScheduleId | int
    ) -> api.DistributionData:
        """Get complete distribution data with computed statistics."""
        return self._rust_call(api.GET_DISTRIBUTION_DATA, self._to_schedule_id(schedule_ref))

    def get_schedule_timeline_data(
        self, schedule_ref: ScheduleSummary | api.ScheduleId | int
    ) -> ScheduleTimelineData:
        """Get complete schedule timeline data with computed statistics and metadata."""
        return self._rust_call(api.GET_SCHEDULE_TIMELINE_DATA, self._to_schedule_id(schedule_ref))

    def get_insights_data(
        self, schedule_ref: ScheduleSummary | api.ScheduleId | int
    ) -> InsightsData:
        """Get complete insights data with computed analytics and metadata."""
        return self._rust_call(api.GET_INSIGHTS_DATA, self._to_schedule_id(schedule_ref))

    def get_trends_data(
        self,
        schedule_ref: ScheduleSummary | api.ScheduleId | int,
        n_bins: int = 10,
        bandwidth: float = 0.3,
        n_smooth_points: int = 100,
    ) -> api.TrendsData:
        """Get complete trends data with computed statistics and smoothed curves."""
        return self._rust_call(
            api.GET_TRENDS_DATA,
            self._to_schedule_id(schedule_ref),
            n_bins,
            bandwidth,
            n_smooth_points,
        )

    def get_compare_data(
        self,
        schedule_a_ref: ScheduleSummary | api.ScheduleId | int,
        schedule_b_ref: ScheduleSummary | api.ScheduleId | int,
    ) -> CompareData:
        """Get comparison data between two schedules."""
        return self._rust_call(
            api.GET_COMPARE_DATA,
            self._to_schedule_id(schedule_a_ref),
            self._to_schedule_id(schedule_b_ref),
        )

    def get_validation_report(
        self, schedule_ref: ScheduleSummary | api.ScheduleId | int
    ) -> ValidationReport:
        """Get validation report for a schedule."""
        return self._rust_call(api.GET_VALIDATION_REPORT, self._to_schedule_id(schedule_ref))

    def fetch_dark_periods(self, schedule_ref: ScheduleSummary | api.ScheduleId | int) -> pd.DataFrame:
        """Fetch dark periods for a schedule (with global fallback)."""
        df_polars = self._rust_call("py_fetch_dark_periods", self._to_schedule_id(schedule_ref))
        return df_polars.to_pandas()  # type: ignore[no-any-return]

    def fetch_possible_periods(
        self, schedule_ref: ScheduleSummary | api.ScheduleId | int
    ) -> pd.DataFrame:
        """Fetch possible/visibility periods for a schedule."""
        df_polars = self._rust_call("py_fetch_possible_periods", self._to_schedule_id(schedule_ref))
        return df_polars.to_pandas()  # type: ignore[no-any-return]

    def get_visibility_histogram(
        self,
        schedule_ref: ScheduleSummary | api.ScheduleId | int,
        start: pd.Timestamp,
        end: pd.Timestamp,
        bin_duration_minutes: int,
        priority_range: tuple[int, int] | None = None,
        block_ids: list[int] | None = None,
    ) -> list[dict[str, Any]]:
        """
        Compute visibility histogram from the backend.

        Returns a list of time bins with counts of visible scheduling blocks.
        """
        schedule_id = self._to_schedule_id(schedule_ref)

        start_unix = int(start.timestamp())
        end_unix = int(end.timestamp())

        if priority_range is None and block_ids is None:
            try:
                result = self._rust_call(
                    "py_get_visibility_histogram_analytics",
                    schedule_id,
                    start_unix,
                    end_unix,
                    bin_duration_minutes,
                )
                if result:
                    logger.debug(
                        "Using pre-computed visibility histogram for schedule %s", schedule_id
                    )
                    return result  # type: ignore[no-any-return]
            except Exception as e:
                logger.debug("Analytics histogram not available, falling back: %s", e)

        priority_min = priority_range[0] if priority_range else None
        priority_max = priority_range[1] if priority_range else None

        return self._rust_call(  # type: ignore[no-any-return]
            api.GET_VISIBILITY_HISTOGRAM,
            schedule_id,
            start_unix,
            end_unix,
            bin_duration_minutes,
            priority_min,
            priority_max,
            block_ids,
        )

    def get_schedule_time_range(
        self, schedule_ref: ScheduleSummary | api.ScheduleId | int
    ) -> tuple[pd.Timestamp, pd.Timestamp] | None:
        """Get the time range (min/max timestamps) for a schedule's visibility periods."""
        schedule_id = self._to_schedule_id(schedule_ref)
        result = self._rust_call(api.GET_SCHEDULE_TIME_RANGE, schedule_id)

        if result is None:
            return None

        start_unix, end_unix = result

        start_time = pd.Timestamp(start_unix, unit="s", tz="UTC")
        end_time = pd.Timestamp(end_unix, unit="s", tz="UTC")

        return start_time, end_time

    # ========================================================================
    # Local DataFrame Operations
    # ========================================================================

    @with_retry(max_attempts=2, backoff_factor=1.5)
    def load_schedule(
        self, source: str | Path | Any, format: Literal["auto", "json"] = "auto"
    ) -> pd.DataFrame:
        """
        Load schedule data from a path or file-like object.

        Args:
            source: Path to schedule file or file-like object
            format: File format ('auto' or 'json')

        Returns:
            DataFrame with schedule data

        Raises:
            ServerError: If loading fails
        """
        try:
            if hasattr(source, "read"):
                content = source.read()
                if isinstance(content, bytes):
                    content = content.decode("utf-8")
                if hasattr(source, "seek"):
                    source.seek(0)

                if format == "auto":
                    raise ValueError("Format must be specified when reading from a buffer")
                if format == "json":
                    return cast(
                        pd.DataFrame,
                        tsi_rust_api.load_schedule_from_string(
                            content, format="json", use_pandas=True
                        ),
                    )
                raise ValueError(f"Unsupported format: {format}")

            return cast(
                pd.DataFrame,
                tsi_rust_api.load_schedule_file(Path(source), format=format, use_pandas=True),
            )
        except Exception as e:
            raise ServerError(
                f"Failed to load schedule from {source}",
                details={"path": str(source), "format": format, "error": str(e)},
            ) from e

    def filter_by_priority(
        self, df: pd.DataFrame, min_priority: float = 0.0, max_priority: float = 10.0
    ) -> pd.DataFrame:
        """Filter dataframe by priority range."""
        return cast(pd.DataFrame, self._local_backend.filter_by_priority(df, min_priority, max_priority))

    def filter_by_scheduled(self, df: pd.DataFrame, filter_type: str = "All") -> pd.DataFrame:
        """Filter dataframe by scheduled status."""
        return cast(pd.DataFrame, self._local_backend.filter_by_scheduled(df, filter_type))  # type: ignore[arg-type]

    def filter_dataframe(
        self,
        df: pd.DataFrame,
        priority_min: float = 0.0,
        priority_max: float = 10.0,
        scheduled_filter: str = "All",
        priority_bins: list[str] | None = None,
        block_ids: list[str | int] | None = None,
    ) -> pd.DataFrame:
        """
        Filter DataFrame based on multiple criteria using Rust backend.

        This is the canonical filtering function - all filtering should go through
        this function which delegates to the Rust backend for performance.

        Args:
            df: DataFrame to filter
            priority_min: Minimum priority
            priority_max: Maximum priority
            scheduled_filter: 'All', 'Scheduled', or 'Unscheduled'
            priority_bins: Optional list of priority bin labels to include
            block_ids: Optional list of scheduling block IDs to include

        Returns:
            Filtered DataFrame
        """
        # Convert block_ids to strings for Rust backend
        block_ids_str = [str(bid) for bid in block_ids] if block_ids else None

        return cast(
            pd.DataFrame,
            self._local_backend.filter_dataframe(
                df,
                priority_min=priority_min,
                priority_max=priority_max,
                scheduled_filter=scheduled_filter,  # type: ignore[arg-type]
                priority_bins=priority_bins,
                block_ids=block_ids_str,
            ),
        )

    def validate_dataframe(self, df: pd.DataFrame) -> tuple[bool, list[str]]:
        """
        Validate DataFrame data quality (coordinates, priorities, etc.).

        Uses Rust backend for 5x faster validation.

        Args:
            df: DataFrame to validate

        Returns:
            Tuple of (is_valid, list of issues)
        """
        try:
            return self._local_backend.validate_dataframe(df)
        except Exception as e:
            return False, [f"Validation failed: {e}"]

    # ========================================================================
    # Time Conversions
    # ========================================================================

    @staticmethod
    def mjd_to_datetime(mjd: float) -> datetime:
        """
        Convert Modified Julian Date to Python datetime object.

        Args:
            mjd: Modified Julian Date value

        Returns:
            Python datetime object with UTC timezone
        """
        return cast(datetime, tsi_rust_api.TSIBackend.mjd_to_datetime(mjd))

    @staticmethod
    def datetime_to_mjd(dt: datetime) -> float:
        """
        Convert Python datetime object to Modified Julian Date.

        Args:
            dt: Python datetime object (must have timezone info)

        Returns:
            Modified Julian Date value
        """
        return cast(float, tsi_rust_api.TSIBackend.datetime_to_mjd(dt))

    @staticmethod
    def parse_visibility_periods(visibility_str: str) -> list[tuple[Any, Any]]:
        """
        Parse visibility period string.

        Args:
            visibility_str: String representation of visibility periods

        Returns:
            List of (start_datetime, stop_datetime) tuples
        """
        return cast(list[tuple[Any, Any]], tsi_rust_api.TSIBackend.parse_visibility_periods(visibility_str))


# ============================================================================
# Singleton Instance
# ============================================================================

# Global singleton instance for convenience
backend = BackendService()


__all__ = [
    "BackendService",
    "backend",
    "ScheduleSummary",
]
