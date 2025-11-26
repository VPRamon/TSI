"""Analytics computations shared between adapters."""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from typing import cast

import pandas as pd

from core.time import format_datetime_utc
from tsi_rust_api import TSIBackend


@dataclass(frozen=True)
class AnalyticsSnapshot:
    """Thin dataclass capturing dataset-level aggregates."""

    total_observations: int
    scheduled_count: int
    unscheduled_count: int
    scheduling_rate: float
    mean_priority: float
    median_priority: float
    mean_priority_scheduled: float
    mean_priority_unscheduled: float
    total_visibility_hours: float
    mean_requested_hours: float


@dataclass(frozen=True)
class CandidatePlacement:
    """Describe a hypothetical scheduling position for an observation."""

    window_start: pd.Timestamp
    window_stop: pd.Timestamp
    candidate_start: pd.Timestamp
    candidate_end: pd.Timestamp
    anchor: str
    conflicts: tuple[str, ...]


_BACKEND = TSIBackend(use_pandas=True)


def compute_metrics(df: pd.DataFrame) -> AnalyticsSnapshot:
    """Compute dataset-level summary statistics via the Rust backend."""

    rust_metrics = _BACKEND.compute_metrics(df)
    return AnalyticsSnapshot(**rust_metrics)


def compute_correlations(df: pd.DataFrame, *, columns: Sequence[str]) -> pd.DataFrame:
    """Compute a Spearman correlation matrix for the selected columns."""

    cols_to_analyze = [col for col in columns if col in df.columns]
    if len(cols_to_analyze) < 2:
        return pd.DataFrame()

    return df[cols_to_analyze].dropna().corr(method="spearman")


def get_top_observations(df: pd.DataFrame, *, by: str, n: int = 10) -> pd.DataFrame:
    """Return the top *n* rows ordered by *by* using the Rust backend."""

    return cast(pd.DataFrame, _BACKEND.get_top_observations(df, by=by, n=n))


def find_conflicts(df: pd.DataFrame) -> pd.DataFrame:
    """Detect scheduling conflicts using the Rust backend."""

    # Check if required columns for conflict detection are present
    required_cols = {"scheduled_start_dt", "scheduled_stop_dt"}
    if not required_cols.issubset(df.columns):
        # Cannot detect conflicts without scheduled datetime columns
        return pd.DataFrame()

    try:
        return cast(pd.DataFrame, _BACKEND.find_conflicts(df))
    except Exception:
        # If Rust backend fails (e.g., due to dtype issues), return empty conflicts
        return pd.DataFrame()


def _get_duration_timedelta(row: pd.Series) -> pd.Timedelta | None:
    """Return the requested observation duration as a timedelta."""

    duration_seconds = row.get("requestedDurationSec")
    if pd.notna(duration_seconds):
        try:
            return pd.to_timedelta(float(duration_seconds), unit="s")
        except (TypeError, ValueError):
            pass

    duration_hours = row.get("requested_hours")
    if pd.notna(duration_hours):
        try:
            return pd.to_timedelta(float(duration_hours), unit="h")
        except (TypeError, ValueError):
            pass

    return None


def _build_conflicts(
    row: pd.Series,
    candidate_start: pd.Timestamp,
    candidate_end: pd.Timestamp,
    window_start: pd.Timestamp,
    window_stop: pd.Timestamp,
    scheduled_df: pd.DataFrame,
) -> list[str]:
    """Identify all constraint violations for a hypothetical placement."""

    conflicts: list[str] = []

    if candidate_start < window_start:
        conflicts.append("Start before visibility window")
    if candidate_end > window_stop:
        conflicts.append(
            "End outside visibility window "
            f"({format_datetime_utc(candidate_end)} > {format_datetime_utc(window_stop)})"
        )

    fixed_start = row.get("fixed_start_dt")
    if pd.notna(fixed_start) and candidate_start < fixed_start:
        conflicts.append(
            "Violates fixed start "
            f"({format_datetime_utc(candidate_start)} < {format_datetime_utc(fixed_start)})"
        )

    fixed_stop = row.get("fixed_stop_dt")
    if pd.notna(fixed_stop) and candidate_end > fixed_stop:
        conflicts.append(
            "Violates fixed end "
            f"({format_datetime_utc(candidate_end)} > {format_datetime_utc(fixed_stop)})"
        )

    if not scheduled_df.empty:
        overlaps_mask = (candidate_start < scheduled_df["scheduled_stop_dt"]) & (
            candidate_end > scheduled_df["scheduled_start_dt"]
        )
        overlapping = scheduled_df[overlaps_mask]

        if not overlapping.empty:
            max_details = 3
            for _, other in overlapping.head(max_details).iterrows():
                conflicts.append(
                    "Solapa con bloque "
                    f"{other.get('schedulingBlockId')} "
                    f"({format_datetime_utc(other['scheduled_start_dt'])} - "
                    f"{format_datetime_utc(other['scheduled_stop_dt'])})"
                )

            remaining = len(overlapping) - max_details
            if remaining > 0:
                conflicts.append(f"… y {remaining} conflictos adicionales")

    return conflicts


def suggest_candidate_positions(df: pd.DataFrame, row: pd.Series) -> list[CandidatePlacement]:
    """Suggest feasible positions for an unscheduled observation."""

    visibility_periods = row.get("visibility_periods_parsed")
    if not visibility_periods:
        return []

    duration = _get_duration_timedelta(row)
    if duration is None or duration <= pd.Timedelta(0):
        return []

    scheduled_df = df[df["scheduled_flag"]].copy()
    if not scheduled_df.empty:
        scheduled_df = scheduled_df[
            scheduled_df["scheduled_start_dt"].notna() & scheduled_df["scheduled_stop_dt"].notna()
        ]

    candidates: dict[tuple[pd.Timestamp, pd.Timestamp], CandidatePlacement] = {}

    for window_start, window_stop in visibility_periods:
        if pd.isna(window_start) or pd.isna(window_stop):
            continue

        candidate_start = window_start
        candidate_end = candidate_start + duration
        conflicts = _build_conflicts(
            row,
            candidate_start,
            candidate_end,
            window_start,
            window_stop,
            scheduled_df,
        )
        placement = CandidatePlacement(
            window_start=window_start,
            window_stop=window_stop,
            candidate_start=candidate_start,
            candidate_end=candidate_end,
            anchor="Window start",
            conflicts=tuple(conflicts),
        )
        candidates[(window_start, candidate_start)] = placement

        if scheduled_df.empty:
            continue

        in_window_mask = (scheduled_df["scheduled_stop_dt"] <= window_stop) & (
            scheduled_df["scheduled_stop_dt"] >= window_start
        )
        scheduled_in_window = scheduled_df[in_window_mask]

        for _, other in scheduled_in_window.iterrows():
            candidate_start = other["scheduled_stop_dt"]
            candidate_end = candidate_start + duration
            conflicts = _build_conflicts(
                row,
                candidate_start,
                candidate_end,
                window_start,
                window_stop,
                scheduled_df,
            )
            placement = CandidatePlacement(
                window_start=window_start,
                window_stop=window_stop,
                candidate_start=candidate_start,
                candidate_end=candidate_end,
                anchor=("After block " f"{other.get('schedulingBlockId')}"),
                conflicts=tuple(conflicts),
            )
            candidates[(window_start, candidate_start)] = placement

    ordered_keys = sorted(candidates.keys())
    return [candidates[key] for key in ordered_keys]


def compute_distribution_stats(series: pd.Series) -> dict[str, float]:
    """Compute descriptive statistics for a numeric series."""

    clean_series = series.dropna()
    if clean_series.empty:
        return {}

    return {
        "mean": float(clean_series.mean()),
        "median": float(clean_series.median()),
        "std": float(clean_series.std()),
        "min": float(clean_series.min()),
        "max": float(clean_series.max()),
        "q25": float(clean_series.quantile(0.25)),
        "q75": float(clean_series.quantile(0.75)),
        "count": int(len(clean_series)),
    }


def generate_insights(df: pd.DataFrame, metrics: AnalyticsSnapshot) -> list[str]:
    """Generate textual insights from the dataset."""

    insights = [
        (
            f"**Scheduling Rate**: {metrics.scheduling_rate * 100:.1f}% "
            f"({metrics.scheduled_count:,} of {metrics.total_observations:,}) observations scheduled."
        ),
    ]

    if metrics.mean_priority_scheduled > 0 and metrics.mean_priority_unscheduled > 0:
        diff = metrics.mean_priority_scheduled - metrics.mean_priority_unscheduled
        if abs(diff) > 0.5:
            direction = "higher" if diff > 0 else "lower"
            insights.append(
                f"**Priority Bias**: Scheduled observations have {direction} average priority "
                f"({metrics.mean_priority_scheduled:.2f}) vs unscheduled "
                f"({metrics.mean_priority_unscheduled:.2f})."
            )

    if metrics.total_visibility_hours > 0:
        insights.append(
            f"**Total Visibility**: {metrics.total_visibility_hours:,.0f} cumulative visibility hours."
        )

    corr_matrix = compute_correlations(
        df,
        columns=[
            "priority",
            "requested_hours",
            "total_visibility_hours",
            "elevation_range_deg",
        ],
    )
    if not corr_matrix.empty and "priority" in corr_matrix:
        for column in corr_matrix.columns:
            if column == "priority":
                continue
            corr_val = corr_matrix.loc["priority", column]
            # Type guard for correlation value
            if isinstance(corr_val, (int, float)) and abs(float(corr_val)) > 0.3:
                direction = "positive" if float(corr_val) > 0 else "negative"
                insights.append(
                    f"**Correlation**: Priority has {direction} correlation ({corr_val:.2f}) with {column}."
                )

    conflicts = find_conflicts(df)
    if not conflicts.empty:
        insights.append(
            f"**Integrity Issues**: {len(conflicts)} scheduled observations have conflicts."
        )

    return insights
