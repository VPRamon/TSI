"""Sky Map control components for filtering and configuration."""

from __future__ import annotations

from collections.abc import Sequence
from datetime import datetime, timedelta

import pandas as pd
import streamlit as st

from tsi import state
from tsi.components.toolbar import render_reset_filters_button, render_toggle


def render_sidebar_controls(
    df: pd.DataFrame,
    priority_min: float,
    priority_max: float,
    priority_bins: list[str],
) -> dict:
    """
    Render all sidebar controls for sky map filtering.

    Args:
        df: The prepared DataFrame
        priority_min: Minimum priority value
        priority_max: Maximum priority value
        priority_bins: List of available priority bins

    Returns:
        Dictionary containing all control values
    """
    st.markdown("### 🎛️ Settings")
    with st.container(border=True):
        # Status and color selectors
        scheduled_filter = _render_status_selector()
        color_column = _render_color_selector()

        st.markdown("---")

        # Sliders
        priority_range = _render_priority_slider(priority_min, priority_max)
        schedule_window = _render_schedule_window_control(df)

        st.markdown("---")

        # Toggles
        flip_ra = render_toggle(
            "Invertir eje RA",
            default=True,
            key="sky_flip_ra",
        )

        if render_reset_filters_button():
            state.reset_filters()
            reset_sky_map_controls()

    return {
        "scheduled_filter": scheduled_filter,
        "color_column": color_column,
        "priority_range": priority_range,
        "selected_bins": priority_bins,  # Using all bins
        "schedule_window": schedule_window,
        "flip_ra": flip_ra,
    }


def _render_status_selector() -> str:
    """Render scheduling status radio selector."""
    scheduled_filter = st.radio(
        "Scheduling Status",
        options=["All", "Scheduled", "Unscheduled"],
        key="sky_scheduled_filter",
    )
    state.set_scheduled_filter(scheduled_filter)
    return scheduled_filter


def _render_color_selector() -> str:
    """Render color-by option selector."""
    color_options = {
        "Priority": "priority_bin",
        "Status": "scheduled_flag",
    }

    color_choice = st.radio(
        "Color by",
        options=list(color_options.keys()),
        index=0,
        key="sky_color_choice",
    )
    return color_options[color_choice]


def _render_priority_slider(priority_min: float, priority_max: float) -> tuple[float, float]:
    """Render priority range slider."""
    stored_range = state.get_priority_range()
    
    # Determine default range
    if (
        stored_range is None
        or stored_range[0] < priority_min
        or stored_range[1] > priority_max
        or stored_range == (0.0, 10.0)
    ):
        default_range = (priority_min, priority_max)
    else:
        default_range = (
            max(priority_min, stored_range[0]),
            min(priority_max, stored_range[1]),
        )

    priority_range = st.slider(
        "Priority Range",
        min_value=priority_min,
        max_value=priority_max,
        value=default_range,
        step=0.1,
        key="sky_priority_range",
    )
    state.set_priority_range(priority_range)
    return priority_range


def _render_schedule_window_control(df: pd.DataFrame) -> tuple[datetime, datetime] | None:
    """Render slider to filter by scheduled start datetime."""
    if "scheduled_flag" not in df.columns or "scheduled_start_dt" not in df.columns:
        st.caption("The dataset does not include scheduling dates.")
        return None

    scheduled = df[df["scheduled_flag"]]["scheduled_start_dt"].dropna()
    if scheduled.empty:
        st.caption("No observations include a scheduled date.")
        return None

    min_dt = scheduled.min()
    max_dt = scheduled.max()

    min_dt_dt = _ts_to_datetime(min_dt)
    max_dt_dt = _ts_to_datetime(max_dt)

    if min_dt_dt == max_dt_dt:
        max_dt_dt = max_dt_dt + timedelta(hours=1)

    stored_window = state.get_schedule_window()
    if stored_window:
        default_window = (
            max(min_dt_dt, stored_window[0]),
            min(max_dt_dt, stored_window[1]),
        )
    else:
        default_window = (min_dt_dt, max_dt_dt)

    return st.slider(
        "Scheduled window (UTC)",
        min_value=min_dt_dt,
        max_value=max_dt_dt,
        value=default_window,
        format="YYYY-MM-DD HH:mm",
        key="sky_schedule_window",
        help="Show only scheduled targets whose start time falls within this window.",
    )


def _ts_to_datetime(ts: pd.Timestamp) -> datetime:
    """Convert pandas Timestamp to naive datetime preserving UTC."""
    from typing import cast
    
    # Floor to seconds to avoid nanoseconds warning
    ts = ts.floor("s")
    if ts.tzinfo is not None:
        return cast(datetime, ts.tz_convert(None).to_pydatetime())
    return cast(datetime, ts.to_pydatetime())


def reset_sky_map_controls() -> None:
    """Clear widget-level state so defaults apply after reset."""
    # Delete all widget keys that start with 'sky_' to reset all sky map widgets
    keys_to_delete = [
        key for key in st.session_state.keys() 
        if isinstance(key, str) and key.startswith("sky_")
    ]

    for key in keys_to_delete:
        del st.session_state[key]
