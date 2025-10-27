"""Pure utilities for Modified Julian Date conversions."""

from __future__ import annotations

import ast
from collections.abc import Sequence
from datetime import datetime, timezone
from typing import cast

import pandas as pd

SECONDS_PER_DAY = 86_400.0
MJD_UNIX_EPOCH = 40_587.0  # Unix epoch (1970-01-01) expressed in MJD


def mjd_to_datetime(mjd: float) -> pd.Timestamp:
    """Convert Modified Julian Date to a timezone-aware UTC timestamp."""

    seconds_since_epoch = (float(mjd) - MJD_UNIX_EPOCH) * SECONDS_PER_DAY
    return pd.to_datetime(seconds_since_epoch, unit="s", utc=True)


def datetime_to_mjd(dt: datetime) -> float:
    """Convert a timezone-aware datetime to Modified Julian Date."""

    if dt.tzinfo is None:
        raise ValueError("Datetime must be timezone-aware")

    timestamp = dt.astimezone(timezone.utc).timestamp()
    return (timestamp / SECONDS_PER_DAY) + MJD_UNIX_EPOCH


def parse_visibility_periods(
    visibility_str: str | None,
) -> list[tuple[pd.Timestamp, pd.Timestamp]]:
    """Parse stringified visibility periods into datetime tuples."""

    if not visibility_str or visibility_str.strip() == "":
        return []

    try:
        periods = ast.literal_eval(visibility_str)
    except (ValueError, SyntaxError):
        return []

    if not isinstance(periods, list):
        return []

    result: list[tuple[pd.Timestamp, pd.Timestamp]] = []
    for period in periods:
        if not isinstance(period, tuple) or len(period) != 2:
            continue
        try:
            start, stop = float(period[0]), float(period[1])
        except (TypeError, ValueError):
            continue
        result.append((mjd_to_datetime(start), mjd_to_datetime(stop)))
    return result


def parse_optional_mjd(value: float | None) -> pd.Timestamp | None:
    """Convert an optional MJD value to datetime."""

    if value is None or (isinstance(value, float) and pd.isna(value)):
        return None
    return mjd_to_datetime(float(value))


def get_time_range(
    periods: Sequence[tuple[pd.Timestamp, pd.Timestamp]],
) -> tuple[pd.Timestamp | None, pd.Timestamp | None]:
    """Return the earliest start and latest stop across *periods*."""

    if not periods:
        return None, None

    starts = [period[0] for period in periods]
    stops = [period[1] for period in periods]
    return min(starts), max(stops)


def format_datetime_utc(dt: pd.Timestamp) -> str:
    """Pretty-print a UTC timestamp."""

    return cast(str, dt.strftime("%Y-%m-%d %H:%M:%S UTC"))
