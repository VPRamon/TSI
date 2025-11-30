"""Lightweight time helpers backed by the Rust backend."""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from typing import Any

import pandas as pd

from tsi_rust_api import TSIBackend


MJD_EPOCH = datetime(1858, 11, 17, tzinfo=timezone.utc)
SECONDS_PER_DAY = 86400.0


def _ensure_utc(dt: datetime) -> datetime:
    """Ensure datetime objects are timezone-aware in UTC."""
    if dt.tzinfo is None:
        return dt.replace(tzinfo=timezone.utc)
    return dt.astimezone(timezone.utc)


@dataclass(frozen=True)
class ModifiedJulianDate:
    """Lightweight Modified Julian Date representation with datetime helpers."""

    value: float

    def to_datetime(self) -> datetime:
        """Convert to a timezone-aware UTC datetime."""
        return MJD_EPOCH + timedelta(days=self.value)

    def to_timestamp(self) -> pd.Timestamp:
        """Convert to pandas Timestamp."""
        return pd.Timestamp(self.to_datetime())

    @classmethod
    def from_datetime(cls, dt: datetime) -> ModifiedJulianDate:
        """Create an MJD from a Python datetime."""
        dt_utc = _ensure_utc(dt)
        delta = dt_utc - MJD_EPOCH
        return cls(delta.total_seconds() / SECONDS_PER_DAY)

    @classmethod
    def from_timestamp(cls, ts: pd.Timestamp) -> ModifiedJulianDate:
        """Create an MJD from a pandas Timestamp."""
        if ts.tzinfo is None:
            ts = ts.tz_localize("UTC")
        else:
            ts = ts.tz_convert("UTC")
        return cls.from_datetime(ts.to_pydatetime())

    def __float__(self) -> float:
        return float(self.value)


def mjd_to_datetime(mjd: float) -> pd.Timestamp:
    """Convert Modified Julian Date to a pandas Timestamp (UTC)."""
    return ModifiedJulianDate(mjd).to_timestamp()


def datetime_to_mjd(dt: datetime) -> float:
    """Convert a timezone-aware datetime to Modified Julian Date."""
    return float(ModifiedJulianDate.from_datetime(dt).value)


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
    return ModifiedJulianDate(float(value)).to_timestamp()


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
    "ModifiedJulianDate",
    "mjd_to_datetime",
    "datetime_to_mjd",
    "parse_visibility_periods",
    "parse_optional_mjd",
    "get_time_range",
    "format_datetime_utc",
]
