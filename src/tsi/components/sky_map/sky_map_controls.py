"""Sky Map control components for filtering and configuration."""

from __future__ import annotations

from datetime import datetime, timedelta, timezone
from typing import TYPE_CHECKING, Any

import streamlit as st

from tsi import state
from tsi.components.toolbar.toolbar import (
    render_priority_range_control,
    render_reset_filters_button,
    render_toggle,
)
from tsi.services.time_utils import ModifiedJulianDate

from tsi_rust import Period

if TYPE_CHECKING:
    from tsi_rust import LightweightBlock, SkyMapData


def render_sidebar_controls(
    blocks: list[Any],
    priority_min: float,
    priority_max: float,
    priority_bins: list[str],
) -> dict:
    """
    Render all sidebar controls for sky map filtering.

    Args:
        blocks: List of SchedulingBlock PyO3 objects
        priority_min: Minimum priority value
        priority_max: Maximum priority value
        priority_bins: List of available priority bins

    Returns:
        Dictionary containing all control values
    """
    st.markdown("### 🎛️ Settings")
    with st.container(border=True):
        scheduled_filter = _render_status_selector()
        color_column = _render_color_selector()

        st.markdown("---")

        stored_range = state.get_priority_range()
        priority_range = render_priority_range_control(
            priority_min,
            priority_max,
            stored_range,
            key="sky_priority_range",
        )
        state.set_priority_range(priority_range)

        schedule_window = _render_schedule_window_control(blocks)

        st.markdown("---")

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
        "selected_bins": priority_bins,
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


def _render_schedule_window_control(blocks: list[LightweightBlock]) -> Period | None:
    """Render slider to filter by scheduled start datetime."""

    scheduled_periods = [
        block.scheduled_period for block in blocks if getattr(block, "scheduled_period", None)
    ]

    if not scheduled_periods:
        st.caption("No observations include a scheduled date.")
        state.set_schedule_window(None)
        return None

    start_datetimes = [_mjd_to_datetime(period.start_mjd) for period in scheduled_periods]
    stop_datetimes = [_mjd_to_datetime(period.stop_mjd) for period in scheduled_periods]

    min_dt = min(start_datetimes)
    max_dt = max(stop_datetimes)

    if min_dt == max_dt:
        max_dt = max_dt + timedelta(hours=1)

    default_window = _get_default_schedule_window(min_dt, max_dt)

    selected_start, selected_stop = st.slider(
        "Scheduled window (UTC)",
        min_value=min_dt,
        max_value=max_dt,
        value=default_window,
        format="YYYY-MM-DD HH:mm",
        key="sky_schedule_window",
        help="Show only scheduled targets whose start time falls within this window.",
    )

    selected_window = Period(
        float(ModifiedJulianDate.from_datetime(selected_start)),
        float(ModifiedJulianDate.from_datetime(selected_stop)),
    )
    state.set_schedule_window(selected_window)
    return selected_window


def _get_default_schedule_window(
    min_dt: datetime,
    max_dt: datetime,
) -> tuple[datetime, datetime]:
    """Determine the default slider window based on stored state."""
    stored_window = state.get_schedule_window()
    stored_start: datetime | None = None
    stored_stop: datetime | None = None

    if isinstance(stored_window, Period):
        stored_start = _mjd_to_datetime(stored_window.start_mjd)
        stored_stop = _mjd_to_datetime(stored_window.stop_mjd)
    elif isinstance(stored_window, tuple) and len(stored_window) == 2:
        stored_start = _normalize_datetime(stored_window[0])
        stored_stop = _normalize_datetime(stored_window[1])

    if stored_start is None or stored_stop is None:
        return (min_dt, max_dt)

    bounded_start = max(min_dt, stored_start)
    bounded_stop = min(max_dt, stored_stop)

    if bounded_start >= bounded_stop:
        return (min_dt, max_dt)

    return (bounded_start, bounded_stop)


def _normalize_datetime(value: Any) -> datetime | None:
    """Convert previous widget values to timezone-aware datetimes."""
    if value is None:
        return None

    candidate = value
    if hasattr(candidate, "to_pydatetime"):
        candidate = candidate.to_pydatetime()

    if isinstance(candidate, datetime):
        if candidate.tzinfo is None:
            return candidate.replace(tzinfo=timezone.utc)
        return candidate.astimezone(timezone.utc)

    return None


def _mjd_to_datetime(mjd_value: float) -> datetime:
    """Helper to convert raw MJD values to datetime for UI controls."""
    return ModifiedJulianDate(mjd_value).to_datetime()


def reset_sky_map_controls() -> None:
    """Clear widget-level state so defaults apply after reset."""
    # Delete all widget keys that start with 'sky_' to reset all sky map widgets
    keys_to_delete = [
        key for key in st.session_state.keys() if isinstance(key, str) and key.startswith("sky_")
    ]

    for key in keys_to_delete:
        del st.session_state[key]
