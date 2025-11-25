"""Insights and conclusions page with analytics."""

import streamlit as st

from tsi import state
from tsi.components.insights_controls import render_filter_controls
from tsi.components.insights_metrics import render_key_metrics, render_priority_analysis
from tsi.components.insights_analysis import render_automated_insights, render_correlation_analysis
from tsi.components.insights_tables import render_top_observations, render_integrity_checks
from tsi.components.insights_reports import render_report_downloads
from tsi.services.insights_filtering import (
    check_filter_support,
    compute_impossible_mask,
    apply_insights_filter,
)
from tsi.services.analytics import (
    compute_correlations,
    compute_metrics,
    find_conflicts,
    get_top_observations,
    generate_insights,
)


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

    df = state.get_prepared_data()

    if df is None:
        st.warning("No data loaded. Please return to the landing page.")
        return

    # Check if filtering is supported and compute impossible mask
    filter_supported = check_filter_support(df)
    impossible_mask = compute_impossible_mask(df) if filter_supported else None

    # Render filter controls
    with col2:
        filter_mode = render_filter_controls(filter_supported)

    # Apply filter based on user selection
    filtered_df = apply_insights_filter(df, filter_mode, impossible_mask)

    if filtered_df.empty:
        st.warning("⚠️ No observations available with the selected filter.")
        return

    # Compute analytics
    with st.spinner("Computing analytics..."):
        metrics = compute_metrics(filtered_df)
        insights = generate_insights(filtered_df, metrics)
        correlations = compute_correlations(filtered_df)
        top_priority = get_top_observations(filtered_df, by="priority", n=10)
        top_visibility = get_top_observations(filtered_df, by="total_visibility_hours", n=10)
        conflicts = find_conflicts(filtered_df)

    # Key Metrics
    render_key_metrics(metrics)
    st.divider()

    # Priority comparison
    render_priority_analysis(metrics)
    st.divider()

    # Insights
    render_automated_insights(insights)
    st.divider()

    # Correlation analysis
    render_correlation_analysis(correlations)
    st.divider()

    # Top observations
    render_top_observations(top_priority, top_visibility)
    st.divider()

    # Integrity checks
    render_integrity_checks(conflicts)
    st.divider()

    # Report generation
    render_report_downloads(metrics, insights, correlations, top_priority, conflicts)
