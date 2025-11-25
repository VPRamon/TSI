"""Visibility schedule control components."""

from __future__ import annotations

import streamlit as st

from tsi import state
from tsi.components.toolbar import (
    render_number_input,
    render_priority_range_control,
    render_reset_filters_button,
)


def render_sidebar_controls(
    priority_min: float,
    priority_max: float,
) -> tuple[float, float]:
    """
    Render sidebar controls for visibility schedule.

    Args:
        priority_min: Minimum priority value
        priority_max: Maximum priority value

    Returns:
        Priority range tuple (min, max)
    """
    st.header("Visibility Histogram Controls")

    stored_range = state.get_priority_range()
    
    priority_range = render_priority_range_control(
        priority_min,
        priority_max,
        stored_range,
        key="timeline_priority_range",
    )
    state.set_priority_range(priority_range)

    st.caption("⚙️ Histogram adjustments are also available below the chart.")

    st.divider()

    if render_reset_filters_button():
        state.reset_filters()
        st.session_state.pop("visibility_histogram_generated", None)

    return priority_range


def render_histogram_settings(
    priority_min: float,
    priority_max: float,
    all_block_ids: list,
) -> dict:
    """
    Render histogram settings panel.

    Args:
        priority_min: Minimum priority value
        priority_max: Maximum priority value
        all_block_ids: List of all block IDs

    Returns:
        Dictionary containing all settings
    """
    settings_container = st.expander("Histogram Settings", expanded=True)
    
    with settings_container:
        st.markdown(
            "Customize the bin width and apply additional filters without opening the sidebar."
        )

        # Priority filter
        priority_filter_range = _render_priority_filter_section(priority_min, priority_max)

        # Block ID filter
        selected_block_ids = _render_block_id_filter_section(all_block_ids)

        st.divider()

        # Bin configuration
        num_bins, bin_duration_minutes = _render_bin_configuration_section()

        st.info("💡 **Tip:** Adjust the mode and bin thickness to focus on specific time scales.")

    return {
        "priority_filter_range": priority_filter_range,
        "selected_block_ids": selected_block_ids,
        "num_bins": num_bins,
        "bin_duration_minutes": bin_duration_minutes,
    }


def _render_priority_filter_section(
    priority_min: float,
    priority_max: float,
) -> tuple[float, float]:
    """Render priority filter section."""
    st.subheader("🎯 Priority Filter")
    priority_filter_range = st.slider(
        "Filter by Priority Range",
        min_value=priority_min,
        max_value=priority_max,
        value=(priority_min, priority_max),
        step=0.1,
        key="visibility_histogram_priority_filter",
        help="Filter blocks by priority range for the histogram",
    )
    return priority_filter_range


def _render_block_id_filter_section(all_block_ids: list) -> list | None:
    """Render block ID filter section."""
    st.subheader("🔢 Block ID Filter")
    
    enable_block_filter = st.checkbox(
        "Filter by specific Block IDs",
        value=False,
        key="visibility_histogram_enable_block_filter",
        help="Enable to select specific scheduling blocks to display in the histogram",
    )

    selected_block_ids = None
    if enable_block_filter:
        selected_block_ids = st.multiselect(
            "Select Block IDs",
            options=all_block_ids,
            default=None,
            key="visibility_histogram_block_ids",
            help="Choose specific blocks to include in the histogram. Leave empty to include all blocks.",
        )
        if selected_block_ids:
            st.caption(f"✓ {len(selected_block_ids)} block(s) selected")
        else:
            st.info("💡 Select at least one block ID to apply filtering")

    return selected_block_ids


def _render_bin_configuration_section() -> tuple[int | None, float | None]:
    """Render bin configuration section."""
    st.subheader("📊 Bin Configuration")
    
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
            default=50,
            key="visibility_histogram_bins",
        )
        st.caption(
            "Increase the number of bins for finer resolution or decrease it for smoother trends."
        )
        if num_bins and num_bins > 100:
            st.warning(
                "⚠️ High bin counts (>100) may take 10+ seconds to compute. Consider using fewer bins or filtering data first."
            )
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

    return num_bins, bin_duration_minutes


def render_generate_button() -> bool:
    """
    Render the generate histogram button.

    Returns:
        True if button was clicked or histogram was previously generated
    """
    col_btn1, col_btn2 = st.columns([1, 4])
    with col_btn1:
        generate_clicked = st.button(
            "🔄 Generate Histogram", type="primary", use_container_width=True
        )
    with col_btn2:
        st.caption("")

    return generate_clicked or "visibility_histogram_generated" in st.session_state
