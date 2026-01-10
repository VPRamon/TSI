"""Backend client layer for the Streamlit UI.

This module exposes high-level functions the UI needs to talk to the Rust
backend. All functions delegate to the PyO3 bindings and keep return types
simple so the UI doesn't depend on backend internals.
"""

from __future__ import annotations

import logging
from dataclasses import dataclass
from typing import TYPE_CHECKING, Any

import pandas as pd
import tsi_rust as api
from numpy import int64

from app_config import get_settings
from tsi.error_handling import log_error, with_retry
from tsi.exceptions import ServerError

if TYPE_CHECKING:
    from tsi_rust import (
        CompareData,
        InsightsData,
        ScheduleTimelineData,
        SkyMapData,
        ValidationReport,
        VisibilityMapData,
    )

logger = logging.getLogger(__name__)


@dataclass
class ScheduleSummary:
    """Minimal schedule metadata used across the UI."""

    id: int
    name: str

    @property
    def ref(self) -> api.ScheduleId:
        return api.ScheduleId(int(self.id))


def _import_rust() -> Any:
    """Import the Rust backend module, respecting configuration flags."""
    settings = get_settings()

    if not settings.enable_backend:
        raise ServerError(
            "Rust backend is disabled in configuration", details={"enable_backend": False}
        )

    try:
        import tsi_rust  # type: ignore[import-not-found]
    except ImportError as e:
        raise ServerError(
            "Rust backend is not available. Please compile the extension before using backend features.",
            details={"install_command": "maturin develop --release"},
        ) from e
    return tsi_rust


def _rust_call(method: str, *args: Any) -> Any:
    """Call a Rust backend function by name with consistent error handling."""
    try:
        rust = _import_rust()
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


def _to_schedule_id(schedule_ref: ScheduleSummary | api.ScheduleId | int | int64) -> api.ScheduleId:
    """Normalize any schedule reference to the ScheduleId wrapper."""
    if isinstance(schedule_ref, ScheduleSummary):
        return schedule_ref.ref
    if isinstance(schedule_ref, api.ScheduleId):
        return schedule_ref
    return api.ScheduleId(int(schedule_ref))


def _to_int(schedule_id: api.ScheduleId | int | int64 | Any) -> int:
    """Normalize a backend ScheduleId or numeric-like value to a plain int.

    The Rust `ScheduleId` is exposed as a Python object with a `value`
    attribute (no `__int__` implementation). Handle that here.
    """
    if isinstance(schedule_id, api.ScheduleId):
        return int(schedule_id.value)
    return int(schedule_id)


@with_retry(max_attempts=3, backoff_factor=1.5)
def upload_schedule(
    schedule_name: str,
    schedule_json: str,
    visibility_json: str | None = None,
) -> ScheduleSummary:
    """Upload and store a schedule via the backend."""
    try:
        result = _rust_call(
            api.POST_SCHEDULE,
            schedule_name,
            schedule_json,
            visibility_json,
        )
        schedule_id = _to_int(result)
        return ScheduleSummary(id=schedule_id, name=schedule_name)
    except Exception as e:
        raise ServerError(
            f"Failed to store schedule '{schedule_name}'",
            details={"schedule_name": schedule_name, "error": str(e)},
        ) from e


@with_retry(max_attempts=3, backoff_factor=1.5)
def list_schedules() -> list[ScheduleSummary]:
    """List available schedules using the backend."""
    raw_schedules = _rust_call(api.LIST_SCHEDULES)
    summaries: list[ScheduleSummary] = []

    for raw in raw_schedules:
        schedule_id = _extract_field(raw, "schedule_id")
        schedule_name = _extract_field(raw, "schedule_name") or _extract_field(raw, "name")

        if schedule_id is None:
            logger.warning("Skipping schedule entry with no id: %s", raw)
            continue

        summaries.append(
            ScheduleSummary(
                id=_to_int(schedule_id),
                name=(
                    str(schedule_name)
                    if schedule_name is not None
                    else f"Schedule {_to_int(schedule_id)}"
                ),
            )
        )

    return summaries


def _extract_field(obj: Any, field: str) -> Any:
    """Extract attribute or dict key safely from backend objects."""
    if hasattr(obj, field):
        return getattr(obj, field)
    if isinstance(obj, dict):
        return obj.get(field)
    return None


def get_sky_map_data(schedule_ref: ScheduleSummary | api.ScheduleId | int) -> SkyMapData:
    """Get complete sky map data with computed bins and metadata."""
    return _rust_call(api.GET_SKY_MAP_DATA, _to_schedule_id(schedule_ref))


def get_visibility_map_data(
    schedule_ref: ScheduleSummary | api.ScheduleId | int,
) -> VisibilityMapData:
    """Fetch visibility map metadata and block summaries from the backend."""
    return _rust_call(api.GET_VISIBILITY_MAP_DATA, _to_schedule_id(schedule_ref))


def get_distribution_data(
    schedule_ref: ScheduleSummary | api.ScheduleId | int,
) -> api.DistributionData:
    """Get complete distribution data with computed statistics."""
    return _rust_call(api.GET_DISTRIBUTION_DATA, _to_schedule_id(schedule_ref))


def get_schedule_timeline_data(
    schedule_ref: ScheduleSummary | api.ScheduleId | int,
) -> ScheduleTimelineData:
    """Get complete schedule timeline data with computed statistics and metadata."""
    return _rust_call(api.GET_SCHEDULE_TIMELINE_DATA, _to_schedule_id(schedule_ref))


def get_insights_data(
    schedule_ref: ScheduleSummary | api.ScheduleId | int,
) -> InsightsData:
    """Get complete insights data with computed analytics and metadata."""
    return _rust_call(api.GET_INSIGHTS_DATA, _to_schedule_id(schedule_ref))


def get_trends_data(
    schedule_ref: ScheduleSummary | api.ScheduleId | int,
    n_bins: int = 10,
    bandwidth: float = 0.3,
    n_smooth_points: int = 100,
) -> Any:
    """Get scheduling trends data from the backend."""
    return _rust_call(
        api.GET_TRENDS_DATA,
        _to_schedule_id(schedule_ref),
        n_bins,
        bandwidth,
        n_smooth_points,
    )


def get_compare_data(
    current_schedule_ref: ScheduleSummary | api.ScheduleId | int,
    comparison_schedule_ref: ScheduleSummary | api.ScheduleId | int,
    current_name: str,
    comparison_name: str,
) -> CompareData:
    """Get comparison data for two schedules."""
    return _rust_call(
        api.GET_COMPARE_DATA,
        _to_schedule_id(current_schedule_ref),
        _to_schedule_id(comparison_schedule_ref),
        current_name,
        comparison_name,
    )


def fetch_dark_periods(schedule_ref: ScheduleSummary | api.ScheduleId | int) -> pd.DataFrame:
    """Fetch dark periods for a schedule (with global fallback)."""
    df_polars = _rust_call("py_fetch_dark_periods", _to_schedule_id(schedule_ref))
    return df_polars.to_pandas()  # type: ignore[no-any-return]


def fetch_possible_periods(schedule_ref: ScheduleSummary | api.ScheduleId | int) -> pd.DataFrame:
    """Fetch possible/visibility periods for a schedule."""
    df_polars = _rust_call("py_fetch_possible_periods", _to_schedule_id(schedule_ref))
    return df_polars.to_pandas()  # type: ignore[no-any-return]


def get_visibility_histogram(
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
    schedule_id = _to_schedule_id(schedule_ref)

    start_unix = int(start.timestamp())
    end_unix = int(end.timestamp())

    if priority_range is None and block_ids is None:
        try:
            result = _rust_call(
                "py_get_visibility_histogram_analytics",
                schedule_id,
                start_unix,
                end_unix,
                bin_duration_minutes,
            )
            if result:
                logger.debug("Using pre-computed visibility histogram for schedule %s", schedule_id)
                return result  # type: ignore[no-any-return]
        except Exception as e:
            logger.debug("Analytics histogram not available, falling back: %s", e)

    priority_min = priority_range[0] if priority_range else None
    priority_max = priority_range[1] if priority_range else None

    return _rust_call(  # type: ignore[no-any-return]
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
    schedule_ref: ScheduleSummary | api.ScheduleId | int,
) -> tuple[pd.Timestamp, pd.Timestamp] | None:
    """Get the time range (min/max timestamps) for a schedule's visibility periods."""
    schedule_id = _to_schedule_id(schedule_ref)
    result = _rust_call(api.GET_SCHEDULE_TIME_RANGE, schedule_id)

    if result is None:
        return None

    start_unix, end_unix = result

    start_time = pd.Timestamp(start_unix, unit="s", tz="UTC")
    end_time = pd.Timestamp(end_unix, unit="s", tz="UTC")

    return start_time, end_time


def get_validation_report(schedule_ref: ScheduleSummary | api.ScheduleId | int) -> ValidationReport:
    """Get validation report data for a schedule."""
    return _rust_call(api.GET_VALIDATION_REPORT, _to_schedule_id(schedule_ref))


__all__ = [
    "ScheduleSummary",
    "upload_schedule",
    "list_schedules",
    "fetch_dark_periods",
    "fetch_possible_periods",
    "get_visibility_map_data",
    "get_distribution_data",
    "get_sky_map_data",
    "get_schedule_timeline_data",
    "get_insights_data",
    "get_trends_data",
    "get_compare_data",
    "get_visibility_histogram",
    "get_schedule_time_range",
    "get_validation_report",
]
