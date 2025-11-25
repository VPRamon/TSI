"""Scheduling trends page control components."""

from __future__ import annotations

import streamlit as st
import pandas as pd


def render_sidebar_controls(df: pd.DataFrame) -> dict:
    """
    Render sidebar configuration controls for scheduling trends analysis.
    
    Args:
        df: Source DataFrame
        
    Returns:
        Dictionary with all control values
    """
    with st.sidebar:
        st.header("⚙️ Configuration")
        
        st.subheader("Data Filters")
        
        # Visibility range
        vis_min = float(df["total_visibility_hours"].min())
        vis_max = float(df["total_visibility_hours"].max())
        vis_range = st.slider(
            "Visibility range (hours)",
            min_value=vis_min,
            max_value=vis_max,
            value=(vis_min, vis_max),
            key="vis_range",
        )
        
        # Requested time range
        time_min = float(df["requested_hours"].min())
        time_max = float(df["requested_hours"].max())
        time_range = st.slider(
            "Requested time range (hours)",
            min_value=time_min,
            max_value=time_max,
            value=(time_min, time_max),
            key="time_range",
        )
        
        # Priority level selector
        priority_levels = sorted(df["priority"].dropna().unique())
        selected_priorities = st.multiselect(
            "Priority levels",
            options=priority_levels,
            default=priority_levels,
            key="selected_priorities",
        )
        
        st.divider()
        
        st.subheader("Plot Configuration")
        
        # Library selector
        plot_library = st.selectbox(
            "Plot library",
            options=["altair", "plotly"],
            index=0,
            key="plot_library",
        )
        
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
        
        # Exclude visibility = 0
        exclude_zero_vis = st.checkbox(
            "Exclude visibility = 0 for model",
            value=True,
            help="If enabled, the model is trained only with observations that have visibility > 0",
            key="exclude_zero_vis",
        )
        
        # Class weight
        class_weight_option = st.selectbox(
            "Class weighting",
            options=["balanced", "None"],
            index=0,
            help="'balanced' adjusts weights inversely proportional to class frequencies",
            key="class_weight",
        )
        
        # Fixed time for prediction
        fixed_time = st.slider(
            "Fixed requested time (for prediction)",
            min_value=time_min,
            max_value=time_max,
            value=(time_min + time_max) / 2,
            key="fixed_time",
        )
    
    return {
        "vis_range": vis_range,
        "time_range": time_range,
        "selected_priorities": selected_priorities,
        "plot_library": plot_library,
        "n_bins": n_bins,
        "bandwidth": bandwidth,
        "exclude_zero_vis": exclude_zero_vis,
        "class_weight": class_weight_option,
        "fixed_time": fixed_time,
    }
