"""Data preparation helpers previously located in ``core.transformations``."""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from typing import Literal

import numpy as np
import pandas as pd

from tsi.services.utils.time import parse_optional_mjd, parse_visibility_periods

NumericColumns = [
    "priority",
    "minObservationTimeInSec",
    "requestedDurationSec",
    "decInDeg",
    "raInDeg",
    "minAzimuthAngleInDeg",
    "maxAzimuthAngleInDeg",
    "minElevationAngleInDeg",
    "maxElevationAngleInDeg",
    "num_visibility_periods",
    "total_visibility_hours",
    "requested_hours",
    "elevation_range_deg",
]

MJDColumns = [
    "fixedStartTime",
    "fixedStopTime",
    "scheduled_period.start",
    "scheduled_period.stop",
]


@dataclass(frozen=True)
class PreparationResult:
    """Return type bundling the prepared dataframe and warnings."""

    dataframe: pd.DataFrame
    warnings: list[str]


def parse_visibility_for_rows(df: pd.DataFrame, visibility_column: str = "visibility") -> pd.Series:
    """Parse visibility periods lazily using the Rust backend parser."""

    if visibility_column not in df.columns:
        return pd.Series([None] * len(df), index=df.index)

    def _parse(value: object) -> list[tuple[object, object]] | None:
        if value is None or (isinstance(value, float) and pd.isna(value)):
            return None
        try:
            return parse_visibility_periods(str(value))
        except Exception:
            return None

    return df[visibility_column].apply(_parse)


def prepare_dataframe(df: pd.DataFrame) -> PreparationResult:
    """Prepare and enrich a pre-processed scheduling dataframe."""

    prepared = df.copy()
    warnings: list[str] = []

    for column in NumericColumns:
        if column in prepared.columns:
            prepared[column] = pd.to_numeric(prepared[column], errors="coerce")

    for column in MJDColumns:
        if column in prepared.columns:
            prepared[column] = pd.to_numeric(prepared[column], errors="coerce")

    if "scheduled_flag" in prepared.columns and prepared["scheduled_flag"].dtype == object:
        prepared["scheduled_flag"] = (
            prepared["scheduled_flag"]
            .map(lambda x: str(x).lower() == "true" if pd.notna(x) else False)
            .astype(bool)
        )

    if "visibility" in prepared.columns:
        prepared["visibility_periods_parsed"] = None

    if "fixedStartTime" in prepared.columns:
        prepared["fixed_start_dt"] = prepared["fixedStartTime"].apply(parse_optional_mjd)
    if "fixedStopTime" in prepared.columns:
        prepared["fixed_stop_dt"] = prepared["fixedStopTime"].apply(parse_optional_mjd)
    if "scheduled_period.start" in prepared.columns:
        prepared["scheduled_start_dt"] = prepared["scheduled_period.start"].apply(parse_optional_mjd)
    if "scheduled_period.stop" in prepared.columns:
        prepared["scheduled_stop_dt"] = prepared["scheduled_period.stop"].apply(parse_optional_mjd)

    return PreparationResult(dataframe=prepared, warnings=warnings)


def validate_schema(
    df: pd.DataFrame,
    *,
    required_columns: set[str],
    expected_dtypes: dict[str, str] | None = None,
) -> tuple[bool, list[str]]:
    """Validate that *df* contains the requested columns and dtypes."""

    errors: list[str] = []
    missing = required_columns - set(df.columns)
    if missing:
        errors.append(f"Missing columns: {sorted(missing)}")

    if expected_dtypes:
        for column, expected in expected_dtypes.items():
            if column not in df.columns:
                continue
            actual = str(df[column].dtype)
            if actual != expected:
                errors.append(f"Column '{column}' has dtype {actual}, expected {expected}")

    return not errors, errors


def validate_dataframe(df: pd.DataFrame) -> tuple[bool, list[str]]:
    """Perform lightweight validation on the prepared dataframe."""

    issues: list[str] = []
    if "schedulingBlockId" in df.columns and df["schedulingBlockId"].isna().any():
        issues.append("Some rows have missing schedulingBlockId")

    if "priority" in df.columns:
        priority_values = pd.to_numeric(df["priority"], errors="coerce")
        invalid_mask = ~np.isfinite(priority_values)
        if invalid_mask.any():
            issues.append(f"{invalid_mask.sum()} rows have invalid priority values")

    if "decInDeg" in df.columns:
        invalid_dec = df[(df["decInDeg"] < -90) | (df["decInDeg"] > 90)]
        if not invalid_dec.empty:
            issues.append(f"{len(invalid_dec)} rows have invalid declination")

    if "raInDeg" in df.columns:
        invalid_ra = df[(df["raInDeg"] < 0) | (df["raInDeg"] >= 360)]
        if not invalid_ra.empty:
            issues.append(f"{len(invalid_ra)} rows have invalid right ascension")

    return len(issues) == 0, issues


def filter_dataframe(
    df: pd.DataFrame,
    *,
    priority_range: tuple[float, float] = (0.0, 10.0),
    scheduled_filter: Literal["All", "Scheduled", "Unscheduled"] = "All",
    priority_bins: Sequence[str] | None = None,
    block_ids: Sequence[str | int] | None = None,
) -> pd.DataFrame:
    """Return a filtered view of *df* according to UI criteria."""

    min_priority, max_priority = priority_range
    filtered = df[(df["priority"] >= min_priority) & (df["priority"] <= max_priority)]

    if scheduled_filter == "Scheduled":
        filtered = filtered[filtered["scheduled_flag"]]
    elif scheduled_filter == "Unscheduled":
        filtered = filtered[~filtered["scheduled_flag"]]

    if priority_bins:
        filtered = filtered[filtered["priority_bin"].isin(priority_bins)]

    if block_ids:
        # Filter by block IDs - handle both string and int types
        filtered = filtered[filtered["schedulingBlockId"].isin(block_ids)]

    return filtered


__all__ = [
    "PreparationResult",
    "parse_visibility_for_rows",
    "prepare_dataframe",
    "validate_schema",
    "validate_dataframe",
    "filter_dataframe",
]
