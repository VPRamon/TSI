"""Distributions page - statistical visualizations."""

import streamlit as st

from tsi import state
from tsi.plots.distributions import build_figures

FILTER_OPTIONS = ("all", "exclude_impossible")
FILTER_LABELS = {
    "all": "📋 Todas las observaciones",
    "exclude_impossible": "✅ Solo observaciones posibles",
}


def render() -> None:
    """Render the Distributions page."""
    # Title and filter controls in same row
    col1, col2 = st.columns([3, 1])

    with col1:
        st.title("📊 Distributions")

    df = state.get_prepared_data()

    if df is None:
        st.warning("No data loaded. Please return to the landing page.")
        return

    filtered_df = df

    # Initialize session state only if not present
    if state.KEY_DIST_FILTER_MODE not in st.session_state:
        st.session_state[state.KEY_DIST_FILTER_MODE] = "all"

    filter_supported = (
        "minObservationTimeInSec" in df.columns and "total_visibility_hours" in df.columns
    )
    impossible_mask = None

    if filter_supported:
        TOLERANCE_SEC = 1
        min_duration_secs = df["minObservationTimeInSec"].fillna(0)
        visibility_secs = df["total_visibility_hours"] * 3600.0
        impossible_mask = (min_duration_secs - TOLERANCE_SEC > visibility_secs).fillna(False)

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
                key=state.KEY_DIST_FILTER_MODE,
                horizontal=False,
                label_visibility="collapsed",
            )
    else:
        st.session_state[state.KEY_DIST_FILTER_MODE] = "all"

    # Use default bins value
    priority_bins = 20

    st.markdown(
        """
        Statistical analysis and distribution visualizations of key scheduling parameters.
        """
    )

    # Apply filter based on user selection
    if filter_supported and impossible_mask is not None:
        if filter_mode == "exclude_impossible":
            filtered_df = df[~impossible_mask].copy()

    if filtered_df.empty:
        st.warning("⚠️ No observations available with the selected filter.")
        return

    # Build all distribution figures
    with st.spinner("Generating plots..."):
        figures = build_figures(filtered_df, priority_bins=priority_bins)

    # Display figures in organized layout

    # Priority distribution
    st.subheader("Priority Distribution")
    st.plotly_chart(figures["priority_hist"], width='stretch')

    # Two-column layout for other distributions
    col1, col2 = st.columns(2)

    with col1:
        st.subheader("Visibility Hours")
        st.plotly_chart(figures["visibility_hist"], width='stretch')

        st.subheader("Elevation Constraint Range")
        st.plotly_chart(figures["elevation_hist"], width='stretch')

    with col2:
        st.subheader("Requested Duration")
        st.plotly_chart(figures["duration_hist"], width='stretch')

        st.subheader("Scheduling Status")
        st.plotly_chart(figures["scheduled_bar"], width='stretch')

    # Comparison plot (full width)
    st.subheader("Priority Comparison by Scheduling Status")
    st.plotly_chart(figures["priority_violin"], width='stretch')

    # Statistical summary
    st.divider()
    st.subheader("Statistical Summary")

    col1, col2, col3 = st.columns(3)

    with col1:
        st.markdown("**Priority**")
        st.write(f"Mean: {filtered_df['priority'].mean():.2f}")
        st.write(f"Median: {filtered_df['priority'].median():.2f}")
        st.write(f"Std Dev: {filtered_df['priority'].std():.2f}")

    with col2:
        st.markdown("**Visibility Hours**")
        st.write(f"Mean: {filtered_df['total_visibility_hours'].mean():.1f}")
        st.write(f"Median: {filtered_df['total_visibility_hours'].median():.1f}")
        st.write(f"Total: {filtered_df['total_visibility_hours'].sum():,.0f}")

    with col3:
        st.markdown("**Requested Hours**")
        st.write(f"Mean: {filtered_df['requested_hours'].mean():.2f}")
        st.write(f"Median: {filtered_df['requested_hours'].median():.2f}")
        st.write(f"Total: {filtered_df['requested_hours'].sum():,.1f}")
