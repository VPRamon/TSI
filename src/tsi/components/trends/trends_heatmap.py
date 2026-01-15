"""Scheduling trends page heatmap display components."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd
import streamlit as st

if TYPE_CHECKING:
    from tsi_rust import HeatmapBin


def render_heatmap_section(
    heatmap_bins: list[HeatmapBin],
    n_bins: int,
) -> None:
    """
    Display 2D heatmap of visibility × requested time.

    Args:
        heatmap_bins: List of HeatmapBin from Rust backend
        n_bins: Number of bins for discretization
    """
    if not heatmap_bins:
        st.warning("No heatmap data available")
        return

    # Convert to DataFrame for plotting
    df = pd.DataFrame(
        [
            {
                "vis_bin": b.visibility_mid,
                "time_bin": b.time_mid,
                "rate": b.scheduled_rate,
                "count": b.count,
            }
            for b in heatmap_bins
        ]
    )

    # Pivot for heatmap display
    heatmap_data = df.pivot(index="time_bin", columns="vis_bin", values="rate")

    st.markdown("**Heatmap: Visibility × Requested Time**")
    st.caption("Shows scheduling rate for different combinations of visibility and requested time")

    # Use Streamlit's experimental data editor with color mapping
    st.dataframe(
        heatmap_data.style.background_gradient(cmap="RdYlGn", vmin=0, vmax=1), width="stretch"
    )

    with st.expander("View heatmap data"):
        display_df = df.copy()
        display_df["rate"] = display_df["rate"].apply(lambda x: f"{x:.1%}")
        st.dataframe(display_df, width="stretch")
