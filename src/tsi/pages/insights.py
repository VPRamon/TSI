"""Insights and conclusions page with analytics."""

import streamlit as st

from tsi import state
from tsi.components.insights.insights_analysis import render_automated_insights, render_correlation_analysis
from tsi.components.insights.insights_controls import render_filter_controls
from tsi.components.insights.insights_metrics import render_key_metrics, render_priority_analysis
from tsi.components.insights.insights_reports import render_report_downloads
from tsi.components.insights.insights_tables import render_integrity_checks, render_top_observations
from tsi.components.shared.filters import render_exclude_impossible_checkbox
from tsi.services.data.analytics import generate_insights
from tsi.services.database import get_insights_data


def render() -> None:
    """Render the Insights & Conclusions page."""
    # Title and filter controls in same row
    col1, col2 = st.columns([3, 1])

    with col1:
        st.title("💡 Insights & Conclusions")

    st.markdown(
        """
        Automated analytics, correlations, and scheduling integrity analysis
        with downloadable reports.
        """
    )

    schedule_id = state.get_schedule_id()

    if schedule_id is None:
        st.info("Load a schedule from the database to view insights.")
        return

    schedule_id = int(schedule_id)

    # Render filter control using shared component
    with col2:
        filter_impossible = render_exclude_impossible_checkbox(
            help_text="Exclude observations with zero visibility hours"
        )

    try:
        with st.spinner("Loading insights data..."):
            insights_data = get_insights_data(
                schedule_id=schedule_id,
                filter_impossible=filter_impossible
            )
    except Exception as exc:
        st.error(f"Failed to load insights data from the backend: {exc}")
        return

    if insights_data.total_count == 0:
        st.warning("⚠️ No observations available with the selected filter.")
        return

    # Generate insights from metrics
    insights = generate_insights(insights_data.blocks, insights_data.metrics)

    # Key Metrics
    render_key_metrics(insights_data.metrics)
    st.divider()

    # Priority comparison
    render_priority_analysis(insights_data.metrics)
    st.divider()

    # Insights
    render_automated_insights(insights)
    st.divider()

    # Correlation analysis
    render_correlation_analysis(insights_data.correlations)
    st.divider()

    # Top observations
    render_top_observations(insights_data.top_priority, insights_data.top_visibility)
    st.divider()

    # Integrity checks
    render_integrity_checks(insights_data.conflicts)
    st.divider()

    # Report generation
    render_report_downloads(
        insights_data.metrics,
        insights,
        insights_data.correlations,
        insights_data.top_priority,
        insights_data.conflicts
    )
