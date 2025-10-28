"""Insights and conclusions page with analytics."""

import streamlit as st

from tsi import state
from tsi.components.data_preview import render_conflicts_table, render_data_preview
from tsi.components.metrics import render_comparison_metrics
from tsi.plots.distributions import build_correlation_heatmap
from tsi.services.analytics import (
    compute_correlations,
    compute_metrics,
    find_conflicts,
    generate_correlation_insights,
    generate_insights,
    get_top_observations,
)
from tsi.services.report import generate_html_report, generate_markdown_report

FILTER_OPTIONS = ("all", "exclude_impossible")
FILTER_LABELS = {
    "all": "📋 All blocks",
    "exclude_impossible": "✅ Filter invalid requests",
}


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

    filtered_df = df

    # Initialize session state only if not present
    if state.KEY_INSIGHTS_FILTER_MODE not in st.session_state:
        st.session_state[state.KEY_INSIGHTS_FILTER_MODE] = "all"

    filter_supported = "total_visibility_hours" in df.columns and (
        "minObservationTimeInSec" in df.columns or "requested_hours" in df.columns
    )
    impossible_mask = None

    if filter_supported:
        TOLERANCE_SEC = 1
        visibility_secs = df["total_visibility_hours"] * 3600.0

        # Check both minimum observation time and requested duration
        impossible_conditions = []

        if "minObservationTimeInSec" in df.columns:
            min_duration_secs = df["minObservationTimeInSec"].fillna(0)
            impossible_conditions.append(
                (min_duration_secs - TOLERANCE_SEC > visibility_secs).fillna(False)
            )

        if "requested_hours" in df.columns:
            requested_secs = df["requested_hours"] * 3600.0
            impossible_conditions.append(
                (requested_secs - TOLERANCE_SEC > visibility_secs).fillna(False)
            )

        # An observation is impossible if ANY of the conditions is true
        if impossible_conditions:
            impossible_mask = impossible_conditions[0]
            for condition in impossible_conditions[1:]:
                impossible_mask = impossible_mask | condition

    # Filter controls aligned to the right, vertically stacked
    filter_mode = "all"
    if filter_supported:
        with col2:
            # Add empty space to align vertically with title
            st.markdown("<div style='margin-top: 1.5rem;'></div>", unsafe_allow_html=True)
            filter_mode = st.radio(
                "Filtrar:",
                options=FILTER_OPTIONS,
                format_func=lambda x: FILTER_LABELS[x],
                key=state.KEY_INSIGHTS_FILTER_MODE,
                horizontal=False,
                label_visibility="collapsed",
            )
    else:
        st.session_state[state.KEY_INSIGHTS_FILTER_MODE] = "all"

    # Apply filter based on user selection
    if filter_supported and impossible_mask is not None:
        if filter_mode == "exclude_impossible":
            filtered_df = df[~impossible_mask]

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

    st.divider()

    # Priority comparison
    st.header("🔍 Priority Analysis")

    render_comparison_metrics(
        label1="Scheduled",
        value1=metrics.mean_priority_scheduled,
        label2="Unscheduled",
        value2=metrics.mean_priority_unscheduled,
        metric_name="Mean Priority",
    )

    st.divider()

    # Insights
    st.header("✨ Automated Insights")

    for insight in insights:
        st.markdown(f"- {insight}")

    st.divider()

    # Correlation analysis
    st.header("📊 Correlation Analysis")

    if not correlations.empty:
        # Generate correlation insights
        correlation_insights = generate_correlation_insights(correlations)

        col1, col2 = st.columns([1, 2])

        with col1:
            st.markdown("**Correlation Interpretation**")
            st.markdown("---")
            for insight in correlation_insights:
                st.markdown(insight)
                st.markdown("")  # Add spacing

            st.caption(
                """
                **Technical note:** Spearman correlation values (ρ) range from -1 to +1:
                - **ρ > 0**: Positive correlation (both variables increase together)
                - **ρ < 0**: Negative correlation (inverse relationship)
                - **|ρ| ≥ 0.7**: Strong correlation
                - **0.4 ≤ |ρ| < 0.7**: Moderate correlation
                - **|ρ| < 0.4**: Weak correlation

                Spearman correlation is robust to outliers and does not assume linearity.
                """
            )

        with col2:
            fig = build_correlation_heatmap(correlations)
            st.plotly_chart(fig, use_container_width=True)
    else:
        st.info("Insufficient data for correlation analysis")

    st.divider()

    # Top observations
    st.header("🏆 Top Observations")

    tab1, tab2 = st.tabs(["By Priority", "By Visibility Hours"])

    with tab1:
        if not top_priority.empty:
            render_data_preview(
                top_priority,
                max_rows=10,
                title="Top 10 by Priority",
            )
        else:
            st.info("No data available")

    with tab2:
        if not top_visibility.empty:
            render_data_preview(
                top_visibility,
                max_rows=10,
                title="Top 10 by Total Visibility Hours",
            )
        else:
            st.info("No data available")

    st.divider()

    # Integrity checks
    st.header("🔎 Scheduling Integrity")

    render_conflicts_table(conflicts)

    if not conflicts.empty:
        st.warning(
            """
            **Conflict Types:**
            - **Outside visibility**: Scheduled period doesn't fall within any visibility window
            - **Before/after fixed constraints**: Scheduled outside the fixed start/stop times
            """
        )

    st.divider()

    # Report generation
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
            width="stretch",
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
            width="stretch",
        )

    st.caption("Reports contain all key metrics, insights, correlations, and conflict information.")
