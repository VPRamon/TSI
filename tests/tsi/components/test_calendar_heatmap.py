"""Unit tests for :mod:`tsi.components.calendar_heatmap`."""

from __future__ import annotations

from collections.abc import Iterator

import pandas as pd
import pytest

from tsi.components.calendar_heatmap import build_calendar_heatmap

pytestmark = pytest.mark.unit


@pytest.fixture
def streamlit_mock() -> Iterator[None]:
    """Mock Streamlit widgets used within plotting helpers."""

    import sys
    from unittest.mock import MagicMock

    original = sys.modules.get("streamlit")
    sys.modules["streamlit"] = MagicMock()

    yield

    if original is None:
        del sys.modules["streamlit"]
    else:
        sys.modules["streamlit"] = original


@pytest.fixture
def sample_schedule_df() -> pd.DataFrame:
    """Create a minimal schedule DataFrame for testing."""

    return pd.DataFrame(
        {
            "schedulingBlockId": ["SB001", "SB002", "SB003"],
            "scheduled_flag": [True, True, False],
            "scheduled_start_dt": [
                pd.Timestamp("2025-01-01 10:00:00", tz="UTC"),
                pd.Timestamp("2025-01-01 14:00:00", tz="UTC"),
                pd.NaT,
            ],
            "scheduled_stop_dt": [
                pd.Timestamp("2025-01-01 12:00:00", tz="UTC"),
                pd.Timestamp("2025-01-01 16:00:00", tz="UTC"),
                pd.NaT,
            ],
        }
    )


def test_build_calendar_heatmap__with_valid_schedule__returns_figure_and_bins(
    streamlit_mock: None, sample_schedule_df: pd.DataFrame
) -> None:
    """The heatmap builder should return a figure and non-empty bins."""

    # Given: a schedule dataframe with two scheduled blocks

    # When: constructing the calendar heatmap
    fig, bins_df = build_calendar_heatmap(sample_schedule_df, x_unit="hours", y_unit="days")

    # Then: a figure is returned and bins dataframe contains occupancy information
    assert fig is not None
    assert not bins_df.empty
    assert {"occupancy", "x_start", "y_start", "conflict"}.issubset(bins_df.columns)


def test_build_calendar_heatmap__computes_occupancy_in_range(
    streamlit_mock: None, sample_schedule_df: pd.DataFrame
) -> None:
    """Occupancy should stay within 0 and 1 when restricting the range."""

    # Given / When: building the heatmap within a single-day window
    _, bins_df = build_calendar_heatmap(
        sample_schedule_df,
        x_unit="hours",
        y_unit="days",
        range_start=pd.Timestamp("2025-01-01", tz="UTC"),
        range_end=pd.Timestamp("2025-01-02", tz="UTC"),
    )

    # Then: occupancy values should be normalized between zero and one
    assert (bins_df["occupancy"] >= 0).all()
    assert (bins_df["occupancy"] <= 1).all()
    assert (bins_df["occupancy"] > 0).any()


def test_build_calendar_heatmap__with_empty_dataset__returns_empty_bins(
    streamlit_mock: None,
) -> None:
    """Even with an empty schedule the helper should succeed."""

    # Given: an empty dataframe
    empty_df = pd.DataFrame(
        {
            "schedulingBlockId": [],
            "scheduled_flag": [],
            "scheduled_start_dt": [],
            "scheduled_stop_dt": [],
        }
    )

    # When: building the heatmap
    fig, bins_df = build_calendar_heatmap(empty_df, x_unit="days", y_unit="weeks")

    # Then: still returns a figure and dataframe (possibly empty)
    assert fig is not None
    assert isinstance(bins_df, pd.DataFrame)


def test_build_calendar_heatmap__with_pending_duration__highlights_gaps(
    streamlit_mock: None, sample_schedule_df: pd.DataFrame
) -> None:
    """Pending duration should annotate available slots without errors."""

    # Given: a pending duration threshold
    pending_duration = pd.Timedelta(hours=1)

    # When: building the heatmap with pending duration
    fig, bins_df = build_calendar_heatmap(
        sample_schedule_df,
        x_unit="hours",
        y_unit="days",
        range_start=pd.Timestamp("2025-01-01", tz="UTC"),
        range_end=pd.Timestamp("2025-01-02", tz="UTC"),
        pending_duration=pending_duration,
    )

    # Then: highlights should be produced without errors
    assert fig is not None
    assert isinstance(bins_df, pd.DataFrame)


def test_build_calendar_heatmap__captures_overlaps(
    streamlit_mock: None, sample_schedule_df: pd.DataFrame
) -> None:
    """Overlapping bins should be reported in the overlaps column."""

    # Given / When: building the heatmap
    _, bins_df = build_calendar_heatmap(
        sample_schedule_df,
        x_unit="hours",
        y_unit="days",
        range_start=pd.Timestamp("2025-01-01", tz="UTC"),
        range_end=pd.Timestamp("2025-01-02", tz="UTC"),
    )

    # Then: overlaps column should exist and contain at least one populated list
    assert "overlaps" in bins_df.columns
    assert bins_df["overlaps"].apply(lambda overlaps: isinstance(overlaps, list)).all()
    assert bins_df["overlaps"].apply(len).any()
