"""Data preview and table components."""

from typing import Any

import pandas as pd
import streamlit as st


def render_data_preview(
    df: pd.DataFrame,
    max_rows: int = 10,
    columns: list[str] | None = None,
    title: str = "Data Preview",
) -> None:
    """
    Render a data preview table with optional column selection.

    Args:
        df: DataFrame to display
        max_rows: Maximum number of rows to show
        columns: Specific columns to display (None for all)
        title: Section title
    """
    st.subheader(title)

    if columns:
        display_df = df[columns].head(max_rows)
    else:
        display_df = df.head(max_rows)

    st.dataframe(
        display_df,
        width="stretch",
        hide_index=True,
    )

    st.caption(f"Showing {len(display_df)} of {len(df)} rows")


def render_conflicts_table(conflicts_df: pd.DataFrame) -> None:
    """
    Render a formatted conflicts table with highlighting.

    Args:
        conflicts_df: DataFrame with conflict information
    """
    if conflicts_df.empty:
        st.success("✅ No scheduling conflicts detected!")
        return

    st.warning(f"⚠️ {len(conflicts_df)} scheduling conflicts detected")

    st.dataframe(
        conflicts_df,
        width="stretch",
        hide_index=True,
        height=300,
    )
