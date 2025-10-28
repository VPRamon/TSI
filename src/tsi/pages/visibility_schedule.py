"""Visibility and schedule timeline page."""

import pandas as pd
import streamlit as st

from tsi import state
from tsi.components.toolbar import (
    render_number_input,
    render_priority_filter,
    render_reset_filters_button,
)
from tsi.plots.timeline import build_visibility_histogram
from tsi.services.loaders import get_filtered_dataframe


def render() -> None:
    """Render the Visibility Map & Schedule page."""
    st.title("📅 Visibility Map & Schedule")

    st.markdown(
        """
        Histogram showing the number of targets visible over the observation period.
        This view is optimized for analyzing large datasets with thousands of blocks.
        """
    )

    df = state.get_prepared_data()

    if df is None:
        st.warning("No data loaded. Please return to the landing page.")
        return

    # Calculate priority range from data
    if "priority" in df.columns:
        priority_values = df["priority"].dropna()
        if not priority_values.empty:
            priority_min = float(priority_values.min())
            priority_max = float(priority_values.max())
            if priority_min == priority_max:
                priority_max = priority_min + 1.0
        else:
            priority_min, priority_max = 0.0, 10.0
    else:
        priority_min, priority_max = 0.0, 10.0

    # Sidebar controls
    with st.sidebar:
        st.header("Visibility Histogram Controls")

        # Filters
        stored_range = state.get_priority_range()
        # If stored range is None, outside actual data range, or is the generic default,
        # use the full data range
        if (
            stored_range is None
            or stored_range[0] < priority_min
            or stored_range[1] > priority_max
            or stored_range == (0.0, 10.0)
        ):
            default_range = (priority_min, priority_max)
        else:
            default_range = stored_range

        priority_range = render_priority_filter(
            "timeline_priority_range",
            min_value=priority_min,
            max_value=priority_max,
            default=default_range,
        )
        state.set_priority_range(priority_range)

        st.caption("⚙️ Histogram adjustments are also available below the chart.")

        st.divider()

        if render_reset_filters_button():
            state.reset_filters()
            # Clear histogram generation flag so it doesn't auto-generate on reset
            st.session_state.pop("visibility_histogram_generated", None)
            # Streamlit will auto-rerun on button click

    # Main-panel histogram settings so they remain visible even if the sidebar is collapsed
    settings_container = st.expander("Histogram Settings", expanded=True)
    with settings_container:
        st.markdown("Customize the bin width without opening the sidebar.")

        bin_mode = st.radio(
            "Bin Size Mode",
            options=["Number of bins", "Fixed duration"],
            index=0,
            key="visibility_histogram_bin_mode",
            help="Choose whether to control the histogram by total bin count or by a fixed time width.",
        )

        bin_duration_minutes: float | None = None
        num_bins: int | None

        if bin_mode == "Number of bins":
            num_bins = render_number_input(
                "Number of Time Bins",
                min_value=10,
                max_value=500,
                default=50,  # Reduced from 100 for better performance
                key="visibility_histogram_bins",
            )
            st.caption(
                "Increase the number of bins for finer resolution or decrease it for smoother trends."
            )
            if num_bins and num_bins > 100:
                st.warning("⚠️ High bin counts (>100) may take 10+ seconds to compute. Consider using fewer bins or filtering data first.")
        else:
            num_bins = None
            col1, col2 = st.columns([1, 1])
            with col1:
                bin_width_value = st.number_input(
                    "Bin Width",
                    min_value=0.1,
                    max_value=168.0,
                    value=1.0,
                    step=0.5,
                    key="visibility_histogram_bin_width_value",
                    help="Set how wide each histogram bin should be.",
                )
            with col2:
                bin_width_unit = st.selectbox(
                    "Bin Width Unit",
                    options=["Minutes", "Hours", "Days"],
                    index=1,
                    key="visibility_histogram_bin_width_unit",
                )
            unit_to_minutes = {"Minutes": 1, "Hours": 60, "Days": 1440}
            bin_duration_minutes = bin_width_value * unit_to_minutes[bin_width_unit]
            st.caption(
                "Use a fixed duration when you need the histogram bins to align with specific operational windows."
            )

        st.info("💡 **Tip:** Adjust the mode and bin thickness to focus on specific time scales.")

    # Filter data BEFORE parsing visibility - major performance improvement
    filtered_df = get_filtered_dataframe(
        df,
        priority_range=priority_range,
        scheduled_filter="All",
    )

    # Show statistics FIRST - this gives immediate feedback while histogram loads
    st.subheader("📊 Dataset Statistics")
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Total Blocks", f"{len(df):,}")

    with col2:
        st.metric("Filtered Blocks", f"{len(filtered_df):,}")

    with col3:
        scheduled_count = filtered_df["scheduled_flag"].sum()
        st.metric("Scheduled", f"{int(scheduled_count):,}")

    with col4:
        avg_vis_periods = filtered_df["num_visibility_periods"].mean()
        if pd.notna(avg_vis_periods):
            st.metric("Avg Visibility Periods", f"{avg_vis_periods:.1f}")
        else:
            st.metric("Avg Visibility Periods", "N/A")

    st.divider()

    # Information panel BEFORE the heavy computation
    st.info(
        """
        **How to read this chart:**
        - **X-axis**: Time period of observations (UTC)
        - **Y-axis**: Number of blocks/targets that are visible at that time
        - **Color**: Intensity indicates density of visible targets (darker = more targets)

        This histogram aggregates all visibility windows into time bins, showing when
        the telescope has the most observation opportunities.
        """
    )

    # Create a placeholder for the histogram
    histogram_container = st.container()

    # Add a manual "Generate" button to give users control over when to run expensive computation
    col_btn1, col_btn2 = st.columns([1, 4])
    with col_btn1:
        generate_clicked = st.button("🔄 Generate Histogram", type="primary", use_container_width=True)
    with col_btn2:
        st.caption("Click to build the histogram. Large datasets may take 10-30 seconds.")
    
    # Only build histogram if button was clicked or if we have a cached result
    if generate_clicked or "visibility_histogram_generated" in st.session_state:
        # Mark that we've generated it at least once
        st.session_state["visibility_histogram_generated"] = True
        
        with histogram_container:
            with st.spinner("🔄 Building visibility histogram... This may take 10-30 seconds for large datasets."):
                fig = build_visibility_histogram(
                    df=filtered_df,
                    num_bins=num_bins,
                    bin_duration_minutes=bin_duration_minutes,
                )
            
            st.plotly_chart(fig, use_container_width=True)
    else:
        with histogram_container:
            st.info("👆 Click 'Generate Histogram' above to build the visualization. This prevents automatic computation on page load.")
