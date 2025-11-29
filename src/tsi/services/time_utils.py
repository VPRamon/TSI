"""Lightweight time helpers backed by the Rust backend."""

from __future__ import annotations

from collections.abc import Sequence
from datetime import datetime
from typing import Any

import pandas as pd

from tsi_rust_api import TSIBackend


def mjd_to_datetime(mjd: float) -> pd.Timestamp:
    """Convert Modified Julian Date to a pandas Timestamp (UTC)."""
    return pd.Timestamp(TSIBackend.mjd_to_datetime(mjd))


def datetime_to_mjd(dt: datetime) -> float:
    """Convert a timezone-aware datetime to Modified Julian Date."""
    if dt.tzinfo is None:
        raise ValueError("Datetime must be timezone-aware")
    return float(TSIBackend.datetime_to_mjd(dt))


def parse_visibility_periods(visibility_str: str) -> list[tuple[pd.Timestamp, pd.Timestamp]]:
    """Parse visibility periods into timestamp tuples."""
    if not visibility_str or str(visibility_str).strip() == "":
        return []

    try:
        periods: list[tuple[Any, Any]] = TSIBackend.parse_visibility_periods(str(visibility_str))
        return [(pd.Timestamp(start), pd.Timestamp(stop)) for start, stop in periods]
    except Exception:
        return []


def parse_optional_mjd(value: float | None) -> pd.Timestamp | None:
    """Convert an optional MJD value to a Timestamp, preserving missing values."""
    if value is None or (isinstance(value, float) and pd.isna(value)):
        return None
    return mjd_to_datetime(float(value))


def get_time_range(
    periods: Sequence[tuple[pd.Timestamp, pd.Timestamp]],
) -> tuple[pd.Timestamp | None, pd.Timestamp | None]:
    """Return earliest start and latest stop across periods."""
    if not periods:
        return None, None

    starts = [period[0] for period in periods]
    stops = [period[1] for period in periods]
    return min(starts), max(stops)


def format_datetime_utc(dt: pd.Timestamp) -> str:
    """Format a UTC timestamp with an explicit suffix."""
    return dt.strftime("%Y-%m-%d %H:%M:%S UTC")


__all__ = [
    "mjd_to_datetime",
    "datetime_to_mjd",
    "parse_visibility_periods",
    "parse_optional_mjd",
    "get_time_range",
    "format_datetime_utc",
]
