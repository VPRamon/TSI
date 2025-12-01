"""Scheduling trends page control components."""

from __future__ import annotations

from typing import TYPE_CHECKING

import streamlit as st

if TYPE_CHECKING:
    from tsi_rust import TrendsData


def render_sidebar_controls(trends_data: TrendsData) -> dict:
    """
    Render sidebar configuration controls for scheduling trends analysis.

    Args:
        trends_data: TrendsData from Rust backend

    Returns:
        Dictionary with all control values
    """
    with st.sidebar:
        st.header("⚙️ Configuration")

        st.subheader("Data Filters")

        # Extract ranges from blocks
        blocks = trends_data.blocks
        if not blocks:
            st.warning("No data available")
            return {}
        
        vis_values = [b.total_visibility_hours for b in blocks]
        time_values = [b.requested_hours for b in blocks]
        priority_values = sorted(set(b.priority for b in blocks))

        # Visibility range
        vis_min = min(vis_values)
        vis_max = max(vis_values)

        # Handle edge case where min == max
        if vis_min == vis_max:
            st.info(f"All observations have visibility = {vis_min:.1f} hours")
            vis_range = (vis_min, vis_max)
        else:
            vis_range = st.slider(
                "Visibility range (hours)",
                min_value=vis_min,
                max_value=vis_max,
                value=(vis_min, vis_max),
                key="vis_range",
            )

        # Requested time range
        time_min = min(time_values)
        time_max = max(time_values)

        # Handle edge case where min == max
        if time_min == time_max:
            st.info(f"All observations have requested time = {time_min:.1f} hours")
            time_range = (time_min, time_max)
        else:
            time_range = st.slider(
                "Requested time range (hours)",
                min_value=time_min,
                max_value=time_max,
                value=(time_min, time_max),
                key="time_range",
            )

        # Priority level selector
        selected_priorities = st.multiselect(
            "Priority levels",
            options=priority_values,
            default=priority_values,
            key="selected_priorities",
        )

        st.divider()

        st.subheader("Plot Configuration")

        # Number of bins
        n_bins = st.slider(
            "Number of bins",
            min_value=5,
            max_value=20,
            value=10,
            key="n_bins",
        )

        # Bandwidth for smoothing
        bandwidth = st.slider(
            "Bandwidth (smoothing)",
            min_value=0.1,
            max_value=0.6,
            value=0.3,
            step=0.05,
            key="bandwidth",
        )

        st.divider()

        st.subheader("Logistic Model")

        # Info about automatic filtering
        st.info(
            "ℹ️ Impossible blocks (visibility = 0) are automatically excluded during ETL. "
            "All analyses use clean data."
        )

        # Class weight
        class_weight_option = st.selectbox(
            "Class weighting",
            options=["balanced", "None"],
            index=0,
            help="'balanced' adjusts weights inversely proportional to class frequencies",
            key="class_weight",
        )

    return {
        "vis_range": vis_range,
        "time_range": time_range,
        "selected_priorities": selected_priorities,
        "n_bins": n_bins,
        "bandwidth": bandwidth,
        "class_weight": class_weight_option,
    }
