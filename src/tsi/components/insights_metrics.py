"""Insights page metrics display components."""

from __future__ import annotations

import streamlit as st

from tsi.components.metrics import render_comparison_metrics
from tsi.models.schemas import AnalyticsMetrics


def render_key_metrics(metrics: AnalyticsMetrics) -> None:
    """
    Display key metrics in a three-column layout.

    Args:
        metrics: Computed metrics object
    """
    st.header("📈 Key Metrics")

    col1, col2, col3 = st.columns(3)

    with col1:
        st.metric(
            "Scheduling Rate",
            f"{metrics.scheduling_rate * 100:.1f}%",
            delta=f"{metrics.scheduled_count:,} of {metrics.total_observations:,}",
        )

    with col2:
        st.metric(
            "Mean Priority",
            f"{metrics.mean_priority:.2f}",
            delta=f"Median: {metrics.median_priority:.2f}",
        )

    with col3:
        st.metric(
            "Total Visibility",
            f"{metrics.total_visibility_hours:,.0f} hrs",
        )


def render_priority_analysis(metrics: AnalyticsMetrics) -> None:
    """
    Display priority comparison between scheduled and unscheduled observations.

    Args:
        metrics: Computed metrics object
    """
    st.header("🔍 Priority Analysis")

    render_comparison_metrics(
        label1="Scheduled",
        value1=metrics.mean_priority_scheduled,
        label2="Unscheduled",
        value2=metrics.mean_priority_unscheduled,
        metric_name="Mean Priority",
    )
