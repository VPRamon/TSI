"""Session state management utilities."""

import logging
from typing import Any, cast

import streamlit as st
from tsi_rust import ScheduleId

logger = logging.getLogger(__name__)

# State keys
KEY_CURRENT_PAGE = "current_page"
KEY_DATA_FILENAME = "data_filename"
KEY_SCHEDULE_REF = "schedule_ref"
KEY_SCHEDULE_NAME = "schedule_name"
KEY_PRIORITY_RANGE = "priority_range"
KEY_SCHEDULED_FILTER = "scheduled_filter"
KEY_SELECTED_BINS = "selected_bins"
KEY_SELECTED_BLOCK_IDS = "selected_block_ids"
KEY_SCHEDULE_WINDOW = "scheduled_time_window"
KEY_DIST_FILTER_MODE = "dist_filter_mode"
KEY_INSIGHTS_FILTER_MODE = "insights_filter_mode"


def initialize_state() -> None:
    """Initialize session state with default values."""
    if KEY_CURRENT_PAGE not in st.session_state:
        st.session_state[KEY_CURRENT_PAGE] = None

    if KEY_DATA_FILENAME not in st.session_state:
        st.session_state[KEY_DATA_FILENAME] = None

    if KEY_SCHEDULE_REF not in st.session_state:
        st.session_state[KEY_SCHEDULE_REF] = None

    if KEY_SCHEDULE_NAME not in st.session_state:
        st.session_state[KEY_SCHEDULE_NAME] = None

    if KEY_PRIORITY_RANGE not in st.session_state:
        st.session_state[KEY_PRIORITY_RANGE] = None

    if KEY_SCHEDULED_FILTER not in st.session_state:
        st.session_state[KEY_SCHEDULED_FILTER] = "All"

    if KEY_SELECTED_BINS not in st.session_state:
        st.session_state[KEY_SELECTED_BINS] = None

    if KEY_SELECTED_BLOCK_IDS not in st.session_state:
        st.session_state[KEY_SELECTED_BLOCK_IDS] = None

    if KEY_SCHEDULE_WINDOW not in st.session_state:
        st.session_state[KEY_SCHEDULE_WINDOW] = None

    if KEY_DIST_FILTER_MODE not in st.session_state:
        st.session_state[KEY_DIST_FILTER_MODE] = "all"

    if KEY_INSIGHTS_FILTER_MODE not in st.session_state:
        st.session_state[KEY_INSIGHTS_FILTER_MODE] = "all"


def has_data() -> bool:
    """Check if data has been loaded (either from backend or file upload)."""
    if st.session_state.get(KEY_SCHEDULE_REF) is not None:
        return True
    return False


def get_current_page() -> str | None:
    """Get the current page name."""
    return cast(str | None, st.session_state.get(KEY_CURRENT_PAGE))


def set_current_page(page: str) -> None:
    """Set the current page name."""
    st.session_state[KEY_CURRENT_PAGE] = page


def get_priority_range() -> tuple[float, float]:
    """Get the priority filter range."""
    result = st.session_state.get(KEY_PRIORITY_RANGE, (0.0, 10.0))
    return cast(tuple[float, float], result)


def set_priority_range(range_vals: tuple[float, float]) -> None:
    """Set the priority filter range."""
    st.session_state[KEY_PRIORITY_RANGE] = range_vals


def set_scheduled_filter(filter_val: str) -> None:
    """Set the scheduled/unscheduled filter."""
    st.session_state[KEY_SCHEDULED_FILTER] = filter_val


def reset_filters() -> None:
    """Reset all filters to defaults."""
    # Set to None so each page can use its data's full range
    st.session_state[KEY_PRIORITY_RANGE] = None
    st.session_state[KEY_SCHEDULED_FILTER] = "All"
    st.session_state[KEY_SELECTED_BINS] = None
    st.session_state[KEY_SELECTED_BLOCK_IDS] = None
    st.session_state[KEY_SCHEDULE_WINDOW] = None
    st.session_state[KEY_DIST_FILTER_MODE] = "all"
    st.session_state[KEY_INSIGHTS_FILTER_MODE] = "all"


def get_schedule_window() -> Any:
    """Get selected scheduled time window."""
    return st.session_state.get(KEY_SCHEDULE_WINDOW)


def set_schedule_window(window: Any) -> None:
    """Persist the selected scheduled time window."""
    st.session_state[KEY_SCHEDULE_WINDOW] = window


def get_data_filename() -> str | None:
    """Get the loaded dataset filename."""
    return cast(str | None, st.session_state.get(KEY_DATA_FILENAME))


def set_data_filename(filename: str) -> None:
    """Set the loaded dataset filename."""
    st.session_state[KEY_DATA_FILENAME] = filename


def get_schedule_ref() -> ScheduleId:
    """Get the current schedule reference from the backend."""

    schedule_ref = st.session_state.get(KEY_SCHEDULE_REF)
    if schedule_ref is None:
        raise RuntimeError("Load a schedule from the backend to view the validation report.")

    return schedule_ref if isinstance(schedule_ref, ScheduleId) else ScheduleId(schedule_ref)


def set_schedule_ref(schedule_ref: int | ScheduleId) -> None:
    """Set the current schedule reference."""
    st.session_state[KEY_SCHEDULE_REF] = (
        schedule_ref if isinstance(schedule_ref, ScheduleId) else ScheduleId(schedule_ref)
    )


def get_schedule_name() -> str | None:
    """Get the current schedule name."""
    return cast(str | None, st.session_state.get(KEY_SCHEDULE_NAME))


def set_schedule_name(schedule_name: str | None) -> None:
    """Set the current schedule name."""
    st.session_state[KEY_SCHEDULE_NAME] = schedule_name
