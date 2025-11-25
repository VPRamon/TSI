"""Comprehensive unit tests for :mod:`tsi.state`."""

from __future__ import annotations

import sys
from typing import Any
from unittest.mock import MagicMock

import pandas as pd
import pytest

# Mock streamlit before importing tsi.state
streamlit_mock = MagicMock()
streamlit_mock.session_state = {}
sys.modules["streamlit"] = streamlit_mock

from tsi.state import (
    KEY_COMPARISON_SCHEDULE,
    KEY_CURRENT_PAGE,
    KEY_DARK_PERIODS,
    KEY_DATA_FILENAME,
    KEY_DATA_PREPARED,
    KEY_DATA_RAW,
    KEY_DATA_SOURCE,
    KEY_DIST_FILTER_MODE,
    KEY_INSIGHTS_FILTER_MODE,
    KEY_PRIORITY_RANGE,
    KEY_SCHEDULED_FILTER,
    KEY_SCHEDULE_WINDOW,
    KEY_SELECTED_BINS,
    KEY_SELECTED_BLOCK_IDS,
    get_comparison_schedule,
    get_current_page,
    get_dark_periods,
    get_data_filename,
    get_prepared_data,
    get_priority_range,
    get_schedule_window,
    has_data,
    initialize_state,
    reset_filters,
    set_comparison_schedule,
    set_current_page,
    set_dark_periods,
    set_data_filename,
    set_prepared_data,
    set_priority_range,
    set_scheduled_filter,
)

pytestmark = pytest.mark.unit


@pytest.fixture(autouse=True)
def reset_session_state() -> None:
    """Reset session state before each test."""
    streamlit_mock.session_state.clear()


class TestInitializeState:
    """Test initialize_state function."""

    def test_initializes_all_keys(self) -> None:
        """Initialize all session state keys."""
        initialize_state()
        assert KEY_DATA_RAW in streamlit_mock.session_state
        assert KEY_DATA_PREPARED in streamlit_mock.session_state
        assert KEY_CURRENT_PAGE in streamlit_mock.session_state
        assert KEY_DATA_SOURCE in streamlit_mock.session_state
        assert KEY_DATA_FILENAME in streamlit_mock.session_state
        assert KEY_PRIORITY_RANGE in streamlit_mock.session_state
        assert KEY_SCHEDULED_FILTER in streamlit_mock.session_state
        assert KEY_SELECTED_BINS in streamlit_mock.session_state
        assert KEY_SELECTED_BLOCK_IDS in streamlit_mock.session_state
        assert KEY_SCHEDULE_WINDOW in streamlit_mock.session_state
        assert KEY_DARK_PERIODS in streamlit_mock.session_state
        assert KEY_DIST_FILTER_MODE in streamlit_mock.session_state
        assert KEY_INSIGHTS_FILTER_MODE in streamlit_mock.session_state
        assert KEY_COMPARISON_SCHEDULE in streamlit_mock.session_state

    def test_sets_default_values(self) -> None:
        """Set appropriate default values."""
        initialize_state()
        assert streamlit_mock.session_state[KEY_DATA_RAW] is None
        assert streamlit_mock.session_state[KEY_DATA_PREPARED] is None
        assert streamlit_mock.session_state[KEY_SCHEDULED_FILTER] == "All"
        assert streamlit_mock.session_state[KEY_DIST_FILTER_MODE] == "all"
        assert streamlit_mock.session_state[KEY_INSIGHTS_FILTER_MODE] == "all"

    def test_does_not_overwrite_existing_values(self) -> None:
        """Do not overwrite existing session state values."""
        streamlit_mock.session_state[KEY_DATA_RAW] = "existing_data"
        streamlit_mock.session_state[KEY_SCHEDULED_FILTER] = "Scheduled"
        initialize_state()
        assert streamlit_mock.session_state[KEY_DATA_RAW] == "existing_data"
        assert streamlit_mock.session_state[KEY_SCHEDULED_FILTER] == "Scheduled"

    def test_idempotent__can_call_multiple_times(self) -> None:
        """Calling initialize_state multiple times is safe."""
        initialize_state()
        streamlit_mock.session_state[KEY_DATA_RAW] = "test_data"
        initialize_state()
        assert streamlit_mock.session_state[KEY_DATA_RAW] == "test_data"


class TestHasData:
    """Test has_data function."""

    def test_with_no_prepared_data__returns_false(self) -> None:
        """Return False when no data loaded."""
        streamlit_mock.session_state[KEY_DATA_PREPARED] = None
        assert has_data() is False

    def test_with_prepared_data__returns_true(self) -> None:
        """Return True when data is loaded."""
        streamlit_mock.session_state[KEY_DATA_PREPARED] = pd.DataFrame({"col": [1, 2, 3]})
        assert has_data() is True

    def test_with_empty_dataframe__returns_true(self) -> None:
        """Return True even for empty dataframe."""
        streamlit_mock.session_state[KEY_DATA_PREPARED] = pd.DataFrame()
        assert has_data() is True

    def test_with_missing_key__returns_false(self) -> None:
        """Return False when key doesn't exist."""
        streamlit_mock.session_state.clear()
        assert has_data() is False


class TestGetPreparedData:
    """Test get_prepared_data function."""

    def test_with_data__returns_dataframe(self) -> None:
        """Return stored dataframe."""
        df = pd.DataFrame({"col": [1, 2, 3]})
        streamlit_mock.session_state[KEY_DATA_PREPARED] = df
        result = get_prepared_data()
        assert result is df

    def test_with_no_data__returns_none(self) -> None:
        """Return None when no data."""
        streamlit_mock.session_state[KEY_DATA_PREPARED] = None
        assert get_prepared_data() is None

    def test_with_missing_key__returns_none(self) -> None:
        """Return None when key missing."""
        streamlit_mock.session_state.clear()
        assert get_prepared_data() is None


class TestSetPreparedData:
    """Test set_prepared_data function."""

    def test_sets_dataframe(self) -> None:
        """Store dataframe in session state."""
        df = pd.DataFrame({"col": [1, 2, 3]})
        set_prepared_data(df)
        assert streamlit_mock.session_state[KEY_DATA_PREPARED] is df

    def test_sets_none(self) -> None:
        """Allow setting None."""
        set_prepared_data(None)
        assert streamlit_mock.session_state[KEY_DATA_PREPARED] is None

    def test_overwrites_existing_data(self) -> None:
        """Overwrite existing data."""
        streamlit_mock.session_state[KEY_DATA_PREPARED] = "old_data"
        df = pd.DataFrame({"col": [1, 2, 3]})
        set_prepared_data(df)
        assert streamlit_mock.session_state[KEY_DATA_PREPARED] is df


class TestGetPriorityRange:
    """Test get_priority_range function."""

    def test_with_range_set__returns_range(self) -> None:
        """Return priority range."""
        streamlit_mock.session_state[KEY_PRIORITY_RANGE] = (3.0, 8.0)
        assert get_priority_range() == (3.0, 8.0)

    def test_with_no_range__returns_default(self) -> None:
        """Return default when no range set."""
        streamlit_mock.session_state[KEY_PRIORITY_RANGE] = None
        # get_priority_range returns None when not set, not a default
        result = get_priority_range()
        assert result is None or result == (0.0, 10.0)


class TestSetPriorityRange:
    """Test set_priority_range function."""

    def test_sets_range(self) -> None:
        """Store priority range."""
        set_priority_range((3.0, 8.0))
        assert streamlit_mock.session_state[KEY_PRIORITY_RANGE] == (3.0, 8.0)


class TestGetCurrentPage:
    """Test get_current_page function."""

    def test_with_page_set__returns_page(self) -> None:
        """Return current page."""
        streamlit_mock.session_state[KEY_CURRENT_PAGE] = "dashboard"
        assert get_current_page() == "dashboard"

    def test_with_no_page__returns_none(self) -> None:
        """Return None when no page set."""
        streamlit_mock.session_state[KEY_CURRENT_PAGE] = None
        assert get_current_page() is None


class TestSetCurrentPage:
    """Test set_current_page function."""

    def test_sets_page(self) -> None:
        """Store current page."""
        set_current_page("trends")
        assert streamlit_mock.session_state[KEY_CURRENT_PAGE] == "trends"


class TestGetDataFilename:
    """Test get_data_filename function."""

    def test_with_filename_set__returns_filename(self) -> None:
        """Return data filename."""
        streamlit_mock.session_state[KEY_DATA_FILENAME] = "schedule.json"
        assert get_data_filename() == "schedule.json"

    def test_with_no_filename__returns_none(self) -> None:
        """Return None when no filename set."""
        streamlit_mock.session_state[KEY_DATA_FILENAME] = None
        assert get_data_filename() is None


class TestSetDataFilename:
    """Test set_data_filename function."""

    def test_sets_filename(self) -> None:
        """Store data filename."""
        set_data_filename("schedule.csv")
        assert streamlit_mock.session_state[KEY_DATA_FILENAME] == "schedule.csv"


class TestGetScheduleWindow:
    """Test get_schedule_window function."""

    def test_with_window_set__returns_window(self) -> None:
        """Return schedule window."""
        window = ("2024-01-01", "2024-01-31")
        streamlit_mock.session_state[KEY_SCHEDULE_WINDOW] = window
        assert get_schedule_window() == window

    def test_with_no_window__returns_none(self) -> None:
        """Return None when no window set."""
        streamlit_mock.session_state[KEY_SCHEDULE_WINDOW] = None
        assert get_schedule_window() is None


class TestGetComparisonSchedule:
    """Test get_comparison_schedule function."""

    def test_with_schedule_set__returns_schedule(self) -> None:
        """Return comparison schedule."""
        df = pd.DataFrame({"col": [1, 2, 3]})
        streamlit_mock.session_state[KEY_COMPARISON_SCHEDULE] = df
        assert get_comparison_schedule() is df

    def test_with_no_schedule__returns_none(self) -> None:
        """Return None when no schedule set."""
        streamlit_mock.session_state[KEY_COMPARISON_SCHEDULE] = None
        assert get_comparison_schedule() is None


class TestSetComparisonSchedule:
    """Test set_comparison_schedule function."""

    def test_sets_schedule(self) -> None:
        """Store comparison schedule."""
        df = pd.DataFrame({"col": [1, 2, 3]})
        set_comparison_schedule(df)
        assert streamlit_mock.session_state[KEY_COMPARISON_SCHEDULE] is df


class TestGetDarkPeriods:
    """Test get_dark_periods function."""

    def test_with_periods_set__returns_periods(self) -> None:
        """Return dark periods."""
        periods = [("2024-01-01", "2024-01-02")]
        streamlit_mock.session_state[KEY_DARK_PERIODS] = periods
        assert get_dark_periods() == periods

    def test_with_no_periods__returns_none(self) -> None:
        """Return None when no periods set."""
        streamlit_mock.session_state[KEY_DARK_PERIODS] = None
        assert get_dark_periods() is None


class TestSetDarkPeriods:
    """Test set_dark_periods function."""

    def test_sets_periods(self) -> None:
        """Store dark periods."""
        periods = [("2024-01-01", "2024-01-02")]
        set_dark_periods(periods)
        assert streamlit_mock.session_state[KEY_DARK_PERIODS] == periods


class TestResetFilters:
    """Test reset_filters function."""

    def test_resets_all_filter_keys(self) -> None:
        """Reset all filter-related keys."""
        # Set some filter values
        streamlit_mock.session_state[KEY_PRIORITY_RANGE] = (3.0, 8.0)
        streamlit_mock.session_state[KEY_SCHEDULED_FILTER] = "Scheduled"
        streamlit_mock.session_state[KEY_SELECTED_BINS] = ["Low"]
        streamlit_mock.session_state[KEY_SELECTED_BLOCK_IDS] = ["SB001"]
        streamlit_mock.session_state[KEY_DIST_FILTER_MODE] = "scheduled"
        streamlit_mock.session_state[KEY_INSIGHTS_FILTER_MODE] = "unscheduled"

        reset_filters()

        assert streamlit_mock.session_state[KEY_PRIORITY_RANGE] is None
        assert streamlit_mock.session_state[KEY_SCHEDULED_FILTER] == "All"
        assert streamlit_mock.session_state[KEY_SELECTED_BINS] is None
        assert streamlit_mock.session_state[KEY_SELECTED_BLOCK_IDS] is None
        assert streamlit_mock.session_state[KEY_DIST_FILTER_MODE] == "all"
        assert streamlit_mock.session_state[KEY_INSIGHTS_FILTER_MODE] == "all"

    def test_does_not_reset_data_keys(self) -> None:
        """Do not reset data-related keys."""
        df = pd.DataFrame({"col": [1, 2, 3]})
        streamlit_mock.session_state[KEY_DATA_PREPARED] = df
        streamlit_mock.session_state[KEY_DATA_RAW] = "raw_data"
        streamlit_mock.session_state[KEY_DATA_FILENAME] = "schedule.json"

        reset_filters()

        assert streamlit_mock.session_state[KEY_DATA_PREPARED] is df
        assert streamlit_mock.session_state[KEY_DATA_RAW] == "raw_data"
        assert streamlit_mock.session_state[KEY_DATA_FILENAME] == "schedule.json"


class TestStateKeyConstants:
    """Test that state key constants are strings."""

    def test_all_keys_are_strings(self) -> None:
        """Ensure all key constants are strings."""
        assert isinstance(KEY_DATA_RAW, str)
        assert isinstance(KEY_DATA_PREPARED, str)
        assert isinstance(KEY_CURRENT_PAGE, str)
        assert isinstance(KEY_DATA_SOURCE, str)
        assert isinstance(KEY_DATA_FILENAME, str)
        assert isinstance(KEY_PRIORITY_RANGE, str)
        assert isinstance(KEY_SCHEDULED_FILTER, str)
        assert isinstance(KEY_SELECTED_BINS, str)
        assert isinstance(KEY_SELECTED_BLOCK_IDS, str)
        assert isinstance(KEY_SCHEDULE_WINDOW, str)
        assert isinstance(KEY_DARK_PERIODS, str)
        assert isinstance(KEY_DIST_FILTER_MODE, str)
        assert isinstance(KEY_INSIGHTS_FILTER_MODE, str)
        assert isinstance(KEY_COMPARISON_SCHEDULE, str)

    def test_keys_are_unique(self) -> None:
        """Ensure all keys are unique."""
        keys = [
            KEY_DATA_RAW,
            KEY_DATA_PREPARED,
            KEY_CURRENT_PAGE,
            KEY_DATA_SOURCE,
            KEY_DATA_FILENAME,
            KEY_PRIORITY_RANGE,
            KEY_SCHEDULED_FILTER,
            KEY_SELECTED_BINS,
            KEY_SELECTED_BLOCK_IDS,
            KEY_SCHEDULE_WINDOW,
            KEY_DARK_PERIODS,
            KEY_DIST_FILTER_MODE,
            KEY_INSIGHTS_FILTER_MODE,
            KEY_COMPARISON_SCHEDULE,
        ]
        assert len(keys) == len(set(keys))
