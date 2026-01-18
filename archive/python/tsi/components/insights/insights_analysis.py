"""Insights page analysis display components."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd
import streamlit as st

from tsi.plots.distributions import build_correlation_heatmap
from tsi.services.data.analytics import generate_correlation_insights

if TYPE_CHECKING:
    from typing import Any


def render_automated_insights(insights: list[str]) -> None:
    """
    Display automated insights as a bulleted list.

    Args:
        insights: List of insight strings
    """
    st.header("✨ Automated Insights")

    for insight in insights:
        st.markdown(f"- {insight}")


def render_correlation_analysis(correlations: list[Any]) -> None:
    """
    Display correlation analysis with heatmap and interpretation.

    Args:
        correlations: List of CorrelationEntry objects from Rust backend
    """
    st.header("📊 Correlation Analysis")

    if not correlations:
        st.info("Insufficient data for correlation analysis")
        return

    # Convert Rust CorrelationEntry objects to DataFrame in matrix format
    # First, collect all unique variables
    variables_set: set[Any] = set()
    for corr in correlations:
        variables_set.add(corr.variable1)
        variables_set.add(corr.variable2)

    variables = sorted(variables_set)

    # Create correlation matrix
    corr_matrix = pd.DataFrame(
        1.0,  # diagonal is always 1
        index=variables,
        columns=variables,
    )

    # Fill in the correlation values
    for corr in correlations:
        corr_matrix.loc[corr.variable1, corr.variable2] = corr.correlation
        corr_matrix.loc[corr.variable2, corr.variable1] = corr.correlation

    # Generate correlation insights
    correlation_insights = generate_correlation_insights(corr_matrix)

    col1, col2 = st.columns([1, 2])

    with col1:
        st.markdown("**Correlation Interpretation**")
        st.markdown("---")
        for insight in correlation_insights:
            st.markdown(insight)
            st.markdown("")  # Add spacing

        st.caption(
            """
            **Technical note:** Spearman correlation values (ρ) range from -1 to +1:
            - **ρ > 0**: Positive correlation (both variables increase together)
            - **ρ < 0**: Negative correlation (inverse relationship)
            - **|ρ| ≥ 0.7**: Strong correlation
            - **0.4 ≤ |ρ| < 0.7**: Moderate correlation
            - **|ρ| < 0.4**: Weak correlation

            Spearman correlation is robust to outliers and does not assume linearity.
            """
        )

    with col2:
        fig = build_correlation_heatmap(corr_matrix)
        st.plotly_chart(fig, width="stretch")
