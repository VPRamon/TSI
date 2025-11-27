"""Data preparation helpers used by adapters."""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from typing import Literal

import numpy as np
import pandas as pd

from tsi.services.rust_compat import (
    parse_optional_mjd_rust as parse_optional_mjd,
    parse_visibility_periods_rust as parse_visibility_periods,
)


@dataclass(frozen=True)
class PreparationResult:
    """Return type bundling the prepared dataframe and warnings."""

    dataframe: pd.DataFrame
    warnings: list[str]


def parse_visibility_for_rows(df: pd.DataFrame, visibility_column: str = "visibility") -> pd.Series:
    """
    Parse visibility periods for a subset of rows on-demand.

    Use this function when you need parsed visibility data for specific rows
    (e.g., after filtering). This avoids parsing all 2647 rows during initial load.

    Args:
        df: DataFrame with visibility column (string format)
        visibility_column: Name of the visibility column to parse

    Returns:
        pd.Series with parsed visibility periods (list of tuples)

    Example:
        # After filtering to 10 rows
        filtered_df = df[df['scheduled_flag']].head(10)
        filtered_df['visibility_periods_parsed'] = parse_visibility_for_rows(filtered_df)
    """
    if visibility_column not in df.columns:
        return pd.Series([None] * len(df), index=df.index)

    result = df[visibility_column].apply(
        lambda x: parse_visibility_periods(x) if pd.notna(x) else None
    )
    return result  # type: ignore[return-value,no-any-return]


def prepare_dataframe(df: pd.DataFrame) -> PreparationResult:
    """
    Prepare and enrich a pre-processed scheduling dataframe.

    This function assumes the CSV has been pre-processed with all derived columns
    (scheduled_flag, requested_hours, elevation_range_deg, etc.).

    It only performs lightweight operations:
    - Type coercion for numeric columns
    - Parsing visibility periods for interactive use
    - Converting MJD times to datetime objects

    Args:
        df: Pre-processed DataFrame with all derived columns

    Returns:
        PreparationResult with prepared DataFrame and warnings
    """
    prepared = df.copy()
    warnings: list[str] = []

    # Ensure numeric types (lightweight, always do this)
    numeric_columns = [
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
    for column in numeric_columns:
        if column in prepared.columns:
            prepared[column] = pd.to_numeric(prepared[column], errors="coerce")

    mjd_columns = [
        "fixedStartTime",
        "fixedStopTime",
        "scheduled_period.start",
        "scheduled_period.stop",
    ]
    for column in mjd_columns:
        if column in prepared.columns:
            prepared[column] = pd.to_numeric(prepared[column], errors="coerce")

    # Convert scheduled_flag to boolean if it's a string
    if "scheduled_flag" in prepared.columns:
        if prepared["scheduled_flag"].dtype == object:
            prepared["scheduled_flag"] = (
                prepared["scheduled_flag"]
                .map(lambda x: str(x).lower() == "true" if pd.notna(x) else False)
                .astype(bool)
            )

    # NOTE: visibility_periods_parsed parsing is DISABLED during initial load
    # for performance (parsing ~317k MJD conversions takes 3-4 seconds).
    # Pages that need parsed visibility should call parse_visibility_periods()
    # on-demand for the filtered subset of rows they're displaying.
    # This reduces load time from ~4.5s to ~0.5s.

    # Set as None/empty - pages can parse lazily if needed
    if "visibility" in prepared.columns:
        prepared["visibility_periods_parsed"] = None

    # NOTE: Datetime columns (fixed_start_dt, scheduled_start_dt, etc.) are created
    # on-demand by pages that need them, not during preparation, to avoid conflicts
    # with the Rust backend which expects specific column types.

    # Add datetime columns (lightweight conversion)
    if "fixedStartTime" in prepared.columns:
        prepared["fixed_start_dt"] = prepared["fixedStartTime"].apply(parse_optional_mjd)
    if "fixedStopTime" in prepared.columns:
        prepared["fixed_stop_dt"] = prepared["fixedStopTime"].apply(parse_optional_mjd)
    if "scheduled_period.start" in prepared.columns:
        prepared["scheduled_start_dt"] = prepared["scheduled_period.start"].apply(parse_optional_mjd)
    if "scheduled_period.stop" in prepared.columns:
        prepared["scheduled_stop_dt"] = prepared["scheduled_period.stop"].apply(parse_optional_mjd)

    return PreparationResult(dataframe=prepared, warnings=warnings)


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
