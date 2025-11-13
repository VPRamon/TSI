"""Scheduling Trends Page.

Scheduling probability analysis with logistic model
and interactive visualizations.
"""

from __future__ import annotations

from typing import TYPE_CHECKING, cast

import streamlit as st

if TYPE_CHECKING:
    from altair import Chart

from tsi import state
from tsi.modeling.trends import (
    EmpiricalRates,
    LogisticModelResult,
    compute_empirical_rates,
    create_prediction_grid,
    fit_logistic_with_interactions,
    predict_probs,
    smooth_trend,
)
from tsi.plots.trends import (
    bar_rate_by_priority,
    heatmap_visibility_priority,
    loess_trend,
    pred_curve_vs_visibility,
)


@st.cache_resource(show_spinner="Computing empirical rates...")
def _compute_empirical_cached(
    df_hash: int,
    n_bins: int,
) -> EmpiricalRates:
    """Compute empirical rates with cache."""
    df = state.get_prepared_data()
    return compute_empirical_rates(df, n_bins=n_bins)


@st.cache_data(show_spinner="Computing smoothed trend...")
def _smooth_trend_cached(
    df_hash: int,
    x_col: str,
    bandwidth: float,
) -> tuple:
    """Compute smoothed trend with cache."""
    df = state.get_prepared_data()
    try:
        result = smooth_trend(df, x_col=x_col, bandwidth=bandwidth)
        return result, None
    except Exception as e:
        return None, str(e)


@st.cache_resource(show_spinner="Training logistic model...")
def _fit_model_cached(
    df_hash: int,
    exclude_zero_visibility: bool,
    class_weight: str,
) -> tuple[LogisticModelResult | None, str | None]:
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
    required_cols = ["priority", "total_visibility_hours", "requested_hours", "scheduled_flag"]
    missing_cols = [col for col in required_cols if col not in df.columns]

    if missing_cols:
        st.error(
            f"""
            ❌ **Missing required columns:** {', '.join(missing_cols)}

            This analysis requires the columns: priority, total_visibility_hours,
            requested_hours, scheduled_flag.
            """
        )
        return

    # Sidebar: Controls
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

    # Filter data
    df_filtered = df[
        (df["total_visibility_hours"] >= vis_range[0])
        & (df["total_visibility_hours"] <= vis_range[1])
        & (df["requested_hours"] >= time_range[0])
        & (df["requested_hours"] <= time_range[1])
        & (df["priority"].isin(selected_priorities))
    ].copy()

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

    # General metrics
    st.divider()
    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Total observations", f"{len(df_filtered):,}")

    with col2:
        n_scheduled = int(df_filtered["scheduled_flag"].sum())
        st.metric("Scheduled", f"{n_scheduled:,}")

    with col3:
        rate_scheduled = df_filtered["scheduled_flag"].mean() * 100
        st.metric("% Scheduled", f"{rate_scheduled:.1f}%")

    with col4:
        zero_vis = (df_filtered["total_visibility_hours"] == 0).sum()
        st.metric("Visibility = 0", f"{zero_vis:,}")

    st.divider()

    # ===== SECTION 1: EMPIRICAL PROPORTIONS BY PRIORITY =====
    st.subheader("1️⃣ Empirical proportions by priority")
    st.caption(
        "📊 Shows the **empirical rate** of scheduling for each priority level. "
        "The tooltip includes the number of observations (n)."
    )

    try:
        empirical = _compute_empirical_cached(df_hash, n_bins)

        fig_priority = bar_rate_by_priority(
            empirical.by_priority,
            library=plot_library,
            title="Scheduling rate by priority",
        )

        if plot_library == "altair":
            st.altair_chart(cast("Chart", fig_priority), use_container_width=True)
        else:
            st.plotly_chart(fig_priority, use_container_width=True)

    except Exception as e:
        st.error(f"❌ Error computing empirical rates: {e}")

    st.divider()

    # ===== SECTION 2: SMOOTHED CURVES =====
    st.subheader("2️⃣ Smoothed curves (trends)")
    st.caption(
        "📈 Smoothed trends using weighted moving average (similar to LOESS). "
        "Shows how the scheduling rate varies with visibility and requested time."
    )

    col1, col2 = st.columns(2)

    with col1:
        st.markdown("**Visibility → Scheduling rate**")

        smooth_vis, error_vis = _smooth_trend_cached(
            df_hash,
            x_col="total_visibility_hours",
            bandwidth=bandwidth,
        )

        if error_vis:
            st.warning(f"⚠️ {error_vis}")
        elif smooth_vis is not None:
            fig_vis = loess_trend(
                smooth_vis,
                library=plot_library,
                title="Trend: Visibility",
                x_label="Visibility (hours)",
                y_label="Scheduling rate",
            )

            if plot_library == "altair":
                st.altair_chart(cast("Chart", fig_vis), use_container_width=True)
            else:
                st.plotly_chart(fig_vis, use_container_width=True)

    with col2:
        st.markdown("**Requested time → Scheduling rate**")

        smooth_time, error_time = _smooth_trend_cached(
            df_hash,
            x_col="requested_hours",
            bandwidth=bandwidth,
        )

        if error_time:
            st.warning(f"⚠️ {error_time}")
        elif smooth_time is not None:
            fig_time = loess_trend(
                smooth_time,
                library=plot_library,
                title="Trend: Requested time",
                x_label="Requested time (hours)",
                y_label="Scheduling rate",
            )

            if plot_library == "altair":
                st.altair_chart(cast("Chart", fig_time), use_container_width=True)
            else:
                st.plotly_chart(fig_time, use_container_width=True)

    st.divider()

    # ===== SECTION 3: 2D HEATMAP =====
    st.subheader("3️⃣ Heatmap: Visibility × Priority")
    st.caption(
        "🔥 2D heatmap showing the **mean empirical rate** of scheduling "
        "as a function of visibility (X) and priority (Y)."
    )

    try:
        fig_heatmap = heatmap_visibility_priority(
            df_filtered,
            library=plot_library,
            n_bins_vis=n_bins,
            n_bins_priority=n_bins,
        )

        if plot_library == "altair":
            st.altair_chart(cast("Chart", fig_heatmap), use_container_width=True)
        else:
            st.plotly_chart(fig_heatmap, use_container_width=True)

    except Exception as e:
        st.error(f"❌ Error generating heatmap: {e}")

    st.divider()

    # ===== SECTION 4: LOGISTIC MODEL WITH INTERACTIONS =====
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
        exclude_zero_vis,
        class_weight_option,
    )

    if model_error:
        st.error(f"❌ Error training model: {model_error}")
        return

    if model_result is None:
        st.warning("⚠️ Could not train the model.")
        return

    # Show model metrics
    st.markdown("**Model metrics:**")

    col1, col2, col3, col4 = st.columns(4)

    with col1:
        st.metric("Samples (training)", f"{model_result.n_samples:,}")

    with col2:
        st.metric("Scheduled (training)", f"{model_result.n_scheduled:,}")

    with col3:
        st.metric("Accuracy", f"{model_result.accuracy:.3f}")

    with col4:
        if model_result.auc_score is not None:
            st.metric("AUC", f"{model_result.auc_score:.3f}")
        else:
            st.metric("AUC", "N/A")

    st.caption(
        f"✅ Model trained successfully. "
        f"Features with interactions: {len(model_result.feature_names)}"
    )

    # Plot of estimated probability vs visibility
    st.markdown("**Estimated probability vs Visibility**")
    st.caption(
        "This plot shows how scheduling probability changes with visibility "
        "for different priority levels (interaction: priority × visibility). "
        f"Requested time is held constant at **{fixed_time:.1f} hours**."
    )

    try:
        # Create prediction grid
        priority_levels_for_pred = sorted(df_filtered["priority"].dropna().unique())[
            :5
        ]  # Max 5 levels
        if len(priority_levels_for_pred) > 5:
            st.info(
                f"ℹ️ Showing only 5 priority levels for clarity. "
                f"Total available: {len(priority_levels_for_pred)}"
            )

        grid = create_prediction_grid(
            visibility_range=(vis_range[0], vis_range[1]),
            priority_levels=priority_levels_for_pred,
            requested_time=fixed_time,
            n_points=100,
        )

        # Predict
        grid_with_probs = predict_probs(
            grid,
            model_result,
            fixed_params=None,  # Already fixed in grid
        )

        # Plot
        fig_pred = pred_curve_vs_visibility(
            grid_with_probs,
            library=plot_library,
            fixed_time=fixed_time,
        )

        if plot_library == "altair":
            st.altair_chart(cast("Chart", fig_pred), use_container_width=True)
        else:
            st.plotly_chart(fig_pred, use_container_width=True)

        st.caption(
            "📌 Each curve represents a different **priority level**. "
            "Higher priority observations have higher predicted probability of being scheduled. "
            "The model captures the interaction between priority and visibility."
        )

    except Exception as e:
        st.error(f"❌ Error generating predictions: {e}")

    st.divider()

    # Additional information
    with st.expander("ℹ️ Model information"):
        st.markdown(
            f"""
            ### Model specifications

            - **Type:** Logistic regression with interaction terms
            - **Variables:** priority, visibility (total_visibility_hours), requested_time (requested_hours)
            - **Interactions:** priority × visibility, visibility × requested_time
            - **Preprocessing:** Standardization (StandardScaler)
            - **Class weighting:** {class_weight_option}
            - **Exclude visibility = 0:** {'Yes' if exclude_zero_vis else 'No'}

            ### Generated features

            The model includes {len(model_result.feature_names)} features:

            ```
            {', '.join(model_result.feature_names)}
            ```

            ### Interpretation

            - **Captured interactions:** If visibility = 0, the probability will be ~0 regardless of priority.
            - **High priority + high visibility:** Higher scheduling probability.
            - **Requested time:** Can reduce probability if too high (limited resources).
            """
        )
