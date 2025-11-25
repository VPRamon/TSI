"""Insights page report generation components."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi.models.schemas import AnalyticsMetrics
from tsi.services.report import generate_html_report, generate_markdown_report


def render_report_downloads(
    metrics: AnalyticsMetrics,
    insights: list[str],
    correlations: pd.DataFrame,
    top_priority: pd.DataFrame,
    conflicts: pd.DataFrame,
) -> None:
    """
    Display report download buttons.
    
    Args:
        metrics: Computed metrics object
        insights: List of insight strings
        correlations: Correlation analysis data
        top_priority: Top observations by priority
        conflicts: Detected conflicts
    """
    st.header("📥 Download Report")
    
    col1, col2 = st.columns(2)
    
    with col1:
        # Markdown report
        md_report = generate_markdown_report(
            metrics=metrics,
            insights=insights,
            correlations=correlations,
            top_priority=top_priority,
            conflicts=conflicts,
        )
        
        st.download_button(
            label="📄 Download Markdown Report",
            data=md_report,
            file_name="telescope_scheduling_report.md",
            mime="text/markdown",
            use_container_width=True,
        )
    
    with col2:
        # HTML report
        html_report = generate_html_report(
            metrics=metrics,
            insights=insights,
            correlations=correlations,
            top_priority=top_priority,
            conflicts=conflicts,
        )
        
        st.download_button(
            label="🌐 Download HTML Report",
            data=html_report,
            file_name="telescope_scheduling_report.html",
            mime="text/html",
            use_container_width=True,
        )
    
    st.caption("Reports contain all key metrics, insights, correlations, and conflict information.")
