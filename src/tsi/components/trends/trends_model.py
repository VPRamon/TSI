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
    model_result: LogisticModelResult,
) -> None:
    """
    Display prediction plot showing probability vs visibility.

    Args:
        model_result: Trained model results with prediction data
    """
    st.markdown("**Model Predictions**")
    st.caption(
        "The logistic model predicts scheduling probability based on "
        "priority, visibility, and requested time with interaction effects."
    )

    # Display coefficients
    if hasattr(model_result, 'coefficients') and model_result.coefficients:
        with st.expander("View model coefficients"):
            import pandas as pd
            coef_df = pd.DataFrame({
                "Feature": model_result.feature_names,
                "Coefficient": model_result.coefficients
            })
            st.dataframe(coef_df, width="stretch")



def render_model_information(
    model_result: LogisticModelResult,
) -> None:
    """
    Display model information in an expander.

    Args:
        model_result: Trained model results
    """
    with st.expander("ℹ️ Model information"):
        st.markdown(
            f"""
            ### Model specifications

            - **Type:** Logistic regression with interaction terms
            - **Variables:** priority, visibility (total_visibility_hours), requested_time (requested_hours)
            - **Interactions:** priority × visibility, visibility × requested_time
            - **Preprocessing:** Standardization (StandardScaler)

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
