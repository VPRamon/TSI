"""Session state management utilities."""

from typing import Any

import streamlit as st

# State keys
KEY_DATA_RAW = "data_raw"
KEY_DATA_PREPARED = "data_prepared"
KEY_CURRENT_PAGE = "current_page"
KEY_DATA_SOURCE = "data_source"
KEY_DATA_FILENAME = "data_filename"
KEY_PRIORITY_RANGE = "priority_range"
KEY_SCHEDULED_FILTER = "scheduled_filter"
KEY_SELECTED_BINS = "selected_bins"
KEY_SELECTED_BLOCK_IDS = "selected_block_ids"
KEY_SCHEDULE_WINDOW = "scheduled_time_window"
KEY_DARK_PERIODS = "dark_periods"
KEY_DIST_FILTER_MODE = "dist_filter_mode"
KEY_INSIGHTS_FILTER_MODE = "insights_filter_mode"
KEY_COMPARISON_SCHEDULE = "comparison_schedule"


def initialize_state() -> None:
    """Initialize session state with default values."""
    if KEY_DATA_RAW not in st.session_state:
        st.session_state[KEY_DATA_RAW] = None

    if KEY_DATA_PREPARED not in st.session_state:
        st.session_state[KEY_DATA_PREPARED] = None

    if KEY_CURRENT_PAGE not in st.session_state:
        st.session_state[KEY_CURRENT_PAGE] = None

    if KEY_DATA_SOURCE not in st.session_state:
        st.session_state[KEY_DATA_SOURCE] = None

    if KEY_DATA_FILENAME not in st.session_state:
        st.session_state[KEY_DATA_FILENAME] = None

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

    if KEY_DARK_PERIODS not in st.session_state:
        st.session_state[KEY_DARK_PERIODS] = None

    if KEY_DIST_FILTER_MODE not in st.session_state:
        st.session_state[KEY_DIST_FILTER_MODE] = "all"

    if KEY_INSIGHTS_FILTER_MODE not in st.session_state:
        st.session_state[KEY_INSIGHTS_FILTER_MODE] = "all"

    if KEY_COMPARISON_SCHEDULE not in st.session_state:
        st.session_state[KEY_COMPARISON_SCHEDULE] = None


def has_data() -> bool:
    """Check if data has been loaded."""
    return st.session_state.get(KEY_DATA_PREPARED) is not None


def get_prepared_data() -> Any:
    """Get the prepared DataFrame from session state."""
    return st.session_state.get(KEY_DATA_PREPARED)


def set_prepared_data(df: Any) -> None:
    """Set the prepared DataFrame in session state."""
    st.session_state[KEY_DATA_PREPARED] = df


def get_current_page() -> str | None:
    """Get the current page name."""
    return st.session_state.get(KEY_CURRENT_PAGE)


def set_current_page(page: str) -> None:
    """Set the current page name."""
    st.session_state[KEY_CURRENT_PAGE] = page


def get_priority_range() -> tuple[float, float]:
    """Get the priority filter range."""
    result = st.session_state.get(KEY_PRIORITY_RANGE, (0.0, 10.0))
    return result  # type: ignore[return-value,no-any-return]


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


def get_dark_periods() -> Any:
    """Return the loaded dark periods DataFrame, if any."""

    return st.session_state.get(KEY_DARK_PERIODS)


def set_dark_periods(df: Any | None) -> None:
    """Store dark periods data in the session state."""

    st.session_state[KEY_DARK_PERIODS] = df


def get_schedule_window() -> Any:
    """Get selected scheduled time window."""
    return st.session_state.get(KEY_SCHEDULE_WINDOW)


def get_data_filename() -> str | None:
    """Get the loaded dataset filename."""
    return st.session_state.get(KEY_DATA_FILENAME)


def set_data_filename(filename: str) -> None:
    """Set the loaded dataset filename."""
    st.session_state[KEY_DATA_FILENAME] = filename


def get_comparison_schedule() -> Any:
    """Get the comparison schedule DataFrame from session state."""
    return st.session_state.get(KEY_COMPARISON_SCHEDULE)


def set_comparison_schedule(df: Any) -> None:
    """Set the comparison schedule DataFrame in session state."""
    st.session_state[KEY_COMPARISON_SCHEDULE] = df
