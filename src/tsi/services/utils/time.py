"""Lightweight time helpers backed by the Rust backend."""

from __future__ import annotations

from collections.abc import Sequence
from datetime import datetime

import pandas as pd

try:
    from tsi_rust import ModifiedJulianDate
except ImportError as exc:  # pragma: no cover - enforced by build setup
    raise ImportError(
        "ModifiedJulianDate not available from tsi_rust. "
        "Please compile the Rust backend with: maturin develop --release"
    ) from exc

from tsi_rust_api import TSIBackend


def mjd_to_datetime(mjd: float | ModifiedJulianDate) -> pd.Timestamp:
    """
    Convert Modified Julian Date to a pandas Timestamp (UTC).

    Delegates to the Rust backend for accurate, fast conversion.
    """
    mjd_val = _mjd_value(mjd)
    dt = TSIBackend.mjd_to_datetime(mjd_val)
    return pd.Timestamp(dt)


def datetime_to_mjd(dt: datetime) -> float:
    """
    Convert a timezone-aware datetime to Modified Julian Date.

    Delegates to the Rust backend for accurate, fast conversion.
    """
    return TSIBackend.datetime_to_mjd(dt)


def _mjd_value(mjd: float | ModifiedJulianDate) -> float:
    """Normalize float-like or Rust-backed ModifiedJulianDate values."""
    if isinstance(mjd, ModifiedJulianDate):
        try:
            return float(mjd)
        except TypeError:
            value = getattr(mjd, "value", None)
            if value is not None:
                return float(value)
            raise
    return float(mjd)


def parse_visibility_periods(visibility_str: str) -> list[tuple[pd.Timestamp, pd.Timestamp]]:
    """Parse visibility periods into timestamp tuples.

    Expected format: "[(mjd_start, mjd_stop), (mjd_start, mjd_stop), ...]"
    """
    if not visibility_str or str(visibility_str).strip() in ("", "[]"):
        return []

    try:
        # Parse the string representation of a list of tuples
        import ast

        parsed = ast.literal_eval(str(visibility_str))

        if not isinstance(parsed, list):
            return []

        result = []
        for item in parsed:
            if isinstance(item, (tuple, list)) and len(item) == 2:
                start_mjd, stop_mjd = item
                # Convert MJD values to timestamps using shared MJD helpers
                start_ts = mjd_to_datetime(float(start_mjd))
                stop_ts = mjd_to_datetime(float(stop_mjd))
                result.append((start_ts, stop_ts))

        return result
    except Exception:
        return []


def parse_optional_mjd(value: float | ModifiedJulianDate | None) -> pd.Timestamp | None:
    """Convert an optional MJD value to a Timestamp, preserving missing values."""
    if value is None:
        return None

    if isinstance(value, float) and pd.isna(value):
        return None

    return mjd_to_datetime(_mjd_value(value))


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
