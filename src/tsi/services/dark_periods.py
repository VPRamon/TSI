"""Utilities for loading and normalizing dark periods data."""

from __future__ import annotations

import io
import json
from pathlib import Path
from typing import Iterable, Sequence

import pandas as pd

from core.time import datetime_to_mjd, mjd_to_datetime

# Candidate keys that may contain the list of dark periods in the JSON payload.
_PERIOD_KEYS = (
    "dark_periods",
    "darkPeriods",
    "dark_period",
    "darkPeriod",
    "periods",
    "DarkPeriods",
)

# Candidate keys for the start and stop timestamps inside each period.
_START_KEYS = (
    "start",
    "startMjd",
    "start_mjd",
    "startTime",
    "start_time",
    "startTimeUtc",
    "startUTC",
    "startUtc",
)
_STOP_KEYS = (
    "stop",
    "stopMjd",
    "stop_mjd",
    "end",
    "endMjd",
    "end_mjd",
    "stopTime",
    "stop_time",
    "stopTimeUtc",
    "stopUTC",
    "stopUtc",
    "endTime",
    "end_time",
)


def load_dark_periods(file_or_buffer: io.IOBase | str | Path | dict | list) -> pd.DataFrame:
    """Load dark periods from a JSON file, buffer or pre-parsed object.

    The CTA scheduling datasets expose *dark periods* as a list of windows where
    observations are allowed. Unfortunately the upstream format is not
    standardised, so this helper accepts a variety of shapes:

    - the JSON root can be a list or contain a list under keys such as
      ``darkPeriods`` or ``dark_periods``;
    - each period can be represented as a dictionary with ``start``/``stop``
      pairs, or as a two-item list/tuple.

    The values may be floats (MJD), strings encoding MJDs, or ISO timestamps.

    Returns a DataFrame with UTC timestamps, MJD values, durations and the list
    of months touched by each period. An empty DataFrame is returned if no
    usable periods are found.
    """

    payload = _read_json_payload(file_or_buffer)
    periods = _extract_periods(payload)

    if not periods:
        return pd.DataFrame(
            columns=[
                "start_dt",
                "stop_dt",
                "start_mjd",
                "stop_mjd",
                "duration_hours",
                "months",
            ]
        )

    df = pd.DataFrame(periods, columns=["start_dt", "stop_dt"])
    df["start_mjd"] = df["start_dt"].apply(datetime_to_mjd)
    df["stop_mjd"] = df["stop_dt"].apply(datetime_to_mjd)
    df["duration_hours"] = (
        (df["stop_dt"] - df["start_dt"]).dt.total_seconds() / 3600.0
    )
    df["months"] = df.apply(
        lambda row: list(_enumerate_months(row["start_dt"], row["stop_dt"])), axis=1
    )

    return df.sort_values("start_dt").reset_index(drop=True)


def _read_json_payload(file_or_buffer: io.IOBase | str | Path | dict | list):
    """Return the parsed JSON payload from ``file_or_buffer``."""

    if isinstance(file_or_buffer, (dict, list)):
        return file_or_buffer

    if isinstance(file_or_buffer, (str, Path)):
        path = Path(file_or_buffer)
        with path.open("r", encoding="utf-8") as handle:
            return json.load(handle)

    if hasattr(file_or_buffer, "read"):
        contents = file_or_buffer.read()
        if isinstance(contents, bytes):
            contents = contents.decode("utf-8")
        if hasattr(file_or_buffer, "seek"):
            file_or_buffer.seek(0)
        return json.loads(contents)

    raise TypeError(f"Unsupported dark periods input: {type(file_or_buffer)}")


def _extract_periods(payload) -> list[tuple[pd.Timestamp, pd.Timestamp]]:
    """Extract a list of (start, stop) timestamps from the payload."""

    raw_periods: Sequence | None = None

    if isinstance(payload, dict):
        for key in _PERIOD_KEYS:
            candidate = payload.get(key)
            if isinstance(candidate, Sequence) and not isinstance(candidate, (str, bytes)):
                raw_periods = candidate
                break

        if raw_periods is None:
            # Fallback: pick the first list value in the dict.
            for value in payload.values():
                if isinstance(value, Sequence) and not isinstance(value, (str, bytes)):
                    raw_periods = value
                    break

    elif isinstance(payload, Sequence) and not isinstance(payload, (str, bytes)):
        raw_periods = payload

    if raw_periods is None:
        return []

    periods: list[tuple[pd.Timestamp, pd.Timestamp]] = []
    for period in raw_periods:
        start_dt, stop_dt = _parse_period(period)
        if start_dt is None or stop_dt is None:
            continue
        if stop_dt <= start_dt:
            continue
        periods.append((start_dt, stop_dt))

    return periods


def _parse_period(period) -> tuple[pd.Timestamp | None, pd.Timestamp | None]:
    """Parse a period entry returning UTC timestamps or ``(None, None)``."""

    start_value = None
    stop_value = None

    if isinstance(period, dict):
        for key in _START_KEYS:
            if key in period:
                start_value = period[key]
                break
        for key in _STOP_KEYS:
            if key in period:
                stop_value = period[key]
                break
    elif isinstance(period, Sequence) and len(period) >= 2 and not isinstance(period, (str, bytes)):
        start_value, stop_value = period[0], period[1]

    start_dt = _parse_time_value(start_value)
    stop_dt = _parse_time_value(stop_value)
    return start_dt, stop_dt


def _parse_time_value(value) -> pd.Timestamp | None:
    """Parse a single timestamp value from various formats."""

    if value is None:
        return None

    # Handle nested dictionary format (e.g., {"format": "MJD", "scale": "UTC", "value": 61771.0})
    if isinstance(value, dict):
        # Try to extract the actual value from nested structure
        if "value" in value:
            value = value["value"]
        elif "mjd" in value:
            value = value["mjd"]
        elif "MJD" in value:
            value = value["MJD"]
        else:
            # If no recognizable key, return None
            return None

    if isinstance(value, (int, float)):
        return mjd_to_datetime(float(value))

    if isinstance(value, str):
        stripped = value.strip()
        if stripped == "":
            return None
        try:
            return mjd_to_datetime(float(stripped))
        except ValueError:
            dt = pd.to_datetime(stripped, utc=True, errors="coerce")
            if pd.isna(dt):
                return None
            if dt.tzinfo is None:
                dt = dt.tz_localize("UTC")
            return dt

    return None


def _enumerate_months(start: pd.Timestamp, stop: pd.Timestamp) -> Iterable[str]:
    """Yield month labels (``YYYY-MM``) touched between ``start`` and ``stop``."""

    current = pd.Timestamp(year=start.year, month=start.month, day=1, tz="UTC")

    # Normalise stop to include the month of the stop timestamp itself.
    end_month = pd.Timestamp(year=stop.year, month=stop.month, day=1, tz="UTC")

    while current <= end_month:
        yield current.strftime("%Y-%m")
        # Advance one month.
        if current.month == 12:
            current = pd.Timestamp(year=current.year + 1, month=1, day=1, tz="UTC")
        else:
            current = pd.Timestamp(year=current.year, month=current.month + 1, day=1, tz="UTC")
