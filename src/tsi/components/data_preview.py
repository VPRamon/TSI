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


def render_summary_table(summary_data: dict[str, Any]) -> None:
    """
    Render a summary table from key-value pairs.

    Args:
        summary_data: Dictionary of label -> value pairs
    """
    df = pd.DataFrame(
        {
            "Metric": list(summary_data.keys()),
            "Value": list(summary_data.values()),
        }
    )

    st.dataframe(df, width="stretch", hide_index=True)


def render_filterable_table(
    df: pd.DataFrame,
    searchable_columns: list[str] | None = None,
    title: str = "Data Table",
) -> pd.DataFrame:
    """
    Render a table with search/filter capabilities.

    Args:
        df: DataFrame to display
        searchable_columns: Columns to enable text search on
        title: Table title

    Returns:
        Filtered DataFrame
    """
    st.subheader(title)

    filtered_df = df.copy()

    if searchable_columns:
        col1, col2 = st.columns([3, 1])

        with col1:
            search_term = st.text_input(
                "Search",
                placeholder=f"Search in: {', '.join(searchable_columns)}",
            )

        if search_term:
            # Filter rows where any searchable column contains the search term
            mask = (
                filtered_df[searchable_columns]
                .apply(lambda col: col.astype(str).str.contains(search_term, case=False, na=False))
                .any(axis=1)
            )
            filtered_df = filtered_df[mask]

    st.dataframe(
        filtered_df,
        width="stretch",
        hide_index=True,
        height=400,
    )

    st.caption(f"Showing {len(filtered_df)} of {len(df)} rows")

    return filtered_df


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
