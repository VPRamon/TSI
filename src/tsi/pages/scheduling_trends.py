"""Scheduling Trends Page.

Scheduling probability analysis with logistic model
and interactive visualizations.
"""

from __future__ import annotations

from typing import Any

import streamlit as st

from tsi import state
from tsi.components.trends_controls import render_sidebar_controls
from tsi.components.trends_empirical import render_empirical_proportions
from tsi.components.trends_heatmap import render_heatmap_section
from tsi.components.trends_metrics import render_overview_metrics
from tsi.components.trends_model import (
    render_model_information,
    render_model_metrics,
    render_prediction_plot,
)
from tsi.components.trends_smoothed import render_smoothed_trends
from tsi.modeling.trends import (
    compute_empirical_rates,
    fit_logistic_with_interactions,
    smooth_trend,
)
from tsi.services.trends_processing import apply_trends_filters, validate_required_columns


@st.cache_resource(show_spinner="Computing empirical rates...")
def _compute_empirical_cached(df_hash: int, n_bins: int) -> Any:
    """Compute empirical rates with cache."""
    df = state.get_prepared_data()
    return compute_empirical_rates(df, n_bins=n_bins)


@st.cache_data(show_spinner="Computing smoothed trend...")
def _smooth_trend_cached(df_hash: int, x_col: str, bandwidth: float) -> tuple[Any, str | None]:
    """Compute smoothed trend with cache."""
    df = state.get_prepared_data()
    try:
        result = smooth_trend(df, x_col=x_col, bandwidth=bandwidth)
        return result, None
    except Exception as e:
        return None, str(e)


@st.cache_resource(show_spinner="Training logistic model...")
def _fit_model_cached(
    df_hash: int, exclude_zero_visibility: bool, class_weight: str
) -> tuple[Any, str | None]:
    """Train logistic model with cache."""
    df = state.get_prepared_data()
    try:
        model_result = fit_logistic_with_interactions(
            df,
            exclude_zero_visibility=exclude_zero_visibility,
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

    df = state.get_prepared_data()

    if df is None:
        st.warning("⚠️ No data loaded. Please return to the landing page.")
        return

    # Validate required columns
    is_valid, missing_cols = validate_required_columns(df)

    if not is_valid:
        st.error(
            f"""
            ❌ **Missing required columns:** {', '.join(missing_cols)}

            This analysis requires the columns: priority, total_visibility_hours,
            requested_hours, scheduled_flag.
            """
        )
        return

    # Render sidebar controls
    controls = render_sidebar_controls(df)

    # Apply filters
    df_filtered = apply_trends_filters(
        df,
        controls["vis_range"],
        controls["time_range"],
        controls["selected_priorities"],
    )

    if len(df_filtered) < 10:
        st.warning(
            f"""
            ⚠️ **Insufficient data after filtering:** {len(df_filtered)} rows.

            Adjust the filters in the sidebar to include more data.
            """
        )
        return

    # Dataframe hash for caching
    df_hash = hash(tuple(df_filtered.index))

    # Overview metrics
    st.divider()
    render_overview_metrics(df_filtered)
    st.divider()

    # Section 1: Empirical proportions
    empirical = _compute_empirical_cached(df_hash, controls["n_bins"])
    render_empirical_proportions(empirical, controls["plot_library"])
    st.divider()

    # Section 2: Smoothed curves
    smooth_vis, error_vis = _smooth_trend_cached(
        df_hash,
        x_col="total_visibility_hours",
        bandwidth=controls["bandwidth"],
    )
    smooth_time, error_time = _smooth_trend_cached(
        df_hash,
        x_col="requested_hours",
        bandwidth=controls["bandwidth"],
    )
    render_smoothed_trends(
        smooth_vis,
        error_vis,
        smooth_time,
        error_time,
        controls["plot_library"],
    )
    st.divider()

    # Section 3: 2D Heatmap
    render_heatmap_section(df_filtered, controls["plot_library"], controls["n_bins"])
    st.divider()

    # Section 4: Logistic model
    st.subheader("4️⃣ Logistic model with interactions")
    st.caption(
        "🤖 Multivariable logistic model with 3 predictor variables: "
        "**priority**, **visibility**, and **requested time**. "
        "Includes interaction terms (priority × visibility, visibility × time) "
        "to capture non-linear effects. "
        "Predicts the **estimated probability** of scheduling."
    )

    # Train model
    model_result, model_error = _fit_model_cached(
        df_hash,
        controls["exclude_zero_vis"],
        controls["class_weight"],
    )

    if model_error:
        st.error(f"❌ Error training model: {model_error}")
        return

    if model_result is None:
        st.warning("⚠️ Could not train the model.")
        return

    # Display model metrics
    render_model_metrics(model_result)

    # Display prediction plot
    render_prediction_plot(
        df_filtered,
        model_result,
        controls["vis_range"],
        controls["fixed_time"],
        controls["plot_library"],
    )

    st.divider()

    # Model information
    render_model_information(
        model_result,
        controls["class_weight"],
        controls["exclude_zero_vis"],
    )
