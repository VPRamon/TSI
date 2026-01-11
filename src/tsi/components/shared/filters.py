"""Shared filter UI components."""

from __future__ import annotations

from typing import Literal, cast

import streamlit as st


def render_status_filter(
    *,
    key: str = "status_filter",
    options: list[str] | None = None,
    default: str = "All",
    label: str = "Scheduling Status",
    horizontal: bool = False,
) -> Literal["All", "Scheduled", "Unscheduled"]:
    """
    Render a radio button filter for scheduling status.

    This is a common filter pattern used in sky map and other pages
    to filter observations by their scheduling status.

    Args:
        key: Session state key for the widget
        options: List of options (default: ["All", "Scheduled", "Unscheduled"])
        default: Default selection (default: "All")
        label: Label text for the radio button group
        horizontal: Whether to render horizontally (default: False)

    Returns:
        Selected status: "All", "Scheduled", or "Unscheduled"

    Example:
        >>> status = render_status_filter(key="sky_map_status")
        >>> filtered_df = backend.filter_by_scheduled(df, status)
    """
    if options is None:
        options = ["All", "Scheduled", "Unscheduled"]

    # Ensure default is in options
    if default not in options:
        default = options[0]

    selected = st.radio(
        label,
        options=options,
        index=options.index(default),
        key=key,
        horizontal=horizontal,
    )
    return cast(Literal['All', 'Scheduled', 'Unscheduled'], selected)


__all__ = [
    "render_status_filter",
]
