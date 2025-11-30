"""Scheduling trends page smoothed trends components."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd
import streamlit as st

if TYPE_CHECKING:
    from tsi_rust import SmoothedPoint


def render_smoothed_trends(
    smoothed_points: list[SmoothedPoint],
    x_var_name: str,
) -> None:
    """
    Display smoothed trends section.

    Args:
        smoothed_points: List of SmoothedPoint from Rust backend
        x_var_name: Name of the x variable ('total_visibility_hours' or 'requested_hours')
    """
    if not smoothed_points:
        st.warning(f"No smoothed data available for {x_var_name}")
        return

    # Convert to DataFrame for plotting
    df = pd.DataFrame([
        {
            "x": p.x,
            "rate": p.y_smoothed,
            "n_samples": p.n_samples,
        }
        for p in smoothed_points
    ])

    # Create simple line chart
    st.markdown(f"**Smoothed Scheduling Rate by {x_var_name}**")
    st.line_chart(df.set_index("x")["rate"])
    
    with st.expander("View smoothed data"):
        display_df = df.copy()
        display_df.columns = [x_var_name, "Rate", "Samples"]
        display_df["Rate"] = display_df["Rate"].apply(lambda x: f"{x:.1%}")
        st.dataframe(display_df, width="stretch")
