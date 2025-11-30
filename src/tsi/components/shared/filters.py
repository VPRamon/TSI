"""Shared filter UI components.

This module provides reusable filter widgets that appear across multiple pages,
reducing duplication and ensuring consistent behavior and styling.

Usage:
    from tsi.components.shared.filters import (
        render_exclude_impossible_checkbox,
        render_status_filter,
    )

    # In a page or control module:
    filter_impossible = render_exclude_impossible_checkbox()
    status = render_status_filter(key="my_page_status")
"""

from __future__ import annotations

from typing import Literal

import streamlit as st


def render_exclude_impossible_checkbox(
    *,
    key: str | None = None,
    default: bool = False,
    label: str = "Exclude Impossible",
    help_text: str = "Exclude observations with zero visibility hours",
) -> bool:
    """
    Render a checkbox to exclude impossible/zero-visibility observations.

    This is a common filter pattern used in distributions, insights, and
    other analysis pages.

    Args:
        key: Optional session state key. If None, uses local state.
        default: Default checkbox value (default: False)
        label: Label text for the checkbox
        help_text: Help tooltip text

    Returns:
        True if the checkbox is checked (filter should be applied)

    Example:
        >>> filter_impossible = render_exclude_impossible_checkbox(
        ...     key="my_page_exclude_impossible",
        ...     default=True,
        ... )
        >>> if filter_impossible:
        ...     data = filter_impossible_observations(data)
    """
    return st.checkbox(
        label,
        value=default,
        key=key,
        help=help_text,
    )


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
    return selected  # type: ignore[return-value]


def render_exclude_zero_visibility_checkbox(
    *,
    key: str = "exclude_zero_vis",
    default: bool = True,
    label: str = "Exclude visibility = 0 for model",
    help_text: str = "If enabled, the model is trained only with observations that have visibility > 0",
) -> bool:
    """
    Render a checkbox to exclude zero-visibility observations for model training.

    This is a specialized variant used in trends/modeling pages with
    different defaults and help text than the general filter.

    Args:
        key: Session state key for the widget
        default: Default checkbox value (default: True for model training)
        label: Label text for the checkbox
        help_text: Help tooltip text

    Returns:
        True if zero-visibility should be excluded

    Example:
        >>> exclude = render_exclude_zero_visibility_checkbox()
        >>> if exclude:
        ...     df = df[df["total_visibility_hours"] > 0]
    """
    return st.checkbox(
        label,
        value=default,
        key=key,
        help=help_text,
    )


__all__ = [
    "render_exclude_impossible_checkbox",
    "render_status_filter",
    "render_exclude_zero_visibility_checkbox",
]
