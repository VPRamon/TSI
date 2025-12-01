"""Observation details table component for scheduled timeline."""

from __future__ import annotations

import pandas as pd
import streamlit as st

from tsi.services.utils.time import mjd_to_datetime


def _apply_filters(blocks: list, filters: dict) -> list:
    """
    Apply search and filter criteria to blocks.

    Args:
        blocks: List of ScheduleTimelineBlock objects
        filters: Dictionary with filter values from render_search_filters

    Returns:
        Filtered list of blocks
    """
    filtered_blocks = blocks

    # Filter by block ID
    if filters["search_id"]:
        search_term = filters["search_id"].lower()
        filtered_blocks = [
            block for block in filtered_blocks
            if search_term in str(block.scheduling_block_id).lower()
        ]

    # Filter by month
    if filters["search_month"]:
        filtered_blocks = [
            block for block in filtered_blocks
            if mjd_to_datetime(block.scheduled_start_mjd).strftime("%Y-%m").startswith(filters["search_month"])
        ]

    # Filter by minimum priority
    if filters["min_priority_filter"] is not None:
        filtered_blocks = [
            block for block in filtered_blocks
            if block.priority >= filters["min_priority_filter"]
        ]

    return filtered_blocks


def _blocks_to_dataframe(blocks: list) -> pd.DataFrame:
    """
    Convert blocks to a formatted DataFrame for display.

    Args:
        blocks: List of ScheduleTimelineBlock objects

    Returns:
        DataFrame with formatted columns
    """
    display_data = []
    for block in blocks:
        display_data.append({
            "Block ID": block.scheduling_block_id,
            "Priority": block.priority,
            "Scheduled Start": mjd_to_datetime(block.scheduled_start_mjd).strftime("%Y-%m-%d %H:%M UTC"),
            "Scheduled Stop": mjd_to_datetime(block.scheduled_stop_mjd).strftime("%Y-%m-%d %H:%M UTC"),
            "Duration (h)": (block.scheduled_stop_mjd - block.scheduled_start_mjd) * 24.0,
            "RA (°)": block.ra_deg,
            "Dec (°)": block.dec_deg,
            "Requested (h)": block.requested_hours,
            "Total Visibility (h)": block.total_visibility_hours,
        })

    return pd.DataFrame(display_data)


def render_observation_details_table(blocks: list, filters: dict) -> pd.DataFrame:
    """
    Render observation details table with search and filter capabilities.

    Displays a filterable table of scheduled observations with download option.

    Args:
        blocks: List of ScheduleTimelineBlock objects
        filters: Dictionary with filter values from render_search_filters

    Returns:
        DataFrame of filtered observations (for potential further use)
    """
    st.markdown("---")
    st.subheader("📊 Observation Details")

    # Apply filters
    filtered_blocks = _apply_filters(blocks, filters)

    # Display count
    st.caption(f"Showing {len(filtered_blocks):,} of {len(blocks):,} observations")

    # Convert to DataFrame
    display_df = _blocks_to_dataframe(filtered_blocks)

    # Display the table with formatting
    if not display_df.empty:
        st.dataframe(
            display_df.style.format(
                {
                    "Priority": "{:.2f}",
                    "Duration (h)": "{:.2f}",
                    "RA (°)": "{:.2f}",
                    "Dec (°)": "{:.2f}",
                    "Requested (h)": "{:.2f}",
                    "Total Visibility (h)": "{:.2f}",
                },
                na_rep="-",
            ),
            width='stretch',
            height=400,
            hide_index=True,
        )

        # Download button
        _render_download_button(display_df)
    else:
        st.info("No observations match the current filters.")

    return display_df


def _render_download_button(display_df: pd.DataFrame) -> None:
    """
    Render CSV download button for the observations table.

    Args:
        display_df: DataFrame to download
    """
    csv = display_df.to_csv(index=False).encode("utf-8")
    st.download_button(
        label="📥 Download table as CSV",
        data=csv,
        file_name=f"scheduled_timeline_{pd.Timestamp.now().strftime('%Y%m%d_%H%M%S')}.csv",
        mime="text/csv",
        key="timeline_download_csv",
    )
