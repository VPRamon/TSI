"""Analytics computations shared between adapters."""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from typing import cast

import pandas as pd

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


_BACKEND = TSIBackend(use_pandas=True)


def compute_correlations(df: pd.DataFrame, *, columns: Sequence[str]) -> pd.DataFrame:
    """Compute a Spearman correlation matrix for the selected columns."""

    cols_to_analyze = [col for col in columns if col in df.columns]
    if len(cols_to_analyze) < 2:
        return pd.DataFrame()

    return df[cols_to_analyze].dropna().corr(method="spearman")


def find_conflicts(df: pd.DataFrame) -> pd.DataFrame:
    """Detect scheduling conflicts using the Rust backend."""

    required_cols = {"scheduled_start_dt", "scheduled_stop_dt"}
    if not required_cols.issubset(df.columns):
        return pd.DataFrame()

    try:
        return cast(pd.DataFrame, _BACKEND.find_conflicts(df))
    except Exception:
        return pd.DataFrame()


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
