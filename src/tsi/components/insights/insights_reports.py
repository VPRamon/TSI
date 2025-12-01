"""Insights page report generation components."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd
import streamlit as st

from tsi.models.schemas import AnalyticsMetrics
from tsi.services.utils.report import generate_html_report, generate_markdown_report

if TYPE_CHECKING:
    from typing import Any


def render_report_downloads(
    metrics: AnalyticsMetrics | Any,
    insights: list[str],
    correlations: list[Any],
    top_priority: list[Any],
    conflicts: list[Any],
) -> None:
    """
    Display report download buttons.

    Args:
        metrics: Computed metrics object (AnalyticsMetrics from Rust or Python)
        insights: List of insight strings
        correlations: List of CorrelationEntry objects
        top_priority: List of TopObservation objects
        conflicts: List of ConflictRecord objects
    """
    st.header("📥 Download Report")

    # Convert Rust data structures to pandas DataFrames for report generation
    if correlations:
        # Convert correlations to matrix format
        variables = set()
        for corr in correlations:
            variables.add(corr.variable1)
            variables.add(corr.variable2)
        
        variables = sorted(variables)
        corr_matrix = pd.DataFrame(
            1.0,  # diagonal is always 1
            index=variables,
            columns=variables,
        )
        
        for corr in correlations:
            corr_matrix.loc[corr.variable1, corr.variable2] = corr.correlation
            corr_matrix.loc[corr.variable2, corr.variable1] = corr.correlation
    else:
        corr_matrix = pd.DataFrame()

    # Convert top priority to DataFrame
    if top_priority:
        top_priority_df = pd.DataFrame([
            {
                "scheduling_block_id": obs.scheduling_block_id,
                "priority": obs.priority,
                "total_visibility_hours": obs.total_visibility_hours,
                "requested_hours": obs.requested_hours,
                "scheduled": obs.scheduled,
            }
            for obs in top_priority
        ])
    else:
        top_priority_df = pd.DataFrame()

    # Convert conflicts to DataFrame
    if conflicts:
        conflicts_df = pd.DataFrame([
            {
                "block_id_1": conflict.block_id_1,
                "block_id_2": conflict.block_id_2,
                "start_time_1": conflict.start_time_1,
                "stop_time_1": conflict.stop_time_1,
                "start_time_2": conflict.start_time_2,
                "stop_time_2": conflict.stop_time_2,
                "overlap_hours": conflict.overlap_hours,
            }
            for conflict in conflicts
        ])
    else:
        conflicts_df = pd.DataFrame()

    col1, col2 = st.columns(2)

    with col1:
        # Markdown report
        md_report = generate_markdown_report(
            metrics=metrics,
            insights=insights,
            correlations=corr_matrix,
            top_priority=top_priority_df,
            conflicts=conflicts_df,
        )

        st.download_button(
            label="📄 Download Markdown Report",
            data=md_report,
            file_name="telescope_scheduling_report.md",
            mime="text/markdown",
            width='stretch',
        )

    with col2:
        # HTML report
        html_report = generate_html_report(
            metrics=metrics,
            insights=insights,
            correlations=corr_matrix,
            top_priority=top_priority_df,
            conflicts=conflicts_df,
        )

        st.download_button(
            label="🌐 Download HTML Report",
            data=html_report,
            file_name="telescope_scheduling_report.html",
            mime="text/html",
            width='stretch',
        )

    st.caption("Reports contain all key metrics, insights, correlations, and conflict information.")
