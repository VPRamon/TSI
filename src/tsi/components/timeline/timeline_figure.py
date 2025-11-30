"""Timeline figure rendering helper component."""

from __future__ import annotations

import streamlit as st

from tsi.plots.timeline_monthly import build_monthly_timeline


def render_timeline_figure(
    blocks: list,
    priority_range: tuple[float, float],
    dark_periods: list[tuple[float, float]] | None = None,
    key: str = "timeline_chart",
) -> object:
    """Build and render the monthly timeline Plotly figure.

    This function constructs the scheduled timeline visualization showing
    monthly chronological view of scheduled observations, and displays it
    via Streamlit.

    Args:
        blocks: List of ScheduleTimelineBlock objects with scheduled observations
        priority_range: Tuple of (min_priority, max_priority) for color normalization
        dark_periods: Optional list of (start_mjd, stop_mjd) tuples for dark period windows
        key: Streamlit chart key

    Returns:
        The Plotly figure object produced by build_monthly_timeline.
    """
    # Build the monthly timeline figure
    fig = build_monthly_timeline(
        blocks=blocks,
        priority_range=priority_range,
        dark_periods=dark_periods,
    )

    # Configure the Plotly chart display
    config = {
        "displayModeBar": True,
        "displaylogo": False,
        "modeBarButtonsToRemove": ["lasso2d", "select2d"],
        "scrollZoom": True,
    }

    # Display the figure
    st.plotly_chart(fig, use_container_width=True, config=config, key=key)

    return fig
