"""Empirical trends plotting helpers used by the trends page."""

from __future__ import annotations

import pandas as pd
import streamlit as st

try:
    from tsi.plots.trends import empirical_proportions_plot
except Exception:  # pragma: no cover - plotting library may evolve
    empirical_proportions_plot = None


def render_empirical_proportions(df: pd.DataFrame, plot_library: str) -> None:
    """
    Render empirical proportions plot section.

    Args:
        df: Filtered DataFrame
        plot_library: 'altair' or 'plotly'
    """
    st.subheader("1️⃣ Empirical proportions by visibility")
    st.caption("Shows the observed scheduling proportion as a function of visibility.")

    if empirical_proportions_plot is None:
        st.error("❌ Empirical proportions plotting function is not available in `tsi.plots.trends`.")
        return

    try:
        fig = empirical_proportions_plot(df, library=plot_library)
        if plot_library == "altair":
            st.altair_chart(fig, width="stretch")
        else:
            st.plotly_chart(fig, width="stretch")
    except Exception as e:
        st.error(f"❌ Error generating empirical proportions: {e}")
