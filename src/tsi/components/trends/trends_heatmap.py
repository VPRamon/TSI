"""Scheduling trends page heatmap display components."""

from __future__ import annotations

from typing import TYPE_CHECKING, cast

import pandas as pd
import streamlit as st

if TYPE_CHECKING:
    from altair import Chart

from tsi.plots.trends import heatmap_visibility_priority


def render_heatmap_section(
    df: pd.DataFrame,
    plot_library: str,
    n_bins: int,
) -> None:
    """
    Display 2D heatmap of visibility × priority.

    Args:
        df: Filtered DataFrame
        plot_library: Plotting library to use ('altair' or 'plotly')
        n_bins: Number of bins for discretization
    """
    st.subheader("3️⃣ Heatmap: Visibility × Priority")
    st.caption(
        "🔥 2D heatmap showing the **mean empirical rate** of scheduling "
        "as a function of visibility (X) and priority (Y)."
    )

    try:
        fig_heatmap = heatmap_visibility_priority(
            df,
            library=plot_library,
            n_bins_vis=n_bins,
            n_bins_priority=n_bins,
        )

        if plot_library == "altair":
            st.altair_chart(cast("Chart", fig_heatmap), width='stretch')
        else:
            st.plotly_chart(fig_heatmap, width='stretch')

    except Exception as e:
        st.error(f"❌ Error generating heatmap: {e}")
