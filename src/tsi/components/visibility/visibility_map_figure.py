"""Visibility Map figure rendering helper component."""

from __future__ import annotations

import streamlit as st

from tsi.plots.timeline import build_visibility_histogram_from_bins
from tsi.services.database import get_schedule_time_range, get_visibility_histogram


def render_visibility_map_figure(
    schedule_id: int | None,
    settings: dict,
    effective_priority_range: tuple[float, float],
    total_blocks: int,
    key: str = "visibility_histogram_chart",
) -> object:
    """Build and render the visibility histogram Plotly figure.

    This function fetches histogram data from the Rust backend, constructs the
    visualization, and displays it via Streamlit.

    Args:
        schedule_id: Schedule ID in the database (required, cannot be None)
        settings: Dict of histogram settings from controls component containing:
            - num_bins: Number of bins (if bin_duration_minutes not set)
            - bin_duration_minutes: Duration of each bin in minutes
            - selected_block_ids: Optional list of specific block IDs to include
        effective_priority_range: Tuple of (min_priority, max_priority) after filtering
        total_blocks: Total number of blocks matching the filters
        key: Streamlit chart key

    Returns:
        The Plotly figure object produced by build_visibility_histogram_from_bins,
        or None if generation fails.

    Raises:
        ValueError: If schedule_id is None or database is not reachable
        RuntimeError: If backend histogram generation fails
    """
    # Validate that we have a schedule ID
    if schedule_id is None:
        raise ValueError(
            "Schedule ID is required to generate visibility map. "
            "Please load a schedule from the database first."
        )

    # Get time range from database
    time_range = get_schedule_time_range(schedule_id)
    if time_range is None:
        raise RuntimeError(
            f"No visibility periods found for schedule ID {schedule_id}. "
            "Cannot generate visibility map without visibility data."
        )

    start_time, end_time = time_range

    # Prepare filters - convert to integers for backend
    priority_range_tuple = (
        int(effective_priority_range[0]),
        int(effective_priority_range[1]),
    )

    # Convert block IDs to list of integers if present
    block_ids_list = None
    if settings.get("selected_block_ids"):
        block_ids_list = [int(bid) for bid in settings["selected_block_ids"]]

    # Determine bin duration
    bin_duration_minutes = settings.get("bin_duration_minutes")
    if bin_duration_minutes is not None and bin_duration_minutes > 0:
        bin_duration_minutes = int(bin_duration_minutes)
    else:
        # Calculate from num_bins
        time_range_minutes = (end_time - start_time).total_seconds() / 60
        num_bins = settings.get("num_bins", 50)
        bin_duration_minutes = max(1, int(time_range_minutes / num_bins))

    # Call backend histogram function
    bins = get_visibility_histogram(
        schedule_id=schedule_id,
        start=start_time,
        end=end_time,
        bin_duration_minutes=bin_duration_minutes,
        priority_range=priority_range_tuple,
        block_ids=block_ids_list,
    )

    # Build visualization from bins
    fig = build_visibility_histogram_from_bins(
        bins=bins,
        total_blocks=total_blocks,
        bin_duration_minutes=bin_duration_minutes,
    )

    # Display the figure
    st.plotly_chart(fig, width="stretch", key=key)

    return fig
