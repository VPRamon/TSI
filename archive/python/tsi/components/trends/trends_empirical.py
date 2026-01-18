"""Empirical trends plotting helpers used by the trends page."""

from __future__ import annotations

from typing import TYPE_CHECKING

import pandas as pd
import streamlit as st

if TYPE_CHECKING:
    from tsi_rust import EmpiricalRatePoint


def render_empirical_proportions(
    empirical_points: list[EmpiricalRatePoint], x_var_name: str
) -> None:
    """
    Render empirical proportions plot section.

    Args:
        empirical_points: List of EmpiricalRatePoint from Rust backend
        x_var_name: Name of the x variable ('priority', 'total_visibility_hours', or 'requested_hours')
    """
    if not empirical_points:
        st.warning(f"No data available for {x_var_name}")
        return

    # Convert to DataFrame for plotting
    df = pd.DataFrame(
        [
            {
                "x": p.mid_value,
                "rate": p.scheduled_rate,
                "count": p.count,
                "label": p.bin_label,
            }
            for p in empirical_points
        ]
    )

    # Create simple plot with Streamlit
    st.markdown(f"**Empirical Scheduling Rate by {x_var_name}**")

    # Display as a table and bar chart
    display_df = df.copy()
    display_df.columns = [x_var_name, "Rate", "Count", "Scheduled"]
    display_df["Rate"] = display_df["Rate"].apply(lambda x: f"{x:.1%}")

    st.bar_chart(df.set_index("x")["rate"])

    with st.expander("View data table"):
        st.dataframe(display_df, width="stretch")
