"""Scheduling trends page logistic model display components."""

from __future__ import annotations

from typing import TYPE_CHECKING, cast

import pandas as pd
import streamlit as st

if TYPE_CHECKING:
    from altair import Chart

from tsi.modeling.trends import LogisticModelResult, create_prediction_grid, predict_probs
from tsi.plots.trends import pred_curve_vs_visibility


def render_model_metrics(model_result: LogisticModelResult) -> None:
    """
    Display logistic model training metrics.
    
    Args:
        model_result: Trained model results
    """
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


def render_prediction_plot(
    df: pd.DataFrame,
    model_result: LogisticModelResult,
    vis_range: tuple[float, float],
    fixed_time: float,
    plot_library: str,
) -> None:
    """
    Display prediction plot showing probability vs visibility.
    
    Args:
        df: Filtered DataFrame
        model_result: Trained model results
        vis_range: Visibility range for predictions
        fixed_time: Fixed requested time value
        plot_library: Plotting library to use ('altair' or 'plotly')
    """
    st.markdown("**Estimated probability vs Visibility**")
    st.caption(
        "This plot shows how scheduling probability changes with visibility "
        "for different priority levels (interaction: priority × visibility). "
        f"Requested time is held constant at **{fixed_time:.1f} hours**."
    )

    try:
        # Create prediction grid
        priority_levels_for_pred = sorted(df["priority"].dropna().unique())[:5]  # Max 5 levels

        if len(sorted(df["priority"].dropna().unique())) > 5:
            st.info(
                f"ℹ️ Showing only 5 priority levels for clarity. "
                f"Total available: {len(sorted(df['priority'].dropna().unique()))}"
            )

        grid = create_prediction_grid(
            visibility_range=vis_range,
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


def render_model_information(
    model_result: LogisticModelResult,
    class_weight: str,
    exclude_zero_vis: bool,
) -> None:
    """
    Display model information in an expander.
    
    Args:
        model_result: Trained model results
        class_weight: Class weighting option used
        exclude_zero_vis: Whether visibility = 0 was excluded
    """
    with st.expander("ℹ️ Model information"):
        st.markdown(
            f"""
            ### Model specifications

            - **Type:** Logistic regression with interaction terms
            - **Variables:** priority, visibility (total_visibility_hours), requested_time (requested_hours)
            - **Interactions:** priority × visibility, visibility × requested_time
            - **Preprocessing:** Standardization (StandardScaler)
            - **Class weighting:** {class_weight}
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
