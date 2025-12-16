"""Scheduling Trends Page.

Scheduling probability analysis with logistic model
and interactive visualizations.
"""

from __future__ import annotations

from typing import Any

import streamlit as st

from tsi import state
from tsi.components.trends.trends_controls import render_sidebar_controls
from tsi.components.trends.trends_empirical import render_empirical_proportions
from tsi.components.trends.trends_heatmap import render_heatmap_section
from tsi.components.trends.trends_metrics import render_overview_metrics
from tsi.components.trends.trends_model import (
    render_model_information,
    render_model_metrics,
    render_prediction_plot,
)
from tsi.components.trends.trends_smoothed import render_smoothed_trends
from tsi.modeling.trends import fit_logistic_with_interactions
from tsi.services import database as db
from tsi.utils.error_display import display_backend_error

@st.cache_data(show_spinner="Training logistic model...")
def _fit_model_cached(
    schedule_id: int,
    class_weight: str,
    vis_range: tuple[float, float],
    time_range: tuple[float, float],
    selected_priorities: list[float],
) -> tuple[Any, str | None]:
    """Train logistic model with cache using Rust-loaded data."""
    try:
        # Load data from Rust backend (impossible blocks already filtered during ETL)
        trends_data = db.get_trends_data(schedule_id=schedule_id)
        
        # Filter blocks based on controls
        blocks = trends_data.blocks
        filtered_blocks = [
            b for b in blocks
            if (b.total_visibility_hours >= vis_range[0] and
                b.total_visibility_hours <= vis_range[1] and
                b.requested_hours >= time_range[0] and
                b.requested_hours <= time_range[1] and
                b.priority in selected_priorities)
        ]
        
        if len(filtered_blocks) < 20:
            return None, f"Insufficient data for model training: {len(filtered_blocks)} blocks (minimum 20 required)"
        
        # Convert to DataFrame for model training
        import pandas as pd
        df_model = pd.DataFrame([
            {
                "priority": b.priority,
                "total_visibility_hours": b.total_visibility_hours,
                "requested_hours": b.requested_hours,
                "scheduled_flag": 1 if b.scheduled else 0,
            }
            for b in filtered_blocks
        ])
        
        model_result = fit_logistic_with_interactions(
            df_model,
            exclude_zero_visibility=False,  # Already filtered if requested
            class_weight=class_weight if class_weight != "None" else None,
        )
        return model_result, None
    except Exception as e:
        return None, str(e)


def render() -> None:
    """Render the Scheduling Trends page."""
    st.title("📈 Scheduling Trends")

    st.markdown(
        """
        Analysis of **scheduling probability** based on priority,
        visibility and requested time, including interactions between variables.
        """
    )

    schedule_id = state.get_schedule_id()

    if schedule_id is None:
        st.info("Load a schedule from the database to view trends.")
        return

    schedule_id = int(schedule_id)

    # Load trends data from Rust backend
    try:
        with st.spinner("Loading trends data..."):
            trends_data = db.get_trends_data(
                schedule_id=schedule_id,
                n_bins=10,  # Will be updated based on controls
                bandwidth=0.3,  # Will be updated based on controls
                n_smooth_points=100,
            )
    except Exception as exc:
        display_backend_error(exc)
        return

    if trends_data.metrics.total_count == 0:
        st.warning("⚠️ No observations available.")
        return

    # Render sidebar controls
    controls = render_sidebar_controls(trends_data)
    
    # Reload with updated parameters if controls changed
    if (controls["n_bins"] != 10 or 
        controls["bandwidth"] != 0.3):
        with st.spinner("Recomputing with updated parameters..."):
            trends_data = db.get_trends_data(
                schedule_id=schedule_id,
                n_bins=controls["n_bins"],
                bandwidth=controls["bandwidth"],
                n_smooth_points=100,
            )
    
    # Filter blocks based on controls
    filtered_blocks = [
        b for b in trends_data.blocks
        if (b.total_visibility_hours >= controls["vis_range"][0] and
            b.total_visibility_hours <= controls["vis_range"][1] and
            b.requested_hours >= controls["time_range"][0] and
            b.requested_hours <= controls["time_range"][1] and
            b.priority in controls["selected_priorities"])
    ]
    
    if len(filtered_blocks) == 0:
        st.warning("⚠️ No observations match the selected filters.")
        return

    # Display overview metrics
    st.header("📊 Overview Metrics")
    render_overview_metrics(trends_data.metrics)

    # Empirical trends by priority
    st.header("Priority-Based Scheduling")
    render_empirical_proportions(trends_data.by_priority, "priority")

    # Display empirical trends by bins
    st.header("📐 Empirical Proportions (Binned)")
    render_empirical_proportions(trends_data.by_visibility, "total_visibility_hours")
    render_empirical_proportions(trends_data.by_time, "requested_hours")

    # Smoothed trends
    st.header("🌊 Smoothed Trends")
    render_smoothed_trends(trends_data.smoothed_visibility, "total_visibility_hours")
    render_smoothed_trends(trends_data.smoothed_time, "requested_hours")

    # Heatmap section
    st.header("🔥 Heatmap: Visibility × Requested Time")
    render_heatmap_section(trends_data.heatmap_bins, n_bins=controls["n_bins"])

    # Logistic model section
    st.header("🎯 Logistic Regression Model")

    with st.spinner("Training logistic model..."):
        model_result, error = _fit_model_cached(
            schedule_id,
            class_weight=controls["class_weight"],
            vis_range=controls["vis_range"],
            time_range=controls["time_range"],
            selected_priorities=controls["selected_priorities"],
        )

    if error:
        st.error(f"❌ Model training failed: {error}")
        return

    if model_result is None:
        st.warning("⚠️ Insufficient data to fit the model.")
        return

    # Render model metrics
    render_model_metrics(model_result)

    # Render model information in expander
    render_model_information(model_result)

    # Render prediction plot
    render_prediction_plot(model_result)
