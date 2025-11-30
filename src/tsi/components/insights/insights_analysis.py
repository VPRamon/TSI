"""Insights page analysis display components."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi.plots.distributions import build_correlation_heatmap
from tsi.services.analytics import generate_correlation_insights


def render_automated_insights(insights: list[str]) -> None:
    """
    Display automated insights as a bulleted list.

    Args:
        insights: List of insight strings
    """
    st.header("✨ Automated Insights")

    for insight in insights:
        st.markdown(f"- {insight}")


def render_correlation_analysis(correlations: pd.DataFrame) -> None:
    """
    Display correlation analysis with heatmap and interpretation.

    Args:
        correlations: DataFrame with correlation data
    """
    st.header("📊 Correlation Analysis")

    if correlations.empty:
        st.info("Insufficient data for correlation analysis")
        return

    # Generate correlation insights
    correlation_insights = generate_correlation_insights(correlations)

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
        fig = build_correlation_heatmap(correlations)
        st.plotly_chart(fig, width='stretch')
