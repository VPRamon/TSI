"""Scheduling trends page metrics display components."""

from __future__ import annotations

from typing import TYPE_CHECKING

import streamlit as st

if TYPE_CHECKING:
    from tsi_rust import TrendsMetrics


def render_overview_metrics(metrics: TrendsMetrics) -> None:
    """
    Display overview metrics for scheduling trends analysis.

    Args:
        metrics: TrendsMetrics from Rust backend
    """
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Total observations", f"{metrics.total_count:,}")

    with col2:
        st.metric("Scheduled", f"{metrics.scheduled_count:,}")

    with col3:
        st.metric("% Scheduled", f"{metrics.scheduling_rate * 100:.1f}%")

    with col4:
        st.metric("Visibility = 0", f"{metrics.zero_visibility_count:,}")
